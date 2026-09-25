//! Native host protocol adapter for deterministic file-edit preflight.
//! Shell, terminal continuation and MCP effects remain outside this classifier.

use super::directive_context::{self, Context};
use hive_core::policy::{evaluate, Binding, Decision, Evaluation, Requirement, RuleResult};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

const LIMIT: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Host {
    Codex,
    Claude,
    Antigravity,
}

impl Host {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "antigravity" => Some(Self::Antigravity),
            _ => None,
        }
    }
}

fn deny(host: Host, reason: &str) -> Value {
    match host {
        Host::Codex | Host::Claude => json!({"hookSpecificOutput": {
            "hookEventName":"PreToolUse", "permissionDecision":"deny",
            "permissionDecisionReason": reason}}),
        Host::Antigravity => json!({"decision":"deny", "reason":reason}),
    }
}

pub(super) fn run(args: &[String]) -> ExitCode {
    if args == ["--help"] {
        println!("hive policy hook --host codex|claude|antigravity --event PreToolUse|SessionStart|PreInvocation|Stop --target <dir> --stdin-json [--expected-policy <digest>] [--review-run <id>] [--expected-context <digest>]\nNative file-edit policy and optional session-bound review notice; registration and host trust are separate.");
        return ExitCode::SUCCESS;
    }
    if !matches!(args.len(), 7 | 9 | 11 | 13)
        || args[0] != "--host"
        || args[2] != "--event"
        || args[4] != "--target"
        || args[6] != "--stdin-json"
    {
        eprintln!("invalid native policy hook arguments; inspect hive policy hook --help");
        return ExitCode::from(2);
    }
    let Some(host) = Host::parse(&args[1]) else {
        eprintln!("unsupported native policy host");
        return ExitCode::from(2);
    };
    let mut expected_policy = None;
    let mut review_run = None;
    let mut expected_context = None;
    for pair in args[7..].as_chunks::<2>().0 {
        match pair[0].as_str() {
            "--expected-policy" if expected_policy.is_none() => {
                expected_policy = Some(pair[1].as_str());
            }
            "--expected-context" if expected_context.is_none() => {
                expected_context = Some(pair[1].as_str());
            }
            "--review-run" if review_run.is_none() && args[3] == "Stop" => {
                review_run = Some(pair[1].as_str());
            }
            _ => {
                eprintln!("invalid native hook option");
                return ExitCode::from(2);
            }
        }
    }
    // A turn ending is neither task success nor authority to continue or capture memory.
    if args[3] == "Stop" && (review_run.is_none() || host == Host::Antigravity) {
        println!(
            "{}",
            if host == Host::Antigravity {
                json!({"decision":"allow"})
            } else {
                json!({})
            }
        );
        return ExitCode::SUCCESS;
    }
    if expected_policy.is_some_and(|policy| policy != env!("HIVE_NATIVE_POLICY_DIGEST")) {
        let response = if args[3] == "PreToolUse" {
            deny(
                host,
                "registered policy changed; preview and approve the updated hook definition",
            )
        } else {
            json!({})
        };
        println!("{response}");
        return ExitCode::SUCCESS;
    }
    let mut bytes = Vec::new();
    let input = std::io::stdin()
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read native hook input")
        .and({
            if bytes.len() as u64 > LIMIT {
                Err("native hook input exceeds limit")
            } else {
                Ok(())
            }
        })
        .and_then(|()| {
            serde_json::from_slice::<Value>(&bytes).map_err(|_| "invalid native hook JSON")
        });
    let output = match input {
        Ok(payload) if args[3] == "Stop" => {
            review_notice(host, Path::new(&args[5]), review_run, &payload)
        }
        Ok(payload) => respond_approved(
            host,
            &args[3],
            Path::new(&args[5]),
            &payload,
            expected_context,
        ),
        Err(reason) if args[3] == "PreToolUse" => deny(host, reason),
        Err(_) => json!({}),
    };
    println!("{output}");
    // Exit 0 carries native JSON denial. Hive's legacy exit 3 is never forwarded.
    ExitCode::SUCCESS
}

fn respond_approved(
    host: Host,
    event: &str,
    target: &Path,
    payload: &Value,
    expected_context: Option<&str>,
) -> Value {
    let context = expected_context
        .map(|expected| {
            let target = crate::run::PinnedTarget::open_policy_configuration(target)
                .map_err(|_| "directive context target unavailable".to_owned())?;
            let context = directive_context::load_runtime(&target)?
                .ok_or("approved directive context missing")?;
            if context.digest != expected {
                return Err("directive context changed; preview and approve it again".to_owned());
            }
            Ok(context)
        })
        .transpose();
    match context {
                Ok(context) => respond_context(host, event, target, payload, context.as_ref()),
                Err(_) if event == "PreToolUse" => deny(host, "approved directive context unavailable or changed; repair or explicitly reapprove the context before file edits"),
                Err(_) => context_output(host, "Hive directive recovery unavailable: review the approved context sources and hook preview. Do not treat missing context as permission."),
            }
}

fn review_notice(host: Host, target: &Path, run: Option<&str>, payload: &Value) -> Value {
    let Some(run) = run else {
        return json!({});
    };
    let Some(session) = payload["session_id"].as_str() else {
        return json!({});
    };
    if payload["hook_event_name"] != "Stop" {
        return json!({});
    }
    let host = match host {
        Host::Codex => "codex",
        Host::Claude => "claude",
        Host::Antigravity => return json!({"decision":"allow"}),
    };
    match crate::run::policy_review::pending_notice(target, run, host, session) {
        Ok(count) if count > 0 => {
            json!({"systemMessage":format!("Hive: {count} policy review candidate(s) await explicit review. No policy changes are authorized by this notice.")})
        }
        _ => json!({}),
    }
}

fn paths(host: Host, payload: &Value) -> Result<Vec<String>, &'static str> {
    let (name, input) = match host {
        Host::Codex | Host::Claude => {
            if payload["hook_event_name"] != "PreToolUse" {
                return Err("native event mismatch");
            }
            (payload["tool_name"].as_str(), &payload["tool_input"])
        }
        Host::Antigravity => (
            payload["toolCall"]["name"].as_str(),
            &payload["toolCall"]["args"],
        ),
    };
    let value = match (host, name) {
        (Host::Codex, Some("apply_patch")) => {
            return patch_paths(input["command"].as_str().ok_or("missing patch")?)
        }
        (Host::Claude, Some("Write" | "Edit" | "MultiEdit")) => input["file_path"].as_str(),
        (
            Host::Antigravity,
            Some("write_to_file" | "replace_file_content" | "multi_replace_file_content"),
        ) => input["TargetFile"].as_str(),
        _ => {
            return Err(
                "unsupported tool: register this guard only for documented file-edit matchers",
            )
        }
    }
    .ok_or("missing native file target")?;
    if value.is_empty() || value.len() > 4096 {
        return Err("invalid native file target");
    }
    Ok(vec![value.to_owned()])
}

fn patch_paths(patch: &str) -> Result<Vec<String>, &'static str> {
    let mut lines = patch.lines();
    if lines.next() != Some("*** Begin Patch") {
        return Err("unsupported patch envelope");
    }
    let mut targets = Vec::new();
    let mut ended = false;
    for line in lines {
        if ended {
            return Err("trailing patch data");
        }
        if line == "*** End Patch" {
            ended = true;
            continue;
        }
        if let Some(path) = [
            "*** Add File: ",
            "*** Update File: ",
            "*** Delete File: ",
            "*** Move to: ",
        ]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
        {
            if path.is_empty() || path.len() > 4096 || targets.len() == 64 {
                return Err("invalid patch target count or path");
            }
            targets.push(path.to_owned());
        } else if line.starts_with("*** ") && line != "*** End of File" {
            return Err("unsupported patch directive");
        }
    }
    if !ended || targets.is_empty() {
        return Err("incomplete patch envelope");
    }
    Ok(targets)
}

#[cfg(test)]
fn respond(host: Host, event: &str, target: &Path, payload: &Value) -> Value {
    respond_context(host, event, target, payload, None)
}

fn context_output(host: Host, text: &str) -> Value {
    if host == Host::Antigravity {
        json!({"injectSteps":[{"ephemeralMessage":text}]})
    } else {
        json!({"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":text}})
    }
}

fn respond_context(
    host: Host,
    event: &str,
    target: &Path,
    payload: &Value,
    context: Option<&Context>,
) -> Value {
    if (host == Host::Antigravity && event == "PreInvocation")
        || (host != Host::Antigravity && event == "SessionStart")
    {
        // No parent-session or policy-hash cache: every compact event restores again.
        if host == Host::Antigravity && payload["invocationNum"].as_u64() != Some(0) {
            return json!({});
        }
        if host != Host::Antigravity
            && (payload["hook_event_name"]
                .as_str()
                .is_some_and(|name| name != "SessionStart")
                || payload["source"].as_str().is_some_and(|source| {
                    !matches!(source, "startup" | "resume" | "compact" | "clear")
                }))
        {
            return json!({});
        }
        return context_output(
            host,
            &context.map_or_else(|| directive_context::CORE.to_owned(), Context::restore),
        );
    }

    if event != "PreToolUse" {
        return json!({});
    }
    let checked = check_files(host, target, payload, context);
    match checked {
        // Antigravity requires a decision. "allow" grants tool permission; "ask" retains its
        // native permission review and respects prior Always Allow settings instead.
        Ok((Decision::Allow, _)) if host == Host::Antigravity => json!({"decision":"ask",
            "reason":"Hive file checks passed; host permission review still applies"}),
        Ok((Decision::Allow, paths)) => {
            let detail = context.map_or_else(String::new, |context| context.edit(&paths));
            if detail.is_empty() { json!({}) } else {
                // Additional context does not grant permission or override another guard.
                json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":detail}})
            }
        },
        Ok(_) => deny(
            host,
            "Approved protected paths require their authorized mutation workflow; direct file mutation refused",
        ),
        Err(reason) => deny(host, reason),
    }
}

fn check_files(
    host: Host,
    target: &Path,
    payload: &Value,
    context: Option<&Context>,
) -> Result<(Decision, Vec<String>), &'static str> {
    let target = target
        .canonicalize()
        .map_err(|_| "policy target unavailable")?;
    if !target.is_dir() {
        return Err("policy target must be a directory");
    }
    let binding = Binding {
        operation_id: "native-file-preflight".to_owned(),
        policy_digest: env!("HIVE_NATIVE_POLICY_DIGEST").to_owned(),
        target_digest: sha256_digest(target.to_string_lossy().as_bytes()),
    };
    let mut decision = Decision::Allow;
    let mut checked_paths = Vec::new();
    for path in paths(host, payload)? {
        let absolute = if Path::new(&path).is_absolute() {
            std::path::PathBuf::from(&path)
        } else {
            let cwd = if host == Host::Antigravity {
                payload["workspacePaths"]
                    .as_array()
                    .filter(|roots| roots.len() == 1)
                    .and_then(|roots| roots[0].as_str())
            } else {
                payload["cwd"].as_str()
            }
            .ok_or("relative edit requires a bound native working directory")?;
            Path::new(cwd)
                .canonicalize()
                .map_err(|_| "native working directory unavailable")?
                .join(&path)
        };
        let absolute = absolute.to_str().ok_or("native file target is not UTF-8")?;
        let relative = crate::observed_hook_relative(&target, absolute)
            .ok_or("file target is outside the registered policy scope")?;
        hive_core::inspect_host_edit_path(&target, &relative)
            .map_err(|_| "native file target has an unsafe path ancestor")?;
        if crate::is_protected_hive_path(&relative)
            || relative == Path::new(".agents/policy-context.toml")
            || relative.starts_with(".agents/policy-hooks/")
            || context.is_some_and(|context| {
                context.protects(&relative.to_string_lossy().replace('\\', "/"))
            })
        {
            decision = Decision::Deny;
        }
        checked_paths.push(relative.to_string_lossy().replace('\\', "/"));
    }
    let rule_id = "hive-owned-state".to_owned();
    let request = Evaluation {
        schema_version: 1,
        binding: binding.clone(),
        requirements: vec![Requirement {
            rule_id: rule_id.clone(),
            mandatory: true,
        }],
        results: vec![RuleResult {
            rule_id,
            binding,
            decision,
            code: "hive.native-file-preflight".to_owned(),
            evidence_digest: None,
        }],
    };
    Ok((evaluate(&request).decision, checked_paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_denials_do_not_echo_raw_input_and_never_override_permissions() {
        let root = tempfile::tempdir().expect("root");
        for (host, mut payload) in [
            (
                Host::Codex,
                json!({"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Delete File: .hive/config/harness.toml\n*** End Patch"}}),
            ),
            (
                Host::Claude,
                json!({"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":".hive/config/harness.toml","content":"PRIVATE-SENTINEL"}}),
            ),
            (
                Host::Antigravity,
                json!({"toolCall":{"name":"write_to_file","args":{"TargetFile":".hive/config/harness.toml","CodeContent":"PRIVATE-SENTINEL"}}}),
            ),
        ] {
            payload["cwd"] = json!(root.path());
            payload["workspacePaths"] = json!([root.path()]);
            let result = respond(host, "PreToolUse", root.path(), &payload);
            let expected = if host == Host::Antigravity {
                &result["decision"]
            } else {
                &result["hookSpecificOutput"]["permissionDecision"]
            };
            assert_eq!(expected, "deny");
            assert!(!result.to_string().contains("PRIVATE-SENTINEL"));
        }
        let allowed = json!({"cwd":root.path(),"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"src/app.rs"}});
        assert_eq!(
            respond(Host::Claude, "PreToolUse", root.path(), &allowed),
            json!({})
        );
        assert_eq!(std::fs::read_dir(root.path()).expect("readonly").count(), 0);
    }

    #[test]
    fn malformed_unknown_and_move_targets_cannot_become_allow() {
        let root = tempfile::tempdir().expect("root");
        for payload in [
            json!({}),
            json!({"hook_event_name":"PostToolUse","tool_name":"Write"}),
            json!({"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"anything"}}),
        ] {
            assert_eq!(
                respond(Host::Claude, "PreToolUse", root.path(), &payload)["hookSpecificOutput"]
                    ["permissionDecision"],
                "deny"
            );
        }
        let moved = json!({"cwd":root.path(),"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Update File: src/app.rs\n*** Move to: .hive/config/harness.toml\n@@\n-old\n+new\n*** End Patch"}});
        assert_eq!(
            respond(Host::Codex, "PreToolUse", root.path(), &moved)["hookSpecificOutput"]
                ["permissionDecision"],
            "deny"
        );
        assert!(patch_paths("*** Begin Patch\n*** End Patch").is_err());
        assert_eq!(
            respond(Host::Claude, "Stop", root.path(), &json!({})),
            json!({})
        );
    }

    #[test]
    fn antigravity_uses_required_permission_decision_and_injects_static_context_only_at_start() {
        let root = tempfile::tempdir().expect("root");
        let payload = json!({"workspacePaths":[root.path()],"toolCall":{"name":"write_to_file","args":{"TargetFile":"src/app.rs"}}});
        assert_eq!(
            respond(Host::Antigravity, "PreToolUse", root.path(), &payload)["decision"],
            "ask"
        );
        let start = respond(
            Host::Antigravity,
            "PreInvocation",
            root.path(),
            &json!({"invocationNum":0}),
        );
        assert!(start["injectSteps"][0]["ephemeralMessage"]
            .as_str()
            .is_some());
        for payload in [
            json!({"invocationNum":1}),
            json!({"invocationNum":-1}),
            json!({}),
        ] {
            assert_eq!(
                respond(Host::Antigravity, "PreInvocation", root.path(), &payload),
                json!({})
            );
        }
        assert_eq!(std::fs::read_dir(root.path()).expect("readonly").count(), 0);
    }
}
