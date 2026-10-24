//! Explicit project documents use the terminal's preserving preparation and apply journal.
use super::{
    commit_prepared, existing, finish_existing_apply, global_preferences, json, preferences,
    prepare_settings_raw, questions, request, user_setup, write_answers, InteractionRequest,
    InterfaceLanguage, Options, ProgressPhase, ProgressStore, SetupError, Value,
};
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceDecisions, PreferenceInput, PreferenceScope,
};
use hive_render::{RenderError, SetupMode, SetupOutcome};
use std::{io::Write, path::Path};

fn as_render(error: SetupError) -> RenderError {
    match error {
        SetupError::Input(message) => RenderError::Input(message),
        SetupError::Conflict(message) => RenderError::Conflict(message),
        SetupError::Verification(message) => RenderError::Verification(message),
        SetupError::Internal(message) => RenderError::Internal(message),
    }
}

fn meaningful(value: &Value) -> Value {
    let mut value = value.clone();
    if let Some(object) = value.as_object_mut() {
        object.remove("preference_decisions");
        object.remove("schema_version");
    }
    value
}

// Keep provenance replacement and preservation rules in one ordered pass.
#[allow(clippy::too_many_lines)]
fn normalize(
    input: &[u8],
    stored: Option<&[u8]>,
    channel: PreferenceInput,
    answered_at: Option<u64>,
) -> Result<Value, RenderError> {
    let supplied: Value =
        serde_yaml::from_slice(input).map_err(|e| RenderError::Input(e.to_string()))?;
    let mut answers = hive_render::normalized_project_answers(input)?;
    let previous = stored
        .map(existing::answers)
        .transpose()
        .map_err(as_render)?;
    if supplied
        .get("project_identity")
        .is_none_or(|value| value.as_str() == Some(""))
    {
        if let Some(identity) = previous.as_ref().and_then(|v| v.get("project_identity")) {
            answers["project_identity"] = identity.clone();
        }
    }
    answers["schema_version"] = json!(2);
    // Incoming provenance is data, never evidence that a human answered a question.
    answers["preference_decisions"] = json!({});
    preferences::record_defaults(&mut answers).map_err(as_render)?;
    let mut records: PreferenceDecisions =
        serde_json::from_value(answers["preference_decisions"].clone())
            .map_err(|e| RenderError::Internal(e.to_string()))?;
    let old: PreferenceDecisions = previous
        .as_ref()
        .map(|p| serde_json::from_value(p["preference_decisions"].clone()))
        .transpose()
        .map_err(|e| RenderError::Input(e.to_string()))?
        .unwrap_or_default();
    for (pointer, inherited) in records
        .iter_mut()
        .filter(|(_, record)| record.origin == DecisionOrigin::Inherited)
    {
        if let Some(prior) = old.get(pointer).filter(|record| {
            record.origin == DecisionOrigin::Inherited
                && record.value_digest == inherited.value_digest
                && record.inherited_from == inherited.inherited_from
        }) {
            *inherited = prior.clone();
        } else {
            inherited.input_channel = channel;
            inherited.decided_at = answered_at.filter(|_| supplied.get("setup_mode").is_some());
        }
    }
    let entries = answers
        .as_object()
        .expect("validated project object")
        .clone();
    for (key, value) in entries {
        if matches!(
            key.as_str(),
            "schema_version" | "preference_decisions" | "user_store_binding"
        ) {
            continue;
        }
        let pointer = format!("/{key}");
        let prefix = format!("{pointer}/");
        if records
            .get(&pointer)
            .is_some_and(|record| record.origin == DecisionOrigin::Inherited)
        {
            continue;
        }
        let unchanged = previous.as_ref().and_then(|p| p.get(&key)) == Some(&value);
        let trusted = old
            .iter()
            .filter(|(path, record)| {
                unchanged
                    && (*path == &pointer || path.starts_with(&prefix))
                    && !matches!(
                        record.origin,
                        DecisionOrigin::LegacyUnknown
                            | DecisionOrigin::ExternalChange
                            | DecisionOrigin::Inherited
                    )
                    && (record.origin != DecisionOrigin::Default || supplied.get(&key).is_none())
            })
            .collect::<Vec<_>>();
        records.retain(|path, _| path != &pointer && !path.starts_with(&prefix));
        if trusted.is_empty() {
            let explicit = supplied
                .get(&key)
                .is_some_and(|value| key != "project_identity" || value.as_str() != Some(""));
            records.insert(
                pointer,
                PreferenceDecision::new(
                    &value,
                    if explicit {
                        DecisionOrigin::Explicit
                    } else {
                        DecisionOrigin::Default
                    },
                    channel,
                    PreferenceScope::Project,
                    env!("CARGO_PKG_VERSION"),
                    answered_at.filter(|_| explicit),
                )
                .map_err(RenderError::Input)?,
            );
        } else {
            for (path, record) in trusted {
                records.insert(path.clone(), record.clone());
            }
        }
    }
    answers["preference_decisions"] =
        serde_json::to_value(records).map_err(|e| RenderError::Internal(e.to_string()))?;
    // Check the metadata we produced using the same renderer contract.
    hive_render::normalized_project_answers(
        &serde_json::to_vec(&answers).map_err(|e| RenderError::Internal(e.to_string()))?,
    )
}

fn file_scope(
    arguments: &crate::SetupArguments,
) -> Result<(Options, hive_render::GlobalProjectPreferences), RenderError> {
    hive_core::ensure_consumer_target(&arguments.target)
        .map_err(|e| RenderError::Safety(e.to_string()))?;
    let target = crate::user_install::open_canonical_user_root_for_setup(&arguments.target)
        .map_err(RenderError::Conflict)?
        .0;
    let root = crate::user_install::open_canonical_user_root_for_setup(&arguments.user_root)
        .map_err(RenderError::Conflict)?
        .0;
    let mut global = user_setup::project_preferences(&root).map_err(RenderError::Input)?;
    global.codex_plugin_files =
        crate::user_install::validated_codex_plugin_files(&root).unwrap_or_default();
    let options = Options {
        target,
        root,
        interaction: InteractionRequest {
            json: true,
            non_interactive: true,
            ..InteractionRequest::default()
        },
        resume: false,
        show: false,
        dry_run: arguments.mode == SetupMode::DryRun,
        describe: false,
        section: None,
        capabilities: Some(arguments.capabilities.clone()),
        reconfigure_roles: arguments.reconfigure_roles.clone(),
    };
    Ok((options, global))
}

fn validate_file(
    options: &Options,
    mut answers: Value,
    saved: Option<&[u8]>,
    caps: &Path,
    global: &hive_render::GlobalProjectPreferences,
) -> Result<SetupOutcome, RenderError> {
    if let Some(stored) = saved {
        let current = hive_render::normalized_project_answers(stored)?;
        answers["schema_version"] = current["schema_version"].clone();
        answers["preference_decisions"] = current
            .get("preference_decisions")
            .cloned()
            .unwrap_or_else(|| json!({}));
    }
    let input = write_answers(&answers).map_err(as_render)?;
    crate::execute_setup_and_registry(
        &request(options, input.path(), caps, global, SetupMode::Validate),
        &options.root,
        &options.target,
        global.wiki_enabled,
    )
}

pub(in crate::user_setup) fn execute(
    arguments: &crate::SetupArguments,
) -> Result<SetupOutcome, RenderError> {
    let (mut options, global) = file_scope(arguments)?;
    let input =
        user_setup::read_bounded_regular(&arguments.answers, user_setup::MAX_USER_SETUP_BYTES)
            .map_err(as_render)?;
    let cap_bytes =
        user_setup::read_bounded_regular(&arguments.capabilities, user_setup::MAX_USER_SETUP_BYTES)
            .map_err(as_render)?;
    hive_core::run::CapabilityResolution::parse_json(&cap_bytes)
        .map_err(|e| RenderError::Input(e.to_string()))?;
    let output = crate::user_install::open_user_root_for_setup(&options.target)
        .map_err(RenderError::Conflict)?;
    let saved = existing::read(&output).map_err(as_render)?;
    let store = ProgressStore::open(&options.root, Some(&options.target)).map_err(as_render)?;
    if arguments.mode == SetupMode::Apply {
        store.recover_pending_state().map_err(as_render)?;
    }
    let mut snapshot = store.load().map_err(as_render)?;
    let pending = snapshot
        .progress
        .as_ref()
        .filter(|p| p.phase != ProgressPhase::Complete);
    let baseline = pending
        .map(|p| serde_json::to_vec(&p.answers))
        .transpose()
        .map_err(|e| RenderError::Internal(e.to_string()))?;
    let mut answers = normalize(
        &input,
        baseline.as_deref().or(saved.as_deref()),
        arguments.input_channel,
        (arguments.mode == SetupMode::Apply)
            .then(user_setup::feature_now_unix)
            .transpose()
            .map_err(as_render)?,
    )?;
    let mut caps =
        tempfile::NamedTempFile::new().map_err(|e| RenderError::Internal(e.to_string()))?;
    caps.write_all(&cap_bytes)
        .map_err(|e| RenderError::Internal(e.to_string()))?;
    if arguments.mode == SetupMode::Validate {
        return validate_file(&options, answers, saved.as_deref(), caps.path(), &global);
    }
    if let Some(progress) = pending.filter(|_| arguments.mode == SetupMode::Apply) {
        if meaningful(&answers) != meaningful(&progress.answers)
            || (!options.reconfigure_roles.is_empty()
                && options.reconfigure_roles != progress.reconfigure_roles)
        {
            return Err(RenderError::Conflict(
                "project file differs from unfinished setup; resume the original intent first"
                    .into(),
            ));
        }
        if let Some(context) = &progress.project_context {
            let current: Value = serde_json::from_slice(&cap_bytes)
                .map_err(|e| RenderError::Input(e.to_string()))?;
            if context.capabilities != current {
                return Err(RenderError::Conflict(
                    "capability evidence differs from unfinished setup".into(),
                ));
            }
        }
        answers.clone_from(&progress.answers);
        options
            .reconfigure_roles
            .clone_from(&progress.reconfigure_roles);
        if matches!(
            progress.phase,
            ProgressPhase::Applying | ProgressPhase::RecoverableFailure
        ) {
            finish_existing_apply(
                &options,
                &store,
                progress,
                &global,
                questions::language(&answers, InterfaceLanguage::En),
            )
            .map_err(as_render)?;
            return validate_file(&options, answers, None, caps.path(), &global);
        }
    }
    apply_file(
        arguments,
        &options,
        &global,
        &store,
        &mut snapshot,
        answers,
        caps.path(),
        &cap_bytes,
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_file(
    arguments: &crate::SetupArguments,
    options: &Options,
    global: &hive_render::GlobalProjectPreferences,
    store: &ProgressStore,
    snapshot: &mut user_setup::progress_store::ProgressSnapshot,
    answers: Value,
    caps: &Path,
    cap_bytes: &[u8],
) -> Result<SetupOutcome, RenderError> {
    let pending = snapshot
        .progress
        .as_ref()
        .filter(|p| p.phase != ProgressPhase::Complete);
    let output = crate::user_install::open_user_root_for_setup(&options.target)
        .map_err(RenderError::Conflict)?;
    let saved = existing::read(&output).map_err(as_render)?;
    let input = write_answers(&answers).map_err(as_render)?;
    if pending.is_none()
        && saved.as_deref().is_some_and(|bytes| {
            hive_render::normalized_project_answers(bytes).is_ok_and(|current| current == answers)
        })
    {
        let validation = request(options, input.path(), caps, global, SetupMode::Validate);
        if let Ok(current) = crate::execute_setup_and_registry(
            &validation,
            &options.root,
            &options.target,
            global.wiki_enabled,
        ) {
            if global_preferences(&options.root).map_err(as_render)? != *global {
                return Err(RenderError::Conflict(
                    "global settings changed during project validation".into(),
                ));
            }
            return Ok(current);
        }
    }
    let prepared = prepare_settings_raw(options, input.path(), caps, global)?;
    if arguments.mode == SetupMode::DryRun {
        return Ok(prepared.outcome);
    }
    let user = crate::user_install::open_user_root_for_setup(&options.root)
        .map_err(RenderError::Conflict)?;
    let _lease = crate::user_install::lock_user_setup(&user).map_err(RenderError::Conflict)?;
    if global_preferences(&options.root).map_err(as_render)? != *global {
        return Err(RenderError::Conflict(
            "global project settings changed before file apply".into(),
        ));
    }
    let mut progress = match pending {
        Some(progress) => progress.clone(),
        None => store
            .new_progress("file-apply".into(), answers.clone())
            .map_err(as_render)?,
    };
    progress.answers = answers;
    if pending.is_none() {
        progress
            .reconfigure_roles
            .clone_from(&arguments.reconfigure_roles);
    }
    let mut outcome = prepared.outcome.clone();
    outcome.changed_paths = commit_prepared(
        options,
        store,
        snapshot,
        &mut progress,
        global,
        cap_bytes,
        prepared,
    )
    .map_err(as_render)?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn input() -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/setup/answers-no-role-no-hook.yml"),
        )
        .unwrap()
    }

    #[test]
    fn file_values_inheritance_and_automatic_defaults_have_distinct_sources() {
        let values = normalize(&input(), None, PreferenceInput::Agent, Some(123)).unwrap();
        assert_eq!(values["schema_version"], 2);
        let records = &values["preference_decisions"];
        assert_eq!(records["/project_name"]["origin"], "explicit");
        assert_eq!(records["/project_name"]["input_channel"], "agent");
        assert_eq!(records["/project_name"]["decided_at"], 123);
        assert_eq!(records["/project_identity"]["origin"], "default");
        assert!(records["/project_identity"]["decided_at"].is_null());
        assert_eq!(records["/interface_language"]["origin"], "inherited");
        assert_eq!(records["/interface_language"]["input_channel"], "agent");
    }

    #[test]
    fn reapplying_the_same_file_keeps_recorded_times_and_existing_identity() {
        let first = normalize(&input(), None, PreferenceInput::AnswersFile, Some(123)).unwrap();
        let stored = serde_json::to_vec(&first).unwrap();
        let second = normalize(&input(), Some(&stored), PreferenceInput::Agent, Some(456)).unwrap();
        assert_eq!(first, second);
        let mut changed: Value = serde_yaml::from_slice(&input()).unwrap();
        changed["project_name"] = json!("renamed project");
        let changed = normalize(
            &serde_json::to_vec(&changed).unwrap(),
            Some(&stored),
            PreferenceInput::Agent,
            Some(456),
        )
        .unwrap();
        assert_eq!(changed["project_identity"], first["project_identity"]);
        assert_eq!(
            changed["preference_decisions"]["/project_name"]["decided_at"],
            456
        );
    }

    #[test]
    fn incoming_metadata_cannot_claim_that_an_agent_value_was_a_terminal_answer() {
        let mut supplied: Value = serde_yaml::from_slice(&input()).unwrap();
        let record = PreferenceDecision::new(
            &supplied["project_name"],
            DecisionOrigin::Explicit,
            PreferenceInput::Terminal,
            PreferenceScope::Project,
            env!("CARGO_PKG_VERSION"),
            Some(5),
        )
        .unwrap();
        supplied["schema_version"] = json!(2);
        supplied["preference_decisions"] = json!({"/project_name":record});
        let values = normalize(
            &serde_json::to_vec(&supplied).unwrap(),
            None,
            PreferenceInput::Agent,
            Some(123),
        )
        .unwrap();
        assert_eq!(
            values["preference_decisions"]["/project_name"]["input_channel"],
            "agent"
        );
        assert_eq!(
            values["preference_decisions"]["/project_name"]["decided_at"],
            123
        );
    }

    #[test]
    fn legacy_file_apply_uses_the_same_preserving_pipeline_and_validates_without_writes() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let mut global = super::super::super::default_answers(Some(InterfaceLanguage::Ko));
        global["selected_hosts"] = json!(["codex"]);
        let path = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec(&global).unwrap()).unwrap();
        let answers = root.path().join("input.yml");
        std::fs::write(&answers, input()).unwrap();
        let mut args = crate::SetupArguments {
            target: target.path().into(),
            user_root: root.path().into(),
            answers,
            capabilities: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/setup/capabilities-codex-host-native.json"),
            mode: SetupMode::DryRun,
            reconfigure_roles: BTreeSet::new(),
            input_channel: PreferenceInput::Agent,
        };
        execute(&args).unwrap();
        assert!(!target.path().join(".hive").exists());
        args.mode = SetupMode::Apply;
        execute(&args).unwrap();
        let saved = std::fs::read(target.path().join(".hive/setup-answers.yml")).unwrap();
        let settings: Value = serde_yaml::from_slice(&saved).unwrap();
        assert_eq!(settings["schema_version"], 2);
        assert_eq!(
            settings["preference_decisions"]["/project_name"]["input_channel"],
            "agent"
        );
        args.mode = SetupMode::Validate;
        execute(&args).unwrap();
        assert_eq!(
            std::fs::read(target.path().join(".hive/setup-answers.yml")).unwrap(),
            saved
        );
        args.mode = SetupMode::Apply;
        let repeated = execute(&args).unwrap();
        assert!(
            repeated.changed_paths.is_empty(),
            "repeated paths: {:?}",
            repeated.changed_paths
        );
        assert_eq!(
            std::fs::read(target.path().join(".hive/setup-answers.yml")).unwrap(),
            saved
        );
    }
}
