//! A read-only v1 interpretation. Constructing a proposal never grants consent or writes it.
use super::{SetupError, UserSetupConfig};
use crate::usage_monitor::SavedMonitorChoice;
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope, UsageProtection,
};
use hive_core::sha256_digest;
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Serialize)]
pub(super) struct Preview {
    pub migration_id: String,
    pub source_digest: String,
    pub monitor_choice: SavedMonitorChoice,
    pub candidate: UserSetupConfig,
    pub pending_questions: Vec<&'static str>,
}

pub(super) fn inspect(
    root: &Path,
    config: &UserSetupConfig,
) -> Result<Option<Preview>, SetupError> {
    if config.schema_version == 2 {
        return Ok(None);
    }
    let choice = crate::usage_monitor::inspect_saved_choice(root)
        .map_err(|failure| SetupError::Verification(failure.message().to_owned()))?;
    prepare(config, choice).map(Some)
}

pub(super) fn prepare(
    config: &UserSetupConfig,
    monitor_choice: SavedMonitorChoice,
) -> Result<Preview, SetupError> {
    let source =
        serde_json::to_value(config).map_err(|error| SetupError::Internal(error.to_string()))?;
    let source_digest =
        hive_core::preferences::declaration_digest(&source).map_err(SetupError::Internal)?;
    let mut candidate = config.clone();
    candidate.schema_version = 2;
    candidate.usage_guard.policy = Some(UsageProtection::from_legacy(
        config.usage_guard.enabled,
        config.usage_guard.quota_reset_guard_enabled,
        monitor_choice.enabled,
    ));
    // The predecessor showed a popup for its opted-in monitor. Preserve that effect only.
    candidate.usage_guard.popup_enabled = Some(monitor_choice.enabled);
    let mut value = serde_json::to_value(&candidate)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    value
        .as_object_mut()
        .expect("typed settings object")
        .remove("preference_decisions");
    value
        .as_object_mut()
        .expect("typed settings object")
        .remove("schema_version");
    candidate.preference_decisions.retain(|key, decision| {
        value
            .pointer(key)
            .is_some_and(|declaration| decision.matches_declaration(declaration).unwrap_or(false))
    });
    if let Some(choice) = config.preference_decisions.get("/usage_guard/enabled") {
        candidate.preference_decisions.insert(
            "/usage_guard/preflight/threshold_enabled".to_owned(),
            choice.clone(),
        );
    }
    let mut declarations = BTreeMap::new();
    collect_declarations("", &value, &mut declarations);
    for (key, declaration) in declarations {
        if candidate.preference_decisions.contains_key(&key) {
            continue;
        }
        let mut decision = PreferenceDecision::new(
            &declaration,
            if key == "/usage_guard/warnings_enabled" {
                DecisionOrigin::Default
            } else {
                DecisionOrigin::LegacyUnknown
            },
            PreferenceInput::Migration,
            PreferenceScope::User,
            env!("CARGO_PKG_VERSION"),
            None,
        )
        .map_err(SetupError::Internal)?;
        if decision.origin == DecisionOrigin::LegacyUnknown {
            "legacy-unknown".clone_into(&mut decision.policy_version);
        }
        candidate.preference_decisions.insert(key, decision);
    }
    // Reuse both schema and semantic validation before a proposal can reach any apply path.
    let bytes =
        serde_json::to_vec(&candidate).map_err(|error| SetupError::Internal(error.to_string()))?;
    let candidate = super::parse_and_validate_installed_config(&bytes)?;
    let migration_id = sha256_digest(
        serde_json::to_vec(&json!([
            "usage-policy-v2",
            env!("CARGO_PKG_VERSION"),
            source_digest,
            monitor_choice
        ]))
        .map_err(|error| SetupError::Internal(error.to_string()))?
        .as_slice(),
    );
    Ok(Preview {
        migration_id,
        source_digest,
        monitor_choice,
        candidate,
        pending_questions: vec!["legacy-usage-choice", "usage-warnings"],
    })
}

fn collect_declarations(pointer: &str, value: &Value, declarations: &mut BTreeMap<String, Value>) {
    if let Some(object) = value.as_object().filter(|object| !object.is_empty()) {
        for (key, nested) in object {
            collect_declarations(
                &format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                nested,
                declarations,
            );
        }
    } else {
        declarations.insert(pointer.to_owned(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy(threshold: bool, reset: bool) -> UserSetupConfig {
        super::super::parse_and_validate_installed_config(&serde_json::to_vec(&json!({
            "schema_version":1,"interface_language":"en","wiki":{"language":"both"},
            "profile":{"contexts":["web-developer"]},"persona":{"id":"balanced"},"selected_hosts":["codex"],
            "skills":{"mode":"all"},"usage_guard":{"enabled":threshold,"quota_reset_guard_enabled":reset,
            "stop_remaining_percent":5,"project_overrides":{"kept-project":40}}
        })).unwrap()).unwrap()
    }

    fn choice(enabled: bool) -> SavedMonitorChoice {
        SavedMonitorChoice {
            enabled,
            configured: true,
            config_digest: sha256_digest(b"saved choice"),
        }
    }

    #[test]
    fn migration_preserves_every_legacy_reason_and_never_invents_an_answer_time() {
        for bits in 0..8 {
            let (threshold, reset, monitor) = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0);
            let old = legacy(threshold, reset);
            let preview = prepare(&old, choice(monitor)).unwrap();
            let next = preview.candidate;
            let policy = next.usage_guard.policy.as_ref().unwrap();
            assert!(policy.enforcement_enabled);
            assert!(!policy.warnings_enabled);
            assert_eq!(policy.preflight.threshold_enabled, threshold);
            assert_eq!(policy.preflight.reset_enabled, threshold && reset);
            assert!(policy.preflight.zero_quota_enabled);
            assert_eq!(
                policy.preflight.observation_failure,
                hive_core::preferences::PreflightFailure::Block
            );
            assert_eq!(policy.midway.enabled, monitor);
            assert_eq!(policy.midway.threshold_enabled, monitor && threshold);
            assert_eq!(policy.midway.reset_enabled, monitor);
            assert_eq!(policy.midway.zero_quota_enabled, monitor);
            assert_eq!(next.usage_guard.stop_remaining_percent, 5);
            assert_eq!(
                next.usage_guard.project_overrides,
                old.usage_guard.project_overrides
            );
            assert_eq!(next.profile, old.profile);
            assert_eq!(next.wiki, old.wiki);
            assert!(next
                .preference_decisions
                .values()
                .all(|record| record.decided_at.is_none() && record.question_id.is_none()));
            assert_eq!(
                next.preference_decisions["/usage_guard/preflight/zero_quota_enabled"].origin,
                DecisionOrigin::LegacyUnknown
            );
            assert_eq!(preview.pending_questions.len(), 2);
        }
    }

    #[test]
    fn existing_explicit_choices_and_same_migration_identity_are_preserved() {
        let mut old = legacy(true, false);
        old.preference_decisions.insert(
            "/usage_guard/enabled".into(),
            PreferenceDecision::new(
                &json!(true),
                DecisionOrigin::Explicit,
                PreferenceInput::AnswersFile,
                PreferenceScope::User,
                "0.13.1",
                Some(100),
            )
            .unwrap(),
        );
        let first = prepare(&old, choice(false)).unwrap();
        let second = prepare(&old, choice(false)).unwrap();
        assert_eq!(first.migration_id, second.migration_id);
        assert_eq!(first.candidate, second.candidate);
        let mapped =
            &first.candidate.preference_decisions["/usage_guard/preflight/threshold_enabled"];
        assert_eq!(mapped.origin, DecisionOrigin::Explicit);
        assert_eq!(mapped.decided_at, Some(100));
        assert!(!first
            .candidate
            .preference_decisions
            .contains_key("/usage_guard/enabled"));
        assert!(inspect(Path::new("unused"), &first.candidate)
            .unwrap()
            .is_none());
    }

    #[test]
    fn absent_monitor_is_safe_but_damaged_choice_defers_migration_without_writes() {
        let root = tempfile::tempdir().unwrap();
        let old = legacy(false, false);
        let inspected = inspect(root.path(), &old).unwrap().unwrap();
        assert!(!inspected.monitor_choice.enabled);
        assert!(!root.path().join(".hive").exists());
        let path = root.path().join(".hive/runtime/usage-monitor/config.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"damaged").unwrap();
        assert!(inspect(root.path(), &old).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"damaged");
        assert!(!root.path().join(".hive/config").exists());
    }
}
