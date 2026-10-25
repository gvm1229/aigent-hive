//! Operation-scoped consent; a stored monitoring preference alone never starts a controller.
use super::{
    catalog,
    interactive::{persist, Conversation},
    terminal::Terminal,
};
use crate::user_setup::{
    self,
    progress_store::{ProgressStore, SetupProgress},
    Arguments, SetupError,
};
use hive_core::preferences::{
    declaration_digest, PreferenceDecision, PreferenceInput, PreferenceScope,
};
use hive_core::setup::AnswerInput;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::user_setup) struct Approval {
    native_preview: String,
    selection_digest: String,
    confirmation_digest: String,
    channel: PreferenceInput,
}

fn wanted(answers: &Value) -> bool {
    answers["usage_guard"]["enforcement_enabled"] == true
        && answers["usage_guard"]["midway"]["enabled"] == true
}

fn previously_wanted(before: Option<&[u8]>) -> Result<bool, SetupError> {
    before
        .map(user_setup::parse_and_validate_installed_config)
        .transpose()
        .map(|config| {
            config
                .and_then(|c| c.usage_guard.policy)
                .is_some_and(|p| p.enforcement_enabled && p.midway.enabled)
        })
}

fn validate_existing_selection(preview: &Value, before: Option<&[u8]>) -> Result<(), SetupError> {
    let previous = before
        .map(user_setup::parse_and_validate_installed_config)
        .transpose()?;
    if enabled(preview)
        && previous.is_some_and(|config| {
            config.schema_version == 2
                && config
                    .usage_guard
                    .policy
                    .is_some_and(|p| !p.enforcement_enabled || !p.midway.enabled)
        })
    {
        return Err(SetupError::Conflict("existing monitor registration differs from the disabled policy; use hive usage monitor disable for this user root before requesting a new activation".into()));
    }
    Ok(())
}

fn selection(answers: &Value) -> Result<String, SetupError> {
    let mut choices = answers.clone();
    choices
        .as_object_mut()
        .ok_or_else(|| SetupError::Input("invalid monitor settings".into()))?
        .remove("preference_decisions");
    declaration_digest(&choices).map_err(SetupError::Internal)
}

fn native(root: &Path) -> Result<Value, SetupError> {
    crate::usage_monitor::setup_activation_preview(root)
        .map_err(|e| SetupError::Conflict(e.message().into()))
}

fn approval(
    preview: &Value,
    answers: &Value,
    channel: PreferenceInput,
) -> Result<Approval, SetupError> {
    let digest = preview["preview_digest"]
        .as_str()
        .filter(|s| crate::is_sha256_digest(s))
        .ok_or_else(|| SetupError::Verification("monitor preview has no valid scope".into()))?
        .to_owned();
    let selection_digest = selection(answers)?;
    let confirmation_digest =
        declaration_digest(&json!({"monitor":digest,"settings":selection_digest}))
            .map_err(SetupError::Internal)?;
    Ok(Approval {
        native_preview: digest,
        selection_digest,
        confirmation_digest,
        channel,
    })
}

#[cfg(test)]
pub(in crate::user_setup) fn fixture_approval(answers:&Value)->Approval {
    approval(&json!({"preview_digest":format!("sha256:{}","a".repeat(64))}),answers,PreferenceInput::Terminal).unwrap()
}

fn enabled(preview: &Value) -> bool {
    preview["activation_binding"]["previous"]["enabled"] == true
}
fn supported(preview: &Value, answers: &Value) -> bool {
    preview["supports_current_os"] == true
        && answers["selected_hosts"]
            .as_array()
            .is_some_and(|hosts| hosts.iter().any(|host| host == "codex"))
}

pub(in crate::user_setup) fn validate(
    scope: PreferenceScope,
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    if let Some(approved) = &progress.monitor_start {
        if scope != PreferenceScope::User
            || !wanted(&progress.answers)
            || progress.monitor_stop.is_some()
            || approved.channel == PreferenceInput::Migration
            || !crate::is_sha256_digest(&approved.native_preview)
            || selection(&progress.answers)? != approved.selection_digest
            || declaration_digest(
                &json!({"monitor":approved.native_preview,"settings":approved.selection_digest}),
            )
            .map_err(SetupError::Internal)?
                != approved.confirmation_digest
        {
            return Err(SetupError::Conflict(
                "monitor activation no longer matches the confirmed settings".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn collect<R: BufRead, W: Write>(
    root: &Path,
    before: Option<&[u8]>,
    store: &ProgressStore,
    progress: &mut Conversation,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    collect_using(before, store, progress, terminal, || native(root))
}

fn collect_using<R: BufRead, W: Write>(
    before: Option<&[u8]>,
    store: &ProgressStore,
    progress: &mut Conversation,
    terminal: &mut Terminal<R, W>,
    preview: impl Fn() -> Result<Value, SetupError>,
) -> Result<bool, SetupError> {
    if !wanted(&progress.answers) {
        progress.monitor_start = None;
        return Ok(true);
    }
    progress.monitor_stop = None;
    let current = preview()?;
    validate_existing_selection(&current, before)?;
    if enabled(&current) {
        progress.monitor_start = None;
        return Ok(true);
    }
    let reviewing = progress
        .selected_section
        .as_deref()
        .is_none_or(|s| s == "usage");
    if previously_wanted(before)? && !reviewing {
        progress.monitor_start = None;
        return Ok(true);
    }
    let desired = approval(&current, &progress.answers, PreferenceInput::Terminal)?;
    if progress.monitor_start.as_ref() == Some(&desired) {
        return Ok(true);
    }
    progress.monitor_start = None;
    let question = catalog()?
        .questions
        .into_iter()
        .find(|q| q.id == "user-monitor-activation")
        .ok_or_else(|| SetupError::Internal("missing monitor activation question".into()))?;
    progress.step.clone_from(&question.id);
    persist(store, progress)?;
    loop {
        if !supported(&current, &progress.answers) {
            terminal.say("현재 환경에서는 새 감시 등록을 지원하지 않습니다. 아니요를 고르면 감시를 끄고 다른 설정을 계속할 수 있습니다.",
                "New monitoring is unsupported here. Choose no to disable monitoring and continue other settings.")?;
        }
        let AnswerInput::Answered { value, origin } = terminal.ask(&question, None)? else {
            return Ok(false);
        };
        if value == json!(false) {
            progress.answers["usage_guard"]["midway"]["enabled"] = json!(false);
            let mut decision = PreferenceDecision::new(
                &json!(false),
                origin,
                PreferenceInput::Terminal,
                PreferenceScope::User,
                env!("CARGO_PKG_VERSION"),
                Some(user_setup::feature_now_unix()?),
            )
            .map_err(SetupError::Internal)?;
            decision.question_id = Some(question.id.clone());
            decision.question_revision = Some(question.revision);
            progress.answers["preference_decisions"]["/usage_guard/midway/enabled"] =
                json!(decision);
        } else {
            if !supported(&current, &progress.answers) {
                continue;
            }
            if approval(&preview()?, &progress.answers, PreferenceInput::Terminal)? != desired {
                return Err(SetupError::Conflict(
                    "monitor scope changed during confirmation; review saved answers".into(),
                ));
            }
            progress.monitor_start = Some(desired);
        }
        progress
            .answered_questions
            .insert(question.id.clone(), question.revision);
        progress
            .question_bindings
            .insert(question.id.clone(), (&question).into());
        persist(store, progress)?;
        return Ok(true);
    }
}

pub(super) fn describe(
    arguments: &Arguments,
    config: &user_setup::UserSetupConfig,
    result: &mut crate::ActionResult,
) -> Result<(), SetupError> {
    let answers = serde_json::to_value(config).map_err(|e| SetupError::Internal(e.to_string()))?;
    if !wanted(&answers) {
        return Ok(());
    }
    let preview = native(&arguments.user_root)?;
    if !enabled(&preview) {
        let approved = approval(&preview, &answers, PreferenceInput::AnswersFile)?;
        result.data.get_or_insert_with(|| json!({}))["monitor_activation"] = json!({
            "confirmation_digest":approved.confirmation_digest,"platform_eligible":supported(&preview,&answers),"runtime_verification":"not-performed",
            "scope":preview["scope"],"login_registration":true,"requires_separate_confirmation":true});
    }
    Ok(())
}

pub(super) fn bind_file(
    root: &Path,
    before: Option<&[u8]>,
    progress: &mut SetupProgress,
    requested: Option<&str>,
    channel: PreferenceInput,
) -> Result<(), SetupError> {
    bind_file_using(before, progress, requested, channel, || native(root))
}

fn bind_file_using(
    before: Option<&[u8]>,
    progress: &mut SetupProgress,
    requested: Option<&str>,
    channel: PreferenceInput,
    preview: impl FnOnce() -> Result<Value, SetupError>,
) -> Result<(), SetupError> {
    if !wanted(&progress.answers) {
        if requested.is_some() {
            return Err(SetupError::Input(
                "disabled monitoring cannot approve activation".into(),
            ));
        }
        progress.monitor_start = None;
        return Ok(());
    }
    let preview = preview()?;
    validate_existing_selection(&preview, before)?;
    if enabled(&preview) {
        if requested.is_some() {
            return Err(SetupError::Input(
                "monitor is already selected; no new activation is needed".into(),
            ));
        }
        progress.monitor_start = None;
        return Ok(());
    }
    if requested.is_none() && progress.monitor_start.is_none() && previously_wanted(before)? {
        return Ok(());
    }
    if !supported(&preview, &progress.answers) {
        return Err(SetupError::Input(
            "monitor activation requires supported Windows Codex setup".into(),
        ));
    }
    let mut desired = approval(&preview, &progress.answers, channel)?;
    if requested.is_none() {
        if let Some(saved) = &progress.monitor_start {
            desired.channel = saved.channel;
        }
    }
    let selected = requested.or(progress
        .monitor_start
        .as_ref()
        .map(|a| a.confirmation_digest.as_str()));
    if selected != Some(desired.confirmation_digest.as_str()) {
        return Err(SetupError::Input("monitor activation requires --monitor-confirm <digest> from --dry-run or interactive setup".into()));
    }
    progress.monitor_start = Some(desired);
    Ok(())
}

pub(super) fn preflight(root: &Path, progress: &SetupProgress) -> Result<(), SetupError> {
    validate(PreferenceScope::User, progress)?;
    if let Some(approved) = &progress.monitor_start {
        if native(root)?["preview_digest"] != approved.native_preview {
            return Err(SetupError::Conflict(
                "monitor scope changed after the setup preview".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn check_file_replay(
    progress: &SetupProgress,
    requested: Option<&str>,
) -> Result<(), SetupError> {
    if requested.is_some_and(|digest| {
        progress
            .monitor_start
            .as_ref()
            .is_none_or(|saved| saved.confirmation_digest != digest)
    }) {
        return Err(SetupError::Conflict(
            "monitor approval differs from the pending setup".into(),
        ));
    }
    Ok(())
}

pub(super) fn finish(
    root: &Path,
    answers: &[u8],
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    finish_using(answers, progress, |digest| {
        crate::usage_monitor::activate_after_setup(root, digest)
            .map_err(|e| SetupError::Verification(e.message().into()))
    })
}
fn finish_using(
    answers: &[u8],
    progress: &SetupProgress,
    activate: impl FnOnce(&str) -> Result<Value, SetupError>,
) -> Result<(), SetupError> {
    let Some(approved) = &progress.monitor_start else {
        return Ok(());
    };
    validate(PreferenceScope::User, progress)?;
    let actual: Value = serde_json::from_slice(answers)
        .map_err(|_| SetupError::Input("invalid monitor setup answers".into()))?;
    if selection(&actual)? != approved.selection_digest {
        return Err(SetupError::Conflict(
            "monitor setup selection changed".into(),
        ));
    }
    let result = activate(&approved.native_preview)?;
    if result["enabled"] != true || result["activation"] != "verified" {
        return Err(SetupError::Verification(
            "monitor controller readiness was not verified".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, io::Cursor};
    use user_setup::InterfaceLanguage;

    fn fixture() -> (tempfile::TempDir, ProgressStore, Conversation) {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut answers = super::super::default_answers(Some(InterfaceLanguage::Ko));
        answers["usage_guard"]["enforcement_enabled"] = json!(true);
        answers["usage_guard"]["midway"]["enabled"] = json!(true);
        let record = store.new_progress("usage-monitor".into(), answers).unwrap();
        let conversation = Conversation {
            record,
            snapshot: store.load().unwrap(),
            fresh: true,
        };
        (root, store, conversation)
    }
    fn preview(supported: bool) -> Value {
        json!({"preview_digest":format!("sha256:{}","a".repeat(64)),"supports_current_os":supported,
            "activation_binding":{"previous":{"enabled":false}},"scope":"current-user Codex"})
    }

    #[test]
    fn no_disables_the_choice_and_eof_preserves_it_in_both_languages() {
        for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
            for input in ["no\n", "\n", ""] {
                let (root, store, mut progress) = fixture();
                let mut terminal = Terminal {
                    reader: Cursor::new(input),
                    writer: Vec::new(),
                    language,
                };
                let done = collect_using(None, &store, &mut progress, &mut terminal, || {
                    Ok(preview(true))
                })
                .unwrap();
                assert_eq!(done, !input.is_empty());
                assert_eq!(wanted(&progress.answers), input.is_empty());
                assert!(progress.monitor_start.is_none());
                assert!(!root.path().join(".hive/config/user-setup.yml").exists());
                if done {
                    let origin = if input == "\n" {
                        "accepted-default"
                    } else {
                        "explicit"
                    };
                    assert_eq!(
                        progress.answers["preference_decisions"]["/usage_guard/midway/enabled"]
                            ["origin"],
                        origin
                    );
                    assert_eq!(progress.answers["usage_guard"]["enforcement_enabled"], true);
                }
            }
        }
    }

    #[test]
    fn yes_is_scope_bound_and_cached_without_repeating_the_question() {
        let (_root, store, mut progress) = fixture();
        let mut terminal = Terminal {
            reader: Cursor::new("yes\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(
            collect_using(None, &store, &mut progress, &mut terminal, || Ok(preview(
                true
            )))
            .unwrap()
        );
        let saved = progress.monitor_start.clone().unwrap();
        terminal.reader = Cursor::new("");
        terminal.writer.clear();
        assert!(
            collect_using(None, &store, &mut progress, &mut terminal, || Ok(preview(
                true
            )))
            .unwrap()
        );
        assert!(terminal.writer.is_empty());
        assert_eq!(progress.monitor_start, Some(saved.clone()));
        finish_using(
            &serde_json::to_vec(&progress.answers).unwrap(),
            &progress,
            |digest| {
                assert_eq!(digest, saved.native_preview);
                Ok(json!({"enabled":true,"activation":"verified"}))
            },
        )
        .unwrap();
        assert!(finish_using(
            &serde_json::to_vec(&progress.answers).unwrap(),
            &progress,
            |_| { Ok(json!({"enabled":true,"activation":"awaiting-first-observation"})) }
        )
        .is_err());
        progress.answers["usage_guard"]["stop_remaining_percent"] = json!(35);
        assert!(finish_using(
            &serde_json::to_vec(&progress.answers).unwrap(),
            &progress,
            |_| panic!("changed selection activated")
        )
        .is_err());
    }

    #[test]
    fn unsupported_yes_reasks_only_that_question_and_never_activates() {
        let (_root, store, mut progress) = fixture();
        let mut terminal = Terminal {
            reader: Cursor::new("yes\nno\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(
            collect_using(None, &store, &mut progress, &mut terminal, || Ok(preview(
                false
            )))
            .unwrap()
        );
        assert!(!wanted(&progress.answers));
        assert!(progress.monitor_start.is_none());
        assert!(String::from_utf8(terminal.writer)
            .unwrap()
            .contains("unsupported"));
    }

    #[test]
    fn a_scope_change_during_yes_never_saves_an_approval() {
        let (_root, store, mut progress) = fixture();
        let calls = Cell::new(0);
        let mut terminal = Terminal {
            reader: Cursor::new("yes\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(
            collect_using(None, &store, &mut progress, &mut terminal, || {
                let mut value = preview(true);
                if calls.replace(calls.get() + 1) > 0 {
                    value["preview_digest"] = json!(format!("sha256:{}", "b".repeat(64)));
                }
                Ok(value)
            })
            .is_err()
        );
        assert!(progress.monitor_start.is_none());
    }

    #[test]
    fn file_confirmation_is_explicit_and_timestamp_metadata_does_not_change_its_scope() {
        let (_root, _store, mut progress) = fixture();
        let expected = approval(
            &preview(true),
            &progress.answers,
            PreferenceInput::AnswersFile,
        )
        .unwrap();
        assert!(
            bind_file_using(None, &mut progress, None, PreferenceInput::Agent, || Ok(
                preview(true)
            ))
            .is_err()
        );
        progress.answers["preference_decisions"] =
            json!({"/usage_guard/midway/enabled":{"decided_at":123}});
        bind_file_using(
            None,
            &mut progress,
            Some(&expected.confirmation_digest),
            PreferenceInput::Agent,
            || Ok(preview(true)),
        )
        .unwrap();
        assert_eq!(
            progress.monitor_start.as_ref().unwrap().channel,
            PreferenceInput::Agent
        );
        validate(PreferenceScope::User, &progress).unwrap();
        assert!(validate(PreferenceScope::Project, &progress).is_err());
        assert!(check_file_replay(&progress, Some(&format!("sha256:{}", "c".repeat(64)))).is_err());
    }

    #[test]
    fn unrelated_terminal_and_file_changes_keep_a_manually_disabled_runtime_off() {
        let (_root, store, mut progress) = fixture();
        let before = serde_json::to_vec(&progress.answers).unwrap();
        progress.selected_section = Some("persona".into());
        let mut terminal = Terminal {
            reader: Cursor::new(""),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(
            collect_using(Some(&before), &store, &mut progress, &mut terminal, || Ok(
                preview(true)
            ))
            .unwrap()
        );
        assert!(terminal.writer.is_empty() && progress.monitor_start.is_none());
        bind_file_using(
            Some(&before),
            &mut progress,
            None,
            PreferenceInput::Agent,
            || Ok(preview(true)),
        )
        .unwrap();
        assert!(progress.monitor_start.is_none());
        finish_using(&before, &progress, |_| {
            panic!("unrelated setup activated runtime")
        })
        .unwrap();
    }

    #[test]
    fn file_resume_keeps_the_original_terminal_consent_channel() {
        let (_root, _store, mut progress) = fixture();
        progress.monitor_start =
            Some(approval(&preview(true), &progress.answers, PreferenceInput::Terminal).unwrap());
        bind_file_using(None, &mut progress, None, PreferenceInput::Agent, || {
            Ok(preview(true))
        })
        .unwrap();
        assert_eq!(
            progress.monitor_start.as_ref().unwrap().channel,
            PreferenceInput::Terminal
        );
    }

    #[test]
    fn a_disabled_v2_policy_never_reactivates_a_stale_enabled_registration() {
        let (_root, store, mut progress) = fixture();
        let mut previous = progress.answers.clone();
        previous["usage_guard"]["enforcement_enabled"] = json!(false);
        let previous = serde_json::to_vec(&previous).unwrap();
        let mut stale = preview(true);
        stale["activation_binding"]["previous"]["enabled"] = json!(true);
        let mut terminal = Terminal {
            reader: Cursor::new("yes\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(collect_using(
            Some(&previous),
            &store,
            &mut progress,
            &mut terminal,
            || Ok(stale.clone())
        )
        .is_err());
        assert!(terminal.writer.is_empty() && progress.monitor_start.is_none());
        assert!(bind_file_using(
            Some(&previous),
            &mut progress,
            None,
            PreferenceInput::Agent,
            || Ok(stale)
        )
        .is_err());
    }
}
