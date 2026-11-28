//! Consented Claude Code enforcement of the preview-browser choice (OV170-022).
//!
//! The guidance block asks agents to use the chosen browser; this optional layer makes Claude Code
//! refuse the other browser's tools. It adds entries only to `permissions.deny` of
//! `<user-root>/.claude/settings.json`, only after the user approves the exact digest that a
//! preview showed, and only after a verified copy in `<user-root>/hive-backups`. Every other
//! settings byte stays unchanged, and `revert` removes only the entries Hive added, from a
//! Hive-owned record.
use super::claude_statusline::{skip_space, skip_value, top_level_members, Members};
use crate::user_setup::{BrowserPreferences, BrowserTarget};
use crate::{emit_action_result, ActionResult, Evidence};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SETTINGS_PATH: &str = ".claude/settings.json";
const STATE_PATH: &str = ".hive/state/claude-browser-deny.json";
const MAX_SETTINGS_BYTES: u64 = 1024 * 1024;
const OPTION: &str = "--claude-browser-deny";
const COMMAND: &str = "hive setup --scope user --section tools --claude-browser-deny";
const IN_APP_TOOLS: &str = "mcp__Claude_Browser__*";
const IN_APP_NEW_TAB: &str = "mcp__Claude_Browser__tabs_create";
const CHROME_TOOLS: &str = "mcp__claude-in-chrome__*";
const CHROME_NEW_TAB: &str = "mcp__claude-in-chrome__tabs_create_mcp";
const ACTION_NAME: &str = "EnforceClaudeBrowserChoice";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DenyRecord {
    schema_version: u32,
    /// Exact settings bytes before Hive added its entries; `None` when the file was absent.
    original: Option<String>,
    /// Digest of the settings bytes that Hive wrote.
    applied_digest: String,
    /// Only the entries Hive added; entries the user already had are never recorded.
    added: Vec<String>,
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

type Failure = (&'static str, String);

pub(crate) fn handles(arguments: &[String]) -> bool {
    arguments.iter().any(|argument| argument == OPTION)
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    let result = parse(arguments)
        .and_then(|arguments| execute(&arguments))
        .unwrap_or_else(|(code, message)| failure(code, &message));
    emit_action_result(&result)
}

/// The deny entries for a browser choice, in a fixed order. A new-tab tool already covered by
/// the wildcard of the denied browser is not repeated.
fn deny_entries(browser: &BrowserPreferences) -> Vec<String> {
    let mut entries = Vec::new();
    match browser.target {
        BrowserTarget::DefaultBrowser => {
            entries.push(IN_APP_TOOLS);
            if browser.reuse_active_tab {
                entries.push(CHROME_NEW_TAB);
            }
        }
        BrowserTarget::InApp => {
            entries.push(CHROME_TOOLS);
            if browser.reuse_active_tab {
                entries.push(IN_APP_NEW_TAB);
            }
        }
    }
    entries.into_iter().map(str::to_owned).collect()
}

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
            OPTION => &mut action,
            "--confirm" => &mut confirm,
            "--user-root" => &mut root,
            "--output" if value == "json" => {
                index += 2;
                continue;
            }
            _ => {
                return Err(input(format!(
                    "unknown browser-enforcement option: {option}"
                )))
            }
        };
        if slot.replace(value).is_some() {
            return Err(input(format!("duplicate option: {option}")));
        }
        index += 2;
    }
    if scope.as_deref() != Some("user") || section.as_deref() != Some("tools") {
        return Err(input(
            "the Claude browser enforcement is set up with --scope user --section tools".to_owned(),
        ));
    }
    let action = match action.as_deref() {
        Some("preview") => Action::Preview,
        Some("apply") => Action::Apply,
        Some("revert") => Action::Revert,
        _ => return Err(input(format!("{OPTION} requires preview, apply or revert"))),
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
    ("hive.claude-browser-deny-conflict", message.to_owned())
}

fn failure(code: &'static str, message: &str) -> ActionResult {
    let (status, exit_code) = if code == "hive.input" {
        ("error", 2)
    } else {
        ("blocked", 3)
    };
    ActionResult {
        schema_version: 1,
        action: ACTION_NAME,
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

fn success(code: &'static str, message: String, data: Value) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: ACTION_NAME,
        status: "success",
        exit_code: 0,
        code,
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(data),
    }
}

struct Settings {
    bytes: Option<Vec<u8>>,
    text: String,
    /// String entries of `permissions.deny`; `None` when the member is absent.
    deny: Option<Vec<String>>,
}

fn read_settings(root: &cap_std::fs::Dir) -> Result<Settings, Failure> {
    let bytes = crate::user_install::read_user_setup_file(
        root,
        Path::new(SETTINGS_PATH),
        MAX_SETTINGS_BYTES,
    )
    .map_err(|error| conflict(&error))?;
    let Some(raw) = &bytes else {
        return Ok(Settings {
            bytes,
            text: "{}\n".to_owned(),
            deny: None,
        });
    };
    let text = std::str::from_utf8(raw)
        .map_err(|_| conflict("Claude settings.json is not UTF-8"))?
        .to_owned();
    let value = serde_json::from_str::<Value>(&text)
        .map_err(|_| conflict("Claude settings.json is not valid JSON; nothing changed"))?;
    let object = value
        .as_object()
        .ok_or_else(|| conflict("Claude settings.json is not a JSON object"))?;
    let deny = match object.get("permissions") {
        None => None,
        Some(Value::Object(permissions)) => match permissions.get("deny") {
            None => None,
            Some(Value::Array(items)) => Some(
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            ),
            Some(_) => return Err(conflict(
                "permissions.deny in Claude settings.json is not a list; edit it manually first",
            )),
        },
        Some(_) => {
            return Err(conflict(
                "permissions in Claude settings.json is not an object; edit it manually first",
            ))
        }
    };
    Ok(Settings { bytes, text, deny })
}

fn read_record(root: &cap_std::fs::Dir) -> Result<Option<DenyRecord>, Failure> {
    crate::user_install::read_user_setup_file(root, Path::new(STATE_PATH), MAX_SETTINGS_BYTES * 2)
        .map_err(|error| conflict(&error))?
        .map(|bytes| {
            serde_json::from_slice::<DenyRecord>(&bytes)
                .map_err(|_| conflict("the Hive browser-enforcement record is damaged"))
        })
        .transpose()
}

fn optional_digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".to_owned(), sha256_digest)
}

fn approval_digest(
    action: Action,
    entries: &[String],
    before: Option<&[u8]>,
    after: Option<&[u8]>,
) -> String {
    sha256_digest(
        json!({
            "schema": "hive-claude-browser-deny-v1",
            "action": if action == Action::Revert {"revert"} else {"apply"},
            "settings_path": SETTINGS_PATH,
            "entries": entries,
            "before": optional_digest(before),
            "after": optional_digest(after),
        })
        .to_string()
        .as_bytes(),
    )
}

fn load_browser(root: &cap_std::fs::Dir) -> Result<BrowserPreferences, Failure> {
    crate::user_setup::load_operational_config(root)
        .map_err(|error| conflict(error.message()))?
        .map(|config| config.browser)
        .ok_or_else(|| {
            conflict("global Hive setup is required first: hive setup --scope user --section tools")
        })
}

fn execute(arguments: &Arguments) -> Result<ActionResult, Failure> {
    let root = crate::user_install::open_user_root_for_setup(&arguments.root)
        .map_err(|error| conflict(&error))?;
    let browser = load_browser(&root)?;
    execute_with(arguments, &root, &browser)
}

struct Plan {
    after: Option<Vec<u8>>,
    entries: Vec<String>,
    record: Option<DenyRecord>,
}

fn plan_apply(
    settings: &Settings,
    record: Option<&DenyRecord>,
    desired: &[String],
) -> Result<Option<Plan>, Failure> {
    let present = settings.deny.clone().unwrap_or_default();
    let missing: Vec<String> = desired
        .iter()
        .filter(|entry| !present.contains(entry))
        .cloned()
        .collect();
    if let Some(record) = record {
        if !missing.is_empty() || record.added.iter().any(|entry| !desired.contains(entry)) {
            return Err(conflict(&format!(
                "Hive already added browser deny entries for another choice or they were edited; run `{COMMAND} revert` first, then preview again"
            )));
        }
    }
    if missing.is_empty() {
        return Ok(None);
    }
    let after = add_entries(&settings.text, &missing)
        .map_err(|message| conflict(&message))?
        .into_bytes();
    let record = DenyRecord {
        schema_version: 1,
        original: settings
            .bytes
            .as_deref()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
        applied_digest: sha256_digest(&after),
        added: missing.clone(),
    };
    Ok(Some(Plan {
        after: Some(after),
        entries: missing,
        record: Some(record),
    }))
}

fn plan_revert(settings: &Settings, record: &DenyRecord) -> Result<Plan, Failure> {
    let current = settings.bytes.as_deref();
    let after = if optional_digest(current) == record.applied_digest {
        // Nothing else changed since apply: restore the exact original bytes.
        record
            .original
            .as_ref()
            .map(|text| text.as_bytes().to_vec())
    } else {
        let present = settings.deny.clone().unwrap_or_default();
        let still: Vec<String> = record
            .added
            .iter()
            .filter(|entry| present.contains(entry))
            .cloned()
            .collect();
        if still.is_empty() {
            current.map(<[u8]>::to_vec)
        } else {
            Some(
                remove_entries(&settings.text, &still)
                    .map_err(|message| conflict(&message))?
                    .into_bytes(),
            )
        }
    };
    Ok(Plan {
        after,
        entries: record.added.clone(),
        record: None,
    })
}

#[allow(clippy::too_many_lines)]
fn execute_with(
    arguments: &Arguments,
    root: &cap_std::fs::Dir,
    browser: &BrowserPreferences,
) -> Result<ActionResult, Failure> {
    if !root.is_dir(".claude") {
        return Err(conflict(
            "Claude Code settings folder .claude was not found in the user root; start Claude Code once first",
        ));
    }
    let settings = read_settings(root)?;
    let record = read_record(root)?;
    let desired = deny_entries(browser);
    let target = match browser.target {
        BrowserTarget::DefaultBrowser => "default-browser",
        BrowserTarget::InApp => "in-app",
    };
    let revert_command = format!("{COMMAND} revert");
    let plan = if arguments.action == Action::Revert {
        let record = record.as_ref().ok_or_else(|| {
            conflict("Hive has not added browser deny entries to Claude settings")
        })?;
        plan_revert(&settings, record)?
    } else {
        let Some(plan) = plan_apply(&settings, record.as_ref(), &desired)? else {
            return Ok(success(
                "hive.claude-browser-deny-current",
                "Claude settings already deny the browser tools for this choice".to_owned(),
                json!({
                    "settings_path": SETTINGS_PATH,
                    "browser_target": target,
                    "reuse_active_tab": browser.reuse_active_tab,
                    "deny_entries": desired,
                    "hive_owned": record.is_some(),
                    "revert_command": record.as_ref().map(|_| revert_command.clone()),
                }),
            ));
        };
        plan
    };
    let digest = approval_digest(
        arguments.action,
        &plan.entries,
        settings.bytes.as_deref(),
        plan.after.as_deref(),
    );
    let action_name = if arguments.action == Action::Revert {
        "revert"
    } else {
        "apply"
    };
    let mut data = json!({
        "settings_path": SETTINGS_PATH,
        "action": action_name,
        "browser_target": target,
        "reuse_active_tab": browser.reuse_active_tab,
        "member": "permissions.deny",
        if arguments.action == Action::Revert {"removed_entries"} else {"added_entries"}: plan.entries,
        "existing_deny_entries": settings.deny.clone().unwrap_or_default(),
        "before_digest": optional_digest(settings.bytes.as_deref()),
        "after_digest": optional_digest(plan.after.as_deref()),
        "approval_digest": digest,
        "other_settings_preserved": true,
        "backup_root": "hive-backups",
        "revert_command": revert_command,
    });
    let confirmed = arguments.action == Action::Apply
        || (arguments.action == Action::Revert && arguments.confirm.is_some());
    if !confirmed {
        let mut result = success(
            "hive.claude-browser-deny-preview",
            format!(
                "preview only: show the deny entries to the user and {action_name} only after approval; a verified backup is taken first"
            ),
            data,
        );
        result.next_action = Some(format!("{COMMAND} {action_name} --confirm {digest}"));
        return Ok(result);
    }
    if arguments.confirm.as_deref() != Some(digest.as_str()) {
        return Err(conflict(
            "the approval digest does not match the current settings; run the preview again",
        ));
    }
    let backup = crate::user_install::backup::before_host_settings_change(
        &arguments.root,
        "claude-browser-deny",
        SETTINGS_PATH,
    )
    .map_err(|error| conflict(&format!("{error}; nothing was changed")))?;
    if let Some(object) = data.as_object_mut() {
        object.insert(
            "backup".to_owned(),
            json!(backup
                .as_deref()
                .map(|path| path.to_string_lossy().replace('\\', "/"))),
        );
    }
    // Keep the revert record before the host file changes, so a failed write stays recoverable.
    if let Some(record) = &plan.record {
        let bytes = serde_json::to_vec_pretty(record)
            .map_err(|error| ("hive.internal", error.to_string()))?;
        root.create_dir_all(".hive/state")
            .map_err(|error| conflict(&error.to_string()))?;
        let existing = crate::user_install::read_user_setup_file(
            root,
            Path::new(STATE_PATH),
            MAX_SETTINGS_BYTES * 2,
        )
        .map_err(|error| conflict(&error))?;
        crate::user_install::replace_user_setup_file(
            root,
            Path::new(STATE_PATH),
            existing.as_deref(),
            Some(&bytes),
        )
        .map_err(|error| conflict(&error))?;
    }
    if plan.after.as_deref() != settings.bytes.as_deref() {
        crate::user_install::replace_user_setup_file(
            root,
            Path::new(SETTINGS_PATH),
            settings.bytes.as_deref(),
            plan.after.as_deref(),
        )
        .map_err(|error| conflict(&error))?;
    }
    if arguments.action == Action::Revert {
        let existing = crate::user_install::read_user_setup_file(
            root,
            Path::new(STATE_PATH),
            MAX_SETTINGS_BYTES * 2,
        )
        .map_err(|error| conflict(&error))?;
        crate::user_install::replace_user_setup_file(
            root,
            Path::new(STATE_PATH),
            existing.as_deref(),
            None,
        )
        .map_err(|error| conflict(&error))?;
    }
    let reverted = arguments.action == Action::Revert;
    let mut result = success(
        if reverted {
            "hive.claude-browser-deny-reverted"
        } else {
            "hive.claude-browser-deny-applied"
        },
        if reverted {
            "Hive's browser deny entries are removed from Claude settings".to_owned()
        } else {
            "Claude settings now deny the browser tools outside your choice; restart Claude Code sessions to apply".to_owned()
        },
        data,
    );
    result.changed_paths = vec![SETTINGS_PATH.to_owned(), STATE_PATH.to_owned()];
    result.evidence = vec![Evidence {
        kind: "file",
        locator: SETTINGS_PATH.to_owned(),
        digest: optional_digest(plan.after.as_deref()),
    }];
    Ok(result)
}

/// Add the enforcement offer to a completed tools-section result when Claude is present.
pub(crate) fn offer(root: &Path, result: &mut ActionResult) {
    let Ok(dir) = crate::user_install::open_user_root_for_setup(root) else {
        return;
    };
    if !dir.is_dir(".claude") {
        return;
    }
    let (Ok(browser), Ok(settings)) = (load_browser(&dir), read_settings(&dir)) else {
        return;
    };
    let present = settings.deny.unwrap_or_default();
    let enforced = deny_entries(&browser)
        .iter()
        .all(|entry| present.contains(entry));
    let preview = format!("{COMMAND} preview");
    if let Some(data) = result.data.get_or_insert_with(|| json!({})).as_object_mut() {
        data.insert(
            "claude_browser_deny".to_owned(),
            json!({
                "enforced": enforced,
                "preview_command": preview,
                "reason": "optional: make Claude Code refuse the browser tools outside your choice",
            }),
        );
    }
    if !enforced && result.next_action.is_none() {
        result.next_action = Some(format!(
            "optional: to make Claude Code follow the browser choice, review the deny entries: {preview}"
        ));
    }
}

fn render(value: &Value, indent: &str) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map(|pretty| pretty.replace('\n', &format!("\n{indent}")))
        .map_err(|error| error.to_string())
}

/// Insert a member at the end of the object `text[members.open..=members.close]`, copying the
/// indentation of the first existing member.
fn insert_member(
    text: &str,
    members: &Members,
    key: &str,
    value: &Value,
    top: bool,
) -> Result<String, String> {
    let key = serde_json::to_string(key).map_err(|error| error.to_string())?;
    if let Some(first) = members.first() {
        let gap = &text[members.open + 1..first.key_start];
        let rendered = match gap.rfind('\n') {
            Some(newline) => render(value, &gap[newline + 1..])?,
            None => value.to_string(),
        };
        let last_end = members
            .last()
            .map_or(members.close, |member| member.value_end);
        Ok(format!(
            "{},{gap}{key}: {rendered}{}",
            &text[..last_end],
            &text[last_end..]
        ))
    } else if top {
        Ok(format!(
            "{}\n  {key}: {}\n{}",
            &text[..=members.open],
            render(value, "  ")?,
            &text[members.close..]
        ))
    } else {
        Ok(format!(
            "{}{{{key}: {value}}}{}",
            &text[..members.open],
            &text[members.close + 1..]
        ))
    }
}

/// Absolute byte ranges of `permissions` and of `permissions.deny`, when present.
type Span = (usize, usize);

fn locate(text: &str) -> Result<(Members, Option<Span>, Option<Span>), String> {
    let members = top_level_members(text)?;
    let Some(permissions) = members.iter().find(|member| member.key == "permissions") else {
        return Ok((members, None, None));
    };
    let (start, end) = (permissions.value_start, permissions.value_end);
    if !text[start..end].starts_with('{') {
        return Err("permissions in Claude settings.json is not an object".to_owned());
    }
    let inner = top_level_members(&text[start..end])?;
    let deny = match inner.iter().find(|member| member.key == "deny") {
        Some(deny) => {
            if !text[start + deny.value_start..].starts_with('[') {
                return Err("permissions.deny in Claude settings.json is not a list".to_owned());
            }
            Some((start + deny.value_start, start + deny.value_end))
        }
        None => None,
    };
    Ok((members, Some((start, end)), deny))
}

/// Append `entries` to `permissions.deny`, creating either member when absent. Every other byte
/// stays as it was, which the parsed comparison below proves.
fn add_entries(text: &str, entries: &[String]) -> Result<String, String> {
    let (members, permissions, deny) = locate(text)?;
    let output = match (permissions, deny) {
        (None, _) => insert_member(
            text,
            &members,
            "permissions",
            &json!({"deny": entries}),
            true,
        )?,
        (Some((start, end)), None) => {
            let object = &text[start..end];
            let inner = top_level_members(object)?;
            let updated = insert_member(object, &inner, "deny", &json!(entries), false)?;
            format!("{}{updated}{}", &text[..start], &text[end..])
        }
        (Some(_), Some((start, end))) => {
            let array = &text[start..end];
            let close = array.len() - 1;
            let content_end = array[..close].trim_end().len();
            let quoted: Vec<String> = entries
                .iter()
                .map(|entry| json!(entry).to_string())
                .collect();
            let updated = if content_end == 1 {
                format!("[{}{}", quoted.join(", "), &array[content_end..])
            } else {
                let first = skip_space(array.as_bytes(), 1);
                let gap = &array[1..first];
                let separator = if gap.contains('\n') {
                    format!(",{gap}")
                } else {
                    ", ".to_owned()
                };
                let mut added = String::new();
                for entry in &quoted {
                    added.push_str(&separator);
                    added.push_str(entry);
                }
                format!("{}{added}{}", &array[..content_end], &array[content_end..])
            };
            format!("{}{updated}{}", &text[..start], &text[end..])
        }
    };
    let mut expected = serde_json::from_str::<Value>(text).map_err(|error| error.to_string())?;
    let permissions = expected
        .as_object_mut()
        .ok_or("Claude settings.json is not a JSON object")?
        .entry("permissions")
        .or_insert_with(|| json!({}));
    let deny = permissions
        .as_object_mut()
        .ok_or("permissions is not an object")?
        .entry("deny")
        .or_insert_with(|| json!([]));
    let list = deny
        .as_array_mut()
        .ok_or("permissions.deny is not a list")?;
    list.extend(entries.iter().map(|entry| json!(entry)));
    verify(&output, &expected)?;
    Ok(output)
}

/// Remove the first occurrence of each entry from `permissions.deny`, leaving every other byte.
fn remove_entries(text: &str, entries: &[String]) -> Result<String, String> {
    let mut output = text.to_owned();
    let mut expected = serde_json::from_str::<Value>(text).map_err(|error| error.to_string())?;
    for entry in entries {
        let (_, _, deny) = locate(&output)?;
        let Some((start, end)) = deny else {
            break;
        };
        let elements = array_elements(output.as_bytes(), start)
            .ok_or("permissions.deny in Claude settings.json is not a simple list")?;
        let Some(index) = elements.iter().position(|(from, to)| {
            serde_json::from_str::<Value>(&output[*from..*to]).ok() == Some(json!(entry))
        }) else {
            continue;
        };
        let (from, to) = if elements.len() == 1 {
            (start + 1, end - 1)
        } else if index > 0 {
            (elements[index - 1].1, elements[index].1)
        } else {
            (elements[0].0, elements[1].0)
        };
        output.replace_range(from..to, "");
        if let Some(list) = expected
            .pointer_mut("/permissions/deny")
            .and_then(Value::as_array_mut)
        {
            if let Some(position) = list.iter().position(|item| item == &json!(entry)) {
                list.remove(position);
            }
        }
    }
    verify(&output, &expected)?;
    Ok(output)
}

fn verify(output: &str, expected: &Value) -> Result<(), String> {
    let parsed = serde_json::from_str::<Value>(output)
        .map_err(|_| "the deny-entry change would break settings.json".to_owned())?;
    if &parsed != expected {
        return Err("the deny-entry change would alter other settings".to_owned());
    }
    Ok(())
}

/// Byte ranges of each element of the JSON array that starts at `start`.
fn array_elements(bytes: &[u8], start: usize) -> Option<Vec<(usize, usize)>> {
    let mut elements = Vec::new();
    let mut index = skip_space(bytes, start + 1);
    if bytes.get(index) == Some(&b']') {
        return Some(elements);
    }
    loop {
        let end = skip_value(bytes, index)?;
        elements.push((index, end));
        index = skip_space(bytes, end);
        match bytes.get(index)? {
            b',' => index = skip_space(bytes, index + 1),
            b']' => return Some(elements),
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preferences(target: BrowserTarget, reuse_active_tab: bool) -> BrowserPreferences {
        BrowserPreferences {
            target,
            reuse_active_tab,
            detected_default: None,
        }
    }

    fn call(
        root: &Path,
        browser: &BrowserPreferences,
        action: &str,
        confirm: Option<&str>,
    ) -> ActionResult {
        let mut values = vec![
            "--scope",
            "user",
            "--section",
            "tools",
            OPTION,
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
        parse(&values)
            .and_then(|parsed| {
                let dir = crate::user_install::open_user_root_for_setup(&parsed.root)
                    .map_err(|error| conflict(&error))?;
                execute_with(&parsed, &dir, browser)
            })
            .unwrap_or_else(|(code, message)| failure(code, &message))
    }

    fn digest(result: &ActionResult) -> String {
        result.data.as_ref().unwrap()["approval_digest"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn seeded(settings: Option<&str>) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".claude")).unwrap();
        if let Some(settings) = settings {
            std::fs::write(root.path().join(SETTINGS_PATH), settings).unwrap();
        }
        root
    }

    fn read(root: &Path) -> String {
        std::fs::read_to_string(root.join(SETTINGS_PATH)).unwrap()
    }

    #[test]
    fn entries_follow_target_and_tab_choice_in_a_fixed_order() {
        assert_eq!(
            deny_entries(&preferences(BrowserTarget::DefaultBrowser, true)),
            [IN_APP_TOOLS, CHROME_NEW_TAB]
        );
        assert_eq!(
            deny_entries(&preferences(BrowserTarget::DefaultBrowser, false)),
            [IN_APP_TOOLS]
        );
        assert_eq!(
            deny_entries(&preferences(BrowserTarget::InApp, true)),
            [CHROME_TOOLS, IN_APP_NEW_TAB]
        );
        assert_eq!(
            deny_entries(&preferences(BrowserTarget::InApp, false)),
            [CHROME_TOOLS]
        );
    }

    #[test]
    fn preview_shows_entries_and_apply_requires_the_exact_digest() {
        let root = seeded(None);
        let browser = preferences(BrowserTarget::DefaultBrowser, true);
        let preview = call(root.path(), &browser, "preview", None);
        assert_eq!(preview.code, "hive.claude-browser-deny-preview");
        let data = preview.data.as_ref().unwrap();
        assert_eq!(
            data["added_entries"],
            json!([
                "mcp__Claude_Browser__*",
                "mcp__claude-in-chrome__tabs_create_mcp"
            ])
        );
        assert!(data["approval_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
        assert!(preview
            .next_action
            .as_deref()
            .unwrap()
            .contains(&digest(&preview)));
        assert!(!root.path().join(SETTINGS_PATH).exists());

        let wrong = call(root.path(), &browser, "apply", Some("sha256:wrong"));
        assert_eq!(wrong.code, "hive.claude-browser-deny-conflict");
        assert!(!root.path().join(SETTINGS_PATH).exists());
        assert!(!root.path().join("hive-backups").exists());

        let applied = call(root.path(), &browser, "apply", Some(&digest(&preview)));
        assert_eq!(
            applied.code, "hive.claude-browser-deny-applied",
            "{}",
            applied.message
        );
        let written: Value = serde_json::from_str(&read(root.path())).unwrap();
        assert_eq!(
            written,
            json!({"permissions": {"deny": ["mcp__Claude_Browser__*", "mcp__claude-in-chrome__tabs_create_mcp"]}})
        );
        assert!(root.path().join(STATE_PATH).exists());

        let revert = call(root.path(), &browser, "revert", None);
        assert_eq!(revert.code, "hive.claude-browser-deny-preview");
        let reverted = call(root.path(), &browser, "revert", Some(&digest(&revert)));
        assert_eq!(reverted.code, "hive.claude-browser-deny-reverted");
        assert!(!root.path().join(SETTINGS_PATH).exists());
        assert!(!root.path().join(STATE_PATH).exists());
    }

    #[test]
    fn apply_preserves_other_bytes_and_existing_entries_and_revert_restores_them() {
        let original = "{\n    \"model\": \"opus\",\n    \"permissions\": {\n        \"allow\": [\"Bash(ls:*)\"],\n        \"deny\": [\n            \"Bash(rm:*)\",\n            \"mcp__Claude_Browser__*\"\n        ]\n    },\n    \"zeta\": [1, {\"a\": \"]\"}]\n}\n";
        let root = seeded(Some(original));
        let browser = preferences(BrowserTarget::DefaultBrowser, true);
        let preview = call(root.path(), &browser, "preview", None);
        let data = preview.data.as_ref().unwrap();
        // The user's own wildcard entry is kept and not recorded as Hive's.
        assert_eq!(
            data["added_entries"],
            json!(["mcp__claude-in-chrome__tabs_create_mcp"])
        );
        let applied = call(root.path(), &browser, "apply", Some(&digest(&preview)));
        assert_eq!(applied.exit_code, 0, "{}", applied.message);
        let backup = applied.data.as_ref().unwrap()["backup"].as_str().unwrap();
        assert!(backup.starts_with("hive-backups/claude-browser-deny-"));
        assert_eq!(
            std::fs::read_to_string(root.path().join(backup).join("files").join(SETTINGS_PATH))
                .unwrap(),
            original
        );
        let written = read(root.path());
        assert_eq!(
            written,
            original.replace(
                "\"mcp__Claude_Browser__*\"\n",
                "\"mcp__Claude_Browser__*\",\n            \"mcp__claude-in-chrome__tabs_create_mcp\"\n"
            )
        );

        // A second preview changes nothing.
        let again = call(root.path(), &browser, "preview", None);
        assert_eq!(again.code, "hive.claude-browser-deny-current");
        assert_eq!(read(root.path()), written);

        let revert = call(root.path(), &browser, "revert", None);
        assert_eq!(
            revert.data.as_ref().unwrap()["removed_entries"],
            json!(["mcp__claude-in-chrome__tabs_create_mcp"])
        );
        call(root.path(), &browser, "revert", Some(&digest(&revert)));
        assert_eq!(read(root.path()), original);
    }

    #[test]
    fn in_app_without_tab_reuse_adds_one_entry_and_revert_after_edits_removes_only_it() {
        let root = seeded(Some(
            "{\"model\": \"opus\", \"permissions\": {\"deny\": [\"Read(./.env)\"]}}",
        ));
        let browser = preferences(BrowserTarget::InApp, false);
        let preview = call(root.path(), &browser, "preview", None);
        assert_eq!(
            preview.data.as_ref().unwrap()["added_entries"],
            json!(["mcp__claude-in-chrome__*"])
        );
        call(root.path(), &browser, "apply", Some(&digest(&preview)));
        assert_eq!(
            read(root.path()),
            "{\"model\": \"opus\", \"permissions\": {\"deny\": [\"Read(./.env)\", \"mcp__claude-in-chrome__*\"]}}"
        );
        std::fs::write(
            root.path().join(SETTINGS_PATH),
            read(root.path()).replace("\"opus\"", "\"sonnet\""),
        )
        .unwrap();
        let revert = call(root.path(), &browser, "revert", None);
        let done = call(root.path(), &browser, "revert", Some(&digest(&revert)));
        assert_eq!(
            done.code, "hive.claude-browser-deny-reverted",
            "{}",
            done.message
        );
        assert_eq!(
            read(root.path()),
            "{\"model\": \"sonnet\", \"permissions\": {\"deny\": [\"Read(./.env)\"]}}"
        );
    }

    #[test]
    fn missing_members_are_created_and_a_changed_choice_needs_revert_first() {
        let root = seeded(Some("{\n  \"permissions\": {\n    \"allow\": []\n  }\n}\n"));
        let browser = preferences(BrowserTarget::InApp, true);
        let preview = call(root.path(), &browser, "preview", None);
        call(root.path(), &browser, "apply", Some(&digest(&preview)));
        assert_eq!(
            read(root.path()),
            "{\n  \"permissions\": {\n    \"allow\": [],\n    \"deny\": [\n      \"mcp__claude-in-chrome__*\",\n      \"mcp__Claude_Browser__tabs_create\"\n    ]\n  }\n}\n"
        );
        let switched = preferences(BrowserTarget::DefaultBrowser, true);
        assert_eq!(
            call(root.path(), &switched, "preview", None).code,
            "hive.claude-browser-deny-conflict"
        );
    }

    #[test]
    fn invalid_settings_and_missing_claude_folder_change_nothing() {
        let browser = BrowserPreferences::default();
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            call(root.path(), &browser, "preview", None).code,
            "hive.claude-browser-deny-conflict"
        );
        let root = seeded(Some("{\"permissions\": {\"deny\": \"all\"}}"));
        assert_eq!(
            call(root.path(), &browser, "preview", None).code,
            "hive.claude-browser-deny-conflict"
        );
        assert_eq!(read(root.path()), "{\"permissions\": {\"deny\": \"all\"}}");
        assert_eq!(
            call(root.path(), &browser, "revert", None).code,
            "hive.claude-browser-deny-conflict"
        );
    }

    #[test]
    fn options_require_the_tools_section_and_a_confirmed_apply() {
        let root = tempfile::tempdir().unwrap();
        let base = |section: &str, action: &str| {
            vec![
                "--scope".to_owned(),
                "user".to_owned(),
                "--section".to_owned(),
                section.to_owned(),
                OPTION.to_owned(),
                action.to_owned(),
                "--user-root".to_owned(),
                root.path().to_str().unwrap().to_owned(),
            ]
        };
        assert!(parse(&base("usage", "preview")).is_err());
        assert!(parse(&base("tools", "apply")).is_err());
        assert!(parse(&base("tools", "enable")).is_err());
        assert!(parse(&base("tools", "preview")).is_ok());
    }
}
