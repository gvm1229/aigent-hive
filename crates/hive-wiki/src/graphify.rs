//! Safe normalization of Graphify code-only output into the Hive graph contract.

use crate::graph::{GraphEdge, GraphEvidence, GraphGeneration, GraphNode};
use hive_core::{sha256_digest, validate_project_relative};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MAX_GRAPHIFY_BYTES: usize = 128 * 1024 * 1024;

/// Normalize one full `graphify extract --force --code-only` result.
///
/// Nodes without a project-relative source locator are excluded. Edges remain only when both
/// endpoints survived that grounding gate. The upstream schema never becomes a Hive API.
///
/// # Errors
///
/// Returns an error for unsupported scopes, malformed or oversized input, unsafe locators,
/// duplicate nodes, unknown fields, and unsupported evidence classes.
#[allow(clippy::too_many_lines)]
pub fn normalize_graphify_code(scope: &str, bytes: &[u8]) -> Result<GraphGeneration, String> {
    if !matches!(scope, "source" | "project") {
        return Err("Graphify code graph supports only source or project scope".to_owned());
    }
    if bytes.is_empty() || bytes.len() > MAX_GRAPHIFY_BYTES {
        return Err("Graphify code graph input is empty or oversized".to_owned());
    }
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("Graphify code graph JSON is malformed: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "Graphify code graph root must be an object".to_owned())?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "nodes" | "edges" | "hyperedges" | "input_tokens" | "output_tokens"
        )
    }) {
        return Err("Graphify code graph contains an unknown root field".to_owned());
    }
    let upstream_nodes = object
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "Graphify code graph nodes must be an array".to_owned())?;
    let upstream_edges = object
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| "Graphify code graph edges must be an array".to_owned())?;

    let mut nodes = BTreeMap::new();
    for value in upstream_nodes {
        let node = value
            .as_object()
            .ok_or_else(|| "Graphify node must be an object".to_owned())?;
        let id = string_field(node, "id")?;
        if id.len() > 512 || nodes.contains_key(id) {
            return Err("Graphify node id is oversized or duplicated".to_owned());
        }
        let source_file = string_field(node, "source_file")?;
        if source_file.is_empty() {
            continue;
        }
        validate_project_relative(Path::new(source_file))
            .map_err(|error| format!("Graphify node source path is unsafe: {error}"))?;
        let source_location = string_field(node, "source_location")?;
        if !valid_source_location(source_location) {
            return Err("Graphify node source location is invalid".to_owned());
        }
        let locator = format!("{source_file}:{source_location}");
        let content_digest = sha256_digest(
            &serde_json_canonicalizer::to_vec(value)
                .map_err(|error| format!("canonicalize Graphify node: {error}"))?,
        );
        nodes.insert(
            id.to_owned(),
            GraphNode {
                id: id.to_owned(),
                locator,
                content_digest,
                visibility: scope.to_owned(),
                lifecycle: "active".to_owned(),
            },
        );
    }

    let grounded = nodes.keys().cloned().collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for value in upstream_edges {
        let edge = value
            .as_object()
            .ok_or_else(|| "Graphify edge must be an object".to_owned())?;
        let from = string_field(edge, "source")?;
        let to = string_field(edge, "target")?;
        if !grounded.contains(from) || !grounded.contains(to) {
            continue;
        }
        let relation = string_field(edge, "relation")?;
        if relation.is_empty() || relation.len() > 128 {
            return Err("Graphify edge relation is invalid".to_owned());
        }
        let evidence = match string_field(edge, "confidence")? {
            "EXTRACTED" => GraphEvidence::Extracted,
            "INFERRED" => GraphEvidence::Inferred,
            "AMBIGUOUS" => GraphEvidence::Ambiguous,
            _ => return Err("Graphify edge confidence is unsupported".to_owned()),
        };
        let source_digest = nodes
            .get(from)
            .map(|node| node.content_digest.clone())
            .ok_or_else(|| "Graphify grounded source node disappeared".to_owned())?;
        edges.push(GraphEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            relation: relation.to_owned(),
            evidence,
            source_digest,
        });
    }
    let mut nodes = nodes.into_values().collect::<Vec<_>>();
    nodes.sort();
    edges.sort();
    edges.dedup();
    let payload = serde_json::json!({
        "schema_version": 1,
        "scope": scope,
        "engine": "graphify-code",
        "nodes": &nodes,
        "edges": &edges,
    });
    let generation_digest = sha256_digest(
        &serde_json_canonicalizer::to_vec(&payload)
            .map_err(|error| format!("canonicalize Graphify generation: {error}"))?,
    );
    Ok(GraphGeneration {
        schema_version: 1,
        scope: scope.to_owned(),
        engine: "graphify-code".to_owned(),
        generation_digest,
        nodes,
        edges,
    })
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Graphify field {field} must be a string"))
}

fn valid_source_location(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('L') else {
        return false;
    };
    let mut parts = rest.split("-L");
    let first = parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    let second = parts
        .next()
        .is_none_or(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    first && second && parts.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::normalize_graphify_code;

    const GRAPH: &[u8] = br#"{
      "nodes":[
        {"id":"src_main","source_file":"src/main.rs","source_location":"L1","_origin":"ast"},
        {"id":"src_store","source_file":"src/store.rs","source_location":"L2-L8","_origin":"ast"},
        {"id":"external","source_file":"","source_location":"","_origin":"ast"}
      ],
      "edges":[
        {"source":"src_main","target":"src_store","relation":"calls","confidence":"EXTRACTED"},
        {"source":"src_store","target":"external","relation":"returns","confidence":"EXTRACTED"}
      ],
      "hyperedges":[],"input_tokens":0,"output_tokens":0
    }"#;

    #[test]
    fn normalization_is_deterministic_and_excludes_ungrounded_nodes() {
        let first = normalize_graphify_code("project", GRAPH).expect("normalize");
        let second = normalize_graphify_code("project", GRAPH).expect("normalize");
        assert_eq!(first, second);
        assert_eq!(first.engine, "graphify-code");
        assert_eq!(first.nodes.len(), 2);
        assert_eq!(first.edges.len(), 1);
    }

    #[test]
    fn hostile_paths_unknown_fields_and_private_scopes_fail_closed() {
        let traversal = String::from_utf8(GRAPH.to_vec())
            .expect("UTF-8")
            .replace("src/main.rs", "../outside.rs");
        assert!(normalize_graphify_code("project", traversal.as_bytes()).is_err());
        let unknown = String::from_utf8(GRAPH.to_vec())
            .expect("UTF-8")
            .replace("\"output_tokens\":0", "\"output_tokens\":0,\"watch\":true");
        assert!(normalize_graphify_code("project", unknown.as_bytes()).is_err());
        assert!(normalize_graphify_code("confidential", GRAPH).is_err());
    }
}
