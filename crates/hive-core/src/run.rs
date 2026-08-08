//! Pure durable-run, owner-binding, and dispatch-brief contracts.

use crate::role::RoleDocument;
use crate::{sha256_digest, validate_json_schema, validate_project_relative};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;

const CAPABILITY_SCHEMA: &str = include_str!("../../../schemas/capability-matrix.schema.json");
const RUN_STATUS_SCHEMA: &str = include_str!("../../../schemas/run-status.schema.json");
const DISPATCH_BRIEF_SCHEMA: &str = include_str!("../../../schemas/dispatch-brief.schema.json");
const MAX_RUN_DOCUMENT_BYTES: usize = 1024 * 1024;
const MAX_RUN_FRONTMATTER_BYTES: usize = 128 * 1024;
const MAX_RUN_BODY_BYTES: usize = 768 * 1024;
const MAX_CRITERIA: usize = 512;
const MAX_ACTIVE_ROLES: usize = 128;
const MAX_EVIDENCE_PER_CRITERION: usize = 64;
const MAX_LOCATOR_BYTES: usize = 1024;
const MAX_STATUS_TEXT_BYTES: usize = 8192;
const MAX_HOST_VERSION_BYTES: usize = 128;
const OWNER_PIN_FIELDS: [&str; 7] = [
    "host",
    "host_version",
    "surface",
    "external_runtime",
    "resolved_owner",
    "resolution_evidence_digest",
    "subagent_support",
];

/// Supported subscription-authenticated host families.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Host {
    /// `OpenAI` Codex.
    Codex,
    /// Claude Code.
    Claude,
    /// Gemini Antigravity.
    Antigravity,
}

/// Host surface that supplied normalized capability evidence.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostSurface {
    /// Desktop application surface.
    App,
    /// Command-line surface.
    Cli,
    /// Host plugin surface.
    Plugin,
    /// Current in-session metadata surface.
    InSession,
}

/// Normalized external-runtime detection state.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityDetection {
    /// Compatible external runtime evidence exists.
    Available,
    /// Both catalog and executable evidence prove absence.
    Absent,
    /// A runtime is present but not compatible.
    Incompatible,
    /// Absence cannot be proven.
    Unknown,
}

/// External orchestration runtime identifiers.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalRuntime {
    /// Oh My Codex.
    Omx,
    /// Oh My Claude Code.
    Omc,
}

/// Owner of model calls, subagents, and continuation for one run.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedOwner {
    /// Active host native capability.
    HostNative,
    /// Compatible OMX capability on Codex.
    Omx,
    /// Compatible OMC capability on Claude.
    Omc,
}

/// Truthful capability support level from the normalized host matrix.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportLevel {
    /// Qualified support.
    Supported,
    /// Available with explicit limitations.
    BestEffort,
    /// Known not to be available.
    Unsupported,
    /// Not sufficiently qualified.
    Unverified,
}

/// Capability evidence source.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityEvidenceSource {
    /// Active host catalog or plugin metadata.
    HostCatalog,
    /// Side-effect-free public executable probe.
    PublicExecutable,
}

/// Outcome of one capability evidence item.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityEvidenceOutcome {
    /// Evidence qualifies the expected external runtime.
    Compatible,
    /// Evidence proves the expected runtime is absent.
    Absent,
    /// Evidence finds an unsupported runtime version or surface.
    Incompatible,
    /// The evidence surface could not be queried.
    Unavailable,
}

/// One normalized capability evidence item.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityEvidence {
    /// Evidence source class.
    pub source: CapabilityEvidenceSource,
    /// Bounded public locator for the observed surface.
    pub locator: String,
    /// Normalized observation.
    pub outcome: CapabilityEvidenceOutcome,
    /// Digest of the exact observed evidence bytes.
    pub digest: String,
}

/// Existing capability-matrix JSON contract.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityResolution {
    /// Capability matrix schema version.
    pub schema_version: u32,
    /// Active host.
    pub host: Host,
    /// Active host version.
    pub host_version: String,
    /// Host surface.
    pub surface: HostSurface,
    /// Derived detection state.
    pub detection: CapabilityDetection,
    /// Expected external runtime, if one is present.
    pub external_runtime: Option<ExternalRuntime>,
    /// Host-native default or matching explicitly selected/pinned external owner.
    pub resolved_owner: ResolvedOwner,
    /// Qualified capability support map.
    pub capabilities: BTreeMap<String, JsonValue>,
    /// Optional hook-event qualification map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_events: Option<BTreeMap<String, JsonValue>>,
    /// Digest of the entire normalized object except this field.
    pub evidence_digest: String,
    /// Normalized positive and negative evidence.
    pub evidence: Vec<CapabilityEvidence>,
}

impl CapabilityResolution {
    /// Parse, schema-validate, digest-verify, and validate the selected owner.
    ///
    /// # Errors
    ///
    /// Returns a typed error when JSON, schema, digest, evidence, or declared
    /// owner does not satisfy the existing capability-matrix contract.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, CapabilityContractError> {
        if bytes.len() > MAX_RUN_DOCUMENT_BYTES {
            return Err(CapabilityContractError::TooLarge);
        }
        let value: JsonValue = serde_json::from_slice(bytes)
            .map_err(|error| CapabilityContractError::Malformed(error.to_string()))?;
        validate_json_schema(CAPABILITY_SCHEMA, &value, "capability matrix")
            .map_err(CapabilityContractError::Schema)?;
        let resolution: Self = serde_json::from_value(value.clone())
            .map_err(|error| CapabilityContractError::Malformed(error.to_string()))?;
        verify_capability_digest(&value, &resolution.evidence_digest)?;
        let derived = derive_owner(&resolution)?;
        if resolution.detection != derived.detection
            || resolution.external_runtime != derived.external_runtime
            || resolution.resolved_owner != derived.owner
        {
            return Err(CapabilityContractError::UserSelectedOwner);
        }
        let _ = resolution.subagent_support()?;
        Ok(resolution)
    }

    /// Return the qualified subagent support claim.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the required claim is missing or malformed.
    pub fn subagent_support(&self) -> Result<SupportLevel, CapabilityContractError> {
        let value = self
            .capabilities
            .get("subagents")
            .ok_or(CapabilityContractError::MissingCapability("subagents"))?;
        serde_json::from_value(value.clone()).map_err(|_| {
            CapabilityContractError::Malformed(
                "capabilities.subagents is not a support level".to_owned(),
            )
        })
    }

    /// Return whether at least one qualified host-native hook may be offered.
    ///
    /// External owners and absent, incompatible, or unknown detection remain
    /// ineligible. Call [`Self::host_native_hook_event_supported`] before
    /// authorizing any exact event.
    #[must_use]
    pub fn fallback_hooks_eligible(&self) -> bool {
        self.hook_events.as_ref().is_some_and(|events| {
            events
                .keys()
                .any(|event| self.host_native_hook_event_supported(event))
        })
    }

    /// Return whether the exact event has qualified host-native support.
    #[must_use]
    pub fn host_native_hook_event_supported(&self, event: &str) -> bool {
        matches!(self.detection, CapabilityDetection::Available)
            && matches!(self.resolved_owner, ResolvedOwner::HostNative)
            && self
                .hook_events
                .as_ref()
                .and_then(|events| events.get(event))
                .and_then(JsonValue::as_object)
                .and_then(|claim| claim.get("support"))
                .and_then(JsonValue::as_str)
                == Some("supported")
    }

    /// Derive the immutable owner binding stored in a run checkpoint.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the required subagent claim is malformed.
    pub fn owner_binding(&self) -> Result<OwnerBinding, CapabilityContractError> {
        Ok(OwnerBinding {
            host: self.host,
            host_version: self.host_version.clone(),
            surface: self.surface,
            external_runtime: self.external_runtime,
            resolved_owner: self.resolved_owner,
            resolution_evidence_digest: self.evidence_digest.clone(),
            subagent_support: self.subagent_support()?,
        })
    }
}

/// Immutable owner evidence pinned to one durable run.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBinding {
    /// Pinned active host.
    pub host: Host,
    /// Pinned host version.
    pub host_version: String,
    /// Pinned host surface.
    pub surface: HostSurface,
    /// Pinned external runtime, if any.
    pub external_runtime: Option<ExternalRuntime>,
    /// Pinned owner.
    pub resolved_owner: ResolvedOwner,
    /// Full capability-resolution evidence digest.
    pub resolution_evidence_digest: String,
    /// Pinned truthful subagent support claim.
    pub subagent_support: SupportLevel,
}

/// Capability-resolution contract errors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CapabilityContractError {
    /// Capability input exceeds the bounded contract.
    TooLarge,
    /// JSON or typed data is malformed.
    Malformed(String),
    /// Input violates the capability-matrix JSON Schema.
    Schema(String),
    /// The full-object evidence digest is invalid.
    EvidenceDigestMismatch,
    /// Evidence outcomes contradict one another.
    ContradictoryEvidence,
    /// Evidence cannot derive a supported state.
    IndeterminateEvidence,
    /// A required capability claim is missing.
    MissingCapability(&'static str),
    /// Declared owner is incompatible with the detected host/runtime path.
    UserSelectedOwner,
}

impl Display for CapabilityContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => formatter.write_str("capability matrix exceeds the bounded contract"),
            Self::Malformed(message) | Self::Schema(message) => formatter.write_str(message),
            Self::EvidenceDigestMismatch => {
                formatter.write_str("capability evidence digest does not bind the full object")
            }
            Self::ContradictoryEvidence => {
                formatter.write_str("capability evidence contains contradictory outcomes")
            }
            Self::IndeterminateEvidence => {
                formatter.write_str("capability evidence cannot derive a detection state")
            }
            Self::MissingCapability(name) => {
                write!(formatter, "capability matrix is missing {name}")
            }
            Self::UserSelectedOwner => formatter.write_str(
                "capability owner must remain host-native or match the compatible active-host runtime",
            ),
        }
    }
}

impl Error for CapabilityContractError {}

#[derive(Debug, Clone, Copy)]
struct DerivedOwner {
    detection: CapabilityDetection,
    external_runtime: Option<ExternalRuntime>,
    owner: ResolvedOwner,
}

fn verify_capability_digest(
    value: &JsonValue,
    expected: &str,
) -> Result<(), CapabilityContractError> {
    let mut payload = value.clone();
    payload
        .as_object_mut()
        .ok_or_else(|| {
            CapabilityContractError::Malformed("capability matrix must be an object".to_owned())
        })?
        .remove("evidence_digest");
    let canonical = serde_json_canonicalizer::to_vec(&payload)
        .map_err(|error| CapabilityContractError::Malformed(error.to_string()))?;
    if sha256_digest(&canonical) != expected {
        return Err(CapabilityContractError::EvidenceDigestMismatch);
    }
    Ok(())
}

fn derive_owner(
    resolution: &CapabilityResolution,
) -> Result<DerivedOwner, CapabilityContractError> {
    let outcomes = resolution
        .evidence
        .iter()
        .map(|item| item.outcome)
        .collect::<BTreeSet<_>>();
    let compatible = outcomes.contains(&CapabilityEvidenceOutcome::Compatible);
    let incompatible = outcomes.contains(&CapabilityEvidenceOutcome::Incompatible);
    let absent = outcomes.contains(&CapabilityEvidenceOutcome::Absent);
    let unavailable = outcomes.contains(&CapabilityEvidenceOutcome::Unavailable);
    if compatible && (incompatible || absent) || incompatible && absent {
        return Err(CapabilityContractError::ContradictoryEvidence);
    }
    if resolution.host == Host::Antigravity {
        if compatible || incompatible {
            return Err(CapabilityContractError::ContradictoryEvidence);
        }
        return Ok(DerivedOwner {
            detection: if complete_absence(resolution) && !unavailable {
                CapabilityDetection::Absent
            } else {
                CapabilityDetection::Unknown
            },
            external_runtime: None,
            owner: ResolvedOwner::HostNative,
        });
    }
    let expected_runtime = match resolution.host {
        Host::Codex => ExternalRuntime::Omx,
        Host::Claude => ExternalRuntime::Omc,
        Host::Antigravity => unreachable!("Antigravity returned above"),
    };
    let expected_owner = match expected_runtime {
        ExternalRuntime::Omx => ResolvedOwner::Omx,
        ExternalRuntime::Omc => ResolvedOwner::Omc,
    };
    if compatible {
        let owner = match resolution.resolved_owner {
            ResolvedOwner::HostNative => ResolvedOwner::HostNative,
            owner if owner == expected_owner => owner,
            _ => return Err(CapabilityContractError::UserSelectedOwner),
        };
        return Ok(DerivedOwner {
            detection: CapabilityDetection::Available,
            external_runtime: Some(expected_runtime),
            owner,
        });
    }
    if incompatible {
        return Ok(DerivedOwner {
            detection: CapabilityDetection::Incompatible,
            external_runtime: Some(expected_runtime),
            owner: ResolvedOwner::HostNative,
        });
    }
    if complete_absence(resolution) && !unavailable {
        return Ok(DerivedOwner {
            detection: CapabilityDetection::Absent,
            external_runtime: None,
            owner: ResolvedOwner::HostNative,
        });
    }
    if unavailable || absent {
        return Ok(DerivedOwner {
            detection: CapabilityDetection::Unknown,
            external_runtime: None,
            owner: ResolvedOwner::HostNative,
        });
    }
    Err(CapabilityContractError::IndeterminateEvidence)
}

fn complete_absence(resolution: &CapabilityResolution) -> bool {
    resolution.evidence.iter().any(|item| {
        item.source == CapabilityEvidenceSource::HostCatalog
            && item.outcome == CapabilityEvidenceOutcome::Absent
    }) && resolution.evidence.iter().any(|item| {
        item.source == CapabilityEvidenceSource::PublicExecutable
            && item.outcome == CapabilityEvidenceOutcome::Absent
    })
}

/// Parsed run plan with exact Markdown bytes and ordered criterion ids.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunPlan {
    body: Vec<u8>,
    criteria: Vec<String>,
}

impl RunPlan {
    /// Parse required Markdown checkboxes and extract unique safe criterion ids.
    ///
    /// Supported checkboxes are `- [ ]` and `- [x]`. The id may be bracketed
    /// (`[C1]`) or the first colon/whitespace-delimited token.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed checkboxes, unsafe or duplicate ids,
    /// invalid UTF-8, oversized input, or a plan with no required criteria.
    pub fn parse_markdown(bytes: &[u8]) -> Result<Self, RunContractError> {
        if bytes.len() > MAX_RUN_DOCUMENT_BYTES {
            return Err(RunContractError::TooLarge("run PLAN.md"));
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| RunContractError::InvalidUtf8("run PLAN.md"))?;
        let mut criteria = Vec::new();
        let mut seen = BTreeSet::new();
        for line in text.lines() {
            let trimmed = line.trim_start();
            let payload = checkbox_payload(trimmed)?;
            let Some(payload) = payload else {
                continue;
            };
            let criterion = criterion_id(payload)?;
            if !seen.insert(criterion.to_owned()) {
                return Err(RunContractError::DuplicateCriterion(criterion.to_owned()));
            }
            criteria.push(criterion.to_owned());
            if criteria.len() > MAX_CRITERIA {
                return Err(RunContractError::TooLarge("required criteria"));
            }
        }
        if criteria.is_empty() {
            return Err(RunContractError::NoCriteria);
        }
        Ok(Self {
            body: bytes.to_vec(),
            criteria,
        })
    }

    /// Return required criteria in source order.
    #[must_use]
    pub fn criteria(&self) -> &[String] {
        &self.criteria
    }

    /// Return exact PLAN.md bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Return the digest of the exact bounded plan bytes.
    #[must_use]
    pub fn digest(&self) -> String {
        sha256_digest(&self.body)
    }
}

fn checkbox_payload(line: &str) -> Result<Option<&str>, RunContractError> {
    for prefix in ["- [ ] ", "- [x] ", "- [X] "] {
        if let Some(payload) = line.strip_prefix(prefix) {
            if payload.trim().is_empty() {
                return Err(RunContractError::MalformedCriterion(line.to_owned()));
            }
            return Ok(Some(payload.trim()));
        }
    }
    if line == "- [ ]" || line == "- [x]" || line == "- [X]" {
        return Err(RunContractError::MalformedCriterion(line.to_owned()));
    }
    if line.starts_with("- [") {
        return Err(RunContractError::MalformedCriterion(line.to_owned()));
    }
    Ok(None)
}

fn criterion_id(payload: &str) -> Result<&str, RunContractError> {
    let candidate = if let Some(remainder) = payload.strip_prefix('[') {
        remainder
            .split_once(']')
            .map(|(identifier, _)| identifier)
            .ok_or_else(|| RunContractError::MalformedCriterion(payload.to_owned()))?
    } else {
        payload
            .split(|character: char| character.is_whitespace() || character == ':')
            .next()
            .unwrap_or_default()
    };
    if !valid_criterion_id(candidate) {
        return Err(RunContractError::MalformedCriterion(payload.to_owned()));
    }
    Ok(candidate)
}

/// Durable run state.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunState {
    /// Plan exists but execution has not started.
    Planned,
    /// Owner is executing the next action.
    Executing,
    /// Deterministic or independent verification is in progress.
    Verifying,
    /// A safety, authority, or external-owner blocker exists.
    Blocked,
    /// Usage policy forbids another dispatch.
    UsageLimited,
    /// A fresh session can resume explicitly.
    ResumeReady,
    /// Every required criterion passed.
    Succeeded,
    /// User or owner cancelled the run.
    Cancelled,
}

impl RunState {
    const fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Cancelled)
    }
}

/// Run STATUS.md frontmatter, including additive owner-pin fields.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunStatus {
    /// Run status schema version.
    pub schema_version: u32,
    /// Stable run id.
    pub run_id: String,
    /// Monotonic revision.
    pub revision: u64,
    /// Current durable state.
    pub state: RunState,
    /// Required criterion ids.
    pub required_criteria: Vec<String>,
    /// Passed criterion ids.
    pub passed_criteria: Vec<String>,
    /// Failed criterion ids.
    pub failed_criteria: Vec<String>,
    /// Stable role ids bound to the run.
    pub active_roles: Vec<String>,
    /// Next bounded host-owned action.
    pub next_action: Option<String>,
    /// Latest safe evidence locators.
    #[serde(default)]
    pub latest_evidence: Vec<String>,
    /// Current blocker, if any.
    #[serde(default)]
    pub blocker: Option<String>,
    /// Last update timestamp.
    pub updated_at: String,
    /// Pinned host.
    #[serde(default)]
    pub host: Option<Host>,
    /// Pinned host version.
    #[serde(default)]
    pub host_version: Option<String>,
    /// Pinned host surface.
    #[serde(default)]
    pub surface: Option<HostSurface>,
    /// Pinned external runtime.
    #[serde(default)]
    pub external_runtime: Option<ExternalRuntime>,
    /// Pinned resolved owner.
    #[serde(default)]
    pub resolved_owner: Option<ResolvedOwner>,
    /// Pinned capability evidence digest.
    #[serde(default)]
    pub resolution_evidence_digest: Option<String>,
    /// Pinned subagent support.
    #[serde(default)]
    pub subagent_support: Option<SupportLevel>,
    /// Bounded resume note for a fresh session.
    #[serde(default)]
    pub resume_note: Option<String>,
    /// Safe evidence locators keyed by passed criterion.
    #[serde(default)]
    pub criterion_evidence: BTreeMap<String, Vec<String>>,
}

/// Parsed STATUS.md with exact body and legacy pin-presence information.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunStatusDocument {
    status: RunStatus,
    body: Vec<u8>,
    pin_fields_present: bool,
    criterion_evidence_present: bool,
}

impl RunStatusDocument {
    /// Parse a schema-valid status for diagnosis.
    ///
    /// Legacy schema-version-1 documents without owner-pin fields may parse,
    /// but [`Self::validate_checkpoint`] rejects them for checkpoint or resume.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed frontmatter, schema violations, or
    /// invalid base criterion/state semantics.
    pub fn parse_markdown(bytes: &[u8]) -> Result<Self, RunContractError> {
        if bytes.len() > MAX_RUN_DOCUMENT_BYTES {
            return Err(RunContractError::TooLarge("run STATUS.md"));
        }
        let (frontmatter, body) = split_run_frontmatter(bytes)?;
        if frontmatter.len() > MAX_RUN_FRONTMATTER_BYTES {
            return Err(RunContractError::TooLarge("run status frontmatter"));
        }
        if body.len() > MAX_RUN_BODY_BYTES {
            return Err(RunContractError::TooLarge("run status body"));
        }
        std::str::from_utf8(body).map_err(|_| RunContractError::InvalidUtf8("run status body"))?;
        let value: JsonValue = serde_yaml::from_slice(frontmatter)
            .map_err(|error| RunContractError::Malformed(error.to_string()))?;
        validate_json_schema(RUN_STATUS_SCHEMA, &value, "run status")
            .map_err(RunContractError::Schema)?;
        let object = value.as_object().ok_or_else(|| {
            RunContractError::Malformed("run status must be an object".to_owned())
        })?;
        let pin_fields_present = OWNER_PIN_FIELDS
            .iter()
            .all(|field| object.contains_key(*field));
        let criterion_evidence_present = object.contains_key("criterion_evidence");
        let status: RunStatus = serde_json::from_value(value)
            .map_err(|error| RunContractError::Malformed(error.to_string()))?;
        validate_status_diagnostic(&status)?;
        Ok(Self {
            status,
            body: body.to_vec(),
            pin_fields_present,
            criterion_evidence_present,
        })
    }

    /// Create a checkpoint-valid document from typed state and bounded body.
    ///
    /// # Errors
    ///
    /// Returns a typed error when checkpoint semantics are incomplete.
    pub fn from_status(status: RunStatus, body: Vec<u8>) -> Result<Self, RunContractError> {
        if body.len() > MAX_RUN_BODY_BYTES {
            return Err(RunContractError::TooLarge("run status body"));
        }
        std::str::from_utf8(&body).map_err(|_| RunContractError::InvalidUtf8("run status body"))?;
        let document = Self {
            status,
            body,
            pin_fields_present: true,
            criterion_evidence_present: true,
        };
        document.validate_checkpoint()?;
        Ok(document)
    }

    /// Return parsed typed status.
    #[must_use]
    pub const fn status(&self) -> &RunStatus {
        &self.status
    }

    /// Return exact Markdown body bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Require complete owner pin and criterion evidence for checkpoint/resume.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an incomplete legacy pin or unsafe/missing
    /// evidence.
    pub fn validate_checkpoint(&self) -> Result<(), RunContractError> {
        validate_status_diagnostic(&self.status)?;
        if !self.pin_fields_present {
            return Err(RunContractError::IncompleteOwnerPin);
        }
        let _ = self.owner_binding()?;
        if !self.criterion_evidence_present {
            return Err(RunContractError::MissingCriterionEvidence);
        }
        validate_status_checkpoint(&self.status)?;
        validate_criterion_evidence(&self.status)?;
        validate_typed_status_schema(&self.status)
    }

    /// Return the complete immutable owner binding.
    ///
    /// # Errors
    ///
    /// Returns a typed error if any pin field is absent or contradictory.
    pub fn owner_binding(&self) -> Result<OwnerBinding, RunContractError> {
        if !self.pin_fields_present {
            return Err(RunContractError::IncompleteOwnerPin);
        }
        let binding = OwnerBinding {
            host: self
                .status
                .host
                .ok_or(RunContractError::IncompleteOwnerPin)?,
            host_version: self
                .status
                .host_version
                .clone()
                .ok_or(RunContractError::IncompleteOwnerPin)?,
            surface: self
                .status
                .surface
                .ok_or(RunContractError::IncompleteOwnerPin)?,
            external_runtime: self.status.external_runtime,
            resolved_owner: self
                .status
                .resolved_owner
                .ok_or(RunContractError::IncompleteOwnerPin)?,
            resolution_evidence_digest: self
                .status
                .resolution_evidence_digest
                .clone()
                .ok_or(RunContractError::IncompleteOwnerPin)?,
            subagent_support: self
                .status
                .subagent_support
                .ok_or(RunContractError::IncompleteOwnerPin)?,
        };
        validate_owner_binding(&binding)?;
        Ok(binding)
    }

    /// Encode canonical JSON frontmatter with LF delimiters and exact body.
    ///
    /// # Errors
    ///
    /// Returns a typed error when checkpoint semantics or the schema fail.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, RunContractError> {
        self.validate_checkpoint()?;
        let value = serde_json::to_value(&self.status)
            .map_err(|error| RunContractError::Malformed(error.to_string()))?;
        validate_json_schema(RUN_STATUS_SCHEMA, &value, "run status")
            .map_err(RunContractError::Schema)?;
        let frontmatter = serde_json_canonicalizer::to_string(&self.status)
            .map_err(|error| RunContractError::Malformed(error.to_string()))?;
        let mut output = Vec::with_capacity(frontmatter.len() + self.body.len() + 10);
        output.extend_from_slice(b"---\n");
        output.extend_from_slice(frontmatter.as_bytes());
        output.extend_from_slice(b"\n---\n");
        output.extend_from_slice(&self.body);
        Ok(output)
    }

    /// Return the digest of deterministic canonical STATUS.md bytes.
    ///
    /// # Errors
    ///
    /// Returns a typed error when canonical encoding fails.
    pub fn canonical_digest(&self) -> Result<String, RunContractError> {
        self.encode_canonical().map(|bytes| sha256_digest(&bytes))
    }
}

/// Legal transition result.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TransitionOutcome {
    /// Exact same-revision retry; no write is needed.
    Idempotent,
    /// Legal next revision.
    Advance,
}

/// Validate a pure run-state transition.
///
/// Exact retries are idempotent. Every other legal transition increments the
/// revision by exactly one, keeps the owner pin immutable, preserves required
/// criteria and prior passes/evidence, and respects terminal/resume gates.
///
/// # Errors
///
/// Returns a typed error for incomplete checkpoints or an illegal transition.
pub fn validate_transition(
    previous: &RunStatusDocument,
    next: &RunStatusDocument,
) -> Result<TransitionOutcome, RunContractError> {
    previous.validate_checkpoint()?;
    next.validate_checkpoint()?;
    if previous == next {
        return Ok(TransitionOutcome::Idempotent);
    }
    if next.status.revision
        != previous
            .status
            .revision
            .checked_add(1)
            .ok_or(RunContractError::InvalidRevision {
                expected: previous.status.revision,
                actual: next.status.revision,
            })?
    {
        return Err(RunContractError::InvalidRevision {
            expected: previous.status.revision.saturating_add(1),
            actual: next.status.revision,
        });
    }
    if previous.status.state.terminal() {
        return Err(RunContractError::TerminalStateImmutable);
    }
    if previous.owner_binding()? != next.owner_binding()? {
        return Err(RunContractError::OwnerPinChanged);
    }
    if as_set(&previous.status.required_criteria) != as_set(&next.status.required_criteria) {
        return Err(RunContractError::RequiredCriteriaChanged);
    }
    ensure_no_criteria_regression(&previous.status, &next.status)?;
    if !legal_state_transition(previous.status.state, next.status.state) {
        return Err(RunContractError::IllegalStateTransition {
            from: previous.status.state,
            to: next.status.state,
        });
    }
    Ok(TransitionOutcome::Advance)
}

fn ensure_no_criteria_regression(
    previous: &RunStatus,
    next: &RunStatus,
) -> Result<(), RunContractError> {
    let previous_passed = as_set(&previous.passed_criteria);
    let next_passed = as_set(&next.passed_criteria);
    if !previous_passed.is_subset(&next_passed) {
        return Err(RunContractError::CriterionRegression);
    }
    for criterion in previous_passed {
        let old = previous
            .criterion_evidence
            .get(&criterion)
            .map_or_else(BTreeSet::new, |items| as_set(items));
        let new = next
            .criterion_evidence
            .get(&criterion)
            .map_or_else(BTreeSet::new, |items| as_set(items));
        if !old.is_subset(&new) {
            return Err(RunContractError::CriterionRegression);
        }
    }
    Ok(())
}

fn legal_state_transition(from: RunState, to: RunState) -> bool {
    if from == to {
        return !from.terminal();
    }
    match from {
        RunState::Planned => matches!(
            to,
            RunState::Executing
                | RunState::Blocked
                | RunState::UsageLimited
                | RunState::ResumeReady
                | RunState::Cancelled
        ),
        RunState::Executing => matches!(
            to,
            RunState::Verifying
                | RunState::Blocked
                | RunState::UsageLimited
                | RunState::ResumeReady
                | RunState::Cancelled
        ),
        RunState::Verifying => matches!(
            to,
            RunState::Executing
                | RunState::Succeeded
                | RunState::Blocked
                | RunState::UsageLimited
                | RunState::ResumeReady
                | RunState::Cancelled
        ),
        RunState::Blocked | RunState::UsageLimited => {
            matches!(to, RunState::ResumeReady | RunState::Cancelled)
        }
        RunState::ResumeReady => matches!(
            to,
            RunState::Executing
                | RunState::Verifying
                | RunState::Blocked
                | RunState::UsageLimited
                | RunState::Cancelled
        ),
        RunState::Succeeded | RunState::Cancelled => false,
    }
}

fn validate_status_diagnostic(status: &RunStatus) -> Result<(), RunContractError> {
    if status.schema_version != 1 || !valid_run_id(&status.run_id) {
        return Err(RunContractError::InvalidRunId(status.run_id.clone()));
    }
    if status.required_criteria.is_empty() {
        return Err(RunContractError::NoCriteria);
    }
    let required = as_set(&status.required_criteria);
    let passed = as_set(&status.passed_criteria);
    let failed = as_set(&status.failed_criteria);
    if !passed.is_subset(&required) || !failed.is_subset(&required) {
        return Err(RunContractError::CriterionNotRequired);
    }
    if !passed.is_disjoint(&failed) {
        return Err(RunContractError::CriterionSetsOverlap);
    }
    let complete = passed == required && failed.is_empty();
    if (status.state == RunState::Succeeded) != complete {
        return Err(RunContractError::InvalidSuccessState);
    }
    Ok(())
}

fn validate_status_checkpoint(status: &RunStatus) -> Result<(), RunContractError> {
    if status.required_criteria.len() > MAX_CRITERIA
        || status.passed_criteria.len() > MAX_CRITERIA
        || status.failed_criteria.len() > MAX_CRITERIA
    {
        return Err(RunContractError::TooLarge("run criterion list"));
    }
    if status.active_roles.len() > MAX_ACTIVE_ROLES {
        return Err(RunContractError::TooLarge("active role list"));
    }
    validate_unique_ids(
        "required_criteria",
        &status.required_criteria,
        valid_criterion_id,
    )?;
    validate_unique_ids(
        "passed_criteria",
        &status.passed_criteria,
        valid_criterion_id,
    )?;
    validate_unique_ids(
        "failed_criteria",
        &status.failed_criteria,
        valid_criterion_id,
    )?;
    validate_unique_ids("active_roles", &status.active_roles, valid_role_id)?;
    validate_bounded_status_text(status.next_action.as_deref())?;
    validate_bounded_status_text(status.blocker.as_deref())?;
    validate_bounded_status_text(status.resume_note.as_deref())?;
    validate_state_fields(status)?;
    if status.latest_evidence.len() > MAX_CRITERIA {
        return Err(RunContractError::TooLarge("latest evidence list"));
    }
    let mut unique = BTreeSet::new();
    for locator in &status.latest_evidence {
        validate_evidence_locator(&status.run_id, locator)?;
        if !unique.insert(locator) {
            return Err(RunContractError::DuplicateEvidence(locator.clone()));
        }
    }
    Ok(())
}

fn validate_bounded_status_text(value: Option<&str>) -> Result<(), RunContractError> {
    if value.is_some_and(|text| text.len() > MAX_STATUS_TEXT_BYTES || text.contains('\0')) {
        return Err(RunContractError::TooLarge("run status text"));
    }
    Ok(())
}

fn validate_state_fields(status: &RunStatus) -> Result<(), RunContractError> {
    let next_present = status
        .next_action
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let blocker_present = status
        .blocker
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    match status.state {
        RunState::Succeeded => {
            if status.next_action.is_some()
                || status.blocker.is_some()
                || status.resume_note.is_some()
            {
                return Err(RunContractError::InconsistentStateFields);
            }
        }
        RunState::Cancelled => {
            if status.next_action.is_some() || status.resume_note.is_some() {
                return Err(RunContractError::InconsistentStateFields);
            }
        }
        RunState::Blocked | RunState::UsageLimited => {
            if !next_present || !blocker_present || status.resume_note.is_some() {
                return Err(RunContractError::InconsistentStateFields);
            }
        }
        RunState::ResumeReady => {
            let resume_present = status
                .resume_note
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            if !next_present || status.blocker.is_some() || !resume_present {
                return Err(RunContractError::InconsistentStateFields);
            }
        }
        RunState::Planned | RunState::Executing | RunState::Verifying => {
            if !next_present || status.blocker.is_some() || status.resume_note.is_some() {
                return Err(RunContractError::InconsistentStateFields);
            }
        }
    }
    Ok(())
}

fn validate_criterion_evidence(status: &RunStatus) -> Result<(), RunContractError> {
    let passed = as_set(&status.passed_criteria);
    let evidence_keys = status
        .criterion_evidence
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if evidence_keys != passed {
        return Err(RunContractError::MissingCriterionEvidence);
    }
    if status.criterion_evidence.len() > MAX_CRITERIA {
        return Err(RunContractError::TooLarge("criterion evidence map"));
    }
    for locators in status.criterion_evidence.values() {
        if locators.is_empty() {
            return Err(RunContractError::MissingCriterionEvidence);
        }
        if locators.len() > MAX_EVIDENCE_PER_CRITERION {
            return Err(RunContractError::TooLarge(
                "criterion evidence locator list",
            ));
        }
        let mut unique = BTreeSet::new();
        for locator in locators {
            validate_evidence_locator(&status.run_id, locator)?;
            if !unique.insert(locator) {
                return Err(RunContractError::DuplicateEvidence(locator.clone()));
            }
        }
    }
    Ok(())
}

fn validate_owner_binding(binding: &OwnerBinding) -> Result<(), RunContractError> {
    if binding.host_version.trim().is_empty()
        || binding.host_version.len() > MAX_HOST_VERSION_BYTES
        || binding.host_version.contains(['\0', '\r', '\n'])
        || !valid_digest(&binding.resolution_evidence_digest)
    {
        return Err(RunContractError::IncompleteOwnerPin);
    }
    let valid = match binding.resolved_owner {
        ResolvedOwner::Omx => {
            binding.host == Host::Codex && binding.external_runtime == Some(ExternalRuntime::Omx)
        }
        ResolvedOwner::Omc => {
            binding.host == Host::Claude && binding.external_runtime == Some(ExternalRuntime::Omc)
        }
        ResolvedOwner::HostNative => matches!(
            (binding.host, binding.external_runtime),
            (Host::Codex, None | Some(ExternalRuntime::Omx))
                | (Host::Claude, None | Some(ExternalRuntime::Omc))
                | (Host::Antigravity, None)
        ),
    };
    if !valid {
        return Err(RunContractError::IncompleteOwnerPin);
    }
    Ok(())
}

fn validate_typed_status_schema(status: &RunStatus) -> Result<(), RunContractError> {
    let value = serde_json::to_value(status)
        .map_err(|error| RunContractError::Malformed(error.to_string()))?;
    validate_json_schema(RUN_STATUS_SCHEMA, &value, "run status")
        .map_err(RunContractError::Schema)?;
    let frontmatter = serde_json_canonicalizer::to_vec(status)
        .map_err(|error| RunContractError::Malformed(error.to_string()))?;
    if frontmatter.len() > MAX_RUN_FRONTMATTER_BYTES {
        return Err(RunContractError::TooLarge("run status frontmatter"));
    }
    Ok(())
}

fn validate_unique_ids(
    field: &'static str,
    values: &[String],
    validator: fn(&str) -> bool,
) -> Result<(), RunContractError> {
    let mut unique = BTreeSet::new();
    for value in values {
        if !validator(value) {
            return Err(RunContractError::InvalidIdentifier {
                field,
                value: value.clone(),
            });
        }
        if !unique.insert(value) {
            return Err(RunContractError::DuplicateIdentifier {
                field,
                value: value.clone(),
            });
        }
    }
    Ok(())
}

fn validate_evidence_locator(run_id: &str, locator: &str) -> Result<(), RunContractError> {
    if locator.is_empty()
        || locator.len() > MAX_LOCATOR_BYTES
        || locator.contains(['\0', '\r', '\n'])
    {
        return Err(RunContractError::UnsafeEvidenceLocator(locator.to_owned()));
    }
    let Some((path, digest)) = locator.split_once('#') else {
        return Err(RunContractError::UnsafeEvidenceLocator(locator.to_owned()));
    };
    if !valid_digest(digest) {
        return Err(RunContractError::UnsafeEvidenceLocator(locator.to_owned()));
    }
    let path = Path::new(path);
    let evidence_root = Path::new(".hive")
        .join("runs")
        .join(run_id)
        .join("evidence");
    if validate_project_relative(path).is_err()
        || path
            .strip_prefix(evidence_root)
            .map_or(true, |relative| relative.as_os_str().is_empty())
    {
        return Err(RunContractError::UnsafeEvidenceLocator(locator.to_owned()));
    }
    Ok(())
}

fn valid_run_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (2..=127).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_role_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (2..=63).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_criterion_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'.'))
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn as_set(values: &[String]) -> BTreeSet<String> {
    values.iter().cloned().collect()
}

fn split_run_frontmatter(bytes: &[u8]) -> Result<(&[u8], &[u8]), RunContractError> {
    let (remainder, delimiter) = if let Some(remainder) = bytes.strip_prefix(b"---\n") {
        (remainder, b"\n---\n".as_slice())
    } else if let Some(remainder) = bytes.strip_prefix(b"---\r\n") {
        (remainder, b"\r\n---\r\n".as_slice())
    } else {
        return Err(RunContractError::Malformed(
            "run status frontmatter start is missing".to_owned(),
        ));
    };
    let index = remainder
        .windows(delimiter.len())
        .position(|window| window == delimiter)
        .ok_or_else(|| {
            RunContractError::Malformed("run status frontmatter end is missing".to_owned())
        })?;
    Ok((&remainder[..index], &remainder[index + delimiter.len()..]))
}

/// Pure run-contract errors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RunContractError {
    /// A bounded run artifact exceeded its maximum.
    TooLarge(&'static str),
    /// A Markdown artifact is not UTF-8.
    InvalidUtf8(&'static str),
    /// YAML/JSON or typed data is malformed.
    Malformed(String),
    /// Frontmatter violates its JSON Schema.
    Schema(String),
    /// Run id is invalid.
    InvalidRunId(String),
    /// No required criterion was found.
    NoCriteria,
    /// A criterion checkbox or id is malformed.
    MalformedCriterion(String),
    /// A criterion id occurs more than once.
    DuplicateCriterion(String),
    /// A bounded identifier is invalid.
    InvalidIdentifier { field: &'static str, value: String },
    /// A bounded identifier is duplicated.
    DuplicateIdentifier { field: &'static str, value: String },
    /// Passed or failed criterion is not required.
    CriterionNotRequired,
    /// Passed and failed criterion sets overlap.
    CriterionSetsOverlap,
    /// Succeeded is not exactly equivalent to complete required criteria.
    InvalidSuccessState,
    /// Next action, blocker, or resume note contradicts state.
    InconsistentStateFields,
    /// Evidence locator is unsafe.
    UnsafeEvidenceLocator(String),
    /// Passed criterion lacks safe evidence.
    MissingCriterionEvidence,
    /// Evidence locator is duplicated for one criterion.
    DuplicateEvidence(String),
    /// Legacy or partial owner pin cannot checkpoint or resume.
    IncompleteOwnerPin,
    /// Non-idempotent transition does not increment revision by one.
    InvalidRevision { expected: u64, actual: u64 },
    /// Terminal state cannot change.
    TerminalStateImmutable,
    /// Owner pin changed within a run.
    OwnerPinChanged,
    /// Required criteria changed within a run.
    RequiredCriteriaChanged,
    /// Passed criterion or evidence regressed.
    CriterionRegression,
    /// State transition is not in the legal table.
    IllegalStateTransition { from: RunState, to: RunState },
}

impl Display for RunContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge(label) => write!(formatter, "{label} exceeds the bounded contract"),
            Self::InvalidUtf8(label) => write!(formatter, "{label} must be UTF-8"),
            Self::Malformed(message) | Self::Schema(message) => formatter.write_str(message),
            Self::InvalidRunId(run_id) => write!(formatter, "invalid run id: {run_id}"),
            Self::NoCriteria => formatter.write_str("run requires at least one criterion"),
            Self::MalformedCriterion(value) => write!(formatter, "malformed criterion: {value}"),
            Self::DuplicateCriterion(value) => write!(formatter, "duplicate criterion: {value}"),
            Self::InvalidIdentifier { field, value } => {
                write!(formatter, "{field} contains an invalid id: {value}")
            }
            Self::DuplicateIdentifier { field, value } => {
                write!(formatter, "{field} contains a duplicate id: {value}")
            }
            Self::CriterionNotRequired => {
                formatter.write_str("passed and failed criteria must be required")
            }
            Self::CriterionSetsOverlap => {
                formatter.write_str("passed and failed criteria must be disjoint")
            }
            Self::InvalidSuccessState => formatter
                .write_str("succeeded must exactly match complete required criterion passes"),
            Self::InconsistentStateFields => {
                formatter.write_str("next_action, blocker, or resume_note contradicts run state")
            }
            Self::UnsafeEvidenceLocator(value) => {
                write!(formatter, "unsafe evidence locator: {value}")
            }
            Self::MissingCriterionEvidence => {
                formatter.write_str("every passed criterion requires safe evidence")
            }
            Self::DuplicateEvidence(value) => {
                write!(formatter, "duplicate criterion evidence: {value}")
            }
            Self::IncompleteOwnerPin => {
                formatter.write_str("checkpoint and resume require a complete owner pin")
            }
            Self::InvalidRevision { expected, actual } => {
                write!(formatter, "expected revision {expected}, found {actual}")
            }
            Self::TerminalStateImmutable => formatter.write_str("terminal run state is immutable"),
            Self::OwnerPinChanged => formatter.write_str("run owner pin cannot change"),
            Self::RequiredCriteriaChanged => {
                formatter.write_str("required criteria cannot change within a run")
            }
            Self::CriterionRegression => {
                formatter.write_str("passed criteria and their evidence cannot regress")
            }
            Self::IllegalStateTransition { from, to } => {
                write!(
                    formatter,
                    "illegal run state transition: {from:?} -> {to:?}"
                )
            }
        }
    }
}

impl Error for RunContractError {}

/// Expected no-mid-run-switch result.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwnerContinuity {
    /// Current capability evidence exactly matches the pin.
    Matched,
    /// Dispatch must stop without changing owner.
    Blocked(OwnerBlockReason),
    /// The pinned external runtime is present but incompatible.
    Unsupported(OwnerUnsupportedReason),
}

/// Typed blocked owner-continuity reasons.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwnerBlockReason {
    /// No current normalized capability object is available.
    CapabilityEvidenceMissing,
    /// Pinned external runtime disappeared or cannot be observed.
    ExternalRuntimeMissing,
    /// Active host changed.
    HostChanged,
    /// Owner or runtime changed.
    OwnerChanged,
    /// Full normalized evidence changed without an explicit rebind.
    EvidenceDrift,
}

/// Typed unsupported owner-continuity reasons.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwnerUnsupportedReason {
    /// Pinned external runtime now reports incompatible.
    ExternalRuntimeIncompatible,
}

/// Compare current capability evidence with an immutable run owner pin.
#[must_use]
pub fn verify_owner_continuity(
    pinned: &OwnerBinding,
    current: Option<&CapabilityResolution>,
) -> OwnerContinuity {
    let Some(current) = current else {
        return if pinned.external_runtime.is_some()
            && pinned.resolved_owner != ResolvedOwner::HostNative
        {
            OwnerContinuity::Blocked(OwnerBlockReason::ExternalRuntimeMissing)
        } else {
            OwnerContinuity::Blocked(OwnerBlockReason::CapabilityEvidenceMissing)
        };
    };
    if current.host != pinned.host {
        return OwnerContinuity::Blocked(OwnerBlockReason::HostChanged);
    }
    if pinned.resolved_owner != ResolvedOwner::HostNative
        && current.detection == CapabilityDetection::Incompatible
    {
        return OwnerContinuity::Unsupported(OwnerUnsupportedReason::ExternalRuntimeIncompatible);
    }
    if current.resolved_owner != pinned.resolved_owner
        || current.external_runtime != pinned.external_runtime
    {
        return if pinned.resolved_owner != ResolvedOwner::HostNative
            && matches!(
                current.detection,
                CapabilityDetection::Absent | CapabilityDetection::Unknown
            ) {
            OwnerContinuity::Blocked(OwnerBlockReason::ExternalRuntimeMissing)
        } else {
            OwnerContinuity::Blocked(OwnerBlockReason::OwnerChanged)
        };
    }
    if current.host_version != pinned.host_version
        || current.surface != pinned.surface
        || current.evidence_digest != pinned.resolution_evidence_digest
    {
        return OwnerContinuity::Blocked(OwnerBlockReason::EvidenceDrift);
    }
    OwnerContinuity::Matched
}

/// Provider-neutral bounded data prepared for a host-owned dispatch.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchBrief {
    /// Dispatch-brief schema version.
    pub schema_version: u32,
    /// Stable run id.
    pub run_id: String,
    /// Exact run revision.
    pub run_revision: u64,
    /// Stable role id.
    pub role_id: String,
    /// Work assigned to the persistent role.
    pub responsibilities: Vec<String>,
    /// Work the persistent role must not absorb.
    pub non_responsibilities: Vec<String>,
    /// Verification duties required from the role.
    pub verification_duties: Vec<String>,
    /// Bounded next action.
    pub next_action: String,
    /// Pinned host.
    pub host: Host,
    /// Pinned host version.
    pub host_version: String,
    /// Pinned host surface.
    pub surface: HostSurface,
    /// Pinned external runtime.
    pub external_runtime: Option<ExternalRuntime>,
    /// Pinned owner.
    pub resolved_owner: ResolvedOwner,
    /// Pinned capability evidence digest.
    pub resolution_evidence_digest: String,
    /// Truthful subagent support.
    pub subagent_support: SupportLevel,
    /// Role-selected project-relative context.
    pub context_paths: Vec<String>,
    /// Role-selected project-relative write scope.
    pub write_scope: Vec<String>,
    /// Remaining required criterion ids.
    pub acceptance_criteria: Vec<String>,
    /// Safe prior evidence locators.
    pub evidence: Vec<String>,
    /// Optional safe handoff path.
    pub handoff_path: Option<String>,
    /// Optional bounded resume note.
    pub resume_note: Option<String>,
    /// Always true: Hive prepared data and did not launch anything.
    pub prepared_only: bool,
}

impl DispatchBrief {
    /// Encode a deterministic schema-valid brief.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the brief is malformed or falsely claims it
    /// performed a launch.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, DispatchContractError> {
        if !self.prepared_only {
            return Err(DispatchContractError::InvalidBrief(
                "dispatch brief must remain prepare-only".to_owned(),
            ));
        }
        let value = serde_json::to_value(self)
            .map_err(|error| DispatchContractError::InvalidBrief(error.to_string()))?;
        validate_json_schema(DISPATCH_BRIEF_SCHEMA, &value, "dispatch brief")
            .map_err(DispatchContractError::InvalidBrief)?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|error| DispatchContractError::InvalidBrief(error.to_string()))
    }

    /// Return the digest of deterministic brief bytes.
    ///
    /// # Errors
    ///
    /// Returns a typed error when brief encoding fails.
    pub fn canonical_digest(&self) -> Result<String, DispatchContractError> {
        self.encode_canonical().map(|bytes| sha256_digest(&bytes))
    }
}

/// Dispatch preparation errors that never trigger a launch or owner fallback.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DispatchContractError {
    /// Run or role contract is invalid.
    InvalidBrief(String),
    /// Plan and status criterion ids differ.
    PlanStatusMismatch,
    /// Requested role is not active in the run.
    RoleNotActive,
    /// Persistent role assignment does not match this run.
    AssignmentMismatch {
        expected_run_id: String,
        actual_assignment: Option<String>,
    },
    /// Persistent role handoff path is not scoped below this run.
    HandoffRunMismatch { run_id: String, path: String },
    /// Run has no next action to prepare.
    MissingNextAction,
    /// Run state does not authorize preparation of another dispatch.
    RunNotDispatchable(RunState),
    /// Current evidence cannot safely continue the pin.
    Blocked(OwnerBlockReason),
    /// Required subagent/runtime capability is unsupported.
    Unsupported(DispatchUnsupportedReason),
}

/// Typed unsupported dispatch reasons.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DispatchUnsupportedReason {
    /// Owner capability is unsupported or unverified.
    SubagentsUnavailable,
    /// Pinned external runtime is now incompatible.
    ExternalRuntimeIncompatible,
}

impl Display for DispatchContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBrief(message) => formatter.write_str(message),
            Self::PlanStatusMismatch => {
                formatter.write_str("run PLAN.md and STATUS.md criteria differ")
            }
            Self::RoleNotActive => formatter.write_str("role is not active in this run"),
            Self::AssignmentMismatch {
                expected_run_id,
                actual_assignment,
            } => write!(
                formatter,
                "role assignment mismatch: expected {expected_run_id}, found {actual_assignment:?}"
            ),
            Self::HandoffRunMismatch { run_id, path } => {
                write!(
                    formatter,
                    "role handoff path is outside run {run_id}: {path}"
                )
            }
            Self::MissingNextAction => formatter.write_str("run has no next action to prepare"),
            Self::RunNotDispatchable(state) => {
                write!(
                    formatter,
                    "run state does not authorize dispatch: {state:?}"
                )
            }
            Self::Blocked(reason) => write!(formatter, "dispatch blocked: {reason:?}"),
            Self::Unsupported(reason) => write!(formatter, "dispatch unsupported: {reason:?}"),
        }
    }
}

impl Error for DispatchContractError {}

/// Prepare a provider-neutral dispatch brief without spawning or invoking a runtime.
///
/// # Errors
///
/// Returns a typed blocked/unsupported outcome for owner drift or insufficient
/// subagent support. This function performs no filesystem or process operation.
pub fn prepare_dispatch_brief(
    plan: &RunPlan,
    status: &RunStatusDocument,
    role: &RoleDocument,
    current: Option<&CapabilityResolution>,
) -> Result<DispatchBrief, DispatchContractError> {
    status
        .validate_checkpoint()
        .map_err(|error| DispatchContractError::InvalidBrief(error.to_string()))?;
    role.validate_runtime()
        .map_err(|error| DispatchContractError::InvalidBrief(error.to_string()))?;
    if !matches!(
        status.status.state,
        RunState::Executing | RunState::Verifying
    ) {
        return Err(DispatchContractError::RunNotDispatchable(
            status.status.state,
        ));
    }
    if as_set(plan.criteria()) != as_set(&status.status.required_criteria) {
        return Err(DispatchContractError::PlanStatusMismatch);
    }
    if !status.status.active_roles.contains(&role.profile().role_id) {
        return Err(DispatchContractError::RoleNotActive);
    }
    validate_role_run_binding(role, &status.status.run_id)?;
    let binding = status
        .owner_binding()
        .map_err(|error| DispatchContractError::InvalidBrief(error.to_string()))?;
    match verify_owner_continuity(&binding, current) {
        OwnerContinuity::Matched => {}
        OwnerContinuity::Blocked(reason) => {
            return Err(DispatchContractError::Blocked(reason));
        }
        OwnerContinuity::Unsupported(OwnerUnsupportedReason::ExternalRuntimeIncompatible) => {
            return Err(DispatchContractError::Unsupported(
                DispatchUnsupportedReason::ExternalRuntimeIncompatible,
            ));
        }
    }
    if matches!(
        binding.subagent_support,
        SupportLevel::Unsupported | SupportLevel::Unverified
    ) {
        return Err(DispatchContractError::Unsupported(
            DispatchUnsupportedReason::SubagentsUnavailable,
        ));
    }
    let next_action = status
        .status
        .next_action
        .clone()
        .ok_or(DispatchContractError::MissingNextAction)?;
    let passed = as_set(&status.status.passed_criteria);
    let acceptance_criteria = status
        .status
        .required_criteria
        .iter()
        .filter(|criterion| !passed.contains(*criterion))
        .cloned()
        .collect();
    let evidence = collect_dispatch_evidence(&status.status);
    let brief = DispatchBrief {
        schema_version: 1,
        run_id: status.status.run_id.clone(),
        run_revision: status.status.revision,
        role_id: role.profile().role_id.clone(),
        responsibilities: role.profile().responsibilities.clone(),
        non_responsibilities: role.profile().non_responsibilities.clone(),
        verification_duties: role.profile().verification_duties.clone(),
        next_action,
        host: binding.host,
        host_version: binding.host_version,
        surface: binding.surface,
        external_runtime: binding.external_runtime,
        resolved_owner: binding.resolved_owner,
        resolution_evidence_digest: binding.resolution_evidence_digest,
        subagent_support: binding.subagent_support,
        context_paths: role.profile().context_paths.clone(),
        write_scope: role.profile().write_scope.clone(),
        acceptance_criteria,
        evidence,
        handoff_path: role.profile().handoff_path.clone(),
        resume_note: status.status.resume_note.clone(),
        prepared_only: true,
    };
    let _ = brief.encode_canonical()?;
    Ok(brief)
}

fn validate_role_run_binding(
    role: &RoleDocument,
    run_id: &str,
) -> Result<(), DispatchContractError> {
    if role.profile().current_assignment.as_deref() != Some(run_id) {
        return Err(DispatchContractError::AssignmentMismatch {
            expected_run_id: run_id.to_owned(),
            actual_assignment: role.profile().current_assignment.clone(),
        });
    }
    let expected = Path::new(".hive")
        .join("runs")
        .join(run_id)
        .join("HANDOFF.md");
    let path = role.profile().handoff_path.as_deref().unwrap_or("");
    if Path::new(path) != expected {
        return Err(DispatchContractError::HandoffRunMismatch {
            run_id: run_id.to_owned(),
            path: path.to_owned(),
        });
    }
    Ok(())
}

fn collect_dispatch_evidence(status: &RunStatus) -> Vec<String> {
    let mut evidence = BTreeSet::new();
    evidence.extend(status.latest_evidence.iter().cloned());
    for locators in status.criterion_evidence.values() {
        evidence.extend(locators.iter().cloned());
    }
    evidence.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{
        prepare_dispatch_brief, validate_transition, CapabilityContractError, CapabilityResolution,
        DispatchContractError, ExternalRuntime, Host, HostSurface, OwnerBlockReason,
        OwnerContinuity, ResolvedOwner, RunContractError, RunPlan, RunState, RunStatus,
        RunStatusDocument, SupportLevel, TransitionOutcome,
    };
    use crate::role::RoleDocument;
    use crate::sha256_digest;
    use serde_json::{json, Value as JsonValue};
    use std::collections::BTreeMap;

    fn with_digest(mut value: JsonValue) -> Vec<u8> {
        let canonical =
            serde_json_canonicalizer::to_vec(&value).expect("capability should canonicalize");
        value["evidence_digest"] = JsonValue::String(sha256_digest(&canonical));
        serde_json::to_vec(&value).expect("capability should encode")
    }

    fn compatible_capability(host_version: &str) -> CapabilityResolution {
        let value = json!({
            "schema_version": 1,
            "host": "codex",
            "host_version": host_version,
            "surface": "cli",
            "detection": "available",
            "external_runtime": "omx",
            "resolved_owner": "omx",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "supported",
                "subagents": "supported",
                "persistent-role-binding": "supported",
                "continuous-loop": "supported",
                "usage-sensor": "unverified",
                "independent-judge": "supported"
            },
            "evidence": [{
                "source": "host-catalog",
                "locator": "fixture:codex-catalog",
                "outcome": "compatible",
                "digest": format!("sha256:{}", "a".repeat(64))
            }]
        });
        CapabilityResolution::parse_json(&with_digest(value)).expect("compatible capability")
    }

    fn absent_capability() -> CapabilityResolution {
        let value = json!({
            "schema_version": 1,
            "host": "codex",
            "host_version": "fixture",
            "surface": "cli",
            "detection": "absent",
            "external_runtime": null,
            "resolved_owner": "host-native",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "best-effort",
                "subagents": "best-effort",
                "persistent-role-binding": "best-effort",
                "continuous-loop": "unsupported",
                "usage-sensor": "unsupported",
                "independent-judge": "best-effort"
            },
            "evidence": [
                {
                    "source": "host-catalog",
                    "locator": "fixture:empty-catalog",
                    "outcome": "absent",
                    "digest": format!("sha256:{}", "b".repeat(64))
                },
                {
                    "source": "public-executable",
                    "locator": "fixture:omx-not-found",
                    "outcome": "absent",
                    "digest": format!("sha256:{}", "c".repeat(64))
                }
            ]
        });
        CapabilityResolution::parse_json(&with_digest(value)).expect("absent capability")
    }

    fn incompatible_capability() -> CapabilityResolution {
        let value = json!({
            "schema_version": 1,
            "host": "codex",
            "host_version": "fixture",
            "surface": "cli",
            "detection": "incompatible",
            "external_runtime": "omx",
            "resolved_owner": "host-native",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "best-effort",
                "subagents": "best-effort",
                "persistent-role-binding": "best-effort",
                "continuous-loop": "unsupported",
                "usage-sensor": "unsupported",
                "independent-judge": "best-effort"
            },
            "evidence": [{
                "source": "public-executable",
                "locator": "fixture:omx-unsupported",
                "outcome": "incompatible",
                "digest": format!("sha256:{}", "d".repeat(64))
            }]
        });
        CapabilityResolution::parse_json(&with_digest(value)).expect("incompatible capability")
    }

    fn unknown_capability() -> CapabilityResolution {
        let value = json!({
            "schema_version": 1,
            "host": "codex",
            "host_version": "fixture",
            "surface": "cli",
            "detection": "unknown",
            "external_runtime": null,
            "resolved_owner": "host-native",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "best-effort",
                "subagents": "best-effort",
                "persistent-role-binding": "best-effort",
                "continuous-loop": "unsupported",
                "usage-sensor": "unsupported",
                "independent-judge": "best-effort"
            },
            "evidence": [{
                "source": "host-catalog",
                "locator": "fixture:catalog-unavailable",
                "outcome": "unavailable",
                "digest": format!("sha256:{}", "e".repeat(64))
            }]
        });
        CapabilityResolution::parse_json(&with_digest(value)).expect("unknown capability")
    }

    fn role() -> RoleDocument {
        role_for(Some("demo"), Some(".hive/runs/demo/HANDOFF.md"))
    }

    fn role_for(assignment: Option<&str>, handoff_path: Option<&str>) -> RoleDocument {
        let assignment = assignment.unwrap_or("null");
        let handoff_path = handoff_path.unwrap_or("null");
        let bytes = format!(
            "---\nschema_version: 1\nrole_id: reviewer\ndisplay_name: Reviewer\nresponsibilities: [verify]\nnon_responsibilities: [implement]\ncontext_paths: [docs/**]\nallowed_capabilities: [filesystem-read]\nwrite_scope: [.hive/runs/]\nverification_duties: [attach evidence]\ncurrent_assignment: {assignment}\nhandoff_path: {handoff_path}\n---\n# Reviewer\n"
        );
        RoleDocument::parse(bytes.as_bytes(), "reviewer").expect("valid role")
    }

    fn status_for(
        capability: &CapabilityResolution,
        state: RunState,
        revision: u64,
        passed: &[&str],
    ) -> RunStatusDocument {
        let binding = capability.owner_binding().expect("owner binding");
        let required = vec!["build".to_owned(), "tests".to_owned()];
        let passed_criteria = passed
            .iter()
            .map(|criterion| (*criterion).to_owned())
            .collect::<Vec<_>>();
        let criterion_evidence = passed_criteria
            .iter()
            .map(|criterion| {
                (
                    criterion.clone(),
                    vec![format!(
                        ".hive/runs/demo/evidence/{criterion}.json#sha256:{}",
                        "0".repeat(64)
                    )],
                )
            })
            .collect::<BTreeMap<_, _>>();
        let complete = passed_criteria.len() == required.len();
        RunStatusDocument::from_status(
            RunStatus {
                schema_version: 1,
                run_id: "demo".to_owned(),
                revision,
                state,
                required_criteria: required,
                passed_criteria,
                failed_criteria: Vec::new(),
                active_roles: vec!["reviewer".to_owned()],
                next_action: (!complete).then(|| "continue verification".to_owned()),
                latest_evidence: Vec::new(),
                blocker: None,
                updated_at: "2026-07-24T00:00:00Z".to_owned(),
                host: Some(binding.host),
                host_version: Some(binding.host_version),
                surface: Some(binding.surface),
                external_runtime: binding.external_runtime,
                resolved_owner: Some(binding.resolved_owner),
                resolution_evidence_digest: Some(binding.resolution_evidence_digest),
                subagent_support: Some(binding.subagent_support),
                resume_note: None,
                criterion_evidence,
            },
            b"# Status\n".to_vec(),
        )
        .expect("valid status")
    }

    #[test]
    fn compatible_capability_accepts_host_native_default_or_matching_external_pin() {
        let compatible = compatible_capability("fixture");
        assert_eq!(compatible.resolved_owner, ResolvedOwner::Omx);
        assert_eq!(compatible.external_runtime, Some(ExternalRuntime::Omx));

        let mut value: JsonValue = serde_json::from_slice(&with_digest(json!({
            "schema_version": 1,
            "host": "codex",
            "host_version": "fixture",
            "surface": "cli",
            "detection": "available",
            "external_runtime": "omx",
            "resolved_owner": "omx",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "supported",
                "subagents": "supported",
                "persistent-role-binding": "supported",
                "continuous-loop": "supported",
                "usage-sensor": "unverified",
                "independent-judge": "supported"
            },
            "evidence": [{
                "source": "host-catalog",
                "locator": "fixture:catalog",
                "outcome": "compatible",
                "digest": format!("sha256:{}", "a".repeat(64))
            }]
        })))
        .expect("valid JSON");
        value["resolved_owner"] = json!("host-native");
        let bytes = with_digest({
            value
                .as_object_mut()
                .expect("object")
                .remove("evidence_digest");
            value
        });
        let host_native = CapabilityResolution::parse_json(&bytes)
            .expect("compatible capability may retain the host-native default");
        assert_eq!(host_native.resolved_owner, ResolvedOwner::HostNative);
        assert_eq!(host_native.external_runtime, Some(ExternalRuntime::Omx));

        let mut mismatched: JsonValue =
            serde_json::from_slice(&bytes).expect("host-native capability JSON");
        mismatched
            .as_object_mut()
            .expect("object")
            .remove("evidence_digest");
        mismatched["resolved_owner"] = json!("omc");
        assert!(matches!(
            CapabilityResolution::parse_json(&with_digest(mismatched)),
            Err(CapabilityContractError::Schema(_) | CapabilityContractError::UserSelectedOwner)
        ));

        let mut digest_tamper: JsonValue = serde_json::from_slice(&with_digest(json!({
            "schema_version": 1,
            "host": "codex",
            "host_version": "fixture",
            "surface": "cli",
            "detection": "unknown",
            "external_runtime": null,
            "resolved_owner": "host-native",
            "capabilities": {
                "instructions": "supported",
                "simple-question-isolation": "best-effort",
                "subagents": "best-effort",
                "persistent-role-binding": "best-effort",
                "continuous-loop": "unsupported",
                "usage-sensor": "unsupported",
                "independent-judge": "best-effort"
            },
            "evidence": [{
                "source": "host-catalog",
                "locator": "fixture:unavailable",
                "outcome": "unavailable",
                "digest": format!("sha256:{}", "f".repeat(64))
            }]
        })))
        .expect("valid JSON");
        digest_tamper["host_version"] = json!("tampered");
        assert_eq!(
            CapabilityResolution::parse_json(
                &serde_json::to_vec(&digest_tamper).expect("tampered JSON")
            ),
            Err(CapabilityContractError::EvidenceDigestMismatch)
        );
    }

    #[test]
    fn optional_hooks_require_exact_supported_host_native_events() {
        let mut compatible = compatible_capability("fixture");
        assert!(!compatible.fallback_hooks_eligible());
        assert!(!absent_capability().fallback_hooks_eligible());
        assert!(!incompatible_capability().fallback_hooks_eligible());
        assert!(!unknown_capability().fallback_hooks_eligible());

        compatible.resolved_owner = ResolvedOwner::HostNative;
        compatible.hook_events = Some(BTreeMap::from([
            (
                "PreToolUse".to_owned(),
                json!({"support": "best-effort", "evidence": []}),
            ),
            (
                "Stop".to_owned(),
                json!({"support": "supported", "evidence": []}),
            ),
        ]));
        assert!(compatible.fallback_hooks_eligible());
        assert!(compatible.host_native_hook_event_supported("Stop"));
        assert!(!compatible.host_native_hook_event_supported("PreToolUse"));

        compatible.resolved_owner = ResolvedOwner::Omx;
        assert!(!compatible.fallback_hooks_eligible());
        assert!(!compatible.host_native_hook_event_supported("Stop"));
    }

    #[test]
    fn run_plan_rejects_duplicate_malformed_and_missing_criteria() {
        let valid =
            RunPlan::parse_markdown(b"# Plan\n\n- [ ] [build] Compile\n- [x] tests: Test\n")
                .expect("valid plan");
        assert_eq!(valid.criteria(), ["build", "tests"]);
        assert_eq!(
            valid.body(),
            b"# Plan\n\n- [ ] [build] Compile\n- [x] tests: Test\n"
        );
        assert!(matches!(
            RunPlan::parse_markdown(b"- [ ] build: one\n- [x] build: two\n"),
            Err(RunContractError::DuplicateCriterion(_))
        ));
        assert!(matches!(
            RunPlan::parse_markdown(b"- [ ] ../escape: bad\n"),
            Err(RunContractError::MalformedCriterion(_))
        ));
        assert!(matches!(
            RunPlan::parse_markdown(b"- [q] build: malformed\n"),
            Err(RunContractError::MalformedCriterion(_))
        ));
        assert_eq!(
            RunPlan::parse_markdown(b"# No checklist\n"),
            Err(RunContractError::NoCriteria)
        );
    }

    #[test]
    fn legacy_status_parses_for_diagnosis_but_cannot_checkpoint() {
        let bytes = b"---\nschema_version: 1\nrun_id: demo\nrevision: 1\nstate: executing\nrequired_criteria: [build, tests]\npassed_criteria: []\nfailed_criteria: []\nactive_roles: [reviewer]\nnext_action: build\nlatest_evidence: []\nblocker: null\nupdated_at: 2026-07-24T00:00:00Z\n---\n# Legacy\n";
        let document = RunStatusDocument::parse_markdown(bytes).expect("legacy status diagnosis");
        assert_eq!(
            document.validate_checkpoint(),
            Err(RunContractError::IncompleteOwnerPin)
        );
        assert!(matches!(
            RunStatusDocument::parse_markdown(b"---\n{bad\n---\n# Status\n"),
            Err(RunContractError::Malformed(_))
        ));
    }

    #[test]
    fn legacy_zero_four_status_acceptance_is_preserved_for_diagnosis() {
        let bytes = b"---\nschema_version: 1\nrun_id: legacy-run\nrevision: 1\nstate: resume-ready\nrequired_criteria: ['criterion with spaces']\npassed_criteria: []\nfailed_criteria: []\nactive_roles: ['Review Team']\nnext_action: ''\nlatest_evidence: ['https://legacy.invalid/evidence']\nblocker: null\nupdated_at: 2026-07-24T00:00:00Z\n---\n# Legacy 0.4 status\n";
        let document =
            RunStatusDocument::parse_markdown(bytes).expect("legacy 0.4 status diagnosis");

        assert_eq!(
            document.status().required_criteria,
            ["criterion with spaces"]
        );
        assert_eq!(document.status().active_roles, ["Review Team"]);
        assert_eq!(
            document.validate_checkpoint(),
            Err(RunContractError::IncompleteOwnerPin)
        );

        let pinned_without_evidence = b"---\nschema_version: 1\nrun_id: legacy-run\nrevision: 1\nstate: resume-ready\nrequired_criteria: ['criterion with spaces']\npassed_criteria: []\nfailed_criteria: []\nactive_roles: ['Review Team']\nnext_action: ''\nlatest_evidence: ['https://legacy.invalid/evidence']\nblocker: null\nupdated_at: 2026-07-24T00:00:00Z\nhost: codex\nhost_version: legacy\nsurface: cli\nexternal_runtime: null\nresolved_owner: host-native\nresolution_evidence_digest: sha256:0000000000000000000000000000000000000000000000000000000000000000\nsubagent_support: best-effort\n---\n# Legacy pinned status\n";
        let pinned = RunStatusDocument::parse_markdown(pinned_without_evidence)
            .expect("legacy status with additive pin should parse");
        assert_eq!(
            pinned.validate_checkpoint(),
            Err(RunContractError::MissingCriterionEvidence)
        );
    }

    #[test]
    fn rejects_ninety_nine_percent_success_and_missing_evidence() {
        let capability = compatible_capability("fixture");
        let binding = capability.owner_binding().expect("binding");
        let status = RunStatus {
            schema_version: 1,
            run_id: "demo".to_owned(),
            revision: 1,
            state: RunState::Succeeded,
            required_criteria: vec!["build".to_owned(), "tests".to_owned()],
            passed_criteria: vec!["build".to_owned()],
            failed_criteria: Vec::new(),
            active_roles: vec!["reviewer".to_owned()],
            next_action: None,
            latest_evidence: Vec::new(),
            blocker: None,
            updated_at: "2026-07-24T00:00:00Z".to_owned(),
            host: Some(binding.host),
            host_version: Some(binding.host_version),
            surface: Some(binding.surface),
            external_runtime: binding.external_runtime,
            resolved_owner: Some(binding.resolved_owner),
            resolution_evidence_digest: Some(binding.resolution_evidence_digest),
            subagent_support: Some(binding.subagent_support),
            resume_note: None,
            criterion_evidence: BTreeMap::new(),
        };
        assert!(matches!(
            RunStatusDocument::from_status(status, b"# Status\n".to_vec()),
            Err(RunContractError::InvalidSuccessState)
        ));
    }

    #[test]
    fn transitions_are_monotonic_idempotent_and_resume_gated() {
        let capability = compatible_capability("fixture");
        let executing = status_for(&capability, RunState::Executing, 1, &[]);
        assert_eq!(
            validate_transition(&executing, &executing),
            Ok(TransitionOutcome::Idempotent)
        );

        let mut same_state_status = executing.status().clone();
        same_state_status.revision = 2;
        let same_state =
            RunStatusDocument::from_status(same_state_status, b"# Executing\n".to_vec())
                .expect("same-state checkpoint");
        assert_eq!(
            validate_transition(&executing, &same_state),
            Ok(TransitionOutcome::Advance)
        );

        let mut stale_revision_status = executing.status().clone();
        stale_revision_status.next_action = Some("different work".to_owned());
        let stale_revision =
            RunStatusDocument::from_status(stale_revision_status, b"# Executing\n".to_vec())
                .expect("semantically valid stale revision");
        assert!(matches!(
            validate_transition(&executing, &stale_revision),
            Err(RunContractError::InvalidRevision {
                expected: 2,
                actual: 1
            })
        ));

        let mut blocked_status = executing.status().clone();
        blocked_status.revision = 2;
        blocked_status.state = RunState::Blocked;
        blocked_status.blocker = Some("external runtime failed".to_owned());
        blocked_status.next_action = Some("resolve external runtime".to_owned());
        let blocked = RunStatusDocument::from_status(blocked_status, b"# Blocked\n".to_vec())
            .expect("blocked status");
        assert_eq!(
            validate_transition(&executing, &blocked),
            Ok(TransitionOutcome::Advance)
        );

        let mut direct_status = blocked.status().clone();
        direct_status.revision = 3;
        direct_status.state = RunState::Executing;
        direct_status.blocker = None;
        direct_status.next_action = Some("continue".to_owned());
        let direct = RunStatusDocument::from_status(direct_status, b"# Executing\n".to_vec())
            .expect("semantically valid executing status");
        assert!(matches!(
            validate_transition(&blocked, &direct),
            Err(RunContractError::IllegalStateTransition { .. })
        ));

        let mut resume_status = blocked.status().clone();
        resume_status.revision = 3;
        resume_status.state = RunState::ResumeReady;
        resume_status.blocker = None;
        resume_status.resume_note = Some("load PLAN, STATUS, and evidence".to_owned());
        let resume_ready =
            RunStatusDocument::from_status(resume_status, b"# Resume ready\n".to_vec())
                .expect("resume-ready status");
        assert_eq!(
            validate_transition(&blocked, &resume_ready),
            Ok(TransitionOutcome::Advance)
        );

        let mut next_execution = resume_ready.status().clone();
        next_execution.revision = 4;
        next_execution.state = RunState::Executing;
        next_execution.resume_note = None;
        let resumed = RunStatusDocument::from_status(next_execution, b"# Resumed\n".to_vec())
            .expect("resumed executing status");
        assert_eq!(
            validate_transition(&resume_ready, &resumed),
            Ok(TransitionOutcome::Advance)
        );
    }

    #[test]
    fn terminal_and_criterion_regressions_are_rejected() {
        let capability = compatible_capability("fixture");
        let verifying = status_for(&capability, RunState::Verifying, 1, &["build"]);
        let mut regressed_status = verifying.status().clone();
        regressed_status.revision = 2;
        regressed_status.state = RunState::Executing;
        regressed_status.passed_criteria.clear();
        regressed_status.criterion_evidence.clear();
        let regressed = RunStatusDocument::from_status(regressed_status, b"# Regressed\n".to_vec())
            .expect("semantically valid regression candidate");
        assert_eq!(
            validate_transition(&verifying, &regressed),
            Err(RunContractError::CriterionRegression)
        );

        let succeeded = status_for(&capability, RunState::Succeeded, 7, &["build", "tests"]);
        let mut rewritten_status = succeeded.status().clone();
        rewritten_status.revision = 8;
        rewritten_status.updated_at = "2026-07-24T00:01:00Z".to_owned();
        let rewritten = RunStatusDocument::from_status(rewritten_status, b"# Rewritten\n".to_vec())
            .expect("semantically valid terminal rewrite candidate");
        assert_eq!(
            validate_transition(&succeeded, &rewritten),
            Err(RunContractError::TerminalStateImmutable)
        );
    }

    #[test]
    fn owner_disappearance_incompatibility_and_digest_drift_never_switch() {
        let original = compatible_capability("fixture");
        let binding = original.owner_binding().expect("binding");
        assert_eq!(
            super::verify_owner_continuity(&binding, None),
            OwnerContinuity::Blocked(OwnerBlockReason::ExternalRuntimeMissing)
        );
        assert!(matches!(
            super::verify_owner_continuity(&binding, Some(&absent_capability())),
            OwnerContinuity::Blocked(OwnerBlockReason::ExternalRuntimeMissing)
        ));
        assert!(matches!(
            super::verify_owner_continuity(&binding, Some(&incompatible_capability())),
            OwnerContinuity::Unsupported(_)
        ));
        let drift = compatible_capability("fixture-drift");
        assert_eq!(
            super::verify_owner_continuity(&binding, Some(&drift)),
            OwnerContinuity::Blocked(OwnerBlockReason::EvidenceDrift)
        );
    }

    #[test]
    fn dispatch_brief_is_prepare_only_and_deterministic() {
        let capability = compatible_capability("fixture");
        let plan = RunPlan::parse_markdown(b"- [ ] build: Compile\n- [ ] tests: Test\n")
            .expect("valid plan");
        let status = status_for(&capability, RunState::Executing, 1, &[]);
        let brief = prepare_dispatch_brief(&plan, &status, &role(), Some(&capability))
            .expect("brief should prepare");

        assert!(brief.prepared_only);
        assert_eq!(brief.host, Host::Codex);
        assert_eq!(brief.surface, HostSurface::Cli);
        assert_eq!(brief.resolved_owner, ResolvedOwner::Omx);
        assert_eq!(brief.subagent_support, SupportLevel::Supported);
        assert_eq!(brief.responsibilities, ["verify"]);
        assert_eq!(brief.non_responsibilities, ["implement"]);
        assert_eq!(brief.verification_duties, ["attach evidence"]);
        assert_eq!(
            brief.encode_canonical().expect("brief encoding"),
            brief.encode_canonical().expect("brief encoding")
        );
        assert_eq!(
            brief.canonical_digest().expect("brief digest"),
            brief.canonical_digest().expect("brief digest")
        );
        assert_eq!(
            status.encode_canonical().expect("status encoding"),
            status.encode_canonical().expect("status encoding")
        );
        assert_eq!(
            status.canonical_digest().expect("status digest"),
            status.canonical_digest().expect("status digest")
        );

        let drift = compatible_capability("drift");
        assert!(matches!(
            prepare_dispatch_brief(&plan, &status, &role(), Some(&drift)),
            Err(DispatchContractError::Blocked(
                OwnerBlockReason::EvidenceDrift
            ))
        ));

        let mut blocked_status = status.status().clone();
        blocked_status.revision = 2;
        blocked_status.state = RunState::Blocked;
        blocked_status.blocker = Some("manual approval required".to_owned());
        let blocked = RunStatusDocument::from_status(blocked_status, b"# Blocked\n".to_vec())
            .expect("blocked status");
        assert!(matches!(
            prepare_dispatch_brief(&plan, &blocked, &role(), Some(&capability)),
            Err(DispatchContractError::RunNotDispatchable(RunState::Blocked))
        ));

        assert!(matches!(
            prepare_dispatch_brief(
                &plan,
                &status,
                &role_for(Some("other-run"), Some(".hive/runs/demo/HANDOFF.md")),
                Some(&capability)
            ),
            Err(DispatchContractError::AssignmentMismatch { .. })
        ));
        assert!(matches!(
            prepare_dispatch_brief(
                &plan,
                &status,
                &role_for(Some("demo"), Some(".hive/runs/other-run/HANDOFF.md")),
                Some(&capability)
            ),
            Err(DispatchContractError::HandoffRunMismatch { .. })
        ));
        assert!(matches!(
            prepare_dispatch_brief(
                &plan,
                &status,
                &role_for(Some("demo"), None),
                Some(&capability)
            ),
            Err(DispatchContractError::HandoffRunMismatch { .. })
        ));
        for wrong_handoff in [
            ".hive/runs/demo/OTHER.md",
            ".hive/runs/demo/subdir/HANDOFF.md",
        ] {
            assert!(matches!(
                prepare_dispatch_brief(
                    &plan,
                    &status,
                    &role_for(Some("demo"), Some(wrong_handoff)),
                    Some(&capability)
                ),
                Err(DispatchContractError::HandoffRunMismatch { .. })
            ));
        }
    }

    #[test]
    fn typed_checkpoint_enforces_schema_bounds() {
        let capability = compatible_capability("fixture");
        let mut status = status_for(&capability, RunState::Executing, 1, &[])
            .status()
            .clone();
        status.host_version = Some("x".repeat(129));
        assert!(matches!(
            RunStatusDocument::from_status(status, b"# Status\n".to_vec()),
            Err(RunContractError::IncompleteOwnerPin)
        ));

        for hostile in [
            "../outside.json",
            ".hive/runs/demo/STATUS.md",
            ".hive/runs/other-run/evidence/build.json",
        ] {
            let mut unsafe_evidence = status_for(&capability, RunState::Verifying, 1, &["build"])
                .status()
                .clone();
            unsafe_evidence
                .criterion_evidence
                .insert("build".to_owned(), vec![hostile.to_owned()]);
            assert!(
                matches!(
                    RunStatusDocument::from_status(unsafe_evidence, b"# Status\n".to_vec()),
                    Err(RunContractError::UnsafeEvidenceLocator(_))
                ),
                "hostile evidence locator was accepted: {hostile}"
            );
        }

        let mut unsafe_latest = status_for(&capability, RunState::Executing, 1, &[])
            .status()
            .clone();
        unsafe_latest.latest_evidence = vec![".hive/runs/other-run/evidence/prior.json".to_owned()];
        assert!(matches!(
            RunStatusDocument::from_status(unsafe_latest, b"# Status\n".to_vec()),
            Err(RunContractError::UnsafeEvidenceLocator(_))
        ));

        let mut fragment_evidence = status_for(&capability, RunState::Executing, 1, &[])
            .status()
            .clone();
        fragment_evidence.latest_evidence = vec![format!(
            ".hive/runs/demo/evidence/prior.json#sha256:{}",
            "0".repeat(64)
        )];
        RunStatusDocument::from_status(fragment_evidence, b"# Status\n".to_vec())
            .expect("same-run evidence with a fragment should remain valid");
    }
}
