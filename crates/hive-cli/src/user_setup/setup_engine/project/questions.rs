use super::{
    json, user_setup, BufRead, InterfaceLanguage, Path, ProgressStore, SetupError, Terminal, Value,
    Write,
};
use crate::user_setup::progress_store::{ProgressSnapshot, SetupProgress};
use hive_core::preferences::{PreferenceDecision, PreferenceInput, PreferenceScope};
use hive_core::setup::{AnswerInput, Question};

pub(super) fn defaults(
    target: &Path,
    global: &hive_render::GlobalProjectPreferences,
    host: &str,
) -> Value {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.chars().any(char::is_control))
        .unwrap_or("project");
    json!({"schema_version":2,"project_name":name,"project_identity":"","setup_mode":"expedited",
        "project_kind":"general","primary_host":host,"usage_stop_remaining_percent":global.usage_stop_remaining_percent,
        "elevated_judge_quorum":"2/3","critical_judge_quorum":"3/3+human","persistent_roles":[],
        "knowledge_include_paths":[],"knowledge_exclude_paths":[".env","**/.env","**/.env.*","**/credentials*","**/secrets/**"],
        "root_knowledge_promotion_categories":[],"confidential_knowledge_categories":[],
        "approved_optional_skills":[],"approved_fallback_hooks":[],"preference_decisions":{}})
}

pub(super) fn collect<R: BufRead, W: Write>(
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    global: &hive_render::GlobalProjectPreferences,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    terminal.language = language(&progress.answers, terminal.language);
    let catalog = super::super::catalog()?;
    let mode = catalog
        .questions
        .iter()
        .find(|q| q.id == "project-mode")
        .expect("project mode");
    if !super::super::interactive::answered(progress, mode)
        && !ask(store, snapshot, progress, mode, terminal)?
    {
        return Ok(false);
    }
    let quick = progress.answers["setup_mode"] == "expedited";
    if !quick {
        seed_overrides(&mut progress.answers, global);
    }
    for question in catalog
        .questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::Project)
    {
        if question.id == "project-mode"
            || (quick
                && !matches!(
                    question.id.as_str(),
                    "project-name" | "project-kind" | "project-host"
                ))
        {
            continue;
        }
        if !question.applicable(&progress.answers)
            || super::super::interactive::answered(progress, question)
        {
            continue;
        }
        if !ask(store, snapshot, progress, question, terminal)? {
            return Ok(false);
        }
        if question.id == "project-language" {
            terminal.language = if progress.answers["interface_language"] == "ko" {
                InterfaceLanguage::Ko
            } else {
                InterfaceLanguage::En
            };
        }
    }
    if quick {
        for key in ["interface_language", "wiki", "persona", "skills"] {
            progress
                .answers
                .as_object_mut()
                .expect("answers")
                .remove(key);
        }
    }
    if progress.answers["project_identity"]
        .as_str()
        .is_none_or(str::is_empty)
    {
        progress.answers["project_identity"] = progress.answers["project_name"].clone();
    }
    super::preferences::record_defaults(&mut progress.answers)?;
    *snapshot = store.save(snapshot, progress)?;
    Ok(true)
}

pub(super) fn language(answers: &Value, inherited: InterfaceLanguage) -> InterfaceLanguage {
    if answers["setup_mode"] != "custom" {
        return inherited;
    }
    match answers["interface_language"].as_str() {
        Some("ko") => InterfaceLanguage::Ko,
        Some("en") => InterfaceLanguage::En,
        _ => inherited,
    }
}

fn seed_overrides(answers: &mut Value, global: &hive_render::GlobalProjectPreferences) {
    let object = answers.as_object_mut().expect("answers");
    object
        .entry("interface_language")
        .or_insert_with(|| json!(global.interface_language));
    object
        .entry("wiki")
        .or_insert_with(|| json!({"enabled":global.wiki_enabled,"language":global.wiki_language}));
    object.entry("persona").or_insert_with(|| {
        let mut value = json!({"id":global.persona_id});
        if let Some(text) = &global.persona_custom_description {
            value["custom_description"] = json!(text);
        }
        value
    });
    object
        .entry("skills")
        .or_insert_with(|| json!({"mode":"individual","selected":global.selected_project_skills}));
}

pub(super) fn preview<R: BufRead, W: Write>(
    answers: &Value,
    global: &hive_render::GlobalProjectPreferences,
    terminal: &mut Terminal<R, W>,
) -> Result<(), SetupError> {
    let mut shown = answers.clone();
    if shown["setup_mode"] == "expedited" {
        terminal.say(
            "언어·문서·답변 방식은 전역 설정을 따르고, 기능은 프로젝트 권장 묶음을 사용합니다. 아래는 현재 적용값입니다.",
            "Language, documents, and response style inherit global settings; features use the project default bundle. Current effective values follow.",
        )?;
        seed_overrides(&mut shown, global);
        shown["usage_stop_remaining_percent"] = json!(global.usage_stop_remaining_percent);
    }
    let catalog = super::super::catalog()?;
    for question in catalog
        .questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::Project && q.applicable(&shown))
    {
        if let Some(value) = shown.pointer(&question.key) {
            writeln!(
                terminal.writer,
                "{} {}",
                question
                    .prompt
                    .get(terminal.language == InterfaceLanguage::Ko),
                super::super::terminal::display_value(
                    question,
                    value,
                    terminal.language == InterfaceLanguage::Ko
                )
            )
            .map_err(|error| SetupError::Internal(error.to_string()))?;
        }
    }
    Ok(())
}

fn ask<R: BufRead, W: Write>(
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    question: &Question,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    progress.step.clone_from(&question.id);
    *snapshot = store.save(snapshot, progress)?;
    let (value, origin) = loop {
        let current = progress
            .answers
            .pointer(&question.key)
            .filter(|value| question.validate_value(value).is_ok());
        let AnswerInput::Answered { value, origin } = terminal.ask(question, current)? else {
            return Ok(false);
        };
        if super::super::interactive::config_answer_valid(question, &progress.answers, &value)? {
            break (value, origin);
        }
        terminal.say(
            "입력값의 구조와 범위를 확인해 주세요.",
            "Check the answer's structure and bounds.",
        )?;
    };
    super::super::interactive::set_value(&mut progress.answers, &question.key, value.clone())?;
    let mut decision = PreferenceDecision::new(
        &value,
        origin,
        PreferenceInput::Terminal,
        PreferenceScope::Project,
        env!("CARGO_PKG_VERSION"),
        Some(user_setup::feature_now_unix()?),
    )
    .map_err(SetupError::Input)?;
    decision.question_id = Some(question.id.clone());
    decision.question_revision = Some(question.revision);
    progress.answers["preference_decisions"][&question.key] = json!(decision);
    progress
        .answered_questions
        .insert(question.id.clone(), question.revision);
    progress
        .question_bindings
        .insert(question.id.clone(), question.into());
    *snapshot = store.save(snapshot, progress)?;
    Ok(true)
}
