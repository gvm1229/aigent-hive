//! Withdraw an unfinished activation while preserving the other confirmed setup choices.
use super::{catalog, interactive, terminal::Terminal};
use crate::{
    usage_monitor::{self, review::Withdrawal},
    user_setup::{
        self,
        progress_store::{ProgressPhase, ProgressStore, SetupProgress},
        InterfaceLanguage, SetupError,
    },
    ActionResult,
};
use hive_core::{
    preferences::{
        declaration_digest, DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope,
    },
    setup::{AnswerInput, InteractionMode},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    path::Path,
};

const QUESTION: &str = "user-monitor-withdrawal";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Answers {
    question_id: String,
    question_revision: u32,
    answer: bool,
    confirmed: bool,
    preview_digest: String,
    #[serde(default = "file_channel")]
    input_channel: PreferenceInput,
}
fn file_channel() -> PreferenceInput {
    PreferenceInput::AnswersFile
}
fn native(error: &crate::run::AdapterError) -> SetupError {
    SetupError::Conflict(error.message().into())
}

fn eligible(progress: &SetupProgress) -> Result<(), SetupError> {
    if !matches!(
        progress.phase,
        ProgressPhase::Applying | ProgressPhase::RecoverableFailure
    ) || progress.monitor_start.is_none()
    {
        return Err(SetupError::Input(
            "no unfinished monitor consent to review".into(),
        ));
    }
    Ok(())
}

fn preview(root: &Path, progress: &SetupProgress) -> Result<Value, SetupError> {
    eligible(progress)?;
    let (_, cap) = crate::user_install::open_canonical_user_root_for_setup(root)
        .map_err(SetupError::Conflict)?;
    let current = crate::user_install::read_user_setup_file(
        &cap,
        Path::new(user_setup::USER_SETUP_RELATIVE),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let mut value = json!({"operation_id":progress.operation_id,
        "intent_digest":ProgressStore::monitor_intent_digest(progress)?,
        "config_digest":user_setup::progress_store::optional_digest(current.as_deref()),
        "scope":usage_monitor::review::scope(root).map_err(|error| native(&error))?,
        "question":catalog()?.questions.into_iter().find(|q| q.id == QUESTION)
            .ok_or_else(|| SetupError::Internal("missing withdrawal question".into()))?,
        "effect":"stop-owned-monitor-and-apply-other-confirmed-settings","history_preserved":true,"sensor_queries":0});
    value["preview_digest"] = json!(declaration_digest(&value).map_err(SetupError::Internal)?);
    Ok(value)
}

pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    mode: InteractionMode,
    dry_run: bool,
    answers: Option<&Path>,
    terminal: &mut Terminal<R, W>,
) -> Result<ActionResult, SetupError> {
    if mode == InteractionMode::NeedsInput && !dry_run {
        return Ok(interactive::pending(
            terminal.language,
            "hive.setup-needs-input",
        ));
    }
    let store = ProgressStore::open(root, None)?;
    if !dry_run && mode != InteractionMode::NeedsInput {
        store.recover_monitor_handoff()?;
    }
    let progress = store
        .load()?
        .progress
        .ok_or_else(|| SetupError::Input("no saved setup to review".into()))?;
    if progress.phase == ProgressPhase::Complete && store.resume_ready(&progress) {
        return Ok(complete(terminal.language, &progress.operation_id, None));
    }
    if !dry_run && progress.step == QUESTION && progress.monitor_start.is_none() {
        return super::recovery::resume(root, &progress, terminal.language, false);
    }
    if !dry_run {
        if let Some(result) = resume(root, &progress, terminal.language)? {
            return Ok(result);
        }
    }
    let shown = preview(root, &progress)?;
    if dry_run || (mode != InteractionMode::Terminal && answers.is_none()) {
        let mut result = interactive::pending(terminal.language, "hive.setup-needs-input");
        if dry_run {
            result.status = "success";
            result.exit_code = 0;
            result.code = "hive.user-setup-preview";
        }
        let mut data = shown;
        // Runtime observations describe the current state; they are not consent inputs.
        data["actual_monitor_state"] = usage_monitor::setup_status(root);
        result.data = Some(data);
        result.next_action = Some(format!(
            "{} --review",
            crate::update_bridge::resume_command(root).replacen(
                "hive update",
                "hive setup --scope user --resume",
                1
            )
        ));
        return Ok(result);
    }
    let channel = if let Some(path) = answers {
        confirmed_file_channel(path, &shown)?
    } else {
        let question = catalog()?
            .questions
            .into_iter()
            .find(|q| q.id == QUESTION)
            .ok_or_else(|| SetupError::Internal("missing withdrawal question".into()))?;
        match terminal.ask(&question, None)? {
            AnswerInput::Answered { value, .. } if value == true => {}
            AnswerInput::Answered { .. } => {
                return super::recovery::resume(root, &progress, terminal.language, false)
            }
            AnswerInput::Interrupted => {
                return Ok(interactive::pending(
                    terminal.language,
                    "hive.setup-input-pending",
                ))
            }
        }
        terminal.say("이 사용자 계정의 Hive 감시를 종료합니다. 확인한 다른 설정의 적용은 마무리하고 사용량 기록은 보존합니다.",
            "Stop the owned Hive monitor for this user. Finish the other confirmed settings and preserve usage history.")?;
        interactive::display_monitor(&usage_monitor::setup_status(root), terminal)?;
        let mut displayed = progress.answers.clone();
        displayed["usage_guard"]["midway"]["enabled"] = json!(false);
        interactive::display_preview(&displayed, terminal)?;
        if !terminal.confirm()? {
            return Ok(interactive::pending(
                terminal.language,
                "hive.setup-input-pending",
            ));
        }
        PreferenceInput::Terminal
    };
    let consent = Withdrawal {
        operation_id: progress.operation_id.clone(),
        intent_digest: shown["intent_digest"].as_str().unwrap().into(),
        config_digest: shown["config_digest"].as_str().unwrap().into(),
        preview_digest: shown["preview_digest"].as_str().unwrap().into(),
        question_id: QUESTION.into(),
        question_revision: 1,
        channel,
        answered_at: user_setup::feature_now_unix()?,
        scope: shown["scope"].clone(),
    };
    apply(root, &progress, &consent, terminal.language, Some(&shown))
}

fn confirmed_file_channel(path: &Path, shown: &Value) -> Result<PreferenceInput, SetupError> {
    let input: Answers = serde_json::from_slice(&user_setup::read_bounded_regular(path, 4096)?)
        .map_err(|_| SetupError::Input("invalid monitor review answer file".into()))?;
    if input.question_id != QUESTION
        || input.question_revision != 1
        || !input.answer
        || !input.confirmed
        || !matches!(
            input.input_channel,
            PreferenceInput::AnswersFile | PreferenceInput::Agent
        )
        || shown["preview_digest"] != input.preview_digest
    {
        return Err(SetupError::Input(
            "monitor review answer does not confirm this preview".into(),
        ));
    }
    Ok(input.input_channel)
}

pub(super) fn resume(
    root: &Path,
    progress: &SetupProgress,
    language: InterfaceLanguage,
) -> Result<Option<ActionResult>, SetupError> {
    if progress.monitor_start.is_none() {
        return Ok(None);
    }
    let Some(consent) = usage_monitor::review::pending(root)
        .map_err(|error| native(&error))?
        .filter(|c| c.operation_id == progress.operation_id)
    else {
        return Ok(None);
    };
    apply(root, progress, &consent, language, None).map(Some)
}

fn apply(
    root: &Path,
    progress: &SetupProgress,
    consent: &Withdrawal,
    language: InterfaceLanguage,
    shown: Option<&Value>,
) -> Result<ActionResult, SetupError> {
    eligible(progress)?;
    let (user_root, root_cap) = crate::user_install::open_canonical_user_root_for_setup(root)
        .map_err(SetupError::Conflict)?;
    let lease = crate::user_install::lock_user_setup(&root_cap).map_err(SetupError::Conflict)?;
    let store = ProgressStore::open(root, None)?;
    if store.load()?.progress.as_ref() != Some(progress)
        || ProgressStore::monitor_intent_digest(progress)? != consent.intent_digest
    {
        return Err(SetupError::Conflict(
            "monitor review inputs changed; preserved".into(),
        ));
    }
    if let Some(shown) = shown {
        if preview(root, progress)? != *shown {
            return Err(SetupError::Conflict(
                "monitor review inputs changed; preserved".into(),
            ));
        }
    }
    if shown.is_none() {
        user_setup::progress_store::recover_config(&root_cap, &store, progress)?;
    }
    let current = crate::user_install::read_user_setup_file(
        &root_cap,
        Path::new(user_setup::USER_SETUP_RELATIVE),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let desired = user_setup::canonical_config(&user_setup::parse_and_validate_config(
        &serde_json::to_vec(&progress.answers).map_err(|e| SetupError::Internal(e.to_string()))?,
    )?)?;
    if user_setup::progress_store::optional_digest(current.as_deref()) != consent.config_digest
        && current.as_deref() != Some(desired.as_slice())
    {
        return Err(SetupError::Conflict(
            "settings changed after monitor withdrawal; preserved".into(),
        ));
    }
    let work = (|| {
        let stopped = usage_monitor::review::stop(root, consent).map_err(|error| native(&error))?;
        interruption("stopped");
        let (prior, _) = super::recovery::settings(root, progress, Some(&stopped))?;
        interruption("settings");
        let mut answers = progress.answers.clone();
        answers["usage_guard"]["midway"]["enabled"] = json!(false);
        let mut decision = PreferenceDecision::new(
            &json!(false),
            DecisionOrigin::Explicit,
            consent.channel,
            PreferenceScope::User,
            env!("CARGO_PKG_VERSION"),
            Some(consent.answered_at),
        )
        .map_err(SetupError::Internal)?;
        decision.question_id = Some(QUESTION.into());
        decision.question_revision = Some(1);
        answers["preference_decisions"]["/usage_guard/midway/enabled"] = json!(decision);
        let next_store = ProgressStore::open(root, None)?;
        let replacement = next_store.handoff_monitor(progress, answers, &stopped)?;
        interruption("handoff");
        let arguments = user_setup::Arguments {
            answers: "saved:monitor-withdrawal".into(),
            mode: user_setup::SetupMode::Apply,
            user_root,
            root_cap,
        };
        let bytes = serde_json::to_vec(&replacement.answers)
            .map_err(|e| SetupError::Internal(e.to_string()))?;
        super::legacy::checkpoint_stopped(
            &arguments,
            &bytes,
            Some(&desired),
            &replacement,
            &stopped,
        )?;
        interruption("checkpoint");
        Ok((replacement, prior.changed_paths))
    })();
    let (replacement, prior_paths) = match work {
        Ok(value) => value,
        Err(error) => {
            // Handoff can already have cancelled the old operation; never revive its receipt.
            let current = ProgressStore::open(root, None)?;
            if !current.monitor_handoff_pending()?
                && current
                    .load()?
                    .progress
                    .as_ref()
                    .is_some_and(|p| p.operation_id == progress.operation_id)
            {
                current.record_failure(progress)?;
            }
            return Err(error);
        }
    };
    drop(lease);
    let mut result = super::recovery::resume(root, &replacement, language, false)?;
    result.changed_paths.extend(prior_paths);
    result.changed_paths.sort();
    result.changed_paths.dedup();
    Ok(result)
}

pub(super) fn report(
    root: &Path,
    progress: &SetupProgress,
    language: InterfaceLanguage,
    result: &mut ActionResult,
) -> Result<(), SetupError> {
    if progress.step != QUESTION {
        return Ok(());
    }
    let consent = usage_monitor::review::pending(root)
        .map_err(|error| native(&error))?
        .ok_or_else(|| SetupError::Conflict("monitor withdrawal receipt is missing".into()))?;
    result.message = complete(
        language,
        &progress.operation_id,
        Some(&consent.operation_id),
    )
    .message;
    result.data.get_or_insert_with(|| json!({}))["monitor_review"] = json!({"withdrawn":true,
        "cancelled_operation_id":consent.operation_id,"operation_id":progress.operation_id,
        "monitor_stopped":true,"history_preserved":true,"input_channel":consent.channel});
    Ok(())
}

fn interruption(point: &str) {
    #[cfg(debug_assertions)]
    if std::env::var("HIVE_TEST_SETUP_REVIEW_EXIT_AFTER").as_deref() == Ok(point) {
        std::process::exit(89);
    }
    #[cfg(not(debug_assertions))]
    let _ = point;
}

fn complete(language: InterfaceLanguage, operation: &str, cancelled: Option<&str>) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "SetupHiveUser",
        status: "success",
        exit_code: 0,
        code: "hive.user-setup-complete",
        message: match (language, cancelled.is_some()) {
            (InterfaceLanguage::Ko, true) => {
                "감시 동의를 철회하고 다른 설정의 적용·검증을 마쳤습니다."
            }
            (InterfaceLanguage::En, true) => {
                "Monitoring consent was withdrawn. Other settings were applied and verified."
            }
            (InterfaceLanguage::Ko, false) => "이미 적용·검증을 마친 설정입니다.",
            (InterfaceLanguage::En, false) => "This setup was already applied and verified.",
        }
        .into(),
        changed_paths: vec![],
        evidence: vec![],
        next_action: None,
        data: Some(json!({"operation_id":operation,"model_calls":0})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> (tempfile::TempDir, SetupProgress) {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut answers = super::super::default_answers(Some(InterfaceLanguage::Ko));
        answers["usage_guard"]["enforcement_enabled"] = json!(true);
        answers["usage_guard"]["midway"]["enabled"] = json!(true);
        let mut progress = store
            .new_progress("user-monitor-activation".into(), answers)
            .unwrap();
        progress.monitor_start = Some(super::super::monitor_start::fixture_approval(
            &progress.answers,
        ));
        super::super::application::begin(&store, &store.load().unwrap(), &mut progress).unwrap();
        (root, progress)
    }

    #[test]
    fn withdrawal_preview_machine_input_and_eof_preserve_the_original_operation() {
        for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
            let (root, original) = fixture();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let original_bytes = std::fs::read(root.path().join(&store.relative)).unwrap();
            for (mode, dry_run, input) in [
                (InteractionMode::NonInteractive, true, "unused\n"),
                (InteractionMode::NonInteractive, false, "yes\n"),
                (InteractionMode::Terminal, false, ""),
                (InteractionMode::Terminal, false, "yes\nno\n"),
            ] {
                let mut terminal = Terminal {
                    reader: Cursor::new(input),
                    writer: Vec::new(),
                    language,
                };
                let result = run(root.path(), mode, dry_run, None, &mut terminal).unwrap();
                assert_eq!(result.exit_code, if dry_run { 0 } else { 2 });
                if dry_run {
                    assert!(result.data.unwrap()["preview_digest"]
                        .as_str()
                        .unwrap()
                        .starts_with("sha256:"));
                }
                assert_eq!(
                    std::fs::read(root.path().join(&store.relative)).unwrap(),
                    original_bytes
                );
                assert_eq!(store.load().unwrap().progress, Some(original.clone()));
                assert!(!root.path().join(".hive/runtime/usage-monitor").exists());
                assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
                if mode == InteractionMode::NonInteractive {
                    assert_eq!(terminal.reader.position(), 0);
                }
            }
        }
    }

    #[test]
    fn file_review_rejects_forged_channel_unknown_fields_and_stale_confirmation_without_writes() {
        let (root, original) = fixture();
        let shown = preview(root.path(), &original).unwrap();
        let base = json!({"question_id":QUESTION,"question_revision":1,"answer":true,"confirmed":true,
            "preview_digest":shown["preview_digest"]});
        for (key, value) in [
            ("input_channel", json!("terminal")),
            ("unknown", json!(true)),
            ("confirmed", json!(false)),
            ("preview_digest", json!("sha256:stale")),
            ("question_revision", json!(2)),
        ] {
            let mut input = base.clone();
            input[key] = value;
            let path = root.path().join("review.json");
            std::fs::write(&path, serde_json::to_vec(&input).unwrap()).unwrap();
            let mut terminal = Terminal {
                reader: Cursor::new(""),
                writer: Vec::new(),
                language: InterfaceLanguage::En,
            };
            assert!(run(
                root.path(),
                InteractionMode::NonInteractive,
                false,
                Some(&path),
                &mut terminal
            )
            .is_err());
            assert!(!root.path().join(".hive/runtime/usage-monitor").exists());
            assert_eq!(
                ProgressStore::open(root.path(), None)
                    .unwrap()
                    .load()
                    .unwrap()
                    .progress,
                Some(original.clone())
            );
        }
    }
}
