//! Deterministic derived relations from canonical Markdown Wiki pages.

use crate::WikiPage;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};

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
    let edges = extract_markdown_edges(pages);
    let payload = serde_json::json!({
        "schema_version": 1,
        "scope": scope,
        "engine": "native-markdown",
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
        engine: "native-markdown".to_owned(),
        generation_digest,
        nodes,
        edges,
    })
}

/// Return bounded direct relationships for one node without exposing page bodies.
#[must_use]
pub fn query_generation(
    generation: &GraphGeneration,
    node_id: &str,
    limit: usize,
) -> Vec<GraphEdge> {
    generation
        .edges
        .iter()
        .filter(|edge| edge.from == node_id || edge.to == node_id)
        .take(limit.min(50))
        .cloned()
        .collect()
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
    use super::{build_native_generation, extract_markdown_edges, query_generation, GraphEvidence};
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
    fn query_is_metadata_only_and_bounded() {
        let generation = build_native_generation("project", &[page("first")]).expect("generation");
        let hits = query_generation(&generation, "first", 100);
        assert_eq!(hits.len(), 9);
        assert!(hits.iter().all(|edge| edge.from == "first"));
        assert!(query_generation(&generation, "missing", 10).is_empty());
    }
}
