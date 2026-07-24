use crate::{
    classify_release, observe_surface_delta, select_migration_route,
    validate_cross_major_preservation, verify_release_repository, BackupEntry, BackupManifest,
    MajorApproval, MigrationKind, PreservationDigest, ReleaseVerification, RollbackState,
    SemVersion, SurfaceDelta, UpdateError,
};
use hive_core::{
    ensure_no_symlink_ancestors, ensure_no_symlink_ancestors_for_hive_skill_projection,
    is_hive_skill_projection_path, sha256_digest, validate_hive_skill_projection_relative,
    validate_project_relative,
};
use hive_render::{
    execute_setup, shared_marker_foreign_digest, update_path_is_owned, RenderError, SetupChange,
    SetupMode, SetupOutcome, SetupRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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
    /// Extracted local TUF repository.
    pub repository: &'a Path,
    /// Independently protected trusted-root bytes.
    pub trusted_root_bytes: &'a [u8],
    /// Verification clock supplied by the CLI.
    pub now_unix: i64,
    /// Dry-run or apply.
    pub mode: UpdateMode,
    /// Exact user-supplied breaking target. Never inferred by Hive.
    pub exact_major_target: Option<SemVersion>,
    /// Exact optional breaking-release authority.
    pub major_approval: Option<&'a MajorApproval>,
}

/// Persisted release rollback state.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateState {
    /// Schema generation.
    pub schema_version: u32,
    /// Installed product version.
    pub product_version: String,
    /// Last accepted release manifest digest.
    pub release_manifest_digest: String,
    /// TUF/release rollback floor.
    pub rollback: RollbackState,
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
    hive_core::ensure_consumer_target(request.target)
        .map_err(|error| UpdateError::Input(error.to_string()))?;
    if journal_exists(request.target)? {
        recover_update(request.target)?;
    }
    let prepared = prepare_update(request)?;
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
    activate_update(request, prepared)
}

fn prepare_update(request: &UpdateRequest<'_>) -> Result<PreparedUpdate, UpdateError> {
    let previous_state = read_update_state(request.target)?;
    let verified = verify_release_repository(
        request.trusted_root_bytes,
        request.repository,
        request.now_unix,
        previous_state.as_ref().map(|state| &state.rollback),
    )?;
    let installed = read_installed_harness(request.target)?;
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
    let transition = validate_update_transition(&installed, &verified, request.exact_major_target)?;
    let preservation_before = if transition.migration_kind == MigrationKind::CrossMajor {
        Some(snapshot_protected_tree(request.target)?)
    } else {
        None
    };

    let capabilities_json = installed_capabilities_json(request.target)?;
    let answers_yaml =
        installed_answers_yaml(request.target, installed.usage_stop_remaining_percent)?;
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
    let dry_run = execute_setup(&SetupRequest {
        target: request.target,
        answers: answers.path(),
        capabilities: capabilities.path(),
        mode: SetupMode::DryRun,
        reconfigure_roles: BTreeSet::new(),
    })
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

fn snapshot_protected_tree(target: &Path) -> Result<Vec<PreservationDigest>, UpdateError> {
    let mut result = Vec::new();
    snapshot_directory(target, target, &mut result)?;
    result.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(result)
}

fn snapshot_directory(
    target: &Path,
    directory: &Path,
    result: &mut Vec<PreservationDigest>,
) -> Result<(), UpdateError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| UpdateError::Internal(format!("cannot scan preservation tree: {error}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            UpdateError::Internal(format!("cannot scan preservation tree: {error}"))
        })?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(target).map_err(|_| {
            UpdateError::Internal("preservation path escaped the consumer target".to_owned())
        })?;
        let relative_string = relative.to_string_lossy().replace('\\', "/");
        if preservation_path_is_excluded(&relative_string) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            UpdateError::Internal(format!(
                "cannot inspect preservation path {relative_string}: {error}"
            ))
        })?;
        if metadata.is_dir() {
            snapshot_directory(target, &path, result)?;
            continue;
        }
        if is_cross_major_system_path(&relative_string) {
            continue;
        }
        let digest = if metadata.file_type().is_symlink() {
            let destination = fs::read_link(&path).map_err(|error| {
                UpdateError::Internal(format!(
                    "cannot read preservation symlink {relative_string}: {error}"
                ))
            })?;
            sha256_digest(format!("symlink\0{}", destination.to_string_lossy()).as_bytes())
        } else if metadata.is_file() {
            if relative_string == "AGENTS.md" {
                let bytes = fs::read(&path).map_err(|error| {
                    UpdateError::Internal(format!(
                        "cannot read shared marker for preservation: {error}"
                    ))
                })?;
                shared_marker_foreign_digest(&bytes).map_err(map_render_error)?
            } else {
                hash_regular_file(&path)?
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

fn hash_regular_file(path: &Path) -> Result<String, UpdateError> {
    let mut file = fs::File::open(path)
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
        request.target,
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
        rollback: verified.next_rollback_state.clone(),
    };
    let mut journal = create_journal(
        &transaction_id,
        &installed,
        &verified,
        &plan_digest,
        &dry_run.changes,
        next_state,
    )?;
    persist_journal(request.target, &journal)?;

    let applied = execute_setup(&SetupRequest {
        target: request.target,
        answers: answers.path(),
        capabilities: capabilities.path(),
        mode: SetupMode::Apply,
        reconfigure_roles: BTreeSet::new(),
    })
    .map_err(|error| {
        let mapped = map_render_error(error);
        let _ = mark_needs_recovery(request.target, &mut journal);
        mapped
    })?;
    if applied.changes != dry_run.changes || applied.tree_digest != dry_run.tree_digest {
        mark_needs_recovery(request.target, &mut journal)?;
        return Err(UpdateError::Conflict(
            "setup plan changed between dry-run and activation".to_owned(),
        ));
    }
    verify_after_digests(request.target, &journal.changes)?;
    if migration_kind == MigrationKind::CrossMajor {
        let before = preservation_before.as_deref().ok_or_else(|| {
            UpdateError::Internal("cross-major preservation baseline is missing".to_owned())
        })?;
        let after = snapshot_protected_tree(request.target)?;
        let mutable = cross_major_mutable_paths(&applied.changes)?;
        if let Err(error) = validate_cross_major_preservation(before, &after, &mutable) {
            mark_needs_recovery(request.target, &mut journal)?;
            return Err(error);
        }
    }
    journal.state = JournalState::Committed;
    journal.journal_digest = journal_digest(&journal)?;
    persist_journal(request.target, &journal)?;
    write_update_state(request.target, &journal.next_state)?;
    let index = hive_wiki::rebuild_index(request.target).map_err(|error| {
        UpdateError::Verification(format!(
            "successor committed but disposable index rebuild failed: {error}"
        ))
    })?;
    remove_exact_regular_file(request.target, Path::new(JOURNAL_PATH))?;
    let _pruned_backup_count = prune_expired_backups(request.target, request.now_unix);
    Ok(UpdateOutcome {
        changed_paths: applied.changed_paths,
        plan_digest,
        compatibility_report_digest,
        migration_table_digest: verified.manifest.migration_table_digest,
        source_version: installed.harness_version,
        target_version: verified.manifest.release_version,
        migration_id,
        backup_id: Some(backup.transaction_id),
        index_digest: Some(index.logical_digest),
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

fn persist_journal(target: &Path, journal: &UpdateJournal) -> Result<(), UpdateError> {
    ensure_no_symlink_ancestors(target, Path::new(JOURNAL_PATH))
        .map_err(|error| UpdateError::Conflict(error.to_string()))?;
    write_atomic(
        &target.join(JOURNAL_PATH),
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
    let Some(bytes) = read_optional_bounded(target, Path::new(JOURNAL_PATH), MAX_CONFIG_BYTES)?
    else {
        return Ok(());
    };
    let journal: UpdateJournal = serde_json::from_slice(&bytes)
        .map_err(|error| UpdateError::Conflict(format!("update journal is invalid: {error}")))?;
    validate_recovery_journal(target, &journal)?;
    match journal.state {
        JournalState::Committed => {
            verify_after_digests(target, &journal.changes)?;
            write_update_state(target, &journal.next_state)?;
            hive_wiki::rebuild_index(target).map_err(|error| {
                UpdateError::Verification(format!(
                    "cannot rebuild disposable index during forward recovery: {error}"
                ))
            })?;
        }
        JournalState::Prepared | JournalState::NeedsRecovery => {
            rollback_changes(target, &journal)?;
        }
    }
    remove_exact_regular_file(target, Path::new(JOURNAL_PATH))
}

fn validate_recovery_journal(
    target: &Path,
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
        || journal.next_state.rollback.manifest_digest != journal.release_manifest_digest
        || journal.next_state.rollback.root_version == 0
        || journal.next_state.rollback.timestamp_version == 0
        || journal.next_state.rollback.snapshot_version == 0
        || journal.next_state.rollback.targets_version == 0
        || journal.next_state.rollback.release_sequence == 0
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
    target: &Path,
    transaction_id: &str,
    source_version: &str,
    target_version: &str,
    now_unix: i64,
    tree_digest: &str,
    changes: &[SetupChange],
) -> Result<BackupManifest, UpdateError> {
    let root_relative = PathBuf::from(format!(".hive/backups/{transaction_id}"));
    ensure_no_symlink_ancestors(target, &root_relative)
        .map_err(|error| UpdateError::Conflict(error.to_string()))?;
    let root = target.join(&root_relative);
    fs::create_dir_all(root.join("files"))
        .map_err(|error| UpdateError::Internal(format!("cannot create update backup: {error}")))?;
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
                write_new_file(&root.join(&backup_path), &bytes)?;
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
    write_new_file(
        &root.join("backup-manifest.json"),
        &json_line(&manifest, "backup manifest")?,
    )?;
    Ok(manifest)
}

fn collect_canonical_snapshot_paths(
    target: &Path,
) -> Result<BTreeMap<String, String>, UpdateError> {
    let mut paths = BTreeMap::new();
    for root in CANONICAL_BACKUP_ROOTS {
        collect_canonical_path(target, Path::new(root), &mut paths)?;
    }
    Ok(paths)
}

fn collect_canonical_path(
    target: &Path,
    relative: &Path,
    paths: &mut BTreeMap<String, String>,
) -> Result<(), UpdateError> {
    if !crate::backup_path_is_allowed(&relative.to_string_lossy()) {
        return Ok(());
    }
    ensure_no_symlink_ancestors(target, relative)
        .map_err(|error| UpdateError::Conflict(error.to_string()))?;
    let absolute = target.join(relative);
    let metadata = match fs::symlink_metadata(&absolute) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
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
    let mut children = fs::read_dir(&absolute)
        .map_err(|error| {
            UpdateError::Internal(format!(
                "cannot enumerate canonical backup path {}: {error}",
                relative.display()
            ))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| UpdateError::Internal(format!("cannot read backup entry: {error}")))?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children {
        collect_canonical_path(target, &relative.join(child.file_name()), paths)?;
    }
    Ok(())
}

fn prune_expired_backups(target: &Path, now_unix: i64) -> usize {
    let backups_relative = Path::new(".hive/backups");
    if ensure_no_symlink_ancestors(target, backups_relative).is_err() {
        return 0;
    }
    let backups = target.join(backups_relative);
    let Ok(entries) = fs::read_dir(&backups) else {
        return 0;
    };
    let mut pruned = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !valid_transaction_directory_name(&name) {
            continue;
        }
        let root = entry.path();
        let Some(manifest) = validated_expired_backup(&root, &name, now_unix) else {
            continue;
        };
        if remove_validated_backup(&root, &manifest).is_ok() {
            pruned += 1;
        }
    }
    pruned
}

fn valid_transaction_directory_name(name: &str) -> bool {
    name.strip_prefix("txn-").is_some_and(|suffix| {
        suffix.len() == 24
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn validated_expired_backup(
    root: &Path,
    transaction_id: &str,
    now_unix: i64,
) -> Option<BackupManifest> {
    let metadata = fs::symlink_metadata(root).ok()?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return None;
    }
    let bytes = fs::read(root.join("backup-manifest.json")).ok()?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return None;
    }
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

fn remove_validated_backup(root: &Path, manifest: &BackupManifest) -> Result<(), UpdateError> {
    let mut expected_files = BTreeSet::from(["backup-manifest.json".to_owned()]);
    for entry in &manifest.entries {
        let backup = root.join(&entry.backup_path);
        match entry.prior_digest.as_deref() {
            Some(expected) => {
                let bytes = fs::read(&backup).map_err(|error| {
                    UpdateError::Rollback(format!("cannot verify expired backup: {error}"))
                })?;
                if bytes.len() as u64 != entry.prior_length || sha256_digest(&bytes) != expected {
                    return Err(UpdateError::Rollback(
                        "expired backup bytes differ from the manifest".to_owned(),
                    ));
                }
                expected_files.insert(entry.backup_path.clone());
            }
            None if backup.exists() => {
                return Err(UpdateError::Conflict(
                    "expired backup contains an unexpected file".to_owned(),
                ));
            }
            None => {}
        }
    }
    let (actual_files, mut directories) = enumerate_backup_tree(root)?;
    if actual_files != expected_files {
        return Err(UpdateError::Conflict(
            "expired backup contains foreign or missing files".to_owned(),
        ));
    }
    for relative in actual_files {
        let absolute = root.join(relative);
        let metadata = fs::symlink_metadata(&absolute)
            .map_err(|error| UpdateError::Rollback(format!("cannot recheck backup: {error}")))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(UpdateError::Conflict(
                "expired backup changed before cleanup".to_owned(),
            ));
        }
        fs::remove_file(absolute)
            .map_err(|error| UpdateError::Rollback(format!("cannot prune backup file: {error}")))?;
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        fs::remove_dir(root.join(directory)).map_err(|error| {
            UpdateError::Rollback(format!("cannot prune backup directory: {error}"))
        })?;
    }
    fs::remove_dir(root)
        .map_err(|error| UpdateError::Rollback(format!("cannot prune backup root: {error}")))
}

fn enumerate_backup_tree(root: &Path) -> Result<(BTreeSet<String>, Vec<PathBuf>), UpdateError> {
    let mut files = BTreeSet::new();
    let mut directories = Vec::new();
    enumerate_backup_directory(root, Path::new(""), &mut files, &mut directories)?;
    Ok((files, directories))
}

fn enumerate_backup_directory(
    root: &Path,
    relative: &Path,
    files: &mut BTreeSet<String>,
    directories: &mut Vec<PathBuf>,
) -> Result<(), UpdateError> {
    let directory = root.join(relative);
    for entry in fs::read_dir(&directory)
        .map_err(|error| UpdateError::Rollback(format!("cannot enumerate backup: {error}")))?
    {
        let entry = entry
            .map_err(|error| UpdateError::Rollback(format!("cannot read backup entry: {error}")))?;
        let child = relative.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| UpdateError::Rollback(format!("cannot inspect backup: {error}")))?;
        if file_type.is_symlink() {
            return Err(UpdateError::Conflict(
                "expired backup contains a symlink".to_owned(),
            ));
        }
        if file_type.is_dir() {
            directories.push(child.clone());
            enumerate_backup_directory(root, &child, files, directories)?;
        } else if file_type.is_file() {
            files.insert(child.to_string_lossy().replace('\\', "/"));
        } else {
            return Err(UpdateError::Conflict(
                "expired backup contains a nonregular entry".to_owned(),
            ));
        }
    }
    Ok(())
}

fn rollback_changes(target: &Path, journal: &UpdateJournal) -> Result<(), UpdateError> {
    let backup_root = target.join(format!(".hive/backups/{}", journal.transaction_id));
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
                let bytes = fs::read(backup_root.join(&change.backup_path)).map_err(|error| {
                    UpdateError::Rollback(format!(
                        "cannot read recovery backup for {}: {error}",
                        change.path
                    ))
                })?;
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

fn verify_after_digests(target: &Path, changes: &[JournalChange]) -> Result<(), UpdateError> {
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

fn read_installed_harness(target: &Path) -> Result<InstalledHarness, UpdateError> {
    let bytes = read_required_bounded(target, Path::new(HARNESS_PATH), MAX_CONFIG_BYTES)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| UpdateError::Input("installed harness TOML is not UTF-8".to_owned()))?;
    toml::from_str(text)
        .map_err(|error| UpdateError::Input(format!("invalid installed harness TOML: {error}")))
}

fn installed_capabilities_json(target: &Path) -> Result<Vec<u8>, UpdateError> {
    let bytes = read_required_bounded(target, Path::new(CAPABILITIES_PATH), MAX_CONFIG_BYTES)?;
    let value: JsonValue = serde_yaml::from_slice(&bytes).map_err(|error| {
        UpdateError::Input(format!("invalid installed capability YAML: {error}"))
    })?;
    serde_json::to_vec(&value).map_err(|error| {
        UpdateError::Internal(format!("cannot normalize installed capabilities: {error}"))
    })
}

fn installed_answers_yaml(target: &Path, threshold: u8) -> Result<Vec<u8>, UpdateError> {
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

fn read_update_state(target: &Path) -> Result<Option<UpdateState>, UpdateError> {
    read_optional_bounded(target, Path::new(UPDATE_STATE_PATH), MAX_CONFIG_BYTES)?
        .map(|bytes| {
            serde_json::from_slice(&bytes).map_err(|error| {
                UpdateError::Verification(format!("invalid update state: {error}"))
            })
        })
        .transpose()
}

fn write_update_state(target: &Path, state: &UpdateState) -> Result<(), UpdateError> {
    ensure_no_symlink_ancestors(target, Path::new(UPDATE_STATE_PATH))
        .map_err(|error| UpdateError::Conflict(error.to_string()))?;
    write_atomic(
        &target.join(UPDATE_STATE_PATH),
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

fn mark_needs_recovery(target: &Path, journal: &mut UpdateJournal) -> Result<(), UpdateError> {
    journal.state = JournalState::NeedsRecovery;
    journal.journal_digest = journal_digest(journal)?;
    ensure_no_symlink_ancestors(target, Path::new(JOURNAL_PATH))
        .map_err(|error| UpdateError::Conflict(error.to_string()))?;
    write_atomic(
        &target.join(JOURNAL_PATH),
        &json_line(journal, "update journal")?,
    )
}

fn journal_exists(target: &Path) -> Result<bool, UpdateError> {
    Ok(read_optional_bounded(target, Path::new(JOURNAL_PATH), MAX_CONFIG_BYTES)?.is_some())
}

fn digest_optional(target: &Path, relative: &Path) -> Result<Option<String>, UpdateError> {
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
    target: &Path,
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
    target: &Path,
    relative: &Path,
    maximum: u64,
) -> Result<Option<Vec<u8>>, UpdateError> {
    validate_update_relative(relative)?;
    ensure_update_no_symlink_ancestors(target, relative)?;
    let absolute = target.join(relative);
    let metadata = match fs::symlink_metadata(&absolute) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
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
    let mut file = fs::File::open(&absolute).map_err(|error| {
        UpdateError::Internal(format!("cannot open {}: {error}", relative.display()))
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

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpdateError::Internal("backup file has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| UpdateError::Internal(format!("cannot create backup parent: {error}")))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| UpdateError::Conflict(format!("cannot create backup file: {error}")))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist backup file: {error}")))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpdateError::Internal("atomic file has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| UpdateError::Internal(format!("cannot create atomic parent: {error}")))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| UpdateError::Internal(format!("cannot create atomic temp: {error}")))?;
    temporary
        .write_all(bytes)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| UpdateError::Internal(format!("cannot persist atomic temp: {error}")))?;
    let persisted = temporary.persist(path).map_err(|error| {
        UpdateError::Internal(format!(
            "cannot atomically replace {}: {}",
            path.display(),
            error.error
        ))
    })?;
    persisted.sync_all().map_err(|error| {
        UpdateError::Internal(format!(
            "cannot sync atomic replacement {}: {error}",
            path.display()
        ))
    })?;
    sync_parent_directory(parent)?;
    Ok(())
}

fn write_atomic_relative(target: &Path, relative: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    validate_update_relative(relative)?;
    ensure_update_no_symlink_ancestors(target, relative)?;
    write_atomic(&target.join(relative), bytes)
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> Result<(), UpdateError> {
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            UpdateError::Internal(format!(
                "cannot sync atomic parent {}: {error}",
                parent.display()
            ))
        })
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> Result<(), UpdateError> {
    Ok(())
}

fn remove_exact_regular_file(target: &Path, relative: &Path) -> Result<(), UpdateError> {
    validate_update_relative(relative)?;
    ensure_update_no_symlink_ancestors(target, relative)?;
    let absolute = target.join(relative);
    match fs::symlink_metadata(&absolute) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            fs::remove_file(&absolute).map_err(|error| {
                UpdateError::Rollback(format!("cannot remove {}: {error}", relative.display()))
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
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

fn validate_update_relative(relative: &Path) -> Result<(), UpdateError> {
    if is_hive_skill_projection_path(relative) {
        validate_hive_skill_projection_relative(relative)
            .map_err(|error| UpdateError::Input(error.to_string()))
    } else {
        validate_project_relative(relative).map_err(|error| UpdateError::Input(error.to_string()))
    }
}

fn ensure_update_no_symlink_ancestors(target: &Path, relative: &Path) -> Result<(), UpdateError> {
    if is_hive_skill_projection_path(relative) {
        ensure_no_symlink_ancestors_for_hive_skill_projection(target, relative)
            .map_err(|error| UpdateError::Conflict(error.to_string()))
    } else {
        ensure_no_symlink_ancestors(target, relative)
            .map_err(|error| UpdateError::Conflict(error.to_string()))
    }
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
            .join("../../tests/fixtures/phase1")
            .join(name)
    }

    fn release_fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0")
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
        })
        .expect("current setup");

        let harness_path = target.join(HARNESS_PATH);
        let harness = fs::read_to_string(&harness_path)
            .expect("harness")
            .replace("0.7.0", version);
        fs::write(harness_path, harness).expect("legacy harness");

        let active_path = target.join(".hive/config/active-skills.yml");
        let expected = legacy_builtin_names(version);
        let skill_root = target.join(".agents/skills");
        for entry in fs::read_dir(&skill_root).expect("projected Skills") {
            let entry = entry.expect("projected Skill entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            if !expected.contains(&name.as_str()) {
                fs::remove_dir_all(entry.path()).expect("remove future projection");
            }
        }
        if expected.is_empty() {
            fs::remove_file(active_path).expect("remove future active ledger");
        } else {
            let historical_setup = include_bytes!(
                "../../../tests/fixtures/phase6/migrations/0.4.0-0.6.0-setup-harness.SKILL.md"
            );
            fs::write(skill_root.join("setup-harness/SKILL.md"), historical_setup)
                .expect("historical setup projection");
            if version == "0.5.0" {
                let historical = include_bytes!(
                    "../../../tests/fixtures/phase6/migrations/0.5.0-hive-run-resume.SKILL.md"
                );
                fs::write(skill_root.join("hive-run-resume/SKILL.md"), historical)
                    .expect("historical resume projection");
            }
            let mut active: serde_yaml::Value =
                serde_yaml::from_slice(&fs::read(&active_path).expect("active skills"))
                    .expect("active skills");
            let skills = active
                .as_mapping_mut()
                .and_then(|mapping| mapping.get_mut(serde_yaml::Value::from("skills")))
                .and_then(serde_yaml::Value::as_sequence_mut)
                .expect("skills");
            skills.retain(|entry| {
                entry
                    .as_mapping()
                    .and_then(|mapping| mapping.get(serde_yaml::Value::from("name")))
                    .and_then(serde_yaml::Value::as_str)
                    .is_some_and(|name| expected.contains(&name))
            });
            let setup = skills
                .iter_mut()
                .find(|entry| {
                    entry
                        .as_mapping()
                        .and_then(|mapping| mapping.get(serde_yaml::Value::from("name")))
                        .and_then(serde_yaml::Value::as_str)
                        == Some("setup-harness")
                })
                .and_then(serde_yaml::Value::as_mapping_mut)
                .expect("setup entry");
            setup.insert(
                serde_yaml::Value::from("content_digest"),
                serde_yaml::Value::from(sha256_digest(historical_setup)),
            );
            if version == "0.5.0" {
                let resume = skills
                    .iter_mut()
                    .find(|entry| {
                        entry
                            .as_mapping()
                            .and_then(|mapping| mapping.get(serde_yaml::Value::from("name")))
                            .and_then(serde_yaml::Value::as_str)
                            == Some("hive-run-resume")
                    })
                    .and_then(serde_yaml::Value::as_mapping_mut)
                    .expect("resume entry");
                resume.insert(
                    serde_yaml::Value::from("content_digest"),
                    serde_yaml::Value::from(
                        "sha256:edfbee35142b8a228d4cdb36d2674b719548fea9884d4a2b6a31353adcebb7c5",
                    ),
                );
            }
            fs::write(
                active_path,
                serde_yaml::to_string(&active).expect("active skills"),
            )
            .expect("legacy active skills");
        }
        fs::write(target.join("README.md"), b"foreign project bytes\n").expect("foreign readme");
        fs::create_dir_all(target.join(".omx")).expect("omx");
        fs::write(target.join(".omx/state.json"), b"{\"foreign\":true}\n")
            .expect("foreign orchestration");
    }

    fn update_request<'a>(
        target: &'a Path,
        fixture: &'a Path,
        root: &'a [u8],
        mode: UpdateMode,
    ) -> UpdateRequest<'a> {
        UpdateRequest {
            target,
            repository: fixture,
            trusted_root_bytes: root,
            now_unix: 1_800_000_000,
            mode,
            exact_major_target: None,
            major_approval: None,
        }
    }

    fn rollback_state() -> RollbackState {
        RollbackState {
            root_version: 1,
            timestamp_version: 1,
            snapshot_version: 1,
            targets_version: 1,
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
                rollback: rollback_state(),
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
        let fixture = release_fixture();
        let root = fs::read(fixture.join("metadata/root.json")).expect("root");
        let verified =
            verify_release_repository(&root, &fixture, 1_800_000_000, None).expect("release");
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
        let manifest = create_backup(
            target.path(),
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
            prune_expired_backups(target.path(), 1_000 + crate::BACKUP_RETENTION_SECONDS,),
            0
        );
        assert_eq!(
            prune_expired_backups(target.path(), 1_001 + crate::BACKUP_RETENTION_SECONDS,),
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
        let root = fs::read(fixture.join("metadata/root.json")).expect("root");
        for source_version in ["0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0"] {
            let target = tempfile::tempdir().expect("target");
            let consumer = target.path().canonicalize().expect("consumer");
            install_legacy_consumer(&consumer, source_version);
            let harness_path = consumer.join(HARNESS_PATH);
            let harness = fs::read_to_string(&harness_path).expect("harness").replace(
                "usage_stop_remaining_percent = 20",
                "usage_stop_remaining_percent = 37",
            );
            fs::write(&harness_path, harness).expect("configured threshold");
            let before_readme = fs::read(consumer.join("README.md")).expect("readme");
            let before_omx = fs::read(consumer.join(".omx/state.json")).expect("omx");

            let dry_run = execute_update(&update_request(
                &consumer,
                &fixture,
                &root,
                UpdateMode::DryRun,
            ))
            .unwrap_or_else(|error| panic!("{source_version} dry-run failed: {error}"));
            assert_eq!(dry_run.source_version, source_version);
            assert_eq!(dry_run.target_version, "0.7.0");
            assert_eq!(dry_run.migration_id, "same-major-render-v1");

            let applied = execute_update(&update_request(
                &consumer,
                &fixture,
                &root,
                UpdateMode::Apply,
            ))
            .unwrap_or_else(|error| panic!("{source_version} apply failed: {error}"));
            assert_eq!(applied.source_version, source_version);
            assert_eq!(applied.target_version, "0.7.0");
            assert_eq!(
                fs::read(consumer.join("README.md")).expect("readme"),
                before_readme
            );
            assert_eq!(
                fs::read(consumer.join(".omx/state.json")).expect("omx"),
                before_omx
            );
            let migrated = fs::read_to_string(consumer.join(HARNESS_PATH)).expect("harness");
            assert!(migrated.contains("0.7.0"));
            assert!(
                migrated.contains("usage_stop_remaining_percent = 37"),
                "{migrated}"
            );
        }
    }

    #[test]
    fn signed_update_dry_run_and_apply_preserve_foreign_bytes_and_rebuild_index() {
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_legacy_consumer(&consumer, "0.6.0");
        let fixture = release_fixture();
        let root = fs::read(fixture.join("metadata/root.json")).expect("root");
        let before_readme = fs::read(consumer.join("README.md")).expect("readme");
        let before_omx = fs::read(consumer.join(".omx/state.json")).expect("omx");

        let dry_run = execute_update(&update_request(
            &consumer,
            &fixture,
            &root,
            UpdateMode::DryRun,
        ))
        .expect("dry-run");
        assert_eq!(dry_run.source_version, "0.6.0");
        assert_eq!(dry_run.target_version, "0.7.0");
        assert_eq!(dry_run.migration_id, "same-major-render-v1");
        assert!(dry_run.backup_id.is_none());
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(fs::read_to_string(consumer.join(HARNESS_PATH))
            .expect("harness")
            .contains("0.6.0"));

        let applied = execute_update(&update_request(
            &consumer,
            &fixture,
            &root,
            UpdateMode::Apply,
        ))
        .expect("apply");
        assert_eq!(applied.migration_id, "same-major-render-v1");
        assert!(applied.backup_id.is_some());
        assert!(applied.index_digest.is_some());
        assert!(consumer.join(UPDATE_STATE_PATH).is_file());
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(consumer.join(".hive/index/hive.sqlite3").is_file());
        for skill in ["hive-update", "hive-migrate"] {
            assert!(consumer
                .join(format!(".agents/skills/{skill}/SKILL.md"))
                .is_file());
        }
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
    fn forged_legacy_skill_and_matching_writable_ledger_are_rejected() {
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_legacy_consumer(&consumer, "0.6.0");
        let skill_path = consumer.join(".agents/skills/hive-run-resume/SKILL.md");
        let forged = b"---\nname: hive-run-resume\ndescription: attacker bytes\n---\n";
        fs::write(&skill_path, forged).expect("forged projected Skill");

        let active_path = consumer.join(".hive/config/active-skills.yml");
        let mut active: serde_yaml::Value =
            serde_yaml::from_slice(&fs::read(&active_path).expect("active skills"))
                .expect("active skills");
        let skills = active
            .as_mapping_mut()
            .and_then(|mapping| mapping.get_mut(serde_yaml::Value::from("skills")))
            .and_then(serde_yaml::Value::as_sequence_mut)
            .expect("skills");
        let resume = skills
            .iter_mut()
            .find(|entry| {
                entry
                    .as_mapping()
                    .and_then(|mapping| mapping.get(serde_yaml::Value::from("name")))
                    .and_then(serde_yaml::Value::as_str)
                    == Some("hive-run-resume")
            })
            .and_then(serde_yaml::Value::as_mapping_mut)
            .expect("resume entry");
        resume.insert(
            serde_yaml::Value::from("content_digest"),
            serde_yaml::Value::from(sha256_digest(forged)),
        );
        fs::write(
            &active_path,
            serde_yaml::to_string(&active).expect("active skills"),
        )
        .expect("forged active ledger");

        let fixture = release_fixture();
        let root = fs::read(fixture.join("metadata/root.json")).expect("root");
        assert!(matches!(
            execute_update(&update_request(
                &consumer,
                &fixture,
                &root,
                UpdateMode::DryRun,
            )),
            Err(UpdateError::Conflict(message))
                if message.contains("authenticated release history")
        ));
        assert_eq!(
            fs::read(skill_path).expect("forged bytes preserved"),
            forged
        );
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
        assert!(!consumer.join(JOURNAL_PATH).exists());
    }

    #[test]
    fn injected_activation_failure_keeps_legacy_generation_and_recovers_exactly() {
        let _environment = UPDATE_ENVIRONMENT.lock().expect("environment lock");
        let target = tempfile::tempdir().expect("target");
        let consumer = target.path().canonicalize().expect("consumer");
        install_legacy_consumer(&consumer, "0.6.0");
        let fixture = release_fixture();
        let root = fs::read(fixture.join("metadata/root.json")).expect("root");
        let before_readme = fs::read(consumer.join("README.md")).expect("readme");
        std::env::set_var("HIVE_TEST_ACTIVATION_FAIL_AFTER", "1");
        let result = execute_update(&update_request(
            &consumer,
            &fixture,
            &root,
            UpdateMode::Apply,
        ));
        std::env::remove_var("HIVE_TEST_ACTIVATION_FAIL_AFTER");

        assert!(result.is_err());
        assert!(consumer.join(JOURNAL_PATH).is_file());
        assert!(fs::read_to_string(consumer.join(HARNESS_PATH))
            .expect("harness")
            .contains("0.6.0"));
        assert_eq!(
            fs::read(consumer.join("README.md")).expect("readme"),
            before_readme
        );
        recover_update(&consumer).expect("recovery");
        assert!(!consumer.join(JOURNAL_PATH).exists());
        assert!(fs::read_to_string(consumer.join(HARNESS_PATH))
            .expect("harness")
            .contains("0.6.0"));
        assert!(!consumer.join(UPDATE_STATE_PATH).exists());
    }
}
