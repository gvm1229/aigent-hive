//! Retire an unconfirmed draft that can no longer resume, without touching saved settings.
//!
//! A draft is unconfirmed while it is awaiting answers or previewed: nothing was applied and no
//! consent was recorded. When the saved settings changed after the draft started (for example a
//! software update rewrote them), the draft can never resume or apply, so it would otherwise block
//! every later setup change. Retiring it keeps an exact copy of the draft and leaves the settings
//! file unchanged.
use super::{
    ProgressPhase, ProgressSnapshot, ProgressStore, SetupError, SetupProgress,
    MAX_USER_SETUP_BYTES, QUESTION_REVISION,
};
use std::path::{Path, PathBuf};

const DIRECTORY: &str = ".hive/config/setup-progress/abandoned";

fn path(operation_id: &str) -> Result<PathBuf, SetupError> {
    if operation_id.len() != 64
        || !operation_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SetupError::Verification(
            "invalid setup operation identity".into(),
        ));
    }
    Ok(Path::new(DIRECTORY).join(format!("{operation_id}.yml")))
}

pub(super) fn is_record_path(relative: &Path) -> bool {
    relative
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".yml"))
        .and_then(|id| path(id).ok())
        .as_deref()
        == Some(relative)
}

pub(super) fn validate_scope(
    store: &ProgressStore,
    relative: &Path,
    bytes: &[u8],
) -> Result<(), SetupError> {
    let progress: SetupProgress = serde_yaml::from_slice(bytes)
        .map_err(|_| SetupError::Verification("invalid retired setup draft".into()))?;
    if path(&progress.operation_id)?.as_path() != relative || !unconfirmed(&progress) {
        return Err(SetupError::Conflict("retired setup draft changed".into()));
    }
    store.validate(&progress)
}

/// Nothing was applied or confirmed yet, so retiring the draft cannot lose an applied change.
pub(in crate::user_setup) fn unconfirmed(progress: &SetupProgress) -> bool {
    matches!(
        progress.phase,
        ProgressPhase::AwaitingAnswers | ProgressPhase::Previewed
    ) && progress.legacy_migration.is_none()
}

impl ProgressStore {
    /// An unconfirmed draft that no resume path can continue: its settings base changed, its
    /// protocol changed, or it answered a question the current catalog no longer has.
    pub(in crate::user_setup) fn incompatible_draft(
        &self,
        progress: &SetupProgress,
    ) -> Result<bool, SetupError> {
        if !unconfirmed(progress) || self.resume_ready(progress) {
            return Ok(false);
        }
        if progress.base_config_digest != self.base_config_digest
            || progress.base_config_mode != self.base_config_mode
            || progress.question_revision != QUESTION_REVISION
        {
            return Ok(true);
        }
        super::questions::removed_question(self.binding.scope, progress)
    }

    /// Keeps an exact copy of the unconfirmed draft and removes its resume record.
    /// The settings file is never read for writing or changed here.
    pub(in crate::user_setup) fn abandon_draft(
        &self,
        snapshot: &ProgressSnapshot,
    ) -> Result<String, SetupError> {
        self.verify_directory_binding()?;
        let progress = snapshot
            .progress
            .as_ref()
            .filter(|progress| unconfirmed(progress))
            .ok_or_else(|| SetupError::Conflict("no unconfirmed setup draft to retire".into()))?;
        let archive = path(&progress.operation_id)?;
        let bytes = match &snapshot.expected {
            Some(bytes) => bytes.clone(),
            None => serde_yaml::to_string(progress)
                .map_err(|error| SetupError::Internal(error.to_string()))?
                .into_bytes(),
        };
        match self.read_state(&archive, MAX_USER_SETUP_BYTES)? {
            Some(existing) if existing == bytes => {}
            Some(_) => {
                return Err(SetupError::Conflict(
                    "a different retired copy of this setup draft exists; both were preserved"
                        .into(),
                ))
            }
            None => self.write_state(&archive, None, Some(&bytes))?,
        }
        self.clear(snapshot)?;
        Ok(archive.to_string_lossy().replace('\\', "/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn draft(root: &Path, phase: ProgressPhase) -> (ProgressStore, Vec<u8>) {
        let store = ProgressStore::open(root, None).unwrap();
        let mut progress = store
            .new_progress(
                "user-monitor-activation".into(),
                json!({"interface_language":"ko"}),
            )
            .unwrap();
        progress.phase = phase;
        store.save(&store.load().unwrap(), &progress).unwrap();
        let bytes = fs::read(root.join(&store.relative)).unwrap();
        (store, bytes)
    }

    #[test]
    fn changed_settings_make_an_unconfirmed_draft_retirable_and_keep_settings_and_a_copy() {
        let root = tempfile::tempdir().unwrap();
        let (store, bytes) = draft(root.path(), ProgressPhase::Previewed);
        assert!(!store
            .incompatible_draft(store.load().unwrap().progress.as_ref().unwrap())
            .unwrap());
        let settings = root.path().join(super::super::USER_SETUP_RELATIVE);
        fs::write(&settings, "written later by an update").unwrap();
        let changed = ProgressStore::open(root.path(), None).unwrap();
        let snapshot = changed.load().unwrap();
        assert!(changed
            .incompatible_draft(snapshot.progress.as_ref().unwrap())
            .unwrap());
        let archive = changed.abandon_draft(&snapshot).unwrap();
        assert!(archive.starts_with(DIRECTORY));
        assert_eq!(fs::read(root.path().join(&archive)).unwrap(), bytes);
        assert_eq!(
            fs::read_to_string(&settings).unwrap(),
            "written later by an update"
        );
        assert!(changed.load().unwrap().progress.is_none());
    }

    #[test]
    fn confirmed_operations_are_never_retired() {
        for phase in [ProgressPhase::Applying, ProgressPhase::RecoverableFailure] {
            let root = tempfile::tempdir().unwrap();
            let (store, bytes) = draft(root.path(), ProgressPhase::AwaitingAnswers);
            let mut progress = store.load().unwrap().progress.unwrap();
            progress.phase = phase;
            let confirmed = serde_yaml::to_string(&progress).unwrap();
            assert_ne!(confirmed.as_bytes(), bytes.as_slice());
            fs::write(root.path().join(&store.relative), &confirmed).unwrap();
            fs::write(root.path().join(super::super::USER_SETUP_RELATIVE), "x").unwrap();
            let changed = ProgressStore::open(root.path(), None).unwrap();
            let snapshot = changed.load().unwrap();
            assert!(!changed
                .incompatible_draft(snapshot.progress.as_ref().unwrap())
                .unwrap());
            assert!(changed.abandon_draft(&snapshot).is_err());
            assert_eq!(
                fs::read_to_string(root.path().join(&store.relative)).unwrap(),
                confirmed
            );
            assert!(!root.path().join(DIRECTORY).exists());
        }
    }
}
