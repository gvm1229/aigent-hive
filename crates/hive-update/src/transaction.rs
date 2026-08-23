use crate::{
    classify_release, observe_surface_delta, select_migration_route,
    validate_cross_major_preservation, verify_release_bundle, BackupEntry, BackupManifest,
    MajorApproval, MigrationKind, PreservationDigest, ReleaseState, ReleaseVerification,
    SemVersion, SurfaceDelta, UpdateError,
};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::{
    is_hive_directive_projection_path, is_hive_skill_projection_path, sha256_digest,
    validate_hive_directive_projection_relative, validate_hive_skill_projection_relative,
    validate_project_relative, SOURCE_MARKER_FILE,
};
use hive_render::{
    execute_release_update_in, shared_marker_foreign_digest_for_path, update_path_is_owned,
    RenderError, SetupChange, SetupMode, SetupOutcome, SetupRequest,
};
#[cfg(test)]
use hive_render::{
    execute_setup, historical_project_upgrade_candidate_in, GlobalProjectPreferences,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::NamedTempFile;

const ANSWERS_PATH: &str = ".hive/setup-answers.yml";
const CAPABILITIES_PATH: &str = ".hive/config/capability-resolution.yml";
const HARNESS_PATH: &str = ".hive/config/harness.toml";
const UPDATE_STATE_PATH: &str = ".hive/config/update-state.json";
const JOURNAL_PATH: &str = ".hive/runtime/update-journal.json";
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
const MAX_BACKUP_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_BACKUP_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
const HEX: &[u8; 16] = b"0123456789abcdef";
const SHARED_INDEX_MINIMUM_VERSION: SemVersion = SemVersion {
    major: 0,
    minor: 8,
    patch: 0,
};
const PROJECT_INDEX_FILES: &[&str] = &[
    ".hive/index/hive.sqlite3",
    ".hive/index/hive.sqlite3-wal",
    ".hive/index/hive.sqlite3-shm",
    ".hive/index/hive.sqlite3-journal",
    ".hive/index/.stale",
];
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(all(test, unix))]
std::thread_local! {
    static DIRECTORY_SYNC_EVENTS: std::cell::RefCell<Vec<PathBuf>> =
        const { std::cell::RefCell::new(Vec::new()) };
}
const CANONICAL_BACKUP_ROOTS: &[&str] = &[
    ".hive/setup-answers.yml",
    ".hive/config",
    ".hive/team",
    ".hive/runs",
    ".hive/knowledge",
];

/// Update operation mode.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UpdateMode {
    /// Verify and produce a stable plan without target mutation.
    DryRun,
    /// Back up, journal, activate, validate, and rebuild the disposable index.
    Apply,
}

/// Offline update request.
#[derive(Debug)]
pub struct UpdateRequest<'a> {
    /// Installed consumer project.
    pub target: &'a Path,
    /// Extracted local release bundle.
    pub repository: &'a Path,
    /// Verification clock supplied by the CLI.
    pub now_unix: i64,
    /// Dry-run or apply.
    pub mode: UpdateMode,
    /// Exact user-supplied breaking target. Never inferred by Hive.
    pub exact_major_target: Option<SemVersion>,
    /// Exact optional breaking-release authority.
    pub major_approval: Option<&'a MajorApproval>,
}

/// Persisted accepted release identity.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateState {
    /// Schema generation.
    pub schema_version: u32,
    /// Installed product version.
    pub product_version: String,
    /// Last accepted release manifest digest.
    pub release_manifest_digest: String,
    /// Last accepted release used for local downgrade refusal.
    pub accepted_release: ReleaseState,
}

/// Successful update result.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UpdateOutcome {
    /// Exact manifest-owned paths planned or changed.
    pub changed_paths: Vec<String>,
    /// Stable digest binding release, installed baseline, and exact mutations.
    pub plan_digest: String,
    /// Stable digest of independently observed compatibility and preservation.
    pub compatibility_report_digest: String,
    /// Digest of the signed migration table bound into major confirmation.
    pub migration_table_digest: String,
    /// Installed source version.
    pub source_version: String,
    /// Candidate target version.
    pub target_version: String,
    /// Exact compiled migration implementation selected by the signed route.
    pub migration_id: String,
    /// Backup transaction id on apply.
    pub backup_id: Option<String>,
    /// Logical rebuilt index digest on apply.
    pub index_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InstalledHarness {
    harness_version: String,
    source_release_version: String,
    usage_stop_remaining_percent: u8,
    #[serde(default)]
    preference_provenance: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JournalState {
    Prepared,
    Committed,
    NeedsRecovery,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalChange {
    path: String,
    before_digest: Option<String>,
    after_digest: Option<String>,
    backup_path: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateJournal {
    schema_version: u32,
    transaction_id: String,
    state: JournalState,
    source_version: String,
    target_version: String,
    release_manifest_digest: String,
    plan_digest: String,
    backup_manifest_path: String,
    changes: Vec<JournalChange>,
    next_state: UpdateState,
    journal_digest: String,
}

#[derive(Serialize)]
struct PlanDigestPayload<'a> {
    schema_version: u32,
    source_version: &'a str,
    target_version: &'a str,
    release_manifest_digest: &'a str,
    tree_digest: &'a str,
    changes: Vec<PlanDigestChange<'a>>,
}

#[derive(Serialize)]
struct PlanDigestChange<'a> {
    path: &'a str,
    before_digest: Option<&'a str>,
    after_digest: Option<&'a str>,
    foreign_before_digest: Option<&'a str>,
    foreign_after_digest: Option<&'a str>,
}

struct PreparedUpdate {
    installed: InstalledHarness,
    verified: ReleaseVerification,
    answers: NamedTempFile,
    capabilities: NamedTempFile,
    dry_run: SetupOutcome,
    plan_digest: String,
    compatibility_report_digest: String,
    preservation_before: Option<Vec<PreservationDigest>>,
    migration_kind: MigrationKind,
    migration_id: String,
}

struct TargetRoot<'a> {
    dir: &'a Dir,
}

struct ValidatedTransition {
    migration_kind: MigrationKind,
    migration_id: String,
    observed: SurfaceDelta,
}

#[derive(Serialize)]
struct CompatibilityReportPayload<'a> {
    schema_version: u32,
    source_version: &'a str,
    target_version: &'a str,
    observed_surface_delta: SurfaceDelta,
    migration_id: &'a str,
    surface_inventory_digest: &'a str,
    migration_table_digest: &'a str,
    preservation_digest: &'a str,
}

/// Verify and optionally activate an offline release.
///
/// Release authentication, rollback, classification, migration routing, and
/// renderer dry-run complete before any target write.
///
/// # Errors
///
/// Returns an error when release verification, compatibility, staging,
/// activation, or durable recovery cannot complete safely.
pub fn execute_update(request: &UpdateRequest<'_>) -> Result<UpdateOutcome, UpdateError> {
    let target_dir = open_target_capability(request.target)?;
    execute_update_in(&target_dir, request)
}

/// Verify and optionally activate an offline release through an already-pinned target.
///
/// # Errors
///
/// Returns an error when release verification, compatibility, staging,
/// activation, or durable recovery cannot complete safely.
pub fn execute_update_in(
    target_dir: &Dir,
    request: &UpdateRequest<'_>,
) -> Result<UpdateOutcome, UpdateError> {
    let target = TargetRoot { dir: target_dir };
    ensure_pinned_consumer_target(&target)?;
    if journal_exists(&target)? {
        return Err(UpdateError::RecoveryRequired(
            "update recovery required before another dry-run or apply; run hive update --recover"
                .to_owned(),
        ));
    }
    let prepared = prepare_update(&target, request)?;
    if request.mode == UpdateMode::DryRun {
        return Ok(UpdateOutcome {
            changed_paths: prepared.dry_run.changed_paths,
            plan_digest: prepared.plan_digest,
            compatibility_report_digest: prepared.compatibility_report_digest,
            migration_table_digest: prepared.verified.manifest.migration_table_digest,
            source_version: prepared.installed.harness_version,
            target_version: prepared.verified.manifest.release_version,
            migration_id: prepared.migration_id,
            backup_id: None,
            index_digest: None,
        });
    }
    activate_update(&target, request, prepared)
}

fn prepare_update(
    target: &TargetRoot<'_>,
    request: &UpdateRequest<'_>,
) -> Result<PreparedUpdate, UpdateError> {
    let previous_state = read_update_state(target)?;
    let verified = verify_release_bundle(
        request.repository,
        previous_state.as_ref().map(|state| &state.accepted_release),
    )?;
    let installed = read_installed_harness(target)?;
    if installed.harness_version != installed.source_release_version {
        return Err(UpdateError::Verification(
            "installed harness and source release versions differ".to_owned(),
        ));
    }
    if !(1..=99).contains(&installed.usage_stop_remaining_percent) {
        return Err(UpdateError::Verification(
            "installed usage threshold must be an integer from 1 through 99".to_owned(),
        ));
    }
    require_operational_update_prerequisites(&installed, &verified.manifest.release_version)?;
    let transition = validate_update_transition(&installed, &verified, request.exact_major_target)?;
    let preservation_before = if transition.migration_kind == MigrationKind::CrossMajor {
        Some(snapshot_protected_tree(target)?)
    } else {
        None
    };

    let capabilities_json = installed_capabilities_json(target)?;
    let answers_yaml = installed_answers_yaml(target, installed.usage_stop_remaining_percent)?;
    let mut answers = NamedTempFile::new()
        .map_err(|error| UpdateError::Internal(format!("cannot stage setup answers: {error}")))?;
    answers
        .write_all(&answers_yaml)
        .and_then(|()| answers.as_file().sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist setup answers: {error}")))?;
    let mut capabilities = NamedTempFile::new()
        .map_err(|error| UpdateError::Internal(format!("cannot stage capabilities: {error}")))?;
    capabilities
        .write_all(&capabilities_json)
        .and_then(|()| capabilities.as_file().sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist capabilities: {error}")))?;
    let dry_run = execute_release_update_in(
        &SetupRequest {
            target: request.target,
            answers: answers.path(),
            capabilities: capabilities.path(),
            mode: SetupMode::DryRun,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        },
        target.dir,
        &installed.harness_version,
    )
    .map_err(map_render_error)?;
    let plan_digest = plan_digest(
        &installed.harness_version,
        &verified.manifest.release_version,
        &verified.manifest_digest,
        &dry_run.tree_digest,
        &dry_run.changes,
    )?;
    let preservation_digest = match preservation_before.as_deref() {
        Some(before) => {
            let after = planned_cross_major_successor(before, &dry_run.changes)?;
            let mutable = cross_major_mutable_paths(&dry_run.changes)?;
            validate_cross_major_preservation(before, &after, &mutable)?;
            preservation_report_digest(before, &after, &mutable)?
        }
        None => sha256_digest(b"no-cross-major-preservation-report"),
    };
    let compatibility_report_digest = compatibility_report_digest(
        &installed.harness_version,
        &verified,
        &transition,
        &preservation_digest,
    )?;
    validate_major_binding(
        request.mode,
        &installed.harness_version,
        &verified,
        request.exact_major_target,
        request.major_approval,
        &plan_digest,
        &compatibility_report_digest,
        transition.migration_kind,
    )?;
    Ok(PreparedUpdate {
        installed,
        verified,
        answers,
        capabilities,
        dry_run,
        plan_digest,
        compatibility_report_digest,
        preservation_before,
        migration_kind: transition.migration_kind,
        migration_id: transition.migration_id,
    })
}

fn require_operational_update_prerequisites(
    installed: &InstalledHarness,
    target_version: &str,
) -> Result<(), UpdateError> {
    let target_version: SemVersion = target_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    if target_version >= SHARED_INDEX_MINIMUM_VERSION && installed.preference_provenance.is_none() {
        return Err(UpdateError::Unsupported(
            "Hive 0.8+ project update requires validated user setup and transactional shared-registry binding; run `hive setup --scope user` and connected project setup before retrying"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_update_transition(
    installed: &InstalledHarness,
    verified: &ReleaseVerification,
    exact_major_target: Option<SemVersion>,
) -> Result<ValidatedTransition, UpdateError> {
    let source_version: SemVersion = installed
        .harness_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    let target_version: SemVersion = verified
        .manifest
        .release_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    let compiled_version: SemVersion = env!("CARGO_PKG_VERSION")
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Internal(error.to_string()))?;
    if target_version != compiled_version {
        return Err(UpdateError::Verification(format!(
            "release target {target_version} differs from running updater {compiled_version}"
        )));
    }
    if source_version > target_version {
        return Err(UpdateError::Compatibility(
            "release would downgrade the installed harness".to_owned(),
        ));
    }
    let route = select_migration_route(&verified.migration_table, source_version)?;
    let classification_baseline = if source_version.major == target_version.major {
        route
            .from_max
            .parse()
            .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?
    } else {
        source_version
    };
    let observed = observe_surface_delta(classification_baseline, &verified.surface_inventory)?;
    classify_release(
        classification_baseline,
        target_version,
        verified.manifest.classification,
        observed,
        exact_major_target,
    )?;
    if source_version.major == target_version.major && route.kind != crate::MigrationKind::SameMajor
    {
        return Err(UpdateError::Compatibility(
            "same-major update selected a cross-major migration".to_owned(),
        ));
    }
    if source_version.major != target_version.major
        && route.kind != crate::MigrationKind::CrossMajor
    {
        return Err(UpdateError::Compatibility(
            "cross-major update selected a same-major migration".to_owned(),
        ));
    }
    Ok(ValidatedTransition {
        migration_kind: route.kind,
        migration_id: route.migration_id.clone(),
        observed,
    })
}

fn compatibility_report_digest(
    source_version: &str,
    verified: &ReleaseVerification,
    transition: &ValidatedTransition,
    preservation_digest: &str,
) -> Result<String, UpdateError> {
    let payload = CompatibilityReportPayload {
        schema_version: 1,
        source_version,
        target_version: &verified.manifest.release_version,
        observed_surface_delta: transition.observed,
        migration_id: &transition.migration_id,
        surface_inventory_digest: &verified.manifest.surface_inventory_digest,
        migration_table_digest: &verified.manifest.migration_table_digest,
        preservation_digest,
    };
    let bytes = serde_json_canonicalizer::to_vec(&payload).map_err(|error| {
        UpdateError::Internal(format!("cannot canonicalize compatibility report: {error}"))
    })?;
    Ok(sha256_digest(&bytes))
}

#[allow(clippy::too_many_arguments)]
fn validate_major_binding(
    mode: UpdateMode,
    source_version: &str,
    verified: &ReleaseVerification,
    exact_major_target: Option<SemVersion>,
    approval: Option<&MajorApproval>,
    plan_digest: &str,
    compatibility_report_digest: &str,
    migration_kind: MigrationKind,
) -> Result<(), UpdateError> {
    if migration_kind != MigrationKind::CrossMajor {
        if exact_major_target.is_some() || approval.is_some() {
            return Err(UpdateError::Compatibility(
                "major-release authority was supplied for a same-major update".to_owned(),
            ));
        }
        return Ok(());
    }
    let source: SemVersion = source_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    let target: SemVersion = verified
        .manifest
        .release_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    let Some(approval) = approval else {
        return if mode == UpdateMode::DryRun {
            Ok(())
        } else {
            Err(UpdateError::Compatibility(
                "cross-major apply requires the exact dry-run confirmation document".to_owned(),
            ))
        };
    };
    if approval.source_version != source
        || approval.exact_target != target
        || exact_major_target != Some(target)
        || approval.release_plan_digest != plan_digest
        || approval.compatibility_report_digest != compatibility_report_digest
        || approval.migration_table_digest != verified.manifest.migration_table_digest
        || !approval.human_confirmed
        || !crate::version::is_sha256_digest(&approval.confirmation_digest)
    {
        return Err(UpdateError::Compatibility(
            "major confirmation does not bind this source, target, dry-run plan, compatibility report, and signed migration table"
                .to_owned(),
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct PreservationReport<'a> {
    schema_version: u32,
    before: &'a [PreservationDigest],
    after: &'a [PreservationDigest],
    mutable_system_paths: &'a BTreeSet<String>,
}

fn preservation_report_digest(
    before: &[PreservationDigest],
    after: &[PreservationDigest],
    mutable_system_paths: &BTreeSet<String>,
) -> Result<String, UpdateError> {
    let bytes = serde_json_canonicalizer::to_vec(&PreservationReport {
        schema_version: 1,
        before,
        after,
        mutable_system_paths,
    })
    .map_err(|error| {
        UpdateError::Internal(format!("cannot canonicalize preservation report: {error}"))
    })?;
    Ok(sha256_digest(&bytes))
}

fn planned_cross_major_successor(
    before: &[PreservationDigest],
    changes: &[SetupChange],
) -> Result<Vec<PreservationDigest>, UpdateError> {
    let mut after: BTreeMap<String, PreservationDigest> = before
        .iter()
        .cloned()
        .map(|entry| (entry.path.clone(), entry))
        .collect();
    for change in changes {
        if change.path == "AGENTS.md" {
            let prior = change.foreign_before_digest.as_deref().ok_or_else(|| {
                UpdateError::Compatibility(
                    "shared-marker change omits the prior foreign-byte digest".to_owned(),
                )
            })?;
            let successor = change.foreign_after_digest.as_deref().ok_or_else(|| {
                UpdateError::Compatibility(
                    "shared-marker change omits the successor foreign-byte digest".to_owned(),
                )
            })?;
            if prior != successor {
                return Err(UpdateError::Compatibility(
                    "cross-major renderer changed foreign AGENTS.md bytes".to_owned(),
                ));
            }
            let entry = after.get_mut("AGENTS.md").ok_or_else(|| {
                UpdateError::Compatibility(
                    "cross-major preservation baseline omits AGENTS.md".to_owned(),
                )
            })?;
            if entry.digest != prior {
                return Err(UpdateError::Conflict(
                    "AGENTS.md changed after preservation snapshot".to_owned(),
                ));
            }
            successor.clone_into(&mut entry.digest);
        } else if !is_cross_major_system_path(&change.path) {
            return Err(UpdateError::Compatibility(format!(
                "cross-major plan changes protected path: {}",
                change.path
            )));
        }
    }
    Ok(after.into_values().collect())
}

fn cross_major_mutable_paths(changes: &[SetupChange]) -> Result<BTreeSet<String>, UpdateError> {
    let mut mutable = BTreeSet::new();
    for change in changes {
        if change.path == "AGENTS.md" {
            continue;
        }
        if !is_cross_major_system_path(&change.path)
            || !update_path_is_owned(Path::new(&change.path)).map_err(map_render_error)?
        {
            return Err(UpdateError::Compatibility(format!(
                "cross-major plan contains a non-system mutation: {}",
                change.path
            )));
        }
        mutable.insert(change.path.clone());
    }
    Ok(mutable)
}

fn is_cross_major_system_path(path: &str) -> bool {
    matches!(
        path,
        ".hive/.gitignore"
            | ".hive/LICENSE-AIGENT-HIVE.txt"
            | ".hive/README.md"
            | ".hive/config/harness.toml"
            | ".hive/config/active-skills.yml"
            | ".hive/config/capability-resolution.yml"
            | ".hive/config/update-state.json"
    ) || is_hive_skill_projection_path(Path::new(path))
}

fn snapshot_protected_tree(
    target: &TargetRoot<'_>,
) -> Result<Vec<PreservationDigest>, UpdateError> {
    let mut result = Vec::new();
    snapshot_directory(target.dir, Path::new(""), &mut result)?;
    result.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(result)
}

fn snapshot_directory(
    directory: &Dir,
    relative_parent: &Path,
    result: &mut Vec<PreservationDigest>,
) -> Result<(), UpdateError> {
    let mut entries = directory
        .entries()
        .map_err(|error| UpdateError::Internal(format!("cannot scan preservation tree: {error}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            UpdateError::Internal(format!("cannot scan preservation tree: {error}"))
        })?;
    entries.sort_by_key(cap_std::fs::DirEntry::file_name);
    for entry in entries {
        let file_name = entry.file_name();
        let relative = relative_parent.join(&file_name);
        let relative_string = relative.to_string_lossy().replace('\\', "/");
        if preservation_path_is_excluded(&relative_string) {
            continue;
        }
        let metadata = directory.symlink_metadata(&file_name).map_err(|error| {
            UpdateError::Internal(format!(
                "cannot inspect preservation path {relative_string}: {error}"
            ))
        })?;
        if metadata.is_dir() {
            let child = directory.open_dir_nofollow(&file_name).map_err(|error| {
                UpdateError::Conflict(format!(
                    "cannot open preservation directory {relative_string} no-follow: {error}"
                ))
            })?;
            snapshot_directory(&child, &relative, result)?;
            continue;
        }
        if is_cross_major_system_path(&relative_string) {
            continue;
        }
        let digest = if metadata.file_type().is_symlink() {
            let destination = directory.read_link(&file_name).map_err(|error| {
                UpdateError::Internal(format!(
                    "cannot read preservation symlink {relative_string}: {error}"
                ))
            })?;
            sha256_digest(format!("symlink\0{}", destination.to_string_lossy()).as_bytes())
        } else if metadata.is_file() {
            if matches!(
                relative_string.as_str(),
                "AGENTS.md" | "CLAUDE.md" | "GEMINI.md" | ".prettierignore"
            ) {
                let bytes = read_parent_file(directory, &file_name, MAX_BACKUP_FILE_BYTES)?;
                shared_marker_foreign_digest_for_path(Path::new(&relative_string), &bytes)
                    .map_err(map_render_error)?
            } else {
                hash_regular_file(directory, &file_name)?
            }
        } else {
            return Err(UpdateError::Compatibility(format!(
                "cross-major preservation cannot authenticate special file: {relative_string}"
            )));
        };
        result.push(PreservationDigest {
            path: relative_string.clone(),
            kind: preservation_kind(&relative_string).to_owned(),
            digest,
        });
    }
    Ok(())
}

fn preservation_path_is_excluded(path: &str) -> bool {
    [
        ".git",
        ".omx",
        ".omc",
        ".codex",
        ".hive/index",
        ".hive/runtime",
        ".hive/backups",
    ]
    .iter()
    .any(|root| path == *root || path.starts_with(&format!("{root}/")))
}

fn preservation_kind(path: &str) -> &'static str {
    if path == "AGENTS.md"
        || Path::new(path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            && (path.starts_with(".hive/knowledge/")
                || path.starts_with(".hive/team/")
                || path.starts_with(".hive/runs/"))
    {
        "user-markdown-body"
    } else if path == ".hive/setup-answers.yml"
        || path.starts_with(".hive/config/")
        || path.starts_with(".hive/hooks/")
    {
        "preference"
    } else if path == "docs" || path.starts_with("docs/") {
        "document"
    } else {
        "project-file"
    }
}

fn hash_regular_file(parent: &Dir, file_name: &OsStr) -> Result<String, UpdateError> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(file_name, &options)
        .map_err(|error| UpdateError::Internal(format!("cannot hash preserved file: {error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            UpdateError::Internal(format!("cannot hash preserved file: {error}"))
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(encoded)
}

fn activate_update(
    target: &TargetRoot<'_>,
    request: &UpdateRequest<'_>,
    prepared: PreparedUpdate,
) -> Result<UpdateOutcome, UpdateError> {
    let PreparedUpdate {
        installed,
        verified,
        answers,
        capabilities,
        dry_run,
        plan_digest,
        compatibility_report_digest,
        preservation_before,
        migration_kind,
        migration_id,
    } = prepared;
    let transaction_id = transaction_id(&plan_digest, request.now_unix);
    let backup = create_backup(
        target,
        &transaction_id,
        &installed.harness_version,
        &verified.manifest.release_version,
        request.now_unix,
        &dry_run.tree_digest,
        &dry_run.changes,
    )?;
    let next_state = UpdateState {
        schema_version: 1,
        product_version: verified.manifest.release_version.clone(),
        release_manifest_digest: verified.manifest_digest.clone(),
        accepted_release: verified.next_release_state.clone(),
    };
    let mut journal = create_journal(
        &transaction_id,
        &installed,
        &verified,
        &plan_digest,
        &dry_run.changes,
        next_state,
    )?;
    persist_journal(target, &journal)?;

    let applied = execute_release_update_in(
        &SetupRequest {
            target: request.target,
            answers: answers.path(),
            capabilities: capabilities.path(),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        },
        target.dir,
        &installed.harness_version,
    )
    .map_err(|error| {
        let mapped = map_render_error(error);
        let _ = mark_needs_recovery(target, &mut journal);
        mapped
    })?;
    if applied.changes != dry_run.changes || applied.tree_digest != dry_run.tree_digest {
        mark_needs_recovery(target, &mut journal)?;
        return Err(UpdateError::Conflict(
            "setup plan changed between dry-run and activation".to_owned(),
        ));
    }
    verify_after_digests(target, &journal.changes)?;
    if migration_kind == MigrationKind::CrossMajor {
        let before = preservation_before.as_deref().ok_or_else(|| {
            UpdateError::Internal("cross-major preservation baseline is missing".to_owned())
        })?;
        let after = snapshot_protected_tree(target)?;
        let mutable = cross_major_mutable_paths(&applied.changes)?;
        if let Err(error) = validate_cross_major_preservation(before, &after, &mutable) {
            mark_needs_recovery(target, &mut journal)?;
            return Err(error);
        }
    }
    journal.state = JournalState::Committed;
    journal.journal_digest = journal_digest(&journal)?;
    persist_journal(target, &journal)?;
    write_update_state(target, &journal.next_state)?;
    let index_digest = update_project_index_for_version(target, &verified.manifest.release_version)
        .map_err(|error| {
            UpdateError::Verification(format!(
                "successor committed but derived-index maintenance failed: {error}"
            ))
        })?;
    remove_exact_regular_file(target, Path::new(JOURNAL_PATH))?;
    let _pruned_backup_count = prune_expired_backups_in(target, request.now_unix);
    Ok(UpdateOutcome {
        changed_paths: applied.changed_paths,
        plan_digest,
        compatibility_report_digest,
        migration_table_digest: verified.manifest.migration_table_digest,
        source_version: installed.harness_version,
        target_version: verified.manifest.release_version,
        migration_id,
        backup_id: Some(backup.transaction_id),
        index_digest,
    })
}

fn create_journal(
    transaction_id: &str,
    installed: &InstalledHarness,
    verified: &ReleaseVerification,
    plan_digest: &str,
    changes: &[SetupChange],
    next_state: UpdateState,
) -> Result<UpdateJournal, UpdateError> {
    let mut journal = UpdateJournal {
        schema_version: 1,
        transaction_id: transaction_id.to_owned(),
        state: JournalState::Prepared,
        source_version: installed.harness_version.clone(),
        target_version: verified.manifest.release_version.clone(),
        release_manifest_digest: verified.manifest_digest.clone(),
        plan_digest: plan_digest.to_owned(),
        backup_manifest_path: format!(".hive/backups/{transaction_id}/backup-manifest.json"),
        changes: changes
            .iter()
            .map(|change| {
                let backup_path = crate::backup_storage_path(&change.path).ok_or_else(|| {
                    UpdateError::Compatibility(format!(
                        "update change has no safe backup encoding: {}",
                        change.path
                    ))
                })?;
                Ok(JournalChange {
                    path: change.path.clone(),
                    before_digest: change.before_digest.clone(),
                    after_digest: change.after_digest.clone(),
                    backup_path,
                })
            })
            .collect::<Result<Vec<_>, UpdateError>>()?,
        next_state,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest(&journal)?;
    Ok(journal)
}

fn persist_journal(target: &TargetRoot<'_>, journal: &UpdateJournal) -> Result<(), UpdateError> {
    write_atomic_relative(
        target,
        Path::new(JOURNAL_PATH),
        &json_line(journal, "update journal")?,
    )
}

/// Recover an incomplete durable update journal.
///
/// Prepared transactions roll back only when every live path is still either
/// the journaled before or after digest. Committed transactions forward-complete
/// update state and derived-index rebuild.
///
/// # Errors
///
/// Returns an error when the journal is invalid, live bytes conflict with the
/// recorded transaction, or rollback/forward recovery cannot complete.
pub fn recover_update(target: &Path) -> Result<(), UpdateError> {
    let target_dir = open_target_capability(target)?;
    recover_update_in(&target_dir)
}

/// Recover an incomplete update through an already-pinned target capability.
///
/// # Errors
///
/// Returns an error when the journal is invalid, live bytes conflict with the
/// recorded transaction, or rollback/forward recovery cannot complete.
pub fn recover_update_in(target_dir: &Dir) -> Result<(), UpdateError> {
    let target = TargetRoot { dir: target_dir };
    ensure_pinned_consumer_target(&target)?;
    let Some(bytes) = read_optional_bounded(&target, Path::new(JOURNAL_PATH), MAX_CONFIG_BYTES)?
    else {
        return Ok(());
    };
    let journal: UpdateJournal = serde_json::from_slice(&bytes)
        .map_err(|error| UpdateError::Conflict(format!("update journal is invalid: {error}")))?;
    validate_recovery_journal(&target, &journal)?;
    match journal.state {
        JournalState::Committed => {
            verify_after_digests(&target, &journal.changes)?;
            write_update_state(&target, &journal.next_state)?;
            update_project_index_for_version(&target, &journal.target_version).map_err(
                |error| {
                    UpdateError::Verification(format!(
                        "cannot maintain derived index during forward recovery: {error}"
                    ))
                },
            )?;
        }
        JournalState::Prepared | JournalState::NeedsRecovery => {
            rollback_changes(&target, &journal)?;
        }
    }
    remove_exact_regular_file(&target, Path::new(JOURNAL_PATH))
}

fn validate_recovery_journal(
    target: &TargetRoot<'_>,
    journal: &UpdateJournal,
) -> Result<BackupManifest, UpdateError> {
    if journal.schema_version != 1
        || !valid_transaction_directory_name(&journal.transaction_id)
        || journal_digest(journal)? != journal.journal_digest
        || !is_sha256_digest(&journal.release_manifest_digest)
        || !is_sha256_digest(&journal.plan_digest)
        || journal.next_state.schema_version != 1
        || journal.next_state.product_version != journal.target_version
        || journal.next_state.release_manifest_digest != journal.release_manifest_digest
        || journal.next_state.accepted_release.manifest_digest != journal.release_manifest_digest
        || journal.next_state.accepted_release.release_sequence == 0
        || journal.next_state.accepted_release.release_version != journal.target_version
        || journal.changes.is_empty()
        || journal.source_version.parse::<SemVersion>().is_err()
        || journal.target_version.parse::<SemVersion>().is_err()
    {
        return Err(UpdateError::Conflict(
            "update journal violates the durable recovery contract".to_owned(),
        ));
    }
    let expected_manifest_path = format!(
        ".hive/backups/{}/backup-manifest.json",
        journal.transaction_id
    );
    if journal.backup_manifest_path != expected_manifest_path {
        return Err(UpdateError::Conflict(
            "update journal references an unexpected backup manifest".to_owned(),
        ));
    }
    let manifest_bytes = read_required_bounded(
        target,
        Path::new(&journal.backup_manifest_path),
        MAX_CONFIG_BYTES,
    )?;
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| UpdateError::Conflict(format!("backup manifest is invalid: {error}")))?;
    if manifest.schema_version != 1
        || manifest.transaction_id != journal.transaction_id
        || manifest.source_version != journal.source_version
        || manifest.target_version != journal.target_version
        || manifest.expires_at_unix
            != manifest
                .created_at_unix
                .checked_add(crate::BACKUP_RETENTION_SECONDS)
                .ok_or_else(|| UpdateError::Conflict("backup expiry overflows".to_owned()))?
        || !is_sha256_digest(&manifest.tree_digest)
        || backup_manifest_digest(&manifest)? != manifest.manifest_digest
    {
        return Err(UpdateError::Conflict(
            "backup manifest does not authenticate the recovery journal".to_owned(),
        ));
    }
    crate::backup::retention_decision(
        &manifest,
        manifest.created_at_unix,
        &BTreeSet::from([manifest.transaction_id.clone()]),
    )
    .map_err(|error| UpdateError::Conflict(error.to_string()))?;
    let mut paths = BTreeSet::new();
    for change in &journal.changes {
        let path = Path::new(&change.path);
        let expected_backup_path = crate::backup_storage_path(&change.path);
        let digests_valid = change.before_digest.as_deref().is_none_or(is_sha256_digest)
            && change.after_digest.as_deref().is_none_or(is_sha256_digest);
        let entry = manifest
            .entries
            .iter()
            .find(|entry| entry.path == change.path);
        if !paths.insert(change.path.as_str())
            || validate_update_relative(path).is_err()
            || !update_path_is_owned(path).map_err(map_render_error)?
            || expected_backup_path.as_deref() != Some(change.backup_path.as_str())
            || (change.before_digest.is_none() && change.after_digest.is_none())
            || !digests_valid
            || entry.is_none_or(|entry| {
                entry.backup_path != change.backup_path
                    || entry.prior_digest != change.before_digest
                    || !matches!(
                        entry.ownership.as_str(),
                        "manifest-owned" | "canonical-protected"
                    )
            })
        {
            return Err(UpdateError::Conflict(
                "update journal contains an unauthorized or unbound mutation".to_owned(),
            ));
        }
    }
    Ok(manifest)
}

fn create_backup(
    target: &TargetRoot<'_>,
    transaction_id: &str,
    source_version: &str,
    target_version: &str,
    now_unix: i64,
    tree_digest: &str,
    changes: &[SetupChange],
) -> Result<BackupManifest, UpdateError> {
    let root_relative = PathBuf::from(format!(".hive/backups/{transaction_id}"));
    let mut snapshot_paths = collect_canonical_snapshot_paths(target)?;
    for change in changes {
        snapshot_paths
            .entry(change.path.clone())
            .or_insert_with(|| "manifest-owned".to_owned());
    }
    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;
    for (path, ownership) in snapshot_paths {
        if !crate::backup_path_is_allowed(&path) {
            return Err(UpdateError::Compatibility(format!(
                "update backup attempts to include an excluded path: {path}"
            )));
        }
        let backup_path = crate::backup_storage_path(&path).ok_or_else(|| {
            UpdateError::Compatibility(format!(
                "update backup path has no safe storage encoding: {path}"
            ))
        })?;
        let planned_change = changes.iter().find(|change| change.path == path);
        let expected = planned_change.and_then(|change| change.before_digest.as_deref());
        match read_optional_bounded(target, Path::new(&path), MAX_BACKUP_FILE_BYTES)? {
            Some(bytes) => {
                if planned_change.is_some() && expected.is_none() {
                    return Err(UpdateError::Conflict(format!(
                        "update baseline appeared before backup: {path}"
                    )));
                }
                let digest = sha256_digest(&bytes);
                if expected.is_some_and(|expected| expected != digest) {
                    return Err(UpdateError::Conflict(format!(
                        "update baseline changed before backup: {path}"
                    )));
                }
                total_bytes = total_bytes
                    .checked_add(bytes.len() as u64)
                    .ok_or_else(|| UpdateError::Input("backup size overflows".to_owned()))?;
                if total_bytes > MAX_BACKUP_TOTAL_BYTES {
                    return Err(UpdateError::Compatibility(
                        "canonical backup exceeds the bounded size limit".to_owned(),
                    ));
                }
                write_new_target_file(target, &root_relative.join(&backup_path), &bytes)?;
                entries.push(BackupEntry {
                    path,
                    ownership,
                    prior_digest: Some(digest),
                    prior_length: bytes.len() as u64,
                    backup_path,
                });
            }
            None if expected.is_some() => {
                return Err(UpdateError::Conflict(format!(
                    "update baseline disappeared before backup: {path}"
                )));
            }
            None => entries.push(BackupEntry {
                path,
                ownership,
                prior_digest: None,
                prior_length: 0,
                backup_path,
            }),
        }
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let mut manifest = BackupManifest {
        schema_version: 1,
        transaction_id: transaction_id.to_owned(),
        source_version: source_version.to_owned(),
        target_version: target_version.to_owned(),
        created_at_unix: now_unix,
        expires_at_unix: now_unix
            .checked_add(crate::BACKUP_RETENTION_SECONDS)
            .ok_or_else(|| UpdateError::Input("backup timestamp overflows".to_owned()))?,
        tree_digest: tree_digest.to_owned(),
        entries,
        manifest_digest: String::new(),
    };
    manifest.manifest_digest = backup_manifest_digest(&manifest)?;
    write_new_target_file(
        target,
        &root_relative.join("backup-manifest.json"),
        &json_line(&manifest, "backup manifest")?,
    )?;
    Ok(manifest)
}

fn collect_canonical_snapshot_paths(
    target: &TargetRoot<'_>,
) -> Result<BTreeMap<String, String>, UpdateError> {
    let mut paths = BTreeMap::new();
    for root in CANONICAL_BACKUP_ROOTS {
        collect_canonical_path(target, Path::new(root), &mut paths)?;
    }
    Ok(paths)
}

fn collect_canonical_path(
    target: &TargetRoot<'_>,
    relative: &Path,
    paths: &mut BTreeMap<String, String>,
) -> Result<(), UpdateError> {
    if !crate::backup_path_is_allowed(&relative.to_string_lossy()) {
        return Ok(());
    }
    let Some((parent, file_name)) = capability_parent(target.dir, relative, false)? else {
        return Ok(());
    };
    let metadata = match parent.symlink_metadata(&file_name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(UpdateError::Internal(format!(
                "cannot inspect canonical backup path {}: {error}",
                relative.display()
            )))
        }
    };
    if metadata.file_type().is_symlink() {
        return Err(UpdateError::Conflict(format!(
            "canonical backup path is a symlink: {}",
            relative.display()
        )));
    }
    if metadata.is_file() {
        paths.insert(
            relative.to_string_lossy().replace('\\', "/"),
            "canonical-protected".to_owned(),
        );
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(UpdateError::Conflict(format!(
            "canonical backup path is not a regular file or directory: {}",
            relative.display()
        )));
    }
    let directory = parent.open_dir_nofollow(&file_name).map_err(|error| {
        UpdateError::Conflict(format!(
            "cannot open canonical backup path {} no-follow: {error}",
            relative.display()
        ))
    })?;
    let mut children = directory
        .entries()
        .map_err(|error| {
            UpdateError::Internal(format!(
                "cannot enumerate canonical backup path {}: {error}",
                relative.display()
            ))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| UpdateError::Internal(format!("cannot read backup entry: {error}")))?;
    children.sort_by_key(cap_std::fs::DirEntry::file_name);
    for child in children {
        collect_canonical_path(target, &relative.join(child.file_name()), paths)?;
    }
    Ok(())
}

fn valid_transaction_directory_name(name: &str) -> bool {
    name.strip_prefix("txn-").is_some_and(|suffix| {
        suffix.len() == 24
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn prune_expired_backups_in(target: &TargetRoot<'_>, now_unix: i64) -> usize {
    let Some((hive, backups_name)) =
        capability_parent(target.dir, Path::new(".hive/backups"), false)
            .ok()
            .flatten()
    else {
        return 0;
    };
    let Ok(backups) = hive.open_dir_nofollow(&backups_name) else {
        return 0;
    };
    let Ok(entries) = backups.entries() else {
        return 0;
    };
    let mut pruned = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !valid_transaction_directory_name(&name) {
            continue;
        }
        let Some(manifest) = validated_expired_backup_in(target, &name, now_unix) else {
            continue;
        };
        if remove_validated_backup_in(target, &manifest).is_ok() {
            pruned += 1;
        }
    }
    pruned
}

fn validated_expired_backup_in(
    target: &TargetRoot<'_>,
    transaction_id: &str,
    now_unix: i64,
) -> Option<BackupManifest> {
    let relative = PathBuf::from(format!(".hive/backups/{transaction_id}"));
    let (parent, name) = capability_parent(target.dir, &relative, false)
        .ok()
        .flatten()?;
    let metadata = parent.symlink_metadata(&name).ok()?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return None;
    }
    parent.open_dir_nofollow(&name).ok()?;
    let bytes = read_optional_bounded(
        target,
        &relative.join("backup-manifest.json"),
        MAX_CONFIG_BYTES,
    )
    .ok()??;
    let manifest: BackupManifest = serde_json::from_slice(&bytes).ok()?;
    if manifest.transaction_id != transaction_id
        || backup_manifest_digest(&manifest).ok()? != manifest.manifest_digest
        || crate::backups_to_prune(std::slice::from_ref(&manifest), now_unix, &BTreeSet::new())
            .ok()?
            != [transaction_id.to_owned()]
    {
        return None;
    }
    Some(manifest)
}

fn remove_validated_backup_in(
    target: &TargetRoot<'_>,
    manifest: &BackupManifest,
) -> Result<(), UpdateError> {
    let root_relative = PathBuf::from(format!(".hive/backups/{}", manifest.transaction_id));
    let Some((parent, name)) = capability_parent(target.dir, &root_relative, false)? else {
        return Err(UpdateError::Conflict(
            "expired backup disappeared before cleanup".to_owned(),
        ));
    };
    let root = parent.open_dir_nofollow(&name).map_err(|error| {
        UpdateError::Conflict(format!("cannot open expired backup no-follow: {error}"))
    })?;
    let mut expected_files = BTreeSet::from(["backup-manifest.json".to_owned()]);
    for entry in &manifest.entries {
        let relative = root_relative.join(&entry.backup_path);
        match entry.prior_digest.as_deref() {
            Some(expected) => {
                let bytes = read_required_bounded(target, &relative, MAX_BACKUP_FILE_BYTES)?;
                if bytes.len() as u64 != entry.prior_length || sha256_digest(&bytes) != expected {
                    return Err(UpdateError::Rollback(
                        "expired backup bytes differ from the manifest".to_owned(),
                    ));
                }
                expected_files.insert(entry.backup_path.clone());
            }
            None if read_optional_bounded(target, &relative, MAX_BACKUP_FILE_BYTES)?.is_some() => {
                return Err(UpdateError::Conflict(
                    "expired backup contains an unexpected file".to_owned(),
                ))
            }
            None => {}
        }
    }
    let mut actual_files = BTreeSet::new();
    let mut directories = Vec::new();
    enumerate_backup_capability(&root, Path::new(""), &mut actual_files, &mut directories)?;
    if actual_files != expected_files {
        return Err(UpdateError::Conflict(
            "expired backup contains foreign or missing files".to_owned(),
        ));
    }
    for relative in actual_files {
        remove_exact_regular_file(target, &root_relative.join(relative))?;
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for relative in directories {
        remove_exact_directory(target, &root_relative.join(relative))?;
    }
    drop(root);
    remove_exact_directory(target, &root_relative)
}

fn enumerate_backup_capability(
    directory: &Dir,
    relative: &Path,
    files: &mut BTreeSet<String>,
    directories: &mut Vec<PathBuf>,
) -> Result<(), UpdateError> {
    let entries = directory
        .entries()
        .map_err(|error| UpdateError::Rollback(format!("cannot enumerate backup: {error}")))?;
    for entry in entries {
        let entry = entry
            .map_err(|error| UpdateError::Rollback(format!("cannot read backup entry: {error}")))?;
        let name = entry.file_name();
        let child = relative.join(&name);
        let metadata = directory
            .symlink_metadata(&name)
            .map_err(|error| UpdateError::Rollback(format!("cannot inspect backup: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(UpdateError::Conflict(
                "expired backup contains a symlink".to_owned(),
            ));
        }
        if metadata.is_dir() {
            directories.push(child.clone());
            let nested = directory.open_dir_nofollow(&name).map_err(|error| {
                UpdateError::Conflict(format!("cannot open backup directory no-follow: {error}"))
            })?;
            enumerate_backup_capability(&nested, &child, files, directories)?;
        } else if metadata.is_file() {
            files.insert(child.to_string_lossy().replace('\\', "/"));
        } else {
            return Err(UpdateError::Conflict(
                "expired backup contains a nonregular entry".to_owned(),
            ));
        }
    }
    Ok(())
}

fn rollback_changes(target: &TargetRoot<'_>, journal: &UpdateJournal) -> Result<(), UpdateError> {
    let backup_root = PathBuf::from(format!(".hive/backups/{}", journal.transaction_id));
    for change in &journal.changes {
        let current = digest_optional(target, Path::new(&change.path))?;
        if current != change.before_digest && current != change.after_digest {
            return Err(UpdateError::Conflict(format!(
                "recovery preserves racing bytes at {}",
                change.path
            )));
        }
    }
    for change in journal.changes.iter().rev() {
        let current = digest_optional(target, Path::new(&change.path))?;
        match change.before_digest.as_deref() {
            Some(expected) => {
                let bytes = read_required_bounded(
                    target,
                    &backup_root.join(&change.backup_path),
                    MAX_BACKUP_FILE_BYTES,
                )?;
                if sha256_digest(&bytes) != expected {
                    return Err(UpdateError::Rollback(format!(
                        "recovery backup digest mismatch for {}",
                        change.path
                    )));
                }
                write_atomic_relative(target, Path::new(&change.path), &bytes)?;
            }
            None if current.is_some() => {
                remove_exact_regular_file(target, Path::new(&change.path))?;
            }
            None => {}
        }
    }
    Ok(())
}

fn verify_after_digests(
    target: &TargetRoot<'_>,
    changes: &[JournalChange],
) -> Result<(), UpdateError> {
    for change in changes {
        if digest_optional(target, Path::new(&change.path))? != change.after_digest {
            return Err(UpdateError::Conflict(format!(
                "activated bytes differ from the journaled plan: {}",
                change.path
            )));
        }
    }
    Ok(())
}

fn read_installed_harness(target: &TargetRoot<'_>) -> Result<InstalledHarness, UpdateError> {
    let bytes = read_required_bounded(target, Path::new(HARNESS_PATH), MAX_CONFIG_BYTES)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| UpdateError::Input("installed harness TOML is not UTF-8".to_owned()))?;
    toml::from_str(text)
        .map_err(|error| UpdateError::Input(format!("invalid installed harness TOML: {error}")))
}

fn installed_capabilities_json(target: &TargetRoot<'_>) -> Result<Vec<u8>, UpdateError> {
    let bytes = read_required_bounded(target, Path::new(CAPABILITIES_PATH), MAX_CONFIG_BYTES)?;
    let value: JsonValue = serde_yaml::from_slice(&bytes).map_err(|error| {
        UpdateError::Input(format!("invalid installed capability YAML: {error}"))
    })?;
    serde_json::to_vec(&value).map_err(|error| {
        UpdateError::Internal(format!("cannot normalize installed capabilities: {error}"))
    })
}

fn installed_answers_yaml(target: &TargetRoot<'_>, threshold: u8) -> Result<Vec<u8>, UpdateError> {
    let bytes = read_required_bounded(target, Path::new(ANSWERS_PATH), MAX_CONFIG_BYTES)?;
    let mut value: serde_yaml::Value = serde_yaml::from_slice(&bytes)
        .map_err(|error| UpdateError::Input(format!("invalid installed setup answers: {error}")))?;
    let mapping = value.as_mapping_mut().ok_or_else(|| {
        UpdateError::Input("installed setup answers must be a YAML mapping".to_owned())
    })?;
    let key = serde_yaml::Value::from("usage_stop_remaining_percent");
    if !mapping.contains_key(&key) {
        return Err(UpdateError::Verification(
            "installed setup answers omit usage_stop_remaining_percent".to_owned(),
        ));
    }
    mapping.insert(key, serde_yaml::Value::from(threshold));
    serde_yaml::to_string(&value)
        .map(String::into_bytes)
        .map_err(|error| UpdateError::Internal(format!("cannot stage setup answers: {error}")))
}

fn read_update_state(target: &TargetRoot<'_>) -> Result<Option<UpdateState>, UpdateError> {
    read_optional_bounded(target, Path::new(UPDATE_STATE_PATH), MAX_CONFIG_BYTES)?
        .map(|bytes| {
            serde_json::from_slice(&bytes).map_err(|error| {
                UpdateError::Verification(format!("invalid update state: {error}"))
            })
        })
        .transpose()
}

fn write_update_state(target: &TargetRoot<'_>, state: &UpdateState) -> Result<(), UpdateError> {
    write_atomic_relative(
        target,
        Path::new(UPDATE_STATE_PATH),
        &json_line(state, "update state")?,
    )
}

fn plan_digest(
    source_version: &str,
    target_version: &str,
    release_manifest_digest: &str,
    tree_digest: &str,
    changes: &[SetupChange],
) -> Result<String, UpdateError> {
    let payload = PlanDigestPayload {
        schema_version: 1,
        source_version,
        target_version,
        release_manifest_digest,
        tree_digest,
        changes: changes
            .iter()
            .map(|change| PlanDigestChange {
                path: &change.path,
                before_digest: change.before_digest.as_deref(),
                after_digest: change.after_digest.as_deref(),
                foreign_before_digest: change.foreign_before_digest.as_deref(),
                foreign_after_digest: change.foreign_after_digest.as_deref(),
            })
            .collect(),
    };
    let bytes = serde_json_canonicalizer::to_vec(&payload)
        .map_err(|error| UpdateError::Internal(format!("cannot digest update plan: {error}")))?;
    Ok(sha256_digest(&bytes))
}

fn transaction_id(plan_digest: &str, now_unix: i64) -> String {
    let digest = sha256_digest(format!("{plan_digest}\0{now_unix}").as_bytes());
    let hex = &digest["sha256:".len()..];
    format!("txn-{}", &hex[..24])
}

fn journal_digest(journal: &UpdateJournal) -> Result<String, UpdateError> {
    let mut value = serde_json::to_value(journal)
        .map_err(|error| UpdateError::Internal(format!("cannot encode update journal: {error}")))?;
    value
        .as_object_mut()
        .ok_or_else(|| UpdateError::Internal("update journal is not an object".to_owned()))?
        .remove("journal_digest");
    let bytes = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| UpdateError::Internal(format!("cannot digest update journal: {error}")))?;
    Ok(sha256_digest(&bytes))
}

fn backup_manifest_digest(manifest: &BackupManifest) -> Result<String, UpdateError> {
    let mut value = serde_json::to_value(manifest).map_err(|error| {
        UpdateError::Internal(format!("cannot encode backup manifest: {error}"))
    })?;
    value
        .as_object_mut()
        .ok_or_else(|| UpdateError::Internal("backup manifest is not an object".to_owned()))?
        .remove("manifest_digest");
    let bytes = serde_json_canonicalizer::to_vec(&value).map_err(|error| {
        UpdateError::Internal(format!("cannot digest backup manifest: {error}"))
    })?;
    Ok(sha256_digest(&bytes))
}

fn mark_needs_recovery(
    target: &TargetRoot<'_>,
    journal: &mut UpdateJournal,
) -> Result<(), UpdateError> {
    journal.state = JournalState::NeedsRecovery;
    journal.journal_digest = journal_digest(journal)?;
    write_atomic_relative(
        target,
        Path::new(JOURNAL_PATH),
        &json_line(journal, "update journal")?,
    )
}

fn journal_exists(target: &TargetRoot<'_>) -> Result<bool, UpdateError> {
    Ok(read_optional_bounded(target, Path::new(JOURNAL_PATH), MAX_CONFIG_BYTES)?.is_some())
}

fn digest_optional(
    target: &TargetRoot<'_>,
    relative: &Path,
) -> Result<Option<String>, UpdateError> {
    read_optional_bounded(target, relative, MAX_CONFIG_BYTES * 16)
        .map(|bytes| bytes.map(|bytes| sha256_digest(&bytes)))
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn read_required_bounded(
    target: &TargetRoot<'_>,
    relative: &Path,
    maximum: u64,
) -> Result<Vec<u8>, UpdateError> {
    read_optional_bounded(target, relative, maximum)?.ok_or_else(|| {
        UpdateError::Verification(format!(
            "required installed file is missing: {}",
            relative.display()
        ))
    })
}

fn read_optional_bounded(
    target: &TargetRoot<'_>,
    relative: &Path,
    maximum: u64,
) -> Result<Option<Vec<u8>>, UpdateError> {
    validate_update_relative(relative)?;
    let Some((parent, file_name)) = capability_parent(target.dir, relative, false)? else {
        return Ok(None);
    };
    let metadata = match parent.symlink_metadata(&file_name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(UpdateError::Internal(format!(
                "cannot inspect {}: {error}",
                relative.display()
            )))
        }
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum {
        return Err(UpdateError::Conflict(format!(
            "installed path is not a bounded regular file: {}",
            relative.display()
        )));
    }
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = parent.open_with(&file_name, &options).map_err(|error| {
        UpdateError::Conflict(format!(
            "cannot open {} no-follow: {error}",
            relative.display()
        ))
    })?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len())
            .map_err(|_| UpdateError::Internal("installed file is too large".to_owned()))?,
    );
    Read::by_ref(&mut file)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| {
            UpdateError::Internal(format!("cannot read {}: {error}", relative.display()))
        })?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > maximum {
        return Err(UpdateError::Conflict(format!(
            "installed file changed during read: {}",
            relative.display()
        )));
    }
    Ok(Some(bytes))
}

fn write_new_target_file(
    target: &TargetRoot<'_>,
    relative: &Path,
    bytes: &[u8],
) -> Result<(), UpdateError> {
    let (parent, file_name) = capability_parent(target.dir, relative, true)?.ok_or_else(|| {
        UpdateError::Internal("backup parent disappeared during creation".to_owned())
    })?;
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(&file_name, &options)
        .map_err(|error| UpdateError::Conflict(format!("cannot create backup file: {error}")))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist backup file: {error}")))?;
    sync_capability_directory(&parent, relative)
}

#[cfg(test)]
fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpdateError::Internal("test file has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| UpdateError::Internal(format!("cannot create test parent: {error}")))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| UpdateError::Conflict(format!("cannot create test file: {error}")))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist test file: {error}")))
}

#[cfg(test)]
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpdateError::Internal("test file has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| UpdateError::Internal(format!("cannot create test parent: {error}")))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| UpdateError::Internal(format!("cannot create test temp: {error}")))?;
    temporary
        .write_all(bytes)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist test temp: {error}")))?;
    temporary
        .persist(path)
        .map(|_| ())
        .map_err(|error| UpdateError::Internal(format!("cannot replace test file: {error}")))
}

fn write_atomic_relative(
    target: &TargetRoot<'_>,
    relative: &Path,
    bytes: &[u8],
) -> Result<(), UpdateError> {
    validate_update_relative(relative)?;
    let (parent, file_name) = capability_parent(target.dir, relative, true)?.ok_or_else(|| {
        UpdateError::Internal("atomic parent disappeared during creation".to_owned())
    })?;
    let temporary_name = OsString::from(format!(
        ".hive-update-tmp-{}-{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut temporary = parent
        .open_with(&temporary_name, &options)
        .map_err(|error| UpdateError::Internal(format!("cannot create atomic temp: {error}")))?;
    temporary
        .write_all(bytes)
        .and_then(|()| temporary.sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist atomic temp: {error}")))?;
    drop(temporary);
    if let Err(error) = parent.rename(&temporary_name, &parent, &file_name) {
        let _ = parent.remove_file(&temporary_name);
        return Err(UpdateError::Internal(format!(
            "cannot atomically replace {}: {error}",
            relative.display()
        )));
    }
    #[cfg(unix)]
    {
        let mut read_options = CapOpenOptions::new();
        read_options.read(true).follow(FollowSymlinks::No);
        parent
            .open_with(&file_name, &read_options)
            .and_then(|file| file.sync_all())
            .map_err(|error| {
                UpdateError::Internal(format!(
                    "cannot sync atomic replacement {}: {error}",
                    relative.display()
                ))
            })?;
    }
    sync_capability_directory(&parent, relative)?;
    Ok(())
}

#[cfg(unix)]
fn sync_capability_directory(parent: &Dir, relative: &Path) -> Result<(), UpdateError> {
    parent
        .open(Path::new("."))
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            UpdateError::Internal(format!(
                "cannot sync atomic parent for {}: {error}",
                relative.display()
            ))
        })?;
    #[cfg(test)]
    DIRECTORY_SYNC_EVENTS.with(|events| events.borrow_mut().push(relative.to_path_buf()));
    Ok(())
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn sync_capability_directory(_parent: &Dir, _relative: &Path) -> Result<(), UpdateError> {
    Ok(())
}

fn remove_exact_regular_file(target: &TargetRoot<'_>, relative: &Path) -> Result<(), UpdateError> {
    validate_update_relative(relative)?;
    let Some((parent, file_name)) = capability_parent(target.dir, relative, false)? else {
        return Ok(());
    };
    match parent.symlink_metadata(&file_name) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            parent.remove_file(&file_name).map_err(|error| {
                UpdateError::Rollback(format!("cannot remove {}: {error}", relative.display()))
            })?;
            sync_capability_directory(&parent, relative)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(UpdateError::Conflict(format!(
            "recovery path is not a regular file: {}",
            relative.display()
        ))),
        Err(error) => Err(UpdateError::Internal(format!(
            "cannot inspect recovery path {}: {error}",
            relative.display()
        ))),
    }
}

fn remove_exact_directory(target: &TargetRoot<'_>, relative: &Path) -> Result<(), UpdateError> {
    validate_update_relative(relative)?;
    let Some((parent, file_name)) = capability_parent(target.dir, relative, false)? else {
        return Ok(());
    };
    match parent.symlink_metadata(&file_name) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            parent.remove_dir(&file_name).map_err(|error| {
                UpdateError::Rollback(format!(
                    "cannot remove backup directory {}: {error}",
                    relative.display()
                ))
            })?;
            sync_capability_directory(&parent, relative)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(UpdateError::Conflict(format!(
            "backup cleanup path is not a directory: {}",
            relative.display()
        ))),
        Err(error) => Err(UpdateError::Internal(format!(
            "cannot inspect backup directory {}: {error}",
            relative.display()
        ))),
    }
}

fn validate_update_relative(relative: &Path) -> Result<(), UpdateError> {
    if is_hive_skill_projection_path(relative) {
        validate_hive_skill_projection_relative(relative)
            .map_err(|error| UpdateError::Input(error.to_string()))
    } else if is_hive_directive_projection_path(relative) {
        validate_hive_directive_projection_relative(relative)
            .map_err(|error| UpdateError::Input(error.to_string()))
    } else {
        validate_project_relative(relative).map_err(|error| UpdateError::Input(error.to_string()))
    }
}

fn ensure_pinned_consumer_target(target: &TargetRoot<'_>) -> Result<(), UpdateError> {
    if read_optional_bounded(target, Path::new(SOURCE_MARKER_FILE), MAX_CONFIG_BYTES)?.is_some() {
        return Err(UpdateError::Input(
            "consumer update commands are forbidden in the Hive source workspace".to_owned(),
        ));
    }
    Ok(())
}

fn open_target_capability(target: &Path) -> Result<Dir, UpdateError> {
    let parent = target
        .parent()
        .ok_or_else(|| UpdateError::Input("update target has no parent directory".to_owned()))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = target
        .file_name()
        .ok_or_else(|| UpdateError::Input("update target has no directory name".to_owned()))?;
    let parent_dir = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|error| {
        UpdateError::Conflict(format!("cannot open update target parent: {error}"))
    })?;
    parent_dir.open_dir_nofollow(name).map_err(|error| {
        UpdateError::Conflict(format!(
            "update target cannot be opened as a no-follow directory {}: {error}",
            target.display()
        ))
    })
}

fn capability_parent(
    target: &Dir,
    relative: &Path,
    create_missing: bool,
) -> Result<Option<(Dir, OsString)>, UpdateError> {
    validate_update_relative(relative)?;
    let file_name = relative
        .file_name()
        .ok_or_else(|| UpdateError::Input("update path has no file name".to_owned()))?
        .to_os_string();
    let parent = relative
        .parent()
        .ok_or_else(|| UpdateError::Input("update path has no parent".to_owned()))?;
    let mut current = target.try_clone().map_err(|error| {
        UpdateError::Internal(format!("cannot clone target capability: {error}"))
    })?;
    for component in parent.components() {
        let name = component.as_os_str();
        match current.symlink_metadata(name) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(UpdateError::Conflict(format!(
                    "update ancestor is not a no-follow directory: {}",
                    relative.display()
                )))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && create_missing => {
                current.create_dir(name).map_err(|error| {
                    UpdateError::Conflict(format!(
                        "cannot create update ancestor {}: {error}",
                        relative.display()
                    ))
                })?;
                sync_capability_directory(&current, relative)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(UpdateError::Conflict(format!(
                    "cannot inspect update ancestor {}: {error}",
                    relative.display()
                )))
            }
        }
        current = current.open_dir_nofollow(name).map_err(|error| {
            UpdateError::Conflict(format!(
                "cannot open update ancestor {} no-follow: {error}",
                relative.display()
            ))
        })?;
    }
    Ok(Some((current, file_name)))
}

fn read_parent_file(parent: &Dir, file_name: &OsStr, maximum: u64) -> Result<Vec<u8>, UpdateError> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = parent.open_with(file_name, &options).map_err(|error| {
        UpdateError::Conflict(format!("cannot open target file no-follow: {error}"))
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| UpdateError::Internal(format!("cannot inspect target file: {error}")))?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err(UpdateError::Conflict(
            "target file is not a bounded regular file".to_owned(),
        ));
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len())
            .map_err(|_| UpdateError::Internal("target file is too large".to_owned()))?,
    );
    file.take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| UpdateError::Internal(format!("cannot read target file: {error}")))?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > maximum {
        return Err(UpdateError::Conflict(
            "target file changed during read".to_owned(),
        ));
    }
    Ok(bytes)
}

fn rebuild_index_in(target: &TargetRoot<'_>) -> Result<hive_wiki::IndexOutcome, UpdateError> {
    let lock = Path::new(".hive/index/.knowledge.lock");
    write_new_target_file(target, lock, b"update-index-rebuild\n")?;
    let result = rebuild_index_while_locked(target);
    let cleanup = remove_exact_regular_file(target, lock);
    match (result, cleanup) {
        (Ok(outcome), Ok(())) => Ok(outcome),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

fn update_project_index_for_version(
    target: &TargetRoot<'_>,
    target_version: &str,
) -> Result<Option<String>, UpdateError> {
    let target_version: SemVersion = target_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Internal(error.to_string()))?;
    if target_version < SHARED_INDEX_MINIMUM_VERSION {
        return rebuild_index_in(target).map(|outcome| Some(outcome.logical_digest));
    }
    for relative in PROJECT_INDEX_FILES {
        remove_exact_regular_file(target, Path::new(relative))?;
    }
    Ok(None)
}

fn rebuild_index_while_locked(
    target: &TargetRoot<'_>,
) -> Result<hive_wiki::IndexOutcome, UpdateError> {
    let staging = tempfile::tempdir()
        .map_err(|error| UpdateError::Internal(format!("cannot stage index rebuild: {error}")))?;
    let staging_path = staging
        .path()
        .canonicalize()
        .map_err(|error| UpdateError::Internal(format!("cannot pin staged index root: {error}")))?;
    let staged_knowledge = staging_path.join(".hive/knowledge");
    fs::create_dir_all(&staged_knowledge)
        .map_err(|error| UpdateError::Internal(format!("cannot stage knowledge tree: {error}")))?;
    if let Some((hive, knowledge_name)) =
        capability_parent(target.dir, Path::new(".hive/knowledge"), false)?
    {
        match hive.symlink_metadata(&knowledge_name) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                let knowledge = hive.open_dir_nofollow(&knowledge_name).map_err(|error| {
                    UpdateError::Conflict(format!("cannot open knowledge tree no-follow: {error}"))
                })?;
                copy_capability_tree(&knowledge, &staged_knowledge, 0)?;
            }
            Ok(_) => {
                return Err(UpdateError::Conflict(
                    "knowledge root is not a no-follow directory".to_owned(),
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(UpdateError::Internal(format!(
                    "cannot inspect knowledge root: {error}"
                )))
            }
        }
    }
    let outcome = hive_wiki::rebuild_index(&staging_path).map_err(|error| {
        UpdateError::Verification(format!("cannot rebuild disposable index: {error}"))
    })?;
    let bytes = fs::read(staging_path.join(".hive/index/hive.sqlite3"))
        .map_err(|error| UpdateError::Internal(format!("cannot read staged index: {error}")))?;
    write_atomic_relative(target, Path::new(".hive/index/hive.sqlite3"), &bytes)?;
    for relative in [
        ".hive/index/hive.sqlite3-wal",
        ".hive/index/hive.sqlite3-shm",
        ".hive/index/hive.sqlite3-journal",
        ".hive/index/.stale",
    ] {
        remove_exact_regular_file(target, Path::new(relative))?;
    }
    Ok(outcome)
}

fn copy_capability_tree(source: &Dir, destination: &Path, depth: usize) -> Result<(), UpdateError> {
    if depth > 32 {
        return Err(UpdateError::Compatibility(
            "knowledge tree exceeds the maximum directory depth".to_owned(),
        ));
    }
    let mut entries = source
        .entries()
        .map_err(|error| {
            UpdateError::Internal(format!("cannot enumerate knowledge tree: {error}"))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| UpdateError::Internal(format!("cannot read knowledge entry: {error}")))?;
    entries.sort_by_key(cap_std::fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        let metadata = source.symlink_metadata(&name).map_err(|error| {
            UpdateError::Internal(format!("cannot inspect knowledge entry: {error}"))
        })?;
        let staged = destination.join(&name);
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            fs::create_dir(&staged).map_err(|error| {
                UpdateError::Internal(format!("cannot stage knowledge directory: {error}"))
            })?;
            let child = source.open_dir_nofollow(&name).map_err(|error| {
                UpdateError::Conflict(format!(
                    "cannot open knowledge directory no-follow: {error}"
                ))
            })?;
            copy_capability_tree(&child, &staged, depth + 1)?;
        } else if metadata.is_file() && !metadata.file_type().is_symlink() {
            let bytes = read_parent_file(source, &name, MAX_BACKUP_FILE_BYTES)?;
            fs::write(staged, bytes).map_err(|error| {
                UpdateError::Internal(format!("cannot stage knowledge file: {error}"))
            })?;
        } else {
            return Err(UpdateError::Conflict(
                "knowledge tree contains a symlink or special entry".to_owned(),
            ));
        }
    }
    Ok(())
}

fn json_line(value: &impl Serialize, label: &str) -> Result<Vec<u8>, UpdateError> {
    let mut bytes = serde_json::to_vec(value)
        .map_err(|error| UpdateError::Internal(format!("cannot encode {label}: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn map_render_error(error: RenderError) -> UpdateError {
    match error {
        RenderError::Input(message) => UpdateError::Input(message),
        RenderError::Safety(message) | RenderError::Conflict(message) => {
            UpdateError::Conflict(message)
        }
        RenderError::Verification(message) => UpdateError::Verification(message),
        RenderError::Unsupported(message) => UpdateError::Unsupported(message),
        RenderError::Internal(message) => UpdateError::Internal(message),
        RenderError::Rollback(message) => UpdateError::Rollback(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static UPDATE_ENVIRONMENT: Mutex<()> = Mutex::new(());

    fn phase1_fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/setup")
            .join(name)
    }

    fn historical_release_fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/release/versions/valid-0.7.0")
    }

    fn release_fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/release/versions")
            .join(format!("valid-{}", env!("CARGO_PKG_VERSION")))
    }

    fn published_release_fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/release/versions/valid-0.9.0")
    }

    fn legacy_builtin_names(version: &str) -> &'static [&'static str] {
        const V04: &[&str] = &[
            "setup-harness",
            "hive-simple-question",
            "hive-prompt-refine",
            "hive-knowledge-capture",
            "hive-knowledge-query",
            "hive-knowledge-maintenance",
        ];
        const V05: &[&str] = &[
            "setup-harness",
            "hive-simple-question",
            "hive-prompt-refine",
            "hive-knowledge-capture",
            "hive-knowledge-query",
            "hive-knowledge-maintenance",
            "hive-run-checkpoint",
            "hive-run-resume",
            "hive-role-handoff",
        ];
        const V06: &[&str] = &[
            "setup-harness",
            "hive-simple-question",
            "hive-prompt-refine",
            "hive-knowledge-capture",
            "hive-knowledge-query",
            "hive-knowledge-maintenance",
            "hive-run-checkpoint",
            "hive-run-resume",
            "hive-role-handoff",
            "hive-judge-package",
        ];
        match version {
            "0.1.0" | "0.2.0" | "0.3.0" => &[],
            "0.4.0" => V04,
            "0.5.0" => V05,
            "0.6.0" => V06,
            _ => panic!("unsupported legacy fixture version: {version}"),
        }
    }

    fn install_legacy_consumer(target: &Path, version: &str) {
        let target = target.canonicalize().expect("consumer target");
        execute_setup(&SetupRequest {
            target: &target,
            answers: &phase1_fixture("answers-base.yml"),
            capabilities: &phase1_fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(GlobalProjectPreferences {
                interface_language: "en".to_owned(),
                wiki_enabled: true,
                wiki_backend: "markdown".to_owned(),
                wiki_language: "both".to_owned(),
                persona_id: "balanced".to_owned(),
                persona_custom_description: None,
                selected_project_skills: legacy_builtin_names("0.6.0")
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect(),
                usage_guard_enabled: false,
                codexbar_fallback_enabled: false,
                discord_guard_enabled: false,
                discord_webhook_url_env: None,
                discord_message_fields: vec![
                    "remaining-usage".to_owned(),
                    "project".to_owned(),
                    "request".to_owned(),
                    "progress".to_owned(),
                    "host".to_owned(),
                    "resume".to_owned(),
                ],
                usage_stop_remaining_percent: 20,
            }),
        })
        .expect("connected current setup");

        let harness_path = target.join(HARNESS_PATH);
        let harness = fs::read_to_string(&harness_path)
            .expect("harness")
            .replace(env!("CARGO_PKG_VERSION"), version);
        fs::write(harness_path, harness).expect("legacy harness");
        fs::remove_dir_all(target.join(".agents/directives"))
            .expect("remove post-legacy directive projections");
        fs::remove_file(target.join(".hive/config/project-base.json"))
            .expect("remove post-legacy full project-base ledger");

        let active_path = target.join(".hive/config/active-skills.yml");
        let expected = legacy_builtin_names(version);
        let skill_root = target.join(".agents/skills");
        for entry in fs::read_dir(&skill_root).expect("projected Skills") {
            let entry = entry.expect("projected Skill entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            if expected.contains(&name.as_str()) {
                let metadata = entry.path().join("agents");
                if metadata.exists() {
                    fs::remove_dir_all(metadata).expect("remove post-legacy Skill metadata");
                }
            } else {
                fs::remove_dir_all(entry.path()).expect("remove future projection");
            }
        }
        if expected.is_empty() {
            fs::remove_file(active_path).expect("remove future active ledger");
        } else {
            install_historical_skill_projection(&skill_root, &active_path, version, expected);
        }
        fs::write(target.join("README.md"), b"foreign project bytes\n").expect("foreign readme");
        fs::create_dir_all(target.join(".omx")).expect("omx");
        fs::write(target.join(".omx/state.json"), b"{\"foreign\":true}\n")
            .expect("foreign orchestration");
    }

    fn install_historical_consumer(target: &Path, version: &str) {
        let target = target.canonicalize().expect("consumer target");
        let preferences = GlobalProjectPreferences {
            interface_language: "en".to_owned(),
            wiki_enabled: true,
            wiki_backend: "markdown".to_owned(),
            wiki_language: "both".to_owned(),
            persona_id: "balanced".to_owned(),
            persona_custom_description: None,
            selected_project_skills: Vec::new(),
            usage_guard_enabled: false,
            codexbar_fallback_enabled: false,
            discord_guard_enabled: false,
            discord_webhook_url_env: None,
            discord_message_fields: vec![
                "remaining-usage".to_owned(),
                "project".to_owned(),
                "request".to_owned(),
                "progress".to_owned(),
                "host".to_owned(),
                "resume".to_owned(),
            ],
            usage_stop_remaining_percent: 20,
        };
        execute_setup(&SetupRequest {
            target: &target,
            answers: &phase1_fixture("answers-base.yml"),
            capabilities: &phase1_fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(preferences),
        })
        .expect("current setup before historical projection");

        let harness_path = target.join(HARNESS_PATH);
        let harness = fs::read_to_string(&harness_path)
            .expect("harness")
            .replace(
                &format!("harness_version = \"{}\"", env!("CARGO_PKG_VERSION")),
                &format!("harness_version = \"{version}\""),
            )
            .replace(
                &format!("source_release_version = \"{}\"", env!("CARGO_PKG_VERSION")),
                &format!("source_release_version = \"{version}\""),
            );
        fs::write(&harness_path, harness).expect("historical harness");

        let capability =
            Dir::open_ambient_dir(&target, ambient_authority()).expect("target capability");
        let historical = historical_project_upgrade_candidate_in(&capability, version)
            .expect("embedded historical project base");
        for file in &historical.files {
            let path = target.join(&file.path);
            fs::create_dir_all(path.parent().expect("historical parent"))
                .expect("historical directory");
            fs::write(path, &file.content).expect("historical projection");
        }
        if !historical
            .files
            .iter()
            .any(|file| file.path == ".agents/directives/03-session-coordination.md")
        {
            fs::remove_file(target.join(".agents/directives/03-session-coordination.md"))
                .expect("remove post-historical directive projection");
        }
        let files = historical
            .files
            .iter()
            .map(|file| {
                serde_json::json!({
                    "path": file.path,
                    "kind": file.kind,
                    "content_digest": file.content_digest,
                    "content": String::from_utf8_lossy(&file.content),
                })
            })
            .collect::<Vec<_>>();
        let mut ledger = serde_json::json!({
            "schema_version": 1,
            "product_version": version,
            "files": files,
        });
        let digest = sha256_digest(
            &serde_json_canonicalizer::to_vec(&ledger).expect("canonical unsigned ledger"),
        );
        ledger
            .as_object_mut()
            .expect("ledger object")
            .insert("ledger_digest".to_owned(), JsonValue::String(digest));
        let mut bytes = serde_json_canonicalizer::to_vec(&ledger).expect("canonical ledger");
        bytes.push(b'\n');
        fs::write(target.join(".hive/config/project-base.json"), bytes).expect("project base");
        fs::write(target.join("README.md"), b"foreign project bytes\n").expect("foreign readme");
        fs::create_dir_all(target.join(".omx")).expect("omx");
        fs::write(target.join(".omx/state.json"), b"{\"foreign\":true}\n")
            .expect("foreign orchestration");
    }

    fn install_historical_skill_projection(
        skill_root: &Path,
        active_path: &Path,
        version: &str,
        expected: &[&str],
    ) {
        // The current projection uses public short names.  A legacy fixture is
        // deliberately rebuilt with its original directories so update tests
        // exercise migration from the exact historical on-disk shape.
        for name in expected {
            fs::create_dir_all(skill_root.join(name))
                .expect("create historical Skill projection directory");
        }
        let historical_setup = include_bytes!(
            "../../../tests/fixtures/release/migrations/0.4.0-0.6.0-setup-harness.SKILL.md"
        );
        let historical_query = include_bytes!(
            "../../../tests/fixtures/release/migrations/0.4.0-0.6.0-hive-knowledge-query.SKILL.md"
        );
        let historical_capture = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-knowledge-capture/SKILL.md"
        );
        let historical_maintenance = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-knowledge-maintenance/SKILL.md"
        );
        let historical_prompt_refine = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-prompt-refine/SKILL.md"
        );
        let historical_simple_question = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-simple-question/SKILL.md"
        );
        let historical_checkpoint = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-run-checkpoint/SKILL.md"
        );
        let historical_resume =
            include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-run-resume/SKILL.md");
        let historical_handoff = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-role-handoff/SKILL.md"
        );
        let historical_judge = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-judge-package/SKILL.md"
        );
        fs::write(skill_root.join("setup-harness/SKILL.md"), historical_setup)
            .expect("historical setup projection");
        fs::write(
            skill_root.join("hive-knowledge-capture/SKILL.md"),
            historical_capture,
        )
        .expect("historical knowledge capture projection");
        fs::write(
            skill_root.join("hive-knowledge-query/SKILL.md"),
            historical_query,
        )
        .expect("historical knowledge query projection");
        fs::write(
            skill_root.join("hive-knowledge-maintenance/SKILL.md"),
            historical_maintenance,
        )
        .expect("historical knowledge maintenance projection");
        fs::write(
            skill_root.join("hive-prompt-refine/SKILL.md"),
            historical_prompt_refine,
        )
        .expect("historical prompt refinement projection");
        fs::write(
            skill_root.join("hive-simple-question/SKILL.md"),
            historical_simple_question,
        )
        .expect("historical simple question projection");
        for (name, bytes) in [
            ("hive-run-checkpoint", historical_checkpoint.as_slice()),
            ("hive-run-resume", historical_resume.as_slice()),
            ("hive-role-handoff", historical_handoff.as_slice()),
            ("hive-judge-package", historical_judge.as_slice()),
        ] {
            if expected.contains(&name) {
                fs::write(skill_root.join(format!("{name}/SKILL.md")), bytes)
                    .expect("historical execution Skill projection");
            }
        }
        if version == "0.5.0" {
            let historical = include_bytes!(
                "../../../tests/fixtures/release/migrations/0.5.0-hive-run-resume.SKILL.md"
            );
            fs::write(skill_root.join("hive-run-resume/SKILL.md"), historical)
                .expect("historical resume projection");
        }

        rewrite_historical_active_skills(active_path, expected, version);
    }

    fn rewrite_historical_active_skills(active_path: &Path, expected: &[&str], version: &str) {
        let skills = hive_projection::historical_builtin_skills(version)
            .expect("embedded historical active Skills");
        assert_eq!(
            skills
                .iter()
                .map(|skill| skill.name.as_str())
                .collect::<BTreeSet<_>>(),
            expected.iter().copied().collect::<BTreeSet<_>>(),
            "legacy fixture must use the exact authenticated Skill inventory"
        );
        let active = hive_projection::ActiveSkills {
            schema_version: 1,
            skills,
        };
        fs::write(
            active_path,
            serde_yaml::to_string(&active).expect("active skills"),
        )
        .expect("legacy active skills");
    }

    fn update_request<'a>(
        target: &'a Path,
        fixture: &'a Path,
        mode: UpdateMode,
    ) -> UpdateRequest<'a> {
        UpdateRequest {
            target,
            repository: fixture,
            now_unix: 1_800_000_000,
            mode,
            exact_major_target: None,
            major_approval: None,
        }
    }

    fn release_state() -> ReleaseState {
        ReleaseState {
            release_version: "0.7.0".to_owned(),
            release_sequence: 7,
            manifest_digest: format!("sha256:{}", "c".repeat(64)),
        }
    }

    fn prepared_recovery_fixture(target: &Path, live_bytes: &[u8]) -> UpdateJournal {
        let transaction_id = format!("txn-{}", "a".repeat(24));
        let relative = Path::new(".hive/config/harness.toml");
        let backup_relative = PathBuf::from(format!(
            ".hive/backups/{transaction_id}/files/.hive/config/harness.toml"
        ));
        write_new_file(&target.join(backup_relative), b"before").expect("backup");
        write_new_file(&target.join(relative), live_bytes).expect("live file");
        let mut backup_manifest = BackupManifest {
            schema_version: 1,
            transaction_id: transaction_id.clone(),
            source_version: "0.6.0".to_owned(),
            target_version: "0.7.0".to_owned(),
            created_at_unix: 1_000,
            expires_at_unix: 1_000 + crate::BACKUP_RETENTION_SECONDS,
            tree_digest: format!("sha256:{}", "b".repeat(64)),
            entries: vec![BackupEntry {
                path: relative.to_string_lossy().into_owned(),
                ownership: "manifest-owned".to_owned(),
                prior_digest: Some(sha256_digest(b"before")),
                prior_length: 6,
                backup_path: "files/.hive/config/harness.toml".to_owned(),
            }],
            manifest_digest: String::new(),
        };
        backup_manifest.manifest_digest =
            backup_manifest_digest(&backup_manifest).expect("backup manifest digest");
        write_new_file(
            &target.join(format!(
                ".hive/backups/{transaction_id}/backup-manifest.json"
            )),
            &json_line(&backup_manifest, "backup manifest").expect("backup manifest bytes"),
        )
        .expect("backup manifest");
        let mut journal = UpdateJournal {
            schema_version: 1,
            transaction_id: transaction_id.clone(),
            state: JournalState::Prepared,
            source_version: "0.6.0".to_owned(),
            target_version: "0.7.0".to_owned(),
            release_manifest_digest: format!("sha256:{}", "c".repeat(64)),
            plan_digest: format!("sha256:{}", "d".repeat(64)),
            backup_manifest_path: format!(".hive/backups/{transaction_id}/backup-manifest.json"),
            changes: vec![JournalChange {
                path: relative.to_string_lossy().into_owned(),
                before_digest: Some(sha256_digest(b"before")),
                after_digest: Some(sha256_digest(b"after")),
                backup_path: "files/.hive/config/harness.toml".to_owned(),
            }],
            next_state: UpdateState {
                schema_version: 1,
                product_version: "0.7.0".to_owned(),
                release_manifest_digest: format!("sha256:{}", "c".repeat(64)),
                accepted_release: release_state(),
            },
            journal_digest: String::new(),
        };
        journal.journal_digest = journal_digest(&journal).expect("journal digest");
        write_new_file(
            &target.join(JOURNAL_PATH),
            &json_line(&journal, "journal").expect("journal bytes"),
        )
        .expect("journal");
        journal
    }

    fn committed_recovery_fixture(target: &Path, target_version: &str) -> UpdateJournal {
        let mut journal = prepared_recovery_fixture(target, b"after");
        let backup_manifest_path = target.join(&journal.backup_manifest_path);
        let mut backup_manifest: BackupManifest =
            serde_json::from_slice(&fs::read(&backup_manifest_path).expect("backup manifest"))
                .expect("decode backup manifest");
        backup_manifest.target_version = target_version.to_owned();
        backup_manifest.manifest_digest =
            backup_manifest_digest(&backup_manifest).expect("backup manifest digest");
        write_atomic(
            &backup_manifest_path,
            &json_line(&backup_manifest, "backup manifest").expect("backup manifest bytes"),
        )
        .expect("replace backup manifest");

        journal.state = JournalState::Committed;
        journal.target_version = target_version.to_owned();
        journal.next_state.product_version = target_version.to_owned();
        journal.next_state.accepted_release.release_version = target_version.to_owned();
        journal.journal_digest = journal_digest(&journal).expect("journal digest");
        write_atomic(
            &target.join(JOURNAL_PATH),
            &json_line(&journal, "journal").expect("journal bytes"),
        )
        .expect("replace journal");
        journal
    }

    fn snapshot_regular_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
        fn visit(root: &Path, relative: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
            let mut entries = fs::read_dir(root.join(relative))
                .expect("snapshot directory")
                .collect::<Result<Vec<_>, _>>()
                .expect("snapshot entries");
            entries.sort_by_key(std::fs::DirEntry::file_name);
            for entry in entries {
                let child = relative.join(entry.file_name());
                let metadata = fs::symlink_metadata(entry.path()).expect("snapshot metadata");
                if metadata.is_dir() {
                    visit(root, &child, files);
                } else if metadata.is_file() {
                    files.insert(
                        child.to_string_lossy().replace('\\', "/"),
                        fs::read(entry.path()).expect("snapshot bytes"),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        visit(root, Path::new(""), &mut files);
        files
    }

    #[test]
    fn signed_current_apply_rejects_unconnected_0_7_without_any_target_mutation() {
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        execute_setup(&SetupRequest {
            target: &consumer,
            answers: &phase1_fixture("answers-base.yml"),
            capabilities: &phase1_fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect("unconnected 0.7 setup");
        let harness_path = consumer.join(HARNESS_PATH);
        let harness = fs::read_to_string(&harness_path)
            .expect("installed harness")
            .replace(env!("CARGO_PKG_VERSION"), "0.7.0");
        fs::write(&harness_path, &harness).expect("historical harness version");
        assert!(harness.contains("harness_version = \"0.7.0\""));
        assert!(!harness.contains("preference_provenance"));
        let before = snapshot_regular_files(&consumer);
        let fixture = release_fixture();

        let error = execute_update(&update_request(&consumer, &fixture, UpdateMode::Apply))
            .expect_err("unconnected historical install must require setup");

        assert!(matches!(error, UpdateError::Unsupported(_)), "{error:?}");
        assert_eq!(error.code(), "hive.update-migration-unsupported");
        assert!(error.to_string().contains("validated user setup"));
        assert!(error.to_string().contains("shared-registry binding"));
        assert_eq!(snapshot_regular_files(&consumer), before);
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(!consumer.join(".hive/backups").exists());
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
    }

    #[test]
    fn current_updater_rejects_published_0_9_release_without_target_mutation() {
        let fixture = published_release_fixture();
        for mode in [UpdateMode::DryRun, UpdateMode::Apply] {
            let target = tempfile::tempdir().expect("target");
            let consumer = target.path().canonicalize().expect("consumer");
            install_legacy_consumer(&consumer, "0.6.0");
            let before = snapshot_regular_files(&consumer);

            let error = execute_update(&update_request(&consumer, &fixture, mode))
                .expect_err("running updater must reject another exact release version");

            assert!(matches!(error, UpdateError::Verification(_)));
            assert!(error.to_string().contains(&format!(
                "release target 0.9.0 differs from running updater {}",
                env!("CARGO_PKG_VERSION")
            )));
            assert_eq!(snapshot_regular_files(&consumer), before);
            assert!(!consumer.join(JOURNAL_PATH).exists());
            assert!(!consumer.join(UPDATE_STATE_PATH).exists());
            assert!(!consumer.join(".hive/backups").exists());
        }
    }

    #[test]
    fn synthetic_current_release_fixture_is_explicitly_test_only_and_public_only() {
        let fixture = release_fixture();
        let marker = fs::read_to_string(fixture.join("TEST-ONLY.md")).expect("test-only marker");
        let marker_lowered = marker.to_ascii_lowercase();
        assert!(marker.contains(&format!(
            "Test-only `{}` release bundle fixture",
            env!("CARGO_PKG_VERSION")
        )));
        assert!(marker.contains("Local updater and version-parity tests only"));
        assert!(marker_lowered.contains("public release artifact"));
        let manifest = fs::read_to_string(fixture.join("bundle-manifest.json"))
            .expect("synthetic integrity manifest");
        assert!(manifest.contains(&format!(
            r#""release_version":"{}""#,
            env!("CARGO_PKG_VERSION")
        )));
        assert!(manifest.contains("migration-table.json"));
        assert!(manifest.contains("release-surface-inventory.json"));

        for (path, bytes) in snapshot_regular_files(&fixture) {
            let path_lowered = path.to_ascii_lowercase();
            assert!(
                !["private", "secret", "credential", ".key", ".pem"]
                    .iter()
                    .any(|marker| path_lowered.contains(marker)),
                "private-material-like fixture path: {path}"
            );
            let lowered = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
            for forbidden in [
                "private_key",
                "secret_key",
                "begin private key",
                "pkcs8",
                "signing seed",
                "credential",
            ] {
                assert!(!lowered.contains(forbidden), "{path}: {forbidden}");
            }
        }
    }

    #[test]
    fn transaction_id_is_deterministic_and_contains_no_path_data() {
        let digest = format!("sha256:{}", "a".repeat(64));
        assert_eq!(transaction_id(&digest, 42), transaction_id(&digest, 42));
        assert!(transaction_id(&digest, 42).starts_with("txn-"));
        assert_ne!(transaction_id(&digest, 42), transaction_id(&digest, 43));
    }

    #[test]
    fn plan_digest_binds_every_before_and_after_digest() {
        let change = SetupChange {
            path: ".hive/config/harness.toml".to_owned(),
            before_digest: Some(format!("sha256:{}", "a".repeat(64))),
            after_digest: Some(format!("sha256:{}", "b".repeat(64))),
            foreign_before_digest: None,
            foreign_after_digest: None,
        };
        let first = plan_digest(
            "0.6.0",
            "0.7.0",
            &format!("sha256:{}", "c".repeat(64)),
            &format!("sha256:{}", "d".repeat(64)),
            std::slice::from_ref(&change),
        )
        .expect("digest");
        let changed = SetupChange {
            after_digest: Some(format!("sha256:{}", "e".repeat(64))),
            ..change
        };
        let second = plan_digest(
            "0.6.0",
            "0.7.0",
            &format!("sha256:{}", "c".repeat(64)),
            &format!("sha256:{}", "d".repeat(64)),
            &[changed],
        )
        .expect("digest");
        assert_ne!(first, second);
    }

    #[test]
    fn major_confirmation_binds_every_real_update_report_field() {
        let fixture = historical_release_fixture();
        let verified = verify_release_bundle(&fixture, None).expect("release");
        let source: SemVersion = "0.6.0".parse().expect("source");
        let target: SemVersion = "0.7.0".parse().expect("target");
        let plan = format!("sha256:{}", "1".repeat(64));
        let report = format!("sha256:{}", "2".repeat(64));
        let approval = MajorApproval {
            source_version: source,
            exact_target: target,
            release_plan_digest: plan.clone(),
            compatibility_report_digest: report.clone(),
            migration_table_digest: verified.manifest.migration_table_digest.clone(),
            human_confirmed: true,
            confirmation_digest: format!("sha256:{}", "3".repeat(64)),
        };
        validate_major_binding(
            UpdateMode::Apply,
            "0.6.0",
            &verified,
            Some(target),
            Some(&approval),
            &plan,
            &report,
            MigrationKind::CrossMajor,
        )
        .expect("bound approval");

        let mut attacks = Vec::new();
        attacks.push(MajorApproval {
            source_version: "0.5.0".parse().expect("source"),
            ..approval.clone()
        });
        attacks.push(MajorApproval {
            exact_target: "0.8.0".parse().expect("target"),
            ..approval.clone()
        });
        attacks.push(MajorApproval {
            release_plan_digest: format!("sha256:{}", "4".repeat(64)),
            ..approval.clone()
        });
        attacks.push(MajorApproval {
            compatibility_report_digest: format!("sha256:{}", "5".repeat(64)),
            ..approval.clone()
        });
        attacks.push(MajorApproval {
            migration_table_digest: format!("sha256:{}", "6".repeat(64)),
            ..approval.clone()
        });
        attacks.push(MajorApproval {
            human_confirmed: false,
            ..approval
        });
        for attack in attacks {
            assert!(validate_major_binding(
                UpdateMode::Apply,
                "0.6.0",
                &verified,
                Some(target),
                Some(&attack),
                &plan,
                &report,
                MigrationKind::CrossMajor,
            )
            .is_err());
        }
        assert!(validate_major_binding(
            UpdateMode::Apply,
            "0.6.0",
            &verified,
            Some(target),
            None,
            &plan,
            &report,
            MigrationKind::CrossMajor,
        )
        .is_err());
        validate_major_binding(
            UpdateMode::DryRun,
            "0.6.0",
            &verified,
            Some(target),
            None,
            &plan,
            &report,
            MigrationKind::CrossMajor,
        )
        .expect("dry-run prepares confirmation");
    }

    #[test]
    fn cross_major_plan_rejects_protected_and_foreign_marker_drift() {
        let before = vec![
            PreservationDigest {
                path: "AGENTS.md".to_owned(),
                kind: "user-markdown-body".to_owned(),
                digest: format!("sha256:{}", "a".repeat(64)),
            },
            PreservationDigest {
                path: "README.md".to_owned(),
                kind: "document".to_owned(),
                digest: format!("sha256:{}", "b".repeat(64)),
            },
        ];
        let marker = SetupChange {
            path: "AGENTS.md".to_owned(),
            before_digest: Some(format!("sha256:{}", "c".repeat(64))),
            after_digest: Some(format!("sha256:{}", "d".repeat(64))),
            foreign_before_digest: Some(format!("sha256:{}", "a".repeat(64))),
            foreign_after_digest: Some(format!("sha256:{}", "a".repeat(64))),
        };
        let after =
            planned_cross_major_successor(&before, std::slice::from_ref(&marker)).expect("plan");
        validate_cross_major_preservation(&before, &after, &BTreeSet::new())
            .expect("foreign bytes preserved");

        let foreign_drift = SetupChange {
            foreign_after_digest: Some(format!("sha256:{}", "e".repeat(64))),
            ..marker.clone()
        };
        assert!(planned_cross_major_successor(&before, &[foreign_drift]).is_err());

        let protected = SetupChange {
            path: "README.md".to_owned(),
            before_digest: Some(format!("sha256:{}", "b".repeat(64))),
            after_digest: Some(format!("sha256:{}", "f".repeat(64))),
            foreign_before_digest: None,
            foreign_after_digest: None,
        };
        assert!(planned_cross_major_successor(&before, &[protected]).is_err());
    }

    #[test]
    fn prepared_recovery_restores_only_the_journaled_after_bytes() {
        let target = tempfile::tempdir().expect("target");
        prepared_recovery_fixture(target.path(), b"after");
        recover_update(target.path()).expect("recover");
        assert_eq!(
            fs::read(target.path().join(".hive/config/harness.toml")).expect("restored"),
            b"before"
        );
        assert!(!target.path().join(JOURNAL_PATH).exists());
    }

    #[test]
    fn committed_pre_0_8_recovery_rebuilds_the_project_index() {
        let target = tempfile::tempdir().expect("target");
        committed_recovery_fixture(target.path(), "0.7.0");
        fs::create_dir_all(target.path().join(".hive/knowledge/Wiki")).expect("Wiki directory");
        fs::create_dir_all(target.path().join(".hive/knowledge/Raw")).expect("Raw directory");
        write_new_file(
            &target.path().join(".hive/knowledge/suppression.yml"),
            b"schema_version: 1\nentries: []\n",
        )
        .expect("suppression ledger");

        recover_update(target.path()).expect("recover");

        assert!(target.path().join(".hive/index/hive.sqlite3").is_file());
        assert!(!target.path().join(JOURNAL_PATH).exists());
    }

    #[test]
    fn post_0_8_project_index_cleanup_returns_no_digest() {
        let target = tempfile::tempdir().expect("target");
        write_new_file(&target.path().join(".hive/index/hive.sqlite3"), b"derived").expect("index");
        let target_dir = open_target_capability(target.path()).expect("target capability");
        let target_root = TargetRoot { dir: &target_dir };

        let digest =
            update_project_index_for_version(&target_root, "1.0.0").expect("index cleanup");

        assert!(digest.is_none());
        assert!(!target.path().join(".hive/index/hive.sqlite3").exists());
    }

    #[test]
    fn committed_0_8_recovery_removes_only_fixed_project_index_files() {
        let target = tempfile::tempdir().expect("target");
        committed_recovery_fixture(target.path(), "0.8.0");
        for relative in PROJECT_INDEX_FILES {
            write_new_file(&target.path().join(relative), relative.as_bytes()).expect("index file");
        }
        write_new_file(
            &target.path().join(".hive/index/foreign.sqlite3"),
            b"foreign",
        )
        .expect("foreign index-adjacent file");

        recover_update(target.path()).expect("recover");

        for relative in PROJECT_INDEX_FILES {
            assert!(!target.path().join(relative).exists(), "{relative}");
        }
        assert_eq!(
            fs::read(target.path().join(".hive/index/foreign.sqlite3")).expect("foreign bytes"),
            b"foreign"
        );
        assert!(!target.path().join(JOURNAL_PATH).exists());
    }

    #[test]
    fn recovery_preserves_concurrent_user_bytes_and_keeps_the_journal() {
        let target = tempfile::tempdir().expect("target");
        prepared_recovery_fixture(target.path(), b"user-edit");
        assert!(matches!(
            recover_update(target.path()),
            Err(UpdateError::Conflict(_))
        ));
        assert_eq!(
            fs::read(target.path().join(".hive/config/harness.toml")).expect("preserved"),
            b"user-edit"
        );
        assert!(target.path().join(JOURNAL_PATH).exists());
    }

    #[cfg(unix)]
    #[test]
    fn pinned_recovery_uses_displaced_root_and_preserves_replacement() {
        let temporary = tempfile::tempdir().expect("temporary");
        let target = temporary.path().join("consumer");
        let displaced = temporary.path().join("consumer-displaced");
        fs::create_dir(&target).expect("target");
        prepared_recovery_fixture(&target, b"after");
        let target_dir = open_target_capability(&target).expect("target capability");
        fs::rename(&target, &displaced).expect("displace target");
        fs::create_dir(&target).expect("replacement target");
        fs::write(target.join("sentinel"), b"replacement").expect("replacement sentinel");

        recover_update_in(&target_dir).expect("pinned recovery");

        assert_eq!(
            fs::read(displaced.join(HARNESS_PATH)).expect("restored harness"),
            b"before"
        );
        assert!(!displaced.join(JOURNAL_PATH).exists());
        assert_eq!(
            fs::read(target.join("sentinel")).expect("replacement sentinel"),
            b"replacement"
        );
        assert_eq!(
            fs::read_dir(&target).expect("replacement entries").count(),
            1
        );
    }

    #[cfg(windows)]
    #[test]
    fn pinned_target_capability_blocks_ambient_replacement_while_open() {
        let temporary = tempfile::tempdir().expect("temporary");
        let target = temporary.path().join("consumer");
        let displaced = temporary.path().join("consumer-displaced");
        fs::create_dir(&target).expect("target");
        let target_dir = open_target_capability(&target).expect("target capability");

        fs::rename(&target, &displaced)
            .expect_err("open Windows target capability should block replacement");
        assert!(target.is_dir());
        assert!(!displaced.exists());

        drop(target_dir);
        fs::rename(&target, &displaced).expect("rename after target capability release");
    }

    #[test]
    fn dry_run_requires_explicit_recovery_without_mutating_any_durable_bytes() {
        let target = tempfile::tempdir().expect("target");
        let target_path = target.path().canonicalize().expect("canonical target");
        prepared_recovery_fixture(&target_path, b"after");
        write_new_file(
            &target_path.join(UPDATE_STATE_PATH),
            b"{\"existing\":\"state\"}\n",
        )
        .expect("state");
        write_new_file(
            &target_path.join(".hive/index/hive.sqlite3"),
            b"existing-index",
        )
        .expect("index");
        let before = snapshot_regular_files(&target_path);
        let fixture = release_fixture();

        let error = execute_update(&update_request(&target_path, &fixture, UpdateMode::DryRun))
            .expect_err("dry-run must not recover");

        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert!(error.to_string().contains("recovery required"));
        assert_eq!(snapshot_regular_files(&target_path), before);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_replace_syncs_the_containing_directory() {
        let target = tempfile::tempdir().expect("target");
        fs::create_dir_all(target.path().join(".hive/runtime")).expect("runtime directory");
        let target_dir = open_target_capability(target.path()).expect("target capability");
        let target_root = TargetRoot { dir: &target_dir };
        DIRECTORY_SYNC_EVENTS.with(|events| events.borrow_mut().clear());

        write_atomic_relative(
            &target_root,
            Path::new(".hive/runtime/state.json"),
            b"{\"state\":\"ready\"}\n",
        )
        .expect("atomic write");

        assert_eq!(
            fs::read(target.path().join(".hive/runtime/state.json")).expect("state bytes"),
            b"{\"state\":\"ready\"}\n"
        );
        DIRECTORY_SYNC_EVENTS.with(|events| {
            assert_eq!(
                events.borrow().as_slice(),
                [PathBuf::from(".hive/runtime/state.json")]
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn new_backup_syncs_every_created_ancestor_and_the_file_entry() {
        let target = tempfile::tempdir().expect("target");
        let target_dir = open_target_capability(target.path()).expect("target capability");
        let target_root = TargetRoot { dir: &target_dir };
        let relative = Path::new(".hive/backups/txn-test/files/config.toml");
        DIRECTORY_SYNC_EVENTS.with(|events| events.borrow_mut().clear());

        write_new_target_file(&target_root, relative, b"backup").expect("backup write");

        DIRECTORY_SYNC_EVENTS.with(|events| {
            assert_eq!(events.borrow().as_slice(), [relative; 5]);
        });
    }

    #[cfg(unix)]
    #[test]
    fn recovery_syncs_restored_bytes_before_the_journal_deletion() {
        let target = tempfile::tempdir().expect("target");
        prepared_recovery_fixture(target.path(), b"after");
        let target_dir = open_target_capability(target.path()).expect("target capability");
        DIRECTORY_SYNC_EVENTS.with(|events| events.borrow_mut().clear());

        recover_update_in(&target_dir).expect("recover");

        DIRECTORY_SYNC_EVENTS.with(|events| {
            let events = events.borrow();
            let restored = events
                .iter()
                .position(|event| event == Path::new(HARNESS_PATH))
                .expect("restored file sync");
            let journal = events
                .iter()
                .position(|event| event == Path::new(JOURNAL_PATH))
                .expect("journal deletion sync");
            assert!(restored < journal);
            assert_eq!(journal, events.len() - 1);
        });
    }

    #[test]
    fn public_update_and_recovery_apis_reject_a_pinned_source_workspace() {
        let target = tempfile::tempdir().expect("target");
        fs::write(target.path().join("hive-source.json"), b"{}\n").expect("source marker");
        let target_dir = open_target_capability(target.path()).expect("target capability");
        let fixture = release_fixture();
        let request = update_request(target.path(), &fixture, UpdateMode::DryRun);

        let update_error =
            execute_update_in(&target_dir, &request).expect_err("pinned source update");
        let recovery_error = recover_update_in(&target_dir).expect_err("pinned source recovery");
        let wrapper_recovery_error =
            recover_update(target.path()).expect_err("source recovery wrapper");

        for error in [update_error, recovery_error, wrapper_recovery_error] {
            assert!(matches!(error, UpdateError::Input(_)));
            assert!(error.to_string().contains("source workspace"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn pinned_update_applies_to_displaced_root_without_touching_replacement() {
        let temporary = tempfile::tempdir().expect("temporary");
        let parent = temporary.path().canonicalize().expect("canonical parent");
        let target = parent.join("consumer");
        let displaced = parent.join("consumer-displaced");
        fs::create_dir(&target).expect("target");
        install_historical_consumer(&target, "0.9.1");
        let target_dir = open_target_capability(&target).expect("target capability");
        fs::rename(&target, &displaced).expect("displace target");
        fs::create_dir(&target).expect("replacement target");
        fs::write(target.join("sentinel"), b"replacement").expect("replacement sentinel");
        let fixture = release_fixture();

        execute_update_in(
            &target_dir,
            &update_request(&target, &fixture, UpdateMode::Apply),
        )
        .expect("pinned apply");

        assert_eq!(
            fs::read(target.join("sentinel")).expect("replacement sentinel"),
            b"replacement"
        );
        assert_eq!(
            fs::read_dir(&target).expect("replacement entries").count(),
            1
        );
        assert!(fs::read_to_string(displaced.join(HARNESS_PATH))
            .expect("updated harness")
            .contains(env!("CARGO_PKG_VERSION")));
    }

    #[cfg(unix)]
    #[test]
    fn pinned_update_rejects_replaced_ancestor_and_file_symlinks() {
        use std::os::unix::fs::symlink;

        for replace_file in [false, true] {
            let temporary = tempfile::tempdir().expect("temporary");
            let target = temporary.path().join("consumer");
            fs::create_dir(&target).expect("target");
            install_historical_consumer(&target, "0.9.1");
            let target_dir = open_target_capability(&target).expect("target capability");
            let outside = temporary.path().join("outside");
            fs::create_dir(&outside).expect("outside");
            fs::write(outside.join("sentinel"), b"outside").expect("outside sentinel");
            if replace_file {
                let harness = target.join(HARNESS_PATH);
                fs::rename(&harness, target.join(".hive/config/harness-real.toml"))
                    .expect("displace harness");
                symlink(outside.join("sentinel"), &harness).expect("replace harness");
            } else {
                let config = target.join(".hive/config");
                fs::rename(&config, target.join(".hive/config-real")).expect("displace config");
                symlink(&outside, &config).expect("replace config");
            }
            let fixture = release_fixture();

            let result = execute_update_in(
                &target_dir,
                &update_request(&target, &fixture, UpdateMode::DryRun),
            );

            assert!(matches!(result, Err(UpdateError::Conflict(_))));
            assert_eq!(
                fs::read(outside.join("sentinel")).expect("outside sentinel"),
                b"outside"
            );
        }
    }

    #[test]
    fn forged_recovery_path_is_rejected_without_touching_foreign_bytes() {
        let target = tempfile::tempdir().expect("target");
        let mut journal = prepared_recovery_fixture(target.path(), b"after");
        write_new_file(&target.path().join("README.md"), b"foreign").expect("foreign file");
        journal.changes[0].path = "README.md".to_owned();
        journal.changes[0].backup_path = "files/README.md".to_owned();
        journal.changes[0].after_digest = Some(sha256_digest(b"foreign"));
        journal.journal_digest = journal_digest(&journal).expect("journal digest");
        write_atomic(
            &target.path().join(JOURNAL_PATH),
            &json_line(&journal, "journal").expect("journal bytes"),
        )
        .expect("replace journal");

        assert!(matches!(
            recover_update(target.path()),
            Err(UpdateError::Conflict(_))
        ));
        assert_eq!(
            fs::read(target.path().join("README.md")).expect("foreign bytes"),
            b"foreign"
        );
        assert_eq!(
            fs::read(target.path().join(".hive/config/harness.toml")).expect("live bytes"),
            b"after"
        );
    }

    #[test]
    fn canonical_backup_excludes_index_and_prunes_only_after_seven_days() {
        let target = tempfile::tempdir().expect("target");
        write_new_file(&target.path().join(".hive/config/harness.toml"), b"harness")
            .expect("config");
        write_new_file(
            &target.path().join(".hive/knowledge/Wiki/page.md"),
            b"knowledge",
        )
        .expect("knowledge");
        write_new_file(&target.path().join(".hive/index/hive.sqlite3"), b"derived").expect("index");
        let transaction_id = format!("txn-{}", "a".repeat(24));
        let target_dir = open_target_capability(target.path()).expect("target capability");
        let target_root = TargetRoot { dir: &target_dir };
        let manifest = create_backup(
            &target_root,
            &transaction_id,
            "0.6.0",
            "0.7.0",
            1_000,
            &format!("sha256:{}", "b".repeat(64)),
            &[],
        )
        .expect("backup");
        assert_eq!(manifest.entries.len(), 2);
        assert!(manifest
            .entries
            .iter()
            .all(|entry| !entry.path.starts_with(".hive/index/")));
        assert_eq!(
            prune_expired_backups_in(&target_root, 1_000 + crate::BACKUP_RETENTION_SECONDS,),
            0
        );
        assert_eq!(
            prune_expired_backups_in(&target_root, 1_001 + crate::BACKUP_RETENTION_SECONDS,),
            1
        );
        assert!(!target
            .path()
            .join(format!(".hive/backups/{transaction_id}"))
            .exists());
        assert!(target.path().join(".hive/index/hive.sqlite3").exists());
    }

    #[test]
    fn every_supported_same_major_generation_dry_runs_and_applies_without_foreign_drift() {
        let _environment = UPDATE_ENVIRONMENT.lock().expect("environment lock");
        let fixture = release_fixture();
        for source_version in ["0.9.1", "0.9.2", "0.9.3", "0.9.4"] {
            let target = tempfile::tempdir().expect("target");
            let consumer = target.path().canonicalize().expect("consumer");
            install_historical_consumer(&consumer, source_version);
            let harness_path = consumer.join(HARNESS_PATH);
            let harness = fs::read_to_string(&harness_path).expect("harness").replace(
                "usage_stop_remaining_percent = 20",
                "usage_stop_remaining_percent = 37",
            );
            fs::write(&harness_path, harness).expect("configured threshold");
            let before_readme = fs::read(consumer.join("README.md")).expect("readme");
            let before_omx = fs::read(consumer.join(".omx/state.json")).expect("omx");

            let dry_run = execute_update(&update_request(&consumer, &fixture, UpdateMode::DryRun))
                .unwrap_or_else(|error| panic!("{source_version} dry-run failed: {error}"));
            assert_eq!(dry_run.source_version, source_version);
            assert_eq!(dry_run.target_version, env!("CARGO_PKG_VERSION"));
            assert_eq!(dry_run.migration_id, "same-major-render-v1");

            let applied = execute_update(&update_request(&consumer, &fixture, UpdateMode::Apply))
                .unwrap_or_else(|error| panic!("{source_version} apply failed: {error}"));
            assert_eq!(applied.source_version, source_version);
            assert_eq!(applied.target_version, env!("CARGO_PKG_VERSION"));
            assert_eq!(
                fs::read(consumer.join("README.md")).expect("readme"),
                before_readme
            );
            assert_eq!(
                fs::read(consumer.join(".omx/state.json")).expect("omx"),
                before_omx
            );
            let migrated = fs::read_to_string(consumer.join(HARNESS_PATH)).expect("harness");
            assert!(migrated.contains(env!("CARGO_PKG_VERSION")));
            assert!(
                migrated.contains("usage_stop_remaining_percent = 37"),
                "{migrated}"
            );
        }
    }

    #[test]
    fn signed_update_preserves_foreign_bytes_and_removes_legacy_project_index() {
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_historical_consumer(&consumer, "0.9.1");
        let fixture = release_fixture();
        let before_readme = fs::read(consumer.join("README.md")).expect("readme");
        let before_omx = fs::read(consumer.join(".omx/state.json")).expect("omx");

        let dry_run = execute_update(&update_request(&consumer, &fixture, UpdateMode::DryRun))
            .expect("dry-run");
        assert_eq!(dry_run.source_version, "0.9.1");
        assert_eq!(dry_run.target_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(dry_run.migration_id, "same-major-render-v1");
        assert!(dry_run.backup_id.is_none());
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(fs::read_to_string(consumer.join(HARNESS_PATH))
            .expect("harness")
            .contains("0.9.1"));

        let applied =
            execute_update(&update_request(&consumer, &fixture, UpdateMode::Apply)).expect("apply");
        assert_eq!(applied.migration_id, "same-major-render-v1");
        assert!(applied.backup_id.is_some());
        assert!(applied.index_digest.is_none());
        assert!(consumer.join(UPDATE_STATE_PATH).is_file());
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(!consumer.join(".hive/index/hive.sqlite3").exists());
        assert!(consumer
            .join(".agents/directives/03-session-coordination.md")
            .is_file());
        assert_eq!(
            fs::read(consumer.join("README.md")).expect("readme"),
            before_readme
        );
        assert_eq!(
            fs::read(consumer.join(".omx/state.json")).expect("omx"),
            before_omx
        );
    }

    #[test]
    fn forged_historical_projection_is_rejected() {
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_historical_consumer(&consumer, "0.9.1");
        let projection = consumer.join(".agents/directives/00-project-harness.md");
        let forged = b"forged historical directive\n";
        fs::write(&projection, forged).expect("forged historical projection");

        let fixture = release_fixture();
        assert!(matches!(
            execute_update(&update_request(
                &consumer,
                &fixture,
                UpdateMode::DryRun,
            )),
            Err(UpdateError::Conflict(message))
                if message.contains("projected historical bytes changed")
        ));
        assert_eq!(
            fs::read(projection).expect("forged bytes preserved"),
            forged
        );
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
        assert!(!consumer.join(JOURNAL_PATH).exists());
    }

    #[test]
    fn signed_update_rejects_a_foreign_file_at_a_new_projection_path() {
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_historical_consumer(&consumer, "0.9.1");
        let directive = consumer.join(".agents/directives/03-session-coordination.md");
        fs::create_dir_all(directive.parent().expect("directive parent"))
            .expect("directive parent");
        let foreign = b"foreign directive bytes\x00\xff\n";
        fs::write(&directive, foreign).expect("foreign directive");
        let fixture = release_fixture();

        let error = execute_update(&update_request(&consumer, &fixture, UpdateMode::DryRun))
            .expect_err("new projection must not overwrite a foreign file");

        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("collides with a foreign file"));
        assert_eq!(fs::read(directive).expect("foreign directive"), foreign);
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
        assert!(!consumer.join(JOURNAL_PATH).exists());
    }

    #[test]
    fn injected_activation_failure_keeps_legacy_generation_and_recovers_exactly() {
        let _environment = UPDATE_ENVIRONMENT.lock().expect("environment lock");
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_historical_consumer(&consumer, "0.9.1");
        let fixture = release_fixture();
        let before_readme = fs::read(consumer.join("README.md")).expect("readme");
        let activation_fault = format!("{:?}@1", std::thread::current().id());
        std::env::set_var("HIVE_TEST_ACTIVATION_FAIL_AFTER", activation_fault);
        let result = execute_update(&update_request(&consumer, &fixture, UpdateMode::Apply));
        std::env::remove_var("HIVE_TEST_ACTIVATION_FAIL_AFTER");

        assert!(result.is_err());
        assert!(consumer.join(JOURNAL_PATH).is_file());
        assert!(fs::read_to_string(consumer.join(HARNESS_PATH))
            .expect("harness")
            .contains("0.9.1"));
        assert_eq!(
            fs::read(consumer.join("README.md")).expect("readme"),
            before_readme
        );
        recover_update(&consumer).expect("recovery");
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(fs::read_to_string(consumer.join(HARNESS_PATH))
            .expect("harness")
            .contains("0.9.1"));
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
    }
}
