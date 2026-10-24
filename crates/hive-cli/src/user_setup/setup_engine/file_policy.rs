//! Translate predecessor answer files without letting them enable unrepresented new controls.
use crate::user_setup::{self, SetupError, SetupMode, UserSetupConfig};
use hive_core::preferences::UsageProtection;
use serde_json::{json, Value};
use std::path::Path;

pub(super) struct Normalized {
    pub baseline: Option<UserSetupConfig>,
    pub migration: Option<user_setup::progress_store::LegacyMigration>,
}

pub(super) fn normalize(
    root: &Path,
    config: &mut UserSetupConfig,
    supplied: &mut Value,
    current: Option<&UserSetupConfig>,
    quick: bool,
    mode: SetupMode,
) -> Result<Normalized, SetupError> {
    // A validation command must still be able to check an untouched predecessor installation.
    if mode == SetupMode::Validate && current.is_none_or(|c| c.schema_version == 1) {
        return Ok(Normalized {
            baseline: current.cloned(),
            migration: None,
        });
    }
    let preview = current
        .map(|c| user_setup::migration::inspect(root, c))
        .transpose()?
        .flatten();
    let migration = preview
        .as_ref()
        .map(|p| user_setup::progress_store::LegacyMigration {
            migration_id: p.migration_id.clone(),
            monitor_choice: p.monitor_choice.clone(),
            from_schema: 1,
            questions: p.pending_questions.iter().map(|id| (*id).into()).collect(),
        });
    let baseline = preview.map(|p| p.candidate).or_else(|| current.cloned());
    if config.schema_version == 2 {
        return Ok(Normalized {
            baseline,
            migration,
        });
    }
    let fresh = baseline.is_none();
    let legacy_install = current.is_some_and(|c| c.schema_version == 1);
    let mut policy = baseline
        .as_ref()
        .and_then(|c| c.usage_guard.policy.clone())
        .unwrap_or_default();
    let threshold = supplied
        .pointer("/usage_guard/enabled")
        .and_then(Value::as_bool);
    let reset = supplied
        .pointer("/usage_guard/quota_reset_guard_enabled")
        .or_else(|| supplied.pointer("/usage_guard/reset_booster_enabled"))
        .and_then(Value::as_bool);
    if legacy_install {
        let prior = current.expect("legacy installation");
        policy = UsageProtection::from_legacy(
            if quick {
                prior.usage_guard.enabled
            } else {
                threshold.unwrap_or(prior.usage_guard.enabled)
            },
            if quick {
                prior.usage_guard.quota_reset_guard_enabled
            } else {
                reset.unwrap_or(prior.usage_guard.quota_reset_guard_enabled)
            },
            policy.midway.enabled,
        );
    } else if !quick {
        if let Some(threshold) = threshold {
            policy.preflight.threshold_enabled = threshold;
            if fresh {
                policy.enforcement_enabled = threshold;
            }
            if policy.midway.enabled {
                policy.midway.threshold_enabled = threshold;
            }
            if !threshold {
                policy.preflight.reset_enabled = false;
            }
        }
        if let Some(reset) = reset {
            policy.preflight.reset_enabled = policy.preflight.threshold_enabled && reset;
        }
    }
    config.schema_version = 2;
    config.usage_guard.enabled = false;
    config.usage_guard.quota_reset_guard_enabled = true;
    config.usage_guard.popup_enabled = config
        .usage_guard
        .popup_enabled
        .or_else(|| baseline.as_ref().and_then(|c| c.usage_guard.popup_enabled))
        .or(Some(false));
    config.usage_guard.policy = Some(policy.clone());
    mark_supplied(supplied, &policy, threshold, reset, fresh, quick);
    Ok(Normalized {
        baseline,
        migration,
    })
}

fn mark_supplied(
    supplied: &mut Value,
    policy: &UsageProtection,
    threshold: Option<bool>,
    reset: Option<bool>,
    fresh: bool,
    quick: bool,
) {
    // Mark only the new declarations actually represented by the legacy file as supplied.
    // Master and monitoring choices in an existing v2 installation have no legacy equivalent.
    if !quick {
        if threshold.is_some() {
            supplied["usage_guard"]["preflight"] =
                json!({"threshold_enabled":policy.preflight.threshold_enabled});
            if fresh {
                supplied["usage_guard"]["enforcement_enabled"] = json!(policy.enforcement_enabled);
            }
            if policy.midway.enabled {
                supplied["usage_guard"]["midway"] =
                    json!({"threshold_enabled":policy.midway.threshold_enabled});
            }
        }
        if reset.is_some() || threshold == Some(false) {
            if !supplied["usage_guard"]["preflight"].is_object() {
                supplied["usage_guard"]["preflight"] = json!({});
            }
            supplied["usage_guard"]["preflight"]["reset_enabled"] =
                json!(policy.preflight.reset_enabled);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::InterfaceLanguage;

    fn legacy() -> (UserSetupConfig, Value) {
        let mut value = super::super::default_answers(Some(InterfaceLanguage::En));
        value["schema_version"] = json!(1);
        value["usage_guard"] =
            json!({"enabled":true,"quota_reset_guard_enabled":true,"stop_remaining_percent":17});
        let config =
            user_setup::parse_and_validate_config(&serde_json::to_vec(&value).unwrap()).unwrap();
        (config, value)
    }

    #[test]
    fn fresh_legacy_quick_answers_do_not_activate_old_defaults() {
        let root = tempfile::tempdir().unwrap();
        for quick in [true, false] {
            let (mut config, mut supplied) = legacy();
            normalize(
                root.path(),
                &mut config,
                &mut supplied,
                None,
                quick,
                SetupMode::Apply,
            )
            .unwrap();
            let policy = config.usage_guard.policy.as_ref().unwrap();
            assert_eq!(config.schema_version, 2);
            assert_eq!(policy.enforcement_enabled, !quick);
            assert_eq!(policy.preflight.threshold_enabled, !quick);
            assert_eq!(policy.preflight.reset_enabled, !quick);
            assert!(!policy.preflight.zero_quota_enabled);
            assert!(!policy.midway.enabled);
            assert_eq!(
                policy.preflight.observation_failure,
                hive_core::preferences::PreflightFailure::Warn
            );
            assert_eq!(config.usage_guard.stop_remaining_percent, 17);
        }
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn old_files_preserve_existing_v2_master_monitor_and_warning_choices() {
        let root = tempfile::tempdir().unwrap();
        let value = super::super::default_answers(Some(InterfaceLanguage::En));
        let current =
            user_setup::parse_and_validate_config(&serde_json::to_vec(&value).unwrap()).unwrap();
        let (mut config, mut supplied) = legacy();
        normalize(
            root.path(),
            &mut config,
            &mut supplied,
            Some(&current),
            false,
            SetupMode::Apply,
        )
        .unwrap();
        let policy = config.usage_guard.policy.as_ref().unwrap();
        assert!(!policy.enforcement_enabled);
        assert!(!policy.midway.enabled);
        assert!(policy.warnings_enabled);
        assert!(policy.preflight.threshold_enabled);
        assert!(!policy.effective_preflight().threshold_enabled);
        assert!(supplied
            .pointer("/usage_guard/enforcement_enabled")
            .is_none());
        let roundtrip =
            user_setup::parse_and_validate_config(&serde_json::to_vec(&config).unwrap()).unwrap();
        assert_eq!(config, roundtrip);
        let first = config.clone();
        let (mut retried, mut old_input) = legacy();
        normalize(
            root.path(),
            &mut retried,
            &mut old_input,
            Some(&first),
            true,
            SetupMode::Apply,
        )
        .unwrap();
        assert_eq!(retried.usage_guard.policy, first.usage_guard.policy);
    }

    #[test]
    fn untouched_legacy_validation_stays_pure_but_writes_preserve_its_independent_zero_guard() {
        let root = tempfile::tempdir().unwrap();
        let (mut config, mut supplied) = legacy();
        config.usage_guard.enabled = false;
        supplied["usage_guard"]["enabled"] = json!(false);
        let current = config.clone();
        normalize(
            root.path(),
            &mut config,
            &mut supplied,
            Some(&current),
            false,
            SetupMode::Validate,
        )
        .unwrap();
        assert_eq!(config, current);
        let baseline = normalize(
            root.path(),
            &mut config,
            &mut supplied,
            Some(&current),
            false,
            SetupMode::Apply,
        )
        .unwrap()
        .baseline
        .unwrap();
        let policy = config.usage_guard.policy.as_ref().unwrap();
        assert!(policy.enforcement_enabled);
        assert!(policy.preflight.zero_quota_enabled);
        assert!(!policy.preflight.threshold_enabled);
        assert!(!policy.midway.enabled);
        assert!(!policy.warnings_enabled);
        assert_eq!(
            baseline.preference_decisions["/usage_guard/enforcement_enabled"].origin,
            hive_core::preferences::DecisionOrigin::LegacyUnknown
        );
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn damaged_legacy_monitor_defers_file_conversion_without_writes() {
        let root = tempfile::tempdir().unwrap();
        let (mut config, mut supplied) = legacy();
        let current = config.clone();
        let path = root.path().join(".hive/runtime/usage-monitor/config.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"unverified old monitor").unwrap();
        assert!(normalize(
            root.path(),
            &mut config,
            &mut supplied,
            Some(&current),
            false,
            SetupMode::Apply
        )
        .is_err());
        assert_eq!(config, current);
        assert_eq!(std::fs::read(path).unwrap(), b"unverified old monitor");
        assert!(!root.path().join(".hive/config").exists());
    }
}
