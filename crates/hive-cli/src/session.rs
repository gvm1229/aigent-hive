use cap_fs_ext::OpenOptionsFollowExt;
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use hive_core::{ensure_consumer_target, sha256_digest, validate_project_relative};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SESSION_DIRECTORY: &str = ".hive/runtime/active-sessions";
const LOCK_FILE: &str = ".session-coordination.lock";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    Begin,
    Check,
    Update,
    Close,
    Recover,
}

#[derive(Debug)]
struct Arguments {
    action: Action,
    target: PathBuf,
    host: Option<String>,
    session_id: Option<String>,
    process_id: Option<u32>,
    paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionManifest {
    schema_version: u32,
    host: String,
    session_id: String,
    process_id: u32,
    paths: Vec<String>,
}

#[derive(Serialize)]
struct SessionResult {
    schema_version: u32,
    action: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
    changed_paths: Vec<String>,
    evidence: Vec<Evidence>,
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct Evidence {
    kind: &'static str,
    locator: String,
    digest: String,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Liveness {
    Live,
    Dead,
    #[cfg(not(target_os = "linux"))]
    Unknown,
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    let result = match parse(arguments) {
        Ok(arguments) => execute(&arguments).unwrap_or_else(|message| {
            failure(message, "hive.session-coordination-blocked", "conflict", 3)
        }),
        Err(message) => failure(message, "hive.invalid-input", "error", 2),
    };
    emit(&result);
    ExitCode::from(result.exit_code)
}

fn parse(arguments: &[String]) -> Result<Arguments, String> {
    let action = match arguments.first().map(String::as_str) {
        Some("begin") => Action::Begin,
        Some("check") => Action::Check,
        Some("update") => Action::Update,
        Some("close") => Action::Close,
        Some("recover") => Action::Recover,
        _ => return Err("session requires begin, check, update, close, or recover".to_owned()),
    };
    let mut target = None;
    let mut host = None;
    let mut session_id = None;
    let mut process_id = None;
    let mut paths = Vec::new();
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--target" if target.is_none() => target = Some(PathBuf::from(value)),
            "--host" if host.is_none() => host = Some(parse_host(value)?),
            "--session-id" if session_id.is_none() => session_id = Some(parse_session_id(value)?),
            "--process-id" if process_id.is_none() => {
                process_id = Some(
                    value
                        .parse::<u32>()
                        .ok()
                        .filter(|value| *value > 0)
                        .ok_or_else(|| "process id must be a positive u32".to_owned())?,
                );
            }
            "--path" => paths.push(normalize_path(value)?),
            "--output" if output.is_none() => output = Some(value.clone()),
            "--target" | "--host" | "--session-id" | "--process-id" | "--output" => {
                return Err(format!("duplicate session option: {option}"));
            }
            _ => return Err(format!("unknown session option: {option}")),
        }
        index += 2;
    }
    if output.as_deref() != Some("json") {
        return Err("session requires --output json".to_owned());
    }
    let target = target.ok_or_else(|| "missing required option --target".to_owned())?;
    match action {
        Action::Recover => {
            if host.is_some() || session_id.is_some() || process_id.is_some() || !paths.is_empty() {
                return Err("session recover accepts only --target and --output json".to_owned());
            }
        }
        Action::Close => {
            if process_id.is_some() || !paths.is_empty() {
                return Err("session close accepts no --process-id or --path".to_owned());
            }
            if host.is_none() {
                return Err("missing required option --host".to_owned());
            }
            if session_id.is_none() {
                return Err("missing required option --session-id".to_owned());
            }
        }
        Action::Begin | Action::Check | Action::Update => {
            require_identity(host.as_ref(), session_id.as_ref(), process_id)?;
            if paths.is_empty() {
                return Err(
                    "session begin, check, and update require at least one --path".to_owned(),
                );
            }
        }
    }
    let paths = paths
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(Arguments {
        action,
        target,
        host,
        session_id,
        process_id,
        paths,
    })
}

fn require_identity(
    host: Option<&String>,
    session_id: Option<&String>,
    process_id: Option<u32>,
) -> Result<(), String> {
    if host.is_none() {
        return Err("missing required option --host".to_owned());
    }
    if session_id.is_none() {
        return Err("missing required option --session-id".to_owned());
    }
    if process_id.is_none() {
        return Err("missing required option --process-id".to_owned());
    }
    Ok(())
}

fn parse_host(value: &str) -> Result<String, String> {
    match value {
        "codex" | "claude" | "antigravity" => Ok(value.to_owned()),
        _ => Err("session --host must be codex, claude, or antigravity".to_owned()),
    }
}

fn parse_session_id(value: &str) -> Result<String, String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(
            "session id must use 1..128 ASCII letters, digits, dot, hyphen, or underscore"
                .to_owned(),
        );
    }
    Ok(value.to_owned())
}

fn normalize_path(value: &str) -> Result<String, String> {
    let path = Path::new(value);
    validate_project_relative(path).map_err(|error| error.to_string())?;
    let portable = value.replace('\\', "/");
    if portable == ".hive" || portable.starts_with(".hive/runtime/") {
        return Err("session paths must not reserve Hive runtime state".to_owned());
    }
    Ok(portable)
}

fn execute(arguments: &Arguments) -> Result<SessionResult, String> {
    ensure_consumer_target(&arguments.target).map_err(|error| error.to_string())?;
    let root = Dir::open_ambient_dir(&arguments.target, ambient_authority())
        .map_err(|error| format!("cannot open consumer target: {error}"))?;
    let sessions = open_session_directory(&root)?;
    let _lock = acquire_lock(&sessions)?;
    match arguments.action {
        Action::Begin => begin(&sessions, arguments),
        Action::Check => check(&sessions, arguments),
        Action::Update => update(&sessions, arguments),
        Action::Close => close(&sessions, arguments),
        Action::Recover => recover(&sessions),
    }
}

fn begin(sessions: &Dir, arguments: &Arguments) -> Result<SessionResult, String> {
    let manifest = requested_manifest(arguments)?;
    let own_name = manifest_file_name(&manifest);
    let existing = read_manifests(sessions)?;
    if let Some((_, prior)) = existing.iter().find(|(name, _)| name == &own_name) {
        if prior != &manifest {
            return Err(
                "session already exists with another path set; use hive session update".to_owned(),
            );
        }
        return success(
            "BeginSession",
            "hive.session-already-active",
            "session reservation is already active",
            Vec::new(),
            &manifest,
        );
    }
    ensure_no_live_conflict(&existing, &manifest)?;
    write_manifest(sessions, &own_name, &manifest)?;
    success(
        "BeginSession",
        "hive.session-begun",
        "session reservation created",
        vec![session_locator(&own_name)],
        &manifest,
    )
}

fn check(sessions: &Dir, arguments: &Arguments) -> Result<SessionResult, String> {
    let manifest = requested_manifest(arguments)?;
    let existing = read_manifests(sessions)?;
    ensure_no_live_conflict(&existing, &manifest)?;
    success(
        "CheckSession",
        "hive.session-clear",
        "no live conflicting session reservation",
        Vec::new(),
        &manifest,
    )
}

fn update(sessions: &Dir, arguments: &Arguments) -> Result<SessionResult, String> {
    let manifest = requested_manifest(arguments)?;
    let own_name = manifest_file_name(&manifest);
    let existing = read_manifests(sessions)?;
    if !existing.iter().any(|(name, _)| name == &own_name) {
        return Err(
            "session update requires an existing reservation; use hive session begin".to_owned(),
        );
    }
    let foreign = existing
        .into_iter()
        .filter(|(name, _)| name != &own_name)
        .collect::<Vec<_>>();
    ensure_no_live_conflict(&foreign, &manifest)?;
    replace_manifest(sessions, &own_name, &manifest)?;
    success(
        "UpdateSession",
        "hive.session-updated",
        "session reservation updated",
        vec![session_locator(&own_name)],
        &manifest,
    )
}

fn close(sessions: &Dir, arguments: &Arguments) -> Result<SessionResult, String> {
    let host = arguments.host.as_deref().expect("validated host");
    let session_id = arguments
        .session_id
        .as_deref()
        .expect("validated session id");
    let name = manifest_file_name_from(host, session_id);
    let Some(manifest) = read_manifest_optional(sessions, &name)? else {
        return Ok(SessionResult {
            schema_version: 1,
            action: "CloseSession",
            status: "success",
            exit_code: 0,
            code: "hive.session-not-active",
            message: "session reservation was not active".to_owned(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
            data: Some(json!({ "host": host, "session_id": session_id })),
        });
    };
    if manifest.host != host || manifest.session_id != session_id {
        return Err("session manifest identity does not match its file name".to_owned());
    }
    sessions
        .remove_file(&name)
        .map_err(|error| format!("cannot close session reservation: {error}"))?;
    success(
        "CloseSession",
        "hive.session-closed",
        "session reservation closed",
        vec![session_locator(&name)],
        &manifest,
    )
}

fn recover(sessions: &Dir) -> Result<SessionResult, String> {
    let manifests = read_manifests(sessions)?;
    let mut removed = Vec::new();
    #[cfg(target_os = "linux")]
    let unknown: Vec<String> = Vec::new();
    #[cfg(not(target_os = "linux"))]
    let mut unknown: Vec<String> = Vec::new();
    for (name, manifest) in manifests {
        match process_liveness(manifest.process_id) {
            Liveness::Dead => {
                sessions
                    .remove_file(&name)
                    .map_err(|error| format!("cannot remove stale session manifest: {error}"))?;
                removed.push(session_locator(&name));
            }
            #[cfg(not(target_os = "linux"))]
            Liveness::Unknown => unknown.push(session_locator(&name)),
            Liveness::Live => {}
        }
    }
    Ok(SessionResult {
        schema_version: 1,
        action: "RecoverSession",
        status: "success",
        exit_code: 0,
        code: if removed.is_empty() { "hive.session-no-recoverable-stale-state" } else { "hive.session-recovered-stale-state" },
        message: if unknown.is_empty() { "stale session recovery completed".to_owned() } else { "stale session recovery completed; some process liveness states could not be verified".to_owned() },
        changed_paths: removed,
        evidence: Vec::new(),
        next_action: (!unknown.is_empty()).then_some("close the named session from its owning host, or retry recovery on a platform that can verify its process liveness".to_owned()),
        data: Some(json!({ "unverified_manifests": unknown })),
    })
}

fn requested_manifest(arguments: &Arguments) -> Result<SessionManifest, String> {
    Ok(SessionManifest {
        schema_version: 1,
        host: arguments
            .host
            .clone()
            .ok_or_else(|| "missing required option --host".to_owned())?,
        session_id: arguments
            .session_id
            .clone()
            .ok_or_else(|| "missing required option --session-id".to_owned())?,
        process_id: arguments
            .process_id
            .ok_or_else(|| "missing required option --process-id".to_owned())?,
        paths: arguments.paths.clone(),
    })
}

fn ensure_no_live_conflict(
    existing: &[(String, SessionManifest)],
    requested: &SessionManifest,
) -> Result<(), String> {
    for (name, manifest) in existing {
        if manifest.host == requested.host && manifest.session_id == requested.session_id {
            continue;
        }
        if process_liveness(manifest.process_id) != Liveness::Live {
            return Err(format!("cannot determine whether session manifest {} is stale; run hive session recover or close it from its owning host", session_locator(name)));
        }
        if manifest.paths.iter().any(|left| {
            requested
                .paths
                .iter()
                .any(|right| paths_overlap(left, right))
        }) {
            return Err(format!("live session reservation conflicts at {}; stop before an automated overlapping write", session_locator(name)));
        }
    }
    Ok(())
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn open_session_directory(root: &Dir) -> Result<Dir, String> {
    let hive = open_or_create_directory(root, ".hive")?;
    let runtime = open_or_create_directory(&hive, "runtime")?;
    open_or_create_directory(&runtime, "active-sessions")
}

fn open_or_create_directory(parent: &Dir, name: &str) -> Result<Dir, String> {
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => {
            return Err(format!(
                "session state ancestor is not a safe directory: {name}"
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => parent
            .create_dir(name)
            .map_err(|error| format!("cannot create session state directory {name}: {error}"))?,
        Err(error) => {
            return Err(format!(
                "cannot inspect session state directory {name}: {error}"
            ))
        }
    }
    parent
        .open_dir(name)
        .map_err(|error| format!("cannot open session state directory {name}: {error}"))
}

struct Lock<'a> {
    directory: &'a Dir,
}

impl Drop for Lock<'_> {
    fn drop(&mut self) {
        let _ = self.directory.remove_file(LOCK_FILE);
    }
}

fn acquire_lock(sessions: &Dir) -> Result<Lock<'_>, String> {
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    for _ in 0..2 {
        match sessions.open_with(LOCK_FILE, &options) {
            Ok(mut file) => {
                let initialized = file
                    .write_all(format!("process_id={}\n", std::process::id()).as_bytes())
                    .and_then(|()| file.sync_all());
                if let Err(error) = initialized {
                    let _ = sessions.remove_file(LOCK_FILE);
                    return Err(format!(
                        "cannot initialize session coordination lock: {error}"
                    ));
                }
                return Ok(Lock {
                    directory: sessions,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let process_id = read_lock_process_id(sessions)?;
                if process_liveness(process_id) == Liveness::Dead {
                    sessions.remove_file(LOCK_FILE).map_err(|error| {
                        format!("cannot clear verified stale session coordination lock: {error}")
                    })?;
                    continue;
                }
                return Err(
                    "session coordination is busy or its lock owner cannot be verified".to_owned(),
                );
            }
            Err(error) => {
                return Err(format!(
                    "cannot acquire exclusive session coordination lock: {error}"
                ))
            }
        }
    }
    Err("session coordination lock was recreated concurrently".to_owned())
}

fn read_lock_process_id(sessions: &Dir) -> Result<u32, String> {
    let metadata = sessions
        .symlink_metadata(LOCK_FILE)
        .map_err(|error| format!("cannot inspect session coordination lock: {error}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 128 {
        return Err("session coordination lock is not a safe bounded regular file".to_owned());
    }
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = sessions
        .open_with(LOCK_FILE, &options)
        .map_err(|error| format!("cannot open session coordination lock: {error}"))?;
    let mut bytes = String::new();
    file.take(129)
        .read_to_string(&mut bytes)
        .map_err(|error| format!("cannot read session coordination lock: {error}"))?;
    bytes
        .strip_prefix("process_id=")
        .and_then(|value| value.strip_suffix('\n'))
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| "session coordination lock has no valid process id".to_owned())
}

fn read_manifests(sessions: &Dir) -> Result<Vec<(String, SessionManifest)>, String> {
    let mut manifests = Vec::new();
    for entry in sessions
        .entries()
        .map_err(|error| format!("cannot list session manifests: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("cannot inspect session manifest entry: {error}"))?;
        let name = entry.file_name();
        if name == OsStr::new(LOCK_FILE) {
            continue;
        }
        let name = name
            .to_str()
            .ok_or_else(|| "session manifest file name is not UTF-8".to_owned())?
            .to_owned();
        if !Path::new(&name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            return Err(format!("unexpected session coordination entry: {name}"));
        }
        let metadata = sessions
            .symlink_metadata(&name)
            .map_err(|error| format!("cannot inspect session manifest {name}: {error}"))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_MANIFEST_BYTES
        {
            return Err(format!(
                "session manifest is not a safe bounded regular file: {name}"
            ));
        }
        let manifest = read_manifest_optional(sessions, &name)?
            .ok_or_else(|| format!("session manifest disappeared while reading: {name}"))?;
        if manifest_file_name(&manifest) != name {
            return Err(format!(
                "session manifest identity does not match its path: {name}"
            ));
        }
        manifests.push((name, manifest));
    }
    manifests.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(manifests)
}

fn read_manifest_optional(sessions: &Dir, name: &str) -> Result<Option<SessionManifest>, String> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = match sessions.open_with(name, &options) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("cannot open session manifest {name}: {error}")),
    };
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read session manifest {name}: {error}"))?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(format!("session manifest exceeds its byte limit: {name}"));
    }
    parse_manifest(&bytes)
        .map(Some)
        .map_err(|error| format!("invalid session manifest {name}: {error}"))
}

fn write_manifest(sessions: &Dir, name: &str, manifest: &SessionManifest) -> Result<(), String> {
    let bytes = render_manifest(manifest)?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = sessions
        .open_with(name, &options)
        .map_err(|error| format!("cannot exclusively create session manifest: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("cannot write session manifest: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("cannot persist session manifest: {error}"))
}

fn replace_manifest(sessions: &Dir, name: &str, manifest: &SessionManifest) -> Result<(), String> {
    sessions
        .remove_file(name)
        .map_err(|error| format!("cannot replace session manifest: {error}"))?;
    write_manifest(sessions, name, manifest)
}

fn render_manifest(manifest: &SessionManifest) -> Result<Vec<u8>, String> {
    validate_manifest(manifest)?;
    let yaml = serde_yaml::to_string(manifest)
        .map_err(|error| format!("cannot serialize session manifest: {error}"))?;
    Ok(format!("---\n{yaml}---\n# Aigent Hive active session\n\nEphemeral reservation for one Hive-aware automated edit set.\n").into_bytes())
}

fn parse_manifest(bytes: &[u8]) -> Result<SessionManifest, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| "session manifest is not UTF-8".to_owned())?;
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing opening YAML delimiter".to_owned());
    };
    let Some((yaml, _body)) = rest.split_once("---\n") else {
        return Err("missing closing YAML delimiter".to_owned());
    };
    let manifest =
        serde_yaml::from_str::<SessionManifest>(yaml).map_err(|error| error.to_string())?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &SessionManifest) -> Result<(), String> {
    if manifest.schema_version != 1 {
        return Err("unsupported session manifest schema version".to_owned());
    }
    let _ = parse_host(&manifest.host)?;
    let _ = parse_session_id(&manifest.session_id)?;
    if manifest.process_id == 0 || manifest.paths.is_empty() {
        return Err("session manifest identity or paths are invalid".to_owned());
    }
    let canonical = manifest
        .paths
        .iter()
        .map(|path| normalize_path(path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if canonical.len() != manifest.paths.len()
        || canonical.iter().cloned().collect::<Vec<_>>() != manifest.paths
    {
        return Err("session manifest paths are not sorted unique canonical paths".to_owned());
    }
    Ok(())
}

fn manifest_file_name(manifest: &SessionManifest) -> String {
    manifest_file_name_from(&manifest.host, &manifest.session_id)
}

fn manifest_file_name_from(host: &str, session_id: &str) -> String {
    format!("{host}-{session_id}.md")
}

fn session_locator(name: &str) -> String {
    format!("{SESSION_DIRECTORY}/{name}")
}

fn success(
    action: &'static str,
    code: &'static str,
    message: &str,
    changed_paths: Vec<String>,
    manifest: &SessionManifest,
) -> Result<SessionResult, String> {
    let manifest_bytes = render_manifest(manifest)?;
    Ok(SessionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![Evidence {
            kind: "session-manifest",
            locator: session_locator(&manifest_file_name(manifest)),
            digest: sha256_digest(&manifest_bytes),
        }],
        next_action: None,
        data: Some(
            json!({ "host": manifest.host, "session_id": manifest.session_id, "paths": manifest.paths }),
        ),
    })
}

fn process_liveness(process_id: u32) -> Liveness {
    if process_id == std::process::id() {
        return Liveness::Live;
    }
    #[cfg(target_os = "linux")]
    {
        if Path::new("/proc").join(process_id.to_string()).is_dir() {
            Liveness::Live
        } else {
            Liveness::Dead
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        process_liveness_platform(process_id)
    }
}

#[cfg(windows)]
fn process_liveness_platform(process_id: u32) -> Liveness {
    use std::process::Command;

    // Windows has no portable `/proc` probe. Query its system task listing through
    // an absolute system path; an unavailable or unparseable probe stays unknown so
    // Hive never removes a possibly live foreign reservation.
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let tasklist = PathBuf::from(system_root)
        .join("System32")
        .join("tasklist.exe");
    let output = match Command::new(tasklist)
        .args(["/FI", &format!("PID eq {process_id}"), "/FO", "CSV", "/NH"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return Liveness::Unknown,
    };
    let stdout = String::from_utf8(output.stdout).ok();
    match stdout {
        Some(stdout) if stdout.contains(&format!("\"{process_id}\"")) => Liveness::Live,
        Some(_) => Liveness::Dead,
        None => Liveness::Unknown,
    }
}

#[cfg(target_os = "macos")]
fn process_liveness_platform(process_id: u32) -> Liveness {
    use std::process::Command;

    // macOS has no Linux-style `/proc` tree. Its system `ps` utility can make a
    // bounded, read-only liveness check; an unavailable or unparseable result
    // remains unknown so Hive never removes a possibly live reservation.
    let expected = process_id.to_string();
    let output = match Command::new("/bin/ps")
        .args(["-p", &expected, "-o", "pid="])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return Liveness::Unknown,
    };
    let stdout = String::from_utf8(output.stdout).ok();
    match stdout {
        Some(stdout) if stdout.split_whitespace().any(|value| value == expected) => Liveness::Live,
        Some(_) => Liveness::Dead,
        None => Liveness::Unknown,
    }
}

#[cfg(all(not(windows), not(target_os = "linux"), not(target_os = "macos")))]
fn process_liveness_platform(_process_id: u32) -> Liveness {
    Liveness::Unknown
}

fn failure(
    message: String,
    code: &'static str,
    status: &'static str,
    exit_code: u8,
) -> SessionResult {
    SessionResult {
        schema_version: 1,
        action: "CoordinateSession",
        status,
        exit_code,
        code,
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: Some("resolve the named live or unverifiable session reservation before retrying an automated write".to_owned()),
        data: None,
    }
}

fn emit(result: &SessionResult) {
    println!("{}", serde_json::to_string(result).unwrap_or_else(|_| "{\"schema_version\":1,\"action\":\"CoordinateSession\",\"status\":\"error\",\"exit_code\":10,\"code\":\"hive.internal-error\",\"message\":\"JSON serialization failed\",\"changed_paths\":[],\"evidence\":[],\"next_action\":null}".to_owned()));
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_equal_ancestor_and_descendant_path_overlap() {
        assert!(paths_overlap("src", "src/lib.rs"));
        assert!(paths_overlap("src/lib.rs", "src"));
        assert!(paths_overlap("AGENTS.md", "AGENTS.md"));
        assert!(!paths_overlap("src", "scripts"));
        assert!(!paths_overlap("src", "source"));
    }

    #[test]
    fn manifest_round_trip_requires_canonical_paths() {
        let manifest = SessionManifest {
            schema_version: 1,
            host: "codex".to_owned(),
            session_id: "session-1".to_owned(),
            process_id: 42,
            paths: vec!["AGENTS.md".to_owned(), "src/lib.rs".to_owned()],
        };
        let rendered = render_manifest(&manifest).expect("render");
        assert_eq!(
            parse_manifest(&rendered).expect("parse").paths,
            manifest.paths
        );
    }
}
