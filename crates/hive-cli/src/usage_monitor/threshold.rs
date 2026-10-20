//! Shared-policy threshold episodes. A receipt is scoped history, never an automatic permit.
use super::state::MonitorState;
use crate::usage_control::MonitorPolicy;
use hive_core::usage_guard::{UsageSnapshot, UsageWindow};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub(super) struct ActiveSession {
    pub id: String,
    pub turn: String,
}

pub(super) fn policies(
    root: &Path,
    contexts: &[crate::usage::MonitorThread],
    active: &[ActiveSession],
) -> (Vec<MonitorPolicy>, u64) {
    let mut policies = Vec::new();
    let mut errors = 0;
    for session in active {
        if session.turn.is_empty() {
            errors += 1;
            continue;
        }
        let Some(mut context) = contexts.iter().find(|context| context.id == session.id) else {
            errors += 1;
            continue;
        };
        let mut seen = std::collections::BTreeSet::new();
        while let Some(parent) = context.parent_id.as_deref() {
            if !seen.insert(context.id.as_str()) {
                break;
            }
            let Some(found) = contexts.iter().find(|context| context.id == parent) else {
                break;
            };
            context = found;
        }
        if context.parent_id.is_some() {
            errors += 1;
            continue;
        }
        let Some(cwd) = context.cwd.as_deref() else {
            errors += 1;
            continue;
        };
        match crate::usage_control::monitor_policy(root, cwd, &session.id) {
            Ok(Some(policy)) => policies.push(policy),
            Ok(None) => {}
            Err(_) => errors += 1,
        }
    }
    (policies, errors)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    pub session_digest: String,
    pub target_digest: String,
    pub policy_digest: String,
    pub control_digest: String,
    pub account_digest: String,
    #[serde(default)]
    pub quota_pool: Option<String>,
    pub window: UsageWindow,
    pub remaining_percent: f64,
    pub threshold_percent: u8,
    pub confirmed: bool,
}

pub(super) fn observe(
    state: &mut MonitorState,
    policies: &[MonitorPolicy],
    snapshots: &[UsageSnapshot],
    at: u64,
) {
    // Recovery above the floor rearms even when the affected conversation is idle.
    state.threshold_receipts.retain(|receipt| {
        snapshots.iter().any(|sample| {
            sample.account_scope_digest == receipt.account_digest
                && sample.quota_pool == receipt.quota_pool
                && sample.quota_window == receipt.window
                && sample.remaining_percent <= f64::from(receipt.threshold_percent)
        })
    });
    let mut reached = Vec::new();
    for policy in policies {
        if !policy.enabled && !policy.zero_enabled {
            continue;
        }
        let block = match crate::usage_control::evaluate_monitor_policy(policy, snapshots, at) {
            Ok(Some(block)) => block,
            Ok(None) => continue,
            Err(_) => {
                state.threshold_policy_errors = state.threshold_policy_errors.saturating_add(1);
                continue;
            }
        };
        let Some(sample) = snapshots.iter().find(|sample| {
            sample.quota_window == block.window
                && sample.remaining_percent == block.remaining_percent
        }) else {
            continue;
        };
        let current = state.threshold_receipts.iter().position(|receipt| {
            receipt.session_digest == policy.session_digest
                && receipt.target_digest == policy.target_digest
        });
        if current.is_some_and(|index| {
            let receipt = &state.threshold_receipts[index];
            receipt.policy_digest == policy.policy_digest
                && receipt.control_digest == policy.control_digest
                && receipt.account_digest == sample.account_scope_digest
                && receipt.quota_pool == sample.quota_pool
                && receipt.window == block.window
                && f64::from(receipt.threshold_percent) == block.threshold_percent
        }) {
            continue;
        }
        if let Some(index) = current {
            state.threshold_receipts.remove(index);
        }
        if state.threshold_receipts.len() >= 32 {
            state.threshold_policy_errors = state.threshold_policy_errors.saturating_add(1);
            continue;
        }
        let receipt = Receipt {
            session_digest: policy.session_digest.clone(),
            target_digest: policy.target_digest.clone(),
            policy_digest: policy.policy_digest.clone(),
            control_digest: policy.control_digest.clone(),
            account_digest: sample.account_scope_digest.clone(),
            quota_pool: sample.quota_pool.clone(),
            window: block.window,
            remaining_percent: block.remaining_percent,
            threshold_percent: if block.threshold_percent == 0.0 {
                0
            } else {
                policy.threshold
            },
            confirmed: false,
        };
        reached.push(receipt.clone());
        state.threshold_receipts.push(receipt);
    }
    if reached.is_empty() {
        return;
    }
    let reset_pending = state.incident.as_ref().is_some_and(|incident| {
        incident.reason == "usage-reset" && incident.stop_dispatched_at.is_none()
    });
    if !reset_pending {
        state.halt("usage-threshold", at);
    }
    if let Some(incident) = state.incident.as_mut() {
        if !reset_pending {
            incident.target_sessions = Some(
                reached
                    .iter()
                    .map(|receipt| receipt.session_digest.clone())
                    .collect(),
            );
        }
        incident.threshold_details = reached;
    }
}

pub(super) fn handled(
    state: &MonitorState,
    policy: &MonitorPolicy,
    snapshots: &[UsageSnapshot],
) -> bool {
    state.threshold_receipts.iter().any(|receipt| {
        receipt.confirmed
            && receipt.session_digest == policy.session_digest
            && receipt.target_digest == policy.target_digest
            && receipt.policy_digest == policy.policy_digest
            && receipt.control_digest == policy.control_digest
            && receipt.threshold_percent == policy.threshold
            && snapshots.iter().any(|sample| {
                sample.account_scope_digest == receipt.account_digest
                    && sample.quota_pool == receipt.quota_pool
                    && sample.quota_window == receipt.window
                    && (sample.remaining_percent > 0.0 || !policy.zero_enabled)
                    && sample.remaining_percent <= f64::from(receipt.threshold_percent)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::{sha256_digest, usage_guard::SourceConfidence};

    fn policy(session: &str, threshold: u8) -> MonitorPolicy {
        MonitorPolicy {
            session_digest: sha256_digest(session.as_bytes()),
            target_digest: sha256_digest(b"target"),
            policy_digest: sha256_digest(&[threshold]),
            control_digest: "absent".into(),
            threshold,
            enabled: true,
            zero_enabled: true,
        }
    }
    fn sample(remaining: f64, at: u64) -> Vec<UsageSnapshot> {
        vec![UsageSnapshot {
            schema_version: 1,
            sensor_id: "codex-app-server".into(),
            sensor_version: "0.160.1".into(),
            host_scope: "codex".into(),
            account_scope_digest: sha256_digest(b"account"),
            quota_pool: None,
            quota_window: UsageWindow::Session,
            remaining_percent: remaining,
            measured_at_unix_seconds: i64::try_from(at).unwrap(),
            expires_at_unix_seconds: i64::try_from(at + 60).unwrap(),
            resets_at_unix_seconds: 1000,
            source_confidence: SourceConfidence::High,
        }]
    }

    #[test]
    fn first_low_or_equal_sample_stops_only_the_policies_whose_floor_was_reached() {
        let mut state = MonitorState::new();
        let low = policy("lower", 10);
        let high = policy("higher", 20);
        observe(
            &mut state,
            &[low.clone(), high.clone()],
            &sample(20.0, 100),
            100,
        );
        let incident = state.incident.as_ref().unwrap();
        assert_eq!(incident.reason, "usage-threshold");
        assert_eq!(
            incident.target_sessions,
            Some(vec![high.session_digest.clone()])
        );
        assert!(!handled(&state, &high, &sample(19.0, 101)));
        let digest = incident.digest.clone();
        state.threshold_receipts[0].confirmed = true;
        assert!(handled(&state, &high, &sample(19.0, 101)));
        assert!(!handled(&state, &low, &sample(9.0, 101)));
        observe(&mut state, &[high], &sample(19.0, 115), 115);
        assert_eq!(state.incident.unwrap().digest, digest);
    }

    #[test]
    fn confirmation_is_conversation_account_policy_and_control_scoped_and_preserves_zero() {
        let mut state = MonitorState::new();
        let scope = policy("one", 10);
        observe(
            &mut state,
            std::slice::from_ref(&scope),
            &sample(9.0, 100),
            100,
        );
        state.threshold_receipts[0].confirmed = true;
        let restored: MonitorState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert!(handled(&restored, &scope, &sample(8.0, 115)));
        assert!(!handled(&restored, &scope, &sample(0.0, 115)));
        assert!(!handled(&restored, &policy("two", 10), &sample(8.0, 115)));
        assert!(!handled(&restored, &policy("one", 11), &sample(8.0, 115)));
        let mut changed = scope.clone();
        changed.control_digest = sha256_digest(b"new-control");
        assert!(!handled(&restored, &changed, &sample(8.0, 115)));
        let mut changed_sample = sample(8.0, 115);
        changed_sample[0].account_scope_digest = sha256_digest(b"other");
        assert!(!handled(&restored, &scope, &changed_sample));
    }

    #[test]
    fn recovery_above_floor_rearms_but_session_disable_is_preserved() {
        let mut state = MonitorState::new();
        let scope = policy("one", 10);
        observe(
            &mut state,
            std::slice::from_ref(&scope),
            &sample(9.0, 100),
            100,
        );
        let first = state.incident.as_ref().unwrap().digest.clone();
        observe(&mut state, &[], &sample(11.0, 115), 115);
        assert!(state.threshold_receipts.is_empty());
        observe(
            &mut state,
            std::slice::from_ref(&scope),
            &sample(9.0, 130),
            130,
        );
        assert_ne!(state.incident.as_ref().unwrap().digest, first);
        let mut disabled = scope;
        disabled.enabled = false;
        let mut other = MonitorState::new();
        observe(
            &mut other,
            std::slice::from_ref(&disabled),
            &sample(1.0, 130),
            130,
        );
        assert!(other.incident.is_none());
        observe(
            &mut other,
            std::slice::from_ref(&disabled),
            &sample(0.0, 131),
            131,
        );
        assert!(other.incident.is_some());
        assert_eq!(other.threshold_receipts[0].threshold_percent, 0);
        disabled.zero_enabled = false;
        let mut opted_out = MonitorState::new();
        observe(&mut opted_out, &[disabled], &sample(0.0, 131), 131);
        assert!(opted_out.incident.is_none());
    }

    #[test]
    fn reset_and_floor_from_one_observation_share_one_interruption() {
        let mut state = MonitorState::new();
        state.halt("usage-reset", 100);
        let generation = state.generation;
        observe(&mut state, &[policy("one", 20)], &sample(10.0, 100), 100);
        assert_eq!(state.generation, generation);
        assert_eq!(state.incident.as_ref().unwrap().reason, "usage-reset");
        assert_eq!(state.incident.as_ref().unwrap().threshold_details.len(), 1);
    }

    #[test]
    fn independent_zero_rearms_after_floor_and_usage_pools_do_not_share_resume_receipts() {
        let mut state = MonitorState::new();
        let scope = policy("one", 10);
        let mut low = sample(8.0, 100);
        low[0].schema_version = 2;
        low[0].quota_pool = Some("primary".into());
        observe(&mut state, std::slice::from_ref(&scope), &low, 100);
        state.threshold_receipts[0].confirmed = true;
        let generation = state.generation;
        let mut other = low.clone();
        other[0].quota_pool = Some("secondary".into());
        assert!(!handled(&state, &scope, &other));
        assert!(handled(&state, &scope, &low));
        low[0].remaining_percent = 0.0;
        low[0].measured_at_unix_seconds = 101;
        observe(&mut state, &[scope], &low, 101);
        assert!(state.generation > generation);
        assert_eq!(state.threshold_receipts[0].threshold_percent, 0);
    }

    #[cfg(windows)]
    #[test]
    fn a_verified_stop_can_never_be_used_as_an_automatic_dispatch_bypass() {
        use crate::run::PinnedTarget;
        use crate::usage_monitor::{
            configuration, threshold_already_handled, with_state, write, CONFIG,
        };
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        write(&root, CONFIG, &configuration(temporary.path()).unwrap()).unwrap();
        let scope = policy("one", 10);
        with_state(&root, |state| {
            observe(state, std::slice::from_ref(&scope), &sample(9.0, 100), 100);
            state.threshold_receipts[0].confirmed = true;
            Ok(())
        })
        .unwrap();
        assert!(threshold_already_handled(
            &root,
            &scope,
            &sample(8.0, 101),
            false
        ));
        assert!(!threshold_already_handled(
            &root,
            &scope,
            &sample(8.0, 101),
            true
        ));
        assert!(!threshold_already_handled(
            &root,
            &policy("other", 10),
            &sample(8.0, 101),
            false
        ));
        assert!(!threshold_already_handled(
            &root,
            &scope,
            &sample(0.0, 101),
            false
        ));
    }
}
