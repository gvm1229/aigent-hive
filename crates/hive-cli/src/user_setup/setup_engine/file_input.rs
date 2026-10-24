//! Answer transport records provenance without claiming that an assistant is the user.
use crate::user_setup::{self, Arguments, SetupBase, SetupError, SetupMode, UserSetupConfig};
use crate::ActionResult;
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope,
};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

#[derive(Clone)]
pub(in crate::user_setup) struct FileOptions {
    pub channel: PreferenceInput,
    pub quick: bool,
    pub index_rebuild_scope: Option<String>,
}

impl Default for FileOptions {
    fn default() -> Self {
        Self {
            channel: PreferenceInput::AnswersFile,
            quick: false,
            index_rebuild_scope: None,
        }
    }
}

pub(in crate::user_setup) fn file_options(
    arguments: &[String],
) -> Result<(Vec<String>, FileOptions), SetupError> {
    let mut options = FileOptions {
        quick: arguments.iter().any(|arg| arg == "--quick-answers"),
        ..FileOptions::default()
    };
    let mut filtered = Vec::new();
    let mut seen = false;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--input-channel" {
            if seen {
                return Err(SetupError::Input("duplicate input channel".into()));
            }
            options.channel = match arguments.get(index + 1).map(String::as_str) {
                Some("answers-file") => PreferenceInput::AnswersFile,
                Some("agent") => PreferenceInput::Agent,
                _ => {
                    return Err(SetupError::Input(
                        "answer input channel must be answers-file or agent".into(),
                    ))
                }
            };
            seen = true;
            index += 2;
        } else if arguments[index] == "--index-rebuild-scope" {
            let digest = arguments.get(index + 1).filter(|value|
                value.len() == 71 && value.starts_with("sha256:")
                    && value[7..].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
                .ok_or_else(|| SetupError::Input("index rebuild scope requires a SHA-256 digest".into()))?;
            if options.index_rebuild_scope.replace(digest.clone()).is_some() {
                return Err(SetupError::Input("duplicate index rebuild scope".into()));
            }
            index += 2;
        } else {
            filtered.push(arguments[index].clone());
            index += 1;
        }
    }
    Ok((filtered, options))
}

pub(in crate::user_setup) fn execute(
    arguments: &Arguments,
    options: &FileOptions,
) -> Result<ActionResult, SetupError> {
    if options.index_rebuild_scope.is_some() && arguments.mode != SetupMode::Apply {
        return Err(SetupError::Input("index rebuild approval requires --apply".into()));
    }
    let input =
        user_setup::read_bounded_regular(&arguments.answers, user_setup::MAX_ANSWERS_BYTES)?;
    let existing = crate::user_install::read_user_setup_file(
        &arguments.root_cap,
        Path::new(user_setup::USER_SETUP_RELATIVE),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let current = existing
        .as_deref()
        .map(user_setup::parse_and_validate_installed_config)
        .transpose()
        .map_err(|error| SetupError::Conflict(format!("installed user setup is invalid: {}", error.message())))?;
    let allow_legacy = existing.as_deref() == Some(input.as_slice())
        && user_setup::has_legacy_recommended_skill_selection(&input)?;
    let mut config = if allow_legacy {
        user_setup::parse_and_validate_installed_config(&input)?
    } else {
        user_setup::parse_and_validate_config(&input)?
    };
    let mut supplied: Value = serde_yaml::from_slice(&input)
        .map_err(|_| SetupError::Input("invalid answer document".into()))?;
    user_setup::migrate_legacy_skill_names(&mut supplied)?;
    user_setup::migrate_legacy_single_profile(&mut supplied);
    let legacy_input = (config.schema_version == 1).then(|| config.clone());
    let normalized = super::file_policy::normalize(
        &arguments.user_root, &mut config, &mut supplied, current.as_ref(), options.quick,
        arguments.mode,
    )?;
    if arguments.mode == SetupMode::Validate {
        // Validation is not a new answer and must not invent provenance for an old file.
        config.preference_decisions = current
            .as_ref()
            .map(|config| config.preference_decisions.clone())
            .unwrap_or_default();
    } else {
        annotate(
            &mut config,
            &supplied,
            normalized.baseline.as_ref(),
            options,
            if arguments.mode == SetupMode::Apply {
                Some(user_setup::feature_now_unix()?)
            } else {
                None
            },
        )?;
    }
    let bytes =
        serde_json::to_vec(&config).map_err(|error| SetupError::Internal(error.to_string()))?;
    if arguments.mode == SetupMode::Apply {
        return super::file_apply::apply(arguments, &config, &bytes, existing.as_deref(), legacy_input.as_ref(), normalized.migration, options.index_rebuild_scope.as_deref());
    }
    let mut result = user_setup::execute_answers(arguments, &bytes, SetupBase::Expected(existing.as_deref()))?;
    if arguments.mode == SetupMode::DryRun {
        super::index_rebuild::describe(arguments, &config, &mut result)?;
    }
    Ok(result)
}

fn declarations(config: &UserSetupConfig) -> Result<BTreeMap<String, Value>, SetupError> {
    let mut value =
        serde_json::to_value(config).map_err(|error| SetupError::Internal(error.to_string()))?;
    let object = value.as_object_mut().expect("typed user settings");
    object.remove("schema_version");
    object.remove("preference_decisions");
    let mut declarations = BTreeMap::new();
    collect("", &value, &mut declarations);
    for question in super::catalog()?
        .questions
        .into_iter()
        .filter(|q| q.scope == PreferenceScope::User)
    {
        if let Some(declaration) = value.pointer(&question.key) {
            let prefix = format!("{}/", question.key);
            declarations.retain(|key, _| !key.starts_with(&prefix));
            declarations.insert(question.key, declaration.clone());
        }
    }
    Ok(declarations)
}

fn collect(pointer: &str, value: &Value, output: &mut BTreeMap<String, Value>) {
    if let Some(object) = value.as_object().filter(|object| !object.is_empty()) {
        for (key, value) in object {
            collect(
                &format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                value,
                output,
            );
        }
    } else {
        output.insert(pointer.into(), value.clone());
    }
}

fn annotate(
    config: &mut UserSetupConfig,
    supplied: &Value,
    current: Option<&UserSetupConfig>,
    options: &FileOptions,
    answered_at: Option<u64>,
) -> Result<(), SetupError> {
    let defaults = super::default_answers(Some(config.interface_language));
    let values = declarations(config)?;
    let mut choices: BTreeMap<_, _> = current.into_iter().flat_map(|c| c.preference_decisions.iter())
        .filter(|(key, decision)| !values.contains_key(*key) && decision.matches_declaration(&Value::Null).unwrap_or(false))
        .map(|(key, decision)| (key.clone(), decision.clone())).collect();
    for (key, value) in values {
        let provided = supplied.pointer(&key).is_some();
        if let Some(prior) = current
            .and_then(|c| c.preference_decisions.get(&key))
            .filter(|d| d.matches_declaration(&value).unwrap_or(false))
            .filter(|d| !(provided && d.input_channel == PreferenceInput::Migration
                && matches!(d.origin, DecisionOrigin::LegacyUnknown | DecisionOrigin::Default)))
        {
            choices.insert(key, prior.clone());
            continue;
        }
        let origin = if !provided {
            DecisionOrigin::Default
        } else if options.quick
            && key != "/interface_language"
            && defaults.pointer(&key) == Some(&value)
        {
            DecisionOrigin::AcceptedDefault
        } else if options.channel == PreferenceInput::Agent {
            DecisionOrigin::ExternalChange
        } else {
            DecisionOrigin::Explicit
        };
        let time = if matches!(
            origin,
            DecisionOrigin::Explicit | DecisionOrigin::AcceptedDefault
        ) {
            answered_at
        } else {
            None
        };
        let choice = PreferenceDecision::new(
            &value,
            origin,
            options.channel,
            PreferenceScope::User,
            env!("CARGO_PKG_VERSION"),
            time,
        )
        .map_err(SetupError::Input)?;
        choices.insert(key, choice);
    }
    config.preference_decisions = choices;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::InterfaceLanguage;
    use serde_json::json;

    #[test]
    fn index_scope_option_requires_one_exact_digest_and_is_not_implied_by_apply() {
        let args = ["--apply".into(), "--answers".into(), "input.yml".into()];
        assert!(file_options(&args).unwrap().1.index_rebuild_scope.is_none());
        assert!(file_options(&["--index-rebuild-scope".into(), "yes".into()]).is_err());
        let digest = format!("sha256:{}", "a".repeat(64));
        let args = ["--apply".into(), "--index-rebuild-scope".into(), digest.clone()];
        let (filtered, options) = file_options(&args).unwrap();
        assert_eq!(filtered, ["--apply"]);
        assert_eq!(options.index_rebuild_scope.as_deref(), Some(digest.as_str()));
        let doubled = [args.as_slice(), &args[1..]].concat();
        assert!(file_options(&doubled).is_err());
    }

    fn input() -> Value {
        super::super::default_answers(Some(InterfaceLanguage::Ko))
    }
    fn parse(value: &Value) -> UserSetupConfig {
        user_setup::parse_and_validate_config(&serde_json::to_vec(value).unwrap()).unwrap()
    }

    #[test]
    fn supplied_v2_choices_replace_unknown_migration_origins_even_when_values_match() {
        let root = tempfile::tempdir().unwrap();
        let mut old = input();
        old["schema_version"] = json!(1);
        old["usage_guard"] = json!({"enabled":false,"stop_remaining_percent":5});
        let current = parse(&old);
        let baseline = user_setup::migration::inspect(root.path(), &current).unwrap().unwrap().candidate;
        let supplied = serde_json::to_value(&baseline).unwrap();
        let mut config = baseline.clone();
        annotate(&mut config, &supplied, Some(&baseline), &FileOptions::default(), Some(42)).unwrap();
        let record = &config.preference_decisions["/usage_guard/enforcement_enabled"];
        assert_eq!(record.origin, DecisionOrigin::Explicit);
        assert_eq!(record.input_channel, PreferenceInput::AnswersFile);
        assert_eq!(record.decided_at, Some(42));
        assert!(user_setup::migration::pending_questions(&config).is_empty());
    }

    #[test]
    fn explicit_optional_removal_keeps_its_record_when_the_file_omits_that_value() {
        let mut supplied = input();
        let mut current = parse(&supplied);
        let removal = PreferenceDecision::new(&Value::Null, DecisionOrigin::Explicit,
            PreferenceInput::Terminal, PreferenceScope::User, "0.14.0", Some(42)).unwrap();
        current.preference_decisions.insert("/profile/description".into(), removal.clone());
        let mut config = parse(&supplied);
        annotate(&mut config, &supplied, Some(&current), &FileOptions::default(), Some(99)).unwrap();
        assert_eq!(config.preference_decisions["/profile/description"], removal);
        supplied["profile"]["description"] = json!("New details");
        let mut changed = parse(&supplied);
        annotate(&mut changed, &supplied, Some(&current), &FileOptions::default(), Some(99)).unwrap();
        assert_eq!(changed.preference_decisions["/profile/description"].input_channel, PreferenceInput::AnswersFile);
        assert_eq!(changed.preference_decisions["/profile/description"].decided_at, Some(99));
    }

    #[test]
    fn quick_bundle_direct_values_and_automatic_defaults_have_different_origins() {
        let mut value = input();
        value.as_object_mut().unwrap().remove("update_check");
        let mut config = parse(&value);
        annotate(
            &mut config,
            &value,
            None,
            &FileOptions {
                quick: true,
                ..FileOptions::default()
            },
            Some(42),
        )
        .unwrap();
        let choices = &config.preference_decisions;
        assert_eq!(
            choices["/interface_language"].origin,
            DecisionOrigin::Explicit
        );
        assert_eq!(
            choices["/interface_language"].input_channel,
            PreferenceInput::AnswersFile
        );
        assert_eq!(
            choices["/usage_guard/enforcement_enabled"].origin,
            DecisionOrigin::AcceptedDefault
        );
        assert_eq!(
            choices["/usage_guard/enforcement_enabled"].decided_at,
            Some(42)
        );
        assert_eq!(
            choices["/update_check/enabled"].origin,
            DecisionOrigin::Default
        );
        assert_eq!(choices["/update_check/enabled"].decided_at, None);
        assert_eq!(choices["/skills"].origin, DecisionOrigin::AcceptedDefault);
        assert!(!choices.contains_key("/skills/mode"));
        user_setup::parse_and_validate_config(&serde_json::to_vec(&config).unwrap()).unwrap();
    }

    #[test]
    fn agent_input_cannot_claim_a_terminal_answer_from_its_own_metadata() {
        let mut value = input();
        let forged = PreferenceDecision::new(
            &json!(false),
            DecisionOrigin::Explicit,
            PreferenceInput::Terminal,
            PreferenceScope::User,
            "0.14.0",
            Some(1),
        )
        .unwrap();
        value["preference_decisions"]["/usage_guard/enforcement_enabled"] = json!(forged);
        let mut config = parse(&value);
        annotate(
            &mut config,
            &value,
            None,
            &FileOptions {
                channel: PreferenceInput::Agent,
                quick: false,
                ..FileOptions::default()
            },
            Some(42),
        )
        .unwrap();
        let choice = &config.preference_decisions["/usage_guard/enforcement_enabled"];
        assert_eq!(choice.origin, DecisionOrigin::ExternalChange);
        assert_eq!(choice.input_channel, PreferenceInput::Agent);
        assert_eq!(choice.decided_at, None);
        assert_eq!(choice.question_id, None);
    }

    #[test]
    fn unchanged_choices_keep_their_original_record_across_repeated_requests() {
        let value = input();
        let mut original = parse(&value);
        annotate(
            &mut original,
            &value,
            None,
            &FileOptions::default(),
            Some(42),
        )
        .unwrap();
        let mut repeated = parse(&value);
        annotate(
            &mut repeated,
            &value,
            Some(&original),
            &FileOptions {
                channel: PreferenceInput::Agent,
                quick: true,
                ..FileOptions::default()
            },
            Some(99),
        )
        .unwrap();
        assert_eq!(repeated, original);
        let mut changed = value.clone();
        changed["persona"]["id"] = json!("friendly");
        let mut revised = parse(&changed);
        annotate(
            &mut revised,
            &changed,
            Some(&original),
            &FileOptions::default(),
            Some(99),
        )
        .unwrap();
        assert_eq!(
            revised.preference_decisions["/persona/id"].decided_at,
            Some(99)
        );
        assert_eq!(
            revised.preference_decisions["/interface_language"],
            original.preference_decisions["/interface_language"]
        );
    }

    #[test]
    fn file_input_channel_is_explicit_and_cannot_select_terminal_or_migration() {
        let args: Vec<_> = [
            "--quick-answers",
            "answers.yml",
            "--input-channel",
            "agent",
            "--apply",
        ]
        .map(str::to_owned)
        .into();
        let (filtered, options) = file_options(&args).unwrap();
        assert_eq!(filtered, ["--quick-answers", "answers.yml", "--apply"]);
        assert_eq!(options.channel, PreferenceInput::Agent);
        assert!(options.quick);
        for channel in ["terminal", "migration", "unknown"] {
            assert!(file_options(&["--input-channel".into(), channel.into()]).is_err());
        }
        assert!(file_options(&["--input-channel".into()]).is_err());
        assert!(file_options(&[
            "--input-channel".into(),
            "agent".into(),
            "--input-channel".into(),
            "agent".into()
        ])
        .is_err());
    }
}
