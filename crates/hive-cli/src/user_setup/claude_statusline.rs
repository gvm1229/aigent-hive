//! Consented Claude Code status-line connection for usage measurement (OV160-013 U4).
//!
//! Claude subscription usage is readable only from the status-line command input. This module
//! changes only the `statusLine` member of `<user-root>/.claude/settings.json`, and only after
//! the user approves the exact digest that a preview showed. An existing status-line command
//! keeps running: Hive passes the same input through to it. Other settings bytes stay unchanged,
//! and `revert` restores the previous state from a Hive-owned record.
use crate::{emit_action_result, ActionResult, Evidence};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SETTINGS_PATH: &str = ".claude/settings.json";
const STATE_PATH: &str = ".hive/state/claude-statusline.json";
const MAX_SETTINGS_BYTES: u64 = 1024 * 1024;
const CAPTURE_COMMAND: &str = "hive usage statusline --host claude";
const PREVIEW_COMMAND: &str = "hive setup --scope user --section usage --claude-statusline preview";
const DESKTOP_LIMIT: &str = "Claude desktop app Code tab: status-line execution is not yet verified; if no measurement arrives there, use Claude Code in a terminal or keep the missing-measurement warning";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ConnectionRecord {
    schema_version: u32,
    /// Exact settings bytes before Hive connected the status line; `None` when absent.
    original: Option<String>,
    /// Digest of the settings bytes that Hive wrote.
    applied_digest: String,
    previous_status_line: Option<Value>,
    connected_status_line: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    Preview,
    Apply,
    Revert,
}

struct Arguments {
    action: Action,
    confirm: Option<String>,
    root: PathBuf,
}

pub(crate) fn handles(arguments: &[String]) -> bool {
    arguments
        .iter()
        .any(|argument| argument == "--claude-statusline")
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    let result = parse(arguments)
        .and_then(|arguments| execute(&arguments))
        .unwrap_or_else(|(code, message)| failure(code, &message));
    emit_action_result(&result)
}

type Failure = (&'static str, String);

fn parse(arguments: &[String]) -> Result<Arguments, Failure> {
    let mut scope = None;
    let mut section = None;
    let mut action = None;
    let mut confirm = None;
    let mut root = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .cloned()
            .ok_or_else(|| input(format!("missing value for {option}")))?;
        let slot = match option {
            "--scope" => &mut scope,
            "--section" => &mut section,
            "--claude-statusline" => &mut action,
            "--confirm" => &mut confirm,
            "--user-root" => &mut root,
            "--output" if value == "json" => {
                index += 2;
                continue;
            }
            _ => return Err(input(format!("unknown status-line option: {option}"))),
        };
        if slot.replace(value).is_some() {
            return Err(input(format!("duplicate option: {option}")));
        }
        index += 2;
    }
    if scope.as_deref() != Some("user") || section.as_deref() != Some("usage") {
        return Err(input(
            "the Claude status line is set up with --scope user --section usage".to_owned(),
        ));
    }
    let action = match action.as_deref() {
        Some("preview") => Action::Preview,
        Some("apply") => Action::Apply,
        Some("revert") => Action::Revert,
        _ => {
            return Err(input(
                "--claude-statusline requires preview, apply or revert".to_owned(),
            ))
        }
    };
    if action == Action::Apply && confirm.is_none() {
        return Err(input(
            "apply requires --confirm <approval_digest> from the preview".to_owned(),
        ));
    }
    if action == Action::Preview && confirm.is_some() {
        return Err(input(
            "preview never changes settings; use apply --confirm".to_owned(),
        ));
    }
    let root = match root {
        Some(root) => PathBuf::from(root),
        None => {
            crate::user_install::resolve_user_root_path().map_err(|error| ("hive.input", error))?
        }
    };
    Ok(Arguments {
        action,
        confirm,
        root,
    })
}

fn input(message: String) -> Failure {
    ("hive.input", message)
}

fn conflict(message: &str) -> Failure {
    ("hive.claude-statusline-conflict", message.to_owned())
}

fn failure(code: &'static str, message: &str) -> ActionResult {
    let (status, exit_code) = if code == "hive.input" {
        ("error", 2)
    } else {
        ("blocked", 3)
    };
    ActionResult {
        schema_version: 1,
        action: "ConnectClaudeStatusLine",
        status,
        exit_code,
        code,
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

/// True when the status-line value is a Hive capture command (alone or chained).
fn is_connected(status_line: Option<&Value>) -> bool {
    status_line
        .and_then(|value| value.get("command"))
        .and_then(Value::as_str)
        .is_some_and(|command| command.starts_with(CAPTURE_COMMAND))
}

/// The connected value. An existing command receives the same input through a pipe, so it runs
/// in the same host shell with its own quoting unchanged.
fn connected_value(previous: Option<&Value>) -> Result<Value, Failure> {
    let Some(previous) = previous else {
        return Ok(json!({"type": "command", "command": CAPTURE_COMMAND}));
    };
    let command = previous
        .get("command")
        .and_then(Value::as_str)
        .filter(|command| !command.trim().is_empty())
        .ok_or_else(|| {
            conflict("the existing statusLine has no command to keep; edit it manually first")
        })?;
    let mut value = previous.clone();
    value["type"] = json!("command");
    value["command"] = json!(format!("{CAPTURE_COMMAND} --pass-through | {command}"));
    Ok(value)
}

struct Settings {
    bytes: Option<Vec<u8>>,
    status_line: Option<Value>,
}

fn read_settings(root: &cap_std::fs::Dir) -> Result<Settings, Failure> {
    let bytes = crate::user_install::read_user_setup_file(
        root,
        Path::new(SETTINGS_PATH),
        MAX_SETTINGS_BYTES,
    )
    .map_err(|error| conflict(&error))?;
    let status_line = match &bytes {
        Some(bytes) => {
            let value = serde_json::from_slice::<Value>(bytes)
                .map_err(|_| conflict("Claude settings.json is not valid JSON; nothing changed"))?;
            let object = value
                .as_object()
                .ok_or_else(|| conflict("Claude settings.json is not a JSON object"))?;
            object.get("statusLine").cloned()
        }
        None => None,
    };
    Ok(Settings { bytes, status_line })
}

fn read_record(root: &cap_std::fs::Dir) -> Result<Option<ConnectionRecord>, Failure> {
    crate::user_install::read_user_setup_file(root, Path::new(STATE_PATH), MAX_SETTINGS_BYTES * 2)
        .map_err(|error| conflict(&error))?
        .map(|bytes| {
            serde_json::from_slice::<ConnectionRecord>(&bytes)
                .map_err(|_| conflict("the Hive status-line record is damaged"))
        })
        .transpose()
}

fn optional_digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".to_owned(), sha256_digest)
}

fn approval_digest(action: Action, before: Option<&[u8]>, after: Option<&[u8]>) -> String {
    sha256_digest(
        json!({
            "schema": "hive-claude-statusline-v1",
            "action": if action == Action::Revert {"revert"} else {"apply"},
            "settings_path": SETTINGS_PATH,
            "before": optional_digest(before),
            "after": optional_digest(after),
        })
        .to_string()
        .as_bytes(),
    )
}

#[allow(clippy::too_many_lines)]
fn execute(arguments: &Arguments) -> Result<ActionResult, Failure> {
    let root = crate::user_install::open_user_root_for_setup(&arguments.root)
        .map_err(|error| conflict(&error))?;
    if !root.is_dir(".claude") {
        return Err(conflict(
            "Claude Code settings folder .claude was not found in the user root; start Claude Code once first",
        ));
    }
    let settings = read_settings(&root)?;
    let record = read_record(&root)?;
    let (after, record_change, summary) = if arguments.action == Action::Revert {
        let record =
            record.ok_or_else(|| conflict("Hive has not connected the Claude status line"))?;
        let current = settings.bytes.as_deref();
        let after = if optional_digest(current) == record.applied_digest {
            // Nothing else changed since apply: restore the exact original bytes.
            record
                .original
                .as_ref()
                .map(|text| text.as_bytes().to_vec())
        } else if settings.status_line.as_ref() == Some(&record.connected_status_line) {
            let text = std::str::from_utf8(current.unwrap_or_default())
                .map_err(|_| conflict("Claude settings.json is not UTF-8"))?;
            Some(
                splice_status_line(text, record.previous_status_line.as_ref())
                    .map_err(|message| conflict(&message))?
                    .into_bytes(),
            )
        } else {
            return Err(conflict(
                "the statusLine changed after Hive connected it; revert it manually",
            ));
        };
        (after, None, record.previous_status_line)
    } else {
        if is_connected(settings.status_line.as_ref()) {
            return Ok(ActionResult {
                schema_version: 1,
                action: "ConnectClaudeStatusLine",
                status: "success",
                exit_code: 0,
                code: "hive.claude-statusline-connected",
                message: "the Claude status line already records usage for Hive".to_owned(),
                changed_paths: Vec::new(),
                evidence: Vec::new(),
                next_action: None,
                data: Some(json!({
                    "settings_path": SETTINGS_PATH,
                    "status_line": settings.status_line,
                    "revert_command": "hive setup --scope user --section usage --claude-statusline revert",
                    "desktop_app_limit": DESKTOP_LIMIT,
                })),
            });
        }
        let connected = connected_value(settings.status_line.as_ref())?;
        let text = match settings.bytes.as_deref() {
            Some(bytes) => std::str::from_utf8(bytes)
                .map_err(|_| conflict("Claude settings.json is not UTF-8"))?
                .to_owned(),
            None => "{}\n".to_owned(),
        };
        let after = splice_status_line(&text, Some(&connected))
            .map_err(|message| conflict(&message))?
            .into_bytes();
        let record = ConnectionRecord {
            schema_version: 1,
            original: settings
                .bytes
                .as_deref()
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
            applied_digest: sha256_digest(&after),
            previous_status_line: settings.status_line.clone(),
            connected_status_line: connected.clone(),
        };
        (Some(after), Some(record), Some(connected))
    };
    let digest = approval_digest(
        arguments.action,
        settings.bytes.as_deref(),
        after.as_deref(),
    );
    let action_name = if arguments.action == Action::Revert {
        "revert"
    } else {
        "apply"
    };
    let data = json!({
        "settings_path": SETTINGS_PATH,
        "action": action_name,
        "current_status_line": settings.status_line,
        "proposed_status_line": summary,
        "chains_existing_command": arguments.action != Action::Revert && settings.status_line.is_some(),
        "before_digest": optional_digest(settings.bytes.as_deref()),
        "after_digest": optional_digest(after.as_deref()),
        "approval_digest": digest,
        "other_settings_preserved": true,
        "revert_command": "hive setup --scope user --section usage --claude-statusline revert",
        "desktop_app_limit": DESKTOP_LIMIT,
    });
    let confirmed = arguments.action == Action::Apply
        || (arguments.action == Action::Revert && arguments.confirm.is_some());
    if !confirmed {
        return Ok(ActionResult {
            schema_version: 1,
            action: "ConnectClaudeStatusLine",
            status: "success",
            exit_code: 0,
            code: "hive.claude-statusline-preview",
            message: format!(
                "preview only: show the proposed statusLine to the user and {action_name} only after approval"
            ),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: Some(format!(
                "hive setup --scope user --section usage --claude-statusline {action_name} --confirm {digest}"
            )),
            data: Some(data),
        });
    }
    if arguments.confirm.as_deref() != Some(digest.as_str()) {
        return Err(conflict(
            "the approval digest does not match the current settings; run the preview again",
        ));
    }
    // Keep the revert record before the host file changes, so a failed write stays recoverable.
    if let Some(record) = &record_change {
        {
            let bytes = serde_json::to_vec_pretty(record)
                .map_err(|error| ("hive.internal", error.to_string()))?;
            root.create_dir_all(".hive/state")
                .map_err(|error| conflict(&error.to_string()))?;
            let existing = crate::user_install::read_user_setup_file(
                &root,
                Path::new(STATE_PATH),
                MAX_SETTINGS_BYTES * 2,
            )
            .map_err(|error| conflict(&error))?;
            crate::user_install::replace_user_setup_file(
                &root,
                Path::new(STATE_PATH),
                existing.as_deref(),
                Some(&bytes),
            )
            .map_err(|error| conflict(&error))?;
        }
    }
    crate::user_install::replace_user_setup_file(
        &root,
        Path::new(SETTINGS_PATH),
        settings.bytes.as_deref(),
        after.as_deref(),
    )
    .map_err(|error| conflict(&error))?;
    if arguments.action == Action::Revert {
        let existing = crate::user_install::read_user_setup_file(
            &root,
            Path::new(STATE_PATH),
            MAX_SETTINGS_BYTES * 2,
        )
        .map_err(|error| conflict(&error))?;
        crate::user_install::replace_user_setup_file(
            &root,
            Path::new(STATE_PATH),
            existing.as_deref(),
            None,
        )
        .map_err(|error| conflict(&error))?;
    }
    Ok(ActionResult {
        schema_version: 1,
        action: "ConnectClaudeStatusLine",
        status: "success",
        exit_code: 0,
        code: if arguments.action == Action::Revert {
            "hive.claude-statusline-reverted"
        } else {
            "hive.claude-statusline-connected"
        },
        message: if arguments.action == Action::Revert {
            "the previous Claude status line is restored".to_owned()
        } else {
            "the Claude status line now records usage for Hive; the first measurement arrives after the next Claude response".to_owned()
        },
        changed_paths: vec![SETTINGS_PATH.to_owned(), STATE_PATH.to_owned()],
        evidence: vec![Evidence {
            kind: "file",
            locator: SETTINGS_PATH.to_owned(),
            digest: optional_digest(after.as_deref()),
        }],
        next_action: None,
        data: Some(data),
    })
}

/// Add the connection offer to a completed usage-section result when Claude is not connected.
pub(crate) fn offer(root: &Path, result: &mut ActionResult) {
    let Ok(dir) = crate::user_install::open_user_root_for_setup(root) else {
        return;
    };
    if !dir.is_dir(".claude") {
        return;
    }
    let Ok(settings) = read_settings(&dir) else {
        return;
    };
    let connected = is_connected(settings.status_line.as_ref());
    if let Some(data) = result.data.get_or_insert_with(|| json!({})).as_object_mut() {
        data.insert(
            "claude_statusline".to_owned(),
            json!({
                "connected": connected,
                "preview_command": PREVIEW_COMMAND,
                "reason": "Claude usage is measurable only from the status-line input",
                "desktop_app_limit": DESKTOP_LIMIT,
            }),
        );
    }
    if !connected && result.next_action.is_none() {
        result.next_action = Some(format!(
            "to measure Claude usage, review the status-line connection: {PREVIEW_COMMAND}"
        ));
    }
}

/// Replace, insert or remove only the top-level `statusLine` member of a JSON object text.
fn splice_status_line(text: &str, value: Option<&Value>) -> Result<String, String> {
    let members = top_level_members(text)?;
    let rendered = value
        .map(|value| {
            serde_json::to_string_pretty(value)
                .map(|pretty| pretty.replace('\n', "\n  "))
                .map_err(|error| error.to_string())
        })
        .transpose()?;
    let existing = members.iter().find(|member| member.key == "statusLine");
    let output = match (existing, rendered) {
        (Some(member), Some(rendered)) => format!(
            "{}{}{}",
            &text[..member.value_start],
            rendered,
            &text[member.value_end..]
        ),
        (Some(member), None) => {
            let (start, end) = match (member.next_key_start, member.previous_value_end) {
                (Some(next), _) => (member.key_start, next),
                (None, Some(previous)) => (previous, member.value_end),
                (None, None) => (member.key_start, member.value_end),
            };
            format!("{}{}", &text[..start], &text[end..])
        }
        (None, Some(rendered)) => {
            let close = members.close;
            if members.is_empty() {
                format!(
                    "{}\n  \"statusLine\": {}\n{}",
                    &text[..=members.open],
                    rendered,
                    &text[close..]
                )
            } else {
                let last_end = members.last().map_or(close, |member| member.value_end);
                format!(
                    "{},\n  \"statusLine\": {}{}",
                    &text[..last_end],
                    rendered,
                    &text[last_end..]
                )
            }
        }
        (None, None) => text.to_owned(),
    };
    // The spliced text must stay valid JSON with every other member unchanged.
    let before = serde_json::from_str::<Value>(text).map_err(|error| error.to_string())?;
    let after = serde_json::from_str::<Value>(&output)
        .map_err(|_| "the status-line change would break settings.json".to_owned())?;
    let (Some(before), Some(after)) = (before.as_object(), after.as_object()) else {
        return Err("Claude settings.json is not a JSON object".to_owned());
    };
    let others_same = before
        .iter()
        .filter(|(key, _)| key.as_str() != "statusLine")
        .all(|(key, old)| after.get(key) == Some(old))
        && after
            .keys()
            .filter(|key| key.as_str() != "statusLine")
            .count()
            == before
                .keys()
                .filter(|key| key.as_str() != "statusLine")
                .count();
    if !others_same || after.get("statusLine") != value {
        return Err("the status-line change would alter other settings".to_owned());
    }
    Ok(output)
}

pub(super) struct Member {
    pub(super) key: String,
    pub(super) key_start: usize,
    pub(super) value_start: usize,
    pub(super) value_end: usize,
    previous_value_end: Option<usize>,
    next_key_start: Option<usize>,
}

pub(super) struct Members {
    pub(super) open: usize,
    pub(super) close: usize,
    list: Vec<Member>,
}

impl std::ops::Deref for Members {
    type Target = Vec<Member>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

pub(super) fn top_level_members(text: &str) -> Result<Members, String> {
    let bytes = text.as_bytes();
    let invalid = || "Claude settings.json is not a simple JSON object".to_owned();
    let mut index = skip_space(bytes, 0);
    if bytes.get(index) != Some(&b'{') {
        return Err(invalid());
    }
    let open = index;
    index += 1;
    let mut list: Vec<Member> = Vec::new();
    loop {
        index = skip_space(bytes, index);
        match bytes.get(index) {
            Some(b'}') => {
                return Ok(Members {
                    open,
                    close: index,
                    list,
                })
            }
            Some(b'"') => {}
            _ => return Err(invalid()),
        }
        let key_start = index;
        let key_end = skip_string(bytes, index).ok_or_else(invalid)?;
        let key =
            serde_json::from_str::<String>(&text[key_start..key_end]).map_err(|_| invalid())?;
        index = skip_space(bytes, key_end);
        if bytes.get(index) != Some(&b':') {
            return Err(invalid());
        }
        let value_start = skip_space(bytes, index + 1);
        let value_end = skip_value(bytes, value_start).ok_or_else(invalid)?;
        let previous_value_end = list.last().map(|member| member.value_end);
        if let Some(previous) = list.last_mut() {
            previous.next_key_start = Some(key_start);
        }
        list.push(Member {
            key,
            key_start,
            value_start,
            value_end,
            previous_value_end,
            next_key_start: None,
        });
        index = skip_space(bytes, value_end);
        match bytes.get(index) {
            Some(b',') => index += 1,
            Some(b'}') => {}
            _ => return Err(invalid()),
        }
    }
}

pub(super) fn skip_space(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
    {
        index += 1;
    }
    index
}

fn skip_string(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 1;
    while let Some(byte) = bytes.get(index) {
        match byte {
            b'\\' => index += 2,
            b'"' => return Some(index + 1),
            _ => index += 1,
        }
    }
    None
}

pub(super) fn skip_value(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes.get(start)? {
        b'"' => skip_string(bytes, start),
        b'{' | b'[' => {
            let mut depth = 0_usize;
            let mut index = start;
            while let Some(byte) = bytes.get(index) {
                match byte {
                    b'"' => {
                        index = skip_string(bytes, index)?;
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(index + 1);
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            None
        }
        _ => {
            let mut index = start;
            while bytes.get(index).is_some_and(|byte| {
                !matches!(byte, b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n')
            }) {
                index += 1;
            }
            (index > start).then_some(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(root: &Path, action: &str, confirm: Option<&str>) -> Vec<String> {
        let mut values = vec![
            "--scope",
            "user",
            "--section",
            "usage",
            "--claude-statusline",
            action,
            "--user-root",
            root.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        if let Some(confirm) = confirm {
            values.extend(["--confirm".to_owned(), confirm.to_owned()]);
        }
        values
    }

    fn call(root: &Path, action: &str, confirm: Option<&str>) -> ActionResult {
        parse(&arguments(root, action, confirm))
            .and_then(|parsed| execute(&parsed))
            .unwrap_or_else(|(code, message)| failure(code, &message))
    }

    #[test]
    fn preview_changes_nothing_and_apply_requires_the_exact_digest() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".claude")).unwrap();
        let preview = call(root.path(), "preview", None);
        assert_eq!(preview.code, "hive.claude-statusline-preview");
        assert!(!root.path().join(SETTINGS_PATH).exists());
        let data = preview.data.unwrap();
        assert_eq!(
            data["proposed_status_line"]["command"],
            "hive usage statusline --host claude"
        );
        assert!(data["desktop_app_limit"]
            .as_str()
            .unwrap()
            .contains("desktop app"));
        let wrong = call(root.path(), "apply", Some("sha256:wrong"));
        assert_eq!(wrong.code, "hive.claude-statusline-conflict");
        assert!(!root.path().join(SETTINGS_PATH).exists());
        let digest = data["approval_digest"].as_str().unwrap();
        let applied = call(root.path(), "apply", Some(digest));
        assert_eq!(applied.code, "hive.claude-statusline-connected");
        let written: Value =
            serde_json::from_slice(&std::fs::read(root.path().join(SETTINGS_PATH)).unwrap())
                .unwrap();
        assert_eq!(
            written["statusLine"]["command"],
            "hive usage statusline --host claude"
        );
        // Revert of a created file removes it again.
        let revert = call(root.path(), "revert", None);
        assert_eq!(revert.code, "hive.claude-statusline-preview");
        let digest = revert.data.unwrap()["approval_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        let reverted = call(root.path(), "revert", Some(&digest));
        assert_eq!(reverted.code, "hive.claude-statusline-reverted");
        assert!(!root.path().join(SETTINGS_PATH).exists());
        assert!(!root.path().join(STATE_PATH).exists());
    }

    #[test]
    fn existing_status_line_is_chained_and_other_bytes_are_preserved() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".claude")).unwrap();
        let original = "{\n    \"model\": \"opus\",\n    \"statusLine\": {\"type\": \"command\", \"command\": \"bash ~/.claude/line.sh\", \"padding\": 1},\n    \"zeta\": [1, {\"a\": \"}\"}]\n}\n";
        std::fs::write(root.path().join(SETTINGS_PATH), original).unwrap();
        let preview = call(root.path(), "preview", None);
        let data = preview.data.unwrap();
        assert_eq!(data["chains_existing_command"], true);
        let applied = call(
            root.path(),
            "apply",
            Some(data["approval_digest"].as_str().unwrap()),
        );
        assert_eq!(applied.exit_code, 0, "{}", applied.message);
        let written = std::fs::read_to_string(root.path().join(SETTINGS_PATH)).unwrap();
        assert!(written.starts_with("{\n    \"model\": \"opus\",\n    \"statusLine\": "));
        assert!(written.ends_with(",\n    \"zeta\": [1, {\"a\": \"}\"}]\n}\n"));
        let parsed: Value = serde_json::from_str(&written).unwrap();
        assert_eq!(
            parsed["statusLine"]["command"],
            "hive usage statusline --host claude --pass-through | bash ~/.claude/line.sh"
        );
        assert_eq!(parsed["statusLine"]["padding"], 1);
        // A second preview reports the connection instead of chaining twice.
        assert_eq!(
            call(root.path(), "preview", None).code,
            "hive.claude-statusline-connected"
        );
        let revert = call(root.path(), "revert", None);
        let digest = revert.data.unwrap()["approval_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        call(root.path(), "revert", Some(&digest));
        assert_eq!(
            std::fs::read_to_string(root.path().join(SETTINGS_PATH)).unwrap(),
            original
        );
    }

    #[test]
    fn revert_after_other_edits_restores_only_the_status_line() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".claude")).unwrap();
        std::fs::write(root.path().join(SETTINGS_PATH), "{\"model\": \"opus\"}").unwrap();
        let preview = call(root.path(), "preview", None).data.unwrap();
        call(
            root.path(),
            "apply",
            Some(preview["approval_digest"].as_str().unwrap()),
        );
        let text = std::fs::read_to_string(root.path().join(SETTINGS_PATH)).unwrap();
        std::fs::write(
            root.path().join(SETTINGS_PATH),
            text.replace("\"opus\"", "\"sonnet\""),
        )
        .unwrap();
        let revert = call(root.path(), "revert", None).data.unwrap();
        let done = call(
            root.path(),
            "revert",
            Some(revert["approval_digest"].as_str().unwrap()),
        );
        assert_eq!(done.code, "hive.claude-statusline-reverted");
        let parsed: Value =
            serde_json::from_slice(&std::fs::read(root.path().join(SETTINGS_PATH)).unwrap())
                .unwrap();
        assert_eq!(parsed, json!({"model": "sonnet"}));
    }

    #[test]
    fn missing_claude_folder_and_invalid_json_change_nothing() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            call(root.path(), "preview", None).code,
            "hive.claude-statusline-conflict"
        );
        std::fs::create_dir(root.path().join(".claude")).unwrap();
        std::fs::write(root.path().join(SETTINGS_PATH), "{not json").unwrap();
        assert_eq!(
            call(root.path(), "preview", None).code,
            "hive.claude-statusline-conflict"
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join(SETTINGS_PATH)).unwrap(),
            "{not json"
        );
    }
}
