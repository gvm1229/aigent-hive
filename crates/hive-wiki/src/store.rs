//! Persistent canonical store for the disposable collection RAG projection.

use crate::collection::{
    derive_collection_id, CollectionKind, CollectionRecord, CollectionRegistry,
    CollectionResolution, CollectionState, CollectionVisibility, COLLECTION_SCHEMA_VERSION,
    USER_ROOT_COLLECTION_ID,
};
use crate::rag::{
    build_rag_index, claim_digest, document_digest, parse_claim_markdown, plan_remember,
    render_claim_markdown, retrieve_serialized, AssertionStatus, CanonicalClaim, CanonicalDocument,
    ClaimKind, ClaimProvenance, GenerationManifest, RagError, RagIndexArtifact, RagLanguage,
    RagSnapshot, RagVisibility, RememberDisposition, RememberPlan, RememberRequest,
    RememberSourceKind, RetrievalRequest, RetrievalResult, RetrievalScope,
    MAX_SERIALIZED_INDEX_BYTES, RAG_SCHEMA_VERSION,
};
use crate::scan::{
    ClaimKind as ScanClaimKind, ReviewedClaim, ScanClaimMetadata, ScanPromotionStatus,
    ScanReviewStatus, ValidatedClaims, SCAN_SCHEMA_VERSION,
};
use crate::shared::SHARED_INDEX_RELATIVE;
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
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

/// User-root-local collection registry.
pub const COLLECTION_REGISTRY_RELATIVE: &str = ".hive/config/collections.yml";
/// Central canonical typed-claim directory.
pub const CLAIMS_RELATIVE: &str = ".hive/knowledge/Claims";
/// Persisted RAG generation manifest.
pub const RAG_MANIFEST_RELATIVE: &str = ".hive/index/rag-generation.json";
/// Canonical compact binding for the exact published manifest and disposable index.
pub const RAG_TRUST_RELATIVE: &str = ".hive/config/rag-trust.json";
/// Crash-recovery journal present only while canonical/derived state is dirty.
pub const RAG_DIRTY_RELATIVE: &str = ".hive/index/rag-dirty.json";

const WIKI_RELATIVE: &str = ".hive/knowledge/Wiki";
const MAX_REGISTRY_BYTES: usize = 1024 * 1024;
const MAX_CLAIM_BYTES: usize = 128 * 1024;
const MAX_WIKI_BYTES: usize = 2 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 128 * 1024;
pub(crate) const MAX_RAG_TRUST_BYTES: usize = 16 * 1024;
const MAX_DIRTY_BYTES: usize = 1024 * 1024;
const MAX_EXTERNAL_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
const DIRTY_SCHEMA_VERSION: u32 = 2;
const RAG_TRUST_SCHEMA_VERSION: u32 = 1;
const MAX_DIRTY_ENTRIES: usize = 4096;

static COLLECTION_INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Explicit, portable request to create or update one collection mapping.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollectionRegistration {
    /// Existing or imported stable collection ID. When present it is never re-derived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    /// Collection role. `user-root` is managed by [`RagStore::ensure_registry`].
    pub kind: CollectionKind,
    /// Current local attachment state.
    pub state: CollectionState,
    /// Human-facing aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Absolute attached root. It is local-only metadata and never an identity seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_locator: Option<PathBuf>,
    /// Legacy registered-project linkage, used for lookup but not identity derivation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project_id: Option<String>,
    /// Default visibility for canonical items in the collection.
    pub default_visibility: CollectionVisibility,
    /// Explicit provider-neutral logical identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portable_identity: Option<String>,
    /// Reviewed scan-inventory digest allowed to seed an initial collection ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_inventory_digest: Option<String>,
}

/// Net persistent changes from a canonical store operation.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StoreCommit {
    /// User-root-relative files whose final bytes changed.
    pub changed_paths: Vec<String>,
    /// Published derived generation.
    pub generation: u64,
    /// Published logical manifest digest.
    pub manifest_digest: String,
}

/// A verified external-canonical RAG generation plus its small local ledger.
///
/// The ledger is intentionally opaque to this generic store. Remote backends
/// validate its schema and bind it to the returned generation before using the
/// projection. It must never contain a second canonical content cache.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExternalIndexSnapshot {
    /// Exact persisted remote revision ledger bytes.
    pub ledger_bytes: Vec<u8>,
    /// Authenticated disposable RAG generation metadata.
    pub manifest: GenerationManifest,
    /// Authenticated disposable `SQLite` projection bytes.
    pub sqlite_bytes: Vec<u8>,
}

/// Collection registration plus rebuilt-index evidence.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollectionCommit {
    /// Stable collection record after canonicalization.
    pub collection: CollectionRecord,
    /// Atomic store commit.
    pub store: StoreCommit,
}

/// Atomic user-root promotion of one exact reviewed scan claim.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScanPromotionCommit {
    /// Source collection claim after its promotion state update.
    pub source_claim: CanonicalClaim,
    /// User-root canonical claim created or matched idempotently.
    pub promoted_claim: CanonicalClaim,
    /// One atomic store commit covering both canonical claims and derived state.
    pub store: StoreCommit,
}

/// Atomic automatic promotion batch derived from one reviewed source collection.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AutomaticScanPromotionCommit {
    /// Source claims whose automatic-promotion lifecycle advanced.
    pub source_claims: Vec<CanonicalClaim>,
    /// User-root shared claims created or matched without user interruption.
    pub promoted_claims: Vec<CanonicalClaim>,
    /// One atomic store commit covering all recorded promotion decisions.
    pub store: StoreCommit,
}

/// Canonical project ledger plus the atomic normalized-store commit derived from it.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProjectRegistrationCommit {
    /// Exact canonical project registry after registration.
    pub registry: crate::shared::ProjectRegistry,
    /// One transaction covering project and collection registries plus derived state.
    pub store: StoreCommit,
}

/// Persisted recovery state. Canonical files remain authoritative.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PersistentDirtyState {
    /// Contract version.
    pub schema_version: u32,
    /// Last published generation, or zero before the first projection.
    pub base_generation: u64,
    /// Manifest digest at `base_generation`, or the deterministic empty sentinel.
    pub base_manifest_digest: String,
    /// Intended new generation.
    pub target_generation: u64,
    /// Exact canonical files changed by the interrupted operation.
    pub entries: Vec<PersistentDirtyEntry>,
}

/// One digest-bound canonical write in a persistent dirty journal.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct PersistentDirtyEntry {
    /// User-root-relative canonical path.
    pub locator: String,
    /// Digest of intended canonical bytes.
    pub target_digest: String,
    /// True when the intended final state is absence rather than an empty file.
    pub delete: bool,
}

/// Canonical trust root for one exact derived RAG generation.
///
/// This compact file is independently owned under `.hive/config`. It contains no
/// knowledge and can be recreated only by rebuilding from canonical sources.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct RagTrustBinding {
    schema_version: u32,
    generation: u64,
    logical_digest: String,
    entry_count: usize,
    sqlite_digest: String,
    manifest_digest: String,
}

/// Capability-pinned user-root RAG store.
pub struct RagStore {
    root_path: PathBuf,
    root: Dir,
}

/// Exclusive capability-backed lock shared by canonical knowledge mutations and
/// short-lived authorization state transitions.
pub struct CanonicalKnowledgeLock {
    _inner: crate::CapabilityKnowledgeLock,
}

/// Exclusive user-root lock covering one complete shared canonical mutation and rebuild.
pub struct SharedKnowledgeOperationLock {
    _inner: crate::CapabilityKnowledgeLock,
}

impl RagStore {
    /// Pin an existing user root without creating or changing any files.
    ///
    /// # Errors
    ///
    /// Returns an error when the root is missing, non-absolute, changes identity,
    /// or contains a symlink or non-directory component.
    pub fn open(user_root: &Path) -> Result<Self, WikiError> {
        let root_path = crate::shared::canonical_root(user_root)?;
        let root = pin_absolute_directory(&root_path, "user root")?;
        Ok(Self { root_path, root })
    }

    pub(crate) fn from_pinned(root_path: PathBuf, root: Dir) -> Self {
        Self { root_path, root }
    }

    /// Load the strict local collection registry without scanning collection content.
    ///
    /// # Errors
    ///
    /// Returns an error when the registry is missing, unsafe, oversized, malformed,
    /// non-canonical, or inconsistent with the pinned user root.
    pub fn load_registry(&self) -> Result<CollectionRegistry, WikiError> {
        self.load_registry_snapshot().map(|(registry, _)| registry)
    }

    /// Load one exact registry byte snapshot and its digest.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::load_registry`].
    pub fn load_registry_snapshot(&self) -> Result<(CollectionRegistry, String), WikiError> {
        let bytes = read_bounded_required(
            &self.root,
            Path::new(COLLECTION_REGISTRY_RELATIVE),
            MAX_REGISTRY_BYTES,
            "collection registry",
        )?;
        let digest = sha256_digest(&bytes);
        self.parse_registry(&bytes)
            .map(|registry| (registry, digest))
    }

    /// Acquire the same exclusive lock used by canonical knowledge publication.
    ///
    /// Authorization issuers and consumers use this only for their bounded runtime
    /// record transitions and release it before invoking a canonical store mutation.
    ///
    /// # Errors
    ///
    /// Returns an error when the lock cannot be created or acquired within the bound.
    pub fn acquire_authorization_lock(&self) -> Result<CanonicalKnowledgeLock, WikiError> {
        crate::CapabilityKnowledgeLock::acquire(&self.root)
            .map(|inner| CanonicalKnowledgeLock { _inner: inner })
    }

    /// Serialize a complete shared mutation across preparation, canonical writes, and rebuild.
    ///
    /// This outer lock is distinct from the inner publication lock, so callers may safely hold
    /// it while invoking store operations that acquire the canonical publication lock.
    ///
    /// # Errors
    ///
    /// Returns an error when the operation lock cannot be created or acquired within the bound.
    pub fn acquire_shared_operation_lock(&self) -> Result<SharedKnowledgeOperationLock, WikiError> {
        crate::CapabilityKnowledgeLock::acquire_shared_operation(&self.root)
            .map(|inner| SharedKnowledgeOperationLock { _inner: inner })
    }

    /// Ensure the user-root collection, registry, generation state, and RAG index exist.
    ///
    /// # Errors
    ///
    /// Returns an error for unsafe or conflicting state, a prior dirty journal,
    /// invalid canonical Markdown, failed index construction, or failed rollback.
    pub fn ensure_registry(&self) -> Result<StoreCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let prior = self.load_registry_optional()?;
        let registry = match prior {
            Some(registry) => registry,
            None => self.default_registry()?,
        };
        if self.index_state_is_complete()? && self.load_registry_optional()?.is_some() {
            return self.current_commit();
        }
        let bytes = registry_bytes(&registry)?;
        self.commit_canonical_writes_locked(vec![(
            PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
            bytes,
        )])
    }

    /// Synchronize the legacy project registry into the normalized collection registry.
    ///
    /// Enabled projects become attached registered-project collections. Disabled or
    /// removed projects retain their stable identity as detached collections so private
    /// knowledge cannot leak through automatic retrieval. All rows publish in one
    /// canonical transaction and one derived-index generation.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid project registry, unsafe project root, identity
    /// conflict, dirty store, failed publication, or failed rollback.
    pub fn sync_project_registry(
        &self,
        projects: &crate::shared::ProjectRegistry,
    ) -> Result<StoreCommit, WikiError> {
        let encoded = serde_yaml::to_string(projects).map_err(|error| {
            WikiError::InvalidInput(format!("invalid project registry: {error}"))
        })?;
        let projects =
            crate::shared::validate_project_registry_bytes(&self.root_path, encoded.as_bytes())?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let prior = self.load_registry_optional()?;
        let registry = self.synchronize_project_collections(&projects, prior.clone())?;
        if prior.as_ref() == Some(&registry) && self.index_state_is_complete()? {
            return self.current_commit();
        }
        self.commit_canonical_writes_locked(vec![(
            PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
            registry_bytes(&registry)?,
        )])
    }

    /// Register one project and atomically synchronize both canonical registries and the index.
    ///
    /// One capability lock covers `projects.yml`, `collections.yml`, the dirty journal, the
    /// normalized `SQLite` projection, and its generation manifest. Re-registering an identical
    /// row is a byte-exact no-op when the current derived state passes the fast schema probe.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid or colliding project, unsafe root, malformed canonical
    /// registry, dirty store, failed index construction, or incomplete rollback.
    pub fn register_project_atomic(
        &self,
        mut project: crate::shared::RegisteredProject,
    ) -> Result<ProjectRegistrationCommit, WikiError> {
        project.root = crate::shared::canonical_root(&project.root)?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;

        let prior_project_bytes = read_bounded_optional(
            &self.root,
            Path::new(crate::shared::PROJECT_REGISTRY_RELATIVE),
            MAX_REGISTRY_BYTES,
            "project registry",
        )?;
        let mut projects = match prior_project_bytes.as_deref() {
            Some(bytes) => crate::shared::validate_project_registry_bytes(&self.root_path, bytes)?,
            None => crate::shared::ProjectRegistry {
                schema_version: 1,
                projects: Vec::new(),
            },
        };
        if let Some(existing) = projects
            .projects
            .iter_mut()
            .find(|existing| existing.id == project.id)
        {
            *existing = project;
        } else {
            projects.projects.push(project);
        }
        projects
            .projects
            .sort_by(|left, right| left.id.cmp(&right.id));
        let project_bytes = serde_yaml::to_string(&projects)
            .map_err(|error| WikiError::Io(format!("cannot serialize project registry: {error}")))?
            .into_bytes();
        let projects =
            crate::shared::validate_project_registry_bytes(&self.root_path, &project_bytes)?;

        let prior_collections = self.load_registry_optional()?;
        let collections =
            self.synchronize_project_collections(&projects, prior_collections.clone())?;
        let collection_bytes = registry_bytes(&collections)?;
        if prior_project_bytes.as_deref() == Some(project_bytes.as_slice())
            && prior_collections.as_ref() == Some(&collections)
            && self.index_state_is_complete()?
        {
            return Ok(ProjectRegistrationCommit {
                registry: projects,
                store: self.current_commit()?,
            });
        }
        let store = self.commit_canonical_writes_locked(vec![
            (
                PathBuf::from(crate::shared::PROJECT_REGISTRY_RELATIVE),
                project_bytes,
            ),
            (
                PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
                collection_bytes,
            ),
        ])?;
        Ok(ProjectRegistrationCommit {
            registry: projects,
            store,
        })
    }

    /// Create or update a registered, directory, imported, or detached collection.
    ///
    /// The local locator is canonicalized and capability-verified, but never contributes
    /// to the ID. Reusing `collection_id` or `source_project_id` stabilizes the mapping
    /// after the initial portable identity or reviewed inventory digest seed.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid identity seed, locator, mapping conflict,
    /// dirty store, invalid canonical source, failed publication, or failed rollback.
    pub fn register_collection(
        &self,
        registration: CollectionRegistration,
    ) -> Result<CollectionCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let prior = self
            .load_registry_optional()?
            .unwrap_or(self.default_registry()?);
        let (registry, collection) =
            self.plan_collection_registration(prior.clone(), registration)?;
        if registry == prior {
            return Ok(CollectionCommit {
                collection,
                store: self.current_commit()?,
            });
        }
        let bytes = registry_bytes(&registry)?;
        let store = self.commit_canonical_writes_locked(vec![(
            PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
            bytes,
        )])?;
        Ok(CollectionCommit { collection, store })
    }

    /// Register a scanned directory and apply its complete reviewed claim set in one transaction.
    ///
    /// The first registration receives a generated portable instance identity independent of
    /// directory contents. A later scan of the same attached locator reuses the persisted ID;
    /// moving a collection requires its explicit ID through [`Self::set_collection_attachment`].
    ///
    /// # Errors
    ///
    /// Returns an error for a stale inventory binding, invalid mapping or claims, dirty state,
    /// failed index construction, publication failure, or incomplete rollback.
    pub fn register_scanned_collection_atomic(
        &self,
        registration: CollectionRegistration,
        validated: &ValidatedClaims,
    ) -> Result<CollectionCommit, WikiError> {
        validate_reviewed_claims_for_apply(validated)?;
        if registration.reviewed_inventory_digest.as_deref()
            != Some(validated.inventory_digest.as_str())
        {
            return Err(WikiError::Conflict(
                "scan registration inventory digest differs from reviewed claims".to_owned(),
            ));
        }
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let prior = self
            .load_registry_optional()?
            .unwrap_or(self.default_registry()?);
        let (registry, collection) =
            self.plan_collection_registration(prior.clone(), registration)?;
        let target_generation = self.next_generation()?;
        let mut writes = self.plan_reviewed_claim_writes_locked(
            &registry,
            &collection.collection_id,
            validated,
            target_generation,
        )?;
        if registry != prior {
            writes.insert(
                PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
                registry_bytes(&registry)?,
            );
        }
        let store = if writes.is_empty() {
            self.current_commit()?
        } else {
            self.commit_canonical_writes_locked(writes.into_iter().collect())?
        };
        Ok(CollectionCommit { collection, store })
    }

    /// Create or refresh the collection mapping for an existing project-registry row.
    ///
    /// The legacy project ID is retained only as linkage and alias. A new collection
    /// still requires a portable logical identity or reviewed inventory digest.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid or unavailable project root, missing initial
    /// identity seed, conflicting mapping, dirty state, publication failure, or rollback.
    pub fn register_project_collection(
        &self,
        project: &crate::shared::RegisteredProject,
        portable_identity: Option<String>,
        reviewed_inventory_digest: Option<String>,
    ) -> Result<CollectionCommit, WikiError> {
        if !project.enabled {
            return Err(WikiError::InvalidInput(
                "disabled project registrations cannot be activated as RAG collections".to_owned(),
            ));
        }
        self.register_collection(CollectionRegistration {
            collection_id: None,
            kind: CollectionKind::RegisteredProject,
            state: CollectionState::Attached,
            aliases: vec![project.id.clone()],
            local_locator: Some(project.root.clone()),
            source_project_id: Some(project.id.clone()),
            default_visibility: match project.visibility {
                crate::shared::KnowledgeVisibility::Shared => CollectionVisibility::Shared,
                crate::shared::KnowledgeVisibility::ProjectPrivate => {
                    CollectionVisibility::ProjectPrivate
                }
                crate::shared::KnowledgeVisibility::Confidential => {
                    CollectionVisibility::Confidential
                }
            },
            portable_identity,
            reviewed_inventory_digest,
        })
    }

    /// Attach or detach an existing collection without changing its identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the collection is unknown, user-root, concurrently
    /// changed, dirty, or the new locator cannot be capability-pinned.
    pub fn set_collection_attachment(
        &self,
        collection_id: &str,
        local_locator: Option<&Path>,
        expected_registry_digest: &str,
    ) -> Result<CollectionCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let current_registry_bytes = read_bounded_required(
            &self.root,
            Path::new(COLLECTION_REGISTRY_RELATIVE),
            MAX_REGISTRY_BYTES,
            "collection registry",
        )?;
        if sha256_digest(&current_registry_bytes) != expected_registry_digest {
            return Err(WikiError::Conflict(
                "collection registry changed after mapping authorization".to_owned(),
            ));
        }
        let mut registry = self.parse_registry(&current_registry_bytes)?;
        let existing = registry
            .collections
            .iter_mut()
            .find(|collection| collection.collection_id == collection_id)
            .ok_or_else(|| {
                WikiError::InvalidInput(format!("unknown collection `{collection_id}`"))
            })?;
        if existing.kind == CollectionKind::UserRoot {
            return Err(WikiError::Conflict(
                "the user-root collection cannot be detached".to_owned(),
            ));
        }
        existing.state = if local_locator.is_some() {
            CollectionState::Attached
        } else {
            CollectionState::Detached
        };
        existing.local_locator = match local_locator {
            Some(locator) => {
                let canonical = crate::shared::canonical_root(locator)?;
                let _pinned = pin_absolute_directory(&canonical, "collection root")?;
                Some(canonical.display().to_string())
            }
            None => None,
        };
        registry = self.canonicalize_registry(&registry)?;
        Self::validate_unique_locators(&registry)?;
        let collection = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == collection_id)
            .ok_or_else(|| WikiError::Io("canonical registry lost the collection".to_owned()))?
            .clone();
        let store = self.commit_canonical_writes_locked(vec![(
            PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
            registry_bytes(&registry)?,
        )])?;
        Ok(CollectionCommit { collection, store })
    }

    /// Load all canonical typed claims and compatible existing Wiki pages.
    ///
    /// # Errors
    ///
    /// Returns an error for a zero generation, invalid registry, unavailable attached
    /// root, unsafe canonical entry, malformed Markdown, or duplicate stable ID.
    pub fn load_canonical_snapshot(&self, generation: u64) -> Result<RagSnapshot, WikiError> {
        self.load_canonical_snapshot_excluding(generation, &BTreeSet::new())
    }

    fn load_canonical_collection_snapshot(
        &self,
        generation: u64,
        collection: &str,
    ) -> Result<RagSnapshot, WikiError> {
        let registry = self.load_registry()?;
        let claims =
            self.load_claims_for_collection(&registry, &BTreeSet::new(), Some(collection))?;
        let documents = self.load_documents_for_collection(&registry, Some(collection))?;
        Ok(RagSnapshot {
            schema_version: RAG_SCHEMA_VERSION,
            generation,
            registry,
            documents,
            claims,
        })
    }

    fn load_canonical_snapshot_excluding(
        &self,
        generation: u64,
        transient_claim_paths: &BTreeSet<PathBuf>,
    ) -> Result<RagSnapshot, WikiError> {
        if generation == 0 {
            return Err(WikiError::InvalidInput(
                "RAG generation must be greater than zero".to_owned(),
            ));
        }
        let registry = self.load_registry()?;
        let claims = self.load_claims_excluding(&registry, transient_claim_paths)?;
        let documents = self.load_documents(&registry)?;
        Ok(RagSnapshot {
            schema_version: RAG_SCHEMA_VERSION,
            generation,
            registry,
            documents,
            claims,
        })
    }

    /// Atomically write one inference-free remember plan and publish its rebuilt index.
    ///
    /// # Errors
    ///
    /// Returns an error for a malformed or stale plan, unknown collection, unsafe
    /// canonical source, dirty store, failed index publication, or failed rollback.
    pub fn apply_remember_plan(&self, plan: &RememberPlan) -> Result<StoreCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        validate_remember_plan_shape(plan)?;
        if plan.disposition == RememberDisposition::Noop {
            return self.current_commit();
        }
        let registry = self.load_registry()?;
        let known = registry
            .collections
            .iter()
            .map(|collection| collection.collection_id.as_str())
            .collect::<BTreeSet<_>>();
        let current = self
            .load_claims(&registry)?
            .into_iter()
            .map(|claim| (claim.claim_id.clone(), claim))
            .collect::<BTreeMap<_, _>>();
        let mut writes = BTreeMap::new();
        let mut rewritten_ids = plan
            .superseded_claims
            .iter()
            .map(|claim| claim.claim_id.clone())
            .collect::<Vec<_>>();
        rewritten_ids.sort();
        rewritten_ids.dedup();
        if rewritten_ids.len() != plan.superseded_claims.len()
            || plan.new_claim.as_ref().is_some_and(|claim| {
                let mut supersedes = claim.supersedes.clone();
                supersedes.sort();
                supersedes.dedup();
                supersedes != rewritten_ids
            })
        {
            return Err(WikiError::InvalidInput(
                "remember plan supersedes do not match its rewritten claims".to_owned(),
            ));
        }
        let replacement_id = plan.new_claim.as_ref().map(|claim| claim.claim_id.as_str());
        if let Some(new_claim) = &plan.new_claim {
            validate_store_claim(new_claim, &known)?;
            if let Some(existing) = current.get(&new_claim.claim_id) {
                if existing != new_claim {
                    return Err(WikiError::Conflict(format!(
                        "claim `{}` changed after remember planning",
                        new_claim.claim_id
                    )));
                }
            } else {
                insert_claim_write(&mut writes, new_claim)?;
            }
        }
        for rewritten in &plan.superseded_claims {
            validate_store_claim(rewritten, &known)?;
            if rewritten.status != AssertionStatus::Superseded {
                return Err(WikiError::InvalidInput(
                    "superseded claim rewrites must have superseded status".to_owned(),
                ));
            }
            let existing = current.get(&rewritten.claim_id).ok_or_else(|| {
                WikiError::Conflict(format!(
                    "claim `{}` disappeared after remember planning",
                    rewritten.claim_id
                ))
            })?;
            if existing.status == AssertionStatus::Superseded
                || existing.claim_key != rewritten.claim_key
                || existing.collection_id != rewritten.collection_id
            {
                return Err(WikiError::Conflict(format!(
                    "claim `{}` is not the planned active current truth",
                    rewritten.claim_id
                )));
            }
            let mut expected = existing.clone();
            expected.status = AssertionStatus::Superseded;
            expected.replacement = replacement_id.map(str::to_owned);
            expected.revision = rewritten.revision;
            expected.digest = claim_digest(&expected);
            if expected != *rewritten {
                return Err(WikiError::Conflict(format!(
                    "claim `{}` changed after supersede planning",
                    rewritten.claim_id
                )));
            }
            insert_claim_write(&mut writes, rewritten)?;
        }
        if writes.is_empty() {
            return self.current_commit();
        }
        self.commit_canonical_writes_locked(writes.into_iter().collect())
    }

    /// Persist the complete agent-reviewed scan claim set without mutating the scanned root.
    ///
    /// Active scan-owned claims absent from the new set are source-invalidated and removed
    /// from retrieval. Semantically unchanged claims retain their canonical bytes even when
    /// an unrelated file changes the enclosing inventory digest.
    ///
    /// # Errors
    ///
    /// Returns an error for unreviewed or malformed claims, an unknown collection,
    /// likely credentials, dirty state, publication failure, or rollback failure.
    #[allow(clippy::too_many_lines)]
    pub fn apply_reviewed_claims(
        &self,
        collection_id: &str,
        validated: &ValidatedClaims,
    ) -> Result<StoreCommit, WikiError> {
        validate_reviewed_claims_for_apply(validated)?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let registry = self.load_registry()?;
        let target_generation = self.next_generation()?;
        let writes = self.plan_reviewed_claim_writes_locked(
            &registry,
            collection_id,
            validated,
            target_generation,
        )?;
        if writes.is_empty() {
            return self.current_commit();
        }
        self.commit_canonical_writes_locked(writes.into_iter().collect())
    }

    /// Automatically promote every reviewed safe-general claim from one collection.
    ///
    /// The reviewed candidate flag is necessary but not sufficient: the claim must also satisfy
    /// the deterministic safe-general policy. Contradictory candidates are recorded as rejected
    /// rather than widening retrieval or leaving a repeated pending state.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown source collection, invalid reviewed policy, dirty state,
    /// failed canonical publication, or incomplete rollback.
    #[allow(clippy::too_many_lines)]
    pub fn auto_promote_reviewed_scan_claims_atomic(
        &self,
        source_collection_id: &str,
    ) -> Result<AutomaticScanPromotionCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let registry = self.load_registry()?;
        let source_collection = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == source_collection_id)
            .ok_or_else(|| {
                WikiError::InvalidInput(format!("unknown collection `{source_collection_id}`"))
            })?;
        if source_collection.kind == CollectionKind::UserRoot {
            return Err(WikiError::InvalidInput(
                "user-root claims cannot be scan promotion sources".to_owned(),
            ));
        }
        let target_generation = self.next_generation()?;
        let mut claims = self.load_claims(&registry)?;
        let candidates = claims
            .iter()
            .filter(|claim| is_pending_automatic_promotion(claim, source_collection_id))
            .cloned()
            .collect::<Vec<_>>();
        let reconciliation_sources = claims
            .iter()
            .filter(|claim| is_promoted_automatic_source(claim, source_collection_id))
            .cloned()
            .collect::<Vec<_>>();
        if candidates.is_empty() && reconciliation_sources.is_empty() {
            return Ok(AutomaticScanPromotionCommit {
                source_claims: Vec::new(),
                promoted_claims: Vec::new(),
                store: self.current_commit()?,
            });
        }
        let known = registry
            .collections
            .iter()
            .map(|collection| collection.collection_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut writes = BTreeMap::new();
        let mut source_claims = Vec::new();
        let mut promoted_claims = Vec::new();
        for source in candidates {
            validate_automatic_promotion_source(&source)?;
            let request = automatic_scan_promotion_request(&source)?;
            match plan_remember(&claims, &request, target_generation) {
                Ok(plan) => {
                    let promoted_source = advance_scan_promotion_status(
                        source,
                        ScanPromotionStatus::Promoted,
                        target_generation,
                    )?;
                    let mut promoted = promoted_claim_from_plan(&plan, &claims, &request)?;
                    writes.extend(remember_plan_writes(&plan, &known)?);
                    apply_remember_plan_to_claims(&mut claims, &plan);
                    if plan.new_claim.is_some() {
                        promoted
                            .scan_metadata
                            .clone_from(&promoted_source.scan_metadata);
                        promoted.revision = target_generation;
                        promoted.digest = claim_digest(&promoted);
                        writes.remove(&PathBuf::from(&promoted.locator));
                        insert_claim_write(&mut writes, &promoted)?;
                        replace_loaded_claim(&mut claims, promoted.clone());
                    }
                    promoted_claims.push(promoted);
                    insert_claim_write(&mut writes, &promoted_source)?;
                    replace_loaded_claim(&mut claims, promoted_source.clone());
                    source_claims.push(promoted_source);
                }
                Err(RagError::Conflict(_)) => {
                    let rejected_source = advance_scan_promotion_status(
                        source,
                        ScanPromotionStatus::Rejected,
                        target_generation,
                    )?;
                    insert_claim_write(&mut writes, &rejected_source)?;
                    replace_loaded_claim(&mut claims, rejected_source.clone());
                    source_claims.push(rejected_source);
                }
                Err(error) => return Err(rag_error(error)),
            }
        }
        // Earlier development builds wrote the derived user-root claim before copying the
        // source's final promotion metadata. Repair that harmless projection drift during the
        // same silent scan-maintenance path; retrieval remains strictly read-only.
        for source in reconciliation_sources {
            let source_metadata = source.scan_metadata.as_ref().ok_or_else(|| {
                WikiError::Verification("promoted automatic source lost typed metadata".to_owned())
            })?;
            let stale_derivatives = claims
                .iter()
                .filter(|claim| {
                    claim.collection_id == USER_ROOT_COLLECTION_ID
                        && claim.status != AssertionStatus::Superseded
                        && claim.provenance.source_kind == RememberSourceKind::ReviewedArtifact
                        && claim.provenance.locator == source.locator
                        && claim.scan_metadata.as_ref().is_some_and(|metadata| {
                            metadata.review_id == source_metadata.review_id
                                && metadata.promotion_status != ScanPromotionStatus::Promoted
                        })
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut derivative in stale_derivatives {
                derivative.scan_metadata = Some(source_metadata.clone());
                derivative.revision = target_generation;
                derivative.digest = claim_digest(&derivative);
                insert_claim_write(&mut writes, &derivative)?;
                replace_loaded_claim(&mut claims, derivative);
            }
        }
        if writes.is_empty() {
            return Ok(AutomaticScanPromotionCommit {
                source_claims,
                promoted_claims,
                store: self.current_commit()?,
            });
        }
        let store = self.commit_canonical_writes_locked(writes.into_iter().collect())?;
        Ok(AutomaticScanPromotionCommit {
            source_claims,
            promoted_claims,
            store,
        })
    }

    fn plan_reviewed_claim_writes_locked(
        &self,
        registry: &CollectionRegistry,
        collection_id: &str,
        validated: &ValidatedClaims,
        target_generation: u64,
    ) -> Result<BTreeMap<PathBuf, Vec<u8>>, WikiError> {
        let collection = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == collection_id)
            .ok_or_else(|| {
                WikiError::InvalidInput(format!("unknown collection `{collection_id}`"))
            })?;
        if collection.kind == CollectionKind::UserRoot {
            return Err(WikiError::InvalidInput(
                "reviewed directory claims require a non-user-root collection".to_owned(),
            ));
        }
        let mut claims = self.load_claims(registry)?;
        let mut writes = BTreeMap::new();
        let reviewed_ids = validated
            .collection_claims
            .iter()
            .map(|claim| claim.claim_id.clone())
            .collect::<BTreeSet<_>>();
        plan_reviewed_claim_upserts(
            collection,
            &validated.collection_claims,
            &validated.inventory_digest,
            target_generation,
            &mut claims,
            &mut writes,
        )?;
        plan_source_invalidations(
            collection_id,
            &reviewed_ids,
            &validated.inventory_digest,
            target_generation,
            &mut claims,
            &mut writes,
        )?;
        Ok(writes)
    }

    /// Promote one exact reviewed scan claim into user-root knowledge atomically.
    ///
    /// `expected_source_digest` is a compare-and-swap token from the current preview. The
    /// user-root request must preserve the source fact, kind, status, and review provenance;
    /// only its stable key and canonical locator may be selected by the caller.
    ///
    /// # Errors
    ///
    /// Returns an error for a stale source digest, non-candidate or rejected review, widened
    /// content, invalid user-root request, dirty state, failed publication, or rollback failure.
    pub fn promote_reviewed_scan_claim_atomic(
        &self,
        source_collection_id: &str,
        review_id: &str,
        expected_source_digest: &str,
        user_root_request: &RememberRequest,
    ) -> Result<ScanPromotionCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let registry = self.load_registry()?;
        let claims = self.load_claims(&registry)?;
        let source = resolve_scan_promotion_source(
            &claims,
            source_collection_id,
            review_id,
            expected_source_digest,
        )?;
        validate_scan_promotion_request(&source, user_root_request)?;
        let target_generation = self.next_generation()?;
        let remember_plan =
            plan_remember(&claims, user_root_request, target_generation).map_err(rag_error)?;
        let promoted_claim = promoted_claim_from_plan(&remember_plan, &claims, user_root_request)?;
        let known = registry
            .collections
            .iter()
            .map(|collection| collection.collection_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut writes = remember_plan_writes(&remember_plan, &known)?;
        let mut source_claim = source;
        if source_claim
            .scan_metadata
            .as_ref()
            .is_some_and(|metadata| metadata.promotion_status == ScanPromotionStatus::PendingReview)
        {
            let metadata = source_claim.scan_metadata.as_mut().ok_or_else(|| {
                WikiError::Verification("resolved scan claim lost its metadata".to_owned())
            })?;
            metadata.promotion_status = ScanPromotionStatus::Promoted;
            source_claim.revision = target_generation;
            source_claim.digest = claim_digest(&source_claim);
            insert_claim_write(&mut writes, &source_claim)?;
        }
        let store = if writes.is_empty() {
            self.current_commit()?
        } else {
            self.commit_canonical_writes_locked(writes.into_iter().collect())?
        };
        Ok(ScanPromotionCommit {
            source_claim,
            promoted_claim,
            store,
        })
    }

    /// List active scan claims awaiting explicit user-root promotion review.
    ///
    /// # Errors
    ///
    /// Returns an error when the store is uninitialized, dirty, corrupt, or the source
    /// collection is unknown. This read path never repairs or creates store state.
    pub fn preview_reviewed_scan_promotions(
        &self,
        source_collection_id: &str,
    ) -> Result<Vec<CanonicalClaim>, WikiError> {
        self.preflight_initialized_snapshot()?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let registry = self.load_registry()?;
        if !registry
            .collections
            .iter()
            .any(|collection| collection.collection_id == source_collection_id)
        {
            return Err(WikiError::InvalidInput(format!(
                "unknown collection `{source_collection_id}`"
            )));
        }
        let mut candidates = self
            .load_claims(&registry)?
            .into_iter()
            .filter(|claim| {
                claim.collection_id == source_collection_id
                    && claim.status != AssertionStatus::Superseded
                    && claim.scan_metadata.as_ref().is_some_and(|metadata| {
                        metadata.review_status == ScanReviewStatus::AgentReviewed
                            && metadata.global_promotion_candidate
                            && metadata.promotion_status == ScanPromotionStatus::PendingReview
                    })
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            scan_review_id(left)
                .cmp(&scan_review_id(right))
                .then_with(|| left.claim_id.cmp(&right.claim_id))
        });
        Ok(candidates)
    }

    /// Rebuild the disposable `SQLite` database solely from canonical registry/Markdown.
    ///
    /// A prior dirty journal is preserved unless the rebuild and publication succeed.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid canonical state, unsafe paths, failed projection,
    /// concurrent replacement, or failed rollback.
    pub fn rebuild(&self) -> Result<StoreCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        let artifact = self.rebuild_artifact_locked()?;
        self.publish_rebuild_locked(&artifact)
    }

    fn rebuild_artifact_locked(&self) -> Result<RagIndexArtifact, WikiError> {
        let dirty = read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?;
        if dirty.is_some() {
            let generation = self.recovery_generation()?;
            let snapshot = self.load_canonical_snapshot(generation)?;
            return build_rag_index(&snapshot).map_err(rag_error);
        }
        if let Some(base) = self.load_manifest_repairable()? {
            let same_generation = self.load_canonical_snapshot(base.generation)?;
            let candidate = build_rag_index(&same_generation).map_err(rag_error)?;
            if candidate.manifest == base {
                return Ok(candidate);
            }
            let generation = base.generation.checked_add(1).ok_or_else(|| {
                WikiError::Conflict("RAG generation counter is exhausted".to_owned())
            })?;
            let snapshot = self.load_canonical_snapshot(generation)?;
            return build_rag_index(&snapshot).map_err(rag_error);
        }
        let snapshot = self.load_canonical_snapshot(1)?;
        build_rag_index(&snapshot).map_err(rag_error)
    }

    /// Persist fail-closed recovery evidence before an external canonical-only mutation.
    ///
    /// Each path is a safe logical locator and each byte vector is the exact intended content.
    /// An empty byte vector represents deletion. The caller must perform the canonical mutation
    /// only after this method succeeds, then call [`Self::rebuild`] to publish and clear the
    /// journal. An interrupted caller leaves retrieval fail-closed until that rebuild.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty or unsafe write set, an already dirty store, invalid
    /// generation lineage, concurrent replacement, or failed rollback.
    pub fn begin_external_canonical_mutation(
        &self,
        writes: &[(PathBuf, Vec<u8>)],
    ) -> Result<PersistentDirtyState, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        let dirty = self.dirty_state(self.next_generation()?, writes)?;
        validate_dirty_state(&dirty)?;
        let bytes = json_bytes(&dirty, "RAG dirty journal")?;
        let mut snapshots = [crate::CapabilityFileSnapshot::capture(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
        )?];
        crate::transactional_capability(&self.root, &mut snapshots, |snapshots| {
            snapshots[0].install_staged(&self.root, &bytes)?;
            Ok(())
        })?;
        Ok(dirty)
    }

    /// Remove this operation's recovery journal after its external canonical write rolled back.
    ///
    /// The exact serialized journal must still be present. A missing or replaced journal fails
    /// closed so this operation cannot erase another writer's recovery evidence.
    ///
    /// # Errors
    ///
    /// Returns an error when the journal is missing, differs from `expected`, is unsafe, or
    /// cannot be removed atomically.
    pub fn abort_external_canonical_mutation(
        &self,
        expected: &PersistentDirtyState,
    ) -> Result<(), WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        validate_dirty_state(expected)?;
        let expected_bytes = json_bytes(expected, "RAG dirty journal")?;
        let current_bytes = read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        .ok_or_else(|| {
            WikiError::Verification(
                "RAG dirty journal disappeared before rollback cleanup".to_owned(),
            )
        })?;
        if current_bytes != expected_bytes {
            return Err(WikiError::Verification(
                "RAG dirty journal changed before rollback cleanup".to_owned(),
            ));
        }
        let mut snapshots = [crate::CapabilityFileSnapshot::capture(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
        )?];
        crate::transactional_capability(&self.root, &mut snapshots, |snapshots| {
            snapshots[0].remove(&self.root)?;
            Ok(())
        })
    }

    /// Report whether an interrupted canonical mutation journal is present.
    ///
    /// # Errors
    ///
    /// Returns an error when the journal path is unsafe or cannot be inspected.
    pub fn is_dirty(&self) -> Result<bool, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        Ok(read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        .is_some())
    }

    /// Query canonical Wiki pages from one complete serialized RAG generation.
    ///
    /// One knowledge lock covers the registry, manifest, and `SQLite` reads so a
    /// concurrent rebuild can expose only its complete old or complete new generation.
    /// Canonical Markdown is never scanned or repaired by this read path.
    ///
    /// # Errors
    ///
    /// Returns an error when derived state is dirty, missing, stale, corrupt, or the
    /// request violates page-query filters, visibility, or result limits.
    pub fn query_wiki_pages(
        &self,
        request: &crate::rag::WikiPageQueryRequest,
    ) -> Result<Vec<crate::rag::WikiPageQueryHit>, WikiError> {
        self.preflight_initialized_snapshot()?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        if read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        .is_some()
        {
            return Err(WikiError::Verification(
                "RAG store is dirty; rebuild from canonical sources before querying Wiki pages"
                    .to_owned(),
            ));
        }
        let registry = self.load_registry()?;
        let manifest = self.load_manifest_required()?;
        let sqlite_bytes = read_bounded_required(
            &self.root,
            Path::new(SHARED_INDEX_RELATIVE),
            MAX_SERIALIZED_INDEX_BYTES,
            "RAG SQLite index",
        )?;
        crate::rag::query_wiki_pages_serialized(&sqlite_bytes, &manifest, &registry, request)
            .map_err(rag_error)
    }

    /// Retrieve from serialized `SQLite` plus the manifest without scanning Markdown.
    ///
    /// # Errors
    ///
    /// Returns an error when derived state is missing, dirty, stale, corrupt, or the
    /// request violates scope, visibility, query, top-k, or byte-budget constraints.
    pub fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            retrieve_serialized(bytes, manifest, registry, request)
        })
    }

    /// Export one explicitly authorized, current canonical partition.
    ///
    /// # Errors
    /// Rejects stale indexes, invalid scope, canonical drift, or oversized input.
    pub fn semantic_corpus(
        &self,
        request: &RetrievalRequest,
        visibility: RagVisibility,
    ) -> Result<crate::rag::SemanticCorpus, WikiError> {
        self.authorized_semantic_corpus(request, visibility, |_| Ok(()))
    }

    /// Consume caller-owned approval under the canonical lock only after partition freshness.
    /// The callback must not reacquire this store's knowledge lock.
    ///
    /// # Errors
    /// Rejects invalid snapshots, stale canonical content, and failed authorization.
    pub fn authorized_semantic_corpus(
        &self,
        request: &RetrievalRequest,
        visibility: RagVisibility,
        authorize: impl FnOnce(&str) -> Result<(), WikiError>,
    ) -> Result<crate::rag::SemanticCorpus, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            let collection = crate::rag::semantic_target_collection(registry, request)?;
            let states = crate::rag::semantic_partition_states_serialized(
                bytes, manifest, registry, request,
            )?;
            let digest = match states.into_iter().find(|state| {
                state.partition.collection_id == collection
                    && state.partition.visibility == visibility
            }) {
                Some(state) => state.digest,
                None => crate::rag::semantic_corpus_digest(registry, &collection, visibility, &[])?,
            };
            let authority_digest = semantic_authority_digest(registry, request).map_err(|_| {
                RagError::RepairRequired("semantic authority is unavailable".to_owned())
            })?;
            Ok((|| {
                self.require_current_partition(
                    manifest.generation,
                    &collection,
                    visibility,
                    &digest,
                )?;
                authorize(&manifest.logical_digest)?;
                let chunks = crate::rag::semantic_corpus_serialized(
                    bytes, manifest, registry, request, visibility,
                )
                .map_err(rag_error)?;
                Ok(crate::rag::SemanticCorpus {
                    generation: manifest.generation,
                    manifest_digest: manifest.logical_digest.clone(),
                    partition_digest: digest,
                    authority_digest,
                    chunks,
                })
            })())
        })?
    }

    fn require_current_partition(
        &self,
        generation: u64,
        collection: &str,
        visibility: RagVisibility,
        expected: &str,
    ) -> Result<(), WikiError> {
        let current = self
            .load_canonical_collection_snapshot(generation, collection)
            .and_then(|snapshot| {
                crate::rag::canonical_partition_digest(&snapshot, collection, visibility)
                    .map_err(rag_error)
            });
        if !current.is_ok_and(|digest| digest == expected) {
            return Err(WikiError::Verification(
                "canonical semantic partition is unavailable or changed".to_owned(),
            ));
        }
        Ok(())
    }

    /// Revalidate returned canonical citations against the same authorized RAG generation.
    ///
    /// # Errors
    /// Rejects stale generations, invalid identities, and changed returned canonical items.
    pub fn semantic_matches(
        &self,
        request: &RetrievalRequest,
        expected_manifest_digest: &str,
        matches: &[crate::rag::SemanticMatch],
    ) -> Result<RetrievalResult, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            if manifest.logical_digest != expected_manifest_digest {
                return Err(RagError::RepairRequired(
                    "semantic generation is stale".to_owned(),
                ));
            }
            let result = crate::rag::semantic_matches_serialized(
                bytes, manifest, registry, request, matches,
            )?;
            self.validate_semantic_hits(registry, &result.hits)
                .map_err(|_| {
                    RagError::RepairRequired(
                        "canonical semantic citations changed or are unavailable".to_owned(),
                    )
                })?;
            Ok(result)
        })
    }

    /// Enumerate only authorized vector partition identities.
    ///
    /// # Errors
    /// Rejects stale or invalid index authority.
    pub fn semantic_partitions(
        &self,
        request: &RetrievalRequest,
    ) -> Result<Vec<crate::rag::SemanticPartition>, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            crate::rag::semantic_partitions_serialized(bytes, manifest, registry, request)
        })
    }

    /// Capture authorized partition input digests without exporting any text.
    ///
    /// # Errors
    /// Rejects stale or invalid index authority.
    pub fn semantic_search_plan(
        &self,
        request: &RetrievalRequest,
    ) -> Result<crate::rag::SemanticSearchPlan, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            Ok(crate::rag::SemanticSearchPlan {
                manifest_digest: manifest.logical_digest.clone(),
                partitions: crate::rag::semantic_partition_states_serialized(
                    bytes, manifest, registry, request,
                )?,
            })
        })
    }

    /// FTS fallback with actual canonical verification of only the returned citations.
    ///
    /// # Errors
    /// Rejects stale returned items without scanning unrelated canonical collections.
    pub fn checked_retrieve(
        &self,
        request: &RetrievalRequest,
    ) -> Result<RetrievalResult, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            let result = retrieve_serialized(bytes, manifest, registry, request)?;
            self.validate_semantic_hits(registry, &result.hits)
                .map_err(|_| {
                    RagError::RepairRequired(
                        "canonical retrieval citations changed or are unavailable".to_owned(),
                    )
                })?;
            Ok(result)
        })
    }

    /// Fuse candidates in one current generation and verify every returned canonical item.
    ///
    /// # Errors
    /// Rejects generation races, invalid candidates, or stale canonical citations.
    pub fn hybrid_retrieve(
        &self,
        request: &RetrievalRequest,
        expected: &str,
        matches: &[crate::rag::SemanticMatch],
    ) -> Result<RetrievalResult, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            if manifest.logical_digest != expected {
                return Err(RagError::RepairRequired(
                    "retrieval generation changed during semantic search".to_owned(),
                ));
            }
            let mut expanded = request.clone();
            expanded.top_k = 100;
            expanded.byte_budget = 1024 * 1024;
            let prepared =
                crate::rag::PreparedRagIndex::from_serialized(bytes, manifest, registry)?;
            let lexical = prepared.retrieve(&expanded)?;
            let semantic = prepared.semantic_matches(&expanded, matches)?;
            let result = crate::rag::fuse_semantic_results(request, lexical, &semantic)?;
            self.validate_semantic_hits(registry, &result.hits)
                .map_err(|_| {
                    RagError::RepairRequired(
                        "canonical hybrid citations changed or are unavailable".to_owned(),
                    )
                })?;
            Ok(result)
        })
    }

    /// Publish only while this partition remains current; other collections do not invalidate it.
    ///
    /// # Errors
    /// Rejects input or authority drift and rolls back an intervening canonical edit.
    pub fn with_semantic_snapshot<T>(
        &self,
        request: &RetrievalRequest,
        visibility: RagVisibility,
        expected: &str,
        expected_authority: &str,
        publish: impl FnOnce() -> Result<T, WikiError>,
        rollback: impl FnOnce() -> Result<(), WikiError>,
    ) -> Result<T, WikiError> {
        self.with_retrieval_snapshot(|bytes, manifest, registry| {
            let collection = crate::rag::semantic_target_collection(registry, request)?;
            let chunks = crate::rag::semantic_corpus_serialized(
                bytes, manifest, registry, request, visibility,
            )?;
            if crate::rag::semantic_corpus_digest(registry, &collection, visibility, &chunks)?
                != expected
            {
                return Err(RagError::RepairRequired(
                    "semantic partition changed before publication".to_owned(),
                ));
            }
            Ok((|| {
                self.require_semantic_authority(request, expected_authority)?;
                self.require_current_partition(
                    manifest.generation,
                    &collection,
                    visibility,
                    expected,
                )?;
                let result = publish()?;
                if self
                    .require_current_partition(
                        manifest.generation,
                        &collection,
                        visibility,
                        expected,
                    )
                    .is_err()
                    || self
                        .require_semantic_authority(request, expected_authority)
                        .is_err()
                {
                    rollback()?;
                    return Err(WikiError::Verification(
                        "canonical partition changed during vector publication".to_owned(),
                    ));
                }
                Ok(result)
            })())
        })?
    }

    fn require_semantic_authority(
        &self,
        request: &RetrievalRequest,
        expected: &str,
    ) -> Result<(), WikiError> {
        if self
            .load_registry()
            .and_then(|registry| semantic_authority_digest(&registry, request))
            .is_ok_and(|digest| digest == expected)
        {
            return Ok(());
        }
        Err(WikiError::Verification(
            "semantic authority changed during operation".to_owned(),
        ))
    }

    fn validate_semantic_hits(
        &self,
        registry: &CollectionRegistry,
        hits: &[crate::rag::RetrievalHit],
    ) -> Result<(), WikiError> {
        let by_id = registry.by_id();
        let mut checked = BTreeSet::new();
        for hit in hits {
            if !checked.insert((&hit.collection_id, &hit.item_kind, &hit.item_id)) {
                continue;
            }
            let collection = by_id.get(hit.collection_id.as_str()).ok_or_else(|| {
                WikiError::Verification("canonical collection is absent".to_owned())
            })?;
            let locator = hit.locator.split('#').next().unwrap_or("");
            if hit.item_kind == "claim" {
                let expected = claim_locator(&hit.collection_id, &hit.item_id);
                if locator != expected {
                    return Err(WikiError::Verification(
                        "claim citation path changed".to_owned(),
                    ));
                }
                let bytes = read_bounded_required(
                    &self.root,
                    Path::new(&expected),
                    MAX_CLAIM_BYTES,
                    "canonical claim",
                )?;
                crate::reject_likely_credentials(&bytes)?;
                let claim = parse_claim_markdown(
                    &expected,
                    std::str::from_utf8(&bytes)
                        .map_err(|_| WikiError::Verification("claim is not UTF-8".to_owned()))?,
                )
                .map_err(rag_error)?;
                if claim.claim_id != hit.item_id
                    || claim.collection_id != hit.collection_id
                    || claim.digest != hit.digest
                    || claim.visibility != hit.visibility
                {
                    return Err(WikiError::Verification(
                        "canonical claim changed".to_owned(),
                    ));
                }
            } else if hit.item_kind == "document" {
                let relative = Path::new(locator)
                    .strip_prefix(WIKI_RELATIVE)
                    .map_err(|_| {
                        WikiError::Verification("document citation is outside Wiki".to_owned())
                    })?;
                let components = relative.components().collect::<Vec<_>>();
                if components.len() != 1
                    || !matches!(components[0], Component::Normal(_))
                    || relative.extension() != Some(OsStr::new("md"))
                {
                    return Err(WikiError::Verification(
                        "document citation is not one canonical page".to_owned(),
                    ));
                }
                let source = if collection.collection_id == USER_ROOT_COLLECTION_ID {
                    self.root
                        .try_clone()
                        .map_err(|error| WikiError::Io(error.to_string()))?
                } else {
                    pin_absolute_directory(
                        Path::new(collection.local_locator.as_deref().ok_or_else(|| {
                            WikiError::Verification("collection root unavailable".to_owned())
                        })?),
                        "collection root",
                    )?
                };
                let (parent, name) =
                    crate::capability_parent(&source, Path::new(WIKI_RELATIVE), false)?
                        .ok_or_else(|| WikiError::Verification("Wiki is absent".to_owned()))?;
                let wiki = parent
                    .open_dir_nofollow(&name)
                    .map_err(|error| WikiError::Io(error.to_string()))?;
                let document = load_wiki_document_file(&wiki, collection, relative.as_os_str())?;
                if document.document_id != hit.item_id
                    || document.digest != hit.digest
                    || document.visibility != hit.visibility
                {
                    return Err(WikiError::Verification(
                        "canonical document changed".to_owned(),
                    ));
                }
            } else {
                return Err(WikiError::Verification(
                    "unknown canonical item kind".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn with_retrieval_snapshot<T>(
        &self,
        operation: impl FnOnce(&[u8], &GenerationManifest, &CollectionRegistry) -> Result<T, RagError>,
    ) -> Result<T, WikiError> {
        self.preflight_initialized_snapshot()?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        if read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        .is_some()
        {
            return Err(WikiError::Verification(
                "RAG store is dirty; rebuild from canonical sources before retrieval".to_owned(),
            ));
        }
        let registry = self.load_registry()?;
        let manifest = self.load_manifest_required()?;
        let sqlite_bytes = read_bounded_required(
            &self.root,
            Path::new(SHARED_INDEX_RELATIVE),
            MAX_SERIALIZED_INDEX_BYTES,
            "RAG SQLite index",
        )?;
        operation(&sqlite_bytes, &manifest, &registry).map_err(rag_error)
    }

    /// Load one external-canonical ledger and its complete disposable projection.
    ///
    /// This method deliberately does not interpret the ledger: the backend owns
    /// its remote identity and revision contract. It does enforce the same
    /// no-follow paths, dirty-state barrier, manifest trust binding, and `SQLite`
    /// byte bounds used by the Markdown backend.
    ///
    /// # Errors
    ///
    /// Returns an error when the ledger path is unsafe, the projection is dirty,
    /// or the ledger, manifest, trust binding, or `SQLite` bytes are unavailable.
    pub fn load_external_index(
        &self,
        ledger_relative: &Path,
    ) -> Result<ExternalIndexSnapshot, WikiError> {
        self.load_external_index_optional(ledger_relative)?
            .ok_or_else(|| {
                WikiError::Verification(
                    "external canonical ledger is missing; complete remote refresh is required"
                        .to_owned(),
                )
            })
    }

    /// Load an external-canonical projection when its ledger has been initialized.
    ///
    /// A missing ledger is a normal pre-setup state. Any partially present or
    /// invalid generation still fails closed rather than being treated as empty.
    ///
    /// # Errors
    ///
    /// Returns an error when the ledger path is unsafe, the store is dirty, or
    /// a partially present ledger and projection cannot be verified.
    pub fn load_external_index_optional(
        &self,
        ledger_relative: &Path,
    ) -> Result<Option<ExternalIndexSnapshot>, WikiError> {
        validate_external_ledger_relative(ledger_relative)?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        if read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        .is_some()
        {
            return Err(WikiError::Verification(
                "RAG store is dirty; refresh the remote canonical source before retrieval"
                    .to_owned(),
            ));
        }
        let Some(ledger_bytes) = read_bounded_optional(
            &self.root,
            ledger_relative,
            MAX_EXTERNAL_CANONICAL_BYTES,
            "external canonical ledger",
        )?
        else {
            return Ok(None);
        };
        let manifest = self.load_manifest_required()?;
        let sqlite_bytes = read_bounded_required(
            &self.root,
            Path::new(SHARED_INDEX_RELATIVE),
            MAX_SERIALIZED_INDEX_BYTES,
            "RAG SQLite index",
        )?;
        Ok(Some(ExternalIndexSnapshot {
            ledger_bytes,
            manifest,
            sqlite_bytes,
        }))
    }

    /// Atomically publish one remote-canonical ledger and its derived index.
    ///
    /// The caller supplies an already validated, complete remote snapshot. The
    /// ledger is the only durable remote-source state; page content remains only
    /// in the disposable `SQLite` projection. A transaction first records the
    /// exact ledger write as dirty, then publishes `SQLite` and its manifest, and
    /// only clears dirty after all bytes are durable.
    ///
    /// # Errors
    ///
    /// Returns an error for an unsafe ledger path, malformed generation, dirty
    /// prior state, concurrent generation change, or failed atomic publication.
    pub fn publish_external_index(
        &self,
        ledger_relative: &Path,
        ledger_bytes: &[u8],
        artifact: &RagIndexArtifact,
    ) -> Result<StoreCommit, WikiError> {
        self.publish_external_index_inner(ledger_relative, ledger_bytes, artifact, false)
    }

    /// Publish a fully re-fetched remote-canonical generation over its own
    /// interrupted external write journal.
    ///
    /// The caller must have independently validated a complete remote inventory
    /// before invoking this recovery path. It cannot recover a Markdown or
    /// other local-canonical dirty journal, and it retains the original dirty
    /// bytes if the new publication cannot complete.
    ///
    /// # Errors
    ///
    /// Returns an error when the ledger path is unsafe, a nonmatching dirty
    /// journal exists, or the next generation cannot be recovered.
    pub fn publish_external_index_recovery(
        &self,
        ledger_relative: &Path,
        ledger_bytes: &[u8],
        artifact: &RagIndexArtifact,
    ) -> Result<StoreCommit, WikiError> {
        self.publish_external_index_inner(ledger_relative, ledger_bytes, artifact, true)
    }

    fn publish_external_index_inner(
        &self,
        ledger_relative: &Path,
        ledger_bytes: &[u8],
        artifact: &RagIndexArtifact,
        recovery: bool,
    ) -> Result<StoreCommit, WikiError> {
        validate_external_ledger_relative(ledger_relative)?;
        if ledger_bytes.is_empty() || ledger_bytes.len() > MAX_EXTERNAL_CANONICAL_BYTES {
            return Err(WikiError::InvalidInput(
                "external canonical ledger exceeds its bounded size".to_owned(),
            ));
        }
        if artifact.manifest.schema_version != RAG_SCHEMA_VERSION
            || artifact.manifest.generation == 0
            || artifact.sqlite_bytes.len() > MAX_SERIALIZED_INDEX_BYTES
        {
            return Err(WikiError::InvalidInput(
                "external RAG artifact has an unsupported generation or size".to_owned(),
            ));
        }
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        let expected_generation = if recovery {
            self.external_recovery_generation_locked(ledger_relative)?
        } else {
            self.reject_dirty_mutation()?;
            self.next_generation()?
        };
        if artifact.manifest.generation != expected_generation {
            return Err(WikiError::Conflict(
                "external RAG artifact generation changed before publication".to_owned(),
            ));
        }
        let writes = vec![(ledger_relative.to_path_buf(), ledger_bytes.to_vec())];
        let dirty = self.dirty_state(artifact.manifest.generation, &writes)?;
        validate_dirty_state(&dirty)?;
        let dirty_bytes = json_bytes(&dirty, "RAG dirty journal")?;
        let manifest_bytes = json_bytes(&artifact.manifest, "RAG generation manifest")?;
        let trust_bytes = rag_trust_bytes_for_manifest(&artifact.manifest, &manifest_bytes)?;
        let paths = [
            PathBuf::from(RAG_DIRTY_RELATIVE),
            ledger_relative.to_path_buf(),
            PathBuf::from(SHARED_INDEX_RELATIVE),
            PathBuf::from(RAG_MANIFEST_RELATIVE),
            PathBuf::from(RAG_TRUST_RELATIVE),
        ];
        let mut snapshots = paths
            .iter()
            .map(|path| crate::CapabilityFileSnapshot::capture(&self.root, path))
            .collect::<Result<Vec<_>, _>>()?;
        crate::transactional_capability(&self.root, &mut snapshots, |snapshots| {
            snapshots[0].install_staged(&self.root, &dirty_bytes)?;
            let mut changed = Vec::new();
            if snapshots[1].install_staged(&self.root, ledger_bytes)? {
                changed.push(path_to_locator(ledger_relative));
            }
            if snapshots[2].install_staged(&self.root, &artifact.sqlite_bytes)? {
                changed.push(SHARED_INDEX_RELATIVE.to_owned());
            }
            if snapshots[3].install_staged(&self.root, &manifest_bytes)? {
                changed.push(RAG_MANIFEST_RELATIVE.to_owned());
            }
            snapshots[0].remove(&self.root)?;
            if snapshots[4].install_staged(&self.root, &trust_bytes)? {
                changed.push(RAG_TRUST_RELATIVE.to_owned());
            }
            changed.sort();
            changed.dedup();
            Ok(StoreCommit {
                changed_paths: changed,
                generation: artifact.manifest.generation,
                manifest_digest: artifact.manifest.logical_digest.clone(),
            })
        })
    }

    /// Reserve no state but report the next publishable external generation.
    ///
    /// Remote adapters use this only to build a complete recovery snapshot after
    /// a disposable `SQLite` loss. Publication rechecks the value under the same
    /// exclusive lock, so a concurrent writer becomes a conflict rather than a
    /// stale-success.
    ///
    /// # Errors
    ///
    /// Returns an error when the store is dirty or the current generation
    /// cannot be read safely.
    pub fn next_external_generation(&self) -> Result<u64, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        self.next_generation()
    }

    /// Return the exact generation that a complete remote refresh may use to
    /// recover an interrupted write to one external ledger.
    ///
    /// This is deliberately narrower than generic dirty recovery: a remote
    /// inventory is canonical only for the selected external ledger and may
    /// not clear a local Markdown mutation journal.
    ///
    /// # Errors
    ///
    /// Returns an error when the ledger path is unsafe or the exact external
    /// dirty state cannot yield a next recovery generation.
    pub fn next_external_recovery_generation(
        &self,
        ledger_relative: &Path,
    ) -> Result<u64, WikiError> {
        validate_external_ledger_relative(ledger_relative)?;
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.external_recovery_generation_locked(ledger_relative)
    }

    /// Validate the current normalized index without changing canonical or derived state.
    ///
    /// # Errors
    ///
    /// Returns a repair-required verification error when the registry, manifest, or `SQLite`
    /// bytes are missing, stale, corrupt, or use the legacy shared-index schema.
    pub fn validate_current(&self) -> Result<StoreCommit, WikiError> {
        let _lock = crate::CapabilityKnowledgeLock::acquire(&self.root)?;
        self.reject_dirty_mutation()?;
        self.current_commit()
    }

    fn default_registry(&self) -> Result<CollectionRegistry, WikiError> {
        self.canonicalize_registry(&CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![CollectionRecord {
                collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
                kind: CollectionKind::UserRoot,
                state: CollectionState::Attached,
                aliases: vec!["user-root".to_owned()],
                local_locator: Some(self.root_path.display().to_string()),
                source_project_id: None,
                default_visibility: CollectionVisibility::Shared,
            }],
        })
    }

    fn preflight_initialized_snapshot(&self) -> Result<(), WikiError> {
        for (relative, max_bytes, label) in [
            (
                COLLECTION_REGISTRY_RELATIVE,
                MAX_REGISTRY_BYTES,
                "collection registry",
            ),
            (
                RAG_MANIFEST_RELATIVE,
                MAX_MANIFEST_BYTES,
                "RAG generation manifest",
            ),
            (
                RAG_TRUST_RELATIVE,
                MAX_RAG_TRUST_BYTES,
                "RAG canonical trust binding",
            ),
            (
                SHARED_INDEX_RELATIVE,
                MAX_SERIALIZED_INDEX_BYTES,
                "RAG SQLite index",
            ),
        ] {
            preflight_bounded_required(&self.root, Path::new(relative), max_bytes, label)?;
        }
        Ok(())
    }

    fn load_registry_optional(&self) -> Result<Option<CollectionRegistry>, WikiError> {
        read_bounded_optional(
            &self.root,
            Path::new(COLLECTION_REGISTRY_RELATIVE),
            MAX_REGISTRY_BYTES,
            "collection registry",
        )?
        .map(|bytes| self.parse_registry(&bytes))
        .transpose()
    }

    fn parse_registry(&self, bytes: &[u8]) -> Result<CollectionRegistry, WikiError> {
        let registry: CollectionRegistry = serde_yaml::from_slice(bytes).map_err(|error| {
            WikiError::InvalidInput(format!("invalid collection registry: {error}"))
        })?;
        let canonical = self.canonicalize_registry(&registry)?;
        if registry != canonical {
            return Err(WikiError::Verification(
                "collection registry is not in canonical sorted form".to_owned(),
            ));
        }
        Self::validate_unique_locators(&canonical)?;
        Ok(canonical)
    }

    fn canonicalize_registry(
        &self,
        registry: &CollectionRegistry,
    ) -> Result<CollectionRegistry, WikiError> {
        let canonical = registry
            .canonicalized()
            .map_err(|error| WikiError::InvalidInput(error.to_string()))?;
        let user_roots = canonical
            .collections
            .iter()
            .filter(|collection| collection.kind == CollectionKind::UserRoot)
            .collect::<Vec<_>>();
        if user_roots.len() != 1 {
            return Err(WikiError::InvalidInput(
                "collection registry requires exactly one user-root record".to_owned(),
            ));
        }
        let expected = self.root_path.display().to_string();
        if user_roots[0].state != CollectionState::Attached
            || user_roots[0].local_locator.as_deref() != Some(expected.as_str())
        {
            return Err(WikiError::Conflict(
                "user-root collection locator does not match the pinned store root".to_owned(),
            ));
        }
        Ok(canonical)
    }

    fn synchronize_project_collections(
        &self,
        projects: &crate::shared::ProjectRegistry,
        prior: Option<CollectionRegistry>,
    ) -> Result<CollectionRegistry, WikiError> {
        let mut registry = prior.unwrap_or(self.default_registry()?);
        let active_ids = projects
            .projects
            .iter()
            .filter(|project| project.enabled)
            .map(|project| project.id.as_str())
            .collect::<BTreeSet<_>>();
        for collection in &mut registry.collections {
            if collection.kind == CollectionKind::RegisteredProject
                && collection
                    .source_project_id
                    .as_deref()
                    .is_none_or(|project_id| !active_ids.contains(project_id))
            {
                collection.state = CollectionState::Detached;
                collection.local_locator = None;
            }
        }
        for project in projects.projects.iter().filter(|project| project.enabled) {
            let canonical = crate::shared::canonical_root(&project.root)?;
            let _pinned = pin_absolute_directory(&canonical, "registered project root")?;
            let default_visibility = match project.visibility {
                crate::shared::KnowledgeVisibility::Shared => CollectionVisibility::Shared,
                crate::shared::KnowledgeVisibility::ProjectPrivate => {
                    CollectionVisibility::ProjectPrivate
                }
                crate::shared::KnowledgeVisibility::Confidential => {
                    CollectionVisibility::Confidential
                }
            };
            if let Some(existing) = registry.collections.iter_mut().find(|collection| {
                collection.source_project_id.as_deref() == Some(project.id.as_str())
            }) {
                if existing.kind != CollectionKind::RegisteredProject {
                    return Err(WikiError::Conflict(format!(
                        "project `{}` is linked to a non-project collection",
                        project.id
                    )));
                }
                existing.state = CollectionState::Attached;
                existing.local_locator = Some(canonical.display().to_string());
                existing.default_visibility = default_visibility;
                existing.aliases.push(project.id.clone());
            } else {
                registry.collections.push(CollectionRecord {
                    collection_id: derive_collection_id("project", &project.id)
                        .map_err(|error| WikiError::InvalidInput(error.to_string()))?,
                    kind: CollectionKind::RegisteredProject,
                    state: CollectionState::Attached,
                    aliases: vec![project.id.clone()],
                    local_locator: Some(canonical.display().to_string()),
                    source_project_id: Some(project.id.clone()),
                    default_visibility,
                });
            }
        }
        let registry = self.canonicalize_registry(&registry)?;
        Self::validate_unique_locators(&registry)?;
        Ok(registry)
    }

    fn validate_unique_locators(registry: &CollectionRegistry) -> Result<(), WikiError> {
        let mut locators = BTreeMap::<String, String>::new();
        for collection in registry
            .collections
            .iter()
            .filter(|collection| collection.state == CollectionState::Attached)
        {
            let locator = collection.local_locator.as_deref().ok_or_else(|| {
                WikiError::InvalidInput("attached collection is missing local locator".to_owned())
            })?;
            let key = locator_key(locator);
            if let Some(existing) = locators.insert(key, collection.collection_id.clone()) {
                if existing != collection.collection_id {
                    return Err(WikiError::Conflict(format!(
                        "collections `{existing}` and `{}` reuse one local locator",
                        collection.collection_id
                    )));
                }
            }
        }
        Ok(())
    }

    fn plan_collection_registration(
        &self,
        mut registry: CollectionRegistry,
        registration: CollectionRegistration,
    ) -> Result<(CollectionRegistry, CollectionRecord), WikiError> {
        let local_locator = Self::prepare_local_locator(&registration)?;
        let collection_id =
            Self::resolve_registration_id(&registry, &registration, local_locator.as_deref())?;
        let record = CollectionRecord {
            collection_id: collection_id.clone(),
            kind: registration.kind,
            state: registration.state,
            aliases: registration.aliases,
            local_locator,
            source_project_id: registration.source_project_id,
            default_visibility: registration.default_visibility,
        };
        if let Some(existing) = registry
            .collections
            .iter_mut()
            .find(|existing| existing.collection_id == collection_id)
        {
            if existing.kind != record.kind {
                return Err(WikiError::Conflict(format!(
                    "collection `{collection_id}` cannot change kind"
                )));
            }
            *existing = record;
        } else {
            registry.collections.push(record);
        }
        registry = self.canonicalize_registry(&registry)?;
        Self::validate_unique_locators(&registry)?;
        let collection = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == collection_id)
            .ok_or_else(|| {
                WikiError::Io("canonical registry lost the registered collection".to_owned())
            })?
            .clone();
        Ok((registry, collection))
    }

    fn resolve_registration_id(
        registry: &CollectionRegistry,
        registration: &CollectionRegistration,
        local_locator: Option<&str>,
    ) -> Result<String, WikiError> {
        if registration.kind == CollectionKind::UserRoot {
            return Err(WikiError::InvalidInput(
                "user-root collection is created only by ensure_registry".to_owned(),
            ));
        }
        if let Some(collection_id) = &registration.collection_id {
            return Ok(collection_id.clone());
        }
        if let Some(source_project_id) = &registration.source_project_id {
            if let Some(existing) = registry.collections.iter().find(|collection| {
                collection.source_project_id.as_deref() == Some(source_project_id)
            }) {
                return Ok(existing.collection_id.clone());
            }
        }
        if let Some(local_locator) = local_locator {
            let requested_locator_key = locator_key(local_locator);
            if let Some(existing) = registry.collections.iter().find(|collection| {
                collection
                    .local_locator
                    .as_deref()
                    .is_some_and(|candidate| locator_key(candidate) == requested_locator_key)
            }) {
                return Ok(existing.collection_id.clone());
            }
        }
        let namespace = match registration.kind {
            CollectionKind::RegisteredProject => "registered-project",
            CollectionKind::Directory => "directory",
            CollectionKind::Imported => "imported",
            CollectionKind::UserRoot => unreachable!("user-root rejected above"),
        };
        if let Some(identity) = &registration.portable_identity {
            return derive_collection_id(namespace, identity)
                .map_err(|error| WikiError::InvalidInput(error.to_string()));
        }
        if let Some(inventory_digest) = &registration.reviewed_inventory_digest {
            validate_prefixed_sha256("reviewed inventory digest", inventory_digest)?;
            return derive_collection_id(namespace, &new_collection_instance_identity()?)
                .map_err(|error| WikiError::InvalidInput(error.to_string()));
        }
        Err(WikiError::InvalidInput(
            "new collections require portable_identity or reviewed_inventory_digest; paths and basenames are never identity seeds"
                .to_owned(),
        ))
    }

    fn prepare_local_locator(
        registration: &CollectionRegistration,
    ) -> Result<Option<String>, WikiError> {
        match (registration.state, registration.local_locator.as_deref()) {
            (CollectionState::Detached, None) => Ok(None),
            (CollectionState::Detached, Some(_)) => Err(WikiError::InvalidInput(
                "detached collections cannot retain a local locator".to_owned(),
            )),
            (CollectionState::Attached, None) => Err(WikiError::InvalidInput(
                "attached collections require a local locator".to_owned(),
            )),
            (CollectionState::Attached, Some(locator)) => {
                let canonical = crate::shared::canonical_root(locator)?;
                let _pinned = pin_absolute_directory(&canonical, "collection root")?;
                Ok(Some(canonical.display().to_string()))
            }
        }
    }

    fn load_claims(&self, registry: &CollectionRegistry) -> Result<Vec<CanonicalClaim>, WikiError> {
        self.load_claims_excluding(registry, &BTreeSet::new())
    }

    #[allow(clippy::too_many_lines)]
    fn load_claims_excluding(
        &self,
        registry: &CollectionRegistry,
        transient_claim_paths: &BTreeSet<PathBuf>,
    ) -> Result<Vec<CanonicalClaim>, WikiError> {
        self.load_claims_for_collection(registry, transient_claim_paths, None)
    }

    #[allow(clippy::too_many_lines)]
    fn load_claims_for_collection(
        &self,
        registry: &CollectionRegistry,
        transient_claim_paths: &BTreeSet<PathBuf>,
        selected: Option<&str>,
    ) -> Result<Vec<CanonicalClaim>, WikiError> {
        let Some((parent, name)) =
            crate::capability_parent(&self.root, Path::new(CLAIMS_RELATIVE), false)?
        else {
            return Ok(Vec::new());
        };
        let claims_root = match parent.open_dir_nofollow(&name) {
            Ok(directory) => directory,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(WikiError::Conflict(format!(
                    "cannot open Claims directory no-follow: {error}"
                )));
            }
        };
        let known = registry
            .collections
            .iter()
            .map(|collection| collection.collection_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut collection_names = directory_names(&claims_root, "Claims directory")?;
        let mut claims = Vec::new();
        for collection_name in collection_names.drain(..) {
            if selected.is_some_and(|id| collection_name != OsStr::new(id)) {
                continue;
            }
            let collection_id = collection_name.to_str().ok_or_else(|| {
                WikiError::Verification("claim collection directory is not UTF-8".to_owned())
            })?;
            if !known.contains(collection_id) {
                return Err(WikiError::Verification(format!(
                    "canonical claims reference unknown collection `{collection_id}`"
                )));
            }
            let metadata = claims_root
                .symlink_metadata(&collection_name)
                .map_err(|error| {
                    WikiError::Io(format!(
                        "cannot inspect claim collection directory: {error}"
                    ))
                })?;
            if !metadata.is_dir() {
                return Err(WikiError::Verification(
                    "Claims entries must be no-follow collection directories".to_owned(),
                ));
            }
            let collection_dir =
                claims_root
                    .open_dir_nofollow(&collection_name)
                    .map_err(|error| {
                        WikiError::Conflict(format!(
                            "cannot open claim collection no-follow: {error}"
                        ))
                    })?;
            for file_name in directory_names(&collection_dir, "claim collection")? {
                let path = Path::new(&file_name);
                let locator_path = Path::new(CLAIMS_RELATIVE).join(collection_id).join(path);
                if transient_claim_paths.contains(&locator_path) {
                    continue;
                }
                if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
                    return Err(WikiError::Verification(
                        "claim collection contains a non-Markdown entry".to_owned(),
                    ));
                }
                let bytes = read_named_file(&collection_dir, &file_name, MAX_CLAIM_BYTES, "claim")?;
                crate::reject_likely_credentials(&bytes).map_err(|error| {
                    WikiError::Verification(format!(
                        "canonical claim contains likely sensitive material: {error}"
                    ))
                })?;
                let locator = format!(
                    "{CLAIMS_RELATIVE}/{collection_id}/{}",
                    path.to_string_lossy()
                );
                let claim = parse_claim_markdown(
                    &locator,
                    std::str::from_utf8(&bytes).map_err(|error| {
                        WikiError::InvalidInput(format!("claim is not UTF-8: {error}"))
                    })?,
                )
                .map_err(rag_error)?;
                let stem = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("");
                if claim.collection_id != collection_id || claim.claim_id != stem {
                    return Err(WikiError::Verification(format!(
                        "claim path does not match canonical IDs: {locator}"
                    )));
                }
                claims.push(claim);
            }
        }
        claims.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
        let mut ids = BTreeSet::new();
        if let Some(duplicate) = claims
            .iter()
            .find(|claim| !ids.insert(claim.claim_id.clone()))
        {
            return Err(WikiError::Verification(format!(
                "duplicate canonical claim ID `{}`",
                duplicate.claim_id
            )));
        }
        Ok(claims)
    }

    fn load_documents(
        &self,
        registry: &CollectionRegistry,
    ) -> Result<Vec<CanonicalDocument>, WikiError> {
        self.load_documents_for_collection(registry, None)
    }

    fn load_documents_for_collection(
        &self,
        registry: &CollectionRegistry,
        selected: Option<&str>,
    ) -> Result<Vec<CanonicalDocument>, WikiError> {
        let mut documents = Vec::new();
        for collection in registry
            .collections
            .iter()
            .filter(|collection| collection.state == CollectionState::Attached)
            .filter(|collection| selected.is_none_or(|id| collection.collection_id == id))
        {
            let locator = collection.local_locator.as_deref().ok_or_else(|| {
                WikiError::Verification("attached collection locator disappeared".to_owned())
            })?;
            let source = if collection.collection_id == USER_ROOT_COLLECTION_ID {
                self.root.try_clone().map_err(|error| {
                    WikiError::Io(format!("cannot clone pinned user root: {error}"))
                })?
            } else {
                pin_absolute_directory(Path::new(locator), "collection root")?
            };
            documents.extend(load_wiki_documents(&source, collection)?);
        }
        documents.sort_by(|left, right| left.document_id.cmp(&right.document_id));
        let mut ids = BTreeSet::new();
        if let Some(duplicate) = documents
            .iter()
            .find(|document| !ids.insert(document.document_id.clone()))
        {
            return Err(WikiError::Verification(format!(
                "duplicate canonical document ID `{}`",
                duplicate.document_id
            )));
        }
        Ok(documents)
    }

    fn commit_canonical_writes_locked(
        &self,
        mut writes: Vec<(PathBuf, Vec<u8>)>,
    ) -> Result<StoreCommit, WikiError> {
        writes.sort_by(|left, right| left.0.cmp(&right.0));
        if writes.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(WikiError::InvalidInput(
                "canonical store transaction contains duplicate paths".to_owned(),
            ));
        }
        let generation = self.next_generation()?;
        let dirty = self.dirty_state(generation, &writes)?;
        let dirty_bytes = json_bytes(&dirty, "RAG dirty journal")?;
        let mut paths = vec![PathBuf::from(RAG_DIRTY_RELATIVE)];
        paths.extend(writes.iter().map(|(path, _)| path.clone()));
        paths.push(PathBuf::from(SHARED_INDEX_RELATIVE));
        paths.push(PathBuf::from(RAG_MANIFEST_RELATIVE));
        paths.push(PathBuf::from(RAG_TRUST_RELATIVE));
        let mut snapshots = paths
            .iter()
            .map(|path| crate::CapabilityFileSnapshot::capture(&self.root, path))
            .collect::<Result<Vec<_>, _>>()?;
        let absent_directories = absent_claim_directories(&self.root, &writes)?;
        let write_count = writes.len();
        let result = crate::transactional_capability(&self.root, &mut snapshots, |snapshots| {
            snapshots[0].install_staged(&self.root, &dirty_bytes)?;
            let mut changed = Vec::new();
            for (index, (path, bytes)) in writes.iter().enumerate() {
                if snapshots[index + 1].install_staged(&self.root, bytes)? {
                    changed.push(path_to_locator(path));
                }
            }
            injected_failure_after_canonical_writes()?;
            let transient_claim_paths = snapshots[1..=write_count]
                .iter()
                .flat_map(crate::CapabilityFileSnapshot::transient_claim_paths)
                .filter(|path| path.starts_with(CLAIMS_RELATIVE))
                .map(Path::to_path_buf)
                .collect::<BTreeSet<_>>();
            let snapshot =
                self.load_canonical_snapshot_excluding(generation, &transient_claim_paths)?;
            let artifact = build_rag_index(&snapshot).map_err(rag_error)?;
            let manifest_bytes = json_bytes(&artifact.manifest, "RAG generation manifest")?;
            let trust_bytes = rag_trust_bytes_for_manifest(&artifact.manifest, &manifest_bytes)?;
            if snapshots[write_count + 1].install_staged(&self.root, &artifact.sqlite_bytes)? {
                changed.push(SHARED_INDEX_RELATIVE.to_owned());
            }
            if snapshots[write_count + 2].install_staged(&self.root, &manifest_bytes)? {
                changed.push(RAG_MANIFEST_RELATIVE.to_owned());
            }
            snapshots[0].remove(&self.root)?;
            if snapshots[write_count + 3].install_staged(&self.root, &trust_bytes)? {
                changed.push(RAG_TRUST_RELATIVE.to_owned());
            }
            changed.sort();
            changed.dedup();
            Ok(StoreCommit {
                changed_paths: changed,
                generation: artifact.manifest.generation,
                manifest_digest: artifact.manifest.logical_digest,
            })
        });
        if result.is_err() {
            cleanup_absent_directories(&self.root, &absent_directories)?;
        }
        result
    }

    fn publish_rebuild_locked(
        &self,
        artifact: &RagIndexArtifact,
    ) -> Result<StoreCommit, WikiError> {
        let registry_bytes = read_bounded_required(
            &self.root,
            Path::new(COLLECTION_REGISTRY_RELATIVE),
            MAX_REGISTRY_BYTES,
            "collection registry",
        )?;
        let dirty = self.dirty_state_for_rebuild(
            artifact.manifest.generation,
            &[(PathBuf::from(COLLECTION_REGISTRY_RELATIVE), registry_bytes)],
        )?;
        let dirty_bytes = json_bytes(&dirty, "RAG dirty journal")?;
        let manifest_bytes = json_bytes(&artifact.manifest, "RAG generation manifest")?;
        let trust_bytes = rag_trust_bytes_for_manifest(&artifact.manifest, &manifest_bytes)?;
        let paths = [
            PathBuf::from(RAG_DIRTY_RELATIVE),
            PathBuf::from(SHARED_INDEX_RELATIVE),
            PathBuf::from(RAG_MANIFEST_RELATIVE),
            PathBuf::from(RAG_TRUST_RELATIVE),
        ];
        let mut snapshots = paths
            .iter()
            .map(|path| crate::CapabilityFileSnapshot::capture(&self.root, path))
            .collect::<Result<Vec<_>, _>>()?;
        crate::transactional_capability(&self.root, &mut snapshots, |snapshots| {
            snapshots[0].install_staged(&self.root, &dirty_bytes)?;
            let mut changed = Vec::new();
            if snapshots[1].install_staged(&self.root, &artifact.sqlite_bytes)? {
                changed.push(SHARED_INDEX_RELATIVE.to_owned());
            }
            if snapshots[2].install_staged(&self.root, &manifest_bytes)? {
                changed.push(RAG_MANIFEST_RELATIVE.to_owned());
            }
            snapshots[0].remove(&self.root)?;
            if snapshots[3].install_staged(&self.root, &trust_bytes)? {
                changed.push(RAG_TRUST_RELATIVE.to_owned());
            }
            changed.sort();
            Ok(StoreCommit {
                changed_paths: changed,
                generation: artifact.manifest.generation,
                manifest_digest: artifact.manifest.logical_digest.clone(),
            })
        })
    }

    fn dirty_state(
        &self,
        target_generation: u64,
        writes: &[(PathBuf, Vec<u8>)],
    ) -> Result<PersistentDirtyState, WikiError> {
        let base = self.load_manifest_repairable()?;
        Self::dirty_state_from_base(base, target_generation, writes)
    }

    fn dirty_state_for_rebuild(
        &self,
        target_generation: u64,
        writes: &[(PathBuf, Vec<u8>)],
    ) -> Result<PersistentDirtyState, WikiError> {
        let base = self.load_manifest_repairable()?;
        Self::dirty_state_from_base(base, target_generation, writes)
    }

    fn dirty_state_from_base(
        base: Option<GenerationManifest>,
        target_generation: u64,
        writes: &[(PathBuf, Vec<u8>)],
    ) -> Result<PersistentDirtyState, WikiError> {
        let mut entries = writes
            .iter()
            .map(|(path, bytes)| PersistentDirtyEntry {
                locator: path_to_locator(path),
                target_digest: sha256_digest(bytes),
                delete: bytes.is_empty(),
            })
            .collect::<Vec<_>>();
        entries.sort();
        entries.dedup();
        if entries.is_empty() {
            return Err(WikiError::InvalidInput(
                "dirty journal requires an exact canonical write set".to_owned(),
            ));
        }
        Ok(PersistentDirtyState {
            schema_version: DIRTY_SCHEMA_VERSION,
            base_generation: base.as_ref().map_or(0, |manifest| manifest.generation),
            base_manifest_digest: base
                .map_or_else(empty_manifest_digest, |manifest| manifest.logical_digest),
            target_generation,
            entries,
        })
    }

    fn next_generation(&self) -> Result<u64, WikiError> {
        self.load_manifest_repairable()?.map_or(Ok(1), |manifest| {
            manifest.generation.checked_add(1).ok_or_else(|| {
                WikiError::Conflict("RAG generation counter is exhausted".to_owned())
            })
        })
    }

    fn recovery_generation(&self) -> Result<u64, WikiError> {
        let base = self.load_manifest_repairable()?;
        let next = base.as_ref().map_or(Ok(1), |manifest| {
            manifest.generation.checked_add(1).ok_or_else(|| {
                WikiError::Conflict("RAG generation counter is exhausted".to_owned())
            })
        })?;
        let Some(bytes) = read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        else {
            return Ok(next);
        };
        let dirty = serde_json::from_slice::<PersistentDirtyState>(&bytes).map_err(|error| {
            WikiError::Verification(format!(
                "RAG dirty journal is corrupt and cannot authorize recovery: {error}"
            ))
        })?;
        validate_dirty_state(&dirty)?;
        let lineage_matches = base.as_ref().map_or_else(
            || dirty.base_generation == 0 && dirty.base_manifest_digest == empty_manifest_digest(),
            |manifest| {
                dirty.base_generation == manifest.generation
                    && dirty.base_manifest_digest == manifest.logical_digest
            },
        );
        if !lineage_matches {
            return Err(WikiError::Conflict(
                "RAG dirty journal lineage differs from the published generation".to_owned(),
            ));
        }
        self.verify_dirty_targets(&dirty)?;
        Ok(next.max(dirty.target_generation))
    }

    fn external_recovery_generation_locked(
        &self,
        ledger_relative: &Path,
    ) -> Result<u64, WikiError> {
        let base = self.load_manifest_repairable()?;
        let next = base.as_ref().map_or(Ok(1), |manifest| {
            manifest.generation.checked_add(1).ok_or_else(|| {
                WikiError::Conflict("RAG generation counter is exhausted".to_owned())
            })
        })?;
        let Some(bytes) = read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        else {
            return Ok(next);
        };
        let dirty = serde_json::from_slice::<PersistentDirtyState>(&bytes).map_err(|error| {
            WikiError::Verification(format!(
                "RAG dirty journal is corrupt and cannot authorize external recovery: {error}"
            ))
        })?;
        validate_dirty_state(&dirty)?;
        let lineage_matches = base.as_ref().map_or_else(
            || dirty.base_generation == 0 && dirty.base_manifest_digest == empty_manifest_digest(),
            |manifest| {
                dirty.base_generation == manifest.generation
                    && dirty.base_manifest_digest == manifest.logical_digest
            },
        );
        if !lineage_matches {
            return Err(WikiError::Conflict(
                "RAG dirty journal lineage differs from the published generation".to_owned(),
            ));
        }
        let expected_locator = path_to_locator(ledger_relative);
        if dirty.entries.len() != 1
            || dirty.entries[0].locator != expected_locator
            || dirty.entries[0].delete
        {
            return Err(WikiError::Verification(
                "external recovery cannot clear a dirty journal outside its selected remote ledger"
                    .to_owned(),
            ));
        }
        Ok(next.max(dirty.target_generation))
    }

    fn verify_dirty_targets(&self, dirty: &PersistentDirtyState) -> Result<(), WikiError> {
        let registry = self.load_registry()?;
        for entry in &dirty.entries {
            let (target_root, path) = self.resolve_dirty_target(&registry, &entry.locator)?;
            let actual = read_bounded_optional(
                &target_root,
                &path,
                MAX_EXTERNAL_CANONICAL_BYTES,
                "dirty-journal canonical target",
            )?;
            if entry.delete {
                if actual.is_some() {
                    return Err(WikiError::Verification(format!(
                        "RAG recovery is blocked until deletion completes: {}",
                        entry.locator
                    )));
                }
            } else if actual.as_deref().map(sha256_digest).as_deref()
                != Some(entry.target_digest.as_str())
            {
                return Err(WikiError::Verification(format!(
                    "RAG recovery is blocked until the exact canonical write completes: {}",
                    entry.locator
                )));
            }
        }
        Ok(())
    }

    fn resolve_dirty_target(
        &self,
        registry: &CollectionRegistry,
        locator: &str,
    ) -> Result<(Dir, PathBuf), WikiError> {
        let path = Path::new(locator);
        let mut components = path.components();
        if components.next() != Some(Component::Normal(OsStr::new("collections"))) {
            return self
                .root
                .try_clone()
                .map(|root| (root, path.to_path_buf()))
                .map_err(|error| WikiError::Io(format!("cannot clone pinned user root: {error}")));
        }
        let Some(Component::Normal(namespace)) = components.next() else {
            return Err(WikiError::Verification(
                "dirty collection locator lacks a namespace".to_owned(),
            ));
        };
        let namespace = namespace.to_str().ok_or_else(|| {
            WikiError::Verification("dirty collection namespace is not UTF-8".to_owned())
        })?;
        let relative = components.as_path();
        if relative.as_os_str().is_empty() {
            return Err(WikiError::Verification(
                "dirty collection locator lacks a canonical relative path".to_owned(),
            ));
        }
        let collection_id = match registry.resolve_collection(namespace) {
            CollectionResolution::Resolved(collection_id) => collection_id,
            CollectionResolution::Unknown => match registry.resolve_project(namespace) {
                CollectionResolution::Resolved(collection_id) => collection_id,
                CollectionResolution::Unknown => {
                    return Err(WikiError::Verification(format!(
                        "dirty collection namespace is unknown: {namespace}"
                    )));
                }
                CollectionResolution::Ambiguous(_) => {
                    return Err(WikiError::Conflict(format!(
                        "dirty project namespace is ambiguous: {namespace}"
                    )));
                }
            },
            CollectionResolution::Ambiguous(_) => {
                return Err(WikiError::Conflict(format!(
                    "dirty collection namespace is ambiguous: {namespace}"
                )));
            }
        };
        let collection = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == collection_id)
            .ok_or_else(|| {
                WikiError::Verification("resolved dirty collection disappeared".to_owned())
            })?;
        if collection.state != CollectionState::Attached {
            return Err(WikiError::Verification(format!(
                "dirty collection is detached: {collection_id}"
            )));
        }
        let root = if collection_id == USER_ROOT_COLLECTION_ID {
            self.root
                .try_clone()
                .map_err(|error| WikiError::Io(format!("cannot clone pinned user root: {error}")))?
        } else {
            let locator = collection.local_locator.as_deref().ok_or_else(|| {
                WikiError::Verification("attached dirty collection lacks a locator".to_owned())
            })?;
            pin_absolute_directory(Path::new(locator), "dirty collection root")?
        };
        Ok((root, relative.to_path_buf()))
    }

    fn reject_dirty_mutation(&self) -> Result<(), WikiError> {
        if read_bounded_optional(
            &self.root,
            Path::new(RAG_DIRTY_RELATIVE),
            MAX_DIRTY_BYTES,
            "RAG dirty journal",
        )?
        .is_some()
        {
            return Err(WikiError::Verification(
                "RAG store has an interrupted canonical write; rebuild before new mutation"
                    .to_owned(),
            ));
        }
        Ok(())
    }

    fn load_manifest_required(&self) -> Result<GenerationManifest, WikiError> {
        self.load_manifest_optional()?.ok_or_else(|| {
            WikiError::Verification(
                "RAG generation manifest is missing; rebuild from canonical sources".to_owned(),
            )
        })
    }

    fn load_manifest_optional(&self) -> Result<Option<GenerationManifest>, WikiError> {
        let manifest_bytes = read_bounded_optional(
            &self.root,
            Path::new(RAG_MANIFEST_RELATIVE),
            MAX_MANIFEST_BYTES,
            "RAG generation manifest",
        )?;
        let trust_bytes = read_bounded_optional(
            &self.root,
            Path::new(RAG_TRUST_RELATIVE),
            MAX_RAG_TRUST_BYTES,
            "RAG canonical trust binding",
        )?;
        match (manifest_bytes, trust_bytes) {
            (None, None) => Ok(None),
            (Some(manifest_bytes), Some(trust_bytes)) => {
                let manifest = parse_generation_manifest(&manifest_bytes)?;
                verify_rag_trust_bytes(&manifest, &manifest_bytes, &trust_bytes)?;
                Ok(Some(manifest))
            }
            _ => Err(WikiError::Verification(
                "RAG generation is incomplete; rebuild from canonical sources".to_owned(),
            )),
        }
    }

    fn load_manifest_repairable(&self) -> Result<Option<GenerationManifest>, WikiError> {
        let Some(bytes) = read_bounded_optional(
            &self.root,
            Path::new(RAG_TRUST_RELATIVE),
            MAX_RAG_TRUST_BYTES,
            "RAG canonical trust binding",
        )?
        else {
            return Ok(None);
        };
        let Ok(binding) = serde_json::from_slice::<RagTrustBinding>(&bytes) else {
            return Ok(None);
        };
        if binding.schema_version != RAG_TRUST_SCHEMA_VERSION
            || binding.generation == 0
            || validate_prefixed_sha256("trusted logical digest", &binding.logical_digest).is_err()
            || validate_prefixed_sha256("trusted SQLite digest", &binding.sqlite_digest).is_err()
            || validate_prefixed_sha256("trusted manifest digest", &binding.manifest_digest)
                .is_err()
            || json_bytes(&binding, "RAG canonical trust binding")? != bytes
        {
            return Ok(None);
        }
        Ok(Some(GenerationManifest {
            schema_version: RAG_SCHEMA_VERSION,
            generation: binding.generation,
            logical_digest: binding.logical_digest,
            entry_count: binding.entry_count,
            sqlite_digest: binding.sqlite_digest,
        }))
    }

    fn current_commit(&self) -> Result<StoreCommit, WikiError> {
        let manifest = self.load_manifest_required()?;
        let registry = self.load_registry()?;
        let sqlite_bytes = read_bounded_required(
            &self.root,
            Path::new(SHARED_INDEX_RELATIVE),
            MAX_SERIALIZED_INDEX_BYTES,
            "RAG SQLite index",
        )?;
        probe_rag_index(&sqlite_bytes, &manifest, &registry).map_err(|error| {
            WikiError::Verification(format!(
                "RAG SQLite index requires rebuild from canonical sources: {error}"
            ))
        })?;
        Ok(StoreCommit {
            changed_paths: Vec::new(),
            generation: manifest.generation,
            manifest_digest: manifest.logical_digest,
        })
    }

    fn index_state_is_complete(&self) -> Result<bool, WikiError> {
        let Some(manifest) = self.load_manifest_repairable()? else {
            return Ok(false);
        };
        let Some(registry) = self.load_registry_optional()? else {
            return Ok(false);
        };
        let Some(sqlite_bytes) = read_bounded_optional(
            &self.root,
            Path::new(SHARED_INDEX_RELATIVE),
            MAX_SERIALIZED_INDEX_BYTES,
            "RAG SQLite index",
        )?
        else {
            return Ok(false);
        };
        Ok(probe_rag_index(&sqlite_bytes, &manifest, &registry).is_ok())
    }
}

fn probe_rag_index(
    sqlite_bytes: &[u8],
    manifest: &GenerationManifest,
    registry: &CollectionRegistry,
) -> Result<(), RagError> {
    let probe = RetrievalRequest {
        scope: RetrievalScope::Global,
        current_collection_id: None,
        query: "zzhiveindexschemaprobe7f3a9d2c".to_owned(),
        query_expansions: Vec::new(),
        top_k: 1,
        byte_budget: 1,
        confidential_collection_id: None,
    };
    retrieve_serialized(sqlite_bytes, manifest, registry, &probe).map(|_| ())
}

fn load_wiki_documents(
    source: &Dir,
    collection: &CollectionRecord,
) -> Result<Vec<CanonicalDocument>, WikiError> {
    let Some((parent, name)) = crate::capability_parent(source, Path::new(WIKI_RELATIVE), false)?
    else {
        return Ok(Vec::new());
    };
    let wiki = match parent.open_dir_nofollow(&name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(WikiError::Conflict(format!(
                "cannot open collection Wiki no-follow: {error}"
            )));
        }
    };
    let mut documents = Vec::new();
    for file_name in directory_names(&wiki, "collection Wiki")? {
        let metadata = wiki.symlink_metadata(&file_name).map_err(|error| {
            WikiError::Io(format!("cannot inspect collection Wiki entry: {error}"))
        })?;
        if metadata.is_dir() {
            continue;
        }
        if !metadata.is_file() {
            return Err(WikiError::Verification(
                "collection Wiki contains a symlink or special file".to_owned(),
            ));
        }
        let path = Path::new(&file_name);
        if path.extension().and_then(|extension| extension.to_str()) != Some("md")
            || matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("index.md" | "log.md")
            )
        {
            continue;
        }
        documents.push(load_wiki_document_file(&wiki, collection, &file_name)?);
    }
    Ok(documents)
}

fn load_wiki_document_file(
    wiki: &Dir,
    collection: &CollectionRecord,
    file_name: &OsStr,
) -> Result<CanonicalDocument, WikiError> {
    let path = Path::new(file_name);
    let bytes = read_named_file(wiki, file_name, MAX_WIKI_BYTES, "Wiki page")?;
    crate::reject_likely_credentials(&bytes).map_err(|error| {
        WikiError::Verification(format!(
            "canonical Wiki page contains likely sensitive material: {error}"
        ))
    })?;
    let locator = format!("{WIKI_RELATIVE}/{}", path.to_string_lossy());
    let page = parse_page_bytes(&bytes, &locator)?;
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if page.frontmatter.id != stem {
        return Err(WikiError::Verification(format!(
            "Wiki filename must match page ID: {locator}"
        )));
    }
    let id_material = format!(
        "document-id-v1\0{}\0{}",
        collection.collection_id, page.frontmatter.id
    );
    let document_id = format!(
        "document-{}",
        raw_sha256(&sha256_digest(id_material.as_bytes()))
    );
    let kind = page.frontmatter.kind.clone();
    let mut document = CanonicalDocument {
        document_id,
        collection_id: collection.collection_id.clone(),
        locator,
        title: page.frontmatter.summary,
        category: crate::canonical_category(&kind).to_owned(),
        kind,
        body: page.body.trim().to_owned(),
        digest: String::new(),
        visibility: collection.default_visibility.into(),
        language: RagLanguage::Und,
        revision: 1,
        tags: page.frontmatter.tags,
        aliases: page.frontmatter.aliases,
        links: page.frontmatter.links,
        sources: page.frontmatter.sources,
        replacement: None,
    };
    document.digest = document_digest(&document);
    Ok(document)
}

fn plan_reviewed_claim_upserts(
    collection: &CollectionRecord,
    reviewed_claims: &[ReviewedClaim],
    inventory_digest: &str,
    target_generation: u64,
    claims: &mut Vec<CanonicalClaim>,
    writes: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), WikiError> {
    for reviewed in reviewed_claims {
        let claim_key = format!("scan.{}", reviewed.claim_id);
        let mut active = claims
            .iter()
            .filter(|claim| {
                claim.collection_id == collection.collection_id
                    && claim.claim_key == claim_key
                    && claim.status != AssertionStatus::Superseded
            })
            .cloned()
            .collect::<Vec<_>>();
        active.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
        let mut candidate =
            reviewed_to_claim(collection, reviewed, inventory_digest, target_generation)?;
        if let Some(current) = active
            .iter()
            .find(|existing| scan_claim_semantically_matches(existing, &candidate))
        {
            let current_id = current.claim_id.clone();
            for existing in active
                .iter()
                .filter(|existing| existing.claim_id != current_id)
            {
                let source_locator = existing.locator.clone();
                let rewritten = superseded_scan_claim(existing, &current_id, target_generation);
                insert_claim_write(writes, &rewritten)?;
                replace_loaded_claim(claims, rewritten);
                invalidate_promoted_claims_for_source(
                    &source_locator,
                    target_generation,
                    claims,
                    writes,
                )?;
            }
            continue;
        }
        for existing in &active {
            let source_locator = existing.locator.clone();
            let rewritten = superseded_scan_claim(existing, &candidate.claim_id, target_generation);
            insert_claim_write(writes, &rewritten)?;
            replace_loaded_claim(claims, rewritten);
            invalidate_promoted_claims_for_source(
                &source_locator,
                target_generation,
                claims,
                writes,
            )?;
        }
        candidate.supersedes = active.iter().map(|claim| claim.claim_id.clone()).collect();
        candidate.digest = claim_digest(&candidate);
        insert_claim_write(writes, &candidate)?;
        claims.push(candidate);
    }
    Ok(())
}

fn plan_source_invalidations(
    collection_id: &str,
    reviewed_ids: &BTreeSet<String>,
    inventory_digest: &str,
    target_generation: u64,
    claims: &mut [CanonicalClaim],
    writes: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), WikiError> {
    let invalidated = claims
        .iter()
        .filter(|claim| {
            claim.collection_id == collection_id
                && claim.status != AssertionStatus::Superseded
                && scan_review_id(claim).is_some_and(|review_id| !reviewed_ids.contains(review_id))
        })
        .cloned()
        .collect::<Vec<_>>();
    for mut rewritten in invalidated {
        let source_locator = rewritten.locator.clone();
        let review_id = scan_review_id(&rewritten)
            .ok_or_else(|| {
                WikiError::Verification(
                    "source-invalidation candidate is not a scan-owned claim".to_owned(),
                )
            })?
            .to_owned();
        rewritten.status = AssertionStatus::Superseded;
        rewritten.replacement = None;
        rewritten.revision = target_generation;
        rewritten.provenance = ClaimProvenance {
            source_kind: RememberSourceKind::ReviewedArtifact,
            summary: format!(
                "Source-invalidated scan claim `{review_id}` absent from reviewed inventory {inventory_digest}"
            ),
            locator: format!("scan-inventory:{inventory_digest}"),
            digest: inventory_digest.to_owned(),
        };
        if let Some(metadata) = &mut rewritten.scan_metadata {
            metadata.review_status = ScanReviewStatus::SourceInvalidated;
        }
        rewritten.digest = claim_digest(&rewritten);
        insert_claim_write(writes, &rewritten)?;
        replace_loaded_claim(claims, rewritten);
        invalidate_promoted_claims_for_source(&source_locator, target_generation, claims, writes)?;
    }
    Ok(())
}

fn invalidate_promoted_claims_for_source(
    source_locator: &str,
    target_generation: u64,
    claims: &mut [CanonicalClaim],
    writes: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), WikiError> {
    let invalidated = claims
        .iter()
        .filter(|claim| {
            claim.collection_id == USER_ROOT_COLLECTION_ID
                && claim.status != AssertionStatus::Superseded
                && claim.provenance.source_kind == RememberSourceKind::ReviewedArtifact
                && claim.provenance.locator == source_locator
        })
        .cloned()
        .collect::<Vec<_>>();
    for mut rewritten in invalidated {
        rewritten.status = AssertionStatus::Superseded;
        rewritten.replacement = None;
        rewritten.revision = target_generation;
        if let Some(metadata) = &mut rewritten.scan_metadata {
            metadata.review_status = ScanReviewStatus::SourceInvalidated;
        }
        rewritten.digest = claim_digest(&rewritten);
        insert_claim_write(writes, &rewritten)?;
        replace_loaded_claim(claims, rewritten);
    }
    Ok(())
}

fn superseded_scan_claim(
    existing: &CanonicalClaim,
    replacement: &str,
    target_generation: u64,
) -> CanonicalClaim {
    let mut rewritten = existing.clone();
    rewritten.status = AssertionStatus::Superseded;
    rewritten.replacement = Some(replacement.to_owned());
    rewritten.revision = target_generation;
    rewritten.digest = claim_digest(&rewritten);
    rewritten
}

fn replace_loaded_claim(claims: &mut [CanonicalClaim], rewritten: CanonicalClaim) {
    if let Some(stored) = claims
        .iter_mut()
        .find(|claim| claim.claim_id == rewritten.claim_id)
    {
        *stored = rewritten;
    }
}

fn reviewed_to_claim(
    collection: &CollectionRecord,
    reviewed: &ReviewedClaim,
    inventory_digest: &str,
    revision: u64,
) -> Result<CanonicalClaim, WikiError> {
    let id_material = serde_json::to_vec(&(
        "reviewed-claim-v1",
        &collection.collection_id,
        &reviewed.claim_id,
        inventory_digest,
        &reviewed.statement,
        &reviewed.version,
        &reviewed.revision,
        &reviewed.applicability,
        &reviewed.evidence,
        reviewed.global_promotion_candidate,
    ))
    .map_err(|error| WikiError::Io(format!("cannot derive reviewed claim ID: {error}")))?;
    let claim_id = format!("claim-{}", raw_sha256(&sha256_digest(&id_material)));
    let locator = claim_locator(&collection.collection_id, &claim_id);
    let mut sources = reviewed
        .evidence
        .iter()
        .map(|evidence| format!("{}#{}", evidence.locator, evidence.content_digest))
        .collect::<Vec<_>>();
    sources.sort();
    sources.dedup();
    let status = if reviewed.kind == ScanClaimKind::Outcome {
        AssertionStatus::Verified
    } else {
        AssertionStatus::Observed
    };
    let summary = reviewed_claim_provenance_summary(inventory_digest);
    let mut claim = CanonicalClaim {
        claim_id,
        claim_key: format!("scan.{}", reviewed.claim_id),
        collection_id: collection.collection_id.clone(),
        document_id: None,
        locator,
        kind: scan_claim_kind(reviewed.kind),
        status,
        visibility: collection.default_visibility.into(),
        normalized_fact: reviewed.statement.trim().to_owned(),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::ReviewedArtifact,
            summary,
            locator: format!("scan-inventory:{inventory_digest}"),
            digest: inventory_digest.to_owned(),
        },
        scan_metadata: Some(ScanClaimMetadata::from_reviewed(reviewed)),
        revision,
        sources,
        supersedes: Vec::new(),
        replacement: None,
        observed_at: None,
        verified_at: None,
        digest: String::new(),
    };
    claim.digest = claim_digest(&claim);
    render_claim_markdown(&claim).map_err(rag_error)?;
    Ok(claim)
}

fn reviewed_claim_provenance_summary(inventory_digest: &str) -> String {
    format!("Agent-reviewed source claim bound to inventory {inventory_digest}")
}

fn scan_review_id(claim: &CanonicalClaim) -> Option<&str> {
    let metadata = claim.scan_metadata.as_ref()?;
    let review_id = claim.claim_key.strip_prefix("scan.")?;
    let inventory_digest = claim.provenance.locator.strip_prefix("scan-inventory:")?;
    (claim.provenance.source_kind == RememberSourceKind::ReviewedArtifact
        && inventory_digest == claim.provenance.digest)
        .then_some(metadata.review_id.as_str())
        .filter(|metadata_id| *metadata_id == review_id)
}

fn scan_claim_semantically_matches(existing: &CanonicalClaim, candidate: &CanonicalClaim) -> bool {
    scan_review_id(existing).is_some()
        && existing.claim_key == candidate.claim_key
        && existing.collection_id == candidate.collection_id
        && existing.document_id == candidate.document_id
        && existing.kind == candidate.kind
        && existing.status == candidate.status
        && existing.visibility == candidate.visibility
        && existing.normalized_fact == candidate.normalized_fact
        && existing.sources == candidate.sources
        && existing
            .scan_metadata
            .as_ref()
            .zip(candidate.scan_metadata.as_ref())
            .is_some_and(|(existing, candidate)| existing.same_review_payload(candidate))
        && existing.replacement == candidate.replacement
        && existing.observed_at == candidate.observed_at
        && existing.verified_at == candidate.verified_at
}

const fn scan_claim_kind(kind: ScanClaimKind) -> ClaimKind {
    match kind {
        ScanClaimKind::ProjectProfile => ClaimKind::ProjectProfile,
        ScanClaimKind::Decision => ClaimKind::Decision,
        ScanClaimKind::Convention => ClaimKind::Convention,
        ScanClaimKind::Preference => ClaimKind::Preference,
        ScanClaimKind::Workflow => ClaimKind::Workflow,
        ScanClaimKind::DependencyEvidence => ClaimKind::DependencyEvidence,
        ScanClaimKind::Outcome => ClaimKind::Outcome,
        ScanClaimKind::Question => ClaimKind::Question,
    }
}

/// Validate the complete reviewed scan claim set before any registry or index mutation.
///
/// # Errors
///
/// Returns an error for an invalid inventory digest, malformed or unreviewed claims,
/// duplicate claim IDs, unsafe evidence locators, or likely credentials.
pub fn validate_reviewed_claims_for_apply(validated: &ValidatedClaims) -> Result<(), WikiError> {
    validate_prefixed_sha256("scan inventory digest", &validated.inventory_digest)?;
    crate::reject_likely_credentials(
        reviewed_claim_provenance_summary(&validated.inventory_digest).as_bytes(),
    )
    .map_err(|_| {
        WikiError::Verification(
            "reviewed scan provenance renderer produced likely credential material".to_owned(),
        )
    })?;
    let mut claim_ids = BTreeSet::new();
    let mut expected_promotion_candidates = BTreeSet::new();
    for claim in &validated.collection_claims {
        validate_reviewed_claim_basics(claim)?;
        if !claim_ids.insert(claim.claim_id.as_str()) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate reviewed claim ID `{}`",
                claim.claim_id
            )));
        }
        if claim.global_promotion_candidate {
            expected_promotion_candidates.insert(claim.claim_id.as_str());
        }
    }
    let supplied_promotion_candidates = validated
        .promotion_candidates
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if supplied_promotion_candidates.len() != validated.promotion_candidates.len()
        || supplied_promotion_candidates != expected_promotion_candidates
    {
        return Err(WikiError::InvalidInput(
            "promotion candidates must exactly match reviewed candidate flags".to_owned(),
        ));
    }
    Ok(())
}

fn validate_reviewed_claim_basics(claim: &ReviewedClaim) -> Result<(), WikiError> {
    if claim.schema_version != SCAN_SCHEMA_VERSION || !claim.agent_reviewed {
        return Err(WikiError::InvalidInput(
            "store accepts only schema-v1 agent-reviewed scan claims".to_owned(),
        ));
    }
    if claim.claim_id.is_empty()
        || claim.claim_id.len() > 96
        || claim.claim_id.starts_with('-')
        || claim.claim_id.ends_with('-')
        || !claim
            .claim_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(WikiError::InvalidInput(
            "reviewed claim ID must be bounded lowercase ASCII".to_owned(),
        ));
    }
    if claim.statement.trim().is_empty()
        || claim.statement.trim() != claim.statement
        || claim.statement.len() > 800
        || claim.statement.chars().any(char::is_control)
        || claim.evidence.is_empty()
        || claim.evidence.len() > 16
    {
        return Err(WikiError::InvalidInput(
            "reviewed claim must remain a bounded atomic statement with evidence".to_owned(),
        ));
    }
    let mut locators = BTreeSet::new();
    for evidence in &claim.evidence {
        let path = Path::new(&evidence.locator);
        if evidence.locator.is_empty()
            || evidence.locator.contains('\\')
            || path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || !locators.insert(evidence.locator.clone())
        {
            return Err(WikiError::InvalidInput(
                "reviewed claim evidence requires unique portable relative locators".to_owned(),
            ));
        }
        validate_prefixed_sha256("reviewed evidence digest", &evidence.content_digest)?;
    }
    for (label, value, limit) in [
        ("reviewed version", claim.version.as_deref(), 128_usize),
        ("reviewed revision", claim.revision.as_deref(), 256_usize),
        (
            "reviewed applicability",
            claim.applicability.as_deref(),
            400_usize,
        ),
    ] {
        if value.is_some_and(|value| {
            value.trim().is_empty()
                || value.trim() != value
                || value.len() > limit
                || value.chars().any(char::is_control)
        }) {
            return Err(WikiError::InvalidInput(format!("invalid {label}")));
        }
    }
    crate::reject_likely_credentials(claim.statement.as_bytes()).map_err(|_| {
        WikiError::InvalidInput(format!(
            "reviewed scan claim `{}` statement contains likely credential material",
            claim.claim_id
        ))
    })?;
    if claim.global_promotion_candidate
        && (!matches!(
            claim.kind,
            ScanClaimKind::Decision | ScanClaimKind::Convention | ScanClaimKind::Workflow
        ) || claim.applicability.is_none())
    {
        return Err(WikiError::InvalidInput(
            "automatic promotion candidates require a reusable decision, convention, or workflow with explicit applicability"
                .to_owned(),
        ));
    }
    Ok(())
}

fn is_pending_automatic_promotion(claim: &CanonicalClaim, source_collection_id: &str) -> bool {
    claim.collection_id == source_collection_id
        && claim.status != AssertionStatus::Superseded
        && claim.scan_metadata.as_ref().is_some_and(|metadata| {
            metadata.review_status == ScanReviewStatus::AgentReviewed
                && metadata.global_promotion_candidate
                && metadata.promotion_status == ScanPromotionStatus::PendingReview
        })
}

fn is_promoted_automatic_source(claim: &CanonicalClaim, source_collection_id: &str) -> bool {
    claim.collection_id == source_collection_id
        && claim.status != AssertionStatus::Superseded
        && claim.scan_metadata.as_ref().is_some_and(|metadata| {
            metadata.review_status == ScanReviewStatus::AgentReviewed
                && metadata.global_promotion_candidate
                && metadata.promotion_status == ScanPromotionStatus::Promoted
        })
}

fn validate_automatic_promotion_source(source: &CanonicalClaim) -> Result<(), WikiError> {
    let metadata = source.scan_metadata.as_ref().ok_or_else(|| {
        WikiError::Verification("automatic promotion source lacks typed metadata".to_owned())
    })?;
    if metadata.review_status != ScanReviewStatus::AgentReviewed
        || !metadata.global_promotion_candidate
        || metadata.promotion_status != ScanPromotionStatus::PendingReview
        || metadata.applicability.is_none()
        || !matches!(
            source.kind,
            ClaimKind::Decision | ClaimKind::Convention | ClaimKind::Workflow
        )
    {
        return Err(WikiError::Conflict(
            "reviewed scan claim is not eligible for automatic user-root promotion".to_owned(),
        ));
    }
    Ok(())
}

fn automatic_scan_promotion_request(source: &CanonicalClaim) -> Result<RememberRequest, WikiError> {
    let key_material = serde_json::to_vec(&(
        "reviewed-scan-promotion-v1",
        source.kind,
        source.normalized_fact.as_str(),
    ))
    .map_err(|error| WikiError::Io(error.to_string()))?;
    let key_digest = sha256_digest(&key_material);
    let key_suffix = key_digest
        .strip_prefix("sha256:")
        .ok_or_else(|| WikiError::Io("promotion digest lost its algorithm prefix".to_owned()))?;
    Ok(RememberRequest {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        claim_key: format!("promoted.{key_suffix}"),
        claim_id: None,
        locator: format!(".hive/knowledge/Claims/user-root/pending-{key_suffix}.md"),
        kind: source.kind,
        status: source.status,
        visibility: RagVisibility::Shared,
        normalized_fact: source.normalized_fact.clone(),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::ReviewedArtifact,
            summary: "Automatically promoted reviewed scan claim".to_owned(),
            locator: source.locator.clone(),
            digest: source.provenance.digest.clone(),
        },
        sources: vec![source.locator.clone()],
        supersedes: Vec::new(),
        expected_active_digest: None,
        observed_at: source.observed_at.clone(),
        verified_at: source.verified_at.clone(),
    })
}

fn advance_scan_promotion_status(
    mut source: CanonicalClaim,
    status: ScanPromotionStatus,
    revision: u64,
) -> Result<CanonicalClaim, WikiError> {
    let metadata = source.scan_metadata.as_mut().ok_or_else(|| {
        WikiError::Verification("automatic promotion source lost typed metadata".to_owned())
    })?;
    metadata.promotion_status = status;
    source.revision = revision;
    source.digest = claim_digest(&source);
    Ok(source)
}

fn apply_remember_plan_to_claims(claims: &mut Vec<CanonicalClaim>, plan: &RememberPlan) {
    for rewritten in &plan.superseded_claims {
        replace_loaded_claim(claims, rewritten.clone());
    }
    if let Some(new_claim) = &plan.new_claim {
        if claims
            .iter()
            .any(|claim| claim.claim_id == new_claim.claim_id)
        {
            replace_loaded_claim(claims, new_claim.clone());
        } else {
            claims.push(new_claim.clone());
        }
    }
}

fn validate_remember_plan_shape(plan: &RememberPlan) -> Result<(), WikiError> {
    match plan.disposition {
        RememberDisposition::Noop
            if plan.new_claim.is_none() && plan.superseded_claims.is_empty() =>
        {
            Ok(())
        }
        RememberDisposition::Insert
            if plan.new_claim.is_some() && plan.superseded_claims.is_empty() =>
        {
            Ok(())
        }
        RememberDisposition::Supersede
            if plan.new_claim.is_some() && !plan.superseded_claims.is_empty() =>
        {
            Ok(())
        }
        _ => Err(WikiError::InvalidInput(
            "remember disposition does not match its canonical write set".to_owned(),
        )),
    }
}

fn resolve_scan_promotion_source(
    claims: &[CanonicalClaim],
    source_collection_id: &str,
    review_id: &str,
    expected_source_digest: &str,
) -> Result<CanonicalClaim, WikiError> {
    validate_prefixed_sha256("expected scan source digest", expected_source_digest)?;
    let mut matches = claims.iter().filter(|claim| {
        claim.collection_id == source_collection_id
            && claim.status != AssertionStatus::Superseded
            && scan_review_id(claim) == Some(review_id)
    });
    let source = matches.next().ok_or_else(|| {
        WikiError::InvalidInput("active reviewed scan claim is unknown".to_owned())
    })?;
    if matches.next().is_some() {
        return Err(WikiError::Conflict(
            "multiple active reviewed scan claims match the promotion request".to_owned(),
        ));
    }
    if source.digest != expected_source_digest {
        return Err(WikiError::Conflict(
            "reviewed scan claim changed after promotion preview".to_owned(),
        ));
    }
    let metadata = source.scan_metadata.as_ref().ok_or_else(|| {
        WikiError::Verification("reviewed scan claim lacks typed metadata".to_owned())
    })?;
    if metadata.review_status != ScanReviewStatus::AgentReviewed
        || !metadata.global_promotion_candidate
        || matches!(
            metadata.promotion_status,
            ScanPromotionStatus::NotCandidate | ScanPromotionStatus::Rejected
        )
    {
        return Err(WikiError::Conflict(
            "reviewed scan claim is not eligible for user-root promotion".to_owned(),
        ));
    }
    Ok(source.clone())
}

fn validate_scan_promotion_request(
    source: &CanonicalClaim,
    request: &RememberRequest,
) -> Result<(), WikiError> {
    if request.collection_id != USER_ROOT_COLLECTION_ID
        || request.claim_key.starts_with("scan.")
        || request.normalized_fact != source.normalized_fact
        || request.kind != source.kind
        || request.status != source.status
        || request.visibility != RagVisibility::Shared
        || request.provenance.source_kind != RememberSourceKind::ReviewedArtifact
        || request.provenance.locator != source.locator
        || request.provenance.digest != source.provenance.digest
        || !request
            .sources
            .iter()
            .any(|locator| locator == &source.locator)
    {
        return Err(WikiError::InvalidInput(
            "promotion request must preserve the exact reviewed fact, kind, status, provenance, and source locator"
                .to_owned(),
        ));
    }
    Ok(())
}

fn promoted_claim_from_plan(
    plan: &RememberPlan,
    claims: &[CanonicalClaim],
    request: &RememberRequest,
) -> Result<CanonicalClaim, WikiError> {
    if let Some(claim) = &plan.new_claim {
        return Ok(claim.clone());
    }
    claims
        .iter()
        .find(|claim| {
            claim.collection_id == USER_ROOT_COLLECTION_ID
                && claim.claim_key == request.claim_key
                && claim.status != AssertionStatus::Superseded
        })
        .cloned()
        .ok_or_else(|| {
            WikiError::Verification("promotion no-op lost its user-root claim".to_owned())
        })
}

fn remember_plan_writes(
    plan: &RememberPlan,
    known_collections: &BTreeSet<&str>,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, WikiError> {
    validate_remember_plan_shape(plan)?;
    let mut writes = BTreeMap::new();
    if let Some(claim) = &plan.new_claim {
        validate_store_claim(claim, known_collections)?;
        insert_claim_write(&mut writes, claim)?;
    }
    for claim in &plan.superseded_claims {
        validate_store_claim(claim, known_collections)?;
        insert_claim_write(&mut writes, claim)?;
    }
    Ok(writes)
}

fn validate_store_claim(
    claim: &CanonicalClaim,
    known_collections: &BTreeSet<&str>,
) -> Result<(), WikiError> {
    if !known_collections.contains(claim.collection_id.as_str()) {
        return Err(WikiError::InvalidInput(format!(
            "claim `{}` references unknown collection `{}`",
            claim.claim_id, claim.collection_id
        )));
    }
    let expected = claim_locator(&claim.collection_id, &claim.claim_id);
    if claim.locator != expected {
        return Err(WikiError::InvalidInput(format!(
            "claim `{}` must use canonical locator `{expected}`",
            claim.claim_id
        )));
    }
    render_claim_markdown(claim).map_err(rag_error)?;
    Ok(())
}

fn insert_claim_write(
    writes: &mut BTreeMap<PathBuf, Vec<u8>>,
    claim: &CanonicalClaim,
) -> Result<(), WikiError> {
    let expected = claim_locator(&claim.collection_id, &claim.claim_id);
    if claim.locator != expected {
        return Err(WikiError::InvalidInput(format!(
            "claim `{}` is outside its canonical Claims path",
            claim.claim_id
        )));
    }
    let bytes = render_claim_markdown(claim)
        .map_err(rag_error)?
        .into_bytes();
    let path = PathBuf::from(expected);
    if writes.insert(path.clone(), bytes).is_some() {
        return Err(WikiError::InvalidInput(format!(
            "duplicate canonical claim write `{}`",
            path.display()
        )));
    }
    Ok(())
}

fn claim_locator(collection_id: &str, claim_id: &str) -> String {
    format!("{CLAIMS_RELATIVE}/{collection_id}/{claim_id}.md")
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

fn json_bytes(value: &impl Serialize, label: &str) -> Result<Vec<u8>, WikiError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| WikiError::Io(format!("cannot serialize {label}: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn rag_trust_bytes_for_manifest(
    manifest: &GenerationManifest,
    manifest_bytes: &[u8],
) -> Result<Vec<u8>, WikiError> {
    if manifest.schema_version != RAG_SCHEMA_VERSION || manifest.generation == 0 {
        return Err(WikiError::Verification(
            "cannot bind an unsupported RAG generation manifest".to_owned(),
        ));
    }
    validate_prefixed_sha256("manifest logical digest", &manifest.logical_digest)?;
    validate_prefixed_sha256("manifest SQLite digest", &manifest.sqlite_digest)?;
    let binding = RagTrustBinding {
        schema_version: RAG_TRUST_SCHEMA_VERSION,
        generation: manifest.generation,
        logical_digest: manifest.logical_digest.clone(),
        entry_count: manifest.entry_count,
        sqlite_digest: manifest.sqlite_digest.clone(),
        manifest_digest: sha256_digest(manifest_bytes),
    };
    json_bytes(&binding, "RAG canonical trust binding")
}

fn parse_generation_manifest(bytes: &[u8]) -> Result<GenerationManifest, WikiError> {
    let manifest: GenerationManifest = serde_json::from_slice(bytes).map_err(|error| {
        WikiError::Verification(format!("invalid RAG generation manifest: {error}"))
    })?;
    if manifest.schema_version != RAG_SCHEMA_VERSION || manifest.generation == 0 {
        return Err(WikiError::Verification(
            "RAG generation manifest has an unsupported schema or generation".to_owned(),
        ));
    }
    validate_prefixed_sha256("manifest logical digest", &manifest.logical_digest)?;
    validate_prefixed_sha256("manifest SQLite digest", &manifest.sqlite_digest)?;
    Ok(manifest)
}

pub(crate) fn verify_rag_trust_bytes(
    manifest: &GenerationManifest,
    manifest_bytes: &[u8],
    trust_bytes: &[u8],
) -> Result<(), WikiError> {
    let binding: RagTrustBinding = serde_json::from_slice(trust_bytes).map_err(|error| {
        WikiError::Verification(format!("invalid RAG canonical trust binding: {error}"))
    })?;
    if binding.schema_version != RAG_TRUST_SCHEMA_VERSION
        || binding.generation == 0
        || validate_prefixed_sha256("trusted logical digest", &binding.logical_digest).is_err()
        || validate_prefixed_sha256("trusted SQLite digest", &binding.sqlite_digest).is_err()
        || validate_prefixed_sha256("trusted manifest digest", &binding.manifest_digest).is_err()
    {
        return Err(WikiError::Verification(
            "RAG canonical trust binding has invalid fields".to_owned(),
        ));
    }
    if json_bytes(&binding, "RAG canonical trust binding")? != trust_bytes {
        return Err(WikiError::Verification(
            "RAG canonical trust binding is not canonically encoded".to_owned(),
        ));
    }
    if binding.generation != manifest.generation
        || binding.logical_digest != manifest.logical_digest
        || binding.entry_count != manifest.entry_count
        || binding.sqlite_digest != manifest.sqlite_digest
        || binding.manifest_digest != sha256_digest(manifest_bytes)
    {
        return Err(WikiError::Verification(
            "RAG manifest and index are not bound to canonical generation state".to_owned(),
        ));
    }
    Ok(())
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

fn read_bounded_required(
    root: &Dir,
    relative: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, WikiError> {
    read_bounded_optional(root, relative, max_bytes, label)?.ok_or_else(|| {
        WikiError::Verification(format!("{label} is missing: {}", relative.display()))
    })
}

fn preflight_bounded_required(
    root: &Dir,
    relative: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<(), WikiError> {
    let Some((parent, name)) = crate::capability_parent(root, relative, false)? else {
        return Err(WikiError::Verification(format!(
            "{label} is missing: {}",
            relative.display()
        )));
    };
    let metadata = parent.symlink_metadata(&name).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            WikiError::Verification(format!("{label} is missing: {}", relative.display()))
        } else {
            WikiError::Io(format!("cannot inspect {label}: {error}"))
        }
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes as u64 {
        return Err(WikiError::Verification(format!(
            "{label} is not a bounded regular file"
        )));
    }
    Ok(())
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

fn directory_names(directory: &Dir, label: &str) -> Result<Vec<OsString>, WikiError> {
    let mut names = directory
        .entries()
        .map_err(|error| WikiError::Io(format!("cannot scan {label}: {error}")))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name())
                .map_err(|error| WikiError::Io(format!("cannot scan {label}: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    Ok(names)
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
            "{label} identity changed before capability pinning"
        )));
    }
    Ok(current)
}

fn absent_claim_directories(
    root: &Dir,
    writes: &[(PathBuf, Vec<u8>)],
) -> Result<Vec<PathBuf>, WikiError> {
    let mut directories = BTreeSet::new();
    for (path, _) in writes {
        if !path.starts_with(CLAIMS_RELATIVE) {
            continue;
        }
        let mut parent = path.parent();
        while let Some(directory) = parent {
            if directory.as_os_str().is_empty() || directory == Path::new(".hive") {
                break;
            }
            if !crate::capability_directory_exists(root, directory)? {
                directories.insert(directory.to_path_buf());
            }
            parent = directory.parent();
        }
    }
    let mut directories = directories.into_iter().collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    Ok(directories)
}

fn cleanup_absent_directories(root: &Dir, directories: &[PathBuf]) -> Result<(), WikiError> {
    for directory in directories {
        crate::remove_capability_empty_directory(root, directory)?;
    }
    Ok(())
}

fn validate_dirty_state(dirty: &PersistentDirtyState) -> Result<(), WikiError> {
    if dirty.schema_version != DIRTY_SCHEMA_VERSION
        || dirty.target_generation <= dirty.base_generation
        || dirty.entries.is_empty()
        || dirty.entries.len() > MAX_DIRTY_ENTRIES
    {
        return Err(WikiError::Verification(
            "RAG dirty journal has invalid generation lineage or empty write set".to_owned(),
        ));
    }
    validate_prefixed_sha256("dirty base manifest digest", &dirty.base_manifest_digest)?;
    let mut locators = BTreeSet::new();
    for entry in &dirty.entries {
        if Path::new(&entry.locator).is_absolute()
            || entry.locator.contains("..")
            || entry.locator.contains('\\')
            || !locators.insert(entry.locator.as_str())
        {
            return Err(WikiError::Verification(
                "RAG dirty journal contains an unsafe canonical locator".to_owned(),
            ));
        }
        validate_prefixed_sha256("dirty target digest", &entry.target_digest)?;
    }
    Ok(())
}

fn validate_external_ledger_relative(relative: &Path) -> Result<(), WikiError> {
    let components = relative.components().collect::<Vec<_>>();
    let [Component::Normal(hive), Component::Normal(index), Component::Normal(file)] =
        components.as_slice()
    else {
        return Err(WikiError::InvalidInput(
            "external canonical ledger must be one .hive/index JSON file".to_owned(),
        ));
    };
    let file = file.to_str().ok_or_else(|| {
        WikiError::InvalidInput("external canonical ledger filename must be UTF-8".to_owned())
    })?;
    if *hive != OsStr::new(".hive")
        || *index != OsStr::new("index")
        || !Path::new(file)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        || file.len() > 128
    {
        return Err(WikiError::InvalidInput(
            "external canonical ledger must be one .hive/index JSON file".to_owned(),
        ));
    }
    Ok(())
}

fn validate_prefixed_sha256(label: &str, value: &str) -> Result<(), WikiError> {
    let Some(raw) = value.strip_prefix("sha256:") else {
        return Err(WikiError::InvalidInput(format!(
            "{label} must use the sha256 digest form"
        )));
    };
    if raw.len() != 64
        || !raw
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WikiError::InvalidInput(format!(
            "{label} must use the sha256 digest form"
        )));
    }
    Ok(())
}

fn empty_manifest_digest() -> String {
    sha256_digest(b"rag-store-empty-manifest-v1")
}

fn raw_sha256(value: &str) -> &str {
    value
        .strip_prefix("sha256:")
        .expect("hive-core SHA-256 digests use the sha256 prefix")
}

fn new_collection_instance_identity() -> Result<String, WikiError> {
    let unix_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| WikiError::Io(format!("system clock precedes Unix epoch: {error}")))?
        .as_nanos();
    let sequence = COLLECTION_INSTANCE_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
    let material = format!(
        "collection-instance-v1\0{unix_nanos}\0{}\0{sequence}",
        std::process::id()
    );
    Ok(format!(
        "instance-{}",
        raw_sha256(&sha256_digest(material.as_bytes()))
    ))
}

fn locator_key(locator: &str) -> String {
    if cfg!(windows) {
        locator.to_lowercase()
    } else {
        locator.to_owned()
    }
}

fn path_to_locator(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn semantic_authority_digest(
    registry: &CollectionRegistry,
    request: &RetrievalRequest,
) -> Result<String, WikiError> {
    let target = crate::rag::semantic_target_collection(registry, request).map_err(rag_error)?;
    let by_id = registry.by_id();
    let selected = by_id
        .get(target.as_str())
        .ok_or_else(|| WikiError::Verification("semantic collection is absent".to_owned()))?;
    let current = request
        .current_collection_id
        .as_deref()
        .map(|id| {
            by_id
                .get(id)
                .ok_or_else(|| WikiError::Verification("current collection is absent".to_owned()))
        })
        .transpose()?;
    serde_json::to_vec(&("hive-vector-authority-v1", selected, current))
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|_| WikiError::Verification("semantic authority is invalid".to_owned()))
}

fn rag_error(error: RagError) -> WikiError {
    match error {
        RagError::InvalidInput(message) => WikiError::InvalidInput(message),
        RagError::Conflict(message) => WikiError::Conflict(message),
        RagError::RepairRequired(message) => WikiError::Verification(message),
        RagError::Io(message) => WikiError::Io(message),
        RagError::Sqlite(message) => WikiError::Sqlite(message),
    }
}

#[cfg(test)]
thread_local! {
    static FAIL_AFTER_CANONICAL_WRITES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn injected_failure_after_canonical_writes() -> Result<(), WikiError> {
    FAIL_AFTER_CANONICAL_WRITES.with(|failure| {
        if failure.replace(false) {
            Err(WikiError::Io(
                "injected store failure after canonical writes".to_owned(),
            ))
        } else {
            Ok(())
        }
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
    use crate::rag::{plan_remember, RagVisibility, RememberRequest};
    use crate::scan::{
        build_inventory, diff_inventory, validate_claims, ClaimEvidence, ClaimEvidenceKind,
        ScanFileKind, ScanInputFile, ScanLimits, ScanOptions, ScanRootKind,
    };
    use crate::shared::{
        KnowledgeLanguage, KnowledgeVisibility, ProjectRegistry, RegisteredProject,
        PROJECT_REGISTRY_RELATIVE,
    };
    use tempfile::TempDir;

    fn digest(value: &str) -> String {
        sha256_digest(value.as_bytes())
    }

    fn raw_digest(value: &str) -> String {
        raw_sha256(&digest(value)).to_owned()
    }

    fn store() -> (TempDir, RagStore) {
        let temporary = tempfile::tempdir().expect("temporary root");
        let store = RagStore::open(temporary.path()).expect("open store");
        store.ensure_registry().expect("initialize store");
        (temporary, store)
    }

    fn registration(root: &Path, inventory: &str) -> CollectionRegistration {
        CollectionRegistration {
            collection_id: None,
            kind: CollectionKind::Directory,
            state: CollectionState::Attached,
            aliases: vec!["reviewed-directory".to_owned()],
            local_locator: Some(root.to_path_buf()),
            source_project_id: None,
            default_visibility: CollectionVisibility::ProjectPrivate,
            portable_identity: None,
            reviewed_inventory_digest: Some(digest(inventory)),
        }
    }

    fn remember_plan(collection_id: &str, seed: &str, revision: u64) -> RememberPlan {
        let claim_id = format!("claim-{}", raw_digest(seed));
        let locator = claim_locator(collection_id, &claim_id);
        plan_remember(
            &[],
            &RememberRequest {
                collection_id: collection_id.to_owned(),
                claim_key: format!("preference.{seed}"),
                claim_id: Some(claim_id),
                locator,
                kind: ClaimKind::Preference,
                status: AssertionStatus::UserStated,
                visibility: RagVisibility::ProjectPrivate,
                normalized_fact: format!("Prefer {seed} output."),
                provenance: ClaimProvenance {
                    source_kind: RememberSourceKind::UserStatement,
                    summary: format!("Reviewed user preference for {seed}"),
                    locator: format!("request:{seed}"),
                    digest: digest(seed),
                },
                sources: vec![format!("request:{seed}")],
                supersedes: Vec::new(),
                expected_active_digest: None,
                observed_at: None,
                verified_at: None,
            },
            revision,
        )
        .expect("remember plan")
    }

    fn wiki_bytes(id: &str, body: &str) -> Vec<u8> {
        format!(
            "---\nschema_version: 1\nid: {id}\nkind: concept\nsummary: {id} summary\ntags:\n- stable\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: '2026-08-01T00:00:00Z'\nupdated_at: '2026-08-01T00:00:00Z'\n---\n\n{body}\n"
        )
        .into_bytes()
    }

    fn scan_inventory(files: &[(&str, &[u8])]) -> crate::scan::ScanInventory {
        let inputs = files
            .iter()
            .map(|(path, bytes)| ScanInputFile {
                relative_path: path,
                bytes,
                observed_byte_len: bytes.len(),
                tracked: true,
                file_kind: ScanFileKind::Regular,
            })
            .collect::<Vec<_>>();
        build_inventory(
            &inputs,
            ScanOptions {
                root_kind: ScanRootKind::Git,
                include_untracked: false,
                limits: ScanLimits::default(),
            },
            None,
        )
        .expect("scan inventory")
    }

    fn reviewed_scan_claim(
        claim_id: &str,
        statement: &str,
        locator: &str,
        inventory: &crate::scan::ScanInventory,
    ) -> ReviewedClaim {
        let content_digest = inventory
            .entries
            .iter()
            .find(|entry| entry.relative_path == locator)
            .and_then(|entry| entry.content_digest.clone())
            .expect("included evidence digest");
        ReviewedClaim {
            schema_version: SCAN_SCHEMA_VERSION,
            claim_id: claim_id.to_owned(),
            kind: ScanClaimKind::ProjectProfile,
            statement: statement.to_owned(),
            version: None,
            revision: None,
            applicability: None,
            evidence: vec![ClaimEvidence {
                locator: locator.to_owned(),
                content_digest,
                kind: ClaimEvidenceKind::Document,
            }],
            agent_reviewed: true,
            global_promotion_candidate: false,
        }
    }

    #[test]
    fn collection_mapping_is_stable_and_never_path_derived() {
        let (_temporary, store) = store();
        let first_root = tempfile::tempdir().expect("first collection");
        let second_root = tempfile::tempdir().expect("second collection");
        let first = store
            .register_collection(registration(first_root.path(), "identical-inventory"))
            .expect("register first");
        let second = store
            .register_collection(registration(second_root.path(), "identical-inventory"))
            .expect("register second");
        assert_ne!(
            first.collection.collection_id,
            second.collection.collection_id
        );
        assert!(!first.collection.collection_id.contains(
            first_root
                .path()
                .file_name()
                .expect("basename")
                .to_string_lossy()
                .as_ref()
        ));
        let repeated = store
            .register_collection(registration(first_root.path(), "changed-inventory"))
            .expect("reuse existing locator mapping");
        assert_eq!(
            repeated.collection.collection_id,
            first.collection.collection_id
        );

        let moved = tempfile::tempdir().expect("moved collection");
        let stabilized = store
            .register_collection(CollectionRegistration {
                collection_id: Some(first.collection.collection_id.clone()),
                local_locator: Some(moved.path().to_path_buf()),
                reviewed_inventory_digest: Some(digest("changed-inventory")),
                ..registration(moved.path(), "ignored")
            })
            .expect("update stable mapping");
        assert_eq!(
            stabilized.collection.collection_id,
            first.collection.collection_id
        );
        assert_eq!(
            stabilized.collection.local_locator.as_deref(),
            Some(
                crate::shared::canonical_root(moved.path())
                    .expect("canonical moved root")
                    .display()
                    .to_string()
                    .as_str()
            )
        );
    }

    #[test]
    fn project_registry_sync_attaches_and_detaches_stable_collections() {
        let (_temporary, store) = store();
        let project = tempfile::tempdir().expect("project root");
        let project_root =
            crate::shared::canonical_root(project.path()).expect("canonical project root");
        let enabled = ProjectRegistry {
            schema_version: 1,
            projects: vec![RegisteredProject {
                id: "project-b".to_owned(),
                root: project_root,
                enabled: true,
                language: KnowledgeLanguage::Both,
                visibility: KnowledgeVisibility::Shared,
            }],
        };
        store
            .sync_project_registry(&enabled)
            .expect("attach registered project");
        let attached = store
            .load_registry()
            .expect("attached registry")
            .collections
            .into_iter()
            .find(|collection| collection.source_project_id.as_deref() == Some("project-b"))
            .expect("project collection");
        assert_eq!(attached.state, CollectionState::Attached);
        assert_eq!(attached.default_visibility, CollectionVisibility::Shared);
        assert!(attached.local_locator.is_some());

        let stable_id = attached.collection_id;
        let disabled = ProjectRegistry {
            schema_version: 1,
            projects: vec![RegisteredProject {
                enabled: false,
                ..enabled.projects[0].clone()
            }],
        };
        store
            .sync_project_registry(&disabled)
            .expect("detach disabled project");
        let detached = store
            .load_registry()
            .expect("detached registry")
            .collections
            .into_iter()
            .find(|collection| collection.source_project_id.as_deref() == Some("project-b"))
            .expect("detached project collection");
        assert_eq!(detached.collection_id, stable_id);
        assert_eq!(detached.state, CollectionState::Detached);
        assert!(detached.local_locator.is_none());
    }

    #[test]
    fn atomic_project_registration_uses_one_lock_and_one_normalized_commit() {
        let (temporary, store) = store();
        let project_root = tempfile::tempdir().expect("project root");
        let project = RegisteredProject {
            id: "atomic-project".to_owned(),
            root: project_root.path().to_path_buf(),
            enabled: true,
            language: KnowledgeLanguage::Both,
            visibility: KnowledgeVisibility::ProjectPrivate,
        };

        let committed = store
            .register_project_atomic(project.clone())
            .expect("single-lock atomic project registration");
        assert_eq!(committed.registry.projects.len(), 1);
        assert_eq!(committed.registry.projects[0].id, project.id);
        assert!(committed
            .store
            .changed_paths
            .iter()
            .any(|path| path == PROJECT_REGISTRY_RELATIVE));
        assert!(committed
            .store
            .changed_paths
            .iter()
            .any(|path| path == COLLECTION_REGISTRY_RELATIVE));
        let collection = store
            .load_registry()
            .expect("normalized registry")
            .collections
            .into_iter()
            .find(|collection| collection.source_project_id.as_deref() == Some("atomic-project"))
            .expect("normalized project collection");
        assert_eq!(collection.state, CollectionState::Attached);
        assert_eq!(
            crate::shared::load_project_registry(temporary.path())
                .expect("canonical project ledger"),
            committed.registry
        );
        assert_eq!(
            store.validate_current().expect("validated current commit"),
            StoreCommit {
                changed_paths: Vec::new(),
                generation: committed.store.generation,
                manifest_digest: committed.store.manifest_digest.clone(),
            }
        );

        let noop = store
            .register_project_atomic(project)
            .expect("idempotent atomic project registration");
        assert!(noop.store.changed_paths.is_empty());
        assert_eq!(noop.store.generation, committed.store.generation);
    }

    #[test]
    fn atomic_project_registration_rolls_back_both_registries_and_index() {
        let (temporary, store) = store();
        let first_root = tempfile::tempdir().expect("first project root");
        store
            .register_project_atomic(RegisteredProject {
                id: "first-project".to_owned(),
                root: first_root.path().to_path_buf(),
                enabled: true,
                language: KnowledgeLanguage::En,
                visibility: KnowledgeVisibility::Shared,
            })
            .expect("initial atomic project registration");
        let paths = [
            PROJECT_REGISTRY_RELATIVE,
            COLLECTION_REGISTRY_RELATIVE,
            SHARED_INDEX_RELATIVE,
            RAG_MANIFEST_RELATIVE,
            RAG_TRUST_RELATIVE,
        ];
        let before = paths
            .iter()
            .map(|path| {
                (
                    *path,
                    std::fs::read(temporary.path().join(path)).expect("prior transaction byte"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let second_root = tempfile::tempdir().expect("second project root");
        FAIL_AFTER_CANONICAL_WRITES.with(|failure| failure.set(true));
        let error = store
            .register_project_atomic(RegisteredProject {
                id: "second-project".to_owned(),
                root: second_root.path().to_path_buf(),
                enabled: true,
                language: KnowledgeLanguage::Ko,
                visibility: KnowledgeVisibility::ProjectPrivate,
            })
            .expect_err("injected atomic registration failure");
        assert!(matches!(error, WikiError::Io(_)));
        for (path, bytes) in before {
            assert_eq!(
                std::fs::read(temporary.path().join(path)).expect("restored transaction byte"),
                bytes,
                "rollback must restore {path}"
            );
        }
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());

        let recovered = store
            .register_project_atomic(RegisteredProject {
                id: "second-project".to_owned(),
                root: second_root.path().to_path_buf(),
                enabled: true,
                language: KnowledgeLanguage::Ko,
                visibility: KnowledgeVisibility::ProjectPrivate,
            })
            .expect("lock released after rollback");
        assert_eq!(recovered.registry.projects.len(), 2);
    }

    #[test]
    #[ignore = "scale qualification: constructs more than 50,000 confidential chunks"]
    fn confidential_oversized_corpus_does_not_disclose_limits_before_authorization() {
        let work = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let temporary = tempfile::tempdir_in(work).expect("root");
        let store = RagStore::open(temporary.path()).expect("store");
        store.ensure_registry().expect("registry");
        let wiki = temporary.path().join(WIKI_RELATIVE);
        std::fs::create_dir_all(&wiki).expect("Wiki");
        let paragraph = format!("{}\n\n", "x".repeat(601));
        let body = paragraph.repeat(800);
        for index in 0..64 {
            let id = format!("confidential-{index}");
            let bytes = wiki_bytes(&id, &body);
            std::fs::write(wiki.join(format!("{id}.md")), bytes).expect("page");
        }
        // Visibility is collection authority for Wiki pages.
        let mut registry = store.load_registry().expect("registry");
        registry.collections[0].default_visibility = CollectionVisibility::Confidential;
        std::fs::write(
            temporary.path().join(COLLECTION_REGISTRY_RELATIVE),
            registry_bytes(&registry).expect("registry bytes"),
        )
        .expect("visibility");
        store.rebuild().expect("large index");
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection(USER_ROOT_COLLECTION_ID.to_owned()),
            current_collection_id: Some(USER_ROOT_COLLECTION_ID.to_owned()),
            query: "vector build".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: Some(USER_ROOT_COLLECTION_ID.to_owned()),
        };
        let mut called = false;
        let denied = store
            .authorized_semantic_corpus(&request, RagVisibility::Confidential, |_| {
                called = true;
                Err(WikiError::Conflict("not authorized".to_owned()))
            })
            .expect_err("denied");
        assert!(called);
        assert_eq!(
            denied.to_string(),
            WikiError::Conflict("not authorized".to_owned()).to_string()
        );
        let mut approved = false;
        let oversized = store
            .authorized_semantic_corpus(&request, RagVisibility::Confidential, |_| {
                approved = true;
                Ok(())
            })
            .expect_err("oversized approved corpus");
        assert!(approved);
        assert!(oversized
            .to_string()
            .contains("semantic corpus exceeds build limits"));
    }

    #[test]
    fn confidential_corpus_callback_must_authorize_before_export() {
        let (temporary, store) = store();
        let mut plan = remember_plan(USER_ROOT_COLLECTION_ID, "confidential-vector", 2);
        plan.new_claim.as_mut().expect("claim").visibility = RagVisibility::Confidential;
        let claim = plan.new_claim.as_mut().expect("claim");
        claim.digest = claim_digest(claim);
        store.apply_remember_plan(&plan).expect("claim");
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection(USER_ROOT_COLLECTION_ID.to_owned()),
            current_collection_id: Some(USER_ROOT_COLLECTION_ID.to_owned()),
            query: "vector build".to_owned(),
            query_expansions: Vec::new(),
            top_k: 100,
            byte_budget: 4096,
            confidential_collection_id: Some(USER_ROOT_COLLECTION_ID.to_owned()),
        };
        assert!(store
            .authorized_semantic_corpus(&request, RagVisibility::Confidential, |_| Err(
                WikiError::Conflict("not authorized".to_owned())
            ))
            .is_err());
        let mut called = 0;
        let corpus = store
            .authorized_semantic_corpus(&request, RagVisibility::Confidential, |digest| {
                assert!(digest.starts_with("sha256:"));
                called += 1;
                Ok(())
            })
            .expect("authorized corpus");
        assert_eq!(called, 1);
        assert_eq!(corpus.chunks.len(), 1);
        assert_eq!(corpus.chunks[0].visibility, RagVisibility::Confidential);
        std::fs::write(
            temporary
                .path()
                .join(&plan.new_claim.as_ref().expect("claim").locator),
            b"external unindexed change",
        )
        .expect("external change");
        let mut consumed = false;
        assert!(store
            .authorized_semantic_corpus(&request, RagVisibility::Confidential, |_| {
                consumed = true;
                Ok(())
            })
            .is_err());
        assert!(!consumed);
    }

    #[test]
    fn semantic_publication_rechecks_canonical_bytes_not_only_the_index() {
        let (temporary, store) = store();
        let initial = store.load_manifest_required().expect("manifest");
        let plan = remember_plan(USER_ROOT_COLLECTION_ID, "concise", initial.generation + 1);
        store.apply_remember_plan(&plan).expect("remember");
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection(USER_ROOT_COLLECTION_ID.to_owned()),
            current_collection_id: None,
            query: "concise".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };
        let retrieved = store.retrieve(&request).expect("retrieval");
        let matches = retrieved
            .hits
            .iter()
            .map(|hit| crate::rag::SemanticMatch {
                chunk_id: hit.chunk_id.clone(),
                digest: hit.digest.clone(),
                score: 0.5,
            })
            .collect::<Vec<_>>();
        let corpus = store
            .semantic_corpus(&request, RagVisibility::ProjectPrivate)
            .expect("corpus");
        assert!(store
            .with_semantic_snapshot(
                &request,
                RagVisibility::ProjectPrivate,
                &corpus.partition_digest,
                &corpus.authority_digest,
                || Ok(()),
                || Ok(())
            )
            .is_ok());
        let path = temporary
            .path()
            .join(&plan.new_claim.as_ref().expect("claim").locator);
        std::fs::write(path, b"not canonical Markdown").expect("external edit");
        let mut published = false;
        assert!(store
            .with_semantic_snapshot(
                &request,
                RagVisibility::ProjectPrivate,
                &corpus.partition_digest,
                &corpus.authority_digest,
                || {
                    published = true;
                    Ok(())
                },
                || Ok(())
            )
            .is_err());
        assert!(!published);
        assert!(store
            .semantic_matches(&request, &retrieved.manifest_digest, &matches)
            .is_err());
        assert!(store.checked_retrieve(&request).is_err());
    }

    #[test]
    #[allow(clippy::too_many_lines)] // One transaction exercises authority drift before and after publication.
    fn semantic_publication_preserves_target_and_current_collection_authority() {
        let (temporary, store) = store();
        let work = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let project = tempfile::tempdir_in(&work).expect("project");
        let registered = store
            .register_collection(registration(
                &project.path().canonicalize().expect("root"),
                "current",
            ))
            .expect("register");
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection(USER_ROOT_COLLECTION_ID.to_owned()),
            current_collection_id: Some(registered.collection.collection_id.clone()),
            query: "vector build".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };
        let before = store
            .semantic_corpus(&request, RagVisibility::Shared)
            .expect("corpus");
        let registry_path = temporary.path().join(COLLECTION_REGISTRY_RELATIVE);
        let original = std::fs::read(&registry_path).expect("registry bytes");
        let mut registry = store.load_registry().expect("registry");
        registry
            .collections
            .iter_mut()
            .find(|item| item.collection_id == registered.collection.collection_id)
            .expect("current")
            .aliases
            .push("changed-current-authority".to_owned());
        let changed = registry_bytes(&registry.canonicalized().expect("canonical registry"))
            .expect("changed registry");
        std::fs::write(&registry_path, &changed).expect("edit registry");
        store.rebuild().expect("refresh index");
        let after = store
            .semantic_corpus(&request, RagVisibility::Shared)
            .expect("new corpus");
        assert_eq!(before.partition_digest, after.partition_digest);
        assert_ne!(before.authority_digest, after.authority_digest);
        let mut published = false;
        assert!(store
            .with_semantic_snapshot(
                &request,
                RagVisibility::Shared,
                &before.partition_digest,
                &before.authority_digest,
                || {
                    published = true;
                    Ok(())
                },
                || Ok(())
            )
            .is_err());
        assert!(!published);
        std::fs::write(&registry_path, &original).expect("restore authority");
        store.rebuild().expect("restore index");
        let mut rolled_back = false;
        assert!(store
            .with_semantic_snapshot(
                &request,
                RagVisibility::Shared,
                &before.partition_digest,
                &before.authority_digest,
                || {
                    std::fs::write(&registry_path, &changed).expect("concurrent registry edit");
                    Ok(())
                },
                || {
                    rolled_back = true;
                    Ok(())
                }
            )
            .is_err());
        assert!(rolled_back);
        std::fs::write(&registry_path, &original).expect("restore authority");
        registry = store.load_registry().expect("registry");
        registry
            .collections
            .iter_mut()
            .find(|item| item.collection_id == USER_ROOT_COLLECTION_ID)
            .expect("target")
            .aliases
            .push("changed-target-authority".to_owned());
        std::fs::write(
            &registry_path,
            registry_bytes(&registry.canonicalized().expect("canonical target"))
                .expect("target bytes"),
        )
        .expect("target edit");
        store.rebuild().expect("target index");
        let after = store
            .semantic_corpus(&request, RagVisibility::Shared)
            .expect("changed target");
        assert_ne!(before.partition_digest, after.partition_digest);
        assert!(store
            .with_semantic_snapshot(
                &request,
                RagVisibility::Shared,
                &before.partition_digest,
                &before.authority_digest,
                || Ok(()),
                || Ok(())
            )
            .is_err());
    }

    #[test]
    fn semantic_partition_freshness_and_citations_ignore_unrelated_collections() {
        let (_user, store) = store();
        let work = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let first = tempfile::tempdir_in(&work).expect("first");
        let other = tempfile::tempdir_in(&work).expect("other");
        for root in [first.path(), other.path()] {
            std::fs::create_dir_all(root.join(WIKI_RELATIVE)).expect("Wiki");
        }
        let first_file = first.path().join(WIKI_RELATIVE).join("first.md");
        let other_file = other.path().join(WIKI_RELATIVE).join("other.md");
        std::fs::write(&first_file, wiki_bytes("first", "alpha source")).expect("first page");
        std::fs::write(&other_file, wiki_bytes("other", "beta source")).expect("other page");
        let registered = store
            .register_collection(registration(
                &first.path().canonicalize().expect("first root"),
                "first",
            ))
            .expect("first registration");
        store
            .register_collection(registration(
                &other.path().canonicalize().expect("other root"),
                "other",
            ))
            .expect("other registration");
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection(registered.collection.collection_id.clone()),
            current_collection_id: None,
            query: "alpha".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };
        let before = store
            .semantic_corpus(&request, RagVisibility::ProjectPrivate)
            .expect("first corpus");
        let plan = store.semantic_search_plan(&request).expect("query plan");
        assert_eq!(plan.partitions.len(), 1);
        assert_eq!(plan.partitions[0].digest, before.partition_digest);
        std::fs::write(
            &other_file,
            b"invalid unrelated private page PRIVATE-SENTINEL",
        )
        .expect("unrelated corruption");
        assert_eq!(
            store
                .checked_retrieve(&request)
                .expect("bounded canonical read")
                .hits
                .len(),
            1
        );
        assert_eq!(
            store
                .semantic_corpus(&request, RagVisibility::ProjectPrivate)
                .expect("independent build")
                .partition_digest,
            before.partition_digest
        );
        std::fs::write(&other_file, wiki_bytes("other", "changed beta source"))
            .expect("other edit");
        store.rebuild().expect("new global generation");
        let after = store
            .semantic_corpus(&request, RagVisibility::ProjectPrivate)
            .expect("unchanged first corpus");
        assert_ne!(before.manifest_digest, after.manifest_digest);
        assert_eq!(before.partition_digest, after.partition_digest);
        store
            .with_semantic_snapshot(
                &request,
                RagVisibility::ProjectPrivate,
                &before.partition_digest,
                &before.authority_digest,
                || Ok(()),
                || Ok(()),
            )
            .expect("same partition may publish");
        std::fs::write(&first_file, b"changed selected page").expect("selected corruption");
        assert!(store.checked_retrieve(&request).is_err());
        assert!(store
            .semantic_corpus(&request, RagVisibility::ProjectPrivate)
            .is_err());
    }

    #[test]
    fn remember_persists_canonical_claim_and_fast_retrieves_without_markdown_scan() {
        let (temporary, store) = store();
        let initial = store.load_manifest_required().expect("initial manifest");
        let plan = remember_plan(USER_ROOT_COLLECTION_ID, "concise", initial.generation + 1);
        let claim = plan.new_claim.as_ref().expect("claim");
        let commit = store.apply_remember_plan(&plan).expect("apply remember");
        let claim_path = temporary.path().join(&claim.locator);
        assert!(claim_path.is_file());
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());

        let result = store
            .retrieve(&RetrievalRequest {
                scope: crate::rag::RetrievalScope::Global,
                current_collection_id: None,
                query: "concise".to_owned(),
                query_expansions: Vec::new(),
                top_k: 5,
                byte_budget: 4096,
                confidential_collection_id: None,
            })
            .expect("fast retrieval");
        assert_eq!(result.generation, commit.generation);
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].item_id, claim.claim_id);

        std::fs::write(&claim_path, b"not canonical Markdown")
            .expect("tamper canonical claim after index publication");
        let still_fast = store
            .retrieve(&RetrievalRequest {
                scope: crate::rag::RetrievalScope::Global,
                current_collection_id: None,
                query: "concise".to_owned(),
                query_expansions: Vec::new(),
                top_k: 5,
                byte_budget: 4096,
                confidential_collection_id: None,
            })
            .expect("retrieval does not scan Markdown");
        assert_eq!(still_fast.hits[0].item_id, claim.claim_id);
        assert!(store.rebuild().is_err());
    }

    #[test]
    fn canonical_write_failure_rolls_back_claim_index_manifest_and_dirty_state() {
        let (temporary, store) = store();
        let prior_index =
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("prior index");
        let prior_manifest =
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE)).expect("prior manifest");
        let prior_trust =
            std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE)).expect("prior trust binding");
        let generation = store.load_manifest_required().expect("manifest").generation + 1;
        let plan = remember_plan(USER_ROOT_COLLECTION_ID, "rollback", generation);
        let claim_path = temporary
            .path()
            .join(&plan.new_claim.as_ref().expect("claim").locator);
        FAIL_AFTER_CANONICAL_WRITES.with(|failure| failure.set(true));
        assert!(store.apply_remember_plan(&plan).is_err());
        assert!(!claim_path.exists());
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());
        assert_eq!(
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("restored index"),
            prior_index
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE)).expect("restored manifest"),
            prior_manifest
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE))
                .expect("restored trust binding"),
            prior_trust
        );
    }

    #[test]
    fn reviewed_scan_claims_are_central_and_never_mutate_scanned_root() {
        let (temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        std::fs::write(scanned.path().join("README.md"), b"reviewed evidence\n")
            .expect("source evidence");
        let registration = store
            .register_collection(registration(scanned.path(), "scan-inventory"))
            .expect("register scan collection");
        let inventory_digest = digest("scan-inventory");
        let claims = ValidatedClaims {
            inventory_digest: inventory_digest.clone(),
            collection_claims: vec![ReviewedClaim {
                schema_version: SCAN_SCHEMA_VERSION,
                claim_id: "project-purpose".to_owned(),
                kind: ScanClaimKind::ProjectProfile,
                statement: "The reviewed project documents its purpose.".to_owned(),
                version: None,
                revision: None,
                applicability: None,
                evidence: vec![ClaimEvidence {
                    locator: "README.md".to_owned(),
                    content_digest: digest("reviewed evidence\n"),
                    kind: ClaimEvidenceKind::Document,
                }],
                agent_reviewed: true,
                global_promotion_candidate: false,
            }],
            promotion_candidates: Vec::new(),
        };
        let first_commit = store
            .apply_reviewed_claims(&registration.collection.collection_id, &claims)
            .expect("apply reviewed claims");
        let replay = store
            .apply_reviewed_claims(&registration.collection.collection_id, &claims)
            .expect("idempotent reviewed-claim replay");
        assert!(replay.changed_paths.is_empty());
        assert_eq!(replay.generation, first_commit.generation);
        assert!(!scanned.path().join(".hive").exists());
        assert_eq!(
            std::fs::read(scanned.path().join("README.md")).expect("unchanged evidence"),
            b"reviewed evidence\n"
        );
        let claims_root = temporary
            .path()
            .join(CLAIMS_RELATIVE)
            .join(&registration.collection.collection_id);
        assert_eq!(
            std::fs::read_dir(claims_root)
                .expect("central claims")
                .count(),
            1
        );
    }

    #[test]
    fn atomic_scan_registration_rolls_back_registry_claims_index_and_manifest() {
        let (temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        let inventory = scan_inventory(&[("README.md", b"atomic scan evidence\n")]);
        let validated = validate_claims(
            &inventory,
            &[reviewed_scan_claim(
                "atomic-scan",
                "The project records atomic scan evidence.",
                "README.md",
                &inventory,
            )],
        )
        .expect("validated scan review");
        let mut request = registration(scanned.path(), "ignored-content-seed");
        request.reviewed_inventory_digest = Some(validated.inventory_digest.clone());
        let prior_registry =
            std::fs::read(temporary.path().join(COLLECTION_REGISTRY_RELATIVE)).expect("registry");
        let prior_index =
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("index");
        let prior_manifest =
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE)).expect("manifest");
        let prior_trust =
            std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE)).expect("trust binding");

        FAIL_AFTER_CANONICAL_WRITES.with(|failure| failure.set(true));
        assert!(store
            .register_scanned_collection_atomic(request.clone(), &validated)
            .is_err());
        assert_eq!(
            std::fs::read(temporary.path().join(COLLECTION_REGISTRY_RELATIVE))
                .expect("restored registry"),
            prior_registry
        );
        assert_eq!(
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("restored index"),
            prior_index
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE)).expect("restored manifest"),
            prior_manifest
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE))
                .expect("restored trust binding"),
            prior_trust
        );
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());
        assert!(!temporary.path().join(CLAIMS_RELATIVE).exists());

        let applied = store
            .register_scanned_collection_atomic(request, &validated)
            .expect("single scan transaction");
        assert!(applied
            .store
            .changed_paths
            .iter()
            .any(|path| path == COLLECTION_REGISTRY_RELATIVE));
        assert!(applied
            .store
            .changed_paths
            .iter()
            .any(|path| path.starts_with(CLAIMS_RELATIVE)));
    }

    #[test]
    fn scan_claim_human_review_id_is_not_misclassified_as_a_credential() {
        let (_temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        let inventory = scan_inventory(&[(
            "docs/facts/en/v0-9-skill-suite-plan.md",
            b"reviewed source fact\n",
        )]);
        let validated = validate_claims(
            &inventory,
            &[reviewed_scan_claim(
                "source-fact-v0-9-skill-suite-plan",
                "Aigent Hive source fact — v0.9 Skill Suite: The completed v0.9 Skill baseline now feeds a separate default-off Hive-native iterative execution plan.",
                "docs/facts/en/v0-9-skill-suite-plan.md",
                &inventory,
            )],
        )
        .expect("reviewed source claim");
        validate_reviewed_claims_for_apply(&validated).expect("storage preflight");
        let mut request = registration(scanned.path(), "source-fact-inventory");
        request.reviewed_inventory_digest = Some(validated.inventory_digest.clone());

        let committed = store
            .register_scanned_collection_atomic(request, &validated)
            .expect("source claim registration");
        let registry = store.load_registry().expect("stored registry");
        let claims = store.load_claims(&registry).expect("stored scan claim");
        assert!(claims.iter().any(|claim| {
            claim.collection_id == committed.collection.collection_id
                && claim.claim_key == "scan.source-fact-v0-9-skill-suite-plan"
        }));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn scan_metadata_round_trips_markdown_sqlite_and_retrieval() {
        let (temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        let inventory = scan_inventory(&[
            ("docs/convention.md", b"reviewed convention intent\n"),
            ("tests/convention.rs", b"verified convention test\n"),
        ]);
        let document_digest = inventory
            .entries
            .iter()
            .find(|entry| entry.relative_path == "docs/convention.md")
            .and_then(|entry| entry.content_digest.clone())
            .expect("document digest");
        let test_digest = inventory
            .entries
            .iter()
            .find(|entry| entry.relative_path == "tests/convention.rs")
            .and_then(|entry| entry.content_digest.clone())
            .expect("test digest");
        let validated = validate_claims(
            &inventory,
            &[ReviewedClaim {
                schema_version: SCAN_SCHEMA_VERSION,
                claim_id: "nextjs-convention".to_owned(),
                kind: ScanClaimKind::Convention,
                statement: "The project uses a reviewed Next.js convention.".to_owned(),
                version: Some("16.0.0".to_owned()),
                revision: Some("release-2026-08".to_owned()),
                applicability: Some("Next.js 16 projects with the verified test.".to_owned()),
                evidence: vec![
                    ClaimEvidence {
                        locator: "docs/convention.md".to_owned(),
                        content_digest: document_digest,
                        kind: ClaimEvidenceKind::Document,
                    },
                    ClaimEvidence {
                        locator: "tests/convention.rs".to_owned(),
                        content_digest: test_digest,
                        kind: ClaimEvidenceKind::Test,
                    },
                ],
                agent_reviewed: true,
                global_promotion_candidate: true,
            }],
        )
        .expect("typed reviewed claim");
        let mut request = registration(scanned.path(), "ignored");
        request.reviewed_inventory_digest = Some(validated.inventory_digest.clone());
        let applied = store
            .register_scanned_collection_atomic(request, &validated)
            .expect("apply typed scan claim");
        let registry = store.load_registry().expect("registry");
        let claim = store
            .load_claims(&registry)
            .expect("canonical claims")
            .into_iter()
            .find(|claim| claim.collection_id == applied.collection.collection_id)
            .expect("scan claim");
        let metadata = claim.scan_metadata.clone().expect("scan metadata");
        assert_eq!(metadata.version.as_deref(), Some("16.0.0"));
        assert_eq!(metadata.source_revision.as_deref(), Some("release-2026-08"));
        assert!(metadata.global_promotion_candidate);
        assert_eq!(metadata.evidence.len(), 2);
        let markdown = std::fs::read_to_string(temporary.path().join(&claim.locator))
            .expect("canonical Markdown");
        assert_eq!(
            parse_claim_markdown(&claim.locator, &markdown).expect("parsed claim"),
            claim
        );
        let result = store
            .retrieve(&RetrievalRequest {
                scope: RetrievalScope::Collection(applied.collection.collection_id.clone()),
                current_collection_id: None,
                query: "Next.js convention".to_owned(),
                query_expansions: Vec::new(),
                top_k: 5,
                byte_budget: 4096,
                confidential_collection_id: None,
            })
            .expect("retrieve scan claim");
        assert_eq!(result.hits[0].scan_metadata.as_ref(), Some(&metadata));

        let mut metadata_only_update = validated.clone();
        metadata_only_update.collection_claims[0].version = Some("16.0.1".to_owned());
        metadata_only_update.collection_claims[0].applicability =
            Some("Next.js 16.0.1 projects with the verified test.".to_owned());
        let updated = store
            .apply_reviewed_claims(&applied.collection.collection_id, &metadata_only_update)
            .expect("metadata-only reviewed rescan");
        assert!(!updated.changed_paths.is_empty());
        let active = store
            .load_claims(&registry)
            .expect("updated canonical claims")
            .into_iter()
            .find(|candidate| {
                candidate.collection_id == applied.collection.collection_id
                    && candidate.status != AssertionStatus::Superseded
            })
            .expect("updated active claim");
        assert_ne!(active.claim_id, claim.claim_id);
        assert_eq!(
            active
                .scan_metadata
                .as_ref()
                .and_then(|metadata| metadata.version.as_deref()),
            Some("16.0.1")
        );
    }

    #[test]
    fn reviewed_scan_promotion_preview_and_apply_are_typed_atomic_and_idempotent() {
        let (_temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        let inventory = scan_inventory(&[("README.md", b"promotion evidence\n")]);
        let mut reviewed = reviewed_scan_claim(
            "promotable-profile",
            "The reviewed project has a portable purpose.",
            "README.md",
            &inventory,
        );
        reviewed.kind = ScanClaimKind::Decision;
        reviewed.applicability = Some("Any Hive-managed repository knowledge scan".to_owned());
        reviewed.global_promotion_candidate = true;
        let validated = validate_claims(&inventory, &[reviewed]).expect("promotion candidate");
        let mut registration = registration(scanned.path(), "ignored");
        registration.reviewed_inventory_digest = Some(validated.inventory_digest.clone());
        let collection = store
            .register_scanned_collection_atomic(registration, &validated)
            .expect("atomic scan registration")
            .collection;
        let preview = store
            .preview_reviewed_scan_promotions(&collection.collection_id)
            .expect("typed promotion preview");
        assert_eq!(preview.len(), 1);
        let source = &preview[0];
        let request = RememberRequest {
            collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
            claim_key: "promoted.project-purpose".to_owned(),
            claim_id: None,
            locator: ".hive/knowledge/Claims/user-root/pending.md".to_owned(),
            kind: source.kind,
            status: source.status,
            visibility: RagVisibility::Shared,
            normalized_fact: source.normalized_fact.clone(),
            provenance: ClaimProvenance {
                source_kind: RememberSourceKind::ReviewedArtifact,
                summary: "Explicitly approved reviewed scan promotion".to_owned(),
                locator: source.locator.clone(),
                digest: source.provenance.digest.clone(),
            },
            sources: vec![source.locator.clone()],
            supersedes: Vec::new(),
            expected_active_digest: None,
            observed_at: None,
            verified_at: None,
        };
        let promoted = store
            .promote_reviewed_scan_claim_atomic(
                &collection.collection_id,
                "promotable-profile",
                &source.digest,
                &request,
            )
            .expect("atomic promotion");
        assert_eq!(
            promoted
                .source_claim
                .scan_metadata
                .as_ref()
                .map(|metadata| metadata.promotion_status),
            Some(ScanPromotionStatus::Promoted)
        );
        assert_eq!(
            promoted.promoted_claim.collection_id,
            USER_ROOT_COLLECTION_ID
        );
        assert!(store
            .preview_reviewed_scan_promotions(&collection.collection_id)
            .expect("post-promotion preview")
            .is_empty());
        let replay = store
            .promote_reviewed_scan_claim_atomic(
                &collection.collection_id,
                "promotable-profile",
                &promoted.source_claim.digest,
                &request,
            )
            .expect("idempotent promotion retry");
        assert!(replay.store.changed_paths.is_empty());
        assert_eq!(
            replay.promoted_claim.claim_id,
            promoted.promoted_claim.claim_id
        );
    }

    #[test]
    fn scan_apply_auto_promotes_safe_general_claims_and_invalidates_their_derivatives() {
        let (_temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        let inventory = scan_inventory(&[("README.md", b"promotion evidence\n")]);
        let mut reviewed = reviewed_scan_claim(
            "automatic-decision",
            "Hive keeps automatic promotion separate from retrieval.",
            "README.md",
            &inventory,
        );
        reviewed.kind = ScanClaimKind::Decision;
        reviewed.applicability = Some("All scanned Hive knowledge collections".to_owned());
        reviewed.global_promotion_candidate = true;
        let validated = validate_claims(&inventory, &[reviewed]).expect("automatic candidate");
        let mut registration = registration(scanned.path(), "ignored");
        registration.reviewed_inventory_digest = Some(validated.inventory_digest.clone());
        let collection = store
            .register_scanned_collection_atomic(registration, &validated)
            .expect("scan registration")
            .collection;

        let promoted = store
            .auto_promote_reviewed_scan_claims_atomic(&collection.collection_id)
            .expect("automatic promotion");
        assert_eq!(promoted.source_claims.len(), 1);
        assert_eq!(promoted.promoted_claims.len(), 1);
        assert_eq!(
            promoted.promoted_claims[0].collection_id,
            USER_ROOT_COLLECTION_ID
        );
        assert_eq!(
            promoted.source_claims[0]
                .scan_metadata
                .as_ref()
                .map(|metadata| metadata.promotion_status),
            Some(ScanPromotionStatus::Promoted)
        );
        assert_eq!(
            promoted.promoted_claims[0]
                .scan_metadata
                .as_ref()
                .map(|metadata| metadata.promotion_status),
            Some(ScanPromotionStatus::Promoted)
        );
        assert!(store
            .auto_promote_reviewed_scan_claims_atomic(&collection.collection_id)
            .expect("automatic promotion replay")
            .store
            .changed_paths
            .is_empty());

        let empty = validate_claims(&inventory, &[]).expect("empty rescan");
        store
            .apply_reviewed_claims(&collection.collection_id, &empty)
            .expect("source invalidation");
        let registry = store.load_registry().expect("registry");
        assert!(store
            .load_claims(&registry)
            .expect("claims")
            .iter()
            .filter(|claim| claim.collection_id == USER_ROOT_COLLECTION_ID)
            .all(|claim| claim.status == AssertionStatus::Superseded));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn reviewed_rescan_invalidates_deleted_claims_and_supersedes_renamed_evidence() {
        let (temporary, store) = store();
        let scanned = tempfile::tempdir().expect("scanned root");
        let registration = store
            .register_collection(registration(scanned.path(), "incremental-inventory"))
            .expect("register scan collection");
        let collection_id = registration.collection.collection_id;
        let before = scan_inventory(&[
            ("docs/keep.md", b"stable evidence\n"),
            ("docs/old.md", b"movable evidence\n"),
            ("docs/delete.md", b"delete-only evidence\n"),
        ]);
        let before_review = validate_claims(
            &before,
            &[
                reviewed_scan_claim(
                    "keep",
                    "The project keeps stable documentation.",
                    "docs/keep.md",
                    &before,
                ),
                reviewed_scan_claim(
                    "moved",
                    "The project documents a movable convention.",
                    "docs/old.md",
                    &before,
                ),
                reviewed_scan_claim(
                    "deleted",
                    "The project documents delete-only behavior.",
                    "docs/delete.md",
                    &before,
                ),
            ],
        )
        .expect("before reviewed claims");
        store
            .apply_reviewed_claims(&collection_id, &before_review)
            .expect("apply before claims");
        let registry = store.load_registry().expect("registry");
        let before_claims = store.load_claims(&registry).expect("before claims");
        let before_by_key = before_claims
            .iter()
            .filter(|claim| claim.collection_id == collection_id)
            .map(|claim| (claim.claim_key.clone(), claim.clone()))
            .collect::<BTreeMap<_, _>>();
        let prior_keep = before_by_key.get("scan.keep").expect("prior keep");
        let prior_moved = before_by_key.get("scan.moved").expect("prior moved");
        let prior_deleted = before_by_key.get("scan.deleted").expect("prior deleted");
        let keep_bytes = std::fs::read(temporary.path().join(&prior_keep.locator))
            .expect("prior unchanged claim bytes");

        let after = scan_inventory(&[
            ("docs/keep.md", b"stable evidence\n"),
            ("docs/new.md", b"movable evidence\n"),
        ]);
        let delta = diff_inventory(&before, &after);
        assert_eq!(delta.removed, vec!["docs/delete.md"]);
        assert_eq!(
            delta.renamed,
            vec![("docs/old.md".to_owned(), "docs/new.md".to_owned())]
        );
        let after_review = validate_claims(
            &after,
            &[
                reviewed_scan_claim(
                    "keep",
                    "The project keeps stable documentation.",
                    "docs/keep.md",
                    &after,
                ),
                reviewed_scan_claim(
                    "moved",
                    "The project documents a movable convention.",
                    "docs/new.md",
                    &after,
                ),
            ],
        )
        .expect("after reviewed claims");
        store
            .apply_reviewed_claims(&collection_id, &after_review)
            .expect("apply incremental reviewed claims");
        let after_claims = store.load_claims(&registry).expect("after claims");
        let active = after_claims
            .iter()
            .filter(|claim| {
                claim.collection_id == collection_id && claim.status != AssertionStatus::Superseded
            })
            .map(|claim| (claim.claim_key.clone(), claim))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            active.keys().cloned().collect::<Vec<_>>(),
            vec!["scan.keep".to_owned(), "scan.moved".to_owned()]
        );
        assert_eq!(active["scan.keep"].claim_id, prior_keep.claim_id);
        assert_eq!(
            std::fs::read(temporary.path().join(&prior_keep.locator))
                .expect("unchanged canonical claim bytes"),
            keep_bytes
        );
        assert_ne!(active["scan.moved"].claim_id, prior_moved.claim_id);
        let superseded_moved = after_claims
            .iter()
            .find(|claim| claim.claim_id == prior_moved.claim_id)
            .expect("superseded moved claim");
        assert_eq!(superseded_moved.status, AssertionStatus::Superseded);
        assert_eq!(
            superseded_moved.replacement.as_deref(),
            Some(active["scan.moved"].claim_id.as_str())
        );
        let invalidated_deleted = after_claims
            .iter()
            .find(|claim| claim.claim_id == prior_deleted.claim_id)
            .expect("source-invalidated deleted claim");
        assert_eq!(invalidated_deleted.status, AssertionStatus::Superseded);
        assert!(invalidated_deleted.replacement.is_none());
        assert_eq!(
            invalidated_deleted.provenance.digest,
            after.inventory_digest
        );
        assert!(invalidated_deleted
            .provenance
            .summary
            .starts_with("Source-invalidated scan claim `deleted`"));

        let stale_result = store
            .retrieve(&RetrievalRequest {
                scope: RetrievalScope::Collection(collection_id.clone()),
                current_collection_id: None,
                query: "delete-only behavior".to_owned(),
                query_expansions: Vec::new(),
                top_k: 10,
                byte_budget: 4096,
                confidential_collection_id: None,
            })
            .expect("query after source invalidation");
        assert!(stale_result
            .hits
            .iter()
            .all(|hit| hit.item_id != prior_deleted.claim_id));
        let replay = store
            .apply_reviewed_claims(&collection_id, &after_review)
            .expect("idempotent incremental replay");
        assert!(replay.changed_paths.is_empty());
    }

    #[test]
    fn rebuild_uses_only_canonical_sources_and_preserves_markdown_bytes() {
        let (temporary, store) = store();
        let wiki = temporary.path().join(WIKI_RELATIVE);
        std::fs::create_dir_all(&wiki).expect("Wiki directory");
        std::fs::write(
            wiki.join("portable.md"),
            wiki_bytes("portable", "Portable evidence."),
        )
        .expect("canonical Wiki page");
        let first = store.rebuild().expect("first rebuild");
        let markdown = std::fs::read(wiki.join("portable.md")).expect("Wiki bytes");
        let trust = std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE))
            .expect("canonical trust binding");
        std::fs::remove_file(temporary.path().join(SHARED_INDEX_RELATIVE))
            .expect("delete disposable SQLite");
        std::fs::remove_file(temporary.path().join(RAG_MANIFEST_RELATIVE))
            .expect("delete disposable manifest");
        let second = store.rebuild().expect("rebuild from canonical files");
        assert_eq!(
            std::fs::read(wiki.join("portable.md")).expect("unchanged Wiki bytes"),
            markdown
        );
        assert_eq!(first.generation, 2);
        assert_eq!(second.generation, 2);
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE))
                .expect("recreated trust binding"),
            trust
        );
        let result = store
            .retrieve(&RetrievalRequest {
                scope: crate::rag::RetrievalScope::Global,
                current_collection_id: None,
                query: "Portable".to_owned(),
                query_expansions: Vec::new(),
                top_k: 5,
                byte_budget: 4096,
                confidential_collection_id: None,
            })
            .expect("retrieval after canonical rebuild");
        assert_eq!(
            result.hits[0].locator,
            ".hive/knowledge/Wiki/portable.md#chunk=0"
        );
    }

    #[test]
    fn simultaneous_sqlite_and_manifest_replacement_cannot_bypass_canonical_trust() {
        let (temporary, store) = store();
        let current = store
            .validate_current()
            .expect("current trusted generation");
        let registry = store.load_registry().expect("canonical registry");
        let mut forged_snapshot = store
            .load_canonical_snapshot(current.generation)
            .expect("canonical snapshot");
        let forged_claim = remember_plan(
            USER_ROOT_COLLECTION_ID,
            "forged-derived-only-claim",
            current.generation,
        )
        .new_claim
        .expect("forged in-memory claim");
        forged_snapshot.claims.push(forged_claim);
        let forged = build_rag_index(&forged_snapshot).expect("internally consistent forgery");
        let request = RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "forged-derived-only-claim".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };
        assert!(
            !retrieve_serialized(&forged.sqlite_bytes, &forged.manifest, &registry, &request,)
                .expect("forged pair is internally consistent")
                .hits
                .is_empty()
        );
        let trust_before = std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE))
            .expect("canonical trust bytes");
        std::fs::write(
            temporary.path().join(SHARED_INDEX_RELATIVE),
            &forged.sqlite_bytes,
        )
        .expect("replace derived SQLite");
        std::fs::write(
            temporary.path().join(RAG_MANIFEST_RELATIVE),
            json_bytes(&forged.manifest, "forged manifest").expect("forged manifest bytes"),
        )
        .expect("replace derived manifest");

        assert!(matches!(
            store.retrieve(&request),
            Err(WikiError::Verification(_))
        ));
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_TRUST_RELATIVE))
                .expect("unchanged canonical trust bytes"),
            trust_before
        );
        let repaired = store.rebuild().expect("repair from canonical sources");
        assert_eq!(repaired.generation, current.generation);
        assert!(store
            .retrieve(&request)
            .expect("trusted retrieval after repair")
            .hits
            .is_empty());
    }

    #[test]
    fn canonical_rebuild_recreates_a_missing_trust_binding() {
        let (temporary, store) = store();
        std::fs::remove_file(temporary.path().join(RAG_TRUST_RELATIVE))
            .expect("remove trust binding");
        let request = RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "anything".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };
        assert!(matches!(
            store.retrieve(&request),
            Err(WikiError::Verification(_))
        ));
        store.rebuild().expect("rebuild missing trust binding");
        assert!(temporary.path().join(RAG_TRUST_RELATIVE).is_file());
        store
            .validate_current()
            .expect("recreated trust binding validates");
    }

    #[test]
    fn rebuild_rejects_corrupt_dirty_authority_without_mutation() {
        let (temporary, store) = store();
        std::fs::write(
            temporary.path().join(SHARED_INDEX_RELATIVE),
            b"corrupt SQLite",
        )
        .expect("corrupt disposable index");
        std::fs::write(temporary.path().join(RAG_MANIFEST_RELATIVE), b"{")
            .expect("corrupt disposable manifest");
        std::fs::write(temporary.path().join(RAG_DIRTY_RELATIVE), b"{")
            .expect("corrupt disposable dirty state");

        let error = store
            .rebuild()
            .expect_err("corrupt recovery authority must fail closed");
        assert!(matches!(error, WikiError::Verification(_)));
        assert_eq!(
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("unchanged index"),
            b"corrupt SQLite"
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE))
                .expect("unchanged manifest"),
            b"{"
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_DIRTY_RELATIVE)).expect("retained journal"),
            b"{"
        );
    }

    #[test]
    fn wiki_page_query_reads_only_the_complete_serialized_snapshot() {
        let (temporary, store) = store();
        let wiki = temporary.path().join(WIKI_RELATIVE);
        std::fs::create_dir_all(&wiki).expect("Wiki directory");
        let page = wiki.join("snapshot.md");
        std::fs::write(&page, wiki_bytes("snapshot", "Stable snapshot evidence."))
            .expect("canonical Wiki page");
        store.rebuild().expect("indexed Wiki page");
        let request = crate::rag::WikiPageQueryRequest {
            current_project_id: None,
            text: Some("snapshot".to_owned()),
            tag: Some("stable".to_owned()),
            category: Some("concept".to_owned()),
            limit: 10,
        };

        let indexed = store
            .query_wiki_pages(&request)
            .expect("query complete serialized snapshot");
        assert_eq!(indexed.len(), 1);
        assert_eq!(indexed[0].kind, "concept");
        assert_eq!(indexed[0].category, "concept");

        std::fs::write(&page, b"not canonical Markdown")
            .expect("tamper canonical page after publication");
        assert_eq!(
            store
                .query_wiki_pages(&request)
                .expect("query never scans canonical Markdown"),
            indexed
        );

        std::fs::write(
            temporary.path().join(RAG_DIRTY_RELATIVE),
            b"externally-created dirty state",
        )
        .expect("external dirty state");
        let error = store
            .query_wiki_pages(&request)
            .expect_err("dirty snapshot must fail closed");
        assert!(matches!(error, WikiError::Verification(_)));
    }

    #[test]
    fn retrieval_rejects_dirty_state_without_repair_or_mutation() {
        let (temporary, store) = store();
        store
            .begin_external_canonical_mutation(&[(
                PathBuf::from(".hive/knowledge/Wiki/pending.md"),
                wiki_bytes("pending", "Pending canonical evidence."),
            )])
            .expect("publish dirty journal");
        let protected = [
            COLLECTION_REGISTRY_RELATIVE,
            RAG_MANIFEST_RELATIVE,
            RAG_TRUST_RELATIVE,
            SHARED_INDEX_RELATIVE,
            RAG_DIRTY_RELATIVE,
        ]
        .map(|relative| {
            (
                relative,
                std::fs::read(temporary.path().join(relative)).expect("protected store bytes"),
            )
        });
        let request = RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "pending".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };

        assert!(matches!(
            store.retrieve(&request),
            Err(WikiError::Verification(_))
        ));
        for (relative, before) in protected {
            assert_eq!(
                std::fs::read(temporary.path().join(relative))
                    .expect("store bytes after retrieval"),
                before,
                "retrieval mutated {relative}"
            );
        }
    }

    #[test]
    fn external_rollback_removes_only_its_exact_dirty_journal() {
        let (temporary, store) = store();
        let expected = store
            .begin_external_canonical_mutation(&[(
                PathBuf::from(".hive/knowledge/Wiki/rolled-back.md"),
                wiki_bytes("rolled-back", "Rolled back canonical evidence."),
            )])
            .expect("publish dirty journal");
        let mut mismatched = expected.clone();
        mismatched.target_generation += 1;

        assert!(matches!(
            store.abort_external_canonical_mutation(&mismatched),
            Err(WikiError::Verification(_))
        ));
        assert!(temporary.path().join(RAG_DIRTY_RELATIVE).is_file());
        store
            .abort_external_canonical_mutation(&expected)
            .expect("remove exact rolled-back journal");
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());
    }

    #[test]
    fn uninitialized_retrieval_preflight_creates_no_store_paths() {
        let temporary = tempfile::tempdir().expect("uninitialized root");
        let store = RagStore::open(temporary.path()).expect("open root without initialization");
        let request = RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "anything".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        };

        assert!(matches!(
            store.retrieve(&request),
            Err(WikiError::Verification(_))
        ));
        assert!(!temporary.path().join(".hive").exists());
    }

    #[test]
    fn rebuild_blocks_partial_external_write_until_every_dirty_target_matches() {
        let (temporary, store) = store();
        let first_relative = PathBuf::from(".hive/knowledge/Wiki/partial-first.md");
        let second_relative = PathBuf::from(".hive/knowledge/Wiki/partial-second.md");
        let first_bytes = wiki_bytes("partial-first", "First complete target.");
        let second_bytes = wiki_bytes("partial-second", "Second complete target.");
        store
            .begin_external_canonical_mutation(&[
                (first_relative.clone(), first_bytes.clone()),
                (second_relative.clone(), second_bytes.clone()),
            ])
            .expect("publish exact external write set");
        let prior_index =
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("prior index");
        let prior_manifest =
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE)).expect("prior manifest");
        let wiki = temporary.path().join(WIKI_RELATIVE);
        std::fs::create_dir_all(&wiki).expect("Wiki directory");
        std::fs::write(temporary.path().join(&first_relative), &first_bytes)
            .expect("first of two canonical writes");

        let error = store
            .rebuild()
            .expect_err("partial write set must not be legitimized");
        assert!(matches!(error, WikiError::Verification(_)));
        assert!(temporary.path().join(RAG_DIRTY_RELATIVE).is_file());
        assert_eq!(
            std::fs::read(temporary.path().join(SHARED_INDEX_RELATIVE)).expect("unchanged index"),
            prior_index
        );
        assert_eq!(
            std::fs::read(temporary.path().join(RAG_MANIFEST_RELATIVE))
                .expect("unchanged manifest"),
            prior_manifest
        );

        std::fs::write(temporary.path().join(&second_relative), &second_bytes)
            .expect("second canonical write");
        let recovered = store.rebuild().expect("complete exact write set");
        assert_eq!(recovered.generation, 2);
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());
    }

    #[test]
    fn rebuild_requires_dirty_deletion_target_to_be_absent() {
        let (temporary, store) = store();
        let relative = PathBuf::from(".hive/knowledge/Wiki/delete-target.md");
        let absolute = temporary.path().join(&relative);
        std::fs::create_dir_all(absolute.parent().expect("Wiki parent")).expect("Wiki directory");
        std::fs::write(
            &absolute,
            wiki_bytes("delete-target", "Delete target evidence."),
        )
        .expect("canonical page");
        store.rebuild().expect("index canonical page");
        let dirty = store
            .begin_external_canonical_mutation(&[(relative, Vec::new())])
            .expect("publish deletion intent");
        assert!(dirty.entries[0].delete);
        assert!(matches!(store.rebuild(), Err(WikiError::Verification(_))));
        std::fs::remove_file(&absolute).expect("complete deletion");
        store.rebuild().expect("rebuild after exact deletion");
        assert!(!temporary.path().join(RAG_DIRTY_RELATIVE).exists());
    }

    #[test]
    fn relational_tampering_invalidates_validation_and_page_queries() {
        let (temporary, store) = store();
        let wiki = temporary.path().join(WIKI_RELATIVE);
        std::fs::create_dir_all(&wiki).expect("Wiki directory");
        std::fs::write(
            wiki.join("integrity.md"),
            wiki_bytes("integrity", "Relational integrity evidence."),
        )
        .expect("canonical Wiki page");
        store.rebuild().expect("indexed Wiki page");
        let request = crate::rag::WikiPageQueryRequest {
            current_project_id: None,
            text: None,
            tag: Some("stable".to_owned()),
            category: Some("concept".to_owned()),
            limit: 10,
        };

        for tamper in [
            "UPDATE documents SET visibility = 'confidential'",
            "UPDATE chunks SET visibility = 'confidential' WHERE item_kind = 'document'",
            "UPDATE documents SET category = 'workflow'",
            "UPDATE chunks SET text = 'poisoned retrieval payload'",
            "UPDATE chunks SET title = 'poisoned title'",
            "UPDATE chunks SET locator = 'poisoned/locator.md#chunk=0'",
            "UPDATE chunks SET digest = 'sha256:0000000000000000000000000000000000000000000000000000000000000000'",
            "UPDATE generation_manifest SET locator = 'poisoned/manifest.md'",
            "DELETE FROM chunks_fts_idx WHERE segid = (SELECT MIN(segid) FROM chunks_fts_idx)",
            "CREATE TABLE injected_auxiliary_state (payload TEXT)",
        ] {
            let connection =
                rusqlite::Connection::open(temporary.path().join(SHARED_INDEX_RELATIVE))
                    .expect("open derived SQLite for tamper fixture");
            connection.execute(tamper, []).expect("tamper derived row");
            drop(connection);

            assert!(matches!(
                store.validate_current(),
                Err(WikiError::Verification(_))
            ));
            assert!(store.query_wiki_pages(&request).is_err());
            store.rebuild().expect("repair from canonical Wiki page");
            assert_eq!(
                store
                    .query_wiki_pages(&request)
                    .expect("query repaired complete generation")
                    .len(),
                1
            );
        }
    }

    #[test]
    fn concurrent_rebuild_and_wiki_query_expose_only_old_or_new_generation() {
        let (temporary, store) = store();
        let wiki = temporary.path().join(WIKI_RELATIVE);
        std::fs::create_dir_all(&wiki).expect("Wiki directory");
        let page = wiki.join("concurrent.md");
        std::fs::write(
            &page,
            wiki_bytes("concurrent", "Concurrent snapshot version old."),
        )
        .expect("old canonical Wiki page");
        store.rebuild().expect("old indexed generation");
        let request = crate::rag::WikiPageQueryRequest {
            current_project_id: None,
            text: Some("concurrent snapshot".to_owned()),
            tag: None,
            category: Some("concept".to_owned()),
            limit: 10,
        };
        let old_digest = store
            .query_wiki_pages(&request)
            .expect("old complete generation")[0]
            .digest
            .clone();

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let writer_barrier = std::sync::Arc::clone(&barrier);
        let writer_root = temporary.path().to_path_buf();
        let writer = std::thread::spawn(move || {
            let writer_store = RagStore::open(&writer_root).expect("open writer store");
            writer_barrier.wait();
            std::fs::write(
                writer_root.join(WIKI_RELATIVE).join("concurrent.md"),
                wiki_bytes("concurrent", "Concurrent snapshot version new."),
            )
            .expect("new canonical Wiki page");
            writer_store.rebuild().expect("publish new generation")
        });
        barrier.wait();

        let mut observed_digests = Vec::new();
        for _ in 0..24 {
            let hits = store
                .query_wiki_pages(&request)
                .expect("query one complete concurrent generation");
            assert_eq!(hits.len(), 1);
            observed_digests.push(hits[0].digest.clone());
        }
        writer.join().expect("writer worker");
        let new_digest = store
            .query_wiki_pages(&request)
            .expect("new complete generation")[0]
            .digest
            .clone();
        assert_ne!(old_digest, new_digest);
        assert!(observed_digests
            .iter()
            .all(|digest| digest == &old_digest || digest == &new_digest));
    }

    #[test]
    fn legacy_shared_sqlite_with_rag_manifest_is_never_treated_as_complete() {
        let (temporary, store) = store();
        let prior_manifest = store.load_manifest_required().expect("RAG manifest");
        crate::shared::ensure_project_registry(temporary.path()).expect("legacy project registry");
        crate::shared::rebuild_legacy_shared_index_for_test(temporary.path())
            .expect("legacy shared index");

        assert!(!store
            .index_state_is_complete()
            .expect("repairable index probe"));
        let error = store
            .current_commit()
            .expect_err("legacy SQLite must require repair");
        assert!(matches!(error, WikiError::Verification(_)));
        assert!(error.to_string().contains("requires rebuild"));

        let repaired = store
            .ensure_registry()
            .expect("rebuild normalized RAG index");
        assert!(repaired.generation > prior_manifest.generation);
        assert!(repaired
            .changed_paths
            .iter()
            .any(|path| path == SHARED_INDEX_RELATIVE));
        assert!(store
            .index_state_is_complete()
            .expect("normalized index probe"));
        assert_eq!(
            store.current_commit().expect("current normalized commit"),
            StoreCommit {
                changed_paths: Vec::new(),
                generation: repaired.generation,
                manifest_digest: repaired.manifest_digest,
            }
        );
    }
}
