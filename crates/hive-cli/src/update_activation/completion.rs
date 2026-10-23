use super::{Language, INSTALL_OUTPUT_LIMIT, INSTALL_TIMEOUT};
use process_wrap::std::CommandWrap;
use serde::Deserialize;
use serde_json::Value;
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
pub(super) struct Report {
    action: String,
    pub status: String,
    pub exit_code: u8,
    pub code: String,
    pub message: String,
    pub data: Option<Value>,
    pub next_action: Option<String>,
}

impl Report {
    pub fn succeeded(&self) -> bool {
        self.status == "success" && self.exit_code == 0
    }

    pub fn summary(&self, language: Language, hosts: &[String]) -> (Language, Vec<String>) {
        let Some(data) = &self.data else {
            return (language, hosts.to_vec());
        };
        let language = match data["interface_language"].as_str() {
            Some("ko") => Language::Ko,
            Some("en") => Language::En,
            _ => language,
        };
        let hosts = data["selected_hosts"].as_array().map_or_else(
            || hosts.to_vec(),
            |values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_owned))
                    .collect()
            },
        );
        (language, hosts)
    }
}

fn decode(bytes: &[u8], success: bool) -> Result<Report, String> {
    let report: Report =
        serde_json::from_slice(bytes).map_err(|_| "update completion returned invalid JSON")?;
    if report.action != "UpdateHiveUser" || success != (report.exit_code == 0) {
        return Err("update completion action or exit status differs".into());
    }
    if report.succeeded() {
        let hosts = report
            .data
            .as_ref()
            .and_then(|data| data["selected_hosts"].as_array())
            .ok_or("update completion omitted verified hosts")?;
        if hosts.is_empty()
            || hosts.len() > 3
            || hosts
                .iter()
                .filter_map(Value::as_str)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != hosts.len()
            || hosts
                .iter()
                .any(|host| !matches!(host.as_str(), Some("codex" | "claude" | "antigravity")))
        {
            return Err("update completion reported invalid hosts".into());
        }
    }
    Ok(report)
}

pub(super) fn run(executable: &Path, root: &Path, interactive: bool) -> Result<Report, String> {
    let mut command = Command::new(executable);
    command
        .args(["__complete-user-update", "--user-root"])
        .arg(root)
        .args([
            "--interaction",
            if interactive {
                "terminal"
            } else {
                "non-interactive"
            },
        ])
        .stdin(if interactive {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut wrapped = CommandWrap::from(command);
    #[cfg(windows)]
    {
        if !interactive {
            #[allow(clippy::default_trait_access)]
            let mut flags = process_wrap::std::CreationFlags(Default::default());
            flags.0 .0 = 0x0800_0000;
            wrapped.wrap(flags);
        }
        wrapped.wrap(process_wrap::std::JobObject);
    }
    #[cfg(unix)]
    if !interactive {
        wrapped.wrap(process_wrap::std::ProcessGroup::leader());
    }
    let mut child = wrapped
        .spawn()
        .map_err(|_| "cannot start the activated update completion")?;
    let output = child
        .stdout()
        .take()
        .ok_or("update completion output unavailable")?;
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let valid = output
            .take(INSTALL_OUTPUT_LIMIT as u64 + 1)
            .read_to_end(&mut bytes)
            .is_ok()
            && bytes.len() <= INSTALL_OUTPUT_LIMIT;
        let _ = send.send(valid.then_some(bytes));
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if interactive || started.elapsed() < INSTALL_TIMEOUT => {
                thread::sleep(Duration::from_millis(25));
            }
            _ => {
                let _ = child.start_kill();
                return Err("update completion interrupted; resume with hive update".into());
            }
        }
    };
    let bytes = receive
        .recv_timeout(Duration::from_secs(5))
        .ok()
        .flatten()
        .ok_or("update completion result is incomplete")?;
    decode(&bytes, status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn completion_requires_matching_action_exit_and_verified_host_scope() {
        let mut value = json!({"action":"UpdateHiveUser","status":"success","exit_code":0,"code":"hive.user-update-complete",
            "message":"done","data":{"interface_language":"ko","selected_hosts":["codex"]},"next_action":null});
        let report = decode(&serde_json::to_vec(&value).unwrap(), true).unwrap();
        assert_eq!(
            report.summary(Language::En, &[]),
            (Language::Ko, vec!["codex".into()])
        );
        assert!(decode(&serde_json::to_vec(&value).unwrap(), false).is_err());
        value["action"] = json!("UnknownAction");
        assert!(decode(&serde_json::to_vec(&value).unwrap(), true).is_err());
        value["action"] = json!("UpdateHiveUser");
        value["data"]["selected_hosts"] = json!(["foreign"]);
        assert!(decode(&serde_json::to_vec(&value).unwrap(), true).is_err());
        value["data"]["selected_hosts"] = json!(["codex", "codex"]);
        assert!(decode(&serde_json::to_vec(&value).unwrap(), true).is_err());
    }
}
