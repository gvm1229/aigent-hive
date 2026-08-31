//! Local transfer receipts and optional rebuild decisions; canonical knowledge stays independent.

use super::{optional, parse_options, required, success, KnowledgeResult};
use crate::user_install::{
    open_user_root_for_setup, read_user_setup_file, replace_user_setup_file,
};
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use hive_wiki::store::RagStore;
use hive_wiki::WikiError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const RECEIPT_ROOT: &str = ".hive/runtime/knowledge-transfer";
const MAX_RECEIPT: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum VectorDecision {
    Unanswered,
    Deferred,
    Requested,
    Complete,
    Pending,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    archive_sha256: String,
    manifest_sha256: String,
    collection_ids: Vec<String>,
    vector_state: VectorDecision,
}

fn path(id: &str) -> Result<PathBuf, WikiError> {
    hive_wiki::vector::validate_id(id)?;
    Ok(Path::new(RECEIPT_ROOT).join(format!("{id}.json")))
}

fn read(root: &Dir, id: &str) -> Result<(Option<Vec<u8>>, Option<Receipt>), WikiError> {
    let bytes = read_user_setup_file(root, &path(id)?, MAX_RECEIPT).map_err(WikiError::Conflict)?;
    let receipt: Option<Receipt> = bytes
        .as_deref()
        .map(serde_json::from_slice)
        .transpose()
        .map_err(|error| WikiError::Verification(format!("invalid transfer receipt: {error}")))?;
    if let Some(receipt) = &receipt {
        if receipt.schema_version != 1
            || receipt.archive_sha256 != format!("sha256:{id}")
            || !super::valid_sha256_digest(&receipt.manifest_sha256)
            || receipt.collection_ids.len() > 10_000
            || receipt
                .collection_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || receipt.collection_ids.iter().any(|id| {
                id != "user-root"
                    && id
                        .strip_prefix("collection-")
                        .is_none_or(|value| hive_wiki::vector::validate_id(value).is_err())
            })
        {
            return Err(WikiError::Verification(
                "transfer receipt identity is invalid".to_owned(),
            ));
        }
    }
    Ok((bytes, receipt))
}

fn save(
    root: &Dir,
    id: &str,
    expected: Option<&[u8]>,
    receipt: &Receipt,
) -> Result<Vec<u8>, WikiError> {
    let bytes = serde_json::to_vec(receipt).map_err(|error| WikiError::Io(error.to_string()))?;
    if bytes.len() as u64 > MAX_RECEIPT {
        return Err(WikiError::InvalidInput(
            "transfer receipt exceeds limit".to_owned(),
        ));
    }
    replace_user_setup_file(root, &path(id)?, expected, Some(&bytes))
        .map_err(WikiError::Conflict)?;
    Ok(bytes)
}

pub(super) fn finish_import(user_root: &Path, result: &mut KnowledgeResult) {
    // Optional index/receipt failures must not disguise a successful canonical import as a rollback.
    let Some(data) = result.data.as_mut() else {
        return;
    };
    if super::require_shared_wiki_enabled(user_root).is_err() {
        data["transfer"] = json!({"complete":false,"state":"fts-disabled"});
        result.next_action =
            Some("canonical knowledge restored; existing Wiki preference disables FTS".to_owned());
        return;
    }
    let ready = (|| -> Result<(), WikiError> {
        let store = RagStore::open(user_root)?;
        if store.validate_current().is_err() {
            let rebuilt = store.rebuild()?;
            result.changed_paths.extend(rebuilt.changed_paths);
            data["index_rebuilt"] = json!(true);
        }
        store.validate_current()?;
        Ok(())
    })();
    if ready.is_err() {
        data["transfer"] = json!({"complete":false,"state":"fts-pending"});
        result.next_action = Some(
            "canonical knowledge restored; repair and refresh FTS before completing transfer"
                .to_owned(),
        );
        return;
    }
    let saved = (|| -> Result<(String, Vec<u8>, Receipt), WikiError> {
        let archive = data["archive_sha256"]
            .as_str()
            .ok_or_else(|| WikiError::Verification("missing archive digest".to_owned()))?;
        let id = archive
            .strip_prefix("sha256:")
            .ok_or_else(|| WikiError::Verification("invalid archive digest".to_owned()))?
            .to_owned();
        let root = open_user_root_for_setup(user_root).map_err(WikiError::Conflict)?;
        let (before, existing) = read(&root, &id)?;
        let mut ids: Vec<String> = serde_json::from_value(data["collection_ids"].clone())
            .map_err(|error| WikiError::Verification(error.to_string()))?;
        ids.sort();
        ids.dedup();
        let manifest = data["manifest_sha256"]
            .as_str()
            .ok_or_else(|| WikiError::Verification("missing manifest digest".to_owned()))?;
        let receipt = match existing {
            Some(receipt)
                if receipt.collection_ids == ids && receipt.manifest_sha256 == manifest =>
            {
                receipt
            }
            Some(_) => {
                return Err(WikiError::Conflict(
                    "transfer receipt differs from validated bundle".to_owned(),
                ))
            }
            None => Receipt {
                schema_version: 1,
                archive_sha256: archive.to_owned(),
                manifest_sha256: manifest.to_owned(),
                collection_ids: ids,
                vector_state: VectorDecision::Unanswered,
            },
        };
        let bytes = save(&root, &id, before.as_deref(), &receipt)?;
        if before.as_deref() != Some(bytes.as_slice()) {
            result
                .changed_paths
                .push(path(&id)?.to_string_lossy().replace('\\', "/"));
        }
        Ok((id, bytes, receipt))
    })();
    if let Ok((id, bytes, receipt)) = saved {
        data["transfer"] = json!({"complete":true,"state":"complete","id":id,"receipt_digest":sha256_digest(&bytes),"vector_state":receipt.vector_state});
        if crate::user_setup::vector_search_enabled(user_root).unwrap_or(false) {
            data["vector_rebuild"] = json!({
                "state": if receipt.vector_state == VectorDecision::Unanswered { json!("question-required") } else { json!(receipt.vector_state) },
                "reason":"FTS is ready; the rebuild decision is separate from the saved vector preference",
                "scope":"imported portable collections only"
            });
        }
    } else {
        data["transfer"] = json!({"complete":true,"state":"record-unavailable"});
        result.next_action = Some("canonical knowledge and FTS ready; repair the local transfer receipt before optional vector setup".to_owned());
    }
    result.changed_paths.sort();
    result.changed_paths.dedup();
}

pub(super) fn run(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let action = arguments.first().map_or("", String::as_str);
    let options = parse_options(
        &arguments[1..],
        &["--user-root", "--id", "--answer", "--receipt-digest"],
    )?;
    let resolved_root = super::resolve_transfer_root(&options)?;
    let user_root = resolved_root.as_path();
    let id = required(&options, "--id")?;
    let root = open_user_root_for_setup(user_root).map_err(WikiError::Conflict)?;
    let (before, receipt) = read(&root, id)?;
    let mut receipt = receipt.ok_or_else(|| {
        WikiError::InvalidInput("completed transfer receipt not found".to_owned())
    })?;
    let before = before.expect("parsed receipt has bytes");
    let mut bytes = before.clone();
    let mut changed = Vec::new();
    let mut scopes = Value::Null;
    if action == "status" {
        if optional(&options, "--answer").is_some()
            || optional(&options, "--receipt-digest").is_some()
        {
            return Err(WikiError::InvalidInput(
                "status does not accept mutation options".to_owned(),
            ));
        }
    } else {
        if required(&options, "--receipt-digest")? != sha256_digest(&before) {
            return Err(WikiError::Conflict(
                "transfer receipt changed; inspect status again".to_owned(),
            ));
        }
        let answer = required(&options, "--answer")?;
        if !["yes", "no", "cancel"].contains(&answer) {
            return Err(WikiError::InvalidInput(
                "vector answer requires yes, no, or cancel".to_owned(),
            ));
        }
        if answer != "cancel" {
            if !crate::user_setup::vector_search_enabled(user_root)
                .map_err(WikiError::Verification)?
            {
                return Err(WikiError::Conflict(
                    "vector preference is not yes; transfer does not change that preference"
                        .to_owned(),
                ));
            }
            receipt.vector_state = if answer == "no" {
                VectorDecision::Deferred
            } else {
                VectorDecision::Requested
            };
            bytes = save(&root, id, Some(&before), &receipt)?;
            changed.push(path(id)?.to_string_lossy().replace('\\', "/"));
            if answer == "yes" {
                scopes = super::vector::rebuild_transferred(user_root, &receipt.collection_ids);
                receipt.vector_state = if scopes["complete"] == true {
                    VectorDecision::Complete
                } else {
                    VectorDecision::Pending
                };
                bytes = save(&root, id, Some(&bytes), &receipt)?;
            }
        }
    }
    let enabled = crate::user_setup::vector_search_enabled(user_root).unwrap_or(false);
    let fts_ready = RagStore::open(user_root)
        .and_then(|store| store.validate_current())
        .is_ok();
    Ok(success(
        "TransferKnowledge",
        "hive.knowledge-transfer-status",
        "transfer state inspected; vector preference unchanged",
        changed,
        &path(id)?.to_string_lossy(),
        &sha256_digest(&bytes),
        json!({
            "id":id,"complete":fts_ready,"receipt_digest":sha256_digest(&bytes),"vector_state":receipt.vector_state,
            "question_required":enabled && receipt.vector_state == VectorDecision::Unanswered,
            "collection_ids":receipt.collection_ids,"vector":scopes,"preference_changed":false
        }),
    ))
}
