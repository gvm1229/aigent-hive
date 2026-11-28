//! Remember command parsing and execution.

use super::{
    advise_subject, ensure_rag_registry, json, map_rag_error, optional, parse_options,
    plan_remember, plan_remember_batch, read_json_bounded, require_shared_wiki_enabled, required,
    sha256_digest, success, AssertionStatus, ClaimKind, ClaimProvenance, KnowledgeResult, Path,
    PathBuf, RagStore, RagVisibility, RememberRequest, RememberSourceKind, WikiError,
    SHARED_INDEX_RELATIVE, USER_ROOT_COLLECTION_ID,
};

/// Existing subjects offered with each recorded fact.
const SUBJECT_SUGGESTION_LIMIT: usize = 5;

pub(super) fn run_remember(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--user-root",
            "--request",
            "--user-statement",
            "--claim-key",
            "--kind",
            "--subject",
            "--batch",
        ],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    if let Some(batch) = optional(&options, "--batch") {
        for option in [
            "--request",
            "--user-statement",
            "--claim-key",
            "--kind",
            "--subject",
        ] {
            if optional(&options, option).is_some() {
                return Err(WikiError::InvalidInput(format!(
                    "--batch cannot be combined with {option}"
                )));
            }
        }
        return run_remember_batch(&user_root, Path::new(batch));
    }
    let request = match (
        optional(&options, "--request"),
        optional(&options, "--user-statement"),
    ) {
        (Some(request), None) => {
            for option in ["--claim-key", "--kind", "--subject"] {
                if optional(&options, option).is_some() {
                    return Err(WikiError::InvalidInput(format!(
                        "{option} requires --user-statement, not --request"
                    )));
                }
            }
            read_json_bounded::<RememberRequest>(Path::new(request), "remember request")?
        }
        (None, Some(statement)) => user_statement_remember_request(&options, statement)?,
        (Some(_), Some(_)) => {
            return Err(WikiError::InvalidInput(
                "remember accepts exactly one of --request or --user-statement".to_owned(),
            ));
        }
        (None, None) => {
            return Err(WikiError::InvalidInput(
                "remember requires --request, --user-statement, or --batch".to_owned(),
            ));
        }
    };
    require_shared_wiki_enabled(&user_root)?;

    // Independent inserts can finish shape and secret validation before initialization.
    // Explicit supersedes need the authenticated current claim set for validation.
    if request.supersedes.is_empty() {
        plan_remember(&[], &request, 1).map_err(map_rag_error)?;
    }
    let store = RagStore::open(&user_root)?;
    let initialized = ensure_rag_registry(&user_root, &store)?;
    let revision = initialized
        .generation
        .checked_add(1)
        .ok_or_else(|| WikiError::Conflict("RAG generation is exhausted".to_owned()))?;
    let snapshot = store.load_canonical_snapshot(revision)?;
    let plan = plan_remember(&snapshot.claims, &request, revision).map_err(map_rag_error)?;
    let subjects = advise_subject(
        &snapshot.claims,
        &request.collection_id,
        &request.normalized_fact,
        request.subject.as_deref(),
        SUBJECT_SUGGESTION_LIMIT,
    );
    let committed = store.apply_remember_plan(&plan)?;
    let graph_update = super::semantic_graph::after_capture(
        &user_root,
        false,
        &request.collection_id,
        match request.visibility {
            RagVisibility::Shared => "shared",
            RagVisibility::ProjectPrivate => "project-private",
            RagVisibility::Confidential => "confidential",
        },
        plan.disposition != hive_wiki::rag::RememberDisposition::Noop,
        &committed.manifest_digest,
    );
    let mut changed_paths = initialized.changed_paths;
    changed_paths.extend(committed.changed_paths.clone());
    Ok(success(
        "RememberKnowledge",
        "hive.knowledge-remembered",
        "agent-reviewed durable knowledge was written to canonical Markdown",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &committed.manifest_digest,
        json!({"plan": plan, "store": committed, "graph_update": graph_update, "subjects": subjects,
               "facts": [notice_fact(&request, plan.disposition)]}),
    ))
}

/// What the god notice lists for one remembered fact (OV170-034).
fn notice_fact(
    request: &RememberRequest,
    disposition: hive_wiki::rag::RememberDisposition,
) -> serde_json::Value {
    json!({
        "kind": request.kind,
        "subject": request.subject,
        "normalized_fact": request.normalized_fact,
        "visibility": request.visibility,
        "disposition": disposition,
    })
}

/// One user-root fact inside a `--batch` file.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchFact {
    normalized_fact: String,
    claim_key: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    subject: Option<String>,
}

/// Strict `--batch` file: several safe user-root facts from one turn.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RememberBatch {
    schema_version: u32,
    facts: Vec<BatchFact>,
}

fn run_remember_batch(user_root: &Path, path: &Path) -> Result<KnowledgeResult, WikiError> {
    let batch = read_json_bounded::<RememberBatch>(path, "remember batch")?;
    if batch.schema_version != 1 {
        return Err(WikiError::InvalidInput(
            "remember batch schema_version must be 1".to_owned(),
        ));
    }
    let requests = batch
        .facts
        .iter()
        .map(|fact| {
            user_statement_request(
                &fact.claim_key,
                fact.kind.as_deref(),
                fact.subject.as_deref(),
                &fact.normalized_fact,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    require_shared_wiki_enabled(user_root)?;
    // Shape, duplicate-key and secret validation finish before any initialization.
    plan_remember_batch(&[], &requests, 1).map_err(map_rag_error)?;
    let store = RagStore::open(user_root)?;
    let initialized = ensure_rag_registry(user_root, &store)?;
    let revision = initialized
        .generation
        .checked_add(1)
        .ok_or_else(|| WikiError::Conflict("RAG generation is exhausted".to_owned()))?;
    let snapshot = store.load_canonical_snapshot(revision)?;
    let plans =
        plan_remember_batch(&snapshot.claims, &requests, revision).map_err(map_rag_error)?;
    // Each fact is advised against the store plus the facts before it in this batch.
    let mut seen_claims = snapshot.claims.clone();
    let mut subjects = Vec::with_capacity(requests.len());
    for (request, plan) in requests.iter().zip(&plans) {
        subjects.push(advise_subject(
            &seen_claims,
            &request.collection_id,
            &request.normalized_fact,
            request.subject.as_deref(),
            SUBJECT_SUGGESTION_LIMIT,
        ));
        if let Some(claim) = &plan.new_claim {
            seen_claims.retain(|existing| existing.claim_id != claim.claim_id);
            seen_claims.push(claim.clone());
        }
    }
    let committed = store.apply_remember_plans(&plans)?;
    let changed = plans
        .iter()
        .any(|plan| plan.disposition != hive_wiki::rag::RememberDisposition::Noop);
    let graph_update = super::semantic_graph::after_capture(
        user_root,
        false,
        USER_ROOT_COLLECTION_ID,
        "shared",
        changed,
        &committed.manifest_digest,
    );
    let mut changed_paths = initialized.changed_paths;
    changed_paths.extend(committed.changed_paths.clone());
    Ok(success(
        "RememberKnowledge",
        "hive.knowledge-remembered",
        "agent-reviewed durable knowledge was written to canonical Markdown",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &committed.manifest_digest,
        json!({"plans": plans, "store": committed, "graph_update": graph_update, "subjects": subjects,
               "facts": requests.iter().zip(&plans)
                   .map(|(request, plan)| notice_fact(request, plan.disposition))
                   .collect::<Vec<_>>()}),
    ))
}

fn user_statement_remember_request(
    options: &[(&str, &str)],
    normalized_fact: &str,
) -> Result<RememberRequest, WikiError> {
    user_statement_request(
        required(options, "--claim-key")?,
        optional(options, "--kind"),
        optional(options, "--subject"),
        normalized_fact,
    )
}

fn user_statement_request(
    claim_key: &str,
    kind: Option<&str>,
    subject: Option<&str>,
    normalized_fact: &str,
) -> Result<RememberRequest, WikiError> {
    let kind = match kind.unwrap_or("preference") {
        "project-profile" => ClaimKind::ProjectProfile,
        "decision" => ClaimKind::Decision,
        "convention" => ClaimKind::Convention,
        "preference" => ClaimKind::Preference,
        "workflow" => ClaimKind::Workflow,
        value => {
            return Err(WikiError::InvalidInput(format!(
                "--kind must be project-profile, decision, convention, preference, or workflow; got {value}"
            )));
        }
    };
    let source = format!("request:{claim_key}");
    Ok(RememberRequest {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        claim_key: claim_key.to_owned(),
        claim_id: None,
        locator: format!("user-root/{claim_key}"),
        kind,
        status: AssertionStatus::UserStated,
        visibility: RagVisibility::Shared,
        normalized_fact: normalized_fact.to_owned(),
        subject: subject.map(str::to_owned),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::UserStatement,
            summary: "Bounded automatic user-statement capture".to_owned(),
            locator: source.clone(),
            digest: sha256_digest(normalized_fact.as_bytes()),
        },
        sources: vec![source],
        supersedes: Vec::new(),
        expected_active_digest: None,
        observed_at: None,
        verified_at: None,
    })
}
