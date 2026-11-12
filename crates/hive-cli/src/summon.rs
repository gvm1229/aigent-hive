//! Inert inbox preparation through the existing signed orchestration transaction.
use crate::run::{parse_options, read_explicit_file, required, PinnedTarget};
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
                        eprintln!("{error}");
                        return ExitCode::from(2);
                    }
                }
            }
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                json!({"status":"refused","enabled":false,"message":error})
            );
            ExitCode::from(2)
        }
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
    let role = roster.gods.get(role).unwrap_or(role);
    let profile = roster.worker_profile(role)?;
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
    let run = get("--run")?;
    let action = get("--action")?;
    let envelope = SummonEnvelope {
        schema_version: 1,
        run_id: run.into(),
        action_id: action.into(),
        role: role.into(),
        profile,
        task_file: format!(".hive/runs/{run}/tasks/{action}.md"),
        task_digest: sha256_digest(&task),
        target_project: target.requested_path().to_string_lossy().into_owned(),
        event_head: get("--expected-head")?.into(),
        control_epoch: get("--control-epoch")?
            .parse()
            .map_err(|_| "invalid control epoch")?,
        authority_id: get("--authority-id")?.into(),
        idempotency_key: format!("{run}-{action}"),
        result_path: format!(".hive/runs/{run}/results/{action}.json"),
        state: DispatchState::Prepared,
        enabled: false,
    };
    let digest = envelope.digest()?;
    if !options.iter().any(|(k, _)| *k == "--event") {
        return Ok(
            json!({"status":"preview","envelope":envelope,"request_digest":digest,"mutation_count":0,"enabled":false}),
        );
    }
    let report: CapabilityReport = serde_json::from_slice(
        &read_explicit_file(Path::new(get("--capabilities")?), 1024 * 1024)
            .map_err(|e| e.message().to_owned())?,
    )
    .map_err(|e| e.to_string())?;
    roster.check(role, &report)?;
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
    let result = crate::orchestration::prepare_summon(&mutation, &envelope, &task)?;
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
        assert_eq!(first["mutation_count"], 0);
        assert!(!root.path().join(".hive").exists());
        let mut bad = args;
        bad[2] = "@../escape".into();
        assert!(execute(&bad).is_err());
    }
}
