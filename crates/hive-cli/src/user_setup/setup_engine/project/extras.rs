//! Optional configuration needs its own explicit consent before the complete settings preview.
use super::{prepare_files, Options, ProgressStore, SetupError, Terminal};
use crate::user_setup::{
    self,
    progress_store::{ProgressSnapshot, SetupProgress},
};
use hive_core::{
    preferences::{DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope},
    setup::{AnswerInput, Question},
};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn answered(progress: &SetupProgress, question: &Question) -> bool {
    progress.answered_questions.get(&question.id) == Some(&question.revision)
        && progress.question_bindings.get(&question.id) == Some(&question.into())
}
fn mark(progress: &mut SetupProgress, question: &Question) {
    progress
        .answered_questions
        .insert(question.id.clone(), question.revision);
    progress
        .question_bindings
        .insert(question.id.clone(), question.into());
}

pub(super) fn configure<R: BufRead, W: Write>(
    options: &Options,
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    if progress.answers["setup_mode"] != "custom" {
        return Ok(true);
    }
    let catalog = super::super::catalog()?;
    let question = |id: &str| {
        catalog
            .questions
            .iter()
            .find(|q| q.id == id)
            .expect("optional question")
    };
    let review = question("project-extras");
    match review_choices(store, snapshot, progress, review, terminal)? {
        Review::Complete => return Ok(true),
        Review::Pending => return Ok(false),
        Review::Edit => {}
    }
    let (_, caps, _) = prepare_files(options, &progress.answers)?;
    let capabilities =
        user_setup::read_bounded_regular(caps.path(), user_setup::MAX_USER_SETUP_BYTES)?;
    let supported = hive_render::supported_setup_hooks(&capabilities)
        .map_err(|error| SetupError::Input(error.to_string()))?;
    if supported.as_array().is_none_or(Vec::is_empty) {
        terminal.say("지원 증거가 있는 후크가 없어 새 후크를 선택할 수 없습니다. 빈 hooks 목록으로 기존 후크 해제는 가능합니다.","No hook support is verified. New hooks cannot be selected; an empty hooks list can revoke existing hooks.")?;
    } else {
        terminal.say("선택 가능한 후크:", "Available hooks:")?;
        writeln!(terminal.writer, "{supported}")
            .map_err(|error| SetupError::Internal(error.to_string()))?;
    }
    let input = question("project-extras-input");
    let previous = serde_json::to_vec(&progress.answers)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let preview = collect(
        options,
        store,
        snapshot,
        progress,
        input,
        &previous,
        &capabilities,
        terminal,
    )?;
    let Some(preview) = preview else {
        return Ok(false);
    };
    terminal.say(
        "승인할 원본·권한·후크 미리보기:",
        "Preview of sources, permissions, and hooks to approve:",
    )?;
    writeln!(terminal.writer, "{}", semantic(&preview))
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let confirm = question("project-extras-confirm");
    progress.step.clone_from(&confirm.id);
    *snapshot = store.save(snapshot, progress)?;
    let AnswerInput::Answered { value, .. } = terminal.ask(confirm, None)? else {
        return Ok(false);
    };
    if value != json!(true) {
        progress.answered_questions.remove(&input.id);
        *snapshot = store.save(snapshot, progress)?;
        return Ok(false);
    }
    let now = user_setup::feature_now_unix()?;
    let records = hive_render::materialize_setup_extras(
        &options.target,
        progress.extras_draft.as_ref().expect("draft"),
        &previous,
        &capabilities,
        now,
    )
    .map_err(|error| SetupError::Conflict(error.to_string()))?;
    if semantic(&records) != semantic(&preview) {
        return Err(SetupError::Conflict(
            "optional source changed after the consent preview; review it again".into(),
        ));
    }
    record_choices(progress, &records, confirm, now)?;
    progress.extras_approval_digest = Some(approval_digest(&records)?);
    mark(progress, review);
    mark(progress, confirm);
    progress.extras_draft = None;
    *snapshot = store.save(snapshot, progress)?;
    Ok(true)
}

enum Review {
    Complete,
    Pending,
    Edit,
}

fn review_choices<R: BufRead, W: Write>(
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    question: &Question,
    terminal: &mut Terminal<R, W>,
) -> Result<Review, SetupError> {
    let current = approval_digest(&progress.answers)?;
    if progress.extras_draft.is_some() {
        return Ok(Review::Edit);
    }
    if answered(progress, question) && progress.extras_approval_digest.as_ref() == Some(&current) {
        return Ok(Review::Complete);
    }
    progress.step.clone_from(&question.id);
    *snapshot = store.save(snapshot, progress)?;
    let AnswerInput::Answered { value, .. } = terminal.ask(question, None)? else {
        return Ok(Review::Pending);
    };
    mark(progress, question);
    if value != json!(true) {
        progress.extras_approval_digest = Some(current);
        *snapshot = store.save(snapshot, progress)?;
        return Ok(Review::Complete);
    }
    progress.extras_draft = Some(
        hive_render::setup_extras_request(
            &serde_json::to_vec(&progress.answers)
                .map_err(|error| SetupError::Internal(error.to_string()))?,
        )
        .map_err(|error| SetupError::Input(error.to_string()))?,
    );
    progress.answered_questions.remove("project-extras-input");
    *snapshot = store.save(snapshot, progress)?;
    Ok(Review::Edit)
}

fn approval_digest(records: &Value) -> Result<String, SetupError> {
    let approvals = json!({
        "approved_optional_skills": records["approved_optional_skills"],
        "approved_fallback_hooks": records["approved_fallback_hooks"],
    });
    hive_core::preferences::declaration_digest(&semantic(&approvals)).map_err(SetupError::Internal)
}

fn record_choices(
    progress: &mut SetupProgress,
    records: &Value,
    confirm: &Question,
    now: u64,
) -> Result<(), SetupError> {
    for key in ["approved_optional_skills", "approved_fallback_hooks"] {
        progress.answers[key] = records[key].clone();
        let mut record = PreferenceDecision::new(
            &records[key],
            DecisionOrigin::Explicit,
            PreferenceInput::Terminal,
            PreferenceScope::Project,
            env!("CARGO_PKG_VERSION"),
            Some(now),
        )
        .map_err(SetupError::Internal)?;
        record.question_id = Some(confirm.id.clone());
        record.question_revision = Some(confirm.revision);
        progress.answers["preference_decisions"][format!("/{key}")] = json!(record);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect<R: BufRead, W: Write>(
    options: &Options,
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    question: &Question,
    previous: &[u8],
    capabilities: &[u8],
    terminal: &mut Terminal<R, W>,
) -> Result<Option<Value>, SetupError> {
    loop {
        if !answered(progress, question) {
            progress.step.clone_from(&question.id);
            *snapshot = store.save(snapshot, progress)?;
            let AnswerInput::Answered { value, .. } =
                terminal.ask(question, progress.extras_draft.as_ref())?
            else {
                return Ok(None);
            };
            if user_setup::progress_contains_secret(&value) {
                terminal.say(
                    "자격 증명이나 비밀 주소는 입력하지 마세요.",
                    "Do not enter credentials or secret URLs.",
                )?;
                continue;
            }
            progress.extras_draft = Some(value);
        }
        match hive_render::materialize_setup_extras(
            &options.target,
            progress.extras_draft.as_ref().expect("draft"),
            previous,
            capabilities,
            0,
        ) {
            Ok(preview) => {
                mark(progress, question);
                *snapshot = store.save(snapshot, progress)?;
                return Ok(Some(preview));
            }
            Err(error) => {
                progress.answered_questions.remove(&question.id);
                terminal.say(
                    "원본 경로·권한·지원 후크를 확인해 다시 입력해 주세요.",
                    "Check the source path, permissions, and supported hooks, then try again.",
                )?;
                writeln!(terminal.writer, "{error}")
                    .map_err(|error| SetupError::Internal(error.to_string()))?;
            }
        }
    }
}

fn semantic(records: &Value) -> Value {
    let mut value = records.clone();
    for key in ["approved_optional_skills", "approved_fallback_hooks"] {
        if let Some(records) = value[key].as_array_mut() {
            for record in records {
                if let Some(object) = record.as_object_mut() {
                    object.remove("approved_at");
                    object.remove("consent_digest");
                }
            }
        }
    }
    value
}
