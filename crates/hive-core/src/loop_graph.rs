//! Provider-neutral, durable loop-graph contracts.
//!
//! This module only validates and transforms data. It never launches a
//! process, schedules work, sleeps, calls a model provider, or chooses a host
//! runtime.

use crate::{sha256_digest, validate_json_schema};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const LOOP_GRAPH_SCHEMA: &str = include_str!("../../../schemas/loop-graph.schema.json");
const LOOP_DISPATCH_SCHEMA: &str = include_str!("../../../schemas/loop-dispatch.schema.json");
const MAX_LOOP_DOCUMENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_LOOP_FRONTMATTER_BYTES: usize = 1536 * 1024;
const MAX_LOOP_BODY_BYTES: usize = 512 * 1024;

/// Durable graph lifecycle state.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoopState {
    /// More provider-owned work may be prepared.
    Active,
    /// Work stopped because authority, safety, usage, or external state blocks it.
    Blocked,
    /// Work stopped after an unrecoverable verification or execution failure.
    Failed,
    /// Every required criterion has independently verified evidence.
    Complete,
}

impl LoopState {
    /// Return whether no later graph revision may change this state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Blocked | Self::Failed | Self::Complete)
    }
}

/// Minimum truthful host support accepted for one node capability.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MinimumSupport {
    /// Only fully supported capability evidence is sufficient.
    Supported,
    /// Supported or explicitly qualified best-effort behavior is sufficient.
    BestEffort,
}

/// Truthful support claim for one provider-neutral capability.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilitySupportLevel {
    /// The qualified host surface supports the capability.
    Supported,
    /// The qualified host surface supports a documented reduced contract.
    BestEffort,
    /// The qualified host surface does not support the capability.
    Unsupported,
    /// Current evidence cannot establish support.
    Unverified,
}

/// Capability required by one graph node.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    /// Stable provider-neutral capability name.
    pub name: String,
    /// Weakest support level acceptable for dispatch.
    pub minimum_support: MinimumSupport,
}

/// Evidence-backed support claim captured with the graph.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilitySupport {
    /// Stable provider-neutral capability name.
    pub name: String,
    /// Truthful qualified support level.
    pub support: CapabilitySupportLevel,
    /// Digest of the exact qualification evidence.
    pub evidence_digest: String,
}

/// Bounded retry and backoff policy for one node.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    /// Total allowed attempts, including the first dispatch.
    pub max_attempts: u32,
    /// Backoff before the first retry.
    pub initial_backoff_seconds: u64,
    /// Integer multiplier applied after each failed attempt.
    pub backoff_multiplier: u32,
    /// Hard upper bound for calculated backoff.
    pub max_backoff_seconds: u64,
    /// Consecutive identical failures that stop retry early.
    pub identical_failure_limit: u32,
}

/// Kind of durable evidence.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// Execution or deterministic tool artifact.
    Artifact,
    /// Result of the required pre-dispatch usage gate.
    UsageAuthorization,
    /// Result produced independently of the executor role.
    IndependentVerification,
    /// Exact approval artifact for out-of-scope steering.
    SteeringAuthorization,
}

/// Normalized evidence result.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceResult {
    /// Artifact exists and matches its digest.
    Present,
    /// The requested boundary permits the action.
    Allowed,
    /// Verification passed.
    Passed,
    /// Verification failed.
    Failed,
    /// Verification could not produce a decision.
    Indeterminate,
}

/// Authority that produced independent verification evidence.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationAuthority {
    /// A deterministic verifier distinct from the executor role.
    Deterministic,
    /// An independent judge whose artifact must be authenticated.
    Judge,
}

/// Predicate that must match one exact durable evidence record.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidencePredicate {
    /// Exact evidence record id.
    pub evidence_id: String,
    /// Required evidence kind.
    pub kind: EvidenceKind,
    /// Required normalized result.
    pub result: EvidenceResult,
}

/// One executable unit in the loop DAG.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopNode {
    /// Stable node id.
    pub id: String,
    /// Persistent role that may execute this node.
    pub executor_role_id: String,
    /// Distinct persistent role that verifies this node.
    pub verifier_role_id: String,
    /// Required criteria owned exclusively by this node.
    pub criterion_ids: Vec<String>,
    /// Evidence predicates required before criteria may pass.
    pub completion_predicates: Vec<EvidencePredicate>,
    /// Provider-neutral capabilities required before dispatch.
    pub required_capabilities: Vec<CapabilityRequirement>,
    /// Retry policy applied without performing any wait or launch.
    pub retry_policy: RetryPolicy,
}

/// Directed dependency edge in the loop DAG.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopEdge {
    /// Stable edge id.
    pub id: String,
    /// Source node id.
    pub from: String,
    /// Destination node id.
    pub to: String,
    /// Evidence predicates required before the edge opens.
    pub predicates: Vec<EvidencePredicate>,
}

/// Durable evidence record bound to a graph revision and optional node attempt.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopEvidence {
    /// Stable evidence id referenced by predicates or dispatch bindings.
    pub id: String,
    /// Evidence kind.
    pub kind: EvidenceKind,
    /// Normalized evidence result.
    pub result: EvidenceResult,
    /// Revision at which this evidence was captured.
    pub graph_revision: u64,
    /// Node this evidence concerns, when applicable.
    pub subject_node_id: Option<String>,
    /// Node attempt this evidence concerns, when applicable.
    pub attempt: Option<u32>,
    /// Executor role that produced the subject work, when applicable.
    pub producer_role_id: Option<String>,
    /// Independent verifier role, when applicable.
    pub verifier_role_id: Option<String>,
    /// Verification authority, when this is verification evidence.
    pub verification_authority: Option<VerificationAuthority>,
    /// Safe bounded locator for the evidence artifact.
    pub locator: String,
    /// Digest of the exact evidence artifact.
    pub digest: String,
    /// Whether an authority-bound signature or equivalent was verified.
    pub authenticated: bool,
}

/// Recorded outcome for one prepared node dispatch.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeAttempt {
    /// Subject node id.
    pub node_id: String,
    /// Contiguous one-based attempt number for that node.
    pub attempt: u32,
    /// Revision at which the result was checkpointed.
    pub graph_revision: u64,
    /// Execution outcome.
    pub outcome: AttemptOutcome,
    /// Digest of the exact dispatch binding used for the attempt.
    pub dispatch_digest: String,
    /// Stable failure fingerprint, required only for failure.
    pub failure_fingerprint: Option<String>,
}

/// Recorded execution outcome.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptOutcome {
    /// Executor reported success; evidence still requires independent verification.
    Succeeded,
    /// Executor reported failure with a stable fingerprint.
    Failed,
}

/// User authority boundary attached to a topology steering revision.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserBoundary {
    /// Steering remains inside the scope already approved by the user.
    WithinApprovedScope,
    /// Steering expands or changes scope with exact user approval evidence.
    ExplicitUserApproval,
}

/// Durable topology steering record.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SteeringRecord {
    /// Exact revision that was steered.
    pub base_revision: u64,
    /// Canonical digest of the exact base revision document.
    pub base_revision_digest: String,
    /// Bounded reason for steering.
    pub reason: String,
    /// Exact ids of added, removed, or modified edges.
    pub affected_edges: Vec<String>,
    /// User authority boundary for this change.
    pub user_boundary: UserBoundary,
    /// Exact approval evidence for expanded scope.
    pub authorization_evidence_id: Option<String>,
}

/// Complete provider-neutral graph state stored in one Markdown revision.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopGraph {
    /// Graph schema version.
    pub schema_version: u32,
    /// Stable run id shared with the run schema v1 surface.
    pub run_id: String,
    /// Monotonic graph revision.
    pub revision: u64,
    /// Canonical digest of the immediately previous graph document.
    pub previous_revision_digest: Option<String>,
    /// Durable lifecycle state.
    pub state: LoopState,
    /// Required reason for blocked or failed terminal state.
    pub terminal_reason: Option<String>,
    /// Explicit root nodes of the DAG.
    pub entry_nodes: Vec<String>,
    /// Complete required criterion set.
    pub required_criteria: Vec<String>,
    /// Monotonic independently verified criterion passes.
    pub passed_criteria: Vec<String>,
    /// Graph nodes.
    pub nodes: Vec<LoopNode>,
    /// Current graph edges.
    pub edges: Vec<LoopEdge>,
    /// Append-only durable evidence.
    pub evidence: Vec<LoopEvidence>,
    /// Append-only durable node attempts.
    pub attempts: Vec<NodeAttempt>,
    /// Pinned provider-neutral capability support snapshot.
    pub capability_support: Vec<CapabilitySupport>,
    /// Append-only topology steering history.
    pub steering: Vec<SteeringRecord>,
}

/// Parsed graph document retaining exact Markdown body bytes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LoopGraphDocument {
    graph: LoopGraph,
    body: Vec<u8>,
}

/// Pure result of calculating the next allowed node attempt.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RetryDecision {
    /// The first attempt may be prepared immediately.
    FirstAttempt { attempt: u32 },
    /// Another attempt may be prepared after the calculated bounded delay.
    Retry {
        /// Next contiguous attempt number.
        attempt: u32,
        /// Pure calculated delay; this module never waits.
        backoff_seconds: u64,
    },
    /// A successful attempt already exists.
    StopSucceeded,
    /// The total attempt budget is exhausted.
    StopBudgetExhausted,
    /// Repeated identical failure reached the early-stop boundary.
    StopRepeatedFailure {
        /// Repeated stable fingerprint.
        fingerprint: String,
        /// Number of consecutive identical failures.
        occurrences: u32,
    },
}

/// Kind of host-owned action described by a dispatch binding.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoopDispatchKind {
    /// First execution attempt for a ready node.
    Node,
    /// Retry permitted by the node retry policy.
    Retry,
    /// Request to prepare a topology steering proposal.
    Steering,
}

/// Provider-neutral, prepare-only binding for one host-owned action.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopDispatchBinding {
    /// Dispatch schema version.
    pub schema_version: u32,
    /// Exact run id.
    pub run_id: String,
    /// Exact graph revision.
    pub graph_revision: u64,
    /// Canonical digest of the exact graph document.
    pub graph_digest: String,
    /// Requested action kind.
    pub kind: LoopDispatchKind,
    /// Target node for node or retry dispatch.
    pub node_id: Option<String>,
    /// Exact target attempt for node or retry dispatch.
    pub attempt: Option<u32>,
    /// Exact executor role for node or retry dispatch.
    pub role_id: Option<String>,
    /// Digest of the bounded host brief.
    pub brief_digest: String,
    /// Digest of the graph's capability support snapshot.
    pub capability_snapshot_digest: String,
    /// Current-revision usage authorization evidence id.
    pub usage_evidence_id: String,
    /// Always true; this contract does not launch work.
    pub prepared_only: bool,
}

impl LoopDispatchBinding {
    /// Encode deterministic schema-valid dispatch data.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the binding violates the structural schema.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, LoopContractError> {
        let value = serde_json::to_value(self)
            .map_err(|error| LoopContractError::Malformed(error.to_string()))?;
        validate_json_schema(LOOP_DISPATCH_SCHEMA, &value, "loop dispatch")
            .map_err(LoopContractError::Schema)?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|error| LoopContractError::Malformed(error.to_string()))
    }

    /// Return the deterministic dispatch binding digest.
    ///
    /// # Errors
    ///
    /// Returns a typed error when canonical encoding fails.
    pub fn canonical_digest(&self) -> Result<String, LoopContractError> {
        self.encode_canonical().map(|bytes| sha256_digest(&bytes))
    }
}

impl LoopGraphDocument {
    /// Parse a bounded schema-valid graph Markdown document.
    ///
    /// YAML-compatible frontmatter is normalized into the typed contract. The
    /// Markdown body is retained byte-for-byte for deterministic fresh-session
    /// recovery.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed, oversized, or semantically invalid
    /// graph state.
    pub fn parse_markdown(bytes: &[u8]) -> Result<Self, LoopContractError> {
        if bytes.len() > MAX_LOOP_DOCUMENT_BYTES {
            return Err(LoopContractError::TooLarge("loop graph document"));
        }
        let (frontmatter, body) = split_frontmatter(bytes)?;
        if frontmatter.len() > MAX_LOOP_FRONTMATTER_BYTES {
            return Err(LoopContractError::TooLarge("loop graph frontmatter"));
        }
        validate_body(body)?;
        let value: serde_json::Value = serde_yaml::from_slice(frontmatter)
            .map_err(|error| LoopContractError::Malformed(error.to_string()))?;
        validate_json_schema(LOOP_GRAPH_SCHEMA, &value, "loop graph")
            .map_err(LoopContractError::Schema)?;
        let graph: LoopGraph = serde_json::from_value(value)
            .map_err(|error| LoopContractError::Malformed(error.to_string()))?;
        validate_graph_semantics(&graph)?;
        Ok(Self {
            graph,
            body: body.to_vec(),
        })
    }

    /// Create a validated graph document from typed state and Markdown bytes.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid graph semantics or body bytes.
    pub fn from_graph(graph: LoopGraph, body: Vec<u8>) -> Result<Self, LoopContractError> {
        validate_body(&body)?;
        validate_typed_schema(&graph)?;
        validate_graph_semantics(&graph)?;
        Ok(Self { graph, body })
    }

    /// Return the validated typed graph.
    #[must_use]
    pub const fn graph(&self) -> &LoopGraph {
        &self.graph
    }

    /// Return exact Markdown body bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Revalidate the complete checkpoint without side effects.
    ///
    /// # Errors
    ///
    /// Returns a typed error for structural or semantic graph corruption.
    pub fn validate_checkpoint(&self) -> Result<(), LoopContractError> {
        validate_body(&self.body)?;
        validate_typed_schema(&self.graph)?;
        validate_graph_semantics(&self.graph)
    }

    /// Encode canonical JSON frontmatter with LF delimiters and exact body.
    ///
    /// # Errors
    ///
    /// Returns a typed error when checkpoint validation or encoding fails.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, LoopContractError> {
        self.validate_checkpoint()?;
        let frontmatter = serde_json_canonicalizer::to_string(&self.graph)
            .map_err(|error| LoopContractError::Malformed(error.to_string()))?;
        let mut output = Vec::with_capacity(frontmatter.len() + self.body.len() + 10);
        output.extend_from_slice(b"---\n");
        output.extend_from_slice(frontmatter.as_bytes());
        output.extend_from_slice(b"\n---\n");
        output.extend_from_slice(&self.body);
        Ok(output)
    }

    /// Return the canonical graph Markdown digest.
    ///
    /// # Errors
    ///
    /// Returns a typed error when canonical encoding fails.
    pub fn canonical_digest(&self) -> Result<String, LoopContractError> {
        self.encode_canonical().map(|bytes| sha256_digest(&bytes))
    }

    /// Validate one prepare-only dispatch against this exact graph revision.
    ///
    /// This checks usage authorization, capability support, graph digest,
    /// executor role, dependency readiness, retry decision, and attempt number.
    /// It performs no launch, wait, or host selection.
    ///
    /// # Errors
    ///
    /// Returns a typed blocked or unsupported result for any stale or unsafe
    /// dispatch binding.
    pub fn validate_dispatch(
        &self,
        binding: &LoopDispatchBinding,
    ) -> Result<(), LoopContractError> {
        self.validate_checkpoint()?;
        let _ = binding.encode_canonical()?;
        if self.graph.state != LoopState::Active {
            return Err(LoopContractError::InvalidDispatchBinding(
                "terminal graph does not authorize dispatch".to_owned(),
            ));
        }
        if binding.run_id != self.graph.run_id
            || binding.graph_revision != self.graph.revision
            || binding.graph_digest != self.canonical_digest()?
        {
            return Err(LoopContractError::InvalidDispatchBinding(
                "dispatch does not bind the exact graph revision".to_owned(),
            ));
        }
        if binding.capability_snapshot_digest != self.graph.capability_snapshot_digest()? {
            return Err(LoopContractError::InvalidDispatchBinding(
                "dispatch capability snapshot digest is stale".to_owned(),
            ));
        }
        let usage = self
            .graph
            .evidence
            .iter()
            .find(|evidence| evidence.id == binding.usage_evidence_id)
            .ok_or_else(|| {
                LoopContractError::InvalidDispatchBinding(
                    "usage authorization evidence is missing".to_owned(),
                )
            })?;
        if usage.kind != EvidenceKind::UsageAuthorization
            || usage.result != EvidenceResult::Allowed
            || usage.graph_revision != self.graph.revision
        {
            return Err(LoopContractError::InvalidDispatchBinding(
                "usage authorization must allow this exact graph revision".to_owned(),
            ));
        }

        match binding.kind {
            LoopDispatchKind::Steering => {
                if usage.subject_node_id.is_some() {
                    return Err(LoopContractError::InvalidDispatchBinding(
                        "steering usage authorization must be graph-scoped".to_owned(),
                    ));
                }
            }
            LoopDispatchKind::Node | LoopDispatchKind::Retry => {
                let node_id = binding.node_id.as_deref().ok_or_else(|| {
                    LoopContractError::InvalidDispatchBinding(
                        "node dispatch is missing its node id".to_owned(),
                    )
                })?;
                let node = self.graph.node(node_id)?;
                if binding.role_id.as_deref() != Some(node.executor_role_id.as_str()) {
                    return Err(LoopContractError::InvalidDispatchBinding(
                        "dispatch role is not the node executor".to_owned(),
                    ));
                }
                if usage.subject_node_id.as_deref() != Some(node_id) {
                    return Err(LoopContractError::InvalidDispatchBinding(
                        "usage authorization is bound to another target".to_owned(),
                    ));
                }
                self.graph.validate_node_capabilities(node_id)?;
                if !dependencies_ready_unchecked(&self.graph, node_id) {
                    return Err(LoopContractError::InvalidDispatchBinding(
                        "node dependencies are not evidence-ready".to_owned(),
                    ));
                }
                let attempt = binding.attempt.ok_or_else(|| {
                    LoopContractError::InvalidDispatchBinding(
                        "node dispatch is missing its attempt".to_owned(),
                    )
                })?;
                if usage.attempt != Some(attempt) {
                    return Err(LoopContractError::InvalidDispatchBinding(
                        "usage authorization is bound to another attempt".to_owned(),
                    ));
                }
                match (binding.kind, self.graph.retry_decision(node_id)?) {
                    (LoopDispatchKind::Node, RetryDecision::FirstAttempt { attempt: expected })
                        if attempt == expected => {}
                    (
                        LoopDispatchKind::Retry,
                        RetryDecision::Retry {
                            attempt: expected, ..
                        },
                    ) if attempt == expected => {}
                    _ => {
                        return Err(LoopContractError::InvalidDispatchBinding(
                            "dispatch kind or attempt contradicts retry state".to_owned(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

impl LoopGraph {
    /// Validate the structural and semantic graph contract.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed DAG, criteria, evidence, retries,
    /// steering, or terminal state.
    pub fn validate(&self) -> Result<(), LoopContractError> {
        validate_typed_schema(self)?;
        validate_graph_semantics(self)
    }

    /// Return one node by stable id.
    ///
    /// # Errors
    ///
    /// Returns [`LoopContractError::UnknownNode`] when no node matches.
    pub fn node(&self, node_id: &str) -> Result<&LoopNode, LoopContractError> {
        self.nodes
            .iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| LoopContractError::UnknownNode(node_id.to_owned()))
    }

    /// Return a permutation-stable digest of the pinned capability snapshot.
    ///
    /// # Errors
    ///
    /// Returns a typed error when graph validation or canonicalization fails.
    pub fn capability_snapshot_digest(&self) -> Result<String, LoopContractError> {
        validate_capability_support(self)?;
        let mut snapshot = self.capability_support.clone();
        snapshot.sort_by(|left, right| left.name.cmp(&right.name));
        let bytes = serde_json_canonicalizer::to_vec(&snapshot)
            .map_err(|error| LoopContractError::Malformed(error.to_string()))?;
        Ok(sha256_digest(&bytes))
    }

    /// Require truthful support for every capability of one node.
    ///
    /// # Errors
    ///
    /// Returns [`LoopContractError::HostCapabilityUnsupported`] for missing,
    /// unsupported, unverified, or insufficient best-effort support.
    pub fn validate_node_capabilities(&self, node_id: &str) -> Result<(), LoopContractError> {
        let node = self.node(node_id)?;
        let support = self
            .capability_support
            .iter()
            .map(|item| (item.name.as_str(), item.support))
            .collect::<BTreeMap<_, _>>();
        for requirement in &node.required_capabilities {
            let observed = support.get(requirement.name.as_str()).copied();
            let accepted = matches!(
                (requirement.minimum_support, observed),
                (
                    MinimumSupport::Supported,
                    Some(CapabilitySupportLevel::Supported)
                ) | (
                    MinimumSupport::BestEffort,
                    Some(CapabilitySupportLevel::Supported | CapabilitySupportLevel::BestEffort)
                )
            );
            if !accepted {
                return Err(LoopContractError::HostCapabilityUnsupported {
                    node_id: node_id.to_owned(),
                    capability: requirement.name.clone(),
                    support: observed,
                });
            }
        }
        Ok(())
    }

    /// Return whether one edge has verified source work and all predicates.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the graph or edge id is invalid.
    pub fn edge_ready(&self, edge_id: &str) -> Result<bool, LoopContractError> {
        self.validate()?;
        let edge = self
            .edges
            .iter()
            .find(|edge| edge.id == edge_id)
            .ok_or_else(|| LoopContractError::UnknownEdge(edge_id.to_owned()))?;
        Ok(edge_ready_unchecked(self, edge))
    }

    /// Return whether one node has successful execution, all completion
    /// predicates, and independently bound PASS evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the graph or node id is invalid.
    pub fn node_completion_ready(&self, node_id: &str) -> Result<bool, LoopContractError> {
        self.validate()?;
        let node = self.node(node_id)?;
        Ok(node_completion_ready_unchecked(self, node))
    }

    /// Return dispatchable node ids in declaration order.
    ///
    /// Terminal graphs return no ready nodes. A node is ready only after all
    /// incoming evidence edges open and while its retry policy permits another
    /// attempt.
    ///
    /// # Errors
    ///
    /// Returns a typed error when graph state is malformed.
    pub fn ready_nodes(&self) -> Result<Vec<String>, LoopContractError> {
        self.validate()?;
        if self.state.is_terminal() {
            return Ok(Vec::new());
        }
        let mut ready = Vec::new();
        for node in &self.nodes {
            if !dependencies_ready_unchecked(self, &node.id) {
                continue;
            }
            if matches!(
                retry_decision_unchecked(self, node),
                RetryDecision::FirstAttempt { .. } | RetryDecision::Retry { .. }
            ) {
                ready.push(node.id.clone());
            }
        }
        Ok(ready)
    }

    /// Calculate the next attempt or deterministic early-stop result.
    ///
    /// The returned backoff is data only. This function never sleeps.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the graph or node id is invalid.
    pub fn retry_decision(&self, node_id: &str) -> Result<RetryDecision, LoopContractError> {
        self.validate()?;
        let node = self.node(node_id)?;
        Ok(retry_decision_unchecked(self, node))
    }
}

/// Pure loop graph transition result.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LoopTransitionOutcome {
    /// Exact retry of the same durable document; no write is required.
    Idempotent,
    /// Legal next revision.
    Advance,
}

/// Validate an exact durable graph revision transition.
///
/// Every non-idempotent transition increments the revision once, binds the
/// previous canonical digest, preserves immutable graph definitions and
/// append-only history, and makes topology changes only through an exact
/// steering record. Blocked, failed, and complete are terminal.
///
/// # Errors
///
/// Returns a typed error for stale revisions, history regression, unbound
/// steering, terminal mutation, or any invalid next checkpoint.
pub fn validate_loop_transition(
    previous: &LoopGraphDocument,
    next: &LoopGraphDocument,
) -> Result<LoopTransitionOutcome, LoopContractError> {
    previous.validate_checkpoint()?;
    next.validate_checkpoint()?;
    if previous == next {
        return Ok(LoopTransitionOutcome::Idempotent);
    }
    let prior = previous.graph();
    let candidate = next.graph();
    if prior.state.is_terminal() {
        return Err(LoopContractError::TerminalStateImmutable);
    }
    if prior.schema_version != candidate.schema_version || prior.run_id != candidate.run_id {
        return Err(LoopContractError::RunIdentityChanged);
    }
    let expected_revision =
        prior
            .revision
            .checked_add(1)
            .ok_or(LoopContractError::InvalidRevision {
                expected: prior.revision,
                actual: candidate.revision,
            })?;
    if candidate.revision != expected_revision {
        return Err(LoopContractError::InvalidRevision {
            expected: expected_revision,
            actual: candidate.revision,
        });
    }
    let prior_digest = previous.canonical_digest()?;
    if candidate.previous_revision_digest.as_deref() != Some(prior_digest.as_str()) {
        return Err(LoopContractError::PreviousDigestMismatch);
    }
    if prior.entry_nodes != candidate.entry_nodes
        || prior.required_criteria != candidate.required_criteria
        || prior.nodes != candidate.nodes
        || prior.capability_support != candidate.capability_support
    {
        return Err(LoopContractError::GraphDefinitionChanged);
    }
    validate_transition_history(prior, candidate)?;
    validate_transition_steering(prior, candidate, &prior_digest)?;
    Ok(LoopTransitionOutcome::Advance)
}

fn validate_transition_history(
    prior: &LoopGraph,
    candidate: &LoopGraph,
) -> Result<(), LoopContractError> {
    require_prefix(
        &prior.passed_criteria,
        &candidate.passed_criteria,
        "passed criteria",
    )?;
    require_prefix(&prior.evidence, &candidate.evidence, "evidence")?;
    require_prefix(&prior.attempts, &candidate.attempts, "attempts")?;
    require_prefix(&prior.steering, &candidate.steering, "steering")?;
    if candidate.evidence[prior.evidence.len()..]
        .iter()
        .any(|evidence| evidence.graph_revision != candidate.revision)
    {
        return Err(LoopContractError::HistoryRegression(
            "new evidence must bind the next revision",
        ));
    }
    if candidate.attempts[prior.attempts.len()..]
        .iter()
        .any(|attempt| attempt.graph_revision != candidate.revision)
    {
        return Err(LoopContractError::HistoryRegression(
            "new attempts must bind the next revision",
        ));
    }
    Ok(())
}

fn validate_transition_steering(
    prior: &LoopGraph,
    candidate: &LoopGraph,
    prior_digest: &str,
) -> Result<(), LoopContractError> {
    let changed_edges = changed_edge_ids(&prior.edges, &candidate.edges);
    let new_steering = &candidate.steering[prior.steering.len()..];
    if changed_edges.is_empty() {
        if !new_steering.is_empty() {
            return Err(LoopContractError::UnnecessarySteering);
        }
    } else {
        if new_steering.len() != 1 {
            return Err(LoopContractError::TopologyChangedWithoutSteering);
        }
        let steering = &new_steering[0];
        if steering.base_revision != prior.revision
            || steering.base_revision_digest != prior_digest
            || as_set(&steering.affected_edges) != as_set(&changed_edges)
        {
            return Err(LoopContractError::InvalidSteering(
                "steering does not bind the exact base revision and changed edges".to_owned(),
            ));
        }
        if steering.user_boundary == UserBoundary::ExplicitUserApproval {
            let evidence_id = steering
                .authorization_evidence_id
                .as_deref()
                .ok_or_else(|| {
                    LoopContractError::InvalidSteering(
                        "explicit user steering is missing approval evidence".to_owned(),
                    )
                })?;
            if !candidate.evidence[prior.evidence.len()..]
                .iter()
                .any(|evidence| evidence.id == evidence_id)
            {
                return Err(LoopContractError::InvalidSteering(
                    "steering approval evidence is not new in this revision".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_typed_schema(graph: &LoopGraph) -> Result<(), LoopContractError> {
    let value = serde_json::to_value(graph)
        .map_err(|error| LoopContractError::Malformed(error.to_string()))?;
    validate_json_schema(LOOP_GRAPH_SCHEMA, &value, "loop graph").map_err(LoopContractError::Schema)
}

fn validate_graph_semantics(graph: &LoopGraph) -> Result<(), LoopContractError> {
    validate_unique_ids(graph.nodes.iter().map(|node| node.id.as_str()), "nodes")?;
    validate_unique_ids(graph.edges.iter().map(|edge| edge.id.as_str()), "edges")?;
    validate_unique_ids(
        graph.evidence.iter().map(|evidence| evidence.id.as_str()),
        "evidence",
    )?;
    validate_capability_support(graph)?;
    validate_nodes_and_criteria(graph)?;
    validate_dag(graph)?;
    validate_attempts(graph)?;
    validate_evidence(graph)?;
    validate_steering_history(graph)?;
    validate_passes_and_state(graph)
}

fn validate_unique_ids<'a>(
    values: impl IntoIterator<Item = &'a str>,
    field: &'static str,
) -> Result<(), LoopContractError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(LoopContractError::DuplicateIdentifier {
                field,
                value: value.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_capability_support(graph: &LoopGraph) -> Result<(), LoopContractError> {
    validate_unique_ids(
        graph
            .capability_support
            .iter()
            .map(|support| support.name.as_str()),
        "capability_support",
    )
}

fn validate_nodes_and_criteria(graph: &LoopGraph) -> Result<(), LoopContractError> {
    let required = as_set(&graph.required_criteria);
    let mut owners = BTreeMap::<&str, &str>::new();
    for node in &graph.nodes {
        if node.executor_role_id == node.verifier_role_id {
            return Err(LoopContractError::ExecutorVerifierConflict(node.id.clone()));
        }
        if node.retry_policy.initial_backoff_seconds > node.retry_policy.max_backoff_seconds {
            return Err(LoopContractError::InvalidRetryPolicy(node.id.clone()));
        }
        validate_unique_ids(
            node.required_capabilities
                .iter()
                .map(|requirement| requirement.name.as_str()),
            "node required capabilities",
        )?;
        validate_unique_ids(
            node.completion_predicates
                .iter()
                .map(|predicate| predicate.evidence_id.as_str()),
            "node completion predicates",
        )?;
        for criterion in &node.criterion_ids {
            if !required.contains(criterion.as_str()) {
                return Err(LoopContractError::UnknownCriterion(criterion.clone()));
            }
            if let Some(first) = owners.insert(criterion, &node.id) {
                return Err(LoopContractError::CriterionAssignedMultipleTimes {
                    criterion: criterion.clone(),
                    first_node: first.to_owned(),
                    second_node: node.id.clone(),
                });
            }
        }
    }
    for criterion in &graph.required_criteria {
        if !owners.contains_key(criterion.as_str()) {
            return Err(LoopContractError::OrphanCriterion(criterion.clone()));
        }
    }
    Ok(())
}

fn validate_dag(graph: &LoopGraph) -> Result<(), LoopContractError> {
    let indices = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut adjacency = vec![Vec::new(); graph.nodes.len()];
    let mut indegree = vec![0_usize; graph.nodes.len()];
    for edge in &graph.edges {
        let from = indices
            .get(edge.from.as_str())
            .copied()
            .ok_or_else(|| LoopContractError::UnknownNode(edge.from.clone()))?;
        let to = indices
            .get(edge.to.as_str())
            .copied()
            .ok_or_else(|| LoopContractError::UnknownNode(edge.to.clone()))?;
        if from == to {
            return Err(LoopContractError::SelfEdge(edge.id.clone()));
        }
        validate_unique_ids(
            edge.predicates
                .iter()
                .map(|predicate| predicate.evidence_id.as_str()),
            "edge predicates",
        )?;
        adjacency[from].push(to);
        indegree[to] += 1;
    }
    let mut entry_indices = BTreeSet::new();
    for entry in &graph.entry_nodes {
        let index = indices
            .get(entry.as_str())
            .copied()
            .ok_or_else(|| LoopContractError::UnknownNode(entry.clone()))?;
        if indegree[index] != 0 {
            return Err(LoopContractError::EntryHasIncomingEdge(entry.clone()));
        }
        entry_indices.insert(index);
    }

    let mut remaining = indegree.clone();
    let mut queue = remaining
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect::<VecDeque<_>>();
    let mut visited_count = 0_usize;
    while let Some(index) = queue.pop_front() {
        visited_count += 1;
        for target in &adjacency[index] {
            remaining[*target] -= 1;
            if remaining[*target] == 0 {
                queue.push_back(*target);
            }
        }
    }
    if visited_count != graph.nodes.len() {
        return Err(LoopContractError::Cycle);
    }

    let mut reachable = BTreeSet::new();
    let mut queue = entry_indices.into_iter().collect::<VecDeque<_>>();
    while let Some(index) = queue.pop_front() {
        if !reachable.insert(index) {
            continue;
        }
        queue.extend(adjacency[index].iter().copied());
    }
    for (index, node) in graph.nodes.iter().enumerate() {
        if !reachable.contains(&index) {
            return Err(LoopContractError::UnreachableNode(node.id.clone()));
        }
    }
    Ok(())
}

fn validate_attempts(graph: &LoopGraph) -> Result<(), LoopContractError> {
    let nodes = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut histories = BTreeMap::<&str, Vec<&NodeAttempt>>::new();
    let mut last_revision = 0_u64;
    for attempt in &graph.attempts {
        let node = nodes
            .get(attempt.node_id.as_str())
            .copied()
            .ok_or_else(|| LoopContractError::UnknownNode(attempt.node_id.clone()))?;
        if attempt.graph_revision < last_revision || attempt.graph_revision > graph.revision {
            return Err(LoopContractError::InvalidAttempt {
                node_id: attempt.node_id.clone(),
                reason: "attempt revisions must be ordered and not from the future".to_owned(),
            });
        }
        last_revision = attempt.graph_revision;
        if !graph.evidence.iter().any(|evidence| {
            evidence.kind == EvidenceKind::UsageAuthorization
                && evidence.result == EvidenceResult::Allowed
                && evidence.subject_node_id.as_deref() == Some(attempt.node_id.as_str())
                && evidence.attempt == Some(attempt.attempt)
                && evidence.graph_revision <= attempt.graph_revision
        }) {
            return Err(LoopContractError::InvalidAttempt {
                node_id: attempt.node_id.clone(),
                reason: "attempt lacks exact pre-dispatch usage authorization".to_owned(),
            });
        }
        match (attempt.outcome, attempt.failure_fingerprint.as_deref()) {
            (AttemptOutcome::Succeeded, None) | (AttemptOutcome::Failed, Some(_)) => {}
            _ => {
                return Err(LoopContractError::InvalidAttempt {
                    node_id: attempt.node_id.clone(),
                    reason: "failure requires exactly one fingerprint and success forbids it"
                        .to_owned(),
                });
            }
        }
        let history = histories.entry(node.id.as_str()).or_default();
        let expected = u32::try_from(history.len())
            .ok()
            .and_then(|length| length.checked_add(1))
            .ok_or_else(|| LoopContractError::InvalidAttempt {
                node_id: node.id.clone(),
                reason: "attempt count overflowed".to_owned(),
            })?;
        if attempt.attempt != expected {
            return Err(LoopContractError::InvalidAttempt {
                node_id: node.id.clone(),
                reason: format!(
                    "attempts must be contiguous: expected {expected}, found {}",
                    attempt.attempt
                ),
            });
        }
        if !history.is_empty()
            && !matches!(
                retry_decision_from_history(node, history),
                RetryDecision::Retry {
                    attempt: next,
                    ..
                } if next == attempt.attempt
            )
        {
            return Err(LoopContractError::InvalidAttempt {
                node_id: node.id.clone(),
                reason: "attempt exists after success or retry early-stop".to_owned(),
            });
        }
        history.push(attempt);
    }
    Ok(())
}

fn validate_evidence(graph: &LoopGraph) -> Result<(), LoopContractError> {
    let mut last_revision = 0_u64;
    for evidence in &graph.evidence {
        if evidence.graph_revision < last_revision || evidence.graph_revision > graph.revision {
            return Err(invalid_evidence(
                evidence,
                "evidence revisions must be ordered and not from the future",
            ));
        }
        last_revision = evidence.graph_revision;
        match evidence.kind {
            EvidenceKind::Artifact => validate_artifact_evidence(graph, evidence)?,
            EvidenceKind::UsageAuthorization => validate_usage_evidence(graph, evidence)?,
            EvidenceKind::IndependentVerification => {
                validate_verification_evidence(graph, evidence)?;
            }
            EvidenceKind::SteeringAuthorization => validate_steering_evidence(evidence)?,
        }
    }
    Ok(())
}

fn validate_artifact_evidence(
    graph: &LoopGraph,
    evidence: &LoopEvidence,
) -> Result<(), LoopContractError> {
    if evidence.result != EvidenceResult::Present
        || evidence.verifier_role_id.is_some()
        || evidence.verification_authority.is_some()
    {
        return Err(invalid_evidence(
            evidence,
            "artifact fields are contradictory",
        ));
    }
    match evidence.subject_node_id.as_deref() {
        Some(node_id) => {
            let node = graph.node(node_id)?;
            let attempt = evidence.attempt.ok_or_else(|| {
                invalid_evidence(evidence, "node artifact is missing its attempt")
            })?;
            let recorded_attempt = find_attempt(graph, node_id, attempt);
            if evidence.producer_role_id.as_deref() != Some(node.executor_role_id.as_str())
                || recorded_attempt.is_none()
                || recorded_attempt
                    .is_some_and(|item| evidence.graph_revision < item.graph_revision)
            {
                return Err(invalid_evidence(
                    evidence,
                    "artifact does not bind an existing executor attempt",
                ));
            }
        }
        None => {
            if evidence.attempt.is_some() || evidence.producer_role_id.is_some() {
                return Err(invalid_evidence(
                    evidence,
                    "graph-scoped artifact contains node fields",
                ));
            }
        }
    }
    Ok(())
}

fn validate_usage_evidence(
    graph: &LoopGraph,
    evidence: &LoopEvidence,
) -> Result<(), LoopContractError> {
    if evidence.result != EvidenceResult::Allowed
        || evidence.producer_role_id.is_some()
        || evidence.verifier_role_id.is_some()
        || evidence.verification_authority.is_some()
    {
        return Err(invalid_evidence(
            evidence,
            "usage authorization fields are contradictory",
        ));
    }
    if let Some(node_id) = evidence.subject_node_id.as_deref() {
        let _ = graph.node(node_id)?;
        if evidence.attempt.is_none() {
            return Err(invalid_evidence(
                evidence,
                "node usage authorization is missing its target attempt",
            ));
        }
    } else if evidence.attempt.is_some() {
        return Err(invalid_evidence(
            evidence,
            "graph-scoped usage authorization contains an attempt",
        ));
    }
    Ok(())
}

fn validate_verification_evidence(
    graph: &LoopGraph,
    evidence: &LoopEvidence,
) -> Result<(), LoopContractError> {
    if !matches!(
        evidence.result,
        EvidenceResult::Passed | EvidenceResult::Failed | EvidenceResult::Indeterminate
    ) {
        return Err(invalid_evidence(
            evidence,
            "verification has a non-verification result",
        ));
    }
    let node_id = evidence
        .subject_node_id
        .as_deref()
        .ok_or_else(|| invalid_evidence(evidence, "verification is missing its node"))?;
    let node = graph.node(node_id)?;
    let attempt_number = evidence
        .attempt
        .ok_or_else(|| invalid_evidence(evidence, "verification is missing its attempt"))?;
    let attempt = find_attempt(graph, node_id, attempt_number)
        .ok_or_else(|| invalid_evidence(evidence, "verification references an unknown attempt"))?;
    if attempt.outcome != AttemptOutcome::Succeeded
        || evidence.graph_revision < attempt.graph_revision
        || evidence.producer_role_id.as_deref() != Some(node.executor_role_id.as_str())
        || evidence.verifier_role_id.as_deref() != Some(node.verifier_role_id.as_str())
        || evidence.producer_role_id == evidence.verifier_role_id
        || evidence.verification_authority.is_none()
    {
        return Err(invalid_evidence(
            evidence,
            "verification is not independently bound to a successful executor attempt",
        ));
    }
    if evidence.verification_authority == Some(VerificationAuthority::Judge)
        && !evidence.authenticated
    {
        return Err(invalid_evidence(
            evidence,
            "judge verification must be authenticated",
        ));
    }
    Ok(())
}

fn validate_steering_evidence(evidence: &LoopEvidence) -> Result<(), LoopContractError> {
    if evidence.result != EvidenceResult::Allowed
        || evidence.subject_node_id.is_some()
        || evidence.attempt.is_some()
        || evidence.producer_role_id.is_some()
        || evidence.verifier_role_id.is_some()
        || evidence.verification_authority.is_some()
        || !evidence.authenticated
    {
        return Err(invalid_evidence(
            evidence,
            "steering authorization must be authenticated and graph-scoped",
        ));
    }
    Ok(())
}

fn validate_steering_history(graph: &LoopGraph) -> Result<(), LoopContractError> {
    if graph.revision == 1 && !graph.steering.is_empty() {
        return Err(LoopContractError::InvalidSteering(
            "initial graph cannot contain steering history".to_owned(),
        ));
    }
    let mut prior_base = 0_u64;
    for steering in &graph.steering {
        if steering.base_revision <= prior_base || steering.base_revision >= graph.revision {
            return Err(LoopContractError::InvalidSteering(
                "steering base revisions must be strictly ordered and historical".to_owned(),
            ));
        }
        prior_base = steering.base_revision;
        match steering.user_boundary {
            UserBoundary::WithinApprovedScope => {
                if steering.authorization_evidence_id.is_some() {
                    return Err(LoopContractError::InvalidSteering(
                        "in-scope steering must not claim new user approval".to_owned(),
                    ));
                }
            }
            UserBoundary::ExplicitUserApproval => {
                let evidence_id =
                    steering
                        .authorization_evidence_id
                        .as_deref()
                        .ok_or_else(|| {
                            LoopContractError::InvalidSteering(
                                "explicit user steering requires approval evidence".to_owned(),
                            )
                        })?;
                let evidence = graph
                    .evidence
                    .iter()
                    .find(|evidence| evidence.id == evidence_id)
                    .ok_or_else(|| {
                        LoopContractError::InvalidSteering(
                            "steering approval evidence is missing".to_owned(),
                        )
                    })?;
                if evidence.kind != EvidenceKind::SteeringAuthorization
                    || evidence.result != EvidenceResult::Allowed
                    || !evidence.authenticated
                    || evidence.graph_revision != steering.base_revision + 1
                {
                    return Err(LoopContractError::InvalidSteering(
                        "steering approval evidence does not bind the steered revision".to_owned(),
                    ));
                }
            }
        }
        if steering.base_revision + 1 == graph.revision
            && graph.previous_revision_digest.as_deref()
                != Some(steering.base_revision_digest.as_str())
        {
            return Err(LoopContractError::InvalidSteering(
                "latest steering does not bind previous_revision_digest".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_passes_and_state(graph: &LoopGraph) -> Result<(), LoopContractError> {
    let required = as_set(&graph.required_criteria);
    for criterion in &graph.passed_criteria {
        if !required.contains(criterion.as_str()) {
            return Err(LoopContractError::UnknownCriterion(criterion.clone()));
        }
        let node = graph
            .nodes
            .iter()
            .find(|node| node.criterion_ids.contains(criterion))
            .ok_or_else(|| LoopContractError::OrphanCriterion(criterion.clone()))?;
        if !node_completion_ready_unchecked(graph, node) {
            return Err(LoopContractError::MissingVerifierEvidence(
                criterion.clone(),
            ));
        }
    }
    let passed = as_set(&graph.passed_criteria);
    let all_passed = passed == required;
    if (graph.state == LoopState::Complete) != all_passed {
        return Err(LoopContractError::InvalidState(
            "complete must exactly equal all required criteria passed".to_owned(),
        ));
    }
    match graph.state {
        LoopState::Blocked | LoopState::Failed if graph.terminal_reason.is_none() => {
            Err(LoopContractError::InvalidState(
                "blocked and failed require a terminal reason".to_owned(),
            ))
        }
        LoopState::Active | LoopState::Complete if graph.terminal_reason.is_some() => {
            Err(LoopContractError::InvalidState(
                "active and complete forbid a terminal reason".to_owned(),
            ))
        }
        _ => Ok(()),
    }
}

fn retry_decision_unchecked(graph: &LoopGraph, node: &LoopNode) -> RetryDecision {
    let history = graph
        .attempts
        .iter()
        .filter(|attempt| attempt.node_id == node.id)
        .collect::<Vec<_>>();
    retry_decision_from_history(node, &history)
}

fn retry_decision_from_history(node: &LoopNode, history: &[&NodeAttempt]) -> RetryDecision {
    let Some(last) = history.last() else {
        return RetryDecision::FirstAttempt { attempt: 1 };
    };
    if last.outcome == AttemptOutcome::Succeeded {
        return RetryDecision::StopSucceeded;
    }
    let fingerprint = last
        .failure_fingerprint
        .as_deref()
        .expect("validated failure has a fingerprint");
    let occurrences = history
        .iter()
        .rev()
        .take_while(|attempt| attempt.failure_fingerprint.as_deref() == Some(fingerprint))
        .count();
    let occurrences = u32::try_from(occurrences).unwrap_or(u32::MAX);
    if occurrences >= node.retry_policy.identical_failure_limit {
        return RetryDecision::StopRepeatedFailure {
            fingerprint: fingerprint.to_owned(),
            occurrences,
        };
    }
    let attempts = u32::try_from(history.len()).unwrap_or(u32::MAX);
    if attempts >= node.retry_policy.max_attempts {
        return RetryDecision::StopBudgetExhausted;
    }
    RetryDecision::Retry {
        attempt: attempts + 1,
        backoff_seconds: bounded_backoff(&node.retry_policy, attempts),
    }
}

fn bounded_backoff(policy: &RetryPolicy, failed_attempts: u32) -> u64 {
    let mut delay = policy
        .initial_backoff_seconds
        .min(policy.max_backoff_seconds);
    for _ in 1..failed_attempts {
        delay = delay
            .saturating_mul(u64::from(policy.backoff_multiplier))
            .min(policy.max_backoff_seconds);
    }
    delay
}

fn dependencies_ready_unchecked(graph: &LoopGraph, node_id: &str) -> bool {
    let incoming = graph
        .edges
        .iter()
        .filter(|edge| edge.to == node_id)
        .collect::<Vec<_>>();
    if incoming.is_empty() {
        return graph.entry_nodes.iter().any(|entry| entry == node_id);
    }
    incoming
        .into_iter()
        .all(|edge| edge_ready_unchecked(graph, edge))
}

fn edge_ready_unchecked(graph: &LoopGraph, edge: &LoopEdge) -> bool {
    let Some(node) = graph.nodes.iter().find(|node| node.id == edge.from) else {
        return false;
    };
    let Some(attempt) = successful_attempt(graph, node) else {
        return false;
    };
    node_completion_ready_unchecked(graph, node)
        && edge.predicates.iter().all(|predicate| {
            predicate_satisfied(graph, predicate, Some(&edge.from), Some(attempt.attempt))
        })
}

fn node_completion_ready_unchecked(graph: &LoopGraph, node: &LoopNode) -> bool {
    let Some(successful_attempt) = successful_attempt(graph, node) else {
        return false;
    };
    if !node.completion_predicates.iter().all(|predicate| {
        predicate_satisfied(
            graph,
            predicate,
            Some(&node.id),
            Some(successful_attempt.attempt),
        )
    }) {
        return false;
    }
    graph.evidence.iter().any(|evidence| {
        evidence.kind == EvidenceKind::IndependentVerification
            && evidence.result == EvidenceResult::Passed
            && evidence.subject_node_id.as_deref() == Some(node.id.as_str())
            && evidence.attempt == Some(successful_attempt.attempt)
            && evidence.producer_role_id.as_deref() == Some(node.executor_role_id.as_str())
            && evidence.verifier_role_id.as_deref() == Some(node.verifier_role_id.as_str())
            && evidence.producer_role_id != evidence.verifier_role_id
            && evidence.verification_authority.is_some()
            && (evidence.verification_authority != Some(VerificationAuthority::Judge)
                || evidence.authenticated)
    })
}

fn predicate_satisfied(
    graph: &LoopGraph,
    predicate: &EvidencePredicate,
    subject_node_id: Option<&str>,
    attempt: Option<u32>,
) -> bool {
    graph.evidence.iter().any(|evidence| {
        evidence.id == predicate.evidence_id
            && evidence.kind == predicate.kind
            && evidence.result == predicate.result
            && evidence.subject_node_id.as_deref() == subject_node_id
            && evidence.attempt == attempt
            && evidence.graph_revision <= graph.revision
    })
}

fn successful_attempt<'a>(graph: &'a LoopGraph, node: &LoopNode) -> Option<&'a NodeAttempt> {
    graph
        .attempts
        .iter()
        .find(|attempt| attempt.node_id == node.id && attempt.outcome == AttemptOutcome::Succeeded)
}

fn find_attempt<'a>(graph: &'a LoopGraph, node_id: &str, attempt: u32) -> Option<&'a NodeAttempt> {
    graph
        .attempts
        .iter()
        .find(|item| item.node_id == node_id && item.attempt == attempt)
}

fn changed_edge_ids(previous: &[LoopEdge], next: &[LoopEdge]) -> Vec<String> {
    let previous = previous
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    let next = next
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    previous
        .keys()
        .chain(next.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|edge_id| previous.get(edge_id) != next.get(edge_id))
        .map(str::to_owned)
        .collect()
}

fn require_prefix<T: PartialEq>(
    previous: &[T],
    next: &[T],
    label: &'static str,
) -> Result<(), LoopContractError> {
    if next.len() < previous.len() || next.get(..previous.len()) != Some(previous) {
        return Err(LoopContractError::HistoryRegression(label));
    }
    Ok(())
}

fn invalid_evidence(evidence: &LoopEvidence, reason: &str) -> LoopContractError {
    LoopContractError::InvalidEvidence {
        evidence_id: evidence.id.clone(),
        reason: reason.to_owned(),
    }
}

fn as_set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

fn validate_body(body: &[u8]) -> Result<(), LoopContractError> {
    if body.len() > MAX_LOOP_BODY_BYTES {
        return Err(LoopContractError::TooLarge("loop graph Markdown body"));
    }
    std::str::from_utf8(body)
        .map(|_| ())
        .map_err(|_| LoopContractError::InvalidUtf8("loop graph Markdown body"))
}

fn split_frontmatter(bytes: &[u8]) -> Result<(&[u8], &[u8]), LoopContractError> {
    let (remainder, delimiter) = if let Some(remainder) = bytes.strip_prefix(b"---\n") {
        (remainder, b"\n---\n".as_slice())
    } else if let Some(remainder) = bytes.strip_prefix(b"---\r\n") {
        (remainder, b"\r\n---\r\n".as_slice())
    } else {
        return Err(LoopContractError::Malformed(
            "loop graph frontmatter start is missing".to_owned(),
        ));
    };
    let index = remainder
        .windows(delimiter.len())
        .position(|window| window == delimiter)
        .ok_or_else(|| {
            LoopContractError::Malformed("loop graph frontmatter end is missing".to_owned())
        })?;
    Ok((&remainder[..index], &remainder[index + delimiter.len()..]))
}

/// Pure loop graph contract errors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LoopContractError {
    /// A bounded graph artifact exceeded its maximum size.
    TooLarge(&'static str),
    /// A Markdown component is not UTF-8.
    InvalidUtf8(&'static str),
    /// YAML, JSON, or typed data is malformed.
    Malformed(String),
    /// Data violates an embedded JSON Schema.
    Schema(String),
    /// Stable ids collide within one collection.
    DuplicateIdentifier {
        /// Collection containing the duplicate.
        field: &'static str,
        /// Duplicated id.
        value: String,
    },
    /// A referenced node does not exist.
    UnknownNode(String),
    /// A referenced edge does not exist.
    UnknownEdge(String),
    /// An edge points from a node to itself.
    SelfEdge(String),
    /// The directed graph contains a cycle.
    Cycle,
    /// A node cannot be reached from any explicit entry.
    UnreachableNode(String),
    /// An entry node has an incoming dependency.
    EntryHasIncomingEdge(String),
    /// A node or pass references a criterion outside the required set.
    UnknownCriterion(String),
    /// A required criterion has no owning node.
    OrphanCriterion(String),
    /// More than one node owns the same criterion.
    CriterionAssignedMultipleTimes {
        /// Duplicated criterion.
        criterion: String,
        /// First owner.
        first_node: String,
        /// Second owner.
        second_node: String,
    },
    /// Executor and verifier roles are not independent.
    ExecutorVerifierConflict(String),
    /// Retry parameters are internally contradictory.
    InvalidRetryPolicy(String),
    /// Attempt history is non-contiguous or violates early-stop semantics.
    InvalidAttempt {
        /// Subject node.
        node_id: String,
        /// Exact violation.
        reason: String,
    },
    /// Evidence fields or bindings are contradictory.
    InvalidEvidence {
        /// Subject evidence id.
        evidence_id: String,
        /// Exact violation.
        reason: String,
    },
    /// A passed criterion lacks independently bound PASS evidence.
    MissingVerifierEvidence(String),
    /// Terminal state fields or completion semantics are contradictory.
    InvalidState(String),
    /// A required host capability is unavailable or insufficient.
    HostCapabilityUnsupported {
        /// Node requiring the capability.
        node_id: String,
        /// Provider-neutral capability name.
        capability: String,
        /// Observed support, or none when absent from the snapshot.
        support: Option<CapabilitySupportLevel>,
    },
    /// A prepare-only dispatch is stale, unauthorized, or incorrectly bound.
    InvalidDispatchBinding(String),
    /// Steering history or authority binding is invalid.
    InvalidSteering(String),
    /// A non-idempotent transition has an unexpected revision.
    InvalidRevision {
        /// Required next revision.
        expected: u64,
        /// Observed candidate revision.
        actual: u64,
    },
    /// Immediate previous canonical digest does not match.
    PreviousDigestMismatch,
    /// Schema version or run identity changed mid-graph.
    RunIdentityChanged,
    /// Immutable entries, criteria, nodes, or capabilities changed.
    GraphDefinitionChanged,
    /// Append-only data regressed or was rewritten.
    HistoryRegression(&'static str),
    /// Edge topology changed without exactly one bound steering record.
    TopologyChangedWithoutSteering,
    /// A steering record was appended without a topology change.
    UnnecessarySteering,
    /// A terminal blocked, failed, or complete graph was mutated.
    TerminalStateImmutable,
}

impl Display for LoopContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge(label) => write!(formatter, "{label} exceeds the bounded contract"),
            Self::InvalidUtf8(label) => write!(formatter, "{label} must be UTF-8"),
            Self::Malformed(message) | Self::Schema(message) => formatter.write_str(message),
            Self::DuplicateIdentifier { field, value } => {
                write!(formatter, "{field} contains duplicate id: {value}")
            }
            Self::UnknownNode(node) => write!(formatter, "unknown loop node: {node}"),
            Self::UnknownEdge(edge) => write!(formatter, "unknown loop edge: {edge}"),
            Self::SelfEdge(edge) => write!(formatter, "loop edge is self-referential: {edge}"),
            Self::Cycle => formatter.write_str("loop graph must be acyclic"),
            Self::UnreachableNode(node) => write!(formatter, "unreachable loop node: {node}"),
            Self::EntryHasIncomingEdge(node) => {
                write!(formatter, "entry node has an incoming edge: {node}")
            }
            Self::UnknownCriterion(criterion) => {
                write!(formatter, "unknown loop criterion: {criterion}")
            }
            Self::OrphanCriterion(criterion) => {
                write!(formatter, "orphan loop criterion: {criterion}")
            }
            Self::CriterionAssignedMultipleTimes {
                criterion,
                first_node,
                second_node,
            } => write!(
                formatter,
                "criterion {criterion} is owned by both {first_node} and {second_node}"
            ),
            Self::ExecutorVerifierConflict(node) => write!(
                formatter,
                "node executor and verifier must be independent: {node}"
            ),
            Self::InvalidRetryPolicy(node) => {
                write!(formatter, "invalid retry policy for node: {node}")
            }
            Self::InvalidAttempt { node_id, reason } => {
                write!(formatter, "invalid attempt for {node_id}: {reason}")
            }
            Self::InvalidEvidence {
                evidence_id,
                reason,
            } => write!(formatter, "invalid evidence {evidence_id}: {reason}"),
            Self::MissingVerifierEvidence(criterion) => write!(
                formatter,
                "criterion lacks independent verifier PASS evidence: {criterion}"
            ),
            Self::InvalidState(reason) => write!(formatter, "invalid loop state: {reason}"),
            Self::HostCapabilityUnsupported {
                node_id,
                capability,
                support,
            } => write!(
                formatter,
                "host_capability_unsupported: node={node_id} capability={capability} support={support:?}"
            ),
            Self::InvalidDispatchBinding(reason) => {
                write!(formatter, "invalid loop dispatch binding: {reason}")
            }
            Self::InvalidSteering(reason) => write!(formatter, "invalid steering: {reason}"),
            Self::InvalidRevision { expected, actual } => {
                write!(formatter, "expected graph revision {expected}, found {actual}")
            }
            Self::PreviousDigestMismatch => {
                formatter.write_str("previous graph revision digest mismatch")
            }
            Self::RunIdentityChanged => formatter.write_str("loop run identity changed"),
            Self::GraphDefinitionChanged => formatter.write_str(
                "loop entries, criteria, nodes, and capability snapshot are immutable",
            ),
            Self::HistoryRegression(label) => {
                write!(formatter, "append-only loop history regressed: {label}")
            }
            Self::TopologyChangedWithoutSteering => formatter
                .write_str("loop topology changed without one exact steering revision"),
            Self::UnnecessarySteering => {
                formatter.write_str("steering revision has no topology change")
            }
            Self::TerminalStateImmutable => {
                formatter.write_str("terminal loop graph state is immutable")
            }
        }
    }
}

impl Error for LoopContractError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(label: &str) -> String {
        sha256_digest(label.as_bytes())
    }

    fn predicate(
        evidence_id: &str,
        kind: EvidenceKind,
        result: EvidenceResult,
    ) -> EvidencePredicate {
        EvidencePredicate {
            evidence_id: evidence_id.to_owned(),
            kind,
            result,
        }
    }

    fn retry_policy() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 4,
            initial_backoff_seconds: 2,
            backoff_multiplier: 3,
            max_backoff_seconds: 10,
            identical_failure_limit: 2,
        }
    }

    fn node(
        id: &str,
        executor: &str,
        verifier: &str,
        criterion: Option<&str>,
        artifact_id: &str,
    ) -> LoopNode {
        LoopNode {
            id: id.to_owned(),
            executor_role_id: executor.to_owned(),
            verifier_role_id: verifier.to_owned(),
            criterion_ids: criterion.into_iter().map(str::to_owned).collect(),
            completion_predicates: vec![predicate(
                artifact_id,
                EvidenceKind::Artifact,
                EvidenceResult::Present,
            )],
            required_capabilities: vec![CapabilityRequirement {
                name: "subagents".to_owned(),
                minimum_support: MinimumSupport::Supported,
            }],
            retry_policy: retry_policy(),
        }
    }

    fn usage(id: &str, node_id: Option<&str>, attempt: Option<u32>, revision: u64) -> LoopEvidence {
        LoopEvidence {
            id: id.to_owned(),
            kind: EvidenceKind::UsageAuthorization,
            result: EvidenceResult::Allowed,
            graph_revision: revision,
            subject_node_id: node_id.map(str::to_owned),
            attempt,
            producer_role_id: None,
            verifier_role_id: None,
            verification_authority: None,
            locator: format!("usage:{id}"),
            digest: digest(id),
            authenticated: false,
        }
    }

    fn artifact(
        id: &str,
        node_id: &str,
        attempt: u32,
        revision: u64,
        producer: &str,
    ) -> LoopEvidence {
        LoopEvidence {
            id: id.to_owned(),
            kind: EvidenceKind::Artifact,
            result: EvidenceResult::Present,
            graph_revision: revision,
            subject_node_id: Some(node_id.to_owned()),
            attempt: Some(attempt),
            producer_role_id: Some(producer.to_owned()),
            verifier_role_id: None,
            verification_authority: None,
            locator: format!("artifact:{id}"),
            digest: digest(id),
            authenticated: false,
        }
    }

    fn verification(
        id: &str,
        node_id: &str,
        attempt: u32,
        revision: u64,
        producer: &str,
        verifier: &str,
        proof: (VerificationAuthority, bool),
    ) -> LoopEvidence {
        LoopEvidence {
            id: id.to_owned(),
            kind: EvidenceKind::IndependentVerification,
            result: EvidenceResult::Passed,
            graph_revision: revision,
            subject_node_id: Some(node_id.to_owned()),
            attempt: Some(attempt),
            producer_role_id: Some(producer.to_owned()),
            verifier_role_id: Some(verifier.to_owned()),
            verification_authority: Some(proof.0),
            locator: format!("verification:{id}"),
            digest: digest(id),
            authenticated: proof.1,
        }
    }

    fn steering_approval(id: &str, revision: u64) -> LoopEvidence {
        LoopEvidence {
            id: id.to_owned(),
            kind: EvidenceKind::SteeringAuthorization,
            result: EvidenceResult::Allowed,
            graph_revision: revision,
            subject_node_id: None,
            attempt: None,
            producer_role_id: None,
            verifier_role_id: None,
            verification_authority: None,
            locator: format!("approval:{id}"),
            digest: digest(id),
            authenticated: true,
        }
    }

    fn attempt(
        node_id: &str,
        number: u32,
        revision: u64,
        outcome: AttemptOutcome,
        fingerprint: Option<&str>,
    ) -> NodeAttempt {
        NodeAttempt {
            node_id: node_id.to_owned(),
            attempt: number,
            graph_revision: revision,
            outcome,
            dispatch_digest: digest(&format!("dispatch-{node_id}-{number}")),
            failure_fingerprint: fingerprint.map(digest),
        }
    }

    fn base_graph() -> LoopGraph {
        LoopGraph {
            schema_version: 1,
            run_id: "loop-run".to_owned(),
            revision: 1,
            previous_revision_digest: None,
            state: LoopState::Active,
            terminal_reason: None,
            entry_nodes: vec!["A".to_owned()],
            required_criteria: vec!["C1".to_owned(), "C2".to_owned()],
            passed_criteria: Vec::new(),
            nodes: vec![
                node("A", "exec-a", "verify-a", Some("C1"), "artifact-a"),
                node("B", "exec-b", "verify-b", Some("C2"), "artifact-b"),
            ],
            edges: vec![LoopEdge {
                id: "edge-a-b".to_owned(),
                from: "A".to_owned(),
                to: "B".to_owned(),
                predicates: vec![predicate(
                    "verify-a",
                    EvidenceKind::IndependentVerification,
                    EvidenceResult::Passed,
                )],
            }],
            evidence: vec![usage("usage-a-1", Some("A"), Some(1), 1)],
            attempts: Vec::new(),
            capability_support: vec![
                CapabilitySupport {
                    name: "subagents".to_owned(),
                    support: CapabilitySupportLevel::Supported,
                    evidence_digest: digest("capability-subagents"),
                },
                CapabilitySupport {
                    name: "independent-judge".to_owned(),
                    support: CapabilitySupportLevel::Supported,
                    evidence_digest: digest("capability-judge"),
                },
            ],
            steering: Vec::new(),
        }
    }

    fn document(graph: LoopGraph) -> LoopGraphDocument {
        LoopGraphDocument::from_graph(graph, b"# Durable loop\n".to_vec())
            .expect("test graph should be valid")
    }

    fn complete_a(previous: &LoopGraphDocument) -> LoopGraphDocument {
        let mut graph = previous.graph().clone();
        graph.revision += 1;
        graph.previous_revision_digest = Some(
            previous
                .canonical_digest()
                .expect("previous graph should encode"),
        );
        graph.attempts.push(attempt(
            "A",
            1,
            graph.revision,
            AttemptOutcome::Succeeded,
            None,
        ));
        graph
            .evidence
            .push(artifact("artifact-a", "A", 1, graph.revision, "exec-a"));
        graph.evidence.push(verification(
            "verify-a",
            "A",
            1,
            graph.revision,
            "exec-a",
            "verify-a",
            (VerificationAuthority::Deterministic, false),
        ));
        graph
            .evidence
            .push(usage("usage-b-1", Some("B"), Some(1), graph.revision));
        graph.passed_criteria.push("C1".to_owned());
        document(graph)
    }

    fn complete_b(previous: &LoopGraphDocument) -> LoopGraphDocument {
        let mut graph = previous.graph().clone();
        graph.revision += 1;
        graph.previous_revision_digest = Some(
            previous
                .canonical_digest()
                .expect("previous graph should encode"),
        );
        graph.attempts.push(attempt(
            "B",
            1,
            graph.revision,
            AttemptOutcome::Succeeded,
            None,
        ));
        graph
            .evidence
            .push(artifact("artifact-b", "B", 1, graph.revision, "exec-b"));
        graph.evidence.push(verification(
            "verify-b",
            "B",
            1,
            graph.revision,
            "exec-b",
            "verify-b",
            (VerificationAuthority::Judge, true),
        ));
        graph.passed_criteria.push("C2".to_owned());
        graph.state = LoopState::Complete;
        document(graph)
    }

    #[test]
    fn validates_dag_and_rejects_self_edges() {
        let graph = base_graph();
        assert_eq!(graph.validate(), Ok(()));

        let mut hostile = graph;
        hostile.edges[0].from = "B".to_owned();
        assert_eq!(
            hostile.validate(),
            Err(LoopContractError::SelfEdge("edge-a-b".to_owned()))
        );
    }

    #[test]
    fn rejects_cycles() {
        let mut graph = base_graph();
        graph
            .nodes
            .push(node("C", "exec-c", "verify-c", None, "artifact-c"));
        graph.edges.push(LoopEdge {
            id: "edge-b-c".to_owned(),
            from: "B".to_owned(),
            to: "C".to_owned(),
            predicates: vec![predicate(
                "verify-b",
                EvidenceKind::IndependentVerification,
                EvidenceResult::Passed,
            )],
        });
        graph.edges.push(LoopEdge {
            id: "edge-c-b".to_owned(),
            from: "C".to_owned(),
            to: "B".to_owned(),
            predicates: vec![predicate(
                "verify-c",
                EvidenceKind::IndependentVerification,
                EvidenceResult::Passed,
            )],
        });

        assert_eq!(graph.validate(), Err(LoopContractError::Cycle));
    }

    #[test]
    fn rejects_unreachable_nodes() {
        let mut graph = base_graph();
        graph
            .nodes
            .push(node("C", "exec-c", "verify-c", None, "artifact-c"));

        assert_eq!(
            graph.validate(),
            Err(LoopContractError::UnreachableNode("C".to_owned()))
        );
    }

    #[test]
    fn rejects_orphan_and_multiply_owned_criteria() {
        let mut orphan = base_graph();
        orphan.nodes[1].criterion_ids.clear();
        assert_eq!(
            orphan.validate(),
            Err(LoopContractError::OrphanCriterion("C2".to_owned()))
        );

        let mut duplicate = base_graph();
        duplicate.nodes[1].criterion_ids.push("C1".to_owned());
        assert!(matches!(
            duplicate.validate(),
            Err(LoopContractError::CriterionAssignedMultipleTimes { .. })
        ));
    }

    #[test]
    fn evidence_predicates_and_independent_verification_gate_progress() {
        let initial = document(base_graph());
        assert_eq!(initial.graph().edge_ready("edge-a-b"), Ok(false));
        let mut graph = initial.graph().clone();
        graph.revision = 2;
        graph.previous_revision_digest = Some(
            initial
                .canonical_digest()
                .expect("initial graph should encode"),
        );
        graph
            .attempts
            .push(attempt("A", 1, 2, AttemptOutcome::Succeeded, None));
        graph.passed_criteria.push("C1".to_owned());
        assert_eq!(
            LoopGraphDocument::from_graph(graph.clone(), Vec::new()),
            Err(LoopContractError::MissingVerifierEvidence("C1".to_owned()))
        );

        graph
            .evidence
            .push(artifact("artifact-a", "A", 1, 2, "exec-a"));
        graph.evidence.push(verification(
            "verify-a",
            "A",
            1,
            2,
            "exec-a",
            "exec-a",
            (VerificationAuthority::Deterministic, false),
        ));
        assert!(matches!(
            graph.validate(),
            Err(LoopContractError::InvalidEvidence { .. })
        ));

        let completed = complete_a(&initial);
        assert_eq!(completed.graph().edge_ready("edge-a-b"), Ok(true));
        assert_eq!(completed.graph().ready_nodes(), Ok(vec!["B".to_owned()]));
        assert_eq!(
            validate_loop_transition(&initial, &completed),
            Ok(LoopTransitionOutcome::Advance)
        );
    }

    #[test]
    fn unauthenticated_judge_evidence_is_rejected() {
        let mut graph = base_graph();
        graph
            .attempts
            .push(attempt("A", 1, 1, AttemptOutcome::Succeeded, None));
        graph
            .evidence
            .push(artifact("artifact-a", "A", 1, 1, "exec-a"));
        graph.evidence.push(verification(
            "verify-a",
            "A",
            1,
            1,
            "exec-a",
            "verify-a",
            (VerificationAuthority::Judge, false),
        ));

        assert!(matches!(
            graph.validate(),
            Err(LoopContractError::InvalidEvidence { .. })
        ));
    }

    #[test]
    fn completion_predicates_bind_the_successful_attempt() {
        let mut graph = base_graph();
        graph
            .evidence
            .push(usage("usage-a-2", Some("A"), Some(2), 1));
        graph
            .attempts
            .push(attempt("A", 1, 1, AttemptOutcome::Failed, Some("first")));
        graph
            .attempts
            .push(attempt("A", 2, 1, AttemptOutcome::Succeeded, None));
        graph
            .evidence
            .push(artifact("artifact-a", "A", 1, 1, "exec-a"));
        graph.evidence.push(verification(
            "verify-a",
            "A",
            2,
            1,
            "exec-a",
            "verify-a",
            (VerificationAuthority::Deterministic, false),
        ));
        graph.passed_criteria.push("C1".to_owned());

        assert_eq!(
            graph.validate(),
            Err(LoopContractError::MissingVerifierEvidence("C1".to_owned()))
        );
    }

    #[test]
    fn retries_calculate_capped_backoff_and_stop_on_fingerprint() {
        let mut graph = base_graph();
        graph
            .evidence
            .push(usage("usage-a-2", Some("A"), Some(2), 1));
        graph
            .attempts
            .push(attempt("A", 1, 1, AttemptOutcome::Failed, Some("same")));
        assert_eq!(
            graph.retry_decision("A"),
            Ok(RetryDecision::Retry {
                attempt: 2,
                backoff_seconds: 2,
            })
        );
        graph
            .attempts
            .push(attempt("A", 2, 1, AttemptOutcome::Failed, Some("same")));
        assert_eq!(
            graph.retry_decision("A"),
            Ok(RetryDecision::StopRepeatedFailure {
                fingerprint: digest("same"),
                occurrences: 2,
            })
        );

        graph
            .evidence
            .push(usage("usage-a-3", Some("A"), Some(3), 1));
        graph
            .attempts
            .push(attempt("A", 3, 1, AttemptOutcome::Failed, Some("same")));
        assert!(matches!(
            graph.validate(),
            Err(LoopContractError::InvalidAttempt { .. })
        ));
    }

    #[test]
    fn retry_backoff_saturates_at_the_policy_cap_and_budget() {
        let mut graph = base_graph();
        for number in 2..=4 {
            graph.evidence.push(usage(
                &format!("usage-a-{number}"),
                Some("A"),
                Some(number),
                1,
            ));
        }
        for (number, fingerprint) in [(1, "one"), (2, "two"), (3, "three")] {
            graph.attempts.push(attempt(
                "A",
                number,
                1,
                AttemptOutcome::Failed,
                Some(fingerprint),
            ));
        }
        assert_eq!(
            graph.retry_decision("A"),
            Ok(RetryDecision::Retry {
                attempt: 4,
                backoff_seconds: 10,
            })
        );
        graph
            .attempts
            .push(attempt("A", 4, 1, AttemptOutcome::Failed, Some("four")));
        assert_eq!(
            graph.retry_decision("A"),
            Ok(RetryDecision::StopBudgetExhausted)
        );
    }

    #[test]
    fn attempts_require_exact_usage_authorization() {
        let mut graph = base_graph();
        graph.evidence.clear();
        graph
            .attempts
            .push(attempt("A", 1, 1, AttemptOutcome::Failed, Some("failure")));

        assert!(matches!(
            graph.validate(),
            Err(LoopContractError::InvalidAttempt { reason, .. })
                if reason.contains("usage authorization")
        ));
    }

    #[test]
    fn dispatch_binding_pins_graph_usage_capability_role_and_attempt() {
        let document = document(base_graph());
        let binding = LoopDispatchBinding {
            schema_version: 1,
            run_id: document.graph().run_id.clone(),
            graph_revision: document.graph().revision,
            graph_digest: document
                .canonical_digest()
                .expect("graph should have a digest"),
            kind: LoopDispatchKind::Node,
            node_id: Some("A".to_owned()),
            attempt: Some(1),
            role_id: Some("exec-a".to_owned()),
            brief_digest: digest("brief-a-1"),
            capability_snapshot_digest: document
                .graph()
                .capability_snapshot_digest()
                .expect("capability snapshot should encode"),
            usage_evidence_id: "usage-a-1".to_owned(),
            prepared_only: true,
        };
        assert_eq!(document.validate_dispatch(&binding), Ok(()));

        let mut stale = binding.clone();
        stale.graph_digest = digest("stale-graph");
        assert!(matches!(
            document.validate_dispatch(&stale),
            Err(LoopContractError::InvalidDispatchBinding(_))
        ));

        let mut wrong_attempt = binding;
        wrong_attempt.attempt = Some(2);
        assert!(matches!(
            document.validate_dispatch(&wrong_attempt),
            Err(LoopContractError::InvalidDispatchBinding(_))
        ));
    }

    #[test]
    fn retry_dispatch_requires_fresh_attempt_bound_usage_evidence() {
        let mut graph = base_graph();
        graph
            .evidence
            .push(usage("usage-a-2", Some("A"), Some(2), 1));
        graph
            .attempts
            .push(attempt("A", 1, 1, AttemptOutcome::Failed, Some("failure")));
        let document = document(graph);
        let binding = LoopDispatchBinding {
            schema_version: 1,
            run_id: document.graph().run_id.clone(),
            graph_revision: 1,
            graph_digest: document
                .canonical_digest()
                .expect("graph should have a digest"),
            kind: LoopDispatchKind::Retry,
            node_id: Some("A".to_owned()),
            attempt: Some(2),
            role_id: Some("exec-a".to_owned()),
            brief_digest: digest("brief-a-2"),
            capability_snapshot_digest: document
                .graph()
                .capability_snapshot_digest()
                .expect("capability snapshot should encode"),
            usage_evidence_id: "usage-a-2".to_owned(),
            prepared_only: true,
        };
        assert_eq!(document.validate_dispatch(&binding), Ok(()));

        let mut reused = binding;
        reused.usage_evidence_id = "usage-a-1".to_owned();
        assert!(matches!(
            document.validate_dispatch(&reused),
            Err(LoopContractError::InvalidDispatchBinding(_))
        ));
    }

    #[test]
    fn steering_dispatch_requires_current_graph_scoped_usage_evidence() {
        let mut graph = base_graph();
        graph.evidence.push(usage("usage-steering", None, None, 1));
        let document = document(graph);
        let binding = LoopDispatchBinding {
            schema_version: 1,
            run_id: document.graph().run_id.clone(),
            graph_revision: 1,
            graph_digest: document
                .canonical_digest()
                .expect("graph should have a digest"),
            kind: LoopDispatchKind::Steering,
            node_id: None,
            attempt: None,
            role_id: None,
            brief_digest: digest("steering-brief"),
            capability_snapshot_digest: document
                .graph()
                .capability_snapshot_digest()
                .expect("capability snapshot should encode"),
            usage_evidence_id: "usage-steering".to_owned(),
            prepared_only: true,
        };
        assert_eq!(document.validate_dispatch(&binding), Ok(()));

        let mut node_scoped = binding;
        node_scoped.usage_evidence_id = "usage-a-1".to_owned();
        assert!(matches!(
            document.validate_dispatch(&node_scoped),
            Err(LoopContractError::InvalidDispatchBinding(_))
        ));
    }

    #[test]
    fn unsupported_required_capability_is_explicit() {
        let mut graph = base_graph();
        graph.capability_support[0].support = CapabilitySupportLevel::Unsupported;
        assert_eq!(graph.validate(), Ok(()));
        let error = graph
            .validate_node_capabilities("A")
            .expect_err("unsupported capability must stop dispatch");
        assert!(matches!(
            error,
            LoopContractError::HostCapabilityUnsupported {
                support: Some(CapabilitySupportLevel::Unsupported),
                ..
            }
        ));
        assert!(error.to_string().contains("host_capability_unsupported"));
    }

    #[test]
    fn topology_changes_require_exact_revision_bound_steering() {
        let previous = document(base_graph());
        let previous_digest = previous
            .canonical_digest()
            .expect("previous graph should encode");
        let mut graph = previous.graph().clone();
        graph.revision = 2;
        graph.previous_revision_digest = Some(previous_digest.clone());
        graph.edges[0].predicates[0] = predicate(
            "artifact-a",
            EvidenceKind::Artifact,
            EvidenceResult::Present,
        );
        graph.steering.push(SteeringRecord {
            base_revision: 1,
            base_revision_digest: previous_digest,
            reason: "Use the deterministic artifact boundary".to_owned(),
            affected_edges: vec!["edge-a-b".to_owned()],
            user_boundary: UserBoundary::WithinApprovedScope,
            authorization_evidence_id: None,
        });
        let next = document(graph.clone());
        assert_eq!(
            validate_loop_transition(&previous, &next),
            Ok(LoopTransitionOutcome::Advance)
        );

        graph.steering.clear();
        let unsteered = document(graph);
        assert_eq!(
            validate_loop_transition(&previous, &unsteered),
            Err(LoopContractError::TopologyChangedWithoutSteering)
        );
    }

    #[test]
    fn expanded_steering_requires_new_authenticated_user_evidence() {
        let previous = document(base_graph());
        let previous_digest = previous
            .canonical_digest()
            .expect("previous graph should encode");
        let mut graph = previous.graph().clone();
        graph.revision = 2;
        graph.previous_revision_digest = Some(previous_digest.clone());
        graph.edges[0].predicates[0] = predicate(
            "artifact-a",
            EvidenceKind::Artifact,
            EvidenceResult::Present,
        );
        graph.evidence.push(steering_approval("approval-1", 2));
        graph.steering.push(SteeringRecord {
            base_revision: 1,
            base_revision_digest: previous_digest,
            reason: "User approved the changed evidence boundary".to_owned(),
            affected_edges: vec!["edge-a-b".to_owned()],
            user_boundary: UserBoundary::ExplicitUserApproval,
            authorization_evidence_id: Some("approval-1".to_owned()),
        });
        let next = document(graph);

        assert_eq!(
            validate_loop_transition(&previous, &next),
            Ok(LoopTransitionOutcome::Advance)
        );
    }

    #[test]
    fn blocked_failed_and_complete_are_terminal() {
        for state in [LoopState::Blocked, LoopState::Failed, LoopState::Complete] {
            assert!(state.is_terminal());
        }

        let active = document(base_graph());
        let mut blocked_graph = active.graph().clone();
        blocked_graph.revision = 2;
        blocked_graph.previous_revision_digest = Some(
            active
                .canonical_digest()
                .expect("active graph should encode"),
        );
        blocked_graph.state = LoopState::Blocked;
        blocked_graph.terminal_reason = Some("host authority unavailable".to_owned());
        let blocked = document(blocked_graph);
        assert_eq!(
            validate_loop_transition(&active, &blocked),
            Ok(LoopTransitionOutcome::Advance)
        );

        let mut changed = blocked.graph().clone();
        changed.revision = 3;
        changed.previous_revision_digest = Some(
            blocked
                .canonical_digest()
                .expect("blocked graph should encode"),
        );
        changed.terminal_reason = Some("different reason".to_owned());
        let changed = document(changed);
        assert_eq!(
            validate_loop_transition(&blocked, &changed),
            Err(LoopContractError::TerminalStateImmutable)
        );

        let mut failed_graph = active.graph().clone();
        failed_graph.revision = 2;
        failed_graph.previous_revision_digest = Some(
            active
                .canonical_digest()
                .expect("active graph should encode"),
        );
        failed_graph.state = LoopState::Failed;
        failed_graph.terminal_reason = Some("retry policy exhausted".to_owned());
        let failed = document(failed_graph);
        assert_eq!(
            validate_loop_transition(&active, &failed),
            Ok(LoopTransitionOutcome::Advance)
        );

        let completed_a = complete_a(&active);
        let complete = complete_b(&completed_a);
        assert_eq!(complete.graph().state, LoopState::Complete);
        assert_eq!(
            validate_loop_transition(&completed_a, &complete),
            Ok(LoopTransitionOutcome::Advance)
        );
    }

    #[test]
    fn fresh_session_recovery_is_byte_and_digest_deterministic() {
        let initial = document(base_graph());
        let completed = complete_a(&initial);
        let encoded = completed
            .encode_canonical()
            .expect("graph should encode canonically");
        let first =
            LoopGraphDocument::parse_markdown(&encoded).expect("fresh session should parse graph");
        let second = LoopGraphDocument::parse_markdown(&encoded)
            .expect("second fresh session should parse graph");

        assert_eq!(first, second);
        assert_eq!(first.body(), b"# Durable loop\n");
        assert_eq!(
            first.encode_canonical().expect("first should re-encode"),
            encoded
        );
        assert_eq!(
            first.canonical_digest().expect("first digest"),
            second.canonical_digest().expect("second digest")
        );
    }

    #[test]
    fn exact_same_revision_is_idempotent() {
        let document = document(base_graph());
        assert_eq!(
            validate_loop_transition(&document, &document),
            Ok(LoopTransitionOutcome::Idempotent)
        );
    }

    #[test]
    fn core_contains_no_process_scheduler_or_foreign_runtime_calls() {
        let source = include_str!("loop_graph.rs");
        for forbidden in [
            concat!("std::process", "::Command"),
            concat!("std::thread", "::sleep"),
            concat!("tokio", "::spawn"),
            concat!("tmux", " new-session"),
            concat!("omx", " run"),
            concat!("omc", " run"),
        ] {
            assert!(
                !source.contains(forbidden),
                "pure loop core contains forbidden runtime primitive: {forbidden}"
            );
        }
    }
}
