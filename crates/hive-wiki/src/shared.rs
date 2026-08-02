//! User-root shared knowledge index built from canonical Markdown sources.

use crate::collection::USER_ROOT_COLLECTION_ID;
use crate::rag::{build_rag_index, RagVisibility, WikiPageQueryRequest};
use crate::store::RagStore;
#[cfg(test)]
use crate::{parse_page_bytes, reject_likely_credentials};
use crate::{QueryHit, WikiError};
#[cfg(test)]
use cap_fs_ext::{DirExt, MetadataExt as CapMetadataExt};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::{ensure_no_symlink_ancestors, sha256_digest};
#[cfg(test)]
use rusqlite::types::Value;
#[cfg(test)]
use rusqlite::{params, Connection, MAIN_DB};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
#[cfg(test)]
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Read;
#[cfg(test)]
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

/// Canonical user-root project registration ledger.
pub const PROJECT_REGISTRY_RELATIVE: &str = ".hive/config/projects.yml";
/// The single user-root disposable RAG database.
pub const SHARED_INDEX_RELATIVE: &str = ".hive/index/hive.sqlite3";
/// Complete user-root RAG operational state removed when Global Wiki is disabled.
///
/// The list includes the canonical trust receipt as well as disposable projection files.
pub const SHARED_DERIVED_RELATIVES: [&str; 8] = [
    SHARED_INDEX_RELATIVE,
    ".hive/index/hive.sqlite3-wal",
    ".hive/index/hive.sqlite3-shm",
    ".hive/index/hive.sqlite3-journal",
    crate::store::RAG_MANIFEST_RELATIVE,
    crate::store::RAG_TRUST_RELATIVE,
    crate::store::RAG_DIRTY_RELATIVE,
    ".hive/index/.stale",
];

#[cfg(test)]
const WIKI_RELATIVE: &str = ".hive/knowledge/Wiki";
const MAX_REGISTRY_BYTES: usize = 1024 * 1024;
#[cfg(test)]
const MAX_PAGE_BYTES: usize = 2 * 1024 * 1024;
#[cfg(test)]
const MAX_SHARED_INDEX_BYTES: usize = 64 * 1024 * 1024;
#[cfg(test)]
static SHARED_INDEX_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Strict canonical registration ledger for shared index sources.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProjectRegistry {
    /// Contract version.
    pub schema_version: u32,
    /// Registered project Wiki sources.
    pub projects: Vec<RegisteredProject>,
}

/// One project Wiki source registered under the user root.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegisteredProject {
    /// Stable project identifier used in indexed provenance.
    pub id: String,
    /// Absolute, no-symlink project root.
    pub root: PathBuf,
    /// Whether the project participates in shared rebuilds.
    pub enabled: bool,
    /// Page language provenance for this project Wiki.
    pub language: KnowledgeLanguage,
    /// Cross-project retrieval boundary for all pages in this project Wiki.
    pub visibility: KnowledgeVisibility,
}

/// Resolve a project or user root to one portable canonical absolute path.
///
/// Windows canonicalization returns a verbatim `\\?\` path. Hive removes only
/// that transport prefix before persisting or comparing the resolved path.
///
/// # Errors
///
/// Returns an I/O error when the path cannot be canonicalized.
pub fn canonical_root(path: &Path) -> Result<PathBuf, WikiError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| WikiError::Io(format!("cannot canonicalize root: {error}")))?;
    Ok(platform_canonical_root(&portable_canonical_path(
        &canonical,
    )))
}

/// Normalize only macOS's operating-system-owned `/var` compatibility alias.
///
/// `std::fs::canonicalize` may retain `/var` even though it is the immutable
/// system link to `/private/var`. Capability pinning deliberately rejects every
/// symlink component, so retaining that spelling makes valid temporary and user
/// roots fail before their first user-controlled component. No arbitrary
/// symlink is resolved here.
fn platform_canonical_root(path: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let var = Path::new("/var");
        if let Ok(relative) = path.strip_prefix(var) {
            return Path::new("/private/var").join(relative);
        }
    }
    path.to_owned()
}

fn portable_canonical_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let Some(text) = path.to_str() else {
            return path.to_owned();
        };
        if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{unc}"));
        }
        if let Some(local) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(local);
        }
    }
    path.to_owned()
}

/// Bounded Wiki language provenance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum KnowledgeLanguage {
    /// English.
    En,
    /// Korean.
    Ko,
    /// Project contains both English and Korean pages.
    Both,
    /// Legacy page set without a declared language.
    Und,
}

impl KnowledgeLanguage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
            Self::Both => "both",
            Self::Und => "und",
        }
    }
}

/// Visibility boundary applied during shared-index queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum KnowledgeVisibility {
    /// Visible from other projects and user-global queries.
    Shared,
    /// Visible only when querying from the owning project.
    ProjectPrivate,
    /// Confidential and visible only from the owning project.
    Confidential,
}

impl KnowledgeVisibility {
    #[cfg(test)]
    const fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::ProjectPrivate => "project-private",
            Self::Confidential => "confidential",
        }
    }
}

/// Deterministic shared-index rebuild evidence.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct SharedIndexOutcome {
    /// User-root-relative derived paths changed by the rebuild.
    pub changed_paths: Vec<String>,
    /// Indexed canonical pages.
    pub page_count: usize,
    /// Enabled registered projects.
    pub project_count: usize,
    /// Logical digest over registry and canonical page provenance.
    pub logical_digest: String,
}

/// Project-registry mutation evidence.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct ProjectRegistryOutcome {
    /// User-root-relative canonical paths changed by the mutation.
    pub changed_paths: Vec<String>,
    /// Registered project count after the mutation.
    pub project_count: usize,
    /// Digest of canonical registry bytes after the mutation.
    pub registry_digest: String,
}

/// Atomic project registration plus optional shared-index rebuild evidence.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct AtomicProjectRegistrationOutcome {
    /// Canonical registry mutation evidence.
    pub registry: ProjectRegistryOutcome,
    /// Shared-index rebuild evidence when requested.
    pub shared_index: Option<SharedIndexOutcome>,
}

/// Atomic project registration while Global Wiki remains disabled.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct DisabledSharedIndexRegistrationOutcome {
    /// Canonical registry mutation evidence.
    pub registry: ProjectRegistryOutcome,
    /// Disposable RAG paths removed by the transaction.
    pub removed_derived_paths: Vec<String>,
}

/// Query hit with complete source and visibility provenance.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct SharedQueryHit {
    /// Stable registry project ID, or `user-root`.
    pub source_project: String,
    /// Stable page ID within the source project.
    pub page_id: String,
    /// Declared source language.
    pub language: String,
    /// Canonical page byte digest.
    pub digest: String,
    /// Indexed visibility boundary.
    pub visibility: String,
    /// Existing Wiki query fields.
    #[serde(flatten)]
    pub page: QueryHit,
}

#[cfg(test)]
#[derive(Debug)]
struct SharedPage {
    source_project: String,
    language: KnowledgeLanguage,
    visibility: KnowledgeVisibility,
    page: crate::WikiPage,
}

/// Parse and validate a strict project registry without filesystem mutation.
///
/// # Errors
///
/// Returns an error for malformed YAML, unsupported versions, duplicate IDs or roots,
/// unsafe project roots, or the user root being registered as a project.
pub fn validate_project_registry_bytes(
    user_root: &Path,
    bytes: &[u8],
) -> Result<ProjectRegistry, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    if bytes.len() > MAX_REGISTRY_BYTES {
        return Err(WikiError::InvalidInput(
            "project registry exceeds the 1 MiB limit".to_owned(),
        ));
    }
    let registry: ProjectRegistry = serde_yaml::from_slice(bytes)
        .map_err(|error| WikiError::InvalidInput(format!("invalid project registry: {error}")))?;
    validate_registry(user_root, &registry)?;
    Ok(registry)
}

/// Read and validate `~/.hive/config/projects.yml` without following a file symlink.
///
/// # Errors
///
/// Returns an error when the canonical ledger is missing, unsafe, too large, or invalid.
pub fn load_project_registry(user_root: &Path) -> Result<ProjectRegistry, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    ensure_no_symlink_ancestors(user_root, Path::new(PROJECT_REGISTRY_RELATIVE))
        .map_err(|error| WikiError::Conflict(error.to_string()))?;
    let root = Dir::open_ambient_dir(user_root, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open user root: {error}")))?;
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = root
        .open_with(PROJECT_REGISTRY_RELATIVE, &options)
        .map_err(|error| WikiError::Io(format!("cannot open project registry: {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect project registry: {error}")))?;
    if !metadata.is_file() {
        return Err(WikiError::Verification(
            "project registry must be a regular no-follow file".to_owned(),
        ));
    }
    if metadata.len() > MAX_REGISTRY_BYTES as u64 {
        return Err(WikiError::Verification(
            "project registry exceeds the 1 MiB limit".to_owned(),
        ));
    }
    let mut bytes = Vec::new();
    file.take((MAX_REGISTRY_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| WikiError::Io(format!("cannot read project registry: {error}")))?;
    if bytes.len() > MAX_REGISTRY_BYTES {
        return Err(WikiError::Verification(
            "project registry exceeds the 1 MiB limit".to_owned(),
        ));
    }
    validate_project_registry_bytes(user_root, &bytes)
}

/// Ensure the canonical user-root project registry exists without changing existing entries.
///
/// # Errors
///
/// Returns an error for an unsafe user root, malformed existing registry, concurrent
/// replacement, or failed activation.
pub fn ensure_project_registry(user_root: &Path) -> Result<ProjectRegistryOutcome, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    let root = Dir::open_ambient_dir(user_root, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open user root: {error}")))?;
    let _lock = crate::CapabilityKnowledgeLock::acquire(&root)?;
    let prior = crate::read_capability_optional(&root, Path::new(PROJECT_REGISTRY_RELATIVE))?;
    if let Some(bytes) = prior.as_deref() {
        let registry = validate_project_registry_bytes(user_root, bytes)?;
        return Ok(ProjectRegistryOutcome {
            changed_paths: Vec::new(),
            project_count: registry.projects.len(),
            registry_digest: sha256_digest(bytes),
        });
    }
    let registry = ProjectRegistry {
        schema_version: 1,
        projects: Vec::new(),
    };
    let bytes = serde_yaml::to_string(&registry)
        .map_err(|error| WikiError::Io(format!("cannot encode project registry: {error}")))?
        .into_bytes();
    let registry_digest = sha256_digest(&bytes);
    let mut snapshots = [crate::CapabilityFileSnapshot::capture(
        &root,
        Path::new(PROJECT_REGISTRY_RELATIVE),
    )?];
    let changed = crate::transactional_capability(&root, &mut snapshots, |snapshots| {
        snapshots[0].install_staged(&root, &bytes)
    })?;
    Ok(ProjectRegistryOutcome {
        changed_paths: if changed {
            vec![PROJECT_REGISTRY_RELATIVE.to_owned()]
        } else {
            Vec::new()
        },
        project_count: 0,
        registry_digest,
    })
}

/// Safely create or update one registered project while preserving all other entries.
///
/// The registry is sorted by project ID and installed through the same capability-pinned,
/// compare-and-swap transaction used for canonical Wiki mutation. Reusing an ID updates that
/// entry; reusing another entry's root is rejected.
///
/// # Errors
///
/// Returns an error for an unsafe user/project root, malformed prior registry, ID/root
/// collision, concurrent registry replacement, or failed rollback.
pub fn register_project(
    user_root: &Path,
    project: RegisteredProject,
) -> Result<ProjectRegistryOutcome, WikiError> {
    Ok(register_project_atomic(user_root, project, false)?.registry)
}

/// Atomically register one project and optionally rebuild the shared index.
///
/// The prior registry remains exact when registry staging, shared-index construction,
/// replacement, or replacement rollback fails. Existing callers can continue using
/// [`register_project`] when they do not need the shared rebuild in the same transaction.
///
/// # Errors
///
/// Returns an error for an unsafe root, invalid registration, failed shared-index rebuild,
/// concurrent replacement, or failed rollback.
pub fn register_project_atomic(
    user_root: &Path,
    mut project: RegisteredProject,
    rebuild_index: bool,
) -> Result<AtomicProjectRegistrationOutcome, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    project.root = portable_canonical_path(&project.root);
    if !rebuild_index {
        return register_project_without_index(user_root, project, false)
            .map(|(outcome, _)| outcome);
    }
    let store = RagStore::open(user_root)?;
    let committed = store.register_project_atomic(project)?;
    let bytes = serde_yaml::to_string(&committed.registry)
        .map_err(|error| WikiError::Io(format!("cannot encode project registry: {error}")))?
        .into_bytes();
    let registry_digest = sha256_digest(&bytes);
    let registry_changed = committed
        .store
        .changed_paths
        .iter()
        .any(|path| path == PROJECT_REGISTRY_RELATIVE);
    let shared_index = if rebuild_index {
        Some(shared_outcome_from_commit(
            &store,
            &committed.registry,
            &committed.store,
        )?)
    } else {
        None
    };
    Ok(AtomicProjectRegistrationOutcome {
        registry: ProjectRegistryOutcome {
            changed_paths: if registry_changed {
                vec![PROJECT_REGISTRY_RELATIVE.to_owned()]
            } else {
                Vec::new()
            },
            project_count: committed.registry.projects.len(),
            registry_digest,
        },
        shared_index,
    })
}

fn register_project_without_index(
    user_root: &Path,
    project: RegisteredProject,
    remove_derived: bool,
) -> Result<(AtomicProjectRegistrationOutcome, Vec<String>), WikiError> {
    let root = Dir::open_ambient_dir(user_root, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open user root: {error}")))?;
    let _lock = crate::CapabilityKnowledgeLock::acquire(&root)?;
    let prior = crate::read_capability_optional(&root, Path::new(PROJECT_REGISTRY_RELATIVE))?;
    let mut registry = match prior.as_deref() {
        Some(bytes) => validate_project_registry_bytes(user_root, bytes)?,
        None => ProjectRegistry {
            schema_version: 1,
            projects: Vec::new(),
        },
    };
    if let Some(existing) = registry
        .projects
        .iter_mut()
        .find(|existing| existing.id == project.id)
    {
        *existing = project;
    } else {
        registry.projects.push(project);
    }
    registry
        .projects
        .sort_by(|left, right| left.id.cmp(&right.id));
    validate_registry(user_root, &registry)?;
    let bytes = serde_yaml::to_string(&registry)
        .map_err(|error| WikiError::Io(format!("cannot encode project registry: {error}")))?
        .into_bytes();
    let registry_digest = sha256_digest(&bytes);
    let mut relatives = vec![PROJECT_REGISTRY_RELATIVE];
    if remove_derived {
        relatives.extend(SHARED_DERIVED_RELATIVES);
    }
    let mut snapshots = relatives
        .iter()
        .map(|relative| crate::CapabilityFileSnapshot::capture(&root, Path::new(relative)))
        .collect::<Result<Vec<_>, _>>()?;
    let (changed, removed_derived_paths) =
        crate::transactional_capability(&root, &mut snapshots, |snapshots| {
            let changed = snapshots[0].install_staged(&root, &bytes)?;
            let mut removed = Vec::new();
            for (snapshot, relative) in snapshots[1..].iter_mut().zip(SHARED_DERIVED_RELATIVES) {
                if snapshot.remove(&root)? {
                    removed.push(relative.to_owned());
                }
            }
            Ok((changed, removed))
        })?;
    Ok((
        AtomicProjectRegistrationOutcome {
            registry: ProjectRegistryOutcome {
                changed_paths: if changed {
                    vec![PROJECT_REGISTRY_RELATIVE.to_owned()]
                } else {
                    Vec::new()
                },
                project_count: registry.projects.len(),
                registry_digest,
            },
            shared_index: None,
        },
        removed_derived_paths,
    ))
}

/// Register one project and atomically remove the RAG projection and trust receipt.
///
/// This is the setup path for a disabled Global Wiki. Canonical Wiki and registry bytes other
/// than the requested registration are preserved.
///
/// # Errors
///
/// Returns an error for an unsafe root, invalid registration, unsafe derived path, concurrent
/// replacement, or rollback failure.
pub fn register_project_with_shared_index_disabled(
    user_root: &Path,
    mut project: RegisteredProject,
) -> Result<DisabledSharedIndexRegistrationOutcome, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    project.root = portable_canonical_path(&project.root);
    let (registered, removed_derived_paths) =
        register_project_without_index(user_root, project, true)?;
    Ok(DisabledSharedIndexRegistrationOutcome {
        registry: registered.registry,
        removed_derived_paths,
    })
}

/// Rebuild the one user-root `SQLite` index from user Wiki plus enabled project Wikis.
///
/// This function never creates or mutates a project-local database.
///
/// # Errors
///
/// Returns an error for invalid registry/source paths, malformed canonical pages, or an
/// unverified temporary database.
#[allow(clippy::too_many_lines)]
pub fn rebuild_shared_index(user_root: &Path) -> Result<SharedIndexOutcome, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    let registry = load_project_registry(user_root)?;
    let store = RagStore::open(user_root)?;
    let recovered = if store.is_dirty()? {
        Some(store.rebuild()?)
    } else {
        None
    };
    let mut synchronized = store.sync_project_registry(&registry)?;
    if let Some(recovered) = recovered {
        synchronized.changed_paths.extend(recovered.changed_paths);
        synchronized.changed_paths.sort();
        synchronized.changed_paths.dedup();
    }
    let current = store.validate_current()?;
    let snapshot = store.load_canonical_snapshot(current.generation)?;
    let candidate = build_rag_index(&snapshot).map_err(|error| {
        WikiError::Verification(format!(
            "cannot verify the canonical RAG rebuild candidate: {error}"
        ))
    })?;
    let mut committed = if candidate.manifest.logical_digest == current.manifest_digest {
        synchronized
    } else {
        let mut rebuilt = store.rebuild()?;
        rebuilt.changed_paths.extend(synchronized.changed_paths);
        rebuilt.changed_paths.sort();
        rebuilt.changed_paths.dedup();
        rebuilt
    };
    committed.changed_paths.sort();
    committed.changed_paths.dedup();
    shared_outcome_from_commit(&store, &registry, &committed)
}

/// Remove user-root RAG operational state while preserving canonical knowledge bytes.
///
/// # Errors
///
/// Returns an error for an unsafe user root, non-regular derived path, concurrent
/// replacement, or rollback failure.
pub fn remove_shared_derived_state(user_root: &Path) -> Result<Vec<String>, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    let root = Dir::open_ambient_dir(user_root, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open user root: {error}")))?;
    let _lock = crate::CapabilityKnowledgeLock::acquire(&root)?;
    let mut snapshots = SHARED_DERIVED_RELATIVES
        .iter()
        .map(|relative| crate::CapabilityFileSnapshot::capture(&root, Path::new(relative)))
        .collect::<Result<Vec<_>, _>>()?;
    crate::transactional_capability(&root, &mut snapshots, |snapshots| {
        let mut changed = Vec::new();
        for (snapshot, relative) in snapshots.iter_mut().zip(SHARED_DERIVED_RELATIVES) {
            if snapshot.remove(&root)? {
                changed.push(relative.to_owned());
            }
        }
        Ok(changed)
    })
}

/// Verify that no user-root RAG projection or trust receipt remains.
///
/// # Errors
///
/// Returns an error for an unsafe user root, unsafe derived path, or any derived artifact that
/// still exists.
pub fn validate_shared_derived_state_absent(user_root: &Path) -> Result<(), WikiError> {
    validate_absolute_root(user_root, "user root")?;
    let root = Dir::open_ambient_dir(user_root, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open user root: {error}")))?;
    let _lock = crate::CapabilityKnowledgeLock::acquire(&root)?;
    for relative in SHARED_DERIVED_RELATIVES {
        let Some((parent, name)) = crate::capability_parent(&root, Path::new(relative), false)?
        else {
            continue;
        };
        match parent.symlink_metadata(&name) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(WikiError::Io(format!(
                    "cannot inspect disposable RAG path {relative}: {error}"
                )))
            }
            Ok(_) => {
                return Err(WikiError::Verification(format!(
                    "disposable RAG path remains while Global Wiki is disabled: {relative}"
                )))
            }
        }
    }
    Ok(())
}

fn shared_outcome_from_commit(
    store: &RagStore,
    registry: &ProjectRegistry,
    committed: &crate::store::StoreCommit,
) -> Result<SharedIndexOutcome, WikiError> {
    let snapshot = store.load_canonical_snapshot(committed.generation)?;
    let shared_collection_ids = snapshot
        .registry
        .collections
        .iter()
        .filter(|collection| {
            collection.collection_id == USER_ROOT_COLLECTION_ID
                || collection.source_project_id.is_some()
        })
        .map(|collection| collection.collection_id.as_str())
        .collect::<BTreeSet<_>>();
    Ok(SharedIndexOutcome {
        changed_paths: committed.changed_paths.clone(),
        page_count: snapshot
            .documents
            .iter()
            .filter(|document| shared_collection_ids.contains(document.collection_id.as_str()))
            .count(),
        project_count: registry
            .projects
            .iter()
            .filter(|project| project.enabled)
            .count(),
        logical_digest: committed.manifest_digest.clone(),
    })
}

#[cfg(test)]
pub(crate) fn rebuild_legacy_shared_index_for_test(
    user_root: &Path,
) -> Result<SharedIndexOutcome, WikiError> {
    validate_absolute_root(user_root, "user root")?;
    let root = Dir::open_ambient_dir(user_root, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot open user root: {error}")))?;
    let _lock = crate::CapabilityKnowledgeLock::acquire(&root)?;
    rebuild_legacy_shared_index_locked(user_root, &root)
}

#[cfg(test)]
fn rebuild_legacy_shared_index_locked(
    user_root: &Path,
    root: &Dir,
) -> Result<SharedIndexOutcome, WikiError> {
    let registry = load_project_registry(user_root)?;
    let pages = collect_shared_pages(user_root, &registry)?;
    let logical_digest = shared_logical_digest(&registry, &pages)?;
    ensure_no_symlink_ancestors(user_root, Path::new(SHARED_INDEX_RELATIVE))
        .map_err(|error| WikiError::Conflict(error.to_string()))?;
    if existing_shared_index_is_current(root, &registry, &pages, &logical_digest)? {
        return Ok(SharedIndexOutcome {
            changed_paths: Vec::new(),
            page_count: pages.len(),
            project_count: registry
                .projects
                .iter()
                .filter(|project| project.enabled)
                .count(),
            logical_digest,
        });
    }
    let connection = build_shared_projection(&registry, &pages, &logical_digest)?;
    verify_connection(&connection, &logical_digest, pages.len())?;
    let serialized = connection
        .serialize(MAIN_DB)
        .map_err(sqlite_error)?
        .to_vec();
    publish_shared_index(root, &serialized)?;
    Ok(SharedIndexOutcome {
        changed_paths: vec![SHARED_INDEX_RELATIVE.to_owned()],
        page_count: pages.len(),
        project_count: registry
            .projects
            .iter()
            .filter(|project| project.enabled)
            .count(),
        logical_digest,
    })
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
fn build_shared_projection(
    registry: &ProjectRegistry,
    pages: &BTreeMap<String, SharedPage>,
    logical_digest: &str,
) -> Result<Connection, WikiError> {
    let mut connection = Connection::open_in_memory().map_err(sqlite_error)?;
    connection
        .execute_batch(
            "PRAGMA journal_mode=DELETE;
             PRAGMA synchronous=FULL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE pages(
               row_key TEXT PRIMARY KEY,
               source_project TEXT NOT NULL,
               page_id TEXT NOT NULL,
               language TEXT NOT NULL,
               visibility TEXT NOT NULL,
               kind TEXT NOT NULL,
               summary TEXT NOT NULL,
               path TEXT NOT NULL,
               content_digest TEXT NOT NULL,
               body TEXT NOT NULL,
               UNIQUE(source_project, page_id)
             );
             CREATE VIRTUAL TABLE pages_fts USING fts5(
               row_key UNINDEXED, summary, body, aliases, tags
             );
             CREATE TABLE tags(
               row_key TEXT NOT NULL, tag TEXT NOT NULL, PRIMARY KEY(row_key, tag)
             );
             CREATE TABLE aliases(
               row_key TEXT NOT NULL, alias TEXT NOT NULL, PRIMARY KEY(row_key, alias)
             );
             CREATE TABLE sources(
               row_key TEXT NOT NULL, locator TEXT NOT NULL, PRIMARY KEY(row_key, locator)
             );",
        )
        .map_err(sqlite_error)?;
    let transaction = connection.transaction().map_err(sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO meta(key, value) VALUES
             ('schema_version', '2'),
             ('logical_digest', ?1),
             ('page_count', ?2),
             ('project_count', ?3)",
            params![
                logical_digest,
                pages.len().to_string(),
                registry
                    .projects
                    .iter()
                    .filter(|project| project.enabled)
                    .count()
                    .to_string()
            ],
        )
        .map_err(sqlite_error)?;
    for item in pages.values() {
        let row_key = row_key(&item.source_project, &item.page.frontmatter.id);
        let meta = &item.page.frontmatter;
        transaction
            .execute(
                "INSERT INTO pages VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    row_key,
                    item.source_project,
                    meta.id,
                    item.language.as_str(),
                    item.visibility.as_str(),
                    meta.kind,
                    meta.summary,
                    item.page.relative_path,
                    item.page.content_digest,
                    item.page.body
                ],
            )
            .map_err(sqlite_error)?;
        transaction
            .execute(
                "INSERT INTO pages_fts VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    row_key,
                    meta.summary,
                    item.page.body,
                    meta.aliases.join(" "),
                    meta.tags.join(" ")
                ],
            )
            .map_err(sqlite_error)?;
        for tag in &meta.tags {
            transaction
                .execute("INSERT INTO tags VALUES (?1, ?2)", params![row_key, tag])
                .map_err(sqlite_error)?;
        }
        for alias in &meta.aliases {
            transaction
                .execute(
                    "INSERT INTO aliases VALUES (?1, ?2)",
                    params![row_key, alias],
                )
                .map_err(sqlite_error)?;
        }
        for source in &meta.sources {
            transaction
                .execute(
                    "INSERT INTO sources VALUES (?1, ?2)",
                    params![row_key, source],
                )
                .map_err(sqlite_error)?;
        }
    }
    transaction.commit().map_err(sqlite_error)?;
    connection
        .execute_batch("PRAGMA optimize;")
        .map_err(sqlite_error)?;
    Ok(connection)
}

/// Validate that the user-root shared index exactly reflects all canonical sources.
///
/// # Errors
///
/// Returns an error when the registry or canonical pages are invalid, the index is missing
/// or unsafe, or any derived schema, row, provenance, or FTS projection is stale.
pub fn validate_shared_index(user_root: &Path) -> Result<String, WikiError> {
    let store = RagStore::open(user_root)?;
    Ok(store.validate_current()?.manifest_digest)
}

/// Query the current shared index with project visibility enforcement.
///
/// `current_project` grants private/confidential access only to that exact registered root.
/// Omitting it performs a user-global query and returns user-root plus shared rows only.
///
/// # Errors
///
/// Returns an error for invalid query options, an unregistered current project, stale index
/// state, or a read-only `SQLite` failure.
pub fn query_shared(
    user_root: &Path,
    current_project: Option<&Path>,
    text: Option<&str>,
    tag: Option<&str>,
    limit: usize,
) -> Result<Vec<SharedQueryHit>, WikiError> {
    query_shared_filtered(user_root, current_project, text, tag, None, limit)
}

/// Query the current RAG index with combined text, tag, and Wiki-category filters.
///
/// This compatibility facade reads one verified RAG generation and never enumerates
/// canonical Markdown at query time.
///
/// # Errors
///
/// Returns an error for invalid query options, an unregistered current project, stale
/// derived state, or a read-only `SQLite` failure.
pub fn query_shared_filtered(
    user_root: &Path,
    current_project: Option<&Path>,
    text: Option<&str>,
    tag: Option<&str>,
    category: Option<&str>,
    limit: usize,
) -> Result<Vec<SharedQueryHit>, WikiError> {
    if text.is_none() && tag.is_none() && category.is_none() {
        return Err(WikiError::InvalidInput(
            "query requires --text, --tag, or --category".to_owned(),
        ));
    }
    if !(1..=100).contains(&limit) {
        return Err(WikiError::InvalidInput(
            "query limit must be from 1 through 100".to_owned(),
        ));
    }
    let registry = load_project_registry(user_root)?;
    let current_id = current_project
        .map(|path| registered_project_id(&registry, path))
        .transpose()?;
    let store = RagStore::open(user_root)?;
    let hits = store.query_wiki_pages(&WikiPageQueryRequest {
        current_project_id: current_id.clone(),
        text: text.map(str::to_owned),
        tag: tag.map(str::to_owned),
        category: category.map(str::to_owned),
        limit,
    })?;
    hits.into_iter()
        .map(|hit| {
            let source_project = hit
                .source_project_id
                .unwrap_or_else(|| "user-root".to_owned());
            let language = registry
                .projects
                .iter()
                .find(|project| project.id == source_project)
                .map_or("und", |project| project.language.as_str())
                .to_owned();
            let page_id = Path::new(&hit.locator)
                .file_stem()
                .and_then(|value| value.to_str())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    WikiError::Verification(format!(
                        "indexed Wiki locator has no page ID: {}",
                        hit.locator
                    ))
                })?
                .to_owned();
            let visibility = match hit.visibility {
                RagVisibility::Shared => "shared",
                RagVisibility::ProjectPrivate => "project-private",
                RagVisibility::Confidential => "confidential",
            }
            .to_owned();
            Ok(SharedQueryHit {
                source_project,
                page_id: page_id.clone(),
                language,
                digest: hit.digest.clone(),
                visibility,
                page: QueryHit {
                    id: page_id,
                    kind: hit.kind,
                    summary: hit.summary,
                    path: hit.locator,
                    content_digest: hit.digest,
                    tags: hit.tags,
                    aliases: hit.aliases,
                    sources: hit.sources,
                },
            })
        })
        .collect()
}

fn validate_registry(user_root: &Path, registry: &ProjectRegistry) -> Result<(), WikiError> {
    if registry.schema_version != 1 {
        return Err(WikiError::InvalidInput(
            "project registry schema_version must be 1".to_owned(),
        ));
    }
    if registry.projects.len() > 256 {
        return Err(WikiError::InvalidInput(
            "project registry cannot contain more than 256 projects".to_owned(),
        ));
    }
    let canonical_user = canonical_root(user_root)?;
    let mut ids = BTreeSet::new();
    let mut roots = BTreeSet::new();
    for project in &registry.projects {
        if !valid_slug(&project.id) {
            return Err(WikiError::InvalidInput(format!(
                "invalid registered project id: {}",
                project.id
            )));
        }
        if !ids.insert(project.id.clone()) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate registered project id: {}",
                project.id
            )));
        }
        validate_absolute_root(&project.root, "registered project root")?;
        let canonical = canonical_root(&project.root).map_err(|error| {
            WikiError::Verification(format!(
                "registered project root is unavailable {}: {error}",
                project.root.display()
            ))
        })?;
        if canonical != platform_canonical_root(&portable_canonical_path(&project.root)) {
            return Err(WikiError::Conflict(format!(
                "registered project root must be canonical and contain no symlink components: {}",
                project.root.display()
            )));
        }
        if canonical == canonical_user {
            return Err(WikiError::InvalidInput(
                "user root cannot be registered as a project".to_owned(),
            ));
        }
        if !roots.insert(canonical) {
            return Err(WikiError::InvalidInput(
                "duplicate registered project root".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_absolute_root(path: &Path, name: &str) -> Result<(), WikiError> {
    let path = platform_canonical_root(path);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(WikiError::InvalidInput(format!(
            "{name} must be an absolute normalized path"
        )));
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        WikiError::Io(format!("cannot inspect {name} {}: {error}", path.display()))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WikiError::Conflict(format!(
            "{name} must be a regular directory and not a symlink"
        )));
    }
    Ok(())
}

#[cfg(test)]
fn collect_shared_pages(
    user_root: &Path,
    registry: &ProjectRegistry,
) -> Result<BTreeMap<String, SharedPage>, WikiError> {
    let mut pages = BTreeMap::new();
    let user_source = pin_source_root(user_root, "user root")?;
    collect_source_pages(
        &user_source,
        "user-root",
        KnowledgeLanguage::Und,
        KnowledgeVisibility::Shared,
        &mut pages,
    )?;
    for project in registry.projects.iter().filter(|project| project.enabled) {
        let project_source = pin_source_root(&project.root, "registered project root")?;
        collect_source_pages(
            &project_source,
            &project.id,
            project.language,
            project.visibility,
            &mut pages,
        )?;
    }
    Ok(pages)
}

#[cfg(test)]
type SharedRootOpenRace = Box<dyn FnMut(&Path) -> bool>;

#[cfg(test)]
thread_local! {
    static INJECT_SHARED_ROOT_OPEN_RACE: std::cell::RefCell<Option<SharedRootOpenRace>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn injected_shared_root_open_race(path: &Path) {
    INJECT_SHARED_ROOT_OPEN_RACE.with(|injected| {
        let Some(mut race) = injected.borrow_mut().take() else {
            return;
        };
        if !race(path) {
            *injected.borrow_mut() = Some(race);
        }
    });
}

#[cfg(test)]
type SharedAncestorOpenRace = Box<dyn FnMut(&Path) -> bool>;

#[cfg(test)]
thread_local! {
    static INJECT_SHARED_ANCESTOR_OPEN_RACE: std::cell::RefCell<Option<SharedAncestorOpenRace>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn injected_shared_ancestor_open_race(path: &Path) {
    INJECT_SHARED_ANCESTOR_OPEN_RACE.with(|injected| {
        let Some(mut race) = injected.borrow_mut().take() else {
            return;
        };
        if !race(path) {
            *injected.borrow_mut() = Some(race);
        }
    });
}

#[cfg(test)]
fn pin_source_root(path: &Path, name: &str) -> Result<Dir, WikiError> {
    let path = platform_canonical_root(path);
    validate_absolute_root(&path, name)?;
    let expected = Dir::open_ambient_dir(&path, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot capture {name} identity: {error}")))?;
    let expected = expected
        .dir_metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect captured {name}: {error}")))?;
    injected_shared_ancestor_open_race(&path);
    let pinned = open_source_root_nofollow(&path, name)?;
    let actual = pinned
        .dir_metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect pinned {name}: {error}")))?;
    if (CapMetadataExt::dev(&actual), CapMetadataExt::ino(&actual))
        != (
            CapMetadataExt::dev(&expected),
            CapMetadataExt::ino(&expected),
        )
    {
        return Err(WikiError::Conflict(format!(
            "{name} identity changed before its no-follow capability was pinned"
        )));
    }
    Ok(pinned)
}

#[cfg(test)]
fn open_source_root_nofollow(path: &Path, name: &str) -> Result<Dir, WikiError> {
    let path = platform_canonical_root(path);
    let mut filesystem_root = PathBuf::new();
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => filesystem_root.push(prefix.as_os_str()),
            Component::RootDir => filesystem_root.push(component.as_os_str()),
            Component::Normal(component) => components.push(component.to_os_string()),
            Component::CurDir | Component::ParentDir => {
                return Err(WikiError::InvalidInput(format!(
                    "{name} path is not lexically safe"
                )));
            }
        }
    }
    if filesystem_root.as_os_str().is_empty() {
        return Err(WikiError::InvalidInput(format!("{name} must be absolute")));
    }
    let mut current =
        Dir::open_ambient_dir(&filesystem_root, ambient_authority()).map_err(|error| {
            WikiError::Io(format!(
                "cannot open {name} filesystem root {}: {error}",
                filesystem_root.display()
            ))
        })?;
    let mut walked = filesystem_root;
    let last = components.len().saturating_sub(1);
    for (index, component) in components.into_iter().enumerate() {
        walked.push(&component);
        let expected = current.symlink_metadata(&component).map_err(|error| {
            WikiError::Conflict(format!(
                "cannot inspect {name} component {}: {error}",
                walked.display()
            ))
        })?;
        if !expected.is_dir() {
            return Err(WikiError::Conflict(format!(
                "{name} component is not a no-follow directory: {}",
                walked.display()
            )));
        }
        if index == last {
            injected_shared_root_open_race(&path);
        }
        let next = current.open_dir_nofollow(&component).map_err(|error| {
            WikiError::Conflict(format!(
                "cannot open {name} component no-follow {}: {error}",
                walked.display()
            ))
        })?;
        let actual = next.dir_metadata().map_err(|error| {
            WikiError::Io(format!(
                "cannot inspect pinned {name} component {}: {error}",
                walked.display()
            ))
        })?;
        if (CapMetadataExt::dev(&actual), CapMetadataExt::ino(&actual))
            != (
                CapMetadataExt::dev(&expected),
                CapMetadataExt::ino(&expected),
            )
        {
            return Err(WikiError::Conflict(format!(
                "{name} component changed while its capability was pinned: {}",
                walked.display()
            )));
        }
        current = next;
    }
    Ok(current)
}

#[cfg(test)]
type SharedPageOpenRace = Box<dyn FnOnce()>;

#[cfg(test)]
thread_local! {
    static INJECT_SHARED_PAGE_OPEN_RACE: std::cell::RefCell<Option<SharedPageOpenRace>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn injected_shared_page_open_race() {
    INJECT_SHARED_PAGE_OPEN_RACE.with(|injected| {
        if let Some(race) = injected.borrow_mut().take() {
            race();
        }
    });
}

#[cfg(test)]
fn collect_source_pages(
    source_root: &Dir,
    source_project: &str,
    language: KnowledgeLanguage,
    visibility: KnowledgeVisibility,
    pages: &mut BTreeMap<String, SharedPage>,
) -> Result<(), WikiError> {
    let Some((parent, name)) =
        crate::capability_parent(source_root, Path::new(WIKI_RELATIVE), false)?
    else {
        return Ok(());
    };
    let wiki = match parent.open_dir_nofollow(&name) {
        Ok(wiki) => wiki,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(WikiError::Verification(format!(
                "cannot open canonical Wiki directory no-follow: {error}"
            )));
        }
    };
    let mut names = wiki
        .entries()
        .map_err(|error| WikiError::Io(format!("cannot scan canonical Wiki: {error}")))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name())
                .map_err(|error| WikiError::Io(format!("cannot scan canonical Wiki: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
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
        injected_shared_page_open_race();
        let relative = format!("{WIKI_RELATIVE}/{}", path.to_string_lossy()).replace('\\', "/");
        let page = read_source_page(
            &wiki,
            &name,
            &relative,
            source_project,
            language,
            visibility,
        )?;
        let key = row_key(source_project, &page.page.frontmatter.id);
        pages.insert(key, page);
    }
    Ok(())
}

#[cfg(test)]
fn read_source_page(
    wiki: &Dir,
    name: &OsStr,
    relative: &str,
    source_project: &str,
    language: KnowledgeLanguage,
    visibility: KnowledgeVisibility,
) -> Result<SharedPage, WikiError> {
    let file = crate::open_capability_file_nofollow(wiki, name).map_err(|error| {
        WikiError::Verification(format!("cannot open Wiki page no-follow: {error}"))
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| WikiError::Io(format!("cannot inspect opened Wiki page: {error}")))?;
    if !metadata.is_file() {
        return Err(WikiError::Verification(
            "opened Wiki page is not a regular file".to_owned(),
        ));
    }
    if metadata.len() > MAX_PAGE_BYTES as u64 {
        return Err(WikiError::Verification(
            "Wiki page exceeds the 2 MiB limit".to_owned(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(u64::try_from(MAX_PAGE_BYTES + 1).expect("page byte limit fits u64"))
        .read_to_end(&mut bytes)
        .map_err(|error| WikiError::Io(format!("cannot read Wiki page: {error}")))?;
    if bytes.len() > MAX_PAGE_BYTES {
        return Err(WikiError::Verification(
            "Wiki page exceeds the 2 MiB limit".to_owned(),
        ));
    }
    reject_likely_credentials(&bytes).map_err(|error| {
        WikiError::Verification(format!(
            "canonical Wiki page contains likely sensitive material at {relative}: {error}"
        ))
    })?;
    let mut page = parse_page_bytes(&bytes, relative)?;
    let stem = Path::new(name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if stem != page.frontmatter.id {
        return Err(WikiError::Verification(format!(
            "Wiki filename must match page id: {relative}"
        )));
    }
    relative.clone_into(&mut page.relative_path);
    page.content_digest = sha256_digest(&bytes);
    Ok(SharedPage {
        source_project: source_project.to_owned(),
        language,
        visibility,
        page,
    })
}

#[cfg(test)]
fn shared_logical_digest(
    registry: &ProjectRegistry,
    pages: &BTreeMap<String, SharedPage>,
) -> Result<String, WikiError> {
    #[derive(Serialize)]
    struct DigestRow<'a> {
        row_key: &'a str,
        source_project: &'a str,
        page_id: &'a str,
        language: &'a str,
        visibility: &'a str,
        content_digest: &'a str,
    }
    #[derive(Serialize)]
    struct DigestDocument<'a> {
        schema_version: u32,
        registry: &'a ProjectRegistry,
        pages: Vec<DigestRow<'a>>,
    }
    let rows = pages
        .iter()
        .map(|(key, page)| DigestRow {
            row_key: key,
            source_project: &page.source_project,
            page_id: &page.page.frontmatter.id,
            language: page.language.as_str(),
            visibility: page.visibility.as_str(),
            content_digest: &page.page.content_digest,
        })
        .collect();
    let bytes = serde_json::to_vec(&DigestDocument {
        schema_version: 2,
        registry,
        pages: rows,
    })
    .map_err(|error| WikiError::Io(format!("cannot encode shared index digest: {error}")))?;
    Ok(sha256_digest(&bytes))
}

fn registered_project_id(registry: &ProjectRegistry, root: &Path) -> Result<String, WikiError> {
    validate_absolute_root(root, "current project root")?;
    let canonical = canonical_root(root)?;
    registry
        .projects
        .iter()
        .find(|project| {
            project.enabled
                && canonical_root(&project.root).is_ok_and(|registered| registered == canonical)
        })
        .map(|project| project.id.clone())
        .ok_or_else(|| {
            WikiError::InvalidInput(
                "current project is not enabled in the project registry".to_owned(),
            )
        })
}

#[cfg(test)]
fn open_shared_index_snapshot_at(parent: &Dir, name: &OsStr) -> Result<Connection, WikiError> {
    let file = crate::open_capability_file_nofollow(parent, name).map_err(|error| {
        WikiError::Verification(format!(
            "cannot open shared SQLite index no-follow: {error}"
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        WikiError::Io(format!(
            "cannot inspect opened shared SQLite index: {error}"
        ))
    })?;
    if !metadata.is_file() {
        return Err(WikiError::Verification(
            "shared SQLite index is not a regular file".to_owned(),
        ));
    }
    let byte_len = usize::try_from(metadata.len()).map_err(|_| {
        WikiError::Verification("shared SQLite index size is not representable".to_owned())
    })?;
    if byte_len == 0 || byte_len > MAX_SHARED_INDEX_BYTES {
        return Err(WikiError::Verification(
            "shared SQLite index has an invalid size".to_owned(),
        ));
    }
    let mut connection = Connection::open_in_memory().map_err(sqlite_error)?;
    connection
        .deserialize_read_exact(MAIN_DB, file, byte_len, false)
        .map_err(sqlite_error)?;
    Ok(connection)
}

#[cfg(test)]
fn existing_shared_index_is_current(
    root: &Dir,
    registry: &ProjectRegistry,
    pages: &BTreeMap<String, SharedPage>,
    logical_digest: &str,
) -> Result<bool, WikiError> {
    let Some((parent, name)) =
        crate::capability_parent(root, Path::new(SHARED_INDEX_RELATIVE), false)?
    else {
        return Ok(false);
    };
    let Some(identity) = shared_index_identity(&parent, &name)? else {
        return Ok(false);
    };
    let validation = open_shared_index_snapshot_at(&parent, &name).and_then(|connection| {
        validate_shared_projection(&connection, registry, pages, logical_digest)
    });
    if shared_index_identity(&parent, &name)? != Some(identity) {
        return Err(WikiError::Conflict(
            "shared SQLite index changed during equivalence validation".to_owned(),
        ));
    }
    Ok(validation.is_ok())
}

#[cfg(test)]
fn validate_shared_projection(
    connection: &Connection,
    registry: &ProjectRegistry,
    pages: &BTreeMap<String, SharedPage>,
    logical_digest: &str,
) -> Result<(), WikiError> {
    const PROJECTION_QUERIES: [(&str, usize); 12] = [
        (
            "SELECT type, name, tbl_name, sql
             FROM sqlite_schema ORDER BY type, name",
            4,
        ),
        ("SELECT key, value FROM meta ORDER BY key", 2),
        (
            "SELECT row_key, source_project, page_id, language, visibility,
                    kind, summary, path, content_digest, body
             FROM pages ORDER BY row_key",
            10,
        ),
        (
            "SELECT rowid, row_key, summary, body, aliases, tags
             FROM pages_fts ORDER BY rowid",
            6,
        ),
        (
            "SELECT id, c0, c1, c2, c3, c4
             FROM pages_fts_content ORDER BY id",
            6,
        ),
        ("SELECT id, block FROM pages_fts_data ORDER BY id", 2),
        ("SELECT id, sz FROM pages_fts_docsize ORDER BY id", 2),
        (
            "SELECT segid, term, pgno
             FROM pages_fts_idx ORDER BY segid, term",
            3,
        ),
        ("SELECT k, v FROM pages_fts_config ORDER BY k", 2),
        ("SELECT row_key, tag FROM tags ORDER BY row_key, tag", 2),
        (
            "SELECT row_key, alias FROM aliases ORDER BY row_key, alias",
            2,
        ),
        (
            "SELECT row_key, locator FROM sources ORDER BY row_key, locator",
            2,
        ),
    ];
    let expected = build_shared_projection(registry, pages, logical_digest)?;
    for (query, columns) in PROJECTION_QUERIES {
        if projection_rows(connection, query, columns)?
            != projection_rows(&expected, query, columns)?
        {
            return Err(WikiError::Verification(
                "shared SQLite index does not match the canonical projection".to_owned(),
            ));
        }
    }
    if projection_rows(connection, "PRAGMA quick_check", 1)? != vec![vec![Value::Text("ok".into())]]
    {
        return Err(WikiError::Verification(
            "shared SQLite index integrity check failed".to_owned(),
        ));
    }
    connection
        .execute(
            "INSERT INTO pages_fts(pages_fts) VALUES('integrity-check')",
            [],
        )
        .map_err(|error| {
            WikiError::Verification(format!("shared SQLite FTS integrity check failed: {error}"))
        })?;
    Ok(())
}

#[cfg(test)]
fn projection_rows(
    connection: &Connection,
    query: &str,
    columns: usize,
) -> Result<Vec<Vec<Value>>, WikiError> {
    let mut statement = connection.prepare(query).map_err(sqlite_error)?;
    let rows = statement
        .query_map([], |row| {
            (0..columns)
                .map(|column| row.get(column))
                .collect::<Result<Vec<Value>, _>>()
        })
        .map_err(sqlite_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sqlite_error)?;
    Ok(rows)
}

#[cfg(test)]
fn verify_connection(
    connection: &Connection,
    digest: &str,
    page_count: usize,
) -> Result<(), WikiError> {
    let stored: String = connection
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
    if stored != digest
        || usize::try_from(count).ok() != Some(page_count)
        || usize::try_from(fts_count).ok() != Some(page_count)
        || integrity != "ok"
    {
        return Err(WikiError::Verification(
            "temporary shared index verification failed".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SharedIndexIdentity {
    dev: u64,
    ino: u64,
}

#[cfg(test)]
impl SharedIndexIdentity {
    fn from_metadata(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            dev: CapMetadataExt::dev(metadata),
            ino: CapMetadataExt::ino(metadata),
        }
    }
}

#[cfg(test)]
fn publish_shared_index(root: &Dir, bytes: &[u8]) -> Result<(), WikiError> {
    if bytes.is_empty() || bytes.len() > MAX_SHARED_INDEX_BYTES {
        return Err(WikiError::Verification(
            "serialized shared index has an invalid size".to_owned(),
        ));
    }
    let (index_dir, index_name) =
        crate::capability_parent(root, Path::new(SHARED_INDEX_RELATIVE), true)?
            .ok_or_else(|| WikiError::Io("shared index parent disappeared".to_owned()))?;
    let expected = shared_index_identity(&index_dir, &index_name)?;
    let (temporary, backup) = unique_shared_index_names(&index_dir)?;
    write_synced_shared_index(&index_dir, &temporary, bytes)?;
    if let Err(error) =
        activate_shared_index(&index_dir, &index_name, &temporary, &backup, expected)
    {
        let _ = index_dir.remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
fn unique_shared_index_names(index_dir: &Dir) -> Result<(OsString, OsString), WikiError> {
    loop {
        let suffix = format!(
            "{}-{}",
            std::process::id(),
            SHARED_INDEX_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let temporary = OsString::from(format!(".hive.sqlite3.tmp-{suffix}"));
        let backup = OsString::from(format!(".hive.sqlite3.backup-{suffix}"));
        if shared_index_identity(index_dir, &temporary)?.is_none()
            && shared_index_identity(index_dir, &backup)?.is_none()
        {
            return Ok((temporary, backup));
        }
    }
}

#[cfg(test)]
fn shared_index_identity(
    index_dir: &Dir,
    name: &OsStr,
) -> Result<Option<SharedIndexIdentity>, WikiError> {
    match index_dir.symlink_metadata(name) {
        Ok(metadata) if !metadata.is_file() => Err(WikiError::Verification(format!(
            "shared SQLite path is not a regular no-follow file: {}",
            name.to_string_lossy()
        ))),
        Ok(metadata) => Ok(Some(SharedIndexIdentity::from_metadata(&metadata))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(WikiError::Io(format!(
            "cannot inspect shared SQLite path {}: {error}",
            name.to_string_lossy()
        ))),
    }
}

#[cfg(test)]
fn write_synced_shared_index(
    index_dir: &Dir,
    temporary: &OsStr,
    bytes: &[u8],
) -> Result<(), WikiError> {
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = index_dir.open_with(temporary, &options).map_err(|error| {
        WikiError::Io(format!(
            "cannot create capability-pinned shared index: {error}"
        ))
    })?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = index_dir.remove_file(temporary);
        return Err(WikiError::Io(format!(
            "cannot persist capability-pinned shared index: {error}"
        )));
    }
    Ok(())
}

#[cfg(test)]
fn activate_shared_index(
    index_dir: &Dir,
    index_name: &OsStr,
    temporary: &OsStr,
    backup: &OsStr,
    expected: Option<SharedIndexIdentity>,
) -> Result<(), WikiError> {
    let staged = shared_index_identity(index_dir, temporary)?
        .ok_or_else(|| WikiError::Conflict("staged shared SQLite index disappeared".to_owned()))?;
    if shared_index_identity(index_dir, index_name)? != expected {
        return Err(WikiError::Conflict(
            "shared SQLite index changed before capability publication".to_owned(),
        ));
    }
    if expected.is_some() {
        index_dir
            .rename(index_name, index_dir, backup)
            .map_err(|error| {
                WikiError::Conflict(format!("cannot claim current shared SQLite index: {error}"))
            })?;
        if shared_index_identity(index_dir, backup)? != expected {
            return Err(WikiError::Conflict(format!(
                "shared SQLite index changed during capability claim; recovery object retained at {}",
                backup.to_string_lossy()
            )));
        }
    }
    let activation = match injected_shared_replacement_failure() {
        Some(error) => Err(error),
        None => index_dir
            .hard_link(temporary, index_dir, index_name)
            .map_err(|error| WikiError::Io(error.to_string())),
    };
    if let Err(error) = activation {
        let cause = format!("cannot activate shared SQLite index: {error}");
        if expected.is_some() {
            return rollback_shared_index(index_dir, index_name, backup, &cause);
        }
        return Err(WikiError::Io(cause));
    }
    if shared_index_identity(index_dir, index_name)? != Some(staged) {
        return Err(WikiError::Conflict(format!(
            "activated shared SQLite index changed before verification; prior index retained at {}",
            backup.to_string_lossy()
        )));
    }
    let mut options = CapOpenOptions::new();
    options.read(true).write(true).follow(FollowSymlinks::No);
    let durability = index_dir
        .open_with(index_name, &options)
        .and_then(|file| file.sync_all())
        .map_err(|error| WikiError::Io(format!("cannot sync shared SQLite index: {error}")))
        .and_then(|()| sync_shared_index_directory(index_dir));
    if let Err(error) = durability {
        if expected.is_some() {
            return rollback_activated_shared_index(
                index_dir,
                index_name,
                backup,
                staged,
                &error.to_string(),
            );
        }
        if shared_index_identity(index_dir, index_name)? == Some(staged) {
            let _ = index_dir.remove_file(index_name);
        }
        return Err(error);
    }
    if expected.is_some() {
        if let Err(error) = index_dir.remove_file(backup) {
            return rollback_activated_shared_index(
                index_dir,
                index_name,
                backup,
                staged,
                &format!("cannot remove prior shared SQLite index backup: {error}"),
            );
        }
    }
    let _ = index_dir.remove_file(temporary);
    Ok(())
}

#[cfg(all(test, unix))]
fn sync_shared_index_directory(index_dir: &Dir) -> Result<(), WikiError> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    index_dir
        .open_with(".", &options)
        .map_err(|error| WikiError::Io(format!("cannot open shared index directory: {error}")))?
        .sync_all()
        .map_err(|error| WikiError::Io(format!("cannot sync shared index directory: {error}")))
}

#[cfg(all(test, not(unix)))]
#[allow(clippy::unnecessary_wraps)]
fn sync_shared_index_directory(_index_dir: &Dir) -> Result<(), WikiError> {
    Ok(())
}

#[cfg(test)]
fn rollback_activated_shared_index(
    index_dir: &Dir,
    index_name: &OsStr,
    backup: &OsStr,
    staged: SharedIndexIdentity,
    cause: &str,
) -> Result<(), WikiError> {
    if shared_index_identity(index_dir, index_name)? != Some(staged) {
        return Err(WikiError::Conflict(format!(
            "{cause}; rollback preserved a foreign destination and retained the prior index at {}",
            backup.to_string_lossy()
        )));
    }
    index_dir.remove_file(index_name).map_err(|error| {
        WikiError::Conflict(format!(
            "{cause}; cannot claim the activated shared index for rollback; prior index retained at {}: {error}",
            backup.to_string_lossy()
        ))
    })?;
    rollback_shared_index(index_dir, index_name, backup, cause)
}

#[cfg(test)]
fn rollback_shared_index(
    index_dir: &Dir,
    index_name: &OsStr,
    backup: &OsStr,
    cause: &str,
) -> Result<(), WikiError> {
    match index_dir.hard_link(backup, index_dir, index_name) {
        Ok(()) => {
            index_dir.remove_file(backup).map_err(|error| {
                WikiError::Io(format!(
                    "{cause}; prior index was restored but its backup cannot be removed: {error}"
                ))
            })?;
            Err(WikiError::Io(cause.to_owned()))
        }
        Err(rollback_error) => Err(WikiError::Conflict(format!(
            "{cause}; rollback preserved a foreign destination and retained the prior index at {}: {rollback_error}",
            backup.to_string_lossy()
        ))),
    }
}

#[cfg(test)]
thread_local! {
    static INJECT_SHARED_REPLACEMENT_FAILURE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn injected_shared_replacement_failure() -> Option<WikiError> {
    INJECT_SHARED_REPLACEMENT_FAILURE.with(|injected| {
        if injected.replace(false) {
            Some(WikiError::Io(
                "injected shared index replacement failure".to_owned(),
            ))
        } else {
            None
        }
    })
}

#[cfg(test)]
fn row_key(source_project: &str, page_id: &str) -> String {
    format!("{source_project}\u{1f}{page_id}")
}

fn valid_slug(value: &str) -> bool {
    let bytes = value.as_bytes();
    (2..=63).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

#[allow(clippy::needless_pass_by_value)]
#[cfg(test)]
fn sqlite_error(error: rusqlite::Error) -> WikiError {
    WikiError::Sqlite(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn page(id: &str, body: &str) -> String {
        format!(
            "---\nschema_version: 1\nid: {id}\nkind: concept\nsummary: {id} summary\ntags: [shared-test]\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: 2026-07-28T00:00:00Z\nupdated_at: 2026-07-28T00:00:00Z\n---\n\n{body}\n"
        )
    }

    fn write_page(root: &Path, id: &str, body: &str) {
        let wiki = root.join(WIKI_RELATIVE);
        fs::create_dir_all(&wiki).unwrap();
        fs::write(wiki.join(format!("{id}.md")), page(id, body)).unwrap();
    }

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let user = temporary.path().join("user");
        let first = temporary.path().join("first");
        let second = temporary.path().join("second");
        fs::create_dir_all(user.join(".hive/config")).unwrap();
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let user = user.canonicalize().unwrap();
        let first = first.canonicalize().unwrap();
        let second = second.canonicalize().unwrap();
        write_page(&user, "root-page", "root searchable");
        write_page(&first, "private-page", "private searchable");
        write_page(&second, "shared-page", "shared searchable");
        let registry = ProjectRegistry {
            schema_version: 1,
            projects: vec![
                RegisteredProject {
                    id: "first-project".to_owned(),
                    root: first.clone(),
                    enabled: true,
                    language: KnowledgeLanguage::En,
                    visibility: KnowledgeVisibility::ProjectPrivate,
                },
                RegisteredProject {
                    id: "second-project".to_owned(),
                    root: second.clone(),
                    enabled: true,
                    language: KnowledgeLanguage::Ko,
                    visibility: KnowledgeVisibility::Shared,
                },
            ],
        };
        fs::write(
            user.join(PROJECT_REGISTRY_RELATIVE),
            serde_yaml::to_string(&registry).unwrap(),
        )
        .unwrap();
        (temporary, user, first, second)
    }

    #[test]
    fn shared_rebuild_has_provenance_and_never_creates_project_databases() {
        let (_temporary, user, first, second) = fixture();
        let outcome = rebuild_shared_index(&user).unwrap();
        assert_eq!(outcome.page_count, 3);
        assert_eq!(outcome.project_count, 2);
        assert!(user.join(SHARED_INDEX_RELATIVE).is_file());
        assert!(!first.join(SHARED_INDEX_RELATIVE).exists());
        assert!(!second.join(SHARED_INDEX_RELATIVE).exists());
        let connection = Connection::open(user.join(SHARED_INDEX_RELATIVE)).unwrap();
        let row: (String, String, String, String) = connection
            .query_row(
                "SELECT c.source_project_id, d.kind, d.category, d.visibility
                 FROM documents d JOIN collections c
                   ON c.collection_id = d.collection_id
                 WHERE d.locator = '.hive/knowledge/Wiki/private-page.md'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            row,
            (
                "first-project".to_owned(),
                "concept".to_owned(),
                "concept".to_owned(),
                "project-private".to_owned()
            )
        );
        let private = query_shared(&user, Some(&first), Some("private"), None, 20).unwrap();
        assert_eq!(private[0].language, "en");
    }

    #[test]
    fn cross_project_and_global_queries_do_not_leak_private_rows() {
        let (_temporary, user, first, second) = fixture();
        rebuild_shared_index(&user).unwrap();
        let global = query_shared(&user, None, Some("searchable"), None, 20).unwrap();
        assert_eq!(
            global
                .iter()
                .map(|hit| hit.page_id.as_str())
                .collect::<Vec<_>>(),
            vec!["root-page", "shared-page"]
        );
        let from_first = query_shared(&user, Some(&first), Some("searchable"), None, 20).unwrap();
        assert!(from_first.iter().any(|hit| hit.page_id == "private-page"));
        let from_second = query_shared(&user, Some(&second), Some("searchable"), None, 20).unwrap();
        assert!(!from_second.iter().any(|hit| hit.page_id == "private-page"));
    }

    #[test]
    fn legacy_shared_query_returns_confidential_only_to_owning_project() {
        let (_temporary, user, first, _second) = fixture();
        let mut registry = load_project_registry(&user).unwrap();
        registry.projects[0].visibility = KnowledgeVisibility::Confidential;
        fs::write(
            user.join(PROJECT_REGISTRY_RELATIVE),
            serde_yaml::to_string(&registry).unwrap(),
        )
        .unwrap();
        rebuild_shared_index(&user).unwrap();

        assert_eq!(
            query_shared(&user, Some(&first), Some("private"), None, 20)
                .unwrap()
                .iter()
                .map(|hit| hit.page_id.as_str())
                .collect::<Vec<_>>(),
            vec!["private-page"]
        );
        assert!(query_shared(&user, None, Some("private"), None, 20)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn registry_rejects_duplicate_and_relative_roots() {
        let (_temporary, user, first, _second) = fixture();
        let duplicate = ProjectRegistry {
            schema_version: 1,
            projects: vec![
                RegisteredProject {
                    id: "same-project".to_owned(),
                    root: first.clone(),
                    enabled: true,
                    language: KnowledgeLanguage::Und,
                    visibility: KnowledgeVisibility::ProjectPrivate,
                },
                RegisteredProject {
                    id: "same-project".to_owned(),
                    root: first,
                    enabled: true,
                    language: KnowledgeLanguage::Und,
                    visibility: KnowledgeVisibility::ProjectPrivate,
                },
            ],
        };
        let bytes = serde_yaml::to_string(&duplicate).unwrap();
        assert!(validate_project_registry_bytes(&user, bytes.as_bytes()).is_err());
        let relative = b"schema_version: 1\nprojects:\n  - id: bad-project\n    root: relative\n    enabled: true\n    language: und\n    visibility: project-private\n";
        assert!(validate_project_registry_bytes(&user, relative).is_err());
    }

    #[test]
    fn registration_is_sorted_idempotent_and_rejects_root_collision() {
        let temporary = tempfile::tempdir().unwrap();
        let user = temporary.path().join("user");
        let first = temporary.path().join("first");
        let second = temporary.path().join("second");
        fs::create_dir_all(&user).unwrap();
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let user = user.canonicalize().unwrap();
        let first = first.canonicalize().unwrap();
        let second = second.canonicalize().unwrap();
        let second_entry = RegisteredProject {
            id: "zeta-project".to_owned(),
            root: second,
            enabled: true,
            language: KnowledgeLanguage::Ko,
            visibility: KnowledgeVisibility::Shared,
        };
        let first_entry = RegisteredProject {
            id: "alpha-project".to_owned(),
            root: first.clone(),
            enabled: true,
            language: KnowledgeLanguage::En,
            visibility: KnowledgeVisibility::ProjectPrivate,
        };
        assert_eq!(
            register_project(&user, second_entry).unwrap().changed_paths,
            vec![PROJECT_REGISTRY_RELATIVE]
        );
        assert_eq!(
            register_project(&user, first_entry.clone())
                .unwrap()
                .project_count,
            2
        );
        let registry = load_project_registry(&user).unwrap();
        assert_eq!(
            registry
                .projects
                .iter()
                .map(|project| project.id.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha-project", "zeta-project"]
        );
        assert!(register_project(&user, first_entry)
            .unwrap()
            .changed_paths
            .is_empty());
        let collision = RegisteredProject {
            id: "other-project".to_owned(),
            root: first,
            enabled: true,
            language: KnowledgeLanguage::Und,
            visibility: KnowledgeVisibility::Confidential,
        };
        assert!(register_project(&user, collision).is_err());
    }

    #[test]
    fn query_uses_the_published_generation_until_explicit_rebuild() {
        let (_temporary, user, _first, second) = fixture();
        rebuild_shared_index(&user).unwrap();
        write_page(&second, "shared-page", "changed searchable");
        assert!(query_shared(&user, None, Some("changed"), None, 20)
            .unwrap()
            .is_empty());
        rebuild_shared_index(&user).unwrap();
        assert_eq!(
            query_shared(&user, None, Some("changed"), None, 20)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn validation_rejects_visibility_tampering_with_retained_meta_digest() {
        let (_temporary, user, _first, _second) = fixture();
        let rebuilt = rebuild_shared_index(&user).unwrap();
        let connection = Connection::open(user.join(SHARED_INDEX_RELATIVE)).unwrap();
        connection
            .execute(
                "UPDATE documents SET visibility = 'shared'
                 WHERE locator = '.hive/knowledge/Wiki/private-page.md'",
                [],
            )
            .unwrap();
        let retained: String = connection
            .query_row("SELECT manifest_digest FROM meta", [], |row| row.get(0))
            .unwrap();
        assert_eq!(retained, rebuilt.logical_digest);
        drop(connection);

        assert!(matches!(
            validate_shared_index(&user),
            Err(WikiError::Verification(_))
        ));
        assert!(matches!(
            query_shared(&user, None, Some("private"), None, 20),
            Err(WikiError::Verification(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn page_replacement_with_external_symlink_is_rejected_at_open_barrier() {
        use std::os::unix::fs::symlink;

        let (temporary, user, _first, _second) = fixture();
        rebuild_legacy_shared_index_for_test(&user).unwrap();
        let index = user.join(SHARED_INDEX_RELATIVE);
        let index_before = fs::read(&index).unwrap();
        let root_page = user.join(WIKI_RELATIVE).join("root-page.md");
        let page_before = fs::read(&root_page).unwrap();
        let external = temporary.path().join("external.md");
        fs::write(&external, page("root-page", "external-only searchable")).unwrap();
        let raced_page = root_page.clone();
        INJECT_SHARED_PAGE_OPEN_RACE.with(|injected| {
            *injected.borrow_mut() = Some(Box::new(move || {
                fs::remove_file(&raced_page).unwrap();
                symlink(&external, &raced_page).unwrap();
            }));
        });

        assert!(matches!(
            rebuild_legacy_shared_index_for_test(&user),
            Err(WikiError::Verification(_))
        ));
        assert_eq!(fs::read(&index).unwrap(), index_before);

        fs::remove_file(&root_page).unwrap();
        fs::write(&root_page, page_before).unwrap();
        rebuild_shared_index(&user).unwrap();
        validate_shared_index(&user).unwrap();
        assert!(query_shared(&user, None, Some("external-only"), None, 20)
            .unwrap()
            .is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn registered_root_retarget_is_rejected_while_capability_is_pinned() {
        use std::os::unix::fs::symlink;

        let (temporary, user, first, _second) = fixture();
        rebuild_legacy_shared_index_for_test(&user).unwrap();
        let index = user.join(SHARED_INDEX_RELATIVE);
        let index_before = fs::read(&index).unwrap();
        let saved = temporary.path().join("first-saved");
        let external = temporary.path().join("external-project");
        fs::create_dir(&external).unwrap();
        write_page(&external, "private-page", "external-only searchable");
        let raced = first.clone();
        let saved_root = saved.clone();
        INJECT_SHARED_ROOT_OPEN_RACE.with(|injected| {
            *injected.borrow_mut() = Some(Box::new(move |path| {
                if path != raced {
                    return false;
                }
                fs::rename(&raced, &saved_root).unwrap();
                symlink(&external, &raced).unwrap();
                true
            }));
        });

        assert!(matches!(
            rebuild_legacy_shared_index_for_test(&user),
            Err(WikiError::Conflict(_))
        ));
        assert_eq!(fs::read(&index).unwrap(), index_before);

        fs::remove_file(&first).unwrap();
        fs::rename(saved, &first).unwrap();
        rebuild_shared_index(&user).unwrap();
        validate_shared_index(&user).unwrap();
        assert!(
            query_shared(&user, Some(&first), Some("external-only"), None, 20)
                .unwrap()
                .is_empty()
        );
    }

    #[cfg(unix)]
    #[test]
    fn registered_root_parent_retarget_is_rejected_before_ancestor_traversal() {
        let (temporary, user, _first, _second) = fixture();
        let controlled = temporary.path().join("controlled");
        let nested = controlled.join("nested-project");
        fs::create_dir_all(&nested).unwrap();
        write_page(&nested, "private-page", "private searchable");
        let nested = nested.canonicalize().unwrap();
        let mut registry = load_project_registry(&user).unwrap();
        registry.projects[0].root = nested.clone();
        fs::write(
            user.join(PROJECT_REGISTRY_RELATIVE),
            serde_yaml::to_string(&registry).unwrap(),
        )
        .unwrap();
        rebuild_legacy_shared_index_for_test(&user).unwrap();
        let index = user.join(SHARED_INDEX_RELATIVE);
        let index_before = fs::read(&index).unwrap();

        let saved = temporary.path().join("controlled-saved");
        let external_parent = temporary.path().join("external-parent");
        let external_project = external_parent.join("nested-project");
        fs::create_dir_all(&external_project).unwrap();
        write_page(
            &external_project,
            "private-page",
            "external-only searchable",
        );
        let raced = nested.clone();
        let controlled_path = controlled.clone();
        let saved_path = saved.clone();
        let external_restore = external_parent.clone();
        INJECT_SHARED_ANCESTOR_OPEN_RACE.with(|injected| {
            *injected.borrow_mut() = Some(Box::new(move |path| {
                if path != raced {
                    return false;
                }
                fs::rename(&controlled_path, &saved_path).unwrap();
                fs::rename(&external_parent, &controlled_path).unwrap();
                true
            }));
        });

        assert!(matches!(
            rebuild_legacy_shared_index_for_test(&user),
            Err(WikiError::Conflict(_))
        ));
        assert_eq!(fs::read(&index).unwrap(), index_before);

        fs::rename(&controlled, external_restore).unwrap();
        fs::rename(saved, &controlled).unwrap();
        rebuild_shared_index(&user).unwrap();
        validate_shared_index(&user).unwrap();
        assert!(
            query_shared(&user, Some(&nested), Some("external-only"), None, 20)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn opened_snapshot_is_independent_of_index_path_replacement() {
        let (_temporary, user, _first, second) = fixture();
        let first = rebuild_shared_index(&user).unwrap();
        let snapshot = fs::read(user.join(SHARED_INDEX_RELATIVE)).unwrap();
        let manifest: crate::rag::GenerationManifest = serde_json::from_slice(
            &fs::read(user.join(crate::store::RAG_MANIFEST_RELATIVE)).unwrap(),
        )
        .unwrap();
        let registry = RagStore::open(&user).unwrap().load_registry().unwrap();

        write_page(&second, "shared-page", "replacement searchable");
        let second = rebuild_shared_index(&user).unwrap();
        assert_ne!(first.logical_digest, second.logical_digest);

        let request = WikiPageQueryRequest {
            current_project_id: None,
            text: Some("replacement".to_owned()),
            tag: None,
            category: None,
            limit: 20,
        };
        assert!(
            crate::rag::query_wiki_pages_serialized(&snapshot, &manifest, &registry, &request)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            query_shared(&user, None, Some("replacement"), None, 20)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn consecutive_shared_rebuilds_replace_the_existing_index() {
        let (_temporary, user, _first, second) = fixture();
        let first = rebuild_shared_index(&user).unwrap();
        write_page(&second, "shared-page", "replacement searchable");
        let second = rebuild_shared_index(&user).unwrap();
        assert_ne!(first.logical_digest, second.logical_digest);
        assert_eq!(validate_shared_index(&user).unwrap(), second.logical_digest);
        assert_eq!(
            query_shared(&user, None, Some("replacement"), None, 20)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn identical_shared_rebuild_is_a_byte_exact_noop() {
        let (_temporary, user, _first, _second) = fixture();
        let first = rebuild_shared_index(&user).unwrap();
        let index = user.join(SHARED_INDEX_RELATIVE);
        let before = fs::read(&index).unwrap();

        let second = rebuild_shared_index(&user).unwrap();

        assert_eq!(second.logical_digest, first.logical_digest);
        assert!(second.changed_paths.is_empty());
        assert_eq!(fs::read(index).unwrap(), before);
    }

    #[test]
    fn legacy_v08_sqlite_migrates_to_rag_without_canonical_byte_change() {
        let (_temporary, user, _first, second) = fixture();
        rebuild_legacy_shared_index_for_test(&user).unwrap();
        let legacy = Connection::open(user.join(SHARED_INDEX_RELATIVE)).unwrap();
        let legacy_schema: String = legacy
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(legacy_schema, "2");
        drop(legacy);
        let canonical = [
            user.join(WIKI_RELATIVE).join("root-page.md"),
            second.join(WIKI_RELATIVE).join("shared-page.md"),
        ]
        .map(|path| (path.clone(), fs::read(path).unwrap()));

        rebuild_shared_index(&user).unwrap();

        let rag = Connection::open(user.join(SHARED_INDEX_RELATIVE)).unwrap();
        let schema: u32 = rag
            .query_row("SELECT schema_version FROM meta", [], |row| row.get(0))
            .unwrap();
        assert_eq!(schema, crate::rag::RAG_SCHEMA_VERSION);
        assert!(rag
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='pages'",
                [],
                |row| row.get::<_, String>(0),
            )
            .is_err());
        for (path, bytes) in canonical {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
        assert_eq!(
            query_shared(&user, None, Some("searchable"), None, 20)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn atomic_registration_can_commit_registry_and_shared_index_together() {
        let (_temporary, user, first, _second) = fixture();
        rebuild_shared_index(&user).unwrap();
        let updated = RegisteredProject {
            id: "first-project".to_owned(),
            root: first,
            enabled: true,
            language: KnowledgeLanguage::Both,
            visibility: KnowledgeVisibility::Shared,
        };
        let outcome = register_project_atomic(&user, updated, true).unwrap();
        assert_eq!(
            outcome.registry.changed_paths,
            vec![PROJECT_REGISTRY_RELATIVE]
        );
        let rebuilt = outcome.shared_index.unwrap();
        assert_eq!(
            validate_shared_index(&user).unwrap(),
            rebuilt.logical_digest
        );
        let registry = load_project_registry(&user).unwrap();
        assert_eq!(registry.projects[0].language, KnowledgeLanguage::Both);
        assert_eq!(registry.projects[0].visibility, KnowledgeVisibility::Shared);
    }

    #[test]
    fn disabled_shared_index_registration_removes_all_derived_state_atomically() {
        let (_temporary, user, first, _second) = fixture();
        rebuild_shared_index(&user).unwrap();
        for relative in SHARED_DERIVED_RELATIVES {
            let path = user.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            if !path.exists() {
                fs::write(path, b"derived\n").unwrap();
            }
        }
        assert!(matches!(
            validate_shared_derived_state_absent(&user),
            Err(WikiError::Verification(_))
        ));
        let page = user.join(WIKI_RELATIVE).join("root-page.md");
        let page_before = fs::read(&page).unwrap();
        let updated = RegisteredProject {
            id: "first-project".to_owned(),
            root: first,
            enabled: true,
            language: KnowledgeLanguage::Both,
            visibility: KnowledgeVisibility::ProjectPrivate,
        };

        let outcome = register_project_with_shared_index_disabled(&user, updated).unwrap();

        assert_eq!(
            outcome.registry.changed_paths,
            vec![PROJECT_REGISTRY_RELATIVE]
        );
        assert_eq!(
            outcome.removed_derived_paths,
            SHARED_DERIVED_RELATIVES.map(str::to_owned)
        );
        validate_shared_derived_state_absent(&user).unwrap();
        assert_eq!(fs::read(page).unwrap(), page_before);
        let registry = load_project_registry(&user).unwrap();
        assert_eq!(registry.projects[0].language, KnowledgeLanguage::Both);
    }
}
