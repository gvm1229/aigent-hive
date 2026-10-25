//! A cancelled activation becomes a new, sealed setup operation with monitoring off.
use super::{
    history, valid_hash, ProgressPhase, ProgressStore, SetupError, SetupProgress, TargetBinding,
    MAX_USER_SETUP_BYTES,
};
use hive_core::preferences::PreferenceScope;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

pub(super) const PATH: &str = ".hive/config/setup-progress/monitor-handoff.json";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::user_setup) struct MonitorHandoff {
    pub schema_version: u8,
    pub binding: TargetBinding,
    pub old_operation_id: String,
    pub old_intent_digest: String,
    pub replacement: SetupProgress,
}

pub(super) fn validate_scope(
    store: &ProgressStore,
    bytes: &[u8],
) -> Result<MonitorHandoff, SetupError> {
    let handoff: MonitorHandoff = serde_json::from_slice(bytes)
        .map_err(|_| SetupError::Verification("invalid monitor setup handoff".into()))?;
    if handoff.schema_version != 1
        || store.binding.scope != PreferenceScope::User
        || handoff.binding != store.binding
        || handoff.old_operation_id.len() != 64
        || !handoff
            .old_operation_id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !valid_hash(&handoff.old_intent_digest)
        || handoff.old_operation_id == handoff.replacement.operation_id
        || handoff.replacement.phase != ProgressPhase::Applying
        || handoff.replacement.monitor_start.is_some()
        || handoff.replacement.project_context.is_some()
        || !handoff.replacement.reconfigure_roles.is_empty()
        || handoff.replacement.extras_draft.is_some()
        || handoff.replacement.index_rebuild_scope.is_some()
        || handoff.replacement.legacy_migration.is_some()
        || handoff.replacement.vector_choice.is_some()
        || !handoff.replacement.host_deactivations.is_empty()
        || handoff
            .replacement
            .answers
            .pointer("/usage_guard/midway/enabled")
            != Some(&json!(false))
    {
        return Err(SetupError::Conflict(
            "monitor setup handoff scope changed; preserved".into(),
        ));
    }
    store.validate(&handoff.replacement)?;
    Ok(handoff)
}

fn unchanged_choices(old: &SetupProgress, new: &SetupProgress) -> Result<(), SetupError> {
    if old.monitor_start.is_none()
        || old.answers.pointer("/usage_guard/midway/enabled") != Some(&json!(true))
    {
        return Err(SetupError::Conflict(
            "monitor setup has no pending activation to withdraw".into(),
        ));
    }
    let mut desired = old.answers.clone();
    desired["usage_guard"]["midway"]["enabled"] = json!(false);
    let mut replacement = new.answers.clone();
    for answers in [&mut desired, &mut replacement] {
        if let Some(records) = answers
            .get_mut("preference_decisions")
            .and_then(Value::as_object_mut)
        {
            records.remove("/usage_guard/midway/enabled");
        }
    }
    if desired != replacement {
        return Err(SetupError::Conflict(
            "monitor withdrawal changed other confirmed settings; preserved".into(),
        ));
    }
    Ok(())
}

impl ProgressStore {
    pub(in crate::user_setup) fn monitor_handoff_pending(&self) -> Result<bool, SetupError> {
        Ok(self.read_state(Path::new(PATH), MAX_USER_SETUP_BYTES)?.is_some())
    }
    pub(in crate::user_setup) fn monitor_intent_digest(progress: &SetupProgress) -> Result<String, SetupError> {
        history::intent_digest(progress)
    }

    /// Called only with the user setup lease and a stopped same-root controller lease.
    pub(in crate::user_setup) fn handoff_monitor(
        &self, old: &SetupProgress, answers: Value, stopped: &crate::usage_monitor::StoppedReader,
    ) -> Result<SetupProgress, SetupError> {
        stopped.verify_root(&self.root_path).map_err(|e| SetupError::Conflict(e.message().into()))?;
        let mut replacement = self.new_progress("user-monitor-withdrawal".into(), answers)?;
        replacement.phase = ProgressPhase::Applying;
        let choice = crate::usage_monitor::inspect_saved_choice(&self.root_path)
            .map_err(|e| SetupError::Conflict(e.message().into()))?;
        if choice.enabled { return Err(SetupError::Conflict("monitor has not stopped".into())); }
        replacement.monitor_stop = choice.configured.then_some(choice);
        unchanged_choices(old, &replacement)?;
        let handoff = MonitorHandoff { schema_version: 1, binding: self.binding.clone(),
            old_operation_id: old.operation_id.clone(), old_intent_digest: history::intent_digest(old)?, replacement };
        let bytes = serde_json::to_vec(&handoff).map_err(|e| SetupError::Internal(e.to_string()))?;
        validate_scope(self, &bytes)?;
        self.write_state(Path::new(PATH), None, Some(&bytes))?;
        self.recover_monitor_handoff_locked()?;
        Ok(handoff.replacement)
    }

    /// The public entry acquires the same user lease as setup apply and recovery.
    pub fn recover_monitor_handoff(&self) -> Result<bool, SetupError> {
        if self.binding.scope != PreferenceScope::User
            || self
                .read_state(Path::new(PATH), MAX_USER_SETUP_BYTES)?
                .is_none()
        {
            return Ok(false);
        }
        let _lease =
            crate::user_install::lock_user_setup(&self.root).map_err(SetupError::Conflict)?;
        self.recover_monitor_handoff_locked()
    }

    pub(in crate::user_setup) fn recover_monitor_handoff_locked(&self) -> Result<bool, SetupError> {
        self.recover_pending_state()?;
        let Some(bytes) = self.read_state(Path::new(PATH), MAX_USER_SETUP_BYTES)? else {
            return Ok(true);
        };
        let handoff = validate_scope(self, &bytes)?;
        let config = crate::user_install::read_user_setup_file(
            &self.config_root,
            &self.config_relative,
            MAX_USER_SETUP_BYTES,
        )
        .map_err(SetupError::Conflict)?;
        let mode = config
            .as_ref()
            .map(|_| {
                crate::user_install::host_policy_mode(&self.config_root, &self.config_relative)
            })
            .transpose()
            .map_err(SetupError::Conflict)?
            .flatten();
        if !self.resume_ready(&handoff.replacement)
            || super::optional_digest(config.as_deref()) != handoff.replacement.base_config_digest
            || mode != handoff.replacement.base_config_mode
        {
            return Err(SetupError::Conflict(
                "settings changed before monitor handoff; preserved".into(),
            ));
        }
        let snapshot = self.load()?;
        let current = snapshot.progress.as_ref().ok_or_else(|| {
            SetupError::Conflict("monitor setup handoff lost its current operation".into())
        })?;
        if current.operation_id == handoff.old_operation_id {
            if !matches!(
                current.phase,
                ProgressPhase::Applying | ProgressPhase::RecoverableFailure
            ) || history::intent_digest(current)? != handoff.old_intent_digest
            {
                return Err(SetupError::Conflict(
                    "old monitor setup intent changed; preserved".into(),
                ));
            }
            unchanged_choices(current, &handoff.replacement)?;
            history::cancel_for_monitor_review(self, current, &handoff.replacement)?;
            #[cfg(test)]
            fault("cancelled")?;
            self.save(&snapshot, &handoff.replacement)?;
            #[cfg(test)]
            fault("replacement")?;
        } else if current != &handoff.replacement {
            return Err(SetupError::Conflict(
                "current setup differs from the monitor handoff; preserved".into(),
            ));
        }
        history::verify_monitor_replacement(
            self,
            &handoff.old_operation_id,
            &handoff.old_intent_digest,
            &handoff.replacement,
        )?;
        self.record_applying(&handoff.replacement)?;
        #[cfg(test)]
        fault("applying")?;
        history::retain_completed(self, &handoff.old_operation_id)?;
        self.write_state(Path::new(PATH), Some(&bytes), None)?;
        Ok(true)
    }
}

#[cfg(test)]
thread_local! { static STOP_AT:std::cell::Cell<Option<&'static str>>=const{std::cell::Cell::new(None)}; }
#[cfg(test)]
fn fault(point: &str) -> Result<(), SetupError> {
    if STOP_AT.with(|cell| cell.get() == Some(point)) {
        STOP_AT.with(|cell| cell.set(None));
        return Err(SetupError::Internal(
            "fixture monitor handoff interruption".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::{self, InterfaceLanguage};

    #[test]
    fn monitor_handoff_rejects_reactivation_other_scopes_and_changed_config_before_cancelling() {
        for change in ["enabled", "scope", "settings"] {
            let (root, store, old, mut handoff) = fixture();
            match change {
                "enabled" => {
                    handoff.replacement.answers["usage_guard"]["midway"]["enabled"] = json!(true);
                }
                "scope" => handoff.binding.target_id = "outside-target".into(),
                _ => {}
            }
            let encoded = serde_json::to_vec(&handoff).unwrap();
            if change == "settings" {
                store
                    .write_state(Path::new(PATH), None, Some(&encoded))
                    .unwrap();
                let file = root.path().join(user_setup::USER_SETUP_RELATIVE);
                let mut changed = std::fs::read(&file).unwrap();
                changed.extend_from_slice(b"\n# outside edit\n");
                std::fs::write(&file, &changed).unwrap();
                assert!(store.recover_monitor_handoff().is_err());
                let receipt: Value = serde_json::from_slice(
                    &std::fs::read(
                        root.path()
                            .join(".hive/history/lifecycle")
                            .join(format!("{}.json", old.operation_id)),
                    )
                    .unwrap(),
                )
                .unwrap();
                assert!(receipt.get("cancelled_by_operation").is_none());
                assert_eq!(std::fs::read(file).unwrap(), changed);
            } else {
                assert!(store
                    .write_state(Path::new(PATH), None, Some(&encoded))
                    .is_err());
            }
            assert_eq!(store.load().unwrap().progress.as_ref(), Some(&old));
        }
    }

    fn fixture() -> (
        tempfile::TempDir,
        ProgressStore,
        SetupProgress,
        MonitorHandoff,
    ) {
        let root = tempfile::tempdir().unwrap();
        let mut answers = user_setup::setup_engine::default_answers(Some(InterfaceLanguage::En));
        answers["usage_guard"]["enforcement_enabled"] = json!(true);
        answers["usage_guard"]["midway"]["enabled"] = json!(true);
        let config =
            user_setup::parse_and_validate_config(&serde_json::to_vec(&answers).unwrap()).unwrap();
        let path = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, user_setup::canonical_config(&config).unwrap()).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut old = store
            .new_progress("user-monitor-activation".into(), answers.clone())
            .unwrap();
        old.monitor_start = Some(user_setup::setup_engine::monitor_start::fixture_approval(
            &answers,
        ));
        old.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &old).unwrap();
        store.record_applying(&old).unwrap();
        answers["usage_guard"]["midway"]["enabled"] = json!(false);
        let mut replacement = store.new_progress("file-apply".into(), answers).unwrap();
        replacement.phase = ProgressPhase::Applying;
        let handoff = MonitorHandoff {
            schema_version: 1,
            binding: store.binding.clone(),
            old_operation_id: old.operation_id.clone(),
            old_intent_digest: history::intent_digest(&old).unwrap(),
            replacement,
        };
        (root, store, old, handoff)
    }

    #[test]
    fn monitor_handoff_replays_each_boundary_without_success_for_the_cancelled_operation() {
        for boundary in ["cancelled", "replacement", "applying"] {
            let (root, store, old, handoff) = fixture();
            let before = std::fs::read(root.path().join(user_setup::USER_SETUP_RELATIVE)).unwrap();
            let encoded = serde_json::to_vec(&handoff).unwrap();
            store
                .write_state(Path::new(PATH), None, Some(&encoded))
                .unwrap();
            STOP_AT.with(|cell| cell.set(Some(boundary)));
            assert!(store.recover_monitor_handoff().is_err());
            assert!(store.recover_monitor_handoff().unwrap());
            assert_eq!(
                store.load().unwrap().progress.as_ref(),
                Some(&handoff.replacement)
            );
            assert!(!store.recover_monitor_handoff().unwrap());
            assert!(store.record_completion(&old).is_err());
            assert!(history::record(&store, &old, ProgressPhase::Applying).is_err());
            let receipt: Value = serde_json::from_slice(
                &std::fs::read(
                    root.path()
                        .join(".hive/history/lifecycle")
                        .join(format!("{}.json", old.operation_id)),
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(receipt["phase"], "recoverable-failure");
            assert_eq!(
                receipt["cancelled_by_operation"],
                handoff.replacement.operation_id
            );
            assert_eq!(
                std::fs::read(root.path().join(user_setup::USER_SETUP_RELATIVE)).unwrap(),
                before
            );
        }
    }

    #[test]
    fn monitor_handoff_rejects_other_settings_and_a_changed_replacement_after_cancellation() {
        let (_root, store, old, mut handoff) = fixture();
        handoff.replacement.answers["persona"]["id"] = json!("friendly");
        let encoded = serde_json::to_vec(&handoff).unwrap();
        store
            .write_state(Path::new(PATH), None, Some(&encoded))
            .unwrap();
        assert!(store.recover_monitor_handoff().is_err());
        assert_eq!(store.load().unwrap().progress.as_ref(), Some(&old));
        handoff.replacement.answers["persona"]["id"] = old.answers["persona"]["id"].clone();
        let valid = serde_json::to_vec(&handoff).unwrap();
        store
            .write_state(Path::new(PATH), Some(&encoded), Some(&valid))
            .unwrap();
        STOP_AT.with(|cell| cell.set(Some("cancelled")));
        assert!(store.recover_monitor_handoff().is_err());
        handoff.replacement.step = "changed-intent".into();
        let changed = serde_json::to_vec(&handoff).unwrap();
        store
            .write_state(Path::new(PATH), Some(&valid), Some(&changed))
            .unwrap();
        assert!(store.recover_monitor_handoff().is_err());
        assert_eq!(store.load().unwrap().progress.as_ref(), Some(&old));
    }
}
