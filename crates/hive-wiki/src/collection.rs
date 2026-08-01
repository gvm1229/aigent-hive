//! Stable logical collection identities and local attachment state.

use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::path::Path;

/// Version of the collection registry contract.
pub const COLLECTION_SCHEMA_VERSION: u32 = 1;
/// Reserved stable identifier for user-root knowledge.
pub const USER_ROOT_COLLECTION_ID: &str = "user-root";

const COLLECTION_ID_PREFIX: &str = "collection-";
const MAX_IDENTITY_BYTES: usize = 1024;
const MAX_ALIAS_BYTES: usize = 256;

/// Portable role of a logical collection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionKind {
    /// Knowledge owned by the current user rather than one project.
    UserRoot,
    /// A registered project collection.
    RegisteredProject,
    /// A portable directory-backed collection.
    Directory,
    /// A collection imported from another installation.
    Imported,
}

/// Whether this installation can currently reach the collection's canonical files.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionState {
    /// Canonical Markdown is available at the local locator.
    Attached,
    /// Metadata remains visible while canonical Markdown is unavailable locally.
    Detached,
}

/// Default visibility used when a canonical item does not narrow it further.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionVisibility {
    /// Visible across registered collections for the user.
    Shared,
    /// Visible only in the owning project scope.
    ProjectPrivate,
    /// Visible only after an explicit collection authorization.
    Confidential,
}

/// One logical collection and this installation's optional attachment.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollectionRecord {
    /// Stable logical identifier. It never includes an absolute path.
    pub collection_id: String,
    /// Portable role of the collection.
    pub kind: CollectionKind,
    /// Current attachment state.
    pub state: CollectionState,
    /// Human-facing aliases. Resolution is Unicode case-insensitive.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Installation-local absolute path, absent when detached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_locator: Option<String>,
    /// Legacy project-registry linkage. It is not the collection identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project_id: Option<String>,
    /// Default visibility for new canonical items.
    pub default_visibility: CollectionVisibility,
}

/// Canonical collection registry metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollectionRegistry {
    /// Registry contract version.
    pub schema_version: u32,
    /// Collections. Serialization order is canonical by identifier.
    pub collections: Vec<CollectionRecord>,
}

/// Exact result of resolving an identifier or alias.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "status", content = "collection_ids", rename_all = "kebab-case")]
pub enum CollectionResolution {
    /// Exactly one collection matched.
    Resolved(String),
    /// No collection matched.
    Unknown,
    /// More than one collection owns the folded alias.
    Ambiguous(Vec<String>),
}

/// Stable collection-contract error.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CollectionError(String);

impl CollectionError {
    fn invalid(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for CollectionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CollectionError {}

impl CollectionRegistry {
    /// Validate and canonicalize a registry without consulting the filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error for an unsupported schema, invalid collection metadata,
    /// duplicate identities, or multiple user-root records.
    pub fn canonicalized(&self) -> Result<Self, CollectionError> {
        if self.schema_version != COLLECTION_SCHEMA_VERSION {
            return Err(CollectionError::invalid(format!(
                "unsupported collection registry schema_version {}",
                self.schema_version
            )));
        }

        let mut canonical = self.clone();
        let mut ids = BTreeSet::new();
        let mut project_ids = BTreeSet::new();
        let mut user_root_count = 0_usize;
        for collection in &mut canonical.collections {
            validate_collection(collection)?;
            if !ids.insert(collection.collection_id.clone()) {
                return Err(CollectionError::invalid(format!(
                    "duplicate collection_id `{}`",
                    collection.collection_id
                )));
            }
            if let Some(project_id) = &collection.source_project_id {
                if !project_ids.insert(project_id.clone()) {
                    return Err(CollectionError::invalid(format!(
                        "duplicate source_project_id `{project_id}`"
                    )));
                }
            }
            if collection.kind == CollectionKind::UserRoot {
                user_root_count += 1;
            }

            collection.aliases.sort_by(|left, right| {
                folded_alias(left)
                    .cmp(&folded_alias(right))
                    .then_with(|| left.cmp(right))
            });
            collection
                .aliases
                .dedup_by(|left, right| folded_alias(left) == folded_alias(right));
        }
        if user_root_count > 1 {
            return Err(CollectionError::invalid(
                "the registry may contain at most one user-root collection",
            ));
        }
        canonical
            .collections
            .sort_by(|left, right| left.collection_id.cmp(&right.collection_id));
        Ok(canonical)
    }

    /// Resolve a collection identifier or Unicode-folded alias.
    #[must_use]
    pub fn resolve_collection(&self, reference: &str) -> CollectionResolution {
        if let Some(exact) = self
            .collections
            .iter()
            .find(|collection| collection.collection_id == reference)
        {
            return CollectionResolution::Resolved(exact.collection_id.clone());
        }

        let folded = folded_alias(reference);
        let mut matches = self
            .collections
            .iter()
            .filter(|collection| {
                collection
                    .aliases
                    .iter()
                    .any(|alias| folded_alias(alias) == folded)
            })
            .map(|collection| collection.collection_id.clone())
            .collect::<Vec<_>>();
        matches.sort();
        matches.dedup();
        match matches.as_slice() {
            [] => CollectionResolution::Unknown,
            [collection_id] => CollectionResolution::Resolved(collection_id.clone()),
            _ => CollectionResolution::Ambiguous(matches),
        }
    }

    /// Resolve the legacy project-registry linkage to a logical collection.
    #[must_use]
    pub fn resolve_project(&self, project_id: &str) -> CollectionResolution {
        let mut matches = self
            .collections
            .iter()
            .filter(|collection| collection.source_project_id.as_deref() == Some(project_id))
            .map(|collection| collection.collection_id.clone())
            .collect::<Vec<_>>();
        matches.sort();
        matches.dedup();
        match matches.as_slice() {
            [] => CollectionResolution::Unknown,
            [collection_id] => CollectionResolution::Resolved(collection_id.clone()),
            _ => CollectionResolution::Ambiguous(matches),
        }
    }

    /// Return collections indexed by their stable identifiers.
    #[must_use]
    pub fn by_id(&self) -> BTreeMap<&str, &CollectionRecord> {
        self.collections
            .iter()
            .map(|collection| (collection.collection_id.as_str(), collection))
            .collect()
    }
}

/// Derive a stable collection ID from a portable namespace and logical identity.
///
/// Absolute paths, path separators, dot-directory components, and drive-like
/// identities are rejected so moving a checkout cannot change its identity.
///
/// # Errors
///
/// Returns an error when either identity component is empty, oversized, or path-like.
pub fn derive_collection_id(
    namespace: &str,
    portable_identity: &str,
) -> Result<String, CollectionError> {
    validate_identity_component("namespace", namespace)?;
    validate_identity_component("portable_identity", portable_identity)?;
    let material = format!("collection-v1\0{namespace}\0{portable_identity}");
    let digest = sha256_digest(material.as_bytes());
    let digest = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| CollectionError::invalid("hive-core returned an invalid SHA-256 digest"))?;
    Ok(format!("{COLLECTION_ID_PREFIX}{digest}"))
}

/// Unicode lower-case fold used for deterministic alias lookup.
///
/// This intentionally does not depend on the host locale. Canonical registries
/// should store aliases in their normalized form when compatibility-equivalent
/// spellings need to resolve to the same collection.
#[must_use]
pub fn folded_alias(value: &str) -> String {
    value.trim().chars().flat_map(char::to_lowercase).collect()
}

fn validate_collection(collection: &CollectionRecord) -> Result<(), CollectionError> {
    if collection.kind == CollectionKind::UserRoot {
        if collection.collection_id != USER_ROOT_COLLECTION_ID {
            return Err(CollectionError::invalid(
                "a user-root collection must use collection_id `user-root`",
            ));
        }
        if collection.source_project_id.is_some() {
            return Err(CollectionError::invalid(
                "a user-root collection cannot have source_project_id",
            ));
        }
    } else {
        validate_derived_collection_id(&collection.collection_id)?;
    }
    if collection.kind == CollectionKind::RegisteredProject
        && collection
            .source_project_id
            .as_deref()
            .is_none_or(str::is_empty)
    {
        return Err(CollectionError::invalid(
            "a registered-project collection requires source_project_id",
        ));
    }
    match (collection.state, collection.local_locator.as_deref()) {
        (CollectionState::Attached, Some(locator)) if Path::new(locator).is_absolute() => {}
        (CollectionState::Attached, _) => {
            return Err(CollectionError::invalid(
                "an attached collection requires an absolute local_locator",
            ));
        }
        (CollectionState::Detached, None) => {}
        (CollectionState::Detached, Some(_)) => {
            return Err(CollectionError::invalid(
                "a detached collection cannot retain a local_locator",
            ));
        }
    }

    for alias in &collection.aliases {
        let trimmed = alias.trim();
        if trimmed.is_empty() || trimmed.len() > MAX_ALIAS_BYTES || has_control(trimmed) {
            return Err(CollectionError::invalid(format!(
                "invalid alias for collection `{}`",
                collection.collection_id
            )));
        }
    }
    if collection
        .source_project_id
        .as_deref()
        .is_some_and(|value| value.trim().is_empty() || has_control(value))
    {
        return Err(CollectionError::invalid(
            "source_project_id must be non-empty and contain no control characters",
        ));
    }
    Ok(())
}

fn validate_derived_collection_id(value: &str) -> Result<(), CollectionError> {
    let Some(digest) = value.strip_prefix(COLLECTION_ID_PREFIX) else {
        return Err(CollectionError::invalid(format!(
            "invalid collection_id `{value}`"
        )));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CollectionError::invalid(format!(
            "invalid collection_id `{value}`"
        )));
    }
    Ok(())
}

fn validate_identity_component(label: &str, value: &str) -> Result<(), CollectionError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() > MAX_IDENTITY_BYTES
        || trimmed != value
        || has_control(trimmed)
    {
        return Err(CollectionError::invalid(format!(
            "{label} must be bounded, trimmed, non-empty text"
        )));
    }
    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains(':')
        || trimmed == "."
        || trimmed == ".."
        || trimmed.split(['.', '-']).any(|part| part == "..")
    {
        return Err(CollectionError::invalid(format!(
            "{label} must be a portable logical identity, not a path"
        )));
    }
    Ok(())
}

fn has_control(value: &str) -> bool {
    value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(alias: &str, state: CollectionState) -> CollectionRecord {
        CollectionRecord {
            collection_id: derive_collection_id("project", alias).expect("portable identity"),
            kind: CollectionKind::RegisteredProject,
            state,
            aliases: vec![alias.to_owned()],
            local_locator: (state == CollectionState::Attached)
                .then(|| std::env::temp_dir().display().to_string()),
            source_project_id: Some(format!("legacy-{alias}")),
            default_visibility: CollectionVisibility::ProjectPrivate,
        }
    }

    #[test]
    fn collection_id_is_stable_and_rejects_paths() {
        let first = derive_collection_id("project", "acme-hive").expect("id");
        let second = derive_collection_id("project", "acme-hive").expect("id");
        assert_eq!(first, second);
        assert!(first.starts_with(COLLECTION_ID_PREFIX));
        assert!(derive_collection_id("project", "C:\\work\\hive").is_err());
        assert!(derive_collection_id("project", "/srv/hive").is_err());
    }

    #[test]
    fn canonicalization_sorts_and_deduplicates_aliases() {
        let mut record = project("Hive", CollectionState::Attached);
        record.aliases = vec!["하이브".to_owned(), "HIVE".to_owned(), "Hive".to_owned()];
        let registry = CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![record],
        }
        .canonicalized()
        .expect("canonical registry");
        assert_eq!(registry.collections[0].aliases, vec!["HIVE", "하이브"]);
        assert_eq!(
            registry.resolve_collection("hive"),
            CollectionResolution::Resolved(registry.collections[0].collection_id.clone())
        );
    }

    #[test]
    fn ambiguous_alias_fails_closed() {
        let mut first = project("one", CollectionState::Attached);
        let mut second = project("two", CollectionState::Attached);
        first.aliases = vec!["shared".to_owned()];
        second.aliases = vec!["SHARED".to_owned()];
        let registry = CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![second, first],
        }
        .canonicalized()
        .expect("canonical registry");
        let CollectionResolution::Ambiguous(ids) = registry.resolve_collection("Shared") else {
            panic!("expected ambiguity");
        };
        assert_eq!(ids.len(), 2);
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn detached_collection_retains_identity_without_locator() {
        let record = project("portable", CollectionState::Detached);
        let id = record.collection_id.clone();
        let registry = CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![record],
        }
        .canonicalized()
        .expect("detached collection");
        assert_eq!(
            registry.resolve_project("legacy-portable"),
            CollectionResolution::Resolved(id)
        );
        assert_eq!(registry.collections[0].local_locator, None);
    }

    #[test]
    fn attachment_invariants_are_enforced() {
        let mut attached = project("bad-attached", CollectionState::Attached);
        attached.local_locator = None;
        let registry = CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![attached],
        };
        assert!(registry.canonicalized().is_err());

        let mut detached = project("bad-detached", CollectionState::Detached);
        detached.local_locator = Some(std::env::temp_dir().display().to_string());
        let registry = CollectionRegistry {
            schema_version: COLLECTION_SCHEMA_VERSION,
            collections: vec![detached],
        };
        assert!(registry.canonicalized().is_err());
    }
}
