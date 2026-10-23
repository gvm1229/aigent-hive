//! One machine result; user guidance uses stderr.
use super::{completion::Report, FlowOutcome, Language};
use crate::ActionResult;
use serde_json::json;
use std::{path::Path, process::ExitCode};

fn result(
    status: &'static str,
    code: &'static str,
    exit_code: u8,
    message: String,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "UpdateHiveUser",
        status,
        exit_code,
        code,
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

pub(super) fn emit(json_output: bool, result: &ActionResult) -> ExitCode {
    if json_output {
        return crate::emit_action_result(result);
    }
    eprintln!("{}", result.message);
    if let Some(next) = &result.next_action {
        eprintln!("{next}");
    }
    ExitCode::from(result.exit_code)
}

pub(super) fn error(json_output: bool, message: String) -> ExitCode {
    emit(
        json_output,
        &result("error", "hive.update-failed", 2, message),
    )
}

pub(super) fn needs_input(json_output: bool, message: &str) -> ExitCode {
    emit(
        json_output,
        &result("needs-input", "hive.update-needs-input", 2, message.into()),
    )
}

pub(super) fn handoff(json_output: bool, language: Language, record: Option<&Path>) -> ExitCode {
    let mut result = result(
        "pending",
        "hive.update-handoff-pending",
        2,
        match language {
            Language::Ko => "별도 프로세스에서 갱신이 계속됩니다. 아직 완료되지 않았습니다.",
            Language::En => "The update continues in a child process. Completion is pending.",
        }
        .into(),
    );
    result.data = Some(json!({"completion_record": record, "completion_verified": false}));
    emit(json_output, &result)
}

pub(super) fn finished(
    outcome: Result<FlowOutcome, String>,
    completion: Option<&Report>,
    language: Language,
) -> ActionResult {
    let language = completion.map_or(language, |report| report.summary(language, &[]).0);
    let (mut result, outcome_name) = match outcome {
        Ok(FlowOutcome::Declined) => (
            result(
                "success",
                "hive.update-declined",
                0,
                match language {
                    Language::Ko => "갱신을 취소했습니다.",
                    Language::En => "Update declined.",
                }
                .into(),
            ),
            "declined",
        ),
        Ok(value) => (
            result(
                "success",
                "hive.update-complete",
                0,
                match language {
                    Language::Ko => "갱신과 사용자 설정 검증을 마쳤습니다.",
                    Language::En => "Update and user setup validation completed.",
                }
                .into(),
            ),
            if value == FlowOutcome::Installed {
                "installed"
            } else {
                "reconciled"
            },
        ),
        Err(message) => {
            let failed = completion.filter(|report| !report.succeeded());
            let pending = completion.is_some_and(|report| report.status == "needs-input");
            (
                result(
                    failed.map_or("error", |report| match report.status.as_str() {
                        "needs-input" => "needs-input",
                        "pending" => "pending",
                        "blocked" => "blocked",
                        "conflict" => "conflict",
                        "unsupported" => "unsupported",
                        "verification-failed" => "verification-failed",
                        _ => "error",
                    }),
                    if pending {
                        "hive.update-input-pending"
                    } else {
                        "hive.update-failed"
                    },
                    failed.map_or(2, |report| report.exit_code),
                    message,
                ),
                "incomplete",
            )
        }
    };
    if let Some(completion) = completion {
        result.changed_paths.clone_from(&completion.changed_paths);
        result.next_action.clone_from(&completion.next_action);
        if result.exit_code == 0 {
            result.message.clone_from(&completion.message);
            if completion.code == "hive.user-update-complete-pending-input" {
                result.code = "hive.update-complete-pending-input";
            }
        }
    }
    result.data = Some(json!({"outcome":outcome_name,"completion":completion}));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_setup_is_not_reported_as_a_complete_update() {
        let completion: Report = serde_json::from_value(json!({
            "action":"UpdateHiveUser","status":"needs-input","exit_code":2,
            "code":"hive.setup-needs-input","message":"Setup pending.",
            "next_action":"hive update", "data":null
        }))
        .unwrap();
        let result = finished(
            Err("Core updated; setup pending.".into()),
            Some(&completion),
            Language::En,
        );
        assert_eq!(result.status, "needs-input");
        assert_eq!(result.exit_code, 2);
        assert_eq!(result.next_action.as_deref(), Some("hive update"));
        assert_eq!(result.data.as_ref().unwrap()["outcome"], "incomplete");
        hive_core::validate_json_schema(
            include_str!("../../../../schemas/action-result.schema.json"),
            &serde_json::to_value(result).unwrap(),
            "update result",
        )
        .unwrap();
    }

    #[test]
    fn optional_questions_preserve_success_and_remain_visible() {
        let completion: Report = serde_json::from_value(json!({
            "action":"UpdateHiveUser","status":"success","exit_code":0,
            "code":"hive.user-update-complete-pending-input","message":"Questions pending.",
            "next_action":"hive update","data":{"pending_questions":["usage-warnings"]}
        }))
        .unwrap();
        let result = finished(Ok(FlowOutcome::Reconciled), Some(&completion), Language::En);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.code, "hive.update-complete-pending-input");
        assert_eq!(
            result.data.as_ref().unwrap()["completion"]["data"]["pending_questions"],
            json!(["usage-warnings"])
        );
    }
}
