//! Stored user choices and versioned usage policies. No sensing or filesystem effects.

use crate::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Response to unavailable usage before starting work.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreflightFailure {
    /// Explain the missing observation without blocking work.
    #[default]
    Warn,
    /// Block work under the user's strict policy.
    Block,
}

/// Response to unavailable usage during monitored work.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MidwayFailure {
    /// Explain the missing observation without interrupting work.
    #[default]
    Warn,
    /// Interrupt under the user's strict monitoring policy.
    Stop,
}

/// Independently selected checks before dispatch.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightPreferences {
    /// Apply the selected remaining-quota floor.
    pub threshold_enabled: bool,
    /// Stop one request when a quota reset is observed.
    pub reset_enabled: bool,
    /// Block on exhausted subscription quota.
    pub zero_quota_enabled: bool,
    /// Policy for a missing or invalid observation.
    pub observation_failure: PreflightFailure,
}

/// Independently selected live-monitor behavior.
// These independent user selections are the persisted public configuration contract.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MidwayPreferences {
    /// The separately consented background monitor.
    pub enabled: bool,
    /// Interrupt at the selected floor.
    pub threshold_enabled: bool,
    /// Interrupt on a confirmed reset.
    pub reset_enabled: bool,
    /// Interrupt on exhausted subscription quota.
    pub zero_quota_enabled: bool,
    /// Policy for an observation gap or monitor health failure.
    pub observation_failure: MidwayFailure,
}

/// Version-2 protection controls, separate from thresholds and integrations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageProtection {
    /// Master opt-in. False dominates every stored sub-selection.
    pub enforcement_enabled: bool,
    /// Non-blocking notices based only on observations obtained elsewhere.
    pub warnings_enabled: bool,
    /// Checks before dispatch.
    pub preflight: PreflightPreferences,
    /// The separately enabled live monitor.
    pub midway: MidwayPreferences,
}

impl Default for UsageProtection {
    fn default() -> Self {
        Self {
            enforcement_enabled: false,
            warnings_enabled: true,
            preflight: PreflightPreferences::default(),
            midway: MidwayPreferences::default(),
        }
    }
}

impl UsageProtection {
    /// Preserve v1 behavior without guessing whether its stored flags were explicit choices.
    ///
    /// `monitor_enabled` must come from a validated monitor configuration. A damaged or
    /// unavailable configuration cannot be represented as `false` by the caller.
    #[must_use]
    pub fn from_legacy(threshold: bool, reset: bool, monitor_enabled: bool) -> Self {
        Self {
            enforcement_enabled: true,
            warnings_enabled: false,
            preflight: PreflightPreferences {
                threshold_enabled: threshold,
                reset_enabled: threshold && reset,
                zero_quota_enabled: true,
                observation_failure: PreflightFailure::Block,
            },
            midway: MidwayPreferences {
                enabled: monitor_enabled,
                threshold_enabled: monitor_enabled && threshold,
                reset_enabled: monitor_enabled,
                zero_quota_enabled: monitor_enabled,
                observation_failure: if monitor_enabled {
                    MidwayFailure::Stop
                } else {
                    MidwayFailure::Warn
                },
            },
        }
    }

    /// Read the effective dispatch policy without changing stored choices.
    #[must_use]
    pub fn effective_preflight(&self) -> PreflightPreferences {
        if self.enforcement_enabled {
            self.preflight.clone()
        } else {
            PreflightPreferences::default()
        }
    }

    /// Read the effective monitor policy without activating its stored selection.
    #[must_use]
    pub fn effective_midway(&self) -> MidwayPreferences {
        if self.enforcement_enabled && self.midway.enabled {
            self.midway.clone()
        } else {
            MidwayPreferences::default()
        }
    }

    /// Whether a dispatch check needs an observation. Notices alone never trigger sensing.
    #[must_use]
    pub fn observation_required(&self) -> bool {
        let policy = self.effective_preflight();
        policy.threshold_enabled || policy.reset_enabled || policy.zero_quota_enabled
    }
}

/// Meaning of a stored choice; never an execution authorization.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionOrigin {
    /// An unconfirmed product default.
    Default,
    /// A value selected by the user.
    Explicit,
    /// The user accepted a displayed default or bundle.
    AcceptedDefault,
    /// A reference to a parent-scope preference.
    Inherited,
    /// A predecessor value without evidence of its original choice.
    LegacyUnknown,
    /// The stored declaration changed outside the recorded setup operation.
    ExternalChange,
}

/// Input transport, independent of the meaning of the choice.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreferenceInput {
    /// Interactive terminal.
    Terminal,
    /// Supplied answer file.
    AnswersFile,
    /// Host-owned assistant input.
    Agent,
    /// Deterministic format or policy migration.
    Migration,
}

/// Scope owning the declaration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreferenceScope {
    /// Global user preferences.
    User,
    /// An explicitly selected project.
    Project,
}

/// Parent declaration, retained separately from the value it currently resolves to.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceReference {
    /// Parent scope; currently only user-to-project inheritance is supported.
    pub scope: PreferenceScope,
    /// Pointer in the parent document, which may differ from the child's pointer.
    pub key: String,
}

/// Provenance stored with a setting. No prompt, session, or credential is retained.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceDecision {
    /// Meaning of this stored choice.
    pub origin: DecisionOrigin,
    /// Transport used to supply it.
    pub input_channel: PreferenceInput,
    /// Scope owning the choice.
    pub scope: PreferenceScope,
    /// Displayed question, if any.
    pub question_id: Option<String>,
    /// Semantic revision of that question.
    pub question_revision: Option<u32>,
    /// Version of the defaults presented to the user.
    pub policy_version: String,
    /// Product version recording the change.
    pub changed_in_version: String,
    /// Unix seconds of an actual answer; absent for unknown historical choices.
    pub decided_at: Option<u64>,
    /// Digest of the stored declaration, not its resolved inherited value.
    pub value_digest: String,
    /// The declaration hashed for an inherited choice, never its resolved value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherited_from: Option<PreferenceReference>,
}

/// JSON Pointer to its decision record. Canonical settings and these records share one write.
pub type PreferenceDecisions = BTreeMap<String, PreferenceDecision>;

impl PreferenceDecision {
    /// Record a declaration without inventing a historical answer time.
    ///
    /// # Errors
    /// Returns an error when the declaration cannot be encoded as canonical JSON.
    pub fn new(
        declaration: &Value,
        origin: DecisionOrigin,
        input_channel: PreferenceInput,
        scope: PreferenceScope,
        version: &str,
        decided_at: Option<u64>,
    ) -> Result<Self, String> {
        let inherited_from = if origin == DecisionOrigin::Inherited {
            let reference: PreferenceReference =
                serde_json::from_value(declaration.get("inherit").cloned().unwrap_or(Value::Null))
                    .map_err(|error| format!("invalid inheritance reference: {error}"))?;
            validate_preference_pointer(&reference.key)?;
            if scope != PreferenceScope::Project || reference.scope != PreferenceScope::User {
                return Err("invalid inheritance scope".to_owned());
            }
            Some(reference)
        } else {
            None
        };
        Ok(Self {
            origin,
            input_channel,
            scope,
            question_id: None,
            question_revision: None,
            policy_version: version.to_owned(),
            changed_in_version: version.to_owned(),
            decided_at: if matches!(
                origin,
                DecisionOrigin::Default | DecisionOrigin::LegacyUnknown
            ) {
                None
            } else {
                decided_at
            },
            value_digest: declaration_digest(declaration)?,
            inherited_from,
        })
    }

    /// Compare only the declaration. A parent's new value is not a local edit.
    ///
    /// # Errors
    /// Returns an error for a declaration that cannot be canonically encoded.
    pub fn matches_declaration(&self, declaration: &Value) -> Result<bool, String> {
        Ok(self.value_digest == declaration_digest(declaration)?)
    }
}

/// Hash the stored value or inheritance reference without storing a second copy.
///
/// # Errors
/// Returns the canonical JSON encoding error.
pub fn declaration_digest(declaration: &Value) -> Result<String, String> {
    serde_json_canonicalizer::to_vec(declaration)
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| format!("cannot fingerprint preference declaration: {error}"))
}

/// Build the stored reference used by a project preference, independent of the resolved value.
///
/// # Errors
/// Rejects malformed or reserved setting pointers.
pub fn inheritance_declaration(key: &str) -> Result<Value, String> {
    validate_preference_pointer(key)?;
    Ok(serde_json::json!({"inherit":{"scope":"user","key":key}}))
}

fn validate_preference_pointer(key: &str) -> Result<(), String> {
    if !key.starts_with('/')
        || key.len() < 2
        || key.len() > 512
        || key.chars().any(char::is_control)
        || key
            .split('/')
            .nth(1)
            .is_some_and(|part| matches!(part, "schema_version" | "preference_decisions"))
    {
        return Err("invalid or reserved preference pointer".to_owned());
    }
    let mut chars = key.chars();
    while let Some(character) = chars.next() {
        if character == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return Err("invalid preference pointer escape".to_owned());
        }
    }
    Ok(())
}

/// Derive truthful provenance after a local edit. This does not write or authorize anything.
///
/// Missing optional declarations are compared as null. Inheritance fingerprints contain
/// the reference, so changing the parent's effective value cannot look like a local edit.
///
/// # Errors
/// Rejects a wrong scope, malformed reference, or invalid fingerprint.
pub fn reconcile_decisions(
    document: &Value,
    decisions: &mut PreferenceDecisions,
    scope: PreferenceScope,
) -> Result<(), String> {
    for (key, record) in decisions {
        validate_preference_pointer(key)?;
        if record.scope != scope {
            return Err("preference decision belongs to another scope".to_owned());
        }
        if record.question_id.is_some() != record.question_revision.is_some()
            || record.question_revision == Some(0)
            || record.question_id.as_ref().is_some_and(|id| {
                id.is_empty() || id.len() > 128 || id.chars().any(char::is_control)
            })
            || (matches!(
                record.origin,
                DecisionOrigin::Default | DecisionOrigin::LegacyUnknown
            ) && record.decided_at.is_some())
        {
            return Err("preference question or answer time is inconsistent".to_owned());
        }
        if !record
            .value_digest
            .strip_prefix("sha256:")
            .is_some_and(|hex| {
                hex.len() == 64
                    && hex
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
        {
            return Err("invalid preference declaration fingerprint".to_owned());
        }
        if record.origin == DecisionOrigin::Inherited {
            let reference = record
                .inherited_from
                .as_ref()
                .ok_or("missing inheritance reference")?;
            if scope != PreferenceScope::Project
                || reference.scope != PreferenceScope::User
                || !record.matches_declaration(&inheritance_declaration(&reference.key)?)?
            {
                return Err("invalid inherited preference declaration".to_owned());
            }
        } else {
            if record.inherited_from.is_some() {
                return Err("non-inherited preference contains an inheritance reference".to_owned());
            }
            let declaration = document.pointer(key).unwrap_or(&Value::Null);
            if !record.matches_declaration(declaration)? {
                record.origin = DecisionOrigin::ExternalChange;
                record.decided_at = None;
                record.question_id = None;
                record.question_revision = None;
                "unknown".clone_into(&mut record.changed_in_version);
                record.value_digest = declaration_digest(declaration)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fresh_policy_never_senses_or_interrupts_for_notices() {
        let policy = UsageProtection::default();
        assert!(policy.warnings_enabled);
        assert!(!policy.enforcement_enabled);
        assert!(!policy.observation_required());
        assert_eq!(
            policy.effective_preflight(),
            PreflightPreferences::default()
        );
        assert_eq!(policy.effective_midway(), MidwayPreferences::default());
    }

    #[test]
    fn ordinary_legacy_opt_out_does_not_erase_independent_zero_protection() {
        let policy = UsageProtection::from_legacy(false, true, false);
        assert_eq!(
            policy.preflight,
            PreflightPreferences {
                threshold_enabled: false,
                reset_enabled: false,
                zero_quota_enabled: true,
                observation_failure: PreflightFailure::Block,
            }
        );
        assert!(policy.observation_required());
        assert!(!policy.warnings_enabled);
    }

    #[test]
    fn legacy_monitor_reset_is_independent_of_preflight_reset() {
        for (threshold, reset, expected_reset) in [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ] {
            for monitor in [false, true] {
                let policy = UsageProtection::from_legacy(threshold, reset, monitor);
                assert_eq!(policy.preflight.reset_enabled, expected_reset);
                assert_eq!(policy.midway.reset_enabled, monitor);
                assert_eq!(policy.midway.zero_quota_enabled, monitor);
                assert_eq!(policy.midway.threshold_enabled, threshold && monitor);
            }
        }
    }

    #[test]
    fn master_opt_out_dominates_every_legacy_selection_without_losing_it() {
        let mut policy = UsageProtection::from_legacy(true, true, true);
        let saved = policy.clone();
        policy.enforcement_enabled = false;
        assert!(!policy.observation_required());
        assert_eq!(policy.effective_midway(), MidwayPreferences::default());
        assert_eq!(
            policy.effective_preflight(),
            PreflightPreferences::default()
        );
        assert_eq!(policy.midway, saved.midway);
        assert_eq!(policy.preflight, saved.preflight);
    }

    #[test]
    fn each_preflight_reason_can_require_sensing_without_a_floor() {
        let mut policy = UsageProtection {
            enforcement_enabled: true,
            ..UsageProtection::default()
        };
        policy.preflight.reset_enabled = true;
        assert!(policy.observation_required());
        policy.preflight.reset_enabled = false;
        policy.preflight.zero_quota_enabled = true;
        assert!(policy.observation_required());
    }

    #[test]
    fn strict_policies_cannot_be_confused_between_layers() {
        let mut value = serde_json::to_value(UsageProtection::default()).unwrap();
        value["preflight"]["observation_failure"] = json!("stop");
        assert!(serde_json::from_value::<UsageProtection>(value).is_err());
        let mut value = serde_json::to_value(UsageProtection::default()).unwrap();
        value["midway"]["observation_failure"] = json!("block");
        assert!(serde_json::from_value::<UsageProtection>(value).is_err());
    }

    #[test]
    fn legacy_migration_never_invents_a_user_answer() {
        let decision = PreferenceDecision::new(
            &json!(true),
            DecisionOrigin::LegacyUnknown,
            PreferenceInput::Migration,
            PreferenceScope::User,
            "0.14.0",
            Some(100),
        )
        .unwrap();
        assert_eq!(decision.decided_at, None);
        assert_eq!(decision.question_id, None);
        assert_eq!(decision.origin, DecisionOrigin::LegacyUnknown);
    }

    #[test]
    fn inheritance_uses_the_reference_and_detects_only_declaration_edits() {
        let declaration = json!({"inherit": {"scope":"user", "key":"/wiki/enabled"}});
        let decision = PreferenceDecision::new(
            &declaration,
            DecisionOrigin::Inherited,
            PreferenceInput::Terminal,
            PreferenceScope::Project,
            "0.14.0",
            Some(100),
        )
        .unwrap();
        for effective_value in [true, false] {
            assert!(!decision
                .matches_declaration(&json!(effective_value))
                .unwrap());
            assert!(decision.matches_declaration(&declaration).unwrap());
        }
        assert!(!decision.matches_declaration(&json!(false)).unwrap());
        assert!(!decision
            .matches_declaration(&json!({"inherit": {"scope":"user", "key":"/usage_guard"}}))
            .unwrap());
    }

    #[test]
    fn canonical_key_order_is_not_an_external_edit() {
        assert_eq!(
            declaration_digest(&serde_json::from_str::<Value>(r#"{"b":2,"a":1}"#).unwrap())
                .unwrap(),
            declaration_digest(&serde_json::from_str::<Value>(r#"{"a":1,"b":2}"#).unwrap())
                .unwrap(),
        );
    }

    #[test]
    fn external_value_edits_do_not_keep_an_explicit_answer_claim() {
        let decision = PreferenceDecision::new(
            &json!(true),
            DecisionOrigin::Explicit,
            PreferenceInput::Terminal,
            PreferenceScope::User,
            "0.14.0",
            Some(100),
        )
        .unwrap();
        let mut records = BTreeMap::from([("/wiki/enabled".to_owned(), decision)]);
        let document = json!({"wiki":{"enabled":false}});
        reconcile_decisions(&document, &mut records, PreferenceScope::User).unwrap();
        let record = &records["/wiki/enabled"];
        assert_eq!(record.origin, DecisionOrigin::ExternalChange);
        assert_eq!(record.decided_at, None);
        assert_eq!(record.changed_in_version, "unknown");
        assert!(record.matches_declaration(&json!(false)).unwrap());
    }

    #[test]
    fn parent_changes_preserve_inheritance_but_wrong_scope_is_rejected() {
        let key = "/wiki/enabled";
        let decision = PreferenceDecision::new(
            &inheritance_declaration(key).unwrap(),
            DecisionOrigin::Inherited,
            PreferenceInput::Terminal,
            PreferenceScope::Project,
            "0.14.0",
            Some(100),
        )
        .unwrap();
        let mut records = BTreeMap::from([(key.to_owned(), decision.clone())]);
        for value in [true, false] {
            reconcile_decisions(
                &json!({"wiki":{"enabled":value}}),
                &mut records,
                PreferenceScope::Project,
            )
            .unwrap();
            assert_eq!(records[key], decision);
        }
        assert!(reconcile_decisions(&json!({}), &mut records, PreferenceScope::User).is_err());
        assert!(inheritance_declaration("/preference_decisions/claim").is_err());
        assert!(inheritance_declaration("/bad~xescape").is_err());
    }

    #[test]
    fn project_pointer_can_inherit_a_differently_named_parent_setting() {
        let decision = PreferenceDecision::new(
            &inheritance_declaration("/usage_guard/stop_remaining_percent").unwrap(),
            DecisionOrigin::Inherited,
            PreferenceInput::Terminal,
            PreferenceScope::Project,
            "0.14.0",
            Some(100),
        )
        .unwrap();
        let mut records = BTreeMap::from([("/usage_stop_remaining_percent".to_owned(), decision)]);
        reconcile_decisions(
            &json!({"usage_stop_remaining_percent":50}),
            &mut records,
            PreferenceScope::Project,
        )
        .unwrap();
        let encoded = serde_json::to_value(&records).unwrap();
        assert_eq!(
            encoded["/usage_stop_remaining_percent"]["inherited_from"]["key"],
            "/usage_guard/stop_remaining_percent"
        );
        records
            .get_mut("/usage_stop_remaining_percent")
            .unwrap()
            .inherited_from = None;
        assert!(reconcile_decisions(&json!({}), &mut records, PreferenceScope::Project).is_err());
    }
}
