use crate::{MigrationTable, ReleaseClass, SemVersion, SurfaceDelta, UpdateError};
use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use ed25519_dalek::{Signature, VerifyingKey};
use hive_core::{sha256_digest, validate_project_relative};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
#[cfg(test)]
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const MAX_METADATA_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TARGET_BYTES: u64 = 512 * 1024 * 1024;
const TUF_SPEC_VERSION: &str = "1.0.31";
const SOURCE_REPOSITORY: &str = "https://github.com/gvm1229/aigent-hive";
const PROVENANCE_STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
const PROVENANCE_PREDICATE_TYPE: &str = "https://slsa.dev/provenance/v1";
const PROVENANCE_BUILD_TYPE: &str = "https://github.com/gvm1229/aigent-hive/release/v1";
const PROVENANCE_BUILDER_PREFIX: &str =
    "https://github.com/gvm1229/aigent-hive/.github/workflows/release.yml@";
const REQUIRED_RELEASE_PAYLOADS: &[&str] = &[
    "targets/bundle-manifest.json",
    "targets/migration-table.json",
    "targets/platform-signing-evidence.json",
    "targets/provenance.intoto.json",
    "targets/release-surface-inventory.json",
];

struct PinnedReleaseRepository {
    requested: PathBuf,
    dir: Dir,
}

impl PinnedReleaseRepository {
    fn open(path: &Path) -> Result<Self, UpdateError> {
        let requested = absolute_lexical(path)?;
        let dir = open_directory_nofollow_path(&requested)?;
        Ok(Self { requested, dir })
    }

    fn read_bounded(&self, relative: &Path, maximum: u64) -> Result<Vec<u8>, UpdateError> {
        validate_project_relative(relative)
            .map_err(|error| UpdateError::Verification(error.to_string()))?;
        let (parent, name) = repository_parent(&self.dir, relative)?;
        let path_metadata = parent.symlink_metadata(&name).map_err(|error| {
            UpdateError::Verification(format!(
                "cannot inspect release file {}: {error}",
                relative.display()
            ))
        })?;
        if !path_metadata.is_file() || path_metadata.len() > maximum {
            return Err(UpdateError::Verification(format!(
                "release file is not a bounded no-follow regular file: {}",
                relative.display()
            )));
        }
        let expected_identity = FileIdentity::from_metadata(&path_metadata);
        let expected_modified = path_metadata.modified().map_err(|error| {
            UpdateError::Verification(format!(
                "cannot inspect release file timestamp {}: {error}",
                relative.display()
            ))
        })?;
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        let mut file = parent.open_with(&name, &options).map_err(|error| {
            UpdateError::Verification(format!(
                "cannot open release file no-follow {}: {error}",
                relative.display()
            ))
        })?;
        let opened_metadata = file.metadata().map_err(|error| {
            UpdateError::Verification(format!(
                "cannot inspect opened release file {}: {error}",
                relative.display()
            ))
        })?;
        if !opened_metadata.is_file()
            || opened_metadata.len() > maximum
            || !expected_identity.matches(&opened_metadata)
        {
            return Err(UpdateError::Verification(format!(
                "release file changed between inspection and open: {}",
                relative.display()
            )));
        }
        let mut bytes = Vec::with_capacity(
            usize::try_from(opened_metadata.len())
                .map_err(|_| UpdateError::Verification("release file is too large".to_owned()))?,
        );
        file.by_ref()
            .take(maximum.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| {
                UpdateError::Verification(format!(
                    "cannot read release file {}: {error}",
                    relative.display()
                ))
            })?;
        let final_metadata = file.metadata().map_err(|error| {
            UpdateError::Verification(format!(
                "cannot re-inspect opened release file {}: {error}",
                relative.display()
            ))
        })?;
        if !final_metadata.is_file()
            || !expected_identity.matches(&final_metadata)
            || final_metadata.len() != opened_metadata.len()
            || final_metadata.modified().ok().as_ref() != Some(&expected_modified)
            || bytes.len() as u64 != opened_metadata.len()
            || bytes.len() as u64 > maximum
        {
            return Err(UpdateError::Verification(format!(
                "release file identity or bytes changed during read: {}",
                relative.display()
            )));
        }
        Ok(bytes)
    }

    fn verify_current(&self) -> Result<(), UpdateError> {
        let current = open_directory_nofollow_path(&self.requested)?;
        let expected = self.dir.dir_metadata().map_err(|error| {
            UpdateError::Verification(format!("cannot inspect pinned release root: {error}"))
        })?;
        let actual = current.dir_metadata().map_err(|error| {
            UpdateError::Verification(format!("cannot re-open release root: {error}"))
        })?;
        if FileIdentity::from_metadata(&expected) != FileIdentity::from_metadata(&actual) {
            return Err(UpdateError::Verification(
                "release repository root changed during verification".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl FileIdentity {
    fn from_metadata(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            device: CapMetadataExt::dev(metadata),
            inode: CapMetadataExt::ino(metadata),
        }
    }

    fn matches(self, metadata: &cap_std::fs::Metadata) -> bool {
        self == Self::from_metadata(metadata)
    }
}

fn absolute_lexical(path: &Path) -> Result<PathBuf, UpdateError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| {
                UpdateError::Verification(format!(
                    "cannot resolve release repository path: {error}"
                ))
            })?
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(name) => normalized.push(name),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(UpdateError::Verification(format!(
                        "release repository path escapes its filesystem root: {}",
                        path.display()
                    )));
                }
            }
        }
    }
    Ok(normalized)
}

fn open_directory_nofollow_path(path: &Path) -> Result<Dir, UpdateError> {
    let mut root = PathBuf::new();
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => root.push(prefix.as_os_str()),
            Component::RootDir => root.push(component.as_os_str()),
            Component::Normal(name) => names.push(name.to_os_string()),
            Component::CurDir | Component::ParentDir => {
                return Err(UpdateError::Verification(format!(
                    "release directory path is not lexically safe: {}",
                    path.display()
                )));
            }
        }
    }
    if root.as_os_str().is_empty() {
        return Err(UpdateError::Verification(format!(
            "release directory path is not absolute: {}",
            path.display()
        )));
    }
    let mut current = Dir::open_ambient_dir(&root, ambient_authority()).map_err(|error| {
        UpdateError::Verification(format!(
            "cannot open release filesystem root {}: {error}",
            root.display()
        ))
    })?;
    let mut walked = root;
    for name in names {
        walked.push(&name);
        match current.symlink_metadata(&name) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(UpdateError::Verification(format!(
                    "release directory component is not a no-follow directory: {}",
                    walked.display()
                )));
            }
            Err(error) => {
                return Err(UpdateError::Verification(format!(
                    "cannot inspect release directory component {}: {error}",
                    walked.display()
                )));
            }
        }
        current = current.open_dir_nofollow(&name).map_err(|error| {
            UpdateError::Verification(format!(
                "cannot open release directory component no-follow {}: {error}",
                walked.display()
            ))
        })?;
    }
    Ok(current)
}

fn repository_parent(root: &Dir, relative: &Path) -> Result<(Dir, OsString), UpdateError> {
    let name = relative
        .file_name()
        .ok_or_else(|| UpdateError::Verification("release file has no name".to_owned()))?
        .to_os_string();
    let mut current = root.try_clone().map_err(|error| {
        UpdateError::Verification(format!("cannot clone release root handle: {error}"))
    })?;
    let mut walked = PathBuf::new();
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let Component::Normal(component) = component else {
                return Err(UpdateError::Verification(format!(
                    "release path is not project-relative: {}",
                    relative.display()
                )));
            };
            walked.push(component);
            match current.symlink_metadata(component) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(UpdateError::Verification(format!(
                        "release path ancestor is not a no-follow directory: {}",
                        walked.display()
                    )));
                }
                Err(error) => {
                    return Err(UpdateError::Verification(format!(
                        "cannot inspect release path ancestor {}: {error}",
                        walked.display()
                    )));
                }
            }
            current = current.open_dir_nofollow(component).map_err(|error| {
                UpdateError::Verification(format!(
                    "cannot open release path ancestor no-follow {}: {error}",
                    walked.display()
                ))
            })?;
        }
    }
    Ok((current, name))
}

/// TUF signed envelope.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEnvelope<T> {
    /// Detached threshold signatures over RFC 8785/JCS `signed`.
    pub signatures: Vec<TufSignature>,
    /// Role payload.
    pub signed: T,
}

/// TUF signature entry.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TufSignature {
    /// Authorized key id.
    pub keyid: String,
    /// Raw 64-byte Ed25519 signature.
    pub sig: String,
}

/// Trusted root envelope.
pub type TufRoot = SignedEnvelope<RootSigned>;

/// Root role metadata.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootSigned {
    /// Exact metadata role.
    #[serde(rename = "_type")]
    pub metadata_type: String,
    /// Supported TUF specification generation.
    pub spec_version: String,
    /// Monotonic metadata version.
    pub version: u64,
    /// UTC expiry.
    pub expires: String,
    /// Consistent snapshot policy.
    pub consistent_snapshot: bool,
    /// Public verification keys.
    pub keys: BTreeMap<String, TufKey>,
    /// Threshold role delegations.
    pub roles: BTreeMap<String, TufRole>,
}

/// TUF public key.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TufKey {
    /// Must be `ed25519`.
    pub keytype: String,
    /// Must be `ed25519`.
    pub scheme: String,
    /// Public key value.
    pub keyval: TufKeyValue,
}

/// TUF public key value.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TufKeyValue {
    /// Lowercase raw public-key hex with `ed25519:` prefix.
    pub public: String,
}

/// TUF threshold role.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TufRole {
    /// Authorized key ids.
    pub keyids: Vec<String>,
    /// Required distinct valid signatures.
    pub threshold: u32,
}

/// Targets envelope.
pub type TufTargets = SignedEnvelope<TargetsSigned>;

/// Targets role payload.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetsSigned {
    /// Exact role.
    #[serde(rename = "_type")]
    pub metadata_type: String,
    /// Supported TUF specification.
    pub spec_version: String,
    /// Monotonic metadata version.
    pub version: u64,
    /// UTC expiry.
    pub expires: String,
    /// Every allowed bundle target.
    pub targets: BTreeMap<String, TargetDescription>,
}

/// Snapshot envelope.
pub type TufSnapshot = SignedEnvelope<SnapshotSigned>;

/// Snapshot role payload.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSigned {
    /// Exact role.
    #[serde(rename = "_type")]
    pub metadata_type: String,
    /// Supported TUF specification.
    pub spec_version: String,
    /// Monotonic metadata version.
    pub version: u64,
    /// UTC expiry.
    pub expires: String,
    /// Version/hash/length of targets metadata.
    pub meta: BTreeMap<String, MetadataDescription>,
}

/// Timestamp envelope.
pub type TufTimestamp = SignedEnvelope<TimestampSigned>;

/// Timestamp role payload.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampSigned {
    /// Exact role.
    #[serde(rename = "_type")]
    pub metadata_type: String,
    /// Supported TUF specification.
    pub spec_version: String,
    /// Monotonic metadata version.
    pub version: u64,
    /// UTC expiry.
    pub expires: String,
    /// Version/hash/length of snapshot metadata.
    pub meta: BTreeMap<String, MetadataDescription>,
}

/// Hash and size description.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataDescription {
    /// Expected byte length.
    pub length: u64,
    /// Hash map; SHA-256 is mandatory.
    pub hashes: BTreeMap<String, String>,
    /// Expected signed metadata version.
    pub version: u64,
}

/// Target hash and size.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetDescription {
    /// Expected byte length.
    pub length: u64,
    /// Hash map; SHA-256 is mandatory.
    pub hashes: BTreeMap<String, String>,
}

/// Closed immutable release bundle manifest.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    /// Schema generation.
    pub schema_version: u32,
    /// Exact product name.
    pub product: String,
    /// Exact product version.
    pub release_version: String,
    /// Monotonic release sequence.
    pub release_sequence: u64,
    /// Declared version class.
    pub classification: ReleaseClass,
    /// Source identity.
    pub source: ReleaseSource,
    /// Minimum updater product version.
    pub minimum_updater_version: String,
    /// Oldest supported installed harness.
    pub minimum_supported_harness_version: String,
    /// Product license.
    pub license: String,
    /// Release surface inventory digest.
    pub surface_inventory_digest: String,
    /// Migration table digest.
    pub migration_table_digest: String,
    /// Provenance statement digest.
    pub provenance_digest: String,
    /// Platform signing evidence digest.
    pub platform_signing_evidence_digest: String,
}

/// Release source identity.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSource {
    /// Canonical source repository.
    pub repository: String,
    /// Exact lowercase Git commit.
    pub commit: String,
    /// Exact release tag.
    pub tag: String,
}

/// Deterministic shipped-surface inventory.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceInventory {
    /// Schema generation.
    pub schema_version: u32,
    /// Exact release version.
    pub product_version: String,
    /// Sorted schemas.
    pub schemas: Vec<String>,
    /// Sorted implemented Skills.
    pub skills: Vec<String>,
    /// Sorted capability ids.
    pub capabilities: Vec<String>,
    /// Sorted migration ids.
    pub migrations: Vec<String>,
    /// Sorted template paths.
    pub templates: Vec<String>,
    /// Sorted ownership declarations.
    pub ownership: Vec<String>,
    /// Sorted host projection paths.
    pub projections: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalSurfaceRegistry {
    schema_version: u32,
    releases: Vec<SurfaceInventory>,
}

/// Public result recorded by the protected platform-signing workflow.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSigningEvidence {
    /// Schema generation.
    pub schema_version: u32,
    /// Per-artifact public verification results.
    pub evidence: Vec<PlatformSigningEvidenceEntry>,
}

/// One platform signature verification result.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSigningEvidenceEntry {
    /// `macos` or `windows`.
    pub platform: String,
    /// Release-target-relative artifact path.
    pub artifact_path: String,
    /// Exact artifact digest.
    pub artifact_digest: String,
    /// `developer-id` or `authenticode`.
    pub scheme: String,
    /// Public signer identity authorized by the release metadata.
    pub signer: PlatformSignerIdentity,
    /// Protected verifier result.
    pub status: String,
}

/// Platform-specific public signer identity.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSignerIdentity {
    /// `apple-team-id` or `authenticode-certificate-thumbprint`.
    pub kind: String,
    /// Exact public identity emitted by the protected platform verifier.
    pub value: String,
}

/// Closed in-toto statement subset accepted by Hive.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceStatement {
    /// Exact in-toto statement type.
    #[serde(rename = "_type")]
    pub statement_type: String,
    /// Subject artifacts.
    pub subject: Vec<ProvenanceSubject>,
    /// Exact SLSA provenance predicate type.
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    /// Source and builder bindings.
    pub predicate: ProvenancePredicate,
}

/// One provenance subject.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceSubject {
    /// Artifact name.
    pub name: String,
    /// Exact digest map.
    pub digest: BTreeMap<String, String>,
}

/// SLSA provenance predicate subset.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenancePredicate {
    /// Immutable source inputs.
    #[serde(rename = "buildDefinition")]
    pub build_definition: ProvenanceBuildDefinition,
    /// Protected workflow identity and timing.
    #[serde(rename = "runDetails")]
    pub run_details: ProvenanceRunDetails,
}

/// Provenance build definition.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceBuildDefinition {
    /// Exact Hive release build type.
    #[serde(rename = "buildType")]
    pub build_type: String,
    /// Caller-visible build parameters.
    #[serde(rename = "externalParameters")]
    pub external_parameters: serde_json::Value,
    /// Locked-build marker.
    #[serde(rename = "internalParameters")]
    pub internal_parameters: ProvenanceInternalParameters,
    /// Source dependency identities.
    #[serde(rename = "resolvedDependencies")]
    pub resolved_dependencies: Vec<ProvenanceResolvedDependency>,
}

/// Locked internal build parameters.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceInternalParameters {
    /// Must be true for a publishable candidate.
    pub locked: bool,
}

/// One resolved source dependency.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceResolvedDependency {
    /// Canonical source URI pinned to the commit.
    pub uri: String,
    /// Exact Git commit digest map.
    pub digest: BTreeMap<String, String>,
}

/// Protected workflow run details.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceRunDetails {
    /// GitHub workflow builder identity.
    pub builder: ProvenanceBuilder,
    /// Invocation identity and bounded timestamps.
    pub metadata: ProvenanceRunMetadata,
}

/// Provenance builder identity.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceBuilder {
    /// Exact workflow identity including its ref.
    pub id: String,
}

/// Provenance invocation metadata.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceRunMetadata {
    /// Non-empty protected workflow invocation id.
    #[serde(rename = "invocationId")]
    pub invocation_id: String,
    /// Exact UTC start.
    #[serde(rename = "startedOn")]
    pub started_on: String,
    /// Exact UTC finish.
    #[serde(rename = "finishedOn")]
    pub finished_on: String,
}

/// Highest previously accepted metadata, preventing rollback/substitution.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollbackState {
    /// Trusted root version.
    pub root_version: u64,
    /// Timestamp version.
    pub timestamp_version: u64,
    /// Snapshot version.
    pub snapshot_version: u64,
    /// Targets version.
    pub targets_version: u64,
    /// Highest release sequence.
    pub release_sequence: u64,
    /// Last accepted manifest digest.
    pub manifest_digest: String,
}

/// Verified bundle target.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VerifiedTarget {
    /// Normalized target path.
    pub path: String,
    /// Verified SHA-256 digest.
    pub digest: String,
    /// Verified length.
    pub length: u64,
}

/// Complete offline release verification result.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReleaseVerification {
    /// Parsed release manifest.
    pub manifest: ReleaseManifest,
    /// Parsed migration table.
    pub migration_table: MigrationTable,
    /// Parsed public surface inventory.
    pub surface_inventory: SurfaceInventory,
    /// Parsed source/build provenance.
    pub provenance: ProvenanceStatement,
    /// Parsed platform signature verification evidence.
    pub platform_signing_evidence: PlatformSigningEvidence,
    /// Verified bundle target list.
    pub targets: Vec<VerifiedTarget>,
    /// Digest of the release manifest target.
    pub manifest_digest: String,
    /// New rollback state to persist only after activation commits.
    pub next_rollback_state: RollbackState,
}

struct VerifiedMetadataChain {
    root: TufRoot,
    timestamp: TufTimestamp,
    snapshot: TufSnapshot,
    targets: TufTargets,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum EvidenceRequirement {
    Integrity,
    Production,
}

/// Verify a complete offline TUF-compatible release repository.
///
/// The trusted root must come from an independently protected caller. The
/// repository itself may be attacker-controlled; all metadata and targets are
/// verified before the result is returned.
///
/// # Errors
///
/// Returns an error when metadata, signatures, expiry, rollback state, target
/// bytes, or release payload contracts fail verification.
pub fn verify_release_repository(
    trusted_root_bytes: &[u8],
    repository_root: &Path,
    now_unix: i64,
    previous: Option<&RollbackState>,
) -> Result<ReleaseVerification, UpdateError> {
    verify_release_repository_with_requirement(
        trusted_root_bytes,
        repository_root,
        now_unix,
        previous,
        EvidenceRequirement::Integrity,
    )
}

/// Verify a release for public publication.
///
/// This performs the complete TUF and provenance verification and additionally
/// requires every platform-signing evidence entry to carry a protected
/// `verified` result. Public test-fixture evidence is rejected.
///
/// # Errors
///
/// Returns an error for every release-verification failure, including fixture
/// or incomplete external platform evidence.
pub fn verify_release_repository_for_publication(
    trusted_root_bytes: &[u8],
    repository_root: &Path,
    now_unix: i64,
    previous: Option<&RollbackState>,
) -> Result<ReleaseVerification, UpdateError> {
    verify_release_repository_with_requirement(
        trusted_root_bytes,
        repository_root,
        now_unix,
        previous,
        EvidenceRequirement::Production,
    )
}

fn verify_release_repository_with_requirement(
    trusted_root_bytes: &[u8],
    repository_root: &Path,
    now_unix: i64,
    previous: Option<&RollbackState>,
    evidence_requirement: EvidenceRequirement,
) -> Result<ReleaseVerification, UpdateError> {
    let repository = PinnedReleaseRepository::open(repository_root)?;
    let chain = verify_metadata_chain(trusted_root_bytes, &repository, now_unix, previous)?;
    let verified_targets = verify_all_targets(&repository, &chain.targets.signed.targets)?;
    let verified = verify_release_payloads(
        &repository,
        verified_targets,
        previous,
        &chain,
        evidence_requirement,
    )?;
    repository.verify_current()?;
    Ok(verified)
}

/// Verify a consecutive TUF root rotation under both old and new thresholds.
///
/// Hive only verifies caller-supplied public metadata. It does not generate,
/// import, or persist any private signing key.
///
/// # Errors
///
/// Returns an error when either threshold fails, the new root is not exactly
/// the next version, or the new root contract/expiry is invalid.
pub fn verify_root_rotation(
    trusted_root_bytes: &[u8],
    candidate_root_bytes: &[u8],
    now_unix: i64,
) -> Result<TufRoot, UpdateError> {
    let trusted: TufRoot = parse_json(trusted_root_bytes, "trusted root")?;
    validate_root(&trusted, now_unix, None)?;
    let candidate: TufRoot = parse_json(candidate_root_bytes, "candidate root")?;
    let expected_version =
        trusted.signed.version.checked_add(1).ok_or_else(|| {
            UpdateError::Verification("trusted root version overflows".to_owned())
        })?;
    if candidate.signed.version != expected_version {
        return Err(UpdateError::Compatibility(
            "candidate root must be the exact next metadata version".to_owned(),
        ));
    }
    verify_role(&trusted.signed, "root", &candidate)?;
    validate_root(&candidate, now_unix, None)?;
    Ok(candidate)
}

fn verify_metadata_chain(
    trusted_root_bytes: &[u8],
    repository: &PinnedReleaseRepository,
    now_unix: i64,
    previous: Option<&RollbackState>,
) -> Result<VerifiedMetadataChain, UpdateError> {
    let root: TufRoot = parse_json(trusted_root_bytes, "trusted root")?;
    validate_root(&root, now_unix, previous)?;
    let timestamp_bytes = read_bundle_file(
        repository,
        Path::new("metadata/timestamp.json"),
        MAX_METADATA_BYTES,
    )?;
    let timestamp: TufTimestamp = parse_json(&timestamp_bytes, "timestamp metadata")?;
    verify_role(&root.signed, "timestamp", &timestamp)?;
    validate_timestamp_metadata(&timestamp, now_unix)?;

    let snapshot_description = timestamp
        .signed
        .meta
        .get("snapshot.json")
        .ok_or_else(|| UpdateError::Verification("timestamp omits snapshot.json".to_owned()))?;
    let snapshot_bytes = read_bundle_file(
        repository,
        Path::new("metadata/snapshot.json"),
        MAX_METADATA_BYTES,
    )?;
    verify_description(&snapshot_bytes, snapshot_description, "snapshot metadata")?;
    let snapshot: TufSnapshot = parse_json(&snapshot_bytes, "snapshot metadata")?;
    if snapshot.signed.version != snapshot_description.version {
        return Err(UpdateError::Verification(
            "snapshot metadata version differs from timestamp".to_owned(),
        ));
    }
    verify_role(&root.signed, "snapshot", &snapshot)?;
    validate_snapshot_metadata(&snapshot, now_unix)?;

    let targets_description = snapshot
        .signed
        .meta
        .get("targets.json")
        .ok_or_else(|| UpdateError::Verification("snapshot omits targets.json".to_owned()))?;
    let targets_bytes = read_bundle_file(
        repository,
        Path::new("metadata/targets.json"),
        MAX_METADATA_BYTES,
    )?;
    verify_description(&targets_bytes, targets_description, "targets metadata")?;
    let targets: TufTargets = parse_json(&targets_bytes, "targets metadata")?;
    if targets.signed.version != targets_description.version {
        return Err(UpdateError::Verification(
            "targets metadata version differs from snapshot".to_owned(),
        ));
    }
    verify_role(&root.signed, "targets", &targets)?;
    validate_targets_metadata(&targets, now_unix)?;
    enforce_rollback(&root, &timestamp, &snapshot, &targets, previous)?;
    Ok(VerifiedMetadataChain {
        root,
        timestamp,
        snapshot,
        targets,
    })
}

fn validate_timestamp_metadata(timestamp: &TufTimestamp, now_unix: i64) -> Result<(), UpdateError> {
    validate_common(
        "timestamp",
        &timestamp.signed.metadata_type,
        &timestamp.signed.spec_version,
        &timestamp.signed.expires,
        now_unix,
    )?;
    if timestamp.signed.version == 0
        || timestamp
            .signed
            .meta
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != ["snapshot.json"]
    {
        return Err(UpdateError::Verification(
            "timestamp metadata contains an unsupported version or role".to_owned(),
        ));
    }
    Ok(())
}

fn validate_snapshot_metadata(snapshot: &TufSnapshot, now_unix: i64) -> Result<(), UpdateError> {
    validate_common(
        "snapshot",
        &snapshot.signed.metadata_type,
        &snapshot.signed.spec_version,
        &snapshot.signed.expires,
        now_unix,
    )?;
    if snapshot.signed.version == 0
        || snapshot
            .signed
            .meta
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != ["targets.json"]
    {
        return Err(UpdateError::Verification(
            "snapshot metadata contains an unsupported version or role".to_owned(),
        ));
    }
    Ok(())
}

fn validate_targets_metadata(targets: &TufTargets, now_unix: i64) -> Result<(), UpdateError> {
    validate_common(
        "targets",
        &targets.signed.metadata_type,
        &targets.signed.spec_version,
        &targets.signed.expires,
        now_unix,
    )?;
    if targets.signed.version == 0 || targets.signed.targets.is_empty() {
        return Err(UpdateError::Verification(
            "targets metadata version or target set is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn verify_all_targets(
    repository: &PinnedReleaseRepository,
    targets: &BTreeMap<String, TargetDescription>,
) -> Result<Vec<VerifiedTarget>, UpdateError> {
    let mut verified_targets = Vec::new();
    for (path, description) in targets {
        let normalized = validate_target_path(path)?;
        let bytes = read_bundle_file(repository, Path::new(&normalized), MAX_TARGET_BYTES)?;
        verify_target_description(&bytes, description, path)?;
        verified_targets.push(VerifiedTarget {
            path: normalized,
            digest: sha256_digest(&bytes),
            length: bytes.len() as u64,
        });
    }
    verified_targets.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(verified_targets)
}

fn verify_release_payloads(
    repository: &PinnedReleaseRepository,
    verified_targets: Vec<VerifiedTarget>,
    previous: Option<&RollbackState>,
    chain: &VerifiedMetadataChain,
    evidence_requirement: EvidenceRequirement,
) -> Result<ReleaseVerification, UpdateError> {
    let manifest_bytes = required_verified_target(
        repository,
        &verified_targets,
        "targets/bundle-manifest.json",
    )?;
    let manifest: ReleaseManifest = parse_json(&manifest_bytes, "release bundle manifest")?;
    validate_release_manifest(&manifest)?;
    let manifest_digest = sha256_digest(&manifest_bytes);
    validate_release_sequence(previous, &manifest, &manifest_digest)?;
    let migration_bytes = required_verified_target(
        repository,
        &verified_targets,
        "targets/migration-table.json",
    )?;
    require_digest(
        &migration_bytes,
        &manifest.migration_table_digest,
        "migration table",
    )?;
    let migration_table: MigrationTable = parse_json(&migration_bytes, "migration table")?;
    crate::validate_migration_table(&migration_table)?;
    if migration_table.target_version != manifest.release_version {
        return Err(UpdateError::Verification(
            "migration target differs from release manifest".to_owned(),
        ));
    }

    let inventory_bytes = required_verified_target(
        repository,
        &verified_targets,
        "targets/release-surface-inventory.json",
    )?;
    require_digest(
        &inventory_bytes,
        &manifest.surface_inventory_digest,
        "release surface inventory",
    )?;
    let surface_inventory: SurfaceInventory =
        parse_json(&inventory_bytes, "release surface inventory")?;
    validate_sorted_inventory(&surface_inventory)?;
    if surface_inventory.product_version != manifest.release_version {
        return Err(UpdateError::Verification(
            "surface inventory version differs from release manifest".to_owned(),
        ));
    }
    validate_inventory_migrations(&surface_inventory, &migration_table)?;

    let provenance_bytes = required_verified_target(
        repository,
        &verified_targets,
        "targets/provenance.intoto.json",
    )?;
    require_digest(&provenance_bytes, &manifest.provenance_digest, "provenance")?;
    let provenance: ProvenanceStatement = parse_json(&provenance_bytes, "provenance statement")?;
    validate_provenance(
        &provenance,
        &manifest,
        &verified_targets,
        evidence_requirement,
    )?;

    let platform_evidence_bytes = required_verified_target(
        repository,
        &verified_targets,
        "targets/platform-signing-evidence.json",
    )?;
    require_digest(
        &platform_evidence_bytes,
        &manifest.platform_signing_evidence_digest,
        "platform signing evidence",
    )?;
    let platform_signing_evidence: PlatformSigningEvidence =
        parse_json(&platform_evidence_bytes, "platform signing evidence")?;
    validate_platform_signing_evidence(
        &platform_signing_evidence,
        &verified_targets,
        evidence_requirement,
    )?;

    Ok(ReleaseVerification {
        manifest: manifest.clone(),
        migration_table,
        surface_inventory,
        provenance,
        platform_signing_evidence,
        targets: verified_targets,
        manifest_digest: manifest_digest.clone(),
        next_rollback_state: RollbackState {
            root_version: chain.root.signed.version,
            timestamp_version: chain.timestamp.signed.version,
            snapshot_version: chain.snapshot.signed.version,
            targets_version: chain.targets.signed.version,
            release_sequence: manifest.release_sequence,
            manifest_digest,
        },
    })
}

fn validate_release_sequence(
    previous: Option<&RollbackState>,
    manifest: &ReleaseManifest,
    manifest_digest: &str,
) -> Result<(), UpdateError> {
    if previous.is_some_and(|previous| {
        manifest.release_sequence < previous.release_sequence
            || (manifest.release_sequence == previous.release_sequence
                && manifest_digest != previous.manifest_digest)
    }) {
        return Err(UpdateError::Compatibility(
            "release sequence rollback or same-sequence substitution detected".to_owned(),
        ));
    }
    Ok(())
}

fn validate_inventory_migrations(
    inventory: &SurfaceInventory,
    migration_table: &MigrationTable,
) -> Result<(), UpdateError> {
    let declared: BTreeSet<&str> = inventory.migrations.iter().map(String::as_str).collect();
    let routed: BTreeSet<&str> = migration_table
        .routes
        .iter()
        .map(|route| route.migration_id.as_str())
        .collect();
    if declared != routed {
        return Err(UpdateError::Verification(
            "release inventory and migration table declare different migrations".to_owned(),
        ));
    }
    Ok(())
}

/// Independently compare a signed cumulative surface inventory with the
/// compiled inventory of its migration baseline.
///
/// Signed release metadata cannot choose its own interpretation: removal from
/// any baseline category is breaking, an addition is a feature, and an exact
/// surface match across a version change is a compatible fix.
///
/// # Errors
///
/// Returns an error when the compiled baseline is missing or malformed.
pub fn observe_surface_delta(
    baseline_version: SemVersion,
    target: &SurfaceInventory,
) -> Result<SurfaceDelta, UpdateError> {
    let registry: HistoricalSurfaceRegistry = serde_yaml::from_str(include_str!(
        "../../../harness/release/historical-surfaces.yml"
    ))
    .map_err(|error| {
        UpdateError::Internal(format!(
            "compiled historical surface registry is invalid: {error}"
        ))
    })?;
    if registry.schema_version != 1 {
        return Err(UpdateError::Internal(
            "compiled historical surface registry schema is unsupported".to_owned(),
        ));
    }
    let baseline = registry
        .releases
        .iter()
        .find(|inventory| inventory.product_version == baseline_version.to_string())
        .ok_or_else(|| {
            UpdateError::Unsupported(format!(
                "no compiled public-surface baseline exists for {baseline_version}"
            ))
        })?;
    validate_historical_inventory(baseline)?;
    let baseline_items = inventory_items(baseline);
    let target_items = inventory_items(target);
    if !baseline_items.is_subset(&target_items) {
        return Ok(SurfaceDelta::Breaking);
    }
    if baseline_items != target_items {
        return Ok(SurfaceDelta::AdditiveFeature);
    }
    let target_version: SemVersion = target
        .product_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    Ok(if target_version == baseline_version {
        SurfaceDelta::None
    } else {
        SurfaceDelta::CompatibleFix
    })
}

fn validate_historical_inventory(inventory: &SurfaceInventory) -> Result<(), UpdateError> {
    inventory
        .product_version
        .parse::<SemVersion>()
        .map_err(|error| {
            UpdateError::Internal(format!("invalid historical surface version: {error}"))
        })?;
    for values in inventory_lists(inventory) {
        if values.windows(2).any(|pair| pair[0] >= pair[1]) || values.iter().any(String::is_empty) {
            return Err(UpdateError::Internal(
                "compiled historical surface inventory is not sorted and unique".to_owned(),
            ));
        }
    }
    Ok(())
}

fn inventory_lists(inventory: &SurfaceInventory) -> [&Vec<String>; 7] {
    [
        &inventory.schemas,
        &inventory.skills,
        &inventory.capabilities,
        &inventory.migrations,
        &inventory.templates,
        &inventory.ownership,
        &inventory.projections,
    ]
}

fn inventory_items(inventory: &SurfaceInventory) -> BTreeSet<String> {
    let labels = [
        "schema",
        "skill",
        "capability",
        "migration",
        "template",
        "ownership",
        "projection",
    ];
    labels
        .into_iter()
        .zip(inventory_lists(inventory))
        .flat_map(|(label, values)| values.iter().map(move |value| format!("{label}:{value}")))
        .collect()
}

fn validate_provenance(
    provenance: &ProvenanceStatement,
    manifest: &ReleaseManifest,
    verified_targets: &[VerifiedTarget],
    requirement: EvidenceRequirement,
) -> Result<(), UpdateError> {
    let build = &provenance.predicate.build_definition;
    let run = &provenance.predicate.run_details;
    let source_uri = format!(
        "git+{}@{}",
        manifest.source.repository, manifest.source.commit
    );
    let source_is_bound = build.resolved_dependencies.iter().any(|dependency| {
        dependency.uri == source_uri
            && dependency.digest.len() == 1
            && dependency
                .digest
                .get("gitCommit")
                .is_some_and(|commit| commit == &manifest.source.commit)
    });
    let started = parse_iso8601_z(&run.metadata.started_on)?;
    let finished = parse_iso8601_z(&run.metadata.finished_on)?;
    let mut subject_names = BTreeSet::new();
    let subjects_valid = !provenance.subject.is_empty()
        && provenance.subject.iter().all(|subject| {
            !subject.name.is_empty()
                && subject_names.insert(subject.name.as_str())
                && subject
                    .name
                    .starts_with(&format!("aigent-hive-{}-", manifest.release_version))
                && subject.digest.len() == 1
                && subject.digest.get("sha256").is_some_and(|digest| {
                    digest.len() == 64
                        && digest
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                })
        });
    let production_subjects_valid = requirement != EvidenceRequirement::Production
        || release_artifact_targets(verified_targets).is_some_and(|artifacts| {
            let subjects: BTreeMap<&str, &str> = provenance
                .subject
                .iter()
                .filter_map(|subject| {
                    subject
                        .digest
                        .get("sha256")
                        .map(|digest| (subject.name.as_str(), digest.as_str()))
                })
                .collect();
            subjects.len() == artifacts.len()
                && artifacts.iter().all(|artifact| {
                    artifact
                        .path
                        .strip_prefix("targets/")
                        .and_then(|name| subjects.get(name))
                        .is_some_and(|digest| {
                            artifact.digest.strip_prefix("sha256:") == Some(*digest)
                        })
                })
        });
    if provenance.statement_type != PROVENANCE_STATEMENT_TYPE
        || provenance.predicate_type != PROVENANCE_PREDICATE_TYPE
        || build.build_type != PROVENANCE_BUILD_TYPE
        || !build.internal_parameters.locked
        || !build.external_parameters.is_object()
        || !source_is_bound
        || !run.builder.id.starts_with(PROVENANCE_BUILDER_PREFIX)
        || run.builder.id.len() == PROVENANCE_BUILDER_PREFIX.len()
        || run.metadata.invocation_id.is_empty()
        || started > finished
        || !subjects_valid
        || !production_subjects_valid
    {
        return Err(UpdateError::Verification(
            "provenance statement does not bind the release source and protected builder"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_platform_signing_evidence(
    evidence: &PlatformSigningEvidence,
    verified_targets: &[VerifiedTarget],
    requirement: EvidenceRequirement,
) -> Result<(), UpdateError> {
    let mut platforms = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut signers = BTreeMap::new();
    let entries_valid = evidence.schema_version == 1
        && evidence.evidence.len() >= 2
        && evidence.evidence.iter().all(|entry| {
            let expected_scheme = match entry.platform.as_str() {
                "macos" => "developer-id",
                "windows" => "authenticode",
                _ => return false,
            };
            platforms.insert(entry.platform.as_str());
            let path = Path::new(&entry.artifact_path);
            let path_valid = validate_project_relative(path).is_ok()
                && entry.artifact_path == entry.artifact_path.replace('\\', "/")
                && paths.insert(entry.artifact_path.as_str());
            let status_valid = match requirement {
                EvidenceRequirement::Integrity => {
                    matches!(
                        entry.status.as_str(),
                        "verified" | "fixture-public-evidence"
                    )
                }
                EvidenceRequirement::Production => entry.status == "verified",
            };
            let signer_valid = match entry.platform.as_str() {
                "macos" => {
                    entry.signer.kind == "apple-team-id"
                        && entry.signer.value.len() == 10
                        && entry
                            .signer
                            .value
                            .bytes()
                            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
                }
                "windows" => {
                    entry.signer.kind == "authenticode-certificate-thumbprint"
                        && entry.signer.value.len() == 40
                        && entry
                            .signer
                            .value
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
                }
                _ => false,
            };
            let signer_consistent = signers
                .insert(entry.platform.as_str(), entry.signer.value.as_str())
                .is_none_or(|prior| prior == entry.signer.value);
            expected_scheme == entry.scheme
                && path_valid
                && is_sha256_digest(&entry.artifact_digest)
                && signer_valid
                && signer_consistent
                && status_valid
        });
    let production_artifacts_valid = requirement != EvidenceRequirement::Production
        || release_artifact_targets(verified_targets).is_some_and(|artifacts| {
            let declared: BTreeMap<&str, &str> = evidence
                .evidence
                .iter()
                .map(|entry| (entry.artifact_path.as_str(), entry.artifact_digest.as_str()))
                .collect();
            declared.len() == artifacts.len()
                && artifacts.iter().all(|artifact| {
                    declared
                        .get(artifact.path.as_str())
                        .is_some_and(|digest| **digest == artifact.digest)
                })
        });
    if !entries_valid
        || platforms != BTreeSet::from(["macos", "windows"])
        || !production_artifacts_valid
    {
        return Err(UpdateError::Verification(
            "platform signing evidence is incomplete, inconsistent, or not production-verified"
                .to_owned(),
        ));
    }
    Ok(())
}

fn release_artifact_targets(targets: &[VerifiedTarget]) -> Option<Vec<&VerifiedTarget>> {
    let artifacts: Vec<_> = targets
        .iter()
        .filter(|target| !REQUIRED_RELEASE_PAYLOADS.contains(&target.path.as_str()))
        .collect();
    (!artifacts.is_empty()).then_some(artifacts)
}

fn validate_root(
    root: &TufRoot,
    now_unix: i64,
    previous: Option<&RollbackState>,
) -> Result<(), UpdateError> {
    validate_common(
        "root",
        &root.signed.metadata_type,
        &root.signed.spec_version,
        &root.signed.expires,
        now_unix,
    )?;
    if root.signed.version == 0
        || !root.signed.consistent_snapshot
        || root
            .signed
            .roles
            .keys()
            .any(|role| !matches!(role.as_str(), "root" | "targets" | "snapshot" | "timestamp"))
    {
        return Err(UpdateError::Verification(
            "trusted root contains an unsupported role".to_owned(),
        ));
    }
    for role_name in ["root", "targets", "snapshot", "timestamp"] {
        let role = root.signed.roles.get(role_name).ok_or_else(|| {
            UpdateError::Verification(format!("trusted root omits {role_name} role"))
        })?;
        validate_role_definition(&root.signed, role_name, role)?;
    }
    validate_root_key_separation(&root.signed)?;
    verify_role(&root.signed, "root", root)?;
    if previous.is_some_and(|previous| root.signed.version < previous.root_version) {
        return Err(UpdateError::Compatibility(
            "trusted root metadata rollback detected".to_owned(),
        ));
    }
    Ok(())
}

fn validate_root_key_separation(root: &RootSigned) -> Result<(), UpdateError> {
    let root_role = root
        .roles
        .get("root")
        .ok_or_else(|| UpdateError::Verification("trusted root omits root role".to_owned()))?;
    if root_role.threshold != 2 || root_role.keyids.len() != 3 {
        return Err(UpdateError::Verification(
            "trusted root must use the product 2-of-3 offline threshold".to_owned(),
        ));
    }
    let mut public_keys = BTreeSet::new();
    for (key_id, key) in &root.keys {
        if key_id.is_empty()
            || key_id.len() > 64
            || !key_id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || key.keytype != "ed25519"
            || key.scheme != "ed25519"
            || decode_prefixed_hex::<32>(&key.keyval.public).is_err()
            || !public_keys.insert(key.keyval.public.as_str())
        {
            return Err(UpdateError::Verification(
                "trusted root contains duplicate or invalid public-key material".to_owned(),
            ));
        }
    }
    let mut assigned = BTreeSet::new();
    for role in root.roles.values() {
        for key_id in &role.keyids {
            if !assigned.insert(key_id.as_str()) {
                return Err(UpdateError::Verification(
                    "trusted root reuses a key across release roles".to_owned(),
                ));
            }
        }
    }
    if assigned.len() != root.keys.len() {
        return Err(UpdateError::Verification(
            "trusted root contains an unassigned verification key".to_owned(),
        ));
    }
    Ok(())
}

fn validate_role_definition(
    root: &RootSigned,
    name: &str,
    role: &TufRole,
) -> Result<(), UpdateError> {
    let unique: BTreeSet<&str> = role.keyids.iter().map(String::as_str).collect();
    if role.threshold == 0
        || usize::try_from(role.threshold)
            .ok()
            .is_none_or(|threshold| threshold > unique.len())
        || unique.len() != role.keyids.len()
        || unique.iter().any(|keyid| !root.keys.contains_key(*keyid))
    {
        return Err(UpdateError::Verification(format!(
            "trusted root {name} role is invalid"
        )));
    }
    Ok(())
}

fn verify_role<T: Serialize>(
    root: &RootSigned,
    role_name: &str,
    envelope: &SignedEnvelope<T>,
) -> Result<(), UpdateError> {
    let role = root.signed_role(role_name)?;
    validate_role_definition(root, role_name, role)?;
    let canonical = serde_json_canonicalizer::to_vec(&envelope.signed).map_err(|error| {
        UpdateError::Verification(format!("cannot canonicalize {role_name} metadata: {error}"))
    })?;
    let mut valid = BTreeSet::new();
    for signature in &envelope.signatures {
        if valid.contains(signature.keyid.as_str()) || !role.keyids.contains(&signature.keyid) {
            continue;
        }
        let key = root.keys.get(&signature.keyid).ok_or_else(|| {
            UpdateError::Verification(format!("{role_name} signature references unknown key"))
        })?;
        if verify_ed25519(key, &signature.sig, &canonical).is_ok() {
            valid.insert(signature.keyid.as_str());
        }
    }
    if valid.len() < usize::try_from(role.threshold).unwrap_or(usize::MAX) {
        return Err(UpdateError::Verification(format!(
            "{role_name} signature threshold is not satisfied"
        )));
    }
    Ok(())
}

impl RootSigned {
    fn signed_role(&self, name: &str) -> Result<&TufRole, UpdateError> {
        self.roles
            .get(name)
            .ok_or_else(|| UpdateError::Verification(format!("trusted root omits {name} role")))
    }
}

fn verify_ed25519(key: &TufKey, signature: &str, message: &[u8]) -> Result<(), UpdateError> {
    if key.keytype != "ed25519" || key.scheme != "ed25519" {
        return Err(UpdateError::Verification(
            "release key uses an unsupported algorithm".to_owned(),
        ));
    }
    let public_key = decode_prefixed_hex::<32>(&key.keyval.public)?;
    let signature = decode_prefixed_hex::<64>(signature)?;
    let key = VerifyingKey::from_bytes(&public_key)
        .map_err(|_| UpdateError::Verification("release public key is invalid".to_owned()))?;
    key.verify_strict(message, &Signature::from_bytes(&signature))
        .map_err(|_| UpdateError::Verification("release signature is invalid".to_owned()))
}

fn decode_prefixed_hex<const N: usize>(value: &str) -> Result<[u8; N], UpdateError> {
    let hex = value.strip_prefix("ed25519:").ok_or_else(|| {
        UpdateError::Verification("Ed25519 material must use the ed25519: prefix".to_owned())
    })?;
    if hex.len() != N * 2
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(UpdateError::Verification(
            "Ed25519 material has invalid lowercase raw hex".to_owned(),
        ));
    }
    let mut output = [0_u8; N];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).map_err(|_| {
            UpdateError::Verification("Ed25519 material contains invalid hex".to_owned())
        })?;
    }
    Ok(output)
}

fn validate_common(
    expected_type: &str,
    actual_type: &str,
    spec_version: &str,
    expires: &str,
    now_unix: i64,
) -> Result<(), UpdateError> {
    if actual_type != expected_type || spec_version != TUF_SPEC_VERSION {
        return Err(UpdateError::Verification(format!(
            "{expected_type} metadata type or TUF version is unsupported"
        )));
    }
    let expires_unix = parse_iso8601_z(expires)?;
    if now_unix >= expires_unix {
        return Err(UpdateError::Verification(format!(
            "{expected_type} metadata is expired"
        )));
    }
    Ok(())
}

fn enforce_rollback(
    root: &TufRoot,
    timestamp: &TufTimestamp,
    snapshot: &TufSnapshot,
    targets: &TufTargets,
    previous: Option<&RollbackState>,
) -> Result<(), UpdateError> {
    if let Some(previous) = previous {
        if root.signed.version < previous.root_version
            || timestamp.signed.version < previous.timestamp_version
            || snapshot.signed.version < previous.snapshot_version
            || targets.signed.version < previous.targets_version
        {
            return Err(UpdateError::Compatibility(
                "TUF metadata rollback detected".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_release_manifest(manifest: &ReleaseManifest) -> Result<(), UpdateError> {
    let release: SemVersion = manifest
        .release_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    let minimum_updater: SemVersion = manifest
        .minimum_updater_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    let minimum_harness: SemVersion = manifest
        .minimum_supported_harness_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    if manifest.schema_version != 1
        || manifest.product != "aigent-hive"
        || manifest.license != "Apache-2.0"
        || manifest.release_sequence == 0
        || manifest.source.repository != SOURCE_REPOSITORY
        || manifest.source.commit.len() != 40
        || !manifest
            .source
            .commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || manifest.source.tag != format!("v{release}")
        || minimum_updater > release
        || minimum_harness > release
        || ![
            &manifest.surface_inventory_digest,
            &manifest.migration_table_digest,
            &manifest.provenance_digest,
            &manifest.platform_signing_evidence_digest,
        ]
        .iter()
        .all(|digest| is_sha256_digest(digest))
    {
        return Err(UpdateError::Verification(
            "release bundle manifest violates the product contract".to_owned(),
        ));
    }
    Ok(())
}

fn validate_sorted_inventory(inventory: &SurfaceInventory) -> Result<(), UpdateError> {
    if inventory.schema_version != 1 {
        return Err(UpdateError::Input(
            "unsupported release surface inventory schema".to_owned(),
        ));
    }
    for values in [
        &inventory.schemas,
        &inventory.skills,
        &inventory.capabilities,
        &inventory.migrations,
        &inventory.templates,
        &inventory.ownership,
        &inventory.projections,
    ] {
        if values.is_empty()
            || values.windows(2).any(|pair| pair[0] >= pair[1])
            || values.iter().any(String::is_empty)
        {
            return Err(UpdateError::Verification(
                "release surface inventory must be non-empty, sorted, and unique".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_target_path(path: &str) -> Result<String, UpdateError> {
    let candidate = Path::new(path);
    validate_project_relative(candidate)
        .map_err(|error| UpdateError::Verification(error.to_string()))?;
    let normalized = path.replace('\\', "/");
    if !normalized.starts_with("targets/")
        || normalized.contains("//")
        || normalized.bytes().any(|byte| byte == 0)
    {
        return Err(UpdateError::Verification(
            "TUF target path is outside the release targets namespace".to_owned(),
        ));
    }
    Ok(normalized)
}

fn read_bundle_file(
    repository: &PinnedReleaseRepository,
    relative: &Path,
    maximum: u64,
) -> Result<Vec<u8>, UpdateError> {
    repository.read_bounded(relative, maximum)
}

fn verify_description(
    bytes: &[u8],
    description: &MetadataDescription,
    label: &str,
) -> Result<(), UpdateError> {
    if bytes.len() as u64 != description.length {
        return Err(UpdateError::Verification(format!(
            "{label} length differs from signed metadata"
        )));
    }
    require_hash_map(bytes, &description.hashes, label)
}

fn verify_target_description(
    bytes: &[u8],
    description: &TargetDescription,
    label: &str,
) -> Result<(), UpdateError> {
    if bytes.len() as u64 != description.length {
        return Err(UpdateError::Verification(format!(
            "{label} length differs from targets metadata"
        )));
    }
    require_hash_map(bytes, &description.hashes, label)
}

fn require_hash_map(
    bytes: &[u8],
    hashes: &BTreeMap<String, String>,
    label: &str,
) -> Result<(), UpdateError> {
    if hashes.keys().any(|algorithm| algorithm != "sha256") {
        return Err(UpdateError::Verification(format!(
            "{label} declares an unsupported digest algorithm"
        )));
    }
    let expected = hashes
        .get("sha256")
        .ok_or_else(|| UpdateError::Verification(format!("{label} omits SHA-256")))?;
    let actual = hex_digest(bytes);
    if expected != &actual {
        return Err(UpdateError::Verification(format!(
            "{label} SHA-256 differs from signed metadata"
        )));
    }
    Ok(())
}

fn required_verified_target(
    repository: &PinnedReleaseRepository,
    targets: &[VerifiedTarget],
    path: &str,
) -> Result<Vec<u8>, UpdateError> {
    let target = targets
        .iter()
        .find(|target| target.path == path)
        .ok_or_else(|| {
            UpdateError::Verification(format!("release omits required target: {path}"))
        })?;
    let bytes = read_bundle_file(repository, Path::new(path), MAX_TARGET_BYTES)?;
    if bytes.len() as u64 != target.length || sha256_digest(&bytes) != target.digest {
        return Err(UpdateError::Verification(format!(
            "required target changed after verification: {path}"
        )));
    }
    Ok(bytes)
}

fn require_digest(bytes: &[u8], expected: &str, label: &str) -> Result<(), UpdateError> {
    if sha256_digest(bytes) != expected {
        return Err(UpdateError::Verification(format!(
            "{label} digest differs from release manifest"
        )));
    }
    Ok(())
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T, UpdateError> {
    serde_json::from_slice(bytes)
        .map_err(|error| UpdateError::Input(format!("invalid {label} JSON: {error}")))
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

fn parse_iso8601_z(value: &str) -> Result<i64, UpdateError> {
    let (date, time) = value
        .strip_suffix('Z')
        .and_then(|value| value.split_once('T'))
        .ok_or_else(|| UpdateError::Input("timestamp must use exact UTC RFC3339".to_owned()))?;
    let mut date_parts = date.split('-');
    let year = parse_part(date_parts.next(), 4)?;
    let month = parse_part(date_parts.next(), 2)?;
    let day = parse_part(date_parts.next(), 2)?;
    if date_parts.next().is_some() {
        return Err(UpdateError::Input("invalid timestamp date".to_owned()));
    }
    let mut time_parts = time.split(':');
    let hour = parse_part(time_parts.next(), 2)?;
    let minute = parse_part(time_parts.next(), 2)?;
    let second = parse_part(time_parts.next(), 2)?;
    if time_parts.next().is_some()
        || !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(UpdateError::Input(
            "invalid timestamp calendar value".to_owned(),
        ));
    }
    let days = days_from_civil(year, month, day);
    days.checked_mul(86_400)
        .and_then(|seconds| seconds.checked_add(i64::from(hour) * 3_600))
        .and_then(|seconds| seconds.checked_add(i64::from(minute) * 60))
        .and_then(|seconds| seconds.checked_add(i64::from(second)))
        .ok_or_else(|| UpdateError::Input("timestamp is out of range".to_owned()))
}

fn parse_part(value: Option<&str>, width: usize) -> Result<i32, UpdateError> {
    let value = value.ok_or_else(|| UpdateError::Input("timestamp part is missing".to_owned()))?;
    if value.len() != width || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(UpdateError::Input("timestamp part is invalid".to_owned()));
    }
    value
        .parse()
        .map_err(|_| UpdateError::Input("timestamp part is out of range".to_owned()))
}

const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i32, month: i32, day: i32) -> i64 {
    let adjusted_year = year - i32::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    i64::from(era * 146_097 + day_of_era - 719_468)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn copy_tree(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).expect("create fixture copy");
        for entry in fs::read_dir(source).expect("read fixture directory") {
            let entry = entry.expect("fixture entry");
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            if entry.file_type().expect("fixture type").is_dir() {
                copy_tree(&source_path, &destination_path);
            } else {
                fs::copy(source_path, destination_path).expect("copy fixture file");
            }
        }
    }

    #[test]
    fn rfc_8032_public_only_known_answer_verifies_without_signing_api() {
        let key = TufKey {
            keytype: "ed25519".to_owned(),
            scheme: "ed25519".to_owned(),
            keyval: TufKeyValue {
                public: concat!(
                    "ed25519:",
                    "d75a980182b10ab7d54bfed3c964073a",
                    "0ee172f3daa62325af021a68f707511a"
                )
                .to_owned(),
            },
        };
        let signature = concat!(
            "ed25519:",
            "e5564300c360ac729086e2cc806e828a",
            "84877f1eb8e5d974d873e06522490155",
            "5fb8821590a33bacc61e39701cf9b46b",
            "d25bf5f0595bbe24655141438e7a100b"
        );
        verify_ed25519(&key, signature, b"").expect("RFC 8032 test vector");
        assert!(verify_ed25519(&key, signature, b"tampered").is_err());
    }

    #[test]
    fn duplicate_signatures_cannot_satisfy_a_threshold() {
        let root = RootSigned {
            metadata_type: "root".to_owned(),
            spec_version: TUF_SPEC_VERSION.to_owned(),
            version: 1,
            expires: "2030-01-01T00:00:00Z".to_owned(),
            consistent_snapshot: true,
            keys: BTreeMap::from([(
                "root-a".to_owned(),
                TufKey {
                    keytype: "ed25519".to_owned(),
                    scheme: "ed25519".to_owned(),
                    keyval: TufKeyValue {
                        public: format!("ed25519:{}", "0".repeat(64)),
                    },
                },
            )]),
            roles: BTreeMap::from([(
                "root".to_owned(),
                TufRole {
                    keyids: vec!["root-a".to_owned()],
                    threshold: 1,
                },
            )]),
        };
        let envelope = SignedEnvelope {
            signatures: vec![
                TufSignature {
                    keyid: "root-a".to_owned(),
                    sig: format!("ed25519:{}", "0".repeat(128)),
                },
                TufSignature {
                    keyid: "root-a".to_owned(),
                    sig: format!("ed25519:{}", "0".repeat(128)),
                },
            ],
            signed: root.clone(),
        };
        assert!(verify_role(&root, "root", &envelope).is_err());
    }

    #[test]
    fn expiry_boundary_and_impossible_calendar_fail_closed() {
        let expiry = parse_iso8601_z("2026-07-24T00:00:00Z").expect("timestamp");
        assert!(validate_common(
            "targets",
            "targets",
            TUF_SPEC_VERSION,
            "2026-07-24T00:00:00Z",
            expiry
        )
        .is_err());
        assert!(parse_iso8601_z("2026-02-30T00:00:00Z").is_err());
    }

    #[test]
    fn target_paths_reject_escape_absolute_and_foreign_namespaces() {
        for invalid in [
            "../manifest.json",
            "/targets/manifest.json",
            "metadata/root.json",
            "targets//manifest.json",
        ] {
            assert!(validate_target_path(invalid).is_err(), "{invalid}");
        }
        assert_eq!(
            validate_target_path("targets/bundle-manifest.json").expect("target"),
            "targets/bundle-manifest.json"
        );
    }

    #[test]
    fn public_only_threshold_fixture_verifies_and_binds_every_target() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0");
        let root = fs::read(fixture.join("metadata/root.json")).expect("public trusted root");
        let verified = verify_release_repository(
            &root,
            &fixture,
            parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
            None,
        )
        .expect("valid release");
        assert_eq!(verified.manifest.release_version, "0.7.0");
        assert_eq!(verified.targets.len(), 5);
        assert_eq!(verified.next_rollback_state.release_sequence, 7);
        assert_eq!(
            verified.provenance.predicate_type,
            PROVENANCE_PREDICATE_TYPE
        );
        assert_eq!(verified.platform_signing_evidence.evidence.len(), 2);

        let previous = RollbackState {
            root_version: 1,
            timestamp_version: 8,
            snapshot_version: 7,
            targets_version: 7,
            release_sequence: 8,
            manifest_digest: format!("sha256:{}", "f".repeat(64)),
        };
        assert!(matches!(
            verify_release_repository(
                &root,
                &fixture,
                parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
                Some(&previous),
            ),
            Err(UpdateError::Compatibility(_))
        ));
    }

    #[test]
    fn observed_surface_delta_comes_from_compiled_history_not_declared_class() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0");
        let root = fs::read(fixture.join("metadata/root.json")).expect("root");
        let verified =
            verify_release_repository(&root, &fixture, 1_800_000_000, None).expect("release");
        assert_eq!(
            observe_surface_delta(
                "0.6.0".parse().expect("baseline"),
                &verified.surface_inventory
            )
            .expect("observed"),
            SurfaceDelta::AdditiveFeature
        );

        let mut removed = verified.surface_inventory.clone();
        removed.capabilities.remove(0);
        assert_eq!(
            observe_surface_delta("0.6.0".parse().expect("baseline"), &removed).expect("observed"),
            SurfaceDelta::Breaking
        );

        let baseline = SurfaceInventory {
            schema_version: 1,
            product_version: "0.6.1".to_owned(),
            schemas: vec!["action-result.schema.json".to_owned()],
            skills: vec!["hive-judge-package".to_owned()],
            capabilities: vec!["authenticated-judge-quorum".to_owned()],
            migrations: Vec::new(),
            templates: vec![".hive/config/harness.toml".to_owned()],
            ownership: vec!["hive-generated-config".to_owned()],
            projections: vec![".agents/skills/hive-judge-package/SKILL.md".to_owned()],
        };
        assert_eq!(
            observe_surface_delta("0.6.0".parse().expect("baseline"), &baseline).expect("observed"),
            SurfaceDelta::CompatibleFix
        );

        let published_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../tests/fixtures/phase6/releases/valid-0.8.0/targets/release-surface-inventory.json",
        );
        let published: SurfaceInventory = parse_json(
            &fs::read(published_path).expect("published inventory"),
            "published inventory",
        )
        .expect("published inventory");
        assert_eq!(
            observe_surface_delta("0.8.0".parse().expect("baseline"), &published)
                .expect("published baseline"),
            SurfaceDelta::None
        );

        let mut next_feature = published;
        next_feature.product_version = "0.9.0".to_owned();
        next_feature.skills.push("hive-loop-engineering".to_owned());
        next_feature.skills.sort();
        assert_eq!(
            observe_surface_delta("0.8.0".parse().expect("baseline"), &next_feature)
                .expect("next feature"),
            SurfaceDelta::AdditiveFeature
        );
    }

    #[test]
    fn publication_verification_rejects_public_fixture_evidence() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0");
        let root = fs::read(fixture.join("metadata/root.json")).expect("public trusted root");
        assert!(matches!(
            verify_release_repository_for_publication(
                &root,
                &fixture,
                parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
                None,
            ),
            Err(UpdateError::Verification(_))
        ));
    }

    #[test]
    fn provenance_semantics_bind_source_builder_and_subjects() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0/targets");
        let manifest: ReleaseManifest = parse_json(
            &fs::read(fixture.join("bundle-manifest.json")).expect("manifest"),
            "manifest",
        )
        .expect("manifest");
        let mut provenance: ProvenanceStatement = parse_json(
            &fs::read(fixture.join("provenance.intoto.json")).expect("provenance"),
            "provenance",
        )
        .expect("provenance");
        validate_provenance(&provenance, &manifest, &[], EvidenceRequirement::Integrity)
            .expect("valid provenance");

        provenance.predicate.build_definition.resolved_dependencies[0].uri =
            "git+https://example.invalid/attacker@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_owned();
        assert!(
            validate_provenance(&provenance, &manifest, &[], EvidenceRequirement::Integrity)
                .is_err()
        );
    }

    #[test]
    fn platform_evidence_requires_authorized_signer_scheme_paths_and_production_status() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../tests/fixtures/phase6/releases/valid-0.7.0/targets/platform-signing-evidence.json",
        );
        let mut evidence: PlatformSigningEvidence =
            parse_json(&fs::read(fixture).expect("evidence"), "evidence").expect("evidence");
        validate_platform_signing_evidence(&evidence, &[], EvidenceRequirement::Integrity)
            .expect("fixture integrity");
        assert!(validate_platform_signing_evidence(
            &evidence,
            &[],
            EvidenceRequirement::Production
        )
        .is_err());
        let mut wrong_signer = evidence.evidence[0].clone();
        wrong_signer.artifact_path = "bin/hive-intel".to_owned();
        wrong_signer.signer.value = "OTHER12345".to_owned();
        evidence.evidence.push(wrong_signer);
        assert!(
            validate_platform_signing_evidence(&evidence, &[], EvidenceRequirement::Integrity)
                .is_err()
        );
        evidence.evidence.pop();
        evidence.evidence[0].scheme = "authenticode".to_owned();
        assert!(
            validate_platform_signing_evidence(&evidence, &[], EvidenceRequirement::Integrity)
                .is_err()
        );
        evidence.evidence[0].scheme = "developer-id".to_owned();
        evidence.evidence[0].signer.kind = "authenticode-certificate-thumbprint".to_owned();
        assert!(
            validate_platform_signing_evidence(&evidence, &[], EvidenceRequirement::Integrity)
                .is_err()
        );
    }

    #[test]
    fn tuf_rejects_a_well_formed_but_unauthorized_platform_signer() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0");
        let temporary = tempfile::tempdir().expect("temporary release");
        copy_tree(&fixture, temporary.path());
        let root = fs::read(temporary.path().join("metadata/root.json")).expect("trusted root");
        let target = temporary
            .path()
            .join("targets/platform-signing-evidence.json");
        let mut evidence: serde_json::Value =
            serde_json::from_slice(&fs::read(&target).expect("evidence")).expect("JSON");
        evidence["evidence"][0]["signer"]["value"] =
            serde_json::Value::String("OTHER12345".to_owned());
        fs::write(
            target,
            serde_json::to_vec(&evidence).expect("serialize evidence"),
        )
        .expect("write tampered evidence");
        assert!(matches!(
            verify_release_repository(
                &root,
                temporary.path(),
                parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
                None,
            ),
            Err(UpdateError::Verification(_))
        ));
    }

    #[test]
    fn tampered_target_is_rejected_before_payload_parsing() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0");
        let temporary = tempfile::tempdir().expect("temporary release");
        copy_tree(&fixture, temporary.path());
        let root = fs::read(temporary.path().join("metadata/root.json")).expect("trusted root");
        let target = temporary.path().join("targets/bundle-manifest.json");
        let mut bytes = fs::read(&target).expect("target");
        bytes[0] ^= 1;
        fs::write(target, bytes).expect("tamper fixture");
        assert!(matches!(
            verify_release_repository(
                &root,
                temporary.path(),
                parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
                None,
            ),
            Err(UpdateError::Verification(_))
        ));
    }

    #[test]
    fn root_rotation_rejects_skipped_version_before_acceptance() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0/metadata/root.json");
        let trusted = fs::read(&fixture).expect("trusted root");
        let mut candidate: TufRoot = parse_json(&trusted, "trusted root").expect("root");
        candidate.signed.version += 2;
        let candidate = serde_json::to_vec(&candidate).expect("candidate");
        assert!(matches!(
            verify_root_rotation(
                &trusted,
                &candidate,
                parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
            ),
            Err(UpdateError::Compatibility(_))
        ));
    }

    #[test]
    fn trusted_root_rejects_duplicate_key_material_across_roles() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase6/releases/valid-0.7.0/metadata/root.json");
        let mut root: TufRoot =
            parse_json(&fs::read(fixture).expect("root"), "root").expect("root");
        let duplicate_public = root
            .signed
            .keys
            .get("targets-a")
            .expect("targets key")
            .keyval
            .public
            .clone();
        root.signed
            .keys
            .get_mut("snapshot-a")
            .expect("snapshot key")
            .keyval
            .public = duplicate_public;
        assert!(validate_root(
            &root,
            parse_iso8601_z("2026-07-24T00:00:00Z").expect("clock"),
            None
        )
        .is_err());
    }
}
