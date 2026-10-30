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
    expected: Option<&[u8]>,
    legacy_input: Option<&UserSetupConfig>,
    migration: Option<user_setup::progress_store::LegacyMigration>,
    options: &super::file_input::FileOptions,
) -> Result<ActionResult, SetupError> {
    let mut config = config.clone();
    let index_rebuild_scope = options.index_rebuild_scope.as_deref();
    let store = ProgressStore::open(&arguments.user_root, None)?;
    store.recover_pending_state()?;
    let snapshot = store.load()?;
    if let Some(pending) = snapshot
        .progress
        .as_ref()
        .filter(|p| p.phase != ProgressPhase::Complete)
    {
        require_same_intent(pending, &config, legacy_input)?;
        super::monitor_start::check_file_replay(pending, options.monitor_confirm.as_deref())?;
        if index_rebuild_scope.is_some_and(|digest| {
            pending
                .index_rebuild_scope
                .as_deref()
                .is_some_and(|saved| saved != digest)
        }) {
            return Err(SetupError::Conflict(
                "index rebuild approval differs from the pending setup".into(),
            ));
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
        preserve_terminal_answers(pending, &mut config)?;
    }
    let bytes =
        serde_json::to_vec(&config).map_err(|error| SetupError::Internal(error.to_string()))?;
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
    let preview =
        user_setup::execute_answers(&preview, &bytes, user_setup::SetupBase::Expected(expected))?;
    let answers: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| SetupError::Internal(error.to_string()))?;
    let mut progress = match snapshot
        .progress
        .as_ref()
        .filter(|p| p.phase != ProgressPhase::Complete)
    {
        Some(pending) => pending.clone(),
        None => store.new_progress("file-apply".into(), answers.clone())?,
    };
    progress.answers = answers;
    super::host_scope::bind_deactivations(&preview, &mut progress)?;
    progress.legacy_migration = migration;
    super::monitor_stop::bind(&arguments.user_root, &mut progress)?;
    super::monitor_start::bind_file(
        &arguments.user_root,
        expected,
        &mut progress,
        options.monitor_confirm.as_deref(),
        options.channel,
    )?;
    super::index_rebuild::bind_file(arguments, &mut progress, index_rebuild_scope)?;
    super::monitor_start::preflight(&arguments.user_root, &progress)?;
    super::application::begin(&store, &snapshot, &mut progress)?;
    super::application::finish(
        &store,
        &progress,
        super::legacy::apply(arguments, &bytes, expected, &progress),
    )
}

fn preserve_terminal_answers(
    progress: &SetupProgress,
    config: &mut UserSetupConfig,
) -> Result<(), SetupError> {
    use hive_core::preferences::{DecisionOrigin, PreferenceDecisions, PreferenceInput};
    let Some(saved) = progress.answers.get("preference_decisions") else {
        return Ok(());
    };
    let records: PreferenceDecisions = serde_json::from_value(saved.clone())
        .map_err(|error| SetupError::Verification(error.to_string()))?;
    let catalog = super::catalog()?;
    for (path, record) in records {
        if record.input_channel == PreferenceInput::Terminal
            && matches!(
                record.origin,
                DecisionOrigin::Explicit | DecisionOrigin::AcceptedDefault
            )
            && catalog.questions.iter().any(|question| {
                question.key == path
                    && record.question_id.as_deref() == Some(question.id.as_str())
                    && record.question_revision == Some(question.revision)
            })
            && config
                .preference_decisions
                .get(&path)
                .is_some_and(|current| current.value_digest == record.value_digest)
        {
            config.preference_decisions.insert(path, record);
        }
    }
    Ok(())
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
    fn same_draft_keeps_current_terminal_choices_but_not_stale_question_evidence() {
        let root = crate::test_tempdir();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut answers = super::super::default_answers(Some(user_setup::InterfaceLanguage::En));
        let mut original = PreferenceDecision::new(
            &json!("en"),
            DecisionOrigin::Explicit,
            PreferenceInput::Terminal,
            PreferenceScope::User,
            env!("CARGO_PKG_VERSION"),
            Some(42),
        )
        .unwrap();
        original.question_id = Some("user-language".into());
        original.question_revision = Some(1);
        answers["preference_decisions"]["/interface_language"] = json!(original);
        let mut progress = store
            .new_progress("usage-enforcement".into(), answers)
            .unwrap();
        let mut candidate = user_setup::parse_and_validate_config(
            &serde_json::to_vec(&super::super::default_answers(Some(
                user_setup::InterfaceLanguage::En,
            )))
            .unwrap(),
        )
        .unwrap();
        let agent = PreferenceDecision::new(
            &json!("en"),
            DecisionOrigin::ExternalChange,
            PreferenceInput::Agent,
            PreferenceScope::User,
            env!("CARGO_PKG_VERSION"),
            None,
        )
        .unwrap();
        candidate
            .preference_decisions
            .insert("/interface_language".into(), agent.clone());
        preserve_terminal_answers(&progress, &mut candidate).unwrap();
        assert_eq!(
            candidate.preference_decisions["/interface_language"],
            original
        );
        candidate
            .preference_decisions
            .insert("/interface_language".into(), agent.clone());
        progress.answers["preference_decisions"]["/interface_language"]["question_revision"] =
            json!(2);
        preserve_terminal_answers(&progress, &mut candidate).unwrap();
        assert_eq!(candidate.preference_decisions["/interface_language"], agent);
    }

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
