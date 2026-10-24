//! Explicit setup approval binds the exact local source declarations before any external read.
use super::{
    registry_bytes, CapMetadataExt, CollectionKind, CollectionState, RagStore, StoreCommit,
    UserSetupIndexScope, WikiError, COLLECTION_REGISTRY_RELATIVE,
};
use hive_core::sha256_digest;
use std::path::{Path, PathBuf};

/// One local source displayed before rebuilding an incomplete setup index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetupIndexSource {
    /// Stable collection identifier, also used when no friendly alias exists.
    pub collection_id: String,
    /// Registered aliases; these do not grant authority.
    pub aliases: Vec<String>,
    /// Exact local directory selected by the registration metadata.
    pub path: String,
}

/// Read-only preview. The digest is operation-scoped input, not stored user consent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetupIndexPreview {
    /// Binds the pinned user root and the complete source declarations.
    pub scope_digest: String,
    /// Attached external sources; the user-root documents are always included.
    pub sources: Vec<SetupIndexSource>,
}

impl RagStore {
    /// Preview an external rebuild only when setup cannot preserve a complete index.
    /// No registered project or directory is opened by this method.
    ///
    /// # Errors
    /// Rejects dirty state, invalid local metadata, or concurrent declaration changes.
    pub fn preview_setup_index_rebuild(&self) -> Result<Option<SetupIndexPreview>, WikiError> {
        let _lock = self.acquire_authorization_lock()?;
        self.reject_dirty_mutation()?;
        if self.index_state_is_complete()? && self.load_registry_optional()?.is_some() {
            return Ok(None);
        }
        let (preview, _) = self.setup_rebuild_scope()?;
        Ok((!preview.sources.is_empty()).then_some(preview))
    }

    /// Rebuild only the source declarations displayed by the matching setup preview.
    /// A changed scope requires new approval. A valid index makes a retry a no-op.
    ///
    /// # Errors
    /// Rejects an obsolete digest, unavailable source, dirty state or failed publication.
    pub fn rebuild_setup_index(&self, approved_digest: &str) -> Result<StoreCommit, WikiError> {
        let _lock = self.acquire_authorization_lock()?;
        self.reject_dirty_mutation()?;
        let (preview, scope) = self.setup_rebuild_scope()?;
        if preview.scope_digest != approved_digest {
            return Err(WikiError::Conflict(
                "setup index sources changed after approval".into(),
            ));
        }
        if self.index_state_is_complete()? && self.load_registry_optional()?.is_some() {
            return self.current_commit();
        }
        self.commit_scoped_canonical_writes_locked(
            vec![(
                PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
                registry_bytes(&scope.registry)?,
            )],
            Some(&scope),
        )
    }

    fn setup_rebuild_scope(&self) -> Result<(SetupIndexPreview, UserSetupIndexScope), WikiError> {
        let declarations = [
            COLLECTION_REGISTRY_RELATIVE,
            crate::shared::PROJECT_REGISTRY_RELATIVE,
        ]
        .map(|path| crate::CapabilityFileSnapshot::capture(&self.root, Path::new(path)))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        let projects = crate::shared::load_project_registry_metadata(&self.root_path)?;
        let registry = self.synchronize_project_collection_metadata(
            &projects,
            self.load_registry_optional()?,
            false,
        )?;
        let identity = self
            .root
            .dir_metadata()
            .map_err(|error| WikiError::Io(error.to_string()))?;
        let projects_digest = match &declarations[1].original {
            crate::CapabilityFileState::Missing => None,
            crate::CapabilityFileState::File { bytes, .. } => Some(sha256_digest(bytes)),
        };
        // Use the normalized resulting registry so publishing that same registry does not
        // invalidate a retry. All other registration metadata remains exactly byte-bound.
        let binding = serde_json::to_vec(&(
            "setup-index-scope-v1",
            self.root_path.as_os_str().as_encoded_bytes(),
            CapMetadataExt::dev(&identity),
            CapMetadataExt::ino(&identity),
            &registry,
            projects_digest,
        ))
        .map_err(|error| WikiError::InvalidInput(error.to_string()))?;
        let sources = registry
            .collections
            .iter()
            .filter(|collection| {
                collection.state == CollectionState::Attached
                    && collection.kind != CollectionKind::UserRoot
            })
            .map(|collection| SetupIndexSource {
                collection_id: collection.collection_id.clone(),
                aliases: collection.aliases.clone(),
                path: collection
                    .local_locator
                    .clone()
                    .expect("validated attached locator"),
            })
            .collect();
        let scope = UserSetupIndexScope {
            registry,
            declarations,
            external_approved: true,
        };
        scope.verify_current(&self.root, &[])?;
        Ok((
            SetupIndexPreview {
                scope_digest: sha256_digest(&binding),
                sources,
            },
            scope,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::SHARED_INDEX_RELATIVE;
    use crate::shared::{
        KnowledgeLanguage, KnowledgeVisibility, RegisteredProject, PROJECT_REGISTRY_RELATIVE,
    };
    use crate::store::RAG_DIRTY_RELATIVE;

    fn registered() -> (tempfile::TempDir, tempfile::TempDir, RagStore) {
        let root = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let store = RagStore::open(root.path()).unwrap();
        store.ensure_registry().unwrap();
        store
            .register_project_atomic(RegisteredProject {
                id: "approved-project".into(),
                root: project.path().to_owned(),
                enabled: true,
                language: KnowledgeLanguage::En,
                visibility: KnowledgeVisibility::Shared,
            })
            .unwrap();
        std::fs::remove_file(root.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        (root, project, store)
    }

    #[test]
    fn setup_rebuild_preview_does_not_open_offline_projects_or_write_an_index() {
        let (root, project, store) = registered();
        let project_path = project.path().to_string_lossy().into_owned();
        project.close().unwrap();
        let before = std::fs::read(root.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap();
        let preview = store.preview_setup_index_rebuild().unwrap().unwrap();
        assert_eq!(preview.sources.len(), 1);
        assert!(
            preview.sources[0].path.ends_with(&project_path)
                || preview.sources[0].path == project_path
        );
        assert!(!root.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(store.rebuild_setup_index(&preview.scope_digest).is_err());
        assert_eq!(
            std::fs::read(root.path().join(COLLECTION_REGISTRY_RELATIVE)).unwrap(),
            before
        );
        assert!(!root.path().join(RAG_DIRTY_RELATIVE).exists());
    }

    #[test]
    fn setup_rebuild_approval_rejects_changes_and_publishes_an_idempotent_result() {
        let (root, _project, store) = registered();
        let preview = store.preview_setup_index_rebuild().unwrap().unwrap();
        assert!(store.rebuild_setup_index("sha256:wrong").is_err());
        let path = root.path().join(PROJECT_REGISTRY_RELATIVE);
        let original = std::fs::read(&path).unwrap();
        let mut changed = original.clone();
        changed.extend_from_slice(b"\n# external edit\n");
        std::fs::write(&path, &changed).unwrap();
        assert!(store.rebuild_setup_index(&preview.scope_digest).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), changed);
        assert!(!root.path().join(SHARED_INDEX_RELATIVE).exists());
        std::fs::write(&path, &original).unwrap();
        store.rebuild_setup_index(&preview.scope_digest).unwrap();
        store.validate_current().unwrap();
        let published = std::fs::read(root.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        assert!(store.preview_setup_index_rebuild().unwrap().is_none());
        assert!(store
            .rebuild_setup_index(&preview.scope_digest)
            .unwrap()
            .changed_paths
            .is_empty());
        assert_eq!(
            std::fs::read(root.path().join(SHARED_INDEX_RELATIVE)).unwrap(),
            published
        );
        assert_eq!(std::fs::read(path).unwrap(), original);
    }

    #[test]
    fn setup_rebuild_binds_the_user_root_and_rechecks_scope_before_publication() {
        let (_root, _project, store) = registered();
        let (other, _other_project, other_store) = registered();
        let preview = store.preview_setup_index_rebuild().unwrap().unwrap();
        assert!(other_store
            .rebuild_setup_index(&preview.scope_digest)
            .is_err());
        let (_, scope) = other_store.setup_rebuild_scope().unwrap();
        let registry_path = other.path().join(COLLECTION_REGISTRY_RELATIVE);
        let mut changed = other_store.load_registry().unwrap();
        changed.collections[0]
            .aliases
            .push("changed-before-write".into());
        let changed = registry_bytes(&changed.canonicalized().unwrap()).unwrap();
        std::fs::write(&registry_path, &changed).unwrap();
        assert!(other_store
            .commit_scoped_canonical_writes_locked(
                vec![(
                    PathBuf::from(COLLECTION_REGISTRY_RELATIVE),
                    registry_bytes(&scope.registry).unwrap()
                )],
                Some(&scope)
            )
            .is_err());
        assert_eq!(std::fs::read(registry_path).unwrap(), changed);
        assert!(!other.path().join(SHARED_INDEX_RELATIVE).exists());
    }
}
