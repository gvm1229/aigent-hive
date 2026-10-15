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

/// Every invocation is bounded; model calls and credentials are never accepted here.
pub(super) fn call(request: &Value, timeout: Duration) -> Result<Value, AdapterError> {
    if !cfg!(windows) {
        return Err(AdapterError::Unsupported(
            "live usage interruption supports Windows Codex only".into(),
        ));
    }
    let system_root =
        std::env::var_os("SystemRoot").ok_or_else(|| failure("SystemRoot missing"))?;
    let executable = Path::new(&system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let encoded = base64::engine::general_purpose::STANDARD.encode(
        SCRIPT
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    let mut command = Command::new(executable);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &encoded,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let mut child = command
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

pub(super) fn notify(reason: &str, success: bool, incident: &str) -> mpsc::Receiver<bool> {
    let message = notification_message(reason, success);
    let message = format!("{message}\n재개: hive usage monitor acknowledge --user-root <사용자 루트> --incident {incident} --output json");
    // The UI notification must never delay termination or its controller heartbeat.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(
            call(
                &json!({"operation":"notify", "message":message}),
                Duration::from_mins(5),
            )
            .is_ok(),
        );
    });
    rx
}

fn notification_message(reason: &str, success: bool) -> &'static str {
    if !success && reason == "sensor-unresponsive" {
        "사용량 감지기가 30초 동안 응답하지 않아 사용량 한도 보호를 우선해 중단을 요청했지만, 일부 실행의 종료를 확인하지 못했습니다. Codex를 직접 종료해 주세요."
    } else if !success {
        "사용량 보호 중단을 요청했지만 일부 실행의 종료를 확인하지 못했습니다. Codex를 직접 종료해 주세요."
    } else if reason == "sensor-unresponsive" {
        "사용량 감지기가 30초 동안 응답하지 않아, 사용량 한도 보호를 우선하여 Codex 작업을 중단했습니다."
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
            assert!(text.contains("30초"));
            assert!(text.contains("사용량 한도 보호"));
            assert_eq!(text.contains("중단했습니다"), stopped);
        }
    }
}
