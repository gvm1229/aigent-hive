//! Explicit review of legacy effects using the shared question and progress contracts.
use super::{
    catalog,
    interactive::{persist, set_value, Conversation},
    terminal::Terminal,
};
use crate::user_setup::{
    self,
    progress_store::{LegacyMigration, ProgressStore, SetupProgress},
    SetupError, UserSetupConfig,
};
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope, UsageProtection,
};
use hive_core::setup::{AnswerInput, Question};
use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    path::Path,
};

pub(super) fn initial_progress(
    root: &Path,
    store: &ProgressStore,
    config: Option<&UserSetupConfig>,
) -> Result<Option<SetupProgress>, SetupError> {
    let Some(config) =
        config.filter(|config| !user_setup::migration::pending_questions(config).is_empty())
    else {
        return Ok(None);
    };
    transition_progress(root, store, config).map(Some)
}

pub(super) fn transition_progress(
    root: &Path,
    store: &ProgressStore,
    config: &UserSetupConfig,
) -> Result<SetupProgress, SetupError> {
    let preview = user_setup::migration::review(root, config)?;
    let mut record = store.new_progress(
        "usage-enforcement".into(),
        serde_json::to_value(&preview.candidate)
            .map_err(|error| SetupError::Internal(error.to_string()))?,
    )?;
    record.legacy_migration = Some(LegacyMigration {
        migration_id: preview.migration_id,
        monitor_choice: preview.monitor_choice,
        from_schema: config.schema_version,
        questions: preview
            .pending_questions
            .iter()
            .map(|id| (*id).into())
            .collect(),
    });
    record.setup_mode = Some("custom".into());
    Ok(record)
}

pub(super) fn collect<R: BufRead, W: Write>(
    root: &Path,
    store: &ProgressStore,
    progress: &mut Conversation,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    let binding = progress
        .legacy_migration
        .as_ref()
        .expect("legacy collection");
    let current = crate::usage_monitor::inspect_saved_choice(root)
        .map_err(|error| SetupError::Conflict(error.message().into()))?;
    if current != binding.monitor_choice {
        return Err(SetupError::Conflict(
            "monitor choice changed; review migration again".into(),
        ));
    }
    let questions = catalog()?;
    let guard = questions
        .questions
        .iter()
        .find(|q| q.id == "usage-enforcement")
        .expect("guard question");
    let warnings = questions
        .questions
        .iter()
        .find(|q| q.id == "usage-warnings")
        .expect("notice question");
    if progress
        .legacy_migration
        .as_ref()
        .expect("review")
        .questions
        .contains(&guard.id)
        && !progress.answered_questions.contains_key(&guard.id)
    {
        show_effects(progress, terminal)?;
        terminal.say(
            "현재 보호를 유지하려면 예, 모든 사용량 차단·중단과 상시 감시를 끄려면 아니요를 선택해 주세요. 권장: 아니요. 잔여량 기준과 지난 기록은 보존합니다.",
            "Choose yes to keep the current protection, or no to disable all usage blocking, interruption, and background monitoring. Recommended: no. Thresholds and past records are preserved.",
        )?;
        if !ask_choice(store, progress, guard, terminal, true)? {
            return Ok(false);
        }
    }
    if progress
        .legacy_migration
        .as_ref()
        .expect("review")
        .questions
        .contains(&warnings.id)
        && !progress.answered_questions.contains_key(&warnings.id)
        && !ask_choice(store, progress, warnings, terminal, false)?
    {
        return Ok(false);
    }
    Ok(true)
}

fn ask_choice<R: BufRead, W: Write>(
    store: &ProgressStore,
    progress: &mut Conversation,
    question: &Question,
    terminal: &mut Terminal<R, W>,
    protection: bool,
) -> Result<bool, SetupError> {
    progress.step.clone_from(&question.id);
    persist(store, progress)?;
    let AnswerInput::Answered { value, origin } = terminal.ask(question, None)? else {
        return Ok(false);
    };
    let enabled = value
        .as_bool()
        .ok_or_else(|| SetupError::Input("expected yes or no".into()))?;
    let mut changes = vec![(question.key.clone(), value)];
    if protection && !enabled {
        let off = serde_json::to_value(UsageProtection::default()).expect("typed policy");
        for key in [
            "/preflight/threshold_enabled",
            "/preflight/reset_enabled",
            "/preflight/zero_quota_enabled",
            "/preflight/observation_failure",
            "/midway/enabled",
            "/midway/threshold_enabled",
            "/midway/reset_enabled",
            "/midway/zero_quota_enabled",
            "/midway/observation_failure",
        ] {
            changes.push((
                format!("/usage_guard{key}"),
                off.pointer(key).expect("policy field").clone(),
            ));
        }
    }
    for (key, value) in changes {
        record_choice(progress, question, &key, value, origin)?;
    }
    progress
        .answered_questions
        .insert(question.id.clone(), question.revision);
    persist(store, progress)?;
    Ok(true)
}

fn record_choice(
    progress: &mut Conversation,
    question: &Question,
    key: &str,
    value: Value,
    origin: DecisionOrigin,
) -> Result<(), SetupError> {
    let mut choice = PreferenceDecision::new(
        &value,
        origin,
        PreferenceInput::Terminal,
        PreferenceScope::User,
        env!("CARGO_PKG_VERSION"),
        Some(user_setup::feature_now_unix()?),
    )
    .map_err(SetupError::Input)?;
    choice.question_id = Some(question.id.clone());
    choice.question_revision = Some(question.revision);
    set_value(&mut progress.answers, key, value)?;
    progress.answers["preference_decisions"][key] = json!(choice);
    Ok(())
}

fn show_effects<R: BufRead, W: Write>(
    progress: &Conversation,
    terminal: &mut Terminal<R, W>,
) -> Result<(), SetupError> {
    terminal.say(
        "현재 사용량 보호의 실제 효과:",
        "Current usage protection effects:",
    )?;
    let threshold = &progress.answers["usage_guard"]["stop_remaining_percent"];
    terminal.say(
        &format!("현재 잔여량 기준: {threshold}%"),
        &format!("Current remaining-quota threshold: {threshold}%"),
    )?;
    for question in catalog()?.questions.iter().filter(|question| {
        question.scope == PreferenceScope::User
            && (question.key.starts_with("/usage_guard/preflight/")
                || question.key.starts_with("/usage_guard/midway/"))
    }) {
        if let Some(value) = progress.answers.pointer(&question.key) {
            let korean = terminal.language == user_setup::InterfaceLanguage::Ko;
            let explicit =
                progress.answers["preference_decisions"][&question.key]["origin"] == "explicit";
            let origin = match (explicit, korean) {
                (true, true) => "직접 선택",
                (true, false) => "explicit choice",
                (false, true) => "이전값",
                (false, false) => "legacy value",
            };
            writeln!(
                terminal.writer,
                "- {} {} [{origin}]",
                question.prompt.get(korean),
                super::terminal::display_value(question, value, korean)
            )
            .map_err(|_| SetupError::Input("terminal connection ended".into()))?;
        }
    }
    terminal.say(
        "옛 설정에서 직접 선택 여부를 확인할 수 없는 값은 출처 불명으로 보존합니다.",
        "Legacy values without a verified decision remain marked as unknown origin.",
    )
}

pub(super) fn checkpoint(
    arguments: &user_setup::Arguments,
    answers: &[u8],
    expected: Option<&[u8]>,
    progress: &SetupProgress,
) -> Result<Option<Vec<u8>>, SetupError> {
    user_setup::progress_store::prepare_config_backup(&arguments.root_cap, progress, expected)?;
    let Some(binding) = &progress.legacy_migration else {
        return Ok(expected.map(<[u8]>::to_vec));
    };
    checkpoint_using(arguments, answers, expected, progress, || {
        crate::usage_monitor::quiesce_for_setup(
            &arguments.user_root,
            &binding.monitor_choice,
            &progress.operation_id,
        )
        .map_err(|failure| SetupError::Verification(failure.message().into()))
    })
}

fn checkpoint_using<T>(
    arguments: &user_setup::Arguments,
    answers: &[u8],
    expected: Option<&[u8]>,
    progress: &SetupProgress,
    stop_reader: impl FnOnce() -> Result<T, SetupError>,
) -> Result<Option<Vec<u8>>, SetupError> {
    use crate::user_install::{read_user_setup_file, replace_user_setup_file};
    let config = user_setup::parse_and_validate_config(answers)?;
    if config.schema_version != 2 {
        return Err(SetupError::Input("migration requires v2 settings".into()));
    }
    let desired = user_setup::canonical_config(&config)?;
    let relative = Path::new(user_setup::USER_SETUP_RELATIVE);
    let actual = read_user_setup_file(
        &arguments.root_cap,
        relative,
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let backup_path = user_setup::progress_store::recovery_path(progress)?;
    let backup = read_user_setup_file(
        &arguments.root_cap,
        &backup_path,
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    if actual.as_deref() == Some(desired.as_slice()) {
        let backup = backup.ok_or_else(|| {
            SetupError::Conflict("migration checkpoint has no recovery record".into())
        })?;
        if user_setup::progress_store::optional_digest(Some(&backup)) != progress.base_config_digest
        {
            return Err(SetupError::Conflict(
                "migration recovery record changed".into(),
            ));
        }
        if user_setup::progress_store::optional_digest(actual.as_deref())
            == progress.base_config_digest
        {
            let _reader = stop_reader()?;
        }
        return Ok(Some(desired));
    }
    let prior =
        expected.ok_or_else(|| SetupError::Conflict("legacy settings disappeared".into()))?;
    if actual.as_deref() != Some(prior)
        || user_setup::progress_store::optional_digest(Some(prior)) != progress.base_config_digest
        || user_setup::parse_and_validate_installed_config(prior)?.schema_version
            != progress
                .legacy_migration
                .as_ref()
                .expect("transition")
                .from_schema
    {
        return Err(SetupError::Conflict(
            "legacy settings changed after preview".into(),
        ));
    }
    if backup.as_deref().is_some_and(|bytes| bytes != prior) {
        return Err(SetupError::Conflict(
            "migration recovery record changed".into(),
        ));
    }
    if backup.is_none() {
        replace_user_setup_file(&arguments.root_cap, &backup_path, None, Some(prior))
            .map_err(SetupError::Conflict)?;
    }
    let _reader = stop_reader()?;
    replace_user_setup_file(&arguments.root_cap, relative, Some(prior), Some(&desired))
        .map_err(SetupError::Conflict)?;
    // Subsequent harness rollback sees this confirmed policy as its baseline, including opt-out.
    Ok(Some(desired))
}

pub(super) fn finish_monitor(
    root: &Path,
    answers: &[u8],
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    let Some(binding) = &progress.legacy_migration else {
        return Ok(());
    };
    let config = user_setup::parse_and_validate_config(answers)?;
    if binding.monitor_choice.enabled
        && config
            .usage_guard
            .policy
            .as_ref()
            .is_some_and(|policy| policy.enforcement_enabled && policy.midway.enabled)
    {
        crate::usage_monitor::resume_after_setup(root, &progress.operation_id)
            .map_err(|failure| SetupError::Verification(failure.message().into()))?;
    }
    Ok(())
}

pub(super) fn apply(
    arguments: &user_setup::Arguments,
    answers: &[u8],
    expected: Option<&[u8]>,
    progress: &SetupProgress,
) -> Result<crate::ActionResult, SetupError> {
    let baseline = checkpoint(arguments, answers, expected, progress)?;
    let applied = user_setup::execute_answers(
        arguments,
        answers,
        user_setup::SetupBase::Bound {
            expected: baseline.as_deref(),
            progress,
        },
    )?;
    let validation = user_setup::Arguments {
        answers: arguments.answers.clone(),
        mode: user_setup::SetupMode::Validate,
        user_root: arguments.user_root.clone(),
        root_cap: arguments
            .root_cap
            .try_clone()
            .map_err(|error| SetupError::Internal(error.to_string()))?,
    };
    user_setup::execute_answers(&validation, answers, user_setup::SetupBase::Current)?;
    finish_monitor(&arguments.user_root, answers, progress)?;
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use user_setup::progress_store::ProgressPhase;

    fn fixture(bits: u8) -> (tempfile::TempDir, ProgressStore, Conversation, Vec<u8>) {
        let root = tempfile::tempdir().unwrap();
        let raw = serde_json::to_vec(&json!({"schema_version":1,"interface_language":"ko",
            "wiki":{"language":"both"},"profile":{"contexts":["web-developer"]},
            "persona":{"id":"balanced"},"selected_hosts":["codex"],"skills":{"mode":"all"},
            "usage_guard":{"enabled":bits & 1 != 0,"quota_reset_guard_enabled":bits & 2 != 0,"stop_remaining_percent":5}})).unwrap();
        let path = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, &raw).unwrap();
        let original = user_setup::parse_and_validate_installed_config(&raw).unwrap();
        let preview = user_setup::migration::prepare(
            &original,
            crate::usage_monitor::SavedMonitorChoice {
                enabled: bits & 4 != 0,
                configured: bits & 4 != 0,
                config_digest: if bits & 4 != 0 {
                    hive_core::sha256_digest(b"monitor fixture")
                } else {
                    "absent".into()
                },
            },
        )
        .unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut record = store
            .new_progress(
                "usage-enforcement".into(),
                serde_json::to_value(preview.candidate).unwrap(),
            )
            .unwrap();
        record.legacy_migration = Some(LegacyMigration {
            migration_id: preview.migration_id,
            monitor_choice: preview.monitor_choice,
            from_schema: 1,
            questions: vec!["usage-enforcement".into(), "usage-warnings".into()],
        });
        let progress = Conversation {
            record,
            snapshot: store.load().unwrap(),
            fresh: false,
        };
        (root, store, progress, raw)
    }

    #[test]
    fn every_legacy_combination_supports_keep_or_disable_with_real_answer_provenance() {
        for bits in 0..8 {
            for keep in [false, true] {
                let (root, store, mut progress, raw) = fixture(bits);
                let before = progress.answers.clone();
                let mut terminal = Terminal {
                    reader: Cursor::new(if keep { "yes\nno\n" } else { "\n\n" }),
                    writer: Vec::new(),
                    language: user_setup::InterfaceLanguage::Ko,
                };
                let questions = catalog().unwrap();
                for (id, protection) in [("usage-enforcement", true), ("usage-warnings", false)] {
                    let question = questions
                        .questions
                        .iter()
                        .find(|question| question.id == id)
                        .unwrap();
                    assert!(
                        ask_choice(&store, &mut progress, question, &mut terminal, protection)
                            .unwrap()
                    );
                }
                let config = user_setup::parse_and_validate_config(
                    &serde_json::to_vec(&progress.answers).unwrap(),
                )
                .unwrap();
                let policy = config.usage_guard.policy.unwrap();
                assert_eq!(policy.enforcement_enabled, keep);
                assert_eq!(policy.warnings_enabled, !keep);
                assert_eq!(config.usage_guard.stop_remaining_percent, 5);
                assert_eq!(progress.answers["wiki"], before["wiki"]);
                assert_eq!(progress.answers["profile"], before["profile"]);
                if keep {
                    assert_eq!(
                        progress.answers["usage_guard"]["preflight"],
                        before["usage_guard"]["preflight"]
                    );
                    assert_eq!(
                        progress.answers["usage_guard"]["midway"],
                        before["usage_guard"]["midway"]
                    );
                } else {
                    assert_eq!(policy.preflight, UsageProtection::default().preflight);
                    assert_eq!(policy.midway, UsageProtection::default().midway);
                }
                let choice = &config.preference_decisions["/usage_guard/enforcement_enabled"];
                assert_eq!(
                    choice.origin,
                    if keep {
                        DecisionOrigin::Explicit
                    } else {
                        DecisionOrigin::AcceptedDefault
                    }
                );
                assert!(choice.decided_at.is_some());
                assert_eq!(
                    std::fs::read(root.path().join(user_setup::USER_SETUP_RELATIVE)).unwrap(),
                    raw
                );
            }
        }
    }

    #[test]
    fn new_review_progress_uses_the_canonical_question_ids() {
        let (root, store, _, raw) = fixture(0);
        let config = user_setup::parse_and_validate_installed_config(&raw).unwrap();
        let progress = initial_progress(root.path(), &store, Some(&config))
            .unwrap()
            .unwrap();
        let questions = &progress.legacy_migration.as_ref().unwrap().questions;
        assert_eq!(questions, &["usage-enforcement", "usage-warnings"]);
        store.save(&store.load().unwrap(), &progress).unwrap();
        for id in questions {
            assert!(catalog()
                .unwrap()
                .questions
                .iter()
                .any(|question| &question.id == id));
        }
    }

    #[test]
    fn cancel_and_eof_keep_the_old_selection_without_inventing_no() {
        for input in ["", ":cancel\n"] {
            let (root, store, mut progress, raw) = fixture(7);
            let before = progress.answers.clone();
            let question = catalog()
                .unwrap()
                .questions
                .into_iter()
                .find(|q| q.id == "usage-enforcement")
                .unwrap();
            let mut terminal = Terminal {
                reader: Cursor::new(input),
                writer: Vec::new(),
                language: user_setup::InterfaceLanguage::Ko,
            };
            assert!(!ask_choice(&store, &mut progress, &question, &mut terminal, true).unwrap());
            assert_eq!(store.load().unwrap().progress.unwrap().answers, before);
            assert!(progress.answered_questions.is_empty());
            assert_eq!(
                std::fs::read(root.path().join(user_setup::USER_SETUP_RELATIVE)).unwrap(),
                raw
            );
        }
    }

    #[test]
    fn confirmed_opt_out_is_checkpointed_before_harness_work_and_retains_its_backup() {
        struct ReaderLease(std::path::PathBuf);
        impl Drop for ReaderLease {
            fn drop(&mut self) {
                let config = user_setup::parse_and_validate_installed_config(
                    &std::fs::read(&self.0).unwrap(),
                )
                .unwrap();
                assert_eq!(config.schema_version, 2);
                assert!(!config.usage_guard.policy.unwrap().enforcement_enabled);
            }
        }
        let (root, store, mut progress, raw) = fixture(7);
        let question = catalog()
            .unwrap()
            .questions
            .into_iter()
            .find(|q| q.id == "usage-enforcement")
            .unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new("no\n"),
            writer: Vec::new(),
            language: user_setup::InterfaceLanguage::Ko,
        };
        ask_choice(&store, &mut progress, &question, &mut terminal, true).unwrap();
        progress.phase = ProgressPhase::Applying;
        persist(&store, &mut progress).unwrap();
        store.record_applying(&progress.record).unwrap();
        let (user_root, root_cap) =
            crate::user_install::open_canonical_user_root_for_setup(root.path()).unwrap();
        let arguments = user_setup::Arguments {
            answers: "fixture".into(),
            mode: user_setup::SetupMode::Apply,
            user_root,
            root_cap,
        };
        let answers = serde_json::to_vec(&progress.answers).unwrap();
        let baseline = checkpoint_using(&arguments, &answers, Some(&raw), &progress.record, || {
            Ok(ReaderLease(
                root.path().join(user_setup::USER_SETUP_RELATIVE),
            ))
        })
        .unwrap();
        let backup = root
            .path()
            .join(user_setup::progress_store::recovery_path(&progress.record).unwrap());
        assert_eq!(std::fs::read(&backup).unwrap(), raw);
        // A later failing harness operation has this off policy as its rollback baseline.
        assert_eq!(
            std::fs::read(root.path().join(user_setup::USER_SETUP_RELATIVE)).unwrap(),
            baseline.clone().unwrap()
        );
        assert_eq!(
            checkpoint_using::<()>(
                &arguments,
                &answers,
                Some(&raw),
                &progress.record,
                || panic!("checkpoint retry must not repeat stop")
            )
            .unwrap(),
            baseline
        );
        let reopened = ProgressStore::open(root.path(), None).unwrap();
        assert_eq!(
            reopened.load().unwrap().progress.unwrap().phase,
            ProgressPhase::Applying
        );
        assert!(backup.exists());
        std::fs::write(
            root.path().join(user_setup::USER_SETUP_RELATIVE),
            b"external change",
        )
        .unwrap();
        assert!(checkpoint_using::<()>(
            &arguments,
            &answers,
            Some(&raw),
            &progress.record,
            || panic!("external edit must be preserved")
        )
        .is_err());
    }
}
