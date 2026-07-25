use crate::UpdateError;
use hive_core::{validate_hive_skill_projection_relative, validate_project_relative};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// Seven days in seconds.
pub const BACKUP_RETENTION_SECONDS: i64 = 7 * 24 * 60 * 60;

/// One exact backed-up entry.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupEntry {
    /// Consumer-project-relative original path.
    pub path: String,
    /// Ownership class recorded by the updater.
    pub ownership: String,
    /// Original SHA-256 digest, or `null` when absent.
    pub prior_digest: Option<String>,
    /// Original byte length.
    pub prior_length: u64,
    /// Relative path inside the backup directory.
    pub backup_path: String,
}

/// Durable backup metadata. `SQLite` and runtime state are never entries.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupManifest {
    /// Schema generation.
    pub schema_version: u32,
    /// Transaction identifier.
    pub transaction_id: String,
    /// Installed source version.
    pub source_version: String,
    /// Candidate target version.
    pub target_version: String,
    /// Unix creation time.
    pub created_at_unix: i64,
    /// Exact seven-day expiry.
    pub expires_at_unix: i64,
    /// Canonical tree digest before update.
    pub tree_digest: String,
    /// Sorted exact entries.
    pub entries: Vec<BackupEntry>,
    /// Digest over the manifest without this field.
    pub manifest_digest: String,
}

/// Retention result for a validated backup directory.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RetentionDecision {
    /// Preserve the directory.
    Keep,
    /// Remove only after validating that no active journal references it.
    Prune,
}

/// Return whether an exact target-relative path may enter an update backup.
///
/// Derived databases, WAL sidecars, caches, runtime state, and backups are
/// intentionally excluded.
#[must_use]
pub fn backup_path_is_allowed(path: &str) -> bool {
    let target_relative = Path::new(path);
    if validate_project_relative(target_relative).is_err()
        && validate_hive_skill_projection_relative(target_relative).is_err()
    {
        return false;
    }
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    if normalized == ".hive/index"
        || normalized.starts_with(".hive/index/")
        || normalized == ".hive/runtime"
        || normalized.starts_with(".hive/runtime/")
        || normalized == ".hive/backups"
        || normalized.starts_with(".hive/backups/")
        || normalized == ".omx"
        || normalized.starts_with(".omx/")
        || normalized == ".omc"
        || normalized.starts_with(".omc/")
    {
        return false;
    }
    !matches!(normalized.rsplit('/').next(), Some(name) if is_derived_file(name))
}

/// Return the exact safe relative storage path for one allowed target path.
///
/// Host discovery namespace components are encoded below `host-skills` rather
/// than reproduced literally inside `.hive/backups`.
#[must_use]
pub fn backup_storage_path(path: &str) -> Option<String> {
    if !backup_path_is_allowed(path) {
        return None;
    }
    let normalized = path.replace('\\', "/");
    let storage = if let Some(suffix) = normalized.strip_prefix(".agents/skills/") {
        format!("files/host-skills/agents/{suffix}")
    } else if let Some(suffix) = normalized.strip_prefix(".claude/skills/") {
        format!("files/host-skills/claude/{suffix}")
    } else {
        format!("files/{normalized}")
    };
    validate_project_relative(Path::new(&storage))
        .is_ok()
        .then_some(storage)
}

fn is_derived_file(name: &str) -> bool {
    let path = Path::new(name);
    path.extension().is_some_and(|extension| {
        ["sqlite", "sqlite3", "db"]
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    }) || ["-wal", "-shm", "-journal"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

/// Validate manifest ordering and compute its retention decision.
pub fn retention_decision(
    manifest: &BackupManifest,
    now_unix: i64,
    active_transaction_ids: &BTreeSet<String>,
) -> Result<RetentionDecision, UpdateError> {
    if manifest.schema_version != 1
        || manifest.transaction_id.is_empty()
        || manifest.expires_at_unix
            != manifest
                .created_at_unix
                .checked_add(BACKUP_RETENTION_SECONDS)
                .ok_or_else(|| UpdateError::Input("backup expiry overflow".to_owned()))?
    {
        return Err(UpdateError::Input(
            "backup manifest violates retention metadata".to_owned(),
        ));
    }
    let mut previous = None;
    for entry in &manifest.entries {
        if !backup_path_is_allowed(&entry.path)
            || backup_storage_path(&entry.path).as_deref() != Some(entry.backup_path.as_str())
        {
            return Err(UpdateError::Input(
                "backup manifest contains an excluded path".to_owned(),
            ));
        }
        if previous.is_some_and(|previous: &str| previous >= entry.path.as_str()) {
            return Err(UpdateError::Input(
                "backup entries must be strictly sorted and unique".to_owned(),
            ));
        }
        previous = Some(entry.path.as_str());
    }
    if active_transaction_ids.contains(&manifest.transaction_id)
        || now_unix <= manifest.expires_at_unix
        || now_unix < manifest.created_at_unix
    {
        return Ok(RetentionDecision::Keep);
    }
    Ok(RetentionDecision::Prune)
}

/// Return transaction ids whose validated backups are older than seven days.
///
/// # Errors
///
/// Returns an error when any candidate manifest violates the backup contract.
pub fn backups_to_prune(
    manifests: &[BackupManifest],
    now_unix: i64,
    active_transaction_ids: &BTreeSet<String>,
) -> Result<Vec<String>, UpdateError> {
    let mut result = Vec::new();
    for manifest in manifests {
        if retention_decision(manifest, now_unix, active_transaction_ids)?
            == RetentionDecision::Prune
        {
            result.push(manifest.transaction_id.clone());
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(created: i64) -> BackupManifest {
        BackupManifest {
            schema_version: 1,
            transaction_id: "txn-1".to_owned(),
            source_version: "0.6.0".to_owned(),
            target_version: "0.7.0".to_owned(),
            created_at_unix: created,
            expires_at_unix: created + BACKUP_RETENTION_SECONDS,
            tree_digest: format!("sha256:{}", "a".repeat(64)),
            entries: vec![BackupEntry {
                path: ".hive/config/harness.toml".to_owned(),
                ownership: "managed".to_owned(),
                prior_digest: Some(format!("sha256:{}", "b".repeat(64))),
                prior_length: 10,
                backup_path: "files/.hive/config/harness.toml".to_owned(),
            }],
            manifest_digest: format!("sha256:{}", "c".repeat(64)),
        }
    }

    #[test]
    fn sqlite_runtime_backups_and_foreign_orchestration_are_excluded() {
        for path in [
            ".hive/index/hive.sqlite3",
            ".hive/index/hive.sqlite3-wal",
            ".hive/runtime/journal.json",
            ".hive/backups/old/file",
            ".omx/state.json",
            ".omc/state.json",
        ] {
            assert!(!backup_path_is_allowed(path), "{path}");
        }
        assert!(backup_path_is_allowed(".hive/knowledge/Wiki/index.md"));
        assert!(backup_path_is_allowed(
            ".agents/skills/hive-update/SKILL.md"
        ));
        assert!(!backup_path_is_allowed(
            "files/.agents/skills/hive-update/SKILL.md"
        ));
        assert!(!backup_path_is_allowed(".agents/foreign.txt"));
        assert_eq!(
            backup_storage_path(".agents/skills/hive-update/SKILL.md").as_deref(),
            Some("files/host-skills/agents/hive-update/SKILL.md")
        );
    }

    #[test]
    fn exact_seven_day_boundary_is_retained_and_older_is_pruned() {
        let manifest = manifest(1_000);
        let active = BTreeSet::new();
        assert_eq!(
            retention_decision(&manifest, 1_000 + BACKUP_RETENTION_SECONDS, &active)
                .expect("decision"),
            RetentionDecision::Keep
        );
        assert_eq!(
            retention_decision(&manifest, 1_001 + BACKUP_RETENTION_SECONDS, &active)
                .expect("decision"),
            RetentionDecision::Prune
        );
    }

    #[test]
    fn active_transaction_backup_is_never_pruned() {
        let manifest = manifest(1_000);
        let active = BTreeSet::from(["txn-1".to_owned()]);
        assert_eq!(
            retention_decision(&manifest, 1_001 + BACKUP_RETENTION_SECONDS, &active)
                .expect("decision"),
            RetentionDecision::Keep
        );
    }
}
