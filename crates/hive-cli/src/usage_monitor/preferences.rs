//! Live policy selection is separate from monitor registration and from dispatch preflight.
use super::{error, state::MonitorState, AdapterError, PinnedTarget};
use hive_core::preferences::{MidwayFailure, MidwayPreferences};
use hive_core::sha256_digest;
use std::path::Path;

#[derive(Clone)]
pub(super) struct Preferences {
    pub checks: MidwayPreferences,
    pub popup_enabled: bool,
}

pub(super) fn load(root: &PinnedTarget) -> Result<Preferences, AdapterError> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Option<(String, Preferences)>>> =
        std::sync::OnceLock::new();
    let bytes = root.read_required(
        Path::new(".hive/config/user-setup.yml"),
        usize::try_from(crate::user_setup::MAX_USER_SETUP_BYTES)
            .expect("setup limit fits supported platforms"),
    )?;
    let digest = sha256_digest(&bytes);
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(None));
    if let Some((prior, value)) = cache
        .lock()
        .map_err(|_| error("monitor policy cache unavailable"))?
        .as_ref()
    {
        if prior == &digest {
            return Ok(value.clone());
        }
    }
    let config = crate::user_setup::parse_and_validate_installed_config(&bytes)
        .map_err(|failure| error(failure.message()))?;
    let checks = config.usage_guard.policy.as_ref().map_or_else(
        || MidwayPreferences {
            enabled: true,
            threshold_enabled: config.usage_guard.enabled,
            reset_enabled: true,
            zero_quota_enabled: true,
            observation_failure: MidwayFailure::Stop,
        },
        hive_core::preferences::UsageProtection::effective_midway,
    );
    let value = Preferences {
        checks,
        popup_enabled: config
            .usage_guard
            .popup_enabled
            .unwrap_or(config.usage_guard.policy.is_none()),
    };
    *cache
        .lock()
        .map_err(|_| error("monitor policy cache unavailable"))? = Some((digest, value.clone()));
    Ok(value)
}

impl Preferences {
    pub fn stop_on_failure(&self) -> bool {
        self.checks.enabled && self.checks.observation_failure == MidwayFailure::Stop
    }

    pub fn notify(&self) -> bool {
        self.popup_enabled && !crate::display_options::quiet()
    }

    pub fn filter_pending(&self, state: &mut MonitorState) {
        let Some(incident) = state
            .incident
            .as_mut()
            .filter(|incident| incident.stop_dispatched_at.is_none())
        else {
            return;
        };
        state.threshold_receipts.retain(|receipt| {
            receipt.confirmed
                || (self.checks.enabled
                    && if receipt.is_zero_quota() {
                        self.checks.zero_quota_enabled
                    } else {
                        self.checks.threshold_enabled
                    })
        });
        let allowed = self.checks.enabled
            && match incident.reason.as_str() {
                "usage-reset" => self.checks.reset_enabled,
                "sensor-unresponsive" | "monitor-fault" => self.stop_on_failure(),
                "usage-threshold" => {
                    incident.threshold_details.retain(|receipt| {
                        if receipt.is_zero_quota() {
                            self.checks.zero_quota_enabled
                        } else {
                            self.checks.threshold_enabled
                        }
                    });
                    if !incident.threshold_details.is_empty() {
                        incident.target_sessions = Some(
                            incident
                                .threshold_details
                                .iter()
                                .map(|receipt| receipt.session_digest.clone())
                                .collect(),
                        );
                    }
                    !incident.threshold_details.is_empty()
                }
                _ => false,
            };
        if !allowed {
            state.suppress_pending();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage_monitor::threshold::Receipt;
    use hive_core::usage_guard::UsageWindow;

    fn selected() -> Preferences {
        Preferences {
            checks: MidwayPreferences {
                enabled: true,
                ..Default::default()
            },
            popup_enabled: false,
        }
    }

    fn receipt(session: &str, zero: bool) -> Receipt {
        Receipt {
            session_digest: session.to_owned(),
            owner_digest: session.to_owned(),
            target_digest: "target".to_owned(),
            policy_digest: "policy".to_owned(),
            control_digest: "absent".to_owned(),
            account_digest: "account".to_owned(),
            quota_pool: None,
            window: UsageWindow::Session,
            remaining_percent: 0.0,
            threshold_percent: 0,
            confirmed: false,
            turn_digest: Some("turn".to_owned()),
            zero_quota: Some(zero),
        }
    }

    #[test]
    fn disabled_reset_is_preserved_as_history_and_cannot_replay_after_reenable() {
        let mut prefs = selected();
        let mut state = MonitorState::new();
        state.halt("usage-reset", 100);
        let original = state.incident.as_ref().unwrap().digest.clone();
        prefs.filter_pending(&mut state);
        let incident = state.incident.as_ref().unwrap();
        assert_eq!(incident.digest, original);
        assert!(incident.policy_suppressed);
        assert_eq!(incident.stop_dispatched_at, None);
        prefs.checks.reset_enabled = true;
        prefs.filter_pending(&mut state);
        assert!(state.incident.as_ref().unwrap().policy_suppressed);
        state.halt("usage-reset", 101);
        prefs.filter_pending(&mut state);
        assert!(!state.incident.as_ref().unwrap().policy_suppressed);
    }

    #[test]
    fn zero_and_zero_percent_threshold_remain_distinct_choices() {
        for zero_enabled in [true, false] {
            let mut prefs = selected();
            prefs.checks.zero_quota_enabled = zero_enabled;
            prefs.checks.threshold_enabled = !zero_enabled;
            let mut state = MonitorState::new();
            state.halt("usage-threshold", 100);
            state.threshold_receipts = vec![receipt("threshold", false), receipt("zero", true)];
            state.incident.as_mut().unwrap().threshold_details = state.threshold_receipts.clone();
            prefs.filter_pending(&mut state);
            let incident = state.incident.as_ref().unwrap();
            assert!(!incident.policy_suppressed);
            assert_eq!(incident.threshold_details.len(), 1);
            assert_eq!(incident.threshold_details[0].is_zero_quota(), zero_enabled);
            assert_eq!(
                incident.target_sessions.as_ref().unwrap(),
                &vec![if zero_enabled { "zero" } else { "threshold" }.to_owned()]
            );
            assert_eq!(state.threshold_receipts.len(), 1);
        }
    }

    #[test]
    fn warning_failures_never_claim_a_stop_and_recovery_rearms_the_episode() {
        let mut state = MonitorState::new();
        state.observation_warning(100);
        let digest = state.incident.as_ref().unwrap().digest.clone();
        assert!(state.incident.as_ref().unwrap().policy_suppressed);
        assert_eq!(state.sensor_outage_stopped_at, None);
        assert_eq!(state.observation_outage_noticed_at, Some(100));
        state.observation_warning(200);
        assert_eq!(state.incident.as_ref().unwrap().digest, digest);
        let mut prefs = selected();
        for reason in ["sensor-unresponsive", "monitor-fault"] {
            state.halt(reason, 300);
            prefs.filter_pending(&mut state);
            assert!(state.incident.as_ref().unwrap().policy_suppressed);
        }
        prefs.checks.observation_failure = MidwayFailure::Stop;
        state.halt("sensor-unresponsive", 400);
        prefs.filter_pending(&mut state);
        assert!(!state.incident.as_ref().unwrap().policy_suppressed);
    }
}
