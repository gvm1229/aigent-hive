//! Local release-integrity verification, compatibility, migration, and update safety.
//!
//! Distribution authenticity stays with npm registry integrity or GitHub artifact attestations.
//! This crate contains no network client, signing API, model runtime, provider SDK, or
//! package-manager execution.

mod backup;
mod merge;
mod migration;
mod release;
mod transaction;
mod version;

pub use backup::{
    backup_path_is_allowed, backup_storage_path, backups_to_prune, BackupEntry, BackupManifest,
    RetentionDecision, BACKUP_RETENTION_SECONDS,
};
pub use merge::{three_way_merge, three_way_merge_hive_directive, MergeDisposition, MergeOutcome};
pub use migration::{
    select_migration_route, validate_cross_major_preservation, validate_migration_table,
    MigrationKind, MigrationRoute, MigrationTable, PreservationDigest,
};
pub use release::{
    observe_surface_delta, verify_release_bundle, ReleaseArtifact, ReleaseManifest, ReleaseState,
    ReleaseVerification, SurfaceInventory, VerifiedTarget,
};
pub use transaction::{
    execute_update, execute_update_in, recover_update, recover_update_in, UpdateMode,
    UpdateOutcome, UpdateRequest, UpdateState,
};
pub use version::{
    classify_release, MajorApproval, ReleaseClass, ReleasePolicyError, SemVersion, SurfaceDelta,
    VersionTransition,
};

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Stable updater failure class used by CLI adapters.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UpdateError {
    /// Malformed input or unsupported wire data.
    Input(String),
    /// Release authentication, digest, expiry, or provenance verification failed.
    Verification(String),
    /// Version, rollback, compatibility, or release policy blocked the update.
    Compatibility(String),
    /// No compiled migration route can safely handle the transition.
    Unsupported(String),
    /// Live bytes changed after planning.
    Conflict(String),
    /// An incomplete durable journal requires an explicit recovery command.
    RecoveryRequired(String),
    /// A local updater operation failed unexpectedly.
    Internal(String),
    /// Recovery could not restore or safely complete a transaction.
    Rollback(String),
}

impl UpdateError {
    /// Stable process exit class.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Input(_) => 2,
            Self::Compatibility(_) | Self::Conflict(_) | Self::RecoveryRequired(_) => 3,
            Self::Unsupported(_) => 4,
            Self::Verification(_) => 5,
            Self::Internal(_) | Self::Rollback(_) => 10,
        }
    }

    /// Stable product code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Input(_) => "hive.update-invalid-input",
            Self::Verification(_) => "hive.update-release-verification-failed",
            Self::Compatibility(_) => "hive.update-compatibility-blocked",
            Self::Unsupported(_) => "hive.update-migration-unsupported",
            Self::Conflict(_) => "hive.update-conflict",
            Self::RecoveryRequired(_) => "hive.update-recovery-required",
            Self::Internal(_) => "hive.internal-error",
            Self::Rollback(_) => "hive.update-rollback-failed",
        }
    }

    /// Stable action status.
    #[must_use]
    pub const fn status(&self) -> &'static str {
        match self {
            Self::Input(_) | Self::Internal(_) | Self::Rollback(_) => "error",
            Self::Verification(_) => "verification-failed",
            Self::Compatibility(_) | Self::RecoveryRequired(_) => "blocked",
            Self::Unsupported(_) => "unsupported",
            Self::Conflict(_) => "conflict",
        }
    }
}

impl Display for UpdateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message)
            | Self::Verification(message)
            | Self::Compatibility(message)
            | Self::Unsupported(message)
            | Self::Conflict(message)
            | Self::RecoveryRequired(message)
            | Self::Internal(message)
            | Self::Rollback(message) => formatter.write_str(message),
        }
    }
}

impl Error for UpdateError {}
