//! Optional semantic candidates never replace FTS authority or canonical citations.
use super::{
    contract_digest, invalid, io_error, lock, optional, required, scope_control, verified_query,
    InstalledRuntime, Selector, Target,
};
use hive_wiki::rag::{
    literal_query_matches, normalize_fusion_scores, RetrievalRequest, SemanticMatch,
    SEMANTIC_FUSION_WEIGHT, SEMANTIC_RANKING_POLICY,
};
use hive_wiki::store::RagStore;
use hive_wiki::vector::{DatabaseKind, VectorFiles};
use hive_wiki::{source, WikiError};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

fn search_metadata(
    used: bool,
    partial: bool,
    fts_order_preserved: bool,
) -> Result<Value, WikiError> {
    let lock = lock()?;
    Ok(
        json!({"requested":"semantic","used":if used {vec!["fts","vector"]} else {vec!["fts"]},
        "fallback":if !used {"unavailable"} else if partial {"partial"} else {"none"},
        "model":{"id":lock["model"]["id"],"revision":lock["model"]["revision"]},
        "engine":{"name":"sqlite-vec","version":"0.1.9"},
        "fusion":{"method":"min-max-score","semantic_weight":SEMANTIC_FUSION_WEIGHT,"fts_weight":1.0-SEMANTIC_FUSION_WEIGHT,
            "ranking_policy":SEMANTIC_RANKING_POLICY,"fts_order_preserved":fts_order_preserved},"contract_digest":contract_digest()?}),
    )
}

fn annotate(
    mut result: Value,
    used: bool,
    partial: bool,
    fts_order_preserved: bool,
) -> Result<Value, WikiError> {
    result["search"] = search_metadata(used, partial, fts_order_preserved)?;
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
        Ok((result, partial, protected)) => annotate(result, true, partial, protected),
        // Preserve the existing FTS validation and original query budgets; do not return a
        // cached pre-error result, partial vector output, or diagnostics from private partitions.
        Err(_) => annotate(
            serde_json::to_value(store.checked_retrieve(request)?).map_err(io_error)?,
            false,
            false,
            false,
        ),
    }
}

fn retain_runtime_candidate(runtimes: &mut Vec<InstalledRuntime>, candidate: InstalledRuntime) {
    // A shared directory ID does not authenticate another scope's receipt or Python identity.
    if !runtimes.contains(&candidate) {
        runtimes.push(candidate);
    }
}

fn hybrid(
    root: &Path,
    store: &RagStore,
    request: &RetrievalRequest,
) -> Result<(Value, bool, bool), WikiError> {
    let files = VectorFiles::open(root, false)?;
    let frame = store.begin_semantic_query(request)?;
    let mut databases = Vec::new();
    let mut controls = Vec::new();
    let mut runtimes: Vec<InstalledRuntime> = Vec::new();
    let mut partial = false;
    for state in &frame.plan().partitions {
        let selector = Selector::Collection {
            partition: state.partition.clone(),
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
        retain_runtime_candidate(&mut runtimes, control.runtime.clone());
        databases.push(json!({"database":files.database_path(&scope_id,DatabaseKind::Generation(&active.id))?,
            "manifest_digest":active.manifest_digest,"expected_database_digest":active.database_digest}));
        controls.push((target, digest));
    }
    if databases.len() > 256 {
        return Err(invalid("semantic scope count exceeds query bound"));
    }
    let mut answer = None;
    let started = std::time::Instant::now();
    for runtime in runtimes {
        let remaining = std::time::Duration::from_secs(10)
            .saturating_sub(started.elapsed())
            .as_secs();
        if remaining == 0 {
            break;
        }
        match verified_query(
            &files,
            &runtime,
            &json!({"schema_version":1,"action":"query-many","runtime":files.runtime_path(&runtime.id)?,
                "databases":databases,"query":request.query,"limit":100,"contract_digest":runtime.contract_digest}),
            remaining,
        ) {
            Ok(result) => {
                answer = Some(result);
                break;
            }
            Err(_) => partial = true,
        }
    }
    let result = answer.ok_or_else(|| invalid("no verified semantic runtime available"))?;
    let matches: Vec<SemanticMatch> =
        serde_json::from_value(result["matches"].clone()).map_err(io_error)?;
    for (target, expected) in controls {
        if scope_control(&target)?.1.as_deref() != Some(&expected) {
            return Err(invalid("semantic authority changed during query"));
        }
    }
    let fusion = frame.finish_with_policy(&matches)?;
    Ok((
        serde_json::to_value(fusion.result).map_err(io_error)?,
        partial,
        fusion.fts_order_preserved,
    ))
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
            json!({"hits":source::query(target.files.root_path(),language,Some(text),None,limit)?,"search":search_metadata(false,false,false)?}),
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
    let corpus = source::semantic_corpus(target.files.root_path(), language)?;
    if corpus.manifest_digest != active.manifest_digest {
        return Err(invalid("source vectors stale"));
    }
    let lexical =
        source::query_with_scores(target.files.root_path(), language, Some(text), None, 100)?;
    let response = verified_query(
        &target.files,
        &control.runtime,
        &json!({"schema_version":1,"action":"query","runtime":target.files.runtime_path(&control.runtime.id)?,
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
    let semantic = semantic
        .into_iter()
        .map(|hit| {
            let score = matches
                .iter()
                .find(|candidate| candidate.chunk_id == hit.path)
                .ok_or_else(|| invalid("source semantic score is absent"))?
                .score;
            Ok((hit, score))
        })
        .collect::<Result<Vec<_>, WikiError>>()?;
    let SourceFusion {
        hits: ranked,
        fts_order_preserved: protected,
    } = fuse_source_hits(text, &corpus, &lexical, &semantic, limit)?;
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
        json!({"manifest_digest":corpus.manifest_digest,"hits":hits,"search":search_metadata(true,false,protected)?}),
    )
}

#[derive(Debug)]
struct SourceFusion {
    hits: Vec<(source::SourceQueryHit, f64, u8)>,
    fts_order_preserved: bool,
}

fn fuse_source_hits(
    query: &str,
    corpus: &source::SourceSemanticCorpus,
    lexical: &[(source::SourceQueryHit, f64)],
    semantic: &[(source::SourceQueryHit, f64)],
    limit: usize,
) -> Result<SourceFusion, WikiError> {
    let mut ranked: BTreeMap<String, (source::SourceQueryHit, f64, u8)> = BTreeMap::new();
    for (mask, hits) in [(1, lexical), (2, semantic)] {
        let mut seen = std::collections::BTreeSet::new();
        let scores =
            normalize_fusion_scores(&hits.iter().map(|(_, score)| *score).collect::<Vec<_>>())
                .map_err(io_error)?;
        let weight = if mask == 2 {
            SEMANTIC_FUSION_WEIGHT
        } else {
            1.0 - SEMANTIC_FUSION_WEIGHT
        };
        for ((hit, _), score) in hits.iter().zip(scores) {
            if !seen.insert(&hit.path) {
                return Err(invalid("duplicate source fusion candidate"));
            }
            let entry = ranked
                .entry(hit.path.clone())
                .or_insert_with(|| (hit.clone(), 0.0, 0));
            if entry.0 != *hit {
                return Err(invalid("source citation changed during fusion"));
            }
            entry.1 += weight * score;
            entry.2 |= mask;
        }
    }
    let pages = corpus
        .pages
        .iter()
        .map(|page| (page.hit.path.as_str(), page))
        .collect::<BTreeMap<_, _>>();
    let lookup = hive_wiki::collection::folded_alias(query);
    let mut protected = false;
    for (hit, _) in lexical {
        let page = pages
            .get(hit.path.as_str())
            .filter(|page| page.hit == *hit)
            .ok_or_else(|| invalid("source literal citation changed"))?;
        protected |= literal_query_matches(
            query,
            hive_wiki::collection::folded_alias(&hit.pair_id) == lookup
                || hit
                    .aliases
                    .iter()
                    .any(|alias| hive_wiki::collection::folded_alias(alias) == lookup),
            &[&hit.pair_id, &hit.title, &hit.summary, &page.body],
        );
    }
    let lexical_order = if protected {
        lexical
            .iter()
            .enumerate()
            .map(|(index, (hit, _))| (hit.path.as_str(), index))
            .collect::<BTreeMap<_, _>>()
    } else {
        BTreeMap::new()
    };
    let mut ranked = ranked.into_values().collect::<Vec<_>>();
    ranked.sort_by(|a, b| {
        match (
            lexical_order.get(a.0.path.as_str()),
            lexical_order.get(b.0.path.as_str()),
        ) {
            (Some(a), Some(b)) => return a.cmp(b),
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }
        b.1.total_cmp(&a.1).then_with(|| a.0.path.cmp(&b.0.path))
    });
    ranked.truncate(limit);
    Ok(SourceFusion {
        hits: ranked,
        fts_order_preserved: protected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    type ScoredSourceHits = Vec<(source::SourceQueryHit, f64)>;

    fn literal_source_fixture() -> (
        source::SourceSemanticCorpus,
        ScoredSourceHits,
        ScoredSourceHits,
    ) {
        let page = |id: &str, body: &str| source::SourceSemanticPage {
            hit: source::SourceQueryHit {
                language: "en".to_owned(),
                pair_id: id.to_owned(),
                topic_slug: id.to_owned(),
                counterpart: format!("../ko/{id}.md"),
                title: id.to_owned(),
                summary: "Test source page".to_owned(),
                path: format!("docs/facts/en/{id}.md"),
                content_digest: hive_core::sha256_digest(id.as_bytes()),
                reviewed_revision: format!("git:{}", "a".repeat(40)),
                tags: Vec::new(),
                aliases: Vec::new(),
                sources: Vec::new(),
            },
            body: body.to_owned(),
        };
        let pages = vec![
            page("first", "Original leading result"),
            page("second", "never write secrets"),
            page("extra", "Related safety guidance"),
        ];
        let lexical = vec![(pages[0].hit.clone(), 10.0), (pages[1].hit.clone(), 1.0)];
        let semantic = vec![(pages[2].hit.clone(), 1.0), (pages[0].hit.clone(), 0.0)];
        (
            source::SourceSemanticCorpus {
                manifest_digest: "test".to_owned(),
                pages,
            },
            lexical,
            semantic,
        )
    }

    #[test]
    fn source_literal_order_matches_knowledge_policy_and_preserves_the_nonliteral_leader() {
        let (mut corpus, mut lexical, mut semantic) = literal_source_fixture();
        let result = fuse_source_hits("never write secrets", &corpus, &lexical, &semantic, 3)
            .expect("literal fusion");
        assert!(result.fts_order_preserved);
        assert_eq!(
            result
                .hits
                .iter()
                .map(|(hit, _, _)| hit.pair_id.as_str())
                .collect::<Vec<_>>(),
            ["first", "second", "extra"]
        );
        assert!(result.hits[0].1 < result.hits[2].1);
        let broad = fuse_source_hits("related safety", &corpus, &lexical, &semantic, 3)
            .expect("weighted fusion");
        assert!(!broad.fts_order_preserved);
        assert_eq!(broad.hits[0].0.pair_id, "extra");
        corpus.pages[0].hit.aliases.push("shortcut".to_owned());
        lexical[0].0.clone_from(&corpus.pages[0].hit);
        semantic[1].0.clone_from(&corpus.pages[0].hit);
        let alias = fuse_source_hits("shortcut", &corpus, &lexical, &semantic, 1)
            .expect("alias-only lookup");
        assert!(alias.fts_order_preserved);
        assert_eq!(alias.hits[0].0.pair_id, "first");
    }

    #[test]
    fn source_literal_policy_never_accepts_duplicate_or_changed_citations() {
        let (mut corpus, mut lexical, semantic) = literal_source_fixture();
        lexical.push(lexical[0].clone());
        assert!(fuse_source_hits("never write secrets", &corpus, &lexical, &semantic, 3).is_err());
        lexical.pop();
        corpus.pages[1].hit.content_digest = hive_core::sha256_digest(b"changed");
        assert!(fuse_source_hits("never write secrets", &corpus, &lexical, &semantic, 3).is_err());
    }

    #[test]
    fn annotation_explains_protected_order_without_inflating_weighted_scores() {
        let result = annotate(json!({"hits":[{"matched_field":"hybrid","score":0.25},{"matched_field":"vector","score":0.75}]}), true, false, true).expect("annotate");
        assert_eq!(result["search"]["fusion"]["fts_order_preserved"], true);
        assert_eq!(
            result["search"]["fusion"]["ranking_policy"],
            SEMANTIC_RANKING_POLICY
        );
        assert_eq!(result["hits"][0]["score"], 0.25);
        assert_eq!(result["hits"][0]["fusion_rank"], 1);
        assert_eq!(result["hits"][1]["score"], 0.75);
    }

    #[test]
    fn shared_runtime_id_keeps_distinct_approval_candidates() {
        let good = InstalledRuntime {
            id: "a".repeat(64),
            python: "python".into(),
            identity: json!({"platform":"approved"}),
            contract_digest: "contract".to_owned(),
            receipt_digest: "approved".to_owned(),
            consent_digest: "consent".to_owned(),
        };
        let mut bad = good.clone();
        bad.receipt_digest = "corrupt".to_owned();
        let mut candidates = Vec::new();
        retain_runtime_candidate(&mut candidates, bad);
        retain_runtime_candidate(&mut candidates, good.clone());
        retain_runtime_candidate(&mut candidates, good.clone());
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[1], good);
    }

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
