use crate::run::AdapterError;
use hive_core::sha256_digest;
use hive_core::usage_guard::{detect_usage_reset, UsageSnapshot};
use serde::{Deserialize, Serialize};

pub(super) const POLL_SECONDS: u64 = 15;
pub(super) const BLIND_SECONDS: u64 = 60;

/// Monotonic controller deadlines, independent of the sensor worker and wall clock.
pub(super) struct Cadence {
    pub next_sample: std::time::Duration,
    last_good: std::time::Duration,
    prior_gap: std::time::Duration,
}
impl Cadence {
    pub fn new() -> Self {
        Self {
            next_sample: std::time::Duration::ZERO,
            last_good: std::time::Duration::ZERO,
            prior_gap: std::time::Duration::ZERO,
        }
    }
    pub fn with_gap(gap: std::time::Duration) -> Self {
        Self {
            prior_gap: gap,
            ..Self::new()
        }
    }
    pub fn begin(&mut self, at: std::time::Duration) {
        self.next_sample = at + std::time::Duration::from_secs(POLL_SECONDS);
    }
    pub fn complete(&mut self, at: std::time::Duration, success: bool) {
        if success {
            self.last_good = at;
            self.prior_gap = std::time::Duration::ZERO;
        } else {
            self.next_sample = at + std::time::Duration::from_secs(2);
        }
    }
    pub fn idle(&mut self, at: std::time::Duration) {
        self.last_good = at;
        self.prior_gap = std::time::Duration::ZERO;
        self.next_sample = at;
    }
    pub fn blind(&self, at: std::time::Duration) -> bool {
        at.saturating_sub(self.last_good)
            .saturating_add(self.prior_gap)
            .as_secs()
            >= BLIND_SECONDS
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MonitorState {
    pub schema_version: u8,
    pub generation: u64,
    pub account_digest: Option<String>,
    pub baseline: Vec<UsageSnapshot>,
    pub last_success: Option<u64>,
    pub heartbeat: u64,
    pub incident: Option<Incident>,
    #[serde(default)]
    pub observation_count: u64,
    #[serde(default)]
    pub observation_watch_started_at: Option<u64>,
    #[serde(default)]
    pub last_query_error: Option<String>,
    #[serde(default)]
    pub query_elapsed_ms: Option<u64>,
    #[serde(default)]
    pub control_connected: bool,
    #[serde(default)]
    pub control_stopping: bool,
    #[serde(default)]
    pub active_turns: u64,
    #[serde(default)]
    pub activity_observed_at: Option<u64>,
    #[serde(default)]
    pub known_sessions: u64,
    #[serde(default)]
    pub unknown_sessions: u64,
    #[serde(default)]
    pub app_alive: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Incident {
    pub digest: String,
    pub reason: String,
    pub detected_at: u64,
    pub stop_method: String,
    // Read-only compatibility for earlier incidents. No app-termination action exists.
    pub termination_verified: bool,
    #[serde(default)]
    pub termination_elapsed_ms: Option<u64>,
    #[serde(default)]
    pub targeted_processes: Option<u64>,
    #[serde(default)]
    pub last_verified_scope: Option<String>,
    #[serde(default)]
    pub confirmed_termination: Option<TerminationReceipt>,
    pub notification: String,
    #[serde(default)]
    pub stop_confirmed: bool,
    #[serde(default)]
    pub stop_error: Option<String>,
    #[serde(default)]
    pub targeted_turns: u64,
    #[serde(default)]
    pub confirmed_turns: u64,
    #[serde(default)]
    pub confirmed_stop: Option<SessionStopReceipt>,
    #[serde(default)]
    pub last_verified_session_scope: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SessionStopReceipt {
    pub completed_at: u64,
    pub scope_digest: String,
    pub targeted_turns: u64,
    pub elapsed_ms: u64,
    pub request_ms: u64,
    pub app_alive: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TerminationReceipt {
    pub completed_at: u64,
    pub scope_digest: String,
    pub targeted_processes: u64,
    pub elapsed_ms: u64,
}

impl Incident {
    pub fn record_session_stop(&mut self, packet: &serde_json::Value, at: u64) {
        self.stop_method = "desktop-session-interrupt".into();
        self.targeted_turns = packet["targeted"].as_u64().unwrap_or_default();
        self.confirmed_turns = packet["confirmed"].as_u64().unwrap_or_default();
        let elapsed = packet["elapsed_ms"].as_u64().unwrap_or(u64::MAX);
        let request = packet["request_ms"].as_u64().unwrap_or(u64::MAX);
        let scope = packet["scope_digest"].as_str().unwrap_or_default();
        let scope_valid = scope.len() == 71
            && scope.starts_with("sha256:")
            && scope[7..].bytes().all(|b| b.is_ascii_hexdigit());
        let verified = scope_valid
            && packet["verified"] == true
            && self.targeted_turns > 0
            && self.targeted_turns == self.confirmed_turns
            && packet["app_alive"] == true
            && packet["unknown_sessions"] == 0
            && packet["active_turns"] == 0;
        self.stop_confirmed = verified
            && (elapsed <= 5000 && request <= 2000
                || self
                    .last_verified_session_scope
                    .as_ref()
                    .is_some_and(|saved| saved == scope));
        self.stop_error = if self.stop_confirmed {
            None
        } else if elapsed > 5000 && packet["unknown_sessions"].as_u64().unwrap_or(1) > 0 {
            Some("session-scope-unverified".into())
        } else if request > 2000 && elapsed > 2000 {
            Some("session-stop-request-late".into())
        } else if elapsed > 5000 {
            Some("session-stop-unconfirmed".into())
        } else {
            None
        };
        if verified && elapsed <= 5000 && request <= 2000 {
            self.last_verified_session_scope = Some(scope.into());
            self.confirmed_stop.get_or_insert(SessionStopReceipt {
                completed_at: at,
                scope_digest: scope.into(),
                targeted_turns: self.targeted_turns,
                elapsed_ms: elapsed,
                request_ms: request,
                app_alive: true,
            });
        }
    }
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            ..Self::default()
        }
    }

    pub fn observation_gap(&mut self, active: bool, at: u64) -> std::time::Duration {
        if !active {
            self.observation_watch_started_at = None;
            return std::time::Duration::ZERO;
        }
        let started = *self.observation_watch_started_at.get_or_insert(at);
        let anchor = self
            .last_success
            .filter(|last| *last >= started)
            .unwrap_or(started);
        if anchor > at {
            return std::time::Duration::from_secs(BLIND_SECONDS);
        }
        std::time::Duration::from_secs(at - anchor)
    }

    pub fn halt(&mut self, reason: &str, now: u64) {
        if self.incident.is_some() {
            return;
        }
        self.generation = self.generation.saturating_add(1);
        let digest = sha256_digest(
            format!(
                "{}:{reason}:{now}:{:?}",
                self.generation, self.account_digest
            )
            .as_bytes(),
        );
        self.incident = Some(Incident {
            digest,
            reason: reason.to_owned(),
            detected_at: now,
            stop_method: "pending".to_owned(),
            termination_verified: false,
            termination_elapsed_ms: None,
            targeted_processes: None,
            last_verified_scope: None,
            confirmed_termination: None,
            notification: "pending".to_owned(),
            stop_confirmed: false,
            stop_error: None,
            targeted_turns: 0,
            confirmed_turns: 0,
            confirmed_stop: None,
            last_verified_session_scope: None,
        });
    }

    /// A baseline is historical evidence, not a reusable permission to run.
    pub fn observe(&mut self, current: &[UsageSnapshot], now: u64) -> Result<(), AdapterError> {
        let first = current
            .first()
            .ok_or_else(|| invalid("empty observation"))?;
        if current.len() > 8
            || first.account_scope_digest.len() != 71
            || !first.account_scope_digest.starts_with("sha256:")
            || !first.account_scope_digest[7..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err(invalid("invalid observation identity"));
        }
        for (index, item) in current.iter().enumerate() {
            if item.host_scope != "codex"
                || item.account_scope_digest != first.account_scope_digest
                || !item.remaining_percent.is_finite()
                || !(0.0..=100.0).contains(&item.remaining_percent)
                || item.measured_at_unix_seconds < 0
                || item.measured_at_unix_seconds > i64::try_from(now).unwrap_or(i64::MAX)
                || i64::try_from(now).unwrap_or(i64::MAX) - item.measured_at_unix_seconds > 5
                || item.expires_at_unix_seconds <= i64::try_from(now).unwrap_or(i64::MAX)
                || item.resets_at_unix_seconds <= item.measured_at_unix_seconds
                || current[..index]
                    .iter()
                    .any(|other| same_window(item, other))
            {
                return Err(invalid("stale or invalid usage observation"));
            }
        }
        if self
            .account_digest
            .as_ref()
            .is_some_and(|account| account != &first.account_scope_digest)
        {
            return Err(invalid(
                "usage account changed; explicit acknowledgement required",
            ));
        }
        if !self.baseline.is_empty() {
            if current.len() != self.baseline.len()
                || current
                    .iter()
                    .any(|item| !self.baseline.iter().any(|old| same_window(item, old)))
            {
                return Err(invalid("usage window set changed"));
            }
            if current.iter().any(|item| {
                self.baseline.iter().any(|old| {
                    same_window(item, old)
                        && (item.measured_at_unix_seconds < old.measured_at_unix_seconds
                            || item.resets_at_unix_seconds.saturating_add(2)
                                < old.resets_at_unix_seconds)
                })
            }) {
                return Err(invalid("usage observation moved backwards"));
            }
            // A provider-confirmed new period also detects a refill hidden by fast consumption.
            let rollover = current.iter().any(|item| {
                self.baseline.iter().any(|old| {
                    same_window(item, old)
                        && item.measured_at_unix_seconds >= old.resets_at_unix_seconds
                        && item.resets_at_unix_seconds
                            > old.resets_at_unix_seconds.saturating_add(2)
                })
            });
            if rollover || detect_usage_reset(current, &self.baseline).is_some() {
                self.halt("usage-reset", now);
            }
        }
        self.account_digest = Some(first.account_scope_digest.clone());
        let mut next = current.to_vec();
        for item in &mut next {
            if let Some(old) = self.baseline.iter().find(|old| same_window(item, old)) {
                // Native reset timestamps are rounded from a countdown and can move by a second.
                // Keep the period anchor stable; this never suppresses a remaining-quota increase.
                if item
                    .resets_at_unix_seconds
                    .abs_diff(old.resets_at_unix_seconds)
                    <= 2
                {
                    item.resets_at_unix_seconds = old.resets_at_unix_seconds;
                }
            }
        }
        self.baseline = next;
        self.last_success = Some(now);
        self.observation_count = self.observation_count.saturating_add(1);
        Ok(())
    }

    pub fn acknowledge(
        &mut self,
        expected: &str,
        current: &[UsageSnapshot],
        now: u64,
    ) -> Result<(), AdapterError> {
        if self.incident.as_ref().map(|i| i.digest.as_str()) != Some(expected) {
            return Err(invalid("confirmation does not match the pending incident"));
        }
        // Account changes can only be accepted at this explicit incident boundary.
        let mut fresh = Self::new();
        fresh.observe(current, now)?;
        fresh.generation = self.generation;
        fresh.heartbeat = self.heartbeat;
        fresh.control_connected = self.control_connected;
        fresh.control_stopping = self.control_stopping;
        fresh.app_alive = self.app_alive;
        fresh.active_turns = self.active_turns;
        fresh.activity_observed_at = self.activity_observed_at;
        fresh.known_sessions = self.known_sessions;
        fresh.unknown_sessions = self.unknown_sessions;
        *self = fresh;
        Ok(())
    }
}

fn same_window(a: &UsageSnapshot, b: &UsageSnapshot) -> bool {
    a.host_scope == b.host_scope
        && a.account_scope_digest == b.account_scope_digest
        && a.quota_pool == b.quota_pool
        && a.quota_window == b.quota_window
}

fn invalid(message: &str) -> AdapterError {
    AdapterError::Safety(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::usage_guard::{SourceConfidence, UsageWindow};
    fn sample(remaining: f64, now: u64, reset: i64) -> Vec<UsageSnapshot> {
        vec![UsageSnapshot {
            schema_version: 1,
            sensor_id: "codex-app-server".into(),
            sensor_version: "0.160.1".into(),
            host_scope: "codex".into(),
            account_scope_digest: format!("sha256:{}", "a".repeat(64)),
            quota_pool: None,
            quota_window: UsageWindow::Weekly,
            remaining_percent: remaining,
            measured_at_unix_seconds: i64::try_from(now).unwrap(),
            expires_at_unix_seconds: i64::try_from(now).unwrap() + 60,
            resets_at_unix_seconds: reset,
            source_confidence: SourceConfidence::High,
        }]
    }
    #[test]
    fn session_stop_requires_actual_interruption_app_survival_and_bounded_confirmation() {
        let mut state = MonitorState::new();
        state.halt("usage-reset", 100);
        let incident = state.incident.as_mut().unwrap();
        let mut packet = serde_json::json!({"scope_digest":format!("sha256:{}","a".repeat(64)),"targeted":2,"confirmed":1,"verified":true,"elapsed_ms":100,"request_ms":10,"app_alive":true,"unknown_sessions":0,"active_turns":0});
        incident.record_session_stop(&packet, 101);
        assert!(!incident.stop_confirmed);
        packet["confirmed"] = serde_json::json!(2);
        packet["app_alive"] = serde_json::json!(false);
        incident.record_session_stop(&packet, 101);
        assert!(!incident.stop_confirmed);
        packet["app_alive"] = serde_json::json!(true);
        packet["elapsed_ms"] = serde_json::json!(5001);
        incident.record_session_stop(&packet, 106);
        assert!(!incident.stop_confirmed);
        assert!(incident.confirmed_stop.is_none());
        packet["unknown_sessions"] = serde_json::json!(1);
        incident.record_session_stop(&packet, 106);
        assert_eq!(
            incident.stop_error.as_deref(),
            Some("session-scope-unverified")
        );
        assert!(!incident.stop_confirmed);
        packet["unknown_sessions"] = serde_json::json!(0);
        packet["elapsed_ms"] = serde_json::json!(200);
        incident.record_session_stop(&packet, 101);
        assert!(incident.stop_confirmed);
        packet["elapsed_ms"] = serde_json::json!(6000);
        incident.record_session_stop(&packet, 107);
        assert!(incident.stop_confirmed);
        packet["scope_digest"] = serde_json::json!(format!("sha256:{}", "b".repeat(64)));
        incident.record_session_stop(&packet, 108);
        assert!(!incident.stop_confirmed);
        assert_eq!(incident.confirmed_stop.as_ref().unwrap().elapsed_ms, 200);
        packet["targeted"] = serde_json::json!(0);
        packet["confirmed"] = serde_json::json!(0);
        incident.record_session_stop(&packet, 109);
        assert!(!incident.stop_confirmed);
    }
    #[test]
    fn countdown_rounding_is_fresh_but_cannot_drift_the_period_anchor() {
        let mut state = MonitorState::new();
        state.observe(&sample(80.0, 100, 200), 100).unwrap();
        for (measured, reset) in [(115, 199), (130, 201), (145, 198)] {
            state
                .observe(&sample(70.0, measured, reset), measured)
                .unwrap();
            assert_eq!(state.baseline[0].resets_at_unix_seconds, 200);
            assert!(state.incident.is_none());
        }
        assert_eq!(state.last_success, Some(145));
        assert!(state.observe(&sample(60.0, 160, 197), 160).is_err());
        state.observe(&sample(99.0, 160, 199), 160).unwrap();
        assert_eq!(state.incident.unwrap().reason, "usage-reset");
    }
    #[test]
    fn first_baseline_then_refill_latches_across_restart() {
        let mut s = MonitorState::new();
        s.observe(&sample(1.0, 100, 200), 100).unwrap();
        assert!(s.incident.is_none());
        s.observe(&sample(99.0, 115, 300), 115).unwrap();
        let saved = serde_json::to_vec(&s).unwrap();
        let mut s: MonitorState = serde_json::from_slice(&saved).unwrap();
        let digest = s.incident.as_ref().unwrap().digest.clone();
        s.observe(&sample(98.0, 130, 300), 130).unwrap();
        assert_eq!(s.incident.as_ref().unwrap().digest, digest);
        assert!(s
            .acknowledge("wrong", &sample(98.0, 130, 300), 130)
            .is_err());
        s.acknowledge(&digest, &sample(98.0, 130, 300), 130)
            .unwrap();
        assert!(s.incident.is_none());
    }
    #[test]
    fn rollover_detects_reset_even_when_new_quota_already_spent() {
        let mut s = MonitorState::new();
        s.observe(&sample(80.0, 100, 110), 100).unwrap();
        s.observe(&sample(70.0, 115, 300), 115).unwrap();
        assert!(s.incident.is_some());
    }
    #[test]
    fn clock_alone_does_not_stop_or_advance_baseline() {
        let mut s = MonitorState::new();
        s.observe(&sample(80.0, 100, 110), 100).unwrap();
        assert!(s.observe(&sample(80.0, 100, 110), 120).is_err());
        assert!(s.incident.is_none());
        assert_eq!(s.last_success, Some(100));
    }
    #[test]
    fn account_change_and_missing_duplicate_and_stale_windows_are_not_success() {
        let mut s = MonitorState::new();
        s.observe(&sample(80.0, 100, 200), 100).unwrap();
        let mut next = sample(70.0, 115, 200);
        next[0].account_scope_digest = "sha256:other".into();
        assert!(s.observe(&next, 115).is_err());
        assert!(s.observe(&[], 115).is_err());
        let mut duplicate = sample(70.0, 115, 200);
        duplicate.push(duplicate[0].clone());
        assert!(s.observe(&duplicate, 115).is_err());
        assert!(s.observe(&sample(70.0, 100, 200), 115).is_err());
        assert_eq!(s.last_success, Some(100));
    }
    #[test]
    fn incident_creation_is_idempotent() {
        let mut s = MonitorState::new();
        s.halt("sensor-unresponsive", 130);
        let d = s.incident.clone().unwrap().digest;
        s.halt("usage-reset", 140);
        assert_eq!(s.generation, 1);
        assert_eq!(s.incident.unwrap().digest, d);
    }
    #[test]
    fn slow_query_does_not_add_another_fifteen_seconds() {
        use std::time::Duration as D;
        let mut c = Cadence::new();
        c.begin(D::ZERO);
        c.complete(D::from_secs(4), true);
        assert_eq!(c.next_sample, D::from_secs(15));
        c.begin(D::from_secs(15));
        c.complete(D::from_secs(20), false);
        assert_eq!(c.next_sample, D::from_secs(22));
        assert!(!c.blind(D::from_millis(63_999)));
        assert!(c.blind(D::from_secs(64)));
        // A permanently unresponsive worker never advances the independent blind deadline.
        let mut c = Cadence::new();
        c.begin(D::ZERO);
        assert!(!c.blind(D::from_millis(59_999)));
        assert!(c.blind(D::from_mins(1)));
    }
    #[test]
    fn failed_retries_do_not_extend_blind_deadline_but_fresh_success_does() {
        use std::time::Duration as D;
        let mut c = Cadence::new();
        for at in [0, 7, 14, 21, 28, 35, 42, 49, 56] {
            c.begin(D::from_secs(at));
            c.complete(D::from_secs(at + 5), false);
        }
        assert!(c.blind(D::from_secs(61)));
        c.complete(D::from_secs(62), true);
        assert!(!c.blind(D::from_millis(121_999)));
        assert!(c.blind(D::from_secs(122)));
    }
    #[test]
    fn invalid_observation_cannot_postpone_protective_stop() {
        use std::time::Duration as D;
        let mut c = Cadence::new();
        let mut state = MonitorState::new();
        state.observe(&sample(80.0, 100, 300), 100).unwrap();
        c.complete(D::from_secs(1), true);
        for at in [116, 131, 146, 160] {
            let accepted = state.observe(&sample(70.0, 100, 300), at).is_ok();
            assert!(!accepted);
            c.complete(D::from_secs(at - 100), accepted);
        }
        assert!(c.blind(D::from_secs(61)));
        state.halt("sensor-unresponsive", 161);
        assert_eq!(state.incident.unwrap().reason, "sensor-unresponsive");
    }
    #[test]
    fn controller_restart_preserves_the_last_normal_observation_deadline() {
        use std::time::Duration as D;
        let mut state = MonitorState::new();
        assert_eq!(state.observation_gap(true, 100), D::ZERO);
        state.last_success = Some(100);
        let bytes = serde_json::to_vec(&state).unwrap();
        let mut restarted: MonitorState = serde_json::from_slice(&bytes).unwrap();
        let cadence = Cadence::with_gap(restarted.observation_gap(true, 155));
        assert!(!cadence.blind(D::from_millis(4_999)));
        assert!(cadence.blind(D::from_secs(5)));
        restarted.last_success = Some(156);
        assert_eq!(restarted.observation_gap(true, 156), D::ZERO);
    }
    #[test]
    fn restart_before_first_success_also_preserves_the_watch_deadline() {
        use std::time::Duration as D;
        let mut state = MonitorState::new();
        state.last_success = Some(10);
        assert_eq!(state.observation_gap(true, 100), D::ZERO);
        let bytes = serde_json::to_vec(&state).unwrap();
        let mut restarted: MonitorState = serde_json::from_slice(&bytes).unwrap();
        let cadence = Cadence::with_gap(restarted.observation_gap(true, 159));
        assert!(cadence.blind(D::from_secs(1)));
        assert_eq!(restarted.observation_gap(false, 200), D::ZERO);
        assert_eq!(restarted.observation_gap(true, 300), D::ZERO);
        restarted.last_success = Some(301);
        assert!(Cadence::with_gap(restarted.observation_gap(true, 300)).blind(D::ZERO));
    }
}
