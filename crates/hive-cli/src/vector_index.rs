//! Bounded generation construction. Only verified immutable copies enter scope control.
use super::{
    auth, contract_digest, invalid, io_error, optional, scope_control, value_digest,
    verify_runtime, worker, ScopeControl, Selector, Target,
};
use hive_wiki::rag::{RetrievalRequest, RetrievalScope};
use hive_wiki::store::RagStore;
use hive_wiki::vector::{DatabaseKind, VectorFiles};
use hive_wiki::{source, WikiError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Snapshot {
    pub(super) id: String,
    pub(super) database_digest: String,
    pub(super) manifest_digest: String,
    pub(super) contract_digest: String,
    pub(super) runtime_id: String,
    pub(super) chunks: usize,
}

struct Corpus {
    manifest_digest: String,
    authority_digest: Option<String>,
    chunks: Vec<Value>,
    request: Option<RetrievalRequest>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildResult {
    schema_version: u32,
    complete: bool,
    phase: String,
    embedded: usize,
    remaining: usize,
    chunks: usize,
    elapsed_seconds: f64,
    database_digest: String,
}

fn corpus(
    target: &Target,
    authorization: Option<(&[(&str, &str)], &str)>,
) -> Result<Corpus, WikiError> {
    match &target.selector {
        Selector::Source { language } => {
            let corpus = source::semantic_corpus(target.files.root_path(), language)?;
            Ok(Corpus { manifest_digest: corpus.manifest_digest, authority_digest: None, request: None,
                chunks: corpus.pages.into_iter().map(|page| json!({"chunk_id":page.hit.path,"digest":page.hit.content_digest,"title":page.hit.title,"text":page.body})).collect() })
        }
        Selector::Collection { partition } => {
            let request = RetrievalRequest {
                scope: RetrievalScope::Collection(partition.collection_id.clone()),
                current_collection_id: target.current_collection_id.clone(),
                query: "vector build".to_owned(),
                query_expansions: Vec::new(),
                top_k: 100,
                byte_budget: 1024 * 1024,
                confidential_collection_id: auth::confidential(target)
                    .then(|| partition.collection_id.clone()),
            };
            let store = RagStore::open(target.files.root_path())?;
            let corpus = if auth::confidential(target) {
                let (options, operation) = authorization.ok_or_else(|| {
                    invalid("confidential corpus requires current build authorization")
                })?;
                store.authorized_semantic_corpus(&request, partition.visibility, |manifest| {
                    auth::consume_locked(target, options, operation, manifest)
                })?
            } else {
                store.semantic_corpus(&request, partition.visibility)?
            };
            Ok(Corpus { manifest_digest: corpus.partition_digest, authority_digest: Some(corpus.authority_digest), request: Some(request),
                chunks: corpus.chunks.into_iter().map(|hit| json!({"chunk_id":hit.chunk_id,"digest":hit.digest,"title":hit.title,"text":hit.text})).collect() })
        }
    }
}

fn compatible(snapshot: &Snapshot, control: &ScopeControl) -> bool {
    snapshot.runtime_id == control.runtime.id
        && snapshot.contract_digest == control.runtime.contract_digest
}

fn restore_staging(target: &Target, control: &ScopeControl) -> Result<Option<String>, WikiError> {
    let checkpoint = control
        .checkpoint
        .as_ref()
        .filter(|snapshot| compatible(snapshot, control))
        .map(|snapshot| (snapshot, DatabaseKind::Checkpoint(snapshot.id.as_str())));
    let active = {
        control
            .active
            .as_ref()
            .filter(|snapshot| compatible(snapshot, control))
            .map(|snapshot| (snapshot, DatabaseKind::Generation(snapshot.id.as_str())))
    };
    for (snapshot, kind) in checkpoint.into_iter().chain(active) {
        // The immutable copy must match authority outside SQLite before any bytes are reused.
        if target
            .files
            .database_digest(&target.scope_id, kind)
            .ok()
            .as_deref()
            != Some(&snapshot.database_digest)
        {
            continue;
        }
        if target
            .files
            .database_digest(&target.scope_id, DatabaseKind::Staging)
            .ok()
            .as_deref()
            == Some(&snapshot.database_digest)
        {
            return Ok(Some(snapshot.database_digest.clone()));
        }
        target.files.quarantine_staging(&target.scope_id)?;
        target.files.copy_database(
            &target.scope_id,
            kind,
            DatabaseKind::Staging,
            &snapshot.database_digest,
        )?;
        return Ok(Some(snapshot.database_digest.clone()));
    }
    target.files.quarantine_staging(&target.scope_id)?;
    Ok(None)
}

fn publish(
    target: &Target,
    corpus: &Corpus,
    before: &ScopeControl,
    expected: &str,
    after: &ScopeControl,
) -> Result<(), WikiError> {
    let after_digest = value_digest(after)?;
    let install = || {
        target
            .files
            .write_control(Some(&target.scope_id), Some(expected), after)
    };
    let rollback = || {
        target
            .files
            .write_control(Some(&target.scope_id), Some(&after_digest), before)
    };
    match &target.selector {
        Selector::Source { .. } => source::with_semantic_snapshot(
            target.files.root_path(),
            &corpus.manifest_digest,
            install,
            rollback,
        ),
        Selector::Collection { partition } => RagStore::open(target.files.root_path())?
            .with_semantic_snapshot(
                corpus
                    .request
                    .as_ref()
                    .ok_or_else(|| invalid("vector request authority is absent"))?,
                partition.visibility,
                &corpus.manifest_digest,
                corpus
                    .authority_digest
                    .as_deref()
                    .ok_or_else(|| invalid("vector operation authority is absent"))?,
                install,
                rollback,
            ),
    }
}

pub(super) fn rebuild(target: &Target, options: &[(&str, &str)]) -> Result<Value, WikiError> {
    auth::validate_fields(target, options)?;
    let seconds = super::super::parse_bounded_usize(
        optional(options, "--max-seconds"),
        30,
        1,
        60,
        "--max-seconds",
    )?;
    let workers =
        super::super::parse_bounded_usize(optional(options, "--workers"), 8, 1, 8, "--workers")?;
    let _lease = target.files.writer(Some(&target.scope_id))?;
    let (control, digest) = scope_control(target)?;
    let before = control
        .filter(|value| value.enabled)
        .ok_or_else(|| invalid("vector scope is disabled; preview and enable first"))?;
    verify_runtime(&target.files, &before.runtime)?;
    let corpus = corpus(target, Some((options, "rebuild")))?;
    let expected = match optional(options, "--rebuild-mode").unwrap_or("resume") {
        "resume" => restore_staging(target, &before)?,
        "fresh" => {
            target.files.quarantine_staging(&target.scope_id)?;
            None
        }
        _ => return Err(invalid("--rebuild-mode must be resume or fresh")),
    };
    let database = target.files.prepare_staging(&target.scope_id)?;
    let request = json!({"schema_version":1,"action":"build","runtime":target.files.runtime_path(&before.runtime.id)?,
        "database":database,"chunks":corpus.chunks,"contract_digest":before.runtime.contract_digest,
        "manifest_digest":corpus.manifest_digest,"workers":workers,"max_seconds":seconds,"expected_database_digest":expected});
    let result = match worker(
        &target.files,
        &before.runtime,
        request,
        u64::try_from(seconds).map_err(io_error)? + 35,
    ) {
        Ok(value) => value,
        Err(error) => {
            target.files.quarantine_staging(&target.scope_id)?;
            return Err(error);
        }
    };
    let result = validate_build_result(target, corpus.chunks.len(), result)?;
    let snapshot = Snapshot {
        id: VectorFiles::fresh_snapshot_id(),
        database_digest: result.database_digest,
        manifest_digest: corpus.manifest_digest.clone(),
        contract_digest: before.runtime.contract_digest.clone(),
        runtime_id: before.runtime.id.clone(),
        chunks: result.chunks,
    };
    let kind = if result.complete {
        DatabaseKind::Generation(&snapshot.id)
    } else {
        DatabaseKind::Checkpoint(&snapshot.id)
    };
    target.files.copy_database(
        &target.scope_id,
        DatabaseKind::Staging,
        kind,
        &snapshot.database_digest,
    )?;
    let mut after = before.clone();
    after.revision = before
        .revision
        .checked_add(1)
        .ok_or_else(|| invalid("vector control revision is exhausted"))?;
    if result.complete {
        after.previous = after.active.take();
        after.active = Some(snapshot.clone());
        after.checkpoint = None;
    } else {
        after.checkpoint = Some(snapshot.clone());
    }
    publish(
        target,
        &corpus,
        &before,
        digest
            .as_deref()
            .ok_or_else(|| invalid("scope control digest is absent"))?,
        &after,
    )?;
    Ok(
        json!({"complete":result.complete,"phase":result.phase,"embedded":result.embedded,"remaining":result.remaining,
        "chunks":result.chunks,"snapshot_id":snapshot.id,"database_digest":snapshot.database_digest,
        "manifest_digest":snapshot.manifest_digest,"worker_seconds":result.elapsed_seconds,
        "requires_new_authorization":auth::confidential(target) && !result.complete,
        "fts_unchanged":true,"changed_paths":changed_paths(target,true)}),
    )
}

pub(super) fn status(target: &Target, control: &ScopeControl) -> Value {
    if auth::confidential(target) {
        return json!({"index_ready":null,"requires_current_action":true});
    }
    let Some(active) = control
        .active
        .as_ref()
        .filter(|snapshot| compatible(snapshot, control))
    else {
        return json!({"index_ready":false,"checkpoint_available":control.checkpoint.is_some()});
    };
    let verified = target
        .files
        .database_digest(&target.scope_id, DatabaseKind::Generation(&active.id))
        .is_ok_and(|digest| digest == active.database_digest);
    let current =
        corpus(target, None).is_ok_and(|corpus| corpus.manifest_digest == active.manifest_digest);
    json!({"index_ready":verified && current,"snapshot_verified":verified,"canonical_current":current,
        "checkpoint_available":control.checkpoint.is_some()})
}

fn validate_build_result(
    target: &Target,
    chunks: usize,
    value: Value,
) -> Result<BuildResult, WikiError> {
    let result = (|| {
        let result: BuildResult = serde_json::from_value(value).map_err(io_error)?;
        if result.schema_version != 1
            || result.chunks != chunks
            || !result.elapsed_seconds.is_finite()
            || result.complete != (result.phase == "ready")
            || !["ready", "embedding", "finalizing"].contains(&result.phase.as_str())
            || target
                .files
                .database_digest(&target.scope_id, DatabaseKind::Staging)?
                != result.database_digest
        {
            return Err(invalid(
                "vector worker output differs from its closed database",
            ));
        }
        Ok(result)
    })();
    if result.is_err() {
        target.files.quarantine_staging(&target.scope_id)?;
    }
    result
}

pub(super) fn rollback(target: &Target, options: &[(&str, &str)]) -> Result<Value, WikiError> {
    auth::validate_fields(target, options)?;
    let _lease = target.files.writer(Some(&target.scope_id))?;
    let (control, digest) = scope_control(target)?;
    let before = control.ok_or_else(|| invalid("vector scope has no rollback state"))?;
    verify_runtime(&target.files, &before.runtime)?;
    let corpus = corpus(target, Some((options, "rollback")))?;
    let previous = before
        .previous
        .as_ref()
        .filter(|snapshot| compatible(snapshot, &before))
        .ok_or_else(|| invalid("no compatible prior vector generation"))?;
    if previous.manifest_digest != corpus.manifest_digest
        || previous.contract_digest != contract_digest()?
        || target
            .files
            .database_digest(&target.scope_id, DatabaseKind::Generation(&previous.id))?
            != previous.database_digest
    {
        return Err(invalid(
            "prior vector generation is stale or corrupt; FTS remains available",
        ));
    }
    let mut after = before.clone();
    after.revision = before
        .revision
        .checked_add(1)
        .ok_or_else(|| invalid("vector control revision is exhausted"))?;
    after.active = after.previous.take();
    after.previous.clone_from(&before.active);
    after.checkpoint = None;
    publish(
        target,
        &corpus,
        &before,
        digest
            .as_deref()
            .ok_or_else(|| invalid("scope control digest is absent"))?,
        &after,
    )?;
    Ok(
        json!({"rolled_back":true,"enabled":after.enabled,"fts_unchanged":true,"changed_paths":changed_paths(target,false)}),
    )
}

fn changed_paths(target: &Target, data: bool) -> Vec<&std::path::Path> {
    let mut paths = vec![target.files.control_relative()];
    if data {
        paths.push(target.files.data_relative());
    }
    if auth::confidential(target) {
        paths.push(std::path::Path::new(
            super::super::KNOWLEDGE_AUTHORIZATION_RELATIVE,
        ));
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::vector::InstalledRuntime;
    use hive_core::sha256_digest;
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    #[test]
    fn killed_mutable_cache_restores_only_an_externally_authenticated_copy() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let root = tempfile::tempdir_in(base).expect("root");
        let files = VectorFiles::open(root.path(), false).expect("files");
        let selector = Selector::Collection {
            partition: hive_wiki::rag::SemanticPartition {
                collection_id: "user-root".to_owned(),
                visibility: hive_wiki::rag::RagVisibility::Shared,
            },
        };
        let scope_id = files.scope_id(&selector).expect("scope");
        let target = Target {
            files,
            selector,
            scope_id,
            current_collection_id: None,
        };
        let stage = target
            .files
            .prepare_staging(&target.scope_id)
            .expect("stage");
        fs::write(&stage, b"verified checkpoint").expect("checkpoint bytes");
        let digest = sha256_digest(b"verified checkpoint");
        let id = VectorFiles::fresh_snapshot_id();
        target
            .files
            .copy_database(
                &target.scope_id,
                DatabaseKind::Staging,
                DatabaseKind::Checkpoint(&id),
                &digest,
            )
            .expect("copy");
        let contract = contract_digest().expect("contract");
        let runtime_id = "a".repeat(64);
        let snapshot = Snapshot {
            id,
            database_digest: digest.clone(),
            manifest_digest: sha256_digest(b"manifest"),
            contract_digest: contract.clone(),
            runtime_id: runtime_id.clone(),
            chunks: 1,
        };
        let control = ScopeControl {
            schema_version: 1,
            revision: 1,
            selector: target.selector.clone(),
            enabled: true,
            consent_digest: sha256_digest(b"consent"),
            runtime: InstalledRuntime {
                id: runtime_id,
                python: PathBuf::new(),
                identity: json!({}),
                contract_digest: contract,
                receipt_digest: sha256_digest(b"receipt"),
                consent_digest: sha256_digest(b"consent"),
            },
            checkpoint: Some(snapshot.clone()),
            active: None,
            previous: None,
        };
        fs::write(&stage, b"killed worker bytes").expect("interruption");
        fs::write(
            format!("{}-journal", stage.display()),
            b"incomplete transaction",
        )
        .expect("journal");
        assert_eq!(
            restore_staging(&target, &control).expect("recovery"),
            Some(digest.clone())
        );
        assert_eq!(fs::read(&stage).expect("restored"), b"verified checkpoint");
        assert_eq!(
            target
                .files
                .database_digest(&target.scope_id, DatabaseKind::Staging)
                .expect("closed"),
            digest
        );
        let checkpoint = target
            .files
            .database_path(&target.scope_id, DatabaseKind::Checkpoint(&snapshot.id))
            .expect("path");
        fs::write(checkpoint, b"forged cache with matching internal checksums")
            .expect("corruption");
        assert_eq!(
            restore_staging(&target, &control).expect("fresh recovery"),
            None
        );
        assert!(!stage.exists());
    }
}
