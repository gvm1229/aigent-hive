//! Deterministic derived relations from canonical Markdown Wiki pages.

use crate::WikiPage;
use hive_core::{ensure_no_symlink_ancestors, sha256_digest};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// One relationship evidence class. Only extracted relations originate in Markdown parsing.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GraphEvidence {
    Extracted,
    Inferred,
    Ambiguous,
}

/// One deterministic Markdown relationship edge.
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub evidence: GraphEvidence,
    pub source_digest: String,
}

/// One scope-isolated, disposable native Markdown graph generation.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphGeneration {
    pub schema_version: u32,
    pub scope: String,
    pub engine: String,
    pub generation_digest: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Canonical node metadata without Markdown body duplication.
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNode {
    pub id: String,
    pub locator: String,
    pub content_digest: String,
    pub visibility: String,
    pub lifecycle: String,
}

/// One verified pointer to an active disposable graph generation.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveGraphPointer {
    pub schema_version: u32,
    pub scope: String,
    pub engine: String,
    pub generation_digest: String,
    pub generation_path: String,
    pub source_digest: String,
    pub extractor_receipt_digest: Option<String>,
}

/// Build a deterministic native graph for exactly one already-authorized scope.
pub fn build_native_generation(scope: &str, pages: &[WikiPage]) -> Result<GraphGeneration, String> {
    if !matches!(
        scope,
        "source" | "project" | "user-root" | "shared" | "private" | "confidential"
    ) {
        return Err("knowledge graph scope is unsupported".to_owned());
    }
    let mut nodes = pages
        .iter()
        .filter(|page| graph_lifecycle(&page.frontmatter.status))
        .map(|page| GraphNode {
            id: page.frontmatter.id.clone(),
            locator: page.relative_path.clone(),
            content_digest: page.content_digest.clone(),
            visibility: scope.to_owned(),
            lifecycle: page.frontmatter.status.clone(),
        })
        .collect::<Vec<_>>();
    nodes.sort();
    nodes.dedup();
    let eligible = pages
        .iter()
        .filter(|page| graph_lifecycle(&page.frontmatter.status))
        .cloned()
        .collect::<Vec<_>>();
    let edges = extract_markdown_edges(&eligible);
    finalize_generation(scope, "native-markdown", nodes, edges)
}

/// Rebuild a native graph by reusing only edges whose canonical page digest is unchanged.
///
/// # Errors
///
/// Returns an error when the prior generation has a different scope or engine, or when the
/// resulting generation cannot be canonicalized.
pub fn build_native_generation_incremental(
    scope: &str,
    prior: &GraphGeneration,
    pages: &[WikiPage],
) -> Result<GraphGeneration, String> {
    if prior.scope != scope || prior.engine != "native-markdown" {
        return Err("prior native graph generation binding mismatch".to_owned());
    }
    let prior_nodes = prior
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for page in pages {
        if !graph_lifecycle(&page.frontmatter.status) {
            continue;
        }
        let unchanged = prior_nodes
            .get(page.frontmatter.id.as_str())
            .is_some_and(|node| node.content_digest == page.content_digest);
        nodes.push(GraphNode {
            id: page.frontmatter.id.clone(),
            locator: page.relative_path.clone(),
            content_digest: page.content_digest.clone(),
            visibility: scope.to_owned(),
            lifecycle: page.frontmatter.status.clone(),
        });
        if unchanged {
            edges.extend(
                prior
                    .edges
                    .iter()
                    .filter(|edge| edge.from == page.frontmatter.id)
                    .cloned(),
            );
        } else {
            edges.extend(extract_markdown_edges(std::slice::from_ref(page)));
        }
    }
    nodes.sort();
    nodes.dedup();
    edges.sort();
    edges.dedup();
    finalize_generation(scope, "native-markdown", nodes, edges)
}

fn graph_lifecycle(value: &str) -> bool {
    matches!(
        value,
        "active" | "contradicted" | "superseded" | "expired" | "revoked"
    )
}

fn finalize_generation(
    scope: &str,
    engine: &str,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
) -> Result<GraphGeneration, String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "scope": scope,
        "engine": engine,
        "nodes": &nodes,
        "edges": &edges,
    });
    let generation_digest = sha256_digest(
        &serde_json_canonicalizer::to_vec(&payload)
            .map_err(|error| format!("canonicalize knowledge graph: {error}"))?,
    );
    Ok(GraphGeneration {
        schema_version: 1,
        scope: scope.to_owned(),
        engine: engine.to_owned(),
        generation_digest,
        nodes,
        edges,
    })
}

/// Return the Hive-owned derived path for one physical graph scope.
pub fn generation_relative_path(scope: &str, scope_digest: &str) -> Result<PathBuf, String> {
    let digest = scope_digest
        .strip_prefix("sha256:")
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| "graph scope digest is invalid".to_owned())?;
    let root = match scope {
        "source" => PathBuf::from(".agents/work/graph/source"),
        "project" => PathBuf::from(".hive/index/graph/project"),
        "user-root" => PathBuf::from(".hive/index/graph/user-root"),
        "shared" => PathBuf::from(".hive/index/graph/shared"),
        "private" => PathBuf::from(".hive/index/graph/private"),
        "confidential" => PathBuf::from(".hive/index/graph/confidential"),
        _ => return Err("knowledge graph scope is unsupported".to_owned()),
    };
    Ok(root.join(digest).join("generation.json"))
}

/// Persist one digest-addressed generation without overwriting a different derived graph.
pub fn persist_generation(target: &Path, generation: &GraphGeneration) -> Result<PathBuf, String> {
    let relative = generation_relative_path(&generation.scope, &generation.generation_digest)?;
    ensure_graph_path(target, &relative, &generation.scope)?;
    let destination = target.join(&relative);
    let parent = destination
        .parent()
        .ok_or_else(|| "graph generation has no parent".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| format!("create graph parent: {error}"))?;
    let bytes = serde_json_canonicalizer::to_vec(generation)
        .map_err(|error| format!("canonicalize graph generation: {error}"))?;
    match std::fs::symlink_metadata(&destination) {
        Ok(metadata) if !metadata.is_file() => {
            return Err("graph generation destination is not a regular file".to_owned());
        }
        Ok(_) => {
            let existing = std::fs::read(&destination)
                .map_err(|error| format!("read graph generation: {error}"))?;
            if existing != bytes {
                return Err(
                    "graph generation digest path is occupied by different bytes".to_owned(),
                );
            }
            return Ok(relative);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("inspect graph generation: {error}")),
    }
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| format!("create graph generation staging: {error}"))?;
    temporary
        .write_all(&bytes)
        .map_err(|error| format!("write graph generation staging: {error}"))?;
    temporary
        .persist(&destination)
        .map_err(|error| format!("activate graph generation: {error}"))?;
    Ok(relative)
}

/// Persist and atomically activate one verified graph generation.
///
/// # Errors
///
/// Returns an error when the generation, source digest, receipt digest, path, or active pointer
/// violates the graph contract or cannot be written safely.
pub fn activate_generation(
    target: &Path,
    generation: &GraphGeneration,
    source_digest: &str,
    extractor_receipt_digest: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    validate_digest(source_digest, "graph source")?;
    if let Some(digest) = extractor_receipt_digest {
        validate_digest(digest, "graph extractor receipt")?;
    }
    let generation_path = persist_generation(target, generation)?;
    let pointer_path = active_pointer_relative_path(&generation.scope, &generation.engine)?;
    ensure_graph_path(target, &pointer_path, &generation.scope)?;
    let destination = target.join(&pointer_path);
    let parent = destination
        .parent()
        .ok_or_else(|| "graph active pointer has no parent".to_owned())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("create graph pointer parent: {error}"))?;
    let pointer = ActiveGraphPointer {
        schema_version: 1,
        scope: generation.scope.clone(),
        engine: generation.engine.clone(),
        generation_digest: generation.generation_digest.clone(),
        generation_path: generation_path.to_string_lossy().replace('\\', "/"),
        source_digest: source_digest.to_owned(),
        extractor_receipt_digest: extractor_receipt_digest.map(str::to_owned),
    };
    let bytes = serde_json_canonicalizer::to_vec(&pointer)
        .map_err(|error| format!("canonicalize graph active pointer: {error}"))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| format!("create graph pointer staging: {error}"))?;
    temporary
        .write_all(&bytes)
        .map_err(|error| format!("write graph pointer staging: {error}"))?;
    temporary
        .persist(&destination)
        .map_err(|error| format!("activate graph pointer: {error}"))?;
    Ok(vec![generation_path, pointer_path])
}

/// Load the active generation only after pointer, path, scope, engine, and digest verification.
///
/// # Errors
///
/// Returns an error for missing, malformed, unsafe, stale, or mismatched active derived state.
pub fn load_active_generation(
    target: &Path,
    scope: &str,
    engine: &str,
) -> Result<(ActiveGraphPointer, GraphGeneration), String> {
    let pointer_path = active_pointer_relative_path(scope, engine)?;
    ensure_graph_path(target, &pointer_path, scope)?;
    let bytes = std::fs::read(target.join(&pointer_path))
        .map_err(|error| format!("read graph active pointer: {error}"))?;
    if bytes.len() > 64 * 1024 {
        return Err("graph active pointer is oversized".to_owned());
    }
    let pointer: ActiveGraphPointer = serde_json::from_slice(&bytes)
        .map_err(|error| format!("graph active pointer is malformed: {error}"))?;
    if pointer.schema_version != 1 || pointer.scope != scope || pointer.engine != engine {
        return Err("graph active pointer binding mismatch".to_owned());
    }
    let expected = generation_relative_path(scope, &pointer.generation_digest)?;
    if pointer.generation_path != expected.to_string_lossy().replace('\\', "/") {
        return Err("graph active generation path mismatch".to_owned());
    }
    ensure_graph_path(target, &expected, scope)?;
    let generation_bytes = std::fs::read(target.join(&expected))
        .map_err(|error| format!("read active graph generation: {error}"))?;
    let generation: GraphGeneration = serde_json::from_slice(&generation_bytes)
        .map_err(|error| format!("active graph generation is malformed: {error}"))?;
    if generation.scope != scope
        || generation.engine != engine
        || generation.generation_digest != pointer.generation_digest
    {
        return Err("active graph generation binding mismatch".to_owned());
    }
    Ok((pointer, generation))
}

/// Remove one active engine pointer and its exact derived generation.
///
/// # Errors
///
/// Returns an error when active state is malformed, unsafe, or cannot be removed.
pub fn remove_active_generation(
    target: &Path,
    scope: &str,
    engine: &str,
) -> Result<Vec<PathBuf>, String> {
    let pointer_path = active_pointer_relative_path(scope, engine)?;
    match std::fs::symlink_metadata(target.join(&pointer_path)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("inspect graph active pointer: {error}")),
        Ok(metadata) if !metadata.is_file() => {
            return Err("graph active pointer is not a regular file".to_owned());
        }
        Ok(_) => {}
    }
    let (pointer, _) = load_active_generation(target, scope, engine)?;
    let mut changed = Vec::new();
    if let Some(generation) = remove_generation(target, scope, &pointer.generation_digest)? {
        changed.push(generation);
    }
    std::fs::remove_file(target.join(&pointer_path))
        .map_err(|error| format!("remove graph active pointer: {error}"))?;
    changed.push(pointer_path);
    Ok(changed)
}

fn active_pointer_relative_path(scope: &str, engine: &str) -> Result<PathBuf, String> {
    if !matches!(engine, "native-markdown" | "graphify-code") {
        return Err("knowledge graph engine is unsupported".to_owned());
    }
    let generation = generation_relative_path(scope, &format!("sha256:{}", "0".repeat(64)))?;
    Ok(generation
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "knowledge graph scope root is invalid".to_owned())?
        .join(format!("active-{engine}.json")))
}

fn ensure_graph_path(target: &Path, relative: &Path, scope: &str) -> Result<(), String> {
    if scope != "source" {
        return ensure_no_symlink_ancestors(target, relative)
            .map_err(|error| format!("graph path is unsafe: {error}"));
    }
    let marker = target.join("hive-source.json");
    let marker_metadata = std::fs::symlink_metadata(&marker)
        .map_err(|error| format!("source graph requires hive-source.json: {error}"))?;
    if marker_metadata.file_type().is_symlink() || !marker_metadata.is_file() {
        return Err("source graph marker is not a regular file".to_owned());
    }
    let mut current = target.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            return Err("source graph path is not normalized".to_owned());
        };
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("source graph path contains a symlink".to_owned());
            }
            Ok(metadata) if current != target.join(relative) && !metadata.is_dir() => {
                return Err("source graph path ancestor is not a directory".to_owned());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("inspect source graph path: {error}")),
        }
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<(), String> {
    let valid = value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    if valid {
        Ok(())
    } else {
        Err(format!("{label} digest is invalid"))
    }
}

/// Remove exactly one digest-addressed derived generation without touching canonical knowledge.
pub fn remove_generation(
    target: &Path,
    scope: &str,
    generation_digest: &str,
) -> Result<Option<PathBuf>, String> {
    let relative = generation_relative_path(scope, generation_digest)?;
    ensure_graph_path(target, &relative, scope)?;
    let destination = target.join(&relative);
    match std::fs::symlink_metadata(&destination) {
        Ok(metadata) if metadata.is_file() => {
            std::fs::remove_file(&destination)
                .map_err(|error| format!("remove graph generation: {error}"))?;
            Ok(Some(relative))
        }
        Ok(_) => Err("graph generation destination is not a regular file".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("inspect graph generation: {error}")),
    }
}

/// Return bounded direct relationships for one node without exposing page bodies.
#[must_use]
pub fn query_generation(
    generation: &GraphGeneration,
    node_id: &str,
    limit: usize,
) -> Vec<GraphEdge> {
    let active = generation
        .nodes
        .iter()
        .filter(|node| node.lifecycle == "active")
        .map(|node| node.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    generation
        .edges
        .iter()
        .filter(|edge| {
            (edge.from == node_id || edge.to == node_id)
                && active.contains(edge.from.as_str())
                && (!generation.nodes.iter().any(|node| node.id == edge.to)
                    || active.contains(edge.to.as_str()))
        })
        .take(limit.min(50))
        .cloned()
        .collect()
}

/// Return at most ten metadata-only nodes for one relationship neighborhood.
#[must_use]
pub fn query_node_metadata(
    generation: &GraphGeneration,
    node_id: &str,
    limit: usize,
) -> Vec<GraphNode> {
    let mut ids = std::collections::BTreeSet::from([node_id.to_owned()]);
    for edge in query_generation(generation, node_id, 50) {
        ids.insert(edge.from);
        ids.insert(edge.to);
    }
    generation
        .nodes
        .iter()
        .filter(|node| node.lifecycle == "active" && ids.contains(&node.id))
        .take(limit.min(10))
        .cloned()
        .collect()
}

/// Write one digest-addressed JSON or self-contained HTML graph export.
///
/// # Errors
///
/// Returns an error for an unsupported format, unsafe path, serialization failure, or write
/// failure. Canonical Markdown and active graph state remain unchanged.
pub fn export_generation(
    target: &Path,
    generation: &GraphGeneration,
    format: &str,
) -> Result<PathBuf, String> {
    if !matches!(format, "json" | "html") {
        return Err("knowledge graph export format is unsupported".to_owned());
    }
    let digest = generation
        .generation_digest
        .strip_prefix("sha256:")
        .and_then(|value| value.get(..16))
        .ok_or_else(|| "knowledge graph generation digest is invalid".to_owned())?;
    let export_root = if generation.scope == "source" {
        PathBuf::from(".agents/work/graph/exports")
    } else {
        PathBuf::from(".hive/exports/knowledge-graph")
    };
    let relative = export_root.join(format!(
        "{}-{}-{}.{}",
        generation.scope, generation.engine, digest, format
    ));
    ensure_graph_path(target, &relative, &generation.scope)?;
    let destination = target.join(&relative);
    let parent = destination
        .parent()
        .ok_or_else(|| "knowledge graph export has no parent".to_owned())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("create graph export parent: {error}"))?;
    let json = serde_json::to_vec(generation)
        .map_err(|error| format!("serialize knowledge graph export: {error}"))?;
    let bytes = if format == "json" {
        json
    } else {
        let escaped = String::from_utf8(json)
            .map_err(|error| format!("encode knowledge graph HTML: {error}"))?
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        format!(
            "<!doctype html><meta charset=\"utf-8\"><title>Aigent Hive knowledge graph</title><pre id=\"graph\">{escaped}</pre>"
        )
        .into_bytes()
    };
    match std::fs::symlink_metadata(&destination) {
        Ok(metadata) if metadata.is_file() => {
            let existing = std::fs::read(&destination)
                .map_err(|error| format!("read knowledge graph export: {error}"))?;
            if existing == bytes {
                return Ok(relative);
            }
            return Err("knowledge graph export digest path has different bytes".to_owned());
        }
        Ok(_) => return Err("knowledge graph export destination is not a regular file".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("inspect knowledge graph export: {error}")),
    }
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| format!("create knowledge graph export staging: {error}"))?;
    temporary
        .write_all(&bytes)
        .map_err(|error| format!("write knowledge graph export staging: {error}"))?;
    temporary
        .persist(&destination)
        .map_err(|error| format!("activate knowledge graph export: {error}"))?;
    Ok(relative)
}

/// Extract direct links, source links, tags, and contradiction references in stable order.
#[must_use]
pub fn extract_markdown_edges(pages: &[WikiPage]) -> Vec<GraphEdge> {
    let mut edges = Vec::new();
    for page in pages {
        let from = page.frontmatter.id.clone();
        let digest = page.content_digest.clone();
        for target in &page.frontmatter.links {
            edges.push(edge(&from, target, "links", &digest));
        }
        for source in &page.frontmatter.sources {
            edges.push(edge(&from, source, "sources", &digest));
        }
        for source in &page.frontmatter.source_links {
            edges.push(edge(&from, source, "source_links", &digest));
        }
        for tag in &page.frontmatter.tags {
            edges.push(edge(&from, tag, "tags", &digest));
        }
        for related in &page.frontmatter.related_concepts {
            edges.push(edge(&from, related, "related_concepts", &digest));
        }
        for topic in &page.frontmatter.topics {
            edges.push(edge(&from, topic, "topics", &digest));
        }
        if let Some(duplicate) = &page.frontmatter.duplicate_of {
            edges.push(edge(&from, duplicate, "duplicate_of", &digest));
        }
        if let Some(replacement) = &page.frontmatter.replacement {
            edges.push(edge(&from, replacement, "replacement", &digest));
        }
        for contradiction in &page.frontmatter.contradictions {
            let target = format!(
                "contradiction:{}",
                sha256_digest(contradiction.summary.as_bytes())
            );
            edges.push(edge(&from, &target, "contradictions", &digest));
        }
    }
    edges.sort();
    edges.dedup();
    edges
}

fn edge(from: &str, to: &str, relation: &str, source_digest: &str) -> GraphEdge {
    GraphEdge {
        from: from.to_owned(),
        to: to.to_owned(),
        relation: relation.to_owned(),
        evidence: GraphEvidence::Extracted,
        source_digest: source_digest.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        activate_generation, build_native_generation, build_native_generation_incremental,
        export_generation, extract_markdown_edges, generation_relative_path,
        load_active_generation, persist_generation, query_generation, query_node_metadata,
        remove_active_generation, remove_generation, GraphEvidence,
    };
    use crate::{Contradiction, WikiFrontmatter, WikiPage};

    fn page(id: &str) -> WikiPage {
        WikiPage {
            frontmatter: WikiFrontmatter {
                schema_version: 1,
                id: id.to_owned(),
                kind: "concept".to_owned(),
                summary: "summary".to_owned(),
                tags: vec!["tag".to_owned()],
                aliases: Vec::new(),
                sources: vec!["source:one".to_owned()],
                source_links: vec!["source:two".to_owned()],
                links: vec!["other".to_owned()],
                related_concepts: vec!["related".to_owned()],
                duplicate_of: Some("duplicate".to_owned()),
                topics: vec!["topic".to_owned()],
                replacement: Some("replacement".to_owned()),
                contradictions: vec![Contradiction {
                    source_a: "source:one".to_owned(),
                    source_b: "source:two".to_owned(),
                    summary: "conflict".to_owned(),
                }],
                status: "active".to_owned(),
                created_at: "2026-08-23T00:00:00Z".to_owned(),
                updated_at: "2026-08-23T00:00:00Z".to_owned(),
            },
            body: "body".to_owned(),
            relative_path: format!(".hive/knowledge/Wiki/{id}.md"),
            content_digest: format!("sha256:{}", "a1".repeat(32)),
        }
    }

    #[test]
    fn extraction_is_stable_and_marks_only_direct_evidence() {
        let first = extract_markdown_edges(&[page("first")]);
        let second = extract_markdown_edges(&[page("first")]);
        assert_eq!(first, second);
        assert_eq!(first.len(), 9);
        assert!(first
            .iter()
            .all(|edge| edge.evidence == GraphEvidence::Extracted));
    }

    #[test]
    fn generation_is_deterministic_and_scope_isolated() {
        let first = build_native_generation("project", &[page("first")]).expect("generation");
        let second = build_native_generation("project", &[page("first")]).expect("generation");
        assert_eq!(first, second);
        assert!(first.nodes.iter().all(|node| node.visibility == "project"));
        assert!(build_native_generation("outside", &[page("first")]).is_err());
    }

    #[test]
    fn incremental_and_full_rebuilds_are_equivalent_for_change_and_delete() {
        let mut second = page("second");
        second.frontmatter.links = vec!["first".to_owned()];
        let prior = build_native_generation("project", &[page("first"), second.clone()])
            .expect("prior generation");
        second.frontmatter.links = vec!["third".to_owned()];
        second.content_digest = format!("sha256:{}", "b2".repeat(32));
        let current = vec![second];
        let incremental = build_native_generation_incremental("project", &prior, &current)
            .expect("incremental generation");
        let full = build_native_generation("project", &current).expect("full generation");
        assert_eq!(incremental, full);
    }

    #[test]
    fn query_is_metadata_only_and_bounded() {
        let generation = build_native_generation("project", &[page("first")]).expect("generation");
        let hits = query_generation(&generation, "first", 100);
        assert_eq!(hits.len(), 9);
        assert!(hits.iter().all(|edge| edge.from == "first"));
        assert!(query_generation(&generation, "missing", 10).is_empty());
        let metadata = query_node_metadata(&generation, "first", 100);
        assert_eq!(metadata.len(), 1);
        assert!(metadata.iter().all(|node| node.locator.contains("Wiki")));
    }

    #[test]
    fn query_excludes_nonactive_nodes_by_default() {
        let mut inactive = page("inactive");
        inactive.frontmatter.status = "superseded".to_owned();
        inactive.frontmatter.links = vec!["first".to_owned()];
        let generation =
            build_native_generation("project", &[page("first"), inactive]).expect("generation");
        assert!(query_generation(&generation, "inactive", 50).is_empty());
        let mut open = page("open");
        open.frontmatter.status = "open-question".to_owned();
        let generation = build_native_generation("project", &[open]).expect("generation");
        assert!(generation.nodes.is_empty());
    }

    #[test]
    fn scope_paths_are_disjoint_and_hive_owned() {
        let digest = format!("sha256:{}", "a1".repeat(32));
        let paths = [
            "source",
            "project",
            "user-root",
            "shared",
            "private",
            "confidential",
        ]
        .into_iter()
        .map(|scope| generation_relative_path(scope, &digest).expect("scope path"))
        .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(paths.len(), 6);
        assert!(paths.iter().all(|path| !path.is_absolute()));
        assert!(generation_relative_path("outside", &digest).is_err());
    }

    #[test]
    fn every_scope_activates_in_a_physically_disjoint_generation() {
        let target = tempfile::tempdir().expect("temporary target");
        std::fs::write(target.path().join("hive-source.json"), b"{}\n").expect("source marker");
        let mut paths = std::collections::BTreeSet::new();
        for scope in [
            "source",
            "project",
            "user-root",
            "shared",
            "private",
            "confidential",
        ] {
            let generation = build_native_generation(scope, &[page(scope)]).expect("generation");
            assert!(generation.nodes.iter().all(|node| node.visibility == scope));
            let changed = activate_generation(
                target.path(),
                &generation,
                &format!("sha256:{}", "e5".repeat(32)),
                None,
            )
            .expect("activate scope");
            for path in changed {
                assert!(paths.insert(path));
            }
        }
        assert_eq!(paths.len(), 12);
    }

    #[test]
    fn generation_persistence_is_digest_addressed_and_idempotent() {
        let target = tempfile::tempdir().expect("temporary target");
        let generation = build_native_generation("project", &[page("first")]).expect("generation");
        let relative = persist_generation(target.path(), &generation).expect("persist");
        let first = std::fs::read(target.path().join(&relative)).expect("generation bytes");
        assert_eq!(
            persist_generation(target.path(), &generation).expect("repeat"),
            relative
        );
        assert_eq!(
            std::fs::read(target.path().join(&relative)).expect("repeat bytes"),
            first
        );
        assert_eq!(
            remove_generation(target.path(), "project", &generation.generation_digest)
                .expect("remove"),
            Some(relative.clone())
        );
        assert!(!target.path().join(relative).exists());
    }

    #[test]
    fn active_pointer_binds_exact_generation_and_detects_tamper() {
        let target = tempfile::tempdir().expect("temporary target");
        let generation = build_native_generation("project", &[page("first")]).expect("generation");
        let changed = activate_generation(
            target.path(),
            &generation,
            &format!("sha256:{}", "b2".repeat(32)),
            None,
        )
        .expect("activate");
        assert_eq!(changed.len(), 2);
        let (_, loaded) =
            load_active_generation(target.path(), "project", "native-markdown").expect("load");
        assert_eq!(loaded, generation);
        std::fs::write(target.path().join(&changed[0]), b"{}\n").expect("tamper");
        assert!(load_active_generation(target.path(), "project", "native-markdown").is_err());
    }

    #[test]
    fn active_generation_removal_is_exact_and_idempotent() {
        let target = tempfile::tempdir().expect("temporary target");
        let generation = build_native_generation("project", &[page("first")]).expect("generation");
        activate_generation(
            target.path(),
            &generation,
            &format!("sha256:{}", "b2".repeat(32)),
            None,
        )
        .expect("activate");
        assert_eq!(
            remove_active_generation(target.path(), "project", "native-markdown")
                .expect("remove")
                .len(),
            2
        );
        assert!(
            remove_active_generation(target.path(), "project", "native-markdown")
                .expect("repeat")
                .is_empty()
        );
    }

    #[test]
    fn json_and_html_exports_are_digest_addressed_and_body_free() {
        let target = tempfile::tempdir().expect("temporary target");
        let generation = build_native_generation("project", &[page("first")]).expect("generation");
        for format in ["json", "html"] {
            let relative =
                export_generation(target.path(), &generation, format).expect("export generation");
            let bytes = std::fs::read(target.path().join(relative)).expect("export bytes");
            assert!(!bytes.windows(4).any(|window| window == b"body"));
        }
        assert!(export_generation(target.path(), &generation, "svg").is_err());
    }
}
