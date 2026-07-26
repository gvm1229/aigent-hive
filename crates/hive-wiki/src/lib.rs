//! Canonical Markdown knowledge operations and a disposable `SQLite` projection.

use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
#[cfg(unix)]
use cap_std::fs::PermissionsExt as CapPermissionsExt;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::{ensure_consumer_target, ensure_no_symlink_ancestors, sha256_digest};
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;

const INDEX_RELATIVE: &str = ".hive/index/hive.sqlite3";
const STALE_RELATIVE: &str = ".hive/index/.stale";
const LOCK_RELATIVE: &str = ".hive/index/.knowledge.lock";
const SUPPRESSION_RELATIVE: &str = ".hive/knowledge/suppression.yml";
const WIKI_RELATIVE: &str = ".hive/knowledge/Wiki";
const RAW_RELATIVE: &str = ".hive/knowledge/Raw";
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RAW_BYTES: usize = 5 * 1024 * 1024;
static CAP_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stable failure classes for CLI exit mapping.
#[derive(Debug)]
pub enum WikiError {
    /// A request or canonical document is malformed.
    InvalidInput(String),
    /// A concurrent or immutable-state conflict was detected.
    Conflict(String),
    /// Canonical or derived state failed verification.
    Verification(String),
    /// A local filesystem operation failed.
    Io(String),
    /// `SQLite` projection or query failed.
    Sqlite(String),
}

impl WikiError {
    /// CLI exit class.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::InvalidInput(_) => 2,
            Self::Conflict(_) => 3,
            Self::Verification(_) => 5,
            Self::Io(_) | Self::Sqlite(_) => 10,
        }
    }

    /// Action-result status.
    #[must_use]
    pub const fn status(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) | Self::Io(_) | Self::Sqlite(_) => "error",
            Self::Conflict(_) => "conflict",
            Self::Verification(_) => "verification-failed",
        }
    }

    /// Stable product code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "hive.knowledge-invalid-input",
            Self::Conflict(_) => "hive.knowledge-conflict",
            Self::Verification(_) => "hive.knowledge-verification-failed",
            Self::Io(_) => "hive.knowledge-io-error",
            Self::Sqlite(_) => "hive.index-error",
        }
    }
}

impl Display for WikiError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message)
            | Self::Conflict(message)
            | Self::Verification(message)
            | Self::Io(message)
            | Self::Sqlite(message) => formatter.write_str(message),
        }
    }
}

impl Error for WikiError {}

/// Contradictory claims with both source locators retained.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Contradiction {
    /// First source locator.
    pub source_a: String,
    /// Second source locator.
    pub source_b: String,
    /// Bounded description of the conflict.
    pub summary: String,
}

/// YAML frontmatter contract for an active Wiki page.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WikiFrontmatter {
    /// Contract version.
    pub schema_version: u32,
    /// Stable page identifier and canonical filename stem.
    pub id: String,
    /// Page semantic kind.
    pub kind: String,
    /// Short page summary.
    pub summary: String,
    /// Normalized tags without `#`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Alternate lookup names.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Immutable Raw source locators.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Explicit page IDs referenced by this page.
    #[serde(default)]
    pub links: Vec<String>,
    /// Source-paired contradictions.
    #[serde(default)]
    pub contradictions: Vec<Contradiction>,
    /// Active state; deprecated and superseded are forbidden.
    pub status: String,
    /// RFC 3339 UTC timestamp.
    pub created_at: String,
    /// RFC 3339 UTC timestamp.
    pub updated_at: String,
}

/// Parsed canonical Wiki page.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WikiPage {
    /// Validated frontmatter.
    pub frontmatter: WikiFrontmatter,
    /// Markdown body after the closing frontmatter marker.
    pub body: String,
    /// Consumer-relative canonical path.
    pub relative_path: String,
    /// Digest of canonical page bytes.
    pub content_digest: String,
}

/// Minimal canonical suppression entry. Deleted prose is intentionally impossible to represent.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SuppressionEntry {
    /// SHA-256 fingerprint of the removed or rejected bytes.
    pub fingerprint: String,
    /// Immutable source or page locator.
    pub source_locator: String,
    /// Bounded deletion/re-ingest reason.
    pub reason: String,
    /// Optional active replacement locator.
    pub replacement: Option<String>,
    /// RFC 3339 UTC timestamp.
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct SuppressionLedger {
    schema_version: u32,
    entries: Vec<SuppressionEntry>,
}

/// Successful canonical integration plus rebuilt index evidence.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct KnowledgeOutcome {
    /// Consumer-relative changed canonical and derived paths.
    pub changed_paths: Vec<String>,
    /// Stable page identifier when applicable.
    pub page_id: Option<String>,
    /// Immutable Raw locator when applicable.
    pub source_locator: Option<String>,
    /// Logical index digest after the operation.
    pub logical_digest: String,
}

/// Rebuilt projection evidence.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct IndexOutcome {
    /// Net paths changed by a standalone rebuild.
    pub changed_paths: Vec<String>,
    /// Indexed active Wiki pages.
    pub page_count: usize,
    /// Indexed Raw revisions.
    pub raw_count: usize,
    /// Deterministic digest over logical rows, independent of `SQLite` bytes.
    pub logical_digest: String,
}

/// One deterministic query result.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct QueryHit {
    /// Stable page ID.
    pub id: String,
    /// Page kind.
    pub kind: String,
    /// Summary.
    pub summary: String,
    /// Canonical consumer-relative path.
    pub path: String,
    /// Content digest.
    pub content_digest: String,
    /// Tags.
    pub tags: Vec<String>,
    /// Aliases.
    pub aliases: Vec<String>,
    /// Sorted canonical immutable source locators.
    pub sources: Vec<String>,
}

/// User-root promotion category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PromotionCategory {
    /// Project-neutral reusable fact.
    Fact,
    /// Reusable user preference.
    Preference,
    /// Portable workflow knowledge.
    Workflow,
}

impl PromotionCategory {
    /// Stable CLI and canonical tag value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Preference => "preference",
            Self::Workflow => "workflow",
        }
    }
}

/// Promotion mutation mode.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PromotionMode {
    /// Validate and report the exact plan without canonical mutation.
    DryRun,
    /// Commit canonical root knowledge and rebuild its disposable index.
    Apply,
}

/// User-root promotion plan or applied result.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct PromotionOutcome {
    /// Root-relative canonical and derived paths.
    pub changed_paths: Vec<String>,
    /// Content-derived root Wiki page identifier.
    pub page_id: String,
    /// Pseudonymous source-project provenance.
    pub project_pseudonym: String,
    /// Typed promotion category.
    pub category: PromotionCategory,
    /// Digest binding policy, source page, current root state, and incoming bytes.
    pub plan_digest: String,
    /// Root logical index digest after apply, or current digest for dry-run.
    pub logical_digest: String,
    /// Whether canonical activation occurred.
    pub applied: bool,
}

#[derive(Debug, Deserialize)]
struct PromotionPolicy {
    project_name: String,
    #[serde(default)]
    project_identity: String,
    #[serde(default)]
    knowledge_exclude_paths: Vec<String>,
    #[serde(default)]
    root_knowledge_promotion_categories: Vec<String>,
    #[serde(default)]
    confidential_knowledge_categories: Vec<String>,
    #[serde(default)]
    user_store_binding: String,
}

#[derive(Debug, Serialize)]
struct PromotionRaw<'a> {
    schema_version: u32,
    category: &'a str,
    project_pseudonym: &'a str,
    source_page_digest: &'a str,
    summary: &'a str,
    body: &'a str,
}

struct PromotionPlan {
    outcome: PromotionOutcome,
    raw_path: String,
    raw_bytes: Vec<u8>,
    wiki_path: String,
    wiki_bytes: Vec<u8>,
    snapshots: [CapabilityFileSnapshot; 4],
}

struct PinnedRoot {
    dir: Dir,
    canonical_path: PathBuf,
}

/// Lint severity.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    /// Canonical contract violation.
    Error,
    /// Discoverability or citation warning.
    Warning,
}

/// One deterministic lint diagnostic.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct LintIssue {
    /// Stable lint code.
    pub code: String,
    /// Severity.
    pub severity: LintSeverity,
    /// Canonical locator.
    pub locator: String,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Ingest an immutable Raw revision and serially integrate a prepared Wiki page.
///
/// # Errors
///
/// Returns an error when the target, source, draft, suppression state, canonical
/// write, or rebuilt index violates its contract.
pub fn ingest(
    target: &Path,
    source: &Path,
    wiki_draft: &Path,
) -> Result<KnowledgeOutcome, WikiError> {
    validate_target(target)?;
    let _lock = KnowledgeLock::acquire(target)?;
    let source_bytes = read_raw_source(source)?;
    if source_bytes.is_empty() {
        return Err(WikiError::InvalidInput(
            "Raw source must not be empty".to_owned(),
        ));
    }
    reject_likely_credentials(&source_bytes)?;
    let fingerprint = sha256_digest(&source_bytes);
    let ledger = read_suppression(target)?;
    if ledger
        .entries
        .iter()
        .any(|entry| entry.fingerprint == fingerprint)
    {
        return Err(WikiError::Conflict(format!(
            "source fingerprint is suppressed: {fingerprint}"
        )));
    }

    let draft_bytes = fs::read(wiki_draft).map_err(|error| {
        WikiError::Io(format!(
            "cannot read Wiki draft {}: {error}",
            wiki_draft.display()
        ))
    })?;
    reject_likely_credentials(&draft_bytes)?;
    let mut page = parse_page_bytes(&draft_bytes, "draft")?;
    let raw_path = raw_revision_path(source, &fingerprint)?;
    let raw_locator = format!("raw:{}#{fingerprint}", raw_path.replace('\\', "/"));
    for locator in &mut page.frontmatter.sources {
        if locator == "raw:self" {
            locator.clone_from(&raw_locator);
        }
    }
    for contradiction in &mut page.frontmatter.contradictions {
        if contradiction.source_a == "raw:self" {
            contradiction.source_a.clone_from(&raw_locator);
        }
        if contradiction.source_b == "raw:self" {
            contradiction.source_b.clone_from(&raw_locator);
        }
    }
    validate_frontmatter(&page.frontmatter)?;
    if page.frontmatter.sources.is_empty() {
        return Err(WikiError::InvalidInput(
            "ingested Wiki page must cite at least one Raw source".to_owned(),
        ));
    }
    let wiki_relative = format!("{WIKI_RELATIVE}/{}.md", page.frontmatter.id);
    ensure_safe_relative(target, Path::new(&raw_path))?;
    ensure_safe_relative(target, Path::new(&wiki_relative))?;

    let raw_absolute = target.join(&raw_path);
    let wiki_absolute = target.join(&wiki_relative);
    match fs::read(&wiki_absolute) {
        Ok(existing_bytes) => {
            let existing = parse_page_bytes(&existing_bytes, &wiki_relative)?;
            merge_existing_page(&existing, &mut page)?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(WikiError::Io(format!(
                "cannot read existing Wiki page {}: {error}",
                wiki_absolute.display()
            )));
        }
    }
    validate_frontmatter(&page.frontmatter)?;
    let canonical_bytes = render_page(&page.frontmatter, &page.body)?;
    let (raw_changed, wiki_changed, index) = commit_ingest_mutation(
        target,
        &raw_absolute,
        &source_bytes,
        &wiki_absolute,
        &canonical_bytes,
    )?;
    let mut changed_paths = Vec::new();
    if raw_changed {
        changed_paths.push(raw_path);
    }
    if wiki_changed {
        changed_paths.push(wiki_relative);
    }
    changed_paths.push(INDEX_RELATIVE.to_owned());
    Ok(KnowledgeOutcome {
        changed_paths,
        page_id: Some(page.frontmatter.id),
        source_locator: Some(raw_locator),
        logical_digest: index.logical_digest,
    })
}

/// Promote one project Wiki page into the user-root canonical knowledge store.
///
/// Promotion is explicit, category-gated, project-neutral, secret-scanned, and
/// pseudonymized. Apply holds the root knowledge lock across plan validation,
/// canonical activation, and `SQLite` rebuild.
///
/// # Errors
///
/// Returns an error for disabled/confidential categories, mismatched user-store
/// binding, excluded or sensitive content, contradiction, unsafe paths, or
/// failed canonical/index activation.
pub fn promote(
    project: &Path,
    user_root: &Path,
    page_id: &str,
    category: PromotionCategory,
    mode: PromotionMode,
) -> Result<PromotionOutcome, WikiError> {
    validate_target(project)?;
    validate_target(user_root)?;
    validate_id(page_id)?;
    let project_root = PinnedRoot::open(project)?;
    let user_root = PinnedRoot::open(user_root)?;
    project_root.validate_project_root()?;
    user_root.validate_knowledge_root()?;
    match mode {
        PromotionMode::DryRun => {
            let plan = build_promotion_plan(&project_root, &user_root, page_id, category)?;
            Ok(plan.outcome)
        }
        PromotionMode::Apply => {
            let _lock = CapabilityKnowledgeLock::acquire(&user_root.dir)?;
            let mut plan = build_promotion_plan(&project_root, &user_root, page_id, category)?;
            capability_test_pause("after-plan-before-claim")?;
            let (raw_changed, wiki_changed, index) = commit_promotion_mutation(
                &user_root,
                &mut plan.snapshots,
                &plan.raw_bytes,
                &plan.wiki_bytes,
            )?;
            let mut changed_paths = vec![INDEX_RELATIVE.to_owned()];
            if raw_changed {
                changed_paths.push(plan.raw_path);
            }
            if wiki_changed {
                changed_paths.push(plan.wiki_path);
            }
            changed_paths.sort();
            plan.outcome.changed_paths = changed_paths;
            plan.outcome.logical_digest = index.logical_digest;
            plan.outcome.applied = true;
            Ok(plan.outcome)
        }
    }
}

#[allow(clippy::too_many_lines)]
fn build_promotion_plan(
    project: &PinnedRoot,
    user_root: &PinnedRoot,
    page_id: &str,
    category: PromotionCategory,
) -> Result<PromotionPlan, WikiError> {
    let policy_relative = Path::new(".hive/setup-answers.yml");
    let policy_bytes =
        read_capability_required(&project.dir, policy_relative).map_err(|error| {
            WikiError::Verification(format!(
                "cannot read project promotion policy {}: {error}",
                policy_relative.display()
            ))
        })?;
    let policy: PromotionPolicy = serde_yaml::from_slice(&policy_bytes).map_err(|error| {
        WikiError::Verification(format!("invalid project promotion policy: {error}"))
    })?;
    validate_promotion_policy(&policy, &user_root.canonical_path, category)?;

    let project_page_relative = PathBuf::from(format!("{WIKI_RELATIVE}/{page_id}.md"));
    let project_page_bytes = read_capability_required(&project.dir, &project_page_relative)
        .map_err(|error| {
            WikiError::InvalidInput(format!(
                "cannot read project Wiki page {}: {error}",
                project_page_relative.display()
            ))
        })?;
    reject_promoted_credentials(&project_page_bytes)?;
    let source_page = parse_page_bytes(
        &project_page_bytes,
        &format!("{WIKI_RELATIVE}/{page_id}.md"),
    )?;
    validate_promotable_page(&project.canonical_path, &source_page, &policy)?;

    let identity = if policy.project_identity.is_empty() {
        policy.project_name.as_str()
    } else {
        policy.project_identity.as_str()
    };
    let project_pseudonym = sha256_digest(identity.as_bytes());
    let semantic_digest = sha256_digest(
        format!(
            "{}\0{}\0{}",
            category.as_str(),
            source_page.frontmatter.summary,
            source_page.body.trim()
        )
        .as_bytes(),
    );
    let digest_hex = semantic_digest
        .strip_prefix("sha256:")
        .expect("sha256_digest always returns the product prefix");
    let root_page_id = format!("shared-{}-{}", category.as_str(), &digest_hex[..24]);
    let raw = PromotionRaw {
        schema_version: 1,
        category: category.as_str(),
        project_pseudonym: &project_pseudonym,
        source_page_digest: &source_page.content_digest,
        summary: &source_page.frontmatter.summary,
        body: source_page.body.trim(),
    };
    let mut raw_bytes = serde_json::to_vec(&raw)
        .map_err(|error| WikiError::Io(format!("cannot serialize promotion source: {error}")))?;
    raw_bytes.push(b'\n');
    reject_promoted_credentials(&raw_bytes)?;
    reject_private_locator_text(&raw_bytes)?;
    let raw_fingerprint = sha256_digest(&raw_bytes);
    let raw_path = raw_revision_path(Path::new("promoted-knowledge.json"), &raw_fingerprint)?;
    let raw_locator = format!("raw:{raw_path}#{raw_fingerprint}");

    let mut tags = vec![category.as_str().to_owned(), "promoted".to_owned()];
    tags.sort();
    let mut root_frontmatter = WikiFrontmatter {
        schema_version: 1,
        id: root_page_id.clone(),
        kind: "concept".to_owned(),
        summary: source_page.frontmatter.summary.clone(),
        tags,
        aliases: Vec::new(),
        sources: vec![raw_locator],
        links: Vec::new(),
        contradictions: Vec::new(),
        status: "active".to_owned(),
        created_at: source_page.frontmatter.created_at.clone(),
        updated_at: source_page.frontmatter.updated_at.clone(),
    };
    let wiki_path = format!("{WIKI_RELATIVE}/{root_page_id}.md");

    let root_pages = scan_pages_capability(&user_root.dir)?;
    for existing in root_pages.values() {
        if existing.frontmatter.id == root_page_id {
            if existing.body.trim() != source_page.body.trim()
                || existing.frontmatter.summary != source_page.frontmatter.summary
            {
                return Err(WikiError::Conflict(
                    "content-derived promotion identity collides with different root knowledge"
                        .to_owned(),
                ));
            }
            merge_sorted_unique(&mut root_frontmatter.sources, &existing.frontmatter.sources);
            root_frontmatter
                .created_at
                .clone_from(&existing.frontmatter.created_at);
            root_frontmatter
                .updated_at
                .clone_from(&existing.frontmatter.updated_at);
        } else if existing.frontmatter.summary == source_page.frontmatter.summary
            && existing
                .frontmatter
                .tags
                .contains(&category.as_str().to_owned())
            && existing.body.trim() != source_page.body.trim()
        {
            return Err(WikiError::Conflict(format!(
                "root knowledge contradiction requires explicit review: {}",
                existing.frontmatter.id
            )));
        }
    }
    validate_frontmatter(&root_frontmatter)?;
    let wiki_bytes = render_page(&root_frontmatter, source_page.body.trim())?;
    reject_promoted_credentials(&wiki_bytes)?;
    reject_private_locator_text(&wiki_bytes)?;
    let root_raw = scan_raw_capability(&user_root.dir)?;
    let ledger = read_suppression_capability(&user_root.dir)?;
    let logical_before = logical_digest(&root_pages, &root_raw, &ledger)?;
    let plan_digest = sha256_digest(
        format!(
            "promotion-v1\0{}\0{}\0{}\0{}\0{}\0{}",
            category.as_str(),
            project_pseudonym,
            source_page.content_digest,
            logical_before,
            sha256_digest(&raw_bytes),
            sha256_digest(&wiki_bytes)
        )
        .as_bytes(),
    );
    let mut changed_paths = vec![INDEX_RELATIVE.to_owned()];
    if root_raw.get(&raw_path) != Some(&raw_fingerprint) {
        changed_paths.push(raw_path.clone());
    }
    if root_pages
        .get(&root_page_id)
        .is_none_or(|page| page.content_digest != sha256_digest(&wiki_bytes))
    {
        changed_paths.push(wiki_path.clone());
    }
    changed_paths.sort();
    let snapshots = [
        CapabilityFileSnapshot::capture(&user_root.dir, Path::new(&raw_path))?,
        CapabilityFileSnapshot::capture(&user_root.dir, Path::new(&wiki_path))?,
        CapabilityFileSnapshot::capture(&user_root.dir, Path::new(STALE_RELATIVE))?,
        CapabilityFileSnapshot::capture(&user_root.dir, Path::new(INDEX_RELATIVE))?,
    ];
    Ok(PromotionPlan {
        outcome: PromotionOutcome {
            changed_paths,
            page_id: root_page_id,
            project_pseudonym,
            category,
            plan_digest,
            logical_digest: logical_before,
            applied: false,
        },
        raw_path,
        raw_bytes,
        wiki_path,
        wiki_bytes,
        snapshots,
    })
}

fn validate_promotion_policy(
    policy: &PromotionPolicy,
    user_root: &Path,
    category: PromotionCategory,
) -> Result<(), WikiError> {
    let category = category.as_str();
    if policy
        .confidential_knowledge_categories
        .iter()
        .any(|candidate| candidate == category)
    {
        return Err(WikiError::Conflict(format!(
            "promotion category is confidential: {category}"
        )));
    }
    if !policy
        .root_knowledge_promotion_categories
        .iter()
        .any(|candidate| candidate == category)
    {
        return Err(WikiError::Conflict(format!(
            "promotion category is not approved by project setup: {category}"
        )));
    }
    let canonical_root = user_root.canonicalize().map_err(|error| {
        WikiError::Verification(format!(
            "cannot canonicalize user root {}: {error}",
            user_root.display()
        ))
    })?;
    let expected_binding = sha256_digest(canonical_root.to_string_lossy().as_bytes());
    if policy.user_store_binding != expected_binding {
        return Err(WikiError::Conflict(
            "project user-store binding does not match the selected user root".to_owned(),
        ));
    }
    Ok(())
}

fn validate_promotable_page(
    project: &Path,
    page: &WikiPage,
    policy: &PromotionPolicy,
) -> Result<(), WikiError> {
    let canonical_project = project.canonicalize().map_err(|error| {
        WikiError::Verification(format!(
            "cannot canonicalize project root {}: {error}",
            project.display()
        ))
    })?;
    let project_text = canonical_project.to_string_lossy();
    if page.body.contains(project_text.as_ref())
        || page.frontmatter.summary.contains(project_text.as_ref())
    {
        return Err(WikiError::Conflict(
            "project-private absolute path is not promotable".to_owned(),
        ));
    }
    for source in &page.frontmatter.sources {
        let Some((path, _)) = parse_raw_locator(source) else {
            return Err(WikiError::Verification(
                "project Wiki page has a non-canonical source locator".to_owned(),
            ));
        };
        if policy
            .knowledge_exclude_paths
            .iter()
            .any(|pattern| promotion_path_matches(pattern, path))
            || sensitive_locator(path)
        {
            return Err(WikiError::Conflict(format!(
                "project source is excluded from root promotion: {path}"
            )));
        }
    }
    reject_private_locator_text(page.body.as_bytes())
}

fn promotion_path_matches(pattern: &str, path: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        path == prefix || path.starts_with(&format!("{prefix}/"))
    } else {
        pattern == path
    }
}

fn sensitive_locator(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    [
        ".env",
        "credential",
        "private",
        "secret",
        "token",
        "keychain",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
}

fn reject_private_locator_text(bytes: &[u8]) -> Result<(), WikiError> {
    let text = String::from_utf8_lossy(bytes);
    let lowered = text.to_ascii_lowercase();
    let has_drive_path = text
        .as_bytes()
        .windows(3)
        .any(|window| window[0].is_ascii_alphabetic() && window[1] == b':' && window[2] == b'\\');
    if lowered.contains("/users/")
        || lowered.contains("/home/")
        || lowered.contains("file://")
        || has_drive_path
    {
        return Err(WikiError::Conflict(
            "promotion contains a private filesystem locator".to_owned(),
        ));
    }
    Ok(())
}

fn reject_promoted_credentials(bytes: &[u8]) -> Result<(), WikiError> {
    reject_likely_credentials(bytes).map_err(|error| match error {
        WikiError::InvalidInput(message) => WikiError::Conflict(message),
        other => other,
    })
}

/// Add a minimal suppression entry without storing removed content.
///
/// # Errors
///
/// Returns an error when the entry or target is invalid or the ledger/index
/// cannot be updated and verified.
pub fn suppress(target: &Path, entry: SuppressionEntry) -> Result<KnowledgeOutcome, WikiError> {
    validate_target(target)?;
    validate_suppression_entry(&entry)?;
    let _lock = KnowledgeLock::acquire(target)?;
    let mut ledger = read_suppression(target)?;
    let ledger_changed = !ledger.entries.contains(&entry);
    if !ledger_changed {
        if let Ok(logical_digest) = ensure_index_current(target) {
            return Ok(KnowledgeOutcome {
                changed_paths: Vec::new(),
                page_id: None,
                source_locator: None,
                logical_digest,
            });
        }
    }
    if ledger_changed {
        ledger.entries.push(entry);
        ledger.entries.sort_by(|left, right| {
            (&left.fingerprint, &left.source_locator, &left.timestamp).cmp(&(
                &right.fingerprint,
                &right.source_locator,
                &right.timestamp,
            ))
        });
    }
    let pages = scan_pages(target)?;
    let raw = scan_raw(target)?;
    if let Some(locator) = active_suppression_locator(&pages, &raw, &ledger) {
        return Err(WikiError::Conflict(format!(
            "suppression cannot coexist with active canonical content: {locator}"
        )));
    }
    let snapshots = [
        FileSnapshot::capture(&target.join(SUPPRESSION_RELATIVE))?,
        FileSnapshot::capture(&target.join(STALE_RELATIVE))?,
        FileSnapshot::capture(&target.join(INDEX_RELATIVE))?,
    ];
    let index = transactional(&snapshots, || {
        if ledger_changed {
            let bytes = serde_yaml::to_string(&ledger).map_err(|error| {
                WikiError::Io(format!("cannot serialize suppression ledger: {error}"))
            })?;
            write_atomic(&target.join(SUPPRESSION_RELATIVE), bytes.as_bytes())?;
        }
        mark_stale(target)?;
        rebuild_index_locked(target)
    })?;
    let mut changed_paths = vec![INDEX_RELATIVE.to_owned()];
    if ledger_changed {
        changed_paths.push(SUPPRESSION_RELATIVE.to_owned());
    }
    Ok(KnowledgeOutcome {
        changed_paths,
        page_id: None,
        source_locator: None,
        logical_digest: index.logical_digest,
    })
}

/// Delete an active Wiki page and retain only minimal suppression metadata.
///
/// # Errors
///
/// Returns an error when the page is missing, has active backlinks, or the
/// canonical deletion, suppression update, or index rebuild fails.
#[allow(clippy::too_many_lines)]
pub fn delete_page(
    target: &Path,
    page_id: &str,
    reason: &str,
    replacement: Option<&str>,
    timestamp: &str,
) -> Result<KnowledgeOutcome, WikiError> {
    validate_target(target)?;
    validate_id(page_id)?;
    validate_suppression_reason(reason)?;
    validate_timestamp(timestamp)?;
    if let Some(value) = replacement {
        validate_suppression_locator("replacement", value)?;
    }
    let _lock = KnowledgeLock::acquire(target)?;
    let pages = scan_pages(target)?;
    let page = pages
        .get(page_id)
        .ok_or_else(|| WikiError::Conflict(format!("Wiki page does not exist: {page_id}")))?;
    let incoming: Vec<_> = pages
        .values()
        .filter(|candidate| {
            effective_links(candidate)
                .iter()
                .any(|link| link == page_id)
        })
        .map(|candidate| candidate.frontmatter.id.clone())
        .collect();
    if !incoming.is_empty() {
        return Err(WikiError::Conflict(format!(
            "Wiki page has active backlinks from: {}",
            incoming.join(", ")
        )));
    }
    let absolute = target.join(&page.relative_path);
    let mut ledger = read_suppression(target)?;
    let wiki_entry = SuppressionEntry {
        fingerprint: page.content_digest.clone(),
        source_locator: format!("wiki:{page_id}"),
        reason: reason.to_owned(),
        replacement: replacement.map(ToOwned::to_owned),
        timestamp: timestamp.to_owned(),
    };
    if !ledger.entries.contains(&wiki_entry) {
        ledger.entries.push(wiki_entry);
    }
    let mut removed_raw = Vec::new();
    for source in &page.frontmatter.sources {
        let still_cited = pages.values().any(|candidate| {
            candidate.frontmatter.id != page_id && candidate.frontmatter.sources.contains(source)
        });
        if still_cited {
            continue;
        }
        let Some((path, fingerprint)) = parse_raw_locator(source) else {
            continue;
        };
        ensure_safe_relative(target, Path::new(path))?;
        let raw_absolute = target.join(path);
        if raw_absolute.is_file() {
            removed_raw.push(path.to_owned());
        }
        let raw_entry = SuppressionEntry {
            fingerprint: fingerprint.to_owned(),
            source_locator: source.clone(),
            reason: reason.to_owned(),
            replacement: replacement.map(ToOwned::to_owned),
            timestamp: timestamp.to_owned(),
        };
        if !ledger.entries.contains(&raw_entry) {
            ledger.entries.push(raw_entry);
        }
    }
    ledger.entries.sort_by(|left, right| {
        (&left.fingerprint, &left.source_locator, &left.timestamp).cmp(&(
            &right.fingerprint,
            &right.source_locator,
            &right.timestamp,
        ))
    });
    let ledger_bytes = serde_yaml::to_string(&ledger)
        .map_err(|error| WikiError::Io(format!("cannot serialize suppression ledger: {error}")))?;
    let mut snapshots = vec![
        FileSnapshot::capture(&absolute)?,
        FileSnapshot::capture(&target.join(SUPPRESSION_RELATIVE))?,
        FileSnapshot::capture(&target.join(STALE_RELATIVE))?,
        FileSnapshot::capture(&target.join(INDEX_RELATIVE))?,
    ];
    for path in &removed_raw {
        snapshots.push(FileSnapshot::capture(&target.join(path))?);
    }
    let index = transactional(&snapshots, || {
        fs::remove_file(&absolute).map_err(|error| {
            WikiError::Io(format!("cannot delete {}: {error}", absolute.display()))
        })?;
        for path in &removed_raw {
            fs::remove_file(target.join(path)).map_err(|error| {
                WikiError::Io(format!(
                    "cannot delete obsolete Raw revision {path}: {error}"
                ))
            })?;
        }
        write_atomic(&target.join(SUPPRESSION_RELATIVE), ledger_bytes.as_bytes())?;
        mark_stale(target)?;
        rebuild_index_locked(target)
    })?;
    let mut changed_paths = vec![
        page.relative_path.clone(),
        SUPPRESSION_RELATIVE.to_owned(),
        INDEX_RELATIVE.to_owned(),
    ];
    changed_paths.extend(removed_raw);
    changed_paths.sort();
    changed_paths.dedup();
    Ok(KnowledgeOutcome {
        changed_paths,
        page_id: Some(page_id.to_owned()),
        source_locator: None,
        logical_digest: index.logical_digest,
    })
}

/// Rebuild the complete `SQLite` projection from canonical tracked sources.
///
/// # Errors
///
/// Returns an error when canonical sources are invalid or the temporary index
/// cannot be created, verified, and activated.
pub fn rebuild_index(target: &Path) -> Result<IndexOutcome, WikiError> {
    validate_target(target)?;
    let _lock = KnowledgeLock::acquire(target)?;
    rebuild_index_locked(target)
}

/// Query the current, non-stale FTS/tag/alias projection.
///
/// # Errors
///
/// Returns an error for invalid query options, stale/missing derived state, or
/// a read-only `SQLite` query failure.
pub fn query(
    target: &Path,
    text: Option<&str>,
    tag: Option<&str>,
    limit: usize,
) -> Result<Vec<QueryHit>, WikiError> {
    validate_target(target)?;
    if text.is_none() && tag.is_none() {
        return Err(WikiError::InvalidInput(
            "query requires --text or --tag".to_owned(),
        ));
    }
    if !(1..=100).contains(&limit) {
        return Err(WikiError::InvalidInput(
            "query limit must be from 1 through 100".to_owned(),
        ));
    }
    ensure_index_current(target)?;
    let index_path = target.join(INDEX_RELATIVE);
    let connection = Connection::open_with_flags(
        nofollow_sqlite_path(&index_path)?,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(sqlite_error)?;
    let rows = if let Some(search) = text {
        let expression = fts_expression(search)?;
        let mut statement = connection
            .prepare(
                "SELECT p.id, p.kind, p.summary, p.path, p.content_hash
                 FROM pages_fts f JOIN pages p ON p.id = f.id
                 WHERE pages_fts MATCH ?1
                   AND (?2 IS NULL OR EXISTS (
                     SELECT 1 FROM tags t WHERE t.page_id = p.id AND t.tag = ?2
                   ))
                 ORDER BY bm25(pages_fts), p.id
                 LIMIT ?3",
            )
            .map_err(sqlite_error)?;
        let sql_limit = i64::try_from(limit)
            .map_err(|_| WikiError::InvalidInput("query limit is too large".to_owned()))?;
        collect_hits(
            &connection,
            statement.query(params![expression, tag, sql_limit]),
            limit,
        )?
    } else {
        let mut statement = connection
            .prepare(
                "SELECT p.id, p.kind, p.summary, p.path, p.content_hash
                 FROM pages p JOIN tags t ON t.page_id = p.id
                 WHERE t.tag = ?1
                 ORDER BY p.id
                 LIMIT ?2",
            )
            .map_err(sqlite_error)?;
        let sql_limit = i64::try_from(limit)
            .map_err(|_| WikiError::InvalidInput("query limit is too large".to_owned()))?;
        collect_hits(&connection, statement.query(params![tag, sql_limit]), limit)?
    };
    Ok(rows)
}

/// Lint canonical pages, citations, contradictions, graph integrity, and index freshness.
///
/// # Errors
///
/// Returns an error only when the target cannot be inspected safely. Canonical
/// lint findings are returned as deterministic diagnostics.
#[allow(clippy::too_many_lines)]
pub fn lint(target: &Path) -> Result<Vec<LintIssue>, WikiError> {
    validate_target(target)?;
    let mut issues = Vec::new();
    let pages = match scan_pages(target) {
        Ok(pages) => pages,
        Err(error) => {
            issues.push(LintIssue {
                code: "invalid-page".to_owned(),
                severity: LintSeverity::Error,
                locator: WIKI_RELATIVE.to_owned(),
                message: error.to_string(),
            });
            return Ok(issues);
        }
    };
    let raw = scan_raw(target)?;
    let mut inbound: BTreeMap<String, usize> = pages.keys().map(|id| (id.clone(), 0)).collect();
    let mut names = BTreeMap::<String, String>::new();
    for page in pages.values() {
        if page.frontmatter.sources.is_empty() {
            issues.push(issue(
                "missing-citation",
                LintSeverity::Error,
                &page.relative_path,
                "active Wiki page has no Raw source locator",
            ));
        }
        for source in &page.frontmatter.sources {
            let citation_valid = parse_raw_locator(source).is_some_and(|(path, fingerprint)| {
                raw.get(path).is_some_and(|digest| digest == fingerprint)
            });
            if !citation_valid {
                issues.push(issue(
                    "missing-citation",
                    LintSeverity::Error,
                    &page.relative_path,
                    &format!("Raw source revision is missing or has a digest mismatch: {source}"),
                ));
            }
        }
        if page.frontmatter.status == "contradicted" && page.frontmatter.contradictions.is_empty() {
            issues.push(issue(
                "missing-contradiction",
                LintSeverity::Error,
                &page.relative_path,
                "contradicted page must record both source locators",
            ));
        }
        for contradiction in &page.frontmatter.contradictions {
            if contradiction.source_a == contradiction.source_b
                || !page.frontmatter.sources.contains(&contradiction.source_a)
                || !page.frontmatter.sources.contains(&contradiction.source_b)
            {
                issues.push(issue(
                    "invalid-contradiction",
                    LintSeverity::Error,
                    &page.relative_path,
                    "contradiction must reference two distinct cited sources",
                ));
            }
        }
        for link in effective_links(page) {
            if let Some(count) = inbound.get_mut(&link) {
                *count += 1;
            } else {
                issues.push(issue(
                    "broken-link",
                    LintSeverity::Error,
                    &page.relative_path,
                    &format!("linked Wiki page does not exist: {link}"),
                ));
            }
        }
        for name in std::iter::once(&page.frontmatter.id).chain(&page.frontmatter.aliases) {
            let folded = name.to_lowercase();
            if let Some(owner) = names.insert(folded, page.frontmatter.id.clone()) {
                if owner != page.frontmatter.id {
                    issues.push(issue(
                        "alias-collision",
                        LintSeverity::Error,
                        &page.relative_path,
                        &format!("alias collides with page {owner}: {name}"),
                    ));
                }
            }
        }
    }
    for page in pages.values() {
        let incoming = inbound
            .get(&page.frontmatter.id)
            .copied()
            .unwrap_or_default();
        if incoming == 0 && effective_links(page).is_empty() {
            issues.push(issue(
                "orphan",
                LintSeverity::Warning,
                &page.relative_path,
                "page has no incoming or outgoing Wiki link",
            ));
        }
    }
    if let Err(error) = ensure_index_current(target) {
        issues.push(issue(
            "stale-index",
            LintSeverity::Error,
            INDEX_RELATIVE,
            &error.to_string(),
        ));
    }
    issues.sort_by(|left, right| {
        (&left.code, &left.locator, &left.message).cmp(&(
            &right.code,
            &right.locator,
            &right.message,
        ))
    });
    Ok(issues)
}

/// Parse and validate one Wiki page without filesystem effects.
///
/// # Errors
///
/// Returns an error when UTF-8, frontmatter, fields, ordering, timestamps, or
/// the Markdown body violate the Wiki page contract.
pub fn parse_page_bytes(bytes: &[u8], locator: &str) -> Result<WikiPage, WikiError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| WikiError::InvalidInput(format!("{locator} is not UTF-8: {error}")))?;
    let normalized = text.replace("\r\n", "\n");
    let rest = normalized
        .strip_prefix("---\n")
        .ok_or_else(|| WikiError::InvalidInput(format!("{locator} is missing YAML frontmatter")))?;
    let marker = rest.find("\n---\n").ok_or_else(|| {
        WikiError::InvalidInput(format!("{locator} has no closing frontmatter marker"))
    })?;
    let frontmatter_text = &rest[..marker];
    let body = &rest[marker + 5..];
    let frontmatter: WikiFrontmatter = serde_yaml::from_str(frontmatter_text).map_err(|error| {
        WikiError::InvalidInput(format!("invalid Wiki frontmatter at {locator}: {error}"))
    })?;
    validate_frontmatter(&frontmatter)?;
    if body.trim().is_empty() {
        return Err(WikiError::InvalidInput(format!(
            "Wiki body must not be empty at {locator}"
        )));
    }
    Ok(WikiPage {
        frontmatter,
        body: body.to_owned(),
        relative_path: locator.to_owned(),
        content_digest: sha256_digest(bytes),
    })
}

#[allow(clippy::too_many_lines)]
fn rebuild_index_locked(target: &Path) -> Result<IndexOutcome, WikiError> {
    let pages = scan_pages(target)?;
    let raw = scan_raw(target)?;
    let ledger = read_suppression(target)?;
    if let Some(locator) = active_suppression_locator(&pages, &raw, &ledger) {
        return Err(WikiError::Verification(format!(
            "suppression ledger overlaps active canonical content: {locator}"
        )));
    }
    let logical_digest = logical_digest(&pages, &raw, &ledger)?;
    let index_directory = target.join(".hive/index");
    fs::create_dir_all(&index_directory)
        .map_err(|error| WikiError::Io(format!("cannot create index directory: {error}")))?;
    ensure_safe_relative(target, Path::new(INDEX_RELATIVE))?;
    let stale = target.join(STALE_RELATIVE);
    let stale_exists = regular_file_exists(&stale)?;
    reject_non_regular_existing_index(&target.join(INDEX_RELATIVE))?;
    let temporary = NamedTempFile::new_in(&index_directory)
        .map_err(|error| WikiError::Io(format!("cannot create temporary index: {error}")))?;
    let temporary_path = temporary.path().to_path_buf();
    let mut connection = Connection::open_with_flags(
        nofollow_sqlite_path(&temporary_path)?,
        OpenFlags::default() | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(sqlite_error)?;
    connection
        .execute_batch(
            "PRAGMA journal_mode=DELETE;
             PRAGMA synchronous=FULL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE pages(
               id TEXT PRIMARY KEY, kind TEXT NOT NULL, summary TEXT NOT NULL,
               path TEXT NOT NULL, status TEXT NOT NULL, content_hash TEXT NOT NULL, body TEXT NOT NULL
             );
             CREATE VIRTUAL TABLE pages_fts USING fts5(id UNINDEXED, summary, body, aliases, tags);
             CREATE TABLE tags(page_id TEXT NOT NULL, tag TEXT NOT NULL, PRIMARY KEY(page_id, tag));
             CREATE TABLE aliases(page_id TEXT NOT NULL, alias TEXT NOT NULL, PRIMARY KEY(page_id, alias));
             CREATE TABLE links(source_id TEXT NOT NULL, target_id TEXT NOT NULL, PRIMARY KEY(source_id, target_id));
             CREATE TABLE sources(page_id TEXT NOT NULL, locator TEXT NOT NULL, PRIMARY KEY(page_id, locator));
             CREATE TABLE contradictions(
               page_id TEXT NOT NULL, source_a TEXT NOT NULL, source_b TEXT NOT NULL, summary TEXT NOT NULL,
               PRIMARY KEY(page_id, source_a, source_b, summary)
             );
             CREATE TABLE raw_objects(path TEXT PRIMARY KEY, content_hash TEXT NOT NULL);",
        )
        .map_err(sqlite_error)?;
    let transaction = connection.transaction().map_err(sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO meta(key, value) VALUES
             ('schema_version', '1'), ('logical_digest', ?1), ('page_count', ?2), ('raw_count', ?3)",
            params![logical_digest, pages.len().to_string(), raw.len().to_string()],
        )
        .map_err(sqlite_error)?;
    for page in pages.values() {
        let meta = &page.frontmatter;
        transaction
            .execute(
                "INSERT INTO pages VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    meta.id,
                    meta.kind,
                    meta.summary,
                    page.relative_path,
                    meta.status,
                    page.content_digest,
                    page.body
                ],
            )
            .map_err(sqlite_error)?;
        transaction
            .execute(
                "INSERT INTO pages_fts VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    meta.id,
                    meta.summary,
                    page.body,
                    meta.aliases.join(" "),
                    meta.tags.join(" ")
                ],
            )
            .map_err(sqlite_error)?;
        for tag in &meta.tags {
            transaction
                .execute("INSERT INTO tags VALUES (?1, ?2)", params![meta.id, tag])
                .map_err(sqlite_error)?;
        }
        for alias in &meta.aliases {
            transaction
                .execute(
                    "INSERT INTO aliases VALUES (?1, ?2)",
                    params![meta.id, alias],
                )
                .map_err(sqlite_error)?;
        }
        for link in effective_links(page) {
            transaction
                .execute("INSERT INTO links VALUES (?1, ?2)", params![meta.id, link])
                .map_err(sqlite_error)?;
        }
        for source in &meta.sources {
            transaction
                .execute(
                    "INSERT INTO sources VALUES (?1, ?2)",
                    params![meta.id, source],
                )
                .map_err(sqlite_error)?;
        }
        for contradiction in &meta.contradictions {
            transaction
                .execute(
                    "INSERT INTO contradictions VALUES (?1, ?2, ?3, ?4)",
                    params![
                        meta.id,
                        contradiction.source_a,
                        contradiction.source_b,
                        contradiction.summary
                    ],
                )
                .map_err(sqlite_error)?;
        }
    }
    for (path, digest) in &raw {
        transaction
            .execute(
                "INSERT INTO raw_objects VALUES (?1, ?2)",
                params![path, digest],
            )
            .map_err(sqlite_error)?;
    }
    transaction.commit().map_err(sqlite_error)?;
    connection
        .execute_batch("PRAGMA optimize;")
        .map_err(sqlite_error)?;
    drop(connection);

    let verification = Connection::open_with_flags(
        nofollow_sqlite_path(&temporary_path)?,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(sqlite_error)?;
    let stored: String = verification
        .query_row(
            "SELECT value FROM meta WHERE key = 'logical_digest'",
            [],
            |row| row.get(0),
        )
        .map_err(sqlite_error)?;
    let count: i64 = verification
        .query_row("SELECT count(*) FROM pages", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let fts_count: i64 = verification
        .query_row("SELECT count(*) FROM pages_fts", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let integrity: String = verification
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if stored != logical_digest
        || usize::try_from(count).ok() != Some(pages.len())
        || usize::try_from(fts_count).ok() != Some(pages.len())
        || integrity != "ok"
    {
        return Err(WikiError::Verification(
            "temporary index logical verification failed".to_owned(),
        ));
    }
    drop(verification);
    temporary
        .persist(target.join(INDEX_RELATIVE))
        .map_err(|error| {
            WikiError::Io(format!(
                "cannot atomically replace rebuilt index: {}",
                error.error
            ))
        })?;
    if stale_exists {
        fs::remove_file(&stale)
            .map_err(|error| WikiError::Io(format!("cannot clear stale marker: {error}")))?;
    }
    let mut changed_paths = vec![INDEX_RELATIVE.to_owned()];
    if stale_exists {
        changed_paths.push(STALE_RELATIVE.to_owned());
    }
    Ok(IndexOutcome {
        changed_paths,
        page_count: pages.len(),
        raw_count: raw.len(),
        logical_digest,
    })
}

fn ensure_index_current(target: &Path) -> Result<String, WikiError> {
    let stale = target.join(STALE_RELATIVE);
    match fs::symlink_metadata(&stale) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(WikiError::Verification(
                "SQLite stale marker is not a regular project-local file".to_owned(),
            ));
        }
        Ok(_) => {
            return Err(WikiError::Verification(
                "SQLite index is marked stale; run `hive index rebuild`".to_owned(),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(WikiError::Io(format!(
                "cannot inspect SQLite stale marker: {error}"
            )));
        }
    }
    let index = target.join(INDEX_RELATIVE);
    if !regular_file_exists(&index)? {
        return Err(WikiError::Verification(
            "SQLite index is missing; run `hive index rebuild`".to_owned(),
        ));
    }
    let pages = scan_pages(target)?;
    let raw = scan_raw(target)?;
    let ledger = read_suppression(target)?;
    if let Some(locator) = active_suppression_locator(&pages, &raw, &ledger) {
        return Err(WikiError::Verification(format!(
            "suppression ledger overlaps active canonical content: {locator}"
        )));
    }
    let expected = logical_digest(&pages, &raw, &ledger)?;
    let connection = Connection::open_with_flags(
        nofollow_sqlite_path(&index)?,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(sqlite_error)?;
    let actual: String = connection
        .query_row(
            "SELECT value FROM meta WHERE key = 'logical_digest'",
            [],
            |row| row.get(0),
        )
        .map_err(sqlite_error)?;
    if actual != expected {
        return Err(WikiError::Verification(format!(
            "SQLite index digest is stale: expected {expected}, found {actual}"
        )));
    }
    Ok(expected)
}

fn regular_file_exists(path: &Path) -> Result<bool, WikiError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(WikiError::Verification(format!(
                "managed SQLite path is not a regular project-local file: {}",
                path.display()
            )))
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(WikiError::Io(format!(
            "cannot inspect managed SQLite path {}: {error}",
            path.display()
        ))),
    }
}

fn reject_non_regular_existing_index(path: &Path) -> Result<(), WikiError> {
    regular_file_exists(path).map(|_| ())
}

fn nofollow_sqlite_path(path: &Path) -> Result<PathBuf, WikiError> {
    let parent = path
        .parent()
        .ok_or_else(|| WikiError::Io("SQLite path has no parent".to_owned()))?;
    let filename = path
        .file_name()
        .ok_or_else(|| WikiError::Io("SQLite path has no filename".to_owned()))?;
    let canonical_parent = parent.canonicalize().map_err(|error| {
        WikiError::Io(format!(
            "cannot canonicalize SQLite parent {}: {error}",
            parent.display()
        ))
    })?;
    Ok(canonical_parent.join(filename))
}

fn scan_pages(target: &Path) -> Result<BTreeMap<String, WikiPage>, WikiError> {
    let wiki = target.join(WIKI_RELATIVE);
    let mut pages = BTreeMap::new();
    if !wiki.is_dir() {
        return Err(WikiError::Verification(format!(
            "canonical Wiki directory is missing: {WIKI_RELATIVE}"
        )));
    }
    let mut entries = fs::read_dir(&wiki)
        .map_err(|error| WikiError::Io(format!("cannot scan Wiki directory: {error}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| WikiError::Io(format!("cannot scan Wiki directory: {error}")))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            WikiError::Io(format!("cannot inspect {}: {error}", path.display()))
        })?;
        if file_type.is_symlink() {
            return Err(WikiError::Verification(format!(
                "Wiki symlink is forbidden: {}",
                path.display()
            )));
        }
        if !file_type.is_file() || path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if matches!(name, "index.md" | "log.md") {
            continue;
        }
        let relative = path
            .strip_prefix(target)
            .map_err(|error| WikiError::Io(error.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path)
            .map_err(|error| WikiError::Io(format!("cannot read {}: {error}", path.display())))?;
        reject_likely_credentials(&bytes).map_err(|error| {
            WikiError::Verification(format!(
                "canonical Wiki page contains likely sensitive material at {relative}: {error}"
            ))
        })?;
        let mut page = parse_page_bytes(&bytes, &relative)?;
        if page
            .frontmatter
            .sources
            .iter()
            .any(|source| source == "raw:self")
        {
            return Err(WikiError::Verification(format!(
                "raw:self is allowed only in prepared drafts: {relative}"
            )));
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if stem != page.frontmatter.id {
            return Err(WikiError::Verification(format!(
                "Wiki filename must match page id: {relative}"
            )));
        }
        page.relative_path = relative;
        page.content_digest = sha256_digest(&bytes);
        if pages.insert(page.frontmatter.id.clone(), page).is_some() {
            return Err(WikiError::Verification("duplicate Wiki page id".to_owned()));
        }
    }
    Ok(pages)
}

fn scan_raw(target: &Path) -> Result<BTreeMap<String, String>, WikiError> {
    let root = target.join(RAW_RELATIVE);
    let mut output = BTreeMap::new();
    if !root.is_dir() {
        return Err(WikiError::Verification(format!(
            "canonical Raw directory is missing: {RAW_RELATIVE}"
        )));
    }
    scan_raw_directory(target, &root, &mut output)?;
    Ok(output)
}

fn scan_raw_directory(
    target: &Path,
    directory: &Path,
    output: &mut BTreeMap<String, String>,
) -> Result<(), WikiError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| WikiError::Io(format!("cannot scan {}: {error}", directory.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| WikiError::Io(format!("cannot scan {}: {error}", directory.display())))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            WikiError::Io(format!("cannot inspect {}: {error}", path.display()))
        })?;
        if file_type.is_symlink() {
            return Err(WikiError::Verification(format!(
                "Raw symlink is forbidden: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            scan_raw_directory(target, &path, output)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(target)
                .map_err(|error| WikiError::Io(error.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            if relative == ".hive/knowledge/Raw/README.md" {
                continue;
            }
            let bytes = read_raw_source(&path).map_err(|error| {
                WikiError::Verification(format!(
                    "canonical Raw revision violates the bounded source contract at {relative}: {error}"
                ))
            })?;
            if bytes.is_empty() {
                return Err(WikiError::Verification(format!(
                    "canonical Raw revision must not be empty: {relative}"
                )));
            }
            let content_digest = sha256_digest(&bytes);
            let expected_digest = path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| format!("sha256:{value}"))
                .ok_or_else(|| {
                    WikiError::Verification(format!(
                        "Raw revision filename is not UTF-8: {relative}"
                    ))
                })?;
            if expected_digest != content_digest {
                return Err(WikiError::Verification(format!(
                    "Raw revision content digest does not match its immutable path: {relative}"
                )));
            }
            reject_likely_credentials(&bytes).map_err(|error| {
                WikiError::Verification(format!(
                    "canonical Raw revision contains likely sensitive material at {relative}: {error}"
                ))
            })?;
            output.insert(relative, content_digest);
        }
    }
    Ok(())
}

fn active_suppression_locator(
    pages: &BTreeMap<String, WikiPage>,
    raw: &BTreeMap<String, String>,
    ledger: &SuppressionLedger,
) -> Option<String> {
    for entry in &ledger.entries {
        let active_page = pages.values().any(|page| {
            page.content_digest == entry.fingerprint
                || entry.source_locator.strip_prefix("wiki:") == Some(page.frontmatter.id.as_str())
        });
        let active_raw = raw.values().any(|digest| digest == &entry.fingerprint)
            || parse_raw_locator(&entry.source_locator).is_some_and(|(path, fingerprint)| {
                raw.get(path).is_some_and(|digest| digest == fingerprint)
            });
        if active_page || active_raw {
            return Some(entry.source_locator.clone());
        }
    }
    None
}

fn logical_digest(
    pages: &BTreeMap<String, WikiPage>,
    raw: &BTreeMap<String, String>,
    ledger: &SuppressionLedger,
) -> Result<String, WikiError> {
    #[derive(Serialize)]
    struct LogicalPage<'a> {
        id: &'a str,
        kind: &'a str,
        path: &'a str,
        status: &'a str,
        content_digest: &'a str,
        tags: &'a [String],
        aliases: &'a [String],
        sources: &'a [String],
        links: Vec<String>,
        contradictions: &'a [Contradiction],
    }
    #[derive(Serialize)]
    struct LogicalIndex<'a> {
        schema_version: u32,
        pages: Vec<LogicalPage<'a>>,
        raw: &'a BTreeMap<String, String>,
        suppression: &'a [SuppressionEntry],
    }
    let logical = LogicalIndex {
        schema_version: 1,
        pages: pages
            .values()
            .map(|page| LogicalPage {
                id: &page.frontmatter.id,
                kind: &page.frontmatter.kind,
                path: &page.relative_path,
                status: &page.frontmatter.status,
                content_digest: &page.content_digest,
                tags: &page.frontmatter.tags,
                aliases: &page.frontmatter.aliases,
                sources: &page.frontmatter.sources,
                links: effective_links(page),
                contradictions: &page.frontmatter.contradictions,
            })
            .collect(),
        raw,
        suppression: &ledger.entries,
    };
    let bytes = serde_json::to_vec(&logical)
        .map_err(|error| WikiError::Io(format!("cannot serialize logical index: {error}")))?;
    Ok(sha256_digest(&bytes))
}

fn collect_hits(
    connection: &Connection,
    rows: Result<rusqlite::Rows<'_>, rusqlite::Error>,
    limit: usize,
) -> Result<Vec<QueryHit>, WikiError> {
    let mut rows = rows.map_err(sqlite_error)?;
    let mut hits = Vec::new();
    while let Some(row) = rows.next().map_err(sqlite_error)? {
        let id: String = row.get(0).map_err(sqlite_error)?;
        hits.push(QueryHit {
            kind: row.get(1).map_err(sqlite_error)?,
            summary: row.get(2).map_err(sqlite_error)?,
            path: row.get(3).map_err(sqlite_error)?,
            content_digest: row.get(4).map_err(sqlite_error)?,
            tags: select_values(connection, "tags", "tag", &id)?,
            aliases: select_values(connection, "aliases", "alias", &id)?,
            sources: select_values(connection, "sources", "locator", &id)?,
            id,
        });
        if hits.len() == limit {
            break;
        }
    }
    Ok(hits)
}

fn select_values(
    connection: &Connection,
    table: &str,
    column: &str,
    page_id: &str,
) -> Result<Vec<String>, WikiError> {
    let sql = format!("SELECT {column} FROM {table} WHERE page_id = ?1 ORDER BY {column}");
    let mut statement = connection.prepare(&sql).map_err(sqlite_error)?;
    let values = statement
        .query_map([page_id], |row| row.get(0))
        .map_err(sqlite_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sqlite_error)?;
    Ok(values)
}

fn fts_expression(value: &str) -> Result<String, WikiError> {
    let tokens: Vec<_> = value
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect();
    if tokens.is_empty() {
        return Err(WikiError::InvalidInput(
            "query text must contain a searchable token".to_owned(),
        ));
    }
    Ok(tokens.join(" AND "))
}

fn render_page(frontmatter: &WikiFrontmatter, body: &str) -> Result<Vec<u8>, WikiError> {
    let yaml = serde_yaml::to_string(frontmatter)
        .map_err(|error| WikiError::Io(format!("cannot serialize Wiki frontmatter: {error}")))?;
    let mut output = format!("---\n{}---\n\n", yaml.trim_start_matches("---\n"));
    output.push_str(body.trim());
    output.push('\n');
    Ok(output.into_bytes())
}

fn merge_existing_page(existing: &WikiPage, incoming: &mut WikiPage) -> Result<(), WikiError> {
    if existing.frontmatter.id != incoming.frontmatter.id {
        return Err(WikiError::Conflict(
            "existing Wiki page identity does not match the incoming draft".to_owned(),
        ));
    }
    if existing.frontmatter.kind != incoming.frontmatter.kind {
        return Err(WikiError::Conflict(format!(
            "Wiki page kind changed without a versioned migration: {}",
            incoming.frontmatter.id
        )));
    }
    incoming
        .frontmatter
        .created_at
        .clone_from(&existing.frontmatter.created_at);
    merge_sorted_unique(&mut incoming.frontmatter.tags, &existing.frontmatter.tags);
    merge_sorted_unique(
        &mut incoming.frontmatter.aliases,
        &existing.frontmatter.aliases,
    );
    merge_sorted_unique(
        &mut incoming.frontmatter.sources,
        &existing.frontmatter.sources,
    );
    merge_sorted_unique(&mut incoming.frontmatter.links, &existing.frontmatter.links);
    for contradiction in &existing.frontmatter.contradictions {
        if !incoming.frontmatter.contradictions.contains(contradiction) {
            incoming
                .frontmatter
                .contradictions
                .push(contradiction.clone());
        }
    }
    incoming.frontmatter.contradictions.sort_by(|left, right| {
        (&left.source_a, &left.source_b, &left.summary).cmp(&(
            &right.source_a,
            &right.source_b,
            &right.summary,
        ))
    });
    Ok(())
}

fn commit_ingest_mutation(
    target: &Path,
    raw_path: &Path,
    raw_bytes: &[u8],
    wiki_path: &Path,
    wiki_bytes: &[u8],
) -> Result<(bool, bool, IndexOutcome), WikiError> {
    let raw_parent_existed = raw_path.parent().is_some_and(Path::exists);
    let snapshots = [
        FileSnapshot::capture(raw_path)?,
        FileSnapshot::capture(wiki_path)?,
        FileSnapshot::capture(&target.join(STALE_RELATIVE))?,
        FileSnapshot::capture(&target.join(INDEX_RELATIVE))?,
    ];
    let result = transactional(&snapshots, || {
        let raw_changed = write_immutable(raw_path, raw_bytes)?;
        let wiki_changed = write_atomic(wiki_path, wiki_bytes)?;
        if cfg!(debug_assertions)
            && env::var("HIVE_WIKI_TEST_FAIL_AFTER_CANONICAL_WRITES").as_deref() == Ok("1")
        {
            return Err(WikiError::Io(
                "injected failure after canonical knowledge writes".to_owned(),
            ));
        }
        mark_stale(target)?;
        let index = rebuild_index_locked(target)?;
        Ok((raw_changed, wiki_changed, index))
    });
    if result.is_err() && !raw_parent_existed {
        if let Some(parent) = raw_path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
    result
}

fn merge_sorted_unique(target: &mut Vec<String>, existing: &[String]) {
    let mut merged: BTreeSet<String> = target.iter().cloned().collect();
    merged.extend(existing.iter().cloned());
    *target = merged.into_iter().collect();
}

fn validate_frontmatter(value: &WikiFrontmatter) -> Result<(), WikiError> {
    if value.schema_version != 1 {
        return Err(WikiError::InvalidInput(
            "Wiki schema_version must be 1".to_owned(),
        ));
    }
    validate_id(&value.id)?;
    if !matches!(
        value.kind.as_str(),
        "source-summary" | "entity" | "concept" | "comparison" | "synthesis" | "open-question"
    ) {
        return Err(WikiError::InvalidInput(format!(
            "unsupported Wiki kind: {}",
            value.kind
        )));
    }
    if !matches!(
        value.status.as_str(),
        "active" | "contradicted" | "open-question"
    ) {
        return Err(WikiError::InvalidInput(format!(
            "deprecated or unsupported Wiki status: {}",
            value.status
        )));
    }
    validate_nonempty("summary", &value.summary)?;
    validate_timestamp(&value.created_at)?;
    validate_timestamp(&value.updated_at)?;
    validate_sorted_unique("tags", &value.tags)?;
    validate_sorted_unique("aliases", &value.aliases)?;
    validate_sorted_unique("sources", &value.sources)?;
    validate_sorted_unique("links", &value.links)?;
    for tag in &value.tags {
        if tag.starts_with('#') || !is_slug(tag) {
            return Err(WikiError::InvalidInput(format!(
                "tag must be a lowercase slug without #: {tag}"
            )));
        }
    }
    for link in &value.links {
        validate_id(link)?;
    }
    for source in &value.sources {
        if source != "raw:self" && parse_raw_locator(source).is_none() {
            return Err(WikiError::InvalidInput(format!(
                "source locator must identify an immutable Raw revision: {source}"
            )));
        }
    }
    for contradiction in &value.contradictions {
        validate_nonempty("contradiction summary", &contradiction.summary)?;
        validate_nonempty("contradiction source_a", &contradiction.source_a)?;
        validate_nonempty("contradiction source_b", &contradiction.source_b)?;
    }
    Ok(())
}

fn validate_suppression_entry(entry: &SuppressionEntry) -> Result<(), WikiError> {
    if !is_sha256(&entry.fingerprint) {
        return Err(WikiError::InvalidInput(
            "suppression fingerprint must be sha256:<64 lowercase hex>".to_owned(),
        ));
    }
    validate_suppression_locator("source_locator", &entry.source_locator)?;
    validate_suppression_reason(&entry.reason)?;
    if let Some(replacement) = &entry.replacement {
        validate_suppression_locator("replacement", replacement)?;
    }
    validate_timestamp(&entry.timestamp)
}

fn validate_suppression_reason(value: &str) -> Result<(), WikiError> {
    if !matches!(
        value,
        "credential-erasure"
            | "duplicate"
            | "invalid"
            | "legal-erasure"
            | "obsolete"
            | "out-of-scope"
            | "retention-expired"
            | "superseded"
            | "user-request"
    ) {
        return Err(WikiError::InvalidInput(
            "suppression reason must be a supported stable reason code".to_owned(),
        ));
    }
    Ok(())
}

fn validate_suppression_locator(name: &str, value: &str) -> Result<(), WikiError> {
    let valid = value
        .strip_prefix("wiki:")
        .or_else(|| value.strip_prefix("external:"))
        .is_some_and(|identifier| identifier.len() <= 80 && is_slug(identifier))
        || parse_raw_locator(value).is_some_and(|(path, _)| {
            path.starts_with(".hive/knowledge/Raw/") && !path.contains('\\')
        });
    if !valid {
        return Err(WikiError::InvalidInput(format!(
            "suppression {name} must be a canonical wiki, external, or immutable Raw locator"
        )));
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), WikiError> {
    if value.len() > 80 || !is_slug(value) {
        return Err(WikiError::InvalidInput(format!(
            "Wiki id must be a lowercase slug: {value}"
        )));
    }
    Ok(())
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn validate_timestamp(value: &str) -> Result<(), WikiError> {
    let bytes = value.as_bytes();
    let shape_valid = bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        });
    if !shape_valid {
        return Err(WikiError::InvalidInput(format!(
            "timestamp must use UTC seconds precision: {value}"
        )));
    }
    let component = |start: usize, end: usize| {
        value[start..end]
            .parse::<u32>()
            .expect("timestamp shape permits only ASCII digits")
    };
    let year = component(0, 4);
    let month = component(5, 7);
    let day = component(8, 10);
    let hour = component(11, 13);
    let minute = component(14, 16);
    let second = component(17, 19);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0
        || maximum_day == 0
        || !(1..=maximum_day).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(WikiError::InvalidInput(format!(
            "timestamp contains an impossible UTC date or time: {value}"
        )));
    }
    Ok(())
}

fn validate_sorted_unique(name: &str, values: &[String]) -> Result<(), WikiError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(WikiError::InvalidInput(format!(
            "{name} must be lexicographically sorted and unique"
        )));
    }
    for value in values {
        validate_nonempty(name, value)?;
    }
    Ok(())
}

fn validate_nonempty(name: &str, value: &str) -> Result<(), WikiError> {
    if value.is_empty() || value.trim() != value || value.contains('\0') {
        return Err(WikiError::InvalidInput(format!(
            "{name} must be a non-empty exact string without surrounding whitespace"
        )));
    }
    Ok(())
}

fn read_suppression(target: &Path) -> Result<SuppressionLedger, WikiError> {
    let path = target.join(SUPPRESSION_RELATIVE);
    let bytes = fs::read(&path).map_err(|error| {
        WikiError::Verification(format!(
            "cannot read suppression ledger {}: {error}",
            path.display()
        ))
    })?;
    let ledger: SuppressionLedger = serde_yaml::from_slice(&bytes)
        .map_err(|error| WikiError::Verification(format!("invalid suppression ledger: {error}")))?;
    if ledger.schema_version != 1 {
        return Err(WikiError::Verification(
            "suppression schema_version must be 1".to_owned(),
        ));
    }
    for entry in &ledger.entries {
        validate_suppression_entry(entry)
            .map_err(|error| WikiError::Verification(error.to_string()))?;
    }
    Ok(ledger)
}

fn raw_revision_path(source: &Path, fingerprint: &str) -> Result<String, WikiError> {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WikiError::InvalidInput("source filename must be UTF-8".to_owned()))?;
    let source_id = slugify(stem);
    if source_id.is_empty() {
        return Err(WikiError::InvalidInput(
            "source filename must contain an ASCII letter or digit".to_owned(),
        ));
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| value.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        .unwrap_or("raw")
        .to_ascii_lowercase();
    let digest = fingerprint
        .strip_prefix("sha256:")
        .ok_or_else(|| WikiError::InvalidInput("invalid source fingerprint".to_owned()))?;
    Ok(format!("{RAW_RELATIVE}/{source_id}/{digest}.{extension}"))
}

fn read_raw_source(source: &Path) -> Result<Vec<u8>, WikiError> {
    let file = fs::File::open(source).map_err(|error| {
        WikiError::Io(format!("cannot read source {}: {error}", source.display()))
    })?;
    let limit = u64::try_from(MAX_RAW_BYTES)
        .expect("Raw byte limit fits u64")
        .saturating_add(1);
    let mut bytes = Vec::new();
    file.take(limit).read_to_end(&mut bytes).map_err(|error| {
        WikiError::Io(format!("cannot read source {}: {error}", source.display()))
    })?;
    if bytes.len() > MAX_RAW_BYTES {
        return Err(WikiError::InvalidInput(
            "Raw source exceeds the 5 MiB Git-suitable limit".to_owned(),
        ));
    }
    Ok(bytes)
}

fn parse_raw_locator(locator: &str) -> Option<(&str, &str)> {
    let rest = locator.strip_prefix("raw:")?;
    let (path, fingerprint) = rest.rsplit_once('#')?;
    if !is_sha256(fingerprint) {
        return None;
    }
    let raw_path = path.strip_prefix(".hive/knowledge/Raw/")?;
    let (source_id, filename) = raw_path.split_once('/')?;
    if filename.contains('/') || !is_slug(source_id) {
        return None;
    }
    let (digest, extension) = filename.rsplit_once('.')?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || extension.is_empty()
        || !extension.bytes().all(|byte| byte.is_ascii_alphanumeric())
        || fingerprint.strip_prefix("sha256:") != Some(digest)
    {
        return None;
    }
    Some((path, fingerprint))
}

fn reject_likely_credentials(bytes: &[u8]) -> Result<(), WikiError> {
    let text = String::from_utf8_lossy(bytes);
    let lowered = text.to_ascii_lowercase();
    let markers = [
        "authorization: bearer ",
        "authorization: basic ",
        "putty-user-key-file-",
        "ghp_",
        "github_pat_",
        "glpat-",
        "npm_",
        "pypi-",
        "hf_",
        "xoxb-",
        "xoxp-",
        "xapp-",
        "sk-proj-",
        "sk-ant-",
        "sk-or-v1-",
    ];
    let assignment_names = [
        "api_key",
        "apikey",
        "api-key",
        "openai_api_key",
        "anthropic_api_key",
        "gemini_api_key",
        "google_api_key",
        "access_token",
        "access-token",
        "auth_token",
        "refresh_token",
        "private_token",
        "token",
        "_auth",
        "auth",
        "client_secret",
        "client-secret",
        "client_key",
        "client-key",
        "client-key-data",
        "private_key",
        "private-key",
        "privatekey",
        "secret",
        "password",
        "passphrase",
        "secret_key",
        "aws_secret_access_key",
    ];
    let assigned_credential = lowered.lines().any(|line| {
        assignment_names
            .iter()
            .any(|name| credential_assignment_value(line, name).is_some_and(suspicious_secret))
    });
    let whitespace_credential = lowered.lines().any(|line| {
        let netrc_shape = line.contains("machine ") && line.contains(" login ");
        netrc_shape && {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields.windows(2).any(|pair| {
                matches!(
                    pair[0].trim_matches(|character: char| !character.is_ascii_alphanumeric()),
                    "password" | "passphrase" | "token"
                ) && suspicious_secret(pair[1].trim_matches(|character: char| {
                    matches!(character, '"' | '\'' | '`' | ',' | ';')
                }))
            })
        }
    });
    let token_prefix = lowered
        .split(|character: char| character.is_whitespace() || matches!(character, '"' | '\''))
        .any(|token| {
            (token.starts_with("sk-") && token.len() >= 12)
                || (token.starts_with("akia") && token.len() >= 16)
                || (token.starts_with("asia") && token.len() >= 16)
                || (token.starts_with("aiza") && token.len() >= 20)
        });
    let private_key_block = lowered
        .lines()
        .any(|line| line.contains("-----begin ") && line.contains("private key"));
    if assigned_credential
        || whitespace_credential
        || token_prefix
        || private_key_block
        || markers.iter().any(|marker| lowered.contains(marker))
    {
        return Err(WikiError::InvalidInput(
            "Raw source appears to contain a credential or private key".to_owned(),
        ));
    }
    Ok(())
}

fn credential_assignment_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let mut offset = 0;
    while let Some(relative_index) = line.get(offset..)?.find(name) {
        let index = offset + relative_index;
        let before = line.get(..index)?;
        let has_identifier_prefix = before
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        if !has_identifier_prefix {
            let after_name = line.get(index + name.len()..)?.trim_start();
            let after_name = after_name
                .strip_prefix('"')
                .or_else(|| after_name.strip_prefix('\''))
                .unwrap_or(after_name)
                .trim_start();
            if let Some(value) = after_name
                .strip_prefix('=')
                .or_else(|| after_name.strip_prefix(':'))
            {
                return Some(
                    value
                        .trim()
                        .trim_matches(|character| matches!(character, '"' | '\'' | '`')),
                );
            }
        }
        offset = index.saturating_add(name.len());
    }
    None
}

fn suspicious_secret(value: &str) -> bool {
    let normalized = value.trim();
    let explicit_placeholder = matches!(
        normalized,
        "redacted"
            | "[redacted]"
            | "<redacted>"
            | "example"
            | "example-value"
            | "placeholder"
            | "<placeholder>"
            | "your-api-key"
            | "changeme"
    );
    if normalized.is_empty()
        || (normalized.starts_with("${") && normalized.ends_with('}'))
        || explicit_placeholder
        || normalized.bytes().all(|byte| matches!(byte, b'*' | b'x'))
    {
        return false;
    }
    normalized.bytes().any(|byte| byte.is_ascii_alphanumeric())
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for byte in value.bytes().map(|byte| byte.to_ascii_lowercase()) {
        if byte.is_ascii_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(char::from(byte));
            separator = false;
        } else {
            separator = true;
        }
    }
    output
}

fn effective_links(page: &WikiPage) -> Vec<String> {
    let mut links: BTreeSet<String> = page.frontmatter.links.iter().cloned().collect();
    let mut remaining = page.body.as_str();
    while let Some(start) = remaining.find("[[") {
        let after = &remaining[start + 2..];
        let Some(end) = after.find("]]") else {
            break;
        };
        let candidate = &after[..end];
        if is_slug(candidate) {
            links.insert(candidate.to_owned());
        }
        remaining = &after[end + 2..];
    }
    links.into_iter().collect()
}

fn issue(code: &str, severity: LintSeverity, locator: &str, message: &str) -> LintIssue {
    LintIssue {
        code: code.to_owned(),
        severity,
        locator: locator.to_owned(),
        message: message.to_owned(),
    }
}

impl PinnedRoot {
    fn open(path: &Path) -> Result<Self, WikiError> {
        let canonical_path = path.canonicalize().map_err(|error| {
            WikiError::Verification(format!(
                "cannot canonicalize knowledge root {}: {error}",
                path.display()
            ))
        })?;
        let parent = canonical_path
            .parent()
            .ok_or_else(|| WikiError::InvalidInput("knowledge root has no parent".to_owned()))?;
        let name = canonical_path.file_name().ok_or_else(|| {
            WikiError::InvalidInput("knowledge root has no directory name".to_owned())
        })?;
        let parent_dir = Dir::open_ambient_dir(parent, ambient_authority())
            .map_err(|error| WikiError::Io(format!("cannot pin knowledge root parent: {error}")))?;
        let expected = parent_dir.symlink_metadata(name).map_err(|error| {
            WikiError::Io(format!(
                "cannot inspect knowledge root {}: {error}",
                canonical_path.display()
            ))
        })?;
        let dir = parent_dir.open_dir_nofollow(name).map_err(|error| {
            WikiError::Conflict(format!(
                "knowledge root cannot be pinned no-follow {}: {error}",
                canonical_path.display()
            ))
        })?;
        let pinned = dir
            .dir_metadata()
            .map_err(|error| WikiError::Io(format!("cannot inspect pinned root: {error}")))?;
        if (CapMetadataExt::dev(&pinned), CapMetadataExt::ino(&pinned))
            != (
                CapMetadataExt::dev(&expected),
                CapMetadataExt::ino(&expected),
            )
        {
            return Err(WikiError::Conflict(
                "knowledge root changed while its capability was pinned".to_owned(),
            ));
        }
        Ok(Self {
            dir,
            canonical_path,
        })
    }

    fn validate_project_root(&self) -> Result<(), WikiError> {
        self.validate_knowledge_root()?;
        if read_capability_optional(&self.dir, Path::new(".hive/setup-answers.yml"))?.is_none() {
            return Err(WikiError::Verification(
                "project promotion policy is missing".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_knowledge_root(&self) -> Result<(), WikiError> {
        if read_capability_optional(&self.dir, Path::new("hive-source.json"))?.is_some() {
            return Err(WikiError::InvalidInput(
                "Hive source workspace cannot be used as a consumer target".to_owned(),
            ));
        }
        for relative in [Path::new(WIKI_RELATIVE), Path::new(RAW_RELATIVE)] {
            let (parent, name) =
                capability_parent(&self.dir, relative, false)?.ok_or_else(|| {
                    WikiError::Verification(format!(
                        "canonical knowledge directory is missing: {}",
                        relative.display()
                    ))
                })?;
            parent.open_dir_nofollow(&name).map_err(|error| {
                WikiError::Conflict(format!(
                    "canonical knowledge directory is not pinned no-follow {}: {error}",
                    relative.display()
                ))
            })?;
        }
        read_capability_required(&self.dir, Path::new(SUPPRESSION_RELATIVE))?;
        Ok(())
    }
}

fn validate_capability_relative(relative: &Path) -> Result<(), WikiError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            !matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        return Err(WikiError::Conflict(format!(
            "managed knowledge path is not a safe relative path: {}",
            relative.display()
        )));
    }
    Ok(())
}

fn capability_parent(
    root: &Dir,
    relative: &Path,
    create_missing: bool,
) -> Result<Option<(Dir, OsString)>, WikiError> {
    validate_capability_relative(relative)?;
    let name = relative
        .file_name()
        .ok_or_else(|| WikiError::Io("managed path has no filename".to_owned()))?
        .to_os_string();
    let mut current = root
        .try_clone()
        .map_err(|error| WikiError::Io(format!("cannot clone root capability: {error}")))?;
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let component = component.as_os_str();
            match current.symlink_metadata(component) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(WikiError::Conflict(format!(
                        "managed knowledge ancestor is not a directory: {}",
                        relative.display()
                    )));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound && create_missing => {
                    current.create_dir(component).map_err(|error| {
                        WikiError::Io(format!(
                            "cannot create managed knowledge directory {}: {error}",
                            relative.display()
                        ))
                    })?;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error) => {
                    return Err(WikiError::Io(format!(
                        "cannot inspect managed knowledge ancestor {}: {error}",
                        relative.display()
                    )));
                }
            }
            current = current.open_dir_nofollow(component).map_err(|error| {
                WikiError::Conflict(format!(
                    "cannot open managed knowledge ancestor no-follow {}: {error}",
                    relative.display()
                ))
            })?;
        }
    }
    Ok(Some((current, name)))
}

fn open_capability_file_nofollow(parent: &Dir, name: &OsStr) -> io::Result<cap_std::fs::File> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    parent.open_with(name, &options)
}

fn read_capability_optional(root: &Dir, relative: &Path) -> Result<Option<Vec<u8>>, WikiError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(None);
    };
    match parent.symlink_metadata(&name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(WikiError::Io(format!(
            "cannot inspect managed knowledge file {}: {error}",
            relative.display()
        ))),
        Ok(metadata) if metadata.is_file() => {
            let mut file = open_capability_file_nofollow(&parent, &name).map_err(|error| {
                WikiError::Io(format!(
                    "cannot open managed knowledge file no-follow {}: {error}",
                    relative.display()
                ))
            })?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(|error| {
                WikiError::Io(format!(
                    "cannot read managed knowledge file {}: {error}",
                    relative.display()
                ))
            })?;
            Ok(Some(bytes))
        }
        Ok(_) => Err(WikiError::Conflict(format!(
            "managed knowledge path is not a regular file: {}",
            relative.display()
        ))),
    }
}

fn read_capability_required(root: &Dir, relative: &Path) -> Result<Vec<u8>, WikiError> {
    read_capability_optional(root, relative)?.ok_or_else(|| {
        WikiError::Verification(format!(
            "managed knowledge file is missing: {}",
            relative.display()
        ))
    })
}

fn scan_pages_capability(root: &Dir) -> Result<BTreeMap<String, WikiPage>, WikiError> {
    let (parent, name) = capability_parent(root, Path::new(WIKI_RELATIVE), false)?
        .ok_or_else(|| WikiError::Verification("canonical Wiki directory is missing".to_owned()))?;
    let wiki = parent.open_dir_nofollow(&name).map_err(|error| {
        WikiError::Conflict(format!(
            "cannot open canonical Wiki directory no-follow: {error}"
        ))
    })?;
    let mut names = wiki
        .entries()
        .map_err(|error| WikiError::Io(format!("cannot scan Wiki directory: {error}")))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name())
                .map_err(|error| WikiError::Io(format!("cannot scan Wiki directory: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let mut pages = BTreeMap::new();
    for name in names {
        let metadata = wiki
            .symlink_metadata(&name)
            .map_err(|error| WikiError::Io(format!("cannot inspect Wiki entry: {error}")))?;
        if !metadata.is_file() {
            if metadata.is_dir() {
                continue;
            }
            return Err(WikiError::Verification(
                "Wiki symlink or special file is forbidden".to_owned(),
            ));
        }
        let path = Path::new(&name);
        if path.extension().and_then(|value| value.to_str()) != Some("md")
            || matches!(
                path.file_name().and_then(|value| value.to_str()),
                Some("index.md" | "log.md")
            )
        {
            continue;
        }
        let mut file = open_capability_file_nofollow(&wiki, &name)
            .map_err(|error| WikiError::Io(format!("cannot open Wiki page: {error}")))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| WikiError::Io(format!("cannot read Wiki page: {error}")))?;
        let relative = format!("{WIKI_RELATIVE}/{}", path.to_string_lossy());
        reject_likely_credentials(&bytes).map_err(|error| {
            WikiError::Verification(format!(
                "canonical Wiki page contains likely sensitive material at {relative}: {error}"
            ))
        })?;
        let mut page = parse_page_bytes(&bytes, &relative)?;
        if page
            .frontmatter
            .sources
            .iter()
            .any(|source| source == "raw:self")
        {
            return Err(WikiError::Verification(format!(
                "raw:self is allowed only in prepared drafts: {relative}"
            )));
        }
        if path.file_stem().and_then(|value| value.to_str()) != Some(page.frontmatter.id.as_str()) {
            return Err(WikiError::Verification(format!(
                "Wiki filename must match page id: {relative}"
            )));
        }
        page.relative_path = relative;
        page.content_digest = sha256_digest(&bytes);
        if pages.insert(page.frontmatter.id.clone(), page).is_some() {
            return Err(WikiError::Verification("duplicate Wiki page id".to_owned()));
        }
    }
    Ok(pages)
}

fn scan_raw_capability(root: &Dir) -> Result<BTreeMap<String, String>, WikiError> {
    let (parent, name) = capability_parent(root, Path::new(RAW_RELATIVE), false)?
        .ok_or_else(|| WikiError::Verification("canonical Raw directory is missing".to_owned()))?;
    let raw = parent.open_dir_nofollow(&name).map_err(|error| {
        WikiError::Conflict(format!(
            "cannot open canonical Raw directory no-follow: {error}"
        ))
    })?;
    let mut output = BTreeMap::new();
    scan_raw_capability_directory(&raw, Path::new(RAW_RELATIVE), &mut output)?;
    Ok(output)
}

fn scan_raw_capability_directory(
    directory: &Dir,
    relative: &Path,
    output: &mut BTreeMap<String, String>,
) -> Result<(), WikiError> {
    let mut names = directory
        .entries()
        .map_err(|error| WikiError::Io(format!("cannot scan Raw directory: {error}")))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name())
                .map_err(|error| WikiError::Io(format!("cannot scan Raw directory: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    for name in names {
        let child_relative = relative.join(&name);
        let metadata = directory
            .symlink_metadata(&name)
            .map_err(|error| WikiError::Io(format!("cannot inspect Raw entry: {error}")))?;
        if metadata.is_dir() {
            let child = directory.open_dir_nofollow(&name).map_err(|error| {
                WikiError::Conflict(format!(
                    "cannot open Raw directory no-follow {}: {error}",
                    child_relative.display()
                ))
            })?;
            scan_raw_capability_directory(&child, &child_relative, output)?;
        } else if metadata.is_file() {
            let relative_text = child_relative.to_string_lossy().replace('\\', "/");
            if relative_text == ".hive/knowledge/Raw/README.md" {
                continue;
            }
            let file = open_capability_file_nofollow(directory, &name)
                .map_err(|error| WikiError::Io(format!("cannot open Raw revision: {error}")))?;
            let mut bytes = Vec::new();
            file.take(u64::try_from(MAX_RAW_BYTES + 1).expect("Raw byte limit fits u64"))
                .read_to_end(&mut bytes)
                .map_err(|error| WikiError::Io(format!("cannot read Raw revision: {error}")))?;
            if bytes.is_empty() || bytes.len() > MAX_RAW_BYTES {
                return Err(WikiError::Verification(format!(
                    "canonical Raw revision violates the bounded source contract at {relative_text}"
                )));
            }
            let digest = sha256_digest(&bytes);
            let expected = child_relative
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| format!("sha256:{value}"))
                .ok_or_else(|| {
                    WikiError::Verification(format!(
                        "Raw revision filename is not UTF-8: {relative_text}"
                    ))
                })?;
            if digest != expected {
                return Err(WikiError::Verification(format!(
                    "Raw revision content digest does not match its immutable path: {relative_text}"
                )));
            }
            reject_likely_credentials(&bytes).map_err(|error| {
                WikiError::Verification(format!(
                    "canonical Raw revision contains likely sensitive material at {relative_text}: {error}"
                ))
            })?;
            output.insert(relative_text, digest);
        } else {
            return Err(WikiError::Verification(format!(
                "Raw symlink or special file is forbidden: {}",
                child_relative.display()
            )));
        }
    }
    Ok(())
}

fn read_suppression_capability(root: &Dir) -> Result<SuppressionLedger, WikiError> {
    let bytes = read_capability_required(root, Path::new(SUPPRESSION_RELATIVE))?;
    let ledger: SuppressionLedger = serde_yaml::from_slice(&bytes)
        .map_err(|error| WikiError::Verification(format!("invalid suppression ledger: {error}")))?;
    if ledger.schema_version != 1 {
        return Err(WikiError::Verification(
            "suppression schema_version must be 1".to_owned(),
        ));
    }
    for entry in &ledger.entries {
        validate_suppression_entry(entry)
            .map_err(|error| WikiError::Verification(error.to_string()))?;
    }
    Ok(ledger)
}

fn capability_temp_name(prefix: &str) -> OsString {
    let counter = CAP_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    OsString::from(format!(
        ".{prefix}-{}-{nanos}-{counter}",
        std::process::id()
    ))
}

fn create_capability_file(parent: &Dir, name: &OsStr, bytes: &[u8]) -> Result<(), WikiError> {
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(name, &options)
        .map_err(|error| WikiError::Io(format!("cannot create managed knowledge file: {error}")))?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = parent.remove_file(name);
        return Err(WikiError::Io(format!(
            "cannot persist managed knowledge file: {error}"
        )));
    }
    Ok(())
}

fn capability_directory_exists(root: &Dir, relative: &Path) -> Result<bool, WikiError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(false);
    };
    match parent.symlink_metadata(&name) {
        Ok(metadata) if metadata.is_dir() => {
            parent.open_dir_nofollow(&name).map_err(|error| {
                WikiError::Conflict(format!(
                    "managed knowledge directory is not no-follow {}: {error}",
                    relative.display()
                ))
            })?;
            Ok(true)
        }
        Ok(_) => Err(WikiError::Conflict(format!(
            "managed knowledge path is not a directory: {}",
            relative.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(WikiError::Io(format!(
            "cannot inspect managed knowledge directory {}: {error}",
            relative.display()
        ))),
    }
}

fn remove_capability_empty_directory(root: &Dir, relative: &Path) -> Result<(), WikiError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(());
    };
    match parent.remove_dir(&name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(WikiError::Io(format!(
            "cannot remove rolled-back knowledge directory {}: {error}",
            relative.display()
        ))),
    }
}

#[derive(Clone, Eq, PartialEq)]
struct CapabilityFileMode {
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}

#[derive(Clone, Eq, PartialEq)]
enum CapabilityFileState {
    Missing,
    File {
        bytes: Vec<u8>,
        mode: CapabilityFileMode,
    },
}

fn capability_file_state(root: &Dir, relative: &Path) -> Result<CapabilityFileState, WikiError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(CapabilityFileState::Missing);
    };
    match parent.symlink_metadata(&name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(CapabilityFileState::Missing),
        Err(error) => Err(WikiError::Io(format!(
            "cannot inspect managed knowledge file {}: {error}",
            relative.display()
        ))),
        Ok(metadata) if metadata.is_file() => {
            let mut file = open_capability_file_nofollow(&parent, &name).map_err(|error| {
                WikiError::Io(format!(
                    "cannot open managed knowledge file {}: {error}",
                    relative.display()
                ))
            })?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(|error| {
                WikiError::Io(format!(
                    "cannot read managed knowledge file {}: {error}",
                    relative.display()
                ))
            })?;
            let permissions = file
                .metadata()
                .map_err(|error| {
                    WikiError::Io(format!(
                        "cannot inspect managed knowledge mode {}: {error}",
                        relative.display()
                    ))
                })?
                .permissions();
            Ok(CapabilityFileState::File {
                bytes,
                mode: CapabilityFileMode {
                    readonly: permissions.readonly(),
                    #[cfg(unix)]
                    mode: CapPermissionsExt::mode(&permissions),
                },
            })
        }
        Ok(_) => Err(WikiError::Conflict(format!(
            "managed knowledge path is not a regular file: {}",
            relative.display()
        ))),
    }
}

struct CapabilityFileSnapshot {
    relative: PathBuf,
    original: CapabilityFileState,
    current: CapabilityFileState,
    original_claim: Option<PathBuf>,
    disposable_claims: Vec<PathBuf>,
    modified: bool,
}

impl CapabilityFileSnapshot {
    fn capture(root: &Dir, relative: &Path) -> Result<Self, WikiError> {
        let state = capability_file_state(root, relative)?;
        Ok(Self {
            relative: relative.to_path_buf(),
            original: state.clone(),
            current: state,
            original_claim: None,
            disposable_claims: Vec::new(),
            modified: false,
        })
    }

    fn claim_current(&mut self, root: &Dir) -> Result<PathBuf, WikiError> {
        let (parent, name) = capability_parent(root, &self.relative, false)?.ok_or_else(|| {
            WikiError::Conflict(format!(
                "canonical path disappeared before claim: {}",
                self.relative.display()
            ))
        })?;
        let claim_name = capability_temp_name("hive-wiki-claim");
        parent
            .rename(&name, &parent, &claim_name)
            .map_err(|error| {
                WikiError::Conflict(format!(
                    "canonical path changed before claim {}: {error}",
                    self.relative.display()
                ))
            })?;
        let claim_relative = self
            .relative
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(&claim_name);
        let claimed = capability_file_state(root, &claim_relative)?;
        if claimed != self.current {
            let restored = parent.hard_link(&claim_name, &parent, &name).is_ok();
            if restored {
                let _ = parent.remove_file(&claim_name);
            }
            return Err(WikiError::Conflict(format!(
                "canonical path changed after planning {}; claimed object retained at {}",
                self.relative.display(),
                claim_relative.display()
            )));
        }
        if self.original_claim.is_none() && self.current == self.original {
            self.original_claim = Some(claim_relative.clone());
        } else {
            self.disposable_claims.push(claim_relative.clone());
        }
        Ok(claim_relative)
    }

    fn install_staged(&mut self, root: &Dir, bytes: &[u8]) -> Result<bool, WikiError> {
        if let CapabilityFileState::File {
            bytes: current_bytes,
            ..
        } = &self.current
        {
            if current_bytes == bytes {
                let live = capability_file_state(root, &self.relative)?;
                if live == self.current {
                    return Ok(false);
                }
                return Err(WikiError::Conflict(format!(
                    "canonical path changed after planning: {}",
                    self.relative.display()
                )));
            }
        }
        let (parent, name) = capability_parent(root, &self.relative, true)?
            .ok_or_else(|| WikiError::Io("canonical parent disappeared".to_owned()))?;
        let temporary_name = capability_temp_name("hive-wiki-tmp");
        create_capability_file(&parent, &temporary_name, bytes)?;
        let temporary_relative = self
            .relative
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(&temporary_name);
        let installed = capability_file_state(root, &temporary_relative)?;
        if !matches!(self.current, CapabilityFileState::Missing) {
            if let Err(error) = self.claim_current(root) {
                let _ = parent.remove_file(&temporary_name);
                return Err(error);
            }
            capability_test_pause("after-claim-before-install")?;
        }
        if let Err(error) = parent.hard_link(&temporary_name, &parent, &name) {
            let _ = parent.remove_file(&temporary_name);
            return Err(WikiError::Conflict(format!(
                "canonical destination was reoccupied before install {}; claimed prior object retained for recovery: {error}",
                self.relative.display()
            )));
        }
        parent.remove_file(&temporary_name).map_err(|error| {
            WikiError::Io(format!(
                "cannot remove staged canonical file {}: {error}",
                self.relative.display()
            ))
        })?;
        self.current = installed;
        self.modified = true;
        Ok(true)
    }

    fn write_immutable(&mut self, root: &Dir, bytes: &[u8]) -> Result<bool, WikiError> {
        match &self.current {
            CapabilityFileState::File {
                bytes: current_bytes,
                ..
            } if current_bytes == bytes => {
                if capability_file_state(root, &self.relative)? == self.current {
                    Ok(false)
                } else {
                    Err(WikiError::Conflict(format!(
                        "immutable Raw revision changed after planning: {}",
                        self.relative.display()
                    )))
                }
            }
            CapabilityFileState::File { .. } => Err(WikiError::Conflict(format!(
                "immutable Raw revision path has different bytes: {}",
                self.relative.display()
            ))),
            CapabilityFileState::Missing => {
                let (parent, name) = capability_parent(root, &self.relative, true)?
                    .ok_or_else(|| WikiError::Io("immutable Raw parent disappeared".to_owned()))?;
                create_capability_file(&parent, &name, bytes).map_err(|error| match error {
                    WikiError::Io(message) => WikiError::Conflict(format!(
                        "immutable Raw destination changed after planning {}: {message}",
                        self.relative.display()
                    )),
                    other => other,
                })?;
                self.current = capability_file_state(root, &self.relative)?;
                self.modified = true;
                Ok(true)
            }
        }
    }

    fn remove(&mut self, root: &Dir) -> Result<bool, WikiError> {
        if matches!(self.current, CapabilityFileState::Missing) {
            if matches!(
                capability_file_state(root, &self.relative)?,
                CapabilityFileState::Missing
            ) {
                return Ok(false);
            }
            return Err(WikiError::Conflict(format!(
                "canonical path appeared after planning: {}",
                self.relative.display()
            )));
        }
        self.claim_current(root)?;
        self.current = CapabilityFileState::Missing;
        self.modified = true;
        Ok(true)
    }

    fn rollback(&mut self, root: &Dir) -> Result<(), WikiError> {
        if !self.modified {
            return Ok(());
        }
        match &self.current {
            CapabilityFileState::Missing => {
                if !matches!(
                    capability_file_state(root, &self.relative)?,
                    CapabilityFileState::Missing
                ) {
                    return Err(WikiError::Conflict(format!(
                        "rollback preserved externally reoccupied path {}",
                        self.relative.display()
                    )));
                }
            }
            CapabilityFileState::File { .. } => {
                let rollback_claim = self.claim_current(root)?;
                self.disposable_claims.push(rollback_claim);
            }
        }
        if matches!(self.original, CapabilityFileState::File { .. }) {
            let claim = self.original_claim.as_ref().ok_or_else(|| {
                WikiError::Io(format!(
                    "rollback claim is missing for {}",
                    self.relative.display()
                ))
            })?;
            let (claim_parent, claim_name) = capability_parent(root, claim, false)?
                .ok_or_else(|| WikiError::Io("rollback claim parent disappeared".to_owned()))?;
            let (live_parent, live_name) = capability_parent(root, &self.relative, true)?
                .ok_or_else(|| {
                    WikiError::Io("rollback destination parent disappeared".to_owned())
                })?;
            claim_parent
                .hard_link(&claim_name, &live_parent, &live_name)
                .map_err(|error| WikiError::Conflict(format!(
                    "rollback preserved externally reoccupied path {}; prior object retained at {}: {error}",
                    self.relative.display(),
                    claim.display()
                )))?;
        }
        self.cleanup_claims(root);
        Ok(())
    }

    fn cleanup_claims(&mut self, root: &Dir) {
        for claim in self
            .original_claim
            .iter()
            .chain(self.disposable_claims.iter())
        {
            if let Ok(Some((parent, name))) = capability_parent(root, claim, false) {
                let _ = parent.remove_file(name);
            }
        }
    }
}

fn transactional_capability<T>(
    root: &Dir,
    snapshots: &mut [CapabilityFileSnapshot],
    operation: impl FnOnce(&mut [CapabilityFileSnapshot]) -> Result<T, WikiError>,
) -> Result<T, WikiError> {
    match operation(snapshots) {
        Ok(outcome) => {
            for snapshot in snapshots {
                snapshot.cleanup_claims(root);
            }
            Ok(outcome)
        }
        Err(operation_error) => {
            capability_test_pause("before-rollback")?;
            let mut rollback_error = None;
            for snapshot in snapshots.iter_mut().rev() {
                if let Err(error) = snapshot.rollback(root) {
                    rollback_error.get_or_insert(error);
                }
            }
            if let Some(rollback_error) = rollback_error {
                return Err(WikiError::Io(format!(
                    "knowledge mutation failed: {operation_error}; rollback failed: {rollback_error}"
                )));
            }
            Err(operation_error)
        }
    }
}

fn capability_test_pause(phase: &str) -> Result<(), WikiError> {
    if !cfg!(debug_assertions) || env::var("HIVE_WIKI_TEST_RACE_PHASE").as_deref() != Ok(phase) {
        return Ok(());
    }
    let directory = env::var_os("HIVE_WIKI_TEST_RACE_DIR")
        .ok_or_else(|| WikiError::Io("race hook directory is missing".to_owned()))?;
    let directory = PathBuf::from(directory);
    fs::write(directory.join("ready"), phase.as_bytes())
        .map_err(|error| WikiError::Io(format!("cannot signal race hook: {error}")))?;
    let deadline = Instant::now() + LOCK_TIMEOUT;
    while !directory.join("continue").exists() {
        if Instant::now() >= deadline {
            return Err(WikiError::Io(format!("race hook timed out at {phase}")));
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn rebuild_index_capability(
    root: &Dir,
    index_snapshot: &mut CapabilityFileSnapshot,
    stale_snapshot: &mut CapabilityFileSnapshot,
) -> Result<IndexOutcome, WikiError> {
    let pages = scan_pages_capability(root)?;
    let raw = scan_raw_capability(root)?;
    let ledger = read_suppression_capability(root)?;
    if let Some(locator) = active_suppression_locator(&pages, &raw, &ledger) {
        return Err(WikiError::Verification(format!(
            "suppression ledger overlaps active canonical content: {locator}"
        )));
    }
    let expected_digest = logical_digest(&pages, &raw, &ledger)?;
    let staging = tempfile::tempdir()
        .map_err(|error| WikiError::Io(format!("cannot create private index staging: {error}")))?;
    fs::create_dir_all(staging.path().join(WIKI_RELATIVE))
        .and_then(|()| fs::create_dir_all(staging.path().join(RAW_RELATIVE)))
        .map_err(|error| WikiError::Io(format!("cannot prepare private index staging: {error}")))?;
    let ledger_bytes = serde_yaml::to_string(&ledger)
        .map_err(|error| WikiError::Io(format!("cannot stage suppression ledger: {error}")))?;
    fs::write(
        staging.path().join(SUPPRESSION_RELATIVE),
        ledger_bytes.as_bytes(),
    )
    .map_err(|error| WikiError::Io(format!("cannot stage suppression ledger: {error}")))?;
    for page in pages.values() {
        let bytes = render_page(&page.frontmatter, page.body.trim())?;
        fs::write(staging.path().join(&page.relative_path), bytes)
            .map_err(|error| WikiError::Io(format!("cannot stage Wiki page: {error}")))?;
    }
    for path in raw.keys() {
        let relative = Path::new(path);
        let bytes = read_capability_required(root, relative)?;
        let destination = staging.path().join(relative);
        fs::create_dir_all(
            destination
                .parent()
                .ok_or_else(|| WikiError::Io("staged Raw path has no parent".to_owned()))?,
        )
        .map_err(|error| WikiError::Io(format!("cannot stage Raw directory: {error}")))?;
        fs::write(destination, bytes)
            .map_err(|error| WikiError::Io(format!("cannot stage Raw revision: {error}")))?;
    }
    let staged = rebuild_index_locked(staging.path())?;
    if staged.logical_digest != expected_digest {
        return Err(WikiError::Verification(
            "private index staging changed the canonical logical digest".to_owned(),
        ));
    }
    let index_bytes = fs::read(staging.path().join(INDEX_RELATIVE))
        .map_err(|error| WikiError::Io(format!("cannot read verified staged index: {error}")))?;
    index_snapshot.install_staged(root, &index_bytes)?;
    let stale_removed = stale_snapshot.remove(root)?;
    let mut changed_paths = vec![INDEX_RELATIVE.to_owned()];
    if stale_removed {
        changed_paths.push(STALE_RELATIVE.to_owned());
    }
    Ok(IndexOutcome {
        changed_paths,
        page_count: pages.len(),
        raw_count: raw.len(),
        logical_digest: expected_digest,
    })
}

fn commit_promotion_mutation(
    root: &PinnedRoot,
    snapshots: &mut [CapabilityFileSnapshot; 4],
    raw_bytes: &[u8],
    wiki_bytes: &[u8],
) -> Result<(bool, bool, IndexOutcome), WikiError> {
    let raw_parent = snapshots[0]
        .relative
        .parent()
        .ok_or_else(|| WikiError::Io("promotion Raw path has no parent".to_owned()))?
        .to_path_buf();
    let raw_parent_existed = capability_directory_exists(&root.dir, &raw_parent)?;
    let result = transactional_capability(&root.dir, snapshots, |snapshots| {
        let raw_changed = snapshots[0].write_immutable(&root.dir, raw_bytes)?;
        let wiki_changed = snapshots[1].install_staged(&root.dir, wiki_bytes)?;
        if cfg!(debug_assertions)
            && env::var("HIVE_WIKI_TEST_FAIL_AFTER_CANONICAL_WRITES").as_deref() == Ok("1")
        {
            return Err(WikiError::Io(
                "injected failure after canonical knowledge writes".to_owned(),
            ));
        }
        snapshots[2].install_staged(&root.dir, b"{\"schema_version\":1,\"stale\":true}\n")?;
        let (canonical, derived) = snapshots.split_at_mut(3);
        let index = rebuild_index_capability(&root.dir, &mut derived[0], &mut canonical[2])?;
        Ok((raw_changed, wiki_changed, index))
    });
    if result.is_err() && !raw_parent_existed {
        remove_capability_empty_directory(&root.dir, &raw_parent)?;
    }
    result
}

enum FileState {
    Missing,
    File(Vec<u8>),
}

struct FileSnapshot {
    path: PathBuf,
    state: FileState,
}

impl FileSnapshot {
    fn capture(path: &Path) -> Result<Self, WikiError> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                Err(WikiError::Conflict(format!(
                    "managed knowledge path is not a regular file: {}",
                    path.display()
                )))
            }
            Ok(_) => Ok(Self {
                path: path.to_path_buf(),
                state: FileState::File(fs::read(path).map_err(|error| {
                    WikiError::Io(format!("cannot snapshot {}: {error}", path.display()))
                })?),
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self {
                path: path.to_path_buf(),
                state: FileState::Missing,
            }),
            Err(error) => Err(WikiError::Io(format!(
                "cannot inspect managed knowledge path {}: {error}",
                path.display()
            ))),
        }
    }

    fn restore(&self) -> Result<(), WikiError> {
        match &self.state {
            FileState::Missing => match fs::symlink_metadata(&self.path) {
                Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
                    fs::remove_file(&self.path).map_err(|error| {
                        WikiError::Io(format!(
                            "cannot remove rolled-back file {}: {error}",
                            self.path.display()
                        ))
                    })
                }
                Ok(_) => Err(WikiError::Io(format!(
                    "cannot roll back non-file path {}",
                    self.path.display()
                ))),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(WikiError::Io(format!(
                    "cannot inspect rollback path {}: {error}",
                    self.path.display()
                ))),
            },
            FileState::File(bytes) => {
                write_atomic(&self.path, bytes)?;
                Ok(())
            }
        }
    }
}

fn transactional<T>(
    snapshots: &[FileSnapshot],
    operation: impl FnOnce() -> Result<T, WikiError>,
) -> Result<T, WikiError> {
    match operation() {
        Ok(outcome) => Ok(outcome),
        Err(operation_error) => {
            let mut rollback_error = None;
            for snapshot in snapshots.iter().rev() {
                if let Err(error) = snapshot.restore() {
                    rollback_error.get_or_insert(error);
                }
            }
            if let Some(rollback_error) = rollback_error {
                return Err(WikiError::Io(format!(
                    "knowledge mutation failed: {operation_error}; rollback failed: {rollback_error}"
                )));
            }
            Err(operation_error)
        }
    }
}

fn write_immutable(path: &Path, bytes: &[u8]) -> Result<bool, WikiError> {
    if path.exists() {
        let existing = fs::read(path)
            .map_err(|error| WikiError::Io(format!("cannot verify {}: {error}", path.display())))?;
        if existing == bytes {
            return Ok(false);
        }
        return Err(WikiError::Conflict(format!(
            "immutable Raw revision path has different bytes: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .ok_or_else(|| WikiError::Io("Raw revision has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| WikiError::Io(format!("cannot create {}: {error}", parent.display())))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| WikiError::Io(format!("cannot create {}: {error}", path.display())))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| WikiError::Io(format!("cannot persist {}: {error}", path.display())))?;
    Ok(true)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<bool, WikiError> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(false);
    }
    let parent = path
        .parent()
        .ok_or_else(|| WikiError::Io("canonical file has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| WikiError::Io(format!("cannot create {}: {error}", parent.display())))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| WikiError::Io(format!("cannot create temporary file: {error}")))?;
    temporary
        .write_all(bytes)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| WikiError::Io(format!("cannot persist temporary file: {error}")))?;
    temporary.persist(path).map_err(|error| {
        WikiError::Io(format!(
            "cannot replace {}: {}",
            path.display(),
            error.error
        ))
    })?;
    Ok(true)
}

fn mark_stale(target: &Path) -> Result<(), WikiError> {
    let path = target.join(STALE_RELATIVE);
    let parent = path
        .parent()
        .ok_or_else(|| WikiError::Io("stale marker has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| WikiError::Io(format!("cannot create index directory: {error}")))?;
    fs::write(path, b"{\"schema_version\":1,\"stale\":true}\n")
        .map_err(|error| WikiError::Io(format!("cannot mark index stale: {error}")))
}

fn validate_target(target: &Path) -> Result<(), WikiError> {
    ensure_consumer_target(target).map_err(|error| WikiError::InvalidInput(error.to_string()))?;
    for relative in [
        Path::new(WIKI_RELATIVE),
        Path::new(RAW_RELATIVE),
        Path::new(SUPPRESSION_RELATIVE),
        Path::new(".hive/index"),
    ] {
        ensure_safe_relative(target, relative)?;
    }
    Ok(())
}

fn ensure_safe_relative(target: &Path, relative: &Path) -> Result<(), WikiError> {
    ensure_no_symlink_ancestors(target, relative)
        .map_err(|error| WikiError::Conflict(error.to_string()))
}

#[allow(clippy::needless_pass_by_value)]
fn sqlite_error(error: rusqlite::Error) -> WikiError {
    WikiError::Sqlite(error.to_string())
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

struct CapabilityKnowledgeLock {
    index: Dir,
    name: OsString,
}

impl CapabilityKnowledgeLock {
    fn acquire(root: &Dir) -> Result<Self, WikiError> {
        let (index, name) = capability_parent(root, Path::new(LOCK_RELATIVE), true)?
            .ok_or_else(|| WikiError::Io("lock directory disappeared".to_owned()))?;
        let started = Instant::now();
        loop {
            let mut options = CapOpenOptions::new();
            options
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            match index.open_with(&name, &options) {
                Ok(mut file) => {
                    if let Err(error) = file
                        .write_all(b"schema_version=1\n")
                        .and_then(|()| file.sync_all())
                    {
                        drop(file);
                        let _ = index.remove_file(&name);
                        return Err(WikiError::Io(format!("cannot persist lock: {error}")));
                    }
                    return Ok(Self { index, name });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if started.elapsed() >= LOCK_TIMEOUT {
                        return Err(WikiError::Conflict(
                            "timed out waiting for canonical knowledge integration lock".to_owned(),
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(WikiError::Io(format!(
                        "cannot acquire canonical knowledge integration lock: {error}"
                    )));
                }
            }
        }
    }
}

impl Drop for CapabilityKnowledgeLock {
    fn drop(&mut self) {
        let _ = self.index.remove_file(&self.name);
    }
}

struct KnowledgeLock {
    path: PathBuf,
    remove_parent_on_drop: bool,
}

impl KnowledgeLock {
    fn acquire(target: &Path) -> Result<Self, WikiError> {
        let path = target.join(LOCK_RELATIVE);
        let parent = path
            .parent()
            .ok_or_else(|| WikiError::Io("lock path has no parent".to_owned()))?;
        let remove_parent_on_drop = !parent.exists();
        fs::create_dir_all(parent)
            .map_err(|error| WikiError::Io(format!("cannot create lock directory: {error}")))?;
        let started = Instant::now();
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(b"schema_version=1\n")
                        .and_then(|()| file.sync_all())
                        .map_err(|error| WikiError::Io(format!("cannot persist lock: {error}")))?;
                    return Ok(Self {
                        path,
                        remove_parent_on_drop,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if started.elapsed() >= LOCK_TIMEOUT {
                        return Err(WikiError::Conflict(
                            "timed out waiting for canonical knowledge integration lock".to_owned(),
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(WikiError::Io(format!(
                        "cannot acquire canonical knowledge integration lock: {error}"
                    )));
                }
            }
        }
    }
}

impl Drop for KnowledgeLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        if self.remove_parent_on_drop {
            if let Some(parent) = self.path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_seed(root: &Path) {
        fs::create_dir_all(root.join(WIKI_RELATIVE)).unwrap();
        fs::create_dir_all(root.join(RAW_RELATIVE)).unwrap();
        fs::write(
            root.join(SUPPRESSION_RELATIVE),
            "schema_version: 1\nentries: []\n",
        )
        .unwrap();
    }

    fn draft(id: &str, body: &str) -> String {
        format!(
            "---\nschema_version: 1\nid: {id}\nkind: concept\nsummary: Summary\ntags: [example]\naliases: []\nsources: [raw:self]\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: 2026-07-24T00:00:00Z\nupdated_at: 2026-07-24T00:00:00Z\n---\n\n{body}\n"
        )
    }

    #[test]
    fn rebuild_is_logically_deterministic_and_queryable() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().canonicalize().unwrap();
        write_seed(&target);
        let source = target.join("source.md");
        let wiki = target.join("draft.md");
        fs::write(&source, "Raw source").unwrap();
        fs::write(&wiki, draft("alpha", "Searchable body")).unwrap();
        let first = ingest(&target, &source, &wiki).unwrap();
        fs::remove_file(target.join(INDEX_RELATIVE)).unwrap();
        let second = rebuild_index(&target).unwrap();
        assert_eq!(first.logical_digest, second.logical_digest);
        let hits = query(&target, Some("Searchable"), None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].sources.len(), 1);
        assert!(parse_raw_locator(&hits[0].sources[0]).is_some());
    }

    #[test]
    fn suppression_contract_has_no_body_field() {
        let entry = SuppressionEntry {
            fingerprint: format!("sha256:{}", "0".repeat(64)),
            source_locator: "wiki:alpha".to_owned(),
            reason: "obsolete".to_owned(),
            replacement: None,
            timestamp: "2026-07-24T00:00:00Z".to_owned(),
        };
        let value = serde_json::to_value(entry).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 5);
        assert!(!value.as_object().unwrap().contains_key("body"));
    }

    #[test]
    fn timestamp_rejects_impossible_dates() {
        assert!(validate_timestamp("2026-02-29T00:00:00Z").is_err());
        assert!(validate_timestamp("2024-02-29T23:59:59Z").is_ok());
        assert!(validate_timestamp("2026-07-24T24:00:00Z").is_err());
    }

    fn promotion_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let project = temporary.path().join("project");
        let user_root = temporary.path().join("user");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&user_root).unwrap();
        let project = project.canonicalize().unwrap();
        let user_root = user_root.canonicalize().unwrap();
        write_seed(&user_root);
        configure_promotion_project(&project, &user_root, "stable-example");
        (temporary, project, user_root)
    }

    fn configure_promotion_project(project: &Path, user_root: &Path, identity: &str) {
        write_seed(project);
        let source = project.join("source.md");
        let wiki = project.join("draft.md");
        fs::write(&source, "Reusable editing preference").unwrap();
        fs::write(
            &wiki,
            draft("preference-page", "Prefer narrow, reversible changes."),
        )
        .unwrap();
        ingest(project, &source, &wiki).unwrap();
        let binding = sha256_digest(user_root.to_string_lossy().as_bytes());
        fs::write(
            project.join(".hive/setup-answers.yml"),
            format!(
                "project_name: example\n\
project_identity: {identity}\n\
knowledge_exclude_paths: []\n\
root_knowledge_promotion_categories: [preference]\n\
confidential_knowledge_categories: []\n\
user_store_binding: {binding}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn promotion_dry_run_is_non_mutating_and_apply_rebuilds_root_index() {
        let (_temporary, project, user_root) = promotion_fixture();
        let before = logical_digest(
            &scan_pages(&user_root).unwrap(),
            &scan_raw(&user_root).unwrap(),
            &read_suppression(&user_root).unwrap(),
        )
        .unwrap();
        let planned = promote(
            &project,
            &user_root,
            "preference-page",
            PromotionCategory::Preference,
            PromotionMode::DryRun,
        )
        .unwrap();
        assert!(!planned.applied);
        assert_eq!(planned.logical_digest, before);
        assert!(!user_root.join(INDEX_RELATIVE).exists());

        let applied = promote(
            &project,
            &user_root,
            "preference-page",
            PromotionCategory::Preference,
            PromotionMode::Apply,
        )
        .unwrap();
        assert!(applied.applied);
        assert!(user_root.join(INDEX_RELATIVE).is_file());
        assert_eq!(
            query(&user_root, Some("reversible"), None, 10)
                .unwrap()
                .len(),
            1
        );
        let page =
            fs::read_to_string(user_root.join(format!("{WIKI_RELATIVE}/{}.md", applied.page_id)))
                .unwrap();
        assert!(!page.contains("stable-example"));
        assert!(!page.contains(project.to_string_lossy().as_ref()));
    }

    #[test]
    fn confidential_promotion_is_blocked_without_root_mutation() {
        let (_temporary, project, user_root) = promotion_fixture();
        let policy = fs::read_to_string(project.join(".hive/setup-answers.yml")).unwrap();
        fs::write(
            project.join(".hive/setup-answers.yml"),
            policy.replace(
                "confidential_knowledge_categories: []",
                "confidential_knowledge_categories: [preference]",
            ),
        )
        .unwrap();
        let error = promote(
            &project,
            &user_root,
            "preference-page",
            PromotionCategory::Preference,
            PromotionMode::Apply,
        )
        .unwrap_err();
        assert!(matches!(error, WikiError::Conflict(_)));
        assert!(!user_root.join(INDEX_RELATIVE).exists());
        assert!(scan_pages(&user_root).unwrap().is_empty());
    }

    #[test]
    fn semantically_identical_projects_merge_sorted_unique_provenance() {
        let (temporary, project_a, user_root) = promotion_fixture();
        let project_b = temporary.path().join("project-b");
        fs::create_dir(&project_b).unwrap();
        let project_b = project_b.canonicalize().unwrap();
        configure_promotion_project(&project_b, &user_root, "stable-project-b");

        let first = promote(
            &project_a,
            &user_root,
            "preference-page",
            PromotionCategory::Preference,
            PromotionMode::Apply,
        )
        .unwrap();
        let second = promote(
            &project_b,
            &user_root,
            "preference-page",
            PromotionCategory::Preference,
            PromotionMode::Apply,
        )
        .unwrap();
        assert_eq!(first.page_id, second.page_id);

        let pages = scan_pages(&user_root).unwrap();
        let page = pages.get(&first.page_id).unwrap();
        assert_eq!(page.frontmatter.sources.len(), 2);
        assert!(page
            .frontmatter
            .sources
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        let raw = scan_raw(&user_root).unwrap();
        assert_eq!(raw.len(), 2);
        let pseudonyms = raw
            .keys()
            .map(|path| {
                let bytes = fs::read(user_root.join(path)).unwrap();
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["project_pseudonym"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            pseudonyms,
            BTreeSet::from([
                sha256_digest(b"stable-example"),
                sha256_digest(b"stable-project-b")
            ])
        );

        let before = fs::read(user_root.join(&page.relative_path)).unwrap();
        let repeated = promote(
            &project_b,
            &user_root,
            "preference-page",
            PromotionCategory::Preference,
            PromotionMode::Apply,
        )
        .unwrap();
        assert_eq!(repeated.page_id, first.page_id);
        assert_eq!(
            fs::read(user_root.join(&page.relative_path)).unwrap(),
            before
        );
        assert_eq!(scan_raw(&user_root).unwrap().len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn managed_ancestor_swap_after_plan_cannot_touch_foreign_tree() {
        use std::os::unix::fs::symlink;

        let (temporary, project, user_root) = promotion_fixture();
        let project_root = PinnedRoot::open(&project).unwrap();
        let user_capability = PinnedRoot::open(&user_root).unwrap();
        let mut plan = build_promotion_plan(
            &project_root,
            &user_capability,
            "preference-page",
            PromotionCategory::Preference,
        )
        .unwrap();

        let pinned_hive = user_root.join(".hive-pinned");
        fs::rename(user_root.join(".hive"), &pinned_hive).unwrap();
        let foreign_hive = temporary.path().join("foreign-hive");
        fs::create_dir_all(foreign_hive.join("knowledge/Raw/promoted-knowledge")).unwrap();
        fs::create_dir_all(foreign_hive.join("knowledge/Wiki")).unwrap();
        fs::create_dir_all(foreign_hive.join("index")).unwrap();
        let sentinel = foreign_hive.join("knowledge/Wiki/sentinel.md");
        fs::write(&sentinel, b"foreign bytes").unwrap();
        symlink(&foreign_hive, user_root.join(".hive")).unwrap();

        let result = commit_promotion_mutation(
            &user_capability,
            &mut plan.snapshots,
            &plan.raw_bytes,
            &plan.wiki_bytes,
        );
        assert!(matches!(result, Err(WikiError::Conflict(_))));
        assert_eq!(fs::read(&sentinel).unwrap(), b"foreign bytes");
        assert_eq!(
            fs::read_dir(foreign_hive.join("knowledge/Raw/promoted-knowledge"))
                .unwrap()
                .count(),
            0
        );
        assert!(!foreign_hive.join("index/hive.sqlite3").exists());
        assert!(!pinned_hive
            .join(format!("knowledge/Wiki/{}.md", plan.outcome.page_id))
            .exists());
    }
}
