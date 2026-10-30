//! Read v1 selections without changing their meaning; write the selected policy version.

use super::DiscordGuardPreferences;
use hive_core::preferences::{MidwayPreferences, PreflightPreferences, UsageProtection};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UsageGuardPreferences {
    // These two fields retain the original v1 declarations. V2 callers use the policy methods.
    pub(crate) enabled: bool,
    pub(crate) quota_reset_guard_enabled: bool,
    pub(crate) policy: Option<UsageProtection>,
    pub(crate) stop_remaining_percent: u8,
    pub(crate) codexbar_fallback_enabled: bool,
    pub(crate) popup_enabled: Option<bool>,
    pub(crate) discord: DiscordGuardPreferences,
    pub(crate) project_overrides: BTreeMap<String, u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WirePreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    stop_remaining_percent: u8,
    #[serde(default)]
    codexbar_fallback_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    popup_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quota_reset_guard_enabled: Option<bool>,
    #[serde(default)]
    discord: DiscordGuardPreferences,
    #[serde(default)]
    project_overrides: BTreeMap<String, u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enforcement_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    warnings_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preflight: Option<PreflightPreferences>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    midway: Option<MidwayPreferences>,
}

impl<'de> Deserialize<'de> for UsageGuardPreferences {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = WirePreferences::deserialize(deserializer)?;
        let policy = match (
            wire.enforcement_enabled,
            wire.warnings_enabled,
            wire.preflight,
            wire.midway,
        ) {
            (None, None, None, None) => None,
            (Some(enforcement_enabled), Some(warnings_enabled), Some(preflight), Some(midway)) => {
                if wire.enabled.is_some() || wire.quota_reset_guard_enabled.is_some() {
                    return Err(serde::de::Error::custom(
                        "usage preferences cannot mix v1 flags with a v2 policy",
                    ));
                }
                Some(UsageProtection {
                    enforcement_enabled,
                    warnings_enabled,
                    preflight,
                    midway,
                })
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "v2 usage preferences require master, warnings, preflight, and midway controls",
                ));
            }
        };
        let popup_enabled = wire
            .popup_enabled
            .or_else(|| policy.is_some().then_some(false));
        Ok(Self {
            enabled: wire.enabled.unwrap_or(false),
            quota_reset_guard_enabled: wire.quota_reset_guard_enabled.unwrap_or(true),
            policy,
            stop_remaining_percent: wire.stop_remaining_percent,
            codexbar_fallback_enabled: wire.codexbar_fallback_enabled,
            popup_enabled,
            discord: wire.discord,
            project_overrides: wire.project_overrides,
        })
    }
}

impl Serialize for UsageGuardPreferences {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WirePreferences {
            enabled: self.policy.is_none().then_some(self.enabled),
            stop_remaining_percent: self.stop_remaining_percent,
            codexbar_fallback_enabled: self.codexbar_fallback_enabled,
            popup_enabled: self.popup_enabled,
            quota_reset_guard_enabled: self
                .policy
                .is_none()
                .then_some(self.quota_reset_guard_enabled),
            discord: self.discord.clone(),
            project_overrides: self.project_overrides.clone(),
            enforcement_enabled: self
                .policy
                .as_ref()
                .map(|policy| policy.enforcement_enabled),
            warnings_enabled: self.policy.as_ref().map(|policy| policy.warnings_enabled),
            preflight: self.policy.as_ref().map(|policy| policy.preflight.clone()),
            midway: self.policy.as_ref().map(|policy| policy.midway.clone()),
        }
        .serialize(serializer)
    }
}

impl UsageGuardPreferences {
    /// The legacy projection flag represents the overall v2 choice, not one check.
    pub(crate) fn projection_enabled(&self) -> bool {
        self.policy
            .as_ref()
            .map_or(self.enabled, |policy| policy.enforcement_enabled)
    }
    pub(crate) fn threshold_enabled(&self) -> bool {
        self.policy.as_ref().map_or(self.enabled, |policy| {
            policy.effective_preflight().threshold_enabled
        })
    }

    pub(crate) fn reset_enabled(&self) -> bool {
        self.policy
            .as_ref()
            .map_or(self.quota_reset_guard_enabled, |policy| {
                policy.effective_preflight().reset_enabled
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn predecessor_roundtrip_keeps_flat_policy_and_defaults() {
        let guard: UsageGuardPreferences = serde_json::from_value(json!({
            "enabled":false,"stop_remaining_percent":5
        }))
        .unwrap();
        assert!(!guard.threshold_enabled());
        assert!(guard.reset_enabled());
        assert_eq!(guard.popup_enabled, None);
        let saved = serde_json::to_value(&guard).unwrap();
        assert_eq!(saved["enabled"], false);
        assert_eq!(saved["quota_reset_guard_enabled"], true);
        assert!(saved.get("enforcement_enabled").is_none());
    }

    #[test]
    fn v2_roundtrip_keeps_dormant_selections_without_flat_aliases() {
        let mut policy = UsageProtection::from_legacy(true, true, true);
        policy.enforcement_enabled = false;
        let mut value = serde_json::to_value(&policy).unwrap();
        value["stop_remaining_percent"] = json!(5);
        let guard: UsageGuardPreferences = serde_json::from_value(value).unwrap();
        assert!(!guard.threshold_enabled());
        assert!(!guard.reset_enabled());
        assert_eq!(guard.popup_enabled, Some(false));
        let saved = serde_json::to_value(&guard).unwrap();
        assert!(saved.get("enabled").is_none());
        assert!(saved.get("quota_reset_guard_enabled").is_none());
        assert_eq!(saved["preflight"]["threshold_enabled"], true);
        assert_eq!(saved["midway"]["enabled"], true);
        assert_eq!(saved["popup_enabled"], false);
        assert_eq!(
            serde_json::from_value::<UsageGuardPreferences>(saved).unwrap(),
            guard
        );
    }

    #[test]
    fn mixed_or_partial_policies_are_not_interpreted_as_legacy() {
        let mut value = serde_json::to_value(UsageProtection::default()).unwrap();
        value["stop_remaining_percent"] = json!(20);
        value["enabled"] = json!(false);
        assert!(serde_json::from_value::<UsageGuardPreferences>(value.clone()).is_err());
        value.as_object_mut().unwrap().remove("enabled");
        value.as_object_mut().unwrap().remove("midway");
        assert!(serde_json::from_value::<UsageGuardPreferences>(value).is_err());
    }
}
