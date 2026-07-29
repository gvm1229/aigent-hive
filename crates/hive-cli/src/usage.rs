use hive_core::sha256_digest;
use hive_core::usage_guard::{
    evaluate_usage, SourceConfidence, UsageDecision, UsagePermit, UsagePermitError, UsagePolicy,
    UsageSnapshot, UsageWindow,
};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod codex_native;

const CODEXBAR_VERSION: &str = "0.45.2";
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const USAGE_TIMEOUT: Duration = Duration::from_mins(1);
const OUTPUT_LIMIT: usize = 1024 * 1024;
const CODEX_USAGE_ARGUMENTS: &[&str] = &[
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
const CLAUDE_USAGE_ARGUMENTS: &[&str] = &[
    "usage",
    "--provider",
    "claude",
    "--all-accounts",
    "--source",
    "cli",
    "--format",
    "json",
    "--json-only",
];
const ANTIGRAVITY_USAGE_ARGUMENTS: &[&str] = &[
    "usage",
    "--provider",
    "antigravity",
    "--source",
    "cli",
    "--format",
    "json",
    "--json-only",
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SensorError {
    Unavailable,
    Unsupported,
    Timeout,
    OutputTooLarge,
    Failed,
    UnsupportedVersion,
    Malformed,
    ClockInvalid,
    FilesystemSafety,
    WrongSession,
    DuplicateData,
    AmbiguousData,
    RowError,
    MissingIdentity,
    AccountNotFound,
    DuplicateAccount,
    WrongProvider,
    NonLocalSource,
    WrongWindows,
    Stale,
    ExecutableChanged,
    FallbackRequired(UsageHost),
}

impl Display for SensorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unavailable => "native usage sensor is unavailable",
            Self::Unsupported => "native usage sensor is unsupported",
            Self::Timeout => "usage sensor timed out",
            Self::OutputTooLarge => "usage sensor exceeded its output limit",
            Self::Failed => "usage sensor process failed",
            Self::UnsupportedVersion => "usage sensor version is unsupported",
            Self::Malformed => "usage sensor returned malformed protocol data",
            Self::ClockInvalid => "usage sensor clock is invalid",
            Self::FilesystemSafety => "usage sensor filesystem safety check failed",
            Self::WrongSession => "usage sensor returned the wrong session",
            Self::DuplicateData => "usage sensor returned duplicate data",
            Self::AmbiguousData => "usage sensor returned ambiguous data",
            Self::RowError => "usage sensor returned an account error",
            Self::MissingIdentity => "usage sensor omitted account identity",
            Self::AccountNotFound => "requested account digest was not found",
            Self::DuplicateAccount => "requested account digest matched more than one account",
            Self::WrongProvider => "usage sensor returned the wrong provider",
            Self::NonLocalSource => "usage sensor returned a non-local source",
            Self::WrongWindows => "usage sensor returned unexpected quota windows",
            Self::Stale => "usage sensor returned a stale snapshot",
            Self::ExecutableChanged => "usage sensor executable changed during qualification",
            Self::FallbackRequired(host) => {
                return write!(
                    formatter,
                    "native {} usage is unavailable and the optional CodexBar fallback is not installed",
                    host.as_str()
                );
            }
        })
    }
}

impl SensorError {
    const fn allows_native_fallback(&self) -> bool {
        matches!(
            self,
            Self::Unavailable | Self::Unsupported | Self::UnsupportedVersion | Self::Malformed
        )
    }
}

struct StrictJsonValue(Value);

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonValueVisitor)
    }
}

struct StrictJsonValueVisitor;

impl<'de> Visitor<'de> for StrictJsonValueVisitor {
    type Value = StrictJsonValue;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(StrictJsonValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        StrictJsonValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictJsonValue>()? {
            values.push(value.0);
        }
        Ok(StrictJsonValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = map.next_entry::<String, StrictJsonValue>()? {
            if values.insert(key.clone(), value.0).is_some() {
                return Err(de::Error::custom(format_args!(
                    "duplicate JSON object key: {key}"
                )));
            }
        }
        Ok(StrictJsonValue(Value::Object(values)))
    }
}

pub(crate) fn parse_strict_native_json(bytes: &[u8]) -> Result<Value, SensorError> {
    serde_json::from_slice::<StrictJsonValue>(bytes)
        .map(|strict| strict.0)
        .map_err(|error| {
            if error.to_string().starts_with("duplicate JSON object key:") {
                SensorError::DuplicateData
            } else {
                SensorError::Malformed
            }
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct QualifiedExecutable {
    path: PathBuf,
    identity: Option<ExecutableIdentity>,
}

impl QualifiedExecutable {
    #[cfg(test)]
    pub(crate) fn synthetic(program: &str) -> Self {
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
    pub(crate) success: bool,
    pub(crate) stdout: Vec<u8>,
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

pub(crate) trait NativeUsageRunner {
    fn read_codex_native(
        &self,
        account_digest: Option<&str>,
        now: SystemTime,
    ) -> Result<NormalizedSnapshot, SensorError>;
}

pub(crate) struct SystemCommandRunner;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UsageHost {
    Codex,
    Claude,
    Antigravity,
}

impl UsageHost {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Antigravity => "antigravity",
        }
    }

    const fn local_source(self) -> &'static str {
        match self {
            Self::Codex => "codex-cli",
            Self::Claude => "claude-cli",
            Self::Antigravity => "antigravity-cli",
        }
    }
}

pub(crate) fn fallback_install_next_action(host: UsageHost) -> String {
    format!(
        "review `hive usage fallback-install --host {} --dry-run --output json`; install only with explicit `--apply --confirm-install` consent",
        host.as_str()
    )
}

impl NativeUsageRunner for SystemCommandRunner {
    fn read_codex_native(
        &self,
        account_digest: Option<&str>,
        now: SystemTime,
    ) -> Result<NormalizedSnapshot, SensorError> {
        codex_native::read(account_digest, now)
    }
}

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
    #[cfg(windows)]
    {
        let directories = std::env::split_paths(search_path).collect::<Vec<_>>();
        for directory in &directories {
            if program == "codex" {
                let package_paths = [
                    "node_modules/@openai/codex/node_modules/@openai/codex-win32-x64/vendor/x86_64-pc-windows-msvc/bin/codex.exe",
                    "node_modules/@openai/codex/vendor/x86_64-pc-windows-msvc/bin/codex.exe",
                ];
                for package_path in package_paths {
                    if let Some(executable) = resolve_executable(&directory.join(package_path)) {
                        return Some(executable);
                    }
                }
            }
            for extension in ["exe", "", "cmd", "bat"] {
                let candidate = if extension.is_empty() {
                    directory.join(program)
                } else {
                    directory.join(format!("{program}.{extension}"))
                };
                if let Some(executable) = resolve_executable(&candidate) {
                    return Some(executable);
                }
            }
        }
        None
    }

    #[cfg(not(windows))]
    std::env::split_paths(search_path).find_map(|directory| {
        let candidate = directory.join(program);
        resolve_executable(&candidate)
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
    Ok(metadata_identity(&metadata))
}

fn metadata_identity(metadata: &Metadata) -> ExecutableIdentity {
    let modified = metadata.modified().ok().and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| (duration.as_secs(), duration.subsec_nanos()))
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        ExecutableIdentity {
            len: metadata.len(),
            modified,
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            change_time: (metadata.ctime(), metadata.ctime_nsec()),
        }
    }
    #[cfg(not(unix))]
    {
        ExecutableIdentity {
            len: metadata.len(),
            modified,
        }
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
    pub(crate) quota_pool: Option<&'static str>,
    pub(crate) window_minutes: Option<u32>,
    pub(crate) remaining_percent: f64,
    pub(crate) resets_at: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct NormalizedSnapshot {
    pub(crate) sensor_id: String,
    pub(crate) sensor_version: String,
    pub(crate) provider: String,
    pub(crate) account_digest: String,
    pub(crate) measured_at: u64,
    pub(crate) expires_at: u64,
    pub(crate) source_confidence: String,
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
                schema_version: if window.quota_pool.is_some() { 2 } else { 1 },
                sensor_id: self.sensor_id.clone(),
                sensor_version: self.sensor_version.clone(),
                host_scope: self.provider.clone(),
                account_scope_digest: self.account_digest.clone(),
                quota_pool: window.quota_pool.map(str::to_owned),
                quota_window: match window.name {
                    "session" => UsageWindow::Session,
                    "weekly" => UsageWindow::Weekly,
                    "provider" => UsageWindow::Provider,
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

    pub(crate) fn selected_window_label(&self) -> &'static str {
        let Some(first) = self.windows.first().map(|window| window.name) else {
            return "unknown";
        };
        if self.windows.len() == 1 {
            first
        } else {
            "multiple"
        }
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
    account_email: Option<String>,
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
                let mut account_email = None;
                let mut account_email_seen = false;
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
                        "accountEmail" => {
                            if account_email_seen {
                                return Err(de::Error::duplicate_field("accountEmail"));
                            }
                            account_email_seen = true;
                            account_email = map.next_value()?;
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
                    account_email,
                })
            }
        }

        deserializer.deserialize_map(UsageVisitor)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CodexBarWindowMinutes {
    #[default]
    Missing,
    Null,
    Value(u32),
}

impl<'de> Deserialize<'de> for CodexBarWindowMinutes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(Self::Null),
            Value::Number(value) => value
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .map(Self::Value)
                .ok_or_else(|| {
                    de::Error::custom("windowMinutes must be an unsigned 32-bit integer")
                }),
            _ => Err(de::Error::custom(
                "windowMinutes must be an unsigned 32-bit integer",
            )),
        }
    }
}

#[derive(Deserialize)]
struct CodexBarWindow {
    #[serde(rename = "usedPercent")]
    used_percent: f64,
    #[serde(default, rename = "windowMinutes")]
    window_minutes: CodexBarWindowMinutes,
    #[serde(rename = "resetsAt")]
    resets_at: Option<String>,
}

#[derive(Deserialize)]
struct CodexBarIdentity {
    #[serde(rename = "providerID")]
    provider_id: String,
    #[serde(rename = "accountEmail")]
    account_email: Option<String>,
}

#[cfg(test)]
pub(crate) fn check_with_runner(
    runner: &impl CommandRunner,
    account_digest: &str,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    validate_account_digest(account_digest)?;
    check_with_runner_for_account(runner, Some(account_digest), now)
}

#[cfg(test)]
pub(crate) fn check_unique_with_runner(
    runner: &impl CommandRunner,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    check_with_runner_for_account(runner, None, now)
}

pub(crate) fn check_preferred_with_runners(
    native: &impl NativeUsageRunner,
    fallback: &impl CommandRunner,
    account_digest: &str,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    validate_account_digest(account_digest)?;
    check_preferred_for_account(native, fallback, Some(account_digest), now)
}

pub(crate) fn check_preferred_unique_with_runners(
    native: &impl NativeUsageRunner,
    fallback: &impl CommandRunner,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    check_preferred_for_account(native, fallback, None, now)
}

fn check_preferred_for_account(
    native: &impl NativeUsageRunner,
    fallback: &impl CommandRunner,
    account_digest: Option<&str>,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    native_then_fallback(
        UsageHost::Codex,
        native.read_codex_native(account_digest, now),
        || check_with_runner_for_account(fallback, account_digest, now),
    )
}

pub(crate) fn native_then_fallback<T>(
    host: UsageHost,
    native: Result<T, SensorError>,
    fallback: impl FnOnce() -> Result<T, SensorError>,
) -> Result<T, SensorError> {
    match native {
        Ok(value) => Ok(value),
        Err(error) if error.allows_native_fallback() => fallback().map_err(|fallback_error| {
            if fallback_error == SensorError::Unavailable {
                SensorError::FallbackRequired(host)
            } else {
                fallback_error
            }
        }),
        Err(error) => Err(error),
    }
}

fn check_with_runner_for_account(
    runner: &impl CommandRunner,
    account_digest: Option<&str>,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    check_codexbar_with_runner_for_account(runner, UsageHost::Codex, account_digest, now)
}

pub(crate) fn check_codexbar_provider_with_runner(
    runner: &impl CommandRunner,
    host: UsageHost,
    account_digest: &str,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    validate_account_digest(account_digest)?;
    check_codexbar_with_runner_for_account(runner, host, Some(account_digest), now)
}

pub(crate) fn check_codexbar_provider_unique_with_runner(
    runner: &impl CommandRunner,
    host: UsageHost,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    check_codexbar_with_runner_for_account(runner, host, None, now)
}

fn check_codexbar_with_runner_for_account(
    runner: &impl CommandRunner,
    host: UsageHost,
    account_digest: Option<&str>,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    let executable = runner.qualify("codexbar").map_err(|error| {
        if error == SensorError::Unavailable {
            SensorError::FallbackRequired(host)
        } else {
            error
        }
    })?;
    let version = runner.run(
        &executable,
        &["--version"],
        command_timeout(VERSION_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !version.success {
        return Err(SensorError::Failed);
    }
    validate_version_for_executable(&executable, &version.stdout)?;
    let arguments = usage_arguments(host);
    let output = runner.run(
        &executable,
        arguments,
        command_timeout(USAGE_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !output.success {
        return Err(SensorError::Failed);
    }
    normalize_output_for_account(&output.stdout, host, account_digest, unix_seconds(now)?)
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

#[cfg(test)]
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
    qualify_and_dispatch_snapshot(
        &snapshot,
        account_digest,
        threshold_percent,
        previous_snapshots,
        sampled_at,
        dispatch_clock,
        dispatch,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn qualify_and_dispatch_preferred_with_runners<T, C>(
    native: &impl NativeUsageRunner,
    fallback: &impl CommandRunner,
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
    let snapshot = check_preferred_with_runners(native, fallback, account_digest, sampled_at)
        .map_err(AutomaticDispatchError::Sensor)?;
    qualify_and_dispatch_snapshot(
        &snapshot,
        account_digest,
        threshold_percent,
        previous_snapshots,
        sampled_at,
        dispatch_clock,
        dispatch,
    )
}

pub(crate) fn qualify_and_dispatch_snapshot<T, C>(
    snapshot: &NormalizedSnapshot,
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
    let evidence = UsageGuardEvidence {
        digest: snapshot.evidence_digest(),
        window: snapshot.selected_window_label(),
    };
    let observation = UsageObservation {
        evidence,
        snapshots: snapshot.core_snapshots(),
    };
    let policy = UsagePolicy::new(
        &snapshot.sensor_id,
        &snapshot.sensor_version,
        &snapshot.provider,
        account_digest,
    )
    .with_stop_remaining_percent(threshold_percent)
    .map_err(|_| AutomaticDispatchError::InvalidPolicy)?;
    let evaluated_at_unix_seconds =
        i64::try_from(unix_seconds(sampled_at).map_err(AutomaticDispatchError::Sensor)?)
            .map_err(|_| AutomaticDispatchError::Sensor(SensorError::ClockInvalid))?;
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

fn validate_version_for_executable(
    executable: &QualifiedExecutable,
    stdout: &[u8],
) -> Result<(), SensorError> {
    if validate_version(stdout).is_ok() {
        return Ok(());
    }
    if !matches!(stdout, b"CodexBar" | b"CodexBar\n" | b"CodexBar\r\n") {
        return Err(SensorError::UnsupportedVersion);
    }
    validate_macos_bundle_version(executable)
}

#[cfg(target_os = "macos")]
fn validate_macos_bundle_version(executable: &QualifiedExecutable) -> Result<(), SensorError> {
    use std::os::unix::fs::OpenOptionsExt;

    const O_NOFOLLOW: i32 = 0x100;

    if executable.identity.is_none() {
        return Err(SensorError::UnsupportedVersion);
    }
    verify_executable_identity(executable)?;
    let helpers = executable
        .path
        .parent()
        .filter(|parent| parent.file_name() == Some(OsStr::new("Helpers")))
        .ok_or(SensorError::UnsupportedVersion)?;
    if executable.path.file_name() != Some(OsStr::new("CodexBarCLI")) {
        return Err(SensorError::UnsupportedVersion);
    }
    let contents = helpers
        .parent()
        .filter(|parent| parent.file_name() == Some(OsStr::new("Contents")))
        .ok_or(SensorError::UnsupportedVersion)?;
    contents
        .parent()
        .filter(|parent| parent.file_name() == Some(OsStr::new("CodexBar.app")))
        .ok_or(SensorError::UnsupportedVersion)?;
    let info_plist = contents.join("Info.plist");
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(O_NOFOLLOW)
        .open(&info_plist)
        .map_err(|_| SensorError::FilesystemSafety)?;
    let before = file.metadata().map_err(|_| SensorError::FilesystemSafety)?;
    if !before.is_file() || before.len() > OUTPUT_LIMIT as u64 {
        return Err(SensorError::UnsupportedVersion);
    }
    let before_identity = metadata_identity(&before);
    let bytes = read_bounded(&mut file, OUTPUT_LIMIT).map_err(|error| match error {
        SensorError::OutputTooLarge => SensorError::UnsupportedVersion,
        _ => SensorError::FilesystemSafety,
    })?;
    let after = file.metadata().map_err(|_| SensorError::FilesystemSafety)?;
    if metadata_identity(&after) != before_identity {
        return Err(SensorError::FilesystemSafety);
    }
    verify_executable_identity(executable)?;
    validate_bundle_identity_and_version(&bytes)
}

#[cfg(not(target_os = "macos"))]
fn validate_macos_bundle_version(_executable: &QualifiedExecutable) -> Result<(), SensorError> {
    Err(SensorError::UnsupportedVersion)
}

#[cfg(target_os = "macos")]
fn validate_bundle_identity_and_version(bytes: &[u8]) -> Result<(), SensorError> {
    let prepared = prepare_plist_bytes(bytes)?;
    let value = plutil_json(&prepared)?;
    let object = value.as_object().ok_or(SensorError::UnsupportedVersion)?;
    if object.get("CFBundleIdentifier").and_then(Value::as_str) != Some("com.steipete.codexbar")
        || object
            .get("CFBundleShortVersionString")
            .and_then(Value::as_str)
            != Some(CODEXBAR_VERSION)
    {
        return Err(SensorError::UnsupportedVersion);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn prepare_plist_bytes(bytes: &[u8]) -> Result<std::borrow::Cow<'_, [u8]>, SensorError> {
    const APPLE_DTD: &str = r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#;

    if bytes.starts_with(b"bplist00") {
        return Ok(std::borrow::Cow::Borrowed(bytes));
    }
    let plist = std::str::from_utf8(bytes).map_err(|_| SensorError::UnsupportedVersion)?;
    let trimmed = plist.trim_start_matches(['\u{feff}', ' ', '\t', '\r', '\n']);
    if !(trimmed.starts_with("<?xml") || trimmed.starts_with("<plist"))
        || ["<!--", "<![CDATA[", "<!ENTITY"]
            .iter()
            .any(|marker| plist.contains(marker))
    {
        return Err(SensorError::UnsupportedVersion);
    }
    for key in ["CFBundleIdentifier", "CFBundleShortVersionString"] {
        let target = format!("<key>{key}</key>");
        if plist.match_indices(&target).count() != 1 {
            return Err(SensorError::UnsupportedVersion);
        }
    }
    let mut cursor = 0;
    while let Some(relative) = plist[cursor..].find("<key>") {
        let start = cursor + relative + "<key>".len();
        let end = plist[start..]
            .find("</key>")
            .map(|relative| start + relative)
            .ok_or(SensorError::UnsupportedVersion)?;
        if plist[start..end].contains('&') {
            return Err(SensorError::UnsupportedVersion);
        }
        cursor = end + "</key>".len();
    }
    let dtd_count = plist.match_indices("<!DOCTYPE").count();
    if dtd_count > 1 || (dtd_count == 1 && !plist.contains(APPLE_DTD)) {
        return Err(SensorError::UnsupportedVersion);
    }
    if dtd_count == 1 {
        Ok(std::borrow::Cow::Owned(
            plist.replacen(APPLE_DTD, "", 1).into_bytes(),
        ))
    } else {
        Ok(std::borrow::Cow::Borrowed(bytes))
    }
}

#[cfg(target_os = "macos")]
fn plutil_json(bytes: &[u8]) -> Result<Value, SensorError> {
    use std::io::Write as _;

    const PLUTIL_OUTPUT_LIMIT: usize = 64 * 1024;

    let plutil = qualify_program("/usr/bin/plutil").map_err(|_| SensorError::UnsupportedVersion)?;
    verify_executable_identity(&plutil)?;
    let mut child = Command::new(&plutil.path)
        .args(["-convert", "json", "-o", "-", "--", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| SensorError::UnsupportedVersion)?;
    let mut stdin = child.stdin.take().ok_or(SensorError::UnsupportedVersion)?;
    let input = bytes.to_vec();
    let writer = thread::spawn(move || stdin.write_all(&input));
    let stdout = child.stdout.take().ok_or(SensorError::UnsupportedVersion)?;
    let stderr = child.stderr.take().ok_or(SensorError::UnsupportedVersion)?;
    let stdout_reader = spawn_bounded_reader(stdout, PLUTIL_OUTPUT_LIMIT);
    let stderr_reader = spawn_bounded_reader(stderr, PLUTIL_OUTPUT_LIMIT);
    let started = Instant::now();
    let status = loop {
        match child
            .try_wait()
            .map_err(|_| SensorError::UnsupportedVersion)?
        {
            Some(status) => break status,
            None if started.elapsed() >= VERSION_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = writer.join();
                verify_executable_identity(&plutil)?;
                return Err(SensorError::UnsupportedVersion);
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    let wrote_all = writer
        .join()
        .map_err(|_| SensorError::UnsupportedVersion)?
        .is_ok();
    let stdout = receive_output(&stdout_reader, started, VERSION_TIMEOUT)
        .map_err(|_| SensorError::UnsupportedVersion)?;
    let stderr = receive_output(&stderr_reader, started, VERSION_TIMEOUT)
        .map_err(|_| SensorError::UnsupportedVersion)?;
    verify_executable_identity(&plutil)?;
    if !status.success() || !wrote_all || !stderr.is_empty() {
        return Err(SensorError::UnsupportedVersion);
    }
    parse_strict_native_json(&stdout).map_err(|_| SensorError::UnsupportedVersion)
}

const fn usage_arguments(host: UsageHost) -> &'static [&'static str] {
    match host {
        UsageHost::Codex => CODEX_USAGE_ARGUMENTS,
        UsageHost::Claude => CLAUDE_USAGE_ARGUMENTS,
        UsageHost::Antigravity => ANTIGRAVITY_USAGE_ARGUMENTS,
    }
}

#[cfg(test)]
fn normalize_output(
    stdout: &[u8],
    account_digest: &str,
    now: u64,
) -> Result<NormalizedSnapshot, SensorError> {
    normalize_output_for_account(stdout, UsageHost::Codex, Some(account_digest), now)
}

fn normalize_output_for_account(
    stdout: &[u8],
    host: UsageHost,
    requested_account_digest: Option<&str>,
    now: u64,
) -> Result<NormalizedSnapshot, SensorError> {
    let rows: Vec<CodexBarRow> =
        serde_json::from_slice(stdout).map_err(|_| SensorError::Malformed)?;
    if rows.iter().any(|row| row.error.is_some()) {
        return Err(SensorError::RowError);
    }
    for row in &rows {
        validate_row_identity(row, host)?;
    }
    let accounts = rows
        .iter()
        .map(|row| row_account_identity(row, host).map(|account| (row, account)))
        .collect::<Result<Vec<_>, _>>()?;
    let (row, account_digest) = if let Some(account_digest) = requested_account_digest {
        let mut matches = accounts
            .iter()
            .copied()
            .filter(|(_, account)| sha256_digest(account.as_bytes()) == account_digest);
        let (row, _account) = matches.next().ok_or(SensorError::AccountNotFound)?;
        if matches.next().is_some() {
            return Err(SensorError::DuplicateAccount);
        }
        (row, account_digest.to_owned())
    } else {
        let mut accounts = accounts.into_iter();
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
    let windows = if host == UsageHost::Antigravity {
        let primary: CodexBarWindow = serde_json::from_value(
            usage
                .primary
                .as_ref()
                .ok_or(SensorError::WrongWindows)?
                .clone(),
        )
        .map_err(|_| SensorError::Malformed)?;
        let [secondary] = usage.secondary.as_slice() else {
            return Err(SensorError::WrongWindows);
        };
        if secondary.is_null() {
            return Err(SensorError::WrongWindows);
        }
        let secondary: CodexBarWindow =
            serde_json::from_value(secondary.clone()).map_err(|_| SensorError::Malformed)?;
        vec![
            normalize_antigravity_window(&primary, "default")?,
            normalize_antigravity_window(&secondary, "antigravity-claude-gpt")?,
        ]
    } else if let Some(primary) = usage.primary.as_ref() {
        let primary: CodexBarWindow =
            serde_json::from_value(primary.clone()).map_err(|_| SensorError::Malformed)?;
        vec![normalize_window(&primary, "session", None, 300)?]
    } else {
        let [secondary] = usage.secondary.as_slice() else {
            return Err(SensorError::WrongWindows);
        };
        if secondary.is_null() {
            return Err(SensorError::WrongWindows);
        }
        let secondary: CodexBarWindow =
            serde_json::from_value(secondary.clone()).map_err(|_| SensorError::Malformed)?;
        vec![normalize_window(&secondary, "weekly", None, 10_080)?]
    };
    Ok(NormalizedSnapshot {
        sensor_id: "codexbar".to_owned(),
        sensor_version: CODEXBAR_VERSION.to_owned(),
        provider: host.as_str().to_owned(),
        account_digest,
        measured_at,
        expires_at,
        source_confidence: "local".to_owned(),
        windows,
    })
}

fn validate_row_identity(row: &CodexBarRow, host: UsageHost) -> Result<(), SensorError> {
    row_account_identity(row, host)?;
    if row.provider != host.as_str() {
        return Err(SensorError::WrongProvider);
    }
    if row.source != "cli" && row.source != host.local_source() {
        return Err(SensorError::NonLocalSource);
    }
    let usage = row.usage.as_ref().ok_or(SensorError::Malformed)?;
    let identity = usage
        .identity
        .as_ref()
        .ok_or(SensorError::MissingIdentity)?;
    if identity.provider_id != host.as_str() {
        return Err(SensorError::WrongProvider);
    }
    Ok(())
}

fn row_account_identity(row: &CodexBarRow, host: UsageHost) -> Result<&str, SensorError> {
    let row_account = row.account.as_deref();
    let direct_usage_account = row
        .usage
        .as_ref()
        .and_then(|usage| usage.account_email.as_deref());
    let identity_usage_account = row
        .usage
        .as_ref()
        .and_then(|usage| usage.identity.as_ref())
        .and_then(|identity| identity.account_email.as_deref());
    let usage_account = unambiguous_account_identity(direct_usage_account, identity_usage_account)?;
    if host == UsageHost::Antigravity {
        return usage_account.ok_or(SensorError::MissingIdentity);
    }
    unambiguous_account_identity(row_account, usage_account)?.ok_or(SensorError::MissingIdentity)
}

fn unambiguous_account_identity<'a>(
    first: Option<&'a str>,
    second: Option<&'a str>,
) -> Result<Option<&'a str>, SensorError> {
    if first.is_some_and(|account| account.trim().is_empty())
        || second.is_some_and(|account| account.trim().is_empty())
    {
        return Err(SensorError::MissingIdentity);
    }
    match (first, second) {
        (Some(account), Some(email)) if account == email => Ok(Some(account)),
        (Some(_), Some(_)) => Err(SensorError::AmbiguousData),
        (Some(account), None) => Ok(Some(account)),
        (None, Some(email)) => Ok(Some(email)),
        (None, None) => Ok(None),
    }
}

fn normalize_window(
    window: &CodexBarWindow,
    name: &'static str,
    quota_pool: Option<&'static str>,
    expected_minutes: u32,
) -> Result<NormalizedWindow, SensorError> {
    if !matches!(
        window.window_minutes,
        CodexBarWindowMinutes::Value(actual) if actual == expected_minutes
    ) {
        return Err(SensorError::WrongWindows);
    }
    if !window.used_percent.is_finite() || !(0.0..=100.0).contains(&window.used_percent) {
        return Err(SensorError::Malformed);
    }
    Ok(NormalizedWindow {
        name,
        quota_pool,
        window_minutes: Some(expected_minutes),
        remaining_percent: 100.0 - window.used_percent,
        resets_at: parse_iso8601_z(window.resets_at.as_deref().ok_or(SensorError::Malformed)?)?,
    })
}

fn normalize_antigravity_window(
    window: &CodexBarWindow,
    quota_pool: &'static str,
) -> Result<NormalizedWindow, SensorError> {
    let window_minutes = match window.window_minutes {
        CodexBarWindowMinutes::Missing => None,
        CodexBarWindowMinutes::Value(10_080) => Some(10_080),
        CodexBarWindowMinutes::Null | CodexBarWindowMinutes::Value(_) => {
            return Err(SensorError::WrongWindows);
        }
    };
    if !window.used_percent.is_finite() || !(0.0..=100.0).contains(&window.used_percent) {
        return Err(SensorError::Malformed);
    }
    Ok(NormalizedWindow {
        name: "provider",
        quota_pool: Some(quota_pool),
        window_minutes,
        remaining_percent: 100.0 - window.used_percent,
        resets_at: parse_iso8601_z(window.resets_at.as_deref().ok_or(SensorError::Malformed)?)?,
    })
}

fn unix_seconds(time: SystemTime) -> Result<u64, SensorError> {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| SensorError::ClockInvalid)
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
        check_codexbar_provider_with_runner, check_preferred_with_runners,
        check_unique_with_runner, check_with_runner, consume_for_automatic_dispatch,
        native_then_fallback, normalize_output, normalize_output_for_account, parse_iso8601_z,
        qualify_and_dispatch_with_runner, resolve_program_in_path, AutomaticDispatchError,
        CommandOutput, CommandRunner, NativeUsageRunner, NormalizedSnapshot, QualifiedExecutable,
        SensorError, UsageHost, ANTIGRAVITY_USAGE_ARGUMENTS, CODEX_USAGE_ARGUMENTS, USAGE_TIMEOUT,
        VERSION_TIMEOUT,
    };
    #[cfg(unix)]
    use super::{qualify_program, SystemCommandRunner};
    #[cfg(target_os = "macos")]
    use super::{validate_version_for_executable, OUTPUT_LIMIT};
    use hive_core::sha256_digest;
    use hive_core::usage_guard::{
        evaluate_usage, UsageDecision, UsagePermitError, UsagePolicy, UsageWindow,
    };
    use serde_json::Value;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;
    #[cfg(target_os = "macos")]
    use std::process::Command;
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

    struct FakeNativeRunner {
        error: SensorError,
        invocations: RefCell<usize>,
    }

    impl NativeUsageRunner for FakeNativeRunner {
        fn read_codex_native(
            &self,
            _account_digest: Option<&str>,
            _now: std::time::SystemTime,
        ) -> Result<NormalizedSnapshot, SensorError> {
            *self.invocations.borrow_mut() += 1;
            Err(self.error.clone())
        }
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

    #[cfg(windows)]
    #[test]
    fn preserves_windows_path_directory_precedence() {
        let shim_directory = tempfile::tempdir().expect("shim directory should exist");
        let native_directory = tempfile::tempdir().expect("native directory should exist");
        let wrapper = shim_directory.path().join("codex.cmd");
        fs::write(&wrapper, b"@exit /b 0\r\n").expect("command shim should be created");
        let native = native_directory.path().join("codex.exe");
        fs::write(&native, b"fixture").expect("native executable should be created");
        let search_path = std::env::join_paths([shim_directory.path(), native_directory.path()])
            .expect("search path should be valid");

        assert_eq!(
            resolve_program_in_path("codex", &search_path),
            Some(
                wrapper
                    .canonicalize()
                    .expect("earlier wrapper should resolve")
            )
        );
    }

    #[cfg(windows)]
    #[test]
    fn resolves_the_native_binary_behind_the_codex_npm_shim() {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let native = directory.path().join(
            "node_modules/@openai/codex/node_modules/@openai/codex-win32-x64/vendor/x86_64-pc-windows-msvc/bin/codex.exe",
        );
        fs::create_dir_all(
            native
                .parent()
                .expect("native fixture should have a parent directory"),
        )
        .expect("native fixture parent should be created");
        fs::write(directory.path().join("codex"), b"#!/bin/sh\n")
            .expect("extensionless shim should be created");
        fs::write(directory.path().join("codex.cmd"), b"@exit /b 0\r\n")
            .expect("command shim should be created");
        fs::write(&native, b"fixture").expect("native executable should be created");

        assert_eq!(
            resolve_program_in_path("codex", &OsString::from(directory.path())),
            Some(
                native
                    .canonicalize()
                    .expect("native fixture should resolve")
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

    fn antigravity_row(
        primary_minutes: Option<Value>,
        secondary_minutes: Option<Value>,
        primary_used: u8,
        secondary_used: u8,
    ) -> Vec<u8> {
        let mut value = serde_json::json!([{
            "provider": "antigravity",
            "account": "Antigravity account label",
            "source": "cli",
            "usage": {
                "primary": {
                    "usedPercent": primary_used,
                    "resetsAt": "2026-07-30T00:00:00Z"
                },
                "secondary": {
                    "usedPercent": secondary_used,
                    "resetsAt": "2026-08-06T00:00:00Z"
                },
                "updatedAt": "2026-07-23T12:00:00Z",
                "identity": {
                    "providerID": "antigravity",
                    "accountEmail": account()
                },
                "extraRateWindows": [{}, {}]
            },
            "error": null
        }]);
        if let Some(minutes) = primary_minutes {
            value[0]["usage"]["primary"]["windowMinutes"] = minutes;
        }
        if let Some(minutes) = secondary_minutes {
            value[0]["usage"]["secondary"]["windowMinutes"] = minutes;
        }
        serde_json::to_vec(&value).expect("fixture should serialize")
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
        let snapshot = check_with_runner(&runner, &account_digest(), now("2026-07-23T12:00:30Z"))
            .expect("valid sensor response should normalize");
        let invocations = runner.invocations.borrow();
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].program, "codexbar");
        assert_eq!(invocations[0].arguments, ["--version"]);
        assert_eq!(invocations[0].timeout, VERSION_TIMEOUT);
        assert_eq!(
            invocations[1].arguments,
            CODEX_USAGE_ARGUMENTS
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        );
        assert_eq!(invocations[1].timeout, USAGE_TIMEOUT);
        let core = snapshot.core_snapshots();
        assert_eq!(core[0].schema_version, 1);
        assert_eq!(core[0].quota_pool, None);
    }

    #[test]
    fn antigravity_omits_all_accounts_and_enforces_both_quota_pools() {
        let output = antigravity_row(None, Some(serde_json::json!(10_080)), 44, 95);
        let runner = FakeRunner::new([Ok(success("CodexBar 0.45.2\n")), Ok(success(output))]);

        let snapshot = check_codexbar_provider_with_runner(
            &runner,
            UsageHost::Antigravity,
            &account_digest(),
            now("2026-07-23T12:00:30Z"),
        )
        .expect("both Antigravity quota pools should normalize");

        let invocations = runner.invocations.borrow();
        assert_eq!(invocations[1].arguments, ANTIGRAVITY_USAGE_ARGUMENTS);
        assert!(!invocations[1]
            .arguments
            .iter()
            .any(|argument| argument == "--all-accounts"));
        assert_eq!(snapshot.provider, "antigravity");
        assert_eq!(snapshot.windows.len(), 2);
        assert_eq!(snapshot.windows[0].name, "provider");
        assert_eq!(snapshot.windows[0].quota_pool, Some("default"));
        assert_eq!(snapshot.windows[0].window_minutes, None);
        assert_eq!(snapshot.windows[1].name, "provider");
        assert_eq!(
            snapshot.windows[1].quota_pool,
            Some("antigravity-claude-gpt")
        );
        assert_eq!(snapshot.windows[1].window_minutes, Some(10_080));
        assert_eq!(snapshot.selected_window_label(), "multiple");
        let core = snapshot.core_snapshots();
        assert!(core.iter().all(|snapshot| snapshot.schema_version == 2));
        assert!(core
            .iter()
            .all(|snapshot| snapshot.quota_window == UsageWindow::Provider));
        assert_eq!(core[0].quota_pool.as_deref(), Some("default"));
        assert_eq!(
            core[1].quota_pool.as_deref(),
            Some("antigravity-claude-gpt")
        );
        let policy = UsagePolicy::new("codexbar", "0.45.2", "antigravity", account_digest())
            .with_stop_remaining_percent(10)
            .expect("threshold should be valid");
        assert!(matches!(
            evaluate_usage(
                &policy,
                &core,
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
    fn antigravity_optional_weekly_metadata_has_one_canonical_identity() {
        let missing = normalize_output_for_account(
            &antigravity_row(None, None, 44, 43),
            UsageHost::Antigravity,
            Some(&account_digest()),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("upstream nil-duration pools should normalize");
        let enriched = normalize_output_for_account(
            &antigravity_row(
                Some(serde_json::json!(10_080)),
                Some(serde_json::json!(10_080)),
                44,
                43,
            ),
            UsageHost::Antigravity,
            Some(&account_digest()),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("weekly-enriched pools should normalize");

        assert_eq!(missing.core_snapshots(), enriched.core_snapshots());
        assert_eq!(missing.evidence_digest(), enriched.evidence_digest());
        assert_eq!(missing.selected_window_label(), "multiple");
        assert_eq!(enriched.selected_window_label(), "multiple");

        for (minutes, expected) in [
            (serde_json::Value::Null, SensorError::WrongWindows),
            (serde_json::json!(300), SensorError::WrongWindows),
            (serde_json::json!(10_079), SensorError::WrongWindows),
            (serde_json::json!(10_081), SensorError::WrongWindows),
            (serde_json::json!(-1), SensorError::Malformed),
            (serde_json::json!(10_080.5), SensorError::Malformed),
            (serde_json::json!("10080"), SensorError::Malformed),
            (serde_json::json!([]), SensorError::Malformed),
            (serde_json::json!({}), SensorError::Malformed),
        ] {
            assert_eq!(
                normalize_output_for_account(
                    &antigravity_row(Some(minutes), None, 44, 43),
                    UsageHost::Antigravity,
                    Some(&account_digest()),
                    parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
                ),
                Err(expected)
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn app_bundle_fallback_requires_exact_stdout_path_identity_and_version() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let contents = directory.path().join("CodexBar.app/Contents");
        let helpers = contents.join("Helpers");
        fs::create_dir_all(&helpers).expect("bundle directories should exist");
        let executable = helpers.join("CodexBarCLI");
        fs::write(&executable, b"fixture").expect("fixture executable should be created");
        make_executable(&executable);
        fs::write(
            contents.join("Info.plist"),
            b"<?xml version=\"1.0\"?><plist><dict><key>CFBundleIdentifier</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></plist>",
        )
        .expect("Info.plist should be created");
        let qualified = qualify_program(
            executable
                .to_str()
                .expect("fixture path should be valid UTF-8"),
        )
        .expect("fixture executable should qualify");

        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar\n"),
            Ok(())
        );
        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar 0.45.3\n"),
            Err(SensorError::UnsupportedVersion)
        );
        assert_eq!(
            validate_version_for_executable(&qualified, b" CodexBar\n"),
            Err(SensorError::UnsupportedVersion)
        );

        fs::write(
            contents.join("Info.plist"),
            b"<plist><dict><key>CFBundleIdentifier</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></plist>",
        )
        .expect("duplicate-key Info.plist should be written");
        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar\n"),
            Err(SensorError::UnsupportedVersion)
        );

        fs::write(
            contents.join("Info.plist"),
            b"<plist><dict><key>CFBundleIdentifier</key><string>example.invalid</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></plist>",
        )
        .expect("wrong-identifier Info.plist should be written");
        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar\n"),
            Err(SensorError::UnsupportedVersion)
        );

        let info_plist = contents.join("Info.plist");
        let linked_plist = directory.path().join("linked-Info.plist");
        fs::write(
            &linked_plist,
            b"<plist><dict><key>CFBundleIdentifier</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></plist>",
        )
        .expect("linked plist target should be written");
        fs::remove_file(&info_plist).expect("regular Info.plist should be removed");
        symlink(&linked_plist, &info_plist).expect("Info.plist symlink should be created");
        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar\n"),
            Err(SensorError::FilesystemSafety)
        );

        fs::remove_file(&info_plist).expect("Info.plist symlink should be removed");
        fs::write(&info_plist, vec![b'x'; OUTPUT_LIMIT + 1])
            .expect("oversized Info.plist should be written");
        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar\n"),
            Err(SensorError::UnsupportedVersion)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn app_bundle_parser_accepts_binary_and_rejects_non_dictionary_xml() {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let contents = directory.path().join("CodexBar.app/Contents");
        let helpers = contents.join("Helpers");
        fs::create_dir_all(&helpers).expect("bundle directories should exist");
        let executable = helpers.join("CodexBarCLI");
        fs::write(&executable, b"fixture").expect("fixture executable should be created");
        make_executable(&executable);
        let info_plist = contents.join("Info.plist");
        fs::write(
            &info_plist,
            b"<?xml version=\"1.0\"?><plist><dict><key>CFBundleIdentifier</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></plist>",
        )
        .expect("Info.plist should be created");
        let qualified = qualify_program(
            executable
                .to_str()
                .expect("fixture path should be valid UTF-8"),
        )
        .expect("fixture executable should qualify");

        let binary_status = Command::new("/usr/bin/plutil")
            .args(["-convert", "binary1", "--"])
            .arg(&info_plist)
            .status()
            .expect("system plutil should execute");
        assert!(binary_status.success());
        assert_eq!(
            validate_version_for_executable(&qualified, b"CodexBar\n"),
            Ok(())
        );

        for hostile in [
            b"<junk><key>CFBundleIdentifier</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string></junk>".as_slice(),
            b"<plist><array><dict><key>CFBundleIdentifier</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></array></plist>".as_slice(),
            b"<?xml version=\"1.0\"?><!DOCTYPE plist [<!ENTITY bundle \"CFBundleIdentifier\">]><plist><dict><key>&bundle;</key><string>com.steipete.codexbar</string><key>CFBundleShortVersionString</key><string>0.45.2</string></dict></plist>".as_slice(),
        ] {
            fs::write(&info_plist, hostile).expect("hostile Info.plist should be written");
            assert_eq!(
                validate_version_for_executable(&qualified, b"CodexBar\n"),
                Err(SensorError::UnsupportedVersion)
            );
        }
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
    fn native_executable_change_fails_closed_without_fallback() {
        let native = FakeNativeRunner {
            error: SensorError::ExecutableChanged,
            invocations: RefCell::new(0),
        };
        let fallback = FakeRunner::new([
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

        assert_eq!(
            check_preferred_with_runners(
                &native,
                &fallback,
                &account_digest(),
                now("2026-07-23T12:00:30Z"),
            ),
            Err(SensorError::ExecutableChanged)
        );
        assert_eq!(*native.invocations.borrow(), 1);
        assert!(fallback.invocations.borrow().is_empty());
    }

    #[test]
    fn native_timeout_fails_closed_without_fallback() {
        let native = FakeNativeRunner {
            error: SensorError::Timeout,
            invocations: RefCell::new(0),
        };
        let fallback = FakeRunner::new([]);

        assert_eq!(
            check_preferred_with_runners(
                &native,
                &fallback,
                &account_digest(),
                now("2026-07-23T12:00:30Z"),
            ),
            Err(SensorError::Timeout)
        );
        assert_eq!(*native.invocations.borrow(), 1);
        assert!(fallback.invocations.borrow().is_empty());
    }

    #[test]
    fn every_native_integrity_error_fails_closed_without_codexbar() {
        for error in [
            SensorError::Timeout,
            SensorError::OutputTooLarge,
            SensorError::Failed,
            SensorError::ClockInvalid,
            SensorError::FilesystemSafety,
            SensorError::WrongSession,
            SensorError::DuplicateData,
            SensorError::AmbiguousData,
            SensorError::RowError,
            SensorError::MissingIdentity,
            SensorError::AccountNotFound,
            SensorError::DuplicateAccount,
            SensorError::WrongProvider,
            SensorError::NonLocalSource,
            SensorError::WrongWindows,
            SensorError::Stale,
            SensorError::ExecutableChanged,
        ] {
            let native = FakeNativeRunner {
                error: error.clone(),
                invocations: RefCell::new(0),
            };
            let fallback = FakeRunner::new([]);

            assert_eq!(
                check_preferred_with_runners(
                    &native,
                    &fallback,
                    &account_digest(),
                    now("2026-07-23T12:00:30Z"),
                ),
                Err(error)
            );
            assert_eq!(*native.invocations.borrow(), 1);
            assert!(fallback.invocations.borrow().is_empty());
        }
    }

    #[test]
    fn allowlisted_native_errors_fallback_at_most_once_for_every_provider() {
        for host in [UsageHost::Codex, UsageHost::Claude, UsageHost::Antigravity] {
            for error in [
                SensorError::Unavailable,
                SensorError::Unsupported,
                SensorError::UnsupportedVersion,
                SensorError::Malformed,
            ] {
                let calls = Cell::new(0);
                assert_eq!(
                    native_then_fallback(host, Err::<u8, _>(error), || {
                        calls.set(calls.get() + 1);
                        Ok(7)
                    }),
                    Ok(7)
                );
                assert_eq!(calls.get(), 1);
            }
        }
    }

    #[test]
    fn unavailable_fallback_is_reported_once_for_the_exact_provider() {
        for host in [UsageHost::Codex, UsageHost::Claude, UsageHost::Antigravity] {
            let calls = Cell::new(0);
            assert_eq!(
                native_then_fallback(host, Err::<u8, _>(SensorError::Unavailable), || {
                    calls.set(calls.get() + 1);
                    Err(SensorError::Unavailable)
                }),
                Err(SensorError::FallbackRequired(host))
            );
            assert_eq!(calls.get(), 1);
        }
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
    fn usage_account_email_is_an_equivalent_digest_identity() {
        let mut value: serde_json::Value = serde_json::from_slice(&row(
            "codex-cli",
            "2026-07-23T12:00:00Z",
            300,
            10_080,
            20,
            30,
        ))
        .expect("fixture should parse");
        value[0]
            .as_object_mut()
            .expect("row should be an object")
            .remove("account");
        value[0]["usage"]["accountEmail"] = serde_json::json!(account());
        let bytes = serde_json::to_vec(&value).expect("fixture should serialize");

        let snapshot = normalize_output(
            &bytes,
            &account_digest(),
            parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse"),
        )
        .expect("usage.accountEmail should identify the account");

        assert_eq!(snapshot.account_digest, account_digest());
    }

    #[test]
    fn conflicting_account_identity_locations_are_rejected() {
        let mut value: serde_json::Value = serde_json::from_slice(&row(
            "codex-cli",
            "2026-07-23T12:00:00Z",
            300,
            10_080,
            20,
            30,
        ))
        .expect("fixture should parse");
        value[0]["usage"]["accountEmail"] = serde_json::json!("different-account");
        let bytes = serde_json::to_vec(&value).expect("fixture should serialize");

        assert_eq!(
            normalize_output(
                &bytes,
                &account_digest(),
                parse_iso8601_z("2026-07-23T12:00:30Z").expect("fixture should parse")
            ),
            Err(SensorError::AmbiguousData)
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
