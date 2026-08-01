//! Provider-neutral bilingual knowledge for the Aigent Hive source workspace.

use super::{
    fts_expression, reject_likely_credentials, sqlite_error, LintIssue, LintSeverity, WikiError,
};
use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::sha256_digest;
use rusqlite::{params, Connection, MAIN_DB};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const SOURCE_MARKER: &str = "hive-source.json";
const WIKI_ROOT: &str = "docs/facts";
const INDEX_DIRECTORY: &str = ".agents/work/source-wiki";
const INDEX_FILE: &str = "index.sqlite3";
const LOCK_FILE: &str = ".index.lock";
const INDEX_RELATIVE: &str = ".agents/work/source-wiki/index.sqlite3";
const LOCK_RELATIVE: &str = ".agents/work/source-wiki/.index.lock";
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const INDEX_MAX_BYTES: usize = 64 * 1024 * 1024;
const FACT_BODY_MAX_BYTES: usize = 800;
const LOCK_MARKER_V1: &[u8] = b"schema_version=1\n";
const LOCK_MARKER_V2: &[u8] = b"schema_version=2\n";
const LOCK_MAX_BYTES: usize = 64;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Result of linting canonical bilingual source Wiki pages.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct SourceLintOutcome {
    /// Number of valid canonical pages inspected.
    pub page_count: usize,
    /// Canonical logical digest when all page contracts are valid.
    pub logical_digest: Option<String>,
    /// Deterministically sorted findings.
    pub issues: Vec<LintIssue>,
}

/// Evidence from rebuilding the disposable source Wiki index.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct SourceIndexOutcome {
    /// Source-relative derived paths changed by the rebuild.
    pub changed_paths: Vec<String>,
    /// Number of indexed English and Korean pages.
    pub page_count: usize,
    /// Deterministic digest over canonical logical rows.
    pub logical_digest: String,
}

/// One deterministic source Wiki query result.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct SourceQueryHit {
    /// Page language (`en` or `ko`).
    pub language: String,
    /// Stable identifier shared by the English and Korean page.
    pub pair_id: String,
    /// Stable paired topic slug.
    pub topic_slug: String,
    /// Exact reciprocal relative path to the other-language page.
    pub counterpart: String,
    /// Human-readable page title.
    pub title: String,
    /// Bounded page summary.
    pub summary: String,
    /// Canonical source-relative page path.
    pub path: String,
    /// Digest of canonical page bytes.
    pub content_digest: String,
    /// Reviewed repository revision (`git:<40 lowercase hex>`).
    pub reviewed_revision: String,
    /// Sorted page tags.
    pub tags: Vec<String>,
    /// Sorted alternate lookup names.
    pub aliases: Vec<String>,
    /// Sorted content-addressed repository sources.
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMarker {
    schema_version: u32,
    kind: String,
    consumer_setup_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct SourceFrontmatter {
    schema_version: u32,
    pair_id: String,
    topic_slug: String,
    language: String,
    counterpart: String,
    title: String,
    summary: String,
    tags: Vec<String>,
    aliases: Vec<String>,
    sources: Vec<String>,
    links: Vec<String>,
    reviewed_revision: String,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
struct SourcePage {
    frontmatter: SourceFrontmatter,
    body: String,
    relative_path: String,
    content_digest: String,
}

#[derive(Debug, Serialize)]
struct LogicalPage<'a> {
    language: &'a str,
    topic_slug: &'a str,
    pair_id: &'a str,
    title: &'a str,
    summary: &'a str,
    tags: &'a [String],
    aliases: &'a [String],
    sources: &'a [String],
    links: &'a [String],
    reviewed_revision: &'a str,
    status: &'a str,
    body: &'a str,
    content_digest: &'a str,
}

struct SourceRoot {
    dir: Dir,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CapabilityFileIdentity {
    dev: u64,
    ino: u64,
}

impl CapabilityFileIdentity {
    fn from_metadata(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            dev: CapMetadataExt::dev(metadata),
            ino: CapMetadataExt::ino(metadata),
        }
    }
}

impl SourceRoot {
    fn open(path: &Path) -> Result<Self, WikiError> {
        let metadata = fs::symlink_metadata(path).map_err(|error| {
            WikiError::Io(format!(
                "cannot inspect source Wiki root {}: {error}",
                path.display()
            ))
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(WikiError::Verification(
                "source Wiki root must be a real directory".to_owned(),
            ));
        }
        let canonical_path = path.canonicalize().map_err(|error| {
            WikiError::Io(format!(
                "cannot canonicalize source Wiki root {}: {error}",
                path.display()
            ))
        })?;
        let parent = canonical_path
            .parent()
            .ok_or_else(|| WikiError::InvalidInput("source root has no parent".to_owned()))?;
        let name = canonical_path
            .file_name()
            .ok_or_else(|| WikiError::InvalidInput("source root has no name".to_owned()))?;
        let parent_dir = Dir::open_ambient_dir(parent, ambient_authority())
            .map_err(|error| WikiError::Io(format!("cannot pin source root parent: {error}")))?;
        let expected = parent_dir.symlink_metadata(name).map_err(|error| {
            WikiError::Io(format!("cannot inspect source root identity: {error}"))
        })?;
        let dir = parent_dir.open_dir_nofollow(name).map_err(|error| {
            WikiError::Verification(format!("cannot pin source root no-follow: {error}"))
        })?;
        let pinned = dir.dir_metadata().map_err(|error| {
            WikiError::Io(format!("cannot inspect pinned source root: {error}"))
        })?;
        if (CapMetadataExt::dev(&pinned), CapMetadataExt::ino(&pinned))
            != (
                CapMetadataExt::dev(&expected),
                CapMetadataExt::ino(&expected),
            )
        {
            return Err(WikiError::Conflict(
                "source root changed while its capability was pinned".to_owned(),
            ));
        }
        let root = Self { dir };
        root.validate_identity()?;
        Ok(root)
    }

    fn validate_identity(&self) -> Result<(), WikiError> {
        let marker_bytes =
            read_capability_file(&self.dir, OsStr::new(SOURCE_MARKER), SOURCE_MARKER).map_err(
                |error| {
                    WikiError::Verification(format!(
                        "source identity marker is missing, unsafe, or unreadable: {error}"
                    ))
                },
            )?;
        let marker: SourceMarker = serde_json::from_slice(&marker_bytes).map_err(|error| {
            WikiError::Verification(format!("invalid source identity marker: {error}"))
        })?;
        if marker.schema_version != 1
            || marker.kind != "aigent-hive-source-workspace"
            || marker.consumer_setup_allowed
        {
            return Err(WikiError::Verification(
                "source identity marker does not match the exact Hive source contract".to_owned(),
            ));
        }
        for language in ["en", "ko"] {
            open_directory_nofollow(
                &self.dir,
                Path::new(&format!("{WIKI_ROOT}/{language}")),
                false,
            )?;
        }
        Ok(())
    }

    fn index_dir(&self, create_missing: bool) -> Result<Dir, WikiError> {
        open_directory_nofollow(&self.dir, Path::new(INDEX_DIRECTORY), create_missing)
    }
}

/// Lint source identity, bilingual page pairs, links, citations, and index freshness.
///
/// # Errors
///
/// Returns an error when the source root or a managed path cannot be inspected
/// without following a symbolic link. Content defects are returned as lint
/// issues.
pub fn lint(root: &Path) -> Result<SourceLintOutcome, WikiError> {
    let root = SourceRoot::open(root)?;
    let scan = scan_pages_for_lint(&root)?;
    let mut issues = scan.issues;
    let page_count = scan.pages.len();
    validate_pairs_and_graph(&scan.pages, &mut issues);
    let logical_digest = if issues
        .iter()
        .any(|issue| issue.severity == LintSeverity::Error)
    {
        None
    } else {
        Some(logical_digest(&scan.pages)?)
    };
    if let Some(expected) = &logical_digest {
        if let Err(error) = load_current_index(&root, &scan.pages, expected) {
            issues.push(issue("stale-index", INDEX_RELATIVE, &error.to_string()));
        }
    }
    sort_issues(&mut issues);
    Ok(SourceLintOutcome {
        page_count,
        logical_digest,
        issues,
    })
}

/// Rebuild the disposable, deterministic bilingual source Wiki index.
///
/// # Errors
///
/// Returns an error when source identity, canonical pages, citations, graph
/// integrity, or capability-scoped index publication fails.
#[allow(clippy::too_many_lines)]
pub fn rebuild_index(root: &Path) -> Result<SourceIndexOutcome, WikiError> {
    let root = SourceRoot::open(root)?;
    let pages = scan_valid_pages(&root)?;
    let logical_digest = logical_digest(&pages)?;
    let mut connection = Connection::open_in_memory().map_err(sqlite_error)?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE pages(
               language TEXT NOT NULL,
               pair_id TEXT NOT NULL,
               topic_slug TEXT NOT NULL,
               counterpart TEXT NOT NULL,
               title TEXT NOT NULL,
               summary TEXT NOT NULL,
               path TEXT NOT NULL,
               content_hash TEXT NOT NULL,
               reviewed_revision TEXT NOT NULL,
               body TEXT NOT NULL,
               PRIMARY KEY(language, topic_slug)
             );
             CREATE VIRTUAL TABLE pages_fts USING fts5(
               language UNINDEXED, topic_slug UNINDEXED, title, summary, body, aliases, tags
             );
             CREATE TABLE tags(
               language TEXT NOT NULL,
               topic_slug TEXT NOT NULL,
               tag TEXT NOT NULL,
               PRIMARY KEY(language, topic_slug, tag)
             );
             CREATE TABLE aliases(
               language TEXT NOT NULL,
               topic_slug TEXT NOT NULL,
               alias TEXT NOT NULL,
               PRIMARY KEY(language, topic_slug, alias)
             );
             CREATE TABLE sources(
               language TEXT NOT NULL,
               topic_slug TEXT NOT NULL,
               locator TEXT NOT NULL,
               PRIMARY KEY(language, topic_slug, locator)
             );",
        )
        .map_err(sqlite_error)?;
    let transaction = connection.transaction().map_err(sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO meta(key, value) VALUES
             ('schema_version', '1'), ('logical_digest', ?1), ('page_count', ?2)",
            params![logical_digest, pages.len().to_string()],
        )
        .map_err(sqlite_error)?;
    for page in pages.values() {
        let meta = &page.frontmatter;
        transaction
            .execute(
                "INSERT INTO pages VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    meta.language,
                    meta.pair_id,
                    meta.topic_slug,
                    meta.counterpart,
                    meta.title,
                    meta.summary,
                    page.relative_path,
                    page.content_digest,
                    meta.reviewed_revision,
                    page.body
                ],
            )
            .map_err(sqlite_error)?;
        transaction
            .execute(
                "INSERT INTO pages_fts VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    meta.language,
                    meta.topic_slug,
                    meta.title,
                    meta.summary,
                    page.body,
                    meta.aliases.join(" "),
                    meta.tags.join(" ")
                ],
            )
            .map_err(sqlite_error)?;
        for tag in &meta.tags {
            transaction
                .execute(
                    "INSERT INTO tags VALUES (?1, ?2, ?3)",
                    params![meta.language, meta.topic_slug, tag],
                )
                .map_err(sqlite_error)?;
        }
        for alias in &meta.aliases {
            transaction
                .execute(
                    "INSERT INTO aliases VALUES (?1, ?2, ?3)",
                    params![meta.language, meta.topic_slug, alias],
                )
                .map_err(sqlite_error)?;
        }
        for source in &meta.sources {
            transaction
                .execute(
                    "INSERT INTO sources VALUES (?1, ?2, ?3)",
                    params![meta.language, meta.topic_slug, source],
                )
                .map_err(sqlite_error)?;
        }
    }
    transaction.commit().map_err(sqlite_error)?;
    connection
        .execute_batch("PRAGMA optimize;")
        .map_err(sqlite_error)?;
    verify_index(&connection, &logical_digest, pages.len())?;
    let serialized = connection
        .serialize(MAIN_DB)
        .map_err(sqlite_error)?
        .to_vec();
    let index_dir = root.index_dir(true)?;
    let mut lock = SourceIndexLock::acquire(&index_dir)?;
    let publication = prepare_index_publication(&index_dir, &serialized)?;
    let marker_changed = lock.migrate_marker()?;
    publish_index(&index_dir, &serialized, publication)?;
    let mut changed_paths = vec![INDEX_RELATIVE.to_owned()];
    if marker_changed {
        changed_paths.push(LOCK_RELATIVE.to_owned());
        changed_paths.sort();
    }
    Ok(SourceIndexOutcome {
        changed_paths,
        page_count: pages.len(),
        logical_digest,
    })
}

/// Query the current source Wiki index without implicitly rebuilding it.
///
/// # Errors
///
/// Returns an error for an invalid language/query/limit, missing or stale
/// derived state, unsafe paths, or read-only `SQLite` failure.
pub fn query(
    root: &Path,
    language: &str,
    text: Option<&str>,
    tag: Option<&str>,
    limit: usize,
) -> Result<Vec<SourceQueryHit>, WikiError> {
    let root = SourceRoot::open(root)?;
    validate_language(language)?;
    if text.is_none() && tag.is_none() {
        return Err(WikiError::InvalidInput(
            "source Wiki query requires text or tag".to_owned(),
        ));
    }
    if !(1..=100).contains(&limit) {
        return Err(WikiError::InvalidInput(
            "source Wiki query limit must be from 1 through 100".to_owned(),
        ));
    }
    let pages = scan_valid_pages(&root)?;
    let expected = logical_digest(&pages)?;
    let loaded = load_current_index(&root, &pages, &expected)?;
    let connection = &loaded.connection;
    let sql_limit = i64::try_from(limit)
        .map_err(|_| WikiError::InvalidInput("query limit is too large".to_owned()))?;
    let rows = if let Some(search) = text {
        let expression = fts_expression(search)?;
        let mut statement = connection
            .prepare(
                "SELECT p.language, p.pair_id, p.topic_slug, p.counterpart, p.title, p.summary,
                        p.path, p.content_hash, p.reviewed_revision
                 FROM pages_fts f
                 JOIN pages p ON p.language = f.language AND p.topic_slug = f.topic_slug
                 WHERE pages_fts MATCH ?1
                   AND p.language = ?2
                   AND (?3 IS NULL OR EXISTS (
                     SELECT 1 FROM tags t
                     WHERE t.language = p.language
                       AND t.topic_slug = p.topic_slug
                       AND t.tag = ?3
                   ))
                 ORDER BY bm25(pages_fts), p.topic_slug
                 LIMIT ?4",
            )
            .map_err(sqlite_error)?;
        collect_hits(
            connection,
            statement.query(params![expression, language, tag, sql_limit]),
        )?
    } else {
        let mut statement = connection
            .prepare(
                "SELECT p.language, p.pair_id, p.topic_slug, p.counterpart, p.title, p.summary,
                        p.path, p.content_hash, p.reviewed_revision
                 FROM pages p
                 JOIN tags t ON t.language = p.language AND t.topic_slug = p.topic_slug
                 WHERE p.language = ?1 AND t.tag = ?2
                 ORDER BY p.topic_slug
                 LIMIT ?3",
            )
            .map_err(sqlite_error)?;
        collect_hits(
            connection,
            statement.query(params![language, tag, sql_limit]),
        )?
    };
    Ok(rows)
}

struct LintScan {
    pages: BTreeMap<(String, String), SourcePage>,
    issues: Vec<LintIssue>,
}

fn scan_pages_for_lint(root: &SourceRoot) -> Result<LintScan, WikiError> {
    let mut pages = BTreeMap::new();
    let mut issues = Vec::new();
    for language in ["en", "ko"] {
        let directory_relative = format!("{WIKI_ROOT}/{language}");
        let directory = open_directory_nofollow(&root.dir, Path::new(&directory_relative), false)?;
        let mut entries = directory
            .read_dir(".")
            .map_err(|error| {
                WikiError::Io(format!(
                    "cannot scan source Wiki directory {directory_relative}: {error}"
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| WikiError::Io(format!("cannot scan source Wiki: {error}")))?;
        entries.sort_by_key(cap_std::fs::DirEntry::file_name);
        for entry in entries {
            let file_name = entry.file_name();
            let path = PathBuf::from(&file_name);
            let file_type = entry.file_type().map_err(|error| {
                WikiError::Io(format!("cannot inspect {}: {error}", path.display()))
            })?;
            if file_type.is_symlink() {
                return Err(WikiError::Verification(format!(
                    "source Wiki symlink is forbidden: {directory_relative}/{}",
                    path.display()
                )));
            }
            let relative = format!("{directory_relative}/{}", path.to_string_lossy());
            if !file_type.is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("md")
            {
                issues.push(issue(
                    "unexpected-entry",
                    &relative,
                    "canonical source Wiki directories may contain only Markdown files",
                ));
                continue;
            }
            let bytes = read_capability_file(&directory, &file_name, &relative)?;
            if let Err(error) = reject_likely_credentials(&bytes) {
                issues.push(issue(
                    "secret-candidate",
                    &relative,
                    &format!("source Wiki page contains likely sensitive material: {error}"),
                ));
                continue;
            }
            match parse_page(&bytes, &relative, language, &path) {
                Ok(page) => {
                    let key = (language.to_owned(), page.frontmatter.topic_slug.clone());
                    if pages.insert(key, page).is_some() {
                        issues.push(issue(
                            "duplicate-page",
                            &relative,
                            "duplicate language/topic page",
                        ));
                    }
                }
                Err(error) => issues.push(issue("invalid-page", &relative, &error.to_string())),
            }
        }
    }
    validate_citations(root, &pages, &mut issues);
    Ok(LintScan { pages, issues })
}

fn scan_valid_pages(
    root: &SourceRoot,
) -> Result<BTreeMap<(String, String), SourcePage>, WikiError> {
    let scan = scan_pages_for_lint(root)?;
    let mut issues = scan.issues;
    validate_pairs_and_graph(&scan.pages, &mut issues);
    sort_issues(&mut issues);
    if let Some(first) = issues.first() {
        return Err(WikiError::Verification(format!(
            "source Wiki lint failed [{}] {}: {}",
            first.code, first.locator, first.message
        )));
    }
    Ok(scan.pages)
}

fn parse_page(
    bytes: &[u8],
    locator: &str,
    expected_language: &str,
    path: &Path,
) -> Result<SourcePage, WikiError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| WikiError::InvalidInput(format!("{locator} is not UTF-8: {error}")))?;
    if text.contains("\r\n") {
        return Err(WikiError::InvalidInput(format!(
            "{locator} must use LF line endings"
        )));
    }
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| WikiError::InvalidInput(format!("{locator} is missing YAML frontmatter")))?;
    let marker = rest.find("\n---\n").ok_or_else(|| {
        WikiError::InvalidInput(format!("{locator} has no closing frontmatter marker"))
    })?;
    let frontmatter: SourceFrontmatter =
        serde_yaml::from_str(&rest[..marker]).map_err(|error| {
            WikiError::InvalidInput(format!(
                "invalid source Wiki frontmatter at {locator}: {error}"
            ))
        })?;
    let body = &rest[marker + 5..];
    validate_frontmatter(&frontmatter, expected_language)?;
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WikiError::InvalidInput(format!("invalid filename at {locator}")))?;
    if stem != frontmatter.topic_slug {
        return Err(WikiError::InvalidInput(format!(
            "filename stem must equal topic_slug at {locator}"
        )));
    }
    if body.trim().is_empty() {
        return Err(WikiError::InvalidInput(format!(
            "source Wiki body must not be empty at {locator}"
        )));
    }
    if body.len() > FACT_BODY_MAX_BYTES {
        return Err(WikiError::InvalidInput(format!(
            "source Wiki fact body exceeds {FACT_BODY_MAX_BYTES} bytes at {locator}"
        )));
    }
    let top_level_headings = body.lines().filter(|line| line.starts_with("# ")).count();
    if top_level_headings != 1 || body.lines().any(|line| line.starts_with("## ")) {
        return Err(WikiError::InvalidInput(format!(
            "source Wiki page must contain one H1 and no subsection headings at {locator}"
        )));
    }
    Ok(SourcePage {
        frontmatter,
        body: body.to_owned(),
        relative_path: locator.to_owned(),
        content_digest: sha256_digest(bytes),
    })
}

fn validate_frontmatter(
    frontmatter: &SourceFrontmatter,
    expected_language: &str,
) -> Result<(), WikiError> {
    if frontmatter.schema_version != 1 {
        return Err(WikiError::InvalidInput(
            "source Wiki schema_version must be 1".to_owned(),
        ));
    }
    validate_slug(&frontmatter.pair_id, "pair_id")?;
    validate_slug(&frontmatter.topic_slug, "topic_slug")?;
    validate_language(&frontmatter.language)?;
    if frontmatter.language != expected_language {
        return Err(WikiError::InvalidInput(format!(
            "page language must match its {expected_language} directory"
        )));
    }
    let other = if expected_language == "en" {
        "ko"
    } else {
        "en"
    };
    let expected_counterpart = format!("../{other}/{}.md", frontmatter.topic_slug);
    if frontmatter.counterpart != expected_counterpart {
        return Err(WikiError::InvalidInput(format!(
            "counterpart must be exactly {expected_counterpart}"
        )));
    }
    if frontmatter.title.trim().is_empty() || frontmatter.summary.trim().is_empty() {
        return Err(WikiError::InvalidInput(
            "source Wiki title and summary must not be empty".to_owned(),
        ));
    }
    validate_sorted_unique_slugs(&frontmatter.tags, "tags")?;
    validate_sorted_unique_strings(&frontmatter.aliases, "aliases")?;
    validate_sorted_unique_strings(&frontmatter.sources, "sources")?;
    validate_sorted_unique_slugs(&frontmatter.links, "links")?;
    if frontmatter.sources.is_empty() {
        return Err(WikiError::InvalidInput(
            "source Wiki page must cite at least one repository source".to_owned(),
        ));
    }
    for source in &frontmatter.sources {
        parse_source_locator(source)?;
    }
    let Some(revision) = frontmatter.reviewed_revision.strip_prefix("git:") else {
        return Err(WikiError::InvalidInput(
            "reviewed_revision must be git:<40 lowercase hex>".to_owned(),
        ));
    };
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WikiError::InvalidInput(
            "reviewed_revision must be git:<40 lowercase hex>".to_owned(),
        ));
    }
    if frontmatter.status != "active" {
        return Err(WikiError::InvalidInput(
            "source Wiki status must be active".to_owned(),
        ));
    }
    Ok(())
}

fn validate_pairs_and_graph(
    pages: &BTreeMap<(String, String), SourcePage>,
    issues: &mut Vec<LintIssue>,
) {
    let mut slugs_by_pair_id = BTreeMap::<&str, BTreeSet<&str>>::new();
    for page in pages.values() {
        slugs_by_pair_id
            .entry(&page.frontmatter.pair_id)
            .or_default()
            .insert(&page.frontmatter.topic_slug);
    }
    for (pair_id, topic_slugs) in slugs_by_pair_id {
        if topic_slugs.len() > 1 {
            let slugs = topic_slugs.into_iter().collect::<Vec<_>>().join(", ");
            let locator = pages
                .values()
                .find(|page| page.frontmatter.pair_id == pair_id)
                .map_or(pair_id, |page| page.relative_path.as_str());
            issues.push(issue(
                "duplicate-pair-id",
                locator,
                &format!("pair_id {pair_id} maps to multiple topic slugs: {slugs}"),
            ));
        }
    }
    let slugs: BTreeSet<&str> = pages
        .values()
        .map(|page| page.frontmatter.topic_slug.as_str())
        .collect();
    for slug in slugs {
        let en = pages.get(&("en".to_owned(), slug.to_owned()));
        let ko = pages.get(&("ko".to_owned(), slug.to_owned()));
        let Some((en, ko)) = en.zip(ko) else {
            let existing = en.or(ko).expect("slug came from at least one page");
            issues.push(issue(
                "missing-pair",
                &existing.relative_path,
                &format!("topic {slug} must have exactly one en and one ko page"),
            ));
            continue;
        };
        let left = &en.frontmatter;
        let right = &ko.frontmatter;
        let equal = left.pair_id == right.pair_id
            && left.topic_slug == right.topic_slug
            && left.tags == right.tags
            && left.sources == right.sources
            && left.links == right.links
            && left.reviewed_revision == right.reviewed_revision
            && left.status == right.status;
        if !equal {
            issues.push(issue(
                "pair-mismatch",
                &en.relative_path,
                &format!("paired metadata differs from {}", ko.relative_path),
            ));
        }
    }
    for page in pages.values() {
        for target in &page.frontmatter.links {
            if !pages.contains_key(&(page.frontmatter.language.clone(), target.clone())) {
                issues.push(issue(
                    "broken-link",
                    &page.relative_path,
                    &format!(
                        "same-language target does not exist: {}/{}",
                        page.frontmatter.language, target
                    ),
                ));
            }
        }
    }
}

fn validate_citations(
    root: &SourceRoot,
    pages: &BTreeMap<(String, String), SourcePage>,
    issues: &mut Vec<LintIssue>,
) {
    let mut checked = BTreeMap::<String, Result<(), String>>::new();
    for page in pages.values() {
        for locator in &page.frontmatter.sources {
            let result = if let Some(previous) = checked.get(locator) {
                previous.clone()
            } else {
                let value = validate_source(root, locator).map_err(|error| error.to_string());
                checked.insert(locator.clone(), value.clone());
                value
            };
            if let Err(message) = result {
                let code = if message.contains("digest mismatch") {
                    "source-digest-mismatch"
                } else {
                    "invalid-source"
                };
                issues.push(issue(
                    code,
                    &page.relative_path,
                    &format!("{locator}: {message}"),
                ));
            }
        }
    }
}

fn validate_source(root: &SourceRoot, locator: &str) -> Result<(), WikiError> {
    let (relative, expected) = parse_source_locator(locator)?;
    if is_secret_candidate_path(&relative) {
        return Err(WikiError::Verification(format!(
            "secret-candidate source path is forbidden: {}",
            relative.display()
        )));
    }
    let (parent, name) = capability_parent(&root.dir, &relative, false)?;
    let bytes = read_capability_file(&parent, &name, &relative.to_string_lossy())?;
    let actual = sha256_digest(&bytes);
    if actual.strip_prefix("sha256:") != Some(expected) {
        return Err(WikiError::Verification(format!(
            "repository source digest mismatch for {}",
            relative.display()
        )));
    }
    Ok(())
}

fn parse_source_locator(locator: &str) -> Result<(PathBuf, &str), WikiError> {
    let Some(rest) = locator.strip_prefix("repo:") else {
        return Err(WikiError::InvalidInput(
            "source locator must start with repo:".to_owned(),
        ));
    };
    let Some((path, digest)) = rest.rsplit_once("#sha256:") else {
        return Err(WikiError::InvalidInput(
            "source locator must end with #sha256:<64 lowercase hex>".to_owned(),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WikiError::InvalidInput(
            "source digest must be 64 lowercase hexadecimal characters".to_owned(),
        ));
    }
    if path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(WikiError::InvalidInput(
            "source path must use canonical forward-slash form".to_owned(),
        ));
    }
    let relative = PathBuf::from(path);
    validate_safe_relative(&relative)?;
    if relative.to_string_lossy().replace('\\', "/") != path {
        return Err(WikiError::InvalidInput(
            "source path must use canonical forward-slash form".to_owned(),
        ));
    }
    Ok((relative, digest))
}

fn validate_safe_relative(path: &Path) -> Result<(), WikiError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(WikiError::InvalidInput(format!(
            "source path must be a safe repository-relative path: {}",
            path.display()
        )));
    }
    if path.components().any(|component| {
        let Component::Normal(part) = component else {
            return false;
        };
        matches!(
            part.to_str(),
            Some(".git" | ".hive" | ".omx" | ".omc" | ".codex" | ".claude" | "omx_wiki")
        )
    }) {
        return Err(WikiError::InvalidInput(format!(
            "source path enters a forbidden namespace: {}",
            path.display()
        )));
    }
    Ok(())
}

fn is_secret_candidate_path(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(part) = component else {
            return false;
        };
        let name = part.to_string_lossy().to_ascii_lowercase();
        matches!(
            name.as_str(),
            ".env"
                | ".netrc"
                | "_netrc"
                | "credentials"
                | "credentials.json"
                | "secrets.json"
                | "id_rsa"
                | "id_ed25519"
        ) || [".pem", ".key", ".p12", ".pfx"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
    })
}

fn open_directory_nofollow(
    root: &Dir,
    relative: &Path,
    create_missing: bool,
) -> Result<Dir, WikiError> {
    validate_capability_relative(relative)?;
    let mut current = root
        .try_clone()
        .map_err(|error| WikiError::Io(format!("cannot clone source Wiki root: {error}")))?;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(WikiError::InvalidInput(
                "managed source Wiki path is unsafe".to_owned(),
            ));
        };
        match current.symlink_metadata(name) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(WikiError::Verification(format!(
                    "managed source Wiki ancestor is not a real directory: {}",
                    relative.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && create_missing => {
                if let Err(error) = current.create_dir(name) {
                    if error.kind() != io::ErrorKind::AlreadyExists {
                        return Err(WikiError::Io(format!(
                            "cannot create managed source Wiki directory {}: {error}",
                            relative.display()
                        )));
                    }
                }
            }
            Err(error) => {
                return Err(WikiError::Verification(format!(
                    "required source Wiki directory is missing or unreadable {}: {error}",
                    relative.display()
                )));
            }
        }
        current = current.open_dir_nofollow(name).map_err(|error| {
            WikiError::Verification(format!(
                "managed source Wiki path cannot be opened no-follow {}: {error}",
                relative.display()
            ))
        })?;
    }
    Ok(current)
}

fn capability_parent(
    root: &Dir,
    relative: &Path,
    create_missing: bool,
) -> Result<(Dir, OsString), WikiError> {
    validate_capability_relative(relative)?;
    let name = relative
        .file_name()
        .ok_or_else(|| WikiError::InvalidInput("managed path has no filename".to_owned()))?
        .to_os_string();
    let parent = relative
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map_or_else(
            || {
                root.try_clone()
                    .map_err(|error| WikiError::Io(format!("cannot clone source root: {error}")))
            },
            |path| open_directory_nofollow(root, path, create_missing),
        )?;
    Ok((parent, name))
}

fn validate_capability_relative(relative: &Path) -> Result<(), WikiError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(WikiError::InvalidInput(format!(
            "managed source Wiki path is unsafe: {}",
            relative.display()
        )));
    }
    Ok(())
}

fn read_capability_file(parent: &Dir, name: &OsStr, locator: &str) -> Result<Vec<u8>, WikiError> {
    let metadata = parent.symlink_metadata(name).map_err(|error| {
        WikiError::Verification(format!("cannot inspect no-follow file {locator}: {error}"))
    })?;
    if !metadata.is_file() {
        return Err(WikiError::Verification(format!(
            "source Wiki file must be a regular no-follow file: {locator}"
        )));
    }
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = parent.open_with(name, &options).map_err(|error| {
        WikiError::Verification(format!("cannot open no-follow file {locator}: {error}"))
    })?;
    let opened = file.metadata().map_err(|error| {
        WikiError::Io(format!(
            "cannot inspect opened source Wiki file {locator}: {error}"
        ))
    })?;
    if CapabilityFileIdentity::from_metadata(&metadata)
        != CapabilityFileIdentity::from_metadata(&opened)
    {
        return Err(WikiError::Conflict(format!(
            "source Wiki file changed while opening: {locator}"
        )));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| WikiError::Io(format!("cannot read {locator}: {error}")))?;
    Ok(bytes)
}

fn logical_digest(pages: &BTreeMap<(String, String), SourcePage>) -> Result<String, WikiError> {
    let logical: Vec<_> = pages
        .values()
        .map(|page| LogicalPage {
            language: &page.frontmatter.language,
            topic_slug: &page.frontmatter.topic_slug,
            pair_id: &page.frontmatter.pair_id,
            title: &page.frontmatter.title,
            summary: &page.frontmatter.summary,
            tags: &page.frontmatter.tags,
            aliases: &page.frontmatter.aliases,
            sources: &page.frontmatter.sources,
            links: &page.frontmatter.links,
            reviewed_revision: &page.frontmatter.reviewed_revision,
            status: &page.frontmatter.status,
            body: page.body.trim(),
            content_digest: &page.content_digest,
        })
        .collect();
    let bytes = serde_json::to_vec(&logical)
        .map_err(|error| WikiError::Io(format!("cannot serialize source Wiki rows: {error}")))?;
    Ok(sha256_digest(&bytes))
}

struct LoadedIndex {
    connection: Connection,
    _lock: SourceIndexReadLock,
}

fn load_current_index(
    root: &SourceRoot,
    pages: &BTreeMap<(String, String), SourcePage>,
    expected: &str,
) -> Result<LoadedIndex, WikiError> {
    let index_dir = root.index_dir(false)?;
    let lock = SourceIndexReadLock::acquire(&index_dir)?;
    let bytes = read_bounded_capability_file(
        &index_dir,
        OsStr::new(INDEX_FILE),
        INDEX_RELATIVE,
        INDEX_MAX_BYTES,
    )?;
    let mut connection = Connection::open_in_memory().map_err(sqlite_error)?;
    connection
        .deserialize_read_exact(MAIN_DB, Cursor::new(&bytes), bytes.len(), true)
        .map_err(sqlite_error)?;
    verify_index(&connection, expected, pages.len())?;
    Ok(LoadedIndex {
        connection,
        _lock: lock,
    })
}

fn verify_index(connection: &Connection, digest: &str, page_count: usize) -> Result<(), WikiError> {
    let actual: String = connection
        .query_row(
            "SELECT value FROM meta WHERE key = 'logical_digest'",
            [],
            |row| row.get(0),
        )
        .map_err(sqlite_error)?;
    let count: i64 = connection
        .query_row("SELECT count(*) FROM pages", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let fts_count: i64 = connection
        .query_row("SELECT count(*) FROM pages_fts", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    let integrity: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if actual != digest
        || usize::try_from(count).ok() != Some(page_count)
        || usize::try_from(fts_count).ok() != Some(page_count)
        || integrity != "ok"
    {
        return Err(WikiError::Verification(
            "source Wiki index is stale or corrupt".to_owned(),
        ));
    }
    Ok(())
}

fn collect_hits(
    connection: &Connection,
    rows: Result<rusqlite::Rows<'_>, rusqlite::Error>,
) -> Result<Vec<SourceQueryHit>, WikiError> {
    let mut rows = rows.map_err(sqlite_error)?;
    let mut hits = Vec::new();
    while let Some(row) = rows.next().map_err(sqlite_error)? {
        let language: String = row.get(0).map_err(sqlite_error)?;
        let topic_slug: String = row.get(2).map_err(sqlite_error)?;
        hits.push(SourceQueryHit {
            language: language.clone(),
            pair_id: row.get(1).map_err(sqlite_error)?,
            topic_slug: topic_slug.clone(),
            counterpart: row.get(3).map_err(sqlite_error)?,
            title: row.get(4).map_err(sqlite_error)?,
            summary: row.get(5).map_err(sqlite_error)?,
            path: row.get(6).map_err(sqlite_error)?,
            content_digest: row.get(7).map_err(sqlite_error)?,
            reviewed_revision: row.get(8).map_err(sqlite_error)?,
            tags: query_strings(
                connection,
                "SELECT tag FROM tags WHERE language = ?1 AND topic_slug = ?2 ORDER BY tag",
                &language,
                &topic_slug,
            )?,
            aliases: query_strings(
                connection,
                "SELECT alias FROM aliases WHERE language = ?1 AND topic_slug = ?2 ORDER BY alias",
                &language,
                &topic_slug,
            )?,
            sources: query_strings(
                connection,
                "SELECT locator FROM sources WHERE language = ?1 AND topic_slug = ?2 ORDER BY locator",
                &language,
                &topic_slug,
            )?,
        });
    }
    Ok(hits)
}

fn query_strings(
    connection: &Connection,
    sql: &str,
    language: &str,
    topic_slug: &str,
) -> Result<Vec<String>, WikiError> {
    let mut statement = connection.prepare(sql).map_err(sqlite_error)?;
    let rows = statement
        .query_map(params![language, topic_slug], |row| row.get(0))
        .map_err(sqlite_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sqlite_error)
}

fn read_bounded_capability_file(
    parent: &Dir,
    name: &OsStr,
    locator: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, WikiError> {
    let metadata = parent.symlink_metadata(name).map_err(|error| {
        WikiError::Verification(format!("cannot inspect no-follow file {locator}: {error}"))
    })?;
    if !metadata.is_file() {
        return Err(WikiError::Verification(format!(
            "source Wiki file must be a regular no-follow file: {locator}"
        )));
    }
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = parent.open_with(name, &options).map_err(|error| {
        WikiError::Verification(format!("cannot open no-follow file {locator}: {error}"))
    })?;
    let opened = file.metadata().map_err(|error| {
        WikiError::Io(format!(
            "cannot inspect opened source Wiki file {locator}: {error}"
        ))
    })?;
    if CapabilityFileIdentity::from_metadata(&metadata)
        != CapabilityFileIdentity::from_metadata(&opened)
    {
        return Err(WikiError::Conflict(format!(
            "source Wiki file changed while opening: {locator}"
        )));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(u64::try_from(max_bytes).unwrap_or(u64::MAX) + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| WikiError::Io(format!("cannot read {locator}: {error}")))?;
    if bytes.len() > max_bytes {
        return Err(WikiError::Verification(format!(
            "source Wiki file exceeds the bounded size for {locator}"
        )));
    }
    Ok(bytes)
}

struct IndexPublicationPlan {
    stale_owned: Vec<OsString>,
    expected: Option<CapabilityFileIdentity>,
    temporary: String,
    claim: String,
}

fn prepare_index_publication(
    index_dir: &Dir,
    bytes: &[u8],
) -> Result<IndexPublicationPlan, WikiError> {
    if bytes.is_empty() || bytes.len() > INDEX_MAX_BYTES {
        return Err(WikiError::Verification(
            "serialized source Wiki index has an invalid size".to_owned(),
        ));
    }
    let stale_owned = stale_owned_index_artifacts(index_dir)?;
    let expected = capability_file_identity(index_dir, INDEX_FILE)?;
    let (temporary, claim) = loop {
        let suffix = unique_suffix();
        let temporary = format!(".index.sqlite3.tmp-{suffix}");
        let claim = format!(".index.sqlite3.claim-{suffix}");
        if capability_file_identity(index_dir, &temporary)?.is_none()
            && capability_file_identity(index_dir, &claim)?.is_none()
        {
            break (temporary, claim);
        }
    };
    Ok(IndexPublicationPlan {
        stale_owned,
        expected,
        temporary,
        claim,
    })
}

fn publish_index(
    index_dir: &Dir,
    bytes: &[u8],
    publication: IndexPublicationPlan,
) -> Result<(), WikiError> {
    let IndexPublicationPlan {
        stale_owned,
        expected,
        temporary,
        claim,
    } = publication;
    write_synced_temporary(index_dir, &temporary, bytes)?;
    if let Err(error) = activate_index(index_dir, &temporary, &claim, expected) {
        let _ = index_dir.remove_file(&temporary);
        return Err(error);
    }
    for name in stale_owned {
        index_dir.remove_file(&name).map_err(|error| {
            WikiError::Io(format!(
                "cannot clean stale source Wiki index artifact {}: {error}",
                name.to_string_lossy()
            ))
        })?;
    }
    sync_source_index_directory(index_dir)
}

#[cfg(unix)]
fn sync_source_index_directory(index_dir: &Dir) -> Result<(), WikiError> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    index_dir
        .open_with(".", &options)
        .map_err(|error| WikiError::Io(format!("cannot open source index directory: {error}")))?
        .sync_all()
        .map_err(|error| WikiError::Io(format!("cannot sync source index directory: {error}")))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn sync_source_index_directory(_index_dir: &Dir) -> Result<(), WikiError> {
    Ok(())
}

fn stale_owned_index_artifacts(index_dir: &Dir) -> Result<Vec<OsString>, WikiError> {
    let mut stale = Vec::new();
    for entry in index_dir
        .read_dir(".")
        .map_err(|error| WikiError::Io(format!("cannot inspect source index directory: {error}")))?
    {
        let entry = entry.map_err(|error| {
            WikiError::Io(format!("cannot inspect source index entry: {error}"))
        })?;
        let name = entry.file_name();
        if !is_owned_index_artifact(&name) {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| {
            WikiError::Io(format!(
                "cannot inspect source index artifact {}: {error}",
                name.to_string_lossy()
            ))
        })?;
        if !file_type.is_file() {
            return Err(WikiError::Verification(format!(
                "Hive-owned source index artifact must be a regular file: {}",
                name.to_string_lossy()
            )));
        }
        stale.push(name);
    }
    stale.sort();
    Ok(stale)
}

fn is_owned_index_artifact(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    let suffix = [".index.sqlite3.claim-", ".index.sqlite3.tmp-"]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix));
    let Some((process, sequence)) = suffix.and_then(|value| value.split_once('-')) else {
        return false;
    };
    !process.is_empty()
        && !sequence.is_empty()
        && process.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
}

fn capability_file_identity(
    directory: &Dir,
    name: &str,
) -> Result<Option<CapabilityFileIdentity>, WikiError> {
    match directory.symlink_metadata(name) {
        Ok(metadata) if !metadata.is_file() => Err(WikiError::Verification(
            "source Wiki index must be a regular no-follow file".to_owned(),
        )),
        Ok(metadata) => Ok(Some(CapabilityFileIdentity::from_metadata(&metadata))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(WikiError::Io(format!(
            "cannot inspect source Wiki index: {error}"
        ))),
    }
}

fn write_synced_temporary(index_dir: &Dir, temporary: &str, bytes: &[u8]) -> Result<(), WikiError> {
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = index_dir.open_with(temporary, &options).map_err(|error| {
        WikiError::Io(format!(
            "cannot create capability-pinned source index: {error}"
        ))
    })?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = index_dir.remove_file(temporary);
        return Err(WikiError::Io(format!(
            "cannot persist capability-pinned source index: {error}"
        )));
    }
    Ok(())
}

fn activate_index(
    index_dir: &Dir,
    temporary: &str,
    claim: &str,
    expected: Option<CapabilityFileIdentity>,
) -> Result<(), WikiError> {
    let current = capability_file_identity(index_dir, INDEX_FILE)?;
    if current != expected {
        return Err(WikiError::Conflict(
            "source Wiki index changed before capability publication".to_owned(),
        ));
    }
    let had_existing = expected.is_some();
    if had_existing {
        index_dir
            .rename(INDEX_FILE, index_dir, claim)
            .map_err(|error| {
                WikiError::Io(format!("cannot claim current source Wiki index: {error}"))
            })?;
        let claimed = match index_dir.symlink_metadata(claim) {
            Ok(metadata) => metadata,
            Err(error) => {
                let rollback = index_dir.rename(claim, index_dir, INDEX_FILE);
                return match rollback {
                    Ok(()) => Err(WikiError::Conflict(format!(
                        "cannot verify claimed source Wiki index: {error}"
                    ))),
                    Err(rollback_error) => Err(WikiError::Conflict(format!(
                        "cannot verify claimed source Wiki index ({error}) and rollback failed \
                         ({rollback_error})"
                    ))),
                };
            }
        };
        if Some(CapabilityFileIdentity::from_metadata(&claimed)) != expected {
            let rollback = index_dir.rename(claim, index_dir, INDEX_FILE);
            return match rollback {
                Ok(()) => Err(WikiError::Conflict(
                    "source Wiki index changed during capability claim".to_owned(),
                )),
                Err(error) => Err(WikiError::Conflict(format!(
                    "source Wiki index claim changed and rollback failed: {error}"
                ))),
            };
        }
    }
    if let Err(activation_error) = index_dir.rename(temporary, index_dir, INDEX_FILE) {
        let rollback = if had_existing {
            index_dir.rename(claim, index_dir, INDEX_FILE)
        } else {
            Ok(())
        };
        return match rollback {
            Ok(()) => Err(WikiError::Io(format!(
                "cannot activate source Wiki index: {activation_error}"
            ))),
            Err(rollback_error) => Err(WikiError::Conflict(format!(
                "source Wiki index activation failed ({activation_error}) and rollback failed \
                 ({rollback_error})"
            ))),
        };
    }
    if had_existing {
        index_dir.remove_file(claim).map_err(|error| {
            WikiError::Io(format!(
                "cannot remove prior source Wiki index claim: {error}"
            ))
        })?;
    }
    Ok(())
}

fn unique_suffix() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

fn validate_language(language: &str) -> Result<(), WikiError> {
    if matches!(language, "en" | "ko") {
        Ok(())
    } else {
        Err(WikiError::InvalidInput(
            "source Wiki language must be en or ko".to_owned(),
        ))
    }
}

fn validate_slug(value: &str, field: &str) -> Result<(), WikiError> {
    if (1..=96).contains(&value.len())
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.ends_with('-')
        && !value.contains("--")
    {
        Ok(())
    } else {
        Err(WikiError::InvalidInput(format!(
            "{field} must be a lowercase ASCII slug"
        )))
    }
}

fn validate_sorted_unique_slugs(values: &[String], field: &str) -> Result<(), WikiError> {
    validate_sorted_unique_strings(values, field)?;
    for value in values {
        validate_slug(value, field)?;
    }
    Ok(())
}

fn validate_sorted_unique_strings(values: &[String], field: &str) -> Result<(), WikiError> {
    if values
        .windows(2)
        .any(|pair| pair[0].as_str() >= pair[1].as_str())
    {
        return Err(WikiError::InvalidInput(format!(
            "{field} must be sorted and unique"
        )));
    }
    if values
        .iter()
        .any(|value| value.is_empty() || value.trim() != value)
    {
        return Err(WikiError::InvalidInput(format!(
            "{field} entries must be nonempty and trimmed"
        )));
    }
    Ok(())
}

fn issue(code: &str, locator: &str, message: &str) -> LintIssue {
    LintIssue {
        code: code.to_owned(),
        severity: LintSeverity::Error,
        locator: locator.to_owned(),
        message: message.to_owned(),
    }
}

fn sort_issues(issues: &mut [LintIssue]) {
    issues.sort_by(|left, right| {
        (&left.code, &left.locator, &left.message).cmp(&(
            &right.code,
            &right.locator,
            &right.message,
        ))
    });
}

struct SourceIndexReadLock {
    _file: fs::File,
}

impl SourceIndexReadLock {
    fn acquire(index_dir: &Dir) -> Result<Self, WikiError> {
        let mut file = open_existing_lock_file(index_dir, false)?;
        acquire_file_lock(&file, true)?;
        let marker = read_lock_marker(&mut file)?;
        if !matches!(marker.as_slice(), [] | LOCK_MARKER_V1 | LOCK_MARKER_V2) {
            return Err(WikiError::Verification(
                "source Wiki index lock has unknown contents".to_owned(),
            ));
        }
        Ok(Self { _file: file })
    }
}

struct SourceIndexLock {
    file: fs::File,
    marker_needs_migration: bool,
}

impl SourceIndexLock {
    fn acquire(index_dir: &Dir) -> Result<Self, WikiError> {
        let mut file = {
            let mut create = CapOpenOptions::new();
            create
                .read(true)
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            match index_dir.open_with(LOCK_FILE, &create) {
                Ok(file) => file.into_std(),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    open_existing_lock_file(index_dir, true)?
                }
                Err(error) => {
                    return Err(WikiError::Io(format!(
                        "cannot create source Wiki index lock: {error}"
                    )));
                }
            }
        };
        acquire_file_lock(&file, false)?;
        let bytes = read_lock_marker(&mut file)?;
        let marker_needs_migration = if bytes.is_empty() || bytes == LOCK_MARKER_V1 {
            true
        } else if bytes == LOCK_MARKER_V2 {
            false
        } else {
            return Err(WikiError::Verification(
                "source Wiki index lock has unknown contents".to_owned(),
            ));
        };
        Ok(Self {
            file,
            marker_needs_migration,
        })
    }

    fn migrate_marker(&mut self) -> Result<bool, WikiError> {
        if !self.marker_needs_migration {
            return Ok(false);
        }
        let bytes = read_lock_marker(&mut self.file)?;
        if bytes == LOCK_MARKER_V2 {
            self.marker_needs_migration = false;
            return Ok(false);
        }
        if !bytes.is_empty() && bytes != LOCK_MARKER_V1 {
            return Err(WikiError::Verification(
                "source Wiki index lock has unknown contents".to_owned(),
            ));
        }
        self.file
            .set_len(0)
            .and_then(|()| self.file.seek(SeekFrom::Start(0)).map(|_| ()))
            .and_then(|()| self.file.write_all(LOCK_MARKER_V2))
            .and_then(|()| self.file.sync_all())
            .map_err(|error| {
                WikiError::Io(format!("cannot migrate source Wiki index lock: {error}"))
            })?;
        self.marker_needs_migration = false;
        Ok(true)
    }
}

fn open_existing_lock_file(index_dir: &Dir, write: bool) -> Result<fs::File, WikiError> {
    let metadata = index_dir.symlink_metadata(LOCK_FILE).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            WikiError::Verification("source Wiki index lock is missing".to_owned())
        } else {
            WikiError::Io(format!("cannot inspect source Wiki index lock: {error}"))
        }
    })?;
    if !metadata.is_file() {
        return Err(WikiError::Verification(
            "source Wiki index lock must be a regular no-follow file".to_owned(),
        ));
    }
    let mut options = CapOpenOptions::new();
    options.read(true).write(write).follow(FollowSymlinks::No);
    let file = index_dir.open_with(LOCK_FILE, &options).map_err(|error| {
        WikiError::Verification(format!(
            "cannot open source Wiki index lock no-follow: {error}"
        ))
    })?;
    let opened = file.metadata().map_err(|error| {
        WikiError::Io(format!(
            "cannot inspect opened source Wiki index lock: {error}"
        ))
    })?;
    if CapabilityFileIdentity::from_metadata(&metadata)
        != CapabilityFileIdentity::from_metadata(&opened)
    {
        return Err(WikiError::Conflict(
            "source Wiki index lock changed while opening".to_owned(),
        ));
    }
    Ok(file.into_std())
}

fn acquire_file_lock(file: &fs::File, shared: bool) -> Result<(), WikiError> {
    let started = Instant::now();
    loop {
        let result = if shared {
            file.try_lock_shared()
        } else {
            file.try_lock()
        };
        match result {
            Ok(()) => return Ok(()),
            Err(fs::TryLockError::WouldBlock) => {
                if started.elapsed() >= LOCK_TIMEOUT {
                    return Err(WikiError::Conflict(
                        "timed out waiting for source Wiki index lock".to_owned(),
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(fs::TryLockError::Error(error)) => {
                return Err(WikiError::Io(format!(
                    "cannot acquire source Wiki index lock: {error}"
                )));
            }
        }
    }
}

fn read_lock_marker(file: &mut fs::File) -> Result<Vec<u8>, WikiError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|error| WikiError::Io(format!("cannot seek source Wiki lock: {error}")))?;
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(u64::try_from(LOCK_MAX_BYTES).unwrap_or(u64::MAX) + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| WikiError::Io(format!("cannot read source Wiki lock: {error}")))?;
    if bytes.len() > LOCK_MAX_BYTES {
        return Err(WikiError::Verification(
            "source Wiki index lock has unknown contents".to_owned(),
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::sync::{Arc, Barrier};
    use tempfile::TempDir;

    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

    fn fixture() -> TempDir {
        let temp = TempDir::new().expect("temp");
        fs::create_dir_all(temp.path().join("docs/facts/en")).expect("en");
        fs::create_dir_all(temp.path().join("docs/facts/ko")).expect("ko");
        fs::create_dir_all(temp.path().join("docs")).expect("docs");
        fs::write(
            temp.path().join(SOURCE_MARKER),
            br#"{"schema_version":1,"kind":"aigent-hive-source-workspace","consumer_setup_allowed":false}"#,
        )
        .expect("marker");
        fs::write(temp.path().join("docs/source.md"), b"canonical source\n").expect("source");
        temp
    }

    fn source_locator(root: &Path) -> String {
        let bytes = fs::read(root.join("docs/source.md")).expect("source");
        format!(
            "repo:docs/source.md#sha256:{}",
            sha256_digest(&bytes).trim_start_matches("sha256:")
        )
    }

    fn page(language: &str, slug: &str, links: &[&str], source: &str, body: &str) -> String {
        let other = if language == "en" { "ko" } else { "en" };
        let title = if language == "en" {
            "Source Architecture"
        } else {
            "소스 아키텍처"
        };
        let summary = if language == "en" {
            "Provider-neutral source boundaries."
        } else {
            "공급자 중립 소스 경계."
        };
        let links = if links.is_empty() {
            "[]\n".to_owned()
        } else {
            links.iter().fold("\n".to_owned(), |mut output, link| {
                use std::fmt::Write as _;
                writeln!(output, "- {link}").expect("write to String");
                output
            })
        };
        format!(
            "---\n\
             schema_version: 1\n\
             pair_id: {slug}\n\
             topic_slug: {slug}\n\
             language: {language}\n\
             counterpart: ../{other}/{slug}.md\n\
             title: {title}\n\
             summary: {summary}\n\
             tags:\n\
             - architecture\n\
             aliases: []\n\
             sources:\n\
             - {source}\n\
             links: {links}\
             reviewed_revision: git:{REVISION}\n\
             status: active\n\
             ---\n\
             # {title}\n\
             \n\
             {body}\n"
        )
    }

    fn write_pair(root: &Path, slug: &str, links: &[&str]) {
        let source = source_locator(root);
        fs::write(
            root.join(format!("docs/facts/en/{slug}.md")),
            page("en", slug, links, &source, "English source knowledge."),
        )
        .expect("en page");
        fs::write(
            root.join(format!("docs/facts/ko/{slug}.md")),
            page("ko", slug, links, &source, "한국어 소스 지식."),
        )
        .expect("ko page");
    }

    #[test]
    fn happy_pair_index_and_query_are_deterministic() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let first = rebuild_index(temp.path()).expect("index");
        let first_bytes = fs::read(temp.path().join(INDEX_RELATIVE)).expect("first index bytes");
        let second = rebuild_index(temp.path()).expect("repeat index");
        let second_bytes = fs::read(temp.path().join(INDEX_RELATIVE)).expect("second index bytes");
        assert_eq!(first.logical_digest, second.logical_digest);
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first.page_count, 2);
        assert!(!temp.path().join(format!("{INDEX_RELATIVE}-wal")).exists());
        assert!(!temp.path().join(format!("{INDEX_RELATIVE}-shm")).exists());
        let linted = lint(temp.path()).expect("lint");
        assert!(linted.issues.is_empty());
        let hits = query(temp.path(), "en", Some("boundaries"), None, 10).expect("query");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].topic_slug, "architecture");
        assert_eq!(hits[0].pair_id, "architecture");
        assert_eq!(hits[0].counterpart, "../ko/architecture.md");
        assert_eq!(hits[0].reviewed_revision, format!("git:{REVISION}"));
        let tagged = query(temp.path(), "ko", None, Some("architecture"), 10).expect("tag query");
        assert_eq!(tagged.len(), 1);
    }

    #[test]
    fn lint_reports_missing_pair_and_pair_mismatch() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        fs::remove_file(temp.path().join("docs/facts/ko/architecture.md")).expect("remove pair");
        let outcome = lint(temp.path()).expect("lint");
        assert!(outcome
            .issues
            .iter()
            .any(|issue| issue.code == "missing-pair"));

        write_pair(temp.path(), "architecture", &[]);
        let path = temp.path().join("docs/facts/ko/architecture.md");
        let changed = fs::read_to_string(&path)
            .expect("read")
            .replace("- architecture\n", "- architecture\n- mismatch\n");
        fs::write(path, changed).expect("write mismatch");
        let outcome = lint(temp.path()).expect("lint");
        assert!(outcome
            .issues
            .iter()
            .any(|issue| issue.code == "pair-mismatch"));
    }

    #[test]
    fn lint_rejects_duplicate_pair_id_across_topic_slugs() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        write_pair(temp.path(), "security", &[]);
        for language in ["en", "ko"] {
            let path = temp
                .path()
                .join(format!("docs/facts/{language}/security.md"));
            let changed = fs::read_to_string(&path)
                .expect("read")
                .replace("pair_id: security", "pair_id: architecture");
            fs::write(path, changed).expect("duplicate pair id");
        }
        let outcome = lint(temp.path()).expect("lint");
        assert!(outcome
            .issues
            .iter()
            .any(|issue| issue.code == "duplicate-pair-id"));
    }

    #[test]
    fn lint_rejects_non_atomic_fact_bodies() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let path = temp.path().join("docs/facts/en/architecture.md");
        let with_subsection = fs::read_to_string(&path)
            .expect("read")
            .replace("English source knowledge.", "## Extra\n\nUnrelated fact.");
        fs::write(&path, with_subsection).expect("write subsection");
        let outcome = lint(temp.path()).expect("lint subsection");
        assert!(outcome.issues.iter().any(|issue| {
            issue.code == "invalid-page" && issue.message.contains("no subsection headings")
        }));

        write_pair(temp.path(), "architecture", &[]);
        let oversized = fs::read_to_string(&path).expect("read").replace(
            "English source knowledge.",
            &"x".repeat(FACT_BODY_MAX_BYTES),
        );
        fs::write(path, oversized).expect("write oversized body");
        let outcome = lint(temp.path()).expect("lint oversized");
        assert!(outcome.issues.iter().any(|issue| {
            issue.code == "invalid-page" && issue.message.contains("fact body exceeds")
        }));
    }

    #[test]
    fn lint_reports_broken_link_and_source_digest() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &["missing"]);
        fs::write(temp.path().join("docs/source.md"), b"changed\n").expect("mutate source");
        let outcome = lint(temp.path()).expect("lint");
        assert!(outcome
            .issues
            .iter()
            .any(|issue| issue.code == "broken-link"));
        assert!(outcome
            .issues
            .iter()
            .any(|issue| issue.code == "source-digest-mismatch"));
    }

    #[test]
    fn source_locator_rejects_noncanonical_components_before_normalization() {
        let digest = "0".repeat(64);
        for path in ["docs//source.md", "docs/./source.md", "docs/../source.md"] {
            assert!(parse_source_locator(&format!("repo:{path}#sha256:{digest}")).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_refused_but_cited_source_content_is_not_ingested() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let external = temp.path().join("external.md");
        fs::write(&external, b"outside\n").expect("external");
        fs::remove_file(temp.path().join("docs/source.md")).expect("remove source");
        symlink(&external, temp.path().join("docs/source.md")).expect("symlink");
        let outcome = lint(temp.path()).expect("lint");
        assert!(outcome
            .issues
            .iter()
            .any(|issue| issue.code == "invalid-source"));

        fs::remove_file(temp.path().join("docs/source.md")).expect("unlink");
        fs::write(
            temp.path().join("docs/source.md"),
            b"token=abcdefghijklmnopqrstuvwxyz\n",
        )
        .expect("credential-bearing cited source");
        let digest = sha256_digest(b"token=abcdefghijklmnopqrstuvwxyz\n");
        let digest = digest.strip_prefix("sha256:").expect("digest prefix");
        for language in ["en", "ko"] {
            let path = temp
                .path()
                .join(WIKI_ROOT)
                .join(language)
                .join("architecture.md");
            let text = fs::read_to_string(&path).expect("read fact");
            fs::write(&path, text.replace(&"0".repeat(64), digest)).expect("update citation");
        }
        let outcome = lint(temp.path()).expect("lint");
        assert!(outcome.issues.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn llm_wiki_ancestor_symlink_is_refused() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let external = temp.path().join("external-wiki");
        fs::rename(temp.path().join(WIKI_ROOT), &external).expect("move wiki");
        symlink(&external, temp.path().join(WIKI_ROOT)).expect("wiki ancestor symlink");

        assert!(lint(temp.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn agents_ancestor_symlink_cannot_receive_index() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let external = temp.path().join("external-agents");
        fs::create_dir(&external).expect("external agents");
        symlink(&external, temp.path().join(".agents")).expect("agents ancestor symlink");

        assert!(rebuild_index(temp.path()).is_err());
        assert!(!external.join("work/source-wiki/index.sqlite3").exists());
    }

    #[test]
    fn legacy_crash_lock_is_migrated_in_place_to_v2_marker() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        fs::create_dir_all(temp.path().join(INDEX_DIRECTORY)).expect("index directory");
        fs::write(temp.path().join(LOCK_RELATIVE), LOCK_MARKER_V1).expect("legacy stale lock");

        let outcome = rebuild_index(temp.path()).expect("recover and rebuild");
        assert_eq!(
            fs::read(temp.path().join(LOCK_RELATIVE)).expect("lock marker"),
            LOCK_MARKER_V2
        );
        assert_eq!(
            outcome.changed_paths,
            vec![LOCK_RELATIVE.to_owned(), INDEX_RELATIVE.to_owned()]
        );
        let repeated = rebuild_index(temp.path()).expect("repeat rebuild");
        assert_eq!(repeated.changed_paths, vec![INDEX_RELATIVE.to_owned()]);
    }

    #[test]
    fn lock_drop_releases_ownership_for_reacquire() {
        let temp = fixture();
        let root = SourceRoot::open(temp.path()).expect("root");
        let index_dir = root.index_dir(true).expect("index dir");
        let mut first = SourceIndexLock::acquire(&index_dir).expect("first lock");
        assert!(first.marker_needs_migration);
        assert!(first.migrate_marker().expect("migrate marker"));
        drop(first);
        let second = SourceIndexLock::acquire(&index_dir).expect("reacquire");
        assert!(!second.marker_needs_migration);
    }

    #[test]
    fn invalid_pages_do_not_migrate_or_publish_derived_state() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        fs::remove_file(temp.path().join("docs/facts/ko/architecture.md")).expect("missing pair");
        fs::create_dir_all(temp.path().join(INDEX_DIRECTORY)).expect("index directory");
        fs::write(temp.path().join(LOCK_RELATIVE), LOCK_MARKER_V1).expect("legacy marker");

        assert!(rebuild_index(temp.path()).is_err());
        assert_eq!(
            fs::read(temp.path().join(LOCK_RELATIVE)).expect("unchanged marker"),
            LOCK_MARKER_V1
        );
        assert!(!temp.path().join(INDEX_RELATIVE).exists());
    }

    #[test]
    fn shared_reader_waits_for_exclusive_writer_and_then_acquires() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        rebuild_index(temp.path()).expect("index");
        let root = SourceRoot::open(temp.path()).expect("root");
        let index_dir = root.index_dir(false).expect("index dir");
        let writer = SourceIndexLock::acquire(&index_dir).expect("writer");
        let reader_dir = index_dir.try_clone().expect("reader dir");
        let (sender, receiver) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            sender
                .send(SourceIndexReadLock::acquire(&reader_dir).map(|_| ()))
                .expect("send reader result");
        });

        assert!(receiver.recv_timeout(Duration::from_millis(50)).is_err());
        drop(writer);
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("reader completed")
            .expect("reader acquired");
        worker.join().expect("reader worker");
    }

    #[test]
    fn unknown_and_sqlite_lock_bytes_fail_closed_without_mutation() {
        for bytes in [
            b"unknown-lock\n".as_slice(),
            b"SQLite format 3\0unexpected".as_slice(),
        ] {
            let temp = fixture();
            write_pair(temp.path(), "architecture", &[]);
            fs::create_dir_all(temp.path().join(INDEX_DIRECTORY)).expect("index directory");
            fs::write(temp.path().join(LOCK_RELATIVE), bytes).expect("hostile lock");

            assert!(rebuild_index(temp.path()).is_err());
            assert_eq!(
                fs::read(temp.path().join(LOCK_RELATIVE)).expect("preserved lock"),
                bytes
            );
        }
    }

    #[test]
    fn concurrent_rebuilds_serialize_and_produce_the_same_index() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        fs::create_dir_all(temp.path().join(INDEX_DIRECTORY)).expect("index directory");
        fs::write(temp.path().join(LOCK_RELATIVE), LOCK_MARKER_V1).expect("legacy lock");
        let root = Arc::new(temp.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(4));
        let mut workers = Vec::new();
        for _ in 0..4 {
            let root = Arc::clone(&root);
            let barrier = Arc::clone(&barrier);
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                rebuild_index(&root)
            }));
        }
        let outcomes = workers
            .into_iter()
            .map(|worker| worker.join().expect("worker").expect("rebuild"))
            .collect::<Vec<_>>();
        assert!(outcomes
            .windows(2)
            .all(|pair| pair[0].logical_digest == pair[1].logical_digest));
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| outcome
                    .changed_paths
                    .iter()
                    .any(|path| path == LOCK_RELATIVE))
                .count(),
            1
        );
        assert_eq!(
            fs::read(temp.path().join(LOCK_RELATIVE)).expect("migrated lock"),
            LOCK_MARKER_V2
        );
        assert_eq!(
            query(temp.path(), "en", Some("boundaries"), None, 10)
                .expect("query")
                .len(),
            1
        );
    }

    #[test]
    fn deserialized_query_connection_is_independent_of_later_path_changes() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        rebuild_index(temp.path()).expect("index");
        let root = SourceRoot::open(temp.path()).expect("root");
        let pages = scan_valid_pages(&root).expect("pages");
        let digest = logical_digest(&pages).expect("digest");
        let loaded = load_current_index(&root, &pages, &digest).expect("loaded connection");
        fs::write(
            temp.path().join(INDEX_RELATIVE),
            b"later hostile replacement",
        )
        .expect("replace ambient index");

        let count: i64 = loaded
            .connection
            .query_row("SELECT count(*) FROM pages", [], |row| row.get(0))
            .expect("same deserialized connection");
        assert_eq!(count, 2);
    }

    #[test]
    fn orphan_claim_is_repaired_without_touching_foreign_entries() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        rebuild_index(temp.path()).expect("index");
        let directory = temp.path().join(INDEX_DIRECTORY);
        let orphan = directory.join(".index.sqlite3.claim-123-1");
        fs::rename(directory.join(INDEX_FILE), &orphan).expect("orphan prior index");
        let foreign = directory.join(".index.sqlite3.claim-foreign");
        fs::write(&foreign, b"foreign").expect("foreign entry");

        rebuild_index(temp.path()).expect("repair orphan");

        assert!(directory.join(INDEX_FILE).is_file());
        assert!(!orphan.exists());
        assert_eq!(fs::read(foreign).expect("foreign preserved"), b"foreign");
        assert_eq!(
            query(temp.path(), "en", Some("boundaries"), None, 10)
                .expect("query repaired index")
                .len(),
            1
        );
    }

    #[cfg(unix)]
    #[test]
    fn v1_marker_and_owned_artifact_symlink_fail_without_any_mutation() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        rebuild_index(temp.path()).expect("index");
        fs::write(temp.path().join(LOCK_RELATIVE), LOCK_MARKER_V1).expect("legacy marker");
        let index_before = fs::read(temp.path().join(INDEX_RELATIVE)).expect("index before");
        let external = temp.path().join("external-artifact");
        fs::write(&external, b"external").expect("external");
        let artifact = temp
            .path()
            .join(INDEX_DIRECTORY)
            .join(".index.sqlite3.tmp-123-1");
        symlink(&external, &artifact).expect("artifact symlink");

        assert!(rebuild_index(temp.path()).is_err());
        assert_eq!(
            fs::read(temp.path().join(LOCK_RELATIVE)).expect("marker preserved"),
            LOCK_MARKER_V1
        );
        assert_eq!(
            fs::read(temp.path().join(INDEX_RELATIVE)).expect("index preserved"),
            index_before
        );
        assert_eq!(fs::read(external).expect("external preserved"), b"external");
        assert!(artifact.is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn pinned_root_survives_ambient_root_retarget() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let root = SourceRoot::open(temp.path()).expect("pinned root");
        let original = temp.path().to_path_buf();
        let moved = original.with_extension("pinned-source");
        fs::rename(&original, &moved).expect("move pinned root");
        fs::create_dir(&original).expect("replacement root");
        fs::write(original.join("foreign"), b"foreign").expect("foreign sentinel");

        let pages = scan_valid_pages(&root).expect("scan pinned root");
        assert_eq!(pages.len(), 2);
        assert!(!original.join(INDEX_RELATIVE).exists());

        drop(root);
        fs::remove_dir_all(&original).expect("remove replacement");
        fs::rename(moved, original).expect("restore temp root");
    }

    #[cfg(unix)]
    #[test]
    fn capability_publication_stays_with_pinned_index_directory() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        rebuild_index(temp.path()).expect("index");
        let root = SourceRoot::open(temp.path()).expect("root");
        let index_dir = root.index_dir(false).expect("pinned index directory");
        let bytes = read_bounded_capability_file(
            &index_dir,
            OsStr::new(INDEX_FILE),
            INDEX_RELATIVE,
            INDEX_MAX_BYTES,
        )
        .expect("serialized index");
        let original = temp.path().join(INDEX_DIRECTORY);
        let moved = temp.path().join(".agents/work/source-wiki-pinned");
        fs::rename(&original, &moved).expect("retarget index directory");
        fs::create_dir(&original).expect("replacement index directory");

        let publication =
            prepare_index_publication(&index_dir, &bytes).expect("publication preflight");
        publish_index(&index_dir, &bytes, publication).expect("capability publication");

        assert!(moved.join(INDEX_FILE).is_file());
        assert!(!original.join(INDEX_FILE).exists());
    }

    #[test]
    fn missing_stale_and_corrupt_indexes_are_detected() {
        let temp = fixture();
        write_pair(temp.path(), "architecture", &[]);
        let missing = lint(temp.path()).expect("lint missing");
        assert!(missing
            .issues
            .iter()
            .any(|issue| issue.code == "stale-index"));

        rebuild_index(temp.path()).expect("index");
        let page_path = temp.path().join("docs/facts/en/architecture.md");
        let changed = fs::read_to_string(&page_path).expect("read").replace(
            "English source knowledge.",
            "Changed English source knowledge.",
        );
        fs::write(page_path, changed).expect("change");
        assert!(query(temp.path(), "en", Some("Changed"), None, 10).is_err());

        rebuild_index(temp.path()).expect("rebuild");
        fs::write(temp.path().join(INDEX_RELATIVE), b"not sqlite").expect("corrupt");
        assert!(query(temp.path(), "en", Some("source"), None, 10).is_err());
    }
}
