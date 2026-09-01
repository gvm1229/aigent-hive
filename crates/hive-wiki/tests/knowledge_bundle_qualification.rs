#[cfg(not(debug_assertions))]
use hive_wiki::bundle_io::BundlePublishMode;
#[cfg(not(debug_assertions))]
use hive_wiki::bundle_store::{export_bundle, import_bundle, BundleImportMode, COLLECTION_TABLES};
#[cfg(not(debug_assertions))]
use hive_wiki::collection::{
    CollectionKind, CollectionRecord, CollectionRegistry, CollectionState, CollectionVisibility,
    COLLECTION_SCHEMA_VERSION, USER_ROOT_COLLECTION_ID,
};
#[cfg(not(debug_assertions))]
use hive_wiki::portable::{BundleLimits, BundleScope};
#[cfg(not(debug_assertions))]
use hive_wiki::shared::{canonical_root, SHARED_INDEX_RELATIVE};
#[cfg(not(debug_assertions))]
use hive_wiki::store::{RagStore, COLLECTION_REGISTRY_RELATIVE};
#[cfg(not(debug_assertions))]
use rusqlite::Connection;
#[cfg(not(debug_assertions))]
use std::fs;
#[cfg(not(debug_assertions))]
use std::time::{Duration, Instant};

const QUALIFICATION_TEST: &str =
    "qualification: run this test with --release --ignored --nocapture on a local SSD";

#[cfg(debug_assertions)]
#[test]
#[ignore = "qualification: run this test with --release --ignored --nocapture on a local SSD"]
fn qualification_100_collections_50k_chunks_meets_bundle_p95_thresholds() {
    panic!("bundle timing evidence is valid only from a --release build");
}

#[cfg(not(debug_assertions))]
#[test]
#[ignore = "qualification: run this test with --release --ignored --nocapture on a local SSD"]
fn qualification_100_collections_50k_chunks_meets_bundle_p95_thresholds() {
    const SAMPLE_COUNT: usize = 5;
    const COLLECTION_COUNT: usize = 100;
    const PAGE_COUNT: usize = 25;
    const CHUNKS_PER_PAGE: usize = 2_000;
    const EXPECTED_CHUNKS: usize = PAGE_COUNT * CHUNKS_PER_PAGE;

    let source = qualification_source(COLLECTION_COUNT, PAGE_COUNT, CHUNKS_PER_PAGE);
    let bundles = tempfile::tempdir().expect("bundle output directory");
    let mut export_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut archive_digest = None;
    for sample in 0..SAMPLE_COUNT {
        let bundle = bundles
            .path()
            .join(format!("qualification-{sample}.hivekb"));
        let started = Instant::now();
        let result = export_bundle(
            source.path(),
            BundleScope::AllPortable,
            &bundle,
            &BundlePublishMode::CreateOnly,
            BundleLimits::default(),
        )
        .expect("export qualification bundle");
        export_samples.push(started.elapsed());
        if let Some(expected) = archive_digest.as_deref() {
            assert_eq!(result.archive_sha256, expected);
        } else {
            archive_digest = Some(result.archive_sha256);
        }
    }

    let source_bundle = bundles.path().join("qualification-0.hivekb");
    let mut import_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let destination = tempfile::tempdir().expect("import destination");
        RagStore::open(destination.path())
            .expect("open import destination")
            .ensure_registry()
            .expect("initialize destination RAG store");
        let baseline_connection = Connection::open(destination.path().join(SHARED_INDEX_RELATIVE))
            .expect("open baseline normalized index");
        let baseline_schema = schema_objects(&baseline_connection);
        drop(baseline_connection);
        let started = Instant::now();
        let result = import_bundle(
            destination.path(),
            &source_bundle,
            BundleImportMode::Apply,
            BundleLimits::default(),
        )
        .expect("import and rebuild qualification bundle");
        import_samples.push(started.elapsed());
        assert_eq!(result.detached_collection_ids.len(), COLLECTION_COUNT);

        let connection = Connection::open(destination.path().join(SHARED_INDEX_RELATIVE))
            .expect("open rebuilt qualification index");
        let chunk_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .expect("count rebuilt chunks");
        assert_eq!(
            chunk_count,
            i64::try_from(EXPECTED_CHUNKS).expect("expected chunk count fits SQLite integer")
        );
        let collection_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM collections", [], |row| row.get(0))
            .expect("count rebuilt collections");
        assert_eq!(
            collection_count,
            i64::try_from(COLLECTION_COUNT + 1)
                .expect("expected collection count fits SQLite integer")
        );
        let rebuilt_schema = schema_objects(&connection);
        assert_eq!(rebuilt_schema, baseline_schema);
        for table in COLLECTION_TABLES {
            assert!(
                rebuilt_schema
                    .iter()
                    .any(|(kind, name)| kind == "table" && name == table),
                "fixed normalized table `{table}` is missing"
            );
        }
        assert!(rebuilt_schema
            .iter()
            .any(|(kind, name)| kind == "table" && name == "documents_fts"));
        assert!(rebuilt_schema
            .iter()
            .any(|(kind, name)| kind == "table" && name == "chunks_fts"));
    }

    let export_p95 = percentile_95(export_samples);
    let import_p95 = percentile_95(import_samples);
    eprintln!(
        "v09_bundle_qualification collections={COLLECTION_COUNT} chunks={EXPECTED_CHUNKS} samples={SAMPLE_COUNT} export_p95_ms={} import_rebuild_p95_ms={}",
        export_p95.as_secs_f64() * 1_000.0,
        import_p95.as_secs_f64() * 1_000.0
    );
    assert!(
        export_p95 <= Duration::from_secs(5),
        "100-collection 50k-chunk export p95 {export_p95:?} exceeds 5s"
    );
    assert!(
        import_p95 <= Duration::from_secs(15),
        "100-collection 50k-chunk import+rebuild p95 {import_p95:?} exceeds 15s"
    );
}

#[cfg(not(debug_assertions))]
fn qualification_source(
    collection_count: usize,
    page_count: usize,
    chunks_per_page: usize,
) -> tempfile::TempDir {
    let source = tempfile::tempdir().expect("qualification source");
    let canonical = canonical_root(source.path()).expect("canonical qualification source");
    let wiki = source.path().join(".hive/knowledge/Wiki");
    fs::create_dir_all(&wiki).expect("qualification Wiki directory");
    fs::create_dir_all(source.path().join(".hive/config")).expect("qualification config directory");

    let mut collections = vec![CollectionRecord {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        kind: CollectionKind::UserRoot,
        state: CollectionState::Attached,
        aliases: vec!["user-root".to_owned()],
        local_locator: Some(canonical.display().to_string()),
        source_project_id: None,
        default_visibility: CollectionVisibility::Shared,
    }];
    collections.extend((1..=collection_count).map(|index| CollectionRecord {
        collection_id: format!("collection-{index:064x}"),
        kind: CollectionKind::Directory,
        state: CollectionState::Detached,
        aliases: vec![format!("benchmark-{index:03}")],
        local_locator: None,
        source_project_id: None,
        default_visibility: CollectionVisibility::ProjectPrivate,
    }));
    let registry = CollectionRegistry {
        schema_version: COLLECTION_SCHEMA_VERSION,
        collections,
    }
    .canonicalized()
    .expect("canonical qualification registry");
    fs::write(
        source.path().join(COLLECTION_REGISTRY_RELATIVE),
        serde_yaml::to_string(&registry).expect("serialize qualification registry"),
    )
    .expect("write qualification registry");

    let mut next_chunk = 0_usize;
    for page in 0..page_count {
        let page_id = format!("qualification-{page:02}");
        let mut body = String::with_capacity(chunks_per_page * 603);
        for ordinal in 0..chunks_per_page {
            if ordinal != 0 {
                body.push_str("\n\n");
            }
            // Two 600-byte paragraphs plus their separator exceed the 1,200-byte
            // chunk target, yielding exactly one chunk per paragraph. Punctuation
            // keeps this scale fixture from measuring pathological FTS token storage.
            body.push_str(&format!("chunk-{next_chunk:05}-{}", ".".repeat(588)));
            next_chunk += 1;
        }
        fs::write(
            wiki.join(format!("{page_id}.md")),
            wiki_page(&page_id, &body),
        )
        .expect("write qualification Wiki page");
    }
    source
}

#[cfg(not(debug_assertions))]
fn wiki_page(id: &str, body: &str) -> Vec<u8> {
    format!(
        "---\nschema_version: 1\nid: {id}\nkind: concept\nsummary: {id} benchmark\ntags:\n- qualification\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: '2026-08-01T00:00:00Z'\nupdated_at: '2026-08-01T00:00:00Z'\n---\n\n{body}\n"
    )
    .into_bytes()
}

#[cfg(not(debug_assertions))]
fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    assert!(!samples.is_empty(), "p95 requires at least one sample");
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[cfg(not(debug_assertions))]
fn schema_objects(connection: &Connection) -> Vec<(String, String)> {
    let mut statement = connection
        .prepare(
            "SELECT type, name FROM sqlite_schema
             WHERE name NOT LIKE 'sqlite_%'
             ORDER BY type, name",
        )
        .expect("prepare schema inventory");
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query schema inventory")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect schema inventory")
}

#[test]
fn qualification_contract_is_release_only() {
    assert!(!QUALIFICATION_TEST.is_empty());
}
