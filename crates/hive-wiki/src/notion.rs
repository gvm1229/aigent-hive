//! Provider-neutral Notion canonical-source synchronization contracts.
//!
//! Hive never acquires OAuth credentials or calls a model provider here. A host
//! plugin, the official hosted MCP endpoint, or an explicitly consented REST
//! adapter supplies capability and complete-inventory receipts. This module
//! validates those receipts and builds the same disposable RAG projection used
//! by the Markdown backend.

use crate::collection::{
    derive_collection_id, CollectionKind, CollectionRecord, CollectionRegistry, CollectionState,
    CollectionVisibility, COLLECTION_SCHEMA_VERSION,
};
use crate::rag::{
    build_rag_index, canonical_wiki_category, document_digest, CanonicalDocument, RagIndexArtifact,
    RagLanguage, RagSnapshot, RagVisibility, RAG_SCHEMA_VERSION,
};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const MAX_ID_BYTES: usize = 500;
const MAX_REVISION_BYTES: usize = 500;
const MAX_PAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_PAGES: usize = 100_000;

/// Supported remote adapters in strict preference order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum NotionAdapter {
    /// Host-installed official Notion plugin or app.
    HostPlugin,
    /// Notion's official hosted MCP endpoint.
    HostedMcp,
    /// Direct Notion REST access explicitly selected by the user.
    Rest,
}

impl NotionAdapter {
    const fn priority(self) -> u8 {
        match self {
            Self::HostPlugin => 0,
            Self::HostedMcp => 1,
            Self::Rest => 2,
        }
    }
}

/// Exact operations required for a complete read/write Notion backend.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum RequiredCapability {
    /// Enumerate every page in the selected scope.
    Inventory,
    /// Read a stable remote revision token.
    Revision,
    /// Fetch complete page Markdown.
    Fetch,
    /// Create a page under the selected scope.
    Create,
    /// Update a page in the selected scope.
    Update,
}

impl RequiredCapability {
    /// Complete capability set. Search is intentionally excluded from normal RAG.
    pub const ALL: [Self; 5] = [
        Self::Inventory,
        Self::Revision,
        Self::Fetch,
        Self::Create,
        Self::Update,
    ];
}

/// Credential-free proof that one host-owned adapter can access one exact scope.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionCapabilityReceipt {
    /// Contract version.
    pub schema_version: u32,
    /// Adapter that produced the receipt.
    pub adapter: NotionAdapter,
    /// Stable workspace identity, never a token.
    pub workspace_id: String,
    /// User-selected page or data-source root.
    pub scope_id: String,
    /// Verified exact operations.
    pub capabilities: Vec<RequiredCapability>,
    /// Explicit permission for the last-resort REST adapter.
    #[serde(default)]
    pub rest_consent: bool,
}

/// One page in a deterministic complete inventory.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionInventoryEntry {
    /// Stable Notion page identity.
    pub page_id: String,
    /// Opaque revision token returned by the adapter.
    pub revision: String,
    /// Explicit remote deletion or loss of access.
    #[serde(default)]
    pub deleted: bool,
}

/// Complete normalized page content for a new or changed inventory entry.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionPage {
    pub page_id: String,
    pub revision: String,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub language: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    /// False when pagination or a provider error left content incomplete.
    pub complete: bool,
    /// True when the adapter reports Markdown truncation.
    pub truncated: bool,
    /// Unsupported blocks that would make the projection lossy.
    #[serde(default)]
    pub unknown_blocks: Vec<String>,
}

/// One bounded, complete-inventory freshness request supplied by a host adapter.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionSyncRequest {
    pub schema_version: u32,
    pub workspace_id: String,
    pub scope_id: String,
    /// Must be true before a generation can be published.
    pub inventory_complete: bool,
    /// Must be absent before a generation can be published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Complete selected-scope inventory, including explicit deletions.
    pub inventory: Vec<NotionInventoryEntry>,
    /// Content for new or revision-changed active pages only.
    #[serde(default)]
    pub pages: Vec<NotionPage>,
}

/// Minimal local revision ledger. It contains no prompt or transcript.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionRevisionEntry {
    pub page_id: String,
    pub revision: String,
    pub digest: String,
}

/// Complete derived Notion state retained beside the disposable SQLite bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionProjection {
    pub schema_version: u32,
    pub workspace_id: String,
    pub scope_id: String,
    pub adapter: NotionAdapter,
    pub registry: CollectionRegistry,
    pub revisions: Vec<NotionRevisionEntry>,
    /// Normalized content is derived remote data, not a local Markdown source tree.
    pub documents: Vec<CanonicalDocument>,
    pub artifact: RagIndexArtifact,
}

/// Successful fresh publication or an unchanged no-op.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotionSyncOutcome {
    pub projection: NotionProjection,
    pub changed_page_ids: Vec<String>,
    pub tombstoned_page_ids: Vec<String>,
    pub remote_requests: usize,
}

/// Stable fail-closed error for remote canonical-source receipts.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NotionError(String);

impl Display for NotionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for NotionError {}

/// Select the first fully capable adapter in plugin → MCP → consented REST order.
pub fn resolve_adapter(
    receipts: &[NotionCapabilityReceipt],
) -> Result<NotionCapabilityReceipt, NotionError> {
    let mut supported = Vec::new();
    let mut rest_without_consent = false;
    for receipt in receipts {
        validate_receipt(receipt)?;
        if receipt.adapter == NotionAdapter::Rest && !receipt.rest_consent {
            rest_without_consent = true;
            continue;
        }
        let actual = receipt
            .capabilities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if RequiredCapability::ALL
            .iter()
            .all(|item| actual.contains(item))
        {
            supported.push(receipt.clone());
        }
    }
    supported.sort_by_key(|receipt| receipt.adapter.priority());
    if let Some(receipt) = supported.into_iter().next() {
        return Ok(receipt);
    }
    if rest_without_consent {
        return Err(NotionError(
            "Notion REST fallback requires explicit consent".to_owned(),
        ));
    }
    Err(NotionError(
        "unsupported: no Notion adapter proves inventory, revision, fetch, create, and update"
            .to_owned(),
    ))
}

/// Validate a complete inventory and publish a backend-neutral SQLite generation.
pub fn sync_snapshot(
    previous: Option<&NotionProjection>,
    capability: &NotionCapabilityReceipt,
    request: &NotionSyncRequest,
) -> Result<NotionSyncOutcome, NotionError> {
    validate_receipt(capability)?;
    if resolve_adapter(std::slice::from_ref(capability)).is_err() {
        return Err(NotionError(
            "unsupported Notion capability receipt".to_owned(),
        ));
    }
    if request.schema_version != 1 || !request.inventory_complete || request.next_cursor.is_some() {
        return Err(NotionError(
            "Notion inventory is incomplete; fresh generation publication is forbidden".to_owned(),
        ));
    }
    validate_id("workspace_id", &request.workspace_id)?;
    validate_id("scope_id", &request.scope_id)?;
    if request.workspace_id != capability.workspace_id || request.scope_id != capability.scope_id {
        return Err(NotionError(
            "Notion sync scope differs from the capability receipt".to_owned(),
        ));
    }
    if let Some(previous) = previous {
        if previous.workspace_id != request.workspace_id || previous.scope_id != request.scope_id {
            return Err(NotionError(
                "Notion scope drift requires an explicit backend migration".to_owned(),
            ));
        }
    }
    if request.inventory.len() > MAX_PAGES || request.pages.len() > MAX_PAGES {
        return Err(NotionError(
            "Notion page inventory exceeds the bounded limit".to_owned(),
        ));
    }

    let prior_revisions = previous
        .map(|value| {
            value
                .revisions
                .iter()
                .map(|entry| (entry.page_id.as_str(), entry.revision.as_str()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let prior_documents = previous
        .map(|value| {
            value
                .documents
                .iter()
                .map(|document| (page_id_from_locator(&document.locator), document.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut inventory = BTreeMap::new();
    for entry in &request.inventory {
        validate_id("page_id", &entry.page_id)?;
        validate_revision(&entry.revision)?;
        if inventory.insert(entry.page_id.as_str(), entry).is_some() {
            return Err(NotionError("duplicate Notion inventory page_id".to_owned()));
        }
    }
    let mut fetched = BTreeMap::new();
    for page in &request.pages {
        validate_page(page)?;
        if fetched.insert(page.page_id.as_str(), page).is_some() {
            return Err(NotionError("duplicate fetched Notion page_id".to_owned()));
        }
    }

    let mut changed_page_ids = Vec::new();
    let mut documents = Vec::new();
    let mut revisions = Vec::new();
    for (page_id, entry) in &inventory {
        if entry.deleted {
            if fetched.contains_key(page_id) {
                return Err(NotionError(
                    "deleted Notion page must not include content".to_owned(),
                ));
            }
            continue;
        }
        let changed = prior_revisions.get(page_id).copied() != Some(entry.revision.as_str());
        let document = if changed {
            let page = fetched.get(page_id).ok_or_else(|| {
                NotionError(format!("changed page content is missing for `{page_id}`"))
            })?;
            if page.revision != entry.revision {
                return Err(NotionError(format!(
                    "fetched revision differs from inventory for `{page_id}`"
                )));
            }
            changed_page_ids.push((*page_id).to_owned());
            page_document(
                page,
                &request.workspace_id,
                &request.scope_id,
                next_generation(previous)?,
            )?
        } else {
            if fetched.contains_key(page_id) {
                return Err(NotionError(format!(
                    "unchanged page `{page_id}` was fetched; changed-only contract violated"
                )));
            }
            prior_documents.get(page_id).cloned().ok_or_else(|| {
                NotionError(format!(
                    "unchanged page `{page_id}` is absent from prior projection"
                ))
            })?
        };
        revisions.push(NotionRevisionEntry {
            page_id: (*page_id).to_owned(),
            revision: entry.revision.clone(),
            digest: document.digest.clone(),
        });
        documents.push(document);
    }
    let inventory_ids = inventory.keys().copied().collect::<BTreeSet<_>>();
    let mut tombstoned_page_ids = prior_revisions
        .keys()
        .copied()
        .filter(|page_id| !inventory_ids.contains(page_id))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    tombstoned_page_ids.extend(
        inventory
            .values()
            .filter(|entry| entry.deleted && prior_revisions.contains_key(entry.page_id.as_str()))
            .map(|entry| entry.page_id.clone()),
    );
    tombstoned_page_ids.sort();
    tombstoned_page_ids.dedup();
    changed_page_ids.sort();
    revisions.sort_by(|left, right| left.page_id.cmp(&right.page_id));
    documents.sort_by(|left, right| left.document_id.cmp(&right.document_id));

    if fetched
        .keys()
        .any(|page_id| !inventory.contains_key(page_id))
    {
        return Err(NotionError(
            "fetched Notion content lies outside the complete inventory".to_owned(),
        ));
    }
    if changed_page_ids.is_empty() && tombstoned_page_ids.is_empty() {
        if let Some(previous) = previous {
            return Ok(NotionSyncOutcome {
                projection: previous.clone(),
                changed_page_ids,
                tombstoned_page_ids,
                remote_requests: 1,
            });
        }
    }

    let registry = notion_registry(&request.workspace_id, &request.scope_id)?;
    let snapshot = RagSnapshot {
        schema_version: RAG_SCHEMA_VERSION,
        generation: next_generation(previous)?,
        registry: registry.clone(),
        documents: documents.clone(),
        claims: Vec::new(),
    };
    let artifact = build_rag_index(&snapshot)
        .map_err(|error| NotionError(format!("cannot build Notion RAG projection: {error}")))?;
    Ok(NotionSyncOutcome {
        projection: NotionProjection {
            schema_version: 1,
            workspace_id: request.workspace_id.clone(),
            scope_id: request.scope_id.clone(),
            adapter: capability.adapter,
            registry,
            revisions,
            documents,
            artifact,
        },
        remote_requests: 1 + request.pages.len(),
        changed_page_ids,
        tombstoned_page_ids,
    })
}

fn validate_receipt(receipt: &NotionCapabilityReceipt) -> Result<(), NotionError> {
    if receipt.schema_version != 1 {
        return Err(NotionError(
            "unsupported Notion capability receipt version".to_owned(),
        ));
    }
    validate_id("workspace_id", &receipt.workspace_id)?;
    validate_id("scope_id", &receipt.scope_id)?;
    let unique = receipt.capabilities.iter().collect::<BTreeSet<_>>();
    if unique.len() != receipt.capabilities.len() {
        return Err(NotionError("duplicate Notion capabilities".to_owned()));
    }
    if receipt.adapter != NotionAdapter::Rest && receipt.rest_consent {
        return Err(NotionError(
            "rest_consent is valid only for the REST fallback".to_owned(),
        ));
    }
    Ok(())
}

fn validate_page(page: &NotionPage) -> Result<(), NotionError> {
    validate_id("page_id", &page.page_id)?;
    validate_revision(&page.revision)?;
    if !page.complete || page.truncated || !page.unknown_blocks.is_empty() {
        return Err(NotionError(format!(
            "Notion page `{}` is incomplete; fresh publication is forbidden",
            page.page_id
        )));
    }
    if page.title.trim().is_empty()
        || page.body.trim().is_empty()
        || page.body.len() > MAX_PAGE_BYTES
    {
        return Err(NotionError(format!(
            "Notion page `{}` has invalid content",
            page.page_id
        )));
    }
    canonical_wiki_category(&page.kind)
        .map_err(|error| NotionError(format!("invalid Notion page kind: {error}")))?;
    parse_language(&page.language)?;
    Ok(())
}

fn validate_id(label: &str, value: &str) -> Result<(), NotionError> {
    if value.trim() != value
        || value.is_empty()
        || value.len() > MAX_ID_BYTES
        || value.chars().any(char::is_control)
        || value.contains(['/', '\\'])
        || value == "."
        || value == ".."
    {
        return Err(NotionError(format!("invalid Notion {label}")));
    }
    Ok(())
}

fn validate_revision(value: &str) -> Result<(), NotionError> {
    if value.trim() != value
        || value.is_empty()
        || value.len() > MAX_REVISION_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(NotionError("invalid Notion revision token".to_owned()));
    }
    Ok(())
}

fn parse_language(value: &str) -> Result<RagLanguage, NotionError> {
    match value {
        "en" => Ok(RagLanguage::En),
        "ko" => Ok(RagLanguage::Ko),
        "both" => Ok(RagLanguage::Both),
        "und" => Ok(RagLanguage::Und),
        _ => Err(NotionError("unsupported Notion page language".to_owned())),
    }
}

fn next_generation(previous: Option<&NotionProjection>) -> Result<u64, NotionError> {
    previous.map_or(Ok(1), |value| {
        value
            .artifact
            .manifest
            .generation
            .checked_add(1)
            .ok_or_else(|| NotionError("Notion generation counter is exhausted".to_owned()))
    })
}

fn notion_registry(workspace_id: &str, scope_id: &str) -> Result<CollectionRegistry, NotionError> {
    let collection_id = derive_collection_id("notion", &format!("{workspace_id}-{scope_id}"))
        .map_err(|error| NotionError(format!("invalid Notion collection identity: {error}")))?;
    CollectionRegistry {
        schema_version: COLLECTION_SCHEMA_VERSION,
        collections: vec![CollectionRecord {
            collection_id,
            kind: CollectionKind::Imported,
            state: CollectionState::Detached,
            aliases: vec!["notion".to_owned()],
            local_locator: None,
            source_project_id: None,
            default_visibility: CollectionVisibility::Shared,
        }],
    }
    .canonicalized()
    .map_err(|error| NotionError(format!("invalid Notion collection registry: {error}")))
}

fn page_document(
    page: &NotionPage,
    workspace_id: &str,
    scope_id: &str,
    generation: u64,
) -> Result<CanonicalDocument, NotionError> {
    let registry = notion_registry(workspace_id, scope_id)?;
    let collection_id = registry.collections[0].collection_id.clone();
    let locator = format!("notion/{workspace_id}/{scope_id}/{}.md", page.page_id);
    let raw_id =
        sha256_digest(format!("notion-page-v1\0{workspace_id}\0{}", page.page_id).as_bytes());
    let document_id = format!(
        "document-{}",
        raw_id.strip_prefix("sha256:").expect("Hive SHA-256 prefix")
    );
    let mut document = CanonicalDocument {
        document_id,
        collection_id,
        locator: locator.clone(),
        title: page.title.clone(),
        kind: page.kind.clone(),
        category: canonical_wiki_category(&page.kind)
            .map_err(|error| NotionError(error.to_string()))?
            .to_owned(),
        body: page.body.clone(),
        digest: String::new(),
        visibility: RagVisibility::Shared,
        language: parse_language(&page.language)?,
        revision: generation,
        tags: page.tags.clone(),
        aliases: page.aliases.clone(),
        links: Vec::new(),
        sources: if page.sources.is_empty() {
            vec![locator]
        } else {
            page.sources.clone()
        },
        replacement: None,
    };
    document.digest = document_digest(&document);
    Ok(document)
}

fn page_id_from_locator(locator: &str) -> &str {
    locator
        .rsplit('/')
        .next()
        .and_then(|value| value.strip_suffix(".md"))
        .unwrap_or(locator)
}
