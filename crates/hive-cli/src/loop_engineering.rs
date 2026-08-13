//! Prepare-only durable loop graph commands.
//!
//! This adapter validates and checkpoints provider-neutral Markdown state. It
//! deliberately never starts a process, schedules work, waits, or selects an
//! external runtime.

use super::{emit_action_result, ActionResult, Evidence};
use crate::judge::verify_authenticated_loop_quorum;
use crate::run::{
    parse_options, portable_relative_path, read_explicit_file, required, run_path, AdapterError,
    FileSnapshot, PinnedTarget,
};
use crate::usage_control::read_installed_config;
use cap_fs_ext::DirExt;
use hive_core::loop_graph::{
    validate_loop_transition, CapabilitySupportLevel, EvidenceKind, EvidenceResult,
    LoopContractError, LoopDispatchBinding, LoopDispatchKind, LoopGraphDocument, LoopState,
    LoopTransitionOutcome, RetryDecision, UserBoundary, VerificationAuthority,
};
use hive_core::run::{
    verify_owner_continuity, CapabilityResolution, OwnerContinuity, RunPlan, RunState,
    RunStatusDocument,
};
use hive_core::{sha256_digest, validate_project_relative};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

const MAX_GRAPH_BYTES: usize = 1024 * 1024;
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_EVIDENCE_FILE_BYTES: usize = 256 * 1024;
const MAX_TOTAL_EVIDENCE_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOTAL_GRAPH_CHAIN_BYTES: usize = 32 * 1024 * 1024;
const MAX_PREPARED_BYTES: usize = 128 * 1024;
const MAX_GRAPH_REVISIONS: u64 = 4096;
const CAPABILITY_MAX_AGE_SECONDS: u64 = 60;
const USAGE_CONTROL_PATH_BYTES: usize = 16 * 1024;

const LOOP_USAGE: &str = "\
Manage a durable prepare-only loop graph.

USAGE:
    hive loop initialize --target <dir> --graph <graph.md> --output json
    hive loop validate --target <dir> --run <run-id> --output json
    hive loop checkpoint --target <dir> --request <request.json> --output json
    hive loop steer --target <dir> --request <request.json> --output json
    hive loop prepare --target <dir> --request <request.json> --output json
    hive loop recover --target <dir> --run <run-id> --output json
";

#[derive(Debug)]
enum LoopCliError {
    Adapter(AdapterError),
    Core(LoopContractError),
}

impl LoopCliError {
    fn status(&self) -> &'static str {
        match self {
            Self::Core(LoopContractError::HostCapabilityUnsupported { .. }) => "unsupported",
            Self::Core(_) => "verification-failed",
            Self::Adapter(error) => error.status(),
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Core(LoopContractError::HostCapabilityUnsupported { .. }) => 4,
            Self::Core(_) => 5,
            Self::Adapter(error) => error.exit_code(),
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Core(LoopContractError::HostCapabilityUnsupported { .. }) => {
                "host_capability_unsupported"
            }
            Self::Core(_) => "hive.loop-verification-failed",
            Self::Adapter(error) => error.code(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Adapter(error) => error.message().to_owned(),
            Self::Core(error) => error.to_string(),
        }
    }
}

impl From<AdapterError> for LoopCliError {
    fn from(error: AdapterError) -> Self {
        Self::Adapter(error)
    }
}

impl From<LoopContractError> for LoopCliError {
    fn from(error: LoopContractError) -> Self {
        Self::Core(error)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionRequest {
    schema_version: u32,
    run_id: String,
    expected_revision: u64,
    #[serde(alias = "expected_digest")]
    expected_graph_digest: String,
    candidate_graph: PathBuf,
    #[serde(default)]
    evidence_files: Vec<EvidenceFileRef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceFileRef {
    evidence_id: String,
    locator: String,
    digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrepareRequest {
    schema_version: u32,
    run_id: String,
    expected_revision: u64,
    #[serde(alias = "expected_digest")]
    expected_graph_digest: String,
    kind: LoopDispatchKind,
    #[serde(default)]
    node_id: Option<String>,
    brief_digest: String,
    usage_evidence_id: String,
    capability_resolution: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsageAuthorizationEnvelope {
    schema_version: u32,
    evidence_id: String,
    run_id: String,
    base_graph_revision: u64,
    base_graph_digest: String,
    authorized_graph_revision: u64,
    dispatch_kind: LoopDispatchKind,
    #[serde(default)]
    subject_node_id: Option<String>,
    #[serde(default)]
    attempt: Option<u32>,
    brief_digest: String,
    decision: EvidenceResult,
    authenticated: bool,
    dispatch_authorization_locator: String,
    dispatch_authorization_digest: String,
    usage_evidence_digest: String,
    session_id: String,
    session_id_digest: String,
    process_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifierEnvelope {
    schema_version: u32,
    evidence_id: String,
    run_id: String,
    graph_revision: u64,
    node_id: String,
    attempt: u32,
    producer_role_id: String,
    verifier_role_id: String,
    authority: VerificationAuthority,
    result: EvidenceResult,
    authenticated: bool,
    #[serde(default)]
    judge_quorum_request: Option<String>,
    #[serde(default)]
    judge_quorum_request_digest: Option<String>,
    #[serde(default)]
    judge_trust_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SteeringAuthorizationEnvelope {
    schema_version: u32,
    evidence_id: String,
    run_id: String,
    base_graph_revision: u64,
    base_graph_digest: String,
    authorized_graph_revision: u64,
    user_boundary: UserBoundary,
    result: EvidenceResult,
    authenticated: bool,
    proposal_digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreparedDispatchRecord {
    schema_version: u32,
    dispatch_id: String,
    binding: LoopDispatchBinding,
    prepared_only: bool,
    spawned: bool,
    dispatch_authorization_digest: String,
    capability_resolution: PathBuf,
    capability_resolution_digest: String,
    capability_resolution_file_digest: String,
    run_status_digest: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DispatchAuthorizationRecord {
    schema_version: u32,
    authorization_id: String,
    run_id: String,
    status_revision: u64,
    role_id: String,
    brief_digest: String,
    usage_evidence_digest: String,
    state: String,
    record_digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsageSessionControl {
    schema_version: u32,
    host_scope: String,
    session_id_digest: String,
    process_id: u32,
    guard_enabled: bool,
    revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsageHaltMarker {
    schema_version: u32,
    host_scope: String,
    session_id_digest: String,
    process_id: u32,
    decision: String,
    selected_window: String,
    threshold_remaining_percent: u8,
    measured_at: u64,
    evidence_digest: String,
    revision: u64,
}

struct LoadedGraphChain {
    documents: Vec<LoopGraphDocument>,
    current_bytes: Vec<u8>,
}

impl LoadedGraphChain {
    fn current(&self) -> &LoopGraphDocument {
        self.documents
            .last()
            .expect("validated graph chains are non-empty")
    }

    fn document_at(&self, revision: u64) -> Option<&LoopGraphDocument> {
        let index = usize::try_from(revision.checked_sub(1)?).ok()?;
        self.documents.get(index)
    }
}

pub(crate) fn run_loop(arguments: &[String]) -> ExitCode {
    if arguments.first().map(String::as_str) == Some("--help") {
        print!("{LOOP_USAGE}");
        return ExitCode::SUCCESS;
    }
    let action = match arguments.first().map(String::as_str) {
        Some("initialize") => "InitializeLoop",
        Some("validate") => "ValidateLoop",
        Some("checkpoint") => "CheckpointLoop",
        Some("steer") => "SteerLoop",
        Some("prepare") => "PrepareLoopDispatch",
        Some("recover") => "RecoverLoop",
        _ => "Loop",
    };
    let result = match arguments.first().map(String::as_str) {
        Some("initialize") => parse_initialize(&arguments[1..]).and_then(initialize),
        Some("validate") => parse_run_target(&arguments[1..]).and_then(validate),
        Some("checkpoint") => {
            parse_request_target(&arguments[1..]).and_then(|arguments| checkpoint(arguments, false))
        }
        Some("steer") => {
            parse_request_target(&arguments[1..]).and_then(|arguments| checkpoint(arguments, true))
        }
        Some("prepare") => parse_request_target(&arguments[1..]).and_then(prepare),
        Some("recover") => parse_run_target(&arguments[1..]).and_then(recover),
        _ => Err(AdapterError::Input(
            "loop requires initialize, validate, checkpoint, steer, prepare, or recover".to_owned(),
        )
        .into()),
    }
    .unwrap_or_else(|error| failure_result(action, &error));
    emit_action_result(&result)
}

fn parse_initialize(arguments: &[String]) -> Result<(PathBuf, PathBuf), LoopCliError> {
    let options = parse_options(arguments, &["--target", "--graph"])?;
    Ok((
        PathBuf::from(required(&options, "--target")?),
        PathBuf::from(required(&options, "--graph")?),
    ))
}

fn parse_run_target(arguments: &[String]) -> Result<(PathBuf, String), LoopCliError> {
    let options = parse_options(arguments, &["--target", "--run"])?;
    Ok((
        PathBuf::from(required(&options, "--target")?),
        required(&options, "--run")?.to_owned(),
    ))
}

fn parse_request_target(arguments: &[String]) -> Result<(PathBuf, PathBuf), LoopCliError> {
    let options = parse_options(arguments, &["--target", "--request"])?;
    Ok((
        PathBuf::from(required(&options, "--target")?),
        PathBuf::from(required(&options, "--request")?),
    ))
}

fn failure_result(action: &'static str, error: &LoopCliError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.message(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({"prepared_only": true, "spawned": false})),
    }
}

fn success_result(
    action: &'static str,
    code: &'static str,
    message: impl Into<String>,
    changed_paths: Vec<String>,
    evidence: Vec<Evidence>,
    next_action: Option<String>,
    data: Value,
) -> ActionResult {
    let mut data = data;
    if let Some(object) = data.as_object_mut() {
        object.insert("prepared_only".to_owned(), Value::Bool(true));
        object.insert("spawned".to_owned(), Value::Bool(false));
    }
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.into(),
        changed_paths,
        evidence,
        next_action,
        data: Some(data),
    }
}

fn validate_run_id(run_id: &str) -> Result<(), LoopCliError> {
    let bytes = run_id.as_bytes();
    if !(2..=127).contains(&bytes.len())
        || !bytes[0].is_ascii_lowercase()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
    {
        return Err(AdapterError::Input(format!("invalid run id: {run_id}")).into());
    }
    Ok(())
}

fn validate_safe_id(value: &str, label: &str) -> Result<(), LoopCliError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 64
        || !bytes[0].is_ascii_alphanumeric()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-'))
    {
        return Err(AdapterError::Input(format!("invalid {label}: {value}")).into());
    }
    Ok(())
}

fn require_digest(value: &str, label: &str) -> Result<(), LoopCliError> {
    let valid = value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    if !valid {
        return Err(
            AdapterError::Input(format!("{label} must be a lowercase sha256 digest")).into(),
        );
    }
    Ok(())
}

fn graph_root(run_id: &str) -> Result<PathBuf, LoopCliError> {
    validate_run_id(run_id)?;
    let path = run_path(run_id, "graph")?;
    Ok(path)
}

fn current_path(run_id: &str) -> Result<PathBuf, LoopCliError> {
    Ok(graph_root(run_id)?.join("CURRENT.md"))
}

fn revisions_path(run_id: &str) -> Result<PathBuf, LoopCliError> {
    Ok(graph_root(run_id)?.join("revisions"))
}

fn revision_path(run_id: &str, revision: u64) -> Result<PathBuf, LoopCliError> {
    if revision == 0 || revision > MAX_GRAPH_REVISIONS {
        return Err(AdapterError::Input(format!(
            "graph revision must be from 1 through {MAX_GRAPH_REVISIONS}"
        ))
        .into());
    }
    Ok(revisions_path(run_id)?.join(format!("{revision:016}.md")))
}

fn prepared_root(run_id: &str) -> Result<PathBuf, LoopCliError> {
    Ok(graph_root(run_id)?.join("prepared"))
}

fn prepared_path(run_id: &str, evidence_id: &str) -> Result<PathBuf, LoopCliError> {
    validate_safe_id(evidence_id, "usage evidence id")?;
    Ok(prepared_root(run_id)?.join(format!("{evidence_id}.json")))
}

fn parse_json<T>(bytes: &[u8], label: &str) -> Result<T, LoopCliError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(bytes)
        .map_err(|error| AdapterError::Input(format!("invalid {label}: {error}")).into())
}

fn canonical_json<T>(value: &T, label: &str) -> Result<Vec<u8>, LoopCliError>
where
    T: Serialize,
{
    serde_json_canonicalizer::to_vec(value)
        .map_err(|error| AdapterError::Internal(format!("cannot encode {label}: {error}")).into())
}

fn require_fresh_modified(path: &Path, max_age_seconds: u64) -> Result<(), LoopCliError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        AdapterError::Safety(format!(
            "cannot inspect fresh evidence {}: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AdapterError::Safety(format!(
            "fresh evidence is not a no-follow regular file: {}",
            path.display()
        ))
        .into());
    }
    let modified = metadata.modified().map_err(|error| {
        AdapterError::Safety(format!(
            "cannot inspect fresh evidence timestamp {}: {error}",
            path.display()
        ))
    })?;
    let age = SystemTime::now().duration_since(modified).map_err(|_| {
        AdapterError::Safety(format!(
            "fresh evidence timestamp is in the future: {}",
            path.display()
        ))
    })?;
    if age.as_secs() > max_age_seconds {
        return Err(AdapterError::Verification(format!(
            "evidence is older than {max_age_seconds} seconds: {}",
            path.display()
        ))
        .into());
    }
    Ok(())
}

fn read_fresh_explicit_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>, LoopCliError> {
    let before = read_explicit_file(path, max_bytes)?;
    require_fresh_modified(path, CAPABILITY_MAX_AGE_SECONDS)?;
    let after = read_explicit_file(path, max_bytes)?;
    if before != after {
        return Err(AdapterError::Conflict(format!(
            "fresh evidence changed during validation: {}",
            path.display()
        ))
        .into());
    }
    Ok(after)
}

fn stable_explicit_path(path: &Path) -> Result<PathBuf, LoopCliError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|current| current.join(path))
        .map_err(|error| {
            AdapterError::Input(format!("cannot resolve current directory: {error}")).into()
        })
}

fn read_fresh_target_file(
    target: &PinnedTarget,
    relative: &Path,
    max_bytes: usize,
) -> Result<Vec<u8>, LoopCliError> {
    let before = target.read_required(relative, max_bytes)?;
    let absolute = target.requested_path().join(relative);
    require_fresh_modified(&absolute, CAPABILITY_MAX_AGE_SECONDS)?;
    let after = target.read_required(relative, max_bytes)?;
    if before != after {
        return Err(AdapterError::Conflict(format!(
            "fresh target evidence changed during validation: {}",
            relative.display()
        ))
        .into());
    }
    Ok(after)
}

fn observed_support(
    resolution: &CapabilityResolution,
    capability: &str,
) -> Result<Option<CapabilitySupportLevel>, LoopCliError> {
    resolution
        .capabilities
        .get(capability)
        .map(|value| {
            serde_json::from_value(value.clone()).map_err(|_| {
                AdapterError::Verification(format!(
                    "capability resolution has a malformed support claim: {capability}"
                ))
                .into()
            })
        })
        .transpose()
}

fn verify_fresh_capability_resolution(
    current: &LoopGraphDocument,
    selected: &SelectedDispatch,
    path: &Path,
) -> Result<(CapabilityResolution, Vec<u8>), LoopCliError> {
    let bytes = read_fresh_explicit_file(path, MAX_GRAPH_BYTES)?;
    let resolution = validate_capability_resolution(current, selected, &bytes)?;
    Ok((resolution, bytes))
}

fn validate_capability_resolution(
    current: &LoopGraphDocument,
    selected: &SelectedDispatch,
    bytes: &[u8],
) -> Result<CapabilityResolution, LoopCliError> {
    let resolution = CapabilityResolution::parse_json(bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    for pinned in &current.graph().capability_support {
        let observed = observed_support(&resolution, &pinned.name)?;
        if pinned.evidence_digest != resolution.evidence_digest || observed != Some(pinned.support)
        {
            if matches!(
                observed,
                None | Some(
                    CapabilitySupportLevel::BestEffort
                        | CapabilitySupportLevel::Unsupported
                        | CapabilitySupportLevel::Unverified
                )
            ) {
                return Err(LoopContractError::HostCapabilityUnsupported {
                    node_id: selected
                        .node_id
                        .clone()
                        .unwrap_or_else(|| "graph".to_owned()),
                    capability: pinned.name.clone(),
                    support: observed,
                }
                .into());
            }
            return Err(AdapterError::Verification(format!(
                "graph capability snapshot differs from fresh resolution: {}",
                pinned.name
            ))
            .into());
        }
    }
    let usage_support = current
        .graph()
        .capability_support
        .iter()
        .find(|support| support.name == "usage-sensor")
        .ok_or_else(|| LoopContractError::HostCapabilityUnsupported {
            node_id: selected
                .node_id
                .clone()
                .unwrap_or_else(|| "graph".to_owned()),
            capability: "usage-sensor".to_owned(),
            support: None,
        })?;
    if !matches!(
        usage_support.support,
        CapabilitySupportLevel::Supported | CapabilitySupportLevel::BestEffort
    ) {
        return Err(LoopContractError::HostCapabilityUnsupported {
            node_id: selected
                .node_id
                .clone()
                .unwrap_or_else(|| "graph".to_owned()),
            capability: "usage-sensor".to_owned(),
            support: Some(usage_support.support),
        }
        .into());
    }
    Ok(resolution)
}

fn load_dispatch_status(
    target: &PinnedTarget,
    current: &LoopGraphDocument,
    selected: &SelectedDispatch,
    capability: &CapabilityResolution,
) -> Result<(RunStatusDocument, Vec<u8>), LoopCliError> {
    let status_path = run_path(&current.graph().run_id, "STATUS.md")?;
    let bytes = target.read_required(&status_path, MAX_GRAPH_BYTES)?;
    let status = RunStatusDocument::parse_markdown(&bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    status
        .validate_checkpoint()
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    if status.status().run_id != current.graph().run_id
        || !same_criteria(
            &status.status().required_criteria,
            &current.graph().required_criteria,
        )
    {
        return Err(AdapterError::Verification(
            "loop graph is not bound to the current run STATUS.md".to_owned(),
        )
        .into());
    }
    if !matches!(
        status.status().state,
        RunState::Executing | RunState::Verifying
    ) {
        return Err(
            AdapterError::OwnerBlocked("run STATUS.md is not dispatchable".to_owned()).into(),
        );
    }
    if let Some(role_id) = selected.role_id.as_deref() {
        if !status
            .status()
            .active_roles
            .iter()
            .any(|active| active == role_id)
        {
            return Err(AdapterError::OwnerBlocked(format!(
                "loop executor is not active in STATUS.md: {role_id}"
            ))
            .into());
        }
    }
    let owner = status
        .owner_binding()
        .map_err(|error| AdapterError::OwnerBlocked(error.to_string()))?;
    match verify_owner_continuity(&owner, Some(capability)) {
        OwnerContinuity::Matched => {}
        OwnerContinuity::Blocked(reason) => {
            return Err(AdapterError::OwnerBlocked(format!(
                "fresh capability resolution does not match run owner: {reason:?}"
            ))
            .into());
        }
        OwnerContinuity::Unsupported(reason) => {
            return Err(AdapterError::OwnerUnsupported(format!(
                "fresh capability resolution is unsupported by run owner: {reason:?}"
            ))
            .into());
        }
    }
    Ok((status, bytes))
}

fn validate_run_contract(
    target: &PinnedTarget,
    run_id: &str,
    criteria: &[String],
) -> Result<(), LoopCliError> {
    validate_run_id(run_id)?;
    let plan_path = run_path(run_id, "PLAN.md")?;
    let plan_bytes = target.read_required(&plan_path, MAX_GRAPH_BYTES)?;
    let plan = RunPlan::parse_markdown(&plan_bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    if !same_criteria(plan.criteria(), criteria) {
        return Err(AdapterError::Verification(
            "loop required criteria do not match the immutable run PLAN.md".to_owned(),
        )
        .into());
    }

    let status_path = run_path(run_id, "STATUS.md")?;
    if let Some(status_bytes) = target.read_optional(&status_path, MAX_GRAPH_BYTES)? {
        let status = RunStatusDocument::parse_markdown(&status_bytes)
            .map_err(|error| AdapterError::Verification(error.to_string()))?;
        if status.status().run_id != run_id {
            return Err(AdapterError::Verification(
                "loop run id does not match STATUS.md".to_owned(),
            )
            .into());
        }
        if !same_criteria(&status.status().required_criteria, criteria) {
            return Err(AdapterError::Verification(
                "loop required criteria do not match STATUS.md".to_owned(),
            )
            .into());
        }
    }
    Ok(())
}

fn same_criteria(left: &[String], right: &[String]) -> bool {
    left.len() == right.len()
        && left.iter().collect::<BTreeSet<_>>() == right.iter().collect::<BTreeSet<_>>()
}

fn validate_initial_graph(document: &LoopGraphDocument) -> Result<(), LoopCliError> {
    let graph = document.graph();
    if graph.revision != 1 || graph.previous_revision_digest.is_some() {
        return Err(AdapterError::Verification(
            "initial loop graph must be revision 1 without a previous digest".to_owned(),
        )
        .into());
    }
    if graph.state != LoopState::Active || graph.terminal_reason.is_some() {
        return Err(AdapterError::Verification(
            "initial loop graph must be active without a terminal reason".to_owned(),
        )
        .into());
    }
    if !graph.passed_criteria.is_empty()
        || !graph.evidence.is_empty()
        || !graph.attempts.is_empty()
        || !graph.steering.is_empty()
    {
        return Err(AdapterError::Verification(
            "initial loop graph cannot contain progress, evidence, attempts, or steering"
                .to_owned(),
        )
        .into());
    }
    Ok(())
}

fn ensure_graph_directory(target: &PinnedTarget, relative: &Path) -> Result<(), LoopCliError> {
    let root = relative
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "graph"))
        .ok_or_else(|| AdapterError::Safety("loop directory escaped graph root".to_owned()))?;
    validate_project_relative(relative).map_err(|error| AdapterError::Safety(error.to_string()))?;
    let mut current = target
        .target_dir()
        .try_clone()
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    let mut walked = PathBuf::new();
    for component in relative.components() {
        let name = component.as_os_str();
        walked.push(name);
        match current.symlink_metadata(name) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(AdapterError::Safety(format!(
                    "loop ancestor is not a no-follow directory: {}",
                    walked.display()
                ))
                .into());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && walked.starts_with(root) => {
                match current.create_dir(name) {
                    Ok(()) => {}
                    Err(create_error) if create_error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(create_error) => {
                        return Err(AdapterError::Safety(format!(
                            "cannot create loop directory {}: {create_error}",
                            walked.display()
                        ))
                        .into());
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(AdapterError::Safety(format!(
                    "run ancestor is missing before loop initialization: {}",
                    walked.display()
                ))
                .into());
            }
            Err(error) => {
                return Err(AdapterError::Safety(format!(
                    "cannot inspect loop ancestor {}: {error}",
                    walked.display()
                ))
                .into());
            }
        }
        current = current.open_dir_nofollow(name).map_err(|error| {
            AdapterError::Safety(format!(
                "cannot open loop ancestor no-follow {}: {error}",
                walked.display()
            ))
        })?;
    }
    Ok(())
}

fn initialize((target_path, graph_path): (PathBuf, PathBuf)) -> Result<ActionResult, LoopCliError> {
    let source_bytes = read_explicit_file(&graph_path, MAX_GRAPH_BYTES)?;
    let source = LoopGraphDocument::parse_markdown(&source_bytes)?;
    validate_initial_graph(&source)?;
    validate_run_id(&source.graph().run_id)?;
    let target = PinnedTarget::open(&target_path)?;
    validate_run_contract(
        &target,
        &source.graph().run_id,
        &source.graph().required_criteria,
    )?;

    let run_id = &source.graph().run_id;
    let current = current_path(run_id)?;
    let revision = revision_path(run_id, 1)?;
    let current_before = target.snapshot_bounded(&current, MAX_GRAPH_BYTES)?;
    let revision_before = target.snapshot_bounded(&revision, MAX_GRAPH_BYTES)?;
    if !matches!(current_before, FileSnapshot::Missing)
        || !matches!(revision_before, FileSnapshot::Missing)
    {
        return Err(AdapterError::Conflict(
            "loop graph is already initialized for this run".to_owned(),
        )
        .into());
    }
    let canonical = source.encode_canonical()?;
    if canonical.len() > MAX_GRAPH_BYTES {
        return Err(AdapterError::Input("canonical loop graph exceeds 1 MiB".to_owned()).into());
    }

    ensure_graph_directory(&target, &revisions_path(run_id)?)?;
    if !target.publish(&revision, &revision_before, &canonical)? {
        return Err(
            AdapterError::Conflict("initial graph revision already exists".to_owned()).into(),
        );
    }
    if let Err(error) = target.publish(&current, &current_before, &canonical) {
        target.restore(&revision, &revision_before, &canonical)?;
        return Err(error.into());
    }

    let digest = sha256_digest(&canonical);
    Ok(success_result(
        "InitializeLoop",
        "hive.loop-initialized",
        "durable loop graph initialized without dispatch",
        vec![
            portable_relative_path(&revision),
            portable_relative_path(&current),
        ],
        vec![Evidence {
            kind: "loop-graph",
            locator: portable_relative_path(&revision),
            digest: digest.clone(),
        }],
        Some("checkpoint evidence or prepare a host-owned node".to_owned()),
        json!({
            "run_id": run_id,
            "graph_revision": 1,
            "graph_digest": digest,
            "state": source.graph().state,
        }),
    ))
}

fn load_graph_chain(target: &PinnedTarget, run_id: &str) -> Result<LoadedGraphChain, LoopCliError> {
    validate_run_id(run_id)?;
    let current_relative = current_path(run_id)?;
    let current_bytes = target.read_required(&current_relative, MAX_GRAPH_BYTES)?;
    let current = LoopGraphDocument::parse_markdown(&current_bytes)?;
    let canonical_current = current.encode_canonical()?;
    if canonical_current != current_bytes {
        return Err(AdapterError::Verification(
            "CURRENT.md is not canonical loop Markdown".to_owned(),
        )
        .into());
    }
    if current.graph().run_id != run_id {
        return Err(
            AdapterError::Verification("CURRENT.md belongs to another run".to_owned()).into(),
        );
    }
    if current.graph().revision == 0 || current.graph().revision > MAX_GRAPH_REVISIONS {
        return Err(AdapterError::Verification(format!(
            "graph revision exceeds recovery bound {MAX_GRAPH_REVISIONS}"
        ))
        .into());
    }

    let mut documents = Vec::new();
    let mut total_chain_bytes = 0_usize;
    let mut last_revision_bytes = Vec::new();
    for revision in 1..=current.graph().revision {
        let relative = revision_path(run_id, revision)?;
        let bytes = target.read_required(&relative, MAX_GRAPH_BYTES)?;
        total_chain_bytes = total_chain_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| AdapterError::Verification("graph chain size overflowed".to_owned()))?;
        if total_chain_bytes > MAX_TOTAL_GRAPH_CHAIN_BYTES {
            return Err(AdapterError::Verification(format!(
                "graph chain exceeds aggregate recovery bound {MAX_TOTAL_GRAPH_CHAIN_BYTES}"
            ))
            .into());
        }
        let document = LoopGraphDocument::parse_markdown(&bytes)?;
        if document.graph().run_id != run_id || document.graph().revision != revision {
            return Err(AdapterError::Verification(format!(
                "immutable graph revision identity mismatch: {}",
                relative.display()
            ))
            .into());
        }
        if document.encode_canonical()? != bytes {
            return Err(AdapterError::Verification(format!(
                "immutable graph revision is not canonical: {}",
                relative.display()
            ))
            .into());
        }
        if let Some(previous) = documents.last() {
            if validate_loop_transition(previous, &document)? != LoopTransitionOutcome::Advance {
                return Err(AdapterError::Verification(
                    "immutable graph chain contains a repeated revision".to_owned(),
                )
                .into());
            }
        } else {
            validate_initial_graph(&document)?;
        }
        last_revision_bytes = bytes;
        documents.push(document);
    }
    if last_revision_bytes != current_bytes {
        return Err(AdapterError::Verification(
            "CURRENT.md does not exactly match its immutable graph revision".to_owned(),
        )
        .into());
    }
    validate_run_contract(target, run_id, &current.graph().required_criteria)?;
    Ok(LoadedGraphChain {
        documents,
        current_bytes,
    })
}

fn validate((target_path, run_id): (PathBuf, String)) -> Result<ActionResult, LoopCliError> {
    let target = PinnedTarget::open(&target_path)?;
    let chain = load_graph_chain(&target, &run_id)?;
    let current = chain.current();
    let digest = current.canonical_digest()?;
    Ok(success_result(
        "ValidateLoop",
        "hive.loop-valid",
        "loop graph and immutable revision chain are valid",
        Vec::new(),
        vec![Evidence {
            kind: "loop-graph",
            locator: portable_relative_path(&current_path(&run_id)?),
            digest: digest.clone(),
        }],
        None,
        json!({
            "run_id": run_id,
            "graph_revision": current.graph().revision,
            "graph_digest": digest,
            "revision_count": chain.documents.len(),
            "state": current.graph().state,
        }),
    ))
}

fn recover((target_path, run_id): (PathBuf, String)) -> Result<ActionResult, LoopCliError> {
    let target = PinnedTarget::open(&target_path)?;
    let chain = load_graph_chain(&target, &run_id)?;
    let current = chain.current();
    let ready_nodes = current.graph().ready_nodes()?;
    let next = ready_nodes.first().cloned();
    if let Some(node_id) = next.as_deref() {
        current.graph().validate_node_capabilities(node_id)?;
    }
    let retry = next
        .as_deref()
        .map(|node_id| current.graph().retry_decision(node_id))
        .transpose()?;
    let next_action = next
        .as_ref()
        .map(|node_id| format!("prepare host-owned node {node_id}"));
    let digest = current.canonical_digest()?;
    Ok(success_result(
        "RecoverLoop",
        "hive.loop-recovered",
        "loop graph recovered deterministically from canonical Markdown",
        Vec::new(),
        vec![Evidence {
            kind: "loop-graph",
            locator: portable_relative_path(&current_path(&run_id)?),
            digest: digest.clone(),
        }],
        next_action,
        json!({
            "run_id": run_id,
            "graph_revision": current.graph().revision,
            "graph_digest": digest,
            "state": current.graph().state,
            "ready_nodes": ready_nodes,
            "next_node": next,
            "retry_decision": retry_decision_value(retry.as_ref()),
            "recovered_from": "canonical-markdown",
        }),
    ))
}

fn retry_decision_value(decision: Option<&RetryDecision>) -> Value {
    match decision {
        Some(RetryDecision::FirstAttempt { attempt }) => {
            json!({"decision": "first-attempt", "attempt": attempt})
        }
        Some(RetryDecision::Retry {
            attempt,
            backoff_seconds,
        }) => json!({
            "decision": "retry",
            "attempt": attempt,
            "backoff_seconds": backoff_seconds,
        }),
        Some(RetryDecision::StopSucceeded) => json!({"decision": "stop-succeeded"}),
        Some(RetryDecision::StopBudgetExhausted) => {
            json!({"decision": "stop-budget-exhausted"})
        }
        Some(RetryDecision::StopRepeatedFailure {
            fingerprint,
            occurrences,
        }) => json!({
            "decision": "stop-repeated-failure",
            "fingerprint": fingerprint,
            "occurrences": occurrences,
        }),
        None => Value::Null,
    }
}

fn evidence_locator(run_id: &str, locator: &str) -> Result<PathBuf, LoopCliError> {
    if locator.is_empty() || locator.contains(['\0', '\r', '\n', '\\']) {
        return Err(
            AdapterError::Safety(format!("unsafe loop evidence locator: {locator}")).into(),
        );
    }
    let path = PathBuf::from(locator);
    validate_project_relative(&path).map_err(|error| AdapterError::Safety(error.to_string()))?;
    if portable_relative_path(&path) != locator {
        return Err(AdapterError::Safety(format!(
            "loop evidence locator is not canonical: {locator}"
        ))
        .into());
    }
    let root = run_path(run_id, "evidence")?;
    let remainder = path.strip_prefix(&root).map_err(|_| {
        AdapterError::Safety(format!(
            "loop evidence must remain below {}",
            root.display()
        ))
    })?;
    if remainder.as_os_str().is_empty() {
        return Err(
            AdapterError::Safety("loop evidence locator must identify a file".to_owned()).into(),
        );
    }
    Ok(path)
}

fn verify_new_evidence(
    target: &PinnedTarget,
    previous: &LoopGraphDocument,
    candidate: &LoopGraphDocument,
    references: &[EvidenceFileRef],
) -> Result<Vec<Evidence>, LoopCliError> {
    let old_count = previous.graph().evidence.len();
    let new_evidence =
        candidate.graph().evidence.get(old_count..).ok_or_else(|| {
            AdapterError::Verification("loop evidence history regressed".to_owned())
        })?;
    if new_evidence.len() != references.len() {
        return Err(AdapterError::Verification(
            "request must name every and only newly appended evidence file".to_owned(),
        )
        .into());
    }
    let mut by_id = BTreeMap::new();
    for reference in references {
        validate_safe_id(&reference.evidence_id, "evidence id")?;
        require_digest(&reference.digest, "evidence digest")?;
        if by_id
            .insert(reference.evidence_id.as_str(), reference)
            .is_some()
        {
            return Err(AdapterError::Input(format!(
                "duplicate evidence reference: {}",
                reference.evidence_id
            ))
            .into());
        }
    }

    let previous_digest = previous.canonical_digest()?;
    let mut total_bytes = 0_usize;
    let mut result = Vec::new();
    for item in new_evidence {
        let reference = by_id.get(item.id.as_str()).ok_or_else(|| {
            AdapterError::Verification(format!("new evidence is absent from request: {}", item.id))
        })?;
        if reference.locator != item.locator || reference.digest != item.digest {
            return Err(AdapterError::Verification(format!(
                "evidence reference does not exactly match graph metadata: {}",
                item.id
            ))
            .into());
        }
        let relative = evidence_locator(&candidate.graph().run_id, &item.locator)?;
        let bytes = target.read_required(&relative, MAX_EVIDENCE_FILE_BYTES)?;
        total_bytes = total_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| AdapterError::Input("loop evidence byte count overflowed".to_owned()))?;
        if total_bytes > MAX_TOTAL_EVIDENCE_BYTES {
            return Err(AdapterError::Input(
                "new loop evidence exceeds the 4 MiB aggregate bound".to_owned(),
            )
            .into());
        }
        if sha256_digest(&bytes) != item.digest {
            return Err(AdapterError::Verification(format!(
                "evidence digest mismatch: {}",
                item.id
            ))
            .into());
        }
        match item.kind {
            EvidenceKind::Artifact => {}
            EvidenceKind::UsageAuthorization => {
                verify_usage_envelope(&bytes, item, previous, candidate, &previous_digest)?;
            }
            EvidenceKind::IndependentVerification => {
                verify_verifier_envelope(target, &bytes, item, candidate)?;
            }
            EvidenceKind::SteeringAuthorization => {
                verify_steering_envelope(&bytes, item, previous, candidate, &previous_digest)?;
            }
        }
        result.push(Evidence {
            kind: "loop-evidence",
            locator: item.locator.clone(),
            digest: item.digest.clone(),
        });
    }
    Ok(result)
}

fn verify_current_evidence_files(
    target: &PinnedTarget,
    chain: &LoadedGraphChain,
) -> Result<(), LoopCliError> {
    let current = chain.current();
    let mut total_bytes = 0_usize;
    for evidence in &current.graph().evidence {
        let relative = evidence_locator(&current.graph().run_id, &evidence.locator)?;
        let bytes = target.read_required(&relative, MAX_EVIDENCE_FILE_BYTES)?;
        total_bytes = total_bytes.checked_add(bytes.len()).ok_or_else(|| {
            AdapterError::Verification("loop evidence size overflowed".to_owned())
        })?;
        if total_bytes > MAX_TOTAL_EVIDENCE_BYTES {
            return Err(AdapterError::Verification(
                "current loop evidence exceeds the aggregate verification bound".to_owned(),
            )
            .into());
        }
        if sha256_digest(&bytes) != evidence.digest {
            return Err(AdapterError::Verification(format!(
                "current loop evidence changed or disappeared: {}",
                evidence.id
            ))
            .into());
        }
        match evidence.kind {
            EvidenceKind::Artifact => {}
            EvidenceKind::UsageAuthorization => {
                let revision = chain.document_at(evidence.graph_revision).ok_or_else(|| {
                    AdapterError::Verification(
                        "usage evidence graph revision is missing".to_owned(),
                    )
                })?;
                let previous = chain
                    .document_at(evidence.graph_revision.checked_sub(1).ok_or_else(|| {
                        AdapterError::Verification(
                            "initial graph cannot contain usage evidence".to_owned(),
                        )
                    })?)
                    .ok_or_else(|| {
                        AdapterError::Verification(
                            "usage evidence base graph revision is missing".to_owned(),
                        )
                    })?;
                let previous_digest = previous.canonical_digest()?;
                verify_usage_envelope(&bytes, evidence, previous, revision, &previous_digest)?;
            }
            EvidenceKind::IndependentVerification => {
                verify_verifier_envelope(target, &bytes, evidence, current)?;
            }
            EvidenceKind::SteeringAuthorization => {
                let revision = chain.document_at(evidence.graph_revision).ok_or_else(|| {
                    AdapterError::Verification(
                        "steering evidence graph revision is missing".to_owned(),
                    )
                })?;
                let previous = chain
                    .document_at(evidence.graph_revision.checked_sub(1).ok_or_else(|| {
                        AdapterError::Verification(
                            "initial graph cannot contain steering evidence".to_owned(),
                        )
                    })?)
                    .ok_or_else(|| {
                        AdapterError::Verification(
                            "steering evidence base graph revision is missing".to_owned(),
                        )
                    })?;
                let previous_digest = previous.canonical_digest()?;
                verify_steering_envelope(&bytes, evidence, previous, revision, &previous_digest)?;
            }
        }
    }
    Ok(())
}

fn verify_usage_envelope(
    bytes: &[u8],
    evidence: &hive_core::loop_graph::LoopEvidence,
    previous: &LoopGraphDocument,
    candidate: &LoopGraphDocument,
    previous_digest: &str,
) -> Result<UsageAuthorizationEnvelope, LoopCliError> {
    let envelope: UsageAuthorizationEnvelope = parse_json(bytes, "usage authorization envelope")?;
    require_digest(&envelope.base_graph_digest, "usage base graph digest")?;
    require_digest(&envelope.brief_digest, "usage brief digest")?;
    if envelope.schema_version != 1
        || envelope.evidence_id != evidence.id
        || envelope.run_id != candidate.graph().run_id
        || envelope.base_graph_revision != previous.graph().revision
        || envelope.base_graph_digest != previous_digest
        || envelope.authorized_graph_revision != candidate.graph().revision
        || envelope.subject_node_id != evidence.subject_node_id
        || envelope.attempt != evidence.attempt
        || envelope.decision != EvidenceResult::Allowed
        || envelope.authenticated != evidence.authenticated
        || !envelope.authenticated
        || envelope.process_id == 0
    {
        return Err(AdapterError::Verification(format!(
            "usage authorization envelope is not bound to evidence {}",
            evidence.id
        ))
        .into());
    }
    require_digest(
        &envelope.dispatch_authorization_digest,
        "dispatch authorization digest",
    )?;
    require_digest(
        &envelope.usage_evidence_digest,
        "usage enforcement evidence digest",
    )?;
    require_digest(&envelope.session_id_digest, "usage session digest")?;
    if envelope.session_id.is_empty()
        || envelope.session_id.len() > 256
        || envelope.session_id.chars().any(char::is_control)
    {
        return Err(AdapterError::Verification(
            "usage authorization session id is invalid".to_owned(),
        )
        .into());
    }
    match envelope.dispatch_kind {
        LoopDispatchKind::Steering if envelope.subject_node_id.is_none() => {}
        LoopDispatchKind::Node | LoopDispatchKind::Retry
            if envelope.subject_node_id.is_some() && envelope.attempt.is_some() => {}
        _ => {
            return Err(AdapterError::Verification(format!(
                "usage authorization envelope has contradictory dispatch scope: {}",
                evidence.id
            ))
            .into());
        }
    }
    Ok(envelope)
}

fn verify_verifier_envelope(
    target: &PinnedTarget,
    bytes: &[u8],
    evidence: &hive_core::loop_graph::LoopEvidence,
    candidate: &LoopGraphDocument,
) -> Result<(), LoopCliError> {
    let envelope: VerifierEnvelope = parse_json(bytes, "independent verifier envelope")?;
    if envelope.evidence_id != evidence.id
        || envelope.run_id != candidate.graph().run_id
        || envelope.graph_revision != evidence.graph_revision
        || Some(envelope.node_id.as_str()) != evidence.subject_node_id.as_deref()
        || Some(envelope.attempt) != evidence.attempt
        || Some(envelope.producer_role_id.as_str()) != evidence.producer_role_id.as_deref()
        || Some(envelope.verifier_role_id.as_str()) != evidence.verifier_role_id.as_deref()
        || Some(envelope.authority) != evidence.verification_authority
        || envelope.result != evidence.result
        || envelope.authenticated != evidence.authenticated
        || envelope.producer_role_id == envelope.verifier_role_id
    {
        return Err(AdapterError::Verification(format!(
            "independent verifier envelope is not bound to evidence {}",
            evidence.id
        ))
        .into());
    }
    match envelope.authority {
        VerificationAuthority::Deterministic => {
            if envelope.schema_version != 1
                || envelope.authenticated
                || envelope.judge_quorum_request.is_some()
                || envelope.judge_quorum_request_digest.is_some()
                || envelope.judge_trust_root.is_some()
            {
                return Err(AdapterError::Verification(format!(
                    "deterministic verifier envelope has Judge-only fields: {}",
                    evidence.id
                ))
                .into());
            }
        }
        VerificationAuthority::Judge => {
            let Some(request_locator) = envelope.judge_quorum_request.as_deref() else {
                return Err(AdapterError::Verification(
                    "judge verifier envelope is missing its quorum request".to_owned(),
                )
                .into());
            };
            let Some(request_digest) = envelope.judge_quorum_request_digest.as_deref() else {
                return Err(AdapterError::Verification(
                    "judge verifier envelope is missing its quorum request digest".to_owned(),
                )
                .into());
            };
            let Some(trust_root) = envelope.judge_trust_root.as_deref() else {
                return Err(AdapterError::Verification(
                    "judge verifier envelope is missing its external trust root".to_owned(),
                )
                .into());
            };
            if envelope.schema_version != 2 || !envelope.authenticated {
                return Err(AdapterError::Verification(
                    "judge verifier envelope must be schema version 2 and authenticated".to_owned(),
                )
                .into());
            }
            require_digest(request_digest, "judge quorum request digest")?;
            let request_path = evidence_locator(&candidate.graph().run_id, request_locator)?;
            let request_bytes = target.read_required(&request_path, MAX_EVIDENCE_FILE_BYTES)?;
            if sha256_digest(&request_bytes) != request_digest {
                return Err(AdapterError::Verification(
                    "judge quorum request changed after verifier evidence was created".to_owned(),
                )
                .into());
            }
            let subject_id = format!(
                "hive-loop:{}:{}:{}:{}:{}",
                candidate.graph().run_id,
                evidence.graph_revision,
                envelope.node_id,
                envelope.attempt,
                evidence.id,
            );
            verify_authenticated_loop_quorum(target, &request_path, trust_root, &subject_id)?;
        }
    }
    Ok(())
}

fn verify_steering_envelope(
    bytes: &[u8],
    evidence: &hive_core::loop_graph::LoopEvidence,
    previous: &LoopGraphDocument,
    candidate: &LoopGraphDocument,
    previous_digest: &str,
) -> Result<(), LoopCliError> {
    let envelope: SteeringAuthorizationEnvelope =
        parse_json(bytes, "steering authorization envelope")?;
    require_digest(&envelope.base_graph_digest, "steering base graph digest")?;
    require_digest(&envelope.proposal_digest, "steering proposal digest")?;
    let proposal_digest = steering_proposal_digest(candidate)?;
    if envelope.schema_version != 1
        || envelope.evidence_id != evidence.id
        || envelope.run_id != candidate.graph().run_id
        || envelope.base_graph_revision != previous.graph().revision
        || envelope.base_graph_digest != previous_digest
        || envelope.authorized_graph_revision != candidate.graph().revision
        || envelope.user_boundary != UserBoundary::ExplicitUserApproval
        || envelope.result != EvidenceResult::Allowed
        || !envelope.authenticated
        || !evidence.authenticated
        || envelope.proposal_digest != proposal_digest
    {
        return Err(AdapterError::Verification(format!(
            "steering authorization envelope is not bound to evidence {}",
            evidence.id
        ))
        .into());
    }
    Ok(())
}

fn steering_proposal_digest(candidate: &LoopGraphDocument) -> Result<String, LoopCliError> {
    let base_revision = candidate.graph().revision.checked_sub(1).ok_or_else(|| {
        AdapterError::Verification("steering candidate has no base revision".to_owned())
    })?;
    let steering = candidate
        .graph()
        .steering
        .iter()
        .find(|record| record.base_revision == base_revision)
        .ok_or_else(|| {
            AdapterError::Verification(
                "steering approval has no exact candidate steering record".to_owned(),
            )
        })?;
    let payload = json!({
        "schema_version": 1,
        "run_id": candidate.graph().run_id,
        "base_revision": steering.base_revision,
        "base_revision_digest": steering.base_revision_digest,
        "candidate_revision": candidate.graph().revision,
        "reason": steering.reason,
        "affected_edges": steering.affected_edges,
        "user_boundary": steering.user_boundary,
        "authorization_evidence_id": steering.authorization_evidence_id,
        "edges": candidate.graph().edges,
    });
    let bytes = serde_json_canonicalizer::to_vec(&payload)
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    Ok(sha256_digest(&bytes))
}

fn load_prepared_record(
    target: &PinnedTarget,
    run_id: &str,
    usage_evidence_id: &str,
) -> Result<Option<(PreparedDispatchRecord, Vec<u8>)>, LoopCliError> {
    let relative = prepared_path(run_id, usage_evidence_id)?;
    let Some(bytes) = target.read_optional(&relative, MAX_PREPARED_BYTES)? else {
        return Ok(None);
    };
    let record: PreparedDispatchRecord = parse_json(&bytes, "prepared dispatch record")?;
    if record.schema_version != 1 || !record.prepared_only || record.spawned {
        return Err(AdapterError::Verification(format!(
            "prepared dispatch record has unsafe execution flags: {}",
            relative.display()
        ))
        .into());
    }
    require_digest(&record.dispatch_id, "prepared dispatch id")?;
    require_digest(
        &record.dispatch_authorization_digest,
        "prepared dispatch authorization digest",
    )?;
    if record.capability_resolution.as_os_str().is_empty() {
        return Err(AdapterError::Verification(format!(
            "prepared dispatch capability locator is empty: {}",
            relative.display()
        ))
        .into());
    }
    require_digest(
        &record.capability_resolution_digest,
        "prepared capability resolution digest",
    )?;
    require_digest(
        &record.capability_resolution_file_digest,
        "prepared capability resolution file digest",
    )?;
    require_digest(&record.run_status_digest, "prepared run status digest")?;
    if record.binding.canonical_digest()? != record.dispatch_id {
        return Err(AdapterError::Verification(format!(
            "prepared dispatch id does not bind its exact action: {}",
            relative.display()
        ))
        .into());
    }
    if canonical_json(&record, "prepared dispatch record")? != bytes {
        return Err(AdapterError::Verification(format!(
            "prepared dispatch record is not canonical: {}",
            relative.display()
        ))
        .into());
    }
    Ok(Some((record, bytes)))
}

#[allow(clippy::too_many_lines)]
fn authenticate_prepared_attempt(
    target: &PinnedTarget,
    chain: &LoadedGraphChain,
    usage: &hive_core::loop_graph::LoopEvidence,
    attempt: &hive_core::loop_graph::NodeAttempt,
    record: &PreparedDispatchRecord,
) -> Result<(), LoopCliError> {
    let binding = &record.binding;
    if !matches!(
        binding.kind,
        LoopDispatchKind::Node | LoopDispatchKind::Retry
    ) || binding.usage_evidence_id != usage.id
        || binding.node_id.as_deref() != Some(attempt.node_id.as_str())
        || binding.attempt != Some(attempt.attempt)
        || record.dispatch_id != attempt.dispatch_digest
    {
        return Err(AdapterError::Verification(
            "prepared dispatch is not bound to the checkpointed run action".to_owned(),
        )
        .into());
    }

    let historical = chain.document_at(binding.graph_revision).ok_or_else(|| {
        AdapterError::Verification(
            "prepared dispatch graph revision is absent from the immutable chain".to_owned(),
        )
    })?;
    let historical_usage = historical
        .graph()
        .evidence
        .iter()
        .find(|evidence| evidence.id == binding.usage_evidence_id)
        .ok_or_else(|| {
            AdapterError::Verification(
                "prepared dispatch usage evidence is absent from its graph revision".to_owned(),
            )
        })?;
    if historical_usage != usage {
        return Err(AdapterError::Verification(
            "prepared dispatch usage evidence differs from immutable graph history".to_owned(),
        )
        .into());
    }

    let previous_revision = binding.graph_revision.checked_sub(1).ok_or_else(|| {
        AdapterError::Verification(
            "initial graph cannot authenticate a prepared dispatch".to_owned(),
        )
    })?;
    let previous = chain.document_at(previous_revision).ok_or_else(|| {
        AdapterError::Verification(
            "prepared dispatch usage base revision is absent from the immutable chain".to_owned(),
        )
    })?;
    let usage_path = evidence_locator(&binding.run_id, &historical_usage.locator)?;
    let usage_bytes = target.read_required(&usage_path, MAX_EVIDENCE_FILE_BYTES)?;
    if sha256_digest(&usage_bytes) != historical_usage.digest {
        return Err(AdapterError::Verification(
            "prepared dispatch usage envelope digest mismatch".to_owned(),
        )
        .into());
    }
    let previous_digest = previous.canonical_digest()?;
    let envelope = verify_usage_envelope(
        &usage_bytes,
        historical_usage,
        previous,
        historical,
        &previous_digest,
    )?;
    if envelope.dispatch_kind != binding.kind
        || envelope.subject_node_id != binding.node_id
        || envelope.attempt != binding.attempt
        || envelope.brief_digest != binding.brief_digest
    {
        return Err(AdapterError::Verification(
            "prepared dispatch usage envelope is bound to another action".to_owned(),
        )
        .into());
    }

    let selected = SelectedDispatch {
        node_id: binding.node_id.clone(),
        attempt: binding.attempt,
        role_id: binding.role_id.clone(),
        backoff_seconds: None,
    };
    let capability_bytes = read_explicit_file(&record.capability_resolution, MAX_GRAPH_BYTES)?;
    if sha256_digest(&capability_bytes) != record.capability_resolution_file_digest {
        return Err(AdapterError::Verification(
            "prepared dispatch capability file digest mismatch".to_owned(),
        )
        .into());
    }
    let capability = validate_capability_resolution(historical, &selected, &capability_bytes)?;
    if capability.evidence_digest != record.capability_resolution_digest {
        return Err(AdapterError::Verification(
            "prepared dispatch capability evidence digest mismatch".to_owned(),
        )
        .into());
    }
    verify_usage_session_state(target, &envelope, &capability)?;

    let (status, status_bytes) = load_dispatch_status(target, historical, &selected, &capability)?;
    if sha256_digest(&status_bytes) != record.run_status_digest {
        return Err(AdapterError::Verification(
            "prepared dispatch run status digest mismatch".to_owned(),
        )
        .into());
    }

    let expected_authorization_id = format!("sha256:{}", envelope.evidence_id);
    let authorization_path = dispatch_authorization_path(
        &envelope.dispatch_authorization_locator,
        &expected_authorization_id,
    )?;
    let authorization_bytes = target.read_required(&authorization_path, MAX_PREPARED_BYTES)?;
    let authorization_digest = authenticate_dispatch_authorization_record(
        &authorization_bytes,
        &envelope.dispatch_authorization_digest,
        historical,
        &status,
        &selected,
        &envelope,
    )?;
    if authorization_digest != record.dispatch_authorization_digest {
        return Err(AdapterError::Verification(
            "prepared dispatch authorization digest does not match the authenticated record"
                .to_owned(),
        )
        .into());
    }

    historical.validate_dispatch(binding)?;
    Ok(())
}

fn validate_new_attempts(
    target: &PinnedTarget,
    chain: &LoadedGraphChain,
    candidate: &LoopGraphDocument,
) -> Result<(), LoopCliError> {
    let old_count = chain.current().graph().attempts.len();
    let new_attempts =
        candidate.graph().attempts.get(old_count..).ok_or_else(|| {
            AdapterError::Verification("loop attempt history regressed".to_owned())
        })?;
    for attempt in new_attempts {
        let mut matched = false;
        for usage in candidate.graph().evidence.iter().filter(|evidence| {
            evidence.kind == EvidenceKind::UsageAuthorization
                && evidence.result == EvidenceResult::Allowed
                && evidence.subject_node_id.as_deref() == Some(attempt.node_id.as_str())
                && evidence.attempt == Some(attempt.attempt)
                && evidence.graph_revision <= attempt.graph_revision
        }) {
            let Some((record, _)) =
                load_prepared_record(target, &candidate.graph().run_id, &usage.id)?
            else {
                continue;
            };
            if record.binding.usage_evidence_id != usage.id
                || record.binding.node_id.as_deref() != Some(attempt.node_id.as_str())
                || record.binding.attempt != Some(attempt.attempt)
                || record.binding.graph_revision > chain.current().graph().revision
                || record.dispatch_id != attempt.dispatch_digest
            {
                continue;
            }
            authenticate_prepared_attempt(target, chain, usage, attempt, &record)?;
            matched = true;
            break;
        }
        if !matched {
            return Err(AdapterError::Verification(format!(
                "attempt {} for node {} lacks its exact prepare-only dispatch record",
                attempt.attempt, attempt.node_id
            ))
            .into());
        }
    }
    Ok(())
}

fn checkpoint(
    (target_path, request_path): (PathBuf, PathBuf),
    steering: bool,
) -> Result<ActionResult, LoopCliError> {
    let request_bytes = read_explicit_file(&request_path, MAX_REQUEST_BYTES)?;
    let request: RevisionRequest = parse_json(&request_bytes, "loop revision request")?;
    if request.schema_version != 1 {
        return Err(AdapterError::Input(
            "loop revision request schema_version must be 1".to_owned(),
        )
        .into());
    }
    validate_run_id(&request.run_id)?;
    require_digest(&request.expected_graph_digest, "expected graph digest")?;
    let target = PinnedTarget::open(&target_path)?;
    let chain = load_graph_chain(&target, &request.run_id)?;
    let previous = chain.current();
    let previous_digest = previous.canonical_digest()?;
    if request.expected_revision != previous.graph().revision
        || request.expected_graph_digest != previous_digest
    {
        return Err(AdapterError::Conflict("loop revision request is stale".to_owned()).into());
    }

    let candidate_bytes = read_explicit_file(&request.candidate_graph, MAX_GRAPH_BYTES)?;
    let candidate = LoopGraphDocument::parse_markdown(&candidate_bytes)?;
    let canonical = candidate.encode_canonical()?;
    if canonical != candidate_bytes {
        return Err(AdapterError::Verification(
            "candidate graph must be canonical loop Markdown".to_owned(),
        )
        .into());
    }
    if candidate.graph().run_id != request.run_id {
        return Err(AdapterError::Verification(
            "candidate graph belongs to another run".to_owned(),
        )
        .into());
    }
    let transition = validate_loop_transition(previous, &candidate)?;
    let topology_changed = previous.graph().edges != candidate.graph().edges;
    if steering && !topology_changed {
        return Err(AdapterError::Verification(
            "steer requires one topology-changing graph revision".to_owned(),
        )
        .into());
    }
    if !steering && topology_changed {
        return Err(AdapterError::Verification(
            "topology changes require the explicit loop steer command".to_owned(),
        )
        .into());
    }

    if transition == LoopTransitionOutcome::Idempotent {
        if steering || !request.evidence_files.is_empty() {
            return Err(AdapterError::Verification(
                "idempotent checkpoint cannot claim new evidence or steering".to_owned(),
            )
            .into());
        }
        return checkpoint_result(
            &request.run_id,
            &candidate,
            Vec::new(),
            Vec::new(),
            steering,
        );
    }

    let mut evidence = verify_new_evidence(&target, previous, &candidate, &request.evidence_files)?;
    validate_new_attempts(&target, &chain, &candidate)?;
    let changed_paths = publish_graph_revision(&target, &chain, &candidate, &canonical)?;
    evidence.push(Evidence {
        kind: "loop-graph",
        locator: portable_relative_path(&revision_path(
            &request.run_id,
            candidate.graph().revision,
        )?),
        digest: candidate.canonical_digest()?,
    });
    checkpoint_result(
        &request.run_id,
        &candidate,
        changed_paths,
        evidence,
        steering,
    )
}

fn publish_graph_revision(
    target: &PinnedTarget,
    chain: &LoadedGraphChain,
    candidate: &LoopGraphDocument,
    canonical: &[u8],
) -> Result<Vec<String>, LoopCliError> {
    let run_id = &candidate.graph().run_id;
    let revision = revision_path(run_id, candidate.graph().revision)?;
    let current = current_path(run_id)?;
    let revision_before = target.snapshot_bounded(&revision, MAX_GRAPH_BYTES)?;
    if revision_before
        .bytes()
        .is_some_and(|existing| existing != canonical)
    {
        return Err(AdapterError::Conflict(format!(
            "immutable loop revision already has different bytes: {}",
            revision.display()
        ))
        .into());
    }
    let current_before = FileSnapshot::File(chain.current_bytes.clone());
    ensure_graph_directory(target, &revisions_path(run_id)?)?;
    let revision_created = target.publish(&revision, &revision_before, canonical)?;
    let current_changed = match target.publish(&current, &current_before, canonical) {
        Ok(changed) => changed,
        Err(error) => {
            if revision_created {
                target.restore(&revision, &revision_before, canonical)?;
            }
            return Err(error.into());
        }
    };
    let mut changed_paths = Vec::new();
    if revision_created {
        changed_paths.push(portable_relative_path(&revision));
    }
    if current_changed {
        changed_paths.push(portable_relative_path(&current));
    }
    Ok(changed_paths)
}

fn checkpoint_result(
    run_id: &str,
    candidate: &LoopGraphDocument,
    changed_paths: Vec<String>,
    evidence: Vec<Evidence>,
    steering: bool,
) -> Result<ActionResult, LoopCliError> {
    let digest = candidate.canonical_digest()?;
    let (action, code, message) = if steering {
        (
            "SteerLoop",
            "hive.loop-steered",
            "loop topology steering checkpoint activated without dispatch",
        )
    } else {
        (
            "CheckpointLoop",
            "hive.loop-checkpointed",
            "loop checkpoint activated without dispatch",
        )
    };
    Ok(success_result(
        action,
        code,
        message,
        changed_paths,
        evidence,
        Some("recover the graph or prepare the next host-owned action".to_owned()),
        json!({
            "run_id": run_id,
            "graph_revision": candidate.graph().revision,
            "graph_digest": digest,
            "state": candidate.graph().state,
            "steering": steering,
        }),
    ))
}

struct SelectedDispatch {
    node_id: Option<String>,
    attempt: Option<u32>,
    role_id: Option<String>,
    backoff_seconds: Option<u64>,
}

struct ValidatedUsageAuthorization {
    evidence: Evidence,
    dispatch_authorization_digest: String,
}

fn select_dispatch(
    document: &LoopGraphDocument,
    request: &PrepareRequest,
) -> Result<SelectedDispatch, LoopCliError> {
    match request.kind {
        LoopDispatchKind::Steering => {
            if request.node_id.is_some() {
                return Err(AdapterError::Input(
                    "steering preparation must be graph-scoped".to_owned(),
                )
                .into());
            }
            Ok(SelectedDispatch {
                node_id: None,
                attempt: None,
                role_id: None,
                backoff_seconds: None,
            })
        }
        LoopDispatchKind::Node | LoopDispatchKind::Retry => {
            let mut eligible = Vec::new();
            for node_id in document.graph().ready_nodes()? {
                let decision = document.graph().retry_decision(&node_id)?;
                let selected = match (request.kind, decision) {
                    (LoopDispatchKind::Node, RetryDecision::FirstAttempt { attempt }) => {
                        Some((attempt, None))
                    }
                    (
                        LoopDispatchKind::Retry,
                        RetryDecision::Retry {
                            attempt,
                            backoff_seconds,
                        },
                    ) => Some((attempt, Some(backoff_seconds))),
                    _ => None,
                };
                if let Some((attempt, backoff_seconds)) = selected {
                    eligible.push((node_id, attempt, backoff_seconds));
                }
            }
            let selected = if let Some(requested) = request.node_id.as_deref() {
                eligible
                    .into_iter()
                    .find(|(node_id, _, _)| node_id == requested)
            } else {
                eligible.into_iter().next()
            }
            .ok_or_else(|| {
                AdapterError::Verification(format!(
                    "no evidence-ready node matches {:?} preparation",
                    request.kind
                ))
            })?;
            document.graph().validate_node_capabilities(&selected.0)?;
            let node = document.graph().node(&selected.0)?;
            Ok(SelectedDispatch {
                node_id: Some(selected.0),
                attempt: Some(selected.1),
                role_id: Some(node.executor_role_id.clone()),
                backoff_seconds: selected.2,
            })
        }
    }
}

fn session_scope_digest(host_scope: &str, session_id: &str) -> String {
    let mut material = Vec::with_capacity(host_scope.len() + session_id.len() + 1);
    material.extend_from_slice(host_scope.as_bytes());
    material.push(0);
    material.extend_from_slice(session_id.as_bytes());
    sha256_digest(&material)
}

fn usage_session_root(session_id_digest: &str) -> Result<PathBuf, LoopCliError> {
    require_digest(session_id_digest, "usage session digest")?;
    let hex = session_id_digest
        .strip_prefix("sha256:")
        .expect("validated digest has a prefix");
    Ok(Path::new(".hive/runtime/usage-guard/sessions").join(hex))
}

fn verify_usage_session_state(
    target: &PinnedTarget,
    envelope: &UsageAuthorizationEnvelope,
    capability: &CapabilityResolution,
) -> Result<(), LoopCliError> {
    let config = read_installed_config(target)?;
    let capability_host = serde_json::to_value(capability.host)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| AdapterError::Internal("cannot encode capability host".to_owned()))?;
    if !config.guard_enabled || config.primary_host != capability_host {
        return Err(AdapterError::OwnerBlocked(
            "installed usage guard is disabled or bound to another host".to_owned(),
        )
        .into());
    }
    let expected_session = session_scope_digest(&config.primary_host, &envelope.session_id);
    if envelope.session_id_digest != expected_session {
        return Err(AdapterError::Verification(
            "usage authorization session binding is invalid".to_owned(),
        )
        .into());
    }
    let root = usage_session_root(&expected_session)?;
    let control_path = root.join("control.json");
    if let Some(bytes) = target.read_optional(&control_path, USAGE_CONTROL_PATH_BYTES)? {
        let control: UsageSessionControl = parse_json(&bytes, "usage session control")?;
        if control.schema_version != 1
            || control.revision == 0
            || control.host_scope != config.primary_host
            || control.session_id_digest != expected_session
            || control.process_id != envelope.process_id
            || !control.guard_enabled
        {
            return Err(AdapterError::OwnerBlocked(
                "usage session control is stale, disabled, or incorrectly bound".to_owned(),
            )
            .into());
        }
    }
    let halt_path = root.join("halt.json");
    if let Some(bytes) = target.read_optional(&halt_path, USAGE_CONTROL_PATH_BYTES)? {
        let halt: UsageHaltMarker = parse_json(&bytes, "usage halt marker")?;
        if halt.schema_version != 1
            || halt.revision == 0
            || halt.host_scope != config.primary_host
            || halt.session_id_digest != expected_session
            || halt.process_id != envelope.process_id
            || !matches!(halt.decision.as_str(), "halted" | "usage-unknown")
            || !matches!(
                halt.selected_window.as_str(),
                "session" | "weekly" | "multiple" | "unknown"
            )
            || !(1..=99).contains(&halt.threshold_remaining_percent)
            || halt.measured_at == 0
            || require_digest(&halt.evidence_digest, "usage halt evidence digest").is_err()
        {
            return Err(
                AdapterError::Safety("usage halt marker is malformed or stale".to_owned()).into(),
            );
        }
        return Err(AdapterError::OwnerBlocked(
            "usage session is halted and cannot authorize loop preparation".to_owned(),
        )
        .into());
    }
    Ok(())
}

fn dispatch_authorization_path(
    locator: &str,
    authorization_id: &str,
) -> Result<PathBuf, LoopCliError> {
    require_digest(authorization_id, "dispatch authorization id")?;
    if locator.contains(['\0', '\r', '\n', '\\']) {
        return Err(
            AdapterError::Safety("dispatch authorization locator is unsafe".to_owned()).into(),
        );
    }
    let path = PathBuf::from(locator);
    validate_project_relative(&path).map_err(|error| AdapterError::Safety(error.to_string()))?;
    if portable_relative_path(&path) != locator
        || path.parent() != Some(Path::new(".hive/runtime/dispatch-authorizations"))
    {
        return Err(AdapterError::Safety(
            "dispatch authorization escaped its runtime namespace".to_owned(),
        )
        .into());
    }
    let hex = authorization_id
        .strip_prefix("sha256:")
        .expect("validated authorization digest has a prefix");
    let expected_name = format!("{hex}.json");
    if path.file_name().and_then(|name| name.to_str()) != Some(expected_name.as_str()) {
        return Err(AdapterError::Verification(
            "dispatch authorization locator does not match its id".to_owned(),
        )
        .into());
    }
    Ok(path)
}

fn authorization_record_digest(
    record: &DispatchAuthorizationRecord,
) -> Result<String, LoopCliError> {
    let payload = json!({
        "schema_version": record.schema_version,
        "authorization_id": record.authorization_id,
        "run_id": record.run_id,
        "status_revision": record.status_revision,
        "role_id": record.role_id,
        "brief_digest": record.brief_digest,
        "usage_evidence_digest": record.usage_evidence_digest,
        "state": record.state,
    });
    let bytes = serde_json_canonicalizer::to_vec(&payload)
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    Ok(sha256_digest(&bytes))
}

fn verify_dispatch_authorization(
    target: &PinnedTarget,
    current: &LoopGraphDocument,
    status: &RunStatusDocument,
    selected: &SelectedDispatch,
    envelope: &UsageAuthorizationEnvelope,
    capability: &CapabilityResolution,
) -> Result<String, LoopCliError> {
    verify_usage_session_state(target, envelope, capability)?;
    let expected_id = format!("sha256:{}", envelope.evidence_id);
    let relative =
        dispatch_authorization_path(&envelope.dispatch_authorization_locator, &expected_id)?;
    let bytes = read_fresh_target_file(target, &relative, MAX_PREPARED_BYTES)?;
    authenticate_dispatch_authorization_record(
        &bytes,
        &envelope.dispatch_authorization_digest,
        current,
        status,
        selected,
        envelope,
    )
}

fn authenticate_dispatch_authorization_record(
    bytes: &[u8],
    expected_file_digest: &str,
    current: &LoopGraphDocument,
    status: &RunStatusDocument,
    selected: &SelectedDispatch,
    envelope: &UsageAuthorizationEnvelope,
) -> Result<String, LoopCliError> {
    let expected_id = format!("sha256:{}", envelope.evidence_id);
    let file_digest = sha256_digest(bytes);
    if file_digest != expected_file_digest {
        return Err(AdapterError::Verification(
            "dispatch authorization file digest mismatch".to_owned(),
        )
        .into());
    }
    let record: DispatchAuthorizationRecord = parse_json(bytes, "dispatch authorization record")?;
    let role_matches = selected.role_id.as_deref().map_or_else(
        || {
            status
                .status()
                .active_roles
                .iter()
                .any(|role| role == &record.role_id)
        },
        |role| role == record.role_id,
    );
    if record.schema_version != 1
        || record.authorization_id != expected_id
        || record.run_id != current.graph().run_id
        || record.status_revision != status.status().revision
        || !role_matches
        || record.brief_digest != envelope.brief_digest
        || record.usage_evidence_digest != envelope.usage_evidence_digest
        || record.state != "issued"
        || record.record_digest != authorization_record_digest(&record)?
    {
        return Err(AdapterError::Verification(
            "dispatch authorization is stale, replayed, or bound to another run action".to_owned(),
        )
        .into());
    }
    Ok(file_digest)
}

fn verify_current_usage_authorization(
    target: &PinnedTarget,
    chain: &LoadedGraphChain,
    request: &PrepareRequest,
    selected: &SelectedDispatch,
    status: &RunStatusDocument,
    capability: &CapabilityResolution,
) -> Result<ValidatedUsageAuthorization, LoopCliError> {
    let current = chain.current();
    let evidence = current
        .graph()
        .evidence
        .iter()
        .find(|evidence| evidence.id == request.usage_evidence_id)
        .ok_or_else(|| {
            AdapterError::Verification(
                "prepare request usage authorization evidence is missing".to_owned(),
            )
        })?;
    if evidence.kind != EvidenceKind::UsageAuthorization
        || evidence.result != EvidenceResult::Allowed
        || evidence.graph_revision != current.graph().revision
        || evidence.subject_node_id != selected.node_id
        || evidence.attempt != selected.attempt
    {
        return Err(AdapterError::Verification(
            "prepare request usage authorization is stale or bound to another target".to_owned(),
        )
        .into());
    }
    let previous_revision = current.graph().revision.checked_sub(1).ok_or_else(|| {
        AdapterError::Verification(
            "initial graph cannot contain prepare authorization evidence".to_owned(),
        )
    })?;
    let previous = chain.document_at(previous_revision).ok_or_else(|| {
        AdapterError::Verification("usage authorization base revision is missing".to_owned())
    })?;
    let relative = evidence_locator(&request.run_id, &evidence.locator)?;
    let bytes = target.read_required(&relative, MAX_EVIDENCE_FILE_BYTES)?;
    if sha256_digest(&bytes) != evidence.digest {
        return Err(AdapterError::Verification(
            "prepare usage authorization digest mismatch".to_owned(),
        )
        .into());
    }
    let previous_digest = previous.canonical_digest()?;
    let envelope = verify_usage_envelope(&bytes, evidence, previous, current, &previous_digest)?;
    if envelope.dispatch_kind != request.kind
        || envelope.subject_node_id != selected.node_id
        || envelope.attempt != selected.attempt
        || envelope.brief_digest != request.brief_digest
    {
        return Err(AdapterError::Verification(
            "usage authorization does not bind the exact prepared dispatch".to_owned(),
        )
        .into());
    }
    let dispatch_authorization_digest =
        verify_dispatch_authorization(target, current, status, selected, &envelope, capability)?;
    Ok(ValidatedUsageAuthorization {
        evidence: Evidence {
            kind: "usage-authorization",
            locator: evidence.locator.clone(),
            digest: evidence.digest.clone(),
        },
        dispatch_authorization_digest,
    })
}

#[allow(clippy::too_many_lines)]
fn prepare((target_path, request_path): (PathBuf, PathBuf)) -> Result<ActionResult, LoopCliError> {
    let request_bytes = read_explicit_file(&request_path, MAX_REQUEST_BYTES)?;
    let request: PrepareRequest = parse_json(&request_bytes, "loop prepare request")?;
    if request.schema_version != 1 {
        return Err(AdapterError::Input(
            "loop prepare request schema_version must be 1".to_owned(),
        )
        .into());
    }
    validate_run_id(&request.run_id)?;
    validate_safe_id(&request.usage_evidence_id, "usage evidence id")?;
    require_digest(&request.expected_graph_digest, "expected graph digest")?;
    require_digest(&request.brief_digest, "brief digest")?;
    let capability_resolution = stable_explicit_path(&request.capability_resolution)?;

    let target = PinnedTarget::open(&target_path)?;
    let chain = load_graph_chain(&target, &request.run_id)?;
    let current = chain.current();
    let current_digest = current.canonical_digest()?;
    if request.expected_revision != current.graph().revision
        || request.expected_graph_digest != current_digest
    {
        return Err(AdapterError::Conflict("loop prepare request is stale".to_owned()).into());
    }

    verify_current_evidence_files(&target, &chain)?;
    let selected = select_dispatch(current, &request)?;
    let (capability, capability_bytes) =
        verify_fresh_capability_resolution(current, &selected, &capability_resolution)?;
    let (status, status_bytes) = load_dispatch_status(&target, current, &selected, &capability)?;
    let usage_authorization = verify_current_usage_authorization(
        &target,
        &chain,
        &request,
        &selected,
        &status,
        &capability,
    )?;
    let binding = LoopDispatchBinding {
        schema_version: 1,
        run_id: request.run_id.clone(),
        graph_revision: current.graph().revision,
        graph_digest: current_digest.clone(),
        kind: request.kind,
        node_id: selected.node_id.clone(),
        attempt: selected.attempt,
        role_id: selected.role_id.clone(),
        brief_digest: request.brief_digest.clone(),
        capability_snapshot_digest: current.graph().capability_snapshot_digest()?,
        usage_evidence_id: request.usage_evidence_id.clone(),
        prepared_only: true,
    };
    current.validate_dispatch(&binding)?;
    let dispatch_digest = binding.canonical_digest()?;
    let record = PreparedDispatchRecord {
        schema_version: 1,
        dispatch_id: dispatch_digest.clone(),
        binding: binding.clone(),
        prepared_only: true,
        spawned: false,
        dispatch_authorization_digest: usage_authorization.dispatch_authorization_digest.clone(),
        capability_resolution: capability_resolution.clone(),
        capability_resolution_digest: capability.evidence_digest.clone(),
        capability_resolution_file_digest: sha256_digest(&capability_bytes),
        run_status_digest: sha256_digest(&status_bytes),
    };
    let bytes = canonical_json(&record, "prepared dispatch record")?;
    if bytes.len() > MAX_PREPARED_BYTES {
        return Err(AdapterError::Internal(
            "prepared dispatch record exceeds its bound".to_owned(),
        )
        .into());
    }

    let relative = prepared_path(&request.run_id, &request.usage_evidence_id)?;
    let before = target.snapshot_bounded(&relative, MAX_PREPARED_BYTES)?;
    if before
        .bytes()
        .is_some_and(|existing| existing != bytes.as_slice())
    {
        return Err(AdapterError::Conflict(
            "usage authorization was already consumed by another dispatch".to_owned(),
        )
        .into());
    }

    let status_path = run_path(&request.run_id, "STATUS.md")?;
    let verify_inputs_unchanged = || -> Result<(), LoopCliError> {
        let observed_current =
            target.read_required(&current_path(&request.run_id)?, MAX_GRAPH_BYTES)?;
        let observed_status = target.read_required(&status_path, MAX_GRAPH_BYTES)?;
        let observed_capability = read_explicit_file(&capability_resolution, MAX_GRAPH_BYTES)?;
        if observed_current != chain.current_bytes
            || observed_status != status_bytes
            || observed_capability != capability_bytes
        {
            return Err(AdapterError::Conflict(
                "loop prepare inputs changed during optimistic validation".to_owned(),
            )
            .into());
        }
        verify_current_evidence_files(&target, &chain)?;
        let observed_usage = verify_current_usage_authorization(
            &target,
            &chain,
            &request,
            &selected,
            &status,
            &capability,
        )?;
        if observed_usage.dispatch_authorization_digest
            != usage_authorization.dispatch_authorization_digest
        {
            return Err(AdapterError::Conflict(
                "loop dispatch authorization changed during optimistic validation".to_owned(),
            )
            .into());
        }
        Ok(())
    };
    verify_inputs_unchanged()?;
    ensure_graph_directory(&target, &prepared_root(&request.run_id)?)?;
    let changed = match target.publish(&relative, &before, &bytes) {
        Ok(changed) => changed,
        Err(AdapterError::Conflict(_)) => {
            match target.publish(&relative, &FileSnapshot::File(bytes.clone()), &bytes) {
                Ok(false) => false,
                Ok(true) => {
                    return Err(AdapterError::Internal(
                        "identical dispatch replay unexpectedly republished bytes".to_owned(),
                    )
                    .into());
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(error) => return Err(error.into()),
    };
    if let Err(error) = verify_inputs_unchanged() {
        if changed {
            target.restore(&relative, &before, &bytes)?;
        }
        return Err(error);
    }
    let changed_paths = changed
        .then(|| portable_relative_path(&relative))
        .into_iter()
        .collect();
    let mut evidence = vec![usage_authorization.evidence];
    evidence.push(Evidence {
        kind: "capability-resolution",
        locator: capability_resolution.display().to_string(),
        digest: sha256_digest(&capability_bytes),
    });
    evidence.push(Evidence {
        kind: "run-status",
        locator: portable_relative_path(&status_path),
        digest: sha256_digest(&status_bytes),
    });
    evidence.push(Evidence {
        kind: "prepared-dispatch",
        locator: portable_relative_path(&relative),
        digest: sha256_digest(&bytes),
    });
    if !changed {
        return Ok(success_result(
            "PrepareLoopDispatch",
            "hive.loop-dispatch-already-prepared",
            "identical loop dispatch was already prepared; no executable action was re-issued",
            changed_paths,
            evidence,
            None,
            json!({
                "dispatch_id": record.dispatch_id,
                "already_prepared": true,
                "replay_blocked": true,
            }),
        ));
    }
    Ok(success_result(
        "PrepareLoopDispatch",
        "hive.loop-dispatch-prepared",
        "host-owned loop dispatch prepared without spawning",
        changed_paths,
        evidence,
        Some("the host may execute this exact prepared binding".to_owned()),
        json!({
            "binding": binding,
            "dispatch_id": record.dispatch_id,
            "dispatch_digest": dispatch_digest,
            "dispatch_authorization_digest": record.dispatch_authorization_digest,
            "capability_resolution_digest": record.capability_resolution_digest,
            "backoff_seconds": selected.backoff_seconds,
            "already_prepared": false,
            "replay_blocked": false,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::loop_graph::{
        AttemptOutcome, CapabilityRequirement, CapabilitySupport, CapabilitySupportLevel,
        EvidencePredicate, LoopEdge, LoopEvidence, LoopGraph, LoopNode, MinimumSupport,
        NodeAttempt, RetryPolicy, SteeringRecord,
    };
    use hive_core::run::RunStatus;
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    const RUN_ID: &str = "demo-run";
    const CRITERION: &str = "criterion-a";
    const SECOND_CRITERION: &str = "criterion-b";

    struct Fixture {
        _temp: TempDir,
        root: PathBuf,
        target: PathBuf,
        input: PathBuf,
        initial: LoopGraphDocument,
        plan_bytes: Vec<u8>,
        capability_path: PathBuf,
    }

    fn digest(label: &str) -> String {
        sha256_digest(label.as_bytes())
    }

    fn capability_bytes(support: CapabilitySupportLevel) -> Vec<u8> {
        let support = serde_json::to_value(support).expect("support value");
        let mut value = json!({
            "schema_version": 1,
            "host": "antigravity",
            "host_version": "fixture",
            "surface": "in-session",
            "detection": "unknown",
            "external_runtime": null,
            "resolved_owner": "host-native",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "supported",
                "subagents": support,
                "persistent-role-binding": "supported",
                "continuous-loop": "supported",
                "usage-sensor": "supported",
                "independent-judge": "unsupported"
            },
            "evidence": [{
                "source": "host-catalog",
                "locator": "fixture:antigravity-catalog",
                "outcome": "unavailable",
                "digest": digest("capability-source")
            }]
        });
        let canonical = serde_json_canonicalizer::to_vec(&value).expect("capability payload");
        value["evidence_digest"] = Value::String(sha256_digest(&canonical));
        serde_json::to_vec(&value).expect("capability bytes")
    }

    fn initial_graph(
        support: CapabilitySupportLevel,
        capability_digest: &str,
    ) -> LoopGraphDocument {
        LoopGraphDocument::from_graph(
            LoopGraph {
                schema_version: 1,
                run_id: RUN_ID.to_owned(),
                revision: 1,
                previous_revision_digest: None,
                state: LoopState::Active,
                terminal_reason: None,
                entry_nodes: vec!["A".to_owned()],
                required_criteria: vec![CRITERION.to_owned(), SECOND_CRITERION.to_owned()],
                passed_criteria: Vec::new(),
                nodes: vec![
                    LoopNode {
                        id: "A".to_owned(),
                        executor_role_id: "exec-a".to_owned(),
                        verifier_role_id: "verify-a".to_owned(),
                        criterion_ids: vec![CRITERION.to_owned()],
                        completion_predicates: vec![EvidencePredicate {
                            evidence_id: "artifact-a".to_owned(),
                            kind: EvidenceKind::Artifact,
                            result: EvidenceResult::Present,
                        }],
                        required_capabilities: vec![CapabilityRequirement {
                            name: "subagents".to_owned(),
                            minimum_support: MinimumSupport::Supported,
                        }],
                        retry_policy: RetryPolicy {
                            max_attempts: 3,
                            initial_backoff_seconds: 1,
                            backoff_multiplier: 2,
                            max_backoff_seconds: 8,
                            identical_failure_limit: 2,
                        },
                    },
                    LoopNode {
                        id: "B".to_owned(),
                        executor_role_id: "exec-b".to_owned(),
                        verifier_role_id: "verify-b".to_owned(),
                        criterion_ids: vec![SECOND_CRITERION.to_owned()],
                        completion_predicates: vec![EvidencePredicate {
                            evidence_id: "artifact-b".to_owned(),
                            kind: EvidenceKind::Artifact,
                            result: EvidenceResult::Present,
                        }],
                        required_capabilities: vec![CapabilityRequirement {
                            name: "subagents".to_owned(),
                            minimum_support: MinimumSupport::Supported,
                        }],
                        retry_policy: RetryPolicy {
                            max_attempts: 3,
                            initial_backoff_seconds: 1,
                            backoff_multiplier: 2,
                            max_backoff_seconds: 8,
                            identical_failure_limit: 2,
                        },
                    },
                ],
                edges: vec![LoopEdge {
                    id: "edge-a-b".to_owned(),
                    from: "A".to_owned(),
                    to: "B".to_owned(),
                    predicates: vec![EvidencePredicate {
                        evidence_id: "verify-a".to_owned(),
                        kind: EvidenceKind::IndependentVerification,
                        result: EvidenceResult::Passed,
                    }],
                }],
                evidence: Vec::new(),
                attempts: Vec::new(),
                capability_support: vec![
                    CapabilitySupport {
                        name: "subagents".to_owned(),
                        support,
                        evidence_digest: capability_digest.to_owned(),
                    },
                    CapabilitySupport {
                        name: "usage-sensor".to_owned(),
                        support: CapabilitySupportLevel::Supported,
                        evidence_digest: capability_digest.to_owned(),
                    },
                ],
                steering: Vec::new(),
            },
            b"# Durable loop\n".to_vec(),
        )
        .expect("initial graph")
    }

    fn fixture(support: CapabilitySupportLevel) -> Fixture {
        let temp = TempDir::new().expect("temporary fixture");
        let fixture_root = temp.path().canonicalize().expect("canonical fixture root");
        let target = fixture_root.join("consumer");
        let run = target.join(".hive/runs").join(RUN_ID);
        fs::create_dir_all(run.join("evidence")).expect("run directories");
        fs::create_dir_all(target.join(".hive/config")).expect("config directory");
        fs::write(
            target.join(".hive/config/harness.toml"),
            b"schema_version = 1\nharness_version = \"0.8.0\"\nsource_release_version = \"0.8.0\"\nprimary_host = \"antigravity\"\nusage_guard_enabled = true\ncodexbar_fallback_enabled = false\nusage_stop_remaining_percent = 60\n",
        )
        .expect("installed usage config");
        let plan_bytes = format!(
            "# Plan\n\n- [ ] {CRITERION}: finish first node\n- [ ] {SECOND_CRITERION}: finish second node\n"
        )
        .into_bytes();
        fs::write(run.join("PLAN.md"), &plan_bytes).expect("run plan");
        let capability_path = fixture_root.join("capabilities.json");
        let capability_bytes = capability_bytes(support);
        fs::write(&capability_path, &capability_bytes).expect("capability file");
        let capability =
            CapabilityResolution::parse_json(&capability_bytes).expect("capability resolution");
        let initial = initial_graph(support, &capability.evidence_digest);
        let binding = capability.owner_binding().expect("owner binding");
        let status = RunStatusDocument::from_status(
            RunStatus {
                schema_version: 1,
                run_id: RUN_ID.to_owned(),
                revision: 1,
                state: RunState::Executing,
                required_criteria: vec![CRITERION.to_owned(), SECOND_CRITERION.to_owned()],
                passed_criteria: Vec::new(),
                failed_criteria: Vec::new(),
                active_roles: vec!["exec-a".to_owned(), "exec-b".to_owned()],
                next_action: Some("prepare node A".to_owned()),
                latest_evidence: Vec::new(),
                blocker: None,
                updated_at: "2026-07-31T00:00:00Z".to_owned(),
                host: Some(binding.host),
                host_version: Some(binding.host_version),
                surface: Some(binding.surface),
                external_runtime: binding.external_runtime,
                resolved_owner: Some(binding.resolved_owner),
                resolution_evidence_digest: Some(binding.resolution_evidence_digest),
                subagent_support: Some(binding.subagent_support),
                resume_note: None,
                criterion_evidence: BTreeMap::new(),
            },
            b"# Run status\n".to_vec(),
        )
        .expect("run status");
        fs::write(
            run.join("STATUS.md"),
            status.encode_canonical().expect("status bytes"),
        )
        .expect("run status file");

        let session_digest = session_scope_digest("antigravity", "loop-session");
        let session_key = session_digest
            .strip_prefix("sha256:")
            .expect("session digest prefix");
        let session_root = target
            .join(".hive/runtime/usage-guard/sessions")
            .join(session_key);
        fs::create_dir_all(&session_root).expect("usage session directory");
        fs::write(
            session_root.join("control.json"),
            serde_json_canonicalizer::to_vec(&json!({
                "schema_version": 1,
                "host_scope": "antigravity",
                "session_id_digest": session_digest,
                "process_id": 4242,
                "guard_enabled": true,
                "revision": 1,
            }))
            .expect("usage control bytes"),
        )
        .expect("usage control");
        let input = fixture_root.join("initial.md");
        fs::write(
            &input,
            initial.encode_canonical().expect("initial graph bytes"),
        )
        .expect("initial input");
        Fixture {
            _temp: temp,
            root: fixture_root,
            target,
            input,
            initial,
            plan_bytes,
            capability_path,
        }
    }

    fn initialize_fixture(fixture: &Fixture) {
        let result =
            initialize((fixture.target.clone(), fixture.input.clone())).expect("initialize loop");
        assert_success_result(&result, "InitializeLoop", &["loop-graph"]);
    }

    #[allow(clippy::too_many_lines)]
    fn usage_checkpoint(
        fixture: &Fixture,
        kind: LoopDispatchKind,
        brief_digest: &str,
    ) -> LoopGraphDocument {
        let initial_digest = fixture.initial.canonical_digest().expect("initial digest");
        let authorization_binding = json!({
            "run_id": RUN_ID,
            "status_revision": 1,
            "role_id": "exec-a",
            "brief_digest": brief_digest,
        });
        let authorization_id = sha256_digest(
            &serde_json_canonicalizer::to_vec(&authorization_binding)
                .expect("authorization binding"),
        );
        let evidence_id = authorization_id
            .strip_prefix("sha256:")
            .expect("authorization id prefix")
            .to_owned();
        let authorization_locator =
            format!(".hive/runtime/dispatch-authorizations/{evidence_id}.json");
        let usage_evidence_digest = digest("usage-observation");
        let mut authorization = DispatchAuthorizationRecord {
            schema_version: 1,
            authorization_id,
            run_id: RUN_ID.to_owned(),
            status_revision: 1,
            role_id: "exec-a".to_owned(),
            brief_digest: brief_digest.to_owned(),
            usage_evidence_digest: usage_evidence_digest.clone(),
            state: "issued".to_owned(),
            record_digest: String::new(),
        };
        authorization.record_digest =
            authorization_record_digest(&authorization).expect("authorization digest");
        let authorization_bytes =
            canonical_json(&authorization, "dispatch authorization").expect("authorization bytes");
        let authorization_path = fixture.target.join(&authorization_locator);
        fs::create_dir_all(authorization_path.parent().expect("authorization parent"))
            .expect("authorization directory");
        fs::write(&authorization_path, &authorization_bytes).expect("dispatch authorization");

        let evidence_locator = format!(".hive/runs/{RUN_ID}/evidence/{evidence_id}.json");
        let session_id = "loop-session";
        let envelope = json!({
            "schema_version": 1,
            "evidence_id": evidence_id,
            "run_id": RUN_ID,
            "base_graph_revision": 1,
            "base_graph_digest": initial_digest,
            "authorized_graph_revision": 2,
            "dispatch_kind": kind,
            "subject_node_id": "A",
            "attempt": 1,
            "brief_digest": brief_digest,
            "decision": "allowed",
            "authenticated": true,
            "dispatch_authorization_locator": authorization_locator,
            "dispatch_authorization_digest": sha256_digest(&authorization_bytes),
            "usage_evidence_digest": usage_evidence_digest,
            "session_id": session_id,
            "session_id_digest": session_scope_digest("antigravity", session_id),
            "process_id": 4242,
        });
        let envelope_bytes = serde_json::to_vec(&envelope).expect("usage envelope");
        let evidence_digest = sha256_digest(&envelope_bytes);
        fs::write(fixture.target.join(&evidence_locator), &envelope_bytes).expect("usage evidence");

        let mut graph = fixture.initial.graph().clone();
        graph.revision = 2;
        graph.previous_revision_digest = Some(initial_digest.clone());
        graph.evidence.push(LoopEvidence {
            id: evidence_id.clone(),
            kind: EvidenceKind::UsageAuthorization,
            result: EvidenceResult::Allowed,
            graph_revision: 2,
            subject_node_id: Some("A".to_owned()),
            attempt: Some(1),
            producer_role_id: None,
            verifier_role_id: None,
            verification_authority: None,
            locator: evidence_locator.clone(),
            digest: evidence_digest.clone(),
            authenticated: true,
        });
        let candidate = LoopGraphDocument::from_graph(graph, fixture.initial.body().to_vec())
            .expect("usage candidate");
        let candidate_path = fixture.root.join("revision-2.md");
        fs::write(
            &candidate_path,
            candidate.encode_canonical().expect("candidate bytes"),
        )
        .expect("candidate input");
        let request_path = fixture.root.join("checkpoint-2.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 1,
                "expected_graph_digest": initial_digest,
                "candidate_graph": candidate_path,
                "evidence_files": [{
                    "evidence_id": evidence_id,
                    "locator": evidence_locator,
                    "digest": evidence_digest,
                }],
            }))
            .expect("checkpoint request"),
        )
        .expect("checkpoint request file");
        let result =
            checkpoint((fixture.target.clone(), request_path), false).expect("usage checkpoint");
        assert_success_result(&result, "CheckpointLoop", &["loop-evidence", "loop-graph"]);
        candidate
    }

    fn prepare_request(
        fixture: &Fixture,
        current: &LoopGraphDocument,
        brief_digest: &str,
    ) -> PathBuf {
        let request_path = fixture.root.join("prepare.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": current.graph().revision,
                "expected_graph_digest": current.canonical_digest().expect("current digest"),
                "kind": "node",
                "node_id": "A",
                "brief_digest": brief_digest,
                "usage_evidence_id": authorization_evidence_id(brief_digest),
                "capability_resolution": fixture.capability_path,
            }))
            .expect("prepare request"),
        )
        .expect("prepare request file");
        request_path
    }

    fn authorization_evidence_id(brief_digest: &str) -> String {
        let binding = json!({
            "run_id": RUN_ID,
            "status_revision": 1,
            "role_id": "exec-a",
            "brief_digest": brief_digest,
        });
        sha256_digest(&serde_json_canonicalizer::to_vec(&binding).expect("authorization binding"))
            .strip_prefix("sha256:")
            .expect("authorization id prefix")
            .to_owned()
    }

    fn fixture_prepared_path(fixture: &Fixture, brief_digest: &str) -> PathBuf {
        fixture
            .target
            .join(".hive/runs/demo-run/graph/prepared")
            .join(format!("{}.json", authorization_evidence_id(brief_digest)))
    }

    fn fixture_usage_evidence_path(fixture: &Fixture, brief_digest: &str) -> PathBuf {
        fixture
            .target
            .join(".hive/runs/demo-run/evidence")
            .join(format!("{}.json", authorization_evidence_id(brief_digest)))
    }

    fn fixture_authorization_path(fixture: &Fixture, brief_digest: &str) -> PathBuf {
        fixture
            .target
            .join(".hive/runtime/dispatch-authorizations")
            .join(format!("{}.json", authorization_evidence_id(brief_digest)))
    }

    fn fixture_usage_session_root(fixture: &Fixture) -> PathBuf {
        let digest = session_scope_digest("antigravity", "loop-session");
        fixture
            .target
            .join(".hive/runtime/usage-guard/sessions")
            .join(
                digest
                    .strip_prefix("sha256:")
                    .expect("session digest prefix"),
            )
    }

    fn expect_loop_error(result: Result<ActionResult, LoopCliError>) -> LoopCliError {
        match result {
            Ok(_) => panic!("operation unexpectedly succeeded"),
            Err(error) => error,
        }
    }

    fn assert_success_result(
        result: &ActionResult,
        expected_action: &str,
        expected_evidence_kinds: &[&str],
    ) {
        assert_eq!(result.schema_version, 1);
        assert_eq!(result.action, expected_action);
        assert_eq!(result.status, "success");
        assert_eq!(result.exit_code, 0);
        assert!(result.code.starts_with("hive."));
        assert!(!result.message.is_empty());
        assert_eq!(
            result
                .evidence
                .iter()
                .map(|evidence| evidence.kind)
                .collect::<Vec<_>>(),
            expected_evidence_kinds
        );
        let data = result.data.as_ref().expect("success result data");
        assert_eq!(data["prepared_only"], true);
        assert_eq!(data["spawned"], false);
    }

    fn attempt_checkpoint_request(
        fixture: &Fixture,
        current: &LoopGraphDocument,
        dispatch_id: &str,
        label: &str,
    ) -> (LoopGraphDocument, PathBuf) {
        let mut graph = current.graph().clone();
        graph.revision = 3;
        graph.previous_revision_digest =
            Some(current.canonical_digest().expect("revision 2 digest"));
        graph.attempts.push(NodeAttempt {
            node_id: "A".to_owned(),
            attempt: 1,
            graph_revision: 3,
            outcome: AttemptOutcome::Succeeded,
            dispatch_digest: dispatch_id.to_owned(),
            failure_fingerprint: None,
        });
        let candidate = LoopGraphDocument::from_graph(graph, current.body().to_vec())
            .expect("attempt checkpoint graph");
        let candidate_path = fixture.root.join(format!("{label}-3.md"));
        fs::write(
            &candidate_path,
            candidate.encode_canonical().expect("candidate bytes"),
        )
        .expect("candidate file");
        let request_path = fixture.root.join(format!("{label}-3.json"));
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 2,
                "expected_graph_digest": current.canonical_digest().expect("current digest"),
                "candidate_graph": candidate_path,
                "evidence_files": [],
            }))
            .expect("request bytes"),
        )
        .expect("request file");
        (candidate, request_path)
    }

    fn checkpoint_successful_attempt(
        fixture: &Fixture,
        current: &LoopGraphDocument,
    ) -> LoopGraphDocument {
        let prepared = fs::read(fixture_prepared_path(fixture, &digest("brief-a-1")))
            .expect("prepared record");
        let record: PreparedDispatchRecord =
            serde_json::from_slice(&prepared).expect("prepared record JSON");
        let (candidate, request_path) =
            attempt_checkpoint_request(fixture, current, &record.dispatch_id, "attempt-success");
        checkpoint((fixture.target.clone(), request_path), false)
            .expect("attempt checkpoint must consume prepared record");
        candidate
    }

    #[test]
    fn initialize_and_recover_preserve_run_markdown() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let plan_path = fixture.target.join(".hive/runs/demo-run/PLAN.md");
        let current_path = fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md");
        let revision_path = fixture
            .target
            .join(".hive/runs/demo-run/graph/revisions/0000000000000001.md");
        assert_eq!(fs::read(&plan_path).expect("plan"), fixture.plan_bytes);
        assert_eq!(
            fs::read(&current_path).expect("current"),
            fs::read(&revision_path).expect("revision")
        );

        let validated =
            validate((fixture.target.clone(), RUN_ID.to_owned())).expect("validate loop");
        assert_success_result(&validated, "ValidateLoop", &["loop-graph"]);

        let current_before = fs::read(&current_path).expect("current before");
        let result = recover((fixture.target.clone(), RUN_ID.to_owned())).expect("recover");
        assert_success_result(&result, "RecoverLoop", &["loop-graph"]);
        assert_eq!(
            result.data.as_ref().expect("data")["recovered_from"],
            "canonical-markdown"
        );
        assert_eq!(
            fs::read(current_path).expect("current after"),
            current_before
        );
        assert_eq!(fs::read(plan_path).expect("plan after"), fixture.plan_bytes);
    }

    #[test]
    fn prepare_is_idempotent_and_never_spawns() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let request = prepare_request(&fixture, &current, &brief_digest);
        let first = prepare((fixture.target.clone(), request.clone())).expect("prepare");
        assert_success_result(
            &first,
            "PrepareLoopDispatch",
            &[
                "usage-authorization",
                "capability-resolution",
                "run-status",
                "prepared-dispatch",
            ],
        );
        assert_eq!(first.changed_paths.len(), 1);
        assert_eq!(first.code, "hive.loop-dispatch-prepared");
        assert!(first.next_action.is_some());
        let first_data = first.data.as_ref().expect("first data");
        assert_eq!(first_data["prepared_only"], true);
        assert_eq!(first_data["spawned"], false);
        let second = prepare((fixture.target.clone(), request)).expect("idempotent prepare");
        assert_success_result(
            &second,
            "PrepareLoopDispatch",
            &[
                "usage-authorization",
                "capability-resolution",
                "run-status",
                "prepared-dispatch",
            ],
        );
        assert_eq!(second.code, "hive.loop-dispatch-already-prepared");
        assert!(second.next_action.is_none());
        assert!(second.changed_paths.is_empty());
        let second_data = second.data.as_ref().expect("second data");
        assert_eq!(second_data["dispatch_id"], first_data["dispatch_id"]);
        assert_eq!(second_data["already_prepared"], true);
        assert_eq!(second_data["replay_blocked"], true);
        assert!(second_data.get("binding").is_none());
        let prepared =
            fs::read(fixture_prepared_path(&fixture, &brief_digest)).expect("prepared record");
        let record: PreparedDispatchRecord =
            serde_json::from_slice(&prepared).expect("prepared JSON");
        assert!(record.prepared_only);
        assert!(!record.spawned);
        assert_eq!(record.dispatch_id, first_data["dispatch_id"]);
        assert_eq!(
            fs::read(fixture.target.join(".hive/runs/demo-run/PLAN.md")).expect("plan"),
            fixture.plan_bytes
        );
    }

    #[test]
    fn concurrent_identical_prepare_issues_one_executable_response() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let request = prepare_request(&fixture, &current, &brief_digest);

        let barrier = Arc::new(Barrier::new(2));
        let mut workers = Vec::new();
        for _ in 0..2 {
            let target = fixture.target.clone();
            let request = request.clone();
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                prepare((target, request)).expect("concurrent prepare")
            }));
        }
        let results = workers
            .into_iter()
            .map(|worker| worker.join().expect("prepare worker"))
            .collect::<Vec<_>>();

        assert_eq!(
            results
                .iter()
                .filter(|result| result.next_action.is_some())
                .count(),
            1
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| result.code == "hive.loop-dispatch-already-prepared")
                .count(),
            1
        );
        let dispatch_ids = results
            .iter()
            .map(|result| {
                result.data.as_ref().expect("result data")["dispatch_id"]
                    .as_str()
                    .expect("dispatch id")
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(dispatch_ids.len(), 1);
        for result in &results {
            assert_eq!(result.data.as_ref().expect("result data")["spawned"], false);
        }
    }

    #[test]
    fn unsupported_capability_returns_exact_code_without_prepare_mutation() {
        let fixture = fixture(CapabilitySupportLevel::Unverified);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let request = prepare_request(&fixture, &current, &brief_digest);
        let error = expect_loop_error(prepare((fixture.target.clone(), request)));
        assert_eq!(error.code(), "host_capability_unsupported");
        assert!(!fixture_prepared_path(&fixture, &brief_digest).exists());
    }

    #[test]
    fn tampered_current_evidence_blocks_prepare_without_mutation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        fs::write(fixture_usage_evidence_path(&fixture, &brief_digest), b"{}")
            .expect("tampered usage evidence");
        let request = prepare_request(&fixture, &current, &brief_digest);
        let error = expect_loop_error(prepare((fixture.target.clone(), request)));
        assert!(error.message().contains("current loop evidence changed"));
        assert!(!fixture_prepared_path(&fixture, &brief_digest).exists());
    }

    #[test]
    fn forged_dispatch_authorization_blocks_prepare_without_mutation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        fs::write(fixture_authorization_path(&fixture, &brief_digest), b"{}")
            .expect("forged dispatch authorization");
        let request = prepare_request(&fixture, &current, &brief_digest);
        let error = expect_loop_error(prepare((fixture.target.clone(), request)));
        assert!(error
            .message()
            .contains("dispatch authorization file digest mismatch"));
        assert!(!fixture_prepared_path(&fixture, &brief_digest).exists());
    }

    #[test]
    fn usage_halt_blocks_prepare_without_mutation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        fs::write(
            fixture_usage_session_root(&fixture).join("halt.json"),
            serde_json_canonicalizer::to_vec(&json!({
                "schema_version": 1,
                "host_scope": "antigravity",
                "session_id_digest": session_scope_digest("antigravity", "loop-session"),
                "process_id": 4242,
                "decision": "halted",
                "selected_window": "session",
                "threshold_remaining_percent": 60,
                "measured_at": 1,
                "evidence_digest": digest("halt-observation"),
                "revision": 2,
            }))
            .expect("halt marker bytes"),
        )
        .expect("halt marker");
        let request = prepare_request(&fixture, &current, &brief_digest);
        let error = expect_loop_error(prepare((fixture.target.clone(), request)));
        assert!(matches!(
            error,
            LoopCliError::Adapter(AdapterError::OwnerBlocked(_))
        ));
        assert!(!fixture_prepared_path(&fixture, &brief_digest).exists());
    }

    #[test]
    fn fresh_capability_drift_blocks_prepare_without_mutation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        fs::write(
            &fixture.capability_path,
            capability_bytes(CapabilitySupportLevel::BestEffort),
        )
        .expect("drifted capability file");
        let request = prepare_request(&fixture, &current, &brief_digest);
        let error = expect_loop_error(prepare((fixture.target.clone(), request)));
        assert_eq!(error.code(), "host_capability_unsupported");
        assert!(!fixture_prepared_path(&fixture, &brief_digest).exists());
    }

    #[test]
    fn attempt_without_exact_prepared_dispatch_is_rejected_without_activation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &digest("brief-a-1"));
        let current_bytes = fs::read(fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md"))
            .expect("current bytes");
        let mut graph = current.graph().clone();
        graph.revision = 3;
        graph.previous_revision_digest =
            Some(current.canonical_digest().expect("revision 2 digest"));
        graph.attempts.push(NodeAttempt {
            node_id: "A".to_owned(),
            attempt: 1,
            graph_revision: 3,
            outcome: AttemptOutcome::Failed,
            dispatch_digest: digest("unprepared-dispatch"),
            failure_fingerprint: Some(digest("failure")),
        });
        let candidate = LoopGraphDocument::from_graph(graph, current.body().to_vec())
            .expect("attempt candidate");
        let candidate_path = fixture.root.join("attempt-3.md");
        fs::write(
            &candidate_path,
            candidate.encode_canonical().expect("candidate bytes"),
        )
        .expect("candidate file");
        let request_path = fixture.root.join("attempt-3.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 2,
                "expected_graph_digest": current.canonical_digest().expect("current digest"),
                "candidate_graph": candidate_path,
                "evidence_files": [],
            }))
            .expect("request bytes"),
        )
        .expect("request file");
        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(error.message().contains("prepare-only dispatch record"));
        assert_eq!(
            fs::read(fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md"))
                .expect("current after rejection"),
            current_bytes
        );
    }

    #[test]
    fn forged_hash_shaped_prepared_record_cannot_authorize_attempt_checkpoint() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let capability_bytes = fs::read(&fixture.capability_path).expect("capability bytes");
        let capability =
            CapabilityResolution::parse_json(&capability_bytes).expect("capability resolution");
        let status_bytes =
            fs::read(fixture.target.join(".hive/runs/demo-run/STATUS.md")).expect("status bytes");
        let authorization_bytes = fs::read(fixture_authorization_path(&fixture, &brief_digest))
            .expect("dispatch authorization bytes");
        let binding = LoopDispatchBinding {
            schema_version: 1,
            run_id: RUN_ID.to_owned(),
            graph_revision: 2,
            graph_digest: current.canonical_digest().expect("current digest"),
            kind: LoopDispatchKind::Node,
            node_id: Some("A".to_owned()),
            attempt: Some(1),
            role_id: Some("exec-a".to_owned()),
            brief_digest: digest("caller-forged-brief"),
            capability_snapshot_digest: current
                .graph()
                .capability_snapshot_digest()
                .expect("capability snapshot"),
            usage_evidence_id: authorization_evidence_id(&brief_digest),
            prepared_only: true,
        };
        let dispatch_id = binding.canonical_digest().expect("dispatch id");
        let forged = PreparedDispatchRecord {
            schema_version: 1,
            dispatch_id: dispatch_id.clone(),
            binding,
            prepared_only: true,
            spawned: false,
            dispatch_authorization_digest: sha256_digest(&authorization_bytes),
            capability_resolution: fixture.capability_path.clone(),
            capability_resolution_digest: capability.evidence_digest,
            capability_resolution_file_digest: sha256_digest(&capability_bytes),
            run_status_digest: sha256_digest(&status_bytes),
        };
        let prepared_path = fixture_prepared_path(&fixture, &brief_digest);
        fs::create_dir_all(prepared_path.parent().expect("prepared parent"))
            .expect("prepared directory");
        fs::write(
            &prepared_path,
            canonical_json(&forged, "forged prepared record").expect("forged record bytes"),
        )
        .expect("forged prepared record");
        let current_path = fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md");
        let current_before = fs::read(&current_path).expect("current before");
        let (_, request_path) =
            attempt_checkpoint_request(&fixture, &current, &dispatch_id, "forged-prepared");

        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(error
            .message()
            .contains("usage envelope is bound to another action"));
        assert_eq!(
            fs::read(current_path).expect("current after"),
            current_before
        );
    }

    #[test]
    fn tampered_usage_envelope_blocks_attempt_checkpoint() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let prepare_path = prepare_request(&fixture, &current, &brief_digest);
        prepare((fixture.target.clone(), prepare_path)).expect("prepare dispatch");
        let prepared =
            fs::read(fixture_prepared_path(&fixture, &brief_digest)).expect("prepared record");
        let record: PreparedDispatchRecord =
            serde_json::from_slice(&prepared).expect("prepared JSON");
        let (_, request_path) =
            attempt_checkpoint_request(&fixture, &current, &record.dispatch_id, "tampered-usage");
        fs::write(fixture_usage_evidence_path(&fixture, &brief_digest), b"{}")
            .expect("tampered usage evidence");

        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(error.message().contains("usage envelope digest mismatch"));
    }

    #[test]
    fn tampered_dispatch_authorization_blocks_attempt_checkpoint() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let prepare_path = prepare_request(&fixture, &current, &brief_digest);
        prepare((fixture.target.clone(), prepare_path)).expect("prepare dispatch");
        let prepared =
            fs::read(fixture_prepared_path(&fixture, &brief_digest)).expect("prepared record");
        let record: PreparedDispatchRecord =
            serde_json::from_slice(&prepared).expect("prepared JSON");
        let (_, request_path) = attempt_checkpoint_request(
            &fixture,
            &current,
            &record.dispatch_id,
            "tampered-authorization",
        );
        fs::write(fixture_authorization_path(&fixture, &brief_digest), b"{}")
            .expect("tampered dispatch authorization");

        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(error
            .message()
            .contains("dispatch authorization file digest mismatch"));
    }

    #[test]
    fn tampered_capability_resolution_blocks_attempt_checkpoint() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let current = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let prepare_path = prepare_request(&fixture, &current, &brief_digest);
        prepare((fixture.target.clone(), prepare_path)).expect("prepare dispatch");
        let prepared =
            fs::read(fixture_prepared_path(&fixture, &brief_digest)).expect("prepared record");
        let record: PreparedDispatchRecord =
            serde_json::from_slice(&prepared).expect("prepared JSON");
        let (_, request_path) = attempt_checkpoint_request(
            &fixture,
            &current,
            &record.dispatch_id,
            "tampered-capability",
        );
        fs::write(
            &fixture.capability_path,
            capability_bytes(CapabilitySupportLevel::BestEffort),
        )
        .expect("tampered capability resolution");

        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(error.message().contains("capability file digest mismatch"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn forged_independent_verifier_envelope_is_rejected_without_activation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let brief_digest = digest("brief-a-1");
        let revision_two = usage_checkpoint(&fixture, LoopDispatchKind::Node, &brief_digest);
        let prepare_path = prepare_request(&fixture, &revision_two, &brief_digest);
        prepare((fixture.target.clone(), prepare_path)).expect("prepare dispatch");
        let current = checkpoint_successful_attempt(&fixture, &revision_two);
        let current_path = fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md");
        let current_before = fs::read(&current_path).expect("current before");

        let artifact_locator = format!(".hive/runs/{RUN_ID}/evidence/artifact-a.bin");
        let artifact_bytes = b"verified artifact".to_vec();
        let artifact_digest = sha256_digest(&artifact_bytes);
        fs::write(fixture.target.join(&artifact_locator), &artifact_bytes)
            .expect("artifact evidence");
        let verifier_locator = format!(".hive/runs/{RUN_ID}/evidence/verify-a.json");
        let forged_envelope = serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_id": "verify-a",
            "run_id": RUN_ID,
            "graph_revision": 4,
            "node_id": "A",
            "attempt": 1,
            "producer_role_id": "exec-a",
            "verifier_role_id": "exec-a",
            "authority": "deterministic",
            "result": "passed",
            "authenticated": false,
        }))
        .expect("forged verifier envelope");
        let verifier_digest = sha256_digest(&forged_envelope);
        fs::write(fixture.target.join(&verifier_locator), &forged_envelope)
            .expect("verifier evidence");

        let mut graph = current.graph().clone();
        graph.revision = 4;
        graph.previous_revision_digest =
            Some(current.canonical_digest().expect("revision 3 digest"));
        graph.evidence.push(LoopEvidence {
            id: "artifact-a".to_owned(),
            kind: EvidenceKind::Artifact,
            result: EvidenceResult::Present,
            graph_revision: 4,
            subject_node_id: Some("A".to_owned()),
            attempt: Some(1),
            producer_role_id: Some("exec-a".to_owned()),
            verifier_role_id: None,
            verification_authority: None,
            locator: artifact_locator.clone(),
            digest: artifact_digest.clone(),
            authenticated: false,
        });
        graph.evidence.push(LoopEvidence {
            id: "verify-a".to_owned(),
            kind: EvidenceKind::IndependentVerification,
            result: EvidenceResult::Passed,
            graph_revision: 4,
            subject_node_id: Some("A".to_owned()),
            attempt: Some(1),
            producer_role_id: Some("exec-a".to_owned()),
            verifier_role_id: Some("verify-a".to_owned()),
            verification_authority: Some(VerificationAuthority::Deterministic),
            locator: verifier_locator.clone(),
            digest: verifier_digest.clone(),
            authenticated: false,
        });
        graph.passed_criteria.push(CRITERION.to_owned());
        let candidate = LoopGraphDocument::from_graph(graph, current.body().to_vec())
            .expect("candidate is valid at the core metadata layer");
        let candidate_path = fixture.root.join("forged-verifier-4.md");
        fs::write(
            &candidate_path,
            candidate.encode_canonical().expect("candidate bytes"),
        )
        .expect("candidate file");
        let request_path = fixture.root.join("forged-verifier-4.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 3,
                "expected_graph_digest": current.canonical_digest().expect("current digest"),
                "candidate_graph": candidate_path,
                "evidence_files": [
                    {
                        "evidence_id": "artifact-a",
                        "locator": artifact_locator,
                        "digest": artifact_digest,
                    },
                    {
                        "evidence_id": "verify-a",
                        "locator": verifier_locator,
                        "digest": verifier_digest,
                    }
                ],
            }))
            .expect("checkpoint request"),
        )
        .expect("checkpoint request file");
        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(error.message().contains("independent verifier envelope"));
        assert_eq!(
            fs::read(current_path).expect("current after"),
            current_before
        );
    }

    #[test]
    fn judge_verifier_boolean_claim_without_quorum_is_rejected_without_mutation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        let evidence = LoopEvidence {
            id: "judge-verify-a".to_owned(),
            kind: EvidenceKind::IndependentVerification,
            result: EvidenceResult::Passed,
            graph_revision: 1,
            subject_node_id: Some("A".to_owned()),
            attempt: Some(1),
            producer_role_id: Some("exec-a".to_owned()),
            verifier_role_id: Some("verify-a".to_owned()),
            verification_authority: Some(VerificationAuthority::Judge),
            locator: format!(".hive/runs/{RUN_ID}/evidence/judge-verify-a.json"),
            digest: digest("judge-verify-a"),
            authenticated: true,
        };
        let envelope = serde_json::to_vec(&json!({
            "schema_version": 2,
            "evidence_id": evidence.id,
            "run_id": RUN_ID,
            "graph_revision": 1,
            "node_id": "A",
            "attempt": 1,
            "producer_role_id": "exec-a",
            "verifier_role_id": "verify-a",
            "authority": "judge",
            "result": "passed",
            "authenticated": true,
        }))
        .expect("judge verifier envelope");
        let error = verify_verifier_envelope(
            &PinnedTarget::open(&fixture.target).expect("pinned target"),
            &envelope,
            &evidence,
            &fixture.initial,
        )
        .expect_err("boolean Judge claim must not be accepted");
        assert!(error
            .message()
            .contains("judge verifier envelope is missing its quorum request"));
        assert!(!fixture
            .target
            .join(".hive/runs/demo-run/graph/CURRENT.md")
            .exists());
    }

    #[test]
    fn stale_checkpoint_does_not_mutate_current_pointer() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let current_path = fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md");
        let current_before = fs::read(&current_path).expect("current before");
        let request_path = fixture.root.join("stale.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 99,
                "expected_graph_digest": digest("stale"),
                "candidate_graph": fixture.input,
                "evidence_files": [],
            }))
            .expect("stale request"),
        )
        .expect("stale request file");
        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), false));
        assert!(matches!(
            error,
            LoopCliError::Adapter(AdapterError::Conflict(_))
        ));
        assert_eq!(
            fs::read(current_path).expect("current after"),
            current_before
        );
    }

    #[test]
    fn in_scope_steering_activates_one_exact_topology_revision() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let initial_digest = fixture.initial.canonical_digest().expect("initial digest");
        let mut graph = fixture.initial.graph().clone();
        graph.revision = 2;
        graph.previous_revision_digest = Some(initial_digest.clone());
        graph.edges[0].predicates = vec![EvidencePredicate {
            evidence_id: "artifact-a".to_owned(),
            kind: EvidenceKind::Artifact,
            result: EvidenceResult::Present,
        }];
        graph.steering.push(SteeringRecord {
            base_revision: 1,
            base_revision_digest: initial_digest.clone(),
            reason: "use the exact executor artifact edge".to_owned(),
            affected_edges: vec!["edge-a-b".to_owned()],
            user_boundary: UserBoundary::WithinApprovedScope,
            authorization_evidence_id: None,
        });
        let candidate = LoopGraphDocument::from_graph(graph, fixture.initial.body().to_vec())
            .expect("steering candidate");
        let candidate_path = fixture.root.join("steering-2.md");
        let candidate_bytes = candidate.encode_canonical().expect("candidate bytes");
        fs::write(&candidate_path, &candidate_bytes).expect("candidate file");
        let request_path = fixture.root.join("steering-2.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 1,
                "expected_graph_digest": initial_digest,
                "candidate_graph": candidate_path,
                "evidence_files": [],
            }))
            .expect("steering request"),
        )
        .expect("steering request file");
        let result = checkpoint((fixture.target.clone(), request_path), true)
            .expect("in-scope steering checkpoint");
        assert_success_result(&result, "SteerLoop", &["loop-graph"]);
        assert_eq!(result.data.as_ref().expect("data")["steering"], true);
        assert_eq!(
            fs::read(fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md"))
                .expect("current graph"),
            candidate_bytes
        );
    }

    #[test]
    fn replayed_explicit_steering_approval_is_rejected_without_activation() {
        let fixture = fixture(CapabilitySupportLevel::Supported);
        initialize_fixture(&fixture);
        let initial_digest = fixture.initial.canonical_digest().expect("initial digest");
        let current_path = fixture.target.join(".hive/runs/demo-run/graph/CURRENT.md");
        let current_before = fs::read(&current_path).expect("current before");
        let locator = format!(".hive/runs/{RUN_ID}/evidence/steering-approval.json");
        let envelope_bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_id": "steering-approval",
            "run_id": RUN_ID,
            "base_graph_revision": 1,
            "base_graph_digest": initial_digest,
            "authorized_graph_revision": 2,
            "user_boundary": "explicit-user-approval",
            "result": "allowed",
            "authenticated": true,
            "proposal_digest": digest("another-topology-proposal"),
        }))
        .expect("approval envelope");
        let evidence_digest = sha256_digest(&envelope_bytes);
        fs::write(fixture.target.join(&locator), &envelope_bytes).expect("approval evidence");

        let mut graph = fixture.initial.graph().clone();
        graph.revision = 2;
        graph.previous_revision_digest = Some(initial_digest.clone());
        graph.edges[0].predicates = vec![EvidencePredicate {
            evidence_id: "artifact-a".to_owned(),
            kind: EvidenceKind::Artifact,
            result: EvidenceResult::Present,
        }];
        graph.evidence.push(LoopEvidence {
            id: "steering-approval".to_owned(),
            kind: EvidenceKind::SteeringAuthorization,
            result: EvidenceResult::Allowed,
            graph_revision: 2,
            subject_node_id: None,
            attempt: None,
            producer_role_id: None,
            verifier_role_id: None,
            verification_authority: None,
            locator: locator.clone(),
            digest: evidence_digest.clone(),
            authenticated: true,
        });
        graph.steering.push(SteeringRecord {
            base_revision: 1,
            base_revision_digest: initial_digest.clone(),
            reason: "user approved one exact edge change".to_owned(),
            affected_edges: vec!["edge-a-b".to_owned()],
            user_boundary: UserBoundary::ExplicitUserApproval,
            authorization_evidence_id: Some("steering-approval".to_owned()),
        });
        let candidate = LoopGraphDocument::from_graph(graph, fixture.initial.body().to_vec())
            .expect("metadata-valid steering candidate");
        let candidate_path = fixture.root.join("explicit-steering-2.md");
        fs::write(
            &candidate_path,
            candidate.encode_canonical().expect("candidate bytes"),
        )
        .expect("candidate file");
        let request_path = fixture.root.join("explicit-steering-2.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "run_id": RUN_ID,
                "expected_revision": 1,
                "expected_graph_digest": initial_digest,
                "candidate_graph": candidate_path,
                "evidence_files": [{
                    "evidence_id": "steering-approval",
                    "locator": locator,
                    "digest": evidence_digest,
                }],
            }))
            .expect("steering request"),
        )
        .expect("steering request file");
        let error = expect_loop_error(checkpoint((fixture.target.clone(), request_path), true));
        assert!(error.message().contains("steering authorization envelope"));
        assert_eq!(
            fs::read(current_path).expect("current after"),
            current_before
        );
    }

    #[test]
    fn adapter_contains_no_runtime_or_scheduler_dependency() {
        let source = include_str!("loop_engineering.rs");
        for forbidden in [
            concat!("std::process", "::Command"),
            concat!("std::thread", "::sleep"),
            concat!("tokio", "::spawn"),
            concat!("tmux", " new-session"),
            concat!(".om", "x/"),
            concat!(".om", "c/"),
            concat!("om", "x run"),
            concat!("om", "c run"),
        ] {
            assert!(
                !source.contains(forbidden),
                "prepare-only loop adapter contains forbidden runtime primitive: {forbidden}"
            );
        }
    }
}
