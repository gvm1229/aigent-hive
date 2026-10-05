//! Host-reviewed relations are disposable, scope-bound data, never canonical knowledge.
use crate::{
    read_capability_optional, transactional_capability, CapabilityFileSnapshot, PinnedRoot,
    WikiError,
};
use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::fs::OpenOptions;
use hive_core::{
    ensure_consumer_target, ensure_no_symlink_ancestors, normalize_platform_root, sha256_digest,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const MAX_DOCUMENTS: usize = 10;
pub const MAX_TEXT_BYTES: usize = 16 * 1024;
const MAX_STATE_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: String,
    pub locator: String,
    pub digest: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub evidence: String,
    pub start: usize,
    pub end: usize,
    pub text_digest: String,
    pub source_digest: String,
    pub target_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub schema_version: u32,
    pub selector: String,
    pub enabled: bool,
    pub host: String,
    pub consent_digest: String,
    pub desired_digest: Option<String>,
    pub processed: BTreeMap<String, String>,
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub last_attempt_digest: Option<String>,
    #[serde(default)]
    pub attempts: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    pub schema_version: u32,
    pub request_digest: String,
    pub selector: String,
    pub host: String,
    pub partition_digest: String,
    pub authority_digest: String,
    pub documents: Vec<Document>,
    pub related_documents: Vec<Document>,
    pub pending_count: usize,
    pub blocked_oversized_count: usize,
    pub max_corrections: u8,
    pub untrusted_content: bool,
    pub next_attempt: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultInput {
    pub schema_version: u32,
    pub request_digest: String,
    pub processed_ids: Vec<String>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema_version: u32,
    pub request_digest: String,
    pub result_digest: String,
    pub host: String,
    pub reviewed: bool,
    pub attempt: u8,
}

fn invalid(message: &str) -> WikiError {
    WikiError::Verification(message.to_owned())
}

/// Canonical digest used by the host request and its bounded result.
/// # Errors
/// Rejects non-serializable input.
pub fn digest(value: &impl Serialize) -> Result<String, WikiError> {
    serde_json_canonicalizer::to_vec(value)
        .map(|v| sha256_digest(&v))
        .map_err(|_| invalid("semantic graph encoding failed"))
}

/// Files are pinned to one exact source/user root and one hashed partition.
pub struct GraphStore {
    root: PinnedRoot,
    relative: PathBuf,
    selector: String,
}

impl GraphStore {
    /// Open without creating files or enabling analysis.
    /// # Errors
    /// Rejects source/consumer confusion and unsafe filesystem roots.
    pub fn open(
        root: &Path,
        source: bool,
        collection: &str,
        visibility: &str,
    ) -> Result<Self, WikiError> {
        let root = normalize_platform_root(
            &std::path::absolute(root).map_err(|_| invalid("invalid semantic root"))?,
        );
        ensure_no_symlink_ancestors(&root, Path::new(hive_core::SOURCE_MARKER_FILE))
            .map_err(|_| invalid("unsafe semantic root"))?;
        if source {
            crate::source::validate_root(&root)?;
        } else {
            ensure_consumer_target(&root)
                .map_err(|_| invalid("source requires source-wiki graph"))?;
        }
        if collection.is_empty()
            || collection.len() > 256
            || source != (visibility == "source")
            || !matches!(
                visibility,
                "source" | "shared" | "project-private" | "confidential"
            )
        {
            return Err(invalid("invalid semantic partition"));
        }
        let pinned = PinnedRoot::open(&root)?;
        let mut root_key = pinned.canonical_path.to_string_lossy().replace('\\', "/");
        if cfg!(windows) {
            root_key.make_ascii_lowercase();
        }
        let selector = digest(&(root_key, source, collection, visibility))?;
        let base = if source {
            ".agents/work/semantic-graph"
        } else {
            ".hive/index/semantic-graph"
        };
        let relative = Path::new(base).join(format!("{}.json", &selector[7..]));
        Ok(Self {
            root: pinned,
            relative,
            selector,
        })
    }

    #[must_use]
    pub fn selector(&self) -> &str {
        &self.selector
    }

    #[must_use]
    pub fn relative(&self) -> &Path {
        &self.relative
    }

    /// Read and verify scope binding. Missing state means disabled.
    /// # Errors
    /// Rejects malformed, oversized, linked or foreign state.
    pub fn read(&self) -> Result<(Option<State>, Option<String>), WikiError> {
        let Some(bytes) = read_capability_optional(&self.root.dir, &self.relative)? else {
            return Ok((None, None));
        };
        if bytes.len() > MAX_STATE_BYTES {
            return Err(invalid("semantic state exceeds limit"));
        }
        let state: State =
            serde_json::from_slice(&bytes).map_err(|_| invalid("malformed semantic state"))?;
        if state.schema_version != 1
            || state.selector != self.selector
            || !matches!(state.host.as_str(), "codex" | "claude" | "antigravity")
        {
            return Err(invalid("semantic state binding mismatch"));
        }
        Ok((Some(state), Some(sha256_digest(&bytes))))
    }

    /// Atomically replace the complete derived generation using compare-and-swap.
    /// # Errors
    /// Preserves prior bytes on source-independent publication conflicts.
    pub fn write(&self, expected: Option<&str>, state: &State) -> Result<(), WikiError> {
        let _writer = self.writer()?;
        if state.selector != self.selector || state.schema_version != 1 {
            return Err(invalid("semantic state scope mismatch"));
        }
        let actual = self.read()?.1;
        if actual.as_deref() != expected {
            return Err(invalid("semantic state changed"));
        }
        let bytes = serde_json_canonicalizer::to_vec(state)
            .map_err(|_| invalid("cannot encode semantic state"))?;
        if bytes.len() > MAX_STATE_BYTES {
            return Err(invalid("semantic state exceeds limit"));
        }
        let mut snapshots = [CapabilityFileSnapshot::capture(
            &self.root.dir,
            &self.relative,
        )?];
        let captured = match &snapshots[0].current {
            crate::CapabilityFileState::Missing => None,
            crate::CapabilityFileState::File { bytes, .. } => Some(sha256_digest(bytes)),
        };
        if captured.as_deref() != expected {
            return Err(invalid("semantic state changed during capture"));
        }
        transactional_capability(&self.root.dir, &mut snapshots, |snapshots| {
            snapshots[0].install_staged(&self.root.dir, &bytes)?;
            Ok(())
        })
    }

    fn writer(&self) -> Result<std::fs::File, WikiError> {
        let lock = self.relative.with_extension("lock");
        let (parent, name) = crate::capability_parent(&self.root.dir, &lock, true)?
            .ok_or_else(|| invalid("semantic writer directory is missing"))?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .follow(FollowSymlinks::No);
        let file = parent
            .open_with(&name, &options)
            .map_err(|_| invalid("cannot open semantic writer lock"))?;
        let metadata = file
            .metadata()
            .map_err(|_| invalid("cannot inspect semantic writer lock"))?;
        if !metadata.is_file() || metadata.nlink() != 1 {
            return Err(invalid("semantic writer lock is not private"));
        }
        let file = file.into_std();
        file.try_lock()
            .map_err(|_| WikiError::Conflict("semantic writer is busy".to_owned()))?;
        Ok(file)
    }
}

/// Prepare only changed authorized documents, with deterministic byte and count limits.
/// # Errors
/// Rejects duplicate identities, disabled state and malformed content fingerprints.
pub fn prepare(
    state: &State,
    documents: &[Document],
    partition: &str,
    authority: &str,
) -> Result<Batch, WikiError> {
    if !state.enabled {
        return Err(invalid("semantic analysis is disabled"));
    }
    let mut sorted = BTreeMap::new();
    for document in documents {
        if document.id.is_empty()
            || document.digest.len() != 71
            || !document.digest.starts_with("sha256:")
            || sorted.insert(document.id.as_str(), document).is_some()
        {
            return Err(invalid("invalid semantic document identity"));
        }
    }
    let mut selected = Vec::new();
    let mut bytes = 0;
    let mut pending = 0;
    let mut oversized = 0;
    let invalidated = invalidated_sources(state, &sorted);
    for (id, doc) in &sorted {
        if state.processed.get(*id) == Some(&doc.digest) && !invalidated.contains(*id) {
            continue;
        }
        pending += 1;
        if doc.text.len() > MAX_TEXT_BYTES {
            oversized += 1;
            continue;
        }
        if selected.len() < MAX_DOCUMENTS && bytes + doc.text.len() <= MAX_TEXT_BYTES {
            selected.push((*doc).clone());
            bytes += doc.text.len();
        }
    }
    // Context is drawn only from the already-authorized partition. Never expose
    // the full catalog or spend more than the shared text budget.
    let selected_ids = selected
        .iter()
        .map(|d| d.id.as_str())
        .collect::<BTreeSet<_>>();
    let terms = |text: &str| {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() >= 3)
            .map(str::to_lowercase)
            .collect::<BTreeSet<_>>()
    };
    let selected_terms = selected
        .iter()
        .flat_map(|d| terms(&d.text))
        .collect::<BTreeSet<_>>();
    let mut candidates = sorted
        .values()
        .filter(|d| !selected_ids.contains(d.id.as_str()))
        .map(|d| (terms(&d.text).intersection(&selected_terms).count(), *d))
        .filter(|(score, _)| *score > 0)
        .collect::<Vec<_>>();
    candidates.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut related_documents = Vec::new();
    for (_, doc) in candidates {
        if related_documents.len() < MAX_DOCUMENTS && bytes + doc.text.len() <= MAX_TEXT_BYTES {
            bytes += doc.text.len();
            related_documents.push(doc.clone());
        }
    }
    let request_digest = digest(&(
        &state.selector,
        state.enabled,
        &state.host,
        &state.consent_digest,
        &state.desired_digest,
        &state.processed,
        &state.relations,
        partition,
        authority,
        &selected,
        &related_documents,
    ))?;
    let next_attempt = if state.last_attempt_digest.as_deref() == Some(&request_digest) {
        state.attempts.saturating_add(1)
    } else {
        1
    };
    Ok(Batch {
        schema_version: 1,
        request_digest,
        selector: state.selector.clone(),
        host: state.host.clone(),
        partition_digest: partition.to_owned(),
        authority_digest: authority.to_owned(),
        documents: selected,
        related_documents,
        pending_count: pending,
        blocked_oversized_count: oversized,
        max_corrections: 1,
        untrusted_content: true,
        next_attempt,
    })
}

fn invalidated_sources(state: &State, current: &BTreeMap<&str, &Document>) -> BTreeSet<String> {
    state
        .relations
        .iter()
        .filter(|edge| {
            current
                .get(edge.from.as_str())
                .is_none_or(|d| d.digest != edge.source_digest)
                || current
                    .get(edge.to.as_str())
                    .is_none_or(|d| d.digest != edge.target_digest)
        })
        .map(|edge| edge.from.clone())
        .collect()
}

fn validate_relations(
    relations: &[Relation],
    selected: &BTreeSet<String>,
    current: &BTreeMap<&str, &Document>,
) -> Result<(), WikiError> {
    for edge in relations {
        if !selected.contains(&edge.from)
            || edge.from == edge.to
            || !matches!(
                edge.kind.as_str(),
                "related" | "supports" | "contradicts" | "depends-on"
            )
            || !matches!(edge.evidence.as_str(), "EXTRACTED" | "INFERRED")
        {
            return Err(invalid("invalid semantic relation"));
        }
        let from = current
            .get(edge.from.as_str())
            .ok_or_else(|| invalid("semantic source unavailable"))?;
        let to = current
            .get(edge.to.as_str())
            .ok_or_else(|| invalid("semantic target unavailable"))?;
        let span = from
            .text
            .get(edge.start..edge.end)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| invalid("invalid semantic evidence range"))?;
        if edge.source_digest != from.digest
            || edge.target_digest != to.digest
            || sha256_digest(span.as_bytes()) != edge.text_digest
        {
            return Err(invalid("semantic evidence changed"));
        }
        if edge.evidence == "EXTRACTED"
            && (edge.kind != "related"
                || (!span.contains(&format!("[[{}]]", edge.to))
                    && !span.contains(&format!("]({})", to.locator))))
        {
            return Err(invalid("extracted relation has no explicit source link"));
        }
    }
    Ok(())
}

/// Validate host results against current authorized source bytes before publication.
/// # Errors
/// Rejects stale input, false extracted evidence, missing review and wrong result bindings.
pub fn apply(
    state: &State,
    documents: &[Document],
    batch: &Batch,
    result: &ResultInput,
    receipt: &Receipt,
) -> Result<State, WikiError> {
    if !state.enabled
        || result.schema_version != 1
        || receipt.schema_version != 1
        || result.request_digest != batch.request_digest
        || receipt.request_digest != batch.request_digest
        || receipt.host != state.host
        || !receipt.reviewed
        || !(1..=2).contains(&receipt.attempt)
        || receipt.attempt != batch.next_attempt
        || receipt.result_digest != digest(result)?
    {
        return Err(invalid("semantic result or review binding mismatch"));
    }
    let expected = prepare(
        state,
        documents,
        &batch.partition_digest,
        &batch.authority_digest,
    )?;
    if expected.request_digest != batch.request_digest
        || expected.documents != batch.documents
        || expected.related_documents != batch.related_documents
    {
        return Err(invalid("semantic preparation is stale"));
    }
    let selected = batch
        .documents
        .iter()
        .map(|d| d.id.clone())
        .collect::<BTreeSet<_>>();
    let processed = result
        .processed_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if selected != processed
        || result.processed_ids.len() != processed.len()
        || result.relations.len() > 100
    {
        return Err(invalid(
            "semantic result omitted or duplicated processed documents",
        ));
    }
    let current = documents
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect::<BTreeMap<_, _>>();
    let disclosed = batch
        .documents
        .iter()
        .chain(&batch.related_documents)
        .map(|d| d.id.as_str())
        .collect::<BTreeSet<_>>();
    if result
        .relations
        .iter()
        .any(|edge| !disclosed.contains(edge.to.as_str()))
    {
        return Err(invalid(
            "semantic target was not included in the prepared input",
        ));
    }
    validate_relations(&result.relations, &selected, &current)?;
    let mut next = state.clone();
    next.last_attempt_digest = None;
    next.attempts = 0;
    let invalidated = invalidated_sources(state, &current);
    next.processed.retain(|id, digest| {
        !invalidated.contains(id)
            && current
                .get(id.as_str())
                .is_some_and(|doc| &doc.digest == digest)
    });
    next.relations.retain(|edge| {
        !selected.contains(&edge.from)
            && current
                .get(edge.from.as_str())
                .is_some_and(|d| d.digest == edge.source_digest)
            && current
                .get(edge.to.as_str())
                .is_some_and(|d| d.digest == edge.target_digest)
    });
    for doc in &batch.documents {
        next.processed.insert(doc.id.clone(), doc.digest.clone());
    }
    next.relations.extend(result.relations.clone());
    next.relations.sort();
    next.relations.dedup();
    next.desired_digest = if next.processed.len() == current.len() {
        None
    } else {
        Some(batch.partition_digest.clone())
    };
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn docs() -> Vec<Document> {
        vec![
            Document {
                id: "a".into(),
                locator: "a.md".into(),
                digest: sha256_digest(b"[[b]]"),
                text: "[[b]]".into(),
            },
            Document {
                id: "b".into(),
                locator: "b.md".into(),
                digest: sha256_digest(b"target"),
                text: "target".into(),
            },
        ]
    }
    fn state() -> State {
        State {
            schema_version: 1,
            selector: sha256_digest(b"scope"),
            enabled: true,
            host: "codex".into(),
            consent_digest: sha256_digest(b"consent"),
            desired_digest: None,
            processed: BTreeMap::new(),
            relations: vec![],
            last_attempt_digest: None,
            attempts: 0,
        }
    }
    fn result(batch: &Batch, docs: &[Document]) -> (ResultInput, Receipt) {
        let result = ResultInput {
            schema_version: 1,
            request_digest: batch.request_digest.clone(),
            processed_ids: batch.documents.iter().map(|d| d.id.clone()).collect(),
            relations: vec![Relation {
                from: "a".into(),
                to: "b".into(),
                kind: "related".into(),
                evidence: "EXTRACTED".into(),
                start: 0,
                end: 5,
                text_digest: sha256_digest(b"[[b]]"),
                source_digest: docs[0].digest.clone(),
                target_digest: docs[1].digest.clone(),
            }],
        };
        let receipt = Receipt {
            schema_version: 1,
            request_digest: batch.request_digest.clone(),
            result_digest: digest(&result).unwrap(),
            host: "codex".into(),
            reviewed: true,
            attempt: 1,
        };
        (result, receipt)
    }
    #[test]
    fn unchanged_documents_are_not_reanalysed_and_deleted_relations_disappear() {
        let state = state();
        let docs = docs();
        let batch = prepare(&state, &docs, "p", "a").unwrap();
        let (r, receipt) = result(&batch, &docs);
        let state = apply(&state, &docs, &batch, &r, &receipt).unwrap();
        assert_eq!(state.relations.len(), 1);
        assert!(prepare(&state, &docs, "p", "a")
            .unwrap()
            .documents
            .is_empty());
        let batch = prepare(&state, &docs[..1], "next", "a").unwrap();
        let r = ResultInput {
            schema_version: 1,
            request_digest: batch.request_digest.clone(),
            processed_ids: batch.documents.iter().map(|doc| doc.id.clone()).collect(),
            relations: vec![],
        };
        let receipt = Receipt {
            schema_version: 1,
            request_digest: batch.request_digest.clone(),
            result_digest: digest(&r).unwrap(),
            host: "codex".into(),
            reviewed: true,
            attempt: 1,
        };
        assert!(apply(&state, &docs[..1], &batch, &r, &receipt)
            .unwrap()
            .relations
            .is_empty());
    }
    #[test]
    fn refuses_stale_cross_scope_and_false_grounding() {
        let state = state();
        let docs = docs();
        let batch = prepare(&state, &docs, "p", "a").unwrap();
        let (mut r, mut receipt) = result(&batch, &docs);
        r.relations[0].to = "foreign".into();
        receipt.result_digest = digest(&r).unwrap();
        assert!(apply(&state, &docs, &batch, &r, &receipt).is_err());
        let (mut r, mut receipt) = result(&batch, &docs);
        r.relations[0].text_digest = sha256_digest(b"wrong");
        receipt.result_digest = digest(&r).unwrap();
        assert!(apply(&state, &docs, &batch, &r, &receipt).is_err());
        let (r, receipt) = result(&batch, &docs);
        let mut changed = docs.clone();
        changed[0].digest = sha256_digest(b"changed");
        assert!(apply(&state, &changed, &batch, &r, &receipt).is_err());
    }

    #[test]
    fn changed_targets_requeue_incoming_sources_before_pruning() {
        let state = state();
        let mut documents = docs();
        let batch = prepare(&state, &documents, "p", "a").unwrap();
        let (r, receipt) = result(&batch, &documents);
        let state = apply(&state, &documents, &batch, &r, &receipt).unwrap();
        documents[1].text = "changed target".to_owned();
        documents[1].digest = sha256_digest(documents[1].text.as_bytes());
        let batch = prepare(&state, &documents, "changed", "a").unwrap();
        assert_eq!(
            batch
                .documents
                .iter()
                .map(|d| d.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        let (r, receipt) = result(&batch, &documents);
        let incremental = apply(&state, &documents, &batch, &r, &receipt).unwrap();
        let full_state = super::tests::state();
        let batch = prepare(&full_state, &documents, "changed", "a").unwrap();
        let (r, receipt) = result(&batch, &documents);
        let full = apply(&full_state, &documents, &batch, &r, &receipt).unwrap();
        assert_eq!(incremental.relations, full.relations);
        assert_eq!(incremental.processed, full.processed);
    }
    #[test]
    fn batches_are_bounded_without_starving_small_documents() {
        let mut documents = (0..30)
            .map(|i| Document {
                id: format!("{i:03}"),
                locator: format!("{i}.md"),
                digest: sha256_digest(b"x"),
                text: "x".repeat(2000),
            })
            .collect::<Vec<_>>();
        documents[0].text = "x".repeat(MAX_TEXT_BYTES + 1);
        let b = prepare(&state(), &documents, "p", "a").unwrap();
        assert_eq!(b.blocked_oversized_count, 1);
        assert!(b.documents.len() <= 10);
        assert!(b.documents.iter().map(|d| d.text.len()).sum::<usize>() <= MAX_TEXT_BYTES);
        assert!(!b.documents.is_empty());
    }
    #[test]
    fn related_context_is_bounded_and_undisclosed_targets_are_refused() {
        let mut state = state();
        let mut documents = docs();
        documents[0].text = "[[b]] target".into();
        documents[0].digest = sha256_digest(documents[0].text.as_bytes());
        state
            .processed
            .insert("b".into(), documents[1].digest.clone());
        documents.push(Document {
            id: "unrelated".into(),
            locator: "unrelated.md".into(),
            digest: sha256_digest(b"separate"),
            text: "separate".into(),
        });
        state
            .processed
            .insert("unrelated".into(), documents[2].digest.clone());
        let batch = prepare(&state, &documents, "p", "a").unwrap();
        assert_eq!(batch.documents.len(), 1);
        assert_eq!(batch.related_documents.len(), 1);
        assert_eq!(batch.related_documents[0].id, "b");
        assert!(
            batch
                .documents
                .iter()
                .chain(&batch.related_documents)
                .map(|d| d.text.len())
                .sum::<usize>()
                <= MAX_TEXT_BYTES
        );
        let (mut r, mut receipt) = result(&batch, &documents);
        assert!(apply(&state, &documents, &batch, &r, &receipt).is_ok());
        r.relations[0].to = "unrelated".into();
        r.relations[0].target_digest = documents[2].digest.clone();
        r.relations[0].evidence = "INFERRED".into();
        receipt.result_digest = digest(&r).unwrap();
        assert!(apply(&state, &documents, &batch, &r, &receipt).is_err());
        let mut forged = batch.clone();
        forged.related_documents.push(documents[2].clone());
        assert!(apply(&state, &documents, &forged, &r, &receipt).is_err());
    }
    #[test]
    fn scope_storage_is_atomic_and_preserves_prior_state_on_conflict() {
        let root = tempfile::tempdir().unwrap();
        let store = GraphStore::open(root.path(), false, "user-root", "shared").unwrap();
        let mut s = state();
        s.selector = store.selector().into();
        store.write(None, &s).unwrap();
        let (_, fingerprint) = store.read().unwrap();
        s.enabled = false;
        assert!(store.write(None, &s).is_err());
        assert!(store.read().unwrap().0.unwrap().enabled);
        store.write(fingerprint.as_deref(), &s).unwrap();
        assert!(!store.read().unwrap().0.unwrap().enabled);
        let private = GraphStore::open(root.path(), false, "user-root", "project-private").unwrap();
        assert!(private.read().unwrap().0.is_none());
        let held = store.writer().unwrap();
        assert!(store.write(store.read().unwrap().1.as_deref(), &s).is_err());
        drop(held);
    }
}
