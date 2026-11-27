//! `hive loop start|status|record|stop`: overnight loop records inside the pinned project.
//!
//! Hive prepares records and host continuation guidance only. It never launches a host or
//! worker process, never waits and never calls a provider: Claude continues with its official
//! `/loop` or scheduled tasks inside the interactive host, and Codex or Gemini run as official
//! non-interactive workers through a host-owned handler. Each exit writes a morning report.
use crate::run::{parse_options, read_explicit_file, required, FileSnapshot, PinnedTarget};
use hive_core::overnight::{
    cancel_proof, morning_report, recover, DecisionPoint, LoopBudget, LoopEvent, LoopGoal,
    LoopInput, LoopRun, LoopStatus, Scorecard, OVERNIGHT_SCHEMA_VERSION,
};
use hive_core::sha256_digest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const LOOPS_ROOT: &str = ".hive/loops";
const MAX_FILE_BYTES: usize = 256 * 1024;
const RECEIPT_LEASE_SECONDS: u64 = 15 * 60;

pub(crate) const USAGE: &str = "\
Overnight loop records. Hive prepares envelopes and guidance only; the host runs every model.

USAGE:
    hive loop start --goal <goal.md> --scorecard <scorecard.json> [--loop <id>] --target <dir> [--output json]
    hive loop status --loop <id> [--receipts <dir>] --target <dir> [--output json]
    hive loop record --loop <id> --event <event.json> --target <dir> [--output json]
    hive loop stop --loop <id> [--cancel-receipt <host-receipt.json>] --target <dir> [--output json]

scorecard.json holds {\"scorecard\":{\"items\":[...]},\"budget\":{...},\"decision_points\":[...]}.
An event is one host report: continued, planned, receipt, receipt-missing, checked, critiqued,
or {\"event\":\"judge\",\"remaining_percent\":N,\"irreversible_next\":\"<decision>\"}.
The loop commits only to loop/<id>; push, merge and release need a person.
";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScorecardFile {
    scorecard: Scorecard,
    budget: LoopBudget,
    #[serde(default)]
    decision_points: Vec<DecisionPoint>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JudgeEvent {
    #[allow(dead_code)]
    event: String,
    #[serde(default)]
    remaining_percent: Option<u8>,
    #[serde(default)]
    irreversible_next: Option<String>,
}

fn now() -> u64 {
    if std::env::var_os("HIVE_TEST_CLOCK").is_some() {
        if let Some(value) = std::env::var("HIVE_TEST_NOW")
            .ok()
            .and_then(|value| value.parse().ok())
        {
            return value;
        }
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

/// Whether the first argument names an overnight-loop subcommand.
pub(crate) fn handles(arguments: &[String]) -> bool {
    matches!(
        arguments.first().map(String::as_str),
        Some("start" | "status" | "record" | "stop")
    )
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.iter().any(|argument| argument == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let (value, code) = match execute(arguments) {
        Ok(value) => (value, ExitCode::SUCCESS),
        Err(message) => (
            json!({"schema_version":1,"action":"OvernightLoop","status":"error","exit_code":2,
                   "code":"hive.loop-invalid-input","message":message}),
            ExitCode::from(2),
        ),
    };
    if !crate::human_output::emit_if_human(&value) {
        println!("{value}");
    }
    code
}

fn execute(arguments: &[String]) -> Result<Value, String> {
    let command = arguments.first().map_or("", String::as_str);
    let options = parse_options(
        &arguments[1..],
        &[
            "--target",
            "--goal",
            "--scorecard",
            "--loop",
            "--event",
            "--receipts",
            "--cancel-receipt",
        ],
    )
    .map_err(|error| error.message().to_owned())?;
    let target = PinnedTarget::open(Path::new(
        required(&options, "--target").map_err(|e| e.message().to_owned())?,
    ))
    .map_err(|error| error.message().to_owned())?;
    match command {
        "start" => start(&target, &options),
        "status" => status(&target, &options),
        "record" => record(&target, &options),
        "stop" => stop(&target, &options),
        _ => Err("loop requires start, status, record or stop".to_owned()),
    }
}

fn option<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
}

fn read_file(path: &str) -> Result<Vec<u8>, String> {
    read_explicit_file(Path::new(path), MAX_FILE_BYTES).map_err(|error| error.message().to_owned())
}

fn loop_path(loop_id: &str, name: &str) -> PathBuf {
    Path::new(LOOPS_ROOT).join(loop_id).join(name)
}

fn json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write(
    target: &PinnedTarget,
    path: &Path,
    bytes: &[u8],
    expected: &FileSnapshot,
) -> Result<(), String> {
    target
        .ensure_owned_parent(path, Path::new(LOOPS_ROOT))
        .map_err(|error| error.message().to_owned())?;
    target
        .publish(path, expected, bytes)
        .map(|_| ())
        .map_err(|error| error.message().to_owned())
}

fn load(
    target: &PinnedTarget,
    loop_id: &str,
) -> Result<(LoopInput, LoopRun, FileSnapshot), String> {
    if !loop_id
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || loop_id.is_empty()
    {
        return Err("loop id must be a lowercase slug".into());
    }
    let read = |name: &str| -> Result<Vec<u8>, String> {
        target
            .read_optional(&loop_path(loop_id, name), MAX_FILE_BYTES)
            .map_err(|error| error.message().to_owned())?
            .ok_or_else(|| format!("no overnight loop named {loop_id}"))
    };
    let input: LoopInput =
        serde_json::from_slice(&read("input.json")?).map_err(|error| error.to_string())?;
    input.validate()?;
    let run_bytes = read("run.json")?;
    let run: LoopRun = serde_json::from_slice(&run_bytes).map_err(|error| error.to_string())?;
    if run.input_digest != input.digest()? || run.loop_id != loop_id {
        return Err("loop records do not belong together".into());
    }
    Ok((input, run, FileSnapshot::File(run_bytes)))
}

fn report(
    target: &PinnedTarget,
    run: &LoopRun,
    input: &LoopInput,
    remaining: Option<u8>,
) -> Result<String, String> {
    let korean = crate::human_output::language() == crate::user_setup::InterfaceLanguage::Ko;
    let text = morning_report(run, input, korean, remaining);
    let path = loop_path(&run.loop_id, "report.md");
    let current = target
        .snapshot(&path)
        .map_err(|error| error.message().to_owned())?;
    if !matches!(&current, FileSnapshot::File(bytes) if bytes == text.as_bytes()) {
        write(target, &path, text.as_bytes(), &current)?;
    }
    Ok(text)
}

fn guidance(run: &LoopRun) -> Value {
    json!({
        "claude": format!("In the interactive Claude host, continue with the official /loop or a scheduled task. Each iteration: read `hive loop status --loop {id}`, plan, record events with `hive loop record`, and stop when status is stopped.", id = run.loop_id),
        "codex": "A host-owned handler runs `codex exec` with the plan envelope and records the receipt with `hive loop record`.",
        "gemini": "A host-owned handler runs Gemini's official non-interactive mode the same way.",
        "next_dedupe_key": run.next_dedupe_key(),
        "branch": run.branch,
        "launched_processes": 0,
    })
}

fn start(target: &PinnedTarget, options: &[(&str, &str)]) -> Result<Value, String> {
    let goal_bytes = read_file(option(options, "--goal").ok_or("--goal is required")?)?;
    let goal_text =
        String::from_utf8(goal_bytes.clone()).map_err(|_| "goal document must be UTF-8")?;
    let title = goal_text
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .unwrap_or("Overnight goal")
        .trim()
        .to_owned();
    let acceptance: Vec<String> = goal_text
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty())
        .take(32)
        .collect();
    let file: ScorecardFile = serde_json::from_slice(&read_file(
        option(options, "--scorecard").ok_or("--scorecard is required")?,
    )?)
    .map_err(|error| format!("invalid scorecard file: {error}"))?;
    let digest = sha256_digest(&goal_bytes);
    let loop_id = option(options, "--loop").map_or_else(
        || format!("loop-{}", &digest["sha256:".len().."sha256:".len() + 12]),
        str::to_owned,
    );
    let input = LoopInput {
        schema_version: OVERNIGHT_SCHEMA_VERSION,
        goal: LoopGoal {
            goal_id: loop_id.clone(),
            title,
            document_digest: digest,
            acceptance,
        },
        scorecard: file.scorecard,
        budget: file.budget,
        decision_points: file.decision_points,
    };
    let run = LoopRun::start(&loop_id, &input, now())?;
    let input_path = loop_path(&loop_id, "input.json");
    if !matches!(
        target
            .snapshot(&input_path)
            .map_err(|error| error.message().to_owned())?,
        FileSnapshot::Missing
    ) {
        return Err(format!(
            "overnight loop {loop_id} already exists; use status or stop"
        ));
    }
    write(
        target,
        &input_path,
        &json_bytes(&input)?,
        &FileSnapshot::Missing,
    )?;
    write(
        target,
        &loop_path(&loop_id, "run.json"),
        &json_bytes(&run)?,
        &FileSnapshot::Missing,
    )?;
    let report = report(target, &run, &input, None)?;
    Ok(
        json!({"schema_version":1,"action":"OvernightLoop","status":"success","exit_code":0,
        "code":"hive.loop-started","message":"overnight loop prepared; the host continues it",
        "data":{"loop":run,"continuation":guidance(&run),"report":report}}),
    )
}

fn status(target: &PinnedTarget, options: &[(&str, &str)]) -> Result<Value, String> {
    let loop_id = option(options, "--loop").ok_or("--loop is required")?;
    let (input, run, _) = load(target, loop_id)?;
    let receipts: BTreeSet<String> = match option(options, "--receipts") {
        Some(directory) => std::fs::read_dir(directory)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| {
                name.strip_suffix(".json")
                    .map(|key| format!("sha256:{key}"))
            })
            .collect(),
        None => BTreeSet::new(),
    };
    let recovery = recover(&run, &receipts, now(), RECEIPT_LEASE_SECONDS);
    let report = report(target, &run, &input, None)?;
    Ok(
        json!({"schema_version":1,"action":"OvernightLoop","status":"success","exit_code":0,
        "code":"hive.loop-status","message":"overnight loop status",
        "data":{"loop":run,"recovery":recovery,"continuation":guidance(&run),"report":report}}),
    )
}

fn record(target: &PinnedTarget, options: &[(&str, &str)]) -> Result<Value, String> {
    let loop_id = option(options, "--loop").ok_or("--loop is required")?;
    let (input, mut run, snapshot) = load(target, loop_id)?;
    let bytes = read_file(option(options, "--event").ok_or("--event is required")?)?;
    let raw: Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid event: {error}"))?;
    let mut remaining = None;
    let stopped = if raw.get("event").and_then(Value::as_str) == Some("judge") {
        let judge: JudgeEvent =
            serde_json::from_value(raw).map_err(|error| format!("invalid judge event: {error}"))?;
        remaining = judge.remaining_percent;
        run.judge(
            &input,
            now(),
            judge.remaining_percent,
            judge.irreversible_next.as_deref(),
        )?
    } else {
        let event: LoopEvent =
            serde_json::from_value(raw).map_err(|error| format!("invalid event: {error}"))?;
        run.apply(&input, &event)?;
        None
    };
    write(
        target,
        &loop_path(loop_id, "run.json"),
        &json_bytes(&run)?,
        &snapshot,
    )?;
    let report = report(target, &run, &input, remaining)?;
    Ok(
        json!({"schema_version":1,"action":"OvernightLoop","status":"success","exit_code":0,
        "code":"hive.loop-recorded","message":"event recorded",
        "data":{"loop":run,"stopped":stopped,"report":report}}),
    )
}

fn stop(target: &PinnedTarget, options: &[(&str, &str)]) -> Result<Value, String> {
    let loop_id = option(options, "--loop").ok_or("--loop is required")?;
    let (input, mut run, snapshot) = load(target, loop_id)?;
    let receipt = option(options, "--cancel-receipt")
        .map(read_file)
        .transpose()?
        .is_some();
    let proof = cancel_proof(&run, receipt);
    if matches!(run.status, LoopStatus::Running) {
        run.apply(&input, &LoopEvent::Cancel)?;
        write(
            target,
            &loop_path(loop_id, "run.json"),
            &json_bytes(&run)?,
            &snapshot,
        )?;
    }
    let report = report(target, &run, &input, None)?;
    Ok(
        json!({"schema_version":1,"action":"OvernightLoop","status":"success","exit_code":0,
        "code":"hive.loop-stopped","message":"overnight loop stopped; no new envelope will be prepared",
        "data":{"loop":run,"cancel_proof":proof,"new_envelopes":0,"report":report}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn start_status_record_and_stop_never_launch_a_process() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("project");
        std::fs::create_dir_all(target.join(".hive")).unwrap();
        let goal = root.path().join("goal.md");
        std::fs::write(
            &goal,
            "# One static website\n\n- builds\n- passes accessibility\n",
        )
        .unwrap();
        let scorecard = root.path().join("scorecard.json");
        std::fs::write(&scorecard, r#"{"scorecard":{"items":[{"id":"build","kind":"build","weight":0.7,"pass_threshold":1.0},{"id":"review","kind":"critique","weight":0.3,"pass_threshold":0.8}]},"budget":{"max_iterations":3,"max_wall_seconds":3600,"usage_floor_percent":10,"plateau_iterations":2,"repeated_failure_limit":2},"decision_points":[]}"#).unwrap();
        let target_text = target.to_str().unwrap();
        let started = execute(&arguments(&[
            "start",
            "--goal",
            goal.to_str().unwrap(),
            "--scorecard",
            scorecard.to_str().unwrap(),
            "--loop",
            "site",
            "--target",
            target_text,
        ]))
        .unwrap();
        assert_eq!(started["code"], "hive.loop-started");
        assert_eq!(started["data"]["continuation"]["launched_processes"], 0);
        assert_eq!(started["data"]["loop"]["branch"], "loop/site");
        assert!(target.join(".hive/loops/site/report.md").is_file());
        let again = execute(&arguments(&[
            "start",
            "--goal",
            goal.to_str().unwrap(),
            "--scorecard",
            scorecard.to_str().unwrap(),
            "--loop",
            "site",
            "--target",
            target_text,
        ]));
        assert!(again.unwrap_err().contains("already exists"));
        let key = started["data"]["continuation"]["next_dedupe_key"]
            .as_str()
            .unwrap()
            .to_owned();
        let event = root.path().join("event.json");
        std::fs::write(
            &event,
            json!({"event":"continued","dedupe_key":key,"worker":"codex","now":now()}).to_string(),
        )
        .unwrap();
        let recorded = execute(&arguments(&[
            "record",
            "--loop",
            "site",
            "--event",
            event.to_str().unwrap(),
            "--target",
            target_text,
        ]))
        .unwrap();
        assert_eq!(
            recorded["data"]["loop"]["iterations"][0]["phase"],
            "continued"
        );
        let duplicate = execute(&arguments(&[
            "record",
            "--loop",
            "site",
            "--event",
            event.to_str().unwrap(),
            "--target",
            target_text,
        ]));
        assert!(duplicate.is_err());
        let status = execute(&arguments(&[
            "status",
            "--loop",
            "site",
            "--target",
            target_text,
        ]))
        .unwrap();
        assert_eq!(status["data"]["recovery"]["action"], "wait-for-receipt");
        let stopped = execute(&arguments(&[
            "stop",
            "--loop",
            "site",
            "--target",
            target_text,
        ]))
        .unwrap();
        assert_eq!(stopped["data"]["cancel_proof"], "unproven");
        assert_eq!(stopped["data"]["new_envelopes"], 0);
        assert_eq!(stopped["data"]["loop"]["status"]["reason"], "cancelled");
        let report = std::fs::read_to_string(target.join(".hive/loops/site/report.md")).unwrap();
        assert!(report.contains("cancelled"));
        let after = execute(&arguments(&[
            "record",
            "--loop",
            "site",
            "--event",
            event.to_str().unwrap(),
            "--target",
            target_text,
        ]));
        assert!(after.unwrap_err().contains("already stopped"));
        assert!(execute(&arguments(&[
            "status",
            "--loop",
            "../x",
            "--target",
            target_text
        ]))
        .is_err());
    }
}
