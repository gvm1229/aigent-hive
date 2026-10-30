//! One machine result; user guidance uses stderr.
use super::{completion::Report, FlowOutcome, Language};
use crate::ActionResult;
use serde_json::{json, Value};
use std::process::ExitCode;

pub(super) struct CoreUpdate {
    pub from_version: Option<String>,
    pub to_version: Option<String>,
    pub requested_version: Option<String>,
    pub status: &'static str,
}

impl Default for CoreUpdate {
    fn default() -> Self {
        Self {
            from_version: None,
            to_version: None,
            requested_version: None,
            status: "not-started",
        }
    }
}

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
    for line in guidance(result) {
        eprintln!("{line}");
    }
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

pub(super) fn cleanup_pending(language: Language, view: &Value, resume: String) -> ActionResult {
    let mut result = result(
        "pending",
        "hive.update-cleanup-pending",
        2,
        match language {
            Language::Ko => "이전 갱신의 옛 실행 파일이 아직 사용 중입니다. 사용이 끝난 뒤 같은 명령으로 재개할 수 있습니다.",
            Language::En => "The previous program image is still in use. Run the same command after it exits.",
        }
        .into(),
    );
    result.data = Some(
        json!({"cleanup":view,"resume_command":resume,"completion_verified":false,"model_calls":0}),
    );
    result.next_action = Some(resume);
    result
}

pub(super) fn cleanup(
    result: &mut ActionResult,
    language: Language,
    mut view: Value,
    error: Option<String>,
) {
    let data = result.data.as_mut().expect("update result data");
    data["steps"]["cleanup"] = json!(if view["state"] == "complete" {
        "verified"
    } else {
        "pending"
    });
    if let Some(error) = error {
        view["error"] = json!(error);
        if result.exit_code == 0 {
            result.status = "pending";
            result.exit_code = 2;
            result.code = "hive.update-cleanup-pending";
            result.message = match language {
                Language::Ko => "프로그램과 설정 검증은 끝났지만 옛 실행 파일 정리가 남았습니다. 같은 명령으로 재개할 수 있습니다.",
                Language::En => "Program and setup validation finished, but old-image cleanup is pending. Resume with the same command.",
            }.into();
        }
    }
    data["cleanup"] = view;
}

pub(super) fn finished(
    outcome: Result<FlowOutcome, String>,
    completion: Option<&Report>,
    language: Language,
    core: &CoreUpdate,
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
                    failed.map_or(message, |report| report.message.clone()),
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
    let mut data = completion
        .and_then(|report| report.data.clone())
        .filter(serde_json::Value::is_object)
        .unwrap_or_else(|| json!({}));
    data["interface_language"] = json!(if language == Language::Ko { "ko" } else { "en" });
    data["from_version"] = json!(core.from_version);
    data["to_version"] = json!(core.to_version);
    data["requested_version"] = json!(core.requested_version);
    if !data["steps"].is_object() {
        data["steps"] =
            json!({"settings":"unverified","harness":"unverified","monitor":"unverified"});
    }
    data["steps"]["core"] = json!(core.status);
    data["outcome"] = json!(outcome_name);
    data["resume_command"] = json!(result.next_action);
    data["completion"] = json!(completion);
    result.data = Some(data);
    result
}

fn cleanup_guidance(data: &Value, ko: bool) -> Option<String> {
    let state = data["cleanup"]["state"].as_str()?;
    Some(
        match (ko, state) {
            (true, "complete") => "옛 실행 파일 정리: 완료",
            (true, "after-process-exit") => "옛 실행 파일 정리: 이 명령이 종료된 뒤 자동 정리 대기",
            (true, _) => "옛 실행 파일 정리: 미완료, 같은 갱신 명령으로 재개",
            (false, "complete") => "Old program image: cleanup verified",
            (false, "after-process-exit") => {
                "Old program image: cleanup is queued for this command's exit"
            }
            (false, _) => "Old program image: cleanup is incomplete; resume the same update",
        }
        .into(),
    )
}

pub(super) fn guidance(result: &ActionResult) -> Vec<String> {
    let Some(data) = &result.data else {
        return Vec::new();
    };
    let ko = data["interface_language"] == "ko";
    let mut lines = Vec::new();
    if let Some(line) = cleanup_guidance(data, ko) {
        lines.push(line);
    }
    if let Some(core) = data["steps"]["core"].as_str() {
        let state = match (ko, core) {
            (true, "verified") => "교체 검증 완료",
            (true, "unchanged") => "기존 버전 유지",
            (true, "not-run" | "not-started") => "교체 실행 없음",
            (true, _) => "교체 결과 확인 못함",
            (false, "verified") => "activation verified",
            (false, "unchanged") => "current version retained",
            (false, "not-run" | "not-started") => "replacement not run",
            (false, _) => "replacement unverified",
        };
        lines.push(if ko {
            format!("프로그램 본체: {state}")
        } else {
            format!("Program: {state}")
        });
    }
    if let Some(keys) = data["settings_changes"]["changed_keys"].as_array() {
        lines.push(if keys.is_empty() {
            if ko {
                "저장된 설정 유지. 사용자 자료와 프로젝트 파일 보존."
            } else {
                "Saved settings retained. User content and project files preserved."
            }
            .into()
        } else if ko {
            format!(
                "변경한 설정 항목: {}. 사용자 자료와 프로젝트 파일 보존.",
                keys.len()
            )
        } else {
            format!(
                "Changed settings: {}. User content and project files preserved.",
                keys.len()
            )
        });
        if !keys.is_empty() && !crate::display_options::quiet() {
            lines.push(if ko { "원하면 AI에게 설정 검토를 요청할 수 있습니다. 직접 설정에는 AI가 필요하지 않습니다." }
                else { "You can ask your AI host to review these settings. Direct setup needs no AI." }.into());
        }
    }
    if let Some(policy) = data["effective_usage_policy"].as_object() {
        let preflight = &policy["preflight"];
        let midway = &policy["midway"];
        let flag = |value: &serde_json::Value| {
            if value == true {
                if ko {
                    "켜짐"
                } else {
                    "on"
                }
            } else if ko {
                "꺼짐"
            } else {
                "off"
            }
        };
        lines.push(if ko {
            format!("전역 기본 보호: 작업 전 기준 {} · 재설정 {} · 0% {} · 작업 중 감시 {}. 세션·프로젝트별 예외는 별도입니다.",
                flag(&preflight["threshold_enabled"]),flag(&preflight["reset_enabled"]),flag(&preflight["zero_quota_enabled"]),flag(&midway["enabled"]))
        } else { format!("Global protection defaults: preflight threshold {} · reset {} · zero quota {} · monitoring {}. Session and project overrides are separate.",
            flag(&preflight["threshold_enabled"]),flag(&preflight["reset_enabled"]),flag(&preflight["zero_quota_enabled"]),flag(&midway["enabled"])) });
    }
    if let Some(state) = data["monitor_state"]["controller_state"].as_str() {
        let state = if ko {
            match state {
                "running" => "실행 중",
                "stopped" => "종료 확인",
                "not-configured" => "설정 없음",
                _ => "확인 못함",
            }
        } else {
            state
        };
        lines.push(if ko {
            format!("확인한 감시 실행 상태: {state}")
        } else {
            format!("Observed monitor state: {state}")
        });
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_cleanup_is_not_verified_and_cleanup_errors_preserve_prior_failures() {
        let core = CoreUpdate {
            status: "verified",
            ..CoreUpdate::default()
        };
        let mut result = finished(Ok(FlowOutcome::Installed), None, Language::Ko, &core);
        cleanup(
            &mut result,
            Language::Ko,
            json!({"state":"after-process-exit"}),
            None,
        );
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.data.as_ref().unwrap()["steps"]["cleanup"], "pending");
        assert!(guidance(&result)
            .iter()
            .any(|line| line.contains("자동 정리 대기")));
        cleanup(
            &mut result,
            Language::Ko,
            json!({"state":"unverified"}),
            Some("spawn failed".into()),
        );
        assert_eq!(result.exit_code, 2);
        assert_eq!(result.status, "pending");
        assert_eq!(result.data.as_ref().unwrap()["steps"]["core"], "verified");
        result.status = "verification-failed";
        result.exit_code = 5;
        result.message = "earlier setup failure".into();
        cleanup(
            &mut result,
            Language::Ko,
            json!({"state":"unverified"}),
            Some("cleanup failed".into()),
        );
        assert_eq!(result.exit_code, 5);
        assert_eq!(result.message, "earlier setup failure");
    }

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
            &CoreUpdate::default(),
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
        let result = finished(
            Ok(FlowOutcome::Reconciled),
            Some(&completion),
            Language::En,
            &CoreUpdate::default(),
        );
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.code, "hive.update-complete-pending-input");
        assert_eq!(
            result.data.as_ref().unwrap()["completion"]["data"]["pending_questions"],
            json!(["usage-warnings"])
        );
    }

    #[test]
    fn failed_settings_keep_the_verified_core_and_child_details() {
        let report: Report = serde_json::from_value(json!({
            "action":"UpdateHiveUser","status":"verification-failed","exit_code":5,
            "code":"hive.user-setup-verification-failed","message":"Settings incomplete.",
            "next_action":"hive update","data":{"steps":{"settings":"incomplete","harness":"unverified"},
                "monitor_state":{"controller_state":"stopped"},"effective_usage_policy":null}
        })).unwrap();
        let core = CoreUpdate {
            from_version: Some("0.13.1".into()),
            to_version: Some("0.14.0-test.1".into()),
            requested_version: Some("0.14.0-test.1".into()),
            status: "verified",
        };
        let result = finished(
            Err(report.message.clone()),
            Some(&report),
            Language::En,
            &core,
        );
        assert_eq!(result.exit_code, 5);
        assert_eq!(result.status, "verification-failed");
        let data = result.data.unwrap();
        assert_eq!(data["steps"]["core"], "verified");
        assert_eq!(data["steps"]["harness"], "unverified");
        assert_eq!(data["to_version"], "0.14.0-test.1");
        assert_eq!(data["monitor_state"]["controller_state"], "stopped");
        assert!(data["effective_usage_policy"].is_null());
    }

    #[test]
    fn unchanged_settings_do_not_repeat_optional_ai_help_or_claim_a_disable() {
        for (language, marker) in [("ko", "설정 유지"), ("en", "settings retained")] {
            let mut result = result("success", "test", 0, "done".into());
            result.data = Some(
                json!({"interface_language":language,"settings_changes":{"changed_keys":[]},
                "monitor_state":{"controller_state":"unverified"}}),
            );
            let lines = guidance(&result).join("\n");
            assert!(lines.contains(marker));
            assert!(!lines.contains("AI"));
            assert!(!lines.contains("disabled"));
            assert!(!lines.contains("해제"));
        }
    }
}
