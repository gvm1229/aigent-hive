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
const NOTICE: &str = include_str!("notice.ps1");
const LOGO: &[u8] =
    include_bytes!("../../../../harness/plugins/aigent-hive/assets/hive-logo-plugin.png");

pub(super) fn command(script: &str) -> Result<Command, AdapterError> {
    command_scoped(script, None)
}

pub(super) fn valid_thread_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

pub(super) fn command_scoped(script: &str, scope: Option<&str>) -> Result<Command, AdapterError> {
    if scope.is_some_and(|id| !valid_thread_id(id)) {
        return Err(failure("invalid qualification thread identity"));
    }
    if !cfg!(windows) {
        return Err(AdapterError::Unsupported(
            "live usage interruption supports Windows Codex only".into(),
        ));
    }
    let system_root =
        std::env::var_os("SystemRoot").ok_or_else(|| failure("SystemRoot missing"))?;
    let executable = Path::new(&system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    // Source bytes are fixed product data. Put bounded chunks in this child's
    // environment so the command line stays small as the adapter grows.
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(script.replace("\r\n", "\n").as_bytes());
    if encoded.len() > 262_144 {
        return Err(failure("embedded Windows adapter exceeds source limit"));
    }
    let chunks = encoded.as_bytes().chunks(16000).collect::<Vec<_>>();
    let scope_argument =
        scope.map_or_else(String::new, |id| format!(" -QualificationThread '{id}'"));
    let bootstrap = format!("$p=[Text.StringBuilder]::new();for($i=0;$i -lt {};$i++){{[void]$p.Append([Environment]::GetEnvironmentVariable('AIGENT_HIVE_FIXED_PS_'+$i))}};& ([scriptblock]::Create([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($p.ToString())))){scope_argument}",chunks.len());
    let mut command = Command::new(executable);
    for (index, chunk) in chunks.iter().enumerate() {
        command.env(
            format!("AIGENT_HIVE_FIXED_PS_{index}"),
            String::from_utf8_lossy(chunk).as_ref(),
        );
    }
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
    let reason = reason.to_owned();
    // The separate UI must never delay interruption or the controller heartbeat.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let delivered = (|| {
            let mut child = command(NOTICE).ok()?.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
            let request = json!({"operation":"notify","message":message,"reason":reason,"user_root":root,"incident":incident,"detected_at":super::now(),
                "logo_png_base64":base64::engine::general_purpose::STANDARD.encode(LOGO)});
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
    if !success && reason == "usage-threshold" {
        "설정한 사용량 하한에 도달하여 해당 Codex 작업의 중단을 요청했습니다. 앱은 유지되며, 실제 중단 확인 상태는 안내 창에 표시됩니다."
    } else if success && reason == "usage-threshold" {
        "설정한 사용량 하한에 도달하여 해당 Codex 작업을 중단했습니다. Codex에서 직접 재개할 수 있습니다."
    } else if !success && reason == "sensor-unresponsive" {
        "사용량 감지기가 60초 동안 응답하지 않아 사용량 한도 보호를 우선해 작업 중단을 요청했습니다. 앱은 유지되며, 중단 확인 상태는 이 창에 표시됩니다."
    } else if !success && reason == "usage-reset" {
        "사용량 초기화를 감지하여 새 할당량 보호를 위해 Codex 작업 중단을 요청했습니다. 앱은 유지되며, 확인 상태는 안내 창에 표시됩니다."
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
    #[cfg(windows)]
    #[test]
    fn fixed_source_larger_than_the_windows_argument_limit_launches_without_truncation() {
        let source = format!(
            "#{}\n[Console]::Out.Write('payload-ok')",
            "x".repeat(40_000)
        );
        let output = super::command(&source).unwrap().output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"payload-ok");
    }
    #[test]
    fn usage_monitor_unresponsive_notice_explains_protection_even_if_stop_is_unverified() {
        for stopped in [true, false] {
            let text = notification_message("sensor-unresponsive", stopped);
            assert!(text.contains("60초"));
            assert!(text.contains("사용량 한도 보호"));
            assert_eq!(text.contains("중단했습니다"), stopped);
        }
    }

    #[test]
    fn notice_has_its_own_bounded_windows_launch_payload() {
        if cfg!(windows) {
            assert!(super::command(super::SCRIPT).is_ok());
            assert!(super::command(super::NOTICE).is_ok());
        }
    }

    #[test]
    fn reset_notice_names_the_cause_without_claiming_unconfirmed_success() {
        let text = notification_message("usage-reset", false);
        assert!(text.contains("사용량 초기화"));
        assert!(text.contains("중단을 요청"));
        assert!(!text.contains("중단했습니다"));
    }

    #[test]
    fn threshold_fallback_names_the_floor_without_inventing_a_monitor_fault() {
        for stopped in [false, true] {
            let text = notification_message("usage-threshold", stopped);
            assert!(text.contains("하한"));
            assert!(!text.contains("오류"));
            assert_eq!(text.contains("중단했습니다"), stopped);
        }
    }
}
