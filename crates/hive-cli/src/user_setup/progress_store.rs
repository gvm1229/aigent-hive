//! Resume data lives in the user root and is bound to one explicit setup target.

use super::{parse_progress_file, SetupError, MAX_USER_SETUP_BYTES, USER_SETUP_RELATIVE};
use crate::user_install::{
    open_canonical_user_root_for_setup, read_user_setup_file, replace_user_setup_file,
};
use cap_std::fs::Dir;
use hive_core::{preferences::PreferenceScope, sha256_digest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

mod history;

const LEGACY_PROGRESS: &str = ".hive/config/user-setup-progress.yml";
const USER_PROGRESS: &str = ".hive/config/setup-progress/user.yml";
const QUESTION_REVISION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TargetBinding {
    pub scope: PreferenceScope,
    pub target_id: String,
    pub path_digest: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ProgressPhase {
    AwaitingAnswers,
    Previewed,
    Applying,
    RecoverableFailure,
    Complete,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SetupProgress {
    pub schema_version: u32,
    pub answer_schema_version: u32,
    pub question_revision: u32,
    pub catalog_digest: String,
    pub binding: TargetBinding,
    pub base_config_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_config_mode: Option<u32>,
    pub operation_id: String,
    pub phase: ProgressPhase,
    pub step: String,
    pub answers: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_section: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub answered_questions: std::collections::BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_migration: Option<LegacyMigration>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyMigration {
    pub migration_id: String,
    pub monitor_choice: crate::usage_monitor::SavedMonitorChoice,
}

pub(super) struct ProgressSnapshot {
    pub progress: Option<SetupProgress>,
    expected: Option<Vec<u8>>,
    legacy: Option<Vec<u8>>,
}

#[derive(Debug)]
pub(super) struct ProgressStore {
    root: Dir,
    config_root: Dir,
    config_relative: PathBuf,
    pub binding: TargetBinding,
    pub relative: PathBuf,
    base_config_digest: String,
    base_config_mode: Option<u32>,
    catalog_digest: String,
}

impl ProgressStore {
    pub fn open(user_root: &Path, target: Option<&Path>) -> Result<Self, SetupError> {
        let (user_path, root) =
            open_canonical_user_root_for_setup(user_root).map_err(SetupError::Conflict)?;
        let (scope, path, config_root, config_relative) = if let Some(target) = target {
            hive_core::ensure_consumer_target(target)
                .map_err(|error| SetupError::Input(error.to_string()))?;
            let (path, project) =
                open_canonical_user_root_for_setup(target).map_err(SetupError::Conflict)?;
            (
                PreferenceScope::Project,
                path,
                project,
                PathBuf::from(".hive/setup-answers.yml"),
            )
        } else {
            let config_root = root.try_clone().map_err(|error| {
                SetupError::Internal(format!("cannot retain setup root: {error}"))
            })?;
            (
                PreferenceScope::User,
                user_path,
                config_root,
                PathBuf::from(USER_SETUP_RELATIVE),
            )
        };
        let base = read_user_setup_file(&config_root, &config_relative, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        let base_config_mode = if base.is_some() {
            crate::user_install::host_policy_mode(&config_root, &config_relative)
                .map_err(SetupError::Conflict)?
        } else {
            None
        };
        // Canonical path bytes avoid lossy display strings and contain no persisted private path.
        let path_digest = sha256_digest(path.as_os_str().as_encoded_bytes());
        let target_id = sha256_digest(format!("{scope:?}:{path_digest}").as_bytes());
        let relative = match scope {
            PreferenceScope::User => PathBuf::from(USER_PROGRESS),
            PreferenceScope::Project => Path::new(".hive/config/setup-progress/projects")
                .join(format!("{}.yml", &path_digest[7..])),
        };
        Ok(Self {
            root,
            config_root,
            config_relative,
            binding: TargetBinding {
                scope,
                target_id,
                path_digest,
            },
            relative,
            base_config_digest: optional_digest(base.as_deref()),
            base_config_mode,
            catalog_digest: catalog_digest(scope),
        })
    }

    pub fn load(&self) -> Result<ProgressSnapshot, SetupError> {
        let expected = self.read(&self.relative)?;
        let legacy = if self.binding.scope == PreferenceScope::User {
            self.read(Path::new(LEGACY_PROGRESS))?
        } else {
            None
        };
        let progress = if let Some(bytes) = &expected {
            let progress: SetupProgress = serde_yaml::from_slice(bytes).map_err(|error| {
                SetupError::Verification(format!("invalid scoped setup progress: {error}"))
            })?;
            self.validate(&progress)?;
            Some(progress)
        } else if let Some(bytes) = &legacy {
            let old = parse_progress_file(bytes)?;
            // Read compatibility only: the predecessor is retained until an explicit clear.
            Some(self.new_progress(old.step, old.answers)?)
        } else {
            None
        };
        Ok(ProgressSnapshot {
            progress,
            expected,
            legacy,
        })
    }

    pub fn new_progress(&self, step: String, answers: Value) -> Result<SetupProgress, SetupError> {
        let mut nonce = [0_u8; 16];
        getrandom::fill(&mut nonce).map_err(|error| {
            SetupError::Internal(format!("cannot identify setup operation: {error}"))
        })?;
        let operation_id = sha256_digest(&nonce)[7..].to_owned();
        let progress = SetupProgress {
            schema_version: 2,
            answer_schema_version: 2,
            question_revision: QUESTION_REVISION,
            catalog_digest: self.catalog_digest.clone(),
            binding: self.binding.clone(),
            base_config_digest: self.base_config_digest.clone(),
            base_config_mode: self.base_config_mode,
            operation_id,
            phase: ProgressPhase::AwaitingAnswers,
            step,
            answers,
            setup_mode: None,
            selected_section: None,
            answered_questions: std::collections::BTreeMap::new(),
            legacy_migration: None,
        };
        self.validate(&progress)?;
        Ok(progress)
    }

    pub fn save(
        &self,
        snapshot: &ProgressSnapshot,
        progress: &SetupProgress,
    ) -> Result<ProgressSnapshot, SetupError> {
        self.validate(progress)?;
        let current = read_user_setup_file(
            &self.config_root,
            &self.config_relative,
            MAX_USER_SETUP_BYTES,
        )
        .map_err(SetupError::Conflict)?;
        if progress.base_config_digest != self.base_config_digest
            || progress.base_config_mode != self.base_config_mode
            || progress.base_config_digest != optional_digest(current.as_deref())
            || progress.catalog_digest != self.catalog_digest
            || progress.question_revision != QUESTION_REVISION
        {
            return Err(SetupError::Conflict(
                "setup inputs or questions changed; inspect pending answers before resuming"
                    .to_owned(),
            ));
        }
        let desired = serde_yaml::to_string(progress).map_err(|error| {
            SetupError::Internal(format!("cannot serialize setup progress: {error}"))
        })?;
        if desired.len() > usize::try_from(MAX_USER_SETUP_BYTES).unwrap_or(usize::MAX) {
            return Err(SetupError::Input(
                "setup progress exceeds the size limit".to_owned(),
            ));
        }
        replace_user_setup_file(
            &self.root,
            &self.relative,
            snapshot.expected.as_deref(),
            Some(desired.as_bytes()),
        )
        .map_err(SetupError::Conflict)?;
        Ok(ProgressSnapshot {
            progress: Some(progress.clone()),
            expected: Some(desired.into_bytes()),
            legacy: snapshot.legacy.clone(),
        })
    }

    pub fn clear(&self, snapshot: &ProgressSnapshot) -> Result<Vec<String>, SetupError> {
        let mut changed = Vec::new();
        // Clear the fallback first, so a successful clear cannot resurrect old answers.
        if let Some(bytes) = &snapshot.legacy {
            replace_user_setup_file(&self.root, Path::new(LEGACY_PROGRESS), Some(bytes), None)
                .map_err(SetupError::Conflict)?;
            changed.push(LEGACY_PROGRESS.to_owned());
        }
        if let Some(bytes) = &snapshot.expected {
            replace_user_setup_file(&self.root, &self.relative, Some(bytes), None)
                .map_err(SetupError::Conflict)?;
            changed.push(self.relative.to_string_lossy().into_owned());
        }
        Ok(changed)
    }

    pub fn resume_ready(&self, progress: &SetupProgress) -> bool {
        progress.base_config_digest == self.base_config_digest
            && progress.base_config_mode == self.base_config_mode
            && progress.catalog_digest == self.catalog_digest
            && progress.question_revision == QUESTION_REVISION
    }

    pub fn matches_config(&self, bytes: Option<&[u8]>) -> bool {
        self.base_config_digest == optional_digest(bytes)
    }

    pub fn record_completion(&self, prior: &SetupProgress) -> Result<(), SetupError> {
        if prior.catalog_digest != self.catalog_digest
            || prior.question_revision != QUESTION_REVISION
        {
            return Err(SetupError::Conflict(
                "setup questions changed before completion".into(),
            ));
        }
        let snapshot = self.load()?;
        if snapshot.progress.as_ref().is_some_and(|current| {
            current.phase == ProgressPhase::Complete
                && current.operation_id == prior.operation_id
                && self.resume_ready(current)
        }) {
            // A worker may have durably completed before its parent receives the reply.
            return history::confirm_completion(self, prior);
        }
        if snapshot.progress.as_ref() != Some(prior) {
            return Err(SetupError::Conflict(
                "setup progress changed before recording completion".to_owned(),
            ));
        }
        let mut complete = prior.clone();
        super::projection_journal::cleanup(
            &self.root,
            prior,
            self.projection_binding(prior, None)?.as_deref(),
        )?;
        let backup = recovery_path(prior)?;
        if let Some(bytes) = self.read(&backup)? {
            if optional_digest(Some(&bytes)) != prior.base_config_digest {
                return Err(SetupError::Conflict("setup recovery backup changed".into()));
            }
            replace_user_setup_file(&self.root, &backup, Some(&bytes), None)
                .map_err(SetupError::Conflict)?;
        }
        if let Some(bytes) = &snapshot.legacy {
            replace_user_setup_file(&self.root, Path::new(LEGACY_PROGRESS), Some(bytes), None)
                .map_err(SetupError::Conflict)?;
        }
        history::record(self, prior, ProgressPhase::Complete)?;
        history::retain_completed(self, &prior.operation_id)?;
        complete
            .base_config_digest
            .clone_from(&self.base_config_digest);
        complete.base_config_mode = self.base_config_mode;
        complete.phase = ProgressPhase::Complete;
        // Completed progress is a resume receipt, not a second copy of free-form answers.
        complete.answers = serde_json::json!({});
        complete.answered_questions.clear();
        self.save(&snapshot, &complete).map(|_| ())
    }

    pub fn record_applying(&self, prior: &SetupProgress) -> Result<(), SetupError> {
        if self.load()?.progress.as_ref() != Some(prior) || !self.resume_ready(prior) {
            return Err(SetupError::Conflict(
                "setup changed before recording apply intent".to_owned(),
            ));
        }
        history::record(self, prior, ProgressPhase::Applying)
    }

    pub fn require_applying(
        &self,
        operation_id: &str,
        answers: &Value,
    ) -> Result<SetupProgress, SetupError> {
        let progress = self
            .load()?
            .progress
            .ok_or_else(|| SetupError::Conflict("setup apply has no saved operation".into()))?;
        if progress.operation_id != operation_id
            || progress.answers != *answers
            || progress.phase != ProgressPhase::Applying
            || !self.resume_ready(&progress)
        {
            return Err(SetupError::Conflict("setup apply operation changed".into()));
        }
        self.record_applying(&progress)?;
        Ok(progress)
    }

    pub fn record_failure(&self, prior: &SetupProgress) -> Result<(), SetupError> {
        history::record(self, prior, ProgressPhase::RecoverableFailure)
    }

    pub fn projection_binding(
        &self,
        progress: &SetupProgress,
        desired: Option<&str>,
    ) -> Result<Option<String>, SetupError> {
        history::projection_binding(self, progress, desired)
    }

    pub fn questions_match(&self, progress: &SetupProgress) -> bool {
        progress.catalog_digest == self.catalog_digest
            && progress.question_revision == QUESTION_REVISION
    }

    fn read(&self, path: &Path) -> Result<Option<Vec<u8>>, SetupError> {
        read_user_setup_file(&self.root, path, MAX_USER_SETUP_BYTES).map_err(SetupError::Conflict)
    }

    fn validate(&self, progress: &SetupProgress) -> Result<(), SetupError> {
        if progress.legacy_migration.as_ref().is_some_and(|migration| {
            self.binding.scope != PreferenceScope::User
                || !valid_hash(&migration.migration_id)
                || (migration.monitor_choice.enabled && !migration.monitor_choice.configured)
                || (migration.monitor_choice.config_digest != "absent"
                    && !valid_hash(&migration.monitor_choice.config_digest))
        }) {
            return Err(SetupError::Verification(
                "invalid legacy setup binding".into(),
            ));
        }
        if progress.schema_version != 2
            || progress.base_config_mode.is_some_and(|mode| mode > 0o7777)
            || progress.answer_schema_version != 2
            || progress.question_revision == 0
            || progress
                .setup_mode
                .as_deref()
                .is_some_and(|mode| !matches!(mode, "expedited" | "custom"))
            || progress.binding != self.binding
            || !valid_hash(&progress.catalog_digest)
            || !valid_hash(&progress.base_config_digest)
            || progress.operation_id.len() != 64
            || !progress
                .operation_id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(SetupError::Verification(
                "setup progress has an invalid scope or revision".to_owned(),
            ));
        }
        super::validate_progress_step(&progress.step)?;
        if let Some(section) = &progress.selected_section {
            super::validate_progress_step(section)?;
        }
        for (question, revision) in &progress.answered_questions {
            super::validate_progress_step(question)?;
            if *revision == 0 {
                return Err(SetupError::Verification(
                    "invalid answered question revision".to_owned(),
                ));
            }
        }
        super::validate_progress_answers_for_scope(&progress.answers, self.binding.scope)
    }
}

pub(super) fn recovery_path(progress: &SetupProgress) -> Result<PathBuf, SetupError> {
    if progress.operation_id.len() != 64
        || !progress
            .operation_id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(SetupError::Verification(
            "invalid recovery operation".into(),
        ));
    }
    Ok(Path::new(".hive/config/setup-progress/recovery")
        .join(format!("{}.yml", progress.operation_id)))
}

pub(super) fn optional_digest(bytes: Option<&[u8]>) -> String {
    // Absence and a present empty file have distinct fingerprints.
    let mut tagged = vec![u8::from(bytes.is_some())];
    tagged.extend_from_slice(bytes.unwrap_or_default());
    sha256_digest(&tagged)
}

pub(super) fn prepare_config_backup(
    root: &Dir,
    progress: &SetupProgress,
    expected: Option<&[u8]>,
) -> Result<(), SetupError> {
    let current = read_user_setup_file(root, Path::new(USER_SETUP_RELATIVE), MAX_USER_SETUP_BYTES)
        .map_err(SetupError::Conflict)?;
    if current.as_deref() != expected {
        return Err(SetupError::Conflict(
            "settings changed before backup".into(),
        ));
    }
    validate_config_snapshot(root, progress, current.as_deref())?;
    let relative = recovery_path(progress)?;
    if let Some(backup) =
        read_user_setup_file(root, &relative, MAX_USER_SETUP_BYTES).map_err(SetupError::Conflict)?
    {
        if optional_digest(Some(&backup)) != progress.base_config_digest {
            return Err(SetupError::Conflict("setup recovery backup changed".into()));
        }
    } else {
        if optional_digest(expected) != progress.base_config_digest {
            return Err(SetupError::Conflict("setup original is unavailable".into()));
        }
        if let Some(bytes) = expected {
            crate::user_install::replace_host_policy_file(root, &relative, None, Some(bytes), None)
                .map_err(SetupError::Conflict)?;
        }
    }
    validate_config_snapshot(root, progress, current.as_deref())
}

pub(super) fn recover_config(
    root: &Dir,
    store: &ProgressStore,
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    store.projection_binding(progress, None)?;
    let backup = read_user_setup_file(root, &recovery_path(progress)?, MAX_USER_SETUP_BYTES)
        .map_err(SetupError::Conflict)?
        .or(crate::user_install::retained_user_setup_original(
            root,
            Path::new(USER_SETUP_RELATIVE),
        )
        .map_err(SetupError::Conflict)?);
    let before = match backup {
        Some(bytes) if optional_digest(Some(&bytes)) == progress.base_config_digest => Some(bytes),
        Some(_) => return Err(SetupError::Conflict("setup recovery backup changed".into())),
        None if progress.base_config_digest == optional_digest(None) => None,
        None => {
            let current =
                read_user_setup_file(root, Path::new(USER_SETUP_RELATIVE), MAX_USER_SETUP_BYTES)
                    .map_err(SetupError::Conflict)?;
            if current.as_deref() == Some(candidate_config(progress)?.as_slice()) {
                // Only complete the published candidate. Unexpected retained prior bytes
                // are rejected when the original backup has already been cleaned up.
                crate::user_install::recover_user_setup_file(
                    root,
                    Path::new(USER_SETUP_RELATIVE),
                    "absent",
                    &sha256_digest(current.as_deref().expect("present candidate")),
                    progress.base_config_mode,
                )
                .map_err(SetupError::Conflict)?;
                return validate_config_snapshot(root, progress, current.as_deref());
            }
            if optional_digest(current.as_deref()) != progress.base_config_digest {
                return Err(SetupError::Conflict(
                    "setup original is unavailable for recovery".into(),
                ));
            }
            current
        }
    };
    let desired = candidate_config(progress)?;
    crate::user_install::recover_user_setup_file(
        root,
        Path::new(USER_SETUP_RELATIVE),
        &before
            .as_deref()
            .map_or_else(|| "absent".into(), sha256_digest),
        &sha256_digest(&desired),
        progress.base_config_mode,
    )
    .map_err(SetupError::Conflict)?;
    let current = read_user_setup_file(root, Path::new(USER_SETUP_RELATIVE), MAX_USER_SETUP_BYTES)
        .map_err(SetupError::Conflict)?;
    validate_config_snapshot(root, progress, current.as_deref())
}

fn candidate_config(progress: &SetupProgress) -> Result<Vec<u8>, SetupError> {
    let answers = serde_json::to_vec(&progress.answers)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    super::canonical_config(&super::parse_and_validate_config(&answers)?)
}

fn validate_config_snapshot(
    root: &Dir,
    progress: &SetupProgress,
    current: Option<&[u8]>,
) -> Result<(), SetupError> {
    let mode = if optional_digest(current) == progress.base_config_digest {
        progress.base_config_mode
    } else if current == Some(candidate_config(progress)?.as_slice()) {
        if cfg!(unix) {
            Some(0o644)
        } else {
            None
        }
    } else {
        return Err(SetupError::Conflict(
            "settings changed outside the interrupted operation".into(),
        ));
    };
    if current.is_some()
        && crate::user_install::host_policy_mode(root, Path::new(USER_SETUP_RELATIVE))
            .map_err(SetupError::Conflict)?
            != mode
    {
        return Err(SetupError::Conflict(
            "settings permissions changed after preview".into(),
        ));
    }
    Ok(())
}

fn catalog_digest(scope: PreferenceScope) -> String {
    let schema = match scope {
        PreferenceScope::User => super::USER_SETUP_SCHEMA,
        PreferenceScope::Project => include_str!("../../../../schemas/setup-answers.schema.json"),
    };
    sha256_digest(
        format!(
            "{schema}\n{}\n{}",
            super::USER_SETUP_CATALOG,
            super::setup_engine::QUESTIONS
        )
        .as_bytes(),
    )
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn config_fixture() -> (tempfile::TempDir, Dir, SetupProgress, Vec<u8>, Vec<u8>) {
        let root = tempfile::tempdir().unwrap();
        let prior = serde_json::to_vec(&json!({"schema_version":1,"interface_language":"ko",
            "wiki":{"language":"ko"},"profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},
            "selected_hosts":["codex"],"skills":{"mode":"all"},"usage_guard":{"enabled":true,"stop_remaining_percent":5}})).unwrap();
        let path = root.path().join(USER_SETUP_RELATIVE);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, &prior).unwrap();
        let mut config = super::super::parse_and_validate_installed_config(&prior).unwrap();
        config.schema_version = 2;
        config.usage_guard.policy = Some(hive_core::preferences::UsageProtection::default());
        config.usage_guard.popup_enabled = Some(false);
        let desired = super::super::canonical_config(&config).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress(
                "usage-enforcement".into(),
                serde_json::to_value(config).unwrap(),
            )
            .unwrap();
        progress.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_applying(&progress).unwrap();
        let (_, cap) = open_canonical_user_root_for_setup(root.path()).unwrap();
        prepare_config_backup(&cap, &progress, Some(&prior)).unwrap();
        (root, cap, progress, prior, desired)
    }

    #[test]
    fn interrupted_config_claim_recovers_the_original_or_keeps_the_published_opt_out() {
        for published in [false, true] {
            let (root, cap, progress, prior, desired) = config_fixture();
            crate::user_install::interrupt_setup_replace(
                &cap,
                Path::new(USER_SETUP_RELATIVE),
                &prior,
                &desired,
                published,
            );
            let store = ProgressStore::open(root.path(), None).unwrap();
            recover_config(&cap, &store, &progress).unwrap();
            let current = fs::read(root.path().join(USER_SETUP_RELATIVE)).unwrap();
            assert_eq!(
                current,
                if published {
                    desired.clone()
                } else {
                    prior.clone()
                }
            );
            assert!(crate::user_install::retained_user_setup_original(
                &cap,
                Path::new(USER_SETUP_RELATIVE)
            )
            .unwrap()
            .is_none());
            prepare_config_backup(&cap, &progress, Some(&current)).unwrap();
        }
    }

    #[test]
    fn completion_cleanup_gap_does_not_require_reinstalling_valid_saved_settings() {
        let (root, cap, progress, prior, desired) = config_fixture();
        replace_user_setup_file(
            &cap,
            Path::new(USER_SETUP_RELATIVE),
            Some(&prior),
            Some(&desired),
        )
        .unwrap();
        let backup = recovery_path(&progress).unwrap();
        replace_user_setup_file(&cap, &backup, Some(&prior), None).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        recover_config(&cap, &store, &progress).unwrap();
        store.record_completion(&progress).unwrap();
        assert_eq!(
            store.load().unwrap().progress.unwrap().phase,
            ProgressPhase::Complete
        );
        assert_eq!(
            fs::read(root.path().join(USER_SETUP_RELATIVE)).unwrap(),
            desired
        );
    }

    #[test]
    fn changed_config_backup_is_preserved_and_cannot_restore_a_claim() {
        let (root, cap, progress, prior, desired) = config_fixture();
        crate::user_install::interrupt_setup_replace(
            &cap,
            Path::new(USER_SETUP_RELATIVE),
            &prior,
            &desired,
            false,
        );
        let backup = root.path().join(recovery_path(&progress).unwrap());
        fs::write(&backup, b"external backup edit").unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        assert!(recover_config(&cap, &store, &progress).is_err());
        assert!(!root.path().join(USER_SETUP_RELATIVE).exists());
        assert_eq!(fs::read(backup).unwrap(), b"external backup edit");
        assert_eq!(
            crate::user_install::retained_user_setup_original(&cap, Path::new(USER_SETUP_RELATIVE))
                .unwrap(),
            Some(prior)
        );
    }

    #[cfg(unix)]
    #[test]
    fn chmod_after_preview_cannot_be_overwritten_by_setup_recovery() {
        use std::os::unix::fs::PermissionsExt;
        let (root, cap, progress, prior, _) = config_fixture();
        fs::set_permissions(
            root.path().join(USER_SETUP_RELATIVE),
            fs::Permissions::from_mode(0o400),
        )
        .unwrap();
        assert!(prepare_config_backup(&cap, &progress, Some(&prior)).is_err());
        let store = ProgressStore::open(root.path(), None).unwrap();
        assert!(!store.resume_ready(&progress));
        assert!(recover_config(&cap, &store, &progress).is_err());
        assert_eq!(
            fs::metadata(root.path().join(USER_SETUP_RELATIVE))
                .unwrap()
                .permissions()
                .mode()
                & 0o7777,
            0o400
        );
    }

    fn save_answer(store: &ProgressStore, value: Value) -> SetupProgress {
        let snapshot = store.load().unwrap();
        let progress = store.new_progress("profile".to_owned(), value).unwrap();
        store.save(&snapshot, &progress).unwrap();
        progress
    }

    #[test]
    fn global_and_two_project_answers_have_separate_locations_and_bindings() {
        let root = tempfile::tempdir().unwrap();
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let user = ProgressStore::open(root.path(), None).unwrap();
        let a = ProgressStore::open(root.path(), Some(first.path())).unwrap();
        let b = ProgressStore::open(root.path(), Some(second.path())).unwrap();
        let user_record = save_answer(&user, json!({"interface_language":"ko"}));
        let a_record = save_answer(&a, json!({"project_name":"A"}));
        let b_record = save_answer(&b, json!({"project_name":"B"}));
        for (store, expected) in [(&user, user_record), (&a, a_record), (&b, b_record)] {
            assert_eq!(store.load().unwrap().progress.unwrap(), expected);
            assert!(!store.relative.is_absolute());
        }
        assert!(!first.path().join(".hive").exists());
        assert!(!second.path().join(".hive").exists());
        assert!(!root.path().join(USER_SETUP_RELATIVE).exists());
    }

    #[test]
    fn concurrent_progress_saves_preserve_the_first_completed_answer() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        save_answer(&store, json!({"profile":"expedited"}));
        let first = store.load().unwrap();
        let second = store.load().unwrap();
        let mut a = first.progress.as_ref().unwrap().clone();
        a.answers = json!({"profile":"custom"});
        store.save(&first, &a).unwrap();
        let mut b = second.progress.as_ref().unwrap().clone();
        b.answers = json!({"profile":"expedited","interface_language":"en"});
        assert!(store.save(&second, &b).is_err());
        assert_eq!(store.load().unwrap().progress.unwrap(), a);
    }

    #[test]
    fn copied_or_moved_project_cannot_adopt_the_previous_progress() {
        let root = tempfile::tempdir().unwrap();
        let projects = tempfile::tempdir().unwrap();
        let original = projects.path().join("original");
        let moved = projects.path().join("moved");
        fs::create_dir(&original).unwrap();
        let a = ProgressStore::open(root.path(), Some(&original)).unwrap();
        save_answer(&a, json!({"project_name":"A"}));
        let a_relative = a.relative.clone();
        drop(a);
        fs::rename(&original, &moved).unwrap();
        let b = ProgressStore::open(root.path(), Some(&moved)).unwrap();
        assert!(b.load().unwrap().progress.is_none());
        fs::copy(root.path().join(a_relative), root.path().join(&b.relative)).unwrap();
        assert!(b.load().is_err());
    }

    #[test]
    fn changed_base_config_and_question_revision_do_not_overwrite_pending_answers() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let prior = save_answer(&store, json!({"interface_language":"ko"}));
        let snapshot = store.load().unwrap();
        let config = root.path().join(USER_SETUP_RELATIVE);
        fs::write(config, "externally changed\n").unwrap();
        assert!(store.save(&snapshot, &prior).is_err());
        let reopened = ProgressStore::open(root.path(), None).unwrap();
        assert!(!reopened.resume_ready(&prior));
        assert_eq!(reopened.load().unwrap().progress.unwrap(), prior);
        let mut different_questions = prior.clone();
        different_questions.question_revision += 1;
        assert!(reopened.save(&snapshot, &different_questions).is_err());
    }

    #[test]
    fn legacy_progress_is_read_without_writes_and_kept_until_explicit_clear() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(LEGACY_PROGRESS);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = "schema_version: 1\nstep: persona\nanswers:\n  interface_language: ko\n";
        fs::write(&path, original).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let snapshot = store.load().unwrap();
        assert!(!root.path().join(USER_PROGRESS).exists());
        let progress = snapshot.progress.as_ref().unwrap();
        assert_eq!(progress.step, "persona");
        store.save(&snapshot, progress).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        assert_eq!(store.load().unwrap().progress.unwrap(), *progress);
        let snapshot = store.load().unwrap();
        assert_eq!(store.clear(&snapshot).unwrap().len(), 2);
        assert!(store.load().unwrap().progress.is_none());
    }

    #[test]
    fn corrupt_records_and_secret_answers_are_rejected_without_replacing_evidence() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        save_answer(&store, json!({"interface_language":"ko"}));
        assert!(store
            .new_progress("profile".to_owned(), json!({"password":"hidden"}))
            .is_err());
        assert!(store
            .new_progress(
                "profile".to_owned(),
                json!({"usage_guard":{"token":"hidden"}})
            )
            .is_err());
        let path = root.path().join(&store.relative);
        fs::write(&path, "not: a-progress-record\n").unwrap();
        assert!(store.load().is_err());
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "not: a-progress-record\n"
        );
    }

    #[test]
    fn source_workspace_never_becomes_a_project_progress_target() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        fs::write(source.path().join("hive-source.json"), "{}").unwrap();
        assert!(ProgressStore::open(root.path(), Some(source.path())).is_err());
        assert!(!root.path().join(".hive").exists());
    }

    #[cfg(unix)]
    #[test]
    fn linked_progress_parent_cannot_redirect_writes_outside_the_user_root() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join(".hive/config")).unwrap();
        std::os::unix::fs::symlink(
            outside.path(),
            root.path().join(".hive/config/setup-progress"),
        )
        .unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        assert!(store.load().is_err());
        assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
    }
}
