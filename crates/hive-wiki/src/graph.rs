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
        for tag in &page.frontmatter.tags {
            edges.push(edge(&from, tag, "tags", &digest));
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
    use super::{extract_markdown_edges, GraphEvidence};
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
                links: vec!["other".to_owned()],
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
        assert_eq!(first.len(), 4);
        assert!(first
            .iter()
            .all(|edge| edge.evidence == GraphEvidence::Extracted));
    }
}
