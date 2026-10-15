use super::{
    command_timeout, parse_strict_native_json, qualify_program, unix_seconds,
    verify_executable_identity, CommandRunner, NormalizedSnapshot, NormalizedWindow, SensorError,
    SystemCommandRunner, OUTPUT_LIMIT, VERSION_TIMEOUT,
};
use hive_core::sha256_digest;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const MINIMUM_VERSION: (u64, u64, u64) = (0, 145, 0);
const APP_SERVER_TIMEOUT: Duration = Duration::from_secs(15);
const SENSOR_ID: &str = "codex-app-server";
const SUPPORTED_PLANS: &[&str] = &[
    "free",
    "go",
    "plus",
    "pro",
    "prolite",
    "team",
    "self_serve_business_usage_based",
    "business",
    "enterprise_cbp_usage_based",
    "enterprise",
    "edu",
];

pub(super) fn read(
    requested_account_digest: Option<&str>,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    read_with_timeout(requested_account_digest, now, APP_SERVER_TIMEOUT)
}

pub(super) fn read_with_timeout(
    requested_account_digest: Option<&str>,
    now: SystemTime,
    timeout: Duration,
) -> Result<NormalizedSnapshot, SensorError> {
    let started = Instant::now();
    let executable = qualify_program("codex")?;
    let version_output = SystemCommandRunner.run(
        &executable,
        &["--version"],
        command_timeout(VERSION_TIMEOUT),
        OUTPUT_LIMIT,
    )?;
    if !version_output.success {
        return Err(SensorError::Failed);
    }
    let version = parse_version(&version_output.stdout)?;
    verify_executable_identity(&executable)?;
    let remaining = timeout
        .checked_sub(started.elapsed())
        .ok_or(SensorError::Timeout)?;
    let (account, limits) = exchange_with_timeout(&executable, remaining)?;
    verify_executable_identity(&executable)?;
    normalize(
        &account,
        &limits,
        &version,
        requested_account_digest,
        unix_seconds(now)?,
    )
}

/// One subscription-authenticated, non-inference reader owned by the monitor worker.
/// Reuse avoids paying CLI startup and shutdown costs for every five-second observation.
pub(crate) struct MonitorSensor {
    connection: Option<MonitorConnection>,
}

struct MonitorConnection {
    executable: super::QualifiedExecutable,
    version: String,
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<Result<Value, SensorError>>,
    opened: Instant,
    next_id: u64,
}

impl Drop for MonitorConnection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl MonitorSensor {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn read(&mut self) -> Result<NormalizedSnapshot, SensorError> {
        if self
            .connection
            .as_ref()
            .is_some_and(|c| c.opened.elapsed() >= Duration::from_mins(4))
        {
            self.connection = None;
        }
        let started = Instant::now();
        let result = self.read_inner(started, Duration::from_secs(5));
        if result.is_err() {
            self.connection = None;
        }
        result
    }

    fn ensure_connection(
        &mut self,
        started: Instant,
        timeout: Duration,
        output_limit: usize,
    ) -> Result<(), SensorError> {
        if self.connection.is_none() {
            let executable = qualify_program("codex")?;
            let output = SystemCommandRunner.run(
                &executable,
                &["--version"],
                timeout.saturating_sub(started.elapsed()),
                OUTPUT_LIMIT,
            )?;
            if !output.success {
                return Err(SensorError::Failed);
            }
            let version = parse_version(&output.stdout)?;
            let mut child = app_server_command(&executable)
                .spawn()
                .map_err(|_| SensorError::Failed)?;
            let stdin = child.stdin.take().ok_or(SensorError::Failed)?;
            let stdout = child.stdout.take().ok_or(SensorError::Failed)?;
            let stderr = child.stderr.take().ok_or(SensorError::Failed)?;
            let responses = spawn_jsonl_reader(stdout, output_limit);
            // This connection is retired before its bounded lifetime output budget is exhausted.
            let _stderr = super::spawn_bounded_reader(stderr, OUTPUT_LIMIT);
            self.connection = Some(MonitorConnection {
                executable,
                version,
                child,
                stdin,
                responses,
                opened: Instant::now(),
                next_id: 1,
            });
            let connection = self.connection.as_mut().ok_or(SensorError::Failed)?;
            monitor_request(
                connection,
                "initialize",
                &serde_json::json!({"clientInfo":{"name":"aigent-hive-usage-monitor","version":env!("CARGO_PKG_VERSION")}}),
                started,
                timeout,
            )?;
            send(
                &mut connection.stdin,
                &serde_json::json!({"method":"initialized","params":{}}),
            )?;
        }
        Ok(())
    }

    /// Metadata-only discovery. No thread is loaded, resumed, or sent a model request.
    pub(crate) fn desktop_thread_ids(&mut self) -> Result<Vec<String>, SensorError> {
        if self
            .connection
            .as_ref()
            .is_some_and(|c| c.opened.elapsed() >= Duration::from_secs(15))
        {
            self.connection = None;
        }
        let result = self.desktop_thread_ids_inner();
        if result.is_err() {
            self.connection = None;
        }
        result
    }

    fn desktop_thread_ids_inner(&mut self) -> Result<Vec<String>, SensorError> {
        let started = Instant::now();
        let timeout = Duration::from_secs(5);
        self.ensure_connection(started, timeout, 32 * OUTPUT_LIMIT)?;
        let connection = self.connection.as_mut().ok_or(SensorError::Failed)?;
        verify_executable_identity(&connection.executable)?;
        let mut ids = std::collections::BTreeSet::new();
        for archived in [false, true] {
            let mut cursor = Value::Null;
            let mut cursors = std::collections::HashSet::new();
            loop {
                let response = monitor_request(
                    connection,
                    "thread/list",
                    &serde_json::json!({
                        "limit":100,"sortKey":"updated_at","useStateDbOnly":true,"archived":archived,
                        "sourceKinds":["cli","vscode","exec","appServer","subAgent","subAgentReview","subAgentCompact","subAgentThreadSpawn","subAgentOther","unknown"],
                        "cursor":cursor
                    }),
                    started,
                    timeout,
                )?;
                let page = response
                    .get("data")
                    .and_then(Value::as_array)
                    .ok_or(SensorError::Malformed)?;
                if page.len() > 100 {
                    return Err(SensorError::OutputTooLarge);
                }
                for thread in page {
                    let id = thread
                        .get("id")
                        .and_then(Value::as_str)
                        .ok_or(SensorError::Malformed)?;
                    if id.len() != 36
                        || !id.bytes().enumerate().all(|(i, b)| {
                            if [8, 13, 18, 23].contains(&i) {
                                b == b'-'
                            } else {
                                b.is_ascii_hexdigit() && !b.is_ascii_uppercase()
                            }
                        })
                    {
                        return Err(SensorError::Malformed);
                    }
                    ids.insert(id.to_owned());
                    if ids.len() > 4096 {
                        return Err(SensorError::OutputTooLarge);
                    }
                }
                cursor = response.get("nextCursor").cloned().unwrap_or(Value::Null);
                if cursor.is_null() {
                    break;
                }
                let text = cursor
                    .as_str()
                    .filter(|s| s.len() <= 4096)
                    .ok_or(SensorError::Malformed)?;
                if !cursors.insert(text.to_owned()) {
                    return Err(SensorError::Malformed);
                }
            }
        }
        verify_executable_identity(&connection.executable)?;
        Ok(ids.into_iter().collect())
    }

    fn read_inner(
        &mut self,
        started: Instant,
        timeout: Duration,
    ) -> Result<NormalizedSnapshot, SensorError> {
        self.ensure_connection(started, timeout, OUTPUT_LIMIT)?;
        let connection = self.connection.as_mut().ok_or(SensorError::Failed)?;
        verify_executable_identity(&connection.executable)?;
        let account = monitor_request(
            connection,
            "account/read",
            &serde_json::json!({"refreshToken":false}),
            started,
            timeout,
        )?;
        let limits = monitor_request(
            connection,
            "account/rateLimits/read",
            &Value::Null,
            started,
            timeout,
        )?;
        verify_executable_identity(&connection.executable)?;
        normalize(
            &account,
            &limits,
            &connection.version,
            None,
            unix_seconds(SystemTime::now())?,
        )
    }
}

fn monitor_request(
    connection: &mut MonitorConnection,
    method: &str,
    params: &Value,
    started: Instant,
    timeout: Duration,
) -> Result<Value, SensorError> {
    let id = connection.next_id;
    connection.next_id += 1;
    send(
        &mut connection.stdin,
        &serde_json::json!({"method":method,"id":id,"params":params}),
    )?;
    await_response(&connection.responses, id, started, timeout)
}

fn app_server_command(executable: &super::QualifiedExecutable) -> Command {
    let mut command = Command::new(&executable.path);
    command
        .args(["app-server", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

fn exchange_with_timeout(
    executable: &super::QualifiedExecutable,
    timeout: Duration,
) -> Result<(Value, Value), SensorError> {
    let mut child = app_server_command(executable).spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            SensorError::Unavailable
        } else {
            SensorError::Failed
        }
    })?;
    let mut stdin = child.stdin.take().ok_or(SensorError::Failed)?;
    let stdout = child.stdout.take().ok_or(SensorError::Failed)?;
    let stderr = child.stderr.take().ok_or(SensorError::Failed)?;
    let responses = spawn_jsonl_reader(stdout, OUTPUT_LIMIT);
    let stderr_reader = super::spawn_bounded_reader(stderr, OUTPUT_LIMIT);
    let started = Instant::now();
    let timeout = command_timeout(timeout);
    let result = (|| {
        send(
            &mut stdin,
            &serde_json::json!({
                "method": "initialize",
                "id": 1,
                "params": {
                    "clientInfo": {
                        "name": "aigent-hive",
                        "title": "Aigent Hive",
                        "version": env!("CARGO_PKG_VERSION"),
                    }
                }
            }),
        )?;
        let _initialize = await_response(&responses, 1, started, timeout)?;
        send(
            &mut stdin,
            &serde_json::json!({"method": "initialized", "params": {}}),
        )?;
        send(
            &mut stdin,
            &serde_json::json!({
                "method": "account/read",
                "id": 2,
                "params": {"refreshToken": false},
            }),
        )?;
        let account = await_response(&responses, 2, started, timeout)?;
        send(
            &mut stdin,
            &serde_json::json!({
                "method": "account/rateLimits/read",
                "id": 3,
                "params": null,
            }),
        )?;
        let limits = await_response(&responses, 3, started, timeout)?;
        Ok((account, limits))
    })();
    drop(stdin);
    let finish_result = finish_child(&mut child, started, timeout);
    let stderr_result = super::receive_output(&stderr_reader, started, timeout);
    finish_result?;
    let _stderr = stderr_result?;
    result
}

fn send(stdin: &mut ChildStdin, value: &Value) -> Result<(), SensorError> {
    serde_json::to_writer(&mut *stdin, value).map_err(|_| SensorError::Failed)?;
    stdin.write_all(b"\n").map_err(|_| SensorError::Failed)?;
    stdin.flush().map_err(|_| SensorError::Failed)
}

fn finish_child(child: &mut Child, started: Instant, timeout: Duration) -> Result<(), SensorError> {
    let graceful_deadline = Duration::from_secs(2);
    let wait_started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) => return Err(SensorError::Failed),
            Ok(None)
                if wait_started.elapsed() < graceful_deadline && started.elapsed() < timeout =>
            {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(SensorError::Timeout);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(SensorError::Failed);
            }
        }
    }
}

fn spawn_jsonl_reader(
    stdout: impl Read + Send + 'static,
    output_limit: usize,
) -> Receiver<Result<Value, SensorError>> {
    let (sender, receiver) = mpsc::sync_channel(16);
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut total = 0_usize;
        loop {
            let line = match read_bounded_line(&mut reader, &mut total, output_limit) {
                Ok(Some(line)) => line,
                Ok(None) => {
                    let _ = sender.send(Err(SensorError::Failed));
                    return;
                }
                Err(error) => {
                    let _ = sender.send(Err(error));
                    return;
                }
            };
            let value = parse_strict_native_json(&line);
            let failed = value.is_err();
            if sender.send(value).is_err() || failed {
                return;
            }
        }
    });
    receiver
}

fn read_bounded_line(
    reader: &mut impl BufRead,
    total: &mut usize,
    output_limit: usize,
) -> Result<Option<Vec<u8>>, SensorError> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|_| SensorError::Failed)?;
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(line))
            };
        }
        let consumed = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        *total = total
            .checked_add(consumed)
            .ok_or(SensorError::OutputTooLarge)?;
        if *total > output_limit || line.len().saturating_add(consumed) > output_limit {
            return Err(SensorError::OutputTooLarge);
        }
        line.extend_from_slice(&available[..consumed]);
        reader.consume(consumed);
        if line.last() == Some(&b'\n') {
            return Ok(Some(line));
        }
    }
}

fn await_response(
    receiver: &Receiver<Result<Value, SensorError>>,
    request_id: u64,
    started: Instant,
    timeout: Duration,
) -> Result<Value, SensorError> {
    loop {
        let remaining = timeout
            .checked_sub(started.elapsed())
            .ok_or(SensorError::Timeout)?;
        let message = receiver
            .recv_timeout(remaining)
            .map_err(|_| SensorError::Timeout)??;
        let object = message.as_object().ok_or(SensorError::Malformed)?;
        let Some(response_id) = object.get("id") else {
            continue;
        };
        if response_id.as_u64() != Some(request_id) {
            return Err(SensorError::WrongSession);
        }
        if object.contains_key("error") && object.contains_key("result") {
            return Err(SensorError::AmbiguousData);
        }
        if object.contains_key("error") {
            return Err(SensorError::Failed);
        }
        return object.get("result").cloned().ok_or(SensorError::Malformed);
    }
}

fn parse_version(stdout: &[u8]) -> Result<String, SensorError> {
    let raw = std::str::from_utf8(stdout)
        .map_err(|_| SensorError::UnsupportedVersion)?
        .trim();
    let version = raw
        .strip_prefix("codex-cli ")
        .ok_or(SensorError::UnsupportedVersion)?;
    let mut parts = version.split('.');
    let parsed = (
        parse_version_part(parts.next())?,
        parse_version_part(parts.next())?,
        parse_version_part(parts.next())?,
    );
    if parts.next().is_some() || parsed < MINIMUM_VERSION {
        return Err(SensorError::UnsupportedVersion);
    }
    Ok(version.to_owned())
}

fn parse_version_part(value: Option<&str>) -> Result<u64, SensorError> {
    let value = value.ok_or(SensorError::UnsupportedVersion)?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SensorError::UnsupportedVersion);
    }
    value.parse().map_err(|_| SensorError::UnsupportedVersion)
}

fn normalize(
    account_response: &Value,
    response: &Value,
    version: &str,
    requested_account_digest: Option<&str>,
    now: u64,
) -> Result<NormalizedSnapshot, SensorError> {
    let account = account_response
        .get("account")
        .and_then(Value::as_object)
        .ok_or(SensorError::MissingIdentity)?;
    if account.get("type").and_then(Value::as_str) != Some("chatgpt") {
        return Err(SensorError::WrongProvider);
    }
    let email = account
        .get("email")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(SensorError::MissingIdentity)?;
    let plan = account
        .get("planType")
        .and_then(Value::as_str)
        .filter(|value| SUPPORTED_PLANS.contains(value))
        .ok_or(SensorError::Unsupported)?;
    let account_digest = sha256_digest(email.trim().as_bytes());
    if requested_account_digest.is_some_and(|requested| requested != account_digest) {
        return Err(SensorError::AccountNotFound);
    }
    let legacy = response.get("rateLimits").filter(|value| !value.is_null());
    let selected_by_id = response
        .get("rateLimitsByLimitId")
        .and_then(Value::as_object)
        .and_then(|limits| limits.get("codex"))
        .filter(|value| !value.is_null());
    let selected = selected_by_id.or(legacy).ok_or(SensorError::WrongWindows)?;
    if selected_by_id.is_some() && legacy.is_some() && selected_by_id != legacy {
        return Err(SensorError::AmbiguousData);
    }
    let selected = selected.as_object().ok_or(SensorError::Malformed)?;
    if selected.get("limitId").and_then(Value::as_str) != Some("codex") {
        return Err(SensorError::WrongProvider);
    }
    if selected.get("planType").and_then(Value::as_str) != Some(plan) {
        return Err(SensorError::AmbiguousData);
    }
    let mut session = None;
    let mut weekly = None;
    for name in ["primary", "secondary"] {
        let Some(value) = selected.get(name).filter(|value| !value.is_null()) else {
            continue;
        };
        let window = normalize_window(value, now)?;
        match window.name {
            "session" if session.is_none() => session = Some(window),
            "weekly" if weekly.is_none() => weekly = Some(window),
            _ => return Err(SensorError::WrongWindows),
        }
    }
    let windows = session.into_iter().chain(weekly).collect::<Vec<_>>();
    if windows.is_empty() {
        return Err(SensorError::WrongWindows);
    }
    Ok(NormalizedSnapshot {
        sensor_id: SENSOR_ID.to_owned(),
        sensor_version: version.to_owned(),
        provider: "codex".to_owned(),
        account_digest,
        measured_at: now,
        expires_at: now.saturating_add(60),
        source_confidence: "local".to_owned(),
        windows,
    })
}

fn normalize_window(value: &Value, now: u64) -> Result<NormalizedWindow, SensorError> {
    let object = value.as_object().ok_or(SensorError::Malformed)?;
    let used = object
        .get("usedPercent")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && (0.0..=100.0).contains(value))
        .ok_or(SensorError::Malformed)?;
    let minutes = object
        .get("windowDurationMins")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(SensorError::WrongWindows)?;
    let (name, expected_minutes) = match minutes {
        300 => ("session", 300),
        10_080 => ("weekly", 10_080),
        _ => return Err(SensorError::WrongWindows),
    };
    let resets_at = object
        .get("resetsAt")
        .and_then(Value::as_u64)
        .filter(|value| *value > now)
        .ok_or(SensorError::Stale)?;
    Ok(NormalizedWindow {
        name,
        quota_pool: None,
        window_minutes: Some(expected_minutes),
        remaining_percent: 100.0 - used,
        resets_at,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::finish_child;
    use super::{await_response, normalize, parse_version, read_bounded_line};
    use crate::usage::{parse_strict_native_json, SensorError};
    use hive_core::sha256_digest;
    use serde_json::json;
    use std::io::Cursor;
    #[cfg(unix)]
    use std::process::Command;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[test]
    fn supports_the_qualified_codex_version_floor() {
        assert_eq!(
            parse_version(b"codex-cli 0.145.0\n").expect("floor is supported"),
            "0.145.0"
        );
        assert_eq!(
            parse_version(b"codex-cli 0.144.5"),
            Err(SensorError::UnsupportedVersion)
        );
        assert_eq!(
            parse_version(b"codex-cli 0.145.0-dev"),
            Err(SensorError::UnsupportedVersion)
        );
    }

    #[test]
    fn native_limit_id_and_duration_are_normalized_without_raw_identity() {
        let rate_limit = json!({
            "limitId": "codex",
            "planType": "pro",
            "primary": {
                "usedPercent": 21.5,
                "windowDurationMins": 10080,
                "resetsAt": 2_000,
            },
            "secondary": null,
        });
        let snapshot = normalize(
            &json!({
                "account": {
                    "type": "chatgpt",
                    "email": "private@example.invalid",
                    "planType": "pro",
                },
                "requiresOpenaiAuth": true,
            }),
            &json!({
                "rateLimits": rate_limit,
                "rateLimitsByLimitId": {"codex": rate_limit},
            }),
            "0.145.0",
            None,
            1_000,
        )
        .expect("native payload should normalize");

        assert_eq!(snapshot.sensor_id, "codex-app-server");
        assert_eq!(
            snapshot.account_digest,
            sha256_digest(b"private@example.invalid")
        );
        assert_eq!(snapshot.windows[0].name, "weekly");
        assert!((snapshot.windows[0].remaining_percent - 78.5).abs() < f64::EPSILON);
    }

    #[test]
    fn duplicate_json_keys_and_oversized_lines_fail_closed() {
        assert_eq!(
            parse_strict_native_json(br#"{"id":1,"id":2}"#),
            Err(SensorError::DuplicateData)
        );
        let mut reader = Cursor::new(b"123456\n".to_vec());
        let mut total = 0;
        assert_eq!(
            read_bounded_line(&mut reader, &mut total, 3),
            Err(SensorError::OutputTooLarge)
        );
    }

    #[test]
    fn native_integrity_mismatches_preserve_their_error_classes() {
        let account = json!({
            "account": {
                "type": "chatgpt",
                "email": "private@example.invalid",
                "planType": "pro",
            },
        });
        let rate_limit = json!({
            "limitId": "codex",
            "planType": "pro",
            "primary": {
                "usedPercent": 21.5,
                "windowDurationMins": 10080,
                "resetsAt": 2_000,
            },
            "secondary": null,
        });
        let mut conflicting = rate_limit.clone();
        conflicting["primary"]["usedPercent"] = json!(22.0);

        assert_eq!(
            normalize(
                &account,
                &json!({
                    "rateLimits": rate_limit,
                    "rateLimitsByLimitId": {"codex": conflicting},
                }),
                "0.145.0",
                None,
                1_000,
            ),
            Err(SensorError::AmbiguousData)
        );
        assert_eq!(
            normalize(
                &account,
                &json!({
                    "rateLimitsByLimitId": {"codex": rate_limit},
                }),
                "0.145.0",
                Some("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
                1_000,
            ),
            Err(SensorError::AccountNotFound)
        );

        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Ok(json!({"id": 9, "result": {}})))
            .expect("response should queue");
        assert_eq!(
            await_response(&receiver, 3, Instant::now(), Duration::from_secs(1)),
            Err(SensorError::WrongSession)
        );

        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Ok(json!({"id": 3, "result": {}, "error": {}})))
            .expect("response should queue");
        assert_eq!(
            await_response(&receiver, 3, Instant::now(), Duration::from_secs(1)),
            Err(SensorError::AmbiguousData)
        );
    }

    #[test]
    fn native_response_timeout_preserves_the_timeout_class() {
        let (_sender, receiver) = mpsc::sync_channel(1);

        assert_eq!(
            await_response(&receiver, 3, Instant::now(), Duration::from_millis(1)),
            Err(SensorError::Timeout)
        );
    }

    #[test]
    fn unsupported_native_account_plan_is_distinct_from_protocol_malformed() {
        assert_eq!(
            normalize(
                &json!({
                    "account": {
                        "type": "chatgpt",
                        "email": "private@example.invalid",
                        "planType": "future-plan",
                    },
                }),
                &json!({}),
                "0.145.0",
                None,
                1_000,
            ),
            Err(SensorError::Unsupported)
        );
    }

    #[cfg(unix)]
    #[test]
    fn native_process_failure_preserves_the_failed_class() {
        let mut child = Command::new("sh")
            .args(["-c", "exit 7"])
            .spawn()
            .expect("fixture process should spawn");

        assert_eq!(
            finish_child(&mut child, Instant::now(), Duration::from_secs(1)),
            Err(SensorError::Failed)
        );
    }
}
