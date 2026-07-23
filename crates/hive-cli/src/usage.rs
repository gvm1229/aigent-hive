use hive_core::sha256_digest;
use hive_core::usage_guard::{SourceConfidence, UsageSnapshot, UsageWindow};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
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
        })
    }
}

#[derive(Debug)]
pub(crate) struct CommandOutput {
    success: bool,
    stdout: Vec<u8>,
}

pub(crate) trait CommandRunner {
    fn run(
        &self,
        program: &str,
        arguments: &[&str],
        timeout: Duration,
        output_limit: usize,
    ) -> Result<CommandOutput, SensorError>;
}

pub(crate) struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(
        &self,
        program: &str,
        arguments: &[&str],
        timeout: Duration,
        output_limit: usize,
    ) -> Result<CommandOutput, SensorError> {
        let resolved_program = resolve_program(program)?;
        let mut child = Command::new(resolved_program)
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
                    return Err(SensorError::Timeout);
                }
                None => thread::sleep(Duration::from_millis(10)),
            }
        };
        let stdout = receive_output(&stdout_reader, started, timeout)?;
        let _stderr = receive_output(&stderr_reader, started, timeout)?;
        Ok(CommandOutput {
            success: status.success(),
            stdout,
        })
    }
}

fn resolve_program(program: &str) -> Result<PathBuf, SensorError> {
    let path = Path::new(program);
    if path.components().count() > 1 {
        return resolve_executable(path).ok_or(SensorError::Unavailable);
    }
    let search_path = std::env::var_os("PATH").ok_or(SensorError::Unavailable)?;
    resolve_program_in_path(program, &search_path).ok_or(SensorError::Unavailable)
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
    candidate.is_file().then(|| {
        candidate
            .canonicalize()
            .unwrap_or_else(|_| candidate.to_path_buf())
    })
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

#[derive(Deserialize)]
struct CodexBarUsage {
    primary: Option<CodexBarWindow>,
    secondary: Option<CodexBarWindow>,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    identity: Option<CodexBarIdentity>,
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
    let version = runner.run(
        "codexbar",
        &["--version"],
        command_timeout(VERSION_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !version.success {
        return Err(SensorError::Failed);
    }
    validate_version(&version.stdout)?;
    let output = runner.run(
        "codexbar",
        USAGE_ARGUMENTS,
        command_timeout(USAGE_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !output.success {
        return Err(SensorError::Failed);
    }
    normalize_output(&output.stdout, account_digest, unix_seconds(now)?)
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

fn normalize_output(
    stdout: &[u8],
    account_digest: &str,
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
    let mut matches = rows
        .iter()
        .filter_map(|row| row.account.as_deref().map(|account| (row, account)))
        .filter(|(_, account)| sha256_digest(account.as_bytes()) == account_digest);
    let (row, _account) = matches.next().ok_or(SensorError::AccountNotFound)?;
    if matches.next().is_some() {
        return Err(SensorError::DuplicateAccount);
    }
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
        vec![normalize_window(primary, "session", 300)?]
    } else {
        vec![normalize_window(
            usage.secondary.as_ref().ok_or(SensorError::WrongWindows)?,
            "weekly",
            10_080,
        )?]
    };
    Ok(NormalizedSnapshot {
        sensor_id: "codexbar",
        sensor_version: CODEXBAR_VERSION,
        provider: "codex",
        account_digest: account_digest.to_owned(),
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
        check_with_runner, normalize_output, parse_iso8601_z, resolve_program_in_path,
        CommandOutput, CommandRunner, SensorError, USAGE_ARGUMENTS, USAGE_TIMEOUT, VERSION_TIMEOUT,
    };
    use hive_core::sha256_digest;
    use hive_core::usage_guard::{evaluate_usage, UsageDecision, UsagePolicy};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::fs;
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
        fn run(
            &self,
            program: &str,
            arguments: &[&str],
            timeout: Duration,
            output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            self.invocations.borrow_mut().push(Invocation {
                program: program.to_owned(),
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
    fn iso8601_z_parser_rejects_offsets_and_impossible_dates() {
        assert!(parse_iso8601_z("2026-07-23T12:00:00Z").is_ok());
        assert!(parse_iso8601_z("2026-07-23T12:00:00+00:00").is_err());
        assert!(parse_iso8601_z("2026-02-29T12:00:00Z").is_err());
        assert!(parse_iso8601_z("2024-02-29T12:00:00Z").is_ok());
    }
}
