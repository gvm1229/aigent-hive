//! Canonical-derived collection, document, chunk, and claim retrieval projection.
//!
//! Markdown remains authoritative. The `SQLite` bytes produced here are a disposable
//! cache whose generation manifest can be checked without scanning canonical files
//! for every query.

use crate::collection::{
    folded_alias, CollectionKind, CollectionRecord, CollectionRegistry, CollectionResolution,
    CollectionState, CollectionVisibility, USER_ROOT_COLLECTION_ID,
};
use crate::scan::{ScanClaimMetadata, ScanPromotionStatus, ScanReviewStatus};
use hive_core::sha256_digest;
use rusqlite::{params, Connection, MAIN_DB};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;
use std::str::FromStr;

/// Version of the disposable RAG projection schema.
pub const RAG_SCHEMA_VERSION: u32 = 6;
/// Version of the canonical typed-claim Markdown contract.
pub const CLAIM_SCHEMA_VERSION: u32 = 1;

/// Maximum size of one authenticated serialized `SQLite` RAG projection.
///
/// The limit accommodates the release-qualified 50,000-chunk corpus while
/// retaining a finite in-memory deserialization boundary.
pub const MAX_SERIALIZED_INDEX_BYTES: usize = 256 * 1024 * 1024;
const MAX_DOCUMENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_FACT_BYTES: usize = 16 * 1024;
const MAX_QUERY_BYTES: usize = 4096;
const MAX_EXPANSIONS: usize = 8;
const MAX_TOP_K: usize = 100;
const MAX_BYTE_BUDGET: usize = 1024 * 1024;
const CHUNK_TARGET_BYTES: usize = 1200;
const MAX_CANDIDATES: usize = 20_000;

/// Stable error classes for canonical projection and retrieval.
#[derive(Debug)]
pub enum RagError {
    /// A request or canonical value is malformed.
    InvalidInput(String),
    /// An identifier, alias, or explicit truth update conflicts.
    Conflict(String),
    /// The disposable projection must be rebuilt from canonical Markdown.
    RepairRequired(String),
    /// A local I/O operation failed.
    Io(String),
    /// `SQLite` projection or retrieval failed.
    Sqlite(String),
}

impl Display for RagError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message)
            | Self::Conflict(message)
            | Self::RepairRequired(message)
            | Self::Io(message)
            | Self::Sqlite(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RagError {}

/// Visibility attached to one canonical document or claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum RagVisibility {
    /// Visible across collections.
    Shared,
    /// Visible only in the owning collection's project scope.
    ProjectPrivate,
    /// Visible only with explicit collection authorization.
    Confidential,
}

impl RagVisibility {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::ProjectPrivate => "project-private",
            Self::Confidential => "confidential",
        }
    }

    fn parse(value: &str) -> Result<Self, RagError> {
        match value {
            "shared" => Ok(Self::Shared),
            "project-private" => Ok(Self::ProjectPrivate),
            "confidential" => Ok(Self::Confidential),
            _ => Err(RagError::RepairRequired(format!(
                "unknown indexed visibility `{value}`"
            ))),
        }
    }
}

impl From<CollectionVisibility> for RagVisibility {
    fn from(value: CollectionVisibility) -> Self {
        match value {
            CollectionVisibility::Shared => Self::Shared,
            CollectionVisibility::ProjectPrivate => Self::ProjectPrivate,
            CollectionVisibility::Confidential => Self::Confidential,
        }
    }
}

/// Normalize the supported Wiki kind taxonomy to one query category.
///
/// # Errors
///
/// Returns an error when `kind` is not part of the canonical or legacy taxonomy.
pub fn canonical_wiki_category(kind: &str) -> Result<&'static str, RagError> {
    match kind {
        "source" | "source-summary" => Ok("source"),
        "entity" => Ok("entity"),
        "concept" => Ok("concept"),
        "comparison" => Ok("comparison"),
        "synthesis" => Ok("synthesis"),
        "question" | "open-question" => Ok("question"),
        "decision" => Ok("decision"),
        "workflow" => Ok("workflow"),
        _ => Err(RagError::InvalidInput(format!(
            "unsupported Wiki kind `{kind}`"
        ))),
    }
}

/// Language classification used by retrieval metadata.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum RagLanguage {
    /// English.
    En,
    /// Korean.
    Ko,
    /// English and Korean.
    Both,
    /// Language is not declared.
    Und,
}

impl RagLanguage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
            Self::Both => "both",
            Self::Und => "und",
        }
    }

    fn parse(value: &str) -> Result<Self, RagError> {
        match value {
            "en" => Ok(Self::En),
            "ko" => Ok(Self::Ko),
            "both" => Ok(Self::Both),
            "und" => Ok(Self::Und),
            _ => Err(RagError::RepairRequired(format!(
                "unknown indexed language `{value}`"
            ))),
        }
    }
}

/// Explicit semantic kind of a canonical claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimKind {
    /// Stable project profile fact.
    ProjectProfile,
    /// Product or implementation decision.
    Decision,
    /// Project convention.
    Convention,
    /// User preference.
    Preference,
    /// Repeatable workflow.
    Workflow,
    /// Dependency or artifact evidence.
    DependencyEvidence,
    /// Verified task outcome.
    Outcome,
    /// Explicit unresolved question.
    Question,
}

impl ClaimKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectProfile => "project-profile",
            Self::Decision => "decision",
            Self::Convention => "convention",
            Self::Preference => "preference",
            Self::Workflow => "workflow",
            Self::DependencyEvidence => "dependency-evidence",
            Self::Outcome => "outcome",
            Self::Question => "question",
        }
    }

    fn parse(value: &str) -> Result<Self, RagError> {
        match value {
            "project-profile" => Ok(Self::ProjectProfile),
            "decision" => Ok(Self::Decision),
            "convention" => Ok(Self::Convention),
            "preference" => Ok(Self::Preference),
            "workflow" => Ok(Self::Workflow),
            "dependency-evidence" => Ok(Self::DependencyEvidence),
            "outcome" => Ok(Self::Outcome),
            "question" => Ok(Self::Question),
            _ => Err(RagError::RepairRequired(format!(
                "unknown indexed claim kind `{value}`"
            ))),
        }
    }
}

/// Evidence state of a canonical claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum AssertionStatus {
    /// Directly and explicitly stated by the user.
    UserStated,
    /// Observed in a reviewed artifact.
    Observed,
    /// Verified against declared acceptance evidence.
    Verified,
    /// Explicitly marked as an inference by a human-reviewed canonical record.
    Inferred,
    /// Retained because active evidence conflicts.
    Conflicted,
    /// Replaced by another canonical claim or invalidated by a reviewed source inventory.
    Superseded,
}

impl AssertionStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UserStated => "user-stated",
            Self::Observed => "observed",
            Self::Verified => "verified",
            Self::Inferred => "inferred",
            Self::Conflicted => "conflicted",
            Self::Superseded => "superseded",
        }
    }

    fn parse(value: &str) -> Result<Self, RagError> {
        match value {
            "user-stated" => Ok(Self::UserStated),
            "observed" => Ok(Self::Observed),
            "verified" => Ok(Self::Verified),
            "inferred" => Ok(Self::Inferred),
            "conflicted" => Ok(Self::Conflicted),
            "superseded" => Ok(Self::Superseded),
            _ => Err(RagError::RepairRequired(format!(
                "unknown indexed assertion status `{value}`"
            ))),
        }
    }
}

/// Allowed provenance for a remember request.
///
/// There is intentionally no raw-transcript or raw-tool-output variant.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum RememberSourceKind {
    /// A direct, bounded user statement selected for durable memory.
    UserStatement,
    /// A bounded summary of an artifact a human or agent deliberately reviewed.
    ReviewedArtifact,
}

/// Provenance retained with a canonical claim.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct ClaimProvenance {
    /// Explicit source class.
    pub source_kind: RememberSourceKind,
    /// Bounded normalized description, never a raw transcript or tool dump.
    pub summary: String,
    /// Stable logical source locator.
    pub locator: String,
    /// SHA-256 digest of the reviewed source bytes or user-statement envelope.
    pub digest: String,
}

/// Canonical Markdown document projected into the disposable index.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDocument {
    /// Stable document identifier.
    pub document_id: String,
    /// Owning logical collection.
    pub collection_id: String,
    /// Relative canonical Markdown locator.
    pub locator: String,
    /// Human-facing title.
    pub title: String,
    /// Exact canonical Wiki kind retained for compatibility results.
    pub kind: String,
    /// Normalized Wiki category used by category filters.
    pub category: String,
    /// Canonical Markdown body.
    pub body: String,
    /// Digest returned by [`document_digest`].
    pub digest: String,
    /// Visibility boundary.
    pub visibility: RagVisibility,
    /// Declared language.
    pub language: RagLanguage,
    /// Monotonic canonical revision.
    pub revision: u64,
    /// Searchable tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Searchable aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Outgoing canonical logical links.
    #[serde(default)]
    pub links: Vec<String>,
    /// Source locators retained for citations.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Replacement locator when this document is superseded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
}

/// One typed atomic claim stored in canonical Markdown.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalClaim {
    /// Stable claim identifier.
    pub claim_id: String,
    /// Human-supplied stable current-truth key.
    pub claim_key: String,
    /// Owning logical collection.
    pub collection_id: String,
    /// Optional containing canonical document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    /// Relative canonical Markdown locator.
    pub locator: String,
    /// Typed claim class.
    pub kind: ClaimKind,
    /// Evidence status.
    pub status: AssertionStatus,
    /// Visibility boundary.
    pub visibility: RagVisibility,
    /// Normalized atomic fact.
    pub normalized_fact: String,
    /// Explicit provenance.
    pub provenance: ClaimProvenance,
    /// Typed scan review and promotion metadata, when this claim came from a scan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_metadata: Option<ScanClaimMetadata>,
    /// Monotonic canonical revision.
    pub revision: u64,
    /// Source locators retained for citations.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Claim identifiers explicitly superseded by this claim.
    #[serde(default)]
    pub supersedes: Vec<String>,
    /// Replacement claim identifier when this claim is superseded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    /// RFC 3339 observation time, if declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
    /// RFC 3339 verification time, if declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
    /// Digest returned by [`claim_digest`].
    pub digest: String,
}

/// Complete canonical input for one disposable index generation.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RagSnapshot {
    /// Projection schema expected by the caller.
    pub schema_version: u32,
    /// Monotonic generation number.
    pub generation: u64,
    /// Canonical logical collection metadata.
    pub registry: CollectionRegistry,
    /// Canonical documents.
    #[serde(default)]
    pub documents: Vec<CanonicalDocument>,
    /// Canonical typed claims.
    #[serde(default)]
    pub claims: Vec<CanonicalClaim>,
}

/// One canonical locator included in a generation manifest.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    /// Owning collection.
    pub collection_id: String,
    /// `document` or `claim`.
    pub item_kind: String,
    /// Stable item identifier.
    pub item_id: String,
    /// Relative canonical locator.
    pub locator: String,
    /// Canonical digest.
    pub digest: String,
    /// Canonical revision.
    pub revision: u64,
}

/// Fast freshness token derived while canonical Markdown is enumerated for rebuild.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GenerationManifest {
    /// Projection schema version.
    pub schema_version: u32,
    /// Monotonic generation number.
    pub generation: u64,
    /// Digest of sorted canonical entries and collection metadata.
    pub logical_digest: String,
    /// Number of canonical entries.
    pub entry_count: usize,
    /// Digest of the exact serialized disposable `SQLite` generation.
    pub sqlite_digest: String,
}

/// Disposable serialized `SQLite` index and its freshness token.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RagIndexArtifact {
    /// Serialized `SQLite` database bytes.
    pub sqlite_bytes: Vec<u8>,
    /// Generation manifest written into those bytes.
    pub manifest: GenerationManifest,
    /// Number of projected documents.
    pub document_count: usize,
    /// Number of projected claims.
    pub claim_count: usize,
    /// Number of projected retrieval chunks.
    pub chunk_count: usize,
}

/// Retrieval scope accepted by the core query API.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalScope {
    /// Current collection when attached, otherwise global visibility.
    Auto,
    /// Shared knowledge plus non-confidential user-root knowledge.
    Global,
    /// Registered project linkage.
    Project(String),
    /// Stable collection identifier or alias.
    Collection(String),
    /// All knowledge visible without widening project-private boundaries.
    AllVisible,
}

impl Display for RetrievalScope {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => formatter.write_str("auto"),
            Self::Global => formatter.write_str("global"),
            Self::Project(project_id) => write!(formatter, "project:{project_id}"),
            Self::Collection(collection_id) => {
                write!(formatter, "collection:{collection_id}")
            }
            Self::AllVisible => formatter.write_str("all-visible"),
        }
    }
}

impl FromStr for RetrievalScope {
    type Err = RagError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "global" => Ok(Self::Global),
            "all-visible" => Ok(Self::AllVisible),
            _ => {
                if let Some(project_id) = value.strip_prefix("project:") {
                    validate_bounded_text("project scope", project_id, 256)?;
                    Ok(Self::Project(project_id.to_owned()))
                } else if let Some(collection_id) = value.strip_prefix("collection:") {
                    validate_bounded_text("collection scope", collection_id, 256)?;
                    Ok(Self::Collection(collection_id.to_owned()))
                } else {
                    Err(RagError::InvalidInput(format!(
                        "unsupported retrieval scope `{value}`"
                    )))
                }
            }
        }
    }
}

/// Bounded retrieval request. Query expansions must be supplied explicitly.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RetrievalRequest {
    /// Scope selector.
    pub scope: RetrievalScope,
    /// Current collection used only by `auto` and confidential authorization checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_collection_id: Option<String>,
    /// User query. It is never persisted in the index.
    pub query: String,
    /// Explicit bounded expansions, never inferred by this module.
    #[serde(default)]
    pub query_expansions: Vec<String>,
    /// Maximum number of returned chunks.
    pub top_k: usize,
    /// Maximum UTF-8 bytes returned across chunk texts.
    pub byte_budget: usize,
    /// Exact collection authorization for confidential chunks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidential_collection_id: Option<String>,
}

/// Page-level compatibility query over the RAG projection.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WikiPageQueryRequest {
    /// Registered project ID whose private rows may be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_project_id: Option<String>,
    /// Optional full-text expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Optional exact tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Optional normalized Wiki category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Maximum returned pages.
    pub limit: usize,
}

/// One page-level result used by the legacy shared-query facade.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WikiPageQueryHit {
    /// Stable RAG document identifier.
    pub document_id: String,
    /// Owning collection.
    pub collection_id: String,
    /// Legacy project linkage, absent for user-root documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project_id: Option<String>,
    /// Exact canonical Wiki kind.
    pub kind: String,
    /// Normalized Wiki category.
    pub category: String,
    /// Human-facing page summary.
    pub summary: String,
    /// Canonical page locator.
    pub locator: String,
    /// Canonical RAG document digest.
    pub digest: String,
    /// Indexed visibility boundary.
    pub visibility: RagVisibility,
    /// Searchable tags.
    pub tags: Vec<String>,
    /// Searchable aliases.
    pub aliases: Vec<String>,
    /// Citation sources.
    pub sources: Vec<String>,
    /// BM25 rank for text queries and zero otherwise.
    pub rank: f64,
}

/// One stable citation-bearing retrieval result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RetrievalHit {
    /// Stable chunk identifier.
    pub chunk_id: String,
    /// Owning collection.
    pub collection_id: String,
    /// Canonical item identifier.
    pub item_id: String,
    /// `document` or `claim`.
    pub item_kind: String,
    /// Canonical locator including the chunk anchor.
    pub locator: String,
    /// Human-facing title.
    pub title: String,
    /// Bounded chunk text.
    pub text: String,
    /// Digest of the canonical item.
    pub digest: String,
    /// Visibility retained for policy-aware callers.
    pub visibility: RagVisibility,
    /// Declared language.
    pub language: RagLanguage,
    /// Optional typed claim kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_kind: Option<ClaimKind>,
    /// Optional evidence state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertion_status: Option<AssertionStatus>,
    /// Typed scan review metadata projected from the canonical claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_metadata: Option<ScanClaimMetadata>,
    /// Deterministic rank score; chunk ID is the final tie-breaker.
    pub score: f64,
    /// Highest-priority field that matched.
    pub matched_field: String,
    /// Canonical citation sources.
    pub sources: Vec<String>,
    /// Optional replacement identifier or locator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    /// Always true: retrieved project content is data, not instructions.
    pub untrusted_content: bool,
}

/// Complete bounded retrieval result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RetrievalResult {
    /// Index generation used by this query.
    pub generation: u64,
    /// Logical manifest digest used by this query.
    pub manifest_digest: String,
    /// Ranked hits.
    pub hits: Vec<RetrievalHit>,
    /// UTF-8 text bytes returned across hits.
    pub returned_bytes: usize,
    /// True when matches existed but top-k or byte budget truncated them.
    pub insufficient_budget: bool,
}

/// Explicit request for an idempotent remember operation.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RememberRequest {
    /// Owning collection.
    pub collection_id: String,
    /// Stable current-truth key supplied by the caller.
    pub claim_key: String,
    /// Optional stable claim ID. A digest-derived ID is used when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
    /// Relative canonical Markdown locator.
    pub locator: String,
    /// Typed claim class.
    pub kind: ClaimKind,
    /// Evidence status. `superseded` is not valid for a new claim.
    pub status: AssertionStatus,
    /// Visibility boundary.
    pub visibility: RagVisibility,
    /// Atomic normalized fact selected for durable memory.
    pub normalized_fact: String,
    /// Explicit bounded provenance.
    pub provenance: ClaimProvenance,
    /// Source locators retained for citations.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Explicit claim IDs to replace. No semantic inference is performed.
    #[serde(default)]
    pub supersedes: Vec<String>,
    /// Expected digest of the single active claim being replaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_active_digest: Option<String>,
    /// RFC 3339 observation time, if declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
    /// RFC 3339 verification time, if declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

/// Outcome class of a pure remember plan.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RememberDisposition {
    /// A new active claim can be written.
    Insert,
    /// The exact active claim already exists.
    Noop,
    /// The new claim explicitly replaces existing claims.
    Supersede,
}

/// Pure canonical changes required by a remember request.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RememberPlan {
    /// Idempotent outcome class.
    pub disposition: RememberDisposition,
    /// New canonical claim, absent for a no-op.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_claim: Option<CanonicalClaim>,
    /// Existing canonical claims rewritten to `superseded`.
    pub superseded_claims: Vec<CanonicalClaim>,
}

/// Active claims and explicit key conflicts for current-truth consumers.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CurrentTruth {
    /// Active claims in stable key/ID order.
    pub active_claims: Vec<CanonicalClaim>,
    /// Claim keys with more than one divergent active digest.
    pub conflicts: BTreeMap<String, Vec<String>>,
}

/// One canonical mutation expected during an index update.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct DirtyEntry {
    /// Owning collection.
    pub collection_id: String,
    /// Relative canonical locator.
    pub locator: String,
    /// Digest expected after the mutation.
    pub target_digest: String,
}

/// Crash-recovery journal derived from an explicit canonical write set.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DirtyJournal {
    /// Generation that was current before mutation.
    pub base_generation: u64,
    /// Generation intended after mutation.
    pub target_generation: u64,
    /// Manifest digest that was current before mutation.
    pub base_manifest_digest: String,
    /// Exact canonical write set.
    pub entries: Vec<DirtyEntry>,
}

/// Deterministic recovery action; canonical Markdown always wins.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DirtyRecoveryDecision {
    /// No journal exists and the index manifest is current.
    Clean,
    /// Canonical files must be used to rebuild the disposable index.
    RebuildRequired,
    /// Journal/index lineage conflicts and requires explicit inspection before write.
    Conflict,
}

#[derive(Debug)]
struct IndexedChunk {
    chunk_id: String,
    item_kind: String,
    item_id: String,
    collection_id: String,
    ordinal: usize,
    title: String,
    text: String,
    locator: String,
    digest: String,
    visibility: RagVisibility,
    language: RagLanguage,
    tags: String,
    aliases: String,
    claim_kind: Option<ClaimKind>,
    assertion_status: Option<AssertionStatus>,
    scan_metadata: Option<ScanClaimMetadata>,
    revision: u64,
    replacement: Option<String>,
}

#[derive(Debug)]
struct Candidate {
    chunk_id: String,
    item_kind: String,
    item_id: String,
    collection_id: String,
    title: String,
    text: String,
    locator: String,
    digest: String,
    visibility: RagVisibility,
    language: RagLanguage,
    tags: String,
    aliases: String,
    claim_kind: Option<ClaimKind>,
    assertion_status: Option<AssertionStatus>,
    scan_metadata: Option<ScanClaimMetadata>,
    replacement: Option<String>,
    rank: f64,
}

#[derive(Debug)]
struct WikiPageCandidate {
    document_id: String,
    collection_id: String,
    source_project_id: Option<String>,
    kind: String,
    category: String,
    summary: String,
    locator: String,
    digest: String,
    visibility: RagVisibility,
    rank: f64,
}

#[derive(Debug)]
struct ResolvedScope {
    target_collection_id: Option<String>,
    current_collection_id: Option<String>,
    explicit_target: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimMarkdownEnvelope {
    schema_version: u32,
    claim_id: String,
    claim_key: String,
    collection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    document_id: Option<String>,
    kind: ClaimKind,
    status: AssertionStatus,
    visibility: RagVisibility,
    provenance: ClaimProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scan_metadata: Option<ScanClaimMetadata>,
    revision: u64,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    supersedes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    replacement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    observed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verified_at: Option<String>,
    digest: String,
}

/// Compute the canonical digest for a document's retrieval-bearing content.
///
/// # Panics
///
/// Panics only if Serde cannot serialize the fixed semantic tuple.
#[must_use]
pub fn document_digest(document: &CanonicalDocument) -> String {
    let material = serde_json::to_vec(&(
        "document-v2",
        &document.collection_id,
        &document.document_id,
        &document.locator,
        &document.title,
        &document.kind,
        &document.category,
        &document.body,
        document.visibility,
        document.language,
        document.revision,
        &document.tags,
        &document.aliases,
        &document.links,
        &document.sources,
        &document.replacement,
    ))
    .expect("document semantic tuple is serializable");
    sha256_digest(&material)
}

/// Compute the canonical semantic digest for a typed claim.
///
/// # Panics
///
/// Panics only if Serde cannot serialize the fixed semantic tuple.
#[must_use]
pub fn claim_digest(claim: &CanonicalClaim) -> String {
    let mut sources = claim.sources.clone();
    sources.sort();
    sources.dedup();
    let mut supersedes = claim.supersedes.clone();
    supersedes.sort();
    supersedes.dedup();
    let material = serde_json::to_vec(&(
        "claim-v2",
        &claim.claim_key,
        &claim.collection_id,
        claim.kind,
        claim.status,
        claim.visibility,
        &claim.normalized_fact,
        &claim.provenance,
        &claim.scan_metadata,
        sources,
        supersedes,
        &claim.replacement,
        &claim.observed_at,
        &claim.verified_at,
    ))
    .expect("claim semantic tuple is serializable");
    sha256_digest(&material)
}

/// Render a typed claim as deterministic canonical Markdown.
///
/// # Errors
///
/// Returns an error when the claim is invalid or frontmatter serialization fails.
pub fn render_claim_markdown(claim: &CanonicalClaim) -> Result<String, RagError> {
    validate_claim(claim)?;
    let envelope = ClaimMarkdownEnvelope {
        schema_version: CLAIM_SCHEMA_VERSION,
        claim_id: claim.claim_id.clone(),
        claim_key: claim.claim_key.clone(),
        collection_id: claim.collection_id.clone(),
        document_id: claim.document_id.clone(),
        kind: claim.kind,
        status: claim.status,
        visibility: claim.visibility,
        provenance: claim.provenance.clone(),
        scan_metadata: claim.scan_metadata.clone(),
        revision: claim.revision,
        sources: claim.sources.clone(),
        supersedes: claim.supersedes.clone(),
        replacement: claim.replacement.clone(),
        observed_at: claim.observed_at.clone(),
        verified_at: claim.verified_at.clone(),
        digest: claim.digest.clone(),
    };
    let frontmatter = serde_yaml::to_string(&envelope)
        .map_err(|error| RagError::InvalidInput(format!("serialize claim Markdown: {error}")))?;
    Ok(format!(
        "---\n{}---\n\n{}\n",
        frontmatter.trim_start_matches("---\n"),
        claim.normalized_fact.trim()
    ))
}

/// Parse and validate a typed claim from canonical Markdown.
///
/// # Errors
///
/// Returns an error for an unsafe locator, malformed frontmatter, unsupported schema,
/// or invalid claim metadata, body, or digest.
pub fn parse_claim_markdown(locator: &str, markdown: &str) -> Result<CanonicalClaim, RagError> {
    validate_locator(locator)?;
    let rest = markdown.strip_prefix("---\n").ok_or_else(|| {
        RagError::InvalidInput("claim Markdown requires YAML frontmatter".to_owned())
    })?;
    let (frontmatter, body) = rest.split_once("\n---\n").ok_or_else(|| {
        RagError::InvalidInput("claim Markdown frontmatter is not terminated".to_owned())
    })?;
    let envelope: ClaimMarkdownEnvelope = serde_yaml::from_str(frontmatter)
        .map_err(|error| RagError::InvalidInput(format!("parse claim frontmatter: {error}")))?;
    if envelope.schema_version != CLAIM_SCHEMA_VERSION {
        return Err(RagError::InvalidInput(format!(
            "unsupported claim schema_version {}",
            envelope.schema_version
        )));
    }
    let claim = CanonicalClaim {
        claim_id: envelope.claim_id,
        claim_key: envelope.claim_key,
        collection_id: envelope.collection_id,
        document_id: envelope.document_id,
        locator: locator.to_owned(),
        kind: envelope.kind,
        status: envelope.status,
        visibility: envelope.visibility,
        normalized_fact: body.trim().to_owned(),
        provenance: envelope.provenance,
        scan_metadata: envelope.scan_metadata,
        revision: envelope.revision,
        sources: envelope.sources,
        supersedes: envelope.supersedes,
        replacement: envelope.replacement,
        observed_at: envelope.observed_at,
        verified_at: envelope.verified_at,
        digest: envelope.digest,
    };
    validate_claim(&claim)?;
    Ok(claim)
}

/// Build and serialize a disposable `SQLite` projection from canonical data.
///
/// # Errors
///
/// Returns an error for invalid canonical input, projection failure, or an oversized index.
pub fn build_rag_index(snapshot: &RagSnapshot) -> Result<RagIndexArtifact, RagError> {
    let canonical = canonicalize_snapshot(snapshot)?;
    let mut manifest = generation_manifest(&canonical)?;
    let mut chunks = Vec::new();
    for document in &canonical.documents {
        chunks.extend(document_chunks(document));
    }
    for claim in &canonical.claims {
        if claim.status != AssertionStatus::Superseded {
            chunks.extend(claim_chunks(claim));
        }
    }
    chunks.sort_by(|left, right| left.chunk_id.cmp(&right.chunk_id));

    let mut connection = Connection::open_in_memory().map_err(sqlite_error)?;
    initialize_schema(&connection)?;
    let transaction = connection.transaction().map_err(sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO meta (schema_version, generation, manifest_digest, entry_count) VALUES (?1, ?2, ?3, ?4)",
            params![
                RAG_SCHEMA_VERSION,
                sql_i64("generation", manifest.generation)?,
                manifest.logical_digest,
                sql_i64_usize("entry_count", manifest.entry_count)?
            ],
        )
        .map_err(sqlite_error)?;

    for collection in &canonical.registry.collections {
        insert_collection(&transaction, collection)?;
    }
    for document in &canonical.documents {
        insert_document(&transaction, document)?;
    }
    for claim in &canonical.claims {
        insert_claim(&transaction, claim)?;
    }
    for chunk in &chunks {
        insert_chunk(&transaction, chunk)?;
    }
    for entry in manifest_entries(&canonical) {
        transaction
            .execute(
                "INSERT INTO generation_manifest (collection_id, item_kind, item_id, locator, digest, revision) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    entry.collection_id,
                    entry.item_kind,
                    entry.item_id,
                    entry.locator,
                    entry.digest,
                    sql_i64("manifest revision", entry.revision)?
                ],
            )
            .map_err(sqlite_error)?;
    }
    transaction.commit().map_err(sqlite_error)?;
    verify_projection(&connection, &manifest, &canonical.registry, chunks.len())?;
    let sqlite_bytes = connection
        .serialize(MAIN_DB)
        .map_err(sqlite_error)?
        .to_vec();
    if sqlite_bytes.len() > MAX_SERIALIZED_INDEX_BYTES {
        return Err(RagError::InvalidInput(format!(
            "serialized RAG index exceeds {MAX_SERIALIZED_INDEX_BYTES} bytes"
        )));
    }
    manifest.sqlite_digest = sha256_digest(&sqlite_bytes);
    Ok(RagIndexArtifact {
        sqlite_bytes,
        manifest,
        document_count: canonical.documents.len(),
        claim_count: canonical.claims.len(),
        chunk_count: chunks.len(),
    })
}

/// Update a verified remote-canonical projection without rewriting unchanged
/// document or chunk rows.
///
/// This intentionally accepts only document-only snapshots. Markdown-backed
/// stores continue to use [`build_rag_index`], while a complete remote
/// inventory can reuse the already-verified rows for unchanged pages. The
/// caller supplies the exact changed and deleted document IDs; their set must
/// describe the entire transition from the prior generation to `snapshot`.
///
/// # Errors
///
/// Returns an error when the prior `SQLite` generation is invalid, the registry
/// or lineage differs, a changed set is incomplete, or `SQLite` cannot apply the
/// bounded row-level transaction.
///
/// # Panics
///
/// Panics only if the validated changed-document ID set no longer resolves in
/// the canonical target snapshot during this in-memory operation.
#[allow(clippy::too_many_lines)]
pub fn build_incremental_remote_rag_index(
    previous: &RagIndexArtifact,
    snapshot: &RagSnapshot,
    changed_document_ids: &BTreeSet<String>,
    deleted_document_ids: &BTreeSet<String>,
) -> Result<RagIndexArtifact, RagError> {
    let canonical = canonicalize_snapshot(snapshot)?;
    if !canonical.claims.is_empty() {
        return Err(RagError::InvalidInput(
            "remote incremental RAG projection cannot contain local claims".to_owned(),
        ));
    }
    if previous.manifest.schema_version != RAG_SCHEMA_VERSION
        || previous.manifest.generation.checked_add(1) != Some(canonical.generation)
    {
        return Err(RagError::Conflict(
            "remote incremental RAG generation does not advance the verified prior generation"
                .to_owned(),
        ));
    }
    let prior_documents = documents_from_serialized(
        &previous.sqlite_bytes,
        &previous.manifest,
        &canonical.registry,
    )?;
    let prior_by_id = prior_documents
        .iter()
        .map(|document| (document.document_id.as_str(), document))
        .collect::<BTreeMap<_, _>>();
    let target_by_id = canonical
        .documents
        .iter()
        .map(|document| (document.document_id.as_str(), document))
        .collect::<BTreeMap<_, _>>();
    if changed_document_ids
        .iter()
        .any(|document_id| !target_by_id.contains_key(document_id.as_str()))
        || deleted_document_ids
            .iter()
            .any(|document_id| !prior_by_id.contains_key(document_id.as_str()))
        || !changed_document_ids.is_disjoint(deleted_document_ids)
    {
        return Err(RagError::InvalidInput(
            "remote incremental RAG changed document set is outside the prior or target generation"
                .to_owned(),
        ));
    }
    let mut expected_target_ids = prior_by_id
        .keys()
        .map(|document_id| (*document_id).to_owned())
        .collect::<BTreeSet<_>>();
    for document_id in deleted_document_ids {
        expected_target_ids.remove(document_id);
    }
    expected_target_ids.extend(changed_document_ids.iter().cloned());
    let actual_target_ids = target_by_id
        .keys()
        .map(|document_id| (*document_id).to_owned())
        .collect::<BTreeSet<_>>();
    if expected_target_ids != actual_target_ids {
        return Err(RagError::InvalidInput(
            "remote incremental RAG changed document set does not cover the complete inventory"
                .to_owned(),
        ));
    }

    let mut manifest = generation_manifest(&canonical)?;
    let mut connection = deserialize_connection(&previous.sqlite_bytes)?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF;",
        )
        .map_err(sqlite_error)?;
    let transaction = connection.transaction().map_err(sqlite_error)?;
    let mut replaced_ids = changed_document_ids.clone();
    replaced_ids.extend(deleted_document_ids.iter().cloned());
    for document_id in &replaced_ids {
        let prior = prior_by_id.get(document_id.as_str()).copied();
        if let Some(document) = prior {
            delete_document_projection(&transaction, document)?;
        }
    }
    for document_id in changed_document_ids {
        let document = target_by_id
            .get(document_id.as_str())
            .expect("changed documents were validated against the target inventory");
        insert_document(&transaction, document)?;
        for chunk in document_chunks(document) {
            insert_chunk(&transaction, &chunk)?;
        }
        insert_document_manifest(&transaction, document)?;
    }
    transaction
        .execute(
            "UPDATE meta SET generation = ?1, manifest_digest = ?2, entry_count = ?3",
            params![
                sql_i64("generation", manifest.generation)?,
                manifest.logical_digest,
                sql_i64_usize("entry_count", manifest.entry_count)?,
            ],
        )
        .map_err(sqlite_error)?;
    transaction.commit().map_err(sqlite_error)?;
    let chunk_count = canonical
        .documents
        .iter()
        .map(|document| document_chunks(document).len())
        .sum();
    verify_projection(&connection, &manifest, &canonical.registry, chunk_count)?;
    let sqlite_bytes = connection
        .serialize(MAIN_DB)
        .map_err(sqlite_error)?
        .to_vec();
    if sqlite_bytes.len() > MAX_SERIALIZED_INDEX_BYTES {
        return Err(RagError::InvalidInput(format!(
            "serialized RAG index exceeds {MAX_SERIALIZED_INDEX_BYTES} bytes"
        )));
    }
    manifest.sqlite_digest = sha256_digest(&sqlite_bytes);
    Ok(RagIndexArtifact {
        sqlite_bytes,
        manifest,
        document_count: canonical.documents.len(),
        claim_count: 0,
        chunk_count,
    })
}

/// Verified resident RAG generation for repeated low-latency retrieval.
///
/// Construction authenticates the exact serialized generation, checks relational
/// integrity, and canonicalizes the collection registry once. The resident `SQLite`
/// connection is immutable; every retrieval still validates the request and resolves
/// its visibility scope independently.
pub struct PreparedRagIndex {
    connection: Connection,
    manifest: GenerationManifest,
    registry: CollectionRegistry,
}

impl PreparedRagIndex {
    /// Authenticate and prepare one immutable serialized RAG generation.
    ///
    /// # Errors
    ///
    /// Returns an error when the bytes are absent, oversized, corrupt, stale, dirty,
    /// or inconsistent with the supplied manifest and collection registry.
    pub fn from_serialized(
        sqlite_bytes: &[u8],
        expected_manifest: &GenerationManifest,
        registry: &CollectionRegistry,
    ) -> Result<Self, RagError> {
        validate_serialized_index(sqlite_bytes, expected_manifest)?;
        let connection = deserialize_connection(sqlite_bytes)?;
        let registry = verify_retrieval_snapshot(&connection, expected_manifest, registry)?;
        Ok(Self {
            connection,
            manifest: expected_manifest.clone(),
            registry,
        })
    }

    /// Retrieve from the already authenticated resident generation.
    ///
    /// # Errors
    ///
    /// Returns an error when the request violates scope, visibility, query, top-k,
    /// or byte-budget constraints, or when the read-only `SQLite` query fails.
    pub fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult, RagError> {
        validate_retrieval_request(request)?;
        retrieve_from_connection(&self.connection, &self.manifest, &self.registry, request)
    }
}

/// Retrieve ranked chunks from serialized disposable index bytes.
///
/// Freshness is checked using only the supplied generation manifest and indexed
/// metadata. This function never enumerates or reads canonical Markdown.
///
/// # Errors
///
/// Returns an error for an invalid or unauthorized scope, stale or corrupt derived bytes,
/// malformed query, or failed bounded `SQLite` retrieval.
#[allow(clippy::too_many_lines)]
pub fn retrieve_serialized(
    sqlite_bytes: &[u8],
    expected_manifest: &GenerationManifest,
    registry: &CollectionRegistry,
    request: &RetrievalRequest,
) -> Result<RetrievalResult, RagError> {
    validate_retrieval_request(request)?;
    validate_serialized_index(sqlite_bytes, expected_manifest)?;
    let connection = deserialize_connection(sqlite_bytes)?;
    let canonical_registry = verify_retrieval_snapshot(&connection, expected_manifest, registry)?;
    retrieve_from_connection(&connection, expected_manifest, &canonical_registry, request)
}

/// Recover the exact document records retained in a verified disposable index.
///
/// This is used only by a remote-canonical backend to retain unchanged pages in
/// the `SQLite` projection between complete remote inventories. It never treats
/// the index as a canonical source: the caller must still perform a fresh
/// remote inventory before publishing a new generation.
///
/// # Errors
///
/// Returns an error when the serialized generation is absent, stale, corrupt,
/// dirty, or inconsistent with the supplied manifest and collection registry.
pub fn documents_from_serialized(
    sqlite_bytes: &[u8],
    expected_manifest: &GenerationManifest,
    registry: &CollectionRegistry,
) -> Result<Vec<CanonicalDocument>, RagError> {
    validate_serialized_index(sqlite_bytes, expected_manifest)?;
    let connection = deserialize_connection(sqlite_bytes)?;
    let _registry = verify_retrieval_snapshot(&connection, expected_manifest, registry)?;
    let claim_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM claims", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if claim_count != 0 {
        return Err(RagError::RepairRequired(
            "remote Notion projection must not contain local canonical claims".to_owned(),
        ));
    }
    let mut statement = connection
        .prepare(
            "SELECT document_id, collection_id, locator, title, kind, category,
                    body, digest, visibility, language, revision, replacement
             FROM documents ORDER BY document_id",
        )
        .map_err(sqlite_error)?;
    let rows = statement
        .query_map([], |row| {
            let revision = row.get::<_, i64>(10)?;
            let revision = u64::try_from(revision).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Integer,
                    Box::new(RagError::RepairRequired(
                        "indexed document revision is outside the supported range".to_owned(),
                    )),
                )
            })?;
            let visibility = RagVisibility::parse(&row.get::<_, String>(8)?)
                .map_err(to_sqlite_conversion_error)?;
            let language = RagLanguage::parse(&row.get::<_, String>(9)?)
                .map_err(to_sqlite_conversion_error)?;
            Ok(CanonicalDocument {
                document_id: row.get(0)?,
                collection_id: row.get(1)?,
                locator: row.get(2)?,
                title: row.get(3)?,
                kind: row.get(4)?,
                category: row.get(5)?,
                body: row.get(6)?,
                digest: row.get(7)?,
                visibility,
                language,
                revision,
                tags: Vec::new(),
                aliases: Vec::new(),
                links: Vec::new(),
                sources: Vec::new(),
                replacement: row.get(11)?,
            })
        })
        .map_err(sqlite_error)?;
    let mut documents = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sqlite_error)?;
    for document in &mut documents {
        document.tags = load_item_values(
            &connection,
            "tags",
            "tag",
            "document",
            &document.document_id,
        )?;
        document.aliases = load_item_values(
            &connection,
            "aliases",
            "alias",
            "document",
            &document.document_id,
        )?;
        document.links = load_item_values(
            &connection,
            "links",
            "locator",
            "document",
            &document.document_id,
        )?;
        document.sources = load_sources(&connection, "document", &document.document_id)?;
    }
    Ok(documents)
}

fn validate_serialized_index(
    sqlite_bytes: &[u8],
    expected_manifest: &GenerationManifest,
) -> Result<(), RagError> {
    if sqlite_bytes.is_empty() || sqlite_bytes.len() > MAX_SERIALIZED_INDEX_BYTES {
        return Err(RagError::RepairRequired(
            "RAG index bytes are absent or exceed the supported bound".to_owned(),
        ));
    }
    verify_serialized_digest(sqlite_bytes, expected_manifest)
}

fn verify_retrieval_snapshot(
    connection: &Connection,
    expected_manifest: &GenerationManifest,
    registry: &CollectionRegistry,
) -> Result<CollectionRegistry, RagError> {
    verify_manifest_fast(connection, expected_manifest)?;
    let dirty_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM dirty_journal", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if dirty_count != 0 {
        return Err(RagError::RepairRequired(
            "RAG index has a dirty journal; rebuild from canonical Markdown".to_owned(),
        ));
    }

    let canonical_registry = registry
        .canonicalized()
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    verify_registry_projection(connection, &canonical_registry)?;
    Ok(canonical_registry)
}

#[allow(clippy::too_many_lines)]
fn retrieve_from_connection(
    connection: &Connection,
    expected_manifest: &GenerationManifest,
    canonical_registry: &CollectionRegistry,
    request: &RetrievalRequest,
) -> Result<RetrievalResult, RagError> {
    let resolved_scope = resolve_scope(canonical_registry, request)?;
    let fts_query = build_fts_query(&request.query, &request.query_expansions)?;
    let folded_query = folded_alias(&request.query);
    let mut statement = connection
        .prepare(
            "SELECT c.chunk_id, c.item_kind, c.item_id, c.collection_id, c.title, c.text, c.locator, c.digest, c.visibility, c.language, c.tags, c.aliases, c.claim_kind, c.assertion_status, c.scan_metadata_json, c.replacement, bm25(chunks_fts, 4.0, 1.0, 2.0, 3.0) AS rank
             FROM chunks_fts
             JOIN chunks c ON c.rowid = chunks_fts.rowid
             JOIN collections co ON co.collection_id = c.collection_id
             WHERE chunks_fts MATCH ?1
               AND (
                   (?6 = 0
                       AND c.collection_id = ?2
                       AND (
                       c.visibility <> 'confidential'
                       OR ?5 = ?2
                   ))
                   OR (
                       ?3 IS NOT NULL
                       AND c.collection_id = ?3
                       AND (co.state = 'attached' OR ?6 = 1)
                       AND (
                           c.visibility IN ('shared', 'project-private')
                           OR (
                               c.visibility = 'confidential'
                               AND ?5 = ?3
                           )
                       )
                   )
                   OR (
                       ?6 = 0
                       AND c.collection_id <> ?2
                       AND c.visibility = 'shared'
                   )
               )
             ORDER BY rank ASC, c.chunk_id ASC
             LIMIT ?7",
        )
        .map_err(sqlite_error)?;
    let mapped = statement
        .query_map(
            params![
                fts_query,
                USER_ROOT_COLLECTION_ID,
                resolved_scope.target_collection_id.as_deref(),
                resolved_scope.current_collection_id.as_deref(),
                request.confidential_collection_id.as_deref(),
                i64::from(resolved_scope.explicit_target),
                sql_i64_usize("candidate limit", MAX_CANDIDATES)?
            ],
            candidate_from_row,
        )
        .map_err(sqlite_error)?;
    let by_id = canonical_registry.by_id();
    let mut candidates = Vec::new();
    for result in mapped {
        let candidate = result.map_err(sqlite_error)?;
        let collection = by_id.get(candidate.collection_id.as_str()).ok_or_else(|| {
            RagError::RepairRequired(format!(
                "indexed collection `{}` is absent from the registry",
                candidate.collection_id
            ))
        })?;
        if is_visible(&candidate, collection, &resolved_scope, request) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| compare_candidates(left, right, &folded_query));

    let mut hits = Vec::new();
    let mut returned_bytes = 0_usize;
    let mut insufficient_budget = false;
    for candidate in candidates {
        if hits.len() == request.top_k {
            insufficient_budget = true;
            break;
        }
        let remaining = request.byte_budget.saturating_sub(returned_bytes);
        if remaining == 0 {
            insufficient_budget = true;
            break;
        }
        let (text, truncated) = truncate_utf8(&candidate.text, remaining);
        if text.is_empty() {
            insufficient_budget = true;
            break;
        }
        let sources = load_sources(connection, &candidate.item_kind, &candidate.item_id)?;
        let matched_field = matched_field(&candidate, &folded_query);
        let rank_score = candidate_score(&candidate, &folded_query);
        returned_bytes += text.len();
        hits.push(RetrievalHit {
            chunk_id: candidate.chunk_id,
            collection_id: candidate.collection_id,
            item_id: candidate.item_id,
            item_kind: candidate.item_kind,
            locator: candidate.locator,
            title: candidate.title,
            text,
            digest: candidate.digest,
            visibility: candidate.visibility,
            language: candidate.language,
            claim_kind: candidate.claim_kind,
            assertion_status: candidate.assertion_status,
            scan_metadata: candidate.scan_metadata,
            score: rank_score,
            matched_field,
            sources,
            replacement: candidate.replacement,
            untrusted_content: true,
        });
        if truncated {
            insufficient_budget = true;
            break;
        }
    }
    Ok(RetrievalResult {
        generation: expected_manifest.generation,
        manifest_digest: expected_manifest.logical_digest.clone(),
        hits,
        returned_bytes,
        insufficient_budget,
    })
}

/// Query canonical Wiki pages from serialized RAG bytes without scanning Markdown.
///
/// The result preserves the legacy shared-Wiki visibility and precedence contract while
/// reading only the verified generation snapshot.
///
/// # Errors
///
/// Returns an error for an invalid filter, unknown current project, stale or corrupt
/// projection, or a read-only `SQLite` failure.
#[allow(clippy::too_many_lines)]
pub fn query_wiki_pages_serialized(
    sqlite_bytes: &[u8],
    expected_manifest: &GenerationManifest,
    registry: &CollectionRegistry,
    request: &WikiPageQueryRequest,
) -> Result<Vec<WikiPageQueryHit>, RagError> {
    if request.text.is_none() && request.tag.is_none() && request.category.is_none() {
        return Err(RagError::InvalidInput(
            "Wiki page query requires text, tag, or category".to_owned(),
        ));
    }
    if !(1..=MAX_TOP_K).contains(&request.limit) {
        return Err(RagError::InvalidInput(
            "Wiki page query limit must be from 1 through 100".to_owned(),
        ));
    }
    verify_serialized_digest(sqlite_bytes, expected_manifest)?;
    let tag = request
        .tag
        .as_deref()
        .map(|value| {
            validate_bounded_text("Wiki query tag", value, 256)?;
            Ok(value)
        })
        .transpose()?;
    let category = request
        .category
        .as_deref()
        .map(canonical_wiki_category)
        .transpose()?;
    if sqlite_bytes.is_empty() || sqlite_bytes.len() > MAX_SERIALIZED_INDEX_BYTES {
        return Err(RagError::RepairRequired(
            "RAG index bytes are absent or exceed the supported bound".to_owned(),
        ));
    }
    let connection = deserialize_connection(sqlite_bytes)?;
    verify_manifest_fast(&connection, expected_manifest)?;
    let dirty_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM dirty_journal", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if dirty_count != 0 {
        return Err(RagError::RepairRequired(
            "RAG index has a dirty journal; rebuild from canonical Markdown".to_owned(),
        ));
    }
    let canonical_registry = registry
        .canonicalized()
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    verify_registry_projection(&connection, &canonical_registry)?;
    let current_collection_id = request
        .current_project_id
        .as_deref()
        .map(|project_id| {
            resolution_to_id(
                canonical_registry.resolve_project(project_id),
                "current project",
            )
        })
        .transpose()?;
    let sql_limit = sql_i64_usize("Wiki page query limit", request.limit)?;
    let mut candidates = Vec::new();
    if let Some(text) = request.text.as_deref() {
        let expression = build_wiki_fts_query(text)?;
        let mut statement = connection
            .prepare(
                "SELECT d.document_id, d.collection_id, c.source_project_id,
                        d.kind, d.category, d.title, d.locator, d.digest,
                        d.visibility,
                        bm25(documents_fts, 4.0, 1.0, 2.0, 3.0) AS rank
                 FROM documents_fts
                 JOIN documents d ON d.rowid = documents_fts.rowid
                 JOIN collections c ON c.collection_id = d.collection_id
                 WHERE documents_fts MATCH ?1
                   AND (?2 IS NULL OR EXISTS (
                     SELECT 1 FROM tags t
                     WHERE t.item_kind = 'document'
                       AND t.item_id = d.document_id AND t.tag = ?2
                   ))
                   AND (?3 IS NULL OR d.category = ?3)
                   AND (
                     (d.collection_id = 'user-root' AND d.visibility <> 'confidential')
                     OR (d.collection_id <> 'user-root' AND d.visibility = 'shared')
                     OR (d.collection_id = ?4 AND c.state = 'attached'
                         AND d.visibility IN ('project-private', 'confidential'))
                   )
                 ORDER BY
                   CASE WHEN d.collection_id = ?4 THEN 0
                        WHEN d.collection_id = 'user-root' THEN 1
                        ELSE 2 END,
                   rank, COALESCE(c.source_project_id, 'user-root'), d.document_id
                 LIMIT ?5",
            )
            .map_err(sqlite_error)?;
        let rows = statement
            .query_map(
                params![expression, tag, category, current_collection_id, sql_limit],
                wiki_page_candidate_from_row,
            )
            .map_err(sqlite_error)?;
        candidates.extend(
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sqlite_error)?,
        );
    } else {
        let mut statement = connection
            .prepare(
                "SELECT d.document_id, d.collection_id, c.source_project_id,
                        d.kind, d.category, d.title, d.locator, d.digest,
                        d.visibility, 0.0 AS rank
                 FROM documents d
                 JOIN collections c ON c.collection_id = d.collection_id
                 WHERE (?1 IS NULL OR EXISTS (
                     SELECT 1 FROM tags t
                     WHERE t.item_kind = 'document'
                       AND t.item_id = d.document_id AND t.tag = ?1
                   ))
                   AND (?2 IS NULL OR d.category = ?2)
                   AND (
                     (d.collection_id = 'user-root' AND d.visibility <> 'confidential')
                     OR (d.collection_id <> 'user-root' AND d.visibility = 'shared')
                     OR (d.collection_id = ?3 AND c.state = 'attached'
                         AND d.visibility IN ('project-private', 'confidential'))
                   )
                 ORDER BY
                   CASE WHEN d.collection_id = ?3 THEN 0
                        WHEN d.collection_id = 'user-root' THEN 1
                        ELSE 2 END,
                   COALESCE(c.source_project_id, 'user-root'), d.document_id
                 LIMIT ?4",
            )
            .map_err(sqlite_error)?;
        let rows = statement
            .query_map(
                params![tag, category, current_collection_id, sql_limit],
                wiki_page_candidate_from_row,
            )
            .map_err(sqlite_error)?;
        candidates.extend(
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sqlite_error)?,
        );
    }
    candidates
        .into_iter()
        .map(|candidate| {
            let tags = load_item_values(
                &connection,
                "tags",
                "tag",
                "document",
                &candidate.document_id,
            )?;
            let aliases = load_item_values(
                &connection,
                "aliases",
                "alias",
                "document",
                &candidate.document_id,
            )?;
            let sources = load_sources(&connection, "document", &candidate.document_id)?;
            Ok(WikiPageQueryHit {
                document_id: candidate.document_id,
                collection_id: candidate.collection_id,
                source_project_id: candidate.source_project_id,
                kind: candidate.kind,
                category: candidate.category,
                summary: candidate.summary,
                locator: candidate.locator,
                digest: candidate.digest,
                visibility: candidate.visibility,
                tags,
                aliases,
                sources,
                rank: candidate.rank,
            })
        })
        .collect()
}

/// Create an idempotent, inference-free plan for one remember request.
///
/// # Errors
///
/// Returns an error for invalid or sensitive input, conflicting current truth,
/// an invalid supersede set, or a stale expected digest.
///
/// # Panics
///
/// Panics only if an internally resolved supersede claim disappears from the same
/// immutable active set.
#[allow(clippy::too_many_lines)]
pub fn plan_remember(
    existing: &[CanonicalClaim],
    request: &RememberRequest,
    revision: u64,
) -> Result<RememberPlan, RagError> {
    validate_remember_request(request, revision)?;
    let mut supersedes = request.supersedes.clone();
    supersedes.sort();
    supersedes.dedup();

    let mut seed = CanonicalClaim {
        claim_id: request.claim_id.clone().unwrap_or_default(),
        claim_key: request.claim_key.clone(),
        collection_id: request.collection_id.clone(),
        document_id: None,
        locator: request.locator.clone(),
        kind: request.kind,
        status: request.status,
        visibility: request.visibility,
        normalized_fact: request.normalized_fact.trim().to_owned(),
        provenance: request.provenance.clone(),
        scan_metadata: None,
        revision,
        sources: sorted_unique(request.sources.clone()),
        supersedes,
        replacement: None,
        observed_at: request.observed_at.clone(),
        verified_at: request.verified_at.clone(),
        digest: String::new(),
    };
    let pre_id_digest = claim_digest(&seed);
    if seed.claim_id.is_empty() {
        seed.claim_id = format!("claim-{}", raw_digest(&pre_id_digest));
        seed.locator = format!(
            ".hive/knowledge/Claims/{}/{}.md",
            seed.collection_id, seed.claim_id
        );
    }
    seed.digest = claim_digest(&seed);
    validate_claim(&seed)?;

    let active = existing
        .iter()
        .filter(|claim| {
            claim.collection_id == request.collection_id
                && claim.status != AssertionStatus::Superseded
        })
        .collect::<Vec<_>>();
    if let Some(exact) = active.iter().find(|claim| {
        claim.claim_key == seed.claim_key
            && claim.digest == seed.digest
            && claim.normalized_fact == seed.normalized_fact
    }) {
        if seed.supersedes.is_empty() {
            let _ = exact;
            return Ok(RememberPlan {
                disposition: RememberDisposition::Noop,
                new_claim: None,
                superseded_claims: Vec::new(),
            });
        }
    }

    if let Some(collision) = existing
        .iter()
        .find(|claim| claim.claim_id == seed.claim_id && claim.digest != seed.digest)
    {
        return Err(RagError::Conflict(format!(
            "claim_id `{}` already refers to digest {}",
            collision.claim_id, collision.digest
        )));
    }

    let mut superseded_claims = Vec::new();
    for superseded_id in &seed.supersedes {
        let claim = active
            .iter()
            .find(|claim| claim.claim_id == *superseded_id)
            .ok_or_else(|| {
                RagError::Conflict(format!(
                    "superseded claim `{superseded_id}` is not active in collection `{}`",
                    seed.collection_id
                ))
            })?;
        if claim.claim_key != seed.claim_key {
            return Err(RagError::Conflict(format!(
                "claim `{superseded_id}` has a different current-truth key"
            )));
        }
        let mut rewritten = (*claim).clone();
        rewritten.status = AssertionStatus::Superseded;
        rewritten.replacement = Some(seed.claim_id.clone());
        rewritten.revision = revision;
        rewritten.digest = claim_digest(&rewritten);
        superseded_claims.push(rewritten);
    }
    if let Some(expected) = &request.expected_active_digest {
        if superseded_claims.len() != 1 {
            return Err(RagError::Conflict(
                "expected_active_digest requires exactly one superseded claim".to_owned(),
            ));
        }
        let original = active
            .iter()
            .find(|claim| claim.claim_id == seed.supersedes[0])
            .expect("superseded claim was resolved");
        if original.digest != *expected {
            return Err(RagError::Conflict(format!(
                "active claim digest changed: expected {expected}, found {}",
                original.digest
            )));
        }
    }
    if seed.supersedes.is_empty()
        && active.iter().any(|claim| {
            claim.claim_key == seed.claim_key && claim.normalized_fact != seed.normalized_fact
        })
    {
        return Err(RagError::Conflict(format!(
            "current truth `{}` already has a divergent active claim; provide explicit supersedes",
            seed.claim_key
        )));
    }

    Ok(RememberPlan {
        disposition: if superseded_claims.is_empty() {
            RememberDisposition::Insert
        } else {
            RememberDisposition::Supersede
        },
        new_claim: Some(seed),
        superseded_claims,
    })
}

/// Select active current truth without interpreting or merging claim text.
#[must_use]
pub fn current_truth(claims: &[CanonicalClaim], collection_id: &str) -> CurrentTruth {
    let mut active_claims = claims
        .iter()
        .filter(|claim| {
            claim.collection_id == collection_id && claim.status != AssertionStatus::Superseded
        })
        .cloned()
        .collect::<Vec<_>>();
    active_claims.sort_by(|left, right| {
        left.claim_key
            .cmp(&right.claim_key)
            .then_with(|| left.claim_id.cmp(&right.claim_id))
    });
    let mut by_key = BTreeMap::<String, BTreeMap<String, Vec<String>>>::new();
    for claim in &active_claims {
        by_key
            .entry(claim.claim_key.clone())
            .or_default()
            .entry(claim.digest.clone())
            .or_default()
            .push(claim.claim_id.clone());
    }
    let conflicts = by_key
        .into_iter()
        .filter_map(|(key, digests)| {
            (digests.len() > 1).then(|| {
                let mut ids = digests.into_values().flatten().collect::<Vec<_>>();
                ids.sort();
                (key, ids)
            })
        })
        .collect();
    CurrentTruth {
        active_claims,
        conflicts,
    }
}

/// Validate and canonicalize a crash-recovery journal.
///
/// # Errors
///
/// Returns an error for invalid generations, digests, locators, or an empty journal.
pub fn canonicalize_dirty_journal(journal: &DirtyJournal) -> Result<DirtyJournal, RagError> {
    if journal.base_generation == 0 || journal.target_generation <= journal.base_generation {
        return Err(RagError::InvalidInput(
            "dirty journal target_generation must advance a non-zero base_generation".to_owned(),
        ));
    }
    validate_sha256("base_manifest_digest", &journal.base_manifest_digest)?;
    if journal.entries.is_empty() {
        return Err(RagError::InvalidInput(
            "dirty journal requires at least one canonical entry".to_owned(),
        ));
    }
    let mut canonical = journal.clone();
    for entry in &canonical.entries {
        validate_bounded_text("dirty collection_id", &entry.collection_id, 256)?;
        validate_locator(&entry.locator)?;
        validate_sha256("target_digest", &entry.target_digest)?;
    }
    canonical.entries.sort();
    canonical.entries.dedup();
    Ok(canonical)
}

/// Record a dirty journal inside an open disposable projection before canonical writes.
///
/// # Errors
///
/// Returns an error when the journal is invalid or the `SQLite` transaction fails.
pub fn write_dirty_journal(
    connection: &mut Connection,
    journal: &DirtyJournal,
) -> Result<(), RagError> {
    let canonical = canonicalize_dirty_journal(journal)?;
    let transaction = connection.transaction().map_err(sqlite_error)?;
    transaction
        .execute("DELETE FROM dirty_journal", [])
        .map_err(sqlite_error)?;
    let payload = serde_json::to_string(&canonical)
        .map_err(|error| RagError::InvalidInput(format!("serialize dirty journal: {error}")))?;
    transaction
        .execute(
            "INSERT INTO dirty_journal (singleton, base_generation, target_generation, base_manifest_digest, payload) VALUES (1, ?1, ?2, ?3, ?4)",
            params![
                sql_i64("dirty base_generation", canonical.base_generation)?,
                sql_i64("dirty target_generation", canonical.target_generation)?,
                canonical.base_manifest_digest,
                payload
            ],
        )
        .map_err(sqlite_error)?;
    transaction.commit().map_err(sqlite_error)
}

/// Clear a dirty journal only after the rebuilt projection is verified and published.
///
/// # Errors
///
/// Returns an error when the `SQLite` update fails.
pub fn clear_dirty_journal(connection: &Connection) -> Result<(), RagError> {
    connection
        .execute("DELETE FROM dirty_journal", [])
        .map_err(sqlite_error)?;
    Ok(())
}

/// Decide recovery from indexed lineage, an optional journal, and canonical manifest.
#[must_use]
pub fn dirty_recovery_decision(
    indexed_manifest: &GenerationManifest,
    journal: Option<&DirtyJournal>,
    canonical_manifest: &GenerationManifest,
) -> DirtyRecoveryDecision {
    let Some(journal) = journal else {
        return if indexed_manifest == canonical_manifest {
            DirtyRecoveryDecision::Clean
        } else {
            DirtyRecoveryDecision::RebuildRequired
        };
    };
    if journal.base_generation != indexed_manifest.generation
        || journal.base_manifest_digest != indexed_manifest.logical_digest
        || journal.target_generation <= journal.base_generation
    {
        return DirtyRecoveryDecision::Conflict;
    }
    DirtyRecoveryDecision::RebuildRequired
}

fn canonicalize_snapshot(snapshot: &RagSnapshot) -> Result<RagSnapshot, RagError> {
    if snapshot.schema_version != RAG_SCHEMA_VERSION {
        return Err(RagError::InvalidInput(format!(
            "unsupported RAG schema_version {}",
            snapshot.schema_version
        )));
    }
    if snapshot.generation == 0 {
        return Err(RagError::InvalidInput(
            "RAG generation must be greater than zero".to_owned(),
        ));
    }
    let mut canonical = snapshot.clone();
    canonical.registry = canonical
        .registry
        .canonicalized()
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    let collection_ids = canonical
        .registry
        .collections
        .iter()
        .map(|collection| collection.collection_id.as_str())
        .collect::<BTreeSet<_>>();
    let collection_visibilities = canonical
        .registry
        .collections
        .iter()
        .map(|collection| {
            (
                collection.collection_id.as_str(),
                RagVisibility::from(collection.default_visibility),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut document_ids = BTreeSet::new();
    for document in &mut canonical.documents {
        validate_document(document, &collection_ids)?;
        if collection_visibilities.get(document.collection_id.as_str())
            != Some(&document.visibility)
        {
            return Err(RagError::InvalidInput(format!(
                "document `{}` visibility must match its collection",
                document.document_id
            )));
        }
        if !document_ids.insert(document.document_id.clone()) {
            return Err(RagError::InvalidInput(format!(
                "duplicate document_id `{}`",
                document.document_id
            )));
        }
        document.tags = sorted_unique(std::mem::take(&mut document.tags));
        document.aliases = sorted_unique(std::mem::take(&mut document.aliases));
        document.links = sorted_unique(std::mem::take(&mut document.links));
        document.sources = sorted_unique(std::mem::take(&mut document.sources));
    }
    canonical
        .documents
        .sort_by(|left, right| left.document_id.cmp(&right.document_id));

    let mut claim_ids = BTreeSet::new();
    for claim in &mut canonical.claims {
        validate_claim(claim)?;
        if !collection_ids.contains(claim.collection_id.as_str()) {
            return Err(RagError::InvalidInput(format!(
                "claim `{}` references unknown collection `{}`",
                claim.claim_id, claim.collection_id
            )));
        }
        if claim
            .document_id
            .as_ref()
            .is_some_and(|document_id| !document_ids.contains(document_id))
        {
            return Err(RagError::InvalidInput(format!(
                "claim `{}` references unknown document",
                claim.claim_id
            )));
        }
        if !claim_ids.insert(claim.claim_id.clone()) {
            return Err(RagError::InvalidInput(format!(
                "duplicate claim_id `{}`",
                claim.claim_id
            )));
        }
        claim.sources = sorted_unique(std::mem::take(&mut claim.sources));
        claim.supersedes = sorted_unique(std::mem::take(&mut claim.supersedes));
    }
    canonical
        .claims
        .sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    Ok(canonical)
}

fn generation_manifest(snapshot: &RagSnapshot) -> Result<GenerationManifest, RagError> {
    let entries = manifest_entries(snapshot);
    let collections = snapshot
        .registry
        .collections
        .iter()
        .map(|collection| {
            (
                &collection.collection_id,
                collection.kind,
                collection.state,
                &collection.aliases,
                &collection.source_project_id,
                collection.default_visibility,
            )
        })
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&(
        RAG_SCHEMA_VERSION,
        snapshot.generation,
        collections,
        &entries,
    ))
    .map_err(|error| RagError::InvalidInput(format!("serialize generation manifest: {error}")))?;
    Ok(GenerationManifest {
        schema_version: RAG_SCHEMA_VERSION,
        generation: snapshot.generation,
        logical_digest: sha256_digest(&bytes),
        entry_count: entries.len(),
        sqlite_digest: String::new(),
    })
}

fn manifest_entries(snapshot: &RagSnapshot) -> Vec<ManifestEntry> {
    let mut entries = snapshot
        .documents
        .iter()
        .map(|document| ManifestEntry {
            collection_id: document.collection_id.clone(),
            item_kind: "document".to_owned(),
            item_id: document.document_id.clone(),
            locator: document.locator.clone(),
            digest: document.digest.clone(),
            revision: document.revision,
        })
        .chain(snapshot.claims.iter().map(|claim| ManifestEntry {
            collection_id: claim.collection_id.clone(),
            item_kind: "claim".to_owned(),
            item_id: claim.claim_id.clone(),
            locator: claim.locator.clone(),
            digest: claim.digest.clone(),
            revision: claim.revision,
        }))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

#[allow(clippy::too_many_lines)]
fn initialize_schema(connection: &Connection) -> Result<(), RagError> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = OFF;
             PRAGMA synchronous = OFF;
             PRAGMA temp_store = MEMORY;
             CREATE TABLE meta (
                 schema_version INTEGER NOT NULL,
                 generation INTEGER NOT NULL,
                 manifest_digest TEXT NOT NULL,
                 entry_count INTEGER NOT NULL
             );
             CREATE TABLE collections (
                 collection_id TEXT PRIMARY KEY,
                 kind TEXT NOT NULL,
                 state TEXT NOT NULL,
                 source_project_id TEXT,
                 default_visibility TEXT NOT NULL
             );
             CREATE TABLE collection_aliases (
                 collection_id TEXT NOT NULL REFERENCES collections(collection_id),
                 alias TEXT NOT NULL,
                 alias_folded TEXT NOT NULL,
                 PRIMARY KEY (collection_id, alias_folded)
             );
             CREATE INDEX collection_aliases_folded ON collection_aliases(alias_folded);
             CREATE TABLE documents (
                 document_id TEXT PRIMARY KEY,
                 collection_id TEXT NOT NULL REFERENCES collections(collection_id),
                 locator TEXT NOT NULL,
                 title TEXT NOT NULL,
                 body TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 category TEXT NOT NULL,
                 digest TEXT NOT NULL,
                 visibility TEXT NOT NULL,
                 language TEXT NOT NULL,
                 revision INTEGER NOT NULL,
                 replacement TEXT
             );
             CREATE INDEX documents_category ON documents(category, collection_id, document_id);
             CREATE INDEX documents_chunk_integrity
                 ON documents(document_id, collection_id, visibility);
             CREATE VIRTUAL TABLE documents_fts USING fts5(
                 title,
                 body,
                 tags,
                 aliases,
                 content = '',
                 tokenize = 'unicode61 remove_diacritics 2'
             );
             CREATE TABLE claims (
                 claim_id TEXT PRIMARY KEY,
                 claim_key TEXT NOT NULL,
                 collection_id TEXT NOT NULL REFERENCES collections(collection_id),
                 document_id TEXT,
                 locator TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 status TEXT NOT NULL,
                 visibility TEXT NOT NULL,
                 normalized_fact TEXT NOT NULL,
                 provenance_json TEXT NOT NULL,
                 scan_metadata_json TEXT,
                 revision INTEGER NOT NULL,
                 digest TEXT NOT NULL,
                 replacement TEXT
             );
             CREATE INDEX claims_current_truth ON claims(collection_id, claim_key, status);
             CREATE TABLE chunks (
                 chunk_id TEXT PRIMARY KEY,
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 collection_id TEXT NOT NULL REFERENCES collections(collection_id),
                 ordinal INTEGER NOT NULL,
                 title TEXT NOT NULL,
                 text TEXT NOT NULL,
                 locator TEXT NOT NULL,
                 digest TEXT NOT NULL,
                 visibility TEXT NOT NULL,
                 language TEXT NOT NULL,
                 tags TEXT NOT NULL,
                 aliases TEXT NOT NULL,
                 claim_kind TEXT,
                 assertion_status TEXT,
                 scan_metadata_json TEXT,
                 revision INTEGER NOT NULL,
                 replacement TEXT,
                 untrusted_content INTEGER NOT NULL CHECK (untrusted_content = 1),
                 UNIQUE (item_kind, item_id, ordinal)
             );
             CREATE INDEX chunks_integrity
                 ON chunks(item_kind, item_id, collection_id, visibility, claim_kind, assertion_status);
             CREATE INDEX chunks_invalid_kind
                 ON chunks(item_kind) WHERE item_kind NOT IN ('document', 'claim');
             CREATE VIRTUAL TABLE chunks_fts USING fts5(
                 title,
                 text,
                 tags,
                 aliases,
                 content = '',
                 tokenize = 'unicode61 remove_diacritics 2'
             );
             CREATE TABLE tags (
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 tag TEXT NOT NULL,
                 PRIMARY KEY (item_kind, item_id, tag)
             );
             CREATE TABLE aliases (
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 alias TEXT NOT NULL,
                 alias_folded TEXT NOT NULL,
                 PRIMARY KEY (item_kind, item_id, alias_folded)
             );
             CREATE INDEX aliases_folded ON aliases(alias_folded);
             CREATE TABLE links (
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 locator TEXT NOT NULL,
                 PRIMARY KEY (item_kind, item_id, locator)
             );
             CREATE TABLE sources (
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 locator TEXT NOT NULL,
                 PRIMARY KEY (item_kind, item_id, locator)
             );
             CREATE TABLE replacements (
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 replacement TEXT NOT NULL,
                 PRIMARY KEY (item_kind, item_id)
             );
             CREATE TABLE generation_manifest (
                 collection_id TEXT NOT NULL,
                 item_kind TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 locator TEXT NOT NULL,
                 digest TEXT NOT NULL,
                 revision INTEGER NOT NULL,
                 PRIMARY KEY (item_kind, item_id)
             );
             CREATE TABLE dirty_journal (
                 singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                 base_generation INTEGER NOT NULL,
                 target_generation INTEGER NOT NULL,
                 base_manifest_digest TEXT NOT NULL,
                 payload TEXT NOT NULL
             );",
        )
        .map_err(sqlite_error)
}

fn insert_collection(
    connection: &Connection,
    collection: &CollectionRecord,
) -> Result<(), RagError> {
    connection
        .execute(
            "INSERT INTO collections (collection_id, kind, state, source_project_id, default_visibility) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                collection.collection_id,
                collection_kind_str(collection.kind),
                collection_state_str(collection.state),
                collection.source_project_id,
                collection_visibility_str(collection.default_visibility)
            ],
        )
        .map_err(sqlite_error)?;
    for alias in &collection.aliases {
        connection
            .execute(
                "INSERT INTO collection_aliases (collection_id, alias, alias_folded) VALUES (?1, ?2, ?3)",
                params![collection.collection_id, alias, folded_alias(alias)],
            )
            .map_err(sqlite_error)?;
    }
    Ok(())
}

fn insert_document(connection: &Connection, document: &CanonicalDocument) -> Result<(), RagError> {
    connection
        .execute(
            "INSERT INTO documents (document_id, collection_id, locator, title, body, kind, category, digest, visibility, language, revision, replacement) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                document.document_id,
                document.collection_id,
                document.locator,
                document.title,
                document.body,
                document.kind,
                document.category,
                document.digest,
                document.visibility.as_str(),
                document.language.as_str(),
                sql_i64("document revision", document.revision)?,
                document.replacement
            ],
        )
        .map_err(sqlite_error)?;
    connection
        .execute(
            "INSERT INTO documents_fts (rowid, title, body, tags, aliases) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                connection.last_insert_rowid(),
                document.title,
                document.body,
                document.tags.join(" "),
                document.aliases.join(" ")
            ],
        )
        .map_err(sqlite_error)?;
    insert_search_metadata(
        connection,
        "document",
        &document.document_id,
        &document.tags,
        &document.aliases,
        &document.links,
        &document.sources,
        document.replacement.as_deref(),
    )
}

fn insert_document_manifest(
    connection: &Connection,
    document: &CanonicalDocument,
) -> Result<(), RagError> {
    connection
        .execute(
            "INSERT INTO generation_manifest (collection_id, item_kind, item_id, locator, digest, revision) VALUES (?1, 'document', ?2, ?3, ?4, ?5)",
            params![
                document.collection_id,
                document.document_id,
                document.locator,
                document.digest,
                sql_i64("manifest revision", document.revision)?,
            ],
        )
        .map_err(sqlite_error)?;
    Ok(())
}

fn delete_document_projection(
    connection: &Connection,
    document: &CanonicalDocument,
) -> Result<(), RagError> {
    let document_rowid: i64 = connection
        .query_row(
            "SELECT rowid FROM documents WHERE document_id = ?1",
            [&document.document_id],
            |row| row.get(0),
        )
        .map_err(sqlite_error)?;
    connection
        .execute(
            "INSERT INTO documents_fts(documents_fts, rowid, title, body, tags, aliases) VALUES ('delete', ?1, ?2, ?3, ?4, ?5)",
            params![
                document_rowid,
                document.title,
                document.body,
                document.tags.join(" "),
                document.aliases.join(" "),
            ],
        )
        .map_err(sqlite_error)?;
    for chunk in document_chunks(document) {
        let chunk_rowid: i64 = connection
            .query_row(
                "SELECT rowid FROM chunks WHERE chunk_id = ?1",
                [&chunk.chunk_id],
                |row| row.get(0),
            )
            .map_err(sqlite_error)?;
        connection
            .execute(
                "INSERT INTO chunks_fts(chunks_fts, rowid, title, text, tags, aliases) VALUES ('delete', ?1, ?2, ?3, ?4, ?5)",
                params![chunk_rowid, chunk.title, chunk.text, chunk.tags, chunk.aliases],
            )
            .map_err(sqlite_error)?;
    }
    connection
        .execute(
            "DELETE FROM chunks WHERE item_kind = 'document' AND item_id = ?1",
            [&document.document_id],
        )
        .map_err(sqlite_error)?;
    for table in ["tags", "aliases", "links", "sources", "replacements"] {
        connection
            .execute(
                &format!("DELETE FROM {table} WHERE item_kind = 'document' AND item_id = ?1"),
                [&document.document_id],
            )
            .map_err(sqlite_error)?;
    }
    connection
        .execute(
            "DELETE FROM generation_manifest WHERE item_kind = 'document' AND item_id = ?1",
            [&document.document_id],
        )
        .map_err(sqlite_error)?;
    connection
        .execute(
            "DELETE FROM documents WHERE document_id = ?1",
            [&document.document_id],
        )
        .map_err(sqlite_error)?;
    Ok(())
}

fn insert_claim(connection: &Connection, claim: &CanonicalClaim) -> Result<(), RagError> {
    let provenance = serde_json::to_string(&claim.provenance)
        .map_err(|error| RagError::InvalidInput(format!("serialize claim provenance: {error}")))?;
    let scan_metadata = claim
        .scan_metadata
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| {
            RagError::InvalidInput(format!("serialize scan claim metadata: {error}"))
        })?;
    connection
        .execute(
            "INSERT INTO claims (claim_id, claim_key, collection_id, document_id, locator, kind, status, visibility, normalized_fact, provenance_json, scan_metadata_json, revision, digest, replacement) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                claim.claim_id,
                claim.claim_key,
                claim.collection_id,
                claim.document_id,
                claim.locator,
                claim.kind.as_str(),
                claim.status.as_str(),
                claim.visibility.as_str(),
                claim.normalized_fact,
                provenance,
                scan_metadata,
                sql_i64("claim revision", claim.revision)?,
                claim.digest,
                claim.replacement
            ],
        )
        .map_err(sqlite_error)?;
    insert_search_metadata(
        connection,
        "claim",
        &claim.claim_id,
        &[],
        &[],
        &[],
        &claim.sources,
        claim.replacement.as_deref(),
    )
}

#[allow(clippy::too_many_arguments)]
fn insert_search_metadata(
    connection: &Connection,
    item_kind: &str,
    item_id: &str,
    tags: &[String],
    aliases: &[String],
    links: &[String],
    sources: &[String],
    replacement: Option<&str>,
) -> Result<(), RagError> {
    for tag in tags {
        connection
            .execute(
                "INSERT INTO tags (item_kind, item_id, tag) VALUES (?1, ?2, ?3)",
                params![item_kind, item_id, tag],
            )
            .map_err(sqlite_error)?;
    }
    for alias in aliases {
        connection
            .execute(
                "INSERT INTO aliases (item_kind, item_id, alias, alias_folded) VALUES (?1, ?2, ?3, ?4)",
                params![item_kind, item_id, alias, folded_alias(alias)],
            )
            .map_err(sqlite_error)?;
    }
    for link in links {
        connection
            .execute(
                "INSERT INTO links (item_kind, item_id, locator) VALUES (?1, ?2, ?3)",
                params![item_kind, item_id, link],
            )
            .map_err(sqlite_error)?;
    }
    for source in sources {
        connection
            .execute(
                "INSERT INTO sources (item_kind, item_id, locator) VALUES (?1, ?2, ?3)",
                params![item_kind, item_id, source],
            )
            .map_err(sqlite_error)?;
    }
    if let Some(replacement) = replacement {
        connection
            .execute(
                "INSERT INTO replacements (item_kind, item_id, replacement) VALUES (?1, ?2, ?3)",
                params![item_kind, item_id, replacement],
            )
            .map_err(sqlite_error)?;
    }
    Ok(())
}

fn insert_chunk(connection: &Connection, chunk: &IndexedChunk) -> Result<(), RagError> {
    let scan_metadata = chunk
        .scan_metadata
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| {
            RagError::InvalidInput(format!("serialize scan chunk metadata: {error}"))
        })?;
    connection
        .execute(
            "INSERT INTO chunks (chunk_id, item_kind, item_id, collection_id, ordinal, title, text, locator, digest, visibility, language, tags, aliases, claim_kind, assertion_status, scan_metadata_json, revision, replacement, untrusted_content) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 1)",
            params![
                chunk.chunk_id,
                chunk.item_kind,
                chunk.item_id,
                chunk.collection_id,
                sql_i64_usize("chunk ordinal", chunk.ordinal)?,
                chunk.title,
                chunk.text,
                chunk.locator,
                chunk.digest,
                chunk.visibility.as_str(),
                chunk.language.as_str(),
                chunk.tags,
                chunk.aliases,
                chunk.claim_kind.map(ClaimKind::as_str),
                chunk.assertion_status.map(AssertionStatus::as_str),
                scan_metadata,
                sql_i64("chunk revision", chunk.revision)?,
                chunk.replacement
            ],
        )
        .map_err(sqlite_error)?;
    connection
        .execute(
            "INSERT INTO chunks_fts (rowid, title, text, tags, aliases) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                connection.last_insert_rowid(),
                chunk.title,
                chunk.text,
                chunk.tags,
                chunk.aliases
            ],
        )
        .map_err(sqlite_error)?;
    Ok(())
}

fn document_chunks(document: &CanonicalDocument) -> Vec<IndexedChunk> {
    let tags = document.tags.join(" ");
    let aliases = document.aliases.join(" ");
    split_text_chunks(&document.body)
        .into_iter()
        .enumerate()
        .map(|(ordinal, text)| {
            let material = format!(
                "chunk-v1\0document\0{}\0{ordinal}\0{}\0{text}",
                document.document_id, document.digest
            );
            IndexedChunk {
                chunk_id: format!("chunk-{}", sha256_digest(material.as_bytes())),
                item_kind: "document".to_owned(),
                item_id: document.document_id.clone(),
                collection_id: document.collection_id.clone(),
                ordinal,
                title: document.title.clone(),
                text,
                locator: format!("{}#chunk={ordinal}", document.locator),
                digest: document.digest.clone(),
                visibility: document.visibility,
                language: document.language,
                tags: tags.clone(),
                aliases: aliases.clone(),
                claim_kind: None,
                assertion_status: None,
                scan_metadata: None,
                revision: document.revision,
                replacement: document.replacement.clone(),
            }
        })
        .collect()
}

fn claim_chunks(claim: &CanonicalClaim) -> Vec<IndexedChunk> {
    split_text_chunks(&claim.normalized_fact)
        .into_iter()
        .enumerate()
        .map(|(ordinal, text)| {
            let material = format!(
                "chunk-v1\0claim\0{}\0{ordinal}\0{}\0{text}",
                claim.claim_id, claim.digest
            );
            IndexedChunk {
                chunk_id: format!("chunk-{}", sha256_digest(material.as_bytes())),
                item_kind: "claim".to_owned(),
                item_id: claim.claim_id.clone(),
                collection_id: claim.collection_id.clone(),
                ordinal,
                title: claim.claim_key.clone(),
                text,
                locator: format!("{}#chunk={ordinal}", claim.locator),
                digest: claim.digest.clone(),
                visibility: claim.visibility,
                language: RagLanguage::Und,
                tags: claim.kind.as_str().to_owned(),
                aliases: claim.claim_key.clone(),
                claim_kind: Some(claim.kind),
                assertion_status: Some(claim.status),
                scan_metadata: claim.scan_metadata.clone(),
                revision: claim.revision,
                replacement: claim.replacement.clone(),
            }
        })
        .collect()
}

fn split_text_chunks(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut chunks = Vec::new();
    let mut current = String::new();
    for paragraph in normalized
        .split("\n\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if paragraph.len() > CHUNK_TARGET_BYTES {
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            chunks.extend(split_oversized_paragraph(paragraph));
            continue;
        }
        let separator_bytes = usize::from(!current.is_empty()) * 2;
        if current.len() + separator_bytes + paragraph.len() > CHUNK_TARGET_BYTES {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(paragraph);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn split_oversized_paragraph(paragraph: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0_usize;
    while start < paragraph.len() {
        let mut end = (start + CHUNK_TARGET_BYTES).min(paragraph.len());
        while !paragraph.is_char_boundary(end) {
            end -= 1;
        }
        if end < paragraph.len() {
            if let Some(relative) = paragraph[start..end].rfind(char::is_whitespace) {
                if relative > CHUNK_TARGET_BYTES / 2 {
                    end = start + relative;
                }
            }
        }
        let chunk = paragraph[start..end].trim();
        if !chunk.is_empty() {
            chunks.push(chunk.to_owned());
        }
        start = end;
        while start < paragraph.len() {
            let Some(character) = paragraph[start..].chars().next() else {
                break;
            };
            if !character.is_whitespace() {
                break;
            }
            start += character.len_utf8();
        }
    }
    chunks
}

fn verify_projection(
    connection: &Connection,
    manifest: &GenerationManifest,
    registry: &CollectionRegistry,
    chunk_count: usize,
) -> Result<(), RagError> {
    verify_manifest_fast(connection, manifest)?;
    verify_registry_projection(connection, registry)?;
    let indexed_entries: i64 = connection
        .query_row("SELECT COUNT(*) FROM generation_manifest", [], |row| {
            row.get(0)
        })
        .map_err(sqlite_error)?;
    let indexed_chunks: i64 = connection
        .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let fts_chunks: i64 = connection
        .query_row("SELECT COUNT(*) FROM chunks_fts", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let indexed_documents: i64 = connection
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let fts_documents: i64 = connection
        .query_row("SELECT COUNT(*) FROM documents_fts", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let expected_entries = sql_i64_usize("entry count", manifest.entry_count)?;
    let expected_chunks = sql_i64_usize("chunk count", chunk_count)?;
    if indexed_entries != expected_entries
        || indexed_chunks != expected_chunks
        || fts_chunks != expected_chunks
        || indexed_documents != fts_documents
    {
        return Err(RagError::RepairRequired(
            "RAG projection count verification failed".to_owned(),
        ));
    }
    Ok(())
}

fn verify_manifest_fast(
    connection: &Connection,
    expected: &GenerationManifest,
) -> Result<(), RagError> {
    if expected.schema_version != RAG_SCHEMA_VERSION {
        return Err(RagError::RepairRequired(
            "expected manifest uses an unsupported schema".to_owned(),
        ));
    }
    let (schema_version, generation, logical_digest, entry_count) = connection
        .query_row(
            "SELECT schema_version, generation, manifest_digest, entry_count FROM meta",
            [],
            |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .map_err(sqlite_error)?;
    let generation = u64::try_from(generation).map_err(|_| {
        RagError::RepairRequired("indexed generation is outside the supported range".to_owned())
    })?;
    let entry_count = usize::try_from(entry_count).map_err(|_| {
        RagError::RepairRequired("indexed entry_count is outside the supported range".to_owned())
    })?;
    if schema_version != expected.schema_version
        || generation != expected.generation
        || logical_digest != expected.logical_digest
        || entry_count != expected.entry_count
    {
        return Err(RagError::RepairRequired(
            "RAG generation manifest is stale or mismatched".to_owned(),
        ));
    }
    verify_relational_integrity(connection)
}

fn verify_serialized_digest(
    sqlite_bytes: &[u8],
    expected: &GenerationManifest,
) -> Result<(), RagError> {
    if sqlite_bytes.is_empty() || sqlite_bytes.len() > MAX_SERIALIZED_INDEX_BYTES {
        return Err(RagError::RepairRequired(
            "RAG index bytes are absent or exceed the supported bound".to_owned(),
        ));
    }
    validate_sha256("manifest sqlite_digest", &expected.sqlite_digest).map_err(|_| {
        RagError::RepairRequired("RAG manifest lacks a valid SQLite payload digest".to_owned())
    })?;
    if sha256_digest(sqlite_bytes) != expected.sqlite_digest {
        return Err(RagError::RepairRequired(
            "RAG SQLite payload differs from its published generation digest".to_owned(),
        ));
    }
    Ok(())
}

fn verify_registry_projection(
    connection: &Connection,
    registry: &CollectionRegistry,
) -> Result<(), RagError> {
    let mut collection_statement = connection
        .prepare(
            "SELECT collection_id, kind, state, source_project_id, default_visibility
             FROM collections ORDER BY collection_id",
        )
        .map_err(sqlite_error)?;
    let indexed_collections = collection_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(sqlite_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sqlite_error)?;
    let expected_collections = registry
        .collections
        .iter()
        .map(|collection| {
            (
                collection.collection_id.clone(),
                collection_kind_str(collection.kind).to_owned(),
                collection_state_str(collection.state).to_owned(),
                collection.source_project_id.clone(),
                collection_visibility_str(collection.default_visibility).to_owned(),
            )
        })
        .collect::<Vec<_>>();
    if indexed_collections != expected_collections {
        return Err(RagError::RepairRequired(
            "indexed collection registry differs from canonical registry".to_owned(),
        ));
    }

    let mut alias_statement = connection
        .prepare(
            "SELECT collection_id, alias, alias_folded
             FROM collection_aliases ORDER BY collection_id, alias_folded",
        )
        .map_err(sqlite_error)?;
    let indexed_aliases = alias_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(sqlite_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sqlite_error)?;
    let mut expected_aliases = registry
        .collections
        .iter()
        .flat_map(|collection| {
            collection.aliases.iter().map(|alias| {
                (
                    collection.collection_id.clone(),
                    alias.clone(),
                    folded_alias(alias),
                )
            })
        })
        .collect::<Vec<_>>();
    expected_aliases
        .sort_by(|left, right| (&left.0, &left.2, &left.1).cmp(&(&right.0, &right.2, &right.1)));
    if indexed_aliases != expected_aliases {
        return Err(RagError::RepairRequired(
            "indexed collection aliases differ from canonical registry".to_owned(),
        ));
    }
    Ok(())
}

fn verify_relational_integrity(connection: &Connection) -> Result<(), RagError> {
    verify_document_relations(connection)?;
    verify_chunk_relations(connection)?;
    verify_indexed_scan_metadata(connection)?;
    let invalid_chunk_kind = relational_violation(
        connection,
        "SELECT EXISTS(
             SELECT 1 FROM chunks
             WHERE item_kind NOT IN ('document', 'claim')
             LIMIT 1
         )",
    )?;
    if invalid_chunk_kind {
        return Err(RagError::RepairRequired(
            "indexed chunk has an unsupported item kind".to_owned(),
        ));
    }
    Ok(())
}

fn verify_document_relations(connection: &Connection) -> Result<(), RagError> {
    let invalid_documents = relational_violation(
        connection,
        "SELECT EXISTS(
                 SELECT 1
                 FROM documents d
                 LEFT JOIN collections c ON c.collection_id = d.collection_id
                 WHERE c.collection_id IS NULL
                    OR d.visibility != c.default_visibility
                    OR d.category != CASE d.kind
                        WHEN 'source' THEN 'source'
                        WHEN 'source-summary' THEN 'source'
                        WHEN 'entity' THEN 'entity'
                        WHEN 'concept' THEN 'concept'
                        WHEN 'comparison' THEN 'comparison'
                        WHEN 'synthesis' THEN 'synthesis'
                        WHEN 'question' THEN 'question'
                        WHEN 'open-question' THEN 'question'
                        WHEN 'decision' THEN 'decision'
                        WHEN 'workflow' THEN 'workflow'
                        ELSE ''
                    END
                 LIMIT 1
             )",
    )?;
    if invalid_documents {
        return Err(RagError::RepairRequired(
            "indexed document visibility, kind, or category violates collection invariants"
                .to_owned(),
        ));
    }
    Ok(())
}

fn verify_chunk_relations(connection: &Connection) -> Result<(), RagError> {
    let invalid_document_chunks = relational_violation(
        connection,
        "SELECT EXISTS(
                 SELECT 1
                 FROM chunks ch
                 LEFT JOIN documents d ON d.document_id = ch.item_id
                 WHERE ch.item_kind = 'document'
                   AND (
                     d.document_id IS NULL
                     OR d.collection_id != ch.collection_id
                     OR d.visibility != ch.visibility
                     OR ch.claim_kind IS NOT NULL
                     OR ch.assertion_status IS NOT NULL
                     OR ch.scan_metadata_json IS NOT NULL
                   )
                 LIMIT 1
             )",
    )?;
    if invalid_document_chunks {
        return Err(RagError::RepairRequired(
            "indexed document chunk metadata diverges from its document".to_owned(),
        ));
    }
    let invalid_claim_chunks = relational_violation(
        connection,
        "SELECT EXISTS(
                 SELECT 1
                 FROM chunks ch
                 LEFT JOIN claims cl ON cl.claim_id = ch.item_id
                 WHERE ch.item_kind = 'claim'
                   AND (
                     cl.claim_id IS NULL
                     OR cl.collection_id != ch.collection_id
                     OR cl.visibility != ch.visibility
                     OR cl.kind != ch.claim_kind
                     OR cl.status != ch.assertion_status
                     OR cl.scan_metadata_json IS NOT ch.scan_metadata_json
                   )
                 LIMIT 1
             )",
    )?;
    if invalid_claim_chunks {
        return Err(RagError::RepairRequired(
            "indexed claim chunk metadata diverges from its claim".to_owned(),
        ));
    }
    Ok(())
}

fn verify_indexed_scan_metadata(connection: &Connection) -> Result<(), RagError> {
    let mut scan_metadata_statement = connection
        .prepare(
            "SELECT scan_metadata_json FROM claims
             WHERE scan_metadata_json IS NOT NULL ORDER BY claim_id",
        )
        .map_err(sqlite_error)?;
    let scan_metadata_rows = scan_metadata_statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(sqlite_error)?;
    for row in scan_metadata_rows {
        let encoded = row.map_err(sqlite_error)?;
        let metadata: ScanClaimMetadata = serde_json::from_str(&encoded).map_err(|error| {
            RagError::RepairRequired(format!("invalid indexed scan metadata: {error}"))
        })?;
        validate_scan_metadata(&metadata)?;
    }
    Ok(())
}

fn relational_violation(connection: &Connection, sql: &str) -> Result<bool, RagError> {
    let invalid: i64 = connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(sqlite_error)?;
    Ok(invalid != 0)
}

fn deserialize_connection(sqlite_bytes: &[u8]) -> Result<Connection, RagError> {
    let mut connection = Connection::open_in_memory().map_err(sqlite_error)?;
    connection
        .deserialize_read_exact(
            MAIN_DB,
            Cursor::new(sqlite_bytes),
            sqlite_bytes.len(),
            false,
        )
        .map_err(sqlite_error)?;
    Ok(connection)
}

fn resolve_scope(
    registry: &CollectionRegistry,
    request: &RetrievalRequest,
) -> Result<ResolvedScope, RagError> {
    let current_collection_id = request
        .current_collection_id
        .as_deref()
        .map(|reference| resolve_collection(registry, reference))
        .transpose()?;
    match &request.scope {
        RetrievalScope::Auto => {
            let target = current_collection_id.clone().and_then(|collection_id| {
                registry
                    .collections
                    .iter()
                    .find(|collection| collection.collection_id == collection_id)
                    .filter(|collection| collection.state == CollectionState::Attached)
                    .map(|collection| collection.collection_id.clone())
            });
            Ok(ResolvedScope {
                target_collection_id: target,
                current_collection_id,
                explicit_target: false,
            })
        }
        RetrievalScope::Global | RetrievalScope::AllVisible => Ok(ResolvedScope {
            target_collection_id: None,
            current_collection_id,
            explicit_target: false,
        }),
        RetrievalScope::Project(project_id) => {
            let collection_id = resolution_to_id(registry.resolve_project(project_id), "project")?;
            Ok(ResolvedScope {
                target_collection_id: Some(collection_id),
                current_collection_id,
                explicit_target: true,
            })
        }
        RetrievalScope::Collection(reference) => Ok(ResolvedScope {
            target_collection_id: Some(resolve_collection(registry, reference)?),
            current_collection_id,
            explicit_target: true,
        }),
    }
}

fn resolve_collection(registry: &CollectionRegistry, reference: &str) -> Result<String, RagError> {
    resolution_to_id(registry.resolve_collection(reference), "collection")
}

fn resolution_to_id(resolution: CollectionResolution, label: &str) -> Result<String, RagError> {
    match resolution {
        CollectionResolution::Resolved(collection_id) => Ok(collection_id),
        CollectionResolution::Unknown => {
            Err(RagError::InvalidInput(format!("unknown {label} scope")))
        }
        CollectionResolution::Ambiguous(ids) => Err(RagError::Conflict(format!(
            "ambiguous {label} scope resolves to {}",
            ids.join(", ")
        ))),
    }
}

fn is_visible(
    candidate: &Candidate,
    collection: &CollectionRecord,
    scope: &ResolvedScope,
    request: &RetrievalRequest,
) -> bool {
    if !scope.explicit_target && candidate.collection_id == USER_ROOT_COLLECTION_ID {
        return candidate.visibility != RagVisibility::Confidential
            || request.confidential_collection_id.as_deref() == Some(USER_ROOT_COLLECTION_ID);
    }
    if !scope.explicit_target && candidate.visibility == RagVisibility::Shared {
        return true;
    }
    let Some(target) = &scope.target_collection_id else {
        return false;
    };
    if candidate.collection_id != *target {
        return false;
    }
    if collection.state == CollectionState::Detached && !scope.explicit_target {
        return false;
    }
    match candidate.visibility {
        RagVisibility::Shared | RagVisibility::ProjectPrivate => true,
        RagVisibility::Confidential => {
            request.confidential_collection_id.as_deref() == Some(target.as_str())
        }
    }
}

fn build_fts_query(query: &str, expansions: &[String]) -> Result<String, RagError> {
    let mut tokens = Vec::new();
    for value in std::iter::once(query).chain(expansions.iter().map(String::as_str)) {
        for token in search_tokens(value) {
            if !tokens.contains(&token) {
                tokens.push(token);
            }
        }
    }
    if tokens.is_empty() {
        return Err(RagError::InvalidInput(
            "retrieval query must contain searchable Unicode text".to_owned(),
        ));
    }
    Ok(tokens
        .into_iter()
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR "))
}

fn build_wiki_fts_query(query: &str) -> Result<String, RagError> {
    let tokens = search_tokens(query);
    if tokens.is_empty() {
        return Err(RagError::InvalidInput(
            "Wiki page query must contain searchable Unicode text".to_owned(),
        ));
    }
    Ok(tokens
        .into_iter()
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND "))
}

fn search_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() || character == '_' {
            current.push(character);
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn candidate_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Candidate> {
    let claim_kind = row
        .get::<_, Option<String>>(12)?
        .map(|value| ClaimKind::parse(&value))
        .transpose()
        .map_err(to_sqlite_conversion_error)?;
    let assertion_status = row
        .get::<_, Option<String>>(13)?
        .map(|value| AssertionStatus::parse(&value))
        .transpose()
        .map_err(to_sqlite_conversion_error)?;
    let visibility =
        RagVisibility::parse(&row.get::<_, String>(8)?).map_err(to_sqlite_conversion_error)?;
    let language =
        RagLanguage::parse(&row.get::<_, String>(9)?).map_err(to_sqlite_conversion_error)?;
    let scan_metadata = row
        .get::<_, Option<String>>(14)?
        .map(|value| {
            serde_json::from_str::<ScanClaimMetadata>(&value).map_err(|error| {
                RagError::RepairRequired(format!("invalid indexed scan metadata: {error}"))
            })
        })
        .transpose()
        .map_err(to_sqlite_conversion_error)?;
    Ok(Candidate {
        chunk_id: row.get(0)?,
        item_kind: row.get(1)?,
        item_id: row.get(2)?,
        collection_id: row.get(3)?,
        title: row.get(4)?,
        text: row.get(5)?,
        locator: row.get(6)?,
        digest: row.get(7)?,
        visibility,
        language,
        tags: row.get(10)?,
        aliases: row.get(11)?,
        claim_kind,
        assertion_status,
        scan_metadata,
        replacement: row.get(15)?,
        rank: row.get(16)?,
    })
}

fn wiki_page_candidate_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WikiPageCandidate> {
    let visibility =
        RagVisibility::parse(&row.get::<_, String>(8)?).map_err(to_sqlite_conversion_error)?;
    Ok(WikiPageCandidate {
        document_id: row.get(0)?,
        collection_id: row.get(1)?,
        source_project_id: row.get(2)?,
        kind: row.get(3)?,
        category: row.get(4)?,
        summary: row.get(5)?,
        locator: row.get(6)?,
        digest: row.get(7)?,
        visibility,
        rank: row.get(9)?,
    })
}

fn compare_candidates(left: &Candidate, right: &Candidate, folded_query: &str) -> Ordering {
    let left_boost = candidate_boost(left, folded_query);
    let right_boost = candidate_boost(right, folded_query);
    right_boost
        .cmp(&left_boost)
        .then_with(|| {
            left.rank
                .partial_cmp(&right.rank)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| left.chunk_id.cmp(&right.chunk_id))
}

fn candidate_boost(candidate: &Candidate, folded_query: &str) -> u8 {
    if candidate
        .aliases
        .split_whitespace()
        .any(|alias| folded_alias(alias) == folded_query)
    {
        3
    } else if candidate
        .tags
        .split_whitespace()
        .any(|tag| folded_alias(tag) == folded_query)
    {
        2
    } else {
        u8::from(candidate.claim_kind == Some(ClaimKind::Preference))
    }
}

fn candidate_score(candidate: &Candidate, folded_query: &str) -> f64 {
    f64::from(candidate_boost(candidate, folded_query)) * 1000.0 - candidate.rank
}

fn matched_field(candidate: &Candidate, folded_query: &str) -> String {
    if candidate
        .aliases
        .split_whitespace()
        .any(|alias| folded_alias(alias) == folded_query)
    {
        "alias".to_owned()
    } else if candidate
        .tags
        .split_whitespace()
        .any(|tag| folded_alias(tag) == folded_query)
    {
        "tag".to_owned()
    } else if folded_alias(&candidate.title).contains(folded_query) {
        "title".to_owned()
    } else {
        "text".to_owned()
    }
}

fn load_sources(
    connection: &Connection,
    item_kind: &str,
    item_id: &str,
) -> Result<Vec<String>, RagError> {
    let mut statement = connection
        .prepare(
            "SELECT locator FROM sources WHERE item_kind = ?1 AND item_id = ?2 ORDER BY locator",
        )
        .map_err(sqlite_error)?;
    let rows = statement
        .query_map(params![item_kind, item_id], |row| row.get(0))
        .map_err(sqlite_error)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sqlite_error)
}

fn load_item_values(
    connection: &Connection,
    table: &str,
    column: &str,
    item_kind: &str,
    item_id: &str,
) -> Result<Vec<String>, RagError> {
    let sql = format!(
        "SELECT {column} FROM {table} WHERE item_kind = ?1 AND item_id = ?2 ORDER BY {column}"
    );
    let mut statement = connection.prepare(&sql).map_err(sqlite_error)?;
    let rows = statement
        .query_map(params![item_kind, item_id], |row| row.get(0))
        .map_err(sqlite_error)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sqlite_error)
}

fn validate_document(
    document: &CanonicalDocument,
    collection_ids: &BTreeSet<&str>,
) -> Result<(), RagError> {
    validate_stable_id("document_id", &document.document_id, "document-")?;
    if !collection_ids.contains(document.collection_id.as_str()) {
        return Err(RagError::InvalidInput(format!(
            "document `{}` references unknown collection `{}`",
            document.document_id, document.collection_id
        )));
    }
    validate_locator(&document.locator)?;
    validate_bounded_text("document title", &document.title, 1024)?;
    let category = canonical_wiki_category(&document.kind)?;
    if document.category != category {
        return Err(RagError::InvalidInput(format!(
            "document `{}` category does not match kind",
            document.document_id
        )));
    }
    if document.body.trim().is_empty() || document.body.len() > MAX_DOCUMENT_BYTES {
        return Err(RagError::InvalidInput(format!(
            "document `{}` body is empty or exceeds {MAX_DOCUMENT_BYTES} bytes",
            document.document_id
        )));
    }
    if document.revision == 0 {
        return Err(RagError::InvalidInput(format!(
            "document `{}` revision must be greater than zero",
            document.document_id
        )));
    }
    validate_sha256("document digest", &document.digest)?;
    if document.digest != document_digest(document) {
        return Err(RagError::InvalidInput(format!(
            "document `{}` digest does not match canonical content",
            document.document_id
        )));
    }
    validate_values("tag", &document.tags, 256)?;
    validate_values("alias", &document.aliases, 256)?;
    validate_values("link", &document.links, 2048)?;
    validate_values("source", &document.sources, 2048)?;
    if let Some(replacement) = &document.replacement {
        validate_bounded_text("document replacement", replacement, 2048)?;
    }
    Ok(())
}

fn validate_claim(claim: &CanonicalClaim) -> Result<(), RagError> {
    validate_stable_id("claim_id", &claim.claim_id, "claim-")?;
    validate_bounded_text("claim_key", &claim.claim_key, 256)?;
    validate_bounded_text("claim collection_id", &claim.collection_id, 256)?;
    validate_locator(&claim.locator)?;
    if claim.normalized_fact.trim().is_empty()
        || claim.normalized_fact.trim() != claim.normalized_fact
        || claim.normalized_fact.len() > MAX_FACT_BYTES
        || claim.normalized_fact.chars().any(char::is_control)
    {
        return Err(RagError::InvalidInput(
            "normalized_fact must be bounded, trimmed, non-empty single-line text".to_owned(),
        ));
    }
    if claim.revision == 0 {
        return Err(RagError::InvalidInput(
            "claim revision must be greater than zero".to_owned(),
        ));
    }
    validate_provenance(&claim.provenance)?;
    if let Some(metadata) = &claim.scan_metadata {
        validate_scan_metadata(metadata)?;
        let scan_owned = claim.claim_key == format!("scan.{}", metadata.review_id);
        let derived_shared = claim.collection_id == USER_ROOT_COLLECTION_ID
            && claim.claim_key.starts_with("promoted.")
            && claim.provenance.source_kind == RememberSourceKind::ReviewedArtifact;
        if !scan_owned && !derived_shared {
            return Err(RagError::InvalidInput(
                "scan claim key does not match its review identifier".to_owned(),
            ));
        }
        if metadata.review_status == ScanReviewStatus::SourceInvalidated
            && claim.status != AssertionStatus::Superseded
        {
            return Err(RagError::InvalidInput(
                "source-invalidated scan metadata requires a superseded claim".to_owned(),
            ));
        }
    } else if claim.claim_key.starts_with("scan.") {
        return Err(RagError::InvalidInput(
            "scan-owned claims require typed scan metadata".to_owned(),
        ));
    }
    validate_values("claim source", &claim.sources, 2048)?;
    validate_values("superseded claim", &claim.supersedes, 256)?;
    if let Some(replacement) = &claim.replacement {
        validate_stable_id("claim replacement", replacement, "claim-")?;
    }
    if claim.status == AssertionStatus::Superseded
        && claim.replacement.is_none()
        && !is_reviewed_source_invalidation(claim)
    {
        return Err(RagError::InvalidInput(
            "a superseded claim requires a replacement or reviewed source invalidation".to_owned(),
        ));
    }
    validate_optional_timestamp("observed_at", claim.observed_at.as_deref())?;
    validate_optional_timestamp("verified_at", claim.verified_at.as_deref())?;
    validate_sha256("claim digest", &claim.digest)?;
    if claim.digest != claim_digest(claim) {
        return Err(RagError::InvalidInput(format!(
            "claim `{}` digest does not match canonical content",
            claim.claim_id
        )));
    }
    super::reject_likely_credentials(claim.normalized_fact.as_bytes())
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    super::reject_likely_credentials(claim.provenance.summary.as_bytes())
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    Ok(())
}

fn validate_scan_metadata(metadata: &ScanClaimMetadata) -> Result<(), RagError> {
    validate_bounded_text("scan review_id", &metadata.review_id, 96)?;
    if !metadata
        .review_id
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || metadata.review_id.starts_with('-')
        || metadata.review_id.ends_with('-')
    {
        return Err(RagError::InvalidInput(
            "scan review_id must be bounded lowercase ASCII".to_owned(),
        ));
    }
    for (label, value, limit) in [
        ("scan version", metadata.version.as_deref(), 128_usize),
        (
            "scan source_revision",
            metadata.source_revision.as_deref(),
            256_usize,
        ),
        (
            "scan applicability",
            metadata.applicability.as_deref(),
            400_usize,
        ),
    ] {
        if let Some(value) = value {
            validate_bounded_text(label, value, limit)?;
        }
    }
    if metadata.evidence.is_empty() || metadata.evidence.len() > 16 {
        return Err(RagError::InvalidInput(
            "scan metadata requires one through sixteen evidence bindings".to_owned(),
        ));
    }
    let mut evidence_locators = BTreeSet::new();
    for evidence in &metadata.evidence {
        validate_locator(&evidence.locator)?;
        validate_sha256("scan evidence digest", &evidence.content_digest)?;
        if !evidence_locators.insert(evidence.locator.as_str()) {
            return Err(RagError::InvalidInput(
                "scan metadata evidence locators must be unique".to_owned(),
            ));
        }
    }
    match (
        metadata.global_promotion_candidate,
        metadata.promotion_status,
    ) {
        (false, ScanPromotionStatus::NotCandidate)
        | (
            true,
            ScanPromotionStatus::PendingReview
            | ScanPromotionStatus::Promoted
            | ScanPromotionStatus::Rejected,
        ) => Ok(()),
        _ => Err(RagError::InvalidInput(
            "scan promotion status conflicts with candidate selection".to_owned(),
        )),
    }
}

fn is_reviewed_source_invalidation(claim: &CanonicalClaim) -> bool {
    let scan_owned = claim.claim_key.starts_with("scan.")
        && claim
            .provenance
            .summary
            .starts_with("Source-invalidated scan claim `")
        && claim.provenance.locator.strip_prefix("scan-inventory:")
            == Some(claim.provenance.digest.as_str());
    let derived_shared = claim.collection_id == USER_ROOT_COLLECTION_ID
        && claim.claim_key.starts_with("promoted.")
        && claim
            .scan_metadata
            .as_ref()
            .is_some_and(|metadata| metadata.review_status == ScanReviewStatus::SourceInvalidated);
    claim.provenance.source_kind == RememberSourceKind::ReviewedArtifact
        && (scan_owned || derived_shared)
}

fn validate_remember_request(request: &RememberRequest, revision: u64) -> Result<(), RagError> {
    if revision == 0 {
        return Err(RagError::InvalidInput(
            "remember revision must be greater than zero".to_owned(),
        ));
    }
    validate_bounded_text("remember collection_id", &request.collection_id, 256)?;
    validate_bounded_text("claim_key", &request.claim_key, 256)?;
    if let Some(claim_id) = &request.claim_id {
        validate_stable_id("claim_id", claim_id, "claim-")?;
    }
    validate_locator(&request.locator)?;
    if request.status == AssertionStatus::Superseded {
        return Err(RagError::InvalidInput(
            "new remember claims cannot start as superseded".to_owned(),
        ));
    }
    validate_provenance(&request.provenance)?;
    validate_values("remember source", &request.sources, 2048)?;
    validate_values("remember supersedes", &request.supersedes, 256)?;
    if let Some(expected) = &request.expected_active_digest {
        validate_sha256("expected_active_digest", expected)?;
    }
    validate_optional_timestamp("observed_at", request.observed_at.as_deref())?;
    validate_optional_timestamp("verified_at", request.verified_at.as_deref())?;
    super::reject_likely_credentials(request.normalized_fact.as_bytes())
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    super::reject_likely_credentials(request.provenance.summary.as_bytes())
        .map_err(|error| RagError::InvalidInput(error.to_string()))?;
    Ok(())
}

fn validate_provenance(provenance: &ClaimProvenance) -> Result<(), RagError> {
    validate_bounded_text("provenance summary", &provenance.summary, 2048)?;
    if provenance.summary.contains('\n') || provenance.summary.contains('\r') {
        return Err(RagError::InvalidInput(
            "provenance summary must be a bounded summary, not raw multi-line output".to_owned(),
        ));
    }
    validate_bounded_text("provenance locator", &provenance.locator, 2048)?;
    validate_sha256("provenance digest", &provenance.digest)
}

fn validate_retrieval_request(request: &RetrievalRequest) -> Result<(), RagError> {
    validate_bounded_text("query", &request.query, MAX_QUERY_BYTES)?;
    if request.query_expansions.len() > MAX_EXPANSIONS {
        return Err(RagError::InvalidInput(format!(
            "query_expansions exceeds {MAX_EXPANSIONS} entries"
        )));
    }
    validate_values(
        "query expansion",
        &request.query_expansions,
        MAX_QUERY_BYTES,
    )?;
    if request.top_k == 0 || request.top_k > MAX_TOP_K {
        return Err(RagError::InvalidInput(format!(
            "top_k must be between 1 and {MAX_TOP_K}"
        )));
    }
    if request.byte_budget == 0 || request.byte_budget > MAX_BYTE_BUDGET {
        return Err(RagError::InvalidInput(format!(
            "byte_budget must be between 1 and {MAX_BYTE_BUDGET}"
        )));
    }
    if let Some(current) = &request.current_collection_id {
        validate_bounded_text("current_collection_id", current, 256)?;
    }
    if let Some(confidential) = &request.confidential_collection_id {
        validate_bounded_text("confidential_collection_id", confidential, 256)?;
    }
    Ok(())
}

fn validate_locator(locator: &str) -> Result<(), RagError> {
    validate_bounded_text("canonical locator", locator, 2048)?;
    if locator.starts_with('/')
        || locator.starts_with('\\')
        || locator.contains('\\')
        || locator.split('/').any(|component| component == "..")
        || locator.get(1..2) == Some(":")
    {
        return Err(RagError::InvalidInput(
            "canonical locator must be a portable relative logical path".to_owned(),
        ));
    }
    Ok(())
}

fn validate_stable_id(label: &str, value: &str, prefix: &str) -> Result<(), RagError> {
    let Some(digest) = value.strip_prefix(prefix) else {
        return Err(RagError::InvalidInput(format!(
            "{label} must use the `{prefix}` SHA-256 form"
        )));
    };
    validate_raw_sha256(label, digest)
}

fn validate_sha256(label: &str, value: &str) -> Result<(), RagError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(RagError::InvalidInput(format!(
            "{label} must use the `sha256:` digest form"
        )));
    };
    validate_raw_sha256(label, digest)
}

fn validate_raw_sha256(label: &str, digest: &str) -> Result<(), RagError> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(RagError::InvalidInput(format!(
            "{label} must use the `sha256:` digest form"
        )));
    }
    Ok(())
}

fn raw_digest(value: &str) -> &str {
    value
        .strip_prefix("sha256:")
        .expect("hive-core SHA-256 digests use the sha256 prefix")
}

fn validate_values(label: &str, values: &[String], max_bytes: usize) -> Result<(), RagError> {
    for value in values {
        validate_bounded_text(label, value, max_bytes)?;
    }
    Ok(())
}

fn validate_bounded_text(label: &str, value: &str, max_bytes: usize) -> Result<(), RagError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.len() > max_bytes
        || value.chars().any(|character| character == '\0')
    {
        return Err(RagError::InvalidInput(format!(
            "{label} must be bounded, trimmed, non-empty text"
        )));
    }
    Ok(())
}

fn validate_optional_timestamp(label: &str, value: Option<&str>) -> Result<(), RagError> {
    if let Some(value) = value {
        super::validate_timestamp(value)
            .map_err(|error| RagError::InvalidInput(format!("invalid {label}: {error}")))?;
    }
    Ok(())
}

fn sql_i64(label: &str, value: u64) -> Result<i64, RagError> {
    i64::try_from(value)
        .map_err(|_| RagError::InvalidInput(format!("{label} exceeds the SQLite integer range")))
}

fn sql_i64_usize(label: &str, value: usize) -> Result<i64, RagError> {
    i64::try_from(value)
        .map_err(|_| RagError::InvalidInput(format!("{label} exceeds the SQLite integer range")))
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn truncate_utf8(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_owned(), false);
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_owned(), true)
}

const fn collection_kind_str(kind: CollectionKind) -> &'static str {
    match kind {
        CollectionKind::UserRoot => "user-root",
        CollectionKind::RegisteredProject => "registered-project",
        CollectionKind::Directory => "directory",
        CollectionKind::Imported => "imported",
    }
}

const fn collection_state_str(state: CollectionState) -> &'static str {
    match state {
        CollectionState::Attached => "attached",
        CollectionState::Detached => "detached",
    }
}

const fn collection_visibility_str(visibility: CollectionVisibility) -> &'static str {
    match visibility {
        CollectionVisibility::Shared => "shared",
        CollectionVisibility::ProjectPrivate => "project-private",
        CollectionVisibility::Confidential => "confidential",
    }
}

#[allow(clippy::needless_pass_by_value)]
fn sqlite_error(error: rusqlite::Error) -> RagError {
    RagError::Sqlite(error.to_string())
}

fn to_sqlite_conversion_error(error: RagError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::{derive_collection_id, CollectionRegistry, COLLECTION_SCHEMA_VERSION};

    fn digest(value: &str) -> String {
        sha256_digest(value.as_bytes())
    }

    fn raw_test_digest(value: &str) -> String {
        raw_digest(&digest(value)).to_owned()
    }

    fn project_collection(alias: &str, state: CollectionState) -> CollectionRecord {
        project_collection_with_visibility(alias, state, CollectionVisibility::ProjectPrivate)
    }

    fn project_collection_with_visibility(
        alias: &str,
        state: CollectionState,
        visibility: CollectionVisibility,
    ) -> CollectionRecord {
        CollectionRecord {
            collection_id: derive_collection_id("project", alias).expect("collection ID"),
            kind: CollectionKind::RegisteredProject,
            state,
            aliases: vec![alias.to_owned()],
            local_locator: (state == CollectionState::Attached)
                .then(|| std::env::temp_dir().display().to_string()),
            source_project_id: Some(format!("project-{alias}")),
            default_visibility: visibility,
        }
    }

    fn registry() -> CollectionRegistry {
        CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![
                CollectionRecord {
                    collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
                    kind: CollectionKind::UserRoot,
                    state: CollectionState::Attached,
                    aliases: vec!["global".to_owned()],
                    local_locator: Some(std::env::temp_dir().display().to_string()),
                    source_project_id: None,
                    default_visibility: CollectionVisibility::Shared,
                },
                project_collection("alpha", CollectionState::Attached),
                project_collection_with_visibility(
                    "secret",
                    CollectionState::Attached,
                    CollectionVisibility::Confidential,
                ),
                project_collection("detached", CollectionState::Detached),
                project_collection_with_visibility(
                    "remote-shared",
                    CollectionState::Detached,
                    CollectionVisibility::Shared,
                ),
            ],
        }
        .canonicalized()
        .expect("registry")
    }

    fn collection_id(registry: &CollectionRegistry, alias: &str) -> String {
        registry
            .collections
            .iter()
            .find(|collection| collection.aliases.iter().any(|known| known == alias))
            .unwrap_or_else(|| panic!("missing test collection alias `{alias}`"))
            .collection_id
            .clone()
    }

    fn document(
        collection_id: &str,
        seed: &str,
        body: &str,
        visibility: RagVisibility,
    ) -> CanonicalDocument {
        let mut document = CanonicalDocument {
            document_id: format!("document-{}", raw_test_digest(seed)),
            collection_id: collection_id.to_owned(),
            locator: format!("docs/facts/{seed}.md"),
            title: seed.to_owned(),
            kind: "concept".to_owned(),
            category: "concept".to_owned(),
            body: body.to_owned(),
            digest: String::new(),
            visibility,
            language: RagLanguage::Both,
            revision: 1,
            tags: vec![seed.to_owned()],
            aliases: Vec::new(),
            links: Vec::new(),
            sources: vec![format!("source:{seed}")],
            replacement: None,
        };
        document.digest = document_digest(&document);
        document
    }

    fn provenance(seed: &str) -> ClaimProvenance {
        ClaimProvenance {
            source_kind: RememberSourceKind::UserStatement,
            summary: format!("Reviewed statement {seed}"),
            locator: format!("request:{seed}"),
            digest: digest(seed),
        }
    }

    fn remember_request(collection_id: &str, fact: &str) -> RememberRequest {
        RememberRequest {
            collection_id: collection_id.to_owned(),
            claim_key: "editor.preference".to_owned(),
            claim_id: None,
            locator: "docs/facts/editor-preference.md".to_owned(),
            kind: ClaimKind::Preference,
            status: AssertionStatus::UserStated,
            visibility: RagVisibility::ProjectPrivate,
            normalized_fact: fact.to_owned(),
            provenance: provenance(fact),
            sources: vec!["request:editor".to_owned()],
            supersedes: Vec::new(),
            expected_active_digest: None,
            observed_at: None,
            verified_at: None,
        }
    }

    fn claim(
        collection_id: &str,
        seed: &str,
        claim_key: &str,
        fact: &str,
        visibility: RagVisibility,
    ) -> CanonicalClaim {
        let mut claim = CanonicalClaim {
            claim_id: format!("claim-{}", raw_test_digest(seed)),
            claim_key: claim_key.to_owned(),
            collection_id: collection_id.to_owned(),
            document_id: None,
            locator: format!("docs/facts/{seed}.md"),
            kind: ClaimKind::Convention,
            status: AssertionStatus::UserStated,
            visibility,
            normalized_fact: fact.to_owned(),
            provenance: provenance(seed),
            scan_metadata: None,
            revision: 1,
            sources: vec![format!("source:{seed}")],
            supersedes: Vec::new(),
            replacement: None,
            observed_at: None,
            verified_at: None,
            digest: String::new(),
        };
        claim.digest = claim_digest(&claim);
        claim
    }

    #[test]
    fn claim_markdown_round_trips() {
        let registry = registry();
        let collection_id = collection_id(&registry, "alpha");
        let plan = plan_remember(
            &[],
            &remember_request(&collection_id, "Use concise output."),
            1,
        )
        .expect("remember plan");
        let mut claim = plan.new_claim.expect("new claim");
        assert_eq!(
            claim.locator,
            format!(
                ".hive/knowledge/Claims/{}/{}.md",
                collection_id, claim.claim_id
            )
        );
        claim.document_id = Some(format!("document-{}", raw_test_digest("claim-page")));
        claim.digest = claim_digest(&claim);
        let markdown = render_claim_markdown(&claim).expect("render claim");
        let parsed = parse_claim_markdown(&claim.locator, &markdown).expect("parse claim");
        assert_eq!(parsed, claim);
    }

    #[test]
    fn remember_is_idempotent_and_supersede_is_explicit() {
        let registry = registry();
        let collection_id = collection_id(&registry, "alpha");
        let first_plan = plan_remember(
            &[],
            &remember_request(&collection_id, "Use concise output."),
            1,
        )
        .expect("initial remember");
        let first = first_plan.new_claim.expect("claim");
        let noop = plan_remember(
            std::slice::from_ref(&first),
            &remember_request(&collection_id, "Use concise output."),
            2,
        )
        .expect("idempotent remember");
        assert_eq!(noop.disposition, RememberDisposition::Noop);

        let divergent = remember_request(&collection_id, "Use detailed output.");
        assert!(matches!(
            plan_remember(std::slice::from_ref(&first), &divergent, 2),
            Err(RagError::Conflict(_))
        ));

        let mut replacement = divergent;
        replacement.supersedes = vec![first.claim_id.clone()];
        replacement.expected_active_digest = Some(first.digest.clone());
        let replacement_plan = plan_remember(std::slice::from_ref(&first), &replacement, 2)
            .expect("explicit supersede");
        assert_eq!(replacement_plan.disposition, RememberDisposition::Supersede);
        assert_eq!(
            replacement_plan.superseded_claims[0].status,
            AssertionStatus::Superseded
        );
    }

    #[test]
    fn remember_rejects_likely_credentials_and_raw_multiline_provenance() {
        let registry = registry();
        let collection_id = collection_id(&registry, "alpha");
        let mut secret = remember_request(
            &collection_id,
            "Token sk-abcdefghijklmnopqrstuvwxyz0123456789",
        );
        assert!(plan_remember(&[], &secret, 1).is_err());
        secret.normalized_fact = "Safe fact".to_owned();
        secret.provenance.summary = "raw\nmultiline\noutput".to_owned();
        assert!(plan_remember(&[], &secret, 1).is_err());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn retrieval_enforces_scopes_confidentiality_and_detached_state() {
        let registry = registry();
        let alpha = registry
            .collections
            .iter()
            .find(|collection| collection.aliases.contains(&"alpha".to_owned()))
            .expect("alpha")
            .collection_id
            .clone();
        let detached = registry
            .collections
            .iter()
            .find(|collection| collection.aliases.contains(&"detached".to_owned()))
            .expect("detached")
            .collection_id
            .clone();
        let secret = registry
            .collections
            .iter()
            .find(|collection| collection.aliases.contains(&"secret".to_owned()))
            .expect("secret")
            .collection_id
            .clone();
        let remote_shared = registry
            .collections
            .iter()
            .find(|collection| collection.aliases.contains(&"remote-shared".to_owned()))
            .expect("remote shared")
            .collection_id
            .clone();
        let snapshot = RagSnapshot {
            schema_version: RAG_SCHEMA_VERSION,
            generation: 7,
            registry: registry.clone(),
            documents: vec![
                document(
                    USER_ROOT_COLLECTION_ID,
                    "global",
                    "Unicode 하이브 global evidence",
                    RagVisibility::Shared,
                ),
                document(
                    &alpha,
                    "private",
                    "Unicode 하이브 private evidence",
                    RagVisibility::ProjectPrivate,
                ),
                document(
                    &secret,
                    "secret",
                    "Unicode 하이브 confidential evidence",
                    RagVisibility::Confidential,
                ),
                document(
                    &detached,
                    "offline",
                    "Unicode 하이브 detached evidence",
                    RagVisibility::ProjectPrivate,
                ),
                document(
                    &remote_shared,
                    "portable",
                    "Unicode 하이브 detached shared evidence",
                    RagVisibility::Shared,
                ),
            ],
            claims: Vec::new(),
        };
        let artifact = build_rag_index(&snapshot).expect("build index");
        let base = RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "하이브".to_owned(),
            query_expansions: vec!["Unicode".to_owned()],
            top_k: 10,
            byte_budget: 4096,
            confidential_collection_id: None,
        };
        let global =
            retrieve_serialized(&artifact.sqlite_bytes, &artifact.manifest, &registry, &base)
                .expect("global retrieval");
        assert_eq!(global.hits.len(), 2);
        assert!(global
            .hits
            .iter()
            .any(|hit| hit.collection_id == USER_ROOT_COLLECTION_ID));
        assert!(global
            .hits
            .iter()
            .any(|hit| hit.collection_id == remote_shared));

        let mut automatic = base.clone();
        automatic.scope = RetrievalScope::Auto;
        automatic.current_collection_id = Some(alpha.clone());
        let auto = retrieve_serialized(
            &artifact.sqlite_bytes,
            &artifact.manifest,
            &registry,
            &automatic,
        )
        .expect("auto retrieval");
        assert_eq!(auto.hits.len(), 3);
        assert!(auto
            .hits
            .iter()
            .any(|hit| hit.collection_id == remote_shared));

        automatic.scope = RetrievalScope::Project("project-secret".to_owned());
        automatic.current_collection_id = Some(secret.clone());
        automatic.confidential_collection_id = None;
        let named_without_approval = retrieve_serialized(
            &artifact.sqlite_bytes,
            &artifact.manifest,
            &registry,
            &automatic,
        )
        .expect("named retrieval without confidential approval");
        assert!(named_without_approval.hits.is_empty());
        assert!(named_without_approval
            .hits
            .iter()
            .all(|hit| hit.collection_id != remote_shared));

        automatic.confidential_collection_id = Some(secret);
        let authorized = retrieve_serialized(
            &artifact.sqlite_bytes,
            &artifact.manifest,
            &registry,
            &automatic,
        )
        .expect("authorized retrieval");
        assert_eq!(authorized.hits.len(), 1);
        assert!(authorized
            .hits
            .iter()
            .any(|hit| hit.visibility == RagVisibility::Confidential));

        let mut explicit_detached = base;
        explicit_detached.scope = RetrievalScope::Collection("detached".to_owned());
        let detached_result = retrieve_serialized(
            &artifact.sqlite_bytes,
            &artifact.manifest,
            &registry,
            &explicit_detached,
        )
        .expect("explicit detached retrieval");
        assert_eq!(detached_result.hits.len(), 1);
        assert!(detached_result
            .hits
            .iter()
            .any(|hit| hit.collection_id == detached));
        assert!(detached_result
            .hits
            .iter()
            .all(|hit| hit.collection_id != remote_shared));
        assert!(detached_result
            .hits
            .iter()
            .all(|hit| hit.collection_id != USER_ROOT_COLLECTION_ID));
    }

    #[test]
    fn explicit_scope_filters_before_the_candidate_limit() {
        let registry = registry();
        let alpha = collection_id(&registry, "alpha");
        let distractor_collection = collection_id(&registry, "remote-shared");
        let target = claim(
            &alpha,
            "candidate-target",
            "target.fact",
            "The prefilterneedle appears only in the target body.",
            RagVisibility::ProjectPrivate,
        );
        let target_id = target.claim_id.clone();
        let mut claims = Vec::with_capacity(MAX_CANDIDATES + 1);
        claims.push(target);
        for index in 0..MAX_CANDIDATES {
            let seed = format!("candidate-distractor-{index}");
            claims.push(claim(
                &distractor_collection,
                &seed,
                &format!("prefilterneedle.{index}"),
                "Unrelated shared material.",
                RagVisibility::Shared,
            ));
        }
        let artifact = build_rag_index(&RagSnapshot {
            schema_version: RAG_SCHEMA_VERSION,
            generation: 1,
            registry: registry.clone(),
            documents: Vec::new(),
            claims,
        })
        .expect("build candidate-limit fixture");
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection(alpha.clone()),
            current_collection_id: None,
            query: "prefilterneedle".to_owned(),
            query_expansions: Vec::new(),
            top_k: 1,
            byte_budget: 4096,
            confidential_collection_id: None,
        };

        let result = retrieve_serialized(
            &artifact.sqlite_bytes,
            &artifact.manifest,
            &registry,
            &request,
        )
        .expect("explicit retrieval after SQL prefilter");

        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].item_id, target_id);
        assert_eq!(result.hits[0].collection_id, alpha);
    }

    #[test]
    fn retrieval_respects_utf8_byte_budget_and_manifest_freshness() {
        let registry = registry();
        let snapshot = RagSnapshot {
            schema_version: RAG_SCHEMA_VERSION,
            generation: 2,
            registry: registry.clone(),
            documents: vec![document(
                USER_ROOT_COLLECTION_ID,
                "budget",
                "하이브 retrieval evidence",
                RagVisibility::Shared,
            )],
            claims: Vec::new(),
        };
        let artifact = build_rag_index(&snapshot).expect("index");
        let request = RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "하이브".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 5,
            confidential_collection_id: None,
        };
        let result = retrieve_serialized(
            &artifact.sqlite_bytes,
            &artifact.manifest,
            &registry,
            &request,
        )
        .expect("retrieval");
        assert!(result.insufficient_budget);
        assert!(result.hits[0]
            .text
            .is_char_boundary(result.hits[0].text.len()));
        assert!(result.returned_bytes <= 5);
        assert_eq!(result.hits[0].sources, vec!["source:budget"]);

        let mut stale = artifact.manifest.clone();
        stale.generation += 1;
        assert!(matches!(
            retrieve_serialized(&artifact.sqlite_bytes, &stale, &registry, &request),
            Err(RagError::RepairRequired(_))
        ));
    }

    #[test]
    fn full_text_indexes_are_contentless_fixed_table_projections() {
        let registry = registry();
        let snapshot = RagSnapshot {
            schema_version: RAG_SCHEMA_VERSION,
            generation: 1,
            registry,
            documents: vec![document(
                USER_ROOT_COLLECTION_ID,
                "contentless",
                "Compact full text evidence",
                RagVisibility::Shared,
            )],
            claims: Vec::new(),
        };
        let artifact = build_rag_index(&snapshot).expect("contentless FTS index");
        let connection = deserialize_connection(&artifact.sqlite_bytes).expect("deserialize index");
        for (table, shadow_content) in [
            ("documents_fts", "documents_fts_content"),
            ("chunks_fts", "chunks_fts_content"),
        ] {
            let schema: String = connection
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("FTS schema");
            assert!(schema.contains("content = ''"));
            let shadow_count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_schema WHERE name = ?1",
                    [shadow_content],
                    |row| row.get(0),
                )
                .expect("FTS shadow-content inventory");
            assert_eq!(shadow_count, 0);
        }
    }

    #[test]
    fn dirty_recovery_never_treats_sqlite_as_authoritative() {
        let manifest = GenerationManifest {
            schema_version: RAG_SCHEMA_VERSION,
            generation: 3,
            logical_digest: digest("manifest-3"),
            entry_count: 4,
            sqlite_digest: digest("sqlite-3"),
        };
        assert_eq!(
            dirty_recovery_decision(&manifest, None, &manifest),
            DirtyRecoveryDecision::Clean
        );
        let mut canonical = manifest.clone();
        canonical.generation = 4;
        canonical.logical_digest = digest("manifest-4");
        let journal = DirtyJournal {
            base_generation: 3,
            target_generation: 4,
            base_manifest_digest: manifest.logical_digest.clone(),
            entries: vec![DirtyEntry {
                collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
                locator: "docs/facts/change.md".to_owned(),
                target_digest: digest("change"),
            }],
        };
        assert_eq!(
            dirty_recovery_decision(&manifest, Some(&journal), &canonical),
            DirtyRecoveryDecision::RebuildRequired
        );
        let mut conflicting = journal;
        conflicting.base_generation = 2;
        assert_eq!(
            dirty_recovery_decision(&manifest, Some(&conflicting), &canonical),
            DirtyRecoveryDecision::Conflict
        );
    }
}
