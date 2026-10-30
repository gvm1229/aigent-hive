//! Bind an existing owned monitor to an explicit v2 setup that disables monitoring.
use crate::usage_monitor::SavedMonitorChoice;
use crate::user_setup::{self, progress_store::SetupProgress, Arguments, SetupError};
use serde_json::json;
use std::path::Path;

fn wanted(answers: &serde_json::Value) -> bool {
    answers["usage_guard"]["enforcement_enabled"] == true
        && answers["usage_guard"]["midway"]["enabled"] == true
}

pub(super) fn bind(root: &Path, progress: &mut SetupProgress) -> Result<(), SetupError> {
    bind_using(progress, || {
        crate::usage_monitor::inspect_saved_choice(root)
            .map_err(|error| SetupError::Conflict(error.message().into()))
    })
}

fn bind_using(
    progress: &mut SetupProgress,
    inspect: impl FnOnce() -> Result<SavedMonitorChoice, SetupError>,
) -> Result<(), SetupError> {
    if progress.legacy_migration.is_some() || wanted(&progress.answers) {
        progress.monitor_stop = None;
        return Ok(());
    }
    let current = inspect()?;
    if progress
        .monitor_stop
        .as_ref()
        .is_some_and(|prior| prior != &current)
    {
        return Err(SetupError::Conflict(
            "monitor choice changed after the setup preview".into(),
        ));
    }
    progress.monitor_stop = current.configured.then_some(current);
    Ok(())
}

pub(super) fn validate_intent(progress: &SetupProgress) -> Result<(), SetupError> {
    if progress.monitor_stop.is_some() && wanted(&progress.answers) {
        return Err(SetupError::Conflict(
            "monitor stop no longer matches the confirmed settings".into(),
        ));
    }
    Ok(())
}

pub(super) fn verify_finished(root: &Path, progress: &SetupProgress) -> Result<(), SetupError> {
    if progress.monitor_stop.is_none() {
        return Ok(());
    }
    validate_intent(progress)?;
    crate::usage_monitor::verify_stopped_for_setup(root)
        .map_err(|error| SetupError::Verification(error.message().into()))
}

pub(super) fn describe(
    arguments: &Arguments,
    config: &user_setup::UserSetupConfig,
    result: &mut crate::ActionResult,
) -> Result<(), SetupError> {
    if config
        .usage_guard
        .policy
        .as_ref()
        .is_none_or(|p| p.effective_midway().enabled)
    {
        return Ok(());
    }
    let choice = crate::usage_monitor::inspect_saved_choice(&arguments.user_root)
        .map_err(|error| SetupError::Conflict(error.message().into()))?;
    if choice.configured {
        result.data.get_or_insert_with(|| json!({}))["monitor_transition"] = json!({
            "action":"disable-owned-monitor","saved_enabled":choice.enabled,
            "running_state":"not-inspected","history_preserved":true,"sensor_queries":0
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::{progress_store::ProgressStore, InterfaceLanguage};
    use hive_core::sha256_digest;

    fn selected(enabled: bool) -> SavedMonitorChoice {
        SavedMonitorChoice {
            enabled,
            configured: true,
            config_digest: sha256_digest(&[u8::from(enabled)]),
        }
    }

    #[test]
    fn disabled_setup_binds_only_owned_configuration_and_rejects_a_changed_choice() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let answers = super::super::default_answers(Some(InterfaceLanguage::Ko));
        let mut progress = store.new_progress("test".into(), answers).unwrap();
        bind(root.path(), &mut progress).unwrap();
        assert!(progress.monitor_stop.is_none());
        assert!(!root.path().join(".hive/runtime/usage-monitor").exists());
        bind_using(&mut progress, || Ok(selected(true))).unwrap();
        assert_eq!(progress.monitor_stop, Some(selected(true)));
        store.save(&store.load().unwrap(), &progress).unwrap();
        assert!(bind_using(&mut progress, || Ok(selected(false))).is_err());
        assert_eq!(progress.monitor_stop, Some(selected(true)));
        progress.answers["usage_guard"]["enforcement_enabled"] = json!(true);
        progress.answers["usage_guard"]["midway"]["enabled"] = json!(true);
        assert!(validate_intent(&progress).is_err());
        bind_using(&mut progress, || {
            panic!("keeping monitoring must not prepare a stop")
        })
        .unwrap();
        assert!(progress.monitor_stop.is_none());
    }

    #[test]
    fn configured_disabled_monitor_is_still_bound_for_exact_scheduler_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress(
                "test".into(),
                super::super::default_answers(Some(InterfaceLanguage::En)),
            )
            .unwrap();
        bind_using(&mut progress, || Ok(selected(false))).unwrap();
        assert_eq!(progress.monitor_stop, Some(selected(false)));
        assert!(verify_finished(root.path(), &progress).is_err());
        assert!(!root.path().join(".hive/runtime/usage-monitor").exists());
    }
}
