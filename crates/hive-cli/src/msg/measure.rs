//! Worker model measurement for `hive msg ack --worker <provider>:<id>`.
//!
//! The role table only suggests a model; the completion notice must name the model that really
//! ran. The host passes the worker's own session id, and Hive opens only that host's session
//! log under the user root (`.codex/sessions/**/rollout-*-<id>.jsonl` or
//! `.claude/projects/*/*/subagents/agent-<id>.jsonl`). Only the model and effort fields are read;
//! no prompt, answer, tool output or path is kept. The record holds the provider, the last model
//! and effort the log reports, and the log digest.
use super::MsgError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const MEASUREMENT_SCHEMA_VERSION: u32 = 1;
const MAX_LOG_BYTES: u64 = 128 * 1024 * 1024;
const MAX_FIELD_CHARS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkerHost {
    Codex,
    Claude,
}

impl WorkerHost {
    fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}

/// `codex:<session id>` or `claude:<subagent id>`.
pub(super) fn parse_worker(spec: &str) -> Result<(WorkerHost, String), MsgError> {
    let (host, id) = spec
        .split_once(':')
        .ok_or_else(|| MsgError::input("--worker is codex:<session-id> or claude:<agent-id>"))?;
    let host = match host {
        "codex" => WorkerHost::Codex,
        "claude" => WorkerHost::Claude,
        _ => return Err(MsgError::input("--worker host is codex or claude")),
    };
    let plain = (6..=MAX_FIELD_CHARS).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    if !plain {
        return Err(MsgError::input(
            "--worker id is 6-64 letters, digits or hyphens",
        ));
    }
    Ok((host, id.to_owned()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Measurement {
    pub schema_version: u32,
    pub message_id: String,
    pub provider: String,
    pub model: String,
    pub effort: Option<String>,
    pub source: String,
    pub log_digest: String,
    pub measured_at: u64,
}

impl Measurement {
    /// `Codex gpt-6-luna · medium`, or without effort when the log has none.
    pub fn label(&self) -> String {
        let provider = hive_core::flow::provider_display_name(&self.provider);
        let model = hive_core::flow::model_display_name(&self.model);
        match &self.effort {
            Some(effort) => format!("{provider} {model} · {effort}"),
            None => format!("{provider} {model}"),
        }
    }
}

/// `.hive/mediation/measurements/<message_id>.json`
pub(super) fn measurement_path(message_id: &str) -> String {
    format!(
        "{}/measurements/{message_id}.json",
        hive_core::mediation::MEDIATION_ROOT
    )
}

/// Find the one log for this worker id, read its model fields, and build the record.
pub(super) fn measure(
    user_root: &Path,
    host: WorkerHost,
    id: &str,
    message_id: &str,
    now: u64,
) -> Result<Measurement, String> {
    let path = locate(user_root, host, id)?;
    let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("worker log is not a regular file".into());
    }
    if meta.len() > MAX_LOG_BYTES {
        return Err("worker log is larger than 128 MiB".into());
    }
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    let (model, effort) =
        read_model(host, &bytes).ok_or_else(|| "worker log names no model".to_owned())?;
    Ok(Measurement {
        schema_version: MEASUREMENT_SCHEMA_VERSION,
        message_id: message_id.to_owned(),
        provider: host.as_str().to_owned(),
        model,
        effort,
        source: "host-session-log".into(),
        log_digest: hive_core::sha256_digest(&bytes),
        measured_at: now,
    })
}

fn locate(user_root: &Path, host: WorkerHost, id: &str) -> Result<PathBuf, String> {
    let found = match host {
        WorkerHost::Codex => {
            let suffix = format!("-{id}.jsonl");
            nested(&user_root.join(".codex/sessions"), 3)
                .into_iter()
                .flat_map(|day| children(&day))
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(&suffix))
                })
                .collect::<Vec<_>>()
        }
        WorkerHost::Claude => {
            let name = format!("agent-{id}.jsonl");
            nested(&user_root.join(".claude/projects"), 2)
                .into_iter()
                .map(|session| session.join("subagents").join(&name))
                .filter(|path| path.is_file())
                .collect()
        }
    };
    match found.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(format!(
            "no {} session log for this worker id",
            host.as_str()
        )),
        _ => Err("more than one session log matches this worker id".into()),
    }
}

/// Directories exactly `depth` levels below `root`, never following symlinks.
fn nested(root: &Path, depth: usize) -> Vec<PathBuf> {
    let mut level = vec![root.to_path_buf()];
    for _ in 0..depth {
        level = level
            .iter()
            .flat_map(|dir| children(dir))
            .filter(|path| fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir()))
            .collect();
    }
    level
}

fn children(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir).map_or_else(
        |_| Vec::new(),
        |entries| entries.flatten().map(|entry| entry.path()).collect(),
    )
}

/// Last model and effort the log reports: Codex `turn_context` payloads, Claude assistant
/// messages (the model the service answered with). Synthetic or malformed values are skipped.
fn read_model(host: WorkerHost, bytes: &[u8]) -> Option<(String, Option<String>)> {
    let text = String::from_utf8_lossy(bytes);
    let mut last = None;
    for line in text.lines() {
        let marker = match host {
            WorkerHost::Codex => "\"turn_context\"",
            WorkerHost::Claude => "\"assistant\"",
        };
        if !line.contains(marker) {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let (model, effort) = match host {
            WorkerHost::Codex if value["type"] == "turn_context" => {
                (&value["payload"]["model"], &value["payload"]["effort"])
            }
            WorkerHost::Claude if value["type"] == "assistant" => {
                (&value["message"]["model"], &value["effort"])
            }
            _ => continue,
        };
        if let Some(model) = model.as_str().filter(|model| plain_field(model)) {
            let effort = effort
                .as_str()
                .filter(|e| plain_field(e))
                .map(str::to_owned);
            last = Some((model.to_owned(), effort));
        }
    }
    last
}

fn plain_field(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FIELD_CHARS
        && !value.starts_with('<')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_spec_accepts_only_known_hosts_and_plain_ids() {
        assert_eq!(
            parse_worker("codex:01a1fcf9-2499-78f1").unwrap(),
            (WorkerHost::Codex, "01a1fcf9-2499-78f1".to_owned())
        );
        assert!(parse_worker("claude:a26b8a461d6733984").is_ok());
        for bad in [
            "gemini:abcdef",
            "codex:../x",
            "claude:abc",
            "codex",
            "claude:a b c d e f",
        ] {
            assert!(parse_worker(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn reads_the_last_reported_model_and_effort_only() {
        let codex = concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"model\":\"ignored\"}}\n",
            "{\"type\":\"turn_context\",\"payload\":{\"model\":\"gpt-6-luna\",\"effort\":\"low\"}}\n",
            "{\"type\":\"turn_context\",\"payload\":{\"model\":\"gpt-6-luna\",\"effort\":\"medium\"}}\n",
        );
        assert_eq!(
            read_model(WorkerHost::Codex, codex.as_bytes()),
            Some(("gpt-6-luna".into(), Some("medium".into())))
        );
        let claude = concat!(
            "{\"type\":\"user\",\"message\":{\"content\":\"assistant\"}}\n",
            "{\"type\":\"assistant\",\"effort\":\"max\",\"message\":{\"model\":\"claude-opus-5-5\"}}\n",
            "{\"type\":\"assistant\",\"message\":{\"model\":\"<synthetic>\"}}\n",
        );
        assert_eq!(
            read_model(WorkerHost::Claude, claude.as_bytes()),
            Some(("claude-opus-5-5".into(), Some("max".into())))
        );
        assert_eq!(
            read_model(WorkerHost::Claude, b"{\"type\":\"assistant\"}\n"),
            None
        );
    }

    #[test]
    fn locates_only_the_matching_host_log_under_the_user_root() {
        let root = tempfile::tempdir().unwrap();
        let day = root.path().join(".codex/sessions/2026/11/25");
        fs::create_dir_all(&day).unwrap();
        fs::write(
            day.join("rollout-2026-11-25T01-02-03-0123abcd-ef.jsonl"),
            "{\"type\":\"turn_context\",\"payload\":{\"model\":\"gpt-6-sol\",\"effort\":\"high\"}}\n",
        )
        .unwrap();
        let sub = root.path().join(".claude/projects/p/s/subagents");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            sub.join("agent-a1b2c3d4e5.jsonl"),
            "{\"type\":\"assistant\",\"message\":{\"model\":\"claude-sonnet-5-5\"}}\n",
        )
        .unwrap();
        let codex = measure(root.path(), WorkerHost::Codex, "0123abcd-ef", "msg-1", 9).unwrap();
        assert_eq!(codex.label(), "Codex gpt-6-sol · high");
        assert!(codex.log_digest.starts_with("sha256:"));
        let claude = measure(root.path(), WorkerHost::Claude, "a1b2c3d4e5", "msg-1", 9).unwrap();
        assert_eq!(claude.label(), "Claude Sonnet 5.5");
        assert!(measure(root.path(), WorkerHost::Claude, "missing99", "msg-1", 9).is_err());
    }
}
