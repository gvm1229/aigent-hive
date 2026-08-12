use crate::{MigrationTable, ReleaseClass, SemVersion, SurfaceDelta, UpdateError};
use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use hive_core::{sha256_digest, validate_project_relative};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
const SOURCE_REPOSITORY: &str = "https://github.com/gvm1229/aigent-hive";
const MANIFEST_PATH: &str = "bundle-manifest.json";
const MIGRATION_PATH: &str = "targets/migration-table.json";
const INVENTORY_PATH: &str = "targets/release-surface-inventory.json";

struct PinnedReleaseBundle {
    requested: PathBuf,
    dir: Dir,
}

impl PinnedReleaseBundle {
    fn open(path: &Path) -> Result<Self, UpdateError> {
        let requested = absolute_lexical(path)?;
        let dir = open_directory_nofollow_path(&requested)?;
        Ok(Self { requested, dir })
    }

    fn read_bounded(&self, relative: &Path, maximum: u64) -> Result<Vec<u8>, UpdateError> {
        validate_project_relative(relative)
            .map_err(|error| UpdateError::Verification(error.to_string()))?;
        let (parent, name) = bundle_parent(&self.dir, relative)?;
        let before = parent.symlink_metadata(&name).map_err(|error| {
            UpdateError::Verification(format!(
                "cannot inspect release bundle file {}: {error}",
                relative.display()
            ))
        })?;
        if !before.is_file() || before.len() > maximum {
            return Err(UpdateError::Verification(format!(
                "release bundle file is not a bounded no-follow regular file: {}",
                relative.display()
            )));
        }
        let expected_identity = FileIdentity::from_metadata(&before);
        let expected_modified = before.modified().map_err(|error| {
            UpdateError::Verification(format!(
                "cannot inspect release bundle file timestamp {}: {error}",
                relative.display()
            ))
        })?;
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        let mut file = parent.open_with(&name, &options).map_err(|error| {
            UpdateError::Verification(format!(
                "cannot open release bundle file no-follow {}: {error}",
                relative.display()
            ))
        })?;
        let opened = file.metadata().map_err(|error| {
            UpdateError::Verification(format!(
                "cannot inspect opened release bundle file {}: {error}",
                relative.display()
            ))
        })?;
        if !opened.is_file() || opened.len() > maximum || !expected_identity.matches(&opened) {
            return Err(UpdateError::Verification(format!(
                "release bundle file changed between inspection and open: {}",
                relative.display()
            )));
        }
        let mut bytes = Vec::with_capacity(
            usize::try_from(opened.len())
                .map_err(|_| UpdateError::Verification("release file is too large".to_owned()))?,
        );
        file.by_ref()
            .take(maximum.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| {
                UpdateError::Verification(format!(
                    "cannot read release bundle file {}: {error}",
                    relative.display()
                ))
            })?;
        let after = file.metadata().map_err(|error| {
            UpdateError::Verification(format!(
                "cannot re-inspect opened release bundle file {}: {error}",
                relative.display()
            ))
        })?;
        if !after.is_file()
            || !expected_identity.matches(&after)
            || after.len() != opened.len()
            || after.modified().ok().as_ref() != Some(&expected_modified)
            || bytes.len() as u64 != opened.len()
            || bytes.len() as u64 > maximum
        {
            return Err(UpdateError::Verification(format!(
                "release bundle file identity or bytes changed during read: {}",
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
                "release bundle root changed during verification".to_owned(),
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
                UpdateError::Verification(format!("cannot resolve release bundle path: {error}"))
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
                        "release bundle path escapes its filesystem root: {}",
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
                    "release bundle directory path is not lexically safe: {}",
                    path.display()
                )));
            }
        }
    }
    if root.as_os_str().is_empty() {
        return Err(UpdateError::Verification(format!(
            "release bundle directory path is not absolute: {}",
            path.display()
        )));
    }
    let mut current = Dir::open_ambient_dir(&root, ambient_authority()).map_err(|error| {
        UpdateError::Verification(format!(
            "cannot open release bundle filesystem root {}: {error}",
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
                    "release bundle directory component is not a no-follow directory: {}",
                    walked.display()
                )));
            }
            Err(error) => {
                return Err(UpdateError::Verification(format!(
                    "cannot inspect release bundle directory component {}: {error}",
                    walked.display()
                )));
            }
        }
        current = current.open_dir_nofollow(&name).map_err(|error| {
            UpdateError::Verification(format!(
                "cannot open release bundle directory component {}: {error}",
                walked.display()
            ))
        })?;
    }
    Ok(current)
}

fn bundle_parent(root: &Dir, relative: &Path) -> Result<(Dir, OsString), UpdateError> {
    let name = relative
        .file_name()
        .ok_or_else(|| {
            UpdateError::Verification("release bundle path has no file name".to_owned())
        })?
        .to_os_string();
    let mut current = root.try_clone().map_err(|error| {
        UpdateError::Verification(format!("cannot clone release bundle root: {error}"))
    })?;
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let Component::Normal(directory) = component else {
                return Err(UpdateError::Verification(
                    "release bundle parent path is not normalized".to_owned(),
                ));
            };
            current = current.open_dir_nofollow(directory).map_err(|error| {
                UpdateError::Verification(format!(
                    "cannot open release bundle parent {}: {error}",
                    relative.display()
                ))
            })?;
        }
    }
    Ok((current, name))
}

/// Closed immutable release bundle manifest.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub product: String,
    pub release_version: String,
    pub release_sequence: u64,
    pub classification: ReleaseClass,
    pub source: ReleaseSource,
    pub minimum_updater_version: String,
    pub minimum_supported_harness_version: String,
    pub license: String,
    pub surface_inventory_digest: String,
    pub migration_table_digest: String,
    pub artifacts: Vec<ReleaseArtifact>,
}

/// Release source identity.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSource {
    pub repository: String,
    pub commit: String,
    pub tag: String,
}

/// One candidate byte bound by the release manifest.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseArtifact {
    pub path: String,
    pub length: u64,
    pub sha256: String,
}

/// Deterministic shipped-surface inventory.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceInventory {
    pub schema_version: u32,
    pub product_version: String,
    pub schemas: Vec<String>,
    pub skills: Vec<String>,
    pub capabilities: Vec<String>,
    pub migrations: Vec<String>,
    pub templates: Vec<String>,
    pub ownership: Vec<String>,
    pub projections: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalSurfaceRegistry {
    schema_version: u32,
    migration_inert_retired_surfaces: Vec<String>,
    releases: Vec<SurfaceInventory>,
}

/// Last locally accepted release identity. This rejects downgrade and same-sequence substitution.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseState {
    pub release_version: String,
    pub release_sequence: u64,
    pub manifest_digest: String,
}

/// Verified bundle target.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VerifiedTarget {
    pub path: String,
    pub digest: String,
    pub length: u64,
}

/// Complete local release-integrity result.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReleaseVerification {
    pub manifest: ReleaseManifest,
    pub migration_table: MigrationTable,
    pub surface_inventory: SurfaceInventory,
    pub targets: Vec<VerifiedTarget>,
    pub manifest_digest: String,
    pub next_release_state: ReleaseState,
}

/// Verify a local release bundle using the manifest's exact version, length, and SHA-256 list.
///
/// Distribution authenticity is established before download by npm registry integrity or GitHub
/// artifact attestations. This verifier deliberately contains no signing key, trust-root, or
/// network path.
///
/// # Errors
///
/// Returns [`UpdateError`] when the bundle layout, manifest, release sequence, artifact bytes,
/// migration table, or cumulative surface inventory fails validation.
pub fn verify_release_bundle(
    bundle_root: &Path,
    previous: Option<&ReleaseState>,
) -> Result<ReleaseVerification, UpdateError> {
    let bundle = PinnedReleaseBundle::open(bundle_root)?;
    let manifest_bytes = bundle.read_bounded(Path::new(MANIFEST_PATH), MAX_MANIFEST_BYTES)?;
    let manifest: ReleaseManifest = parse_json(&manifest_bytes, "release bundle manifest")?;
    validate_release_manifest(&manifest)?;
    let manifest_digest = sha256_digest(&manifest_bytes);
    validate_release_sequence(previous, &manifest, &manifest_digest)?;

    let mut targets = Vec::with_capacity(manifest.artifacts.len());
    for artifact in &manifest.artifacts {
        let bytes = bundle.read_bounded(Path::new(&artifact.path), MAX_ARTIFACT_BYTES)?;
        let digest = sha256_digest(&bytes);
        if bytes.len() as u64 != artifact.length || digest != format!("sha256:{}", artifact.sha256)
        {
            return Err(UpdateError::Verification(format!(
                "release artifact length or SHA-256 differs from the manifest: {}",
                artifact.path
            )));
        }
        targets.push(VerifiedTarget {
            path: artifact.path.clone(),
            digest,
            length: artifact.length,
        });
    }

    let migration_bytes = required_target(&bundle, &targets, MIGRATION_PATH)?;
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

    let inventory_bytes = required_target(&bundle, &targets, INVENTORY_PATH)?;
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
    bundle.verify_current()?;

    Ok(ReleaseVerification {
        manifest: manifest.clone(),
        migration_table,
        surface_inventory,
        targets,
        manifest_digest: manifest_digest.clone(),
        next_release_state: ReleaseState {
            release_version: manifest.release_version,
            release_sequence: manifest.release_sequence,
            manifest_digest,
        },
    })
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
    let mut paths = HashSet::new();
    let artifacts_valid = !manifest.artifacts.is_empty()
        && manifest
            .artifacts
            .windows(2)
            .all(|pair| pair[0].path < pair[1].path)
        && manifest.artifacts.iter().all(|artifact| {
            paths.insert(artifact.path.as_str())
                && artifact.length > 0
                && artifact.length <= MAX_ARTIFACT_BYTES
                && is_sha256_hex(&artifact.sha256)
                && validate_artifact_path(&artifact.path).is_ok()
        })
        && paths.contains(MIGRATION_PATH)
        && paths.contains(INVENTORY_PATH);
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
        || !is_sha256_digest(&manifest.surface_inventory_digest)
        || !is_sha256_digest(&manifest.migration_table_digest)
        || !artifacts_valid
    {
        return Err(UpdateError::Verification(
            "release bundle manifest violates the product contract".to_owned(),
        ));
    }
    Ok(())
}

fn validate_artifact_path(path: &str) -> Result<(), UpdateError> {
    let candidate = Path::new(path);
    validate_project_relative(candidate)
        .map_err(|error| UpdateError::Verification(error.to_string()))?;
    if path != path.replace('\\', "/")
        || !path.starts_with("targets/")
        || path.contains("//")
        || path.bytes().any(|byte| byte == 0)
    {
        return Err(UpdateError::Verification(
            "release artifact path is outside the targets namespace".to_owned(),
        ));
    }
    Ok(())
}

fn validate_release_sequence(
    previous: Option<&ReleaseState>,
    manifest: &ReleaseManifest,
    manifest_digest: &str,
) -> Result<(), UpdateError> {
    let candidate: SemVersion = manifest
        .release_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    if let Some(previous) = previous {
        let installed: SemVersion = previous
            .release_version
            .parse()
            .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
        if candidate < installed
            || manifest.release_sequence < previous.release_sequence
            || (manifest.release_sequence == previous.release_sequence
                && manifest_digest != previous.manifest_digest)
        {
            return Err(UpdateError::Compatibility(
                "release downgrade or same-sequence substitution detected".to_owned(),
            ));
        }
    }
    Ok(())
}

fn required_target(
    bundle: &PinnedReleaseBundle,
    targets: &[VerifiedTarget],
    path: &str,
) -> Result<Vec<u8>, UpdateError> {
    if !targets.iter().any(|target| target.path == path) {
        return Err(UpdateError::Verification(format!(
            "release bundle omits required artifact: {path}"
        )));
    }
    bundle.read_bounded(Path::new(path), MAX_MANIFEST_BYTES)
}

fn require_digest(bytes: &[u8], expected: &str, label: &str) -> Result<(), UpdateError> {
    if sha256_digest(bytes) != expected {
        return Err(UpdateError::Verification(format!(
            "{label} digest differs from the release manifest"
        )));
    }
    Ok(())
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T, UpdateError> {
    serde_json::from_slice(bytes)
        .map_err(|error| UpdateError::Input(format!("invalid {label} JSON: {error}")))
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(is_sha256_hex)
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_sorted_inventory(inventory: &SurfaceInventory) -> Result<(), UpdateError> {
    if inventory.schema_version != 1 {
        return Err(UpdateError::Input(
            "unsupported release surface inventory schema".to_owned(),
        ));
    }
    for values in inventory_lists(inventory) {
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

/// Compare a cumulative surface inventory with the compiled migration baseline.
///
/// # Errors
///
/// Returns [`UpdateError`] when the compiled historical registry is invalid or the target
/// inventory removes or changes a protected surface without a declared inert retirement.
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
    if registry
        .migration_inert_retired_surfaces
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
        || registry
            .migration_inert_retired_surfaces
            .iter()
            .any(String::is_empty)
    {
        return Err(UpdateError::Internal(
            "compiled migration-inert retirement list is not sorted and unique".to_owned(),
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
    let baseline_items = inventory_items(baseline, &registry.migration_inert_retired_surfaces);
    let target_items = inventory_items(target, &registry.migration_inert_retired_surfaces);
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

fn inventory_items(
    inventory: &SurfaceInventory,
    migration_inert_retired_surfaces: &[String],
) -> BTreeSet<String> {
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
        .filter(|item| {
            migration_inert_retired_surfaces
                .binary_search(item)
                .is_err()
        })
        .collect()
}
