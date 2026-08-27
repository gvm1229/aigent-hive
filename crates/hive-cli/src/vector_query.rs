//! Optional semantic candidates never replace FTS authority or canonical citations.
use super::{
    contract_digest, invalid, io_error, lock, optional, required, scope_control, verify_runtime,
    worker, InstalledRuntime, Selector, Target,
};
use hive_wiki::rag::{RetrievalRequest, SemanticMatch};
use hive_wiki::store::RagStore;
use hive_wiki::vector::{DatabaseKind, VectorFiles};
use hive_wiki::{source, WikiError};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

fn search_metadata(used: bool, partial: bool) -> Result<Value, WikiError> {
    let lock = lock()?;
    Ok(
        json!({"requested":"semantic","used":if used {vec!["fts","vector"]} else {vec!["fts"]},
        "fallback":if !used {"unavailable"} else if partial {"partial"} else {"none"},
        "model":{"id":lock["model"]["id"],"revision":lock["model"]["revision"]},
        "engine":{"name":"sqlite-vec","version":"0.1.9"},"rrf_constant":60,"contract_digest":contract_digest()?}),
    )
}

fn annotate(mut result: Value, used: bool, partial: bool) -> Result<Value, WikiError> {
    result["search"] = search_metadata(used, partial)?;
    if let Some(hits) = result["hits"].as_array_mut() {
        for (index, hit) in hits.iter_mut().enumerate() {
            hit["matched_lanes"] = match hit["matched_field"].as_str() {
                Some("vector") => json!(["vector"]),
                Some("hybrid") => json!(["fts", "vector"]),
                _ => json!(["fts"]),
            };
            hit["fusion_rank"] = json!(index + 1);
        }
    }
    Ok(result)
}

pub(super) fn retrieve(
    root: &Path,
    store: &RagStore,
    request: &RetrievalRequest,
) -> Result<Value, WikiError> {
    match hybrid(root, store, request) {
        Ok((result, partial)) => annotate(result, true, partial),
        // Preserve the existing FTS validation and original query budgets; do not return a
        // cached pre-error result, partial vector output, or diagnostics from private partitions.
        Err(_) => annotate(
            serde_json::to_value(store.checked_retrieve(request)?).map_err(io_error)?,
            false,
            false,
        ),
    }
}

fn hybrid(
    root: &Path,
    store: &RagStore,
    request: &RetrievalRequest,
) -> Result<(Value, bool), WikiError> {
    let files = VectorFiles::open(root, false)?;
    let plan = store.semantic_search_plan(request)?;
    let mut databases = Vec::new();
    let mut controls = Vec::new();
    let mut runtime: Option<InstalledRuntime> = None;
    let mut partial = false;
    for state in plan.partitions {
        let selector = Selector::Collection {
            partition: state.partition,
        };
        let scope_id = files.scope_id(&selector)?;
        let target = Target {
            files: VectorFiles::open(root, false)?,
            selector,
            scope_id: scope_id.clone(),
            current_collection_id: request.current_collection_id.clone(),
        };
        let Ok((Some(control), Some(digest))) = scope_control(&target) else {
            partial = true;
            continue;
        };
        let Some(active) = control.active.filter(|active| {
            control.enabled
                && active.manifest_digest == state.digest
                && active.runtime_id == control.runtime.id
                && active.contract_digest == control.runtime.contract_digest
        }) else {
            partial = true;
            continue;
        };
        if control.runtime.contract_digest != contract_digest()? {
            partial = true;
            continue;
        }
        if runtime.is_none() {
            if verify_runtime(&files, &control.runtime).is_err() {
                partial = true;
                continue;
            }
            runtime = Some(control.runtime.clone());
        }
        databases.push(json!({"database":files.database_path(&scope_id,DatabaseKind::Generation(&active.id))?,
            "manifest_digest":active.manifest_digest,"expected_database_digest":active.database_digest}));
        controls.push((target, digest));
    }
    let runtime = runtime.ok_or_else(|| invalid("no active semantic scope"))?;
    if databases.len() > 256 {
        return Err(invalid("semantic scope count exceeds query bound"));
    }
    let result = worker(
        &files,
        &runtime,
        json!({"schema_version":1,"action":"query-many","runtime":files.runtime_path(&runtime.id)?,
        "databases":databases,"query":request.query,"limit":100,"contract_digest":runtime.contract_digest}),
        10,
    )?;
    let matches: Vec<SemanticMatch> =
        serde_json::from_value(result["matches"].clone()).map_err(io_error)?;
    for (target, expected) in controls {
        if scope_control(&target)?.1.as_deref() != Some(&expected) {
            return Err(invalid("semantic authority changed during query"));
        }
    }
    let result = store.hybrid_retrieve(request, &plan.manifest_digest, &matches)?;
    Ok((serde_json::to_value(result).map_err(io_error)?, partial))
}

pub(super) fn source_query(target: &Target, options: &[(&str, &str)]) -> Result<Value, WikiError> {
    let Selector::Source { language } = &target.selector else {
        return Err(invalid("source query requires source scope"));
    };
    let text = required(options, "--query")?;
    let limit =
        super::super::parse_bounded_usize(optional(options, "--top-k"), 5, 1, 100, "--top-k")?;
    match source_hybrid(target, language, text, limit) {
        Ok(result) => Ok(result),
        Err(_) => Ok(
            json!({"hits":source::query(target.files.root_path(),language,Some(text),None,limit)?,"search":search_metadata(false,false)?}),
        ),
    }
}

fn source_hybrid(
    target: &Target,
    language: &str,
    text: &str,
    limit: usize,
) -> Result<Value, WikiError> {
    let (control, scope_digest) = scope_control(target)?;
    let control = control
        .filter(|control| control.enabled)
        .ok_or_else(|| invalid("source vectors disabled"))?;
    let active = control
        .active
        .as_ref()
        .filter(|active| {
            active.runtime_id == control.runtime.id
                && active.contract_digest == control.runtime.contract_digest
        })
        .ok_or_else(|| invalid("source vector generation unavailable"))?;
    verify_runtime(&target.files, &control.runtime)?;
    let corpus = source::semantic_corpus(target.files.root_path(), language)?;
    if corpus.manifest_digest != active.manifest_digest {
        return Err(invalid("source vectors stale"));
    }
    let lexical = source::query(target.files.root_path(), language, Some(text), None, 100)?;
    let response = worker(
        &target.files,
        &control.runtime,
        json!({"schema_version":1,"action":"query","runtime":target.files.runtime_path(&control.runtime.id)?,
        "database":target.files.database_path(&target.scope_id,DatabaseKind::Generation(&active.id))?,"query":text,"limit":100,
        "contract_digest":active.contract_digest,"manifest_digest":active.manifest_digest,"expected_database_digest":active.database_digest}),
        10,
    )?;
    let matches: Vec<SemanticMatch> =
        serde_json::from_value(response["matches"].clone()).map_err(io_error)?;
    let semantic = source::semantic_matches(
        target.files.root_path(),
        language,
        &corpus.manifest_digest,
        &matches,
    )?;
    let mut ranked: BTreeMap<String, (source::SourceQueryHit, f64, u8)> = BTreeMap::new();
    for (mask, hits) in [(1, &lexical), (2, &semantic)] {
        for (index, hit) in hits.iter().enumerate() {
            let entry = ranked
                .entry(hit.path.clone())
                .or_insert_with(|| (hit.clone(), 0.0, 0));
            if entry.0 != *hit {
                return Err(invalid("source citation changed during fusion"));
            }
            entry.1 += 1.0 / (61.0 + f64::from(u32::try_from(index).map_err(io_error)?));
            entry.2 |= mask;
        }
    }
    let mut ranked = ranked.into_values().collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.path.cmp(&b.0.path)));
    ranked.truncate(limit);
    if scope_control(target)?.1 != scope_digest {
        return Err(invalid("source scope changed during query"));
    }
    let references = ranked
        .iter()
        .map(|(hit, score, _)| SemanticMatch {
            chunk_id: hit.path.clone(),
            digest: hit.content_digest.clone(),
            score: *score,
        })
        .collect::<Vec<_>>();
    source::semantic_matches(
        target.files.root_path(),
        language,
        &corpus.manifest_digest,
        &references,
    )?;
    let hits = ranked
        .into_iter()
        .enumerate()
        .map(|(index, (hit, score, mask))| {
            let mut value = serde_json::to_value(hit).map_err(io_error)?;
            value["score"] = json!(score);
            value["fusion_rank"] = json!(index + 1);
            value["matched_lanes"] = match mask {
                3 => json!(["fts", "vector"]),
                2 => json!(["vector"]),
                _ => json!(["fts"]),
            };
            Ok(value)
        })
        .collect::<Result<Vec<_>, WikiError>>()?;
    Ok(
        json!({"manifest_digest":corpus.manifest_digest,"hits":hits,"search":search_metadata(true,false)?}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "manual qualification: requires a consented runtime and synthetic source under tests/work"]
    fn real_source_query_uses_vector_candidates() {
        let path = std::path::PathBuf::from(
            std::env::var_os("HIVE_VECTOR_QUALIFICATION_TARGET").expect("disposable target"),
        );
        let work = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/work")
            .canonicalize()
            .expect("work");
        assert!(path.canonicalize().expect("target").starts_with(work));
        let files = VectorFiles::open(&path, true).expect("source");
        let selector = Selector::Source {
            language: "ko".to_owned(),
        };
        let scope_id = files.scope_id(&selector).expect("scope");
        let target = Target {
            files,
            selector,
            scope_id,
            current_collection_id: None,
        };
        let result =
            source_hybrid(&target, "ko", "자료를 되찾는 방법", 5).expect("actual semantic query");
        assert_eq!(result["search"]["used"], json!(["fts", "vector"]));
        assert!(!result["hits"].as_array().expect("hits").is_empty());
    }
}
