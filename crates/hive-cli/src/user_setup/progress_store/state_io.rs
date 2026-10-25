//! Recover metadata CAS without mistaking a live writer for an abandoned claim.
//! This journal binds file bytes; it never authorizes setup execution.
use super::{history, valid_hash, ProgressStore, SetupError, SetupProgress, TargetBinding};
use crate::user_install::{
    host_policy_mode, inspect_setup_write_claim, lock_setup_state, read_user_setup_file,
    recover_user_setup_file, replace_user_setup_file, SetupWriteClaim,
};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const LIMIT: u64 = 8192;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema_version: u8,
    binding: TargetBinding,
    path: String,
    before: String,
    after: String,
    before_mode: Option<u32>,
}

fn digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".into(), sha256_digest)
}

impl ProgressStore {
    fn intent_path(&self) -> PathBuf {
        Path::new(".hive/config/setup-progress/writes")
            .join(format!("{}.json", &self.binding.target_id[7..]))
    }

    fn validate_intent(&self, bytes: &[u8]) -> Result<Intent, SetupError> {
        let intent: Intent = serde_json::from_slice(bytes)
            .map_err(|_| SetupError::Verification("invalid setup state write record".into()))?;
        if intent.schema_version != 1
            || intent.binding != self.binding
            || !self.state_path(Path::new(&intent.path))
            || [&intent.before, &intent.after]
                .iter()
                .any(|hash| hash.as_str() != "absent" && !valid_hash(hash))
            || intent.before_mode.is_some_and(|mode| mode > 0o7777)
        {
            return Err(SetupError::Conflict(
                "setup state write scope or fingerprints changed".into(),
            ));
        }
        Ok(intent)
    }

    fn state_path(&self, relative: &Path) -> bool {
        relative == self.relative
            || (self.binding.scope == hive_core::preferences::PreferenceScope::User
                && relative == Path::new(super::monitor_review::PATH))
            || history::is_record_path(relative)
            || (self.binding.scope == hive_core::preferences::PreferenceScope::User
                && relative == Path::new(super::LEGACY_PROGRESS))
    }

    fn validate_state(&self, relative: &Path, bytes: &[u8]) -> Result<(), SetupError> {
        if relative == self.relative {
            let progress: SetupProgress = serde_yaml::from_slice(bytes)
                .map_err(|_| SetupError::Verification("invalid retained setup progress".into()))?;
            self.validate(&progress)
        } else if relative == Path::new(super::LEGACY_PROGRESS)
            && self.binding.scope == hive_core::preferences::PreferenceScope::User
        {
            super::parse_progress_file(bytes).map(|_| ())
        } else if relative == Path::new(super::monitor_review::PATH) {
            super::monitor_review::validate_scope(self, bytes).map(|_| ())
        } else {
            history::validate_scope(self, relative, bytes)
        }
    }

    fn read_intent(&self) -> Result<Option<Vec<u8>>, SetupError> {
        let path = self.intent_path();
        let current =
            read_user_setup_file(&self.root, &path, LIMIT).map_err(SetupError::Conflict)?;
        if current.is_some() {
            return Ok(current);
        }
        let claim =
            inspect_setup_write_claim(&self.root, &path, LIMIT).map_err(SetupError::Conflict)?;
        Ok(claim.and_then(|claim| claim.before.or(claim.staged)))
    }

    pub(super) fn read_state(
        &self,
        relative: &Path,
        limit: u64,
    ) -> Result<Option<Vec<u8>>, SetupError> {
        let current =
            read_user_setup_file(&self.root, relative, limit).map_err(SetupError::Conflict)?;
        let Some(claim) =
            inspect_setup_write_claim(&self.root, relative, limit).map_err(SetupError::Conflict)?
        else {
            return Ok(current);
        };
        let bytes = self.read_intent()?.ok_or_else(|| {
            SetupError::Conflict("setup state claim has no recovery record".into())
        })?;
        let intent = self.validate_intent(&bytes)?;
        if Path::new(&intent.path) != relative {
            return Err(SetupError::Conflict(
                "setup state claim differs from its recovery record".into(),
            ));
        }
        self.validate_claim(&intent, &claim)?;
        let current_digest = digest(current.as_deref());
        if current_digest == intent.after || current_digest == intent.before {
            return Ok(current);
        }
        if current.is_none() && claim.before.is_some() {
            return Ok(claim.before);
        }
        Err(SetupError::Conflict(
            "setup state changed during an interrupted write".into(),
        ))
    }

    fn validate_claim(&self, intent: &Intent, claim: &SetupWriteClaim) -> Result<(), SetupError> {
        let path = Path::new(&intent.path);
        if let Some(before) = &claim.before {
            if sha256_digest(before) != intent.before || claim.before_mode != intent.before_mode {
                return Err(SetupError::Conflict("retained setup state changed".into()));
            }
            self.validate_state(path, before)?;
        }
        if let Some(staged) = &claim.staged {
            if sha256_digest(staged) != intent.after {
                return Err(SetupError::Conflict("staged setup state changed".into()));
            }
            self.validate_state(path, staged)?;
        }
        Ok(())
    }

    // The write record is only created or removed, never replaced in place.
    // A failed creation cannot precede a data write; a failed deletion follows settled data.
    fn recover_intent_file(&self) -> Result<(), SetupError> {
        let path = self.intent_path();
        let Some(claim) =
            inspect_setup_write_claim(&self.root, &path, LIMIT).map_err(SetupError::Conflict)?
        else {
            return Ok(());
        };
        let current =
            read_user_setup_file(&self.root, &path, LIMIT).map_err(SetupError::Conflict)?;
        if let Some(before) = &claim.before {
            self.validate_intent(before)?;
            if claim.staged.is_some() || current.as_ref().is_some_and(|value| value != before) {
                return Err(SetupError::Conflict(
                    "setup write record deletion changed".into(),
                ));
            }
            recover_user_setup_file(
                &self.root,
                &path,
                &sha256_digest(before),
                "absent",
                claim.before_mode,
            )
            .map_err(SetupError::Conflict)?;
        } else {
            let candidate = current.as_ref().or(claim.staged.as_ref());
            if let Some(bytes) = candidate {
                self.validate_intent(bytes)?;
            }
            if current
                .as_ref()
                .zip(claim.staged.as_ref())
                .is_some_and(|(a, b)| a != b)
            {
                return Err(SetupError::Conflict(
                    "setup write record publication changed".into(),
                ));
            }
            recover_user_setup_file(
                &self.root,
                &path,
                "absent",
                &digest(candidate.map(Vec::as_slice)),
                None,
            )
            .map_err(SetupError::Conflict)?;
        }
        Ok(())
    }

    fn recover_state_write(&self) -> Result<(), SetupError> {
        self.recover_intent_file()?;
        let journal = self.intent_path();
        let Some(bytes) =
            read_user_setup_file(&self.root, &journal, LIMIT).map_err(SetupError::Conflict)?
        else {
            return Ok(());
        };
        let intent = self.validate_intent(&bytes)?;
        let path = Path::new(&intent.path);
        let current = read_user_setup_file(&self.root, path, super::MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        if let Some(current) = &current {
            self.validate_state(path, current)?;
            let hash = sha256_digest(current);
            let mode = host_policy_mode(&self.root, path).map_err(SetupError::Conflict)?;
            let installed_mode = crate::user_install::setup_file_mode(path);
            if !((hash == intent.before && mode == intent.before_mode)
                || (hash == intent.after && mode == installed_mode))
            {
                return Err(SetupError::Conflict(
                    "published setup state or permissions changed".into(),
                ));
            }
        }
        if let Some(claim) =
            inspect_setup_write_claim(&self.root, path, super::MAX_USER_SETUP_BYTES)
                .map_err(SetupError::Conflict)?
        {
            self.validate_claim(&intent, &claim)?;
            recover_user_setup_file(
                &self.root,
                path,
                &intent.before,
                &intent.after,
                intent.before_mode,
            )
            .map_err(SetupError::Conflict)?;
        } else if ![intent.before.as_str(), intent.after.as_str()]
            .contains(&digest(current.as_deref()).as_str())
        {
            return Err(SetupError::Conflict(
                "setup state no longer matches the write record".into(),
            ));
        }
        replace_user_setup_file(&self.root, &journal, Some(&bytes), None)
            .map_err(SetupError::Conflict)
    }

    pub(in crate::user_setup) fn recover_pending_state(&self) -> Result<(), SetupError> {
        self.verify_directory_binding()?;
        let path = self.intent_path();
        if read_user_setup_file(&self.root, &path, LIMIT)
            .map_err(SetupError::Conflict)?
            .is_none()
            && inspect_setup_write_claim(&self.root, &path, LIMIT)
                .map_err(SetupError::Conflict)?
                .is_none()
        {
            return Ok(());
        }
        let _lease = lock_setup_state(&self.root, &self.binding.target_id[7..])
            .map_err(SetupError::Conflict)?;
        self.recover_state_write()
    }

    pub(super) fn write_state(
        &self,
        relative: &Path,
        expected: Option<&[u8]>,
        desired: Option<&[u8]>,
    ) -> Result<(), SetupError> {
        self.verify_directory_binding()?;
        if !self.state_path(relative) {
            return Err(SetupError::Conflict("unowned setup state path".into()));
        }
        for bytes in [expected, desired].into_iter().flatten() {
            self.validate_state(relative, bytes)?;
        }
        let _lease = lock_setup_state(&self.root, &self.binding.target_id[7..])
            .map_err(SetupError::Conflict)?;
        self.recover_state_write()?;
        let current = read_user_setup_file(&self.root, relative, super::MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        if current.as_deref() != expected
            || inspect_setup_write_claim(&self.root, relative, super::MAX_USER_SETUP_BYTES)
                .map_err(SetupError::Conflict)?
                .is_some()
        {
            return Err(SetupError::Conflict(
                "setup state changed before writing".into(),
            ));
        }
        let before_mode = if current.is_some() {
            host_policy_mode(&self.root, relative).map_err(SetupError::Conflict)?
        } else {
            None
        };
        let intent = Intent {
            schema_version: 1,
            binding: self.binding.clone(),
            path: relative.to_string_lossy().replace('\\', "/"),
            before: digest(expected),
            after: digest(desired),
            before_mode,
        };
        let bytes =
            serde_json::to_vec(&intent).map_err(|error| SetupError::Internal(error.to_string()))?;
        let journal = self.intent_path();
        replace_user_setup_file(&self.root, &journal, None, Some(&bytes))
            .map_err(SetupError::Conflict)?;
        replace_user_setup_file(&self.root, relative, expected, desired)
            .map_err(SetupError::Conflict)?;
        replace_user_setup_file(&self.root, &journal, Some(&bytes), None)
            .map_err(SetupError::Conflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_install::{
        interrupt_setup_creation, interrupt_setup_remove, interrupt_setup_replace,
    };
    use serde_json::json;
    use std::fs;

    fn sample(store: &ProgressStore) -> SetupProgress {
        let progress = store
            .new_progress("user-language".into(), json!({"interface_language":"ko"}))
            .unwrap();
        store.save(&store.load().unwrap(), &progress).unwrap();
        progress
    }

    fn intent(
        store: &ProgressStore,
        relative: &Path,
        before: Option<&[u8]>,
        after: Option<&[u8]>,
    ) -> Vec<u8> {
        serde_json::to_vec(&Intent {
            schema_version: 1,
            binding: store.binding.clone(),
            path: relative.to_string_lossy().replace('\\', "/"),
            before: digest(before),
            after: digest(after),
            before_mode: if before.is_some() {
                host_policy_mode(&store.root, relative).unwrap()
            } else {
                None
            },
        })
        .unwrap()
    }

    fn arm(store: &ProgressStore, relative: &Path, before: &[u8], after: &[u8]) -> Vec<u8> {
        let bytes = intent(store, relative, Some(before), Some(after));
        replace_user_setup_file(&store.root, &store.intent_path(), None, Some(&bytes)).unwrap();
        bytes
    }

    #[test]
    fn interrupted_progress_reads_are_pure_and_explicit_recovery_keeps_published_answers() {
        for published in [false, true] {
            let root = crate::test_tempdir();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let before_progress = sample(&store);
            let before = fs::read(root.path().join(&store.relative)).unwrap();
            let mut after_progress = before_progress.clone();
            after_progress.answers["interface_language"] = json!("en");
            let after = serde_yaml::to_string(&after_progress).unwrap().into_bytes();
            let journal = arm(&store, &store.relative, &before, &after);
            interrupt_setup_replace(&store.root, &store.relative, &before, &after, published);
            let expected = if published {
                &after_progress
            } else {
                &before_progress
            };
            assert_eq!(store.load().unwrap().progress.as_ref(), Some(expected));
            assert!(
                inspect_setup_write_claim(&store.root, &store.relative, 1_048_576)
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                fs::read(root.path().join(store.intent_path())).unwrap(),
                journal
            );
            store.recover_pending_state().unwrap();
            assert_eq!(store.load().unwrap().progress.as_ref(), Some(expected));
            assert!(
                inspect_setup_write_claim(&store.root, &store.relative, 1_048_576)
                    .unwrap()
                    .is_none()
            );
            assert!(!root.path().join(store.intent_path()).exists());
            store.save(&store.load().unwrap(), &after_progress).unwrap();
            assert_eq!(
                store.load().unwrap().progress.as_ref(),
                Some(&after_progress)
            );
        }
    }

    #[test]
    fn lifecycle_record_write_can_resume_before_and_after_publication() {
        for published in [false, true] {
            let root = crate::test_tempdir();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let mut progress = sample(&store);
            progress.phase = super::super::ProgressPhase::Applying;
            store.save(&store.load().unwrap(), &progress).unwrap();
            store.record_applying(&progress).unwrap();
            let path = PathBuf::from(format!(
                ".hive/history/lifecycle/{}.json",
                progress.operation_id
            ));
            let before = fs::read(root.path().join(&path)).unwrap();
            let mut value: serde_json::Value = serde_json::from_slice(&before).unwrap();
            value["phase"] = json!("recoverable-failure");
            let after = serde_json::to_vec(&value).unwrap();
            arm(&store, &path, &before, &after);
            interrupt_setup_replace(&store.root, &path, &before, &after, published);
            store.record_failure(&progress).unwrap();
            let saved: serde_json::Value =
                serde_json::from_slice(&fs::read(root.path().join(&path)).unwrap()).unwrap();
            assert_eq!(saved["phase"], "recoverable-failure");
            assert!(!root.path().join(store.intent_path()).exists());
            store.record_applying(&progress).unwrap();
        }
    }

    #[test]
    fn the_write_record_itself_recovers_creation_and_deletion_without_changing_data() {
        for stage in ["empty", "staged", "published", "deleted"] {
            let root = crate::test_tempdir();
            let store = ProgressStore::open(root.path(), None).unwrap();
            sample(&store);
            let original = fs::read(root.path().join(&store.relative)).unwrap();
            let journal = intent(&store, &store.relative, Some(&original), Some(&original));
            if stage == "deleted" {
                replace_user_setup_file(&store.root, &store.intent_path(), None, Some(&journal))
                    .unwrap();
                interrupt_setup_remove(&store.root, &store.intent_path(), &journal);
            } else {
                interrupt_setup_creation(
                    &store.root,
                    &store.intent_path(),
                    (stage != "empty").then_some(journal.as_slice()),
                    stage == "published",
                );
            }
            store.recover_pending_state().unwrap();
            assert_eq!(
                fs::read(root.path().join(&store.relative)).unwrap(),
                original
            );
            assert!(!root.path().join(store.intent_path()).exists());
            assert!(
                inspect_setup_write_claim(&store.root, &store.intent_path(), LIMIT)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn a_live_writer_and_changed_retained_bytes_block_recovery_without_overwrite() {
        let root = crate::test_tempdir();
        let store = ProgressStore::open(root.path(), None).unwrap();
        sample(&store);
        let before = fs::read(root.path().join(&store.relative)).unwrap();
        let journal = arm(&store, &store.relative, &before, &before);
        let lease = lock_setup_state(&store.root, &store.binding.target_id[7..]).unwrap();
        interrupt_setup_replace(&store.root, &store.relative, &before, &before, false);
        assert!(store.recover_pending_state().is_err());
        assert_eq!(
            fs::read(root.path().join(store.intent_path())).unwrap(),
            journal
        );
        drop(lease);
        store.recover_pending_state().unwrap();
        arm(&store, &store.relative, &before, &before);
        interrupt_setup_replace(&store.root, &store.relative, &before, &before, true);
        fs::write(root.path().join(&store.relative), b"changed externally").unwrap();
        assert!(store.load().is_err());
        assert!(store.recover_pending_state().is_err());
        assert_eq!(
            fs::read(root.path().join(&store.relative)).unwrap(),
            b"changed externally"
        );
        assert!(root.path().join(store.intent_path()).exists());
    }

    #[test]
    fn invalid_write_scope_cannot_select_other_settings_or_discard_the_record() {
        let root = crate::test_tempdir();
        let store = ProgressStore::open(root.path(), None).unwrap();
        sample(&store);
        let original = fs::read(root.path().join(&store.relative)).unwrap();
        let bytes = intent(&store, &store.relative, Some(&original), Some(&original));
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["path"] = json!(".codex/config.toml");
        let bytes = serde_json::to_vec(&value).unwrap();
        replace_user_setup_file(&store.root, &store.intent_path(), None, Some(&bytes)).unwrap();
        assert!(store.recover_pending_state().is_err());
        assert_eq!(
            fs::read(root.path().join(store.intent_path())).unwrap(),
            bytes
        );
        assert_eq!(
            fs::read(root.path().join(&store.relative)).unwrap(),
            original
        );
    }

    #[test]
    fn first_progress_creation_and_confirmed_clear_recover_without_resurrecting_answers() {
        for published in [false, true] {
            let root = crate::test_tempdir();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let progress = store
                .new_progress("user-language".into(), json!({"interface_language":"ko"}))
                .unwrap();
            let bytes = serde_yaml::to_string(&progress).unwrap().into_bytes();
            let journal = intent(&store, &store.relative, None, Some(&bytes));
            replace_user_setup_file(&store.root, &store.intent_path(), None, Some(&journal))
                .unwrap();
            interrupt_setup_creation(&store.root, &store.relative, Some(&bytes), published);
            assert_eq!(store.load().unwrap().progress.is_some(), published);
            store.recover_pending_state().unwrap();
            assert_eq!(store.load().unwrap().progress.is_some(), published);
            if !published {
                store.save(&store.load().unwrap(), &progress).unwrap();
            }
            let journal = intent(&store, &store.relative, Some(&bytes), None);
            replace_user_setup_file(&store.root, &store.intent_path(), None, Some(&journal))
                .unwrap();
            interrupt_setup_remove(&store.root, &store.relative, &bytes);
            assert!(store.load().unwrap().progress.is_none());
            store.recover_pending_state().unwrap();
            assert!(store.load().unwrap().progress.is_none());
            assert!(!root.path().join(store.intent_path()).exists());
        }
    }

    #[test]
    fn changed_original_fingerprint_and_copied_scope_records_are_preserved_and_rejected() {
        let root = crate::test_tempdir();
        let store = ProgressStore::open(root.path(), None).unwrap();
        sample(&store);
        let before = fs::read(root.path().join(&store.relative)).unwrap();
        let mut journal: serde_json::Value = serde_json::from_slice(&intent(
            &store,
            &store.relative,
            Some(&before),
            Some(&before),
        ))
        .unwrap();
        journal["before"] = json!(sha256_digest(b"changed source"));
        let bytes = serde_json::to_vec(&journal).unwrap();
        replace_user_setup_file(&store.root, &store.intent_path(), None, Some(&bytes)).unwrap();
        interrupt_setup_replace(&store.root, &store.relative, &before, &before, false);
        assert!(store.load().is_err());
        assert!(store.recover_pending_state().is_err());
        assert_eq!(
            inspect_setup_write_claim(&store.root, &store.relative, 1_048_576)
                .unwrap()
                .unwrap()
                .before,
            Some(before)
        );

        let other = crate::test_tempdir();
        let other_store = ProgressStore::open(other.path(), None).unwrap();
        let other_progress = sample(&other_store);
        replace_user_setup_file(
            &other_store.root,
            &other_store.intent_path(),
            None,
            Some(&bytes),
        )
        .unwrap();
        assert!(other_store.recover_pending_state().is_err());
        assert_eq!(other_store.load().unwrap().progress, Some(other_progress));
        assert_eq!(
            fs::read(other.path().join(other_store.intent_path())).unwrap(),
            bytes
        );
    }

    #[test]
    fn user_completion_does_not_recover_or_block_on_a_different_projects_write() {
        let root = crate::test_tempdir();
        let target = crate::test_tempdir();
        let project = ProgressStore::open(root.path(), Some(target.path())).unwrap();
        let mut progress = project
            .new_progress("profile".into(), json!({"project_name":"other"}))
            .unwrap();
        progress.phase = super::super::ProgressPhase::Applying;
        project.save(&project.load().unwrap(), &progress).unwrap();
        project.record_applying(&progress).unwrap();
        project.record_completion(&progress).unwrap();
        let path = PathBuf::from(format!(
            ".hive/history/lifecycle/{}.json",
            progress.operation_id
        ));
        let before = fs::read(root.path().join(&path)).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&before).unwrap();
        value["completed_at_unix_nanos"] =
            json!(value["completed_at_unix_nanos"].as_u64().unwrap() + 1);
        let after = serde_json::to_vec(&value).unwrap();
        let journal = arm(&project, &path, &before, &after);
        interrupt_setup_replace(&project.root, &path, &before, &after, true);

        let user = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = sample(&user);
        progress.phase = super::super::ProgressPhase::Applying;
        user.save(&user.load().unwrap(), &progress).unwrap();
        user.record_applying(&progress).unwrap();
        user.record_completion(&progress).unwrap();
        assert_eq!(
            fs::read(root.path().join(project.intent_path())).unwrap(),
            journal
        );
        assert_eq!(fs::read(root.path().join(&path)).unwrap(), after);
        assert!(inspect_setup_write_claim(&project.root, &path, 1_048_576)
            .unwrap()
            .is_some());
        project.recover_pending_state().unwrap();
    }
}
