//! Provider-neutral custom-agent profiles, routing, projection, and attestation.

use crate::judge_auth::{ArtifactKind, JudgeAttestation, JudgeTrustRoot};
use crate::{sha256_digest, validate_json_schema};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const PROFILE_SCHEMA: &str = include_str!("../../../schemas/custom-subagent-profile.schema.json");
const ATTESTATION_SCHEMA: &str =
    include_str!("../../../schemas/custom-subagent-attestation.schema.json");
const HOST_CAPABILITY_SCHEMA: &str =
    include_str!("../../../schemas/host-orchestration-capability.schema.json");
const HOST_MODEL_CATALOG_SCHEMA: &str =
    include_str!("../../../schemas/host-model-catalog.schema.json");

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentScope {
    User,
    Project,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentPermission {
    ReadOnly,
    BoundedWrite,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentEffort {
    Low,
    Medium,
    High,
    Max,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostAgentMapping {
    pub model: String,
    pub effort: AgentEffort,
    pub minimum_version: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomAgentProfile {
    pub schema_version: u32,
    pub role_id: String,
    pub display_name: String,
    pub description: String,
    pub scope: AgentScope,
    pub reserved: bool,
    pub permission: AgentPermission,
    pub positive_triggers: Vec<String>,
    pub negative_triggers: Vec<String>,
    pub host_mappings: BTreeMap<String, HostAgentMapping>,
    pub definition_digest: String,
}

#[derive(Serialize)]
struct ProfilePayload<'a> {
    schema_version: u32,
    role_id: &'a str,
    display_name: &'a str,
    description: &'a str,
    scope: AgentScope,
    reserved: bool,
    permission: AgentPermission,
    positive_triggers: &'a [String],
    negative_triggers: &'a [String],
    host_mappings: &'a BTreeMap<String, HostAgentMapping>,
}

impl CustomAgentProfile {
    /// Parse and validate one closed JSON profile.
    ///
    /// # Errors
    ///
    /// Rejects malformed, schema-invalid, unsafe, incomplete, or digest-mismatched profiles.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, CustomAgentError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        validate_json_schema(PROFILE_SCHEMA, &value, "custom subagent profile")
            .map_err(CustomAgentError::Schema)?;
        let profile: Self = serde_json::from_value(value)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        profile.validate()?;
        Ok(profile)
    }

    /// Compute the canonical profile digest excluding its digest field.
    ///
    /// # Errors
    ///
    /// Returns an error when canonical serialization fails.
    pub fn computed_digest(&self) -> Result<String, CustomAgentError> {
        let payload = ProfilePayload {
            schema_version: self.schema_version,
            role_id: &self.role_id,
            display_name: &self.display_name,
            description: &self.description,
            scope: self.scope,
            reserved: self.reserved,
            permission: self.permission,
            positive_triggers: &self.positive_triggers,
            negative_triggers: &self.negative_triggers,
            host_mappings: &self.host_mappings,
        };
        serde_json_canonicalizer::to_vec(&payload)
            .map(|bytes| sha256_digest(&bytes))
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))
    }

    /// Validate reserved-role, trigger, mapping, and digest invariants.
    ///
    /// # Errors
    ///
    /// Rejects missing mappings, floating aliases, reserved Judge shadowing, and collisions.
    pub fn validate(&self) -> Result<(), CustomAgentError> {
        if self.schema_version != 1
            || self.host_mappings.keys().cloned().collect::<BTreeSet<_>>()
                != ["claude".to_owned(), "codex".to_owned()]
                    .into_iter()
                    .collect()
            || self.computed_digest()? != self.definition_digest
            || has_trigger_collision(&self.positive_triggers, &self.negative_triggers)
            || self
                .host_mappings
                .values()
                .any(|mapping| floating_alias(&mapping.model))
        {
            return Err(CustomAgentError::InvalidProfile);
        }
        let judge = self.role_id == "hive-independent-judge";
        if judge
            && (!self.reserved
                || self.scope != AgentScope::User
                || self.permission != AgentPermission::ReadOnly)
        {
            return Err(CustomAgentError::ReservedRoleViolation);
        }
        if self.reserved && !judge {
            return Err(CustomAgentError::ReservedRoleViolation);
        }
        Ok(())
    }

    /// Render one deterministic host-native definition.
    ///
    /// # Errors
    ///
    /// Rejects unsupported hosts and invalid profiles.
    pub fn render(&self, host: &str) -> Result<Vec<u8>, CustomAgentError> {
        self.validate()?;
        let mapping = self
            .host_mappings
            .get(host)
            .ok_or(CustomAgentError::UnsupportedHost)?;
        let permission = match self.permission {
            AgentPermission::ReadOnly => "read-only",
            AgentPermission::BoundedWrite => "workspace-write",
        };
        match host {
            "codex" => Ok(format!(
                "name = \"{}\"\ndescription = \"{}\"\ndeveloper_instructions = \"{}\"\nmodel = \"{}\"\nmodel_reasoning_effort = \"{}\"\nsandbox_mode = \"{}\"\n# hive_definition_digest = \"{}\"\n",
                self.role_id,
                escaped(&self.description),
                escaped(&self.description),
                mapping.model,
                effort_name(mapping.effort),
                permission,
                self.definition_digest
            )
            .into_bytes()),
            "claude" => Ok(format!(
                "---\nname: {}\ndescription: {}\nmodel: {}\neffort: {}\npermissionMode: {}\nhive_definition_digest: {}\n---\n\n{}\n",
                self.role_id,
                self.description,
                mapping.model,
                effort_name(mapping.effort),
                permission,
                self.definition_digest,
                self.description
            )
            .into_bytes()),
            _ => Err(CustomAgentError::UnsupportedHost),
        }
    }
}

/// Resolve project precedence without allowing a reserved Judge shadow.
///
/// # Errors
///
/// Rejects same-scope collisions and any project Judge definition.
pub fn resolve_profiles(
    user: &[CustomAgentProfile],
    project: &[CustomAgentProfile],
) -> Result<BTreeMap<String, CustomAgentProfile>, CustomAgentError> {
    let mut resolved = BTreeMap::new();
    for profile in user {
        profile.validate()?;
        if profile.scope != AgentScope::User
            || resolved
                .insert(profile.role_id.clone(), profile.clone())
                .is_some()
        {
            return Err(CustomAgentError::ScopeCollision);
        }
    }
    for profile in project {
        profile.validate()?;
        if profile.scope != AgentScope::Project
            || profile.role_id == "hive-independent-judge"
            || resolved
                .get(&profile.role_id)
                .is_some_and(|existing| existing.reserved)
        {
            return Err(CustomAgentError::ReservedRoleViolation);
        }
        resolved.insert(profile.role_id.clone(), profile.clone());
    }
    Ok(resolved)
}

/// Select exactly one positive profile while honoring every negative trigger.
#[must_use]
pub fn route_profile<'a>(
    profiles: &'a BTreeMap<String, CustomAgentProfile>,
    request: &str,
) -> Option<&'a CustomAgentProfile> {
    let request = request.to_ascii_lowercase();
    let mut matches = profiles.values().filter(|profile| {
        !profile
            .negative_triggers
            .iter()
            .any(|trigger| request.contains(&trigger.to_ascii_lowercase()))
            && profile
                .positive_triggers
                .iter()
                .any(|trigger| request.contains(&trigger.to_ascii_lowercase()))
    });
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeAttestation {
    pub schema_version: u32,
    pub attestation_id: String,
    pub run_id: String,
    pub action_id: String,
    pub host: String,
    pub role_id: String,
    pub scope: AgentScope,
    pub model: String,
    pub effort: AgentEffort,
    pub definition_digest: String,
    pub host_capability_digest: String,
    pub native_task_id: String,
    pub issued_at: String,
}

/// The closed, fresh host capability evidence needed before a profile can activate.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostOrchestrationCapability {
    pub schema_version: u32,
    pub captured_at: String,
    pub host: String,
    pub host_version: String,
    pub activation: String,
    pub sources: Vec<String>,
    pub capabilities: HostOrchestrationCapabilities,
    pub limitations: Vec<String>,
}

/// Externally signed model availability catalog for one or more supported hosts.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostModelCatalog {
    pub schema_version: u32,
    pub catalog_id: String,
    pub principal_id: String,
    pub issued_at: String,
    pub models: Vec<HostModelCatalogEntry>,
}

/// One exact host model and effort availability declaration.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostModelCatalogEntry {
    pub host: String,
    pub model: String,
    pub efforts: Vec<AgentEffort>,
    pub minimum_version: String,
}

/// Capability values are deliberately closed: partial and unverified evidence cannot activate.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostOrchestrationCapabilities {
    pub agent_discovery: HostCapabilityStatus,
    pub user_scope: HostCapabilityStatus,
    pub project_scope: HostCapabilityStatus,
    pub model_pin: HostCapabilityStatus,
    pub effort_pin: HostCapabilityStatus,
    pub native_dispatch: HostCapabilityStatus,
    pub launch_ack: HostCapabilityStatus,
    pub result_return: HostCapabilityStatus,
    pub cancel: HostCapabilityStatus,
    pub lookup: HostCapabilityStatus,
    pub idempotency: HostCapabilityStatus,
    pub runtime_attestation: HostCapabilityStatus,
    pub fresh_session: HostCapabilityStatus,
}

/// A host capability observation from its current session evidence.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HostCapabilityStatus {
    Supported,
    Partial,
    Unsupported,
    Unverified,
}

impl HostOrchestrationCapability {
    /// Parse a schema-validated host snapshot and reject all non-activation modes.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed evidence, schema violations, or a non-default-off mode.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, CustomAgentError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        validate_json_schema(
            HOST_CAPABILITY_SCHEMA,
            &value,
            "host orchestration capability",
        )
        .map_err(CustomAgentError::Schema)?;
        let capability: Self = serde_json::from_value(value)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        if capability.activation != "default-off" {
            return Err(CustomAgentError::CapabilityUnsupported);
        }
        Ok(capability)
    }

    /// Require exact host identity, the profile's minimum version, and all activation evidence.
    ///
    /// # Errors
    ///
    /// Returns an error for a mismatched host, insufficient version, or any non-supported gate.
    pub fn verify_profile_activation(
        &self,
        profile: &CustomAgentProfile,
        expected_host: &str,
    ) -> Result<(), CustomAgentError> {
        let mapping = profile
            .host_mappings
            .get(expected_host)
            .ok_or(CustomAgentError::UnsupportedHost)?;
        if self.host != expected_host
            || !version_at_least(&self.host_version, &mapping.minimum_version)
            || ![
                self.capabilities.agent_discovery,
                self.capabilities.user_scope,
                self.capabilities.project_scope,
                self.capabilities.model_pin,
                self.capabilities.effort_pin,
                self.capabilities.runtime_attestation,
                self.capabilities.fresh_session,
            ]
            .iter()
            .all(|status| *status == HostCapabilityStatus::Supported)
        {
            return Err(CustomAgentError::CapabilityUnsupported);
        }
        Ok(())
    }
}

impl HostModelCatalog {
    /// Parse the closed host-model catalog without trusting it.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed or schema-invalid catalog bytes.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, CustomAgentError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        validate_json_schema(HOST_MODEL_CATALOG_SCHEMA, &value, "host model catalog")
            .map_err(CustomAgentError::Schema)?;
        serde_json::from_value(value)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))
    }

    /// Verify a detached external catalog attestation and every exact profile mapping.
    ///
    /// # Errors
    ///
    /// Returns an error when the catalog signature, signer, lifecycle, host model, effort, or
    /// minimum-version binding does not match.
    pub fn verify_profile(
        &self,
        attestation: &JudgeAttestation,
        trust_root: &JudgeTrustRoot,
        profile: &CustomAgentProfile,
    ) -> Result<(), CustomAgentError> {
        attestation
            .verify(
                trust_root,
                ArtifactKind::HostModelCatalog,
                self,
                &self.principal_id,
                &self.issued_at,
            )
            .map_err(|_| CustomAgentError::CapabilityUnsupported)?;
        if profile.host_mappings.iter().all(|(host, mapping)| {
            self.models.iter().any(|entry| {
                entry.host == *host
                    && entry.model == mapping.model
                    && entry.efforts.contains(&mapping.effort)
                    && entry.minimum_version == mapping.minimum_version
            })
        }) {
            Ok(())
        } else {
            Err(CustomAgentError::CapabilityUnsupported)
        }
    }
}

impl RuntimeAttestation {
    /// Parse and validate one runtime attestation against an exact profile.
    ///
    /// # Errors
    ///
    /// Rejects malformed, unsupported, missing, or silently-fallbacked runtime metadata.
    pub fn parse_and_verify(
        bytes: &[u8],
        profile: &CustomAgentProfile,
        expected_host: &str,
    ) -> Result<Self, CustomAgentError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        validate_json_schema(ATTESTATION_SCHEMA, &value, "custom subagent attestation")
            .map_err(CustomAgentError::Schema)?;
        let attestation: Self = serde_json::from_value(value)
            .map_err(|error| CustomAgentError::Malformed(error.to_string()))?;
        let mapping = profile
            .host_mappings
            .get(expected_host)
            .ok_or(CustomAgentError::UnsupportedHost)?;
        if attestation.host != expected_host
            || attestation.role_id != profile.role_id
            || attestation.scope != profile.scope
            || attestation.model != mapping.model
            || attestation.effort != mapping.effort
            || attestation.definition_digest != profile.definition_digest
        {
            return Err(CustomAgentError::AttestationMismatch);
        }
        Ok(attestation)
    }
}

fn has_trigger_collision(positive: &[String], negative: &[String]) -> bool {
    let negative = negative
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    positive
        .iter()
        .any(|value| negative.contains(&value.to_ascii_lowercase()))
}

fn floating_alias(model: &str) -> bool {
    model.ends_with("-latest") || model == "sonnet" || model == "opus" || model == "default"
}

fn version_at_least(observed: &str, minimum: &str) -> bool {
    fn parse(value: &str) -> Option<[u64; 3]> {
        let mut words = value.split_ascii_whitespace();
        let version = words.next_back()?;
        if words.next_back().is_some_and(|word| {
            word.chars()
                .all(|character| character.is_ascii_digit() || character == '.')
        }) {
            return None;
        }
        let mut parts = version.split('.');
        let parsed = [
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        ];
        parts.next().is_none().then_some(parsed)
    }
    matches!((parse(observed), parse(minimum)), (Some(observed), Some(minimum)) if observed >= minimum)
}

fn effort_name(effort: AgentEffort) -> &'static str {
    match effort {
        AgentEffort::Low => "low",
        AgentEffort::Medium => "medium",
        AgentEffort::High => "high",
        AgentEffort::Max => "max",
    }
}

fn escaped(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CustomAgentError {
    Malformed(String),
    Schema(String),
    InvalidProfile,
    ReservedRoleViolation,
    ScopeCollision,
    UnsupportedHost,
    AttestationMismatch,
    CapabilityUnsupported,
}

impl Display for CustomAgentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(error) => write!(formatter, "malformed custom agent data: {error}"),
            Self::Schema(error) => write!(formatter, "custom agent schema violation: {error}"),
            Self::InvalidProfile => formatter.write_str("invalid custom agent profile"),
            Self::ReservedRoleViolation => formatter.write_str("reserved Judge role violation"),
            Self::ScopeCollision => formatter.write_str("custom agent scope collision"),
            Self::UnsupportedHost => formatter.write_str("custom agent host is unsupported"),
            Self::AttestationMismatch => {
                formatter.write_str("custom agent runtime attestation mismatch")
            }
            Self::CapabilityUnsupported => {
                formatter.write_str("host capability evidence cannot activate this custom agent")
            }
        }
    }
}

impl Error for CustomAgentError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(role: &str, scope: AgentScope, reserved: bool) -> CustomAgentProfile {
        let mut profile = CustomAgentProfile {
            schema_version: 1,
            role_id: role.to_owned(),
            display_name: role.to_owned(),
            description: "Perform one exact bounded responsibility.".to_owned(),
            scope,
            reserved,
            permission: if reserved {
                AgentPermission::ReadOnly
            } else {
                AgentPermission::BoundedWrite
            },
            positive_triggers: vec![format!("use {role}")],
            negative_triggers: vec!["simple question".to_owned()],
            host_mappings: BTreeMap::from([
                (
                    "codex".to_owned(),
                    HostAgentMapping {
                        model: "gpt-5.6-terra".to_owned(),
                        effort: AgentEffort::Max,
                        minimum_version: "0.147.0".to_owned(),
                    },
                ),
                (
                    "claude".to_owned(),
                    HostAgentMapping {
                        model: "claude-opus-4-8".to_owned(),
                        effort: AgentEffort::Max,
                        minimum_version: "2.1.163".to_owned(),
                    },
                ),
            ]),
            definition_digest: String::new(),
        };
        profile.definition_digest = profile.computed_digest().expect("digest");
        profile
    }

    #[test]
    fn renders_both_hosts_and_rejects_antigravity() {
        let profile = profile("hive-complex-implementer", AgentScope::Project, false);
        assert!(String::from_utf8(profile.render("codex").expect("codex"))
            .expect("utf8")
            .contains("gpt-5.6-terra"));
        assert!(String::from_utf8(profile.render("claude").expect("claude"))
            .expect("utf8")
            .contains("claude-opus-4-8"));
        assert_eq!(
            profile.render("antigravity"),
            Err(CustomAgentError::UnsupportedHost)
        );
    }

    #[test]
    fn project_precedence_never_shadows_reserved_judge() {
        let judge = profile("hive-independent-judge", AgentScope::User, true);
        let mut shadow = judge.clone();
        shadow.scope = AgentScope::Project;
        shadow.definition_digest = shadow.computed_digest().expect("digest");
        assert_eq!(
            resolve_profiles(&[judge], &[shadow]),
            Err(CustomAgentError::ReservedRoleViolation)
        );
    }

    #[test]
    fn route_requires_one_positive_and_no_negative_match() {
        let profile = profile("hive-routine-implementer", AgentScope::Project, false);
        let profiles = BTreeMap::from([(profile.role_id.clone(), profile)]);
        assert!(route_profile(&profiles, "please use hive-routine-implementer").is_some());
        assert!(
            route_profile(&profiles, "simple question: use hive-routine-implementer").is_none()
        );
    }

    #[test]
    fn attestation_rejects_silent_model_fallback() {
        let profile = profile("hive-complex-implementer", AgentScope::Project, false);
        let value = serde_json::json!({
            "schema_version": 1,
            "attestation_id": "attestation-1",
            "run_id": "run-1",
            "action_id": "action-1",
            "host": "codex",
            "role_id": profile.role_id,
            "scope": "project",
            "model": "gpt-5.6-luna",
            "effort": "max",
            "definition_digest": profile.definition_digest,
            "host_capability_digest": format!("sha256:{}", "a".repeat(64)),
            "native_task_id": "native-1",
            "issued_at": "2026-08-12T00:00:00Z"
        });
        assert_eq!(
            RuntimeAttestation::parse_and_verify(
                &serde_json::to_vec(&value).expect("json"),
                &profile,
                "codex"
            ),
            Err(CustomAgentError::AttestationMismatch)
        );
    }

    #[test]
    fn capability_preflight_requires_exact_version_and_supported_evidence() {
        let profile = profile("hive-complex-implementer", AgentScope::Project, false);
        let value = serde_json::json!({
            "schema_version": 1,
            "captured_at": "2026-08-13T00:00:00Z",
            "host": "codex",
            "host_version": "codex-cli 0.147.0",
            "activation": "default-off",
            "sources": ["test:capability"],
            "capabilities": {
                "agent_discovery": "supported", "user_scope": "supported",
                "project_scope": "supported", "model_pin": "supported",
                "effort_pin": "supported", "native_dispatch": "supported",
                "launch_ack": "supported", "result_return": "supported",
                "cancel": "supported", "lookup": "supported", "idempotency": "supported",
                "runtime_attestation": "supported", "fresh_session": "supported"
            },
            "limitations": []
        });
        let capability =
            HostOrchestrationCapability::parse_json(&serde_json::to_vec(&value).expect("json"))
                .expect("capability");
        assert!(capability
            .verify_profile_activation(&profile, "codex")
            .is_ok());

        let mut stale = value;
        stale["capabilities"]["fresh_session"] = serde_json::json!("unverified");
        let stale =
            HostOrchestrationCapability::parse_json(&serde_json::to_vec(&stale).expect("json"))
                .expect("capability");
        assert_eq!(
            stale.verify_profile_activation(&profile, "codex"),
            Err(CustomAgentError::CapabilityUnsupported)
        );
    }

    #[test]
    fn every_builtin_profile_validates_and_renders_both_hosts() {
        let fixtures = [
            include_bytes!("../../../harness/roles/hive-routine-implementer.json").as_slice(),
            include_bytes!("../../../harness/roles/hive-complex-implementer.json").as_slice(),
            include_bytes!("../../../harness/roles/hive-independent-judge.json").as_slice(),
            include_bytes!("../../../harness/roles/hive-design-specialist.json").as_slice(),
            include_bytes!("../../../harness/roles/hive-article-writer.json").as_slice(),
            include_bytes!("../../../harness/roles/hive-research-specialist.json").as_slice(),
        ];
        for bytes in fixtures {
            let profile = CustomAgentProfile::parse_json(bytes).expect("builtin profile");
            assert!(profile.render("codex").expect("codex").len() > 100);
            assert!(profile.render("claude").expect("claude").len() > 100);
        }
    }
}
