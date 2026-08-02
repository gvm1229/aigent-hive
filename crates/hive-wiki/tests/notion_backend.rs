use hive_wiki::notion::{
    load_persisted_projection, resolve_adapter, retrieve_persisted, sync_and_publish,
    sync_snapshot, NotionAdapter, NotionCapabilityReceipt, NotionInventoryEntry, NotionPage,
    NotionSyncRequest, RequiredCapability, NOTION_LEDGER_RELATIVE,
};
use hive_wiki::rag::{retrieve_serialized, RetrievalRequest, RetrievalScope};
use hive_wiki::shared::SHARED_INDEX_RELATIVE;
use hive_wiki::store::RagStore;
use rusqlite::{Connection, MAIN_DB};
use std::io::Cursor;
use std::path::Path;

fn receipt(adapter: NotionAdapter, rest_consent: bool) -> NotionCapabilityReceipt {
    NotionCapabilityReceipt {
        schema_version: 1,
        adapter,
        workspace_id: "workspace-a".to_owned(),
        scope_id: "scope-a".to_owned(),
        capabilities: RequiredCapability::ALL.to_vec(),
        rest_consent,
    }
}

fn page(revision: &str, body: &str) -> NotionPage {
    NotionPage {
        page_id: "page-a".to_owned(),
        revision: revision.to_owned(),
        title: "Deployment guide".to_owned(),
        body: body.to_owned(),
        kind: "workflow".to_owned(),
        language: "en".to_owned(),
        tags: vec!["deployment".to_owned()],
        aliases: vec!["ship".to_owned()],
        sources: Vec::new(),
        complete: true,
        truncated: false,
        unknown_blocks: Vec::new(),
    }
}

fn page_with_id(page_id: &str, revision: &str, body: &str) -> NotionPage {
    let mut value = page(revision, body);
    value.page_id = page_id.to_owned();
    value.title = format!("Guide {page_id}");
    value
}

fn request(revision: &str, pages: Vec<NotionPage>) -> NotionSyncRequest {
    NotionSyncRequest {
        schema_version: 1,
        workspace_id: "workspace-a".to_owned(),
        scope_id: "scope-a".to_owned(),
        inventory_complete: true,
        next_cursor: None,
        inventory: vec![NotionInventoryEntry {
            page_id: "page-a".to_owned(),
            revision: revision.to_owned(),
            deleted: false,
        }],
        pages,
    }
}

fn request_pages(entries: &[(&str, &str)], pages: Vec<NotionPage>) -> NotionSyncRequest {
    NotionSyncRequest {
        schema_version: 1,
        workspace_id: "workspace-a".to_owned(),
        scope_id: "scope-a".to_owned(),
        inventory_complete: true,
        next_cursor: None,
        inventory: entries
            .iter()
            .map(|(page_id, revision)| NotionInventoryEntry {
                page_id: (*page_id).to_owned(),
                revision: (*revision).to_owned(),
                deleted: false,
            })
            .collect(),
        pages,
    }
}

fn chunk_rowids_for_page(sqlite_bytes: &[u8], page_id: &str) -> Vec<(String, i64)> {
    let mut connection = Connection::open_in_memory().expect("in-memory SQLite");
    connection
        .deserialize_read_exact(
            MAIN_DB,
            Cursor::new(sqlite_bytes),
            sqlite_bytes.len(),
            false,
        )
        .expect("deserialize RAG index");
    let mut statement = connection
        .prepare(
            "SELECT chunks.chunk_id, chunks.rowid
             FROM chunks
             JOIN documents ON documents.document_id = chunks.item_id
             WHERE chunks.item_kind = 'document' AND documents.locator LIKE ?1
             ORDER BY chunks.chunk_id",
        )
        .expect("prepare chunk row query");
    statement
        .query_map([format!("%/{page_id}.md")], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .expect("query chunks")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("read chunk rows")
}

#[test]
fn resolver_prefers_plugin_then_mcp_and_requires_rest_consent() {
    let selected = resolve_adapter(&[
        receipt(NotionAdapter::HostedMcp, false),
        receipt(NotionAdapter::HostPlugin, false),
    ])
    .expect("supported adapter");
    assert_eq!(selected.adapter, NotionAdapter::HostPlugin);

    let error = resolve_adapter(&[receipt(NotionAdapter::Rest, false)])
        .expect_err("REST must be explicitly consented");
    assert!(error
        .to_string()
        .contains("REST fallback requires explicit consent"));
}

#[test]
fn complete_inventory_fetches_only_changed_pages_and_tombstones_remote_deletes() {
    let capability = receipt(NotionAdapter::HostedMcp, false);
    let initial = sync_snapshot(
        None,
        &capability,
        &request("rev-1", vec![page("rev-1", "Alpha deployment procedure")]),
    )
    .expect("initial sync");
    assert_eq!(initial.changed_page_ids, ["page-a"]);
    assert_eq!(initial.projection.documents.len(), 1);

    let unchanged = sync_snapshot(
        Some(&initial.projection),
        &capability,
        &request("rev-1", Vec::new()),
    )
    .expect("unchanged freshness preflight");
    assert!(unchanged.changed_page_ids.is_empty());
    assert_eq!(
        unchanged.projection.artifact.manifest,
        initial.projection.artifact.manifest
    );

    let changed = sync_snapshot(
        Some(&unchanged.projection),
        &capability,
        &request("rev-2", vec![page("rev-2", "Beta deployment procedure")]),
    )
    .expect("changed-only fetch");
    assert_eq!(changed.changed_page_ids, ["page-a"]);
    let result = retrieve_serialized(
        &changed.projection.artifact.sqlite_bytes,
        &changed.projection.artifact.manifest,
        &changed.projection.registry,
        &RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "Beta".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        },
    )
    .expect("backend-neutral retrieval");
    assert_eq!(result.hits.len(), 1);
    assert!(result.hits[0].untrusted_content);

    let deleted = sync_snapshot(
        Some(&changed.projection),
        &capability,
        &NotionSyncRequest {
            inventory: Vec::new(),
            ..request("unused", Vec::new())
        },
    )
    .expect("complete inventory tombstones missing pages");
    assert_eq!(deleted.tombstoned_page_ids, ["page-a"]);
    assert!(deleted.projection.documents.is_empty());
}

#[test]
fn changed_page_reuses_unchanged_sqlite_chunk_rows() {
    let capability = receipt(NotionAdapter::HostedMcp, false);
    let initial = sync_snapshot(
        None,
        &capability,
        &request_pages(
            &[("page-a", "rev-1"), ("page-b", "rev-1")],
            vec![
                page_with_id("page-a", "rev-1", "Alpha deployment procedure"),
                page_with_id("page-b", "rev-1", "Stable rollback procedure"),
            ],
        ),
    )
    .expect("initial two-page sync");
    let untouched_before =
        chunk_rowids_for_page(&initial.projection.artifact.sqlite_bytes, "page-b");
    assert!(!untouched_before.is_empty());

    let changed = sync_snapshot(
        Some(&initial.projection),
        &capability,
        &request_pages(
            &[("page-a", "rev-2"), ("page-b", "rev-1")],
            vec![page_with_id("page-a", "rev-2", "Beta deployment procedure")],
        ),
    )
    .expect("changed-only sync");
    assert_eq!(changed.changed_page_ids, ["page-a"]);
    assert_eq!(changed.remote_requests, 2);
    assert_eq!(
        chunk_rowids_for_page(&changed.projection.artifact.sqlite_bytes, "page-b"),
        untouched_before
    );
}

#[test]
fn partial_or_unfetched_changed_content_never_publishes_a_fresh_generation() {
    let capability = receipt(NotionAdapter::HostPlugin, false);
    let mut partial = request("rev-1", vec![page("rev-1", "content")]);
    partial.inventory_complete = false;
    partial.next_cursor = Some("cursor".to_owned());
    assert!(sync_snapshot(None, &capability, &partial).is_err());

    let missing = request("rev-1", Vec::new());
    assert!(sync_snapshot(None, &capability, &missing)
        .expect_err("changed page content is mandatory")
        .to_string()
        .contains("changed page content is missing"));

    let mut truncated_page = page("rev-1", "content");
    truncated_page.truncated = true;
    assert!(
        sync_snapshot(None, &capability, &request("rev-1", vec![truncated_page]))
            .expect_err("truncated fetch must fail closed")
            .to_string()
            .contains("incomplete")
    );
}

#[test]
fn persisted_notion_mode_keeps_only_a_revision_ledger_and_recovers_from_sqlite_loss() {
    let root = tempfile::tempdir().expect("temporary user root");
    let store = RagStore::open(root.path()).expect("pin user root");
    let capability = receipt(NotionAdapter::HostPlugin, false);
    let first = sync_and_publish(
        &store,
        &capability,
        &request("rev-1", vec![page("rev-1", "Alpha deployment procedure")]),
        false,
    )
    .expect("initial remote generation");
    assert_eq!(first.store.generation, 1);
    assert!(!root.path().join(".hive/knowledge/Wiki").exists());
    let ledger = std::fs::read_to_string(root.path().join(NOTION_LEDGER_RELATIVE))
        .expect("body-free revision ledger");
    assert!(!ledger.contains("Alpha deployment procedure"));
    assert!(ledger.contains("page-a"));

    let recovered = load_persisted_projection(&store)
        .expect("load persisted projection")
        .expect("projection exists");
    assert_eq!(recovered.documents.len(), 1);
    assert_eq!(recovered.documents[0].body, "Alpha deployment procedure");
    let result = retrieve_persisted(
        &store,
        &RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "Alpha".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        },
    )
    .expect("backend-neutral persisted retrieval");
    assert_eq!(result.hits.len(), 1);

    let unchanged = sync_and_publish(&store, &capability, &request("rev-1", Vec::new()), false)
        .expect("bounded unchanged preflight");
    assert!(unchanged.sync.changed_page_ids.is_empty());
    assert!(unchanged.store.changed_paths.is_empty());

    std::fs::remove_file(root.path().join(SHARED_INDEX_RELATIVE)).expect("remove disposable index");
    assert!(sync_and_publish(&store, &capability, &request("rev-1", Vec::new()), false).is_err());
    let rebuilt = sync_and_publish(
        &store,
        &capability,
        &request("rev-1", vec![page("rev-1", "Alpha deployment procedure")]),
        true,
    )
    .expect("remote full rebuild after SQLite loss");
    assert_eq!(rebuilt.store.generation, 2);
    assert!(root.path().join(Path::new(SHARED_INDEX_RELATIVE)).is_file());
}

#[test]
fn complete_remote_rebuild_recovers_only_its_own_dirty_ledger() {
    let root = tempfile::tempdir().expect("temporary user root");
    let store = RagStore::open(root.path()).expect("pin user root");
    let capability = receipt(NotionAdapter::HostPlugin, false);
    sync_and_publish(
        &store,
        &capability,
        &request("rev-1", vec![page("rev-1", "Alpha deployment procedure")]),
        false,
    )
    .expect("initial remote generation");
    store
        .begin_external_canonical_mutation(&[(
            Path::new(NOTION_LEDGER_RELATIVE).to_path_buf(),
            b"remote canonical write awaiting local projection".to_vec(),
        )])
        .expect("persist remote dirty state");
    assert!(retrieve_persisted(
        &store,
        &RetrievalRequest {
            scope: RetrievalScope::Global,
            current_collection_id: None,
            query: "Alpha".to_owned(),
            query_expansions: Vec::new(),
            top_k: 5,
            byte_budget: 4096,
            confidential_collection_id: None,
        }
    )
    .is_err());

    let recovered = sync_and_publish(
        &store,
        &capability,
        &request("rev-2", vec![page("rev-2", "Beta deployment procedure")]),
        true,
    )
    .expect("complete remote inventory recovers the selected ledger");
    assert_eq!(recovered.store.generation, 2);
    assert!(!store.is_dirty().expect("inspect recovery state"));

    store
        .begin_external_canonical_mutation(&[(
            Path::new(".hive/knowledge/Wiki/foreign.md").to_path_buf(),
            b"unrelated canonical write".to_vec(),
        )])
        .expect("persist unrelated dirty state");
    assert!(sync_and_publish(
        &store,
        &capability,
        &request("rev-3", vec![page("rev-3", "Gamma deployment procedure")]),
        true,
    )
    .is_err());
}
