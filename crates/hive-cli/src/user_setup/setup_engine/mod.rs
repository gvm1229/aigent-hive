//! CLI adapters for the shared deterministic question contract.

use super::{InterfaceLanguage, SetupError};
use crate::{ActionResult, Evidence};
use hive_core::preferences::{DecisionOrigin, PreferenceScope};
use hive_core::setup::QuestionCatalog;
use serde_json::{json, Value};

mod interactive;
mod file_input;
mod answer_patch;
mod summary;
pub(crate) use summary::{arguments as summary_arguments, compact as compact_result};
mod file_apply;
mod application;
mod monitor_review;
mod file_policy;
mod index_rebuild;
mod monitor_stop;
pub(in crate::user_setup) mod vector_choice;
pub(in crate::user_setup) mod monitor_start;
mod project;
pub(in crate::user_setup) use project::run as run_project;
pub(in crate::user_setup) use project::execute_file as execute_project_file;
pub(in crate::user_setup) use file_input::{file_options, execute as execute_file};
#[cfg(test)]
pub(in crate::user_setup) use file_input::FileOptions;
mod host_scope;
pub(in crate::user_setup) use host_scope::validate as validate_update_hosts;
mod legacy;
mod recovery;
mod terminal;
pub(in crate::user_setup) mod messages;
mod worker;
mod update;
mod update_summary;
pub(in crate::user_setup) use update::run as run_update_completion;
#[cfg(any(windows, target_os = "linux"))]
pub(in crate::user_setup) use update::legacy_noninteractive as update_without_terminal;
pub(super) use interactive::resume_pending_update;
#[cfg(any(windows, target_os = "linux"))]
pub(super) use interactive::update_with_terminal;
pub(super) use interactive::{handles, run};
pub(in crate::user_setup) use worker::command_timeout;
pub(in crate::user_setup) use worker::run as run_apply_worker;

pub(super) const QUESTIONS: &str = include_str!("../../../../../harness/setup/questions.yml");

pub(super) fn catalog() -> Result<QuestionCatalog, SetupError> {
    let catalog: QuestionCatalog = serde_yaml::from_str(QUESTIONS)
        .map_err(|error| SetupError::Internal(format!("invalid setup questions: {error}")))?;
    catalog.validate().map_err(SetupError::Internal)?;
    Ok(catalog)
}

pub(in crate::user_setup) fn default_answers(language: Option<InterfaceLanguage>) -> Value {
    let mut guard = serde_json::to_value(hive_core::preferences::UsageProtection::default()).expect("typed usage policy");
    guard["stop_remaining_percent"] = json!(super::EXPEDITED_DEFAULT_USAGE_THRESHOLD);
    guard["codexbar_fallback_enabled"] = json!(false);
    guard["popup_enabled"] = json!(false);
    guard["discord"] = serde_json::to_value(super::DiscordGuardPreferences::default()).expect("typed notices");
    guard["project_overrides"] = json!({});
    let wiki_language = if language == Some(InterfaceLanguage::Ko) { "ko" } else { "en" };
    let mut answers = json!({"schema_version":2,"wiki":{"enabled":true,"language":wiki_language,"backend":"markdown"},
        "profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},"selected_hosts":["codex"],
        "skills":{"mode":"all"},"update_check":{"enabled":false},"judge_invocation":"explicit","usage_guard":guard,"preference_decisions":{}});
    if let Some(language) = language { answers["interface_language"] = json!(language); }
    answers
}

pub(super) fn describe_section(arguments: &[String]) -> Result<ActionResult, SetupError> {
    let mut scope = None;
    let mut section = None;
    let mut question = None;
    let mut root = None;
    let mut output = None;
    let mut describe = false;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--describe" && !describe {
            describe = true;
            index += 1;
            continue;
        }
        let slot = match option {
            "--scope" => &mut scope,
            "--section" => &mut section,
            "--question" => &mut question,
            "--user-root" => &mut root,
            "--output" => &mut output,
            _ => return Err(SetupError::Input("unknown describe option".to_owned())),
        };
        let value = arguments
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| SetupError::Input("missing describe option value".to_owned()))?;
        if slot.replace(value.clone()).is_some() {
            return Err(SetupError::Input("duplicate describe option".to_owned()));
        }
        index += 2;
    }
    if !describe || output.as_deref() != Some("json") {
        return Err(SetupError::Input(
            "section description requires --describe --output json".to_owned(),
        ));
    }
    let scope = match scope.as_deref() {
        Some("user") => PreferenceScope::User,
        Some("project") => PreferenceScope::Project,
        _ => {
            return Err(SetupError::Input(
                "choose the user or project scope".to_owned(),
            ))
        }
    };
    let section = question_section(scope, section, question.as_deref())?;
    let user_root = root.map_or_else(super::resolve_user_root, |path| Ok(path.into()))?;
    let root_cap =
        crate::user_install::open_user_root_for_setup(&user_root).map_err(SetupError::Conflict)?;
    let config = super::load_operational_config(&root_cap)?;
    let mut current = config
        .as_ref()
        .filter(|_| scope == PreferenceScope::User)
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| SetupError::Internal(format!("cannot inspect preferences: {error}")))?;
    if question.is_some() && scope == PreferenceScope::User {
        if let Some(config) = &config {
            if let Some(migration) = super::migration::inspect(&user_root, config)? {
                current = Some(serde_json::to_value(migration.candidate).map_err(|e| SetupError::Internal(e.to_string()))?);
            }
        }
    }
    if scope == PreferenceScope::User && section == "vector" {
        let (_, feature) = super::load_feature_answers(&root_cap)?;
        if let Some(answer) = feature.vector_search {
            let values = current.get_or_insert_with(|| json!({}));
            values["operation"] = json!({"vector_search":answer == super::VectorFeatureAnswer::Yes});
            values["preference_decisions"]["/operation/vector_search"] = json!(feature.decision);
        }
    }
    let mut questions = section_data(scope, &section, current.as_ref())?;
    let korean = config
        .as_ref()
        .is_some_and(|c| c.interface_language == InterfaceLanguage::Ko);
    if let Some(id) = &question {
        questions = summary::one_question(questions, id, korean)?;
    }
    Ok(ActionResult {
        schema_version: 1,
        action: if question.is_some() {"DescribeHiveSetupQuestion"} else {"DescribeHiveSetupSection"},
        status: "success",
        exit_code: 0,
        code: if question.is_some() {"hive.setup-question-described"} else {"hive.setup-section-described"},
        message: if korean {
            "요청한 설정 질문 목록"
        } else {
            "Requested setup questions"
        }
        .to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "setup-questions",
            locator: "harness/setup/questions.yml".to_owned(),
            digest: hive_core::sha256_digest(QUESTIONS.as_bytes()),
        }],
        next_action: None,
        data: Some(json!({"section":section,"scope":scope,"questions":questions})),
    })
}

fn question_section(scope: PreferenceScope, section: Option<String>, question: Option<&str>) -> Result<String, SetupError> {
    match (section,question) {
        (Some(section),None)=>Ok(section),
        (None,Some(id))=>catalog()?.questions.into_iter().find(|q|q.scope==scope && q.id==id)
            .map(|q|q.section).ok_or_else(||SetupError::Input("unknown setup question".into())),
        _=>Err(SetupError::Input("choose one question or one section".into())),
    }
}

fn section_data(
    scope: PreferenceScope,
    section: &str,
    current: Option<&Value>,
) -> Result<Value, SetupError> {
    let catalog = catalog()?;
    let questions = catalog.section(scope, section);
    if questions.is_empty() {
        return Err(SetupError::Input(
            "unknown setup question section".to_owned(),
        ));
    }
    let values = questions
        .iter()
        .map(|question| {
            let stored = current.and_then(|current| current.pointer(&question.key));
            let source = current
                .and_then(|current| current["preference_decisions"].get(&question.key))
                .and_then(|record| record.get("origin"))
                .cloned()
                .unwrap_or_else(|| {
                    json!(if stored.is_some() {
                        DecisionOrigin::LegacyUnknown
                    } else {
                        DecisionOrigin::Default
                    })
                });
            json!({"question":question,"current":stored,"source":source,
            "applicable":current.map(|current| question.applicable(current))})
        })
        .collect::<Vec<_>>();
    Ok(json!(values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_questions_are_valid_and_section_output_is_bounded() {
        let catalog = catalog().unwrap();
        assert!(catalog.questions.len() > 30);
        let usage = section_data(PreferenceScope::User, "usage", None).unwrap();
        assert!(serde_json::to_vec(&usage).unwrap().len() < 16_384);
        assert!(usage
            .as_array()
            .unwrap()
            .iter()
            .all(|q| q["question"]["section"] == "usage"));
        assert!(section_data(PreferenceScope::User, "missing", None).is_err());
    }

    #[test]
    fn description_keeps_unanswered_defaults_separate_from_stored_choices() {
        let current = json!({"usage_guard":{"enforcement_enabled":true},
            "preference_decisions":{"/usage_guard/enforcement_enabled":{"origin":"explicit"}}});
        let data = section_data(PreferenceScope::User, "usage", Some(&current)).unwrap();
        let guard = data
            .as_array()
            .unwrap()
            .iter()
            .find(|q| q["question"]["id"] == "usage-enforcement")
            .unwrap();
        assert_eq!(guard["current"], true);
        assert_eq!(guard["question"]["default"], false);
        assert_eq!(guard["source"], "explicit");
    }

    #[test]
    fn described_section_obeys_the_machine_result_contract() {
        let root = tempfile::tempdir().unwrap();
        let args = vec![
            "--scope".to_owned(),
            "user".to_owned(),
            "--describe".to_owned(),
            "--section".to_owned(),
            "usage".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
            "--user-root".to_owned(),
            root.path().display().to_string(),
        ];
        let result = describe_section(&args).unwrap();
        hive_core::validate_json_schema(
            include_str!("../../../../../schemas/action-result.schema.json"),
            &serde_json::to_value(result).unwrap(),
            "section result",
        )
        .unwrap();
        assert!(!root.path().join(".hive").exists());
    }
}
