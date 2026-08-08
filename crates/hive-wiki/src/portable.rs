//! Deterministic, extraction-free `.hivekb` bundle validation.

use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipArchive, ZipWriter};

/// Current `.hivekb` manifest schema.
pub const BUNDLE_SCHEMA_VERSION: u32 = 1;
/// Canonical manifest archive entry.
pub const MANIFEST_PATH: &str = "manifest.json";
/// Digest sidecar for the exact canonical manifest bytes.
pub const MANIFEST_DIGEST_PATH: &str = "manifest-sha256.txt";
const DATA_PREFIX: &str = "data/";
const SHA256_PREFIX: &str = "sha256:";
const EOCD_LENGTH: usize = 22;
const CENTRAL_HEADER_LENGTH: usize = 46;
const LOCAL_HEADER_LENGTH: usize = 30;
const EOCD_SIGNATURE: u32 = 0x0605_4b50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
const STORED_VERSION_NEEDED: u16 = 10;
const UNIX_REGULAR_0644: u32 = 0o100_644;
const DOS_EPOCH_DATE: u16 = 0x0021;
type PortablePayloads = BTreeMap<String, (PortableEntryClassification, Vec<u8>)>;

/// Bounded decoder and encoder resource limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BundleLimits {
    /// Maximum complete archive size.
    pub max_archive_bytes: u64,
    /// Maximum physical ZIP entry count, including the two manifest entries.
    pub max_archive_entries: usize,
    /// Maximum compressed bytes for one physical entry.
    pub max_entry_compressed_bytes: u64,
    /// Maximum expanded bytes for one physical entry.
    pub max_entry_expanded_bytes: u64,
    /// Maximum sum of compressed physical entry bytes.
    pub max_total_compressed_bytes: u64,
    /// Maximum sum of expanded physical entry bytes.
    pub max_total_expanded_bytes: u64,
    /// Maximum canonical manifest byte length.
    pub max_manifest_bytes: u64,
    /// Maximum ASCII archive path byte length.
    pub max_path_bytes: usize,
}

impl Default for BundleLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 256 * 1024 * 1024,
            max_archive_entries: 10_002,
            max_entry_compressed_bytes: 8 * 1024 * 1024,
            max_entry_expanded_bytes: 8 * 1024 * 1024,
            max_total_compressed_bytes: 256 * 1024 * 1024,
            max_total_expanded_bytes: 256 * 1024 * 1024,
            max_manifest_bytes: 4 * 1024 * 1024,
            max_path_bytes: 240,
        }
    }
}

impl BundleLimits {
    fn validate(self) -> Result<(), PortableError> {
        if self.max_archive_bytes == 0
            || self.max_archive_entries < 2
            || self.max_entry_compressed_bytes == 0
            || self.max_entry_expanded_bytes == 0
            || self.max_total_compressed_bytes == 0
            || self.max_total_expanded_bytes == 0
            || self.max_manifest_bytes == 0
            || self.max_path_bytes < MANIFEST_DIGEST_PATH.len()
        {
            return Err(PortableError::InvalidInput(
                "bundle limits must be positive and admit the manifest entries".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Logical source kind carried across machines without an absolute path.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundleSourceKind {
    /// Canonical user-root knowledge.
    UserRoot,
    /// One registered project.
    Project,
    /// One registered or detached collection.
    Collection,
}

/// Portable source identity and canonical logical digest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleSourceIdentity {
    /// Provider-neutral source kind.
    pub kind: BundleSourceKind,
    /// Stable logical identifier; never an absolute path.
    pub id: String,
    /// Digest of the canonical source truth represented by this bundle.
    pub logical_digest: String,
}

/// Explicit export scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum BundleScope {
    /// User-root knowledge only.
    Global,
    /// User-root plus shared projects.
    Shared,
    /// One exact project, including eligible private knowledge.
    Project {
        /// Stable project identifier.
        id: String,
    },
    /// One exact registered or detached collection.
    Collection {
        /// Stable collection identifier.
        id: String,
    },
    /// All portable global, shared, and private knowledge.
    AllPortable,
}

/// Typed source classification used before bytes enter a bundle.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortableEntryClassification {
    /// Canonical Markdown knowledge.
    CanonicalMarkdown,
    /// Portable typed configuration or metadata.
    PortableMetadata,
    /// Canonical suppression data.
    Suppression,
    /// Portable provenance data.
    Provenance,
    /// Disposable `SQLite`, WAL, or SHM state; never bundled.
    DerivedSqlite,
    /// Session, journal, lock, cache, or other runtime state; never bundled.
    RuntimeState,
    /// Machine-bound absolute-path material; never bundled.
    AbsolutePath,
    /// Credential or authentication material; never bundled.
    Credential,
    /// Confidential material; never bundled.
    Confidential,
}

impl PortableEntryClassification {
    const fn is_portable(self) -> bool {
        matches!(
            self,
            Self::CanonicalMarkdown | Self::PortableMetadata | Self::Suppression | Self::Provenance
        )
    }
}

/// One caller-classified candidate entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleEntryInput {
    /// Portable relative path below the bundle `data/` directory.
    pub relative_path: String,
    /// Exact canonical bytes.
    pub bytes: Vec<u8>,
    /// Typed portability decision.
    pub classification: PortableEntryClassification,
}

/// Pure bundle construction request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleRequest {
    /// Logical source identity.
    pub source: BundleSourceIdentity,
    /// Explicit export scope.
    pub scope: BundleScope,
    /// Candidate canonical entries.
    pub entries: Vec<BundleEntryInput>,
}

/// Counts of candidate entries omitted before serialization.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BundleExclusionCounts {
    /// Disposable SQLite-family entries.
    pub derived_sqlite: usize,
    /// Runtime-state entries.
    pub runtime_state: usize,
    /// Machine-bound absolute-path entries.
    pub absolute_path: usize,
    /// Credential entries.
    pub credential: usize,
    /// Confidential entries.
    pub confidential: usize,
}

/// One canonical manifest payload entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifestEntry {
    /// Exact ASCII archive path beginning with `data/`.
    pub path: String,
    /// Exact expanded byte length.
    pub length: u64,
    /// SHA-256 digest of the exact canonical bytes.
    pub sha256: String,
    /// Portable typed classification.
    pub classification: PortableEntryClassification,
}

/// Canonical `.hivekb` manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Logical source identity.
    pub source: BundleSourceIdentity,
    /// Explicit export scope.
    pub scope: BundleScope,
    /// Strictly path-sorted payload inventory.
    pub entries: Vec<BundleManifestEntry>,
}

/// One fully validated in-memory payload entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedBundleEntry {
    archive_path: String,
    relative_path: String,
    classification: PortableEntryClassification,
    bytes: Vec<u8>,
    sha256: String,
}

impl ValidatedBundleEntry {
    /// Exact archive path named in the manifest.
    #[must_use]
    pub fn archive_path(&self) -> &str {
        &self.archive_path
    }

    /// Canonical destination-relative path below `data/`.
    #[must_use]
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    /// Validated portable classification.
    #[must_use]
    pub const fn classification(&self) -> PortableEntryClassification {
        self.classification
    }

    /// Exact validated canonical bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Digest of the exact validated canonical bytes.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Consume the entry into its destination-relative path and exact bytes.
    #[must_use]
    pub fn into_parts(self) -> (String, Vec<u8>) {
        (self.relative_path, self.bytes)
    }
}

/// Extraction-free, fully validated import plan suitable for caller-owned staging.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedBundlePlan {
    manifest: BundleManifest,
    entries: Vec<ValidatedBundleEntry>,
    archive_sha256: String,
    manifest_sha256: String,
}

impl ValidatedBundlePlan {
    /// Validated canonical manifest.
    #[must_use]
    pub const fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    /// Validated canonical payload entries in manifest order.
    #[must_use]
    pub fn entries(&self) -> &[ValidatedBundleEntry] {
        &self.entries
    }

    /// Digest of the complete archive bytes.
    #[must_use]
    pub fn archive_sha256(&self) -> &str {
        &self.archive_sha256
    }

    /// Digest of the exact canonical manifest bytes.
    #[must_use]
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }

    /// Consume the validated plan into staging-ready entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<ValidatedBundleEntry> {
        self.entries
    }
}

/// Deterministically encoded archive plus its self-validated plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedBundle {
    archive: Vec<u8>,
    plan: ValidatedBundlePlan,
    exclusions: BundleExclusionCounts,
}

impl EncodedBundle {
    /// Complete deterministic `.hivekb` bytes.
    #[must_use]
    pub fn archive(&self) -> &[u8] {
        &self.archive
    }

    /// Self-validated import plan for the encoded bytes.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedBundlePlan {
        &self.plan
    }

    /// Counts omitted by typed non-portable classification.
    #[must_use]
    pub const fn exclusions(&self) -> BundleExclusionCounts {
        self.exclusions
    }

    /// Consume the result into exact archive bytes.
    #[must_use]
    pub fn into_archive(self) -> Vec<u8> {
        self.archive
    }
}

/// Portable bundle validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortableError {
    /// Invalid caller input or limits.
    InvalidInput(String),
    /// A declared resource budget was exceeded.
    BudgetExceeded(String),
    /// Invalid, non-canonical, or unsupported ZIP structure.
    InvalidArchive(String),
    /// Malformed or non-canonical manifest data.
    InvalidManifest(String),
    /// Manifest, sidecar, length, or content digest mismatch.
    Integrity(String),
}

impl PortableError {
    /// Stable machine-facing error family.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "hive.bundle-invalid-input",
            Self::BudgetExceeded(_) => "hive.bundle-budget-exceeded",
            Self::InvalidArchive(_) => "hive.bundle-invalid-archive",
            Self::InvalidManifest(_) => "hive.bundle-invalid-manifest",
            Self::Integrity(_) => "hive.bundle-integrity-failed",
        }
    }
}

impl Display for PortableError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message)
            | Self::BudgetExceeded(message)
            | Self::InvalidArchive(message)
            | Self::InvalidManifest(message)
            | Self::Integrity(message) => formatter.write_str(message),
        }
    }
}

impl Error for PortableError {}

/// Build a deterministic Stored ZIP and immediately validate it through the decoder.
///
/// Non-portable typed entries are counted and omitted before any path or bytes are serialized.
///
/// # Errors
///
/// Returns an error for invalid identities, unsafe portable paths, collisions, resource limits,
/// serialization failures, or a failed self-validation.
pub fn encode_bundle(
    request: &BundleRequest,
    limits: BundleLimits,
) -> Result<EncodedBundle, PortableError> {
    limits.validate()?;
    validate_source(&request.source)?;
    validate_scope(&request.scope)?;
    let (portable, exclusions) = collect_portable_entries(&request.entries, limits)?;

    let physical_count = portable
        .len()
        .checked_add(2)
        .ok_or_else(|| PortableError::BudgetExceeded("bundle entry count overflow".to_owned()))?;
    if physical_count > limits.max_archive_entries {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle has {physical_count} entries; limit is {}",
            limits.max_archive_entries
        )));
    }

    let entries = portable
        .iter()
        .map(|(path, (classification, bytes))| {
            Ok(BundleManifestEntry {
                path: path.clone(),
                length: u64::try_from(bytes.len()).map_err(|_| {
                    PortableError::BudgetExceeded("bundle entry length overflow".to_owned())
                })?,
                sha256: sha256_digest(bytes),
                classification: *classification,
            })
        })
        .collect::<Result<Vec<_>, PortableError>>()?;
    let manifest = BundleManifest {
        schema_version: BUNDLE_SCHEMA_VERSION,
        source: request.source.clone(),
        scope: request.scope.clone(),
        entries,
    };
    let manifest_bytes = canonical_manifest_bytes(&manifest)?;
    let manifest_length = u64::try_from(manifest_bytes.len())
        .map_err(|_| PortableError::BudgetExceeded("manifest length overflow".to_owned()))?;
    if manifest_length > limits.max_manifest_bytes {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle manifest exceeds {} bytes",
            limits.max_manifest_bytes
        )));
    }
    let manifest_digest = sha256_digest(&manifest_bytes);
    let sidecar = format!("{manifest_digest}\n").into_bytes();
    let mut files = Vec::with_capacity(physical_count);
    files.push((MANIFEST_PATH.to_owned(), manifest_bytes));
    files.push((MANIFEST_DIGEST_PATH.to_owned(), sidecar));
    files.extend(portable.into_iter().map(|(path, (_, bytes))| (path, bytes)));
    preflight_file_budgets(&files, limits)?;
    let expected_archive_length = canonical_archive_length(&files)?;
    if expected_archive_length > limits.max_archive_bytes {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle archive exceeds {} bytes",
            limits.max_archive_bytes
        )));
    }
    let archive = write_canonical_zip(&files)?;
    let archive_length = u64::try_from(archive.len())
        .map_err(|_| PortableError::BudgetExceeded("archive length overflow".to_owned()))?;
    if archive_length != expected_archive_length {
        return Err(PortableError::InvalidArchive(
            "ZIP writer output length is not canonical".to_owned(),
        ));
    }
    let plan = decode_bundle(&archive, limits)?;
    Ok(EncodedBundle {
        archive,
        plan,
        exclusions,
    })
}

fn collect_portable_entries(
    candidates: &[BundleEntryInput],
    limits: BundleLimits,
) -> Result<(PortablePayloads, BundleExclusionCounts), PortableError> {
    let mut exclusions = BundleExclusionCounts::default();
    let mut portable = BTreeMap::new();
    let mut folded = BTreeSet::new();
    let mut portable_total = 0_u64;
    for candidate in candidates {
        if !candidate.classification.is_portable() {
            increment_exclusion(&mut exclusions, candidate.classification)?;
            continue;
        }
        let archive_path = payload_archive_path(&candidate.relative_path, limits)?;
        if portable.contains_key(&archive_path) {
            return Err(PortableError::InvalidInput(format!(
                "duplicate bundle payload path: {archive_path}"
            )));
        }
        let folded_path = archive_path.to_ascii_lowercase();
        if folded.contains(&folded_path) {
            return Err(PortableError::InvalidInput(format!(
                "bundle payload path collides under ASCII casefold: {archive_path}"
            )));
        }
        let physical_count = portable.len().checked_add(3).ok_or_else(|| {
            PortableError::BudgetExceeded("bundle entry count overflow".to_owned())
        })?;
        if physical_count > limits.max_archive_entries {
            return Err(PortableError::BudgetExceeded(format!(
                "bundle has {physical_count} entries; limit is {}",
                limits.max_archive_entries
            )));
        }
        let length = u64::try_from(candidate.bytes.len())
            .map_err(|_| PortableError::BudgetExceeded("entry length overflow".to_owned()))?;
        check_entry_budgets(&archive_path, length, length, limits)?;
        portable_total = portable_total.checked_add(length).ok_or_else(|| {
            PortableError::BudgetExceeded("bundle total length overflow".to_owned())
        })?;
        if portable_total > limits.max_total_compressed_bytes
            || portable_total > limits.max_total_expanded_bytes
        {
            return Err(PortableError::BudgetExceeded(
                "bundle entries exceed total byte budget".to_owned(),
            ));
        }
        folded.insert(folded_path);
        portable.insert(
            archive_path,
            (candidate.classification, candidate.bytes.clone()),
        );
    }
    Ok((portable, exclusions))
}

/// Validate a complete `.hivekb` archive without extracting or writing any entry.
///
/// # Errors
///
/// Returns an error before yielding bytes when the archive is malformed, non-canonical, unsafe,
/// over budget, or inconsistent with its canonical manifest.
pub fn decode_bundle(
    bytes: &[u8],
    limits: BundleLimits,
) -> Result<ValidatedBundlePlan, PortableError> {
    limits.validate()?;
    let archive_length = u64::try_from(bytes.len())
        .map_err(|_| PortableError::BudgetExceeded("archive length overflow".to_owned()))?;
    if archive_length > limits.max_archive_bytes {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle archive exceeds {} bytes",
            limits.max_archive_bytes
        )));
    }
    let central = scan_canonical_central_directory(bytes, limits)?;
    let raw_entries = read_validated_entries(bytes, &central)?;
    validate_raw_bundle(bytes, raw_entries, limits)
}

fn increment_exclusion(
    counts: &mut BundleExclusionCounts,
    classification: PortableEntryClassification,
) -> Result<(), PortableError> {
    let counter = match classification {
        PortableEntryClassification::DerivedSqlite => &mut counts.derived_sqlite,
        PortableEntryClassification::RuntimeState => &mut counts.runtime_state,
        PortableEntryClassification::AbsolutePath => &mut counts.absolute_path,
        PortableEntryClassification::Credential => &mut counts.credential,
        PortableEntryClassification::Confidential => &mut counts.confidential,
        _ => {
            return Err(PortableError::InvalidInput(
                "portable entry was routed through the exclusion counter".to_owned(),
            ));
        }
    };
    *counter = counter
        .checked_add(1)
        .ok_or_else(|| PortableError::BudgetExceeded("excluded entry count overflow".to_owned()))?;
    Ok(())
}

fn validate_source(source: &BundleSourceIdentity) -> Result<(), PortableError> {
    validate_identity(&source.id, "source id")?;
    if !valid_sha256(&source.logical_digest) {
        return Err(PortableError::InvalidInput(
            "source logical_digest must be lowercase sha256".to_owned(),
        ));
    }
    Ok(())
}

fn validate_scope(scope: &BundleScope) -> Result<(), PortableError> {
    match scope {
        BundleScope::Project { id } => validate_identity(id, "project scope id"),
        BundleScope::Collection { id } => validate_identity(id, "collection scope id"),
        BundleScope::Global | BundleScope::Shared | BundleScope::AllPortable => Ok(()),
    }
}

fn validate_identity(value: &str, field: &str) -> Result<(), PortableError> {
    if value.is_empty()
        || value.len() > 128
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        || !value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
    {
        return Err(PortableError::InvalidInput(format!(
            "{field} must be a bounded ASCII logical identifier"
        )));
    }
    Ok(())
}

fn payload_archive_path(path: &str, limits: BundleLimits) -> Result<String, PortableError> {
    validate_relative_path(
        path,
        limits.max_path_bytes.saturating_sub(DATA_PREFIX.len()),
    )
    .map_err(|error| PortableError::InvalidInput(error.to_string()))?;
    let archive_path = format!("{DATA_PREFIX}{path}");
    validate_relative_path(&archive_path, limits.max_path_bytes)
        .map_err(|error| PortableError::InvalidInput(error.to_string()))?;
    Ok(archive_path)
}

fn validate_relative_path(path: &str, max_bytes: usize) -> Result<(), PortableError> {
    if path.is_empty()
        || path.len() > max_bytes
        || !path.is_ascii()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(PortableError::InvalidArchive(format!(
            "bundle entry path is not normalized portable ASCII: {path:?}"
        )));
    }
    for component in path.split('/') {
        if component.is_empty()
            || matches!(component, "." | "..")
            || component.ends_with('.')
            || is_windows_reserved_component(component)
            || !component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(PortableError::InvalidArchive(format!(
                "bundle entry path contains an unsafe component: {path:?}"
            )));
        }
    }
    Ok(())
}

fn is_windows_reserved_component(component: &str) -> bool {
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _)| stem)
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|suffix| matches!(suffix.as_bytes(), [b'1'..=b'9']))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == SHA256_PREFIX.len() + 64
        && value.starts_with(SHA256_PREFIX)
        && value[SHA256_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn canonical_manifest_bytes(manifest: &BundleManifest) -> Result<Vec<u8>, PortableError> {
    serde_json::to_vec(manifest).map_err(|error| {
        PortableError::InvalidManifest(format!("cannot serialize bundle manifest: {error}"))
    })
}

fn preflight_file_budgets(
    files: &[(String, Vec<u8>)],
    limits: BundleLimits,
) -> Result<(), PortableError> {
    let mut total = 0_u64;
    for (path, bytes) in files {
        validate_relative_path(path, limits.max_path_bytes)?;
        let length = u64::try_from(bytes.len())
            .map_err(|_| PortableError::BudgetExceeded("entry length overflow".to_owned()))?;
        if length > limits.max_entry_compressed_bytes || length > limits.max_entry_expanded_bytes {
            return Err(PortableError::BudgetExceeded(format!(
                "bundle entry exceeds per-entry budget: {path}"
            )));
        }
        total = total.checked_add(length).ok_or_else(|| {
            PortableError::BudgetExceeded("bundle total length overflow".to_owned())
        })?;
    }
    if total > limits.max_total_compressed_bytes || total > limits.max_total_expanded_bytes {
        return Err(PortableError::BudgetExceeded(
            "bundle entries exceed total byte budget".to_owned(),
        ));
    }
    Ok(())
}

fn canonical_archive_length(files: &[(String, Vec<u8>)]) -> Result<u64, PortableError> {
    let mut length = u64::try_from(EOCD_LENGTH)
        .map_err(|_| PortableError::BudgetExceeded("ZIP footer length overflow".to_owned()))?;
    for (path, bytes) in files {
        let path_length = u64::try_from(path.len())
            .map_err(|_| PortableError::BudgetExceeded("entry path length overflow".to_owned()))?;
        let data_length = u64::try_from(bytes.len())
            .map_err(|_| PortableError::BudgetExceeded("entry length overflow".to_owned()))?;
        let headers = u64::try_from(LOCAL_HEADER_LENGTH + CENTRAL_HEADER_LENGTH)
            .map_err(|_| PortableError::BudgetExceeded("ZIP header length overflow".to_owned()))?;
        length = length
            .checked_add(headers)
            .and_then(|value| value.checked_add(path_length.checked_mul(2)?))
            .and_then(|value| value.checked_add(data_length))
            .ok_or_else(|| {
                PortableError::BudgetExceeded("canonical archive length overflow".to_owned())
            })?;
    }
    if length > u64::from(u32::MAX) {
        return Err(PortableError::BudgetExceeded(
            "canonical bundle exceeds non-ZIP64 offset capacity".to_owned(),
        ));
    }
    Ok(length)
}

fn canonical_file_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644)
        .system(System::Unix)
        .large_file(false)
}

fn write_canonical_zip(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, PortableError> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    for (path, bytes) in files {
        writer
            .start_file(path, canonical_file_options())
            .map_err(|error| {
                PortableError::InvalidArchive(format!("cannot start bundle entry: {error}"))
            })?;
        writer.write_all(bytes).map_err(|error| {
            PortableError::InvalidArchive(format!("cannot write bundle entry: {error}"))
        })?;
    }
    writer.finish().map(Cursor::into_inner).map_err(|error| {
        PortableError::InvalidArchive(format!("cannot finish bundle archive: {error}"))
    })
}

#[derive(Debug)]
struct CentralEntry {
    path: String,
    compressed_size: u64,
    expanded_size: u64,
    crc32: u32,
    local_offset: usize,
}

fn scan_canonical_central_directory(
    bytes: &[u8],
    limits: BundleLimits,
) -> Result<Vec<CentralEntry>, PortableError> {
    let bounds = canonical_central_bounds(bytes, limits)?;
    let mut position = bounds.offset;
    let mut entries = Vec::with_capacity(bounds.entry_count);
    let mut exact_paths = BTreeSet::new();
    let mut folded_paths = BTreeSet::new();
    let mut total_compressed = 0_u64;
    let mut total_expanded = 0_u64;
    for _ in 0..bounds.entry_count {
        let (entry, next_position) = parse_central_entry(bytes, position, limits)?;
        if !exact_paths.insert(entry.path.clone()) {
            return Err(PortableError::InvalidArchive(format!(
                "duplicate central-directory path: {}",
                entry.path
            )));
        }
        if !folded_paths.insert(entry.path.to_ascii_lowercase()) {
            return Err(PortableError::InvalidArchive(format!(
                "central-directory path collides under ASCII casefold: {}",
                entry.path
            )));
        }
        total_compressed = total_compressed
            .checked_add(entry.compressed_size)
            .ok_or_else(|| PortableError::BudgetExceeded("compressed total overflow".to_owned()))?;
        total_expanded = total_expanded
            .checked_add(entry.expanded_size)
            .ok_or_else(|| PortableError::BudgetExceeded("expanded total overflow".to_owned()))?;
        entries.push(entry);
        position = next_position;
    }
    if position != bounds.end {
        return Err(PortableError::InvalidArchive(
            "central directory has trailing or missing records".to_owned(),
        ));
    }
    if total_compressed > limits.max_total_compressed_bytes
        || total_expanded > limits.max_total_expanded_bytes
    {
        return Err(PortableError::BudgetExceeded(
            "bundle entries exceed total byte budget".to_owned(),
        ));
    }
    validate_local_headers(bytes, &entries, bounds.offset)?;
    Ok(entries)
}

#[derive(Clone, Copy, Debug)]
struct CentralDirectoryBounds {
    offset: usize,
    end: usize,
    entry_count: usize,
}

fn canonical_central_bounds(
    bytes: &[u8],
    limits: BundleLimits,
) -> Result<CentralDirectoryBounds, PortableError> {
    if bytes.len() < EOCD_LENGTH {
        return Err(PortableError::InvalidArchive(
            "bundle is truncated before the ZIP footer".to_owned(),
        ));
    }
    let eocd = bytes.len() - EOCD_LENGTH;
    if read_u32(bytes, eocd)? != EOCD_SIGNATURE {
        return Err(PortableError::InvalidArchive(
            "bundle lacks a canonical terminal ZIP footer".to_owned(),
        ));
    }
    let disk = read_u16(bytes, eocd + 4)?;
    let central_disk = read_u16(bytes, eocd + 6)?;
    let entries_on_disk = read_u16(bytes, eocd + 8)?;
    let entry_count = read_u16(bytes, eocd + 10)?;
    let central_size = read_u32(bytes, eocd + 12)?;
    let central_offset = read_u32(bytes, eocd + 16)?;
    let comment_length = read_u16(bytes, eocd + 20)?;
    if disk != 0 || central_disk != 0 || entries_on_disk != entry_count || comment_length != 0 {
        return Err(PortableError::InvalidArchive(
            "bundle must be a single-disk ZIP with no archive comment".to_owned(),
        ));
    }
    if entry_count == u16::MAX || central_size == u32::MAX || central_offset == u32::MAX {
        return Err(PortableError::InvalidArchive(
            "ZIP64 bundle structures are not canonical".to_owned(),
        ));
    }
    let entry_count = usize::from(entry_count);
    if entry_count > limits.max_archive_entries {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle has {entry_count} entries; limit is {}",
            limits.max_archive_entries
        )));
    }
    if entry_count < 2 {
        return Err(PortableError::InvalidArchive(
            "bundle must contain both manifest entries".to_owned(),
        ));
    }
    let central_offset = usize::try_from(central_offset).map_err(|_| {
        PortableError::InvalidArchive("central directory offset does not fit memory".to_owned())
    })?;
    let central_size = usize::try_from(central_size).map_err(|_| {
        PortableError::InvalidArchive("central directory size does not fit memory".to_owned())
    })?;
    let central_end = central_offset.checked_add(central_size).ok_or_else(|| {
        PortableError::InvalidArchive("central directory range overflow".to_owned())
    })?;
    if central_end != eocd {
        return Err(PortableError::InvalidArchive(
            "central directory is not contiguous with the terminal footer".to_owned(),
        ));
    }
    Ok(CentralDirectoryBounds {
        offset: central_offset,
        end: central_end,
        entry_count,
    })
}

#[derive(Clone, Copy, Debug)]
struct CentralMetadata {
    made_by: u16,
    version_needed: u16,
    flags: u16,
    method: u16,
    modified_time: u16,
    modified_date: u16,
    extra_length: usize,
    comment_length: usize,
    starting_disk: u16,
    internal_attributes: u16,
    external_attributes: u32,
}

fn parse_central_entry(
    bytes: &[u8],
    position: usize,
    limits: BundleLimits,
) -> Result<(CentralEntry, usize), PortableError> {
    require_range(bytes, position, CENTRAL_HEADER_LENGTH)?;
    if read_u32(bytes, position)? != CENTRAL_SIGNATURE {
        return Err(PortableError::InvalidArchive(
            "malformed central directory entry".to_owned(),
        ));
    }
    let metadata = CentralMetadata {
        made_by: read_u16(bytes, position + 4)?,
        version_needed: read_u16(bytes, position + 6)?,
        flags: read_u16(bytes, position + 8)?,
        method: read_u16(bytes, position + 10)?,
        modified_time: read_u16(bytes, position + 12)?,
        modified_date: read_u16(bytes, position + 14)?,
        extra_length: usize::from(read_u16(bytes, position + 30)?),
        comment_length: usize::from(read_u16(bytes, position + 32)?),
        starting_disk: read_u16(bytes, position + 34)?,
        internal_attributes: read_u16(bytes, position + 36)?,
        external_attributes: read_u32(bytes, position + 38)?,
    };
    let crc32 = read_u32(bytes, position + 16)?;
    let compressed_size = u64::from(read_u32(bytes, position + 20)?);
    let expanded_size = u64::from(read_u32(bytes, position + 24)?);
    let name_length = usize::from(read_u16(bytes, position + 28)?);
    let local_offset = usize::try_from(read_u32(bytes, position + 42)?).map_err(|_| {
        PortableError::InvalidArchive("local header offset does not fit memory".to_owned())
    })?;
    let variable_length = name_length
        .checked_add(metadata.extra_length)
        .and_then(|value| value.checked_add(metadata.comment_length))
        .ok_or_else(|| PortableError::InvalidArchive("central entry length overflow".to_owned()))?;
    let name_start = position + CENTRAL_HEADER_LENGTH;
    require_range(bytes, name_start, variable_length)?;
    let raw_name = &bytes[name_start..name_start + name_length];
    if !raw_name.is_ascii() {
        return Err(PortableError::InvalidArchive(
            "bundle entry names must be ASCII".to_owned(),
        ));
    }
    let path = std::str::from_utf8(raw_name)
        .map_err(|_| PortableError::InvalidArchive("invalid entry name".to_owned()))?
        .to_owned();
    validate_relative_path(&path, limits.max_path_bytes)?;
    validate_central_metadata(&path, metadata)?;
    if compressed_size != expanded_size {
        return Err(PortableError::InvalidArchive(format!(
            "Stored bundle entry size fields differ: {path}"
        )));
    }
    check_entry_budgets(&path, compressed_size, expanded_size, limits)?;
    let next_position = name_start
        .checked_add(variable_length)
        .ok_or_else(|| PortableError::InvalidArchive("central position overflow".to_owned()))?;
    Ok((
        CentralEntry {
            path,
            compressed_size,
            expanded_size,
            crc32,
            local_offset,
        },
        next_position,
    ))
}

fn validate_central_metadata(path: &str, metadata: CentralMetadata) -> Result<(), PortableError> {
    let expected_made_by = (u16::from(u8::from(System::Unix)) << 8) | STORED_VERSION_NEEDED;
    let unix_mode = metadata.external_attributes >> 16;
    if metadata.made_by != expected_made_by
        || metadata.version_needed != STORED_VERSION_NEEDED
        || unix_mode != UNIX_REGULAR_0644
        || metadata.external_attributes & 0xffff != 0
        || metadata.internal_attributes != 0
        || metadata.starting_disk != 0
    {
        return Err(PortableError::InvalidArchive(format!(
            "bundle entry is not a canonical regular 0644 file: {path}"
        )));
    }
    if metadata.flags != 0
        || metadata.method != 0
        || metadata.modified_time != 0
        || metadata.modified_date != DOS_EPOCH_DATE
        || metadata.extra_length != 0
        || metadata.comment_length != 0
    {
        return Err(PortableError::InvalidArchive(format!(
            "bundle entry metadata is not canonical: {path}"
        )));
    }
    Ok(())
}

fn check_entry_budgets(
    path: &str,
    compressed: u64,
    expanded: u64,
    limits: BundleLimits,
) -> Result<(), PortableError> {
    if compressed > limits.max_entry_compressed_bytes || expanded > limits.max_entry_expanded_bytes
    {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle entry exceeds per-entry budget: {path}"
        )));
    }
    Ok(())
}

fn validate_local_headers(
    bytes: &[u8],
    entries: &[CentralEntry],
    central_offset: usize,
) -> Result<(), PortableError> {
    for (index, entry) in entries.iter().enumerate() {
        let position = entry.local_offset;
        require_range(bytes, position, LOCAL_HEADER_LENGTH)?;
        if read_u32(bytes, position)? != LOCAL_SIGNATURE {
            return Err(PortableError::InvalidArchive(format!(
                "bundle entry lacks a local header: {}",
                entry.path
            )));
        }
        let version_needed = read_u16(bytes, position + 4)?;
        let flags = read_u16(bytes, position + 6)?;
        let method = read_u16(bytes, position + 8)?;
        let modified_time = read_u16(bytes, position + 10)?;
        let modified_date = read_u16(bytes, position + 12)?;
        let crc32 = read_u32(bytes, position + 14)?;
        let compressed_size = u64::from(read_u32(bytes, position + 18)?);
        let expanded_size = u64::from(read_u32(bytes, position + 22)?);
        let name_length = usize::from(read_u16(bytes, position + 26)?);
        let extra_length = usize::from(read_u16(bytes, position + 28)?);
        let name_start = position + LOCAL_HEADER_LENGTH;
        require_range(bytes, name_start, name_length + extra_length)?;
        let name_end = name_start + name_length;
        if &bytes[name_start..name_end] != entry.path.as_bytes()
            || version_needed != STORED_VERSION_NEEDED
            || flags != 0
            || method != 0
            || modified_time != 0
            || modified_date != DOS_EPOCH_DATE
            || crc32 != entry.crc32
            || compressed_size != entry.compressed_size
            || expanded_size != entry.expanded_size
            || extra_length != 0
        {
            return Err(PortableError::InvalidArchive(format!(
                "local and central metadata differ: {}",
                entry.path
            )));
        }
        let compressed_size = usize::try_from(entry.compressed_size).map_err(|_| {
            PortableError::BudgetExceeded("entry size does not fit memory".to_owned())
        })?;
        let data_end = name_end.checked_add(compressed_size).ok_or_else(|| {
            PortableError::InvalidArchive("local entry range overflow".to_owned())
        })?;
        let expected_next = entries
            .get(index + 1)
            .map_or(central_offset, |next| next.local_offset);
        if data_end != expected_next || (index == 0 && position != 0) {
            return Err(PortableError::InvalidArchive(
                "bundle contains gaps, preamble, or unreferenced local data".to_owned(),
            ));
        }
    }
    Ok(())
}

fn read_validated_entries(
    bytes: &[u8],
    central: &[CentralEntry],
) -> Result<Vec<(String, Vec<u8>)>, PortableError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|error| {
        PortableError::InvalidArchive(format!("cannot parse bundle ZIP: {error}"))
    })?;
    if archive.len() != central.len() || !archive.comment().is_empty() {
        return Err(PortableError::InvalidArchive(
            "ZIP parser inventory differs from validated central directory".to_owned(),
        ));
    }
    let mut entries = Vec::with_capacity(central.len());
    for (index, expected) in central.iter().enumerate() {
        let mut file = archive.by_index(index).map_err(|error| {
            PortableError::InvalidArchive(format!("cannot open bundle entry: {error}"))
        })?;
        if file.name_raw() != expected.path.as_bytes()
            || file.compression() != CompressionMethod::Stored
            || file.encrypted()
            || file.unix_mode() != Some(UNIX_REGULAR_0644)
            || file.last_modified() != Some(DateTime::default())
            || !file.comment().is_empty()
            || file.extra_data().is_some_and(|data| !data.is_empty())
            || !file.is_file()
        {
            return Err(PortableError::InvalidArchive(format!(
                "parsed entry metadata is not canonical: {}",
                expected.path
            )));
        }
        let capacity = usize::try_from(expected.expanded_size).map_err(|_| {
            PortableError::BudgetExceeded("entry size does not fit memory".to_owned())
        })?;
        let mut content = Vec::with_capacity(capacity);
        file.read_to_end(&mut content).map_err(|error| {
            PortableError::InvalidArchive(format!("cannot read bundle entry: {error}"))
        })?;
        if content.len() != capacity {
            return Err(PortableError::Integrity(format!(
                "bundle entry size differs after read: {}",
                expected.path
            )));
        }
        entries.push((expected.path.clone(), content));
    }
    Ok(entries)
}

fn validate_raw_bundle(
    archive_bytes: &[u8],
    raw_entries: Vec<(String, Vec<u8>)>,
    limits: BundleLimits,
) -> Result<ValidatedBundlePlan, PortableError> {
    let physical_count = raw_entries.len();
    let mut raw_entries = raw_entries.into_iter();
    let (manifest_path, manifest_bytes) = raw_entries.next().ok_or_else(|| {
        PortableError::InvalidArchive("bundle is missing its canonical manifest".to_owned())
    })?;
    let (sidecar_path, sidecar_bytes) = raw_entries.next().ok_or_else(|| {
        PortableError::InvalidArchive("bundle is missing its manifest digest".to_owned())
    })?;
    if manifest_path != MANIFEST_PATH || sidecar_path != MANIFEST_DIGEST_PATH {
        return Err(PortableError::InvalidArchive(
            "manifest entries must be first and in canonical order".to_owned(),
        ));
    }
    let (manifest, manifest_sha256) = parse_canonical_manifest(&manifest_bytes, limits)?;
    let expected_sidecar = format!("{manifest_sha256}\n");
    if sidecar_bytes != expected_sidecar.as_bytes() {
        return Err(PortableError::Integrity(
            "manifest digest sidecar does not match canonical manifest bytes".to_owned(),
        ));
    }
    let expected_physical_count =
        manifest.entries.len().checked_add(2).ok_or_else(|| {
            PortableError::BudgetExceeded("manifest entry count overflow".to_owned())
        })?;
    if expected_physical_count != physical_count {
        return Err(PortableError::Integrity(
            "manifest entry inventory has missing or extra archive entries".to_owned(),
        ));
    }
    let entries = validate_manifest_payloads(&manifest.entries, raw_entries.collect(), limits)?;
    Ok(ValidatedBundlePlan {
        manifest,
        entries,
        archive_sha256: sha256_digest(archive_bytes),
        manifest_sha256,
    })
}

fn parse_canonical_manifest(
    manifest_bytes: &[u8],
    limits: BundleLimits,
) -> Result<(BundleManifest, String), PortableError> {
    let manifest_length = u64::try_from(manifest_bytes.len())
        .map_err(|_| PortableError::BudgetExceeded("manifest length overflow".to_owned()))?;
    if manifest_length > limits.max_manifest_bytes {
        return Err(PortableError::BudgetExceeded(format!(
            "bundle manifest exceeds {} bytes",
            limits.max_manifest_bytes
        )));
    }
    let manifest: BundleManifest = serde_json::from_slice(manifest_bytes).map_err(|error| {
        PortableError::InvalidManifest(format!("cannot parse bundle manifest: {error}"))
    })?;
    if manifest.schema_version != BUNDLE_SCHEMA_VERSION {
        return Err(PortableError::InvalidManifest(format!(
            "unsupported bundle schema_version: {}",
            manifest.schema_version
        )));
    }
    validate_source(&manifest.source)
        .map_err(|error| PortableError::InvalidManifest(error.to_string()))?;
    validate_scope(&manifest.scope)
        .map_err(|error| PortableError::InvalidManifest(error.to_string()))?;
    if canonical_manifest_bytes(&manifest)?.as_slice() != manifest_bytes {
        return Err(PortableError::InvalidManifest(
            "manifest JSON is not in canonical byte form".to_owned(),
        ));
    }
    let manifest_sha256 = sha256_digest(manifest_bytes);
    Ok((manifest, manifest_sha256))
}

fn validate_manifest_payloads(
    manifest_entries: &[BundleManifestEntry],
    raw_payloads: Vec<(String, Vec<u8>)>,
    limits: BundleLimits,
) -> Result<Vec<ValidatedBundleEntry>, PortableError> {
    let mut previous_path: Option<&str> = None;
    let mut folded = BTreeSet::new();
    let mut entries = Vec::with_capacity(manifest_entries.len());
    for (manifest_entry, (raw_path, raw_bytes)) in manifest_entries.iter().zip(raw_payloads) {
        validate_relative_path(&manifest_entry.path, limits.max_path_bytes)
            .map_err(|error| PortableError::InvalidManifest(error.to_string()))?;
        let Some(relative_path) = manifest_entry.path.strip_prefix(DATA_PREFIX) else {
            return Err(PortableError::InvalidManifest(
                "manifest payload path must begin with data/".to_owned(),
            ));
        };
        if relative_path.is_empty() || !manifest_entry.classification.is_portable() {
            return Err(PortableError::InvalidManifest(
                "manifest contains a non-portable payload classification".to_owned(),
            ));
        }
        if previous_path.is_some_and(|previous| previous >= manifest_entry.path.as_str()) {
            return Err(PortableError::InvalidManifest(
                "manifest payload entries must be strictly path-sorted".to_owned(),
            ));
        }
        previous_path = Some(&manifest_entry.path);
        if !folded.insert(manifest_entry.path.to_ascii_lowercase()) {
            return Err(PortableError::InvalidManifest(
                "manifest payload paths collide under ASCII casefold".to_owned(),
            ));
        }
        if raw_path != manifest_entry.path {
            return Err(PortableError::Integrity(
                "manifest path inventory differs from archive order".to_owned(),
            ));
        }
        let actual_length = u64::try_from(raw_bytes.len())
            .map_err(|_| PortableError::BudgetExceeded("entry length overflow".to_owned()))?;
        if manifest_entry.length != actual_length {
            return Err(PortableError::Integrity(format!(
                "manifest length mismatch: {}",
                manifest_entry.path
            )));
        }
        if !valid_sha256(&manifest_entry.sha256)
            || manifest_entry.sha256 != sha256_digest(&raw_bytes)
        {
            return Err(PortableError::Integrity(format!(
                "manifest digest mismatch: {}",
                manifest_entry.path
            )));
        }
        entries.push(ValidatedBundleEntry {
            archive_path: manifest_entry.path.clone(),
            relative_path: relative_path.to_owned(),
            classification: manifest_entry.classification,
            bytes: raw_bytes,
            sha256: manifest_entry.sha256.clone(),
        });
    }
    Ok(entries)
}

fn require_range(bytes: &[u8], offset: usize, length: usize) -> Result<(), PortableError> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| PortableError::InvalidArchive("ZIP byte range overflow".to_owned()))?;
    if end > bytes.len() {
        return Err(PortableError::InvalidArchive(
            "ZIP structure is truncated".to_owned(),
        ));
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, PortableError> {
    require_range(bytes, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, PortableError> {
    require_range(bytes, offset, 4)?;
    Ok(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: u8) -> String {
        format!("sha256:{}", char::from(byte).to_string().repeat(64))
    }

    fn request() -> BundleRequest {
        BundleRequest {
            source: BundleSourceIdentity {
                kind: BundleSourceKind::Project,
                id: "project-alpha".to_owned(),
                logical_digest: digest(b'a'),
            },
            scope: BundleScope::Project {
                id: "project-alpha".to_owned(),
            },
            entries: vec![
                BundleEntryInput {
                    relative_path: ".hive/knowledge/Wiki/gamma.md".to_owned(),
                    bytes: b"gamma canonical\n".to_vec(),
                    classification: PortableEntryClassification::CanonicalMarkdown,
                },
                BundleEntryInput {
                    relative_path: ".hive/config/project.yml".to_owned(),
                    bytes: b"schema_version: 1\n".to_vec(),
                    classification: PortableEntryClassification::PortableMetadata,
                },
                BundleEntryInput {
                    relative_path: ".hive/knowledge/Wiki/alpha.md".to_owned(),
                    bytes: b"alpha canonical\n".to_vec(),
                    classification: PortableEntryClassification::CanonicalMarkdown,
                },
            ],
        }
    }

    fn rewrite_archive(manifest: &BundleManifest, payload: &[(String, Vec<u8>)]) -> Vec<u8> {
        let manifest_bytes = canonical_manifest_bytes(manifest).expect("manifest");
        let sidecar = format!("{}\n", sha256_digest(&manifest_bytes)).into_bytes();
        let mut files = vec![
            (MANIFEST_PATH.to_owned(), manifest_bytes),
            (MANIFEST_DIGEST_PATH.to_owned(), sidecar),
        ];
        files.extend_from_slice(payload);
        write_canonical_zip(&files).expect("ZIP")
    }

    fn payload_from(encoded: &EncodedBundle) -> Vec<(String, Vec<u8>)> {
        encoded
            .plan()
            .entries()
            .iter()
            .map(|entry| (entry.archive_path().to_owned(), entry.bytes().to_vec()))
            .collect()
    }

    fn patch_second_payload_name(archive: &mut [u8], replacement: &[u8]) {
        let central = scan_canonical_central_directory(archive, BundleLimits::default())
            .expect("valid central directory");
        let target = &central[4];
        assert_eq!(target.path.len(), replacement.len());
        let central_start = archive
            .windows(target.path.len())
            .rposition(|window| window == target.path.as_bytes())
            .expect("central filename");
        archive[central_start..central_start + replacement.len()].copy_from_slice(replacement);
        let local_start = target.local_offset + LOCAL_HEADER_LENGTH;
        archive[local_start..local_start + replacement.len()].copy_from_slice(replacement);
    }

    fn central_header_offset(archive: &[u8], entry: &CentralEntry) -> usize {
        archive
            .windows(entry.path.len())
            .rposition(|window| window == entry.path.as_bytes())
            .expect("central filename")
            - CENTRAL_HEADER_LENGTH
    }

    #[test]
    fn deterministic_round_trip_is_sorted_and_byte_identical() {
        let first = encode_bundle(&request(), BundleLimits::default()).expect("first bundle");
        let mut reordered = request();
        reordered.entries.reverse();
        let second = encode_bundle(&reordered, BundleLimits::default()).expect("second bundle");
        assert_eq!(first.archive(), second.archive());
        assert_eq!(first.plan(), second.plan());
        assert_eq!(
            first
                .plan()
                .entries()
                .iter()
                .map(ValidatedBundleEntry::archive_path)
                .collect::<Vec<_>>(),
            vec![
                "data/.hive/config/project.yml",
                "data/.hive/knowledge/Wiki/alpha.md",
                "data/.hive/knowledge/Wiki/gamma.md",
            ]
        );
        let decoded = decode_bundle(first.archive(), BundleLimits::default()).expect("decode");
        assert_eq!(&decoded, first.plan());
        assert_eq!(decoded.archive_sha256(), sha256_digest(first.archive()));

        let mut empty = request();
        empty.entries.clear();
        let empty_bundle = encode_bundle(&empty, BundleLimits::default()).expect("empty bundle");
        assert!(empty_bundle.plan().entries().is_empty());
        assert_eq!(
            decode_bundle(empty_bundle.archive(), BundleLimits::default())
                .expect("decode empty bundle"),
            *empty_bundle.plan()
        );
    }

    #[test]
    fn typed_nonportable_entries_are_omitted_without_path_or_byte_leakage() {
        let mut input = request();
        input.entries.extend([
            BundleEntryInput {
                relative_path: "C:\\private\\hive.sqlite3".to_owned(),
                bytes: b"sqlite-secret-canary".to_vec(),
                classification: PortableEntryClassification::DerivedSqlite,
            },
            BundleEntryInput {
                relative_path: "/runtime/session.json".to_owned(),
                bytes: b"runtime-secret-canary".to_vec(),
                classification: PortableEntryClassification::RuntimeState,
            },
            BundleEntryInput {
                relative_path: "../confidential.md".to_owned(),
                bytes: b"confidential-secret-canary".to_vec(),
                classification: PortableEntryClassification::Confidential,
            },
            BundleEntryInput {
                relative_path: "D:\\machine-bound\\source.md".to_owned(),
                bytes: b"absolute-path-canary".to_vec(),
                classification: PortableEntryClassification::AbsolutePath,
            },
            BundleEntryInput {
                relative_path: "secrets/provider.token".to_owned(),
                bytes: b"credential-canary".to_vec(),
                classification: PortableEntryClassification::Credential,
            },
        ]);
        let encoded = encode_bundle(&input, BundleLimits::default()).expect("filtered bundle");
        assert_eq!(
            encoded.exclusions(),
            BundleExclusionCounts {
                derived_sqlite: 1,
                runtime_state: 1,
                absolute_path: 1,
                credential: 1,
                confidential: 1,
            }
        );
        for canary in [
            b"sqlite-secret-canary".as_slice(),
            b"runtime-secret-canary".as_slice(),
            b"confidential-secret-canary".as_slice(),
            b"absolute-path-canary".as_slice(),
            b"credential-canary".as_slice(),
        ] {
            assert!(!encoded
                .archive()
                .windows(canary.len())
                .any(|value| value == canary));
        }
    }

    #[test]
    fn encoder_rejects_unsafe_ascii_unicode_and_casefold_paths() {
        for path in [
            "/absolute.md",
            "C:/drive.md",
            "//server/share.md",
            "..\\escape.md",
            "a/../escape.md",
            "a//empty.md",
            "a/./dot.md",
            "knowledge/café.md",
            "knowledge/CON.md",
            "knowledge/nul",
            "knowledge/COM1.txt",
            "knowledge/trailing.",
        ] {
            let mut input = request();
            input.entries[0].relative_path = path.to_owned();
            assert!(
                encode_bundle(&input, BundleLimits::default()).is_err(),
                "{path}"
            );
        }

        let mut collision = request();
        collision.entries = vec![
            BundleEntryInput {
                relative_path: "Wiki/Alpha.md".to_owned(),
                bytes: b"one".to_vec(),
                classification: PortableEntryClassification::CanonicalMarkdown,
            },
            BundleEntryInput {
                relative_path: "wiki/alpha.md".to_owned(),
                bytes: b"two".to_vec(),
                classification: PortableEntryClassification::CanonicalMarkdown,
            },
        ];
        assert!(matches!(
            encode_bundle(&collision, BundleLimits::default()),
            Err(PortableError::InvalidInput(_))
        ));

        let mut duplicate = request();
        duplicate.entries.push(duplicate.entries[0].clone());
        assert!(matches!(
            encode_bundle(&duplicate, BundleLimits::default()),
            Err(PortableError::InvalidInput(message)) if message.contains("duplicate")
        ));
    }

    #[test]
    fn decoder_rejects_duplicate_and_casefold_central_names_before_manifest_read() {
        let encoded = encode_bundle(&request(), BundleLimits::default()).expect("bundle");
        let mut duplicate = encoded.archive().to_vec();
        patch_second_payload_name(&mut duplicate, b"data/.hive/knowledge/Wiki/alpha.md");
        assert!(matches!(
            decode_bundle(&duplicate, BundleLimits::default()),
            Err(PortableError::InvalidArchive(message)) if message.contains("duplicate")
        ));

        let mut folded = encoded.archive().to_vec();
        patch_second_payload_name(&mut folded, b"data/.hive/knowledge/Wiki/ALPHA.md");
        assert!(matches!(
            decode_bundle(&folded, BundleLimits::default()),
            Err(PortableError::InvalidArchive(message)) if message.contains("casefold")
        ));
    }

    #[test]
    fn decoder_rejects_noncanonical_paths_modes_and_compression() {
        for path in [
            "../escape",
            "/absolute",
            "C:/drive",
            "a\\backslash",
            "café",
            "data/NUL.txt",
            "data/trailing.",
        ] {
            let archive = write_canonical_zip(&[
                (MANIFEST_PATH.to_owned(), b"{}".to_vec()),
                (path.to_owned(), b"x".to_vec()),
            ])
            .expect("ZIP");
            assert!(
                decode_bundle(&archive, BundleLimits::default()).is_err(),
                "{path}"
            );
        }

        let encoded = encode_bundle(&request(), BundleLimits::default()).expect("bundle");
        let central = scan_canonical_central_directory(encoded.archive(), BundleLimits::default())
            .expect("central");
        let mut symlink = encoded.archive().to_vec();
        let attributes = 0o120_777_u32 << 16;
        let attribute_offset = central_header_offset(encoded.archive(), &central[2]) + 38;
        symlink[attribute_offset..attribute_offset + 4].copy_from_slice(&attributes.to_le_bytes());
        assert!(matches!(
            decode_bundle(&symlink, BundleLimits::default()),
            Err(PortableError::InvalidArchive(message)) if message.contains("regular 0644")
        ));

        let mut compressed = encoded.archive().to_vec();
        let method_offset = central_header_offset(encoded.archive(), &central[2]) + 10;
        compressed[method_offset..method_offset + 2].copy_from_slice(&8_u16.to_le_bytes());
        assert!(matches!(
            decode_bundle(&compressed, BundleLimits::default()),
            Err(PortableError::InvalidArchive(message)) if message.contains("metadata")
        ));

        let mut hardlink = encoded.archive().to_vec();
        let local_offset = u32::try_from(central[2].local_offset).expect("32-bit local offset");
        let hardlink_offset = central_header_offset(encoded.archive(), &central[3]) + 42;
        hardlink[hardlink_offset..hardlink_offset + 4].copy_from_slice(&local_offset.to_le_bytes());
        assert!(matches!(
            decode_bundle(&hardlink, BundleLimits::default()),
            Err(PortableError::InvalidArchive(message)) if message.contains("gaps")
        ));
    }

    #[test]
    fn manifest_sidecar_inventory_length_and_digest_are_bound() {
        let encoded = encode_bundle(&request(), BundleLimits::default()).expect("bundle");
        let payload = payload_from(&encoded);

        let mut wrong_length = encoded.plan().manifest().clone();
        wrong_length.entries[0].length += 1;
        let archive = rewrite_archive(&wrong_length, &payload);
        assert!(matches!(
            decode_bundle(&archive, BundleLimits::default()),
            Err(PortableError::Integrity(message)) if message.contains("length")
        ));

        let mut wrong_digest = encoded.plan().manifest().clone();
        wrong_digest.entries[0].sha256 = digest(b'b');
        let archive = rewrite_archive(&wrong_digest, &payload);
        assert!(matches!(
            decode_bundle(&archive, BundleLimits::default()),
            Err(PortableError::Integrity(message)) if message.contains("digest")
        ));

        let mut extra_payload = payload.clone();
        extra_payload.push(("data/extra.md".to_owned(), b"extra".to_vec()));
        let archive = rewrite_archive(encoded.plan().manifest(), &extra_payload);
        assert!(matches!(
            decode_bundle(&archive, BundleLimits::default()),
            Err(PortableError::Integrity(message)) if message.contains("missing or extra")
        ));

        let missing_manifest = write_canonical_zip(&[
            (MANIFEST_DIGEST_PATH.to_owned(), b"sha256:none\n".to_vec()),
            ("data/orphan.md".to_owned(), b"orphan".to_vec()),
        ])
        .expect("ZIP without manifest");
        assert!(matches!(
            decode_bundle(&missing_manifest, BundleLimits::default()),
            Err(PortableError::InvalidArchive(message)) if message.contains("manifest entries")
        ));

        let mut extra_manifest = payload.clone();
        extra_manifest.push(("manifest-copy.json".to_owned(), b"{}".to_vec()));
        let extra_manifest_archive = rewrite_archive(encoded.plan().manifest(), &extra_manifest);
        assert!(matches!(
            decode_bundle(&extra_manifest_archive, BundleLimits::default()),
            Err(PortableError::Integrity(message)) if message.contains("missing or extra")
        ));

        let missing_payload = &payload[..payload.len() - 1];
        let archive = rewrite_archive(encoded.plan().manifest(), missing_payload);
        assert!(matches!(
            decode_bundle(&archive, BundleLimits::default()),
            Err(PortableError::Integrity(message)) if message.contains("missing or extra")
        ));

        let mut sidecar = encoded.archive().to_vec();
        let needle = encoded.plan().manifest_sha256().as_bytes();
        let position = sidecar
            .windows(needle.len())
            .position(|window| window == needle)
            .expect("sidecar digest");
        sidecar[position + SHA256_PREFIX.len()] = b'0';
        assert!(decode_bundle(&sidecar, BundleLimits::default()).is_err());
    }

    #[test]
    fn decoder_rejects_entry_count_per_entry_total_and_archive_budgets() {
        let encoded = encode_bundle(&request(), BundleLimits::default()).expect("bundle");

        let entry_count = BundleLimits {
            max_archive_entries: 2,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), entry_count),
            Err(PortableError::BudgetExceeded(_))
        ));

        let compressed_entry = BundleLimits {
            max_entry_compressed_bytes: 8,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), compressed_entry),
            Err(PortableError::BudgetExceeded(_))
        ));

        let expanded_entry = BundleLimits {
            max_entry_expanded_bytes: 8,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), expanded_entry),
            Err(PortableError::BudgetExceeded(_))
        ));

        let compressed_total = BundleLimits {
            max_total_compressed_bytes: 32,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), compressed_total),
            Err(PortableError::BudgetExceeded(_))
        ));

        let expanded_total = BundleLimits {
            max_total_expanded_bytes: 32,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), expanded_total),
            Err(PortableError::BudgetExceeded(_))
        ));

        let archive_limit = BundleLimits {
            max_archive_bytes: 32,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), archive_limit),
            Err(PortableError::BudgetExceeded(_))
        ));

        let manifest_limit = BundleLimits {
            max_manifest_bytes: 16,
            ..BundleLimits::default()
        };
        assert!(matches!(
            decode_bundle(encoded.archive(), manifest_limit),
            Err(PortableError::BudgetExceeded(_))
        ));

        assert!(matches!(
            encode_bundle(&request(), compressed_entry),
            Err(PortableError::BudgetExceeded(_))
        ));
        assert!(matches!(
            encode_bundle(&request(), entry_count),
            Err(PortableError::BudgetExceeded(_))
        ));
        assert!(matches!(
            encode_bundle(&request(), archive_limit),
            Err(PortableError::BudgetExceeded(_))
        ));
    }

    #[test]
    fn malformed_truncated_and_noncanonical_manifest_archives_fail_closed() {
        for bytes in [b"not-a-zip".as_slice(), b"PK\x03\x04".as_slice()] {
            assert!(decode_bundle(bytes, BundleLimits::default()).is_err());
        }
        let encoded = encode_bundle(&request(), BundleLimits::default()).expect("bundle");
        for removed in [1, 8, 22] {
            let truncated = &encoded.archive()[..encoded.archive().len() - removed];
            assert!(decode_bundle(truncated, BundleLimits::default()).is_err());
        }

        let payload = payload_from(&encoded);
        let pretty = serde_json::to_vec_pretty(encoded.plan().manifest()).expect("pretty manifest");
        let sidecar = format!("{}\n", sha256_digest(&pretty)).into_bytes();
        let mut files = vec![
            (MANIFEST_PATH.to_owned(), pretty),
            (MANIFEST_DIGEST_PATH.to_owned(), sidecar),
        ];
        files.extend(payload);
        let archive = write_canonical_zip(&files).expect("ZIP");
        assert!(matches!(
            decode_bundle(&archive, BundleLimits::default()),
            Err(PortableError::InvalidManifest(message)) if message.contains("canonical")
        ));
    }
}
