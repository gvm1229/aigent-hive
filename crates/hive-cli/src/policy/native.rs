//! Native host protocol adapter for deterministic file-edit preflight.
//! Shell, terminal continuation and MCP effects remain outside this classifier.

use hive_core::policy::{evaluate, Binding, Decision, Evaluation, Requirement, RuleResult};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

const LIMIT: u64 = 1024 * 1024;
const CONTEXT: &str = "Hive policy: preserve owned state through Hive commands; recheck current evidence before mutations. Hook delivery does not prove compliance. Shell, terminal continuation and MCP effects require their own mutation-boundary checks.";

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
        println!("hive policy hook --host codex|claude|antigravity --event PreToolUse|SessionStart|PreInvocation|Stop --target <dir> --stdin-json\nNative file-edit policy only; registration and host trust are separate.");
        return ExitCode::SUCCESS;
    }
    if args.len() != 7
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
    // A turn ending is neither task success nor authority to continue or capture memory.
    if args[3] == "Stop" {
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
        Ok(payload) => respond(host, &args[3], Path::new(&args[5]), &payload),
        Err(reason) if args[3] == "PreToolUse" => deny(host, reason),
        Err(_) => json!({}),
    };
    println!("{output}");
    // Exit 0 carries native JSON denial. Hive's legacy exit 3 is never forwarded.
    ExitCode::SUCCESS
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

fn respond(host: Host, event: &str, target: &Path, payload: &Value) -> Value {
    if (host == Host::Antigravity && event == "PreInvocation")
        || (host != Host::Antigravity && event == "SessionStart")
    {
        return match host {
            Host::Antigravity => json!({"injectSteps":[{"ephemeralMessage":CONTEXT}]}),
            _ => {
                json!({"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":CONTEXT}})
            }
        };
    }
    if event != "PreToolUse" {
        return json!({});
    }
    let checked = check_files(host, target, payload);
    match checked {
        Ok(Decision::Allow) => json!({}), // Never override another host permission decision.
        Ok(_) => deny(
            host,
            "Hive protected state requires a Hive-owned command; direct file mutation refused",
        ),
        Err(reason) => deny(host, reason),
    }
}

fn check_files(host: Host, target: &Path, payload: &Value) -> Result<Decision, &'static str> {
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
        let relative = crate::normalize_hook_path(&target, absolute)
            .map_err(|_| "unsafe native file target")?
            .ok_or("file target is outside the registered policy scope")?;
        hive_core::ensure_no_symlink_ancestors(&target, &relative)
            .map_err(|_| "native file target has an unsafe path ancestor")?;
        let input = serde_json::from_value::<crate::HookInput>(json!({"schema_version":1,
            "event":"PreToolUse", "operation":"overwrite", "path":absolute}))
        .map_err(|_| "invalid normalized file operation")?;
        let result = crate::protect_hive_owned_state(&target, &input)
            .map_err(|_| "file target cannot be safely normalized")?;
        if result.decision == "block" {
            decision = Decision::Deny;
        }
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
    Ok(evaluate(&request).decision)
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
}
