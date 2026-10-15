use crate::run::AdapterError;
use base64::Engine;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub(super) const SCRIPT: &str = include_str!("windows.ps1");

pub(super) fn command(script: &str) -> Result<Command, AdapterError> {
    if !cfg!(windows) {
        return Err(AdapterError::Unsupported(
            "live usage interruption supports Windows Codex only".into(),
        ));
    }
    let system_root =
        std::env::var_os("SystemRoot").ok_or_else(|| failure("SystemRoot missing"))?;
    let executable = Path::new(&system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    // UTF-8 payload avoids Windows' 32767-character process argument limit. The
    // bootstrap and embedded source are fixed product bytes, never user input.
    let encoded = base64::engine::general_purpose::STANDARD.encode(script.as_bytes());
    let bootstrap = format!("& ([scriptblock]::Create([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{encoded}'))))");
    if bootstrap.len() > 30000 {
        return Err(failure("embedded Windows adapter exceeds launch limit"));
    }
    let mut command = Command::new(executable);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &bootstrap,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    Ok(command)
}

/// Every invocation is bounded; model calls and credentials are never accepted here.
pub(super) fn call(request: &Value, timeout: Duration) -> Result<Value, AdapterError> {
    let mut child = command(SCRIPT)?
        .spawn()
        .map_err(|_| failure("cannot start Windows monitor adapter"))?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| failure("adapter input unavailable"))?;
    input
        .write_all(&serde_json::to_vec(request).map_err(|_| failure("adapter serialization"))?)
        .map_err(|_| failure("adapter input failed"))?;
    drop(input);
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| failure("adapter stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| failure("adapter stderr unavailable"))?;
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout.take(65537).read_to_end(&mut bytes);
        let _ = send.send((result.is_ok(), bytes));
    });
    // Never copy native errors or host command lines into durable usage state.
    thread::spawn(move || {
        let _ = stderr.take(65537).read_to_end(&mut Vec::new());
    });
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(failure("Windows monitor adapter failed"));
                }
                let (ok, bytes) = receive
                    .recv_timeout(timeout.saturating_sub(started.elapsed()))
                    .map_err(|_| failure("adapter output timed out"))?;
                if !ok || bytes.len() > 65536 {
                    return Err(failure("invalid adapter output"));
                }
                return crate::usage::parse_strict_native_json(&bytes)
                    .map_err(|_| failure("invalid adapter JSON"));
            }
            Ok(None) if started.elapsed() < timeout => thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(failure("Windows monitor adapter timed out"));
            }
        }
    }
}

pub(super) fn notify(
    reason: &str,
    success: bool,
    incident: &str,
    root: &Path,
) -> mpsc::Receiver<bool> {
    let message = notification_message(reason, success);
    let root = root.to_path_buf();
    let incident = incident.to_owned();
    let message = format!("{message}\n재개: hive usage monitor acknowledge --user-root <사용자 루트> --incident {incident} --output json");
    // The UI notification must never delay termination or its controller heartbeat.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let delivered = (|| {
            let mut child = command(SCRIPT).ok()?.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
            let request = json!({"operation":"notify","message":message,"user_root":root,"incident":incident});
            child.stdin.take()?.write_all(&serde_json::to_vec(&request).ok()?).ok()?;
            let stdout = child.stdout.take()?;
            let (send, ready) = mpsc::channel();
            thread::spawn(move || {
                use std::io::BufRead;
                let mut line = String::new();
                let _ = std::io::BufReader::new(stdout).take(1024).read_line(&mut line);
                let _ = send.send(line.contains("notified"));
            });
            if ready.recv_timeout(Duration::from_secs(5)).ok() == Some(true) { Some(()) }
            else { let _ = child.kill(); let _ = child.wait(); None }
        })().is_some();
        let _ = tx.send(delivered);
    });
    rx
}

fn notification_message(reason: &str, success: bool) -> &'static str {
    if !success && reason == "sensor-unresponsive" {
        "사용량 감지기가 60초 동안 응답하지 않아 사용량 한도 보호를 우선해 작업 중단을 요청했습니다. 앱은 유지되며, 중단 확인 상태는 이 창에 표시됩니다."
    } else if !success {
        "사용량 보호를 위해 작업 중단을 요청했습니다. 앱은 유지되며, 중단 확인 상태는 이 창에 표시됩니다."
    } else if reason == "sensor-unresponsive" {
        "사용량 감지기가 60초 동안 응답하지 않아, 사용량 한도 보호를 우선하여 Codex 작업을 중단했습니다."
    } else if reason == "usage-reset" {
        "사용량 초기화를 감지하여 새 할당량 보호를 위해 Codex 작업을 중단했습니다."
    } else {
        "사용량 감시기 상태 오류로 사용량 한도 보호를 우선하여 Codex 작업을 중단했습니다."
    }
}

fn failure(message: &str) -> AdapterError {
    AdapterError::Safety(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::notification_message;
    #[test]
    fn usage_monitor_unresponsive_notice_explains_protection_even_if_stop_is_unverified() {
        for stopped in [true, false] {
            let text = notification_message("sensor-unresponsive", stopped);
            assert!(text.contains("60초"));
            assert!(text.contains("사용량 한도 보호"));
            assert_eq!(text.contains("중단했습니다"), stopped);
        }
    }
}
