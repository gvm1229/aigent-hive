//! Retrieve command parsing and execution.

use super::*;

#[allow(clippy::too_many_lines)]
pub(super) fn run_retrieve(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--user-root",
            "--target",
            "--request",
            "--scope",
            "--query",
            "--top-k",
            "--byte-budget",
            "--mode",
            "--authorization-id",
            "--authorization-token",
            "--capabilities",
            "--usage",
        ],
    )?;
    let mode = optional(&options, "--mode").unwrap_or("fts");
    if !["fts", "semantic"].contains(&mode) {
        return Err(WikiError::InvalidInput(
            "--mode must be fts or semantic".to_owned(),
        ));
    }
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let target = PathBuf::from(required(&options, "--target")?);
    let mut request = parse_retrieval_request(&options, RetrievalScope::Auto)?;
    require_shared_wiki_enabled(&user_root)?;
    let store = RagStore::open(&user_root)?;
    let registry = store.load_registry()?;
    let (canonical_target, current_collection_id) =
        derive_optional_current_collection_authority(&registry, &target)?;
    request
        .current_collection_id
        .clone_from(&current_collection_id);
    let authorization_fields = [
        "--authorization-id",
        "--authorization-token",
        "--capabilities",
        "--usage",
    ];
    let authorization_count = authorization_fields
        .iter()
        .filter(|option| optional(&options, option).is_some())
        .count();
    if authorization_count != 0 && authorization_count != authorization_fields.len() {
        return Err(WikiError::InvalidInput(
            "confidential retrieval requires authorization ID, token, capability snapshot, and usage snapshot together"
                .to_owned(),
        ));
    }
    let mut changed_paths = Vec::new();
    if authorization_count == authorization_fields.len() {
        let current_collection_id = current_collection_id.as_deref().ok_or_else(|| {
            WikiError::InvalidInput(
                "confidential retrieval requires an attached current project collection".to_owned(),
            )
        })?;
        let consumption = verify_and_consume_authorization(
            &user_root,
            &store,
            &registry,
            &request,
            &canonical_target,
            current_collection_id,
            required(&options, "--authorization-id")?,
            required(&options, "--authorization-token")?,
            Path::new(required(&options, "--capabilities")?),
            Path::new(required(&options, "--usage")?),
        )?;
        request.confidential_collection_id = Some(consumption.collection_id);
        changed_paths.push(consumption.changed_path);
    }
    let result = if mode == "semantic" {
        vector::retrieve(&user_root, &store, &request)?
    } else {
        serde_json::to_value(store.retrieve(&request)?)
            .map_err(|error| WikiError::Io(error.to_string()))?
    };
    let digest = result["manifest_digest"]
        .as_str()
        .ok_or_else(|| WikiError::Verification("retrieval manifest is absent".to_owned()))?
        .to_owned();
    Ok(success(
        "RetrieveKnowledge",
        "hive.knowledge-retrieved",
        "bounded knowledge retrieval completed",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &digest,
        result,
    ))
}

pub(super) fn parse_retrieval_request(
    options: &[(&str, &str)],
    default_scope: RetrievalScope,
) -> Result<RetrievalRequest, WikiError> {
    let request_path = optional(options, "--request");
    let inline_present = ["--scope", "--query", "--top-k", "--byte-budget"]
        .iter()
        .any(|option| optional(options, option).is_some());
    if request_path.is_some() && inline_present {
        return Err(WikiError::InvalidInput(
            "--request cannot be combined with inline retrieval options".to_owned(),
        ));
    }
    let mut request = if let Some(path) = request_path {
        read_json_bounded::<RetrievalRequest>(Path::new(path), "retrieval request")?
    } else {
        RetrievalRequest {
            scope: optional(options, "--scope").map_or(Ok(default_scope), |value| {
                RetrievalScope::from_str(value).map_err(map_rag_error)
            })?,
            current_collection_id: None,
            query: required(options, "--query")?.to_owned(),
            query_expansions: Vec::new(),
            top_k: parse_bounded_usize(optional(options, "--top-k"), 5, 1, 100, "--top-k")?,
            byte_budget: parse_bounded_usize(
                optional(options, "--byte-budget"),
                16 * 1024,
                1,
                1024 * 1024,
                "--byte-budget",
            )?,
            confidential_collection_id: None,
        }
    };
    if request.current_collection_id.is_some() || request.confidential_collection_id.is_some() {
        return Err(WikiError::InvalidInput(
            "retrieval request files cannot self-assert current or confidential collection authority"
                .to_owned(),
        ));
    }
    request.query = normalize_retrieval_text(&request.query, "query")?;
    let mut seen = BTreeSet::new();
    let mut expansions = Vec::new();
    for expansion in &request.query_expansions {
        let normalized = normalize_retrieval_text(expansion, "query expansion")?;
        if seen.insert(normalized.clone()) {
            expansions.push(normalized);
        }
    }
    if expansions.len() > 8 {
        return Err(WikiError::InvalidInput(
            "query_expansions exceeds 8 normalized entries".to_owned(),
        ));
    }
    if request.top_k == 0 || request.top_k > 100 {
        return Err(WikiError::InvalidInput(
            "top_k must be between 1 and 100".to_owned(),
        ));
    }
    if request.byte_budget == 0 || request.byte_budget > 1024 * 1024 {
        return Err(WikiError::InvalidInput(
            "byte_budget must be between 1 and 1048576".to_owned(),
        ));
    }
    request.query_expansions = expansions;
    Ok(request)
}

pub(super) fn normalize_retrieval_text(value: &str, label: &str) -> Result<String, WikiError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() || normalized.len() > 4096 {
        return Err(WikiError::InvalidInput(format!(
            "{label} must contain 1 through 4096 UTF-8 bytes"
        )));
    }
    Ok(normalized)
}

pub(super) fn resolve_retrieval_collection(
    registry: &hive_wiki::collection::CollectionRegistry,
    scope: &RetrievalScope,
    current_collection_id: &str,
) -> Result<String, WikiError> {
    match scope {
        RetrievalScope::Auto => Ok(current_collection_id.to_owned()),
        RetrievalScope::Project(project_id) => match registry.resolve_project(project_id) {
            CollectionResolution::Resolved(collection_id) => Ok(collection_id),
            CollectionResolution::Unknown => {
                Err(WikiError::InvalidInput("unknown project scope".to_owned()))
            }
            CollectionResolution::Ambiguous(ids) => Err(WikiError::Conflict(format!(
                "ambiguous project scope resolves to {}",
                ids.join(", ")
            ))),
        },
        RetrievalScope::Collection(reference) => {
            resolve_collection_reference(registry, reference, "collection scope")
        }
        RetrievalScope::Global | RetrievalScope::AllVisible => {
            Ok(USER_ROOT_COLLECTION_ID.to_owned())
        }
    }
}
