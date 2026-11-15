//! Reviewed subject assignment for existing knowledge: preview, apply and restore.
//!
//! Hive proposes subjects with deterministic rules only. The active host reviews the
//! proposals (model work stays with the host) and returns an explicit review file.
//! Apply writes one canonical commit plus a restore record; restore returns the exact
//! original claim bytes.

use super::{
    ensure_rag_registry, json, optional, parse_options, read_json_bounded,
    require_shared_wiki_enabled, required, sha256_digest, success, KnowledgeResult, Path, PathBuf,
    RagStore, WikiError, SHARED_INDEX_RELATIVE, USER_ROOT_COLLECTION_ID,
};
use hive_wiki::rag::{
    claim_digest, propose_subject, validate_subject, AssertionStatus, CanonicalClaim,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

const RESTORE_RELATIVE: &str = ".hive/backups/knowledge-subjects";
const MAX_ASSIGNMENTS: usize = 5000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectProposal {
    claim_id: String,
    claim_key: String,
    expected_digest: String,
    normalized_fact: String,
    subject: String,
    rule: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectReview {
    schema_version: u32,
    preview_digest: String,
    assignments: Vec<ReviewedAssignment>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedAssignment {
    claim_id: String,
    expected_digest: String,
    subject: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoreRecord {
    schema_version: u32,
    operation_id: String,
    collection_id: String,
    originals: Vec<CanonicalClaim>,
    applied: Vec<CanonicalClaim>,
}

pub(super) fn run_subjects(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    match arguments.first().map(String::as_str) {
        Some("preview") => preview(&arguments[1..]),
        Some("apply") => apply(&arguments[1..]),
        Some("restore") => restore(&arguments[1..]),
        _ => Err(WikiError::InvalidInput(
            "knowledge subjects requires preview, apply, or restore".to_owned(),
        )),
    }
}

fn collection(options: &[(&str, &str)]) -> Result<String, WikiError> {
    let collection = optional(options, "--collection").unwrap_or(USER_ROOT_COLLECTION_ID);
    if collection != USER_ROOT_COLLECTION_ID {
        return Err(WikiError::InvalidInput(
            "knowledge subjects currently supports only the user-root collection".to_owned(),
        ));
    }
    Ok(collection.to_owned())
}

fn open_store(user_root: &Path) -> Result<(RagStore, Vec<CanonicalClaim>, String), WikiError> {
    require_shared_wiki_enabled(user_root)?;
    let store = RagStore::open(user_root)?;
    let initialized = ensure_rag_registry(user_root, &store)?;
    let snapshot = store.load_canonical_snapshot(initialized.generation.saturating_add(1))?;
    Ok((store, snapshot.claims, initialized.manifest_digest))
}

fn proposals(claims: &[CanonicalClaim], collection_id: &str) -> Vec<SubjectProposal> {
    let mut proposals = claims
        .iter()
        .filter(|claim| {
            claim.collection_id == collection_id
                && claim.status != AssertionStatus::Superseded
                && claim.subject.is_none()
        })
        .map(|claim| {
            let (subject, rule) = propose_subject(claim);
            SubjectProposal {
                claim_id: claim.claim_id.clone(),
                claim_key: claim.claim_key.clone(),
                expected_digest: claim.digest.clone(),
                normalized_fact: claim.normalized_fact.clone(),
                subject,
                rule: rule.to_owned(),
            }
        })
        .collect::<Vec<_>>();
    proposals.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    proposals
}

fn preview_digest(proposals: &[SubjectProposal]) -> Result<String, WikiError> {
    let identity = proposals
        .iter()
        .map(|proposal| (&proposal.claim_id, &proposal.expected_digest))
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&("subject-preview-v1", identity))
        .map_err(|error| WikiError::Io(error.to_string()))?;
    Ok(sha256_digest(&bytes))
}

fn preview(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--user-root", "--collection"])?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let collection_id = collection(&options)?;
    let (_store, claims, manifest_digest) = open_store(&user_root)?;
    let proposals = proposals(&claims, &collection_id);
    let digest = preview_digest(&proposals)?;
    Ok(success(
        "PreviewKnowledgeSubjects",
        "hive.knowledge-subjects-previewed",
        "rule-based subject proposals are ready for host review; nothing was written",
        Vec::new(),
        SHARED_INDEX_RELATIVE,
        &manifest_digest,
        json!({
            "collection_id": collection_id,
            "preview_digest": digest,
            "proposals": proposals,
            "review_format": {"schema_version": 1, "preview_digest": digest, "assignments": [{"claim_id": "<claim_id>", "expected_digest": "<expected_digest>", "subject": "<area/topic>"}]},
        }),
    ))
}

fn apply(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--user-root",
            "--collection",
            "--review",
            "--confirm-subject-assignment",
        ],
    )?;
    if optional(&options, "--confirm-subject-assignment") != Some("yes") {
        return Err(WikiError::InvalidInput(
            "subject assignment requires --confirm-subject-assignment yes after host review"
                .to_owned(),
        ));
    }
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let collection_id = collection(&options)?;
    let review = read_json_bounded::<SubjectReview>(
        Path::new(required(&options, "--review")?),
        "subject review",
    )?;
    if review.schema_version != 1 {
        return Err(WikiError::InvalidInput(
            "subject review schema_version must be 1".to_owned(),
        ));
    }
    if review.assignments.is_empty() || review.assignments.len() > MAX_ASSIGNMENTS {
        return Err(WikiError::InvalidInput(format!(
            "subject review requires 1 through {MAX_ASSIGNMENTS} assignments"
        )));
    }
    let (store, claims, _) = open_store(&user_root)?;
    let current = proposals(&claims, &collection_id);
    if preview_digest(&current)? != review.preview_digest {
        return Err(WikiError::Conflict(
            "knowledge changed since the subject preview; preview again".to_owned(),
        ));
    }
    let by_id = claims
        .iter()
        .map(|claim| (claim.claim_id.as_str(), claim))
        .collect::<BTreeMap<_, _>>();
    let previewed = current
        .iter()
        .map(|proposal| proposal.claim_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut pairs = Vec::new();
    for assignment in &review.assignments {
        validate_subject(&assignment.subject)
            .map_err(|error| WikiError::InvalidInput(error.to_string()))?;
        if !previewed.contains(assignment.claim_id.as_str()) {
            return Err(WikiError::InvalidInput(format!(
                "claim `{}` was not in the subject preview",
                assignment.claim_id
            )));
        }
        let original = by_id[assignment.claim_id.as_str()];
        if original.digest != assignment.expected_digest {
            return Err(WikiError::Conflict(format!(
                "claim `{}` changed after review",
                assignment.claim_id
            )));
        }
        let mut replacement = original.clone();
        replacement.subject = Some(assignment.subject.clone());
        replacement.digest = claim_digest(&replacement);
        pairs.push((original.clone(), replacement));
    }
    let operation_bytes = serde_json::to_vec(&(
        "subject-assignment-v1",
        &review.preview_digest,
        pairs
            .iter()
            .map(|(_, replacement)| (&replacement.claim_id, &replacement.digest))
            .collect::<Vec<_>>(),
    ))
    .map_err(|error| WikiError::Io(error.to_string()))?;
    let operation_id = sha256_digest(&operation_bytes)
        .trim_start_matches("sha256:")
        .chars()
        .take(24)
        .collect::<String>();
    let record = RestoreRecord {
        schema_version: 1,
        operation_id: operation_id.clone(),
        collection_id,
        originals: pairs.iter().map(|(original, _)| original.clone()).collect(),
        applied: pairs
            .iter()
            .map(|(_, replacement)| replacement.clone())
            .collect(),
    };
    let record_path = user_root
        .join(RESTORE_RELATIVE)
        .join(format!("{operation_id}.json"));
    // The restore record exists before the canonical write, so an interrupted apply
    // still has its exact originals.
    write_record(&record_path, &record)?;
    let committed = store.replace_active_claims(&pairs)?;
    let mut changed_paths = committed.changed_paths.clone();
    changed_paths.push(format!("{RESTORE_RELATIVE}/{operation_id}.json"));
    Ok(success(
        "ApplyKnowledgeSubjects",
        "hive.knowledge-subjects-applied",
        "reviewed subjects were written to canonical Markdown with a restore record",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &committed.manifest_digest,
        json!({"operation_id": operation_id, "assigned": pairs.len(), "store": committed,
               "restore": format!("hive knowledge subjects restore --user-root <user-root> --operation {operation_id} --confirm-subject-restore yes --output json")}),
    ))
}

fn restore(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &["--user-root", "--operation", "--confirm-subject-restore"],
    )?;
    if optional(&options, "--confirm-subject-restore") != Some("yes") {
        return Err(WikiError::InvalidInput(
            "subject restore requires --confirm-subject-restore yes".to_owned(),
        ));
    }
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let operation = required(&options, "--operation")?;
    if operation.len() != 24 || !operation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(WikiError::InvalidInput(
            "--operation must be a 24-character operation id".to_owned(),
        ));
    }
    let record_path = user_root
        .join(RESTORE_RELATIVE)
        .join(format!("{operation}.json"));
    let record = read_json_bounded::<RestoreRecord>(&record_path, "subject restore record")?;
    if record.schema_version != 1
        || record.operation_id != operation
        || record.originals.len() != record.applied.len()
    {
        return Err(WikiError::InvalidInput(
            "subject restore record is malformed".to_owned(),
        ));
    }
    let (store, _, _) = open_store(&user_root)?;
    let pairs = record
        .applied
        .iter()
        .cloned()
        .zip(record.originals.iter().cloned())
        .collect::<Vec<_>>();
    let committed = store.replace_active_claims(&pairs)?;
    let restored_path = record_path.with_extension("restored.json");
    fs::rename(&record_path, &restored_path)
        .map_err(|error| WikiError::Io(format!("cannot retire subject restore record: {error}")))?;
    Ok(success(
        "RestoreKnowledgeSubjects",
        "hive.knowledge-subjects-restored",
        "claims returned to their exact pre-assignment canonical bytes",
        committed.changed_paths.clone(),
        SHARED_INDEX_RELATIVE,
        &committed.manifest_digest,
        json!({"operation_id": operation, "restored": pairs.len(), "store": committed}),
    ))
}

fn write_record(path: &Path, record: &RestoreRecord) -> Result<(), WikiError> {
    let parent = path
        .parent()
        .ok_or_else(|| WikiError::Io("subject restore record has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| WikiError::Io(format!("cannot create subject restore folder: {error}")))?;
    let bytes =
        serde_json::to_vec_pretty(record).map_err(|error| WikiError::Io(error.to_string()))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| WikiError::Io(format!("cannot write subject restore record: {error}")))?;
    fs::rename(&temporary, path)
        .map_err(|error| WikiError::Io(format!("cannot publish subject restore record: {error}")))
}
