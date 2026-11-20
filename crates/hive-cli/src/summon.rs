//! Inert inbox preparation through the existing signed orchestration transaction.
//!
//! A repeated request for an already prepared envelope (for example from a new host session)
//! returns the existing envelope, result and receipts with the request and run ids, without
//! consuming authority or preparing anything again. `hive msg inbox` and `hive msg status`
//! read the same `.hive/runs/<run>/inbox/` envelopes through [`inbox_entries`] and
//! [`existing_state`]. Host execution stays disabled.
use crate::run::{parse_options, read_explicit_file, required, run_path, PinnedTarget};
use crate::ActionResult;
use hive_core::{
    orchestration::{DispatchState, SummonEnvelope},
    roster::{CapabilityReport, Roster},
    sha256_digest,
};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

const USAGE:&str="hive summon <role|god> --task <text|@project-file> --target <dir> --user-root <home> --run <id> --action <id> --expected-head <head> --control-epoch <n> --authority-id <id> [--output json]\nWithout --event: inert preview and request digest, no writes.\nWith --event: also require --authority --trust-root --request-digest --now --capabilities; write prepared inbox only. Host execution remains disabled.\n";
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
            if !crate::human_output::emit_if_human(&value) {
                println!("{value}");
            }
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
#[allow(clippy::too_many_lines)]
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
    if let Some(existing) = duplicate_request(&target, &envelope, &digest)? {
        return Ok(existing);
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

const MAX_SUMMON_RECORD_BYTES: usize = 1024 * 1024;

/// The exact same prepared request returns the existing state; a different one is refused.
fn duplicate_request(
    target: &PinnedTarget,
    envelope: &SummonEnvelope,
    digest: &str,
) -> Result<Option<serde_json::Value>, String> {
    let path = run_path(
        &envelope.run_id,
        &format!("inbox/{}.json", envelope.action_id),
    )
    .map_err(|e| e.message().to_owned())?;
    let Some(existing) = target
        .read_optional(&path, MAX_SUMMON_RECORD_BYTES)
        .map_err(|e| e.message().to_owned())?
    else {
        return Ok(None);
    };
    let expected = serde_json::to_vec_pretty(envelope).map_err(|e| e.to_string())?;
    if existing != expected {
        return Err(
            "a different summon envelope is already prepared for this run action; nothing was replaced"
                .into(),
        );
    }
    let state = existing_state(target, &envelope.run_id, &envelope.action_id)?
        .ok_or("prepared summon envelope disappeared")?;
    serde_json::to_value(ActionResult {
        schema_version: 1,
        action: "Summon",
        status: "success",
        exit_code: 0,
        code: "hive.summon-duplicate",
        message: "duplicate summon request: returned the existing prepared state; nothing was prepared or run again".into(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({"duplicate":true,"request_digest":digest,"run_id":envelope.run_id,
            "action_id":envelope.action_id,"existing":state,"mutation_count":0,"enabled":false,
            "redispatched":false})),
    })
    .map(Some)
    .map_err(|e| e.to_string())
}

/// Read-only state of one prepared summon: envelope, result and matching receipts.
pub(crate) fn existing_state(
    target: &PinnedTarget,
    run: &str,
    action: &str,
) -> Result<Option<serde_json::Value>, String> {
    if !hive_core::mediation::is_valid_id(run) || !hive_core::mediation::is_valid_id(action) {
        return Err("summon run and action must be plain ids".into());
    }
    let inbox =
        run_path(run, &format!("inbox/{action}.json")).map_err(|e| e.message().to_owned())?;
    let Some(bytes) = target
        .read_optional(&inbox, MAX_SUMMON_RECORD_BYTES)
        .map_err(|e| e.message().to_owned())?
    else {
        return Ok(None);
    };
    let base = json!({"kind":"summon","id":format!("summon:{run}:{action}"),"run_id":run,
        "action_id":action,"envelope_path":crate::run::portable_relative_path(&inbox),
        "enabled":false,"auto_redispatch":false});
    let envelope = match serde_json::from_slice::<SummonEnvelope>(&bytes)
        .map_err(|e| e.to_string())
        .and_then(|envelope| {
            envelope.validate()?;
            if envelope.run_id != run || envelope.action_id != action {
                return Err("summon envelope is stored outside its run action".into());
            }
            Ok(envelope)
        }) {
        Ok(envelope) => envelope,
        Err(reason) => {
            let mut value = base;
            value["state"] = json!("quarantined");
            value["reason"] = json!(reason);
            return Ok(Some(value));
        }
    };
    let result = target
        .read_optional(Path::new(&envelope.result_path), MAX_SUMMON_RECORD_BYTES)
        .map_err(|e| e.message().to_owned())?;
    let receipts_dir = run_path(run, "receipts").map_err(|e| e.message().to_owned())?;
    let mut receipts = Vec::new();
    let marker = format!("action_id = \"{action}\"");
    for (name, kind) in crate::msg::list_entries(target, &receipts_dir)
        .map_err(|e| e.message().to_owned())?
        .unwrap_or_default()
    {
        if kind != crate::msg::EntryKind::File
            || Path::new(&name).extension().is_none_or(|ext| ext != "md")
        {
            continue;
        }
        let path = receipts_dir.join(&name);
        if let Some(receipt) = target
            .read_optional(&path, MAX_SUMMON_RECORD_BYTES)
            .map_err(|e| e.message().to_owned())?
        {
            if String::from_utf8_lossy(&receipt)
                .lines()
                .any(|line| line == marker)
            {
                receipts.push(json!({"path":crate::run::portable_relative_path(&path),
                    "digest":sha256_digest(&receipt)}));
            }
        }
    }
    let mut value = base;
    value["state"] = json!(envelope.state);
    value["role"] = json!(envelope.role);
    value["request_digest"] = json!(envelope.digest()?);
    value["idempotency_key"] = json!(envelope.idempotency_key);
    value["result"] = json!({"path":envelope.result_path,"present":result.is_some(),
        "digest":result.as_ref().map(|bytes| sha256_digest(bytes)),"trust":hive_core::mediation::UNTRUSTED_DATA});
    value["receipts"] = json!(receipts);
    Ok(Some(value))
}

/// Every prepared summon envelope in `.hive/runs/*/inbox/`, optionally for one role.
pub(crate) fn inbox_entries(
    target: &PinnedTarget,
    role: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    let runs = Path::new(".hive/runs");
    let mut entries = Vec::new();
    for (run, kind) in crate::msg::list_entries(target, runs)
        .map_err(|e| e.message().to_owned())?
        .unwrap_or_default()
    {
        if kind != crate::msg::EntryKind::Dir || !hive_core::mediation::is_valid_id(&run) {
            continue;
        }
        let inbox = runs.join(&run).join("inbox");
        for (name, kind) in crate::msg::list_entries(target, &inbox)
            .map_err(|e| e.message().to_owned())?
            .unwrap_or_default()
        {
            let Some(action) = name.strip_suffix(".json") else {
                continue;
            };
            if kind != crate::msg::EntryKind::File || !hive_core::mediation::is_valid_id(action) {
                continue;
            }
            if let Some(state) = existing_state(target, &run, action)? {
                if role.is_none_or(|role| state["role"] == role || state["state"] == "quarantined")
                {
                    entries.push(state);
                }
            }
        }
    }
    Ok(entries)
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

    #[test]
    #[allow(clippy::too_many_lines)]
    fn summon_duplicate_request_returns_existing_state_without_preparing_again() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().to_str().unwrap();
        let profile = Roster::defaults()
            .unwrap()
            .worker_profile("hive-routine-implementer")
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
        std::fs::write(&report_path, serde_json::to_vec(&report).unwrap()).unwrap();
        let mut args: Vec<String> = [
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
            "--capabilities",
            report_path.to_str().unwrap(),
        ]
        .map(str::to_owned)
        .to_vec();
        let preview = execute(&args).unwrap();
        let envelope: SummonEnvelope =
            serde_json::from_value(preview["data"]["envelope"].clone()).unwrap();
        let digest = preview["data"]["request_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        // A new session repeats the request after the envelope was prepared. The signed
        // authority files do not exist: a duplicate never reaches authority consumption.
        let missing = root.path().join("consumed-authority.toml");
        args.extend(
            [
                "--event",
                missing.to_str().unwrap(),
                "--authority",
                missing.to_str().unwrap(),
                "--trust-root",
                missing.to_str().unwrap(),
                "--request-digest",
                &digest,
                "--now",
                "2026-11-20T00:00:00Z",
            ]
            .map(str::to_owned),
        );
        assert!(
            execute(&args).is_err(),
            "no envelope yet: normal prepare path"
        );
        let inbox = root.path().join(".hive/runs/run-1/inbox");
        std::fs::create_dir_all(&inbox).unwrap();
        std::fs::create_dir_all(root.path().join(".hive/runs/run-1/results")).unwrap();
        std::fs::create_dir_all(root.path().join(".hive/runs/run-1/receipts")).unwrap();
        std::fs::write(
            inbox.join("task-1.json"),
            serde_json::to_vec_pretty(&envelope).unwrap(),
        )
        .unwrap();
        std::fs::write(root.path().join(&envelope.result_path), b"RESULT_OK").unwrap();
        std::fs::write(
            root.path().join(".hive/runs/run-1/receipts/receipt-1.md"),
            "+++\nrun_id = \"run-1\"\naction_id = \"task-1\"\n+++\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join(".hive/runs/run-1/receipts/receipt-2.md"),
            "+++\nrun_id = \"run-1\"\naction_id = \"task-2\"\n+++\n",
        )
        .unwrap();
        let duplicate = execute(&args).unwrap();
        hive_core::validate_json_schema(
            include_str!("../../../schemas/action-result.schema.json"),
            &duplicate,
            "summon duplicate",
        )
        .unwrap();
        assert_eq!(duplicate["code"], "hive.summon-duplicate");
        let data = &duplicate["data"];
        assert_eq!(data["duplicate"], true);
        assert_eq!(data["request_digest"], digest.as_str());
        assert_eq!(data["run_id"], "run-1");
        assert_eq!(data["action_id"], "task-1");
        assert_eq!(data["mutation_count"], 0);
        assert_eq!(data["enabled"], false);
        assert_eq!(data["existing"]["state"], "prepared");
        assert_eq!(data["existing"]["result"]["present"], true);
        assert_eq!(
            data["existing"]["result"]["digest"],
            sha256_digest(b"RESULT_OK")
        );
        assert_eq!(data["existing"]["receipts"].as_array().unwrap().len(), 1);
        assert!(!root.path().join(".hive/runs/run-1/tasks").exists());
        // A different envelope for the same run action is refused, never replaced.
        let mut other = envelope;
        other.authority_id = "auth-2".into();
        std::fs::write(
            inbox.join("task-1.json"),
            serde_json::to_vec_pretty(&other).unwrap(),
        )
        .unwrap();
        assert!(execute(&args)
            .unwrap_err()
            .contains("different summon envelope"));
    }
}
