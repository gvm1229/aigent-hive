use crate::run::AdapterError;
use hive_core::sha256_digest;
use hive_core::usage_guard::{detect_usage_reset, UsageSnapshot};
use serde::{Deserialize, Serialize};

pub(super) const POLL_SECONDS: u64 = 15;
pub(super) const BLIND_SECONDS: u64 = 30;

/// Monotonic controller deadlines, independent of the sensor worker and wall clock.
pub(super) struct Cadence {
    pub next_sample: std::time::Duration,
    last_good: std::time::Duration,
}
impl Cadence {
    pub fn new() -> Self {
        Self {
            next_sample: std::time::Duration::ZERO,
            last_good: std::time::Duration::ZERO,
        }
    }
    pub fn begin(&mut self, at: std::time::Duration) {
        self.next_sample = at + std::time::Duration::from_secs(POLL_SECONDS);
    }
    pub fn complete(&mut self, at: std::time::Duration, success: bool) {
        if success {
            self.last_good = at;
        } else {
            self.next_sample = at + std::time::Duration::from_secs(2);
        }
    }
    pub fn idle(&mut self, at: std::time::Duration) {
        self.last_good = at;
        self.next_sample = at;
    }
    pub fn blind(&self, at: std::time::Duration) -> bool {
        at.saturating_sub(self.last_good).as_secs() >= BLIND_SECONDS
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Incident {
    pub digest: String,
    pub reason: String,
    pub detected_at: u64,
    pub stop_method: String,
    pub termination_verified: bool,
    #[serde(default)]
    pub termination_elapsed_ms: Option<u64>,
    #[serde(default)]
    pub targeted_processes: Option<u64>,
    pub notification: String,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            ..Self::default()
        }
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
            notification: "pending".to_owned(),
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
                            || item.resets_at_unix_seconds < old.resets_at_unix_seconds)
                })
            }) {
                return Err(invalid("usage observation moved backwards"));
            }
            // A provider-confirmed new period also detects a refill hidden by fast consumption.
            let rollover = current.iter().any(|item| {
                self.baseline.iter().any(|old| {
                    same_window(item, old)
                        && item.measured_at_unix_seconds >= old.resets_at_unix_seconds
                        && item.resets_at_unix_seconds > old.resets_at_unix_seconds
                })
            });
            if rollover || detect_usage_reset(current, &self.baseline).is_some() {
                self.halt("usage-reset", now);
            }
        }
        self.account_digest = Some(first.account_scope_digest.clone());
        self.baseline = current.to_vec();
        self.last_success = Some(now);
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
        assert!(!c.blind(D::from_secs(33)));
        assert!(c.blind(D::from_secs(34)));
        // A permanently unresponsive worker never advances the independent blind deadline.
        let mut c = Cadence::new();
        c.begin(D::ZERO);
        assert!(c.blind(D::from_secs(30)));
    }
}
