//! `hive msg hooks`: project-only official hook configuration for message mediation (OV160-003).
//!
//! Preview is read-only and prints an approval digest. Apply and remove change only the
//! project host file (`.claude/settings.local.json`, `.codex/hooks.json` or
//! `.gemini/settings.json`) and the Hive receipt `.hive/config/mediation-hooks/<host>.json`,
//! and only with the exact current preview digest. User-global `~/.claude`, `~/.codex` and
//! `~/.gemini` settings, hooks and credentials are never read or written. Nothing is enabled
//! until an explicit apply. Foreign hook entries and bytes are preserved; only the exact Hive
//! entries recorded in the receipt are inserted or removed.
use super::{option, Context, MsgError, ACTION};
use crate::run::{parse_options, required};
use crate::user_install::{host_policy_mode, read_user_setup_file, replace_host_policy_file};
use crate::ActionResult;
use hive_core::hook_events::{self, HookHost, ReturnShape, HOOK_EVENT_SCHEMA_VERSION};
use hive_core::sha256_digest;
use hive_projection::hook_config::{contains_path, edit_entry, prune_empty};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const LIMIT: u64 = 1024 * 1024;
const HOOK_TIMEOUT_SECONDS: u64 = 10;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Entry {
    pub(super) path: Vec<String>,
    pub(super) value: Value,
}

/// Approved project hook configuration; the receipt the hook command is bound to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HookIntent {
    pub(super) schema_version: u32,
    pub(super) host: HookHost,
    pub(super) role: String,
    pub(super) task: Option<String>,
    pub(super) stop_shape: Option<ReturnShape>,
    pub(super) target_digest: String,
    pub(super) user_root_digest: String,
    pub(super) binding_digest: String,
    pub(super) operation: String,
    pub(super) config_path: String,
    pub(super) before_digest: String,
    pub(super) after_digest: String,
    pub(super) before: Vec<Entry>,
    pub(super) after: Vec<Entry>,
    pub(super) created_config: bool,
    pub(super) created_containers: Vec<Vec<String>>,
    pub(super) config_mode: Option<u32>,
    pub(super) approval_digest: String,
}

impl HookIntent {
    fn encoded(&self) -> Result<Vec<u8>, MsgError> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| {
            MsgError::new("error", 10, "hive.internal-error", error.to_string())
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    fn computed_approval(&self) -> Result<String, MsgError> {
        let mut unsealed = self.clone();
        unsealed.approval_digest.clear();
        Ok(sha256_digest(&unsealed.encoded()?))
    }
}

fn digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".to_owned(), sha256_digest)
}

pub(super) fn config_path(host: HookHost) -> &'static str {
    match host {
        HookHost::Claude => ".claude/settings.local.json",
        HookHost::Codex => ".codex/hooks.json",
        HookHost::Gemini => ".gemini/settings.json",
    }
}

pub(super) fn receipt_path(host: HookHost) -> String {
    format!(".hive/config/mediation-hooks/{}.json", host.as_str())
}

pub(super) fn path_digest(path: &Path) -> String {
    sha256_digest(path.to_string_lossy().as_bytes())
}

/// Absolute user root exactly as the hook command will pass it.
pub(super) fn user_root(options: &[(&str, &str)]) -> Result<PathBuf, MsgError> {
    std::path::absolute(required(options, "--user-root")?)
        .map_err(|error| MsgError::input(format!("cannot resolve --user-root: {error}")))
}

fn read(context: &Context, path: &str) -> Result<Option<Vec<u8>>, MsgError> {
    read_user_setup_file(context.target.target_dir(), Path::new(path), LIMIT)
        .map_err(|message| MsgError::blocked("hive.msg-path-refused", message))
}

/// Read and self-check the approved receipt for `host`.
pub(super) fn installed(context: &Context, host: HookHost) -> Result<Option<HookIntent>, MsgError> {
    let Some(bytes) = read(context, &receipt_path(host))? else {
        return Ok(None);
    };
    let intent: HookIntent = serde_json::from_slice(&bytes)
        .map_err(|_| refused("the mediation hook receipt is unreadable"))?;
    if intent.schema_version != HOOK_EVENT_SCHEMA_VERSION
        || intent.host != host
        || intent.config_path != config_path(host)
        || intent.approval_digest != intent.computed_approval()?
    {
        return Err(refused(
            "the mediation hook receipt does not match its own approval digest",
        ));
    }
    Ok(Some(intent))
}

fn refused(message: impl Into<String>) -> MsgError {
    MsgError::blocked("hive.msg-hooks-refused", message)
}

fn quote_command(arguments: &[&str]) -> Result<String, MsgError> {
    if arguments
        .iter()
        .any(|argument| argument.contains(['\r', '\n', '\0']))
    {
        return Err(refused("hook command contains control characters"));
    }
    if cfg!(windows) {
        if arguments
            .iter()
            .any(|argument| argument.contains(['"', '$', '`', '%', '!']))
        {
            return Err(refused(
                "hook path contains characters unsupported by the Windows command adapter",
            ));
        }
        let quoted: Vec<String> = arguments
            .iter()
            .map(|argument| format!("'{}'", argument.replace('\'', "''")))
            .collect();
        Ok(format!(
            "powershell.exe -NoProfile -NonInteractive -Command \"& {}\"",
            quoted.join(" ")
        ))
    } else {
        let quoted: Vec<String> = arguments
            .iter()
            .map(|argument| format!("'{}'", argument.replace('\'', "'\\''")))
            .collect();
        Ok(quoted.join(" "))
    }
}

/// One Hive entry per allowlisted event of the host.
fn entries(
    host: HookHost,
    role: &str,
    task: Option<&str>,
    binding: &str,
    target: &Path,
    user_root: &Path,
) -> Result<Vec<Entry>, MsgError> {
    let binary =
        std::env::current_exe().map_err(|_| refused("cannot resolve the Hive executable"))?;
    let binary = binary
        .to_str()
        .ok_or_else(|| refused("Hive executable path must be UTF-8"))?
        .to_owned();
    let target = target
        .to_str()
        .ok_or_else(|| refused("target path must be UTF-8"))?;
    let user_root = user_root
        .to_str()
        .ok_or_else(|| refused("user root path must be UTF-8"))?;
    hook_events::host_events(host)
        .into_iter()
        .map(|allowed| {
            let mut arguments = vec![
                binary.as_str(),
                "msg",
                "hook",
                "--host",
                host.as_str(),
                "--event",
                allowed.event,
                "--role",
                role,
            ];
            if let Some(task) = task {
                arguments.extend(["--task", task]);
            }
            arguments.extend([
                "--binding",
                binding,
                "--target",
                target,
                "--user-root",
                user_root,
            ]);
            let handler = json!({"type": "command", "command": quote_command(&arguments)?,
                "timeout": HOOK_TIMEOUT_SECONDS});
            Ok(Entry {
                path: vec!["hooks".to_owned(), allowed.event.to_owned()],
                value: json!({"hooks": [handler]}),
            })
        })
        .collect()
}

fn transform(bytes: Option<&[u8]>, intent: &HookIntent) -> Result<Option<Vec<u8>>, MsgError> {
    let fail = |message: &str| refused(format!("cannot edit the project hook file: {message}"));
    let mut output = bytes.unwrap_or(b"{}\n").to_vec();
    for old in &intent.before {
        let desired = intent.after.iter().find(|entry| entry.path == old.path);
        let path: Vec<&str> = old.path.iter().map(String::as_str).collect();
        output = edit_entry(
            &output,
            &path,
            Some(&old.value),
            desired.map(|entry| &entry.value),
        )
        .map_err(fail)?;
    }
    for new in &intent.after {
        if !intent.before.iter().any(|entry| entry.path == new.path) {
            let path: Vec<&str> = new.path.iter().map(String::as_str).collect();
            output = edit_entry(&output, &path, None, Some(&new.value)).map_err(fail)?;
        }
    }
    if intent.operation == "remove" {
        let mut paths = intent.created_containers.clone();
        paths.sort_by_key(|path| std::cmp::Reverse(path.len()));
        for path in paths {
            let path: Vec<&str> = path.iter().map(String::as_str).collect();
            output = prune_empty(&output, &path).map_err(fail)?;
        }
        if intent.created_config
            && serde_json::from_slice::<Value>(&output).is_ok_and(|value| value == json!({}))
        {
            return Ok(None);
        }
    }
    Ok(Some(output))
}

struct Plan {
    intent: HookIntent,
    config: Option<Vec<u8>>,
    receipt: Option<Vec<u8>>,
    desired: Option<Vec<u8>>,
}

struct Scope {
    host: HookHost,
    role: String,
    task: Option<String>,
    stop_shape: Option<ReturnShape>,
    user_root: PathBuf,
}

fn scope(context: &Context, options: &[(&str, &str)]) -> Result<Scope, MsgError> {
    let host = HookHost::parse(required(options, "--host")?).map_err(MsgError::input)?;
    let role_name = required(options, "--role")?;
    super::checked_id(role_name, "--role")?;
    let role = context.role(role_name)?;
    let task = option(options, "--task")
        .map(|task| super::checked_id(task, "--task"))
        .transpose()?;
    let stop_shape = match (host, option(options, "--return")) {
        (HookHost::Claude, None | Some("context")) => Some(ReturnShape::StopContext),
        (HookHost::Claude, Some("block")) | (HookHost::Codex, None | Some("block")) => {
            Some(ReturnShape::StopBlock)
        }
        (HookHost::Gemini, None) => None,
        (_, Some(other)) => {
            return Err(MsgError::input(format!(
                "--return {other} is not a documented Stop return for {}",
                host.as_str()
            )))
        }
    };
    Ok(Scope {
        host,
        role,
        task,
        stop_shape,
        user_root: user_root(options)?,
    })
}

fn plan(context: &Context, scope: &Scope, operation: &str) -> Result<Plan, MsgError> {
    let config_path = config_path(scope.host);
    let config = read(context, config_path)?;
    let receipt = read(context, &receipt_path(scope.host))?;
    let previous = installed(context, scope.host)?;
    let installed = previous
        .as_ref()
        .filter(|intent| intent.operation == "install");
    if operation == "remove" && installed.is_none() {
        return Err(refused("no approved mediation hooks to remove"));
    }
    let target = context.target.requested_path();
    let target_digest = path_digest(target);
    let user_root_digest = path_digest(&scope.user_root);
    let binding = hook_events::binding_digest(
        scope.host,
        &scope.role,
        scope.task.as_deref(),
        scope.stop_shape,
        &target_digest,
        &user_root_digest,
    )
    .map_err(MsgError::input)?;
    let after = if operation == "install" {
        entries(
            scope.host,
            &scope.role,
            scope.task.as_deref(),
            &binding,
            target,
            &scope.user_root,
        )?
    } else {
        Vec::new()
    };
    let mut created_containers =
        installed.map_or_else(Vec::new, |intent| intent.created_containers.clone());
    for entry in &after {
        for length in 1..=entry.path.len() {
            let path = entry.path[..length].to_vec();
            let names: Vec<&str> = path.iter().map(String::as_str).collect();
            let present = contains_path(config.as_deref().unwrap_or(b"{}"), &names)
                .map_err(|message| refused(format!("project hook file is invalid: {message}")))?;
            if !present && !created_containers.contains(&path) {
                created_containers.push(path);
            }
        }
    }
    let config_mode = if config.is_some() {
        host_policy_mode(context.target.target_dir(), Path::new(config_path))
            .map_err(|message| MsgError::blocked("hive.msg-path-refused", message))?
    } else {
        None
    };
    let mut intent = HookIntent {
        schema_version: HOOK_EVENT_SCHEMA_VERSION,
        host: scope.host,
        role: scope.role.clone(),
        task: scope.task.clone(),
        stop_shape: scope.stop_shape,
        target_digest,
        user_root_digest,
        binding_digest: binding,
        operation: operation.to_owned(),
        config_path: config_path.to_owned(),
        before_digest: digest(config.as_deref()),
        after_digest: String::new(),
        before: installed.map_or_else(Vec::new, |prior| prior.after.clone()),
        after,
        created_config: installed.map_or(config.is_none(), |prior| prior.created_config),
        created_containers,
        config_mode,
        approval_digest: String::new(),
    };
    let desired = transform(config.as_deref(), &intent)?;
    intent.after_digest = digest(desired.as_deref());
    intent.approval_digest = intent.computed_approval()?;
    Ok(Plan {
        intent,
        config,
        receipt,
        desired,
    })
}

fn allowlist(host: HookHost) -> Value {
    hook_events::host_events(host)
        .iter()
        .map(|entry| {
            json!({"event": entry.event, "return": entry.shape, "alternatives": entry.alternatives,
                "recursion_guard": entry.recursion_guard, "source": entry.source, "live": entry.live})
        })
        .collect()
}

fn result(code: &'static str, message: &str, data: Value, changed: Vec<String>) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: ACTION,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths: changed,
        evidence: Vec::new(),
        next_action: None,
        data: Some(data),
    }
}

pub(super) fn run(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let action = arguments
        .first()
        .map(String::as_str)
        .filter(|action| matches!(*action, "preview" | "apply" | "remove" | "status"))
        .ok_or_else(|| MsgError::input("use msg hooks preview|apply|remove|status"))?;
    let options = parse_options(
        &arguments[1..],
        &[
            "--host",
            "--role",
            "--task",
            "--return",
            "--confirm",
            "--target",
            "--user-root",
        ],
    )?;
    let context = Context::open(&options)?;
    if action == "status" {
        return status(&context, &options);
    }
    let scope = scope(&context, &options)?;
    let operation = if action == "remove" {
        "remove"
    } else {
        "install"
    };
    let planned = plan(&context, &scope, operation)?;
    let data = json!({
        "preview": planned.intent,
        "config_path": planned.intent.config_path,
        "receipt_path": receipt_path(scope.host),
        "allowlist": allowlist(scope.host),
        "user_global_settings_changed": false,
        "host_loaded": "unverified",
        "actual_effect": "unverified",
        "authorizes_model_execution": false,
        "launches_host_process": false,
    });
    let confirmation = option(&options, "--confirm");
    if action == "preview" || confirmation.is_none() {
        return Ok(result(
            "hive.msg-hooks-preview",
            "project hook preview only; nothing changed. Apply with --confirm <approval_digest>",
            data,
            Vec::new(),
        ));
    }
    if confirmation != Some(planned.intent.approval_digest.as_str()) {
        return Err(refused(
            "--confirm differs from the current preview digest; preview again and confirm the exact digest",
        ));
    }
    context.target.verify_current()?;
    let config_path = Path::new(&planned.intent.config_path);
    let receipt_relative = receipt_path(scope.host);
    let receipt_bytes = planned.intent.encoded()?;
    let root = context.target.target_dir();
    let receipt_mode = if planned.receipt.is_some() {
        host_policy_mode(root, Path::new(&receipt_relative))
            .map_err(|message| MsgError::blocked("hive.msg-path-refused", message))?
    } else {
        None
    };
    replace_host_policy_file(
        root,
        Path::new(&receipt_relative),
        planned.receipt.as_deref(),
        Some(&receipt_bytes),
        receipt_mode,
    )
    .map_err(refused)?;
    replace_host_policy_file(
        root,
        config_path,
        planned.config.as_deref(),
        planned.desired.as_deref(),
        planned.intent.config_mode,
    )
    .map_err(|message| {
        refused(format!(
            "receipt kept at {receipt_relative}; the project hook file changed: {message}"
        ))
    })?;
    let mut changed = vec![receipt_relative];
    if planned.config != planned.desired {
        changed.push(planned.intent.config_path.clone());
    }
    let (code, message) = if operation == "remove" {
        (
            "hive.msg-hooks-removed",
            "removed only the approved Hive mediation hook entries",
        )
    } else {
        (
            "hive.msg-hooks-configured",
            "project hook entries written; host loading and effect still need a live check",
        )
    };
    Ok(result(code, message, data, changed))
}

fn status(context: &Context, options: &[(&str, &str)]) -> Result<ActionResult, MsgError> {
    let host = HookHost::parse(required(options, "--host")?).map_err(MsgError::input)?;
    let intent = installed(context, host)?;
    let observed = read(context, config_path(host))?;
    let configured = intent.as_ref().is_some_and(|intent| {
        intent.operation == "install" && digest(observed.as_deref()) == intent.after_digest
    });
    Ok(result(
        "hive.msg-hooks-status",
        "mediation hook status; host loading is not observed here",
        json!({
            "host": host,
            "config_path": config_path(host),
            "receipt_path": receipt_path(host),
            "receipt_present": intent.is_some(),
            "operation": intent.as_ref().map(|intent| intent.operation.clone()),
            "role": intent.as_ref().map(|intent| intent.role.clone()),
            "task": intent.as_ref().and_then(|intent| intent.task.clone()),
            "binding_digest": intent.as_ref().map(|intent| intent.binding_digest.clone()),
            "configured": configured,
            "enabled_by_default": false,
            "observed_config_digest": digest(observed.as_deref()),
            "allowlist": allowlist(host),
            "host_loaded": "unverified",
        }),
        Vec::new(),
    ))
}
