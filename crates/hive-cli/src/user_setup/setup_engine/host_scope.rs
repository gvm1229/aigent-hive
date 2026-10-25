//! The frozen updater validates the same host set after the new setup returns.
use crate::user_setup::{
    progress_store::{ProgressPhase, SetupProgress},
    SetupError,
};
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope,
};
use serde_json::json;
use std::collections::BTreeSet;

pub(super) fn announce<R, W: std::io::Write>(
    progress: &SetupProgress,
    terminal: &mut super::terminal::Terminal<R, W>,
) -> Result<(), SetupError> {
    if !progress.update_hosts.is_empty() {
        let message = if terminal.language == crate::user_setup::InterfaceLanguage::Ko {
            "앞서 선택한 앱에 갱신을 적용합니다:"
        } else {
            "This update keeps the apps selected earlier:"
        };
        writeln!(
            terminal.writer,
            "{message} {}",
            progress.update_hosts.join(", ")
        )
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    }
    Ok(())
}

fn valid(hosts: &[String]) -> bool {
    hosts.len() <= 3
        && hosts.iter().collect::<BTreeSet<_>>().len() == hosts.len()
        && hosts
            .iter()
            .all(|host| matches!(host.as_str(), "codex" | "claude" | "antigravity"))
}

fn contains(answers: &serde_json::Value, hosts: &[String]) -> bool {
    answers["selected_hosts"]
        .as_array()
        .is_some_and(|selected| hosts.iter().all(|host| selected.contains(&json!(host))))
}

pub(super) fn check_request(
    progress: &SetupProgress,
    requested: &[String],
) -> Result<(), SetupError> {
    if !valid(requested)
        || (!requested.is_empty()
            && !progress.update_hosts.is_empty()
            && requested.iter().collect::<BTreeSet<_>>()
                != progress.update_hosts.iter().collect::<BTreeSet<_>>())
    {
        return Err(SetupError::Conflict(
            "update host scope changed; saved choices were preserved".into(),
        ));
    }
    if !requested.is_empty()
        && matches!(
            progress.phase,
            ProgressPhase::Applying | ProgressPhase::RecoverableFailure
        )
        && progress.update_hosts.is_empty()
    {
        return Err(SetupError::Conflict(
            "prior apply intent is not bound to this update host scope".into(),
        ));
    }
    Ok(())
}

pub(super) fn bind_deactivations(
    preview: &crate::ActionResult,
    progress: &mut SetupProgress,
) -> Result<(), SetupError> {
    let plans = serde_json::from_value(
        preview
            .data
            .as_ref()
            .and_then(|data| data.get("host_deactivations"))
            .cloned()
            .unwrap_or_else(|| json!([])),
    )
    .map_err(|e| SetupError::Internal(e.to_string()))?;
    if progress.host_deactivations.is_empty() {
        progress.host_deactivations = plans;
    } else if progress.host_deactivations != plans {
        return Err(SetupError::Conflict(
            "host deactivation preview changed; review again".into(),
        ));
    }
    Ok(())
}

pub(super) fn display_deactivations<R, W: std::io::Write>(
    progress: &SetupProgress,
    terminal: &mut super::terminal::Terminal<R, W>,
) -> Result<(), SetupError> {
    for plan in &progress.host_deactivations {
        writeln!(
            terminal.writer,
            "{}: {}",
            if terminal.language == crate::user_setup::InterfaceLanguage::Ko {
                "Hive 연결과 안내만 해제, 설치 원문·사용자 자료 보존"
            } else {
                "Remove Hive registration and guidance; preserve installed sources and user data"
            },
            plan.host.as_str()
        )
        .map_err(|e| SetupError::Input(e.to_string()))?;
    }
    Ok(())
}

pub(in crate::user_setup) fn validate(progress: &SetupProgress) -> Result<(), SetupError> {
    let removed = progress
        .host_deactivations
        .iter()
        .map(|plan| plan.host.as_str())
        .collect::<BTreeSet<_>>();
    if removed.len() != progress.host_deactivations.len()
        || (!removed.is_empty()
            && (progress.binding.scope != PreferenceScope::User
                || (progress.phase != ProgressPhase::Complete
                    && progress.answers["selected_hosts"]
                        .as_array()
                        .is_none_or(|hosts| {
                            removed.iter().any(|host| hosts.contains(&json!(host)))
                        }))))
    {
        return Err(SetupError::Conflict(
            "host deactivation differs from the saved selection".into(),
        ));
    }
    if !valid(&progress.update_hosts)
        || (!progress.update_hosts.is_empty()
            && (progress.binding.scope != PreferenceScope::User
                || (progress.phase != ProgressPhase::Complete
                    && !contains(&progress.answers, &progress.update_hosts))))
    {
        return Err(SetupError::Conflict(
            "saved update host scope differs from setup choices".into(),
        ));
    }
    Ok(())
}

pub(super) fn bind(
    progress: &mut SetupProgress,
    requested: &[String],
    fresh: bool,
    new_operation: bool,
) -> Result<(), SetupError> {
    check_request(progress, requested)?;
    if requested.is_empty() || !progress.update_hosts.is_empty() {
        return validate(progress);
    }
    if fresh && new_operation {
        let value = json!(requested);
        // The old question revision and answer time are not available in its child protocol.
        let decision = PreferenceDecision::new(
            &value,
            DecisionOrigin::LegacyUnknown,
            PreferenceInput::Migration,
            PreferenceScope::User,
            env!("CARGO_PKG_VERSION"),
            None,
        )
        .map_err(SetupError::Internal)?;
        progress.answers["selected_hosts"] = value;
        progress.answers["preference_decisions"]["/selected_hosts"] = json!(decision);
    } else if !contains(&progress.answers, requested) {
        return Err(SetupError::Conflict(
            "saved app choices differ from the update request; review them before retrying".into(),
        ));
    }
    progress.update_hosts = requested.to_vec();
    validate(progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deactivation_scope_remains_valid_after_completed_answers_are_redacted() {
        let root = tempfile::tempdir().unwrap();
        let store =
            crate::user_setup::progress_store::ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress(
                "user-hosts".into(),
                json!({"selected_hosts":["antigravity"]}),
            )
            .unwrap();
        progress.host_deactivations = serde_json::from_value(json!([{
            "host":"codex","manifest_digest":format!("sha256:{}","a".repeat(64)),
            "before":{"host":"codex","state":{"marketplace":null,"plugin":null}},
            "guidance":".codex/AGENTS.md","guidance_before":"absent","guidance_after":"absent"
        }]))
        .unwrap();
        validate(&progress).unwrap();
        progress.answers = json!({});
        progress.phase = ProgressPhase::Applying;
        assert!(validate(&progress).is_err());
        progress.phase = ProgressPhase::Complete;
        validate(&progress).unwrap();
        progress
            .host_deactivations
            .push(progress.host_deactivations[0].clone());
        assert!(validate(&progress).is_err());
    }
    use crate::user_setup::progress_store::ProgressStore;

    #[test]
    fn old_nondefault_host_is_preserved_without_inventing_a_new_answer() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress(
                "user-language".into(),
                json!({
                    "selected_hosts":["codex"], "preference_decisions":{}
                }),
            )
            .unwrap();
        bind(&mut progress, &["antigravity".into()], true, true).unwrap();
        assert_eq!(progress.answers["selected_hosts"], json!(["antigravity"]));
        let decision = &progress.answers["preference_decisions"]["/selected_hosts"];
        assert_eq!(decision["origin"], "legacy-unknown");
        assert!(decision["decided_at"].is_null());
        assert!(decision["question_id"].is_null());
        store.save(&store.load().unwrap(), &progress).unwrap();
        let mut resumed = store.load().unwrap().progress.unwrap();
        bind(&mut resumed, &[], true, false).unwrap();
        assert_eq!(resumed, progress);
        assert!(bind(&mut resumed, &["codex".into()], true, false).is_err());
        resumed.answers["selected_hosts"] = json!(["codex"]);
        assert!(store.save(&store.load().unwrap(), &resumed).is_err());
    }

    #[test]
    fn saved_or_unfinished_choices_are_never_replaced_by_parent_arguments() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let progress = store
            .new_progress("user-hosts".into(), json!({"selected_hosts":["claude"]}))
            .unwrap();
        for (fresh, new_operation) in [(false, true), (true, false), (false, false)] {
            let mut changed = progress.clone();
            assert!(bind(&mut changed, &["codex".into()], fresh, new_operation).is_err());
            assert_eq!(changed, progress);
        }
        let mut preserved = progress.clone();
        bind(&mut preserved, &["claude".into()], false, false).unwrap();
        assert_eq!(preserved.answers, progress.answers);
        for requested in [vec!["unknown".into()], vec!["codex".into(), "codex".into()]] {
            assert!(check_request(&progress, &requested).is_err());
        }
    }
}
