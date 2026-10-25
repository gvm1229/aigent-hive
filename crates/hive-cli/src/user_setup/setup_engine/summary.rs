//! Opt-in setup summaries keep full diagnostic contracts available without repeating their data.
use crate::{user_setup::SetupError, ActionResult};
use serde_json::{json, Map, Value};

pub(crate) fn arguments(input: &[String]) -> Result<(Vec<String>, bool), SetupError> {
    let count = input
        .iter()
        .filter(|arg| arg.as_str() == "--summary")
        .count();
    if count > 1
        || (count == 1
            && (!input.windows(2).any(|pair| pair == ["--output", "json"])
                || input.iter().any(|arg| {
                    matches!(
                        arg.as_str(),
                        "--dry-run" | "--describe" | "--progress" | "feature"
                    )
                })))
    {
        return Err(SetupError::Input("--summary requires JSON setup status, apply, validation or resume; approval previews and describe remain separate".into()));
    }
    Ok((
        input
            .iter()
            .filter(|arg| arg.as_str() != "--summary")
            .cloned()
            .collect(),
        count == 1,
    ))
}

fn pick(value: &Value, keys: &[&str]) -> Value {
    Value::Object(
        keys.iter()
            .filter_map(|key| value.get(*key).map(|value| ((*key).into(), value.clone())))
            .collect(),
    )
}

fn source_counts(stored: &Value, sources: &Value) -> Value {
    let mut counts = Map::new();
    let choices = sources
        .as_object()
        .filter(|values| !values.is_empty())
        .or_else(|| {
            stored
                .get("preference_decisions")
                .and_then(Value::as_object)
        });
    for choice in choices.into_iter().flat_map(|choices| choices.values()) {
        let name = choice["origin"]
            .as_str()
            .filter(|name| {
                matches!(
                    *name,
                    "explicit"
                        | "accepted-default"
                        | "default"
                        | "inherited"
                        | "legacy-unknown"
                        | "external-change"
                )
            })
            .unwrap_or("unverified");
        let next = counts.get(name).and_then(Value::as_u64).unwrap_or(0) + 1;
        counts.insert(name.into(), json!(next));
    }
    Value::Object(counts)
}

fn add_status(data: &Value, summary: &mut Value) {
    let stored = &data["stored"];
    let pending = &data["pending"];
    summary["configured"] = json!(!stored.is_null());
    summary["stored_digest"] = if stored.is_null() {
        Value::Null
    } else {
        json!(hive_core::preferences::declaration_digest(stored).ok())
    };
    summary["settings"] = pick(
        stored,
        &[
            "interface_language",
            "selected_hosts",
            "setup_mode",
            "project_kind",
            "primary_host",
        ],
    );
    summary["settings"]["wiki"] = pick(&stored["wiki"], &["enabled", "language", "backend"]);
    summary["settings"]["persona"] = pick(&stored["persona"], &["id"]);
    let provenance = data
        .get("migration_preview")
        .and_then(|preview| preview.get("candidate"))
        .unwrap_or(stored);
    summary["source_counts"] = source_counts(provenance, &data["sources"]);
    summary["pending"] = if pending["phase"] == "complete" || pending.is_null() {
        Value::Null
    } else {
        pick(pending, &["operation_id", "phase", "step"])
    };
    summary["effective"] = pick(
        &data["effective"],
        &[
            "interface_language",
            "wiki_enabled",
            "wiki_language",
            "persona_id",
            "usage_stop_remaining_percent",
        ],
    );
    summary["effective_usage"] = data["effective_usage"].clone();
    summary["parent"] = data["parent"].clone();
}

fn add_completion(result: &ActionResult, data: &Value, summary: &mut Value) {
    for key in [
        "operation_id",
        "recovered",
        "resumed",
        "remaining_applied",
        "setup_state",
        "interface_language",
        "selected_hosts",
        "settings_draft_preserved",
    ] {
        if let Some(value) = data.get(key) {
            summary[key] = value.clone();
        }
    }
    summary["completion_verified"] = json!(
        result.status == "success"
            && result.exit_code == 0
            && matches!(
                result.code,
                "hive.setup-complete" | "hive.user-setup-complete" | "hive.project-setup-complete"
            )
    );
    summary["steps"] = data["steps"].clone();
    summary["effective_usage_policy"] = data["effective_usage_policy"].clone();
    summary["monitor_review"] = data["monitor_review"].clone();
    summary["host"] = data["host"].clone();
    summary["runtime_qualification"] = json!("not-evaluated-by-setup");
}

pub(crate) fn compact(mut result: ActionResult, requested: bool) -> ActionResult {
    if !requested {
        return result;
    }
    let data = result.data.take().unwrap_or(Value::Null);
    let status = matches!(result.code, "hive.setup-shown" | "hive.project-setup-shown");
    let stored = &data["stored"];
    let changed = result.changed_paths.len();
    let evidence_count = result.evidence.len();
    let mut summary = json!({
        "summary":true,"summary_kind":if status {"status"} else {"completion"},
        "model_calls":0,"model_work_started":false,"verification_scope":"configuration-and-owned-projections","authorizes_dispatch":false,"changed_path_count":changed,
        "changed_paths_omitted":changed>0,"evidence_count":evidence_count,
        "details":"Use the same explicit user root and project target with --show --output json, without --summary. Do not repeat apply to inspect its result."
    });
    if status {
        add_status(&data, &mut summary);
    } else {
        add_completion(&result, &data, &mut summary);
    }
    summary["pending_questions"] = data["pending_questions"].clone();
    let monitor = data
        .get("actual_monitor_state")
        .or_else(|| data.get("monitor_state"))
        .unwrap_or(&Value::Null);
    summary["monitor_state"] = pick(
        monitor,
        &[
            "configuration",
            "saved_enabled",
            "controller_state",
            "monitoring_verified",
            "stop_verified",
            "sensor_queries",
        ],
    );
    let guard = stored
        .get("usage_guard")
        .or_else(|| data.get("stored_usage_policy"))
        .or_else(|| data.get("usage_guard"))
        .unwrap_or(&Value::Null);
    summary["stored_usage_policy"] = pick(
        guard,
        &[
            "enforcement_enabled",
            "warnings_enabled",
            "stop_remaining_percent",
            "preflight",
            "midway",
            "enabled",
            "quota_reset_guard_enabled",
        ],
    );
    result.changed_paths.clear();
    result.evidence.clear();
    if result.message.len() > 768 {
        result.message =
            if data["interface_language"] == "ko" || stored["interface_language"] == "ko" {
                "설정 결과의 상세 정보는 기존 전체 조회 명령으로 확인할 수 있습니다."
            } else {
                "Setup result details are available through the unchanged full result contract."
            }
            .into();
        summary["message_omitted"] = json!(true);
    }
    if result
        .next_action
        .as_ref()
        .is_some_and(|action| action.len() > 1024)
    {
        result.next_action = None;
        summary["next_action_omitted"] = json!(true);
    }
    result.data = Some(summary);
    // The emitter appends one newline to the serialized result.
    let limit = if status { 4095 } else { 8191 };
    if serde_json::to_vec(&result).is_ok_and(|bytes| bytes.len() > limit) {
        result.data = Some(
            json!({"summary":true,"summary_kind":if status {"status"}else{"completion"},
            "details_omitted":true,"changed_path_count":changed,"evidence_count":evidence_count,
            "model_calls":0,"authorizes_dispatch":false,
            "details":"Inspect --show --output json using the same targets without --summary; do not repeat apply."}),
        );
    }
    result
}

pub(super) fn one_question(
    mut questions: Value,
    id: &str,
    korean: bool,
) -> Result<Value, SetupError> {
    let entry = questions
        .as_array_mut()
        .and_then(|items| {
            items
                .iter()
                .position(|entry| entry["question"]["id"] == id)
                .map(|index| items.remove(index))
        })
        .ok_or_else(|| SetupError::Input("unknown setup question".into()))?;
    Ok(json!([question(entry, korean)?]))
}

pub(super) fn question(mut entry: Value, korean: bool) -> Result<Value, SetupError> {
    let locale = if korean { "ko" } else { "en" };
    for field in ["prompt", "help"] {
        if let Some(text) = entry["question"][field].get(locale).cloned() {
            entry["question"][field] = text;
        }
    }
    if let Some(options) = entry["question"]["options"].as_array_mut() {
        for option in options {
            if let Some(label) = option["label"].get(locale).cloned() {
                option["label"] = label;
            }
        }
    }
    if serde_json::to_vec(&entry)
        .map_err(|e| SetupError::Internal(e.to_string()))?
        .len()
        > 7168
    {
        for field in ["current", "effective"] {
            if let Some(value) = entry
                .as_object_mut()
                .and_then(|object| object.remove(field))
            {
                entry[format!("{field}_omitted")] = json!({"digest":hive_core::preferences::declaration_digest(&value).map_err(SetupError::Internal)?,"reason":"value exceeds question response budget; use the explicit section view"});
            }
        }
    }
    if serde_json::to_vec(&entry)
        .map_err(|e| SetupError::Internal(e.to_string()))?
        .len()
        > 7168
    {
        return Err(SetupError::Input(
            "question exceeds the compact response budget; use the explicit section view".into(),
        ));
    }
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_preserves_status_and_bounds_large_preferences_without_claiming_approval() {
        let result = ActionResult {
            schema_version: 1,
            action: "SetupHiveUser",
            status: "success",
            exit_code: 0,
            code: "hive.setup-shown",
            message: "shown".into(),
            changed_paths: vec!["some-change".into()],
            evidence: vec![],
            next_action: None,
            data: Some(
                json!({"stored":{"persona":{"id":"custom","description":"x".repeat(64000)},"interface_language":"ko"},
                "pending":{"operation_id":"a".repeat(64),"phase":"awaiting-answers","answers":{"description":"x".repeat(64000)}}}),
            ),
        };
        let summary = compact(result, true);
        assert!(serde_json::to_vec(&summary).unwrap().len() <= 4096);
        assert_eq!(summary.exit_code, 0);
        let data = summary.data.unwrap();
        assert_eq!(data["configured"], true);
        assert_eq!(data["settings"]["persona"]["id"], "custom");
        assert_eq!(data["changed_path_count"], 1);
        assert_eq!(data["authorizes_dispatch"], false);
        assert!(data["pending"].get("answers").is_none());
    }

    #[test]
    fn a_large_current_answer_is_explicitly_omitted_from_one_localized_question() {
        let q = super::super::catalog()
            .unwrap()
            .questions
            .into_iter()
            .next()
            .unwrap();
        let entry = question(
            json!({"question":q,"current":"가".repeat(10_000),"source":"explicit"}),
            true,
        )
        .unwrap();
        assert!(serde_json::to_vec(&entry).unwrap().len() <= 7168);
        assert!(entry["question"]["prompt"].is_string());
        assert!(entry.get("current").is_none());
        assert!(entry["current_omitted"]["digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
        assert_eq!(entry["source"], "explicit");
    }

    #[test]
    fn summary_cannot_hide_an_approval_preview_or_change_the_input_mode() {
        assert!(arguments(&["--summary".into()]).is_err());
        assert!(arguments(&[
            "--summary".into(),
            "--output".into(),
            "json".into(),
            "--dry-run".into()
        ])
        .is_err());
        let (args, summary) = arguments(&[
            "--show".into(),
            "--summary".into(),
            "--output".into(),
            "json".into(),
        ])
        .unwrap();
        assert!(summary);
        assert_eq!(args, ["--show", "--output", "json"]);
    }
}
