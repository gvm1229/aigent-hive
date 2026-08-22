use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::{ensure_no_symlink_ancestors, sha256_digest};
use hive_wiki::scan::{
    build_inventory, diff_inventory, ScanDecision, ScanDelta, ScanEntry, ScanFileKind,
    ScanInputFile, ScanInventory, ScanLimits, ScanOptions, ScanRootKind, SCAN_SCHEMA_VERSION,
};
use hive_wiki::WikiError;
use process_wrap::std::{ChildWrapper, CommandWrap};
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use process_wrap::std::ProcessGroup;

#[cfg(windows)]
use process_wrap::std::JobObject;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const MAX_GIT_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
const MAX_GIT_VERSION_OUTPUT_BYTES: usize = 4 * 1024;
const GIT_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const GIT_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(10);
const GIT_READER_DRAIN_TIMEOUT: Duration = Duration::from_millis(500);
const PROCESS_TREE_TERMINATION_TIMEOUT: Duration = Duration::from_millis(500);
const GIT_SECURITY_CONFIG_OVERRIDES: &[&str] = &[
    "core.fsmonitor=false",
    "core.untrackedCache=false",
    "core.hooksPath=/dev/null",
    "credential.helper=",
];
const MAX_NON_GIT_DIRECTORIES: usize = 10_000;
const MAX_NON_GIT_DEPTH: usize = 64;

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub(crate) struct DirectoryScanOutcome {
    pub canonical_target: String,
    pub inventory: ScanInventory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<ScanDelta>,
    pub target_mutated: bool,
}

#[derive(Debug)]
struct GitOutput {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

trait GitRunner {
    fn run(&self, arguments: &[OsString]) -> Result<GitOutput, WikiError>;
}

struct SystemGitRunner {
    executable: Option<PathBuf>,
    target: PathBuf,
}

impl SystemGitRunner {
    fn resolve(target: &Path) -> Result<Self, WikiError> {
        let current_directory = env::current_dir()
            .and_then(fs::canonicalize)
            .map_err(|error| WikiError::Io(format!("cannot resolve current directory: {error}")))?;
        let executable = resolve_git_executable_from_path(
            target,
            &current_directory,
            env::var_os("PATH").as_deref(),
        );
        if let Some(path) = executable.as_deref() {
            qualify_git_executable(path)?;
        }
        Ok(Self {
            executable,
            target: target.to_path_buf(),
        })
    }
}

impl GitRunner for SystemGitRunner {
    fn run(&self, arguments: &[OsString]) -> Result<GitOutput, WikiError> {
        validate_git_arguments(arguments, &self.target)?;
        let executable = self.executable.as_deref().ok_or_else(|| {
            WikiError::Io("Git executable is unavailable outside the scan target".to_owned())
        })?;
        let hardened_arguments = hardened_git_arguments(arguments);
        run_bounded_process(
            executable,
            &hardened_arguments,
            &fixed_git_environment(),
            GIT_COMMAND_TIMEOUT,
            MAX_GIT_OUTPUT_BYTES,
        )
    }
}

fn resolve_git_executable_from_path(
    target: &Path,
    current_directory: &Path,
    path_environment: Option<&OsStr>,
) -> Option<PathBuf> {
    let path_environment = path_environment?;
    for directory in env::split_paths(path_environment) {
        if directory.as_os_str().is_empty() || !directory.is_absolute() {
            continue;
        }
        let Ok(canonical_directory) = fs::canonicalize(&directory) else {
            continue;
        };
        if canonical_directory.starts_with(target)
            || canonical_directory.starts_with(current_directory)
        {
            continue;
        }
        for name in git_executable_names() {
            let candidate = canonical_directory.join(name);
            let Ok(canonical_candidate) = fs::canonicalize(&candidate) else {
                continue;
            };
            if canonical_candidate.starts_with(target)
                || canonical_candidate.starts_with(current_directory)
                || !is_executable_file(&canonical_candidate)
            {
                continue;
            }
            return Some(canonical_candidate);
        }
    }
    None
}

#[cfg(windows)]
fn git_executable_names() -> &'static [&'static str] {
    &["git.exe"]
}

#[cfg(not(windows))]
fn git_executable_names() -> &'static [&'static str] {
    &["git"]
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

fn qualify_git_executable(executable: &Path) -> Result<(), WikiError> {
    let arguments = hardened_git_arguments(&[OsString::from("--version")]);
    let output = run_bounded_process(
        executable,
        &arguments,
        &fixed_git_environment(),
        GIT_COMMAND_TIMEOUT,
        MAX_GIT_VERSION_OUTPUT_BYTES,
    )?;
    let version = std::str::from_utf8(&output.stdout)
        .map_err(|_| WikiError::Verification("Git version output is not UTF-8".to_owned()))?;
    if !output.success || !version.trim().starts_with("git version ") {
        return Err(WikiError::Verification(
            "resolved Git executable failed identity qualification".to_owned(),
        ));
    }
    Ok(())
}

fn hardened_git_arguments(arguments: &[OsString]) -> Vec<OsString> {
    let scan_target = arguments
        .get(1)
        .filter(|_| arguments.first().is_some_and(|argument| argument == "-C"));
    let insertion_index = usize::from(scan_target.is_some()) * 2;
    let mut hardened = Vec::with_capacity(
        arguments.len()
            + GIT_SECURITY_CONFIG_OVERRIDES.len().saturating_mul(2)
            + usize::from(scan_target.is_some()).saturating_mul(2),
    );
    hardened.extend_from_slice(&arguments[..insertion_index]);
    for override_value in GIT_SECURITY_CONFIG_OVERRIDES {
        hardened.push(OsString::from("-c"));
        hardened.push(OsString::from(override_value));
    }
    if let Some(target) = scan_target {
        hardened.push(OsString::from("-c"));
        hardened.push(OsString::from(format!(
            "safe.directory={}",
            target.to_string_lossy().replace('\\', "/")
        )));
    }
    hardened.extend_from_slice(&arguments[insertion_index..]);
    hardened
}

fn validate_git_arguments(arguments: &[OsString], target: &Path) -> Result<(), WikiError> {
    let expected_target = target.as_os_str();
    let valid = matches!(
        arguments,
        [flag, supplied_target, command, show_toplevel]
            if flag == "-C"
                && supplied_target == expected_target
                && command == "rev-parse"
                && show_toplevel == "--show-toplevel"
    ) || matches!(
        arguments,
        [flag, supplied_target, command, nul, cached, separator]
            if flag == "-C"
                && supplied_target == expected_target
                && command == "ls-files"
                && nul == "-z"
                && cached == "--cached"
                && separator == "--"
    ) || matches!(
        arguments,
        [flag, supplied_target, command, nul, others, exclude_standard, separator]
            if flag == "-C"
                && supplied_target == expected_target
                && command == "ls-files"
                && nul == "-z"
                && others == "--others"
                && exclude_standard == "--exclude-standard"
                && separator == "--"
    );
    if !valid {
        return Err(WikiError::InvalidInput(
            "Git inventory command arguments are outside the fixed read-only allowlist".to_owned(),
        ));
    }
    Ok(())
}

fn fixed_git_environment() -> Vec<(OsString, OsString)> {
    let environment = vec![
        (OsString::from("GIT_ATTR_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_COUNT"), OsString::from("0")),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_OPTIONAL_LOCKS"), OsString::from("0")),
        (OsString::from("GIT_PAGER"), OsString::from("cat")),
        (OsString::from("GIT_TERMINAL_PROMPT"), OsString::from("0")),
        (OsString::from("LANG"), OsString::from("C")),
        (OsString::from("LC_ALL"), OsString::from("C")),
        (OsString::from("NO_COLOR"), OsString::from("1")),
    ];
    #[cfg(windows)]
    let environment = {
        let mut environment = environment;
        if let Some(system_root) = env::var_os("SystemRoot") {
            environment.push((OsString::from("SystemRoot"), system_root));
        }
        environment
    };
    environment
}

fn run_bounded_process(
    executable: &Path,
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
    timeout: Duration,
    output_limit: usize,
) -> Result<GitOutput, WikiError> {
    let mut command = Command::new(executable);
    command
        .args(arguments)
        .env_clear()
        .envs(environment.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut command = contained_command(command);
    let mut child = command
        .spawn()
        .map_err(|error| WikiError::Io(format!("cannot run Git inventory command: {error}")))?;
    let stdout = child
        .stdout()
        .take()
        .ok_or_else(|| WikiError::Io("cannot capture Git inventory stdout".to_owned()))?;
    let stderr = child
        .stderr()
        .take()
        .ok_or_else(|| WikiError::Io("cannot capture Git inventory stderr".to_owned()))?;
    let output_exceeded = Arc::new(AtomicBool::new(false));
    let stdout_exceeded = Arc::clone(&output_exceeded);
    let stderr_exceeded = Arc::clone(&output_exceeded);
    let stdout_reader = spawn_bounded_reader(stdout, output_limit, stdout_exceeded);
    let stderr_reader = spawn_bounded_reader(stderr, output_limit, stderr_exceeded);

    let deadline = Instant::now() + timeout;
    let stop = loop {
        if output_exceeded.load(Ordering::Acquire) {
            break ProcessStop::OutputExceeded;
        }
        match child.try_wait() {
            Ok(Some(status)) => break ProcessStop::Completed(status),
            Ok(None) if Instant::now() >= deadline => break ProcessStop::TimedOut,
            Ok(None) => thread::sleep(GIT_WAIT_POLL_INTERVAL),
            Err(error) => break ProcessStop::WaitFailed(error.to_string()),
        }
    };
    if !matches!(stop, ProcessStop::Completed(_)) {
        terminate_process_tree(child.as_mut());
    }
    let reader_deadline = Instant::now() + GIT_READER_DRAIN_TIMEOUT;
    let mut stdout = collect_bounded_reader(&stdout_reader, "stdout", reader_deadline);
    let mut stderr = collect_bounded_reader(&stderr_reader, "stderr", reader_deadline);
    if matches!(stop, ProcessStop::Completed(_)) && (stdout.is_err() || stderr.is_err()) {
        terminate_process_tree(child.as_mut());
        let retry_deadline = Instant::now() + GIT_READER_DRAIN_TIMEOUT;
        if stdout.is_err() {
            stdout = collect_bounded_reader(&stdout_reader, "stdout", retry_deadline);
        }
        if stderr.is_err() {
            stderr = collect_bounded_reader(&stderr_reader, "stderr", retry_deadline);
        }
    }
    if output_exceeded.load(Ordering::Acquire) {
        return Err(WikiError::InvalidInput(
            "Git inventory output exceeds the bounded command budget".to_owned(),
        ));
    }
    match stop {
        ProcessStop::Completed(status) => Ok(GitOutput {
            success: status.success(),
            stdout: stdout?,
            stderr: stderr?,
        }),
        ProcessStop::TimedOut => Err(WikiError::Io(
            "Git inventory command exceeded its timeout and was terminated".to_owned(),
        )),
        ProcessStop::OutputExceeded => Err(WikiError::InvalidInput(
            "Git inventory output exceeds the bounded command budget".to_owned(),
        )),
        ProcessStop::WaitFailed(error) => Err(WikiError::Io(format!(
            "cannot wait for Git inventory command: {error}"
        ))),
    }
}

fn contained_command(command: Command) -> CommandWrap {
    // A direct Git child may exit while a malicious descendant keeps its stdout/stderr handles
    // open. OS-owned containment lets the bounded cleanup terminate that entire inherited tree.
    let mut command = CommandWrap::from(command);
    #[cfg(unix)]
    {
        command.wrap(ProcessGroup::leader());
    }
    #[cfg(windows)]
    {
        command.wrap(JobObject);
    }
    command
}

fn terminate_process_tree(child: &mut dyn ChildWrapper) {
    let _ = child.start_kill();
    reap_child_until(child, Instant::now() + PROCESS_TREE_TERMINATION_TIMEOUT);
}

fn reap_child_until(child: &mut dyn ChildWrapper, deadline: Instant) {
    loop {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => return,
            Ok(None) if Instant::now() >= deadline => return,
            Ok(None) => thread::sleep(GIT_WAIT_POLL_INTERVAL),
        }
    }
}

enum ProcessStop {
    Completed(ExitStatus),
    TimedOut,
    OutputExceeded,
    WaitFailed(String),
}

fn read_bounded_stream(
    mut stream: impl Read,
    output_limit: usize,
    output_exceeded: &AtomicBool,
) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(output_limit.min(8 * 1024));
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Ok(output);
        }
        let remaining = output_limit.saturating_sub(output.len());
        if read > remaining {
            output.extend_from_slice(&buffer[..remaining]);
            output_exceeded.store(true, Ordering::Release);
            return Ok(output);
        }
        output.extend_from_slice(&buffer[..read]);
    }
}

fn spawn_bounded_reader(
    stream: impl Read + Send + 'static,
    output_limit: usize,
    output_exceeded: Arc<AtomicBool>,
) -> Receiver<std::io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let result = read_bounded_stream(stream, output_limit, output_exceeded.as_ref());
        let _ = sender.send(result);
    });
    receiver
}

fn collect_bounded_reader(
    reader: &Receiver<std::io::Result<Vec<u8>>>,
    stream_name: &str,
    deadline: Instant,
) -> Result<Vec<u8>, WikiError> {
    reader
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|error| match error {
            RecvTimeoutError::Timeout => WikiError::Io(format!(
                "Git {stream_name} reader exceeded its bounded cleanup deadline"
            )),
            RecvTimeoutError::Disconnected => {
                WikiError::Io(format!("Git {stream_name} reader terminated unexpectedly"))
            }
        })?
        .map_err(|error| WikiError::Io(format!("cannot read Git {stream_name}: {error}")))
}

pub(crate) fn scan_directory(
    target: &Path,
    include_untracked: bool,
    prior: Option<&ScanInventory>,
) -> Result<DirectoryScanOutcome, WikiError> {
    let canonical_target = canonical_scan_target(target)?;
    let runner = SystemGitRunner::resolve(&canonical_target)?;
    scan_canonical_directory_with_runner(&canonical_target, include_untracked, prior, &runner)
}

#[cfg(test)]
fn scan_directory_with_runner(
    target: &Path,
    include_untracked: bool,
    prior: Option<&ScanInventory>,
    runner: &dyn GitRunner,
) -> Result<DirectoryScanOutcome, WikiError> {
    let canonical_target = canonical_scan_target(target)?;
    scan_canonical_directory_with_runner(&canonical_target, include_untracked, prior, runner)
}

fn canonical_scan_target(target: &Path) -> Result<PathBuf, WikiError> {
    let canonical_target = fs::canonicalize(target)
        .map_err(|error| WikiError::Io(format!("cannot resolve scan target: {error}")))?;
    let metadata = fs::symlink_metadata(&canonical_target)
        .map_err(|error| WikiError::Io(format!("cannot inspect scan target: {error}")))?;
    if !metadata.is_dir() {
        return Err(WikiError::InvalidInput(
            "scan target must be a directory".to_owned(),
        ));
    }
    Ok(normalize_windows_canonical_path(canonical_target))
}

#[cfg(windows)]
fn normalize_windows_canonical_path(path: PathBuf) -> PathBuf {
    let normalized = path.to_str().and_then(|text| {
        text.strip_prefix(r"\\?\UNC\")
            .map(|unc| PathBuf::from(format!(r"\\{unc}")))
            .or_else(|| text.strip_prefix(r"\\?\").map(PathBuf::from))
    });
    normalized.unwrap_or(path)
}

#[cfg(not(windows))]
fn normalize_windows_canonical_path(path: PathBuf) -> PathBuf {
    path
}

fn scan_canonical_directory_with_runner(
    canonical_target: &Path,
    include_untracked: bool,
    prior: Option<&ScanInventory>,
    runner: &dyn GitRunner,
) -> Result<DirectoryScanOutcome, WikiError> {
    let limits = ScanLimits::default();
    let git_paths = discover_git_paths(
        canonical_target,
        include_untracked,
        limits.max_discovered_files,
        runner,
    )?;
    let (root_kind, entries) = if let Some(paths) = git_paths {
        let options = ScanOptions {
            root_kind: ScanRootKind::Git,
            include_untracked,
            limits,
        };
        (
            ScanRootKind::Git,
            read_discovered_files(canonical_target, paths, options)?,
        )
    } else {
        let options = ScanOptions {
            root_kind: ScanRootKind::NonGit,
            include_untracked: true,
            limits,
        };
        (
            ScanRootKind::NonGit,
            discover_non_git_files(canonical_target, options)?,
        )
    };
    let options = ScanOptions {
        root_kind,
        include_untracked: root_kind == ScanRootKind::NonGit || include_untracked,
        limits,
    };
    let inventory = finalize_inventory(
        entries,
        options,
        prior.map(|inventory| inventory.inventory_digest.as_str()),
    )?;
    let delta = prior.map(|previous| diff_inventory(previous, &inventory));
    Ok(DirectoryScanOutcome {
        canonical_target: canonical_target.to_string_lossy().into_owned(),
        inventory,
        delta,
        target_mutated: false,
    })
}

fn discover_git_paths(
    target: &Path,
    include_untracked: bool,
    max_discovered_paths: usize,
    runner: &dyn GitRunner,
) -> Result<Option<Vec<(String, bool)>>, WikiError> {
    let prefix = [OsString::from("-C"), target.as_os_str().to_os_string()];
    let mut root_arguments = prefix.to_vec();
    root_arguments.extend([
        OsString::from("rev-parse"),
        OsString::from("--show-toplevel"),
    ]);
    let root = match runner.run(&root_arguments) {
        Ok(output) if output.success => output,
        Ok(output) if output.stderr.is_empty() || is_not_git_repository(&output.stderr) => {
            return Ok(None)
        }
        Ok(output) => return Err(git_failure("repository-root discovery", &output.stderr)),
        Err(WikiError::Io(message))
            if message == "Git executable is unavailable outside the scan target" =>
        {
            return Ok(None)
        }
        Err(error) => return Err(error),
    };
    let root_text = std::str::from_utf8(&root.stdout)
        .map_err(|_| WikiError::Verification("Git root output is not UTF-8".to_owned()))?
        .trim();
    let git_root = canonical_scan_target(Path::new(root_text))?;
    if !target.starts_with(&git_root) {
        return Err(WikiError::Verification(
            "Git repository root does not contain the scan target".to_owned(),
        ));
    }

    let mut tracked_arguments = prefix.to_vec();
    tracked_arguments.extend([
        OsString::from("ls-files"),
        OsString::from("-z"),
        OsString::from("--cached"),
        OsString::from("--"),
    ]);
    let tracked = runner.run(&tracked_arguments)?;
    if !tracked.success {
        return Err(git_failure("tracked file inventory", &tracked.stderr));
    }
    let mut paths = parse_nul_paths(&tracked.stdout, max_discovered_paths)?
        .into_iter()
        .map(|path| (path, true))
        .collect::<BTreeMap<_, _>>();
    if include_untracked {
        let mut untracked_arguments = prefix.to_vec();
        untracked_arguments.extend([
            OsString::from("ls-files"),
            OsString::from("-z"),
            OsString::from("--others"),
            OsString::from("--exclude-standard"),
            OsString::from("--"),
        ]);
        let untracked = runner.run(&untracked_arguments)?;
        if !untracked.success {
            return Err(git_failure("untracked file inventory", &untracked.stderr));
        }
        for path in parse_nul_paths(&untracked.stdout, max_discovered_paths)? {
            paths.entry(path).or_insert(false);
            if paths.len() > max_discovered_paths {
                return Err(WikiError::InvalidInput(format!(
                    "Git scan exceeds the {max_discovered_paths} discovered path budget"
                )));
            }
        }
    }
    Ok(Some(paths.into_iter().collect()))
}

fn is_not_git_repository(stderr: &[u8]) -> bool {
    let detail = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    detail.contains("not a git repository")
        || detail.contains("not a git work tree")
        || detail.contains("must be run in a work tree")
}

fn git_failure(action: &str, stderr: &[u8]) -> WikiError {
    let detail = String::from_utf8_lossy(stderr);
    let bounded = detail.trim().chars().take(240).collect::<String>();
    WikiError::Io(if bounded.is_empty() {
        format!("Git {action} failed")
    } else {
        format!("Git {action} failed: {bounded}")
    })
}

fn parse_nul_paths(bytes: &[u8], max_discovered_paths: usize) -> Result<Vec<String>, WikiError> {
    if bytes.len() > MAX_GIT_OUTPUT_BYTES {
        return Err(WikiError::InvalidInput(
            "Git file inventory exceeds the bounded output budget".to_owned(),
        ));
    }
    let mut paths = Vec::new();
    for raw in bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        if paths.len() >= max_discovered_paths {
            return Err(WikiError::InvalidInput(format!(
                "Git scan exceeds the {max_discovered_paths} discovered path budget"
            )));
        }
        let path = std::str::from_utf8(raw)
            .map_err(|_| WikiError::Verification("Git path is not UTF-8".to_owned()))?;
        validate_scan_relative(path)?;
        if path.contains('\\') || path.contains(':') {
            return Err(WikiError::InvalidInput(
                "Git inventory path is not slash-normalized".to_owned(),
            ));
        }
        paths.push(path.to_owned());
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn read_discovered_files(
    target: &Path,
    paths: Vec<(String, bool)>,
    options: ScanOptions,
) -> Result<Vec<ScanEntry>, WikiError> {
    let mut observe_read = ignore_content_read as fn(&str);
    read_discovered_files_with_observer(target, paths, options, &mut observe_read)
}

fn ignore_content_read(_: &str) {}

fn read_discovered_files_with_observer(
    target: &Path,
    mut paths: Vec<(String, bool)>,
    options: ScanOptions,
    observe_read: &mut dyn FnMut(&str),
) -> Result<Vec<ScanEntry>, WikiError> {
    let limits = options.limits;
    if paths.len() > limits.max_discovered_files {
        return Err(WikiError::InvalidInput(format!(
            "scan discovered {} files, exceeding the {} file budget",
            paths.len(),
            limits.max_discovered_files
        )));
    }
    paths.sort_by(|(left_path, left_tracked), (right_path, right_tracked)| {
        right_tracked
            .cmp(left_tracked)
            .then_with(|| left_path.cmp(right_path))
    });
    if paths.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(WikiError::InvalidInput(
            "scan discovery contains a duplicate path".to_owned(),
        ));
    }
    let root = Dir::open_ambient_dir(target, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open scan target: {error}")))?;
    let mut entries = Vec::with_capacity(paths.len());
    let mut included_count = 0_usize;
    let mut included_bytes = 0_usize;
    for (path, tracked) in paths {
        let entry = classify_one_discovered_file(
            &root,
            target,
            &path,
            tracked,
            options,
            included_count,
            included_bytes,
            observe_read,
        )?;
        if entry.decision == ScanDecision::Included {
            included_count = included_count.saturating_add(1);
            included_bytes = included_bytes.saturating_add(entry.byte_len);
        }
        entries.push(entry);
    }
    Ok(entries)
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn classify_one_discovered_file(
    root: &Dir,
    target: &Path,
    relative: &str,
    tracked: bool,
    options: ScanOptions,
    included_count: usize,
    included_bytes: usize,
    observe_read: &mut dyn FnMut(&str),
) -> Result<ScanEntry, WikiError> {
    let limits = options.limits;
    validate_scan_relative(relative)?;
    if is_foreign_host_namespace(relative) {
        let metadata = root.symlink_metadata(relative).map_err(|error| {
            WikiError::Io(format!("cannot inspect scan path {relative}: {error}"))
        })?;
        let byte_len = usize::try_from(metadata.len()).map_err(|_| {
            WikiError::InvalidInput(format!("scan path byte length is unsupported: {relative}"))
        })?;
        return Ok(ScanEntry {
            relative_path: relative.to_owned(),
            content_digest: None,
            byte_len,
            tracked,
            decision: ScanDecision::Skipped,
            reason: "foreign-host-namespace".to_owned(),
        });
    }
    let relative_path = Path::new(relative);
    if let Some(parent) = relative_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        ensure_no_symlink_ancestors(target, parent)
            .map_err(|error| WikiError::Conflict(error.to_string()))?;
    }
    let metadata = root
        .symlink_metadata(relative)
        .map_err(|error| WikiError::Io(format!("cannot inspect scan path {relative}: {error}")))?;
    let file_type = metadata.file_type();
    let file_kind = if file_type.is_symlink() {
        ScanFileKind::Symlink
    } else if file_type.is_file() {
        ScanFileKind::Regular
    } else {
        ScanFileKind::Special
    };
    let observed_byte_len = usize::try_from(metadata.len()).map_err(|_| {
        WikiError::InvalidInput(format!("scan path byte length is unsupported: {relative}"))
    })?;
    let metadata_only = classify_single_input(
        relative,
        &[],
        observed_byte_len,
        tracked,
        file_kind,
        options,
    );
    match metadata_only {
        Ok(entry) => Ok(apply_global_scan_budget(
            entry,
            included_count,
            included_bytes,
            limits,
        )),
        Err(WikiError::Verification(message))
            if message
                == format!("scan file bytes differ from no-follow metadata length: {relative}") =>
        {
            if included_count >= limits.max_included_files {
                return Ok(skipped_budget_entry(
                    relative,
                    observed_byte_len,
                    tracked,
                    "file-count-budget",
                ));
            }
            if included_bytes.saturating_add(observed_byte_len) > limits.max_total_bytes {
                return Ok(skipped_budget_entry(
                    relative,
                    observed_byte_len,
                    tracked,
                    "total-byte-budget",
                ));
            }
            observe_read(relative);
            let mut open_options = CapOpenOptions::new();
            open_options.read(true).follow(FollowSymlinks::No);
            let file = root.open_with(relative, &open_options).map_err(|error| {
                WikiError::Io(format!("cannot read scan path {relative}: {error}"))
            })?;
            let mut bytes = Vec::with_capacity(observed_byte_len);
            file.take(u64::try_from(limits.max_file_bytes).unwrap_or(u64::MAX) + 1)
                .read_to_end(&mut bytes)
                .map_err(|error| {
                    WikiError::Io(format!("cannot read scan path {relative}: {error}"))
                })?;
            let entry = classify_single_input(
                relative,
                &bytes,
                observed_byte_len,
                tracked,
                file_kind,
                options,
            )?;
            Ok(apply_global_scan_budget(
                entry,
                included_count,
                included_bytes,
                limits,
            ))
        }
        Err(error) => Err(error),
    }
}

fn validate_scan_relative(path: &str) -> Result<(), WikiError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with("//")
        || path.contains('\\')
        || path.contains(':')
        || path.chars().any(char::is_control)
    {
        return Err(WikiError::InvalidInput(
            "scan paths must be normalized project-relative UTF-8 paths".to_owned(),
        ));
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(WikiError::InvalidInput(format!(
            "scan path contains a non-normalized segment: {path}"
        )));
    }
    Ok(())
}

fn is_foreign_host_namespace(path: &str) -> bool {
    path.split('/').any(|part| {
        matches!(
            part.to_ascii_lowercase().as_str(),
            ".agents" | ".claude" | ".codex" | ".omc" | ".omx"
        )
    })
}

fn classify_single_input(
    relative_path: &str,
    bytes: &[u8],
    observed_byte_len: usize,
    tracked: bool,
    file_kind: ScanFileKind,
    options: ScanOptions,
) -> Result<ScanEntry, WikiError> {
    let input = [ScanInputFile {
        relative_path,
        bytes,
        observed_byte_len,
        tracked,
        file_kind,
    }];
    build_inventory(&input, options, None)?
        .entries
        .into_iter()
        .next()
        .ok_or_else(|| WikiError::Verification("scan classifier returned no entry".to_owned()))
}

fn apply_global_scan_budget(
    mut entry: ScanEntry,
    included_count: usize,
    included_bytes: usize,
    limits: ScanLimits,
) -> ScanEntry {
    if entry.decision == ScanDecision::Skipped {
        return entry;
    }
    if included_count >= limits.max_included_files {
        entry.decision = ScanDecision::Skipped;
        entry.content_digest = None;
        "file-count-budget".clone_into(&mut entry.reason);
    } else if included_bytes.saturating_add(entry.byte_len) > limits.max_total_bytes {
        entry.decision = ScanDecision::Skipped;
        entry.content_digest = None;
        "total-byte-budget".clone_into(&mut entry.reason);
    }
    entry
}

fn skipped_budget_entry(
    relative_path: &str,
    observed_byte_len: usize,
    tracked: bool,
    reason: &str,
) -> ScanEntry {
    ScanEntry {
        relative_path: relative_path.to_owned(),
        content_digest: None,
        byte_len: observed_byte_len,
        tracked,
        decision: ScanDecision::Skipped,
        reason: reason.to_owned(),
    }
}

fn finalize_inventory(
    entries: Vec<ScanEntry>,
    options: ScanOptions,
    prior_inventory_digest: Option<&str>,
) -> Result<ScanInventory, WikiError> {
    let included_count = entries
        .iter()
        .filter(|entry| entry.decision == ScanDecision::Included)
        .count();
    let included_bytes = entries
        .iter()
        .filter(|entry| entry.decision == ScanDecision::Included)
        .fold(0_usize, |total, entry| total.saturating_add(entry.byte_len));
    let logical = serde_json::to_vec(&(
        SCAN_SCHEMA_VERSION,
        options.root_kind,
        options.include_untracked,
        &entries,
    ))
    .map_err(|error| WikiError::Io(format!("cannot encode scan inventory: {error}")))?;
    let inventory_digest = sha256_digest(&logical);
    Ok(ScanInventory {
        schema_version: SCAN_SCHEMA_VERSION,
        root_kind: options.root_kind,
        include_untracked: options.include_untracked,
        included_count,
        skipped_count: entries.len().saturating_sub(included_count),
        included_bytes,
        entries,
        unchanged: prior_inventory_digest == Some(inventory_digest.as_str()),
        inventory_digest,
    })
}

fn discover_non_git_files(
    target: &Path,
    options: ScanOptions,
) -> Result<Vec<ScanEntry>, WikiError> {
    discover_non_git_files_with_bounds(target, options, MAX_NON_GIT_DIRECTORIES, MAX_NON_GIT_DEPTH)
}

fn discover_non_git_files_with_bounds(
    target: &Path,
    options: ScanOptions,
    max_directories: usize,
    max_depth: usize,
) -> Result<Vec<ScanEntry>, WikiError> {
    if max_directories == 0 || max_depth == 0 {
        return Err(WikiError::InvalidInput(
            "non-Git traversal limits must be positive".to_owned(),
        ));
    }
    let limits = options.limits;
    let root = Dir::open_ambient_dir(target, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open scan target: {error}")))?;
    let mut pending = vec![PathBuf::new()];
    let mut paths = Vec::new();
    let mut visited_directories = 1_usize;
    let max_enumerated_entries = max_directories.saturating_add(limits.max_discovered_files);
    let mut enumerated_entries = 0_usize;
    while let Some(directory) = pending.pop() {
        let read_path = if directory.as_os_str().is_empty() {
            Path::new(".")
        } else {
            directory.as_path()
        };
        let entries = root
            .read_dir(read_path)
            .map_err(|error| WikiError::Io(format!("cannot enumerate scan target: {error}")))?;
        let mut children = Vec::new();
        for entry in entries {
            enumerated_entries = enumerated_entries.saturating_add(1);
            if enumerated_entries > max_enumerated_entries {
                return Err(WikiError::InvalidInput(format!(
                    "non-Git scan exceeds the {max_enumerated_entries} enumerated entry budget"
                )));
            }
            children.push(
                entry.map_err(|error| {
                    WikiError::Io(format!("cannot enumerate scan entry: {error}"))
                })?,
            );
        }
        children.sort_by_key(cap_std::fs::DirEntry::file_name);
        for entry in children {
            let relative = directory.join(entry.file_name());
            let portable = relative.to_string_lossy().replace('\\', "/");
            let file_type = entry
                .file_type()
                .map_err(|error| WikiError::Io(format!("cannot inspect {portable}: {error}")))?;
            if file_type.is_dir() && !file_type.is_symlink() {
                if !prune_non_git_directory(&portable) {
                    let depth = relative.components().count();
                    if depth > max_depth {
                        return Err(WikiError::InvalidInput(format!(
                            "non-Git scan exceeds the {max_depth} directory depth budget"
                        )));
                    }
                    visited_directories = visited_directories.saturating_add(1);
                    if visited_directories > max_directories {
                        return Err(WikiError::InvalidInput(format!(
                            "non-Git scan exceeds the {max_directories} visited directory budget"
                        )));
                    }
                    pending.push(relative);
                }
                continue;
            }
            paths.push((portable, false));
            if paths.len() > limits.max_discovered_files {
                return Err(WikiError::InvalidInput(format!(
                    "non-Git scan exceeds the {} discovered file budget",
                    limits.max_discovered_files
                )));
            }
        }
    }
    paths.sort_by(|left, right| left.0.cmp(&right.0));
    read_discovered_files(target, paths, options)
}

fn prune_non_git_directory(path: &str) -> bool {
    path.split('/').any(|component| {
        matches!(
            component.to_ascii_lowercase().as_str(),
            ".git"
                | ".hive"
                | ".next"
                | ".venv"
                | "__pycache__"
                | "build"
                | "coverage"
                | "dist"
                | "node_modules"
                | "out"
                | "target"
                | "vendor"
                | "venv"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::Write;

    struct FakeGitRunner {
        target: PathBuf,
        tracked: Vec<u8>,
        untracked: Vec<u8>,
        is_git: bool,
        calls: RefCell<Vec<Vec<OsString>>>,
    }

    impl GitRunner for FakeGitRunner {
        fn run(&self, arguments: &[OsString]) -> Result<GitOutput, WikiError> {
            self.calls.borrow_mut().push(arguments.to_vec());
            let action = arguments
                .get(2)
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if action == "rev-parse" {
                return Ok(GitOutput {
                    success: self.is_git,
                    stdout: if self.is_git {
                        format!("{}\n", self.target.display()).into_bytes()
                    } else {
                        Vec::new()
                    },
                    stderr: Vec::new(),
                });
            }
            let untracked = arguments
                .iter()
                .any(|argument| argument == OsStr::new("--others"));
            Ok(GitOutput {
                success: true,
                stdout: if untracked {
                    self.untracked.clone()
                } else {
                    self.tracked.clone()
                },
                stderr: Vec::new(),
            })
        }
    }

    #[test]
    fn git_discovery_uses_fixed_arguments_and_never_mutates_target() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        fs::write(temporary.path().join("README.md"), b"tracked purpose\n").expect("readme");
        fs::write(temporary.path().join("notes.md"), b"untracked note\n").expect("notes");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let runner = FakeGitRunner {
            target: canonical.clone(),
            tracked: b"README.md\0".to_vec(),
            untracked: b"notes.md\0".to_vec(),
            is_git: true,
            calls: RefCell::new(Vec::new()),
        };
        let before = fs::read_dir(&canonical)
            .expect("before")
            .map(|entry| entry.expect("entry").file_name())
            .collect::<Vec<_>>();
        let outcome =
            scan_directory_with_runner(&canonical, true, None, &runner).expect("scan outcome");
        let after = fs::read_dir(&canonical)
            .expect("after")
            .map(|entry| entry.expect("entry").file_name())
            .collect::<Vec<_>>();
        assert_eq!(before, after);
        assert!(!outcome.target_mutated);
        assert_eq!(outcome.inventory.root_kind, ScanRootKind::Git);
        assert_eq!(outcome.inventory.included_count, 2);
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 3);
        assert!(calls
            .iter()
            .all(|arguments| arguments.first() == Some(&OsString::from("-C"))));
        assert!(calls[2]
            .iter()
            .any(|argument| argument == "--exclude-standard"));
    }

    #[test]
    fn non_git_discovery_uses_allowlist_and_prunes_vendor() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        fs::create_dir_all(temporary.path().join("src")).expect("src");
        fs::create_dir_all(temporary.path().join("vendor/dependency")).expect("vendor");
        fs::write(temporary.path().join("README.md"), b"purpose\n").expect("readme");
        fs::write(temporary.path().join("src/lib.rs"), b"code\n").expect("source");
        fs::write(
            temporary.path().join("vendor/dependency/README.md"),
            b"foreign\n",
        )
        .expect("vendor file");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let runner = FakeGitRunner {
            target: canonical.clone(),
            tracked: Vec::new(),
            untracked: Vec::new(),
            is_git: false,
            calls: RefCell::new(Vec::new()),
        };
        let outcome =
            scan_directory_with_runner(&canonical, false, None, &runner).expect("scan outcome");
        assert_eq!(outcome.inventory.root_kind, ScanRootKind::NonGit);
        assert_eq!(outcome.inventory.included_count, 1);
        assert!(outcome
            .inventory
            .entries
            .iter()
            .any(|entry| entry.relative_path == "src/lib.rs"
                && entry.reason == "non-git-not-allowlisted"));
        assert!(!outcome
            .inventory
            .entries
            .iter()
            .any(|entry| entry.relative_path.starts_with("vendor/")));
    }

    #[test]
    fn hostile_git_path_is_rejected_before_read() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let runner = FakeGitRunner {
            target: canonical.clone(),
            tracked: b"../outside.md\0".to_vec(),
            untracked: Vec::new(),
            is_git: true,
            calls: RefCell::new(Vec::new()),
        };
        assert!(matches!(
            scan_directory_with_runner(&canonical, false, None, &runner),
            Err(WikiError::InvalidInput(_))
        ));
    }

    #[test]
    fn foreign_host_namespace_is_receipted_without_content_read() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        fs::create_dir_all(temporary.path().join(".agents/directives")).expect("foreign namespace");
        fs::write(
            temporary.path().join(".agents/directives/foreign.md"),
            b"foreign instruction\n",
        )
        .expect("foreign fixture");
        fs::write(temporary.path().join("README.md"), b"project purpose\n")
            .expect("readme fixture");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let mut reads = Vec::new();
        let entries = read_discovered_files_with_observer(
            &canonical,
            vec![
                (".agents/directives/foreign.md".to_owned(), true),
                ("README.md".to_owned(), true),
            ],
            ScanOptions {
                root_kind: ScanRootKind::Git,
                include_untracked: false,
                limits: ScanLimits::default(),
            },
            &mut |path| reads.push(path.to_owned()),
        )
        .expect("scan foreign namespace safely");
        assert_eq!(reads, vec!["README.md"]);
        assert!(entries.iter().any(|entry| {
            entry.relative_path == ".agents/directives/foreign.md"
                && entry.decision == ScanDecision::Skipped
                && entry.reason == "foreign-host-namespace"
                && entry.content_digest.is_none()
        }));
        assert!(entries.iter().any(|entry| {
            entry.relative_path == "README.md" && entry.decision == ScanDecision::Included
        }));
    }

    #[test]
    fn git_path_parser_accepts_foreign_namespace_for_safe_skip() {
        assert_eq!(
            parse_nul_paths(b".agents/directives/foreign.md\0README.md\0", 2)
                .expect("foreign namespace must reach classification"),
            vec![".agents/directives/foreign.md", "README.md"]
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_scan_target_removes_extended_length_prefix() {
        assert_eq!(
            normalize_windows_canonical_path(PathBuf::from(r"\\?\C:\work\source")),
            PathBuf::from(r"C:\work\source")
        );
        assert_eq!(
            normalize_windows_canonical_path(PathBuf::from(r"\\?\UNC\server\share\source")),
            PathBuf::from(r"\\server\share\source")
        );
    }

    #[test]
    fn git_path_parser_rejects_count_budget_during_stream_classification() {
        let error = parse_nul_paths(b"a.md\0b.md\0c.md\0", 2)
            .expect_err("third path must exceed the parser budget");
        assert!(
            matches!(error, WikiError::InvalidInput(message) if message.contains("path budget"))
        );
    }

    #[test]
    fn git_root_non_repository_failure_is_the_only_non_git_fallback() {
        assert!(is_not_git_repository(b"fatal: not a git repository"));
        assert!(is_not_git_repository(
            b"fatal: this operation must be run in a work tree"
        ));
        assert!(!is_not_git_repository(
            b"fatal: unsafe repository ownership"
        ));
    }

    #[test]
    fn scan_target_git_candidate_is_never_resolved_or_invoked() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let candidate = canonical.join(git_executable_names()[0]);
        fs::write(&candidate, malicious_git_body()).expect("malicious Git candidate");
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&candidate).expect("metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&candidate, permissions).expect("executable permissions");
        }
        let joined = env::join_paths([canonical.clone()]).expect("PATH");
        let resolved =
            resolve_git_executable_from_path(&canonical, &canonical, Some(joined.as_os_str()));
        assert_eq!(resolved, None);

        let runner = SystemGitRunner {
            executable: resolved,
            target: canonical.clone(),
        };
        let arguments = vec![
            OsString::from("-C"),
            canonical.as_os_str().to_os_string(),
            OsString::from("rev-parse"),
            OsString::from("--show-toplevel"),
        ];
        assert!(matches!(runner.run(&arguments), Err(WikiError::Io(_))));
        assert!(!canonical.join("malicious-git-invoked").exists());
    }

    #[cfg(windows)]
    fn malicious_git_body() -> &'static [u8] {
        b"not a real Windows executable"
    }

    #[cfg(not(windows))]
    fn malicious_git_body() -> &'static [u8] {
        b"#!/bin/sh\ntouch malicious-git-invoked\n"
    }

    #[test]
    fn git_runner_rejects_arguments_outside_read_only_allowlist() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let arguments = vec![
            OsString::from("-C"),
            canonical.as_os_str().to_os_string(),
            OsString::from("status"),
        ];
        assert!(matches!(
            validate_git_arguments(&arguments, &canonical),
            Err(WikiError::InvalidInput(_))
        ));
    }

    #[test]
    fn system_git_runner_uses_qualified_absolute_executable() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let runner = SystemGitRunner::resolve(&target).expect("system Git resolution");
        let executable = runner
            .executable
            .as_deref()
            .expect("Git must be available for the source checkout");
        assert!(executable.is_absolute());
        assert!(!executable.starts_with(&target));

        let initialized = Command::new(executable)
            .args([OsStr::new("-C"), target.as_os_str(), OsStr::new("init")])
            .arg("--quiet")
            .env_clear()
            .envs(fixed_git_environment())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("initialize test repository");
        assert!(initialized.success());

        let arguments = vec![
            OsString::from("-C"),
            target.as_os_str().to_os_string(),
            OsString::from("rev-parse"),
            OsString::from("--show-toplevel"),
        ];
        let output = runner.run(&arguments).expect("qualified Git command");
        assert!(
            output.success,
            "stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let root = std::str::from_utf8(&output.stdout)
            .expect("UTF-8 root")
            .trim();
        assert_eq!(Path::new(root).canonicalize().expect("Git root"), target);
    }

    #[test]
    fn repo_local_fsmonitor_is_force_disabled_before_every_inventory_subcommand() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let runner = SystemGitRunner::resolve(&target).expect("system Git resolution");
        let executable = runner
            .executable
            .as_deref()
            .expect("Git must be available for the source checkout");
        run_test_git(executable, &target, &["init", "--quiet"]);
        fs::write(target.join("tracked.txt"), b"tracked\n").expect("tracked fixture");
        run_test_git(executable, &target, &["add", "--", "tracked.txt"]);

        let hook = target.join("malicious-fsmonitor");
        fs::write(
            &hook,
            b"#!/bin/sh\nprintf invoked > fsmonitor-invoked\nprintf 'builtin:fake-token\\n'\n",
        )
        .expect("fsmonitor hook");
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&hook).expect("hook metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook, permissions).expect("hook executable permissions");
        }
        let hook_value = hook.to_string_lossy().replace('\\', "/");
        run_test_git(
            executable,
            &target,
            &["config", "--local", "core.fsmonitor", &hook_value],
        );

        run_test_git(executable, &target, &["status", "--short"]);
        let sentinel = target.join("fsmonitor-invoked");
        assert!(
            sentinel.exists(),
            "hostile fixture must prove the repository-local fsmonitor is executable"
        );
        fs::remove_file(&sentinel).expect("remove baseline sentinel");

        let arguments = vec![
            OsString::from("-C"),
            target.as_os_str().to_os_string(),
            OsString::from("ls-files"),
            OsString::from("-z"),
            OsString::from("--others"),
            OsString::from("--exclude-standard"),
            OsString::from("--"),
        ];
        let hardened = hardened_git_arguments(&arguments);
        let subcommand = hardened
            .iter()
            .position(|argument| argument == "ls-files")
            .expect("inventory subcommand");
        for override_value in GIT_SECURITY_CONFIG_OVERRIDES {
            let position = hardened
                .windows(2)
                .position(|pair| pair[0] == "-c" && pair[1] == *override_value)
                .expect("security override");
            assert!(position < subcommand);
        }
        assert!(hardened.windows(2).any(|pair| {
            pair[0] == "-c" && pair[1].to_string_lossy().starts_with("safe.directory=")
        }));
        #[cfg(windows)]
        assert!(hardened.windows(2).any(|pair| {
            pair[0] == "-c"
                && pair[1].to_string_lossy().starts_with("safe.directory=")
                && !pair[1].to_string_lossy().contains('\\')
        }));

        let output = runner
            .run(&arguments)
            .expect("hardened untracked inventory");
        assert!(output.success, "Git inventory must remain functional");
        assert!(
            !sentinel.exists(),
            "repository-local fsmonitor must never execute during inventory"
        );
    }

    #[test]
    fn bounded_git_process_times_out_and_is_terminated() {
        let error = run_git_process_helper(
            "timeout",
            Duration::from_millis(50),
            MAX_GIT_VERSION_OUTPUT_BYTES,
        )
        .expect_err("helper must time out");
        assert!(matches!(error, WikiError::Io(message) if message.contains("timeout")));
    }

    #[test]
    fn bounded_git_timeout_kills_descendants_that_hold_output_pipes() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let survivor = temporary.path().join("descendant-survived");
        let started = temporary.path().join("descendant-started");
        let start = Instant::now();
        let error = run_git_process_helper_with_environment(
            "descendant-pipe-parent",
            Duration::from_secs(1),
            MAX_GIT_VERSION_OUTPUT_BYTES,
            &[
                (
                    OsString::from("HIVE_TEST_DESCENDANT_STARTED"),
                    started.as_os_str().to_os_string(),
                ),
                (
                    OsString::from("HIVE_TEST_DESCENDANT_SURVIVOR"),
                    survivor.as_os_str().to_os_string(),
                ),
            ],
        )
        .expect_err("helper tree must time out");
        assert!(matches!(error, WikiError::Io(message) if message.contains("timeout")));
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "timeout and reader cleanup must remain globally bounded"
        );
        assert!(started.exists(), "hostile descendant fixture must start");
        thread::sleep(Duration::from_secs(2));
        assert!(
            !survivor.exists(),
            "the timed-out process tree must not leave a surviving descendant"
        );
    }

    #[test]
    fn completed_git_parent_cannot_orphan_a_descendant_that_holds_output_pipes() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let survivor = temporary.path().join("orphan-survived");
        let started = temporary.path().join("orphan-started");
        let start = Instant::now();
        let output = run_git_process_helper_with_environment(
            "orphan-pipe-parent",
            Duration::from_secs(5),
            MAX_GIT_VERSION_OUTPUT_BYTES,
            &[
                (
                    OsString::from("HIVE_TEST_DESCENDANT_STARTED"),
                    started.as_os_str().to_os_string(),
                ),
                (
                    OsString::from("HIVE_TEST_DESCENDANT_SURVIVOR"),
                    survivor.as_os_str().to_os_string(),
                ),
            ],
        )
        .expect("completed direct child must be collected after its job is drained");
        assert!(output.success);
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "a descendant-held pipe must not extend collection to the command timeout"
        );
        assert!(started.exists(), "hostile orphan fixture must start");
        thread::sleep(Duration::from_secs(2));
        assert!(
            !survivor.exists(),
            "Job Object or process-group containment must kill an orphan after its parent exits"
        );
    }

    #[test]
    fn bounded_git_process_rejects_oversized_streaming_output() {
        let error = run_git_process_helper("oversized", Duration::from_secs(5), 1_024)
            .expect_err("helper output must exceed the budget");
        assert!(matches!(error, WikiError::InvalidInput(message) if message.contains("output")));
    }

    #[test]
    fn classification_and_total_budget_precede_content_retention() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        fs::write(temporary.path().join("LICENSE"), b"skip").expect("license");
        fs::write(temporary.path().join("a.md"), b"aaaa").expect("first evidence");
        fs::write(temporary.path().join("b.md"), b"bbbb").expect("second evidence");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let options = ScanOptions {
            root_kind: ScanRootKind::Git,
            include_untracked: true,
            limits: ScanLimits {
                max_discovered_files: 3,
                max_included_files: 3,
                max_file_bytes: 4,
                max_total_bytes: 4,
            },
        };
        let mut reads = Vec::new();
        let entries = read_discovered_files_with_observer(
            &canonical,
            vec![
                ("b.md".to_owned(), true),
                ("LICENSE".to_owned(), true),
                ("a.md".to_owned(), true),
            ],
            options,
            &mut |path| reads.push(path.to_owned()),
        )
        .expect("bounded discovery");
        assert_eq!(reads, vec!["a.md"]);
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().any(|entry| {
            entry.relative_path == "LICENSE"
                && entry.decision == ScanDecision::Skipped
                && entry.reason == "license-text"
        }));
        assert!(entries.iter().any(|entry| {
            entry.relative_path == "b.md"
                && entry.decision == ScanDecision::Skipped
                && entry.reason == "total-byte-budget"
        }));

        let actual = finalize_inventory(entries, options, None).expect("streaming inventory");
        let expected = build_inventory(
            &[
                ScanInputFile {
                    relative_path: "b.md",
                    bytes: b"bbbb",
                    observed_byte_len: 4,
                    tracked: true,
                    file_kind: ScanFileKind::Regular,
                },
                ScanInputFile {
                    relative_path: "LICENSE",
                    bytes: b"skip",
                    observed_byte_len: 4,
                    tracked: true,
                    file_kind: ScanFileKind::Regular,
                },
                ScanInputFile {
                    relative_path: "a.md",
                    bytes: b"aaaa",
                    observed_byte_len: 4,
                    tracked: true,
                    file_kind: ScanFileKind::Regular,
                },
            ],
            options,
            None,
        )
        .expect("canonical bulk inventory");
        assert_eq!(actual, expected);
    }

    #[test]
    fn non_git_traversal_rejects_excess_visited_directories() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        fs::create_dir_all(temporary.path().join("a")).expect("first directory");
        fs::create_dir_all(temporary.path().join("b")).expect("second directory");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let error =
            discover_non_git_files_with_bounds(&canonical, non_git_options(), 2, MAX_NON_GIT_DEPTH)
                .expect_err("root plus two directories must exceed the budget");
        assert!(
            matches!(error, WikiError::InvalidInput(message) if message.contains("visited directory"))
        );
    }

    #[test]
    fn non_git_traversal_rejects_excess_depth() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        fs::create_dir_all(temporary.path().join("a/b/c")).expect("deep directories");
        let canonical = temporary.path().canonicalize().expect("canonical target");
        let error = discover_non_git_files_with_bounds(
            &canonical,
            non_git_options(),
            MAX_NON_GIT_DIRECTORIES,
            2,
        )
        .expect_err("third nested directory must exceed the depth budget");
        assert!(matches!(error, WikiError::InvalidInput(message) if message.contains("depth")));
    }

    fn non_git_options() -> ScanOptions {
        ScanOptions {
            root_kind: ScanRootKind::NonGit,
            include_untracked: true,
            limits: ScanLimits::default(),
        }
    }

    fn run_git_process_helper(
        mode: &str,
        timeout: Duration,
        output_limit: usize,
    ) -> Result<GitOutput, WikiError> {
        run_git_process_helper_with_environment(mode, timeout, output_limit, &[])
    }

    fn run_git_process_helper_with_environment(
        mode: &str,
        timeout: Duration,
        output_limit: usize,
        extra_environment: &[(OsString, OsString)],
    ) -> Result<GitOutput, WikiError> {
        let executable = env::current_exe().expect("current test executable");
        let arguments = [
            OsString::from("--exact"),
            OsString::from("knowledge_scan::tests::git_process_helper"),
            OsString::from("--nocapture"),
        ];
        let mut environment = fixed_git_environment();
        environment.push((
            OsString::from("HIVE_TEST_GIT_HELPER_MODE"),
            OsString::from(mode),
        ));
        environment.extend_from_slice(extra_environment);
        run_bounded_process(&executable, &arguments, &environment, timeout, output_limit)
    }

    fn run_test_git(executable: &Path, target: &Path, arguments: &[&str]) {
        let status = Command::new(executable)
            .arg("-C")
            .arg(target)
            .args(arguments)
            .env_clear()
            .envs(fixed_git_environment())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("run test Git command");
        assert!(status.success(), "test Git command must succeed");
    }

    #[test]
    #[allow(clippy::zombie_processes)]
    fn git_process_helper() {
        match env::var("HIVE_TEST_GIT_HELPER_MODE").as_deref() {
            Ok("timeout") => thread::sleep(Duration::from_secs(5)),
            Ok("oversized") => {
                std::io::stdout()
                    .write_all(&vec![b'x'; 16 * 1024])
                    .expect("helper stdout");
            }
            Ok(mode @ ("descendant-pipe-parent" | "orphan-pipe-parent")) => {
                let executable = env::current_exe().expect("current test executable");
                let arguments = [
                    OsString::from("--exact"),
                    OsString::from("knowledge_scan::tests::git_process_helper"),
                    OsString::from("--nocapture"),
                ];
                let mut environment = fixed_git_environment();
                environment.push((
                    OsString::from("HIVE_TEST_GIT_HELPER_MODE"),
                    OsString::from("descendant-pipe-child"),
                ));
                for name in [
                    "HIVE_TEST_DESCENDANT_STARTED",
                    "HIVE_TEST_DESCENDANT_SURVIVOR",
                ] {
                    environment.push((
                        OsString::from(name),
                        env::var_os(name).expect("descendant fixture path"),
                    ));
                }
                let _descendant = Command::new(executable)
                    .args(arguments)
                    .env_clear()
                    .envs(environment)
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .expect("spawn pipe-holding descendant");
                if mode == "orphan-pipe-parent" {
                    let started = PathBuf::from(
                        env::var_os("HIVE_TEST_DESCENDANT_STARTED").expect("started path"),
                    );
                    let deadline = Instant::now() + Duration::from_secs(4);
                    while !started.exists() && Instant::now() < deadline {
                        thread::sleep(GIT_WAIT_POLL_INTERVAL);
                    }
                    assert!(
                        started.exists(),
                        "descendant must start before parent exits"
                    );
                } else {
                    thread::sleep(Duration::from_secs(5));
                }
            }
            Ok("descendant-pipe-child") => {
                fs::write(
                    env::var_os("HIVE_TEST_DESCENDANT_STARTED").expect("started path"),
                    b"started",
                )
                .expect("descendant started marker");
                thread::sleep(Duration::from_millis(1_500));
                fs::write(
                    env::var_os("HIVE_TEST_DESCENDANT_SURVIVOR").expect("survivor path"),
                    b"survived",
                )
                .expect("descendant survivor marker");
                thread::sleep(Duration::from_secs(5));
            }
            _ => {}
        }
    }
}
