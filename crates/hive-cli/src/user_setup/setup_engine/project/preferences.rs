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
    if answers["setup_mode"] == "custom" {
        decisions.retain(|_, record| record.origin != DecisionOrigin::Inherited);
    }
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

#[allow(clippy::too_many_lines)]
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
    let mut data = if options.describe {
        let selected_section = if let Some(id) = &options.question {
            super::super::catalog()?
                .questions
                .into_iter()
                .find(|q| q.scope == PreferenceScope::Project && q.id == *id)
                .map(|q| q.section)
                .ok_or_else(|| SetupError::Input("unknown project setup question".into()))?
        } else {
            options.section.clone().ok_or_else(|| {
                SetupError::Input("project describe requires --section or --question".into())
            })?
        };
        let section = selected_section.as_str();
        let mut questions = described(section, stored.as_ref(), &sources, &effective)?;
        let profile = hive_core::project_profile::ProjectProfile::detect(&options.target);
        if stored.is_none() {
            for entry in questions.as_array_mut().expect("questions") {
                if entry["question"]["id"] == "project-kind" {
                    entry["question"]["default"] = json!(profile.project_kind());
                }
            }
        }
        if let Some(id) = &options.question {
            questions = super::super::summary::one_question(
                questions,
                id,
                language == InterfaceLanguage::Ko,
            )?;
        }
        json!({"section":section,"questions":questions,"parent":parent,"project_profile":profile})
    } else {
        json!({"stored":stored,"effective":effective,"sources":sources,"parent":parent,"pending":pending})
    };
    let next_action = offer_agent_setup(
        options,
        &root,
        stored.is_none(),
        pending.as_ref(),
        &mut data,
    );
    let mut result = report(
        "success",
        0,
        if options.question.is_some() {
            "hive.setup-question-described"
        } else {
            "hive.project-setup-shown"
        },
        language,
        ["프로젝트 설정 조회 결과", "Project setup inspection"],
        Some(data),
    );
    result.next_action = next_action;
    Ok(result)
}

/// An unconfigured project offers the agent path with its proposal as the first-class next action.
fn offer_agent_setup(
    options: &Options,
    root: &cap_std::fs::Dir,
    unconfigured: bool,
    pending: Option<&user_setup::progress_store::SetupProgress>,
    data: &mut Value,
) -> Option<String> {
    // An untouched terminal draft never blocks the agent apply, so it counts as fresh here too.
    let draft = pending.filter(|progress| super::file_input::is_untouched_draft(progress));
    let fresh = !options.describe
        && unconfigured
        && pending.is_none_or(|progress| {
            progress.phase == user_setup::progress_store::ProgressPhase::Complete || draft.is_some()
        });
    if !fresh {
        return None;
    }
    // A read-only status call never fails because the agent proposal is unavailable.
    let agent = agent_setup(options, root).unwrap_or_else(|error| {
        json!({"unavailable":true,"reason":error.message().chars().take(240).collect::<String>()})
    });
    let next_action = if agent["user_setup_required"] == true {
        Some(super::user_setup_command(options))
    } else if agent["unavailable"] == true {
        None
    } else {
        Some(super::agent_command(options))
    };
    data["agent_setup"] = agent;
    if draft.is_some() {
        data["agent_setup"]["terminal_draft"] = json!("superseded-on-apply");
    }
    next_action
}

/// Whether the target holds nothing but hidden entries, read with a small bound.
fn project_state(target: &std::path::Path) -> &'static str {
    let Ok(entries) = std::fs::read_dir(target) else {
        return "existing";
    };
    let visible = entries
        .take(256)
        .filter_map(Result::ok)
        .any(|entry| !entry.file_name().to_string_lossy().starts_with('.'));
    if visible {
        "existing"
    } else {
        "empty"
    }
}

/// CLI-detected starting values for the agent; the agent's interview or scan decides the answers.
fn agent_setup(options: &Options, root: &cap_std::fs::Dir) -> Result<Value, SetupError> {
    let Some(config) = user_setup::load_operational_config(root)? else {
        return Ok(json!({"user_setup_required":true,
            "reason":"Finish global user setup first; project file setup reads its saved preferences."}));
    };
    let host = config
        .selected_hosts
        .first()
        .map_or("codex", |host| host.as_str());
    let defaults =
        super::questions::defaults(&options.target, &global_preferences(&options.root)?, host);
    let names = |key: &str| {
        defaults["project_profile"][key]
            .as_array()
            .map_or_else(Vec::new, |facts| {
                facts
                    .iter()
                    .map(|fact| fact["name"].clone())
                    .collect::<Vec<_>>()
            })
    };
    let state = project_state(&options.target);
    let mut detected_defaults = json!({"setup_mode":defaults["setup_mode"],
        "project_name":defaults["project_name"],"primary_host":defaults["primary_host"]});
    if state == "existing" {
        detected_defaults["project_kind"] = defaults["project_kind"].clone();
    }
    Ok(json!({
        "project_state":state,
        "requires_user_confirmation":true,
        "required_answers":["setup_mode","project_name","project_kind","primary_host"],
        "detected_defaults":detected_defaults,
        "selected_hosts":config.selected_hosts.iter().map(|host| host.as_str()).collect::<Vec<_>>(),
        "detected":{"languages":names("languages"),"frameworks":names("frameworks"),
            "package_managers":names("package_managers")},
        "capabilities":"saved-or-detected-by-cli",
        "consent_answers_default_empty":["persistent_roles","approved_optional_skills","approved_fallback_hooks","knowledge_include_paths"]
    }))
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
