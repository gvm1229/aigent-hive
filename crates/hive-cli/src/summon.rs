//! Inert inbox preparation through the existing signed orchestration transaction.
use crate::run::{parse_options, read_explicit_file, required, PinnedTarget};
use crate::ActionResult;
use hive_core::{
    orchestration::{DispatchState, SummonEnvelope},
    roster::{CapabilityReport, Roster},
    sha256_digest,
};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

const USAGE:&str="hive summon <role|god> --task <text|@project-file> --target <dir> --user-root <home> --run <id> --action <id> --expected-head <head> --control-epoch <n> --authority-id <id> --output json\nWithout --event: inert preview and request digest, no writes.\nWith --event: also require --authority --trust-root --request-digest --now --capabilities; write prepared inbox only. Host execution remains disabled.\n";
pub(crate) fn run(args: &[String]) -> ExitCode {
    if args.iter().any(|a| a == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match execute(args) {
        Ok(mut value) => {
            if let Some(god) = args.first().and_then(|name| hive_core::gods::find(name)) {
                if let Some(user) = args
                    .windows(2)
                    .find(|pair| pair[0] == "--user-root")
                    .map(|pair| Path::new(&pair[1]))
                {
                    if let Err(error) = crate::gods::decorate(&mut value, god, user) {
                        return crate::emit_action_result(&refusal(error));
                    }
                }
            }
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => crate::emit_action_result(&refusal(error)),
    }
}
fn refusal(error: String) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "Summon",
        status: "error",
        exit_code: 2,
        code: "hive.summon-refused",
        message: error,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({"enabled":false})),
    }
}
fn execute(args: &[String]) -> Result<serde_json::Value, String> {
    let role = args.first().ok_or("missing role")?;
    let options = parse_options(
        &args[1..],
        &[
            "--task",
            "--target",
            "--user-root",
            "--run",
            "--action",
            "--expected-head",
            "--control-epoch",
            "--authority-id",
            "--event",
            "--authority",
            "--trust-root",
            "--request-digest",
            "--now",
            "--capabilities",
        ],
    )
    .map_err(|e| e.message().to_owned())?;
    let get = |key| required(&options, key).map_err(|e| e.message().to_owned());
    let target =
        PinnedTarget::open(Path::new(get("--target")?)).map_err(|e| e.message().to_owned())?;
    let roster = Roster::load(Path::new(get("--user-root")?), target.requested_path())?;
    let (role, provider, resolved) = roster.resolve_named(role)?;
    let profile = Roster::worker_profile_resolved(role, provider, &resolved)?;
    let capability_digest = options
        .iter()
        .find(|(key, _)| *key == "--capabilities")
        .map(|(_, path)| {
            let bytes = read_explicit_file(Path::new(path), 1024 * 1024)
                .map_err(|e| e.message().to_owned())?;
            let report = CapabilityReport::from_json_bytes(&bytes)?;
            roster.check_resolved(provider, &resolved, &report)?;
            Ok::<_, String>(sha256_digest(&bytes))
        })
        .transpose()?;
    let task_arg = get("--task")?;
    let task = if let Some(path) = task_arg.strip_prefix('@') {
        target
            .read_required(Path::new(path), 256 * 1024)
            .map_err(|e| e.message().to_owned())?
    } else {
        task_arg.as_bytes().to_vec()
    };
    if task.is_empty() || task.len() > 256 * 1024 {
        return Err("task must contain 1..262144 bytes".into());
    }
    let (run, action) = (get("--run")?, get("--action")?);
    let envelope = SummonEnvelope {
        schema_version: 1,
        run_id: run.into(),
        action_id: action.into(),
        role: role.into(),
        profile,
        task_file: format!(".hive/runs/{run}/tasks/{action}.md"),
        task_digest: sha256_digest(&task),
        capability_digest,
        target_digest: sha256_digest(target.requested_path().to_string_lossy().as_bytes()),
        event_head: get("--expected-head")?.into(),
        control_epoch: get("--control-epoch")?
            .parse()
            .map_err(|_| "invalid control epoch")?,
        authority_id: get("--authority-id")?.into(),
        idempotency_key: format!("{run}-{action}"),
        result_path: format!(".hive/runs/{run}/results/{action}.txt"),
        state: DispatchState::Prepared,
        enabled: false,
    };
    let digest = envelope.digest()?;
    if !options.iter().any(|(k, _)| *k == "--event") {
        return serde_json::to_value(ActionResult {
                schema_version: 1, action: "Summon", status: "success", exit_code: 0,
                code: "hive.summon-preview", message: "inert summon preview".into(),
                changed_paths: Vec::new(), evidence: Vec::new(), next_action: None,
                data: Some(json!({"preview":true,"envelope":envelope,"request_digest":digest,"mutation_count":0,"enabled":false})),
            }).map_err(|e| e.to_string());
    }
    get("--capabilities")?;
    if get("--request-digest")? != digest {
        return Err("request digest mismatch".into());
    }
    let mutation: Vec<String> = options
        .iter()
        .filter(|(k, _)| {
            ![
                "--task",
                "--user-root",
                "--action",
                "--authority-id",
                "--capabilities",
            ]
            .contains(k)
        })
        .flat_map(|(k, v)| [(*k).to_owned(), (*v).to_owned()])
        .collect();
    let mut result = crate::orchestration::prepare_summon(&mutation, &envelope, &task)?;
    result.action = "Summon";
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summon_preview_is_inert_and_rejects_path_escape() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().to_str().unwrap();
        let args = [
            "hive-routine-implementer",
            "--task",
            "hello",
            "--target",
            path,
            "--user-root",
            path,
            "--run",
            "run-1",
            "--action",
            "task-1",
            "--expected-head",
            "none",
            "--control-epoch",
            "0",
            "--authority-id",
            "auth-1",
            "--output",
            "json",
        ]
        .map(str::to_owned);
        let first = execute(&args).unwrap();
        assert_eq!(first, execute(&args).unwrap());
        for value in [
            &first,
            &serde_json::to_value(refusal("test refusal".into())).unwrap(),
        ] {
            hive_core::validate_json_schema(
                include_str!("../../../schemas/action-result.schema.json"),
                value,
                "summon action result",
            )
            .unwrap();
            assert_eq!(value["action"], "Summon");
        }
        assert_eq!(first["data"]["mutation_count"], 0);
        assert_eq!(
            first["data"]["envelope"]["target_digest"],
            sha256_digest(
                hive_core::normalize_platform_root(root.path())
                    .to_string_lossy()
                    .as_bytes()
            )
        );
        assert!(first["data"]["envelope"].get("target_project").is_none());
        assert!(!first.to_string().contains(path));
        assert!(!root.path().join(".hive").exists());
        let profile = Roster::defaults()
            .unwrap()
            .worker_profile(&args[0])
            .unwrap();
        let report = CapabilityReport {
            provider: "codex".into(),
            version: "0.167.2".into(),
            version_output: "codex-cli 0.167.2".into(),
            help_output:
                "--model --output-last-message --sandbox --skip-git-repo-check --ephemeral --cd"
                    .into(),
            login_status_output: "Logged in using ChatGPT".into(),
            models: std::collections::BTreeMap::from([(profile.model, vec![profile.effort])]),
            project_hook_events: vec!["UserPromptSubmit".into(), "Stop".into()],
        };
        let report_path = root.path().join("capabilities.json");
        let bytes = serde_json::to_vec(&report).unwrap();
        std::fs::write(&report_path, &bytes).unwrap();
        let mut qualified = args.to_vec();
        qualified.extend([
            "--capabilities".into(),
            report_path.to_str().unwrap().into(),
        ]);
        let bound = execute(&qualified).unwrap();
        assert_eq!(
            bound["data"]["envelope"]["capability_digest"],
            sha256_digest(&bytes)
        );
        std::fs::write(&report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        assert_ne!(
            bound["data"]["request_digest"],
            execute(&qualified).unwrap()["data"]["request_digest"]
        );
        let mut invalid = serde_json::to_value(&report).unwrap();
        invalid["version"] = json!("");
        std::fs::write(&report_path, invalid.to_string()).unwrap();
        assert!(execute(&qualified)
            .unwrap_err()
            .contains("JSON Schema contract"));
        let mut bad = args;
        bad[2] = "@../escape".into();
        assert!(execute(&bad).is_err());
    }
}
