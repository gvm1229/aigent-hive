//! Explicit project-local hook configuration. Presence never proves host activation.

use crate::run::PinnedTarget;
use crate::user_install::replace_host_policy_file;
use crate::{emit_action_result, ActionResult};
use hive_core::{policy::valid_digest, sha256_digest};
use hive_projection::hook_config::{contains_path, edit_entry, prune_empty};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const LIMIT: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: Vec<String>,
    value: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema_version: u32,
    host: String,
    target_digest: String,
    policy_digest: String,
    prior_policy_digest: Option<String>,
    operation: String,
    before_digest: String,
    after_digest: String,
    before: Vec<Entry>,
    after: Vec<Entry>,
    created_containers: Vec<Vec<String>>,
    created_config: bool,
    config_mode: Option<u32>,
    approval_digest: String,
}

struct Plan {
    intent: Intent,
    config_path: PathBuf,
    receipt_path: PathBuf,
    config: Option<Vec<u8>>,
    receipt: Option<Vec<u8>>,
    desired: Option<Vec<u8>>,
}

fn digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".to_owned(), sha256_digest)
}

fn config_path(host: &str) -> Result<PathBuf, String> {
    match host {
        "codex" => Ok(".codex/hooks.json".into()),
        "claude" => Ok(".claude/settings.local.json".into()),
        "antigravity" => Ok(".agents/hooks.json".into()),
        _ => Err("host must be codex, claude, or antigravity".to_owned()),
    }
}

fn command(host: &str, event: &str, target: &Path, policy: &str) -> Result<String, String> {
    let binary = std::env::current_exe().map_err(|_| "cannot resolve Hive executable")?;
    let args = [
        binary.to_str().ok_or("Hive executable must be UTF-8")?,
        "policy",
        "hook",
        "--host",
        host,
        "--event",
        event,
        "--target",
        target.to_str().ok_or("hook target must be UTF-8")?,
        "--stdin-json",
        "--expected-policy",
        policy,
    ];
    if args.iter().any(|arg| arg.contains(['\r', '\n', '\0'])) {
        return Err("hook command contains unsupported control characters".to_owned());
    }
    if cfg!(windows) {
        if args
            .iter()
            .any(|arg| arg.contains(['"', '$', '`', '%', '!']))
        {
            return Err(
                "hook path contains characters unsupported by the Windows command adapter"
                    .to_owned(),
            );
        }
        let quoted = args
            .iter()
            .map(|arg| format!("'{}'", arg.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" ");
        Ok(format!(
            "powershell.exe -NoProfile -NonInteractive -Command \"& {quoted}\""
        ))
    } else {
        Ok(args
            .iter()
            .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
            .collect::<Vec<_>>()
            .join(" "))
    }
}

fn entries(host: &str, target: &Path, policy: &str) -> Result<Vec<Entry>, String> {
    let (namespace, start, matcher) = match host {
        "codex" => ("hooks", "SessionStart", "^apply_patch$"),
        "claude" => ("hooks", "SessionStart", "^(Write|Edit|MultiEdit)$"),
        "antigravity" => (
            "aigent-hive-policy",
            "PreInvocation",
            "^(write_to_file|replace_file_content|multi_replace_file_content)$",
        ),
        _ => return Err("unsupported hook host".to_owned()),
    };
    ["PreToolUse", start, "Stop"]
        .iter()
        .map(|event| {
            let handler =
                json!({"type":"command","command":command(host,event,target,policy)?,"timeout":10});
            let value = if *event == "PreToolUse" {
                json!({"matcher":matcher,"hooks":[handler]})
            } else if host == "antigravity" {
                handler
            } else {
                json!({"hooks":[handler]})
            };
            Ok(Entry {
                path: vec![namespace.to_owned(), (*event).to_owned()],
                value,
            })
        })
        .collect()
}

fn encoded(intent: &Intent) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(intent).map_err(|_| "cannot encode hook intent")?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn receipt_path(target: &PinnedTarget, host: &str) -> Result<PathBuf, String> {
    let source = target
        .read_optional(Path::new("hive-source.json"), 4096)
        .map_err(|error| error.message().to_owned())?;
    if let Some(bytes) = source {
        let marker: Value = serde_json::from_slice(&bytes).map_err(|_| "invalid source marker")?;
        if marker
            != json!({"schema_version":1,"kind":"aigent-hive-source-workspace","consumer_setup_allowed":false})
        {
            return Err("unsupported source target".to_owned());
        }
        return Ok(format!(".agents/policy-hooks/{host}.json").into());
    }
    let harness = target
        .read_required(Path::new(".hive/config/harness.toml"), 64 * 1024)
        .map_err(|error| error.message().to_owned())?;
    let config: toml::Value =
        toml::from_str(std::str::from_utf8(&harness).map_err(|_| "invalid harness encoding")?)
            .map_err(|_| "invalid harness configuration")?;
    if config
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        != Some(1)
    {
        return Err("unsupported project configuration".to_owned());
    }
    Ok(format!(".hive/config/host-policy-hooks/{host}.json").into())
}

fn read(target: &PinnedTarget, path: &Path) -> Result<Option<Vec<u8>>, String> {
    // This adapter selects only fixed approved host/receipt paths. Consumer run
    // readers deliberately reject host namespaces, so keep that boundary intact.
    crate::user_install::read_user_setup_file(target.target_dir(), path, LIMIT as u64)
}

fn validate_intent(intent: &Intent, host: &str, target: &Path) -> Result<(), String> {
    hive_core::validate_json_schema(
        include_str!("../../../../schemas/host-policy-intent.schema.json"),
        &serde_json::to_value(intent).map_err(|_| "cannot inspect hook intent")?,
        "hook configuration intent",
    )
    .map_err(|_| "hook intent violates its schema")?;
    if intent.schema_version != 1
        || intent.host != host
        || !matches!(intent.operation.as_str(), "install" | "remove")
        || intent.target_digest != sha256_digest(target.to_string_lossy().as_bytes())
        || !valid_digest(&intent.policy_digest)
        || !valid_digest(&intent.approval_digest)
        || (intent.before_digest != "absent" && !valid_digest(&intent.before_digest))
        || (intent.after_digest != "absent" && !valid_digest(&intent.after_digest))
        || intent.created_containers.len() > 4
    {
        return Err("hook intent binding is invalid".to_owned());
    }
    let after = if intent.operation == "install" {
        entries(host, target, &intent.policy_digest)?
    } else {
        Vec::new()
    };
    let before = match &intent.prior_policy_digest {
        Some(policy) if valid_digest(policy) => entries(host, target, policy)?,
        None => Vec::new(),
        _ => return Err("invalid prior policy digest".to_owned()),
    };
    if intent.after != after || intent.before != before {
        return Err("hook definitions differ from their bounded contract".to_owned());
    }
    let allowed = entries(host, target, &intent.policy_digest)?;
    if intent.created_containers.iter().any(|path| {
        !allowed
            .iter()
            .any(|entry| !path.is_empty() && entry.path.starts_with(path))
    }) {
        return Err("hook container ownership escaped its namespace".to_owned());
    }
    let mut approved = intent.clone();
    approved.approval_digest.clear();
    if sha256_digest(&encoded(&approved)?) != intent.approval_digest {
        return Err("hook approval digest changed".to_owned());
    }
    Ok(())
}

fn transform(bytes: Option<&[u8]>, intent: &Intent) -> Result<Option<Vec<u8>>, String> {
    let mut output = bytes.unwrap_or(b"{}\n").to_vec();
    for old in &intent.before {
        let desired = intent.after.iter().find(|entry| entry.path == old.path);
        let path = old.path.iter().map(String::as_str).collect::<Vec<_>>();
        output = edit_entry(
            &output,
            &path,
            Some(&old.value),
            desired.map(|entry| &entry.value),
        )
        .map_err(str::to_owned)?;
    }
    for new in &intent.after {
        if !intent.before.iter().any(|entry| entry.path == new.path) {
            output = edit_entry(
                &output,
                &new.path.iter().map(String::as_str).collect::<Vec<_>>(),
                None,
                Some(&new.value),
            )
            .map_err(str::to_owned)?;
        }
    }
    if intent.operation == "remove" {
        let mut paths = intent.created_containers.clone();
        paths.sort_by_key(|path| std::cmp::Reverse(path.len()));
        for path in paths {
            output = prune_empty(
                &output,
                &path.iter().map(String::as_str).collect::<Vec<_>>(),
            )
            .map_err(str::to_owned)?;
        }
        if intent.created_config
            && serde_json::from_slice::<Value>(&output).is_ok_and(|value| value == json!({}))
        {
            return Ok(None);
        }
    }
    Ok(Some(output))
}

fn plan(target: &PinnedTarget, host: &str, operation: &str) -> Result<Plan, String> {
    let config_path = config_path(host)?;
    let receipt_path = receipt_path(target, host)?;
    let config = read(target, &config_path)?;
    let receipt = read(target, &receipt_path)?;
    if operation == "install" {
        require_native_owner(target)?;
    }
    let previous = receipt
        .as_deref()
        .map(serde_json::from_slice::<Intent>)
        .transpose()
        .map_err(|_| "invalid hook receipt")?;
    if let Some(prior) = &previous {
        validate_intent(prior, host, target.requested_path())?;
    }
    let installed = previous
        .as_ref()
        .filter(|intent| intent.operation == "install");
    if operation == "remove" && installed.is_none() {
        return Err("no owned native hook installation to remove".to_owned());
    }
    let policy = env!("HIVE_NATIVE_POLICY_DIGEST").to_owned();
    let after = if operation == "install" {
        entries(host, target.requested_path(), &policy)?
    } else {
        Vec::new()
    };
    let mut created_containers =
        installed.map_or_else(Vec::new, |intent| intent.created_containers.clone());
    for entry in &after {
        for length in 1..=entry.path.len() {
            let path = entry.path[..length].to_vec();
            if !contains_path(
                config.as_deref().unwrap_or(b"{}"),
                &path.iter().map(String::as_str).collect::<Vec<_>>(),
            )
            .map_err(str::to_owned)?
                && !created_containers.contains(&path)
            {
                created_containers.push(path);
            }
        }
    }
    let mut intent = Intent {
        schema_version: 1,
        host: host.to_owned(),
        target_digest: sha256_digest(target.requested_path().to_string_lossy().as_bytes()),
        policy_digest: policy,
        prior_policy_digest: installed.map(|prior| prior.policy_digest.clone()),
        operation: operation.to_owned(),
        before_digest: digest(config.as_deref()),
        after_digest: String::new(),
        before: installed.map_or_else(Vec::new, |prior| prior.after.clone()),
        after,
        created_containers,
        created_config: installed.map_or(config.is_none(), |prior| prior.created_config),
        config_mode: if config.is_some() {
            crate::user_install::host_policy_mode(target.target_dir(), &config_path)?
        } else {
            None
        },
        approval_digest: String::new(),
    };
    let desired = transform(config.as_deref(), &intent)?;
    intent.after_digest = digest(desired.as_deref());
    intent.approval_digest = sha256_digest(&encoded(&intent)?);
    Ok(Plan {
        intent,
        config_path,
        receipt_path,
        config,
        receipt,
        desired,
    })
}

fn result(code: &'static str, data: Value, changed_paths: Vec<String>) -> ActionResult {
    ActionResult {schema_version:1,action:"ConfigurePolicyHooks",status:"success",exit_code:0,code,
        message:"project-local hook configuration; host trust and actual effect require separate verification".to_owned(),changed_paths,evidence:Vec::new(),next_action:None,data:Some(data)}
}

fn configure(args: &[String]) -> Result<ActionResult, String> {
    if !matches!(args.len(), 7 | 9)
        || !matches!(
            args[0].as_str(),
            "preview" | "apply" | "remove" | "status" | "recover"
        )
        || args[1] != "--host"
        || args[3] != "--target"
        || args[5] != "--output"
        || args[6] != "json"
        || (args.len() == 9 && args[7] != "--confirm")
    {
        return Err("expected hooks preview|apply|remove|status|recover --host <host> --target <project> --output json [--confirm <preview-digest>]".to_owned());
    }
    let target = PinnedTarget::open_policy_configuration(Path::new(&args[4]))
        .map_err(|error| error.message().to_owned())?;
    let host = &args[2];
    let action = &args[0];
    if action == "status" || action == "recover" {
        return inspect_or_recover(&target, host, action == "recover");
    }
    if action == "apply" || action == "remove" {
        if let Some(outcome) = replay(&target, host, action)? {
            return Ok(outcome);
        }
    }
    let operation = if action == "remove" {
        "remove"
    } else {
        "install"
    };
    let planned = plan(&target, host, operation)?;
    let data = json!({"preview":planned.intent,"config_path":planned.config_path,"receipt_path":planned.receipt_path,
        "host_loaded":"unverified","actual_effect":"unverified","authorizes_model_execution":false});
    if action == "preview" || (action == "remove" && args.len() == 7) {
        return Ok(result("hive.policy-hooks-preview", data, Vec::new()));
    }
    if args.len() != 9 || args[8] != planned.intent.approval_digest {
        return Err("exact current preview confirmation is required".to_owned());
    }
    if planned.config.is_some()
        && crate::user_install::host_policy_mode(target.target_dir(), &planned.config_path)?
            != planned.intent.config_mode
    {
        return Err("host setting permissions changed after preview".to_owned());
    }
    target
        .verify_current()
        .map_err(|error| error.message().to_owned())?;
    let receipt = encoded(&planned.intent)?;
    replace_host_policy_file(
        target.target_dir(),
        &planned.receipt_path,
        planned.receipt.as_deref(),
        Some(&receipt),
        if planned.receipt.is_some() {
            crate::user_install::host_policy_mode(target.target_dir(), &planned.receipt_path)?
        } else {
            None
        },
    )?;
    apply_configuration(
        &target,
        &planned.config_path,
        planned.config.as_deref(),
        planned.desired.as_deref(),
        &planned.receipt_path,
        planned.intent.config_mode,
    )?;
    let mut changed = Vec::new();
    if planned.receipt.as_deref() != Some(receipt.as_slice()) {
        changed.push(planned.receipt_path.to_string_lossy().replace('\\', "/"));
    }
    if planned.config != planned.desired {
        changed.push(planned.config_path.to_string_lossy().replace('\\', "/"));
    }
    Ok(result("hive.policy-hooks-configured", data, changed))
}

fn replay(target: &PinnedTarget, host: &str, action: &str) -> Result<Option<ActionResult>, String> {
    let path = receipt_path(target, host)?;
    let Some(bytes) = read(target, &path)? else {
        return Ok(None);
    };
    let intent: Intent = serde_json::from_slice(&bytes).map_err(|_| "invalid hook intent")?;
    validate_intent(&intent, host, target.requested_path())?;
    let desired_action = if action == "apply" {
        "install"
    } else {
        "remove"
    };
    if intent.operation != desired_action
        || (action == "apply" && intent.policy_digest != env!("HIVE_NATIVE_POLICY_DIGEST"))
    {
        return Ok(None);
    }
    let current = read(target, &config_path(host)?)?;
    if digest(current.as_deref()) != intent.after_digest {
        return Ok(None);
    }
    inspect_or_recover(target, host, false).map(Some)
}

fn apply_configuration(
    target: &PinnedTarget,
    config: &Path,
    before: Option<&[u8]>,
    after: Option<&[u8]>,
    receipt: &Path,
    expected_mode: Option<u32>,
) -> Result<(), String> {
    target
        .verify_current()
        .map_err(|error| error.message().to_owned())?;
    replace_host_policy_file(target.target_dir(), config, before, after, expected_mode).map_err(
        |error| {
            format!(
                "hook intent retained at {}; inspect status and recover before retrying: {error}",
                receipt.display()
            )
        },
    )?;
    target
        .verify_current()
        .map_err(|error| error.message().to_owned())?;
    if read(target, config)?.as_deref() != after {
        return Err("host settings changed after hook publication".to_owned());
    }
    Ok(())
}

fn inspect_or_recover(
    target: &PinnedTarget,
    host: &str,
    recover: bool,
) -> Result<ActionResult, String> {
    let config_path = config_path(host)?;
    let receipt_path = receipt_path(target, host)?;
    let mut config = read(target, &config_path)?;
    let receipt = read(target, &receipt_path)?;
    let Some(bytes) = receipt else {
        return Ok(result(
            "hive.policy-hooks-status",
            json!({"configured":false,"host_loaded":"unverified","actual_effect":"unverified"}),
            Vec::new(),
        ));
    };
    let intent: Intent = serde_json::from_slice(&bytes).map_err(|_| "invalid hook intent")?;
    validate_intent(&intent, host, target.requested_path())?;
    if recover {
        if intent.operation == "install" {
            require_native_owner(target)?;
        }
        target
            .verify_current()
            .map_err(|error| error.message().to_owned())?;
        crate::user_install::recover_host_policy_file(
            target.target_dir(),
            &config_path,
            &intent.before_digest,
            &intent.after_digest,
            intent.config_mode,
        )?;
        config = read(target, &config_path)?;
    }
    let current = digest(config.as_deref());
    let mut changed = Vec::new();
    if recover && current != intent.after_digest {
        if current != intent.before_digest {
            return Err("host settings changed during hook configuration; preserve them and review the retained intent".to_owned());
        }
        let desired = transform(config.as_deref(), &intent)?;
        if digest(desired.as_deref()) != intent.after_digest {
            return Err("hook recovery differs from the approved output".to_owned());
        }
        apply_configuration(
            target,
            &config_path,
            config.as_deref(),
            desired.as_deref(),
            &receipt_path,
            intent.config_mode,
        )?;
        changed.push(config_path.to_string_lossy().replace('\\', "/"));
    }
    let observed = read(target, &config_path)?;
    let configured = intent.operation == "install"
        && intent.after.iter().all(|entry| {
            edit_entry(
                observed.as_deref().unwrap_or(b"{}"),
                &entry.path.iter().map(String::as_str).collect::<Vec<_>>(),
                None,
                Some(&entry.value),
            )
            .is_ok_and(|next| Some(next.as_slice()) == observed.as_deref())
        });
    Ok(result(
        "hive.policy-hooks-status",
        json!({"host":host,"config_path":config_path,"receipt_path":receipt_path,
        "host_version":null,"host_version_reason":"configuration does not observe the executing host",
        "operating_system":std::env::consts::OS,"policy_digest":intent.policy_digest,"target_digest":intent.target_digest,
        "observed_config_digest":digest(observed.as_deref()),"approved_config_digest":intent.after_digest,
        "configured":configured,"policy_current":intent.policy_digest==env!("HIVE_NATIVE_POLICY_DIGEST"),
        "pending":digest(observed.as_deref())!=intent.after_digest && !configured,
        "host_loaded":"unverified","event_matched":"unverified","checker_executed":"unverified",
        "denial_observed":"unverified","actual_effect":"unverified","authorizes_model_execution":false}),
        changed,
    ))
}

fn require_native_owner(target: &PinnedTarget) -> Result<(), String> {
    if target
        .read_optional(Path::new("hive-source.json"), 4096)
        .map_err(|error| error.message().to_owned())?
        .is_some()
    {
        return Ok(());
    }
    let bytes = target
        .read_required(Path::new(".hive/config/harness.toml"), 64 * 1024)
        .map_err(|error| error.message().to_owned())?;
    let config: toml::Value =
        toml::from_str(std::str::from_utf8(&bytes).map_err(|_| "invalid project configuration")?)
            .map_err(|_| "invalid project configuration")?;
    if config.get("resolved_owner").and_then(toml::Value::as_str) != Some("host-native") {
        return Err("native policy installation requires host-native ownership; status and revocation remain available".to_owned());
    }
    Ok(())
}

pub(super) fn run(args: &[String]) -> ExitCode {
    if args == ["--help"] {
        println!("hive policy hooks preview|apply|remove|status|recover --host codex|claude|antigravity --target <project> --output json [--confirm <preview-digest>]\nPreview is read-only. Apply and removal require their exact preview. Recovery uses the retained approved intent. Host trust and effect remain separately unverified.");
        return ExitCode::SUCCESS;
    }
    let before = observe(args);
    let outcome=configure(args).unwrap_or_else(|message| {
        let after=observe(args);
        let observed=before.as_ref().zip(after.as_ref()).map(|(before,after)| {
            after.iter().filter(|(path,value)| before.get(*path)!=Some(*value)).map(|(path,_)|path.clone()).collect::<Vec<_>>()
        }).unwrap_or_default();
        ActionResult {schema_version:1,action:"ConfigurePolicyHooks",status:"blocked",exit_code:3,
            code:"hive.policy-hook-config-blocked",message,changed_paths:observed,evidence:Vec::new(),
            next_action:Some("Inspect the exact project hook status and retained intent before retrying; do not overwrite foreign settings.".to_owned()),
            data:Some(json!({"change_attribution":"observed across the call; may include concurrent changes","observation_complete":before.is_some()&&after.is_some(),"host_effect":"unverified"}))}
    });
    emit_action_result(&outcome)
}

fn observe(args: &[String]) -> Option<std::collections::BTreeMap<String, String>> {
    if !matches!(args.len(), 7 | 9) || args[1] != "--host" || args[3] != "--target" {
        return None;
    }
    let target = PinnedTarget::open_policy_configuration(Path::new(&args[4])).ok()?;
    let paths = [
        config_path(&args[2]).ok()?,
        receipt_path(&target, &args[2]).ok()?,
    ];
    paths
        .into_iter()
        .map(|path| {
            Some((
                path.to_string_lossy().replace('\\', "/"),
                digest(read(&target, &path).ok()?.as_deref()),
            ))
        })
        .collect()
}
