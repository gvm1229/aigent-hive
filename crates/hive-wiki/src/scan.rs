//! Bounded, read-only project inventory and agent-reviewed claim validation.

use crate::WikiError;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Scan contract version.
pub const SCAN_SCHEMA_VERSION: u32 = 1;

/// Hard limits for a single project inventory.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanLimits {
    /// Maximum discovered entries accepted from the caller.
    pub max_discovered_files: usize,
    /// Maximum files whose content may be included.
    pub max_included_files: usize,
    /// Maximum bytes read from one file.
    pub max_file_bytes: usize,
    /// Maximum aggregate included bytes.
    pub max_total_bytes: usize,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_discovered_files: 10_000,
            max_included_files: 2_000,
            max_file_bytes: 2 * 1024 * 1024,
            max_total_bytes: 32 * 1024 * 1024,
        }
    }
}

impl ScanLimits {
    fn validate(self) -> Result<Self, WikiError> {
        if self.max_discovered_files == 0
            || self.max_included_files == 0
            || self.max_file_bytes == 0
            || self.max_total_bytes == 0
            || self.max_included_files > self.max_discovered_files
            || self.max_file_bytes > self.max_total_bytes
        {
            return Err(WikiError::InvalidInput(
                "scan limits must be positive and internally bounded".to_owned(),
            ));
        }
        Ok(self)
    }
}

/// Discovery mode chosen by the caller.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ScanRootKind {
    /// Git paths discovered through fixed-argument `git ls-files` calls.
    Git,
    /// Non-Git paths restricted to the narrow project-evidence allowlist.
    NonGit,
}

/// File kind observed without following a filesystem link.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ScanFileKind {
    /// Ordinary file.
    Regular,
    /// Symbolic link or reparse-like link.
    Symlink,
    /// Directory, socket, device, or another unsupported type.
    Special,
}

/// One file discovered by a capability-confined caller.
#[derive(Debug, Clone, Copy)]
pub struct ScanInputFile<'a> {
    /// Project-relative slash-separated UTF-8 path.
    pub relative_path: &'a str,
    /// Bounded bytes read without following links. Empty for non-regular entries.
    pub bytes: &'a [u8],
    /// No-follow metadata length, even when content was not read due to a budget.
    pub observed_byte_len: usize,
    /// Whether Git reported the file as tracked.
    pub tracked: bool,
    /// No-follow file kind.
    pub file_kind: ScanFileKind,
}

/// Scan behavior selected by the explicit command.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanOptions {
    /// Discovery mode.
    pub root_kind: ScanRootKind,
    /// Include nonignored untracked Git files supplied by the caller.
    pub include_untracked: bool,
    /// Resource limits.
    pub limits: ScanLimits,
}

/// Stable scan decision for one discovered path.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ScanDecision {
    /// Safe bounded text retained as potential evidence.
    Included,
    /// Metadata-only skip; content is never emitted.
    Skipped,
}

/// Bounded receipt entry. File content is intentionally unrepresentable.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanEntry {
    /// Normalized project-relative locator.
    pub relative_path: String,
    /// Content digest for included files only.
    pub content_digest: Option<String>,
    /// Observed byte length.
    pub byte_len: usize,
    /// Git tracked status.
    pub tracked: bool,
    /// Include or skip decision.
    pub decision: ScanDecision,
    /// Stable classification or bounded skip reason.
    pub reason: String,
}

/// Deterministic scan inventory without raw content.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanInventory {
    /// Contract version.
    pub schema_version: u32,
    /// Discovery mode.
    pub root_kind: ScanRootKind,
    /// Whether untracked input was eligible.
    pub include_untracked: bool,
    /// Included file count.
    pub included_count: usize,
    /// Skipped file count.
    pub skipped_count: usize,
    /// Aggregate bytes represented by included evidence.
    pub included_bytes: usize,
    /// Tracked-first, path-sorted receipts.
    pub entries: Vec<ScanEntry>,
    /// SHA-256 over the logical receipt rows.
    pub inventory_digest: String,
    /// Exact prior digest match when supplied.
    pub unchanged: bool,
}

/// Build a deterministic, content-free inventory from caller-confined files.
///
/// The caller owns Git invocation and no-follow filesystem reads. This function
/// performs no I/O and never infers a fact from file content.
///
/// # Errors
///
/// Returns an error for invalid paths, duplicate input, exceeded budgets, or unsafe content.
pub fn build_inventory(
    files: &[ScanInputFile<'_>],
    options: ScanOptions,
    prior_inventory_digest: Option<&str>,
) -> Result<ScanInventory, WikiError> {
    let limits = options.limits.validate()?;
    if files.len() > limits.max_discovered_files {
        return Err(WikiError::InvalidInput(format!(
            "scan discovered {} files, exceeding the {} file budget",
            files.len(),
            limits.max_discovered_files
        )));
    }
    let mut normalized = Vec::with_capacity(files.len());
    let mut seen = BTreeSet::new();
    for file in files {
        let path = validate_relative_path(file.relative_path)?;
        if !seen.insert(path.clone()) {
            return Err(WikiError::InvalidInput(format!(
                "scan discovery contains a duplicate path: {path}"
            )));
        }
        normalized.push((path, file));
    }
    normalized.sort_by(|(left_path, left), (right_path, right)| {
        right
            .tracked
            .cmp(&left.tracked)
            .then_with(|| left_path.cmp(right_path))
    });

    let mut entries = Vec::with_capacity(normalized.len());
    let mut included_count = 0_usize;
    let mut included_bytes = 0_usize;
    for (path, file) in normalized {
        let skip = classify_skip(&path, file, options, limits, included_count, included_bytes)?;
        if let Some(reason) = skip {
            entries.push(ScanEntry {
                relative_path: path,
                content_digest: None,
                byte_len: file.observed_byte_len,
                tracked: file.tracked,
                decision: ScanDecision::Skipped,
                reason: reason.to_owned(),
            });
            continue;
        }
        let classification = classify_evidence(&path);
        included_count += 1;
        included_bytes = included_bytes.saturating_add(file.observed_byte_len);
        entries.push(ScanEntry {
            relative_path: path,
            content_digest: Some(sha256_digest(file.bytes)),
            byte_len: file.observed_byte_len,
            tracked: file.tracked,
            decision: ScanDecision::Included,
            reason: classification.to_owned(),
        });
    }
    let skipped_count = entries.len().saturating_sub(included_count);
    let logical = serde_json::to_vec(&(
        SCAN_SCHEMA_VERSION,
        options.root_kind,
        options.include_untracked,
        &entries,
    ))
    .map_err(|error| WikiError::Io(format!("cannot encode scan inventory: {error}")))?;
    let inventory_digest = sha256_digest(&logical);
    Ok(ScanInventory {
        schema_version: SCAN_SCHEMA_VERSION,
        root_kind: options.root_kind,
        include_untracked: options.include_untracked,
        included_count,
        skipped_count,
        included_bytes,
        entries,
        unchanged: prior_inventory_digest == Some(inventory_digest.as_str()),
        inventory_digest,
    })
}

fn classify_skip(
    path: &str,
    file: &ScanInputFile<'_>,
    options: ScanOptions,
    limits: ScanLimits,
    included_count: usize,
    included_bytes: usize,
) -> Result<Option<&'static str>, WikiError> {
    if file.file_kind != ScanFileKind::Regular {
        return Ok(Some(match file.file_kind {
            ScanFileKind::Symlink => "symlink",
            ScanFileKind::Special => "special-file",
            ScanFileKind::Regular => unreachable!(),
        }));
    }
    if !file.tracked && !options.include_untracked {
        return Ok(Some("untracked-not-requested"));
    }
    if contains_excluded_component(path) {
        return Ok(Some("generated-vendor-runtime-path"));
    }
    if is_secret_candidate_path(path) {
        return Ok(Some("secret-candidate-path"));
    }
    if is_license_path(path) {
        return Ok(Some("license-text"));
    }
    if options.root_kind == ScanRootKind::NonGit && !non_git_allowlisted(path) {
        return Ok(Some("non-git-not-allowlisted"));
    }
    if !supported_text_path(path) {
        return Ok(Some("unsupported-file-type"));
    }
    if file.observed_byte_len > limits.max_file_bytes {
        return Ok(Some("file-byte-budget"));
    }
    if included_count >= limits.max_included_files {
        return Ok(Some("file-count-budget"));
    }
    if included_bytes.saturating_add(file.observed_byte_len) > limits.max_total_bytes {
        return Ok(Some("total-byte-budget"));
    }
    if file.bytes.len() != file.observed_byte_len {
        return Err(WikiError::Verification(format!(
            "scan file bytes differ from no-follow metadata length: {path}"
        )));
    }
    if file.bytes.contains(&0) || std::str::from_utf8(file.bytes).is_err() {
        return Ok(Some("binary-or-non-utf8"));
    }
    if super::reject_likely_credentials(file.bytes).is_err() {
        return Ok(Some("secret-candidate-content"));
    }
    Ok(None)
}

fn validate_relative_path(path: &str) -> Result<String, WikiError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with("//")
        || path.contains('\\')
        || path.contains(':')
        || path.chars().any(char::is_control)
    {
        return Err(WikiError::InvalidInput(
            "scan paths must be normalized project-relative UTF-8 paths".to_owned(),
        ));
    }
    let parts = path.split('/').collect::<Vec<_>>();
    if parts
        .iter()
        .any(|part| part.is_empty() || matches!(*part, "." | ".."))
    {
        return Err(WikiError::InvalidInput(format!(
            "scan path contains a non-normalized segment: {path}"
        )));
    }
    Ok(parts.join("/"))
}

fn contains_excluded_component(path: &str) -> bool {
    const EXCLUDED: [&str; 16] = [
        ".git",
        ".hive",
        ".next",
        ".venv",
        "__pycache__",
        "build",
        "coverage",
        "dist",
        "generated",
        "node_modules",
        "out",
        "target",
        "tmp",
        "vendor",
        "venv",
        ".cache",
    ];
    path.split('/')
        .any(|part| EXCLUDED.contains(&part.to_ascii_lowercase().as_str()))
}

fn is_secret_candidate_path(path: &str) -> bool {
    path.split('/').any(|component| {
        let component = component.to_ascii_lowercase();
        let secret_extension = Path::new(&component)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| matches!(extension, "key" | "pem" | "p12" | "pfx"));
        component == ".env"
            || component.starts_with(".env.")
            || matches!(
                component.as_str(),
                ".npmrc"
                    | ".pypirc"
                    | "auth.json"
                    | "credentials"
                    | "credentials.json"
                    | "id_dsa"
                    | "id_ed25519"
                    | "id_rsa"
                    | "secrets.json"
            )
            || secret_extension
    })
}

fn is_license_path(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    name.starts_with("license")
        || name.starts_with("copying")
        || name.starts_with("third-party")
        || name.starts_with("third_party")
}

fn supported_text_path(path: &str) -> bool {
    const EXTENSIONS: [&str; 30] = [
        "bat", "c", "cc", "cmd", "cpp", "css", "go", "h", "hpp", "html", "java", "js", "json",
        "jsonc", "jsx", "kt", "kts", "md", "mdx", "ps1", "py", "rs", "scss", "sh", "sql", "toml",
        "ts", "tsx", "txt", "yaml",
    ];
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    if matches!(
        name.as_str(),
        "agents.md"
            | "cargo.toml"
            | "cargo.lock"
            | "dockerfile"
            | "gemfile"
            | "gemfile.lock"
            | "go.mod"
            | "makefile"
            | "package.json"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "poetry.lock"
            | "pyproject.toml"
            | "readme"
            | "yarn.lock"
    ) || name.starts_with("readme.")
    {
        return true;
    }
    name.rsplit_once('.')
        .is_some_and(|(_, extension)| EXTENSIONS.contains(&extension) || extension == "yml")
}

fn non_git_allowlisted(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    lowered == "agents.md"
        || lowered.starts_with("readme")
        || lowered.starts_with("docs/")
        || lowered.starts_with(".agents/directives/")
        || matches!(
            lowered.as_str(),
            "cargo.toml"
                | "go.mod"
                | "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "poetry.lock"
                | "pyproject.toml"
                | "gemfile"
                | "gemfile.lock"
                | "cargo.lock"
                | "dockerfile"
                | "makefile"
                | "yarn.lock"
        )
}

fn classify_evidence(path: &str) -> &'static str {
    let lowered = path.to_ascii_lowercase();
    let extension = Path::new(&lowered)
        .extension()
        .and_then(|value| value.to_str());
    if lowered.starts_with("docs/") || matches!(extension, Some("md" | "mdx")) {
        "project-document"
    } else if lowered.contains("test") || lowered.contains("spec") {
        "test-evidence"
    } else if matches!(
        lowered.rsplit('/').next(),
        Some(
            "cargo.toml"
                | "cargo.lock"
                | "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "poetry.lock"
                | "pyproject.toml"
                | "go.mod"
                | "gemfile"
                | "gemfile.lock"
                | "yarn.lock"
        )
    ) {
        "dependency-manifest"
    } else if matches!(
        lowered.rsplit_once('.').map(|(_, extension)| extension),
        Some("toml" | "yaml" | "yml" | "json" | "jsonc")
    ) {
        "configuration"
    } else {
        "source-evidence"
    }
}

/// Difference between two metadata-only inventories.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanDelta {
    /// Newly included locators.
    pub added: Vec<String>,
    /// Existing locators whose digest changed.
    pub changed: Vec<String>,
    /// Previously included locators no longer present.
    pub removed: Vec<String>,
    /// Same-digest path moves, represented as `(old, new)`.
    pub renamed: Vec<(String, String)>,
}

/// Compare included evidence rows. Promoted facts are intentionally outside
/// this delta and therefore cannot be deleted by a project rescan.
#[must_use]
pub fn diff_inventory(previous: &ScanInventory, current: &ScanInventory) -> ScanDelta {
    let previous_rows = included_rows(previous);
    let current_rows = included_rows(current);
    let mut added = current_rows
        .keys()
        .filter(|path| !previous_rows.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let mut removed = previous_rows
        .keys()
        .filter(|path| !current_rows.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let changed = current_rows
        .iter()
        .filter_map(|(path, digest)| {
            previous_rows
                .get(path)
                .filter(|previous| *previous != digest)
                .map(|_| path.clone())
        })
        .collect::<Vec<_>>();
    let mut renamed = Vec::new();
    let mut consumed_added = BTreeSet::new();
    let mut consumed_removed = BTreeSet::new();
    for old in &removed {
        let Some(old_digest) = previous_rows.get(old) else {
            continue;
        };
        if let Some(new) = added.iter().find(|new| {
            !consumed_added.contains(*new) && current_rows.get(*new) == Some(old_digest)
        }) {
            renamed.push((old.clone(), new.clone()));
            consumed_removed.insert(old.clone());
            consumed_added.insert(new.clone());
        }
    }
    added.retain(|path| !consumed_added.contains(path));
    removed.retain(|path| !consumed_removed.contains(path));
    ScanDelta {
        added,
        changed,
        removed,
        renamed,
    }
}

fn included_rows(inventory: &ScanInventory) -> BTreeMap<String, String> {
    inventory
        .entries
        .iter()
        .filter_map(|entry| {
            entry
                .content_digest
                .as_ref()
                .map(|digest| (entry.relative_path.clone(), digest.clone()))
        })
        .collect()
}

/// Agent-reviewed assertion kind. The scanner never creates these by itself.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimKind {
    /// Bounded project identity and purpose.
    ProjectProfile,
    /// Reviewed product or architecture decision.
    Decision,
    /// Reusable engineering convention.
    Convention,
    /// Explicit user preference, never inferred from implementation alone.
    Preference,
    /// Repeatable project workflow.
    Workflow,
    /// Dependency and its observed version or revision.
    DependencyEvidence,
    /// Verified project outcome.
    Outcome,
    /// Open, evidence-bound question.
    Question,
}

/// Semantic evidence class selected during agent review.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimEvidenceKind {
    /// Human-readable project document.
    Document,
    /// Source or configuration implementation.
    Implementation,
    /// Test definition or result stored as canonical evidence.
    Test,
    /// Build definition or result stored as canonical evidence.
    Build,
    /// Explicit user-authored intention document.
    UserIntent,
}

/// Digest-bound source for a reviewed claim.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClaimEvidence {
    /// Included scan locator.
    pub locator: String,
    /// Exact included content digest.
    pub content_digest: String,
    /// Reviewed semantic evidence class.
    pub kind: ClaimEvidenceKind,
}

/// Review state retained with a canonical scan claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ScanReviewStatus {
    /// An agent reviewed the bounded statement and its exact evidence.
    AgentReviewed,
    /// A later complete rescan no longer contained the reviewed claim.
    SourceInvalidated,
}

/// Explicit global-promotion lifecycle for a reviewed scan claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ScanPromotionStatus {
    /// The reviewed claim was not proposed for user-root promotion.
    NotCandidate,
    /// Promotion requires a separate explicit review and approval.
    PendingReview,
    /// A separately approved user-root fact was created from this claim.
    Promoted,
    /// A separate promotion review rejected the candidate.
    Rejected,
}

/// Durable typed review metadata retained by canonical Markdown and the RAG projection.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanClaimMetadata {
    /// Stable review-local identifier from the scan review file.
    pub review_id: String,
    /// Observed dependency or product version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Source revision or immutable release identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    /// Bounded applicability conditions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicability: Option<String>,
    /// Exact typed evidence bindings.
    pub evidence: Vec<ClaimEvidence>,
    /// Current review state.
    pub review_status: ScanReviewStatus,
    /// Whether the review proposed later user-root promotion.
    pub global_promotion_candidate: bool,
    /// Current explicit promotion state.
    pub promotion_status: ScanPromotionStatus,
}

impl ScanClaimMetadata {
    /// Construct the canonical initial metadata for one validated reviewed claim.
    #[must_use]
    pub fn from_reviewed(claim: &ReviewedClaim) -> Self {
        Self {
            review_id: claim.claim_id.clone(),
            version: claim.version.clone(),
            source_revision: claim.revision.clone(),
            applicability: claim.applicability.clone(),
            evidence: claim.evidence.clone(),
            review_status: ScanReviewStatus::AgentReviewed,
            global_promotion_candidate: claim.global_promotion_candidate,
            promotion_status: if claim.global_promotion_candidate {
                ScanPromotionStatus::PendingReview
            } else {
                ScanPromotionStatus::NotCandidate
            },
        }
    }

    /// Compare the reviewed payload while preserving a later promotion decision.
    #[must_use]
    pub fn same_review_payload(&self, other: &Self) -> bool {
        self.review_id == other.review_id
            && self.version == other.version
            && self.source_revision == other.source_revision
            && self.applicability == other.applicability
            && self.evidence == other.evidence
            && self.global_promotion_candidate == other.global_promotion_candidate
    }
}

/// Candidate supplied by an agent after inspecting the inventory.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReviewedClaim {
    /// Contract version.
    pub schema_version: u32,
    /// Stable caller-generated ASCII identifier.
    pub claim_id: String,
    /// Assertion kind.
    pub kind: ClaimKind,
    /// Bounded atomic statement.
    pub statement: String,
    /// Observed dependency or product version.
    pub version: Option<String>,
    /// Source revision or immutable release identifier.
    pub revision: Option<String>,
    /// Bounded conditions under which a reusable claim applies.
    pub applicability: Option<String>,
    /// Exact scan sources.
    pub evidence: Vec<ClaimEvidence>,
    /// Must be true; automatic extraction is forbidden.
    pub agent_reviewed: bool,
    /// Candidate for a later global promotion plan, never automatic apply.
    pub global_promotion_candidate: bool,
}

/// Validated claims remain split by local collection and later promotion review.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ValidatedClaims {
    /// Source inventory binding.
    pub inventory_digest: String,
    /// Claims accepted for the directory collection.
    pub collection_claims: Vec<ReviewedClaim>,
    /// Subset requiring a separate consolidated promotion approval.
    pub promotion_candidates: Vec<String>,
}

/// Validate reviewed claims against an exact inventory without writing them.
///
/// # Errors
///
/// Returns an error when a claim is unreviewed, malformed, unsupported, duplicated,
/// or not bound to exact included evidence with the required semantic proof.
pub fn validate_claims(
    inventory: &ScanInventory,
    claims: &[ReviewedClaim],
) -> Result<ValidatedClaims, WikiError> {
    let rows = included_rows(inventory);
    let mut ids = BTreeSet::new();
    let mut accepted = Vec::with_capacity(claims.len());
    let mut promotion_candidates = Vec::new();
    for claim in claims {
        validate_claim(claim, &rows, &mut ids)?;
        if claim.global_promotion_candidate {
            promotion_candidates.push(claim.claim_id.clone());
        }
        accepted.push(claim.clone());
    }
    Ok(ValidatedClaims {
        inventory_digest: inventory.inventory_digest.clone(),
        collection_claims: accepted,
        promotion_candidates,
    })
}

fn validate_claim(
    claim: &ReviewedClaim,
    rows: &BTreeMap<String, String>,
    ids: &mut BTreeSet<String>,
) -> Result<(), WikiError> {
    if claim.schema_version != SCAN_SCHEMA_VERSION {
        return Err(WikiError::InvalidInput(
            "reviewed claim schema_version must be 1".to_owned(),
        ));
    }
    if !valid_claim_id(&claim.claim_id) || !ids.insert(claim.claim_id.clone()) {
        return Err(WikiError::InvalidInput(
            "reviewed claim IDs must be unique lowercase ASCII identifiers".to_owned(),
        ));
    }
    let statement = claim.statement.trim();
    if !claim.agent_reviewed || statement.is_empty() || statement.len() > 800 {
        return Err(WikiError::InvalidInput(
            "claims require agent review and a bounded atomic statement".to_owned(),
        ));
    }
    if claim.evidence.is_empty() || claim.evidence.len() > 16 {
        return Err(WikiError::InvalidInput(
            "claims require one through sixteen digest-bound evidence entries".to_owned(),
        ));
    }
    let mut locators = BTreeSet::new();
    for evidence in &claim.evidence {
        let locator = validate_relative_path(&evidence.locator)?;
        if !locators.insert(locator.clone()) || rows.get(&locator) != Some(&evidence.content_digest)
        {
            return Err(WikiError::Verification(format!(
                "claim evidence does not match the included scan inventory: {locator}"
            )));
        }
    }
    let has = |kind| claim.evidence.iter().any(|evidence| evidence.kind == kind);
    match claim.kind {
        ClaimKind::DependencyEvidence if claim.version.is_none() && claim.revision.is_none() => {
            return Err(WikiError::InvalidInput(
                "dependency claims require an observed version or revision".to_owned(),
            ));
        }
        ClaimKind::Convention
            if claim.global_promotion_candidate
                && ((claim.version.is_none() && claim.revision.is_none())
                    || claim
                        .applicability
                        .as_deref()
                        .is_none_or(|value| value.trim().is_empty() || value.len() > 400)
                    || !(has(ClaimEvidenceKind::Test) || has(ClaimEvidenceKind::Build))
                    || !(has(ClaimEvidenceKind::Document)
                        || has(ClaimEvidenceKind::UserIntent))) =>
        {
            return Err(WikiError::InvalidInput(
                "reusable convention candidates require version/revision, applicability, intention, and test/build evidence"
                    .to_owned(),
            ));
        }
        ClaimKind::Outcome
            if (claim.version.is_none() && claim.revision.is_none())
                || !(has(ClaimEvidenceKind::Test) || has(ClaimEvidenceKind::Build)) =>
        {
            return Err(WikiError::InvalidInput(
                "outcome claims require version/revision and test or build evidence".to_owned(),
            ));
        }
        ClaimKind::Preference if !has(ClaimEvidenceKind::UserIntent) => {
            return Err(WikiError::InvalidInput(
                "preference claims require explicit user-intent evidence".to_owned(),
            ));
        }
        ClaimKind::ProjectProfile if !has(ClaimEvidenceKind::Document) => {
            return Err(WikiError::InvalidInput(
                "project-profile claims require document evidence".to_owned(),
            ));
        }
        _ => {}
    }
    Ok(())
}

fn valid_claim_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(root_kind: ScanRootKind) -> ScanOptions {
        ScanOptions {
            root_kind,
            include_untracked: false,
            limits: ScanLimits::default(),
        }
    }

    #[test]
    fn inventory_is_tracked_first_deterministic_and_content_free() {
        let files = [
            ScanInputFile {
                relative_path: "src/lib.rs",
                bytes: b"pub fn stable() {}\n",
                observed_byte_len: b"pub fn stable() {}\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "README.md",
                bytes: b"# Purpose\n",
                observed_byte_len: b"# Purpose\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "notes.md",
                bytes: b"untracked\n",
                observed_byte_len: b"untracked\n".len(),
                tracked: false,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let first = build_inventory(&files, options(ScanRootKind::Git), None).expect("inventory");
        let mut reversed = files;
        reversed.reverse();
        let second =
            build_inventory(&reversed, options(ScanRootKind::Git), None).expect("inventory");
        assert_eq!(first, second);
        assert_eq!(first.entries[0].relative_path, "README.md");
        assert_eq!(first.entries[2].reason, "untracked-not-requested");
        let encoded = serde_json::to_string(&first).expect("receipt");
        assert!(!encoded.contains("pub fn stable"));
        assert!(!encoded.contains("# Purpose"));
    }

    #[test]
    fn secrets_links_binaries_vendor_and_budgets_are_skipped() {
        let secret = b"api_key = 'abcdefghijklmnopqrstuvwxyz0123456789'\n";
        let binary = b"ok\0bad";
        let files = [
            ScanInputFile {
                relative_path: ".env",
                bytes: secret,
                observed_byte_len: secret.len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "docs/secret.md",
                bytes: secret,
                observed_byte_len: secret.len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "docs/link.md",
                bytes: b"",
                observed_byte_len: 0,
                tracked: true,
                file_kind: ScanFileKind::Symlink,
            },
            ScanInputFile {
                relative_path: "assets/image.txt",
                bytes: binary,
                observed_byte_len: binary.len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "vendor/README.md",
                bytes: b"foreign\n",
                observed_byte_len: b"foreign\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let inventory =
            build_inventory(&files, options(ScanRootKind::Git), None).expect("inventory");
        assert_eq!(inventory.included_count, 0);
        let reasons = inventory
            .entries
            .iter()
            .map(|entry| entry.reason.as_str())
            .collect::<BTreeSet<_>>();
        assert!(reasons.contains("secret-candidate-path"));
        assert!(reasons.contains("secret-candidate-content"));
        assert!(reasons.contains("symlink"));
        assert!(reasons.contains("binary-or-non-utf8"));
        assert!(reasons.contains("generated-vendor-runtime-path"));
    }

    #[test]
    fn hostile_exclusion_matrix_covers_generated_license_and_credentials() {
        let credential = b"api_key = 'abcdefghijklmnopqrstuvwxyz0123456789'\n";
        let files = [
            ScanInputFile {
                relative_path: "generated/client.rs",
                bytes: b"generated code\n",
                observed_byte_len: b"generated code\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "vendor/library/README.md",
                bytes: b"third-party code\n",
                observed_byte_len: b"third-party code\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "LICENSE-THIRD-PARTY.md",
                bytes: b"license text\n",
                observed_byte_len: b"license text\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: ".env.production",
                bytes: credential,
                observed_byte_len: credential.len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "docs/credential.md",
                bytes: credential,
                observed_byte_len: credential.len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "assets/binary.txt",
                bytes: b"text\0binary",
                observed_byte_len: b"text\0binary".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let inventory =
            build_inventory(&files, options(ScanRootKind::Git), None).expect("hostile inventory");
        assert_eq!(inventory.included_count, 0);
        let reasons = inventory
            .entries
            .iter()
            .map(|entry| (entry.relative_path.as_str(), entry.reason.as_str()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            reasons["generated/client.rs"],
            "generated-vendor-runtime-path"
        );
        assert_eq!(
            reasons["vendor/library/README.md"],
            "generated-vendor-runtime-path"
        );
        assert_eq!(reasons["LICENSE-THIRD-PARTY.md"], "license-text");
        assert_eq!(reasons[".env.production"], "secret-candidate-path");
        assert_eq!(reasons["docs/credential.md"], "secret-candidate-content");
        assert_eq!(reasons["assets/binary.txt"], "binary-or-non-utf8");
        assert!(inventory
            .entries
            .iter()
            .all(|entry| entry.content_digest.is_none()));
    }

    #[test]
    fn hostile_file_count_and_byte_budgets_fail_closed() {
        let size_limited = build_inventory(
            &[ScanInputFile {
                relative_path: "docs/oversized.md",
                bytes: b"12345",
                observed_byte_len: 5,
                tracked: true,
                file_kind: ScanFileKind::Regular,
            }],
            ScanOptions {
                root_kind: ScanRootKind::Git,
                include_untracked: false,
                limits: ScanLimits {
                    max_discovered_files: 2,
                    max_included_files: 2,
                    max_file_bytes: 4,
                    max_total_bytes: 8,
                },
            },
            None,
        )
        .expect("file-size-limited inventory");
        assert_eq!(size_limited.entries[0].reason, "file-byte-budget");

        let two_files = [
            ScanInputFile {
                relative_path: "docs/a.md",
                bytes: b"abc",
                observed_byte_len: 3,
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "docs/b.md",
                bytes: b"def",
                observed_byte_len: 3,
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let count_limited = build_inventory(
            &two_files,
            ScanOptions {
                root_kind: ScanRootKind::Git,
                include_untracked: false,
                limits: ScanLimits {
                    max_discovered_files: 2,
                    max_included_files: 1,
                    max_file_bytes: 6,
                    max_total_bytes: 6,
                },
            },
            None,
        )
        .expect("included-file-count-limited inventory");
        assert_eq!(count_limited.included_count, 1);
        assert_eq!(count_limited.entries[1].reason, "file-count-budget");

        let total_limited = build_inventory(
            &two_files,
            ScanOptions {
                root_kind: ScanRootKind::Git,
                include_untracked: false,
                limits: ScanLimits {
                    max_discovered_files: 2,
                    max_included_files: 2,
                    max_file_bytes: 3,
                    max_total_bytes: 5,
                },
            },
            None,
        )
        .expect("total-byte-limited inventory");
        assert_eq!(total_limited.included_count, 1);
        assert_eq!(total_limited.entries[1].reason, "total-byte-budget");

        let discovered_error = build_inventory(
            &two_files,
            ScanOptions {
                root_kind: ScanRootKind::Git,
                include_untracked: false,
                limits: ScanLimits {
                    max_discovered_files: 1,
                    max_included_files: 1,
                    max_file_bytes: 3,
                    max_total_bytes: 3,
                },
            },
            None,
        )
        .expect_err("discovered-file budget must fail before classification");
        assert!(matches!(discovered_error, WikiError::InvalidInput(_)));
    }

    #[test]
    fn external_and_non_normalized_paths_are_rejected_before_inventory() {
        for path in [
            "../outside.md",
            "/outside.md",
            "C:/outside.md",
            "docs/../outside.md",
        ] {
            let error = build_inventory(
                &[ScanInputFile {
                    relative_path: path,
                    bytes: b"outside\n",
                    observed_byte_len: b"outside\n".len(),
                    tracked: true,
                    file_kind: ScanFileKind::Regular,
                }],
                options(ScanRootKind::Git),
                None,
            )
            .expect_err("external path must fail closed");
            assert!(matches!(error, WikiError::InvalidInput(_)), "{path}");
        }
    }

    #[test]
    fn non_git_mode_uses_narrow_allowlist() {
        let files = [
            ScanInputFile {
                relative_path: "README.md",
                bytes: b"purpose\n",
                observed_byte_len: b"purpose\n".len(),
                tracked: false,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "src/lib.rs",
                bytes: b"implementation\n",
                observed_byte_len: b"implementation\n".len(),
                tracked: false,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let mut scan_options = options(ScanRootKind::NonGit);
        scan_options.include_untracked = true;
        let inventory = build_inventory(&files, scan_options, None).expect("inventory");
        assert_eq!(inventory.included_count, 1);
        assert_eq!(inventory.entries[1].reason, "non-git-not-allowlisted");
    }

    #[test]
    fn inventory_diff_detects_rename_change_add_and_delete() {
        let before_files = [
            ScanInputFile {
                relative_path: "docs/old.md",
                bytes: b"same\n",
                observed_byte_len: b"same\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "README.md",
                bytes: b"before\n",
                observed_byte_len: b"before\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "docs/removed.md",
                bytes: b"removed\n",
                observed_byte_len: b"removed\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let after_files = [
            ScanInputFile {
                relative_path: "docs/new.md",
                bytes: b"same\n",
                observed_byte_len: b"same\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "README.md",
                bytes: b"after\n",
                observed_byte_len: b"after\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "docs/added.md",
                bytes: b"added\n",
                observed_byte_len: b"added\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let before =
            build_inventory(&before_files, options(ScanRootKind::Git), None).expect("before");
        let after = build_inventory(&after_files, options(ScanRootKind::Git), None).expect("after");
        let delta = diff_inventory(&before, &after);
        assert_eq!(
            delta.renamed,
            vec![("docs/old.md".to_owned(), "docs/new.md".to_owned())]
        );
        assert_eq!(delta.added, vec!["docs/added.md"]);
        assert_eq!(delta.changed, vec!["README.md"]);
        assert_eq!(delta.removed, vec!["docs/removed.md"]);
    }

    #[test]
    fn reviewed_claims_require_exact_evidence_and_semantic_gates() {
        let files = [
            ScanInputFile {
                relative_path: "package.json",
                bytes: b"{\"dependencies\":{\"next\":\"16.0.0\"}}\n",
                observed_byte_len: b"{\"dependencies\":{\"next\":\"16.0.0\"}}\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
            ScanInputFile {
                relative_path: "tests/app.test.ts",
                bytes: b"test('works', () => {})\n",
                observed_byte_len: b"test('works', () => {})\n".len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            },
        ];
        let inventory =
            build_inventory(&files, options(ScanRootKind::Git), None).expect("inventory");
        let digest = |path: &str| {
            inventory
                .entries
                .iter()
                .find(|entry| entry.relative_path == path)
                .and_then(|entry| entry.content_digest.clone())
                .expect("digest")
        };
        let claims = vec![ReviewedClaim {
            schema_version: 1,
            claim_id: "next-16-verified".to_owned(),
            kind: ClaimKind::Outcome,
            statement: "Next.js 16 is used by the project and covered by a test.".to_owned(),
            version: Some("16.0.0".to_owned()),
            revision: None,
            applicability: Some("This project configuration and test suite.".to_owned()),
            evidence: vec![
                ClaimEvidence {
                    locator: "package.json".to_owned(),
                    content_digest: digest("package.json"),
                    kind: ClaimEvidenceKind::Implementation,
                },
                ClaimEvidence {
                    locator: "tests/app.test.ts".to_owned(),
                    content_digest: digest("tests/app.test.ts"),
                    kind: ClaimEvidenceKind::Test,
                },
            ],
            agent_reviewed: true,
            global_promotion_candidate: true,
        }];
        let validated = validate_claims(&inventory, &claims).expect("claims");
        assert_eq!(validated.promotion_candidates, vec!["next-16-verified"]);

        let mut tampered = claims;
        tampered[0].evidence[0].content_digest = sha256_digest(b"tampered");
        assert!(matches!(
            validate_claims(&inventory, &tampered),
            Err(WikiError::Verification(_))
        ));
    }

    #[test]
    fn preference_cannot_be_inferred_from_implementation() {
        let files = [ScanInputFile {
            relative_path: "AGENTS.md",
            bytes: b"Use focused commits.\n",
            observed_byte_len: b"Use focused commits.\n".len(),
            tracked: true,
            file_kind: ScanFileKind::Regular,
        }];
        let inventory =
            build_inventory(&files, options(ScanRootKind::Git), None).expect("inventory");
        let digest = inventory.entries[0].content_digest.clone().expect("digest");
        let claim = ReviewedClaim {
            schema_version: 1,
            claim_id: "focused-commits".to_owned(),
            kind: ClaimKind::Preference,
            statement: "The user prefers focused commits.".to_owned(),
            version: None,
            revision: None,
            applicability: None,
            evidence: vec![ClaimEvidence {
                locator: "AGENTS.md".to_owned(),
                content_digest: digest,
                kind: ClaimEvidenceKind::Implementation,
            }],
            agent_reviewed: true,
            global_promotion_candidate: true,
        };
        assert!(matches!(
            validate_claims(&inventory, &[claim]),
            Err(WikiError::InvalidInput(_))
        ));
    }
}
