//! Preserve existing choices and distinguish missing historical provenance from new defaults.
use crate::user_setup::{self, SetupError};
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceDecisions, PreferenceInput, PreferenceScope,
};
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn answers(bytes: &[u8]) -> Result<Value, SetupError> {
    let original: Value = serde_yaml::from_slice(bytes)
        .map_err(|_| SetupError::Verification("invalid saved project answers".into()))?;
    let mut answers = hive_render::normalized_project_answers(bytes)
        .map_err(|error| SetupError::Verification(error.to_string()))?;
    let mut decisions: PreferenceDecisions = serde_json::from_value(
        answers
            .get("preference_decisions")
            .cloned()
            .unwrap_or_else(|| json!({})),
    )
    .map_err(|error| SetupError::Verification(error.to_string()))?;
    for (key, value) in answers.as_object().expect("project answer object") {
        if matches!(
            key.as_str(),
            "schema_version" | "preference_decisions" | "user_store_binding"
        ) {
            continue;
        }
        let pointer = format!("/{key}");
        if decisions
            .keys()
            .any(|key| key == &pointer || key.starts_with(&format!("{pointer}/")))
        {
            continue;
        }
        let origin = if original.get(key).is_some() {
            DecisionOrigin::LegacyUnknown
        } else {
            DecisionOrigin::Default
        };
        let mut record = PreferenceDecision::new(
            value,
            origin,
            PreferenceInput::Migration,
            PreferenceScope::Project,
            env!("CARGO_PKG_VERSION"),
            None,
        )
        .map_err(SetupError::Internal)?;
        if origin == DecisionOrigin::LegacyUnknown {
            "unknown".clone_into(&mut record.changed_in_version);
            "unknown".clone_into(&mut record.policy_version);
        }
        decisions.insert(pointer, record);
    }
    answers["schema_version"] = json!(2);
    answers["preference_decisions"] =
        serde_json::to_value(decisions).map_err(|error| SetupError::Internal(error.to_string()))?;
    Ok(answers)
}

pub(super) fn capabilities(target: &Path, requested: &Value) -> Result<Option<Value>, SetupError> {
    let root =
        crate::user_install::open_user_root_for_setup(target).map_err(SetupError::Conflict)?;
    let Some(bytes) = crate::user_install::read_user_setup_file(
        &root,
        Path::new(".hive/setup-answers.yml"),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    else {
        return Ok(None);
    };
    let saved = hive_render::normalized_project_answers(&bytes)
        .map_err(|error| SetupError::Verification(error.to_string()))?;
    if saved["primary_host"] != requested["primary_host"] {
        return Ok(None);
    }
    let bytes = crate::user_install::read_user_setup_file(
        &root,
        Path::new(".hive/config/capability-resolution.yml"),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    .ok_or_else(|| SetupError::Verification("saved project capability data is missing".into()))?;
    let value: Value = serde_yaml::from_slice(&bytes)
        .map_err(|_| SetupError::Verification("invalid saved project capabilities".into()))?;
    hive_core::run::CapabilityResolution::parse_json(
        &serde_json::to_vec(&value).map_err(|error| SetupError::Internal(error.to_string()))?,
    )
    .map_err(|error| SetupError::Verification(error.to_string()))?;
    Ok(Some(value))
}
