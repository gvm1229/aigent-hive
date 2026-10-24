use super::{
    global_preferences, json, report, user_setup, InterfaceLanguage, Options, ProgressStore,
    SetupError, Value,
};
use crate::ActionResult;
use hive_core::preferences::{
    inheritance_declaration, DecisionOrigin, PreferenceDecision, PreferenceDecisions,
    PreferenceInput, PreferenceScope,
};

const INHERITED: [(&str, &str); 5] = [
    ("/interface_language", "/interface_language"),
    ("/wiki", "/wiki"),
    ("/persona", "/persona"),
    // This is the product's project default bundle, not the user's global Skill selection.
    ("/skills", "/project_defaults/skills"),
    (
        "/usage_stop_remaining_percent",
        "/usage_guard/stop_remaining_percent",
    ),
];

pub(super) fn record_defaults(answers: &mut Value) -> Result<(), SetupError> {
    let mut decisions: PreferenceDecisions =
        serde_json::from_value(answers["preference_decisions"].clone())
            .map_err(|error| SetupError::Verification(error.to_string()))?;
    for (key, value) in answers.as_object().expect("project answers") {
        if matches!(
            key.as_str(),
            "schema_version" | "preference_decisions" | "user_store_binding"
        ) {
            continue;
        }
        let pointer = format!("/{key}");
        // A structured value may already carry separate choices for its child fields.
        if decisions
            .keys()
            .any(|key| key == &pointer || key.starts_with(&format!("{pointer}/")))
        {
            continue;
        }
        decisions.insert(pointer, decision(value, DecisionOrigin::Default)?);
    }
    if answers["setup_mode"] == "expedited" {
        for (key, parent) in INHERITED {
            let declaration = inheritance_declaration(parent).map_err(SetupError::Internal)?;
            let previous = decisions.get(key);
            if previous.is_some_and(|value| {
                value.origin == DecisionOrigin::Inherited
                    && value.matches_declaration(&declaration).unwrap_or(false)
            }) {
                continue;
            }
            decisions.retain(|child, _| !child.starts_with(&format!("{key}/")));
            let mut inherited = decision(&declaration, DecisionOrigin::Inherited)?;
            if let Some(mode) = decisions.get("/setup_mode") {
                inherited.question_id.clone_from(&mode.question_id);
                inherited.question_revision = mode.question_revision;
                inherited.decided_at = mode.decided_at;
            }
            decisions.insert(key.into(), inherited);
        }
    }
    answers["preference_decisions"] =
        serde_json::to_value(decisions).map_err(|error| SetupError::Internal(error.to_string()))?;
    Ok(())
}

pub(super) fn accept_preview_defaults(answers: &mut Value) -> Result<(), SetupError> {
    let catalog = super::super::catalog()?;
    let now = user_setup::feature_now_unix()?;
    for question in catalog
        .questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::Project && q.applicable(answers))
        .collect::<Vec<_>>()
    {
        let Some(value) = answers.pointer(&question.key).cloned() else {
            continue;
        };
        if answers["preference_decisions"][&question.key]["origin"] != "default" {
            continue;
        }
        let mut record = PreferenceDecision::new(
            &value,
            DecisionOrigin::AcceptedDefault,
            PreferenceInput::Terminal,
            PreferenceScope::Project,
            env!("CARGO_PKG_VERSION"),
            Some(now),
        )
        .map_err(SetupError::Internal)?;
        record.question_id = Some(question.id.clone());
        record.question_revision = Some(question.revision);
        answers["preference_decisions"][&question.key] = json!(record);
    }
    Ok(())
}

fn decision(value: &Value, origin: DecisionOrigin) -> Result<PreferenceDecision, SetupError> {
    PreferenceDecision::new(
        value,
        origin,
        PreferenceInput::Terminal,
        PreferenceScope::Project,
        env!("CARGO_PKG_VERSION"),
        None,
    )
    .map_err(SetupError::Internal)
}

pub(super) fn inspect(
    options: &Options,
    store: &ProgressStore,
    saved: Option<&[u8]>,
    language: InterfaceLanguage,
) -> Result<ActionResult, SetupError> {
    let stored = saved
        .map(serde_yaml::from_slice::<Value>)
        .transpose()
        .map_err(|_| SetupError::Verification("invalid project settings".into()))?;
    let language = stored.as_ref().map_or(language, |value| {
        super::questions::language(value, language)
    });
    let pending = store.load()?.progress;
    let language = pending
        .as_ref()
        .filter(|progress| progress.phase != user_setup::progress_store::ProgressPhase::Complete)
        .map_or(language, |progress| {
            super::questions::language(&progress.answers, language)
        });
    let root = crate::user_install::open_user_root_for_setup(&options.root)
        .map_err(SetupError::Conflict)?;
    let parent_bytes = crate::user_install::read_user_setup_file(
        &root,
        std::path::Path::new(user_setup::USER_SETUP_RELATIVE),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let (sources, effective) = if let (Some(bytes), Some(_)) = (saved, parent_bytes.as_ref()) {
        let (sources, effective) =
            hive_render::inspect_project_preferences(bytes, &global_preferences(&options.root)?)
                .map_err(|error| SetupError::Verification(error.to_string()))?;
        (json!(sources), json!(effective))
    } else {
        (json!({}), Value::Null)
    };
    let parent = parent_bytes.as_deref().map(|bytes| -> Result<Value,SetupError> {
        let value = user_setup::parse_and_validate_installed_config(bytes)?;
        Ok(json!({"schema_version":value.schema_version,"settings_digest":hive_core::sha256_digest(bytes),
            "project_default_policy_version":env!("CARGO_PKG_VERSION")}))
    }).transpose()?;
    let after = crate::user_install::read_user_setup_file(
        &root,
        std::path::Path::new(user_setup::USER_SETUP_RELATIVE),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    if after != parent_bytes {
        return Err(SetupError::Conflict(
            "global settings changed during inspection; retry the read-only command".into(),
        ));
    }
    let data = if options.describe {
        let section = options
            .section
            .as_ref()
            .ok_or_else(|| SetupError::Input("project describe requires --section".into()))?;
        let questions = described(section, stored.as_ref(), &sources, &effective)?;
        json!({"section":section,"questions":questions,"parent":parent})
    } else {
        json!({"stored":stored,"effective":effective,"sources":sources,"parent":parent,"pending":pending})
    };
    Ok(report(
        "success",
        0,
        "hive.project-setup-shown",
        language,
        ["프로젝트 설정 조회 결과", "Project setup inspection"],
        Some(data),
    ))
}

fn described(
    section: &str,
    stored: Option<&Value>,
    sources: &Value,
    effective: &Value,
) -> Result<Value, SetupError> {
    let mut current = stored.cloned().unwrap_or_else(|| json!({}));
    current["preference_decisions"] = sources.clone();
    let mut questions =
        super::super::section_data(PreferenceScope::Project, section, Some(&current))?;
    let values = json!({"interface_language":effective["interface_language"],
        "wiki":{"enabled":effective["wiki_enabled"],"language":effective["wiki_language"]},
        "persona":{"id":effective["persona_id"],"custom_description":effective["persona_custom_description"]},
        "skills":{"mode":"individual","selected":effective["selected_project_skills"]},
        "usage_stop_remaining_percent":effective["usage_stop_remaining_percent"]});
    for entry in questions.as_array_mut().expect("questions") {
        let key = entry["question"]["key"]
            .as_str()
            .expect("question key")
            .to_owned();
        entry["effective"] = values
            .pointer(&key)
            .or_else(|| current.pointer(&key))
            .cloned()
            .unwrap_or(Value::Null);
        let group = format!("/{}", key.split('/').nth(1).expect("pointer component"));
        if let Some(source) = sources.get(&key).or_else(|| sources.get(&group)) {
            entry["source"] = source["origin"].clone();
        }
    }
    Ok(questions)
}
