//! Consume existing validated measurements; this module never acquires usage data.
use super::super::{AdapterError, InstalledUsageConfig, PinnedTarget};
use hive_core::sha256_digest;
use hive_core::usage_guard::{
    evaluate_observation_only, UsageDecision, UsagePolicy, UsageSnapshot,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LegacyEpisode {
    schema_version: u8,
    identity_digest: String,
    policy_digest: String,
    period: i64,
    measured_at: i64,
    healthy_observations: u8,
    low_shown: bool,
    zero_shown: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Episode {
    schema_version: u8,
    identity_digest: String,
    period: i64,
    measured_at: i64,
    policies: BTreeMap<String, PolicyEpisode>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PolicyEpisode {
    threshold: u8,
    healthy_observations: u8,
    low_shown: bool,
    zero_shown: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SavedEpisode {
    Current(Episode),
    Legacy(LegacyEpisode),
}

fn policy_digest(threshold: u8) -> String {
    sha256_digest(format!("quota-warning-v1:{threshold}").as_bytes())
}

fn load_episode(bytes: &[u8], identity: &str) -> Result<Episode, AdapterError> {
    let saved: SavedEpisode = serde_json::from_slice(bytes)
        .map_err(|_| AdapterError::Safety("invalid quota warning episode".to_owned()))?;
    let episode = match saved {
        SavedEpisode::Current(episode) => episode,
        SavedEpisode::Legacy(old) => {
            let threshold = (0..=99)
                .find(|value| policy_digest(*value) == old.policy_digest)
                .filter(|_| old.schema_version == 1)
                .ok_or_else(|| AdapterError::Safety("invalid legacy warning policy".into()))?;
            Episode {
                schema_version: 2,
                identity_digest: old.identity_digest,
                period: old.period,
                measured_at: old.measured_at,
                policies: BTreeMap::from([(
                    old.policy_digest,
                    PolicyEpisode {
                        threshold,
                        healthy_observations: old.healthy_observations,
                        low_shown: old.low_shown,
                        zero_shown: old.zero_shown,
                    },
                )]),
            }
        }
    };
    if episode.schema_version != 2
        || episode.identity_digest != identity
        || episode.policies.len() > 100
        || episode.policies.iter().any(|(key, value)| {
            value.threshold > 99
                || value.healthy_observations > 2
                || *key != policy_digest(value.threshold)
        })
    {
        return Err(AdapterError::Safety(
            "quota warning identity or policy differs".into(),
        ));
    }
    Ok(episode)
}

pub(in super::super) fn observe(
    user_root: Option<&Path>,
    fallback: &PinnedTarget,
    config: &InstalledUsageConfig,
    snapshots: &[UsageSnapshot],
    now: i64,
) -> Vec<Value> {
    if config.policy.is_none() || snapshots.is_empty() {
        return Vec::new();
    }
    let first = &snapshots[0];
    let validation = UsagePolicy::new(
        &first.sensor_id,
        &first.sensor_version,
        &config.primary_host,
        &first.account_scope_digest,
    );
    if !matches!(
        evaluate_observation_only(&validation, snapshots, now),
        UsageDecision::Allow(_)
    ) {
        return Vec::new();
    }
    let Ok(owned) = user_root.map(PinnedTarget::open_usage).transpose() else {
        return Vec::new();
    };
    let root = owned.as_ref().unwrap_or(fallback);
    let show = !crate::display_options::quiet()
        && config
            .policy
            .as_ref()
            .is_some_and(|policy| policy.warnings_enabled);
    snapshots
        .iter()
        .filter_map(|snapshot| claim(root, config, snapshot, show).ok().flatten())
        .collect()
}

fn claim(
    root: &PinnedTarget,
    config: &InstalledUsageConfig,
    sample: &UsageSnapshot,
    show: bool,
) -> Result<Option<Value>, AdapterError> {
    let identity = sha256_digest(
        serde_json::to_vec(&json!([
            "quota-warning",
            sample.host_scope,
            sample.account_scope_digest,
            sample.quota_pool.as_deref().unwrap_or("default"),
            sample.quota_window
        ]))
        .expect("quota identity serializes")
        .as_slice(),
    );
    let path = Path::new(".hive/runtime/usage-notices").join(format!("{}.json", &identity[7..]));
    let file = root.snapshot_bounded(&path, 32768)?;
    let previous: Option<Episode> = file
        .bytes()
        .map(|bytes| load_episode(bytes, &identity))
        .transpose()?;
    let policy_digest = policy_digest(config.threshold);
    // Replaying a cached measurement is not a second recovery observation. A stale process
    // must also not restore an old period after a newer process has already observed its reset.
    if previous
        .as_ref()
        .is_some_and(|old| sample.measured_at_unix_seconds < old.measured_at)
    {
        return Ok(None);
    }
    let repeated = previous
        .as_ref()
        .is_some_and(|old| sample.measured_at_unix_seconds == old.measured_at);
    let mut episode = previous.unwrap_or_else(|| Episode {
        schema_version: 2,
        identity_digest: identity,
        period: sample.resets_at_unix_seconds,
        measured_at: sample.measured_at_unix_seconds,
        policies: BTreeMap::new(),
    });
    if episode.period != sample.resets_at_unix_seconds {
        if repeated {
            return Ok(None);
        }
        episode.period = sample.resets_at_unix_seconds;
        episode.policies.clear();
    }
    episode.measured_at = sample.measured_at_unix_seconds;
    episode
        .policies
        .entry(policy_digest.clone())
        .or_insert(PolicyEpisode {
            threshold: config.threshold,
            healthy_observations: 0,
            low_shown: false,
            zero_shown: false,
        });
    // One already-obtained measurement can recover all remembered thresholds. Changing
    // which policy is selected must not erase another policy's notice or recovery history.
    for policy in episode.policies.values_mut() {
        if sample.remaining_percent > f64::from(policy.threshold) {
            if !repeated {
                policy.healthy_observations = policy.healthy_observations.saturating_add(1).min(2);
            }
            if policy.healthy_observations == 2 {
                policy.low_shown = false;
                policy.zero_shown = false;
            }
        } else {
            policy.healthy_observations = 0;
        }
    }
    let policy = episode
        .policies
        .get_mut(&policy_digest)
        .expect("selected warning policy");
    let kind = if !show || sample.remaining_percent > f64::from(config.threshold) {
        None
    } else if sample.remaining_percent == 0.0 {
        let first = !policy.zero_shown;
        policy.zero_shown = true;
        first.then_some("quota-empty")
    } else {
        let first = !policy.low_shown;
        policy.low_shown = true;
        first.then_some("quota-low")
    };
    let bytes =
        serde_json::to_vec(&episode).map_err(|error| AdapterError::Internal(error.to_string()))?;
    if file.bytes() != Some(bytes.as_slice()) {
        root.verify_current()?;
        crate::user_install::replace_user_setup_file(
            root.target_dir(),
            &path,
            file.bytes(),
            Some(&bytes),
        )
        .map_err(AdapterError::Conflict)?;
    }
    Ok(kind.map(|kind| notice(config, sample, kind)))
}

fn notice(config: &InstalledUsageConfig, sample: &UsageSnapshot, kind: &str) -> Value {
    json!({
        "kind": kind, "window": sample.quota_window, "quota_pool": sample.quota_pool,
        "remaining_percent": sample.remaining_percent, "resets_at": sample.resets_at_unix_seconds,
        "message": match (config.interface_language.as_str(), kind) {
            ("ko", "quota-empty") => "확인한 사용량 구간의 잔여량이 0%입니다.",
            ("ko", _) => "확인한 사용량 구간의 잔여량이 안내 기준 이하입니다.",
            (_, "quota-empty") => "A measured usage window has no quota remaining.",
            _ => "A measured usage window is at or below your warning threshold.",
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::{
        preferences::UsageProtection,
        usage_guard::{SourceConfidence, UsageWindow},
    };

    fn config() -> InstalledUsageConfig {
        let mut config = super::super::super::parse_installed_config(
            super::super::super::tests::installed_config("codex", true, false),
        )
        .unwrap();
        config.policy = Some(UsageProtection::default());
        config.threshold = 20;
        config
    }

    fn sample(remaining: f64, measured: i64) -> UsageSnapshot {
        UsageSnapshot {
            schema_version: 1,
            sensor_id: "fixture".into(),
            sensor_version: "1".into(),
            host_scope: "codex".into(),
            account_scope_digest: sha256_digest(b"account"),
            quota_pool: None,
            quota_window: UsageWindow::Weekly,
            remaining_percent: remaining,
            measured_at_unix_seconds: measured,
            expires_at_unix_seconds: measured + 120,
            resets_at_unix_seconds: 10000,
            source_confidence: SourceConfidence::High,
        }
    }

    #[test]
    fn quota_notices_survive_restart_and_rearm_after_two_distinct_healthy_measurements() {
        let temp = tempfile::tempdir().unwrap();
        let config = config();
        let check = |value, time| {
            observe(
                None,
                &PinnedTarget::open_usage(temp.path()).unwrap(),
                &config,
                &[sample(value, time)],
                time,
            )
        };
        assert_eq!(check(10.0, 1)[0]["kind"], "quota-low");
        assert!(check(10.0, 2).is_empty());
        assert_eq!(check(0.0, 3)[0]["kind"], "quota-empty");
        assert!(check(0.0, 4).is_empty());
        assert!(check(50.0, 5).is_empty());
        assert!(check(50.0, 5).is_empty());
        assert!(check(10.0, 6).is_empty());
        assert!(check(50.0, 7).is_empty());
        assert!(check(50.0, 8).is_empty());
        assert_eq!(check(10.0, 9).len(), 1);
        assert_eq!(check(0.0, 10).len(), 1);
    }

    #[test]
    fn revisiting_thresholds_retains_each_policy_until_recovery_or_a_new_period() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = config();
        let mut check = |threshold, remaining, time, period| {
            config.threshold = threshold;
            let mut measurement = sample(remaining, time);
            measurement.resets_at_unix_seconds = period;
            observe(
                None,
                &PinnedTarget::open_usage(temp.path()).unwrap(),
                &config,
                &[measurement],
                time,
            )
        };
        assert_eq!(check(20, 10.0, 1, 10000).len(), 1);
        assert_eq!(check(40, 10.0, 2, 10000).len(), 1);
        assert!(check(20, 10.0, 3, 10000).is_empty());
        assert!(check(40, 10.0, 4, 10000).is_empty());
        assert!(check(40, 30.0, 5, 10000).is_empty());
        assert!(check(40, 30.0, 5, 10000).is_empty());
        assert!(check(40, 30.0, 6, 10000).is_empty());
        assert_eq!(check(20, 10.0, 7, 10000).len(), 1);
        assert!(check(40, 10.0, 8, 10000).is_empty());
        assert_eq!(check(40, 10.0, 9, 11000).len(), 1);
        assert_eq!(check(20, 10.0, 10, 11000).len(), 1);
        assert!(check(40, 10.0, 1, 10000).is_empty());
    }

    #[test]
    fn legacy_quota_episode_keeps_its_already_shown_notice_during_migration() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let mut config = config();
        assert_eq!(
            observe(None, &root, &config, &[sample(10.0, 1)], 1).len(),
            1
        );
        let path = std::fs::read_dir(temp.path().join(".hive/runtime/usage-notices"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|ext| ext == "json"))
            .unwrap();
        let current: Episode = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let legacy = LegacyEpisode {
            schema_version: 1,
            identity_digest: current.identity_digest,
            policy_digest: policy_digest(20),
            period: current.period,
            measured_at: current.measured_at,
            healthy_observations: 0,
            low_shown: true,
            zero_shown: false,
        };
        std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(observe(None, &root, &config, &[sample(10.0, 2)], 2).is_empty());
        config.threshold = 40;
        assert_eq!(
            observe(None, &root, &config, &[sample(10.0, 3)], 3).len(),
            1
        );
        config.threshold = 20;
        assert!(observe(None, &root, &config, &[sample(10.0, 4)], 4).is_empty());
        let saved: Episode = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(saved.schema_version, 2);
        assert_eq!(saved.policies.len(), 2);
    }

    #[test]
    fn quota_period_account_and_policy_are_independent_and_old_observations_do_not_rearm() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let mut config = config();
        let mut value = sample(10.0, 1);
        assert_eq!(observe(None, &root, &config, &[value.clone()], 1).len(), 1);
        value.measured_at_unix_seconds = 2;
        value.resets_at_unix_seconds += 1000;
        assert_eq!(observe(None, &root, &config, &[value.clone()], 2).len(), 1);
        assert!(observe(None, &root, &config, &[sample(10.0, 1)], 2).is_empty());
        assert!(observe(None, &root, &config, &[value.clone()], 2).is_empty());
        value.account_scope_digest = sha256_digest(b"another account");
        assert_eq!(observe(None, &root, &config, &[value.clone()], 2).len(), 1);
        config.threshold = 30;
        assert_eq!(observe(None, &root, &config, &[value], 2).len(), 1);
    }

    #[test]
    fn quiet_and_warning_opt_out_do_not_claim_notices_and_invalid_data_is_not_recorded() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let mut config = config();
        assert!(claim(&root, &config, &sample(10.0, 1), false)
            .unwrap()
            .is_none());
        config.policy.as_mut().unwrap().warnings_enabled = false;
        assert!(observe(None, &root, &config, &[sample(10.0, 1)], 1).is_empty());
        config.policy.as_mut().unwrap().warnings_enabled = true;
        assert_eq!(
            observe(None, &root, &config, &[sample(10.0, 1)], 1).len(),
            1
        );
        let empty = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(empty.path()).unwrap();
        assert!(observe(None, &root, &config, &[sample(10.0, 1)], 500).is_empty());
        assert!(!empty.path().join(".hive").exists());
    }

    #[test]
    fn concurrent_quota_claims_emit_once() {
        let temp = tempfile::tempdir().unwrap();
        let config = config();
        let barrier = std::sync::Barrier::new(2);
        let count = std::thread::scope(|scope| {
            let check = || {
                let root = PinnedTarget::open_usage(temp.path()).unwrap();
                barrier.wait();
                observe(None, &root, &config, &[sample(10.0, 1)], 1).len()
            };
            let first = scope.spawn(check);
            let second = scope.spawn(check);
            first.join().unwrap() + second.join().unwrap()
        });
        assert_eq!(count, 1);
    }
}
