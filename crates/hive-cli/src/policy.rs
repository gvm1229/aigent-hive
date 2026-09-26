//! Read-only policy evaluation. Native hook translation is a separate surface.

mod configure;
mod directive_context;
mod native;

use crate::{emit_action_result, ActionResult};
use hive_core::policy::{evaluate, Decision, Evaluation};
use serde_json::json;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

const LIMIT: u64 = 1024 * 1024;

fn read_request(path: &Path) -> Result<Evaluation, &'static str> {
    let metadata = path
        .symlink_metadata()
        .map_err(|_| "cannot inspect policy input")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > LIMIT {
        return Err("policy input must be a bounded regular file");
    }
    let file = std::fs::File::open(path).map_err(|_| "cannot open policy input")?;
    let opened = file
        .metadata()
        .map_err(|_| "cannot inspect opened policy input")?;
    if !opened.is_file() || opened.len() > LIMIT {
        return Err("opened policy input is not a bounded regular file");
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read policy input")?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > LIMIT {
        return Err("policy input exceeded the size bound");
    }
    let instance: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid policy JSON")?;
    hive_core::validate_json_schema(
        include_str!("../../../schemas/policy-evaluation.schema.json"),
        &instance,
        "policy input",
    )
    .map_err(|_| "policy input violates its schema")?;
    serde_json::from_value(instance).map_err(|_| "policy input violates its typed contract")
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments
        .first()
        .is_some_and(|argument| argument == "hooks")
    {
        return configure::run(&arguments[1..]);
    }
    if arguments.first().is_some_and(|argument| argument == "hook") {
        return native::run(&arguments[1..]);
    }
    if arguments == ["--help"] || arguments == ["evaluate", "--help"] {
        println!("Policy evaluation and explicit native hook configuration.\n\nhive policy evaluate --input <json> --output json\nhive policy hook --help\nhive policy hooks --help\nEvaluation is evidence, never execution authority.");
        return ExitCode::SUCCESS;
    }
    let result = parse_and_evaluate(arguments);
    emit_action_result(&result)
}

fn parse_and_evaluate(arguments: &[String]) -> ActionResult {
    let mut result = ActionResult {
        schema_version: 1,
        action: "EvaluatePolicy",
        status: "error",
        exit_code: 2,
        code: "hive.policy-invalid-input",
        message: "expected evaluate --input <json> --output json".to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    };
    if arguments.len() != 5
        || arguments[0] != "evaluate"
        || arguments[1] != "--input"
        || arguments[3] != "--output"
        || arguments[4] != "json"
    {
        return result;
    }
    let request = match read_request(Path::new(&arguments[2])) {
        Ok(request) => request,
        Err(message) => {
            message.clone_into(&mut result.message);
            return result;
        }
    };
    let outcome = evaluate(&request);
    (result.status, result.exit_code) = match outcome.decision {
        Decision::Allow => ("success", 0),
        Decision::Deny => ("blocked", 3),
        Decision::Unsupported => ("unsupported", 4),
        Decision::Error => ("verification-failed", 5),
    };
    result.code = "hive.policy-evaluated";
    "policy evidence evaluated; no operation was executed or authorized"
        .clone_into(&mut result.message);
    result.data = Some(json!({"evaluation":outcome}));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_requests_are_bounded_and_never_echo_input_content() {
        let temp = tempfile::tempdir().expect("temporary input");
        let input = temp.path().join("request.json");
        std::fs::write(&input, "secret fixture content, not JSON").expect("write input");
        let args = [
            "evaluate",
            "--input",
            input.to_str().expect("path"),
            "--output",
            "json",
        ]
        .map(str::to_owned);
        let result = parse_and_evaluate(&args);
        assert_eq!(result.exit_code, 2);
        assert!(!result.message.contains("secret"));
        assert!(result.changed_paths.is_empty());
        assert!(read_request(temp.path()).is_err());
    }

    #[test]
    fn valid_aggregate_remains_a_read_only_report() {
        let temp = tempfile::tempdir().expect("temporary input");
        let input = temp.path().join("request.json");
        let binding = json!({"operation_id":"op-1","policy_digest":format!("sha256:{}","a".repeat(64)),"target_digest":format!("sha256:{}","b".repeat(64))});
        let payload = json!({"schema_version":1,"binding":binding,"requirements":[{"rule_id":"path","mandatory":true}],"results":[{"rule_id":"path","binding":binding,"decision":"allow","code":"checked"}]});
        let bytes = serde_json::to_vec(&payload).expect("encode");
        std::fs::write(&input, &bytes).expect("write");
        let args = [
            "evaluate",
            "--input",
            input.to_str().expect("path"),
            "--output",
            "json",
        ]
        .map(str::to_owned);
        let result = parse_and_evaluate(&args);
        assert_eq!(result.exit_code, 0);
        assert_eq!(
            result.data.expect("data")["evaluation"]["authorizes_mutation"],
            false
        );
        assert_eq!(std::fs::read(&input).expect("read"), bytes);
        assert_eq!(std::fs::read_dir(temp.path()).expect("entries").count(), 1);
    }
}
