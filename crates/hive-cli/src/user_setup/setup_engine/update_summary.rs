//! Update evidence is derived from saved settings and a read-only monitor inspection.
use crate::{
    user_setup::{self, SetupError, UserSetupConfig},
    ActionResult,
};
use hive_core::preferences::PreferenceScope;
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn baseline(root: &Path, config: Option<&UserSetupConfig>) -> Option<Value> {
    let config = config?;
    // A legacy comparison needs its original monitor choice. Unreadable state is unknown,
    // never proof that a setting was disabled by this update.
    let migrated = user_setup::migration::inspect(root, config).ok()?;
    serde_json::to_value(
        migrated
            .as_ref()
            .map_or(config, |preview| &preview.candidate),
    )
    .ok()
}

pub(super) fn incomplete(root: &Path, result: &mut ActionResult) {
    let data = result.data.get_or_insert_with(|| json!({}));
    data["steps"] = json!({"settings":"incomplete","harness":"unverified","monitor":"unverified"});
    data["effective_usage_policy"] = Value::Null;
    data["monitor_state"] = crate::usage_monitor::setup_status(root);
    data["resume_command"] = json!(result.next_action);
}

pub(super) fn complete(
    root: &Path,
    before: Option<&Value>,
    saved: &UserSetupConfig,
    result: &mut ActionResult,
) -> Result<(), SetupError> {
    let current = serde_json::to_value(saved).map_err(|e| SetupError::Internal(e.to_string()))?;
    let keys = super::catalog()?
        .questions
        .into_iter()
        .filter(|q| q.scope == PreferenceScope::User && q.applicable(&current))
        .map(|q| q.key)
        .collect::<std::collections::BTreeSet<_>>();
    let changed = before.map(|before| {
        keys.iter()
            .filter(|key| before.pointer(key) != current.pointer(key))
            .cloned()
            .collect::<Vec<_>>()
    });
    let monitor = crate::usage_monitor::setup_status(root);
    let policy = saved.usage_guard.policy.as_ref().map(|policy| json!({
        "scope":"user-defaults","schema_version":2,"enforcement_enabled":policy.enforcement_enabled,
        "warnings_enabled":policy.warnings_enabled,"stop_remaining_percent":saved.usage_guard.stop_remaining_percent,
        "preflight":policy.effective_preflight(),"midway":policy.effective_midway(),
        "session_overrides_evaluated":false,"project_overrides_evaluated":false,"authorizes_dispatch":false
    }));
    let pending = user_setup::migration::pending_questions(saved);
    if !pending.is_empty() {
        result.next_action = Some(crate::update_bridge::resume_command(root));
    }
    let data = result.data.get_or_insert_with(|| json!({}));
    // Projection inventories already have changed_paths/evidence. Keep the compact update
    // contract and recovery facts without copying profile text or every generated file.
    if let Some(data) = data.as_object_mut() {
        data.retain(|key, _| {
            matches!(
                key.as_str(),
                "operation_id"
                    | "recovered"
                    | "remaining_applied"
                    | "monitor_review"
                    | "settings_draft_preserved"
                    | "monitor_transition"
                    | "monitor_activation"
            )
        });
    }
    data["pending_questions"] = json!(pending);
    data["settings_schema"] = json!(saved.schema_version);
    data["interface_language"] = json!(saved.interface_language);
    data["selected_hosts"] = json!(saved.selected_hosts);
    data["stored_usage_policy"] = json!(saved.usage_guard);
    data["model_calls"] = json!(0);
    data["steps"] = json!({"settings":"verified","harness":"verified",
        "monitor":if monitor["controller_state"] == "unverified" {"unverified"} else {"inspected"}});
    data["settings_changes"] = json!({"comparison":"current-invocation", "changed_keys":changed,
        "before_verified":before.is_some(),"stored_digest":hive_core::preferences::declaration_digest(&current)
            .map_err(SetupError::Internal)?,
        "preserved": ["user-content","project-files"]});
    data["effective_usage_policy"] = json!(policy);
    data["monitor_state"] = monitor;
    data["resume_command"] = json!(result.next_action);
    data["to_version"] = json!(env!("CARGO_PKG_VERSION"));
    let path = root.to_string_lossy();
    let quoted = if cfg!(windows) {
        path.replace('\'', "''")
    } else {
        path.replace('\'', "'\"'\"'")
    };
    data["settings_command"] = json!(format!(
        "hive setup --scope user --user-root '{quoted}' --show"
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_policy_is_preserved_and_recovery_details_survive() {
        let root = tempfile::tempdir().unwrap();
        let answers = super::super::default_answers(Some(user_setup::InterfaceLanguage::En));
        let saved =
            user_setup::parse_and_validate_installed_config(&serde_json::to_vec(&answers).unwrap())
                .unwrap();
        let before = serde_json::to_value(&saved).unwrap();
        let mut result =
            super::super::interactive::pending(user_setup::InterfaceLanguage::En, "test");
        result.data = Some(json!({"recovered":true,"monitor_review":{"withdrawn":true}}));
        complete(root.path(), Some(&before), &saved, &mut result).unwrap();
        let data = result.data.unwrap();
        assert_eq!(data["settings_changes"]["changed_keys"], json!([]));
        assert_eq!(data["recovered"], true);
        assert_eq!(data["monitor_review"]["withdrawn"], true);
        assert_eq!(
            data["effective_usage_policy"]["preflight"]["zero_quota_enabled"],
            false
        );
        assert_eq!(data["monitor_state"]["controller_state"], "not-configured");
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn incomplete_application_never_claims_a_verified_policy_or_harness() {
        let root = tempfile::tempdir().unwrap();
        let mut result =
            super::super::interactive::pending(user_setup::InterfaceLanguage::Ko, "test");
        incomplete(root.path(), &mut result);
        let data = result.data.unwrap();
        assert!(data["effective_usage_policy"].is_null());
        assert_eq!(data["steps"]["harness"], "unverified");
        assert_eq!(data["monitor_state"]["sensor_queries"], 0);
        assert!(!root.path().join(".hive").exists());
    }
}
