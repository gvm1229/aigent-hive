//! Canonical collection export and import orchestration for portable `.hivekb` bundles.
//!
//! Archives contain only logical collection metadata and canonical bytes. Machine-local
//! attachment paths, `SQLite` projections, runtime state, credentials, and confidential
//! content never cross the bundle boundary. Import validates the complete merge before
//! changing canonical state, then publishes canonical files and the rebuilt disposable
//! projection as one rollback-capable transaction.

use crate::bundle_io::{
    encode_and_publish_bundle, load_bundle, BundleIoError, BundlePublishMode, BundlePublishOutcome,
};
use crate::collection::{
    folded_alias, CollectionKind, CollectionRecord, CollectionRegistry, CollectionResolution,
    CollectionState, CollectionVisibility, COLLECTION_SCHEMA_VERSION, USER_ROOT_COLLECTION_ID,
};
use crate::portable::{
    encode_bundle, BundleEntryInput, BundleLimits, BundleRequest, BundleScope,
    BundleSourceIdentity, BundleSourceKind, PortableEntryClassification, ValidatedBundlePlan,
};
use crate::rag::{
    build_rag_index, parse_claim_markdown, GenerationManifest, RagVisibility,
    MAX_SERIALIZED_INDEX_BYTES, RAG_SCHEMA_VERSION,
};
use crate::shared::SHARED_INDEX_RELATIVE;
use crate::store::{
    rag_trust_bytes_for_manifest, verify_rag_trust_bytes, PersistentDirtyEntry,
    PersistentDirtyState, RagStore, CLAIMS_RELATIVE, COLLECTION_REGISTRY_RELATIVE,
    MAX_RAG_TRUST_BYTES, RAG_DIRTY_RELATIVE, RAG_MANIFEST_RELATIVE, RAG_TRUST_RELATIVE,
};
use crate::{parse_page_bytes, WikiError};
use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// Portable logical collection inventory carried by every Hive bundle.
pub const PORTABLE_REGISTRY_PATH: &str = ".hive/portable/collections.json";
/// User-root-owned dormant storage for Wiki bytes belonging to detached imports.
pub const DETACHED_IMPORT_ROOT: &str = ".hive/knowledge/Imported";

const PORTABLE_SCHEMA_VERSION: u32 = 1;
const USER_SETUP_RELATIVE: &str = ".hive/config/user-setup.yml";
const WIKI_RELATIVE: &str = ".hive/knowledge/Wiki";
const MERGE_RELATIVE: &str = ".hive/knowledge/Merge";
const SUPPRESSION_RELATIVE: &str = ".hive/knowledge/suppression.yml";
const MAX_REGISTRY_BYTES: usize = 1024 * 1024;
const MAX_USER_SETUP_BYTES: usize = 1024 * 1024;
const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_CLAIM_BYTES: usize = 128 * 1024;
const MAX_WIKI_BYTES: usize = 2 * 1024 * 1024;
const MAX_SUPPRESSION_BYTES: usize = 1024 * 1024;
const MAX_MERGE_PROVENANCE_BYTES: usize = 3 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 128 * 1024;
const MAX_DIRTY_BYTES: usize = 1024 * 1024;
const MAX_COLLECTIONS: usize = 10_000;
const MAX_ALIASES: usize = 256;
const MAX_CHANGED_PATH_BYTES: usize = 240;
const MAX_DIRECTORY_ENTRIES: usize = 10_000;
const MAX_EXPORT_CANDIDATE_BYTES: usize = 256 * 1024 * 1024;

/// Stable fixed tables in the rebuilt disposable collection projection.
pub const COLLECTION_TABLES: [&str; 7] = [
    "collections",
    "documents",
    "chunks",
    "claims",
    "sources",
    "links",
    "replacements",
];

/// Serializable publication disposition for an export command.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundleExportDisposition {
    /// A missing destination was created.
    Created,
    /// The destination already contained the exact deterministic archive.
    Unchanged,
    /// Different prior bytes were replaced under the caller's backup policy.
    Replaced,
}

/// Completed canonical bundle export.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleExportResult {
    /// Logical source identity committed into the manifest.
    pub source: BundleSourceIdentity,
    /// Exact export scope.
    pub scope: BundleScope,
    /// Number of portable payload entries in the archive.
    pub entry_count: usize,
    /// Canonical entries excluded because they appeared to contain credentials.
    pub credential_excluded_count: usize,
    /// Entries excluded because they carried machine-bound absolute locators.
    pub absolute_path_excluded_count: usize,
    /// Entries excluded by the confidential boundary.
    pub confidential_excluded_count: usize,
    /// Durable destination publication result.
    pub disposition: BundleExportDisposition,
    /// Digest of the exact complete archive bytes.
    pub archive_sha256: String,
    /// Exact archive byte length.
    pub byte_length: u64,
    /// Retained prior bundle when replacement was requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<PathBuf>,
}

/// Read-only receipt for an exact prospective portable bundle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleExportPreview {
    /// Logical source identity committed into the prospective manifest.
    pub source: BundleSourceIdentity,
    /// Exact export scope.
    pub scope: BundleScope,
    /// Number of portable payload entries in the archive.
    pub entry_count: usize,
    /// Canonical entries excluded because they appeared to contain credentials.
    pub credential_excluded_count: usize,
    /// Entries excluded because they carried machine-bound absolute locators.
    pub absolute_path_excluded_count: usize,
    /// Entries excluded by the confidential boundary.
    pub confidential_excluded_count: usize,
    /// Digest of the exact archive that a subsequent export would publish.
    pub archive_sha256: String,
    /// Exact prospective archive byte length.
    pub byte_length: u64,
}

/// Explicit import execution mode.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundleImportMode {
    /// Validate and report the exact merge without changing files.
    DryRun,
    /// Atomically activate the validated canonical merge and rebuild the index.
    Apply,
    /// Atomically apply only entries that did not conflict during the reviewed preview.
    ApplyExcludingConflicts,
}

/// Import disposition matching the public JSON contract.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundleImportDisposition {
    /// Every incoming truth was already present byte-for-byte.
    Noop,
    /// A dry-run found a safe non-overlapping merge.
    Planned,
    /// Canonical state and the disposable index were atomically activated.
    Applied,
}

/// Rollback evidence in an import result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleRollbackResult {
    /// Whether rollback was required.
    pub attempted: bool,
    /// Whether every changed path was restored.
    pub succeeded: bool,
    /// Digest of the captured pre-activation states.
    pub backup_digest: Option<String>,
    /// Logical paths restored by a successful rollback.
    pub restored_paths: Vec<String>,
}

impl BundleRollbackResult {
    fn none() -> Self {
        Self {
            attempted: false,
            succeeded: false,
            backup_digest: None,
            restored_paths: Vec::new(),
        }
    }
}

/// Deterministic dry-run or applied bundle import result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleImportResult {
    /// Requested execution mode.
    pub mode: BundleImportMode,
    /// Net merge disposition.
    pub disposition: BundleImportDisposition,
    /// Digest of the exact input archive.
    pub archive_sha256: String,
    /// Digest of the exact canonical manifest.
    pub manifest_sha256: String,
    /// Validated logical source.
    pub source: BundleSourceIdentity,
    /// Validated bundle scope.
    pub scope: BundleScope,
    /// Number of bundle payload entries, including portable registry metadata.
    pub entry_count: usize,
    /// Entries contributing new canonical truth.
    pub added_count: usize,
    /// Entries already present identically.
    pub unchanged_count: usize,
    /// Canonical logical paths excluded because destination bytes or claim identity conflicted.
    pub conflict_paths: Vec<String>,
    /// Digest of the destination bindings and canonical bytes observed during planning.
    pub target_state_digest: String,
    /// Imported collections still awaiting an explicit local attachment.
    pub detached_collection_ids: Vec<String>,
    /// Exact logical collections present in the validated bundle.
    pub collection_ids: Vec<String>,
    /// Final logical paths changed by a successful apply.
    pub changed_paths: Vec<String>,
    /// Whether canonical bytes changed.
    pub canonical_mutation: bool,
    /// Whether the disposable normalized index was rebuilt.
    pub index_rebuilt: bool,
    /// Rollback evidence.
    pub rollback: BundleRollbackResult,
    /// Stable normalized table inventory.
    pub collection_tables: Vec<String>,
}

/// One semantically identical-looking Wiki-page group that needs host review before collapse.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleMergeCandidate {
    /// Stable content-derived candidate identifier.
    pub candidate_id: String,
    /// Canonical portable paths participating in the candidate.
    pub paths: Vec<String>,
}

/// One selectable canonical variant for a same-path conflict.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleMergeVariant {
    /// Exact canonical payload digest.
    pub sha256: String,
}

/// A canonical path with two or more divergent validated payload variants.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleMergeConflict {
    /// Portable path requiring an explicit reviewed selection.
    pub path: String,
    /// Available exact payload variants in digest order.
    pub variants: Vec<BundleMergeVariant>,
}

/// Host-reviewed disposition for one semantic merge candidate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum BundleMergeDecision {
    /// Retain every candidate page as separate active knowledge.
    Separate {
        /// Candidate identifier from [`BundleMergeCandidate`].
        candidate_id: String,
    },
    /// Collapse byte-equivalent meaning into one active page and preserve other originals.
    Equivalent {
        /// Candidate identifier from [`BundleMergeCandidate`].
        candidate_id: String,
        /// One canonical portable Wiki path retained as active knowledge.
        primary_path: String,
    },
    /// Select one same-path canonical Wiki variant after a user-resolved conflict.
    Choose {
        /// Portable path from [`BundleMergeConflict`].
        path: String,
        /// SHA-256 of the selected canonical variant.
        selected_sha256: String,
    },
}

/// Read-only preparation result for a deterministic multi-bundle merge.
///
/// Exact duplicate payloads are represented only once in [`Self::request`]. Divergent paths,
/// claim identities, and collection truth remain conflicts; semantic candidates are deliberately
/// not collapsed until an active host supplies a reviewed decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleMergePreview {
    /// Exact input archive digests in canonical order.
    pub archive_sha256s: Vec<String>,
    /// Number of validated portable entries before exact de-duplication.
    pub input_entry_count: usize,
    /// Number of payload entries already represented byte-for-byte by another input.
    pub exact_duplicate_count: usize,
    /// Canonical entries that cannot be selected automatically.
    pub conflict_paths: Vec<String>,
    /// Selectable same-path payload variants. Other conflict types remain blocked.
    pub conflicts: Vec<BundleMergeConflict>,
    /// Same-fact candidates requiring host review.
    pub semantic_candidates: Vec<BundleMergeCandidate>,
    request: BundleRequest,
    conflicting_entries: BTreeMap<String, Vec<BundleEntryInput>>,
}

impl BundleMergePreview {
    /// Deterministic portable request containing one copy of every non-conflicting payload.
    #[must_use]
    pub fn request(&self) -> &BundleRequest {
        &self.request
    }

    /// Digest binding all input archives and the resulting canonical request.
    #[must_use]
    pub fn merge_digest(&self) -> String {
        let entries = self
            .request
            .entries
            .iter()
            .map(|entry| (&entry.relative_path, sha256_digest(&entry.bytes)))
            .collect::<Vec<_>>();
        sha256_digest(
            &serde_json::to_vec(&(
                "knowledge-transfer-merge-v1",
                &self.archive_sha256s,
                entries,
                &self.conflict_paths,
                &self.conflicts,
                &self.semantic_candidates,
            ))
            .expect("merge preview values serialize"),
        )
    }
}

/// Apply reviewed semantic decisions to a prepared merge request without filesystem effects.
///
/// Every candidate needs exactly one decision. An `equivalent` decision is accepted only for
/// exactly equal kind, summary, and body text; metadata is merged and each removed original is
/// carried as portable merge provenance outside the active Wiki search surface.
pub fn apply_bundle_merge_review(
    preview: &BundleMergePreview,
    decisions: &[BundleMergeDecision],
) -> Result<BundleRequest, WikiError> {
    let semantic_decisions = decisions
        .iter()
        .filter_map(|decision| match decision {
            BundleMergeDecision::Separate { candidate_id }
            | BundleMergeDecision::Equivalent { candidate_id, .. } => {
                Some((candidate_id, decision))
            }
            BundleMergeDecision::Choose { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    let conflict_decisions = decisions
        .iter()
        .filter_map(|decision| match decision {
            BundleMergeDecision::Choose { path, .. } => Some((path, decision)),
            BundleMergeDecision::Separate { .. } | BundleMergeDecision::Equivalent { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    if semantic_decisions.len() + conflict_decisions.len() != decisions.len()
        || semantic_decisions.len() != preview.semantic_candidates.len()
        || conflict_decisions.len() != preview.conflicts.len()
        || preview
            .semantic_candidates
            .iter()
            .any(|candidate| !semantic_decisions.contains_key(&candidate.candidate_id))
        || preview
            .conflicts
            .iter()
            .any(|conflict| !conflict_decisions.contains_key(&conflict.path))
    {
        return Err(WikiError::InvalidInput(
            "merge review must contain one decision for every candidate and selectable conflict"
                .to_owned(),
        ));
    }
    if preview.conflict_paths.iter().any(|path| {
        !preview
            .conflicts
            .iter()
            .any(|conflict| &conflict.path == path)
    }) {
        return Err(WikiError::Conflict(
            "merge has a non-selectable collection or claim conflict".to_owned(),
        ));
    }
    let mut entries = preview
        .request
        .entries
        .iter()
        .cloned()
        .map(|entry| (entry.relative_path.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    for conflict in &preview.conflicts {
        let Some(BundleMergeDecision::Choose {
            selected_sha256, ..
        }) = conflict_decisions.get(&conflict.path)
        else {
            unreachable!("validated conflict decision shape");
        };
        let variants = preview
            .conflicting_entries
            .get(&conflict.path)
            .ok_or_else(|| {
                WikiError::Verification("merge conflict variants are unavailable".to_owned())
            })?;
        let selected = variants
            .iter()
            .find(|variant| sha256_digest(&variant.bytes) == *selected_sha256)
            .ok_or_else(|| {
                WikiError::InvalidInput("selected merge variant is unavailable".to_owned())
            })?
            .clone();
        if selected.classification != PortableEntryClassification::CanonicalMarkdown {
            return Err(WikiError::Conflict(
                "only canonical Wiki conflicts are selectable in a merge review".to_owned(),
            ));
        }
        let (collection_id, leaf, Some(file_name)) = parse_portable_payload_path(&conflict.path)?
        else {
            return Err(WikiError::Verification(
                "merge conflict path is malformed".to_owned(),
            ));
        };
        if leaf != "Wiki" {
            return Err(WikiError::Conflict(
                "only portable Wiki conflicts are selectable in a merge review".to_owned(),
            ));
        }
        entries.insert(conflict.path.clone(), selected.clone());
        for variant in variants {
            if variant.bytes == selected.bytes {
                continue;
            }
            let original_markdown = String::from_utf8(variant.bytes.clone()).map_err(|error| {
                WikiError::Verification(format!("conflicting Wiki page is not UTF-8: {error}"))
            })?;
            parse_page_bytes(
                original_markdown.as_bytes(),
                &format!("{WIKI_RELATIVE}/{file_name}"),
            )?;
            let record = MergeProvenance {
                schema_version: 1,
                primary_path: conflict.path.clone(),
                original_path: conflict.path.clone(),
                original_sha256: sha256_digest(&variant.bytes),
                original_markdown,
            };
            let bytes = serde_json::to_vec(&record).map_err(|error| {
                WikiError::Io(format!("cannot serialize merge provenance: {error}"))
            })?;
            insert_merge_provenance(&mut entries, &collection_id, bytes);
        }
    }
    for candidate in &preview.semantic_candidates {
        let Some(BundleMergeDecision::Equivalent { primary_path, .. }) =
            semantic_decisions.get(&candidate.candidate_id)
        else {
            continue;
        };
        if !candidate.paths.contains(primary_path) {
            return Err(WikiError::InvalidInput(
                "merge review primary path is outside its candidate".to_owned(),
            ));
        }
        let primary = entries.get(primary_path).ok_or_else(|| {
            WikiError::Verification("merge review primary page is unavailable".to_owned())
        })?;
        let (_, _, Some(primary_file)) = parse_portable_payload_path(primary_path)? else {
            return Err(WikiError::Verification(
                "merge review primary path is malformed".to_owned(),
            ));
        };
        let mut primary_page =
            parse_page_bytes(&primary.bytes, &format!("{WIKI_RELATIVE}/{primary_file}"))?;
        let mut provenance = Vec::new();
        for path in &candidate.paths {
            if path == primary_path {
                continue;
            }
            let other = entries.get(path).ok_or_else(|| {
                WikiError::Verification("merge review candidate page is unavailable".to_owned())
            })?;
            let (_, _, Some(other_file)) = parse_portable_payload_path(path)? else {
                return Err(WikiError::Verification(
                    "merge review candidate path is malformed".to_owned(),
                ));
            };
            let other_page =
                parse_page_bytes(&other.bytes, &format!("{WIKI_RELATIVE}/{other_file}"))?;
            if primary_page.frontmatter.kind != other_page.frontmatter.kind
                || primary_page.frontmatter.summary.trim() != other_page.frontmatter.summary.trim()
                || primary_page.body.trim() != other_page.body.trim()
            {
                return Err(WikiError::Conflict(
                    "equivalent merge review changed semantic candidate bytes; keep separate"
                        .to_owned(),
                ));
            }
            merge_page_metadata(&mut primary_page.frontmatter, &other_page.frontmatter);
            let (collection_id, _, _) = parse_portable_payload_path(path)?;
            let record = MergeProvenance {
                schema_version: 1,
                primary_path: primary_path.clone(),
                original_path: path.clone(),
                original_sha256: sha256_digest(&other.bytes),
                original_markdown: String::from_utf8(other.bytes.clone()).map_err(|error| {
                    WikiError::Verification(format!("merged Wiki page is not UTF-8: {error}"))
                })?,
            };
            let bytes = serde_json::to_vec(&record).map_err(|error| {
                WikiError::Io(format!("cannot serialize merge provenance: {error}"))
            })?;
            provenance.push((collection_id, bytes));
        }
        let primary_bytes = render_merge_page(&primary_page)?;
        entries
            .get_mut(primary_path)
            .expect("validated primary remains present")
            .bytes = primary_bytes;
        for path in &candidate.paths {
            if path != primary_path {
                entries.remove(path);
            }
        }
        for (collection_id, bytes) in provenance {
            let fingerprint = sha256_digest(&bytes);
            let file_name = format!(
                "merge-{}.json",
                fingerprint
                    .strip_prefix("sha256:")
                    .expect("digest prefix is stable")
            );
            let relative_path = portable_merge_path(&collection_id, &file_name);
            entries.insert(
                relative_path.clone(),
                BundleEntryInput {
                    relative_path,
                    bytes,
                    classification: PortableEntryClassification::Provenance,
                },
            );
        }
    }
    let entries = entries.into_values().collect::<Vec<_>>();
    let mut source = preview.request.source.clone();
    source.logical_digest = bundle_logical_digest_inputs(&entries)?;
    Ok(BundleRequest {
        source,
        scope: preview.request.scope.clone(),
        entries,
    })
}

fn merge_page_metadata(primary: &mut crate::WikiFrontmatter, other: &crate::WikiFrontmatter) {
    primary.tags.extend(other.tags.iter().cloned());
    primary.aliases.extend(other.aliases.iter().cloned());
    primary.aliases.push(other.id.clone());
    primary.sources.extend(other.sources.iter().cloned());
    primary
        .source_links
        .extend(other.source_links.iter().cloned());
    primary.links.extend(other.links.iter().cloned());
    primary
        .related_concepts
        .extend(other.related_concepts.iter().cloned());
    primary.topics.extend(other.topics.iter().cloned());
    primary
        .contradictions
        .extend(other.contradictions.iter().cloned());
    primary.tags.sort();
    primary.tags.dedup();
    primary.aliases.sort();
    primary.aliases.dedup();
    primary.sources.sort();
    primary.sources.dedup();
    primary.source_links.sort();
    primary.source_links.dedup();
    primary.links.sort();
    primary.links.dedup();
    primary.related_concepts.sort();
    primary.related_concepts.dedup();
    primary.topics.sort();
    primary.topics.dedup();
    primary.contradictions.sort_by(|left, right| {
        (&left.source_a, &left.source_b, &left.summary).cmp(&(
            &right.source_a,
            &right.source_b,
            &right.summary,
        ))
    });
    primary.contradictions.dedup();
}

fn render_merge_page(page: &crate::WikiPage) -> Result<Vec<u8>, WikiError> {
    let yaml = serde_yaml::to_string(&page.frontmatter)
        .map_err(|error| WikiError::Io(format!("cannot serialize merged Wiki page: {error}")))?;
    Ok(format!(
        "---\n{}---\n\n{}\n",
        yaml.trim_start_matches("---\n"),
        page.body.trim()
    )
    .into_bytes())
}

fn insert_merge_provenance(
    entries: &mut BTreeMap<String, BundleEntryInput>,
    collection_id: &str,
    bytes: Vec<u8>,
) {
    let fingerprint = sha256_digest(&bytes);
    let file_name = format!(
        "merge-{}.json",
        fingerprint
            .strip_prefix("sha256:")
            .expect("digest prefix is stable")
    );
    let relative_path = portable_merge_path(collection_id, &file_name);
    entries.insert(
        relative_path.clone(),
        BundleEntryInput {
            relative_path,
            bytes,
            classification: PortableEntryClassification::Provenance,
        },
    );
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PortableCollectionRegistry {
    schema_version: u32,
    collections: Vec<PortableCollectionRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PortableCollectionRecord {
    collection_id: String,
    kind: CollectionKind,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_project_id: Option<String>,
    default_visibility: CollectionVisibility,
}

#[derive(Default)]
struct ExportExclusions {
    credential: usize,
    absolute_path: usize,
    confidential: usize,
    candidate_entries: usize,
    candidate_bytes: usize,
}

struct LoadedRegistry {
    registry: CollectionRegistry,
    bytes: Option<Vec<u8>>,
}

struct LoadedGenerationManifest {
    manifest: Option<GenerationManifest>,
    bytes: Option<Vec<u8>>,
    trust_bytes: Option<Vec<u8>>,
}

#[derive(Clone)]
enum IncomingKind {
    Wiki { file_name: String },
    Claim { file_name: String, claim_id: String },
    Merge { file_name: String },
    Suppression,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MergeProvenance {
    schema_version: u32,
    primary_path: String,
    original_path: String,
    original_sha256: String,
    original_markdown: String,
}

#[derive(Clone)]
struct IncomingPayload {
    collection_id: String,
    relative_path: String,
    bytes: Vec<u8>,
    kind: IncomingKind,
}

struct ValidatedIncoming {
    metadata: PortableCollectionRegistry,
    payloads: Vec<IncomingPayload>,
}

struct TargetRoot {
    collection_id: String,
    dir: Dir,
}

struct PlannedWrite {
    root_index: usize,
    relative: PathBuf,
    logical_locator: String,
    expected_bytes: Option<Vec<u8>>,
    bytes: Vec<u8>,
}

struct ImportPlan {
    roots: Vec<TargetRoot>,
    writes: Vec<PlannedWrite>,
    entry_count: usize,
    added_count: usize,
    unchanged_count: usize,
    conflict_paths: Vec<String>,
    target_state_digest: String,
    detached_collection_ids: Vec<String>,
}

struct TransactionFile {
    root_index: usize,
    logical_locator: String,
    snapshot: crate::CapabilityFileSnapshot,
}

/// Export canonical Markdown and portable collection truth into a deterministic bundle.
///
/// This operation never reads the disposable `SQLite` projection or changes Wiki enable state.
/// It pins source roots no-follow and holds the canonical integration lock while collecting a
/// coherent central registry/claim snapshot.
///
/// # Errors
///
/// Returns an error for an unsafe source, unknown or confidential exact scope, malformed
/// canonical data, an unportable identity, bundle budget failure, or publication conflict.
#[allow(clippy::too_many_lines)]
pub fn export_bundle(
    user_root: &Path,
    scope: BundleScope,
    destination: &Path,
    publish_mode: &BundlePublishMode,
    limits: BundleLimits,
) -> Result<BundleExportResult, WikiError> {
    let prepared = prepare_export_bundle(user_root, scope, limits)?;
    let receipt = encode_and_publish_bundle(&prepared.request, limits, destination, publish_mode)
        .map_err(bundle_io_error)?;
    let disposition = match receipt.outcome() {
        BundlePublishOutcome::Created => BundleExportDisposition::Created,
        BundlePublishOutcome::Unchanged => BundleExportDisposition::Unchanged,
        BundlePublishOutcome::Replaced => BundleExportDisposition::Replaced,
    };
    Ok(BundleExportResult {
        source: prepared.source,
        scope: prepared.scope,
        entry_count: prepared.entry_count,
        credential_excluded_count: prepared.exclusions.credential,
        absolute_path_excluded_count: prepared.exclusions.absolute_path,
        confidential_excluded_count: prepared.exclusions.confidential,
        disposition,
        archive_sha256: receipt.archive_sha256().to_owned(),
        byte_length: receipt.byte_length(),
        backup_path: receipt.backup_path().map(Path::to_path_buf),
    })
}

/// Build the exact portable archive in memory without writing a bundle file.
///
/// The returned digest and length describe the same deterministic bytes that `export_bundle`
/// would publish while the canonical source remains unchanged.
///
/// # Errors
/// Rejects unsafe source paths, invalid canonical data, or bundle resource limits.
pub fn preview_export_bundle(
    user_root: &Path,
    scope: BundleScope,
    limits: BundleLimits,
) -> Result<BundleExportPreview, WikiError> {
    let prepared = prepare_export_bundle(user_root, scope, limits)?;
    let encoded = encode_bundle(&prepared.request, limits)
        .map_err(|error| WikiError::Verification(error.to_string()))?;
    Ok(BundleExportPreview {
        source: prepared.source,
        scope: prepared.scope,
        entry_count: prepared.entry_count,
        credential_excluded_count: prepared.exclusions.credential,
        absolute_path_excluded_count: prepared.exclusions.absolute_path,
        confidential_excluded_count: prepared.exclusions.confidential,
        archive_sha256: encoded.plan().archive_sha256().to_owned(),
        byte_length: u64::try_from(encoded.archive().len())
            .map_err(|_| WikiError::Io("bundle preview length does not fit u64".to_owned()))?,
    })
}

struct ExportPreparation {
    request: BundleRequest,
    source: BundleSourceIdentity,
    scope: BundleScope,
    entry_count: usize,
    exclusions: ExportExclusions,
}

fn prepare_export_bundle(
    user_root: &Path,
    scope: BundleScope,
    _limits: BundleLimits,
) -> Result<ExportPreparation, WikiError> {
    let root_path = crate::shared::canonical_root(user_root)?;
    let root = pin_absolute_directory(&root_path, "user root")?;
    let _lock = crate::CapabilityKnowledgeLock::acquire(&root)?;
    let loaded = load_registry(&root_path, &root)?;
    let selected = select_export_collections(&loaded.registry, &scope)?;
    let portable_registry = portable_registry_from_records(&selected)?;
    let metadata_bytes = canonical_portable_registry_bytes(&portable_registry)?;
    crate::reject_likely_credentials(&metadata_bytes).map_err(|error| {
        WikiError::Verification(format!(
            "portable collection metadata contains likely sensitive material: {error}"
        ))
    })?;

    let mut exclusions = ExportExclusions::default();
    let mut entries = Vec::new();
    push_export_entry(
        &mut entries,
        &mut exclusions,
        BundleEntryInput {
            relative_path: PORTABLE_REGISTRY_PATH.to_owned(),
            bytes: metadata_bytes,
            classification: PortableEntryClassification::PortableMetadata,
        },
    )?;
    for collection in selected {
        collect_export_claims(&root, collection, &scope, &mut entries, &mut exclusions)?;
        if collection.state == CollectionState::Attached {
            let locator = collection.local_locator.as_deref().ok_or_else(|| {
                WikiError::Verification("attached collection lost its local locator".to_owned())
            })?;
            let source_path = crate::shared::canonical_root(Path::new(locator))?;
            let owned_source;
            let source = if source_path == root_path {
                &root
            } else {
                owned_source = pin_absolute_directory(&source_path, "collection root")?;
                &owned_source
            };
            collect_export_wiki(source, collection, &scope, &mut entries, &mut exclusions)?;
            collect_export_suppression(source, collection, &mut entries, &mut exclusions)?;
            collect_export_merge(source, collection, &mut entries, &mut exclusions)?;
        } else {
            let dormant_relative = Path::new(DETACHED_IMPORT_ROOT).join(&collection.collection_id);
            if let Some(source) =
                open_optional_dir(&root, &dormant_relative, "detached imported collection")?
            {
                collect_export_wiki(&source, collection, &scope, &mut entries, &mut exclusions)?;
                collect_export_suppression(&source, collection, &mut entries, &mut exclusions)?;
                collect_export_merge(&source, collection, &mut entries, &mut exclusions)?;
            }
        }
    }

    let portable_entry_count = entries
        .iter()
        .filter(|entry| is_portable_classification(entry.classification))
        .count();
    let logical_digest = bundle_logical_digest_inputs(&entries)?;
    let source = source_identity(&scope, logical_digest);
    let request = BundleRequest {
        source: source.clone(),
        scope: scope.clone(),
        entries,
    };
    Ok(ExportPreparation {
        request,
        source,
        scope,
        entry_count: portable_entry_count,
        exclusions,
    })
}

/// Validate and optionally atomically import a complete canonical knowledge bundle.
///
/// Identical entries are no-ops. Missing entries are added. Existing divergent bytes,
/// contradictory collection identity truth, confidential payloads, and unportable locators fail
/// before mutation. An apply writes canonical files first and rolls every touched path back if
/// publication fails. When Wiki is enabled, it also rebuilds the fixed normalized `SQLite`
/// projection. This explicit operation reads only the installed Wiki enablement flag and never
/// changes or enables retrieval policy.
///
/// # Errors
///
/// Returns an error for an unsafe archive or destination, invalid portable metadata, a merge
/// conflict, dirty destination state, failed index construction, or incomplete rollback.
#[allow(clippy::too_many_lines)]
pub fn import_bundle(
    user_root: &Path,
    bundle: &Path,
    mode: BundleImportMode,
    limits: BundleLimits,
) -> Result<BundleImportResult, WikiError> {
    import_bundle_checked(user_root, bundle, mode, limits, None)
}

/// Exact archive and dry-run digest approved for one transfer apply.
pub struct BundleImportApproval<'a> {
    /// SHA-256 of the archive obtained from the sending computer.
    pub archive_sha256: &'a str,
    /// SHA-256 of the serialized complete dry-run result.
    pub preview_digest: &'a str,
}

/// Apply a reviewed import, rechecking approval while holding the canonical write lock.
///
/// # Errors
/// Rejects changed archive or destination bytes before activation, as well as normal import errors.
pub fn import_bundle_reviewed(
    user_root: &Path,
    bundle: &Path,
    mode: BundleImportMode,
    limits: BundleLimits,
    approval: BundleImportApproval<'_>,
) -> Result<BundleImportResult, WikiError> {
    if mode == BundleImportMode::DryRun {
        return Err(WikiError::InvalidInput(
            "reviewed import requires an apply mode".to_owned(),
        ));
    }
    // Reject stale caller input before the writer lock creates its infrastructure directory.
    // The same approval is checked again under that lock before any canonical activation.
    let preview = import_bundle_checked(user_root, bundle, BundleImportMode::DryRun, limits, None)?;
    if approval.archive_sha256 != preview.archive_sha256
        || approval.preview_digest != import_preview_digest(&preview)?
    {
        return Err(WikiError::Conflict(
            "bundle or destination bytes changed after the transfer preview".to_owned(),
        ));
    }
    import_bundle_checked(user_root, bundle, mode, limits, Some(approval))
}

fn import_bundle_checked(
    user_root: &Path,
    bundle: &Path,
    mode: BundleImportMode,
    limits: BundleLimits,
    approval: Option<BundleImportApproval<'_>>,
) -> Result<BundleImportResult, WikiError> {
    let validated = load_bundle(bundle, limits).map_err(bundle_io_error)?;
    validate_manifest_logical_digest(&validated)?;
    let incoming = validate_incoming(&validated)?;
    let root_path = crate::shared::canonical_root(user_root)?;
    let root = pin_absolute_directory(&root_path, "user root")?;
    let _lock = if mode == BundleImportMode::DryRun {
        None
    } else {
        Some(crate::CapabilityKnowledgeLock::acquire(&root)?)
    };
    reject_dirty(&root)?;
    let rebuild_index = wiki_indexing_enabled(&root)?;
    let loaded = load_registry(&root_path, &root)?;
    let mut plan = build_import_plan(&root_path, root, &loaded, &incoming)?;
    let source = validated.manifest().source.clone();
    let scope = validated.manifest().scope.clone();
    let archive_sha256 = validated.archive_sha256().to_owned();
    let manifest_sha256 = validated.manifest_sha256().to_owned();
    let collection_tables: Vec<String> =
        COLLECTION_TABLES.iter().map(ToString::to_string).collect();
    let collection_ids: Vec<String> = incoming
        .metadata
        .collections
        .iter()
        .map(|record| record.collection_id.clone())
        .collect();

    let mut result = BundleImportResult {
        mode: BundleImportMode::DryRun,
        disposition: if plan.added_count == 0 {
            BundleImportDisposition::Noop
        } else {
            BundleImportDisposition::Planned
        },
        archive_sha256,
        manifest_sha256,
        source,
        scope,
        entry_count: plan.entry_count,
        added_count: plan.added_count,
        unchanged_count: plan.unchanged_count,
        conflict_paths: plan.conflict_paths.clone(),
        target_state_digest: plan.target_state_digest.clone(),
        detached_collection_ids: plan.detached_collection_ids.clone(),
        collection_ids,
        changed_paths: Vec::new(),
        canonical_mutation: false,
        index_rebuilt: false,
        rollback: BundleRollbackResult::none(),
        collection_tables,
    };
    if let Some(approval) = approval {
        if approval.archive_sha256 != result.archive_sha256
            || approval.preview_digest != import_preview_digest(&result)?
        {
            return Err(WikiError::Conflict(
                "bundle or destination bytes changed after the transfer preview".to_owned(),
            ));
        }
    }
    if !plan.conflict_paths.is_empty() && mode == BundleImportMode::Apply {
        return Err(WikiError::Conflict("bundle import has conflicting canonical entries; review them and use the explicit exclude-conflicts transfer option or cancel".to_owned()));
    }
    if mode != BundleImportMode::DryRun && plan.added_count > 0 {
        result.changed_paths = activate_import(&root_path, &mut plan, rebuild_index)?;
        result.disposition = BundleImportDisposition::Applied;
        result.canonical_mutation = true;
        result.index_rebuilt = rebuild_index;
    }
    result.mode = mode;
    Ok(result)
}

/// Digest a dry-run result in the same canonical JSON object order exposed by the CLI.
///
/// # Errors
/// Returns an error if result serialization fails.
pub fn import_preview_digest(result: &BundleImportResult) -> Result<String, WikiError> {
    serde_json::to_value(result)
        .and_then(|value| serde_json::to_vec(&value))
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| WikiError::Io(error.to_string()))
}

/// Validate and deterministically combine several portable bundles without writing canonical data.
///
/// The result intentionally retains semantic candidates. A caller must obtain a reviewed host
/// decision before treating two differently named pages as one knowledge item.
///
/// # Errors
///
/// Returns an error for fewer than two inputs, unsafe or malformed bundles, an unsupported mixed
/// scope, or a resource limit violation. Content conflicts are returned in the preview so callers
/// can present every conflict together.
pub fn preview_bundle_merge(
    bundles: &[PathBuf],
    limits: BundleLimits,
) -> Result<BundleMergePreview, WikiError> {
    if bundles.len() < 2 {
        return Err(WikiError::InvalidInput(
            "multi-bundle merge requires at least two bundle paths".to_owned(),
        ));
    }
    let mut loaded = Vec::with_capacity(bundles.len());
    for bundle in bundles {
        let plan = load_bundle(bundle, limits).map_err(bundle_io_error)?;
        validate_manifest_logical_digest(&plan)?;
        let incoming = validate_incoming(&plan)?;
        loaded.push((plan, incoming));
    }
    loaded.sort_by(|left, right| left.0.archive_sha256().cmp(right.0.archive_sha256()));
    if loaded
        .windows(2)
        .any(|pair| pair[0].0.archive_sha256() == pair[1].0.archive_sha256())
    {
        loaded.dedup_by(|left, right| left.0.archive_sha256() == right.0.archive_sha256());
    }

    let archive_sha256s = loaded
        .iter()
        .map(|(plan, _)| plan.archive_sha256().to_owned())
        .collect::<Vec<_>>();
    let mut input_entry_count = 0_usize;
    let mut exact_duplicate_count = bundles.len().saturating_sub(loaded.len());
    let mut conflicts = Vec::new();
    let mut collections = BTreeMap::<String, PortableCollectionRecord>::new();
    let mut entries = BTreeMap::<String, BundleEntryInput>::new();
    let mut conflicting_entries = BTreeMap::<String, Vec<BundleEntryInput>>::new();
    let mut claim_paths = BTreeMap::<String, String>::new();

    for (_, incoming) in &loaded {
        for collection in &incoming.metadata.collections {
            match collections.get_mut(&collection.collection_id) {
                Some(existing) if same_collection_identity(existing, collection) => {
                    existing.aliases.extend(collection.aliases.iter().cloned());
                    existing.aliases.sort_by(|left, right| {
                        folded_alias(left)
                            .cmp(&folded_alias(right))
                            .then_with(|| left.cmp(right))
                    });
                    existing
                        .aliases
                        .dedup_by(|left, right| folded_alias(left) == folded_alias(right));
                    exact_duplicate_count += 1;
                }
                Some(_) => conflicts.push(format!("collections/{}", collection.collection_id)),
                None => {
                    collections.insert(collection.collection_id.clone(), collection.clone());
                }
            }
        }
        for payload in &incoming.payloads {
            input_entry_count += 1;
            let classification = match payload.kind {
                IncomingKind::Wiki { .. } => PortableEntryClassification::CanonicalMarkdown,
                IncomingKind::Claim { .. } => PortableEntryClassification::Provenance,
                IncomingKind::Merge { .. } => PortableEntryClassification::Provenance,
                IncomingKind::Suppression => PortableEntryClassification::Suppression,
            };
            if let IncomingKind::Claim { claim_id, .. } = &payload.kind {
                if let Some(existing) = claim_paths.get(claim_id) {
                    if existing != &payload.relative_path {
                        conflicts.push(existing.clone());
                        conflicts.push(payload.relative_path.clone());
                        continue;
                    }
                } else {
                    claim_paths.insert(claim_id.clone(), payload.relative_path.clone());
                }
            }
            if let Some(variants) = conflicting_entries.get_mut(&payload.relative_path) {
                if !variants
                    .iter()
                    .any(|variant| variant.bytes == payload.bytes)
                {
                    variants.push(BundleEntryInput {
                        relative_path: payload.relative_path.clone(),
                        bytes: payload.bytes.clone(),
                        classification,
                    });
                }
                continue;
            }
            match entries.get_mut(&payload.relative_path) {
                Some(existing) if existing.bytes == payload.bytes => exact_duplicate_count += 1,
                Some(existing)
                    if classification == PortableEntryClassification::Suppression
                        && existing.classification == PortableEntryClassification::Suppression =>
                {
                    existing.bytes = merge_suppression(Some(&existing.bytes), &payload.bytes)?;
                }
                Some(existing) => {
                    let path = payload.relative_path.clone();
                    let removed = existing.clone();
                    entries.remove(&path);
                    let variants = conflicting_entries.entry(path.clone()).or_default();
                    if !variants
                        .iter()
                        .any(|variant| variant.bytes == removed.bytes)
                    {
                        variants.push(removed);
                    }
                    if !variants
                        .iter()
                        .any(|variant| variant.bytes == payload.bytes)
                    {
                        variants.push(BundleEntryInput {
                            relative_path: path.clone(),
                            bytes: payload.bytes.clone(),
                            classification,
                        });
                    }
                    conflicts.push(path);
                }
                None => {
                    entries.insert(
                        payload.relative_path.clone(),
                        BundleEntryInput {
                            relative_path: payload.relative_path.clone(),
                            bytes: payload.bytes.clone(),
                            classification,
                        },
                    );
                }
            }
        }
    }

    let registry = validate_portable_registry(PortableCollectionRegistry {
        schema_version: PORTABLE_SCHEMA_VERSION,
        collections: collections.into_values().collect(),
    })?;
    let scope = merge_scope(&registry)?;
    entries.insert(
        PORTABLE_REGISTRY_PATH.to_owned(),
        BundleEntryInput {
            relative_path: PORTABLE_REGISTRY_PATH.to_owned(),
            bytes: canonical_portable_registry_bytes(&registry)?,
            classification: PortableEntryClassification::PortableMetadata,
        },
    );
    let request_entries = entries.into_values().collect::<Vec<_>>();
    let logical_digest = bundle_logical_digest_inputs(&request_entries)?;
    let request = BundleRequest {
        source: BundleSourceIdentity {
            kind: BundleSourceKind::UserRoot,
            id: USER_ROOT_COLLECTION_ID.to_owned(),
            logical_digest,
        },
        scope,
        entries: request_entries,
    };
    conflicts.sort();
    conflicts.dedup();
    for variants in conflicting_entries.values_mut() {
        variants
            .sort_by(|left, right| sha256_digest(&left.bytes).cmp(&sha256_digest(&right.bytes)));
    }
    let conflicts_detail = conflicting_entries
        .iter()
        .map(|(path, variants)| BundleMergeConflict {
            path: path.clone(),
            variants: variants
                .iter()
                .map(|variant| BundleMergeVariant {
                    sha256: sha256_digest(&variant.bytes),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let semantic_candidates = semantic_merge_candidates(&request.entries)?;
    Ok(BundleMergePreview {
        archive_sha256s,
        input_entry_count,
        exact_duplicate_count,
        conflict_paths: conflicts,
        conflicts: conflicts_detail,
        semantic_candidates,
        request,
        conflicting_entries,
    })
}

fn same_collection_identity(
    left: &PortableCollectionRecord,
    right: &PortableCollectionRecord,
) -> bool {
    left.collection_id == right.collection_id
        && left.kind == right.kind
        && left.source_project_id == right.source_project_id
        && left.default_visibility == right.default_visibility
}

fn merge_scope(registry: &PortableCollectionRegistry) -> Result<BundleScope, WikiError> {
    if registry.collections.len() == 1 {
        let only = &registry.collections[0];
        return Ok(if only.collection_id == USER_ROOT_COLLECTION_ID {
            BundleScope::Global
        } else {
            BundleScope::Collection {
                id: only.collection_id.clone(),
            }
        });
    }
    if registry
        .collections
        .iter()
        .any(|collection| collection.collection_id == USER_ROOT_COLLECTION_ID)
    {
        return Ok(BundleScope::AllPortable);
    }
    Err(WikiError::Conflict(
        "multiple portable collections require a user-root collection identity".to_owned(),
    ))
}

fn semantic_merge_candidates(
    entries: &[BundleEntryInput],
) -> Result<Vec<BundleMergeCandidate>, WikiError> {
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for entry in entries {
        let Ok((_, leaf, Some(file_name))) = parse_portable_payload_path(&entry.relative_path)
        else {
            continue;
        };
        if leaf != "Wiki" {
            continue;
        }
        let page = parse_page_bytes(&entry.bytes, &format!("{WIKI_RELATIVE}/{file_name}"))?;
        let key = sha256_digest(
            format!(
                "{}\0{}\0{}",
                page.frontmatter.kind,
                page.frontmatter.summary.trim().to_lowercase(),
                page.body.trim().to_lowercase()
            )
            .as_bytes(),
        );
        groups
            .entry(key)
            .or_default()
            .push(entry.relative_path.clone());
    }
    let mut candidates = groups
        .into_iter()
        .filter_map(|(candidate_id, mut paths)| {
            (paths.len() > 1).then(|| {
                paths.sort();
                BundleMergeCandidate {
                    candidate_id,
                    paths,
                }
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    Ok(candidates)
}

fn source_identity(scope: &BundleScope, logical_digest: String) -> BundleSourceIdentity {
    match scope {
        BundleScope::Global | BundleScope::Shared | BundleScope::AllPortable => {
            BundleSourceIdentity {
                kind: BundleSourceKind::UserRoot,
                id: USER_ROOT_COLLECTION_ID.to_owned(),
                logical_digest,
            }
        }
        BundleScope::Project { id } => BundleSourceIdentity {
            kind: BundleSourceKind::Project,
            id: id.clone(),
            logical_digest,
        },
        BundleScope::Collection { id } => BundleSourceIdentity {
            kind: BundleSourceKind::Collection,
            id: id.clone(),
            logical_digest,
        },
    }
}

fn select_export_collections<'a>(
    registry: &'a CollectionRegistry,
    scope: &BundleScope,
) -> Result<Vec<&'a CollectionRecord>, WikiError> {
    let user_root = registry
        .collections
        .iter()
        .find(|collection| collection.collection_id == USER_ROOT_COLLECTION_ID)
        .ok_or_else(|| WikiError::Verification("user-root collection is missing".to_owned()))?;
    let mut selected = match scope {
        BundleScope::Global => vec![user_root],
        BundleScope::Shared => registry
            .collections
            .iter()
            .filter(|collection| {
                collection.collection_id == USER_ROOT_COLLECTION_ID
                    || collection.default_visibility == CollectionVisibility::Shared
            })
            .collect(),
        BundleScope::Project { id } => {
            let collection_id = match registry.resolve_project(id) {
                CollectionResolution::Resolved(collection_id) => collection_id,
                CollectionResolution::Unknown => {
                    return Err(WikiError::InvalidInput(format!(
                        "unknown project export scope `{id}`"
                    )));
                }
                CollectionResolution::Ambiguous(matches) => {
                    return Err(WikiError::Conflict(format!(
                        "ambiguous project export scope `{id}`: {}",
                        matches.join(", ")
                    )));
                }
            };
            vec![registry
                .collections
                .iter()
                .find(|collection| collection.collection_id == collection_id)
                .expect("resolved collection remains present")]
        }
        BundleScope::Collection { id } => vec![registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == *id)
            .ok_or_else(|| {
                WikiError::InvalidInput(format!("unknown collection export scope `{id}`"))
            })?],
        BundleScope::AllPortable => registry.collections.iter().collect(),
    };
    if matches!(
        scope,
        BundleScope::Project { .. } | BundleScope::Collection { .. }
    ) && selected[0].default_visibility == CollectionVisibility::Confidential
    {
        return Err(WikiError::Conflict(
            "an exact confidential collection cannot be exported".to_owned(),
        ));
    }
    selected.retain(|collection| {
        collection.collection_id == USER_ROOT_COLLECTION_ID
            || collection.default_visibility != CollectionVisibility::Confidential
    });
    selected.sort_by(|left, right| left.collection_id.cmp(&right.collection_id));
    Ok(selected)
}

fn portable_registry_from_records(
    records: &[&CollectionRecord],
) -> Result<PortableCollectionRegistry, WikiError> {
    let mut collections = Vec::with_capacity(records.len());
    for record in records {
        if let Some(project_id) = record.source_project_id.as_deref() {
            validate_bundle_identity("source_project_id", project_id)?;
        }
        let mut aliases = record
            .aliases
            .iter()
            .filter(|alias| !is_absolute_like(alias))
            .cloned()
            .collect::<Vec<_>>();
        aliases.sort_by(|left, right| {
            folded_alias(left)
                .cmp(&folded_alias(right))
                .then_with(|| left.cmp(right))
        });
        aliases.dedup_by(|left, right| folded_alias(left) == folded_alias(right));
        collections.push(PortableCollectionRecord {
            collection_id: record.collection_id.clone(),
            kind: record.kind,
            aliases,
            source_project_id: record.source_project_id.clone(),
            default_visibility: record.default_visibility,
        });
    }
    validate_portable_registry(PortableCollectionRegistry {
        schema_version: PORTABLE_SCHEMA_VERSION,
        collections,
    })
}

fn collect_export_wiki(
    source: &Dir,
    collection: &CollectionRecord,
    scope: &BundleScope,
    entries: &mut Vec<BundleEntryInput>,
    exclusions: &mut ExportExclusions,
) -> Result<(), WikiError> {
    let Some(wiki) = open_optional_dir(source, Path::new(WIKI_RELATIVE), "collection Wiki")? else {
        return Ok(());
    };
    for name in directory_names(&wiki, "collection Wiki")? {
        let metadata = wiki
            .symlink_metadata(&name)
            .map_err(|error| WikiError::Io(format!("cannot inspect Wiki entry: {error}")))?;
        if metadata.is_dir() {
            continue;
        }
        if !metadata.is_file() {
            return Err(WikiError::Verification(
                "collection Wiki contains a symlink or special file".to_owned(),
            ));
        }
        let file_name = utf8_name(&name, "Wiki filename")?;
        let path = Path::new(&file_name);
        if path.extension().and_then(|extension| extension.to_str()) != Some("md")
            || matches!(file_name.as_str(), "index.md" | "log.md")
        {
            continue;
        }
        let bytes = read_named_file(&wiki, &name, MAX_WIKI_BYTES, "Wiki page")?;
        let portable_path = portable_wiki_path(&collection.collection_id, &file_name);
        if crate::reject_likely_credentials(&bytes).is_err() {
            exclusions.credential += 1;
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::Credential,
                },
            )?;
            continue;
        }
        let locator = format!("{WIKI_RELATIVE}/{file_name}");
        let page = parse_page_bytes(&bytes, &locator)?;
        if page.frontmatter.id
            != path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("")
        {
            return Err(WikiError::Verification(format!(
                "Wiki filename does not match page ID: {locator}"
            )));
        }
        if page
            .frontmatter
            .sources
            .iter()
            .any(|source| is_absolute_like(source))
        {
            exclusions.absolute_path += 1;
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::AbsolutePath,
                },
            )?;
            continue;
        }
        let visibility: RagVisibility = collection.default_visibility.into();
        if visibility == RagVisibility::Confidential {
            exclusions.confidential += 1;
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::Confidential,
                },
            )?;
        } else if include_visibility(scope, collection, visibility) {
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::CanonicalMarkdown,
                },
            )?;
        }
    }
    Ok(())
}

fn collect_export_claims(
    user_root: &Dir,
    collection: &CollectionRecord,
    scope: &BundleScope,
    entries: &mut Vec<BundleEntryInput>,
    exclusions: &mut ExportExclusions,
) -> Result<(), WikiError> {
    let relative = Path::new(CLAIMS_RELATIVE).join(&collection.collection_id);
    let Some(claims) = open_optional_dir(user_root, &relative, "claim collection")? else {
        return Ok(());
    };
    for name in directory_names(&claims, "claim collection")? {
        let metadata = claims
            .symlink_metadata(&name)
            .map_err(|error| WikiError::Io(format!("cannot inspect claim entry: {error}")))?;
        if !metadata.is_file() {
            return Err(WikiError::Verification(
                "claim collection entries must be no-follow regular files".to_owned(),
            ));
        }
        let file_name = utf8_name(&name, "claim filename")?;
        let path = Path::new(&file_name);
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            return Err(WikiError::Verification(
                "claim collection contains a non-Markdown entry".to_owned(),
            ));
        }
        let bytes = read_named_file(&claims, &name, MAX_CLAIM_BYTES, "claim")?;
        let portable_path = portable_claim_path(&collection.collection_id, &file_name);
        if crate::reject_likely_credentials(&bytes).is_err() {
            exclusions.credential += 1;
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::Credential,
                },
            )?;
            continue;
        }
        let locator = format!(
            "{CLAIMS_RELATIVE}/{}/{}",
            collection.collection_id, file_name
        );
        let claim = parse_claim_markdown(
            &locator,
            std::str::from_utf8(&bytes)
                .map_err(|error| WikiError::InvalidInput(format!("claim is not UTF-8: {error}")))?,
        )
        .map_err(rag_error)?;
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        if claim.collection_id != collection.collection_id || claim.claim_id != stem {
            return Err(WikiError::Verification(format!(
                "claim path does not match canonical IDs: {locator}"
            )));
        }
        if claim_has_absolute_locator(&claim) {
            exclusions.absolute_path += 1;
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::AbsolutePath,
                },
            )?;
        } else if claim.visibility == RagVisibility::Confidential {
            exclusions.confidential += 1;
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::Confidential,
                },
            )?;
        } else if include_visibility(scope, collection, claim.visibility) {
            push_export_entry(
                entries,
                exclusions,
                BundleEntryInput {
                    relative_path: portable_path,
                    bytes,
                    classification: PortableEntryClassification::Provenance,
                },
            )?;
        }
    }
    Ok(())
}

fn collect_export_suppression(
    source: &Dir,
    collection: &CollectionRecord,
    entries: &mut Vec<BundleEntryInput>,
    exclusions: &mut ExportExclusions,
) -> Result<(), WikiError> {
    let Some(bytes) = read_bounded_optional(
        source,
        Path::new(SUPPRESSION_RELATIVE),
        MAX_SUPPRESSION_BYTES,
        "suppression ledger",
    )?
    else {
        return Ok(());
    };
    let canonical = canonical_suppression_bytes(&bytes)?;
    push_export_entry(
        entries,
        exclusions,
        BundleEntryInput {
            relative_path: portable_suppression_path(&collection.collection_id),
            bytes: canonical,
            classification: PortableEntryClassification::Suppression,
        },
    )?;
    Ok(())
}

fn collect_export_merge(
    source: &Dir,
    collection: &CollectionRecord,
    entries: &mut Vec<BundleEntryInput>,
    exclusions: &mut ExportExclusions,
) -> Result<(), WikiError> {
    let Some(directory) = open_optional_dir(source, Path::new(MERGE_RELATIVE), "merge provenance")?
    else {
        return Ok(());
    };
    for name in directory_names(&directory, "merge provenance")? {
        let metadata = directory
            .symlink_metadata(&name)
            .map_err(|error| WikiError::Io(format!("cannot inspect merge provenance: {error}")))?;
        if !metadata.is_file() {
            return Err(WikiError::Verification(
                "merge provenance contains a non-regular file".to_owned(),
            ));
        }
        let file_name = utf8_name(&name, "merge provenance filename")?;
        if Path::new(&file_name)
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("json")
        {
            return Err(WikiError::Verification(
                "merge provenance filename must end with .json".to_owned(),
            ));
        }
        let bytes = read_named_file(
            &directory,
            &name,
            MAX_MERGE_PROVENANCE_BYTES,
            "merge provenance",
        )?;
        validate_merge_provenance(&bytes)?;
        push_export_entry(
            entries,
            exclusions,
            BundleEntryInput {
                relative_path: portable_merge_path(&collection.collection_id, &file_name),
                bytes,
                classification: PortableEntryClassification::Provenance,
            },
        )?;
    }
    Ok(())
}

fn push_export_entry(
    entries: &mut Vec<BundleEntryInput>,
    exclusions: &mut ExportExclusions,
    entry: BundleEntryInput,
) -> Result<(), WikiError> {
    exclusions.candidate_entries = exclusions
        .candidate_entries
        .checked_add(1)
        .ok_or_else(|| WikiError::Verification("export candidate count overflow".to_owned()))?;
    if exclusions.candidate_entries > MAX_DIRECTORY_ENTRIES {
        return Err(WikiError::Verification(format!(
            "export candidates exceed the {MAX_DIRECTORY_ENTRIES} entry bound"
        )));
    }
    exclusions.candidate_bytes = exclusions
        .candidate_bytes
        .checked_add(entry.bytes.len())
        .ok_or_else(|| {
            WikiError::Verification("export candidate byte count overflow".to_owned())
        })?;
    if exclusions.candidate_bytes > MAX_EXPORT_CANDIDATE_BYTES {
        return Err(WikiError::Verification(format!(
            "export candidates exceed the {MAX_EXPORT_CANDIDATE_BYTES} byte bound"
        )));
    }
    entries.push(entry);
    Ok(())
}

fn include_visibility(
    scope: &BundleScope,
    collection: &CollectionRecord,
    visibility: RagVisibility,
) -> bool {
    if visibility == RagVisibility::Confidential {
        return false;
    }
    match scope {
        BundleScope::Shared => visibility == RagVisibility::Shared,
        BundleScope::Global => collection.collection_id == USER_ROOT_COLLECTION_ID,
        BundleScope::Project { .. } | BundleScope::Collection { .. } | BundleScope::AllPortable => {
            true
        }
    }
}

fn validate_manifest_logical_digest(plan: &ValidatedBundlePlan) -> Result<(), WikiError> {
    let computed = bundle_logical_digest_validated(plan)?;
    if plan.manifest().source.logical_digest != computed {
        return Err(WikiError::Verification(
            "bundle source logical digest does not match its portable payloads".to_owned(),
        ));
    }
    match (&plan.manifest().scope, plan.manifest().source.kind) {
        (
            BundleScope::Global | BundleScope::Shared | BundleScope::AllPortable,
            BundleSourceKind::UserRoot,
        ) if plan.manifest().source.id == USER_ROOT_COLLECTION_ID => Ok(()),
        (BundleScope::Project { id }, BundleSourceKind::Project)
            if plan.manifest().source.id == *id =>
        {
            Ok(())
        }
        (BundleScope::Collection { id }, BundleSourceKind::Collection)
            if plan.manifest().source.id == *id =>
        {
            Ok(())
        }
        _ => Err(WikiError::Verification(
            "bundle source identity does not match its declared scope".to_owned(),
        )),
    }
}

#[allow(clippy::too_many_lines)]
fn validate_incoming(plan: &ValidatedBundlePlan) -> Result<ValidatedIncoming, WikiError> {
    let metadata_entries = plan
        .entries()
        .iter()
        .filter(|entry| entry.relative_path() == PORTABLE_REGISTRY_PATH)
        .collect::<Vec<_>>();
    if metadata_entries.len() != 1
        || metadata_entries[0].classification() != PortableEntryClassification::PortableMetadata
    {
        return Err(WikiError::Verification(
            "bundle requires exactly one portable collection registry".to_owned(),
        ));
    }
    crate::reject_likely_credentials(metadata_entries[0].bytes()).map_err(|error| {
        WikiError::Verification(format!(
            "portable collection metadata contains likely sensitive material: {error}"
        ))
    })?;
    let metadata: PortableCollectionRegistry = serde_json::from_slice(metadata_entries[0].bytes())
        .map_err(|error| {
            WikiError::Verification(format!("invalid portable collection registry: {error}"))
        })?;
    let metadata = validate_portable_registry(metadata)?;
    if canonical_portable_registry_bytes(&metadata)? != metadata_entries[0].bytes() {
        return Err(WikiError::Verification(
            "portable collection registry is not canonical JSON".to_owned(),
        ));
    }
    validate_metadata_scope(&metadata, &plan.manifest().scope)?;
    let known = metadata
        .collections
        .iter()
        .map(|collection| collection.collection_id.as_str())
        .collect::<BTreeSet<_>>();
    let records = metadata
        .collections
        .iter()
        .map(|collection| (collection.collection_id.as_str(), collection))
        .collect::<BTreeMap<_, _>>();
    let mut payloads = Vec::new();
    let mut claim_ids = BTreeSet::new();
    let mut suppressions = BTreeSet::new();
    for entry in plan
        .entries()
        .iter()
        .filter(|entry| entry.relative_path() != PORTABLE_REGISTRY_PATH)
    {
        let (collection_id, leaf, file_name) = parse_portable_payload_path(entry.relative_path())?;
        if !known.contains(collection_id.as_str()) {
            return Err(WikiError::Verification(format!(
                "bundle payload references unknown collection `{collection_id}`"
            )));
        }
        crate::reject_likely_credentials(entry.bytes()).map_err(|error| {
            WikiError::Verification(format!(
                "portable payload contains likely sensitive material: {error}"
            ))
        })?;
        let record = records[collection_id.as_str()];
        let kind = match leaf.as_str() {
            "Wiki" => {
                if entry.classification() != PortableEntryClassification::CanonicalMarkdown {
                    return Err(classification_error(entry.relative_path()));
                }
                let file_name = file_name.ok_or_else(|| {
                    WikiError::Verification("portable Wiki entry lacks a filename".to_owned())
                })?;
                if matches!(file_name.as_str(), "index.md" | "log.md") {
                    return Err(WikiError::Verification(
                        "portable bundles cannot replace generated Wiki index or log files"
                            .to_owned(),
                    ));
                }
                let locator = format!("{WIKI_RELATIVE}/{file_name}");
                let page = parse_page_bytes(entry.bytes(), &locator)?;
                if page.frontmatter.id
                    != Path::new(&file_name)
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("")
                {
                    return Err(WikiError::Verification(format!(
                        "portable Wiki filename does not match page ID: {}",
                        entry.relative_path()
                    )));
                }
                if page
                    .frontmatter
                    .sources
                    .iter()
                    .any(|source| is_absolute_like(source))
                {
                    return Err(WikiError::Verification(
                        "portable Wiki page contains a machine-bound source locator".to_owned(),
                    ));
                }
                if record.default_visibility == CollectionVisibility::Confidential {
                    return Err(WikiError::Verification(
                        "portable bundle contains confidential Wiki content".to_owned(),
                    ));
                }
                if matches!(plan.manifest().scope, BundleScope::Shared)
                    && record.default_visibility != CollectionVisibility::Shared
                {
                    return Err(WikiError::Verification(
                        "shared bundle contains project-private Wiki content".to_owned(),
                    ));
                }
                IncomingKind::Wiki { file_name }
            }
            "Claims" => {
                if entry.classification() != PortableEntryClassification::Provenance {
                    return Err(classification_error(entry.relative_path()));
                }
                let file_name = file_name.ok_or_else(|| {
                    WikiError::Verification("portable claim entry lacks a filename".to_owned())
                })?;
                let locator = format!("{CLAIMS_RELATIVE}/{collection_id}/{file_name}");
                let claim = parse_claim_markdown(
                    &locator,
                    std::str::from_utf8(entry.bytes()).map_err(|error| {
                        WikiError::Verification(format!("portable claim is not UTF-8: {error}"))
                    })?,
                )
                .map_err(rag_error)?;
                let stem = Path::new(&file_name)
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("");
                if claim.collection_id != collection_id || claim.claim_id != stem {
                    return Err(WikiError::Verification(format!(
                        "portable claim path does not match canonical IDs: {locator}"
                    )));
                }
                if claim.visibility == RagVisibility::Confidential {
                    return Err(WikiError::Verification(
                        "portable bundle contains a confidential claim".to_owned(),
                    ));
                }
                if claim_has_absolute_locator(&claim) {
                    return Err(WikiError::Verification(
                        "portable claim contains a machine-bound absolute locator".to_owned(),
                    ));
                }
                if matches!(plan.manifest().scope, BundleScope::Shared)
                    && claim.visibility != RagVisibility::Shared
                {
                    return Err(WikiError::Verification(
                        "shared bundle contains project-private claim content".to_owned(),
                    ));
                }
                if !claim_ids.insert(claim.claim_id.clone()) {
                    return Err(WikiError::Verification(format!(
                        "duplicate portable claim ID `{}`",
                        claim.claim_id
                    )));
                }
                IncomingKind::Claim {
                    file_name,
                    claim_id: claim.claim_id,
                }
            }
            "Merge" => {
                if entry.classification() != PortableEntryClassification::Provenance {
                    return Err(classification_error(entry.relative_path()));
                }
                let file_name = file_name.ok_or_else(|| {
                    WikiError::Verification("portable merge entry lacks a filename".to_owned())
                })?;
                if Path::new(&file_name)
                    .extension()
                    .and_then(|extension| extension.to_str())
                    != Some("json")
                {
                    return Err(WikiError::Verification(
                        "portable merge entry must end with .json".to_owned(),
                    ));
                }
                validate_merge_provenance(entry.bytes())?;
                IncomingKind::Merge { file_name }
            }
            "suppression.yml" => {
                if entry.classification() != PortableEntryClassification::Suppression
                    || file_name.is_some()
                {
                    return Err(classification_error(entry.relative_path()));
                }
                if !suppressions.insert(collection_id.clone()) {
                    return Err(WikiError::Verification(format!(
                        "duplicate suppression ledger for collection `{collection_id}`"
                    )));
                }
                if canonical_suppression_bytes(entry.bytes())? != entry.bytes() {
                    return Err(WikiError::Verification(
                        "portable suppression ledger is not canonical YAML".to_owned(),
                    ));
                }
                IncomingKind::Suppression
            }
            _ => return Err(classification_error(entry.relative_path())),
        };
        payloads.push(IncomingPayload {
            collection_id,
            relative_path: entry.relative_path().to_owned(),
            bytes: entry.bytes().to_vec(),
            kind,
        });
    }
    Ok(ValidatedIncoming { metadata, payloads })
}

fn validate_metadata_scope(
    metadata: &PortableCollectionRegistry,
    scope: &BundleScope,
) -> Result<(), WikiError> {
    let ids = metadata
        .collections
        .iter()
        .map(|collection| collection.collection_id.as_str())
        .collect::<BTreeSet<_>>();
    match scope {
        BundleScope::Global => {
            if ids != BTreeSet::from([USER_ROOT_COLLECTION_ID]) {
                return Err(WikiError::Verification(
                    "global bundle metadata must contain only user-root".to_owned(),
                ));
            }
        }
        BundleScope::Shared => {
            if !ids.contains(USER_ROOT_COLLECTION_ID)
                || metadata.collections.iter().any(|collection| {
                    collection.collection_id != USER_ROOT_COLLECTION_ID
                        && collection.default_visibility != CollectionVisibility::Shared
                })
            {
                return Err(WikiError::Verification(
                    "shared bundle metadata exceeds the shared collection scope".to_owned(),
                ));
            }
        }
        BundleScope::Project { id } => {
            if metadata.collections.len() != 1
                || metadata.collections[0].source_project_id.as_deref() != Some(id)
            {
                return Err(WikiError::Verification(
                    "project bundle metadata does not match its exact project scope".to_owned(),
                ));
            }
        }
        BundleScope::Collection { id } => {
            if metadata.collections.len() != 1 || metadata.collections[0].collection_id != *id {
                return Err(WikiError::Verification(
                    "collection bundle metadata does not match its exact collection scope"
                        .to_owned(),
                ));
            }
        }
        BundleScope::AllPortable => {
            if !ids.contains(USER_ROOT_COLLECTION_ID) {
                return Err(WikiError::Verification(
                    "all-portable bundle metadata requires user-root".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn build_import_plan(
    root_path: &Path,
    root: Dir,
    loaded: &LoadedRegistry,
    incoming: &ValidatedIncoming,
) -> Result<ImportPlan, WikiError> {
    let mut merged = loaded.registry.clone();
    let mut detached = Vec::new();
    for portable in &incoming.metadata.collections {
        if let Some(existing) = merged
            .collections
            .iter()
            .find(|collection| collection.collection_id == portable.collection_id)
        {
            validate_same_collection_truth(existing, portable)?;
        } else if portable.collection_id == USER_ROOT_COLLECTION_ID {
            return Err(WikiError::Conflict(
                "destination user-root collection identity is missing".to_owned(),
            ));
        } else {
            merged.collections.push(CollectionRecord {
                collection_id: portable.collection_id.clone(),
                kind: portable.kind,
                state: CollectionState::Detached,
                aliases: portable.aliases.clone(),
                local_locator: None,
                source_project_id: portable.source_project_id.clone(),
                default_visibility: portable.default_visibility,
            });
        }
    }
    merged = merged
        .canonicalized()
        .map_err(|error| WikiError::Conflict(error.to_string()))?;
    validate_local_registry(root_path, &merged)?;
    for portable in &incoming.metadata.collections {
        let destination = merged
            .collections
            .iter()
            .find(|collection| collection.collection_id == portable.collection_id)
            .expect("merged collection remains present");
        if destination.state == CollectionState::Detached
            && destination.collection_id != USER_ROOT_COLLECTION_ID
        {
            detached.push(destination.collection_id.clone());
        }
    }
    detached.sort();
    detached.dedup();

    let mut roots = vec![TargetRoot {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        dir: root,
    }];
    let selected_ids = incoming
        .metadata
        .collections
        .iter()
        .map(|collection| collection.collection_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut attached = merged
        .collections
        .iter()
        .filter(|collection| {
            selected_ids.contains(collection.collection_id.as_str())
                && collection.collection_id != USER_ROOT_COLLECTION_ID
                && collection.state == CollectionState::Attached
        })
        .collect::<Vec<_>>();
    attached.sort_by(|left, right| left.collection_id.cmp(&right.collection_id));
    for collection in attached {
        let locator = collection.local_locator.as_deref().ok_or_else(|| {
            WikiError::Verification("attached collection lost its local locator".to_owned())
        })?;
        roots.push(TargetRoot {
            collection_id: collection.collection_id.clone(),
            dir: pin_absolute_directory(
                &crate::shared::canonical_root(Path::new(locator))?,
                "attached import collection",
            )?,
        });
    }
    let root_indices = roots
        .iter()
        .enumerate()
        .map(|(index, target)| (target.collection_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();

    let registry_bytes = registry_bytes(&merged)?;
    let registry_changed = loaded.bytes.as_deref() != Some(registry_bytes.as_slice());
    let mut writes = Vec::new();
    let mut added_count = usize::from(registry_changed);
    let mut unchanged_count = usize::from(!registry_changed);
    let mut conflict_paths = Vec::new();
    let mut observed = BTreeMap::from([(
        COLLECTION_REGISTRY_RELATIVE.to_owned(),
        loaded.bytes.as_deref().map(sha256_digest),
    )]);
    if registry_changed {
        writes.push(PlannedWrite {
            root_index: 0,
            relative: PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
            logical_locator: COLLECTION_REGISTRY_RELATIVE.to_owned(),
            expected_bytes: loaded.bytes.clone(),
            bytes: registry_bytes,
        });
    }

    let existing_claims = existing_claim_locations(&roots[0].dir)?;
    for payload in &incoming.payloads {
        let collection = merged
            .collections
            .iter()
            .find(|collection| collection.collection_id == payload.collection_id)
            .expect("validated portable collection remains in merged registry");
        let (root_index, relative, logical_locator) = match &payload.kind {
            IncomingKind::Wiki { file_name } => {
                payload_destination(collection, &root_indices, WIKI_RELATIVE, file_name)?
            }
            IncomingKind::Claim {
                file_name,
                claim_id,
            } => {
                if let Some(existing) = existing_claims.get(claim_id) {
                    let expected =
                        format!("{CLAIMS_RELATIVE}/{}/{}", payload.collection_id, file_name);
                    if existing != &expected {
                        let bytes = read_bounded_optional(
                            &roots[0].dir,
                            Path::new(existing),
                            MAX_CLAIM_BYTES,
                            "conflicting claim",
                        )?;
                        observed.insert(
                            payload.relative_path.clone(),
                            bytes.as_deref().map(sha256_digest),
                        );
                        conflict_paths.push(existing.clone());
                        continue;
                    }
                }
                let relative = PathBuf::from(CLAIMS_RELATIVE)
                    .join(&payload.collection_id)
                    .join(file_name);
                (0, relative.clone(), path_locator(&relative)?)
            }
            IncomingKind::Merge { file_name } => {
                payload_destination(collection, &root_indices, MERGE_RELATIVE, file_name)?
            }
            IncomingKind::Suppression => payload_destination(
                collection,
                &root_indices,
                ".hive/knowledge",
                "suppression.yml",
            )?,
        };
        ensure_changed_path_bound(&logical_locator)?;
        let existing = read_bounded_optional(
            &roots[root_index].dir,
            &relative,
            payload_max_bytes(&payload.kind),
            "bundle import destination",
        )?;
        observed.insert(
            payload.relative_path.clone(),
            existing.as_deref().map(sha256_digest),
        );
        let desired = if matches!(payload.kind, IncomingKind::Suppression) {
            merge_suppression(existing.as_deref(), &payload.bytes)?
        } else {
            payload.bytes.clone()
        };
        if existing.as_deref() == Some(desired.as_slice()) {
            unchanged_count += 1;
        } else if existing.is_some() && !matches!(payload.kind, IncomingKind::Suppression) {
            conflict_paths.push(logical_locator);
        } else {
            added_count += 1;
            writes.push(PlannedWrite {
                root_index,
                relative,
                logical_locator,
                expected_bytes: existing,
                bytes: desired,
            });
        }
    }
    writes.sort_by(|left, right| {
        (left.root_index, &left.relative).cmp(&(right.root_index, &right.relative))
    });
    if writes.windows(2).any(|pair| {
        pair[0].root_index == pair[1].root_index && pair[0].relative == pair[1].relative
    }) {
        return Err(WikiError::Verification(
            "bundle merge planned duplicate destination paths".to_owned(),
        ));
    }
    validate_planned_suppressions(&roots, &writes)?;
    conflict_paths.sort();
    conflict_paths.dedup();
    let entry_count = incoming.payloads.len() + 1;
    if added_count + unchanged_count + conflict_paths.len() != entry_count {
        return Err(WikiError::Io(
            "bundle import accounting lost a payload entry".to_owned(),
        ));
    }
    Ok(ImportPlan {
        roots,
        writes,
        entry_count,
        added_count,
        unchanged_count,
        conflict_paths,
        target_state_digest: sha256_digest(
            &serde_json::to_vec(&(root_path, &merged, &existing_claims, &observed))
                .map_err(|error| WikiError::Io(error.to_string()))?,
        ),
        detached_collection_ids: detached,
    })
}

fn payload_destination(
    collection: &CollectionRecord,
    root_indices: &BTreeMap<&str, usize>,
    canonical_parent: &str,
    file_name: &str,
) -> Result<(usize, PathBuf, String), WikiError> {
    if collection.collection_id == USER_ROOT_COLLECTION_ID {
        let relative = PathBuf::from(canonical_parent).join(file_name);
        let locator = path_locator(&relative)?;
        return Ok((0, relative, locator));
    }
    if collection.state == CollectionState::Attached {
        let index = *root_indices
            .get(collection.collection_id.as_str())
            .ok_or_else(|| WikiError::Io("attached collection root was not pinned".to_owned()))?;
        let relative = PathBuf::from(canonical_parent).join(file_name);
        let locator = format!(
            "collections/{}/{}",
            collection.collection_id,
            path_locator(&relative)?
        );
        return Ok((index, relative, locator));
    }
    let relative = PathBuf::from(DETACHED_IMPORT_ROOT)
        .join(&collection.collection_id)
        .join(canonical_parent)
        .join(file_name);
    let locator = path_locator(&relative)?;
    Ok((0, relative, locator))
}

#[allow(clippy::too_many_lines)]
fn activate_import(
    root_path: &Path,
    plan: &mut ImportPlan,
    rebuild_index: bool,
) -> Result<Vec<String>, WikiError> {
    let loaded_manifest = rebuild_index
        .then(|| load_generation_manifest(&plan.roots[0].dir))
        .transpose()?;
    let generation = loaded_manifest
        .as_ref()
        .and_then(|loaded| loaded.manifest.as_ref())
        .map_or(Ok(1), |manifest| {
            manifest.generation.checked_add(1).ok_or_else(|| {
                WikiError::Conflict("RAG generation counter is exhausted".to_owned())
            })
        })?;
    let dirty = PersistentDirtyState {
        schema_version: 2,
        base_generation: loaded_manifest
            .as_ref()
            .and_then(|loaded| loaded.manifest.as_ref())
            .map_or(0, |manifest| manifest.generation),
        base_manifest_digest: loaded_manifest
            .as_ref()
            .and_then(|loaded| loaded.manifest.as_ref())
            .map_or_else(
                || sha256_digest(b"rag-store-empty-manifest-v1"),
                |manifest| manifest.logical_digest.clone(),
            ),
        target_generation: generation,
        entries: plan
            .writes
            .iter()
            .map(|write| PersistentDirtyEntry {
                locator: write.logical_locator.clone(),
                target_digest: sha256_digest(&write.bytes),
                delete: write.bytes.is_empty(),
            })
            .collect(),
    };
    let dirty_bytes = json_bytes(&dirty, "RAG dirty journal")?;

    let derived_file_count = usize::from(rebuild_index) * 3;
    let mut files = Vec::with_capacity(plan.writes.len() + 1 + derived_file_count);
    files.push(TransactionFile {
        root_index: 0,
        logical_locator: RAG_DIRTY_RELATIVE.to_owned(),
        snapshot: crate::CapabilityFileSnapshot::capture(
            &plan.roots[0].dir,
            Path::new(RAG_DIRTY_RELATIVE),
        )?,
    });
    for write in &plan.writes {
        files.push(TransactionFile {
            root_index: write.root_index,
            logical_locator: write.logical_locator.clone(),
            snapshot: crate::CapabilityFileSnapshot::capture(
                &plan.roots[write.root_index].dir,
                &write.relative,
            )?,
        });
    }
    if rebuild_index {
        files.push(TransactionFile {
            root_index: 0,
            logical_locator: SHARED_INDEX_RELATIVE.to_owned(),
            snapshot: crate::CapabilityFileSnapshot::capture(
                &plan.roots[0].dir,
                Path::new(SHARED_INDEX_RELATIVE),
            )?,
        });
        files.push(TransactionFile {
            root_index: 0,
            logical_locator: RAG_MANIFEST_RELATIVE.to_owned(),
            snapshot: crate::CapabilityFileSnapshot::capture(
                &plan.roots[0].dir,
                Path::new(RAG_MANIFEST_RELATIVE),
            )?,
        });
        files.push(TransactionFile {
            root_index: 0,
            logical_locator: RAG_TRUST_RELATIVE.to_owned(),
            snapshot: crate::CapabilityFileSnapshot::capture(
                &plan.roots[0].dir,
                Path::new(RAG_TRUST_RELATIVE),
            )?,
        });
    }
    if !matches!(
        files[0].snapshot.original,
        crate::CapabilityFileState::Missing
    ) {
        return Err(WikiError::Conflict(
            "RAG dirty journal appeared after import planning".to_owned(),
        ));
    }
    for (index, write) in plan.writes.iter().enumerate() {
        if !snapshot_matches_expected(&files[index + 1].snapshot, write.expected_bytes.as_deref()) {
            return Err(WikiError::Conflict(format!(
                "canonical destination changed after import planning: {}",
                write.logical_locator
            )));
        }
    }
    if let Some(loaded_manifest) = &loaded_manifest {
        let manifest_position = plan.writes.len() + 2;
        if !snapshot_matches_expected(
            &files[manifest_position].snapshot,
            loaded_manifest.bytes.as_deref(),
        ) {
            return Err(WikiError::Conflict(
                "RAG generation manifest changed after import planning".to_owned(),
            ));
        }
        let trust_position = manifest_position + 1;
        if !snapshot_matches_expected(
            &files[trust_position].snapshot,
            loaded_manifest.trust_bytes.as_deref(),
        ) {
            return Err(WikiError::Conflict(
                "RAG canonical trust binding changed after import planning".to_owned(),
            ));
        }
    }
    let absent = absent_transaction_directories(&plan.roots, &files)?;
    let operation = activate_files(
        root_path,
        plan,
        &dirty_bytes,
        generation,
        rebuild_index,
        &mut files,
    );
    match operation {
        Ok(mut changed) => {
            for file in &mut files {
                file.snapshot
                    .cleanup_claims(&plan.roots[file.root_index].dir);
            }
            changed.sort();
            changed.dedup();
            Ok(changed)
        }
        Err(operation_error) => {
            let restored = files
                .iter()
                .filter(|file| file.snapshot.modified)
                .map(|file| file.logical_locator.clone())
                .collect::<Vec<_>>();
            let backup_digest = transaction_backup_digest(&files)?;
            let mut rollback_error = None;
            for file in files.iter_mut().rev() {
                if let Err(error) = file.snapshot.rollback(&plan.roots[file.root_index].dir) {
                    rollback_error.get_or_insert(error);
                }
            }
            if rollback_error.is_none() {
                if let Err(error) = cleanup_absent_directories(&plan.roots, &absent) {
                    rollback_error = Some(error);
                }
            }
            if let Some(rollback_error) = rollback_error {
                return Err(WikiError::Io(format!(
                    "bundle activation failed: {operation_error}; rollback failed: {rollback_error}"
                )));
            }
            Err(WikiError::Io(format!(
                "bundle activation failed and {} paths were restored from backup {backup_digest}: {operation_error}",
                restored.len()
            )))
        }
    }
}

fn activate_files(
    root_path: &Path,
    plan: &ImportPlan,
    dirty_bytes: &[u8],
    generation: u64,
    rebuild_index: bool,
    files: &mut [TransactionFile],
) -> Result<Vec<String>, WikiError> {
    files[0]
        .snapshot
        .install_staged(&plan.roots[0].dir, dirty_bytes)?;
    let mut changed = Vec::new();
    for (index, write) in plan.writes.iter().enumerate() {
        if files[index + 1]
            .snapshot
            .install_staged(&plan.roots[write.root_index].dir, &write.bytes)?
        {
            changed.push(write.logical_locator.clone());
        }
    }
    injected_failure_after_canonical_writes()?;
    if !rebuild_index {
        files[0].snapshot.remove(&plan.roots[0].dir)?;
        return Ok(changed);
    }
    let pinned_root = plan.roots[0]
        .dir
        .try_clone()
        .map_err(|error| WikiError::Io(format!("cannot clone pinned import root: {error}")))?;
    let store = RagStore::from_pinned(root_path.to_path_buf(), pinned_root);
    let snapshot = store.load_canonical_snapshot(generation)?;
    let artifact = build_rag_index(&snapshot).map_err(rag_error)?;
    let manifest_bytes = json_bytes(&artifact.manifest, "RAG generation manifest")?;
    let trust_bytes = rag_trust_bytes_for_manifest(&artifact.manifest, &manifest_bytes)?;
    let index_position = plan.writes.len() + 1;
    let manifest_position = index_position + 1;
    let trust_position = manifest_position + 1;
    if files[index_position]
        .snapshot
        .install_staged(&plan.roots[0].dir, &artifact.sqlite_bytes)?
    {
        changed.push(SHARED_INDEX_RELATIVE.to_owned());
    }
    if files[manifest_position]
        .snapshot
        .install_staged(&plan.roots[0].dir, &manifest_bytes)?
    {
        changed.push(RAG_MANIFEST_RELATIVE.to_owned());
    }
    files[0].snapshot.remove(&plan.roots[0].dir)?;
    if files[trust_position]
        .snapshot
        .install_staged(&plan.roots[0].dir, &trust_bytes)?
    {
        changed.push(RAG_TRUST_RELATIVE.to_owned());
    }
    Ok(changed)
}

fn validate_same_collection_truth(
    existing: &CollectionRecord,
    portable: &PortableCollectionRecord,
) -> Result<(), WikiError> {
    let mut portable_existing_aliases = existing
        .aliases
        .iter()
        .filter(|alias| !is_absolute_like(alias))
        .cloned()
        .collect::<Vec<_>>();
    portable_existing_aliases.sort_by(|left, right| {
        folded_alias(left)
            .cmp(&folded_alias(right))
            .then_with(|| left.cmp(right))
    });
    portable_existing_aliases.dedup_by(|left, right| folded_alias(left) == folded_alias(right));
    if existing.kind != portable.kind
        || portable_existing_aliases != portable.aliases
        || existing.source_project_id != portable.source_project_id
        || existing.default_visibility != portable.default_visibility
    {
        return Err(WikiError::Conflict(format!(
            "same collection ID `{}` has divergent portable truth",
            portable.collection_id
        )));
    }
    Ok(())
}

fn validate_portable_registry(
    registry: PortableCollectionRegistry,
) -> Result<PortableCollectionRegistry, WikiError> {
    if registry.schema_version != PORTABLE_SCHEMA_VERSION
        || registry.collections.is_empty()
        || registry.collections.len() > MAX_COLLECTIONS
    {
        return Err(WikiError::Verification(
            "portable collection registry has an unsupported schema or size".to_owned(),
        ));
    }
    if registry
        .collections
        .iter()
        .any(|collection| collection.aliases.len() > MAX_ALIASES)
    {
        return Err(WikiError::Verification(
            "portable collection alias inventory exceeds its bound".to_owned(),
        ));
    }
    for collection in &registry.collections {
        if collection.default_visibility == CollectionVisibility::Confidential {
            return Err(WikiError::Verification(
                "portable collection metadata crosses the confidential boundary".to_owned(),
            ));
        }
        if collection
            .aliases
            .iter()
            .any(|alias| is_absolute_like(alias))
        {
            return Err(WikiError::Verification(
                "portable collection metadata contains a machine-bound alias".to_owned(),
            ));
        }
        if let Some(project_id) = collection.source_project_id.as_deref() {
            validate_bundle_identity("source_project_id", project_id)?;
        }
    }
    let projection = CollectionRegistry {
        schema_version: COLLECTION_SCHEMA_VERSION,
        collections: registry
            .collections
            .iter()
            .map(|collection| CollectionRecord {
                collection_id: collection.collection_id.clone(),
                kind: collection.kind,
                state: CollectionState::Detached,
                aliases: collection.aliases.clone(),
                local_locator: None,
                source_project_id: collection.source_project_id.clone(),
                default_visibility: collection.default_visibility,
            })
            .collect(),
    };
    let canonical = projection
        .canonicalized()
        .map_err(|error| WikiError::Verification(error.to_string()))?;
    let expected = PortableCollectionRegistry {
        schema_version: PORTABLE_SCHEMA_VERSION,
        collections: canonical
            .collections
            .into_iter()
            .map(|collection| PortableCollectionRecord {
                collection_id: collection.collection_id,
                kind: collection.kind,
                aliases: collection.aliases,
                source_project_id: collection.source_project_id,
                default_visibility: collection.default_visibility,
            })
            .collect(),
    };
    if registry != expected {
        return Err(WikiError::Verification(
            "portable collection registry is not canonically sorted".to_owned(),
        ));
    }
    Ok(registry)
}

fn load_registry(root_path: &Path, root: &Dir) -> Result<LoadedRegistry, WikiError> {
    let bytes = read_bounded_optional(
        root,
        Path::new(COLLECTION_REGISTRY_RELATIVE),
        MAX_REGISTRY_BYTES,
        "collection registry",
    )?;
    let registry = if let Some(bytes) = bytes.as_deref() {
        let registry: CollectionRegistry = serde_yaml::from_slice(bytes).map_err(|error| {
            WikiError::Verification(format!("invalid collection registry: {error}"))
        })?;
        let canonical = registry
            .canonicalized()
            .map_err(|error| WikiError::Verification(error.to_string()))?;
        if canonical != registry {
            return Err(WikiError::Verification(
                "collection registry is not in canonical sorted form".to_owned(),
            ));
        }
        canonical
    } else {
        CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![CollectionRecord {
                collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
                kind: CollectionKind::UserRoot,
                state: CollectionState::Attached,
                aliases: vec!["user-root".to_owned()],
                local_locator: Some(root_path.display().to_string()),
                source_project_id: None,
                default_visibility: CollectionVisibility::Shared,
            }],
        }
        .canonicalized()
        .map_err(|error| WikiError::Verification(error.to_string()))?
    };
    validate_local_registry(root_path, &registry)?;
    Ok(LoadedRegistry { registry, bytes })
}

fn wiki_indexing_enabled(root: &Dir) -> Result<bool, WikiError> {
    let Some(bytes) = read_bounded_optional(
        root,
        Path::new(USER_SETUP_RELATIVE),
        MAX_USER_SETUP_BYTES,
        "user setup",
    )?
    else {
        return Ok(true);
    };
    let setup: serde_yaml::Value = serde_yaml::from_slice(&bytes)
        .map_err(|error| WikiError::Verification(format!("invalid user setup: {error}")))?;
    let Some(wiki) = setup.get("wiki") else {
        return Ok(true);
    };
    let Some(enabled) = wiki.get("enabled") else {
        return Ok(true);
    };
    enabled.as_bool().ok_or_else(|| {
        WikiError::Verification("user setup Wiki enablement must be boolean".to_owned())
    })
}

fn validate_local_registry(
    root_path: &Path,
    registry: &CollectionRegistry,
) -> Result<(), WikiError> {
    let user_roots = registry
        .collections
        .iter()
        .filter(|collection| collection.kind == CollectionKind::UserRoot)
        .collect::<Vec<_>>();
    if user_roots.len() != 1
        || user_roots[0].collection_id != USER_ROOT_COLLECTION_ID
        || user_roots[0].state != CollectionState::Attached
        || user_roots[0].local_locator.as_deref() != Some(root_path.display().to_string().as_str())
    {
        return Err(WikiError::Conflict(
            "user-root collection does not match the pinned destination root".to_owned(),
        ));
    }
    let mut locators = BTreeSet::new();
    for collection in registry
        .collections
        .iter()
        .filter(|collection| collection.state == CollectionState::Attached)
    {
        let locator = collection.local_locator.as_deref().ok_or_else(|| {
            WikiError::Verification("attached collection is missing a locator".to_owned())
        })?;
        let key = if cfg!(windows) {
            locator.to_lowercase()
        } else {
            locator.to_owned()
        };
        if !locators.insert(key) {
            return Err(WikiError::Conflict(
                "multiple collections share one attached local locator".to_owned(),
            ));
        }
    }
    Ok(())
}

fn registry_bytes(registry: &CollectionRegistry) -> Result<Vec<u8>, WikiError> {
    let bytes = serde_yaml::to_string(registry)
        .map_err(|error| WikiError::Io(format!("cannot serialize collection registry: {error}")))?
        .into_bytes();
    if bytes.len() > MAX_REGISTRY_BYTES {
        return Err(WikiError::InvalidInput(
            "collection registry exceeds the 1 MiB limit".to_owned(),
        ));
    }
    Ok(bytes)
}

fn canonical_portable_registry_bytes(
    registry: &PortableCollectionRegistry,
) -> Result<Vec<u8>, WikiError> {
    let mut bytes = serde_json::to_vec_pretty(registry).map_err(|error| {
        WikiError::Io(format!(
            "cannot serialize portable collection registry: {error}"
        ))
    })?;
    bytes.push(b'\n');
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(WikiError::InvalidInput(
            "portable collection registry exceeds the 1 MiB limit".to_owned(),
        ));
    }
    Ok(bytes)
}

fn canonical_suppression_bytes(bytes: &[u8]) -> Result<Vec<u8>, WikiError> {
    let mut ledger: crate::SuppressionLedger = serde_yaml::from_slice(bytes)
        .map_err(|error| WikiError::Verification(format!("invalid suppression ledger: {error}")))?;
    if ledger.schema_version != 1 {
        return Err(WikiError::Verification(
            "suppression schema_version must be 1".to_owned(),
        ));
    }
    for entry in &ledger.entries {
        crate::validate_suppression_entry(entry)
            .map_err(|error| WikiError::Verification(error.to_string()))?;
    }
    ledger.entries.sort_by(|left, right| {
        (&left.fingerprint, &left.source_locator, &left.timestamp).cmp(&(
            &right.fingerprint,
            &right.source_locator,
            &right.timestamp,
        ))
    });
    ledger.entries.dedup();
    serde_yaml::to_string(&ledger)
        .map(String::into_bytes)
        .map_err(|error| WikiError::Io(format!("cannot serialize suppression ledger: {error}")))
}

fn merge_suppression(existing: Option<&[u8]>, incoming: &[u8]) -> Result<Vec<u8>, WikiError> {
    let incoming = canonical_suppression_bytes(incoming)?;
    let Some(existing) = existing else {
        return Ok(incoming);
    };
    let canonical_existing = canonical_suppression_bytes(existing)?;
    if canonical_existing != existing {
        return Err(WikiError::Verification(
            "destination suppression ledger is not canonical YAML".to_owned(),
        ));
    }
    let mut destination: crate::SuppressionLedger = serde_yaml::from_slice(existing)
        .map_err(|error| WikiError::Verification(format!("invalid suppression ledger: {error}")))?;
    let source: crate::SuppressionLedger = serde_yaml::from_slice(&incoming)
        .map_err(|error| WikiError::Verification(format!("invalid suppression ledger: {error}")))?;
    destination.entries.extend(source.entries);
    destination.entries.sort_by(|left, right| {
        (&left.fingerprint, &left.source_locator, &left.timestamp).cmp(&(
            &right.fingerprint,
            &right.source_locator,
            &right.timestamp,
        ))
    });
    destination.entries.dedup();
    serde_yaml::to_string(&destination)
        .map(String::into_bytes)
        .map_err(|error| WikiError::Io(format!("cannot serialize suppression merge: {error}")))
}

fn validate_planned_suppressions(
    roots: &[TargetRoot],
    writes: &[PlannedWrite],
) -> Result<(), WikiError> {
    let mut groups = BTreeSet::<(usize, PathBuf)>::new();
    for write in writes {
        if write
            .relative
            .parent()
            .is_some_and(|parent| parent.ends_with(Path::new(WIKI_RELATIVE)))
        {
            let collection_prefix = write.relative.ancestors().nth(4).ok_or_else(|| {
                WikiError::Verification("planned Wiki destination is malformed".to_owned())
            })?;
            groups.insert((write.root_index, collection_prefix.to_path_buf()));
        } else if write.relative.ends_with(Path::new(SUPPRESSION_RELATIVE))
            || write.relative.file_name() == Some(OsStr::new("suppression.yml"))
        {
            let collection_prefix = write.relative.ancestors().nth(3).ok_or_else(|| {
                WikiError::Verification("planned suppression destination is malformed".to_owned())
            })?;
            groups.insert((write.root_index, collection_prefix.to_path_buf()));
        }
    }
    for (root_index, collection_prefix) in groups {
        let wiki_relative = collection_prefix.join(WIKI_RELATIVE);
        let planned_wiki = writes
            .iter()
            .filter(|candidate| {
                candidate.root_index == root_index
                    && candidate.relative.parent() == Some(wiki_relative.as_path())
            })
            .map(|candidate| parse_page_bytes(&candidate.bytes, &candidate.logical_locator))
            .collect::<Result<Vec<_>, _>>()?;
        let suppression_relative = collection_prefix.join(SUPPRESSION_RELATIVE);
        let suppression_bytes = if let Some(write) = writes.iter().find(|candidate| {
            candidate.root_index == root_index && candidate.relative == suppression_relative
        }) {
            Some(write.bytes.clone())
        } else {
            read_bounded_optional(
                &roots[root_index].dir,
                &suppression_relative,
                MAX_SUPPRESSION_BYTES,
                "suppression ledger",
            )?
        };
        let Some(suppression_bytes) = suppression_bytes else {
            continue;
        };
        let ledger: crate::SuppressionLedger =
            serde_yaml::from_slice(&suppression_bytes).map_err(|error| {
                WikiError::Verification(format!("invalid planned suppression ledger: {error}"))
            })?;
        let existing_pages = wiki_inventory(&roots[root_index].dir, &wiki_relative)?;
        for entry in ledger.entries {
            let active_page = existing_pages.iter().any(|(id, digest)| {
                digest == &entry.fingerprint
                    || entry.source_locator.strip_prefix("wiki:") == Some(id.as_str())
            }) || planned_wiki.iter().any(|page| {
                page.content_digest == entry.fingerprint
                    || entry.source_locator.strip_prefix("wiki:")
                        == Some(page.frontmatter.id.as_str())
            });
            if active_page
                || raw_suppression_is_active(&roots[root_index].dir, &collection_prefix, &entry)?
            {
                return Err(WikiError::Conflict(format!(
                    "suppression cannot coexist with active canonical content: {}",
                    entry.source_locator
                )));
            }
        }
    }
    Ok(())
}

fn wiki_inventory(root: &Dir, wiki_relative: &Path) -> Result<BTreeMap<String, String>, WikiError> {
    let Some(wiki) = open_optional_dir(root, wiki_relative, "Wiki inventory")? else {
        return Ok(BTreeMap::new());
    };
    let mut inventory = BTreeMap::new();
    for name in directory_names(&wiki, "Wiki inventory")? {
        let metadata = wiki
            .symlink_metadata(&name)
            .map_err(|error| WikiError::Io(format!("cannot inspect Wiki inventory: {error}")))?;
        if metadata.is_dir() {
            continue;
        }
        if !metadata.is_file() {
            return Err(WikiError::Verification(
                "Wiki inventory contains a symlink or special file".to_owned(),
            ));
        }
        let file_name = utf8_name(&name, "Wiki filename")?;
        if Path::new(&file_name)
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("md")
            || matches!(file_name.as_str(), "index.md" | "log.md")
        {
            continue;
        }
        let bytes = read_named_file(&wiki, &name, MAX_WIKI_BYTES, "Wiki page")?;
        let locator = path_locator(&wiki_relative.join(&file_name))?;
        let page = parse_page_bytes(&bytes, &locator)?;
        inventory.insert(page.frontmatter.id, page.content_digest);
    }
    Ok(inventory)
}

fn raw_suppression_is_active(
    root: &Dir,
    collection_prefix: &Path,
    entry: &crate::SuppressionEntry,
) -> Result<bool, WikiError> {
    let Some((relative, fingerprint)) = crate::parse_raw_locator(&entry.source_locator) else {
        return Ok(false);
    };
    let Some(bytes) = read_bounded_optional(
        root,
        &collection_prefix.join(relative),
        MAX_WIKI_BYTES,
        "suppressed Raw revision",
    )?
    else {
        return Ok(false);
    };
    Ok(sha256_digest(&bytes) == fingerprint || sha256_digest(&bytes) == entry.fingerprint)
}

fn existing_claim_locations(root: &Dir) -> Result<BTreeMap<String, String>, WikiError> {
    let Some(claims) = open_optional_dir(root, Path::new(CLAIMS_RELATIVE), "Claims directory")?
    else {
        return Ok(BTreeMap::new());
    };
    let mut locations = BTreeMap::new();
    let mut count = 0_usize;
    for collection_name in directory_names(&claims, "Claims directory")? {
        let metadata = claims
            .symlink_metadata(&collection_name)
            .map_err(|error| WikiError::Io(format!("cannot inspect Claims entry: {error}")))?;
        if !metadata.is_dir() {
            return Err(WikiError::Verification(
                "Claims entries must be no-follow directories".to_owned(),
            ));
        }
        let collection_id = utf8_name(&collection_name, "claim collection ID")?;
        let collection = claims
            .open_dir_nofollow(&collection_name)
            .map_err(|error| {
                WikiError::Conflict(format!("cannot open claim collection no-follow: {error}"))
            })?;
        for file_name in directory_names(&collection, "claim collection")? {
            count = count.checked_add(1).ok_or_else(|| {
                WikiError::Verification("claim inventory count overflow".to_owned())
            })?;
            if count > 10_000 {
                return Err(WikiError::Verification(
                    "claim inventory exceeds the 10,000 entry import bound".to_owned(),
                ));
            }
            let metadata = collection.symlink_metadata(&file_name).map_err(|error| {
                WikiError::Io(format!("cannot inspect canonical claim: {error}"))
            })?;
            if !metadata.is_file() {
                return Err(WikiError::Verification(
                    "canonical Claims inventory contains a symlink or special file".to_owned(),
                ));
            }
            let bytes = read_named_file(&collection, &file_name, MAX_CLAIM_BYTES, "claim")?;
            crate::reject_likely_credentials(&bytes).map_err(|error| {
                WikiError::Verification(format!(
                    "canonical claim contains likely sensitive material: {error}"
                ))
            })?;
            let file_name = utf8_name(&file_name, "claim filename")?;
            if Path::new(&file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("md")
            {
                return Err(WikiError::Verification(
                    "claim collection contains a non-Markdown entry".to_owned(),
                ));
            }
            let stem = Path::new(&file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| WikiError::Verification("invalid claim filename".to_owned()))?;
            let locator = format!("{CLAIMS_RELATIVE}/{collection_id}/{file_name}");
            let claim = parse_claim_markdown(
                &locator,
                std::str::from_utf8(&bytes).map_err(|error| {
                    WikiError::Verification(format!("canonical claim is not UTF-8: {error}"))
                })?,
            )
            .map_err(rag_error)?;
            if claim.collection_id != collection_id || claim.claim_id != stem {
                return Err(WikiError::Verification(format!(
                    "canonical claim path does not match its IDs: {locator}"
                )));
            }
            if locations.insert(stem.to_owned(), locator).is_some() {
                return Err(WikiError::Verification(format!(
                    "duplicate canonical claim ID `{stem}`"
                )));
            }
        }
    }
    Ok(locations)
}

fn parse_portable_payload_path(
    relative: &str,
) -> Result<(String, String, Option<String>), WikiError> {
    let components = relative.split('/').collect::<Vec<_>>();
    if components.len() == 6
        && components[..3] == [".hive", "portable", "collections"]
        && matches!(components[4], "Wiki" | "Claims" | "Merge")
        && Path::new(components[5])
            .extension()
            .and_then(|extension| extension.to_str())
            == Some(if components[4] == "Merge" {
                "json"
            } else {
                "md"
            })
    {
        return Ok((
            components[3].to_owned(),
            components[4].to_owned(),
            Some(components[5].to_owned()),
        ));
    }
    if components.len() == 5
        && components[..3] == [".hive", "portable", "collections"]
        && components[4] == "suppression.yml"
    {
        return Ok((components[3].to_owned(), components[4].to_owned(), None));
    }
    Err(WikiError::Verification(format!(
        "unsupported portable payload path `{relative}`"
    )))
}

fn portable_wiki_path(collection_id: &str, file_name: &str) -> String {
    format!(".hive/portable/collections/{collection_id}/Wiki/{file_name}")
}

fn portable_claim_path(collection_id: &str, file_name: &str) -> String {
    format!(".hive/portable/collections/{collection_id}/Claims/{file_name}")
}

fn portable_merge_path(collection_id: &str, file_name: &str) -> String {
    format!(".hive/portable/collections/{collection_id}/Merge/{file_name}")
}

fn portable_suppression_path(collection_id: &str) -> String {
    format!(".hive/portable/collections/{collection_id}/suppression.yml")
}

fn claim_has_absolute_locator(claim: &crate::rag::CanonicalClaim) -> bool {
    is_absolute_like(&claim.provenance.locator)
        || claim.sources.iter().any(|source| is_absolute_like(source))
}

fn validate_merge_provenance(bytes: &[u8]) -> Result<(), WikiError> {
    let provenance: MergeProvenance = serde_json::from_slice(bytes)
        .map_err(|error| WikiError::Verification(format!("invalid merge provenance: {error}")))?;
    if provenance.schema_version != 1
        || !is_sha256(&provenance.original_sha256)
        || provenance.primary_path.is_empty()
        || provenance.original_path.is_empty()
        || provenance.original_markdown.is_empty()
        || sha256_digest(provenance.original_markdown.as_bytes()) != provenance.original_sha256
    {
        return Err(WikiError::Verification(
            "merge provenance identity is invalid".to_owned(),
        ));
    }
    let (_, primary_leaf, Some(primary_file)) =
        parse_portable_payload_path(&provenance.primary_path)?
    else {
        return Err(WikiError::Verification(
            "merge provenance primary path is invalid".to_owned(),
        ));
    };
    let (_, original_leaf, Some(original_file)) =
        parse_portable_payload_path(&provenance.original_path)?
    else {
        return Err(WikiError::Verification(
            "merge provenance original path is invalid".to_owned(),
        ));
    };
    if primary_leaf != "Wiki" || original_leaf != "Wiki" {
        return Err(WikiError::Verification(
            "merge provenance must refer to portable Wiki paths".to_owned(),
        ));
    }
    parse_page_bytes(
        provenance.original_markdown.as_bytes(),
        &format!("{WIKI_RELATIVE}/{original_file}"),
    )?;
    let _ = primary_file;
    Ok(())
}

fn is_absolute_like(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with('\\')
        || value.as_bytes().get(1) == Some(&b':')
        || Path::new(value).is_absolute()
}

fn validate_bundle_identity(label: &str, value: &str) -> Result<(), WikiError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(WikiError::InvalidInput(format!(
            "{label} is not a portable bundle identity"
        )));
    }
    Ok(())
}

fn bundle_logical_digest_inputs(entries: &[BundleEntryInput]) -> Result<String, WikiError> {
    let mut inventory = entries
        .iter()
        .filter(|entry| is_portable_classification(entry.classification))
        .map(|entry| {
            (
                entry.relative_path.clone(),
                entry.classification,
                u64::try_from(entry.bytes.len()).unwrap_or(u64::MAX),
                sha256_digest(&entry.bytes),
            )
        })
        .collect::<Vec<_>>();
    inventory.sort_by(|left, right| left.0.cmp(&right.0));
    serde_json::to_vec(&("hive-bundle-logical-v1", inventory))
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| WikiError::Io(format!("cannot digest bundle inventory: {error}")))
}

fn bundle_logical_digest_validated(plan: &ValidatedBundlePlan) -> Result<String, WikiError> {
    let inventory = plan
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.relative_path().to_owned(),
                entry.classification(),
                u64::try_from(entry.bytes().len()).unwrap_or(u64::MAX),
                entry.sha256().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    serde_json::to_vec(&("hive-bundle-logical-v1", inventory))
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| WikiError::Io(format!("cannot digest bundle inventory: {error}")))
}

fn is_portable_classification(classification: PortableEntryClassification) -> bool {
    matches!(
        classification,
        PortableEntryClassification::CanonicalMarkdown
            | PortableEntryClassification::PortableMetadata
            | PortableEntryClassification::Suppression
            | PortableEntryClassification::Provenance
    )
}

fn load_generation_manifest(root: &Dir) -> Result<LoadedGenerationManifest, WikiError> {
    let manifest_bytes = read_bounded_optional(
        root,
        Path::new(RAG_MANIFEST_RELATIVE),
        MAX_MANIFEST_BYTES,
        "RAG generation manifest",
    )?;
    let trust_bytes = read_bounded_optional(
        root,
        Path::new(RAG_TRUST_RELATIVE),
        MAX_RAG_TRUST_BYTES,
        "RAG canonical trust binding",
    )?;
    let (manifest_bytes, trust_bytes) = match (manifest_bytes, trust_bytes) {
        (Some(manifest_bytes), Some(trust_bytes)) => (manifest_bytes, trust_bytes),
        (None, None) => {
            return Ok(LoadedGenerationManifest {
                manifest: None,
                bytes: None,
                trust_bytes: None,
            });
        }
        _ => {
            return Err(WikiError::Verification(
                "RAG generation is missing its independent canonical trust binding".to_owned(),
            ));
        }
    };
    let manifest: GenerationManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|error| {
            WikiError::Verification(format!("invalid RAG generation manifest: {error}"))
        })?;
    if manifest.schema_version != RAG_SCHEMA_VERSION
        || manifest.generation == 0
        || !is_sha256(&manifest.logical_digest)
        || !is_sha256(&manifest.sqlite_digest)
    {
        return Err(WikiError::Verification(
            "RAG generation manifest has invalid lineage".to_owned(),
        ));
    }
    verify_rag_trust_bytes(&manifest, &manifest_bytes, &trust_bytes)?;
    Ok(LoadedGenerationManifest {
        manifest: Some(manifest),
        bytes: Some(manifest_bytes),
        trust_bytes: Some(trust_bytes),
    })
}

fn reject_dirty(root: &Dir) -> Result<(), WikiError> {
    if read_bounded_optional(
        root,
        Path::new(RAG_DIRTY_RELATIVE),
        MAX_DIRTY_BYTES,
        "RAG dirty journal",
    )?
    .is_some()
    {
        return Err(WikiError::Verification(
            "RAG store has an interrupted canonical write; rebuild before import".to_owned(),
        ));
    }
    let _ = read_bounded_optional(
        root,
        Path::new(SHARED_INDEX_RELATIVE),
        MAX_SERIALIZED_INDEX_BYTES,
        "RAG SQLite index",
    )?;
    Ok(())
}

fn json_bytes(value: &impl Serialize, label: &str) -> Result<Vec<u8>, WikiError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| WikiError::Io(format!("cannot serialize {label}: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn path_locator(path: &Path) -> Result<String, WikiError> {
    let locator = path.to_str().ok_or_else(|| {
        WikiError::Verification("canonical destination path is not UTF-8".to_owned())
    })?;
    if locator.contains('\\') {
        Ok(locator.replace('\\', "/"))
    } else {
        Ok(locator.to_owned())
    }
}

fn ensure_changed_path_bound(locator: &str) -> Result<(), WikiError> {
    if locator.is_empty()
        || locator.len() > MAX_CHANGED_PATH_BYTES
        || locator.starts_with('/')
        || locator.contains('\\')
        || locator.split('/').any(|component| component == "..")
    {
        return Err(WikiError::Verification(format!(
            "import destination locator is unsafe or exceeds {MAX_CHANGED_PATH_BYTES} bytes"
        )));
    }
    Ok(())
}

fn payload_max_bytes(kind: &IncomingKind) -> usize {
    match kind {
        IncomingKind::Wiki { .. } => MAX_WIKI_BYTES,
        IncomingKind::Claim { .. } => MAX_CLAIM_BYTES,
        IncomingKind::Merge { .. } => MAX_MERGE_PROVENANCE_BYTES,
        IncomingKind::Suppression => MAX_SUPPRESSION_BYTES,
    }
}

fn absent_transaction_directories(
    roots: &[TargetRoot],
    files: &[TransactionFile],
) -> Result<Vec<(usize, PathBuf)>, WikiError> {
    let mut absent = BTreeSet::new();
    for file in files {
        let mut parent = file.snapshot.relative.parent();
        while let Some(directory) = parent {
            if directory.as_os_str().is_empty()
                || crate::capability_directory_exists(&roots[file.root_index].dir, directory)?
            {
                break;
            }
            absent.insert((file.root_index, directory.to_path_buf()));
            parent = directory.parent();
        }
    }
    let mut absent = absent.into_iter().collect::<Vec<_>>();
    absent.sort_by(|left, right| {
        right
            .1
            .components()
            .count()
            .cmp(&left.1.components().count())
            .then_with(|| left.cmp(right))
    });
    Ok(absent)
}

fn snapshot_matches_expected(
    snapshot: &crate::CapabilityFileSnapshot,
    expected: Option<&[u8]>,
) -> bool {
    match (&snapshot.original, expected) {
        (crate::CapabilityFileState::Missing, None) => true,
        (crate::CapabilityFileState::File { bytes, .. }, Some(expected)) => bytes == expected,
        (crate::CapabilityFileState::Missing, Some(_))
        | (crate::CapabilityFileState::File { .. }, None) => false,
    }
}

fn cleanup_absent_directories(
    roots: &[TargetRoot],
    absent: &[(usize, PathBuf)],
) -> Result<(), WikiError> {
    for (root_index, directory) in absent {
        crate::remove_capability_empty_directory(&roots[*root_index].dir, directory)?;
    }
    Ok(())
}

fn transaction_backup_digest(files: &[TransactionFile]) -> Result<String, WikiError> {
    let inventory = files
        .iter()
        .map(|file| {
            let digest = match &file.snapshot.original {
                crate::CapabilityFileState::Missing => None,
                crate::CapabilityFileState::File { bytes, .. } => Some(sha256_digest(bytes)),
            };
            (file.logical_locator.clone(), digest)
        })
        .collect::<Vec<_>>();
    serde_json::to_vec(&("hive-bundle-backup-v1", inventory))
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| WikiError::Io(format!("cannot digest transaction backup: {error}")))
}

fn read_bounded_optional(
    root: &Dir,
    relative: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<Option<Vec<u8>>, WikiError> {
    let Some((parent, name)) = crate::capability_parent(root, relative, false)? else {
        return Ok(None);
    };
    match parent.symlink_metadata(&name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(WikiError::Io(format!("cannot inspect {label}: {error}"))),
        Ok(metadata) if metadata.is_file() => {
            read_named_file(&parent, &name, max_bytes, label).map(Some)
        }
        Ok(_) => Err(WikiError::Conflict(format!(
            "{label} must be a regular no-follow file"
        ))),
    }
}

fn read_named_file(
    parent: &Dir,
    name: &OsStr,
    max_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, WikiError> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = parent
        .open_with(name, &options)
        .map_err(|error| WikiError::Conflict(format!("cannot open {label} no-follow: {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect opened {label}: {error}")))?;
    if !metadata.is_file() || metadata.len() > max_bytes as u64 {
        return Err(WikiError::Verification(format!(
            "{label} is not a bounded regular file"
        )));
    }
    let mut bytes = Vec::new();
    file.take(u64::try_from(max_bytes + 1).expect("byte bound fits u64"))
        .read_to_end(&mut bytes)
        .map_err(|error| WikiError::Io(format!("cannot read {label}: {error}")))?;
    if bytes.len() > max_bytes {
        return Err(WikiError::Verification(format!(
            "{label} exceeds {max_bytes} bytes"
        )));
    }
    Ok(bytes)
}

fn open_optional_dir(root: &Dir, relative: &Path, label: &str) -> Result<Option<Dir>, WikiError> {
    let Some((parent, name)) = crate::capability_parent(root, relative, false)? else {
        return Ok(None);
    };
    match parent.open_dir_nofollow(&name) {
        Ok(directory) => Ok(Some(directory)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(WikiError::Conflict(format!(
            "cannot open {label} no-follow: {error}"
        ))),
    }
}

fn directory_names(directory: &Dir, label: &str) -> Result<Vec<OsString>, WikiError> {
    let mut names = Vec::new();
    for entry in directory
        .entries()
        .map_err(|error| WikiError::Io(format!("cannot scan {label}: {error}")))?
    {
        let entry =
            entry.map_err(|error| WikiError::Io(format!("cannot scan {label}: {error}")))?;
        if names.len() == MAX_DIRECTORY_ENTRIES {
            return Err(WikiError::Verification(format!(
                "{label} exceeds the {MAX_DIRECTORY_ENTRIES} entry bound"
            )));
        }
        names.push(entry.file_name());
    }
    names.sort();
    Ok(names)
}

fn utf8_name(name: &OsStr, label: &str) -> Result<String, WikiError> {
    name.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| WikiError::Verification(format!("{label} is not UTF-8")))
}

fn pin_absolute_directory(path: &Path, label: &str) -> Result<Dir, WikiError> {
    if !path.is_absolute() {
        return Err(WikiError::InvalidInput(format!("{label} must be absolute")));
    }
    let expected = Dir::open_ambient_dir(path, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot capture {label}: {error}")))?
        .dir_metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect {label}: {error}")))?;
    let mut filesystem_root = PathBuf::new();
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => filesystem_root.push(prefix.as_os_str()),
            Component::RootDir => filesystem_root.push(component.as_os_str()),
            Component::Normal(component) => components.push(component.to_os_string()),
            Component::CurDir | Component::ParentDir => {
                return Err(WikiError::InvalidInput(format!(
                    "{label} is not lexically safe"
                )));
            }
        }
    }
    if filesystem_root.as_os_str().is_empty() {
        return Err(WikiError::InvalidInput(format!("{label} must be absolute")));
    }
    let mut current =
        Dir::open_ambient_dir(&filesystem_root, ambient_authority()).map_err(|error| {
            WikiError::Io(format!(
                "cannot open {label} filesystem root {}: {error}",
                filesystem_root.display()
            ))
        })?;
    for component in components {
        let metadata = current.symlink_metadata(&component).map_err(|error| {
            WikiError::Conflict(format!("cannot inspect {label} component: {error}"))
        })?;
        if !metadata.is_dir() {
            return Err(WikiError::Conflict(format!(
                "{label} contains a symlink or non-directory component"
            )));
        }
        let next = current.open_dir_nofollow(&component).map_err(|error| {
            WikiError::Conflict(format!("cannot open {label} component no-follow: {error}"))
        })?;
        let actual = next
            .dir_metadata()
            .map_err(|error| WikiError::Io(format!("cannot inspect pinned {label}: {error}")))?;
        if (
            CapMetadataExt::dev(&metadata),
            CapMetadataExt::ino(&metadata),
        ) != (CapMetadataExt::dev(&actual), CapMetadataExt::ino(&actual))
        {
            return Err(WikiError::Conflict(format!(
                "{label} changed while its capability was pinned"
            )));
        }
        current = next;
    }
    let actual = current
        .dir_metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect pinned {label}: {error}")))?;
    if (
        CapMetadataExt::dev(&expected),
        CapMetadataExt::ino(&expected),
    ) != (CapMetadataExt::dev(&actual), CapMetadataExt::ino(&actual))
    {
        return Err(WikiError::Conflict(format!(
            "{label} changed while its absolute path was pinned"
        )));
    }
    Ok(current)
}

fn classification_error(path: &str) -> WikiError {
    WikiError::Verification(format!(
        "portable payload path and classification disagree: {path}"
    ))
}

fn bundle_io_error(error: BundleIoError) -> WikiError {
    match error {
        BundleIoError::InvalidInput(message) => WikiError::InvalidInput(message),
        BundleIoError::Conflict(message) => WikiError::Conflict(message),
        BundleIoError::Io(message) => WikiError::Io(message),
        BundleIoError::Portable(error) => WikiError::Verification(error.to_string()),
    }
}

fn rag_error(error: crate::rag::RagError) -> WikiError {
    match error {
        crate::rag::RagError::InvalidInput(message) => WikiError::InvalidInput(message),
        crate::rag::RagError::Conflict(message) => WikiError::Conflict(message),
        crate::rag::RagError::RepairRequired(message) => WikiError::Verification(message),
        crate::rag::RagError::Io(message) => WikiError::Io(message),
        crate::rag::RagError::Sqlite(message) => WikiError::Sqlite(message),
    }
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

#[cfg(test)]
type AfterCanonicalWritesHook = Box<dyn FnOnce() -> Result<(), WikiError>>;

#[cfg(test)]
std::thread_local! {
    static FAIL_AFTER_CANONICAL_WRITES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static AFTER_CANONICAL_WRITES_HOOK: std::cell::RefCell<Option<AfterCanonicalWritesHook>> =
        std::cell::RefCell::new(None);
}

#[cfg(test)]
fn injected_failure_after_canonical_writes() -> Result<(), WikiError> {
    FAIL_AFTER_CANONICAL_WRITES.with(|failure| {
        if failure.replace(false) {
            Err(WikiError::Io(
                "injected bundle failure after canonical writes".to_owned(),
            ))
        } else {
            Ok(())
        }
    })?;
    AFTER_CANONICAL_WRITES_HOOK.with(|hook| {
        let callback = hook.borrow_mut().take();
        callback.map_or(Ok(()), |callback| callback())
    })
}

#[cfg(not(test))]
#[allow(clippy::unnecessary_wraps)]
fn injected_failure_after_canonical_writes() -> Result<(), WikiError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle_io::BundlePublishMode;
    use crate::collection::{CollectionKind, CollectionState, CollectionVisibility};
    use crate::rag::{
        claim_digest, render_claim_markdown, AssertionStatus, CanonicalClaim, ClaimKind,
        ClaimProvenance, RememberSourceKind,
    };
    use crate::scan::{
        ClaimEvidence, ClaimEvidenceKind, ScanClaimMetadata, ScanPromotionStatus, ScanReviewStatus,
    };
    use crate::store::CollectionRegistration;
    use std::fs;
    use tempfile::TempDir;

    const PROJECT_ID: &str = "project-alpha";

    fn collection_id(character: char) -> String {
        format!("collection-{}", character.to_string().repeat(64))
    }

    fn claim_id(character: char) -> String {
        format!("claim-{}", character.to_string().repeat(64))
    }

    fn empty_suppression() -> Vec<u8> {
        canonical_suppression_bytes(b"schema_version: 1\nentries: []\n")
            .expect("canonical empty suppression")
    }

    fn canonical_only_root() -> TempDir {
        let temporary = tempfile::tempdir().expect("temporary root");
        fs::create_dir_all(temporary.path().join(".hive/config")).expect("config directory");
        fs::create_dir_all(temporary.path().join(WIKI_RELATIVE)).expect("Wiki directory");
        fs::create_dir_all(temporary.path().join(".hive/knowledge/Raw")).expect("Raw directory");
        fs::write(
            temporary.path().join(SUPPRESSION_RELATIVE),
            empty_suppression(),
        )
        .expect("suppression ledger");
        temporary
    }

    fn initialized_root() -> TempDir {
        let temporary = canonical_only_root();
        RagStore::open(temporary.path())
            .expect("open store")
            .ensure_registry()
            .expect("initialize registry and index");
        temporary
    }

    fn collection_root() -> TempDir {
        let temporary = tempfile::tempdir().expect("temporary collection");
        fs::create_dir_all(temporary.path().join(WIKI_RELATIVE)).expect("Wiki directory");
        fs::create_dir_all(temporary.path().join(".hive/knowledge/Raw")).expect("Raw directory");
        fs::write(
            temporary.path().join(SUPPRESSION_RELATIVE),
            empty_suppression(),
        )
        .expect("suppression ledger");
        temporary
    }

    fn wiki_bytes(id: &str, body: &str) -> Vec<u8> {
        format!(
            "---\nschema_version: 1\nid: {id}\nkind: concept\nsummary: {id} summary\ntags:\n- stable\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: '2026-08-01T00:00:00Z'\nupdated_at: '2026-08-01T00:00:00Z'\n---\n\n{body}\n"
        )
        .into_bytes()
    }

    fn write_wiki(root: &Path, id: &str, body: &str) {
        fs::write(
            root.join(WIKI_RELATIVE).join(format!("{id}.md")),
            wiki_bytes(id, body),
        )
        .expect("write Wiki page");
    }

    fn canonical_claim(
        collection_id: &str,
        id_character: char,
        visibility: RagVisibility,
        provenance_locator: &str,
    ) -> CanonicalClaim {
        let claim_id = claim_id(id_character);
        let locator = format!("{CLAIMS_RELATIVE}/{collection_id}/{claim_id}.md");
        let mut claim = CanonicalClaim {
            claim_id,
            claim_key: format!("portable-{id_character}"),
            collection_id: collection_id.to_owned(),
            document_id: None,
            locator,
            kind: ClaimKind::Decision,
            status: AssertionStatus::Verified,
            visibility,
            normalized_fact: format!("Portable fact {id_character} is verified."),
            provenance: ClaimProvenance {
                source_kind: RememberSourceKind::ReviewedArtifact,
                summary: format!("Reviewed portable fact {id_character}"),
                locator: provenance_locator.to_owned(),
                digest: sha256_digest(provenance_locator.as_bytes()),
            },
            scan_metadata: None,
            revision: 1,
            sources: vec![provenance_locator.to_owned()],
            supersedes: Vec::new(),
            replacement: None,
            observed_at: Some("2026-08-01T00:00:00Z".to_owned()),
            verified_at: Some("2026-08-01T00:01:00Z".to_owned()),
            digest: String::new(),
        };
        claim.digest = claim_digest(&claim);
        claim
    }

    fn canonical_scan_claim(collection_id: &str) -> CanonicalClaim {
        let inventory_digest = sha256_digest(b"portable scan inventory");
        let evidence_digest = sha256_digest(b"portable reviewed evidence");
        let mut claim = canonical_claim(
            collection_id,
            'e',
            RagVisibility::ProjectPrivate,
            "docs/convention.md",
        );
        claim.claim_key = "scan.portable-convention".to_owned();
        claim.normalized_fact = "The reviewed convention is portable.".to_owned();
        claim.provenance = ClaimProvenance {
            source_kind: RememberSourceKind::ReviewedArtifact,
            summary: "Agent-reviewed portable scan convention".to_owned(),
            locator: format!("scan-inventory:{inventory_digest}"),
            digest: inventory_digest,
        };
        claim.sources = vec![format!("docs/convention.md#{evidence_digest}")];
        claim.scan_metadata = Some(ScanClaimMetadata {
            review_id: "portable-convention".to_owned(),
            version: Some("16.0.0".to_owned()),
            source_revision: Some("release-2026-08".to_owned()),
            applicability: Some("Compatible web projects".to_owned()),
            evidence: vec![ClaimEvidence {
                locator: "docs/convention.md".to_owned(),
                content_digest: evidence_digest,
                kind: ClaimEvidenceKind::Document,
            }],
            review_status: ScanReviewStatus::AgentReviewed,
            global_promotion_candidate: true,
            promotion_status: ScanPromotionStatus::PendingReview,
        });
        claim.digest = claim_digest(&claim);
        claim
    }

    fn write_claim(root: &Path, claim: &CanonicalClaim) {
        let parent = root.join(CLAIMS_RELATIVE).join(&claim.collection_id);
        fs::create_dir_all(&parent).expect("claim directory");
        fs::write(
            parent.join(format!("{}.md", claim.claim_id)),
            render_claim_markdown(claim)
                .expect("render claim")
                .into_bytes(),
        )
        .expect("write claim");
    }

    fn project_bundle(bundle_path: &Path) -> (TempDir, TempDir, String) {
        let source = initialized_root();
        let project = collection_root();
        write_wiki(project.path(), "project-page", "Portable project body.");
        let id = collection_id('a');
        let store = RagStore::open(source.path()).expect("open source store");
        store
            .register_collection(CollectionRegistration {
                collection_id: Some(id.clone()),
                kind: CollectionKind::RegisteredProject,
                state: CollectionState::Attached,
                aliases: vec!["project-alpha".to_owned()],
                local_locator: Some(project.path().to_path_buf()),
                source_project_id: Some(PROJECT_ID.to_owned()),
                default_visibility: CollectionVisibility::ProjectPrivate,
                portable_identity: None,
                reviewed_inventory_digest: None,
            })
            .expect("register project collection");
        write_claim(
            source.path(),
            &canonical_claim(&id, 'a', RagVisibility::ProjectPrivate, "docs/decision.md"),
        );
        store.rebuild().expect("rebuild source claims");
        export_bundle(
            source.path(),
            BundleScope::Project {
                id: PROJECT_ID.to_owned(),
            },
            bundle_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("export project bundle");
        (source, project, id)
    }

    fn global_bundle(bundle_path: &Path, pages: &[(&str, &str)]) -> TempDir {
        let source = initialized_root();
        for (id, body) in pages {
            write_wiki(source.path(), id, body);
        }
        RagStore::open(source.path())
            .expect("open global source store")
            .rebuild()
            .expect("rebuild global source");
        export_bundle(
            source.path(),
            BundleScope::AllPortable,
            bundle_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("export global bundle");
        source
    }

    #[test]
    fn multi_bundle_preview_unions_unique_pages_and_deduplicates_exact_payloads() {
        let directory = tempfile::tempdir().expect("bundle directory");
        let first = directory.path().join("first.hivekb");
        let second = directory.path().join("second.hivekb");
        let _source_a = global_bundle(&first, &[("backup", "Backup instructions.")]);
        let _source_b = global_bundle(&second, &[("restore", "Restore instructions.")]);

        let preview = preview_bundle_merge(
            &[second.clone(), first.clone(), first.clone()],
            BundleLimits::default(),
        )
        .expect("preview merge");
        assert_eq!(preview.archive_sha256s.len(), 2);
        assert!(preview.exact_duplicate_count > 0);
        assert!(preview.conflict_paths.is_empty());
        let encoded =
            encode_bundle(preview.request(), BundleLimits::default()).expect("encode merge");
        assert!(encoded
            .plan()
            .entries()
            .iter()
            .any(|entry| entry.relative_path().ends_with("Wiki/backup.md")));
        assert!(encoded
            .plan()
            .entries()
            .iter()
            .any(|entry| entry.relative_path().ends_with("Wiki/restore.md")));
    }

    #[test]
    fn multi_bundle_preview_reports_divergent_same_path_without_selecting_it() {
        let directory = tempfile::tempdir().expect("bundle directory");
        let first = directory.path().join("first.hivekb");
        let second = directory.path().join("second.hivekb");
        let third = directory.path().join("third.hivekb");
        let _source_a = global_bundle(&first, &[("backup", "Keep backups for thirty days.")]);
        let _source_b = global_bundle(&second, &[("backup", "Keep backups for ninety days.")]);
        let _source_c = global_bundle(&third, &[("backup", "Keep backups for one hundred days.")]);

        let preview = preview_bundle_merge(&[first, second, third], BundleLimits::default())
            .expect("preview conflicting merge");
        assert!(preview
            .conflict_paths
            .iter()
            .any(|path| path.ends_with("Wiki/backup.md")));
        assert_eq!(preview.conflicts.len(), 1);
        let conflict = &preview.conflicts[0];
        assert_eq!(conflict.variants.len(), 3);
        let request = apply_bundle_merge_review(
            &preview,
            &[BundleMergeDecision::Choose {
                path: conflict.path.clone(),
                selected_sha256: conflict.variants[0].sha256.clone(),
            }],
        )
        .expect("reviewed conflict selection");
        assert!(request
            .entries
            .iter()
            .any(|entry| entry.relative_path.ends_with("Wiki/backup.md")));
        assert!(request
            .entries
            .iter()
            .any(|entry| entry.relative_path.contains("/Merge/")));
    }

    #[test]
    fn reviewed_equivalent_pages_keep_one_active_page_and_portable_original() {
        let directory = tempfile::tempdir().expect("bundle directory");
        let first = directory.path().join("first.hivekb");
        let second = directory.path().join("second.hivekb");
        let source_a = initialized_root();
        let source_b = initialized_root();
        let first_page = String::from_utf8(wiki_bytes("first", "Same portable instruction."))
            .expect("UTF-8 page")
            .replace("first summary", "shared summary");
        let second_page = String::from_utf8(wiki_bytes("second", "Same portable instruction."))
            .expect("UTF-8 page")
            .replace("second summary", "shared summary");
        fs::write(
            source_a.path().join(WIKI_RELATIVE).join("first.md"),
            first_page,
        )
        .expect("write first page");
        fs::write(
            source_b.path().join(WIKI_RELATIVE).join("second.md"),
            second_page,
        )
        .expect("write second page");
        RagStore::open(source_a.path()).unwrap().rebuild().unwrap();
        RagStore::open(source_b.path()).unwrap().rebuild().unwrap();
        for (source, bundle) in [(&source_a, &first), (&source_b, &second)] {
            export_bundle(
                source.path(),
                BundleScope::AllPortable,
                bundle,
                &BundlePublishMode::CreateOnly,
                BundleLimits::default(),
            )
            .expect("export semantic candidate");
        }
        let preview = preview_bundle_merge(&[first, second], BundleLimits::default())
            .expect("semantic merge preview");
        assert_eq!(preview.semantic_candidates.len(), 1);
        let candidate = &preview.semantic_candidates[0];
        let request = apply_bundle_merge_review(
            &preview,
            &[BundleMergeDecision::Equivalent {
                candidate_id: candidate.candidate_id.clone(),
                primary_path: candidate.paths[0].clone(),
            }],
        )
        .expect("reviewed merge request");
        assert_eq!(
            request
                .entries
                .iter()
                .filter(|entry| entry.relative_path.contains("/Wiki/"))
                .count(),
            1
        );
        assert!(request
            .entries
            .iter()
            .any(|entry| entry.relative_path.contains("/Merge/")));
        let encoded =
            encode_bundle(&request, BundleLimits::default()).expect("encode reviewed merge");
        let destination = initialized_root();
        let combined = directory.path().join("combined.hivekb");
        fs::write(&combined, encoded.archive()).expect("write combined archive");
        import_bundle(
            destination.path(),
            &combined,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("import reviewed merge");
        assert!(destination.path().join(MERGE_RELATIVE).is_dir());
    }

    fn detached_wiki_path(root: &Path, collection_id: &str) -> PathBuf {
        root.join(DETACHED_IMPORT_ROOT)
            .join(collection_id)
            .join(WIKI_RELATIVE)
            .join("project-page.md")
    }

    fn suppression(entries: Vec<crate::SuppressionEntry>) -> Vec<u8> {
        serde_yaml::to_string(&crate::SuppressionLedger {
            schema_version: 1,
            entries,
        })
        .expect("serialize suppression")
        .into_bytes()
    }

    #[test]
    fn export_is_deterministic_and_excludes_sensitive_nonportable_truth() {
        let source = initialized_root();
        write_wiki(source.path(), "portable-page", "Portable canonical body.");
        write_wiki(
            source.path(),
            "credential-page",
            "secret = abcdefghijklmnopqrstuvwxyz0123456789",
        );
        write_claim(
            source.path(),
            &canonical_claim(
                USER_ROOT_COLLECTION_ID,
                'b',
                RagVisibility::Shared,
                "docs/portable.md",
            ),
        );
        write_claim(
            source.path(),
            &canonical_claim(
                USER_ROOT_COLLECTION_ID,
                'c',
                RagVisibility::Shared,
                "C:\\machine\\private.md",
            ),
        );
        write_claim(
            source.path(),
            &canonical_claim(
                USER_ROOT_COLLECTION_ID,
                'd',
                RagVisibility::Confidential,
                "docs/confidential.md",
            ),
        );
        let output = tempfile::tempdir().expect("bundle output");
        let first_path = output.path().join("first.hivekb");
        let second_path = output.path().join("second.hivekb");
        let first = export_bundle(
            source.path(),
            BundleScope::Global,
            &first_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("first export");
        let second = export_bundle(
            source.path(),
            BundleScope::Global,
            &second_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("second export");
        assert_eq!(
            fs::read(&first_path).unwrap(),
            fs::read(&second_path).unwrap()
        );
        assert_eq!(first.archive_sha256, second.archive_sha256);
        assert_eq!(first.entry_count, 4);
        assert_eq!(first.credential_excluded_count, 1);
        assert_eq!(first.absolute_path_excluded_count, 1);
        assert_eq!(first.confidential_excluded_count, 1);

        let plan = load_bundle(&first_path, BundleLimits::default()).expect("load export");
        let paths = plan
            .entries()
            .iter()
            .map(crate::portable::ValidatedBundleEntry::relative_path)
            .collect::<Vec<_>>();
        assert!(paths.contains(&PORTABLE_REGISTRY_PATH));
        assert!(paths.iter().any(|path| path.ends_with("portable-page.md")));
        assert!(paths
            .iter()
            .any(|path| path.ends_with(&format!("{}.md", claim_id('b')))));
        assert!(!paths.iter().any(|path| path.contains("credential-page")));
        assert!(!paths
            .iter()
            .any(|path| path.ends_with(&format!("{}.md", claim_id('c')))));
        assert!(!paths
            .iter()
            .any(|path| path.ends_with(&format!("{}.md", claim_id('d')))));
        let archive = fs::read(first_path).expect("archive bytes");
        assert!(!archive
            .windows(source.path().display().to_string().len())
            .any(|window| window == source.path().display().to_string().as_bytes()));
    }

    #[test]
    fn bundle_round_trip_preserves_typed_scan_metadata() {
        let source = initialized_root();
        let project = collection_root();
        let id = collection_id('e');
        let store = RagStore::open(source.path()).expect("open source store");
        store
            .register_collection(CollectionRegistration {
                collection_id: Some(id.clone()),
                kind: CollectionKind::RegisteredProject,
                state: CollectionState::Attached,
                aliases: vec!["portable-scan".to_owned()],
                local_locator: Some(project.path().to_path_buf()),
                source_project_id: Some("portable-scan-project".to_owned()),
                default_visibility: CollectionVisibility::ProjectPrivate,
                portable_identity: None,
                reviewed_inventory_digest: None,
            })
            .expect("register scan project");
        let expected = canonical_scan_claim(&id);
        write_claim(source.path(), &expected);
        store.rebuild().expect("index scan claim");

        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("scan.hivekb");
        export_bundle(
            source.path(),
            BundleScope::Project {
                id: "portable-scan-project".to_owned(),
            },
            &bundle_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("export scan claim");
        let destination = initialized_root();
        import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("import scan claim");
        let locator = destination
            .path()
            .join(CLAIMS_RELATIVE)
            .join(&id)
            .join(format!("{}.md", expected.claim_id));
        let markdown = fs::read_to_string(locator).expect("imported scan claim Markdown");
        let parsed = parse_claim_markdown(&expected.locator, &markdown).expect("parse scan claim");
        assert_eq!(parsed.scan_metadata, expected.scan_metadata);
        assert_eq!(parsed.digest, expected.digest);
        assert!(destination.path().join(RAG_TRUST_RELATIVE).is_file());
        RagStore::open(destination.path())
            .expect("open trusted imported store")
            .validate_current()
            .expect("validate imported trust binding");
    }

    #[test]
    fn disabled_wiki_import_is_canonical_only_detached_and_idempotent() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = canonical_only_root();
        let setup_path = destination.path().join(".hive/config/user-setup.yml");
        let disabled = b"schema_version: 1\nwiki:\n  enabled: false\n";
        fs::write(&setup_path, disabled).expect("disabled setup marker");
        assert!(!destination.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!destination.path().join(RAG_MANIFEST_RELATIVE).exists());
        assert!(!destination.path().join(RAG_TRUST_RELATIVE).exists());

        let dry_run = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::DryRun,
            BundleLimits::default(),
        )
        .expect("dry-run import");
        assert_eq!(dry_run.disposition, BundleImportDisposition::Planned);
        assert!(!dry_run.canonical_mutation);
        assert!(!dry_run.index_rebuilt);
        assert_eq!(dry_run.detached_collection_ids, vec![id.clone()]);
        assert!(!detached_wiki_path(destination.path(), &id).exists());

        let applied = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("apply import");
        assert_eq!(applied.disposition, BundleImportDisposition::Applied);
        assert!(applied.canonical_mutation);
        assert!(!applied.index_rebuilt);
        assert!(detached_wiki_path(destination.path(), &id).is_file());
        assert!(destination
            .path()
            .join(CLAIMS_RELATIVE)
            .join(&id)
            .join(format!("{}.md", claim_id('a')))
            .is_file());
        let registry = RagStore::open(destination.path())
            .expect("open destination")
            .load_registry()
            .expect("load imported registry");
        let imported = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == id)
            .expect("imported collection");
        assert_eq!(imported.kind, CollectionKind::RegisteredProject);
        assert_eq!(imported.state, CollectionState::Detached);
        assert!(imported.local_locator.is_none());
        assert_eq!(fs::read(&setup_path).unwrap(), disabled);
        assert!(!destination.path().join(RAG_DIRTY_RELATIVE).exists());
        assert!(!destination.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!destination.path().join(RAG_MANIFEST_RELATIVE).exists());
        assert!(!destination.path().join(RAG_TRUST_RELATIVE).exists());
        assert!(RagStore::open(destination.path())
            .expect("open canonical-only destination")
            .validate_current()
            .is_err());

        let noop = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("idempotent import");
        assert_eq!(noop.disposition, BundleImportDisposition::Noop);
        assert_eq!(noop.added_count, 0);
        assert!(!noop.index_rebuilt);
        assert!(noop.changed_paths.is_empty());

        let reexport_path = bundle_dir.path().join("detached-reexport.hivekb");
        export_bundle(
            destination.path(),
            BundleScope::Collection { id: id.clone() },
            &reexport_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("re-export detached canonical bytes");
        let second_destination = initialized_root();
        import_bundle(
            second_destination.path(),
            &reexport_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("import re-exported detached collection");
        assert_eq!(
            fs::read(detached_wiki_path(second_destination.path(), &id))
                .expect("twice-portable Wiki page"),
            wiki_bytes("project-page", "Portable project body.")
        );
    }

    #[test]
    fn disabled_wiki_import_failure_restores_canonical_state_without_derived_artifacts() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = canonical_only_root();
        let setup_path = destination.path().join(USER_SETUP_RELATIVE);
        let disabled = b"schema_version: 1\nwiki:\n  enabled: false\n";
        fs::write(&setup_path, disabled).expect("disabled setup marker");
        FAIL_AFTER_CANONICAL_WRITES.with(|failure| failure.set(true));

        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect_err("injected canonical-only activation failure");
        assert!(matches!(error, WikiError::Io(_)));
        assert_eq!(fs::read(&setup_path).unwrap(), disabled);
        assert!(!destination
            .path()
            .join(COLLECTION_REGISTRY_RELATIVE)
            .exists());
        assert!(!detached_wiki_path(destination.path(), &id).exists());
        assert!(!destination.path().join(CLAIMS_RELATIVE).exists());
        assert!(!destination.path().join(RAG_DIRTY_RELATIVE).exists());
        assert!(!destination.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!destination.path().join(RAG_MANIFEST_RELATIVE).exists());
        assert!(!destination.path().join(RAG_TRUST_RELATIVE).exists());
    }

    #[test]
    fn existing_exact_attachment_receives_wiki_bytes_without_detached_storage() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        let mapped_project = collection_root();
        RagStore::open(destination.path())
            .expect("open destination")
            .register_collection(CollectionRegistration {
                collection_id: Some(id.clone()),
                kind: CollectionKind::RegisteredProject,
                state: CollectionState::Attached,
                aliases: vec!["project-alpha".to_owned()],
                local_locator: Some(mapped_project.path().to_path_buf()),
                source_project_id: Some(PROJECT_ID.to_owned()),
                default_visibility: CollectionVisibility::ProjectPrivate,
                portable_identity: None,
                reviewed_inventory_digest: None,
            })
            .expect("register exact destination mapping");

        let applied = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("import into attached mapping");
        assert_eq!(applied.disposition, BundleImportDisposition::Applied);
        assert!(applied.detached_collection_ids.is_empty());
        assert!(mapped_project
            .path()
            .join(WIKI_RELATIVE)
            .join("project-page.md")
            .is_file());
        assert!(!destination.path().join(DETACHED_IMPORT_ROOT).exists());
        assert!(applied
            .changed_paths
            .iter()
            .any(|path| { path == &format!("collections/{id}/{WIKI_RELATIVE}/project-page.md") }));
    }

    #[test]
    fn attached_collection_bytes_join_the_same_cross_root_rollback() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        let mapped_project = collection_root();
        RagStore::open(destination.path())
            .expect("open destination")
            .register_collection(CollectionRegistration {
                collection_id: Some(id.clone()),
                kind: CollectionKind::RegisteredProject,
                state: CollectionState::Attached,
                aliases: vec!["project-alpha".to_owned()],
                local_locator: Some(mapped_project.path().to_path_buf()),
                source_project_id: Some(PROJECT_ID.to_owned()),
                default_visibility: CollectionVisibility::ProjectPrivate,
                portable_identity: None,
                reviewed_inventory_digest: None,
            })
            .expect("register exact destination mapping");
        let registry_before =
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let index_before = fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        let manifest_before = fs::read(destination.path().join(RAG_MANIFEST_RELATIVE)).unwrap();
        let trust_before = fs::read(destination.path().join(RAG_TRUST_RELATIVE)).unwrap();
        FAIL_AFTER_CANONICAL_WRITES.with(|failure| failure.set(true));

        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect_err("injected cross-root activation failure");
        assert!(matches!(error, WikiError::Io(_)));
        assert!(!mapped_project
            .path()
            .join(WIKI_RELATIVE)
            .join("project-page.md")
            .exists());
        assert!(!destination.path().join(CLAIMS_RELATIVE).join(&id).exists());
        assert_eq!(
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap(),
            registry_before
        );
        assert_eq!(
            fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap(),
            index_before
        );
        assert_eq!(
            fs::read(destination.path().join(RAG_MANIFEST_RELATIVE)).unwrap(),
            manifest_before
        );
        assert_eq!(
            fs::read(destination.path().join(RAG_TRUST_RELATIVE)).unwrap(),
            trust_before
        );
        assert!(!destination.path().join(RAG_DIRTY_RELATIVE).exists());
    }

    #[test]
    fn same_path_divergence_fails_before_any_other_mutation() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("initial import");
        let divergent_path = detached_wiki_path(destination.path(), &id);
        fs::write(
            &divergent_path,
            wiki_bytes("project-page", "Divergent destination body."),
        )
        .expect("divergent destination");
        let registry_before =
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let index_before = fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        let manifest_before = fs::read(destination.path().join(RAG_MANIFEST_RELATIVE)).unwrap();
        let trust_before = fs::read(destination.path().join(RAG_TRUST_RELATIVE)).unwrap();

        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect_err("divergent bytes must conflict");
        assert!(matches!(error, WikiError::Conflict(_)));
        assert_eq!(
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap(),
            registry_before
        );
        assert_eq!(
            fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap(),
            index_before
        );
        assert_eq!(
            fs::read(destination.path().join(RAG_MANIFEST_RELATIVE)).unwrap(),
            manifest_before
        );
        assert_eq!(
            fs::read(destination.path().join(RAG_TRUST_RELATIVE)).unwrap(),
            trust_before
        );
        assert_eq!(
            fs::read(divergent_path).unwrap(),
            wiki_bytes("project-page", "Divergent destination body.")
        );
    }

    #[test]
    fn same_collection_id_with_divergent_portable_metadata_fails_closed() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        let store = RagStore::open(destination.path()).expect("open destination");
        store
            .register_collection(CollectionRegistration {
                collection_id: Some(id),
                kind: CollectionKind::Directory,
                state: CollectionState::Detached,
                aliases: vec!["project-alpha".to_owned()],
                local_locator: None,
                source_project_id: Some(PROJECT_ID.to_owned()),
                default_visibility: CollectionVisibility::ProjectPrivate,
                portable_identity: None,
                reviewed_inventory_digest: None,
            })
            .expect("register divergent destination truth");
        let registry_before =
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let index_before = fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap();

        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::DryRun,
            BundleLimits::default(),
        )
        .expect_err("divergent same-ID metadata must conflict");
        assert!(matches!(error, WikiError::Conflict(_)));
        assert_eq!(
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap(),
            registry_before
        );
        assert_eq!(
            fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap(),
            index_before
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn reviewed_exclusion_binds_conflicting_bytes_and_preserves_nonconflicting_import() {
        let output = tempfile::tempdir().unwrap();
        let original = output.path().join("original.hivekb");
        let (source, project, id) = project_bundle(&original);
        let destination = initialized_root();
        import_bundle(
            destination.path(),
            &original,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .unwrap();
        let divergent = detached_wiki_path(destination.path(), &id);
        fs::write(
            &divergent,
            wiki_bytes("project-page", "Destination version one."),
        )
        .unwrap();
        fs::write(
            project.path().join(WIKI_RELATIVE).join("new-page.md"),
            wiki_bytes("new-page", "New portable knowledge."),
        )
        .unwrap();
        let updated = output.path().join("updated.hivekb");
        export_bundle(
            source.path(),
            BundleScope::Project {
                id: PROJECT_ID.to_owned(),
            },
            &updated,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .unwrap();
        let preview = import_bundle(
            destination.path(),
            &updated,
            BundleImportMode::DryRun,
            BundleLimits::default(),
        )
        .unwrap();
        assert_eq!(preview.conflict_paths.len(), 1);
        assert!(preview.added_count > 0);
        let expected_preview = import_preview_digest(&preview).unwrap();
        let new_path = divergent.parent().unwrap().join("new-page.md");
        assert!(matches!(
            import_bundle(
                destination.path(),
                &updated,
                BundleImportMode::Apply,
                BundleLimits::default()
            ),
            Err(WikiError::Conflict(_))
        ));
        assert!(!new_path.exists());

        let retained = wiki_bytes("project-page", "Destination version two.");
        fs::write(&divergent, &retained).unwrap();
        let rejected = import_bundle_reviewed(
            destination.path(),
            &updated,
            BundleImportMode::ApplyExcludingConflicts,
            BundleLimits::default(),
            BundleImportApproval {
                archive_sha256: &preview.archive_sha256,
                preview_digest: &expected_preview,
            },
        );
        assert!(matches!(rejected, Err(WikiError::Conflict(_))));
        assert!(!new_path.exists());
        assert_eq!(fs::read(&divergent).unwrap(), retained);

        let current = import_bundle(
            destination.path(),
            &updated,
            BundleImportMode::DryRun,
            BundleLimits::default(),
        )
        .unwrap();
        assert_ne!(current.target_state_digest, preview.target_state_digest);
        let digest = import_preview_digest(&current).unwrap();
        let applied = import_bundle_reviewed(
            destination.path(),
            &updated,
            BundleImportMode::ApplyExcludingConflicts,
            BundleLimits::default(),
            BundleImportApproval {
                archive_sha256: &current.archive_sha256,
                preview_digest: &digest,
            },
        )
        .unwrap();
        assert_eq!(applied.conflict_paths, current.conflict_paths);
        assert_eq!(fs::read(&divergent).unwrap(), retained);
        assert_eq!(
            fs::read(&new_path).unwrap(),
            wiki_bytes("new-page", "New portable knowledge.")
        );
        let retry = import_bundle(
            destination.path(),
            &updated,
            BundleImportMode::ApplyExcludingConflicts,
            BundleLimits::default(),
        )
        .unwrap();
        assert_eq!(retry.disposition, BundleImportDisposition::Noop);
        assert!(!retry.canonical_mutation);
        assert!(retry.changed_paths.is_empty());
    }

    #[test]
    fn reviewed_import_rejects_wrong_archive_before_writes() {
        let output = tempfile::tempdir().unwrap();
        let bundle = output.path().join("source.hivekb");
        let (_source, _project, _id) = project_bundle(&bundle);
        let destination = initialized_root();
        let preview = import_bundle(
            destination.path(),
            &bundle,
            BundleImportMode::DryRun,
            BundleLimits::default(),
        )
        .unwrap();
        let registry = fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let digest = import_preview_digest(&preview).unwrap();
        assert!(matches!(
            import_bundle_reviewed(
                destination.path(),
                &bundle,
                BundleImportMode::Apply,
                BundleLimits::default(),
                BundleImportApproval {
                    archive_sha256: &format!("sha256:{}", "0".repeat(64)),
                    preview_digest: &digest,
                }
            ),
            Err(WikiError::Conflict(_))
        ));
        assert_eq!(
            registry,
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap()
        );
        assert!(!destination.path().join(DETACHED_IMPORT_ROOT).exists());
    }

    #[test]
    fn existing_suppression_blocks_an_incoming_active_wiki_page() {
        let source = initialized_root();
        write_wiki(
            source.path(),
            "blocked-page",
            "This page is active at the source.",
        );
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("global.hivekb");
        export_bundle(
            source.path(),
            BundleScope::Global,
            &bundle_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("export global bundle");

        let destination = initialized_root();
        fs::write(
            destination.path().join(SUPPRESSION_RELATIVE),
            suppression(vec![crate::SuppressionEntry {
                fingerprint: sha256_digest(b"prior removed bytes"),
                source_locator: "wiki:blocked-page".to_owned(),
                reason: "user-request".to_owned(),
                replacement: None,
                timestamp: "2026-08-01T00:00:00Z".to_owned(),
            }]),
        )
        .expect("destination suppression");
        let registry_before =
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect_err("active page must not bypass destination suppression");
        assert!(matches!(error, WikiError::Conflict(_)));
        assert!(!destination
            .path()
            .join(WIKI_RELATIVE)
            .join("blocked-page.md")
            .exists());
        assert_eq!(
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap(),
            registry_before
        );
    }

    #[test]
    fn suppression_ledgers_merge_nonoverlapping_entries_and_then_noop() {
        let source = initialized_root();
        let source_entry = crate::SuppressionEntry {
            fingerprint: sha256_digest(b"source removed truth"),
            source_locator: "external:source-truth".to_owned(),
            reason: "user-request".to_owned(),
            replacement: None,
            timestamp: "2026-08-01T00:00:00Z".to_owned(),
        };
        fs::write(
            source.path().join(SUPPRESSION_RELATIVE),
            suppression(vec![source_entry.clone()]),
        )
        .expect("source suppression");
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("global.hivekb");
        export_bundle(
            source.path(),
            BundleScope::Global,
            &bundle_path,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("export global bundle");

        let destination = initialized_root();
        let destination_entry = crate::SuppressionEntry {
            fingerprint: sha256_digest(b"destination removed truth"),
            source_locator: "external:destination-truth".to_owned(),
            reason: "retention-expired".to_owned(),
            replacement: None,
            timestamp: "2026-08-01T00:00:01Z".to_owned(),
        };
        fs::write(
            destination.path().join(SUPPRESSION_RELATIVE),
            suppression(vec![destination_entry.clone()]),
        )
        .expect("destination suppression");
        let applied = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("merge suppression");
        assert_eq!(applied.disposition, BundleImportDisposition::Applied);
        let merged: crate::SuppressionLedger = serde_yaml::from_slice(
            &fs::read(destination.path().join(SUPPRESSION_RELATIVE)).unwrap(),
        )
        .expect("merged ledger");
        let mut expected = vec![destination_entry, source_entry];
        expected.sort_by(|left, right| {
            (&left.fingerprint, &left.source_locator, &left.timestamp).cmp(&(
                &right.fingerprint,
                &right.source_locator,
                &right.timestamp,
            ))
        });
        assert_eq!(merged.entries, expected);
        let noop = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("idempotent suppression import");
        assert_eq!(noop.disposition, BundleImportDisposition::Noop);
    }

    #[test]
    fn failure_after_canonical_writes_restores_every_byte_and_directory() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        let registry_before =
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let index_before = fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        let manifest_before = fs::read(destination.path().join(RAG_MANIFEST_RELATIVE)).unwrap();
        FAIL_AFTER_CANONICAL_WRITES.with(|failure| failure.set(true));

        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect_err("injected activation failure");
        assert!(matches!(error, WikiError::Io(_)));
        assert!(error.to_string().contains("restored from backup"));
        assert_eq!(
            fs::read(destination.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap(),
            registry_before
        );
        assert_eq!(
            fs::read(destination.path().join(SHARED_INDEX_RELATIVE)).unwrap(),
            index_before
        );
        assert_eq!(
            fs::read(destination.path().join(RAG_MANIFEST_RELATIVE)).unwrap(),
            manifest_before
        );
        assert!(!destination.path().join(RAG_DIRTY_RELATIVE).exists());
        assert!(!destination.path().join(DETACHED_IMPORT_ROOT).exists());
        assert!(!destination.path().join(CLAIMS_RELATIVE).exists());
        assert!(!detached_wiki_path(destination.path(), &id).exists());
    }

    #[cfg(unix)]
    #[test]
    fn root_path_replacement_after_canonical_writes_keeps_using_pinned_root() {
        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        let original = destination.path().to_path_buf();
        let moved = original.with_file_name(format!(
            "{}.pinned",
            original
                .file_name()
                .and_then(OsStr::to_str)
                .expect("UTF-8 temporary root")
        ));
        assert!(!moved.exists());
        let hook_original = original.clone();
        let hook_moved = moved.clone();
        AFTER_CANONICAL_WRITES_HOOK.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || {
                fs::rename(&hook_original, &hook_moved).map_err(|error| {
                    WikiError::Io(format!("cannot move pinned test root: {error}"))
                })?;
                fs::create_dir(&hook_original).map_err(|error| {
                    WikiError::Io(format!("cannot create replacement test root: {error}"))
                })?;
                fs::write(hook_original.join("replacement-marker"), b"replacement").map_err(
                    |error| WikiError::Io(format!("cannot mark replacement test root: {error}")),
                )?;
                Ok(())
            }));
        });

        let result = import_bundle(
            &original,
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        );
        let replacement_marker = fs::read(original.join("replacement-marker"));
        let imported_registry = moved.join(COLLECTION_REGISTRY_RELATIVE).is_file();
        let imported_index = moved.join(SHARED_INDEX_RELATIVE).is_file();
        let imported_page = detached_wiki_path(&moved, &id).is_file();
        if moved.exists() {
            fs::remove_dir_all(&original).expect("remove replacement root");
            fs::rename(&moved, &original).expect("restore pinned temporary root path");
        }

        let applied = result.expect("pinned import survives ambient root replacement");
        assert_eq!(applied.disposition, BundleImportDisposition::Applied);
        assert_eq!(
            replacement_marker.expect("replacement marker"),
            b"replacement"
        );
        assert!(imported_registry);
        assert!(imported_index);
        assert!(imported_page);
    }

    #[cfg(unix)]
    #[test]
    fn detached_import_rejects_symlinked_storage_ancestors() {
        use std::os::unix::fs::symlink;

        let bundle_dir = tempfile::tempdir().expect("bundle directory");
        let bundle_path = bundle_dir.path().join("project.hivekb");
        let (_source, _project, _id) = project_bundle(&bundle_path);
        let destination = initialized_root();
        let outside = tempfile::tempdir().expect("outside directory");
        symlink(
            outside.path(),
            destination.path().join(DETACHED_IMPORT_ROOT),
        )
        .expect("symlink import root");
        let error = import_bundle(
            destination.path(),
            &bundle_path,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect_err("symlinked detached storage must fail");
        assert!(matches!(error, WikiError::Conflict(_)));
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    }
}
