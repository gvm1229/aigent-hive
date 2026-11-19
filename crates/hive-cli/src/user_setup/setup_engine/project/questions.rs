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
    let profile = hive_core::project_profile::ProjectProfile::detect(target);
    json!({"schema_version":2,"project_name":name,"project_identity":"","setup_mode":"expedited",
        "project_kind":profile.project_kind(),"project_profile":profile,"primary_host":host,"usage_stop_remaining_percent":global.usage_stop_remaining_percent,
        "elevated_judge_quorum":"2/3","critical_judge_quorum":"3/3+human","persistent_roles":[],
        "knowledge_include_paths":[],"knowledge_exclude_paths":[".env","**/.env","**/.env.*","**/credentials*","**/secrets/**"],
        "root_knowledge_promotion_categories":[],"confidential_knowledge_categories":[],
        "approved_optional_skills":[],"approved_fallback_hooks":[],"preference_decisions":{}})
}

/// Whether the project flow still asks `question` with the current answers.
fn will_ask(
    question: &Question,
    progress: &SetupProgress,
    global: &hive_render::GlobalProjectPreferences,
    review_sharing: bool,
    review_roles: bool,
) -> bool {
    if question.scope != PreferenceScope::Project {
        return false;
    }
    if question.id == "project-mode" {
        return !super::super::interactive::answered(progress, question);
    }
    let quick = progress.answers["setup_mode"] == "expedited";
    let quick_question = matches!(
        question.id.as_str(),
        "project-name" | "project-kind" | "project-host"
    ) || (review_sharing && question.id == "project-knowledge-promotion")
        || (review_roles && question.id == "project-roles");
    if matches!(
        question.id.as_str(),
        "project-role-update"
            | "project-knowledge-root"
            | "project-extras"
            | "project-extras-input"
            | "project-extras-confirm"
    ) || (quick && !quick_question)
    {
        return false;
    }
    question.applicable(&progress.answers)
        && !(super::super::interactive::answered(progress, question)
            && progress
                .answers
                .pointer(&question.key)
                .is_some_and(|value| {
                    super::input::error(question, &progress.answers, value, global).is_none()
                }))
}

pub(super) fn collect<R: BufRead, W: Write>(
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    global: &hive_render::GlobalProjectPreferences,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    terminal.language = language(&progress.answers, terminal.language);
    let review_sharing = progress.step == "project-knowledge-root";
    let review_roles = progress.step == "project-role-update";
    let catalog = super::super::catalog()?;
    let mut asked = 0;
    // Step number and the total still planned with the answers so far.
    let step = |asked: usize, from: usize, progress: &SetupProgress| {
        let remaining = catalog.questions[from..]
            .iter()
            .filter(|q| will_ask(q, progress, global, review_sharing, review_roles))
            .count();
        Some((asked + 1, asked + remaining.max(1)))
    };
    let mode = catalog
        .questions
        .iter()
        .find(|q| q.id == "project-mode")
        .expect("project mode");
    if !super::super::interactive::answered(progress, mode) {
        let numbered = step(asked, 0, progress);
        if !ask(store, snapshot, progress, mode, global, numbered, terminal)? {
            return Ok(false);
        }
        asked += 1;
    }
    let quick = progress.answers["setup_mode"] == "expedited";
    if !quick {
        seed_overrides(&mut progress.answers, global);
    }
    for (index, question) in catalog.questions.iter().enumerate() {
        if question.id == "project-mode"
            || !will_ask(question, progress, global, review_sharing, review_roles)
        {
            continue;
        }
        let numbered = step(asked, index, progress);
        if !ask(
            store, snapshot, progress, question, global, numbered, terminal,
        )? {
            return Ok(false);
        }
        asked += 1;
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

pub(super) fn seed_overrides(answers: &mut Value, global: &hive_render::GlobalProjectPreferences) {
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

/// Preview of the effective project values, grouped into titled sections of `name: value` rows.
pub(super) fn preview<R: BufRead, W: Write>(
    answers: &Value,
    global: &hive_render::GlobalProjectPreferences,
    changed_files: usize,
    terminal: &mut Terminal<R, W>,
) -> Result<(), SetupError> {
    use super::super::layout::{self, Row};
    let korean = terminal.language == InterfaceLanguage::Ko;
    let mut shown = answers.clone();
    let mut lines = Vec::new();
    if shown["setup_mode"] == "expedited" {
        lines.extend(layout::block(
            if korean {
                "언어·문서·답변 방식은 전역 설정을 따르고, 기능은 프로젝트 권장 묶음을 사용합니다. 아래는 현재 적용값입니다."
            } else {
                "Language, documents, and response style inherit global settings; features use the project default bundle. Current effective values follow."
            },
            layout::INDENT,
        ));
        seed_overrides(&mut shown, global);
        shown["usage_stop_remaining_percent"] = json!(global.usage_stop_remaining_percent);
    }
    let catalog = super::super::catalog()?;
    let (mut general, mut features, mut knowledge) = (Vec::new(), Vec::new(), Vec::new());
    for question in catalog
        .questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::Project && q.applicable(&shown))
    {
        let Some(value) = shown.pointer(&question.key) else {
            continue;
        };
        let row = Row {
            name: layout::question_title(&question.id, korean).to_owned(),
            values: super::super::layout::values(question, value, korean),
        };
        match question.section.as_str() {
            "skills" | "roles" | "extensions" => features.push(row),
            "knowledge" => knowledge.push(row),
            _ => general.push(row),
        }
    }
    features.extend(super::extras::choice_rows(answers, korean));
    let none = if korean { "없음" } else { "None" };
    let sections = [
        (if korean { "일반" } else { "General" }, general),
        (if korean { "기능" } else { "Features" }, features),
        (
            if korean {
                "지식 경로"
            } else {
                "Knowledge paths"
            },
            knowledge,
        ),
        (
            if korean {
                "변경 규모"
            } else {
                "Change size"
            },
            vec![Row {
                name: if korean {
                    "변경 파일 수"
                } else {
                    "Changed file count"
                }
                .to_owned(),
                values: vec![changed_files.to_string()],
            }],
        ),
    ];
    for (name, rows) in sections {
        lines.push(String::new());
        lines.push(layout::section(name));
        if rows.is_empty() {
            lines.push(format!("{}{none}", layout::INDENT));
        } else {
            lines.extend(layout::table(&rows).lines().map(str::to_owned));
        }
    }
    for line in lines {
        writeln!(terminal.writer, "{line}")
            .map_err(|error| SetupError::Internal(error.to_string()))?;
    }
    Ok(())
}

fn ask<R: BufRead, W: Write>(
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    progress: &mut SetupProgress,
    question: &Question,
    global: &hive_render::GlobalProjectPreferences,
    step: Option<(usize, usize)>,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    progress.step.clone_from(&question.id);
    *snapshot = store.save(snapshot, progress)?;
    let mut detected = None;
    if question.id == "project-kind" && progress.answers["setup_mode"] == "expedited" {
        if let Ok(profile) = serde_json::from_value::<hive_core::project_profile::ProjectProfile>(
            progress.answers["project_profile"].clone(),
        ) {
            if profile.project_kind() != "general" {
                let names = profile
                    .languages
                    .iter()
                    .chain(&profile.frameworks)
                    .chain(&profile.package_managers)
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut confirmation = question.clone();
                confirmation.input = hive_core::setup::InputKind::Boolean;
                confirmation.options.clear();
                confirmation.default = Some(json!(true));
                confirmation.prompt.ko =
                    format!("감지 결과: {names} ({}). 맞습니까?", profile.project_kind());
                confirmation.prompt.en = format!(
                    "Detected: {names} ({}). Is this correct?",
                    profile.project_kind()
                );
                match terminal.ask_step(&confirmation, None, step)? {
                    AnswerInput::Answered {
                        value: Value::Bool(true),
                        ..
                    } => {
                        detected = Some((
                            json!(profile.project_kind()),
                            hive_core::preferences::DecisionOrigin::AcceptedDefault,
                        ));
                    }
                    AnswerInput::Answered { .. } => {}
                    AnswerInput::Interrupted => return Ok(false),
                }
            }
        }
    }
    let (value, origin) = loop {
        if let Some(choice) = detected.take() {
            break choice;
        }
        let fallback = super::input::fallback(question, global);
        let current = progress
            .answers
            .pointer(&question.key)
            .filter(|value| {
                question.validate_value(value).is_ok()
                    && super::input::error(question, &progress.answers, value, global).is_none()
            })
            .or(fallback.as_ref());
        let AnswerInput::Answered { mut value, origin } =
            terminal.ask_step(question, current, step)?
        else {
            return Ok(false);
        };
        super::input::normalize(question, &mut value);
        if let Some((ko, en)) = super::input::error(question, &progress.answers, &value, global) {
            terminal.say(ko, en)?;
            continue;
        }
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
