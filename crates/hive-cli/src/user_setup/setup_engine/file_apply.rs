//! Explicit file application shares the terminal's confirmed intent and recovery journal.
use crate::user_setup::{
    self,
    progress_store::{ProgressPhase, ProgressStore, SetupProgress},
    Arguments, SetupError, SetupMode, UserSetupConfig,
};
use crate::ActionResult;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
    arguments: &Arguments,
    config: &UserSetupConfig,
    bytes: &[u8],
    expected: Option<&[u8]>,
    legacy_input: Option<&UserSetupConfig>,
    migration: Option<user_setup::progress_store::LegacyMigration>,
    index_rebuild_scope: Option<&str>,
) -> Result<ActionResult, SetupError> {
    let store = ProgressStore::open(&arguments.user_root, None)?;
    store.recover_pending_state()?;
    let snapshot = store.load()?;
    if let Some(pending) = snapshot
        .progress
        .as_ref()
        .filter(|p| p.phase != ProgressPhase::Complete)
    {
        require_same_intent(pending, config, legacy_input)?;
        if index_rebuild_scope.is_some_and(|digest| pending.index_rebuild_scope.as_deref()
            .is_some_and(|saved| saved != digest)) {
            return Err(SetupError::Conflict("index rebuild approval differs from the pending setup".into()));
        }
        if matches!(
            pending.phase,
            ProgressPhase::Applying | ProgressPhase::RecoverableFailure
        ) {
            return super::recovery::resume(
                &arguments.user_root,
                pending,
                config.interface_language,
                false,
            );
        }
    }
    let _lease =
        crate::user_install::lock_user_setup(&arguments.root_cap).map_err(SetupError::Conflict)?;
    if !store.matches_config(expected) {
        return Err(SetupError::Conflict(
            "settings changed before file application".into(),
        ));
    }
    let preview = Arguments {
        answers: arguments.answers.clone(),
        mode: SetupMode::DryRun,
        user_root: arguments.user_root.clone(),
        root_cap: arguments
            .root_cap
            .try_clone()
            .map_err(|error| SetupError::Internal(error.to_string()))?,
    };
    user_setup::execute_answers(&preview, bytes, user_setup::SetupBase::Expected(expected))?;
    let answers: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| SetupError::Internal(error.to_string()))?;
    let mut progress = match snapshot
        .progress
        .as_ref()
        .filter(|p| p.phase != ProgressPhase::Complete)
    {
        Some(pending) => pending.clone(),
        None => store.new_progress("file-apply".into(), answers.clone())?,
    };
    progress.answers = answers;
    progress.legacy_migration = migration;
    super::index_rebuild::bind_file(arguments, &mut progress, index_rebuild_scope)?;
    progress.phase = ProgressPhase::Applying;
    store.save(&snapshot, &progress)?;
    store.record_applying(&progress)?;
    let applied = match super::legacy::apply(arguments, bytes, expected, &progress) {
        Ok(result) => result,
        Err(error) => {
            store.record_failure(&progress)?;
            return Err(error);
        }
    };
    ProgressStore::open(&arguments.user_root, None)?.record_completion(&progress)?;
    Ok(applied)
}

fn require_same_intent(
    progress: &SetupProgress,
    config: &UserSetupConfig,
    legacy_input: Option<&UserSetupConfig>,
) -> Result<(), SetupError> {
    let mut saved = user_setup::parse_and_validate_config(
        &serde_json::to_vec(&progress.answers)
            .map_err(|error| SetupError::Internal(error.to_string()))?,
    )?;
    let mut requested = if saved.schema_version == 1 {
        legacy_input.unwrap_or(config)
    } else {
        config
    }
    .clone();
    // A retry's newly computed timestamps are not a new choice. Recovery uses the saved records.
    saved.preference_decisions.clear();
    requested.preference_decisions.clear();
    if saved != requested {
        return Err(SetupError::Conflict(
            "answer file differs from the confirmed interrupted setup; resume the saved operation first".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::preferences::{
        DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope,
    };
    use serde_json::json;

    #[test]
    fn file_apply_accepts_the_same_saved_draft_and_rejects_different_choices() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let answers = super::super::default_answers(Some(user_setup::InterfaceLanguage::En));
        let mut progress = store
            .new_progress("file-apply".into(), answers.clone())
            .unwrap();
        let config =
            user_setup::parse_and_validate_config(&serde_json::to_vec(&answers).unwrap()).unwrap();
        assert!(require_same_intent(&progress, &config, None).is_ok());
        progress.phase = ProgressPhase::Applying;
        let record = PreferenceDecision::new(
            &json!(false),
            DecisionOrigin::Explicit,
            PreferenceInput::AnswersFile,
            PreferenceScope::User,
            "0.14.0",
            Some(42),
        )
        .unwrap();
        progress.answers["preference_decisions"]["/usage_guard/enforcement_enabled"] =
            json!(record);
        assert!(require_same_intent(&progress, &config, None).is_ok());
        let mut changed = config;
        changed.usage_guard.stop_remaining_percent = 3;
        assert!(require_same_intent(&progress, &changed, None).is_err());
        assert!(!root.path().join(".hive").exists());
    }
}
