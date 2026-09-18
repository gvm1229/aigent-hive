//! Remember command parsing and execution.

use super::*;

pub(super) fn run_remember(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--user-root",
            "--request",
            "--user-statement",
            "--claim-key",
            "--kind",
        ],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let request = match (
        optional(&options, "--request"),
        optional(&options, "--user-statement"),
    ) {
        (Some(request), None) => {
            for option in ["--claim-key", "--kind"] {
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
                "remember requires --request or --user-statement".to_owned(),
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
    let committed = store.apply_remember_plan(&plan)?;
    let mut changed_paths = initialized.changed_paths;
    changed_paths.extend(committed.changed_paths.clone());
    Ok(success(
        "RememberKnowledge",
        "hive.knowledge-remembered",
        "agent-reviewed durable knowledge was written to canonical Markdown",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &committed.manifest_digest,
        json!({"plan": plan, "store": committed}),
    ))
}

fn user_statement_remember_request(
    options: &[(&str, &str)],
    normalized_fact: &str,
) -> Result<RememberRequest, WikiError> {
    let claim_key = required(options, "--claim-key")?;
    let kind = match optional(options, "--kind").unwrap_or("preference") {
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
