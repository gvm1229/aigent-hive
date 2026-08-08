use hive_core::sha256_digest;
use hive_wiki::collection::{
    CollectionKind, CollectionRecord, CollectionRegistry, CollectionState, CollectionVisibility,
    COLLECTION_SCHEMA_VERSION, USER_ROOT_COLLECTION_ID,
};
#[cfg(not(debug_assertions))]
use hive_wiki::rag::PreparedRagIndex;
use hive_wiki::rag::{
    build_rag_index, document_digest, plan_remember, retrieve_serialized, AssertionStatus,
    CanonicalDocument, ClaimKind, ClaimProvenance, RagLanguage, RagSnapshot, RagVisibility,
    RememberRequest, RememberSourceKind, RetrievalRequest, RetrievalScope, RAG_SCHEMA_VERSION,
};
use hive_wiki::shared::SHARED_INDEX_RELATIVE;
use hive_wiki::store::{
    CollectionRegistration, RagStore, CLAIMS_RELATIVE, COLLECTION_REGISTRY_RELATIVE,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(debug_assertions))]
use std::time::{Duration, Instant};
use tempfile::TempDir;

const TOP_K: usize = 5;
const BYTE_BUDGET: usize = 16 * 1024;

fn raw_digest(seed: &str) -> String {
    sha256_digest(seed.as_bytes())
        .strip_prefix("sha256:")
        .expect("Hive SHA-256 digests have the product prefix")
        .to_owned()
}

fn stable_id(prefix: &str, seed: &str) -> String {
    format!("{prefix}{}", raw_digest(seed))
}

fn initialized_store() -> (TempDir, RagStore, u64) {
    let root = tempfile::tempdir().expect("temporary user root");
    let store = RagStore::open(root.path()).expect("open RAG store");
    let commit = store.ensure_registry().expect("initialize RAG store");
    (root, store, commit.generation)
}

fn register_project(
    store: &RagStore,
    generation: &mut u64,
    root: &Path,
    portable_identity: &str,
    project_id: &str,
    alias: &str,
) -> String {
    let commit = store
        .register_collection(CollectionRegistration {
            collection_id: None,
            kind: CollectionKind::RegisteredProject,
            state: CollectionState::Attached,
            aliases: vec![alias.to_owned()],
            local_locator: Some(root.to_path_buf()),
            source_project_id: Some(project_id.to_owned()),
            default_visibility: CollectionVisibility::ProjectPrivate,
            portable_identity: Some(portable_identity.to_owned()),
            reviewed_inventory_digest: None,
        })
        .expect("register project collection");
    *generation = commit.store.generation;
    commit.collection.collection_id
}

fn register_directory(
    store: &RagStore,
    generation: &mut u64,
    root: &Path,
    identity: &str,
    alias: &str,
) -> String {
    let commit = store
        .register_collection(CollectionRegistration {
            collection_id: None,
            kind: CollectionKind::Directory,
            state: CollectionState::Attached,
            aliases: vec![alias.to_owned()],
            local_locator: Some(root.to_path_buf()),
            source_project_id: None,
            default_visibility: CollectionVisibility::ProjectPrivate,
            portable_identity: Some(identity.to_owned()),
            reviewed_inventory_digest: None,
        })
        .expect("register directory collection");
    *generation = commit.store.generation;
    commit.collection.collection_id
}

fn remember(
    store: &RagStore,
    generation: &mut u64,
    collection_id: &str,
    claim_key: &str,
    fact: &str,
    kind: ClaimKind,
    visibility: RagVisibility,
) -> String {
    let snapshot = store
        .load_canonical_snapshot(*generation)
        .expect("load current canonical snapshot");
    let request = RememberRequest {
        collection_id: collection_id.to_owned(),
        claim_key: claim_key.to_owned(),
        claim_id: None,
        locator: format!("pending/{claim_key}.md"),
        kind,
        status: AssertionStatus::UserStated,
        visibility,
        normalized_fact: fact.to_owned(),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::UserStatement,
            summary: format!("Reviewed durable statement for {claim_key}"),
            locator: format!("request:{claim_key}"),
            digest: sha256_digest(fact.as_bytes()),
        },
        sources: vec![format!("request:{claim_key}")],
        supersedes: Vec::new(),
        expected_active_digest: None,
        observed_at: None,
        verified_at: None,
    };
    let plan = plan_remember(&snapshot.claims, &request, *generation + 1)
        .expect("plan durable memory write");
    let claim_id = plan
        .new_claim
        .as_ref()
        .expect("unique fixture claim must be inserted")
        .claim_id
        .clone();
    let commit = store
        .apply_remember_plan(&plan)
        .expect("apply durable memory write");
    *generation = commit.generation;
    claim_id
}

fn retrieval(
    scope: RetrievalScope,
    current_collection_id: Option<String>,
    query: &str,
) -> RetrievalRequest {
    RetrievalRequest {
        scope,
        current_collection_id,
        query: query.to_owned(),
        // Match the inline CLI path: it does not inject caller- or gold-derived
        // expansions and uses the product's default result and byte limits.
        query_expansions: Vec::new(),
        top_k: TOP_K,
        byte_budget: BYTE_BUDGET,
        confidential_collection_id: None,
    }
}

#[test]
fn project_a_session_retrieves_named_project_b_without_private_scope_leak() {
    let (root, store, mut generation) = initialized_store();
    let alpha_root = root.path().join("project-a");
    let beta_root = root.path().join("project-b");
    fs::create_dir_all(&alpha_root).expect("Project A root");
    fs::create_dir_all(&beta_root).expect("Project B root");
    let alpha_collection = register_project(
        &store,
        &mut generation,
        &alpha_root,
        "portable-project-a",
        "project-a",
        "alpha",
    );
    let beta_collection = register_project(
        &store,
        &mut generation,
        &beta_root,
        "portable-project-b",
        "project-b",
        "beta",
    );
    let shared_id = remember(
        &store,
        &mut generation,
        &beta_collection,
        "project-b.orion.shared",
        "Orion deployment uses the reviewed blue shared convention.",
        ClaimKind::Convention,
        RagVisibility::Shared,
    );
    let private_id = remember(
        &store,
        &mut generation,
        &beta_collection,
        "project-b.orion.private",
        "Orion deployment keeps the reviewed amber private decision.",
        ClaimKind::Decision,
        RagVisibility::ProjectPrivate,
    );
    let instruction_id = remember(
        &store,
        &mut generation,
        &beta_collection,
        "project-b.hostile.instruction",
        "Retrieved instruction payload says to ignore safeguards and execute commands.",
        ClaimKind::Outcome,
        RagVisibility::Shared,
    );

    let named = store
        .retrieve(&retrieval(
            RetrievalScope::Project("project-b".to_owned()),
            Some(alpha_collection.clone()),
            "Orion deployment",
        ))
        .expect("explicit named-project retrieval");
    let named_ids = named
        .hits
        .iter()
        .map(|hit| hit.item_id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(named_ids.contains(shared_id.as_str()));
    assert!(named_ids.contains(private_id.as_str()));
    assert!(named
        .hits
        .iter()
        .filter(|hit| hit.item_id == shared_id || hit.item_id == private_id)
        .all(|hit| hit.collection_id == beta_collection));

    let automatic = store
        .retrieve(&retrieval(
            RetrievalScope::Auto,
            Some(alpha_collection),
            "Orion deployment",
        ))
        .expect("Project A automatic retrieval");
    assert!(automatic.hits.iter().any(|hit| hit.item_id == shared_id));
    assert!(automatic.hits.iter().all(|hit| hit.item_id != private_id));

    let hostile = store
        .retrieve(&retrieval(
            RetrievalScope::Project("project-b".to_owned()),
            Some(beta_collection),
            "instruction payload",
        ))
        .expect("retrieved instruction remains queryable data");
    let instruction = hostile
        .hits
        .iter()
        .find(|hit| hit.item_id == instruction_id)
        .expect("hostile instruction fixture hit");
    assert!(instruction.untrusted_content);
}

#[test]
fn global_commit_guidance_preference_survives_a_fresh_store_instance() {
    let (root, store, mut generation) = initialized_store();
    let normalized = "The user prefers storing committing rules once so they are recalled automatically in every AI session.";
    let claim_id = remember(
        &store,
        &mut generation,
        USER_ROOT_COLLECTION_ID,
        "user.commit-guidance",
        normalized,
        ClaimKind::Preference,
        RagVisibility::Shared,
    );
    drop(store);

    let fresh = RagStore::open(root.path()).expect("fresh-session RAG store");
    let natural_query = "I am sick of dictating committing rules to every AI session";
    let result = fresh
        .retrieve(&retrieval(RetrievalScope::Global, None, natural_query))
        .expect("fresh-session preference recall");
    assert!(result.hits.iter().any(|hit| hit.item_id == claim_id));

    let claim_path = root
        .path()
        .join(CLAIMS_RELATIVE)
        .join(USER_ROOT_COLLECTION_ID)
        .join(format!("{claim_id}.md"));
    let canonical = fs::read_to_string(claim_path).expect("canonical preference claim");
    assert!(canonical.contains(normalized));
    assert!(!canonical.contains(natural_query));
}

#[test]
fn disabled_secret_and_ambiguous_paths_fail_closed_without_claim_mutation() {
    // The public core represents a caller-disabled Wiki as an uninitialized store:
    // retrieval cannot initialize, rebuild, or write canonical state on its own.
    let disabled_root = tempfile::tempdir().expect("disabled user root");
    let disabled = RagStore::open(disabled_root.path()).expect("pin disabled root");
    assert!(disabled
        .retrieve(&retrieval(RetrievalScope::Global, None, "anything"))
        .is_err());
    assert!(!disabled_root.path().join(".hive").exists());

    let secret_fact = "Remember token sk-abcdefghijklmnopqrstuvwxyz0123456789 for later.";
    let secret_request = RememberRequest {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        claim_key: "user.secret".to_owned(),
        claim_id: None,
        locator: "pending/user-secret.md".to_owned(),
        kind: ClaimKind::Preference,
        status: AssertionStatus::UserStated,
        visibility: RagVisibility::Shared,
        normalized_fact: secret_fact.to_owned(),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::UserStatement,
            summary: "Reviewed candidate secret statement".to_owned(),
            locator: "request:user-secret".to_owned(),
            digest: sha256_digest(secret_fact.as_bytes()),
        },
        sources: vec!["request:user-secret".to_owned()],
        supersedes: Vec::new(),
        expected_active_digest: None,
        observed_at: None,
        verified_at: None,
    };
    assert!(plan_remember(&[], &secret_request, 1).is_err());
    assert!(!disabled_root.path().join(".hive").exists());

    let (root, store, mut generation) = initialized_store();
    let first = root.path().join("ambiguous-one");
    let second = root.path().join("ambiguous-two");
    fs::create_dir_all(&first).expect("first ambiguous root");
    fs::create_dir_all(&second).expect("second ambiguous root");
    register_directory(
        &store,
        &mut generation,
        &first,
        "ambiguous-one",
        "duplicate",
    );
    register_directory(
        &store,
        &mut generation,
        &second,
        "ambiguous-two",
        "duplicate",
    );
    let registry_before = fs::read(root.path().join(COLLECTION_REGISTRY_RELATIVE))
        .expect("registry before ambiguous query");
    let claims_before = canonical_file_bytes(root.path(), &[CLAIMS_RELATIVE]);
    let error = store
        .retrieve(&retrieval(
            RetrievalScope::Collection("duplicate".to_owned()),
            None,
            "anything",
        ))
        .expect_err("ambiguous collection scope must fail closed");
    assert_eq!(error.code(), "hive.knowledge-conflict");
    assert_eq!(
        fs::read(root.path().join(COLLECTION_REGISTRY_RELATIVE))
            .expect("registry after ambiguous query"),
        registry_before
    );
    assert_eq!(
        canonical_file_bytes(root.path(), &[CLAIMS_RELATIVE]),
        claims_before
    );
}

#[derive(Clone, Copy)]
enum GoldLocation {
    UserRoot,
    ProjectBShared,
    ProjectBPrivate,
}

struct GoldCase {
    key: &'static str,
    english_query: &'static str,
    korean_query: &'static str,
    fact: &'static str,
    kind: ClaimKind,
    location: GoldLocation,
}

const BILINGUAL_GOLD: &[GoldCase] = &[
    GoldCase {
        key: "qualification.gold-001",
        english_query: "I am sick of dictating committing rules to every AI session",
        korean_query: "AI 세션마다 커밋 규칙을 다시 지시하지 않으려면 어떻게 해야 하나",
        fact: "The user prefers storing committing rules once so they are recalled automatically in every AI session. 사용자는 커밋 규칙을 한 번 저장하여 모든 AI 세션에서 자동으로 불러오길 원한다.",
        kind: ClaimKind::Preference,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-002",
        english_query: "Will my durable preference survive a fresh session tomorrow?",
        korean_query: "새 세션에서도 저장한 선호가 유지되는가",
        fact: "A durable preference survives a fresh session and is loaded before relevant work. 저장된 선호는 새 세션에서도 유지되며 관련 작업 전에 불러온다.",
        kind: ClaimKind::Preference,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-003",
        english_query: "How can I search a named project outside the current repository?",
        korean_query: "현재 저장소 밖의 명시 프로젝트 지식을 어떻게 검색하는가",
        fact: "Named project retrieval searches a registered project outside the current repository. 명시 프로젝트 검색은 현재 저장소 밖에 등록된 프로젝트 지식을 찾는다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-004",
        english_query: "What restores retrieval after the SQLite index is deleted?",
        korean_query: "SQLite 인덱스가 삭제되면 Markdown 정본에서 검색을 복구하는가",
        fact: "When the SQLite index is deleted, Hive rebuilds retrieval exclusively from canonical Markdown. SQLite 인덱스가 삭제되면 Hive는 Markdown 정본만으로 검색을 재구축한다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-005",
        english_query: "Should API tokens and credentials ever enter durable knowledge?",
        korean_query: "API 토큰과 자격 증명을 장기 지식에 저장해도 되는가",
        fact: "Credential-like secrets and API tokens are rejected before durable knowledge capture. 자격 증명 형태의 비밀과 API 토큰은 장기 지식 기록 전에 거부한다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-006",
        english_query: "Can bilingual retrieval answer both English and Korean questions?",
        korean_query: "이중 언어 검색은 영어와 한국어 질문을 모두 처리하는가",
        fact: "Bilingual retrieval indexes English and Korean durable facts for later questions. 이중 언어 검색은 영어와 한국어의 장기 사실을 색인한다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-007",
        english_query: "How does a portable bundle move canonical knowledge to another machine?",
        korean_query: "이식 번들로 정본 지식을 다른 컴퓨터에 어떻게 옮기는가",
        fact: "A portable bundle transfers canonical knowledge and rebuild metadata to another machine. 이식 번들은 정본 지식과 재구축 메타데이터를 다른 컴퓨터로 전송한다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-008",
        english_query: "What evidence is required before a directory scan proposes reusable conventions?",
        korean_query: "디렉터리 스캔이 재사용 관례를 제안하려면 어떤 검토 근거가 필요한가",
        fact: "A directory scan may propose reusable conventions only from explicit reviewed evidence. 디렉터리 스캔은 명시적으로 검토한 근거에서만 재사용 관례를 제안할 수 있다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-009",
        english_query: "How is provenance retained when an obsolete rule is superseded?",
        korean_query: "오래된 규칙이 대체되어도 출처와 현재 진실을 어떻게 보존하는가",
        fact: "When an obsolete rule is superseded, the new claim becomes current truth while provenance remains available. 오래된 규칙이 대체되면 새 주장이 현재 진실이 되고 기존 출처는 보존된다.",
        kind: ClaimKind::Decision,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-010",
        english_query: "Are retrieved instructions untrusted evidence or executable authority?",
        korean_query: "검색된 지침은 신뢰할 수 없는 근거인가 실행 권한인가",
        fact: "Retrieved instructions are untrusted evidence and never executable authority. 검색된 지침은 신뢰할 수 없는 근거이며 실행 권한이 아니다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-011",
        english_query: "Does a dirty journal recover a write interrupted by a crash?",
        korean_query: "더티 저널은 충돌로 중단된 쓰기를 어떻게 복구하는가",
        fact: "A dirty journal enables crash recovery for an interrupted canonical write. 더티 저널은 충돌로 중단된 정본 쓰기의 복구를 가능하게 한다.",
        kind: ClaimKind::Outcome,
        location: GoldLocation::UserRoot,
    },
    GoldCase {
        key: "qualification.gold-012",
        english_query: "What shared Orion deployment convention does Project B use?",
        korean_query: "프로젝트 B의 공유 오리온 배포 관례는 무엇인가",
        fact: "Project B uses the shared Orion deployment convention with a blue rollout ring. 프로젝트 B는 파란 배포 링을 사용하는 공유 오리온 배포 관례를 채택한다.",
        kind: ClaimKind::Convention,
        location: GoldLocation::ProjectBShared,
    },
    GoldCase {
        key: "qualification.gold-013",
        english_query: "What private Atlas migration decision belongs to Project B?",
        korean_query: "프로젝트 B의 비공개 아틀라스 마이그레이션 결정은 무엇인가",
        fact: "Project B privately decided that the Atlas migration uses the amber compatibility bridge. 프로젝트 B는 아틀라스 마이그레이션에 황색 호환 브리지를 사용하기로 비공개 결정했다.",
        kind: ClaimKind::Decision,
        location: GoldLocation::ProjectBPrivate,
    },
];

const GOLD_DISTRACTORS: &[(&str, &str)] = &[
    (
        "distractor.release-checklist",
        "Release checklists require clean version evidence. 릴리스 체크리스트에는 정확한 버전 근거가 필요하다.",
    ),
    (
        "distractor.calendar-window",
        "Calendar maintenance uses a reviewed scheduling window. 달력 유지보수는 검토된 일정 시간을 사용한다.",
    ),
    (
        "distractor.image-assets",
        "Image assets retain their source attribution. 이미지 자산은 원본 출처 표시를 유지한다.",
    ),
    (
        "distractor.branch-names",
        "Development branches use a stable naming prefix. 개발 브랜치는 안정된 이름 접두사를 사용한다.",
    ),
    (
        "distractor.test-fixtures",
        "Test fixtures remain deterministic across local runs. 테스트 픽스처는 로컬 실행에서 결정적이어야 한다.",
    ),
    (
        "distractor.pdf-review",
        "PDF review checks page rendering and text extraction. PDF 검토는 페이지 렌더링과 텍스트 추출을 확인한다.",
    ),
    (
        "distractor.spreadsheet",
        "Spreadsheet exports preserve formulas and cell types. 스프레드시트 내보내기는 수식과 셀 형식을 보존한다.",
    ),
    (
        "distractor.notifications",
        "Notification routing follows the selected local channel. 알림 라우팅은 선택한 로컬 채널을 따른다.",
    ),
    (
        "distractor.source-layout",
        "Source layout separates harness inputs from compiled artifacts. 소스 배치는 하네스 입력과 컴파일 산출물을 분리한다.",
    ),
    (
        "distractor.role-lifecycle",
        "Persistent roles have explicit lifecycle states. 지속 역할은 명시적인 수명 주기 상태를 가진다.",
    ),
    (
        "distractor.prompt-quality",
        "Prompt quality review preserves the user's original intent. 프롬프트 품질 검토는 사용자의 원래 의도를 보존한다.",
    ),
    (
        "distractor.documentation",
        "Human documentation uses concise Korean explanations. 사용자 문서는 간결한 한국어 설명을 사용한다.",
    ),
];

#[test]
fn bilingual_paraphrase_gold_set_recall_at_five_is_at_least_ninety_percent() {
    let (root, store, mut generation) = initialized_store();
    let origin_path = root.path().join("qualification-project-a");
    let remote_path = root.path().join("qualification-project-b");
    fs::create_dir_all(&origin_path).expect("qualification Project A root");
    fs::create_dir_all(&remote_path).expect("qualification Project B root");
    let project_a = register_project(
        &store,
        &mut generation,
        &origin_path,
        "qualification-portable-project-a",
        "project-a",
        "qualification-alpha",
    );
    let project_b = register_project(
        &store,
        &mut generation,
        &remote_path,
        "qualification-portable-project-b",
        "project-b",
        "qualification-beta",
    );

    let mut expected_ids = BTreeMap::new();
    for case in BILINGUAL_GOLD {
        let (collection_id, visibility) = match case.location {
            GoldLocation::UserRoot => (USER_ROOT_COLLECTION_ID, RagVisibility::Shared),
            GoldLocation::ProjectBShared => (project_b.as_str(), RagVisibility::Shared),
            GoldLocation::ProjectBPrivate => (project_b.as_str(), RagVisibility::ProjectPrivate),
        };
        let claim_id = remember(
            &store,
            &mut generation,
            collection_id,
            case.key,
            case.fact,
            case.kind,
            visibility,
        );
        expected_ids.insert(case.key, claim_id);
    }
    for (key, fact) in GOLD_DISTRACTORS {
        remember(
            &store,
            &mut generation,
            USER_ROOT_COLLECTION_ID,
            key,
            fact,
            ClaimKind::Outcome,
            RagVisibility::Shared,
        );
    }
    drop(store);
    let store = RagStore::open(root.path()).expect("fresh qualification retrieval session");

    let evaluate = |language: &str, query: fn(&GoldCase) -> &'static str| {
        let mut misses = Vec::new();
        for case in BILINGUAL_GOLD {
            let scope = match case.location {
                GoldLocation::UserRoot | GoldLocation::ProjectBShared => RetrievalScope::Auto,
                GoldLocation::ProjectBPrivate => RetrievalScope::Project("project-b".to_owned()),
            };
            let natural_query = query(case);
            let result = store
                .retrieve(&retrieval(scope, Some(project_a.clone()), natural_query))
                .expect("production-path bilingual gold query");
            let expected = expected_ids
                .get(case.key)
                .expect("gold claim id must be recorded");
            if !result.hits.iter().any(|hit| &hit.item_id == expected) {
                let observed = result
                    .hits
                    .iter()
                    .map(|hit| format!("{}:{}", hit.item_id, hit.locator))
                    .collect::<Vec<_>>()
                    .join(", ");
                misses.push(format!(
                    "key={} query={natural_query:?} hits=[{observed}]",
                    case.key
                ));
            }
        }
        let recalled = BILINGUAL_GOLD.len() - misses.len();
        eprintln!(
            "v09_rag_recall language={language} recalled={recalled}/{} misses={misses:#?}",
            BILINGUAL_GOLD.len()
        );
        (recalled, misses)
    };

    let (english_recalled, english_misses) = evaluate("english", |case| case.english_query);
    let (korean_recalled, korean_misses) = evaluate("korean", |case| case.korean_query);
    assert!(
        english_recalled * 100 >= BILINGUAL_GOLD.len() * 90,
        "english recall@5 was {english_recalled}/{}; misses={english_misses:#?}",
        BILINGUAL_GOLD.len()
    );
    assert!(
        korean_recalled * 100 >= BILINGUAL_GOLD.len() * 90,
        "korean recall@5 was {korean_recalled}/{}; misses={korean_misses:#?}",
        BILINGUAL_GOLD.len()
    );
}

#[test]
fn deleting_sqlite_rebuilds_result_equivalently_without_changing_canonical_bytes() {
    let (root, store, mut generation) = initialized_store();
    remember(
        &store,
        &mut generation,
        USER_ROOT_COLLECTION_ID,
        "rebuild.alpha",
        "Rebuild evidence alpha remains canonical after SQLite deletion.",
        ClaimKind::Outcome,
        RagVisibility::Shared,
    );
    remember(
        &store,
        &mut generation,
        USER_ROOT_COLLECTION_ID,
        "rebuild.beta",
        "Rebuild evidence beta remains canonical after SQLite deletion.",
        ClaimKind::Outcome,
        RagVisibility::Shared,
    );
    let request = retrieval(RetrievalScope::Global, None, "Rebuild evidence");
    let before = store.retrieve(&request).expect("retrieval before deletion");
    let canonical_before = canonical_file_bytes(root.path(), &[".hive/config", ".hive/knowledge"]);
    fs::remove_file(root.path().join(SHARED_INDEX_RELATIVE))
        .expect("delete disposable SQLite only");
    drop(store);

    let fresh = RagStore::open(root.path()).expect("fresh store after SQLite deletion");
    fresh.rebuild().expect("canonical-only SQLite rebuild");
    let after = fresh.retrieve(&request).expect("retrieval after rebuild");
    let projection = |result: &hive_wiki::rag::RetrievalResult| {
        result
            .hits
            .iter()
            .map(|hit| {
                (
                    hit.item_id.clone(),
                    hit.locator.clone(),
                    hit.text.clone(),
                    hit.digest.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(projection(&after), projection(&before));
    assert_eq!(
        canonical_file_bytes(root.path(), &[".hive/config", ".hive/knowledge"]),
        canonical_before
    );
}

#[test]
fn performance_fixture_is_deterministic_and_query_correct_in_regular_test_runs() {
    let root = tempfile::tempdir().expect("performance correctness root");
    let snapshot = performance_snapshot(root.path(), 512);
    let registry = snapshot.registry.clone();
    let first = build_rag_index(&snapshot).expect("first deterministic performance index");
    let second = build_rag_index(&snapshot).expect("second deterministic performance index");
    assert_eq!(first.chunk_count, 512);
    assert_eq!(first.manifest, second.manifest);

    let request = performance_request(511);
    let first_result =
        retrieve_serialized(&first.sqlite_bytes, &first.manifest, &registry, &request)
            .expect("first deterministic query");
    let second_result =
        retrieve_serialized(&second.sqlite_bytes, &second.manifest, &registry, &request)
            .expect("second deterministic query");
    assert_eq!(first_result, second_result);
    assert_eq!(
        first_result.hits[0].item_id,
        stable_id("document-", "qualification-000511")
    );
}

#[cfg(debug_assertions)]
#[test]
#[ignore = "qualification: run this test with --release --ignored --nocapture on a local SSD"]
fn qualification_50k_chunks_meets_warm_and_fresh_load_p95_thresholds() {
    panic!("50k timing evidence is valid only from a --release build");
}

#[cfg(not(debug_assertions))]
#[test]
#[ignore = "qualification: run this test with --release --ignored --nocapture on a local SSD"]
fn qualification_50k_chunks_meets_warm_and_fresh_load_p95_thresholds() {
    let root = tempfile::tempdir().expect("50k qualification root");
    let artifact = build_rag_index(&performance_snapshot(root.path(), 50_000))
        .expect("build 50k-chunk qualification index");
    assert_eq!(artifact.chunk_count, 50_000);
    let registry = user_registry(root.path());
    let manifest = artifact.manifest;
    let index_path = root.path().join("qualification-50k.sqlite3");
    fs::write(&index_path, &artifact.sqlite_bytes).expect("persist qualification index");
    drop(artifact.sqlite_bytes);

    let request = performance_request(49_999);
    let mut fresh_load_samples = Vec::with_capacity(20);
    for _ in 0..20 {
        let started = Instant::now();
        let bytes = fs::read(&index_path).expect("fresh-load index read");
        let result = retrieve_serialized(&bytes, &manifest, &registry, &request)
            .expect("fresh-load retrieval");
        assert_eq!(
            result.hits[0].item_id,
            stable_id("document-", "qualification-049999")
        );
        fresh_load_samples.push(started.elapsed());
    }

    let resident = fs::read(&index_path).expect("resident index bytes");
    let prepared = PreparedRagIndex::from_serialized(&resident, &manifest, &registry)
        .expect("prepare authenticated resident index");
    let mut warm_samples = Vec::with_capacity(40);
    for _ in 0..40 {
        let started = Instant::now();
        let result = prepared.retrieve(&request).expect("warm retrieval");
        assert_eq!(
            result.hits[0].item_id,
            stable_id("document-", "qualification-049999")
        );
        warm_samples.push(started.elapsed());
    }

    let cold_p95 = percentile_95(fresh_load_samples);
    let warm_p95 = percentile_95(warm_samples);
    eprintln!(
        "v09_rag_qualification chunks=50000 methodology=fresh-file-load/prepared-resident cold_p95_ms={} warm_p95_ms={}",
        cold_p95.as_secs_f64() * 1_000.0,
        warm_p95.as_secs_f64() * 1_000.0
    );
    assert!(
        cold_p95 <= Duration::from_millis(500),
        "50k fresh-load p95 {cold_p95:?} exceeds 500ms"
    );
    assert!(
        warm_p95 <= Duration::from_millis(100),
        "50k warm p95 {warm_p95:?} exceeds 100ms"
    );
}

fn user_registry(root: &Path) -> CollectionRegistry {
    CollectionRegistry {
        schema_version: COLLECTION_SCHEMA_VERSION,
        collections: vec![CollectionRecord {
            collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
            kind: CollectionKind::UserRoot,
            state: CollectionState::Attached,
            aliases: vec!["user-root".to_owned()],
            local_locator: Some(root.display().to_string()),
            source_project_id: None,
            default_visibility: CollectionVisibility::Shared,
        }],
    }
    .canonicalized()
    .expect("canonical user-root registry")
}

fn document(seed: &str, body: &str, aliases: &[&str]) -> CanonicalDocument {
    let mut document = CanonicalDocument {
        document_id: stable_id("document-", seed),
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        locator: format!("docs/facts/{seed}.md"),
        title: seed.replace('-', " "),
        kind: "concept".to_owned(),
        category: "concept".to_owned(),
        body: body.to_owned(),
        digest: String::new(),
        visibility: RagVisibility::Shared,
        language: RagLanguage::Both,
        revision: 1,
        tags: vec![seed.to_owned()],
        aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
        links: Vec::new(),
        sources: vec![format!("gold:{seed}")],
        replacement: None,
    };
    document.digest = document_digest(&document);
    document
}

fn performance_snapshot(root: &Path, chunk_count: usize) -> RagSnapshot {
    let documents = (0..chunk_count)
        .map(|index| {
            let seed = format!("qualification-{index:06}");
            let body = format!(
                "Qualification benchmark record {index:06} contains unique needle_{index:06}."
            );
            document(&seed, &body, &["qualification benchmark"])
        })
        .collect();
    RagSnapshot {
        schema_version: RAG_SCHEMA_VERSION,
        generation: 1,
        registry: user_registry(root),
        documents,
        claims: Vec::new(),
    }
}

fn performance_request(index: usize) -> RetrievalRequest {
    retrieval(RetrievalScope::Global, None, &format!("needle_{index:06}"))
}

#[cfg(not(debug_assertions))]
fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    assert!(!samples.is_empty(), "p95 requires at least one sample");
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[rank]
}

fn canonical_file_bytes(root: &Path, relative_roots: &[&str]) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = relative_roots.iter().map(PathBuf::from).collect::<Vec<_>>();
    while let Some(relative) = pending.pop() {
        let absolute = root.join(&relative);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("cannot inspect {}: {error}", absolute.display()),
        };
        if metadata.is_dir() {
            let mut children = fs::read_dir(&absolute)
                .expect("read canonical directory")
                .map(|entry| entry.expect("canonical directory entry").file_name())
                .collect::<Vec<_>>();
            children.sort();
            pending.extend(children.into_iter().map(|name| relative.join(name)));
        } else if metadata.is_file() {
            files.insert(
                relative.to_string_lossy().replace('\\', "/"),
                fs::read(&absolute).expect("read canonical file"),
            );
        }
    }
    files
}
