use hive_core::sha256_digest;
use hive_core::usage_guard::{
    evaluate_usage, SourceConfidence, UsageDecision, UsagePermit, UsagePermitError, UsagePolicy,
    UsageSnapshot, UsageWindow,
};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CODEXBAR_VERSION: &str = "0.45.2";
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const USAGE_TIMEOUT: Duration = Duration::from_mins(1);
const OUTPUT_LIMIT: usize = 1024 * 1024;
const USAGE_ARGUMENTS: &[&str] = &[
    "usage",
    "--provider",
    "codex",
    "--all-accounts",
    "--source",
    "cli",
    "--format",
    "json",
    "--json-only",
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SensorError {
    Unavailable,
    Timeout,
    OutputTooLarge,
    Failed,
    UnsupportedVersion,
    Malformed,
    RowError,
    MissingIdentity,
    AccountNotFound,
    DuplicateAccount,
    WrongProvider,
    NonLocalSource,
    WrongWindows,
    Stale,
    ExecutableChanged,
}

impl Display for SensorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unavailable => "CodexBar usage sensor is unavailable",
            Self::Timeout => "CodexBar usage sensor timed out",
            Self::OutputTooLarge => "CodexBar usage sensor exceeded its output limit",
            Self::Failed => "CodexBar usage sensor failed",
            Self::UnsupportedVersion => "CodexBar usage sensor version is unsupported",
            Self::Malformed => "CodexBar usage sensor returned malformed data",
            Self::RowError => "CodexBar usage sensor returned an account error",
            Self::MissingIdentity => "CodexBar usage sensor omitted account identity",
            Self::AccountNotFound => "requested account digest was not found",
            Self::DuplicateAccount => "requested account digest matched more than one account",
            Self::WrongProvider => "CodexBar usage sensor returned the wrong provider",
            Self::NonLocalSource => "CodexBar usage sensor returned a non-local source",
            Self::WrongWindows => "CodexBar usage sensor returned unexpected quota windows",
            Self::Stale => "CodexBar usage sensor returned a stale snapshot",
            Self::ExecutableChanged => {
                "CodexBar usage sensor executable changed during qualification"
            }
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct QualifiedExecutable {
    path: PathBuf,
    identity: Option<ExecutableIdentity>,
}

impl QualifiedExecutable {
    #[cfg(test)]
    fn synthetic(program: &str) -> Self {
        Self {
            path: PathBuf::from(program),
            identity: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExecutableIdentity {
    len: u64,
    modified: Option<(u64, u32)>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    mode: u32,
    #[cfg(unix)]
    change_time: (i64, i64),
}

#[derive(Debug)]
pub(crate) struct CommandOutput {
    success: bool,
    stdout: Vec<u8>,
}

pub(crate) trait CommandRunner {
    fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError>;

    fn run(
        &self,
        program: &QualifiedExecutable,
        arguments: &[&str],
        timeout: Duration,
        output_limit: usize,
    ) -> Result<CommandOutput, SensorError>;
}

pub(crate) struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
        qualify_program(program)
    }

    fn run(
        &self,
        program: &QualifiedExecutable,
        arguments: &[&str],
        timeout: Duration,
        output_limit: usize,
    ) -> Result<CommandOutput, SensorError> {
        verify_executable_identity(program)?;
        let mut child = Command::new(&program.path)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    SensorError::Unavailable
                } else {
                    SensorError::Failed
                }
            })?;
        let stdout = child.stdout.take().ok_or(SensorError::Failed)?;
        let stderr = child.stderr.take().ok_or(SensorError::Failed)?;
        let stdout_reader = spawn_bounded_reader(stdout, output_limit);
        let stderr_reader = spawn_bounded_reader(stderr, output_limit);
        let started = Instant::now();
        let status = loop {
            match child.try_wait().map_err(|_| SensorError::Failed)? {
                Some(status) => break status,
                None if started.elapsed() >= timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    verify_executable_identity(program)?;
                    return Err(SensorError::Timeout);
                }
                None => thread::sleep(Duration::from_millis(10)),
            }
        };
        let stdout = receive_output(&stdout_reader, started, timeout);
        let stderr = receive_output(&stderr_reader, started, timeout);
        verify_executable_identity(program)?;
        let stdout = stdout?;
        let _stderr = stderr?;
        Ok(CommandOutput {
            success: status.success(),
            stdout,
        })
    }
}

fn qualify_program(program: &str) -> Result<QualifiedExecutable, SensorError> {
    let path = Path::new(program);
    let resolved = if path.components().count() > 1 {
        resolve_executable(path)
    } else {
        let search_path = std::env::var_os("PATH").ok_or(SensorError::Unavailable)?;
        resolve_program_in_path(program, &search_path)
    }
    .ok_or(SensorError::Unavailable)?;
    let identity = executable_identity(&resolved)?;
    Ok(QualifiedExecutable {
        path: resolved,
        identity: Some(identity),
    })
}

fn resolve_program_in_path(program: &str, search_path: &OsStr) -> Option<PathBuf> {
    std::env::split_paths(search_path).find_map(|directory| {
        let candidate = directory.join(program);
        if let Some(executable) = resolve_executable(&candidate) {
            return Some(executable);
        }
        #[cfg(windows)]
        {
            for extension in ["exe", "cmd", "bat"] {
                let executable = directory.join(format!("{program}.{extension}"));
                if let Some(executable) = resolve_executable(&executable) {
                    return Some(executable);
                }
            }
        }
        None
    })
}

fn resolve_executable(candidate: &Path) -> Option<PathBuf> {
    let resolved = candidate.canonicalize().ok()?;
    let metadata = fs::metadata(&resolved).ok()?;
    if !metadata.is_file() || !is_executable(&metadata) {
        return None;
    }
    Some(resolved)
}

#[cfg(unix)]
fn is_executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &Metadata) -> bool {
    true
}

fn executable_identity(path: &Path) -> Result<ExecutableIdentity, SensorError> {
    let metadata = fs::metadata(path).map_err(|_| SensorError::ExecutableChanged)?;
    if !metadata.is_file() || !is_executable(&metadata) {
        return Err(SensorError::ExecutableChanged);
    }
    let modified = metadata.modified().ok().and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| (duration.as_secs(), duration.subsec_nanos()))
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        Ok(ExecutableIdentity {
            len: metadata.len(),
            modified,
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            change_time: (metadata.ctime(), metadata.ctime_nsec()),
        })
    }
    #[cfg(not(unix))]
    {
        Ok(ExecutableIdentity {
            len: metadata.len(),
            modified,
        })
    }
}

fn verify_executable_identity(program: &QualifiedExecutable) -> Result<(), SensorError> {
    let expected = program
        .identity
        .as_ref()
        .ok_or(SensorError::ExecutableChanged)?;
    let current = executable_identity(&program.path)?;
    if current == *expected {
        Ok(())
    } else {
        Err(SensorError::ExecutableChanged)
    }
}

fn spawn_bounded_reader(
    reader: impl Read + Send + 'static,
    limit: usize,
) -> Receiver<Result<Vec<u8>, SensorError>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = sender.send(read_bounded(reader, limit));
    });
    receiver
}

fn receive_output(
    receiver: &Receiver<Result<Vec<u8>, SensorError>>,
    started: Instant,
    timeout: Duration,
) -> Result<Vec<u8>, SensorError> {
    let remaining = timeout
        .checked_sub(started.elapsed())
        .ok_or(SensorError::Timeout)?;
    receiver
        .recv_timeout(remaining)
        .map_err(|_| SensorError::Timeout)?
}

fn read_bounded(mut reader: impl Read, limit: usize) -> Result<Vec<u8>, SensorError> {
    let mut retained = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut exceeded = false;
    loop {
        let read = reader.read(&mut buffer).map_err(|_| SensorError::Failed)?;
        if read == 0 {
            break;
        }
        let available = limit.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..read.min(available)]);
        exceeded |= read > available;
    }
    if exceeded {
        Err(SensorError::OutputTooLarge)
    } else {
        Ok(retained)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct NormalizedWindow {
    pub(crate) name: &'static str,
    pub(crate) window_minutes: u32,
    pub(crate) remaining_percent: f64,
    pub(crate) resets_at: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct NormalizedSnapshot {
    pub(crate) sensor_id: &'static str,
    pub(crate) sensor_version: &'static str,
    pub(crate) provider: &'static str,
    pub(crate) account_digest: String,
    pub(crate) measured_at: u64,
    pub(crate) expires_at: u64,
    pub(crate) source_confidence: &'static str,
    pub(crate) windows: Vec<NormalizedWindow>,
}

impl NormalizedSnapshot {
    pub(crate) fn evidence_digest(&self) -> String {
        let bytes =
            serde_json::to_vec(&self.core_snapshots()).expect("usage snapshots should serialize");
        sha256_digest(&bytes)
    }

    pub(crate) fn core_snapshots(&self) -> Vec<UsageSnapshot> {
        self.windows
            .iter()
            .map(|window| UsageSnapshot {
                schema_version: 1,
                sensor_id: self.sensor_id.to_owned(),
                sensor_version: self.sensor_version.to_owned(),
                host_scope: self.provider.to_owned(),
                account_scope_digest: self.account_digest.clone(),
                quota_window: match window.name {
                    "session" => UsageWindow::Session,
                    "weekly" => UsageWindow::Weekly,
                    _ => unreachable!("normalization emits only required windows"),
                },
                remaining_percent: window.remaining_percent,
                measured_at_unix_seconds: i64::try_from(self.measured_at)
                    .expect("supported timestamps fit i64"),
                expires_at_unix_seconds: i64::try_from(self.expires_at)
                    .expect("supported timestamps fit i64"),
                resets_at_unix_seconds: i64::try_from(window.resets_at)
                    .expect("supported timestamps fit i64"),
                source_confidence: SourceConfidence::High,
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct CodexBarRow {
    provider: String,
    account: Option<String>,
    source: String,
    usage: Option<CodexBarUsage>,
    error: Option<serde_json::Value>,
}

struct CodexBarUsage {
    primary: Option<serde_json::Value>,
    secondary: Vec<serde_json::Value>,
    updated_at: String,
    identity: Option<CodexBarIdentity>,
}

impl<'de> Deserialize<'de> for CodexBarUsage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UsageVisitor;

        impl<'de> Visitor<'de> for UsageVisitor {
            type Value = CodexBarUsage;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a CodexBar usage object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut primary = None;
                let mut primary_seen = false;
                let mut secondary = Vec::new();
                let mut updated_at = None;
                let mut identity = None;
                let mut identity_seen = false;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "primary" => {
                            if primary_seen {
                                return Err(de::Error::duplicate_field("primary"));
                            }
                            primary_seen = true;
                            let value = map.next_value::<serde_json::Value>()?;
                            if !value.is_null() {
                                primary = Some(value);
                            }
                        }
                        "secondary" => {
                            secondary.push(map.next_value::<serde_json::Value>()?);
                        }
                        "updatedAt" => {
                            if updated_at.is_some() {
                                return Err(de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at = Some(map.next_value()?);
                        }
                        "identity" => {
                            if identity_seen {
                                return Err(de::Error::duplicate_field("identity"));
                            }
                            identity_seen = true;
                            identity = map.next_value()?;
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(CodexBarUsage {
                    primary,
                    secondary,
                    updated_at: updated_at.ok_or_else(|| de::Error::missing_field("updatedAt"))?,
                    identity,
                })
            }
        }

        deserializer.deserialize_map(UsageVisitor)
    }
}

#[derive(Deserialize)]
struct CodexBarWindow {
    #[serde(rename = "usedPercent")]
    used_percent: f64,
    #[serde(rename = "windowMinutes")]
    window_minutes: Option<u32>,
    #[serde(rename = "resetsAt")]
    resets_at: Option<String>,
}

#[derive(Deserialize)]
struct CodexBarIdentity {
    #[serde(rename = "providerID")]
    provider_id: String,
}

pub(crate) fn check_with_runner(
    runner: &impl CommandRunner,
    account_digest: &str,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    validate_account_digest(account_digest)?;
    check_with_runner_for_account(runner, Some(account_digest), now)
}

pub(crate) fn check_unique_with_runner(
    runner: &impl CommandRunner,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    check_with_runner_for_account(runner, None, now)
}

fn check_with_runner_for_account(
    runner: &impl CommandRunner,
    account_digest: Option<&str>,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    let executable = runner.qualify("codexbar")?;
    let version = runner.run(
        &executable,
        &["--version"],
        command_timeout(VERSION_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !version.success {
        return Err(SensorError::Failed);
    }
    validate_version(&version.stdout)?;
    let output = runner.run(
        &executable,
        USAGE_ARGUMENTS,
        command_timeout(USAGE_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !output.success {
        return Err(SensorError::Failed);
    }
    normalize_output_for_account(&output.stdout, account_digest, unix_seconds(now)?)
}

#[derive(Debug)]
pub(crate) enum AutomaticDispatchError {
    Sensor(SensorError),
    InvalidPolicy,
    Blocked(UsageObservation),
    Unknown(UsageObservation),
    Permit(UsagePermitError, UsageObservation),
}

#[derive(Debug)]
pub(crate) struct UsageGuardEvidence {
    pub(crate) digest: String,
    pub(crate) window: &'static str,
}

#[derive(Debug)]
pub(crate) struct UsageObservation {
    pub(crate) evidence: UsageGuardEvidence,
    pub(crate) snapshots: Vec<UsageSnapshot>,
}

#[derive(Debug)]
pub(crate) struct AuthorizedDispatch<T> {
    pub(crate) value: T,
    pub(crate) observation: UsageObservation,
}

pub(crate) fn qualify_and_dispatch_with_runner<T, C>(
    runner: &impl CommandRunner,
    account_digest: &str,
    threshold_percent: u8,
    previous_snapshots: &[UsageSnapshot],
    sampled_at: SystemTime,
    dispatch_clock: C,
    dispatch: impl FnOnce() -> T,
) -> Result<AuthorizedDispatch<T>, AutomaticDispatchError>
where
    C: FnOnce() -> Result<i64, SensorError>,
{
    let snapshot = check_with_runner(runner, account_digest, sampled_at)
        .map_err(AutomaticDispatchError::Sensor)?;
    let evidence = UsageGuardEvidence {
        digest: snapshot.evidence_digest(),
        window: snapshot
            .windows
            .first()
            .map_or("unknown", |window| window.name),
    };
    let observation = UsageObservation {
        evidence,
        snapshots: snapshot.core_snapshots(),
    };
    let policy = UsagePolicy::new("codexbar", CODEXBAR_VERSION, "codex", account_digest)
        .with_stop_remaining_percent(threshold_percent)
        .map_err(|_| AutomaticDispatchError::InvalidPolicy)?;
    let evaluated_at_unix_seconds =
        i64::try_from(unix_seconds(sampled_at).map_err(AutomaticDispatchError::Sensor)?)
            .map_err(|_| AutomaticDispatchError::Sensor(SensorError::Malformed))?;
    match evaluate_usage(
        &policy,
        &observation.snapshots,
        previous_snapshots,
        evaluated_at_unix_seconds,
    ) {
        UsageDecision::Allow(mut permit) => {
            let dispatch_at_unix_seconds =
                dispatch_clock().map_err(AutomaticDispatchError::Sensor)?;
            match consume_for_automatic_dispatch(&mut permit, dispatch_at_unix_seconds, dispatch) {
                Ok(value) => Ok(AuthorizedDispatch { value, observation }),
                Err(error) => Err(AutomaticDispatchError::Permit(error, observation)),
            }
        }
        UsageDecision::Block(_) => Err(AutomaticDispatchError::Blocked(observation)),
        UsageDecision::Unknown(_) => Err(AutomaticDispatchError::Unknown(observation)),
    }
}

fn consume_for_automatic_dispatch<T>(
    permit: &mut UsagePermit,
    dispatch_at_unix_seconds: i64,
    dispatch: impl FnOnce() -> T,
) -> Result<T, UsagePermitError> {
    permit.consume(dispatch_at_unix_seconds)?;
    Ok(dispatch())
}

fn command_timeout(default: Duration) -> Duration {
    if cfg!(debug_assertions) {
        if let Some(milliseconds) = std::env::var("HIVE_USAGE_TEST_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
        {
            return default.min(Duration::from_millis(milliseconds));
        }
    }
    default
}

fn validate_account_digest(value: &str) -> Result<(), SensorError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(SensorError::Malformed);
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SensorError::Malformed);
    }
    Ok(())
}

fn validate_version(stdout: &[u8]) -> Result<(), SensorError> {
    let version = std::str::from_utf8(stdout)
        .map_err(|_| SensorError::UnsupportedVersion)?
        .trim();
    let supported = version == CODEXBAR_VERSION
        || version
            .strip_prefix("CodexBar ")
            .is_some_and(|value| value == CODEXBAR_VERSION)
        || version
            .strip_prefix("codexbar ")
            .is_some_and(|value| value == CODEXBAR_VERSION);
    if supported {
        Ok(())
    } else {
        Err(SensorError::UnsupportedVersion)
    }
}

#[cfg(test)]
fn normalize_output(
    stdout: &[u8],
    account_digest: &str,
    now: u64,
) -> Result<NormalizedSnapshot, SensorError> {
    normalize_output_for_account(stdout, Some(account_digest), now)
}

fn normalize_output_for_account(
    stdout: &[u8],
    requested_account_digest: Option<&str>,
    now: u64,
) -> Result<NormalizedSnapshot, SensorError> {
    let rows: Vec<CodexBarRow> =
        serde_json::from_slice(stdout).map_err(|_| SensorError::Malformed)?;
    if rows.iter().any(|row| row.error.is_some()) {
        return Err(SensorError::RowError);
    }
    for row in &rows {
        validate_row_identity(row)?;
    }
    let (row, account_digest) = if let Some(account_digest) = requested_account_digest {
        let mut matches = rows
            .iter()
            .filter_map(|row| row.account.as_deref().map(|account| (row, account)))
            .filter(|(_, account)| sha256_digest(account.as_bytes()) == account_digest);
        let (row, _account) = matches.next().ok_or(SensorError::AccountNotFound)?;
        if matches.next().is_some() {
            return Err(SensorError::DuplicateAccount);
        }
        (row, account_digest.to_owned())
    } else {
        let mut accounts = rows
            .iter()
            .filter_map(|row| row.account.as_deref().map(|account| (row, account)));
        let (row, account) = accounts.next().ok_or(SensorError::AccountNotFound)?;
        if accounts.next().is_some() {
            return Err(SensorError::DuplicateAccount);
        }
        (row, sha256_digest(account.as_bytes()))
    };
    let usage = row.usage.as_ref().ok_or(SensorError::Malformed)?;
    let measured_at = parse_iso8601_z(&usage.updated_at)?;
    let expires_at = measured_at.saturating_add(60);
    if now > expires_at {
        return Err(SensorError::Stale);
    }
    if measured_at > now.saturating_add(60) {
        return Err(SensorError::Malformed);
    }
    let windows = if let Some(primary) = usage.primary.as_ref() {
        let primary: CodexBarWindow =
            serde_json::from_value(primary.clone()).map_err(|_| SensorError::Malformed)?;
        vec![normalize_window(&primary, "session", 300)?]
    } else {
        let [secondary] = usage.secondary.as_slice() else {
            return Err(SensorError::WrongWindows);
        };
        if secondary.is_null() {
            return Err(SensorError::WrongWindows);
        }
        let secondary: CodexBarWindow =
            serde_json::from_value(secondary.clone()).map_err(|_| SensorError::Malformed)?;
        vec![normalize_window(&secondary, "weekly", 10_080)?]
    };
    Ok(NormalizedSnapshot {
        sensor_id: "codexbar",
        sensor_version: CODEXBAR_VERSION,
        provider: "codex",
        account_digest,
        measured_at,
        expires_at,
        source_confidence: "local",
        windows,
    })
}

fn validate_row_identity(row: &CodexBarRow) -> Result<(), SensorError> {
    if row
        .account
        .as_deref()
        .is_none_or(|account| account.trim().is_empty())
    {
        return Err(SensorError::MissingIdentity);
    }
    if row.provider != "codex" {
        return Err(SensorError::WrongProvider);
    }
    if row.source != "codex-cli" {
        return Err(SensorError::NonLocalSource);
    }
    let usage = row.usage.as_ref().ok_or(SensorError::Malformed)?;
    let identity = usage
        .identity
        .as_ref()
        .ok_or(SensorError::MissingIdentity)?;
    if identity.provider_id != "codex" {
        return Err(SensorError::WrongProvider);
    }
    Ok(())
}

fn normalize_window(
    window: &CodexBarWindow,
    name: &'static str,
    expected_minutes: u32,
) -> Result<NormalizedWindow, SensorError> {
    if window.window_minutes != Some(expected_minutes) {
        return Err(SensorError::WrongWindows);
    }
    if !window.used_percent.is_finite() || !(0.0..=100.0).contains(&window.used_percent) {
        return Err(SensorError::Malformed);
    }
    Ok(NormalizedWindow {
        name,
        window_minutes: expected_minutes,
        remaining_percent: 100.0 - window.used_percent,
        resets_at: parse_iso8601_z(window.resets_at.as_deref().ok_or(SensorError::Malformed)?)?,
    })
}

fn unix_seconds(time: SystemTime) -> Result<u64, SensorError> {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| SensorError::Malformed)
}

fn parse_iso8601_z(value: &str) -> Result<u64, SensorError> {
    let (date, time) = value
        .strip_suffix('Z')
        .and_then(|value| value.split_once('T'))
        .ok_or(SensorError::Malformed)?;
    let mut date_parts = date.split('-');
    let year = parse_part(date_parts.next(), 4)?;
    let month = parse_part(date_parts.next(), 2)?;
    let day = parse_part(date_parts.next(), 2)?;
    if date_parts.next().is_some() {
        return Err(SensorError::Malformed);
    }
    let time = time.split_once('.').map_or(time, |(whole, _)| whole);
    let mut time_parts = time.split(':');
    let hour = parse_part(time_parts.next(), 2)?;
    let minute = parse_part(time_parts.next(), 2)?;
    let second = parse_part(time_parts.next(), 2)?;
    if time_parts.next().is_some()
        || !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(SensorError::Malformed);
    }
    let days = days_from_civil(year, month, day);
    if days < 0 {
        return Err(SensorError::Malformed);
    }
    let seconds = days
        .checked_mul(86_400)
        .and_then(|value| value.checked_add(i64::from(hour) * 3_600))
        .and_then(|value| value.checked_add(i64::from(minute) * 60))
        .and_then(|value| value.checked_add(i64::from(second)))
        .ok_or(SensorError::Malformed)?;
    u64::try_from(seconds).map_err(|_| SensorError::Malformed)
}

fn parse_part(value: Option<&str>, width: usize) -> Result<i32, SensorError> {
    let value = value.ok_or(SensorError::Malformed)?;
    if value.len() != width || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SensorError::Malformed);
    }
    value.parse().map_err(|_| SensorError::Malformed)
}

const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i32, month: i32, day: i32) -> i64 {
    let adjusted_year = year - i32::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    i64::from(era * 146_097 + day_of_era - 719_468)
}

#[cfg(test)]
mod tests {
    use super::{
        check_unique_with_runner, check_with_runner, consume_for_automatic_dispatch,
        normalize_output, parse_iso8601_z, qualify_and_dispatch_with_runner,
        resolve_program_in_path, AutomaticDispatchError, CommandOutput, CommandRunner,
        QualifiedExecutable, SensorError, USAGE_ARGUMENTS, USAGE_TIMEOUT, VERSION_TIMEOUT,
    };
    #[cfg(unix)]
    use super::{qualify_program, SystemCommandRunner};
    use hive_core::sha256_digest;
    use hive_core::usage_guard::{evaluate_usage, UsageDecision, UsagePermitError, UsagePolicy};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq)]
    struct Invocation {
        program: String,
        arguments: Vec<String>,
        timeout: Duration,
        output_limit: usize,
    }

    struct FakeRunner {
        responses: RefCell<VecDeque<Result<CommandOutput, SensorError>>>,
        invocations: RefCell<Vec<Invocation>>,
    }

    impl FakeRunner {
        fn new(responses: impl IntoIterator<Item = Result<CommandOutput, SensorError>>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().collect()),
                invocations: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            Ok(QualifiedExecutable::synthetic(program))
        }

        fn run(
            &self,
            program: &QualifiedExecutable,
            arguments: &[&str],
            timeout: Duration,
            output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            self.invocations.borrow_mut().push(Invocation {
                program: program.path.to_string_lossy().into_owned(),
                arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
                timeout,
                output_limit,
            });
            self.responses
                .borrow_mut()
                .pop_front()
                .expect("fake runner response should exist")
        }
    }

    #[cfg(unix)]
    struct PathSystemRunner<'a> {
        executable: &'a Path,
    }

    #[cfg(unix)]
    impl CommandRunner for PathSystemRunner<'_> {
        fn qualify(&self, _program: &str) -> Result<QualifiedExecutable, SensorError> {
            qualify_program(
                self.executable
                    .to_str()
                    .expect("fixture path should be UTF-8"),
            )
        }

        fn run(
            &self,
            program: &QualifiedExecutable,
            arguments: &[&str],
            timeout: Duration,
            output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            SystemCommandRunner.run(program, arguments, timeout, output_limit)
        }
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .expect("fixture metadata should exist")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(path, permissions).expect("fixture should become executable");
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) {}

    fn success(stdout: impl Into<Vec<u8>>) -> CommandOutput {
        CommandOutput {
            success: true,
            stdout: stdout.into(),
        }
    }

    #[test]
    fn resolves_the_path_entry_before_spawning_the_sensor() {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let executable = directory.path().join("codexbar");
        fs::write(&executable, b"fixture").expect("fixture executable should be created");
        make_executable(&executable);
        let search_path = OsString::from(directory.path());

        assert_eq!(
            resolve_program_in_path("codexbar", &search_path),
            Some(
                executable
                    .canonicalize()
                    .expect("fixture executable should resolve")
            )
        );
    }

    #[cfg(windows)]
    #[test]
    fn resolves_a_windows_command_wrapper_from_path() {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let wrapper = directory.path().join("codexbar.cmd");
        fs::write(&wrapper, b"@exit /b 0\r\n").expect("fixture wrapper should be created");
        let search_path = OsString::from(directory.path());

        assert_eq!(
            resolve_program_in_path("codexbar", &search_path),
            Some(
                wrapper
                    .canonicalize()
                    .expect("fixture wrapper should resolve")
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_a_path_symlink_to_the_installed_sensor_executable() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let installed = directory.path().join("CodexBarCLI");
        let linked = directory.path().join("codexbar");
        fs::write(&installed, b"fixture").expect("fixture executable should be created");
        make_executable(&installed);
        symlink(&installed, &linked).expect("fixture symlink should be created");
        let search_path = OsString::from(directory.path());

        assert_eq!(
            resolve_program_in_path("codexbar", &search_path),
            Some(
                installed
                    .canonicalize()
                    .expect("installed fixture should resolve")
            )
        );
    }

    fn account() -> &'static str {
        "local-account-label"
    }

    fn account_digest() -> String {
        sha256_digest(account().as_bytes())
    }

    fn row(
        source: &str,
        updated_at: &str,
        primary_minutes: u32,
        secondary_minutes: u32,
        primary_used: u8,
        secondary_used: u8,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!([{
            "provider": "codex",
            "account": account(),
            "source": source,
            "usage": {
                "primary": {
                    "usedPercent": primary_used,
                    "windowMinutes": primary_minutes,
                    "resetsAt": "2026-07-24T00:00:00Z"
                },
                "secondary": {
                    "usedPercent": secondary_used,
                    "windowMinutes": secondary_minutes,
                    "resetsAt": "2026-07-30T00:00:00Z"
                },
                "updatedAt": updated_at,
                "identity": {"providerID": "codex"}
            },
            "error": null
        }]))
        .expect("fixture should serialize")
    }

    fn now(value: &str) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(parse_iso8601_z(value).expect("fixture time should parse"))
    }

    #[test]
    fn invokes_only_the_pinned_fixed_codexbar_commands() {
        let runner = FakeRunner::new([
            Ok(success("CodexBar 0.45.2\n")),
            Ok(success(row(
                "codex-cli",
                "2026-07-23T12:00:00Z",
                300,
                10_080,
                20,
                30,
            ))),
        ]);
        check_with_runner(&runner, &account_digest(), now("2026-07-23T12:00:30Z"))
            .expect("valid sensor response should normalize");
        let invocations = runner.invocations.borrow();
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].program, "codexbar");
        assert_eq!(invocations[0].arguments, ["--version"]);
        assert_eq!(invocations[0].timeout, VERSION_TIMEOUT);
        assert_eq!(
            invocations[1].arguments,
            USAGE_ARGUMENTS
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        );
        assert_eq!(invocations[1].timeout, USAGE_TIMEOUT);
    }

    #[test]
    fn unique_account_selection_derives_only_the_account_digest() {
        let runner = FakeRunner::new([
            Ok(success("CodexBar 0.45.2\n")),
            Ok(success(row(
                "codex-cli",
                "2026-07-23T12:00:00Z",
                300,
                10_080,
                20,
                30,
            ))),
        ]);

        let snapshot = check_unique_with_runner(&runner, now("2026-07-23T12:00:30Z"))
            .expect("one qualified local account should be selected");

        assert_eq!(snapshot.account_digest, account_digest());
    }

    #[test]
    fn unique_account_selection_rejects_multiple_rows() {
        let one_row: serde_json::Value = serde_json::from_slice(&row(
            "codex-cli",
            "2026-07-23T12:00:00Z",
            300,
            10_080,
            20,
            30,
        ))
        .expect("fixture row should parse");
        let row = one_row
            .as_array()
            .and_then(|rows| rows.first())
            .expect("fixture should contain one row")
            .clone();
        let duplicate = serde_json::to_vec(&serde_json::json!([row, row]))
            .expect("duplicate fixture should encode");
        let runner = FakeRunner::new([Ok(success("CodexBar 0.45.2\n")), Ok(success(duplicate))]);

        assert_eq!(
            check_unique_with_runner(&runner, now("2026-07-23T12:00:30Z")),
            Err(SensorError::DuplicateAccount)
        );
    }

    #[cfg(unix)]
    #[test]
    fn executable_mutation_between_qualification_commands_fails_closed() {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let executable = directory.path().join("codexbar");
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf '#!/bin/sh\\nexit 9\\n' > '{}'\n  chmod 700 '{}'\n  printf 'CodexBar 0.45.2\\n'\n  exit 0\nfi\nexit 9\n",
            executable.display(),
            executable.display()
        );
        fs::write(&executable, script).expect("fixture executable should be created");
        make_executable(&executable);
        let runner = PathSystemRunner {
            executable: &executable,
        };
        let result = check_with_runner(&runner, &account_digest(), UNIX_EPOCH);

        assert_eq!(result, Err(SensorError::ExecutableChanged));
    }

    #[test]
    fn missing_sensor_fails_closed_without_a_retry() {
        let runner = FakeRunner::new([Err(SensorError::Unavailable)]);
        assert_eq!(
            check_with_runner(&runner, &account_digest(), UNIX_EPOCH),
            Err(SensorError::Unavailable)
        );
        assert_eq!(runner.invocations.borrow().len(), 1);
    }

    #[test]
    fn unsupported_version_stops_before_usage_collection() {
        let runner = FakeRunner::new([Ok(success("CodexBar 0.45.3\n"))]);
        assert_eq!(
            check_with_runner(&runner, &account_digest(), UNIX_EPOCH),
            Err(SensorError::UnsupportedVersion)
        );
        assert_eq!(runner.invocations.borrow().len(), 1);
    }

    #[test]
    fn timeout_fails_closed_without_a_retry() {
        let runner = FakeRunner::new([Err(SensorError::Timeout)]);
        assert_eq!(
            check_with_runner(&runner, &account_digest(), UNIX_EPOCH),
            Err(SensorError::Timeout)
        );
        assert_eq!(runner.invocations.borrow().len(), 1);
    }

    #[test]
    fn malformed_json_is_rejected() {
        assert_eq!(
            normalize_output(b"{", &account_digest(), 0),
            Err(SensorError::Malformed)
        );
    }

    #[test]
    fn any_account_row_error_is_rejected() {
        let bytes = serde_json::to_vec(&serde_json::json!([
            {
                "provider": "codex",
                "account": account(),
                "source": "codex-cli",
                "usage": null,
                "error": {"code": 1, "message": "failed"}
            }
        ]))
        .expect("fixture should serialize");
        assert_eq!(
            normalize_output(&bytes, &account_digest(), 0),
            Err(SensorError::RowError)
        );
    }

    #[test]
    fn missing_and_wrong_accounts_are_rejected_without_exposure() {
        let missing = serde_json::to_vec(&serde_json::json!([{
            "provider": "codex",
            "source": "codex-cli",
            "usage": null,
            "error": null
        }]))
        .expect("fixture should serialize");
        assert_eq!(
            normalize_output(&missing, &account_digest(), 0),
            Err(SensorError::MissingIdentity)
        );
        let valid = row("codex-cli", "2026-07-23T12:00:00Z", 300, 10_080, 20, 30);
        assert_eq!(
            normalize_output(&valid, &sha256_digest(b"another-account"), 0),
            Err(SensorError::AccountNotFound)
        );
    }

    #[test]
    fn non_local_source_is_rejected() {
        let bytes = row("openai-web", "2026-07-23T12:00:00Z", 300, 10_080, 20, 30);
        assert_eq!(
            normalize_output(
                &bytes,
                &account_digest(),
                parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
            ),
            Err(SensorError::NonLocalSource)
        );
    }

    #[test]
    fn stale_snapshot_is_rejected_at_the_sixty_second_boundary() {
        let bytes = row("codex-cli", "2026-07-23T12:00:00Z", 300, 10_080, 20, 30);
        let now = parse_iso8601_z("2026-07-23T12:01:01Z").expect("fixture should parse");
        assert_eq!(
            normalize_output(&bytes, &account_digest(), now),
            Err(SensorError::Stale)
        );
    }

    #[test]
    fn unexpected_windows_are_rejected() {
        let bytes = row("codex-cli", "2026-07-23T12:00:00Z", 301, 10_080, 20, 30);
        assert_eq!(
            normalize_output(
                &bytes,
                &account_digest(),
                parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
            ),
            Err(SensorError::WrongWindows)
        );
    }

    #[test]
    fn session_window_ignores_malformed_duplicate_and_low_weekly_data() {
        let valid = String::from_utf8(row(
            "codex-cli",
            "2026-07-23T12:00:00Z",
            300,
            10_080,
            20,
            99,
        ))
        .expect("fixture should be UTF-8");
        let hostile = valid.replacen(
            "\"secondary\":{\"resetsAt\"",
            "\"secondary\":\"malformed\",\"secondary\":{\"resetsAt\"",
            1,
        );
        let snapshot = normalize_output(
            hostile.as_bytes(),
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("session must take absolute precedence");

        assert_eq!(snapshot.windows.len(), 1);
        assert_eq!(snapshot.windows[0].name, "session");
        assert!((snapshot.windows[0].remaining_percent - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn duplicate_weekly_window_is_unknown_when_session_is_absent() {
        let mut value: serde_json::Value = serde_json::from_slice(&row(
            "codex-cli",
            "2026-07-23T12:00:00Z",
            300,
            10_080,
            20,
            30,
        ))
        .expect("fixture should parse");
        value[0]["usage"]["primary"] = serde_json::Value::Null;
        let valid = serde_json::to_string(&value).expect("fixture should serialize");
        let hostile = valid.replacen(
            "\"secondary\":",
            "\"secondary\":\"duplicate\",\"secondary\":",
            1,
        );

        assert_eq!(
            normalize_output(
                hostile.as_bytes(),
                &account_digest(),
                parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
            ),
            Err(SensorError::WrongWindows)
        );
    }

    #[test]
    fn normalized_windows_drive_allow_and_threshold_block_decisions() {
        let allowed = normalize_output(
            &row("codex-cli", "2026-07-23T12:00:00Z", 300, 10_080, 20, 30),
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("fixture should normalize");
        let policy = UsagePolicy::new("codexbar", "0.45.2", "codex", account_digest())
            .with_stop_remaining_percent(10)
            .expect("threshold should be valid");
        assert!(matches!(
            evaluate_usage(
                &policy,
                &allowed.core_snapshots(),
                &[],
                i64::try_from(
                    parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
                )
                .expect("fixture should fit i64")
            ),
            UsageDecision::Allow(_)
        ));

        let blocked = normalize_output(
            &row("codex-cli", "2026-07-23T12:00:00Z", 300, 10_080, 90, 30),
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("fixture should normalize");
        assert!(matches!(
            evaluate_usage(
                &policy,
                &blocked.core_snapshots(),
                &[],
                i64::try_from(
                    parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
                )
                .expect("fixture should fit i64")
            ),
            UsageDecision::Block(_)
        ));
    }

    #[test]
    fn fractional_percentages_preserve_the_inclusive_boundary() {
        let mut value: serde_json::Value = serde_json::from_slice(&row(
            "codex-cli",
            "2026-07-23T12:00:00Z",
            300,
            10_080,
            89,
            30,
        ))
        .expect("fixture should parse");
        value[0]["usage"]["primary"]["usedPercent"] = serde_json::json!(89.99);
        let bytes = serde_json::to_vec(&value).expect("fixture should serialize");
        let snapshot = normalize_output(
            &bytes,
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("fractional percentage should normalize");
        let policy = UsagePolicy::new("codexbar", "0.45.2", "codex", account_digest())
            .with_stop_remaining_percent(10)
            .expect("threshold should be valid");

        assert!(matches!(
            evaluate_usage(
                &policy,
                &snapshot.core_snapshots(),
                &[],
                i64::try_from(
                    parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
                )
                .expect("fixture should fit i64")
            ),
            UsageDecision::Allow(_)
        ));
    }

    #[test]
    fn dispatch_adapter_consumes_immediately_before_exactly_one_dispatch() {
        let runner = FakeRunner::new([
            Ok(success("CodexBar 0.45.2\n")),
            Ok(success(row(
                "codex-cli",
                "2026-07-23T12:00:00Z",
                300,
                10_080,
                20,
                99,
            ))),
        ]);
        let dispatch_at =
            i64::try_from(parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"))
                .expect("fixture should fit i64");
        let dispatches = RefCell::new(0_u8);
        let result = qualify_and_dispatch_with_runner(
            &runner,
            &account_digest(),
            10,
            &[],
            now("2026-07-23T12:00:30Z"),
            || Ok(dispatch_at),
            || {
                *dispatches.borrow_mut() += 1;
                "represented-dispatch"
            },
        );

        let authorized = result.expect("dispatch should be authorized");
        assert_eq!(authorized.value, "represented-dispatch");
        assert_eq!(authorized.observation.evidence.window, "session");
        assert!(authorized
            .observation
            .evidence
            .digest
            .starts_with("sha256:"));
        assert_eq!(*dispatches.borrow(), 1);
    }

    #[test]
    fn dispatch_adapter_never_invokes_the_dispatch_when_usage_is_blocked() {
        let runner = FakeRunner::new([
            Ok(success("CodexBar 0.45.2\n")),
            Ok(success(row(
                "codex-cli",
                "2026-07-23T12:00:00Z",
                300,
                10_080,
                90,
                20,
            ))),
        ]);
        let dispatches = RefCell::new(0_u8);
        let result = qualify_and_dispatch_with_runner(
            &runner,
            &account_digest(),
            10,
            &[],
            now("2026-07-23T12:00:30Z"),
            || {
                Ok(i64::try_from(
                    parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
                )
                .expect("fixture should fit i64"))
            },
            || *dispatches.borrow_mut() += 1,
        );

        assert!(matches!(result, Err(AutomaticDispatchError::Blocked(_))));
        assert_eq!(*dispatches.borrow(), 0);
    }

    #[test]
    fn dispatch_adapter_uses_trustworthy_history_to_reject_same_reset_increase() {
        let previous = normalize_output(
            &row("codex-cli", "2026-07-23T12:00:10Z", 300, 10_080, 43, 43),
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:20Z").expect("fixture should parse"),
        )
        .expect("previous snapshot")
        .core_snapshots();
        let runner = FakeRunner::new([
            Ok(success("CodexBar 0.45.2\n")),
            Ok(success(row(
                "codex-cli",
                "2026-07-23T12:00:20Z",
                300,
                10_080,
                30,
                43,
            ))),
        ]);
        let dispatches = RefCell::new(0_u8);
        let result = qualify_and_dispatch_with_runner(
            &runner,
            &account_digest(),
            10,
            &previous,
            now("2026-07-23T12:00:30Z"),
            || Ok(1_753_275_630),
            || *dispatches.borrow_mut() += 1,
        );

        assert!(matches!(result, Err(AutomaticDispatchError::Unknown(_))));
        assert_eq!(*dispatches.borrow(), 0);
    }

    #[test]
    fn dispatch_adapter_never_claims_enforcement_or_dispatches_when_sensor_is_unknown() {
        let runner = FakeRunner::new([Err(SensorError::Unavailable)]);
        let dispatches = RefCell::new(0_u8);
        let result = qualify_and_dispatch_with_runner(
            &runner,
            &account_digest(),
            10,
            &[],
            now("2026-07-23T12:00:30Z"),
            || {
                Ok(i64::try_from(
                    parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
                )
                .expect("fixture should fit i64"))
            },
            || *dispatches.borrow_mut() += 1,
        );

        assert!(matches!(
            result,
            Err(AutomaticDispatchError::Sensor(SensorError::Unavailable))
        ));
        assert_eq!(*dispatches.borrow(), 0);
    }

    #[test]
    fn dispatch_adapter_rejects_expiry_without_invoking_the_dispatch() {
        let runner = FakeRunner::new([
            Ok(success("CodexBar 0.45.2\n")),
            Ok(success(row(
                "codex-cli",
                "2026-07-23T12:00:00Z",
                300,
                10_080,
                20,
                30,
            ))),
        ]);
        let dispatches = RefCell::new(0_u8);
        let result = qualify_and_dispatch_with_runner(
            &runner,
            &account_digest(),
            10,
            &[],
            now("2026-07-23T12:00:30Z"),
            || {
                Ok(i64::try_from(
                    parse_iso8601_z("2026-07-23T12:01:00Z").expect("fixture should parse"),
                )
                .expect("fixture should fit i64"))
            },
            || *dispatches.borrow_mut() += 1,
        );

        assert!(matches!(
            result,
            Err(AutomaticDispatchError::Permit(
                UsagePermitError::Expired {
                    expires_at_unix_seconds: _,
                    attempted_at_unix_seconds: _
                },
                _
            ))
        ));
        assert_eq!(*dispatches.borrow(), 0);
    }

    #[test]
    fn consumed_permit_cannot_represent_a_second_dispatch() {
        let snapshot = normalize_output(
            &row("codex-cli", "2026-07-23T12:00:00Z", 300, 10_080, 20, 30),
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("fixture should normalize");
        let policy = UsagePolicy::new("codexbar", "0.45.2", "codex", account_digest());
        let dispatch_at =
            i64::try_from(parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"))
                .expect("fixture should fit i64");
        let UsageDecision::Allow(mut permit) =
            evaluate_usage(&policy, &snapshot.core_snapshots(), &[], dispatch_at)
        else {
            panic!("fixture should issue a permit");
        };
        let dispatches = RefCell::new(0_u8);
        assert_eq!(
            consume_for_automatic_dispatch(&mut permit, dispatch_at, || {
                *dispatches.borrow_mut() += 1;
            }),
            Ok(())
        );
        assert_eq!(
            consume_for_automatic_dispatch(&mut permit, dispatch_at, || {
                *dispatches.borrow_mut() += 1;
            }),
            Err(UsagePermitError::AlreadyConsumed)
        );
        assert_eq!(*dispatches.borrow(), 1);
    }

    #[test]
    fn iso8601_z_parser_rejects_offsets_and_impossible_dates() {
        assert!(parse_iso8601_z("2026-07-23T12:00:00Z").is_ok());
        assert!(parse_iso8601_z("2026-07-23T12:00:00+00:00").is_err());
        assert!(parse_iso8601_z("2026-02-29T12:00:00Z").is_err());
        assert!(parse_iso8601_z("2024-02-29T12:00:00Z").is_ok());
    }
}
