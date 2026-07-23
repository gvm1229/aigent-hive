//! Provider-neutral subscription usage policy evaluation.

use serde::{Deserialize, Serialize};

/// Remaining percentage at or below which automatic work must stop.
pub const DEFAULT_STOP_THRESHOLD_PERCENT: f64 = 10.0;
/// Maximum accepted measurement clock skew into the future.
pub const DEFAULT_MAX_FUTURE_SKEW_SECONDS: i64 = 5;
/// Maximum lifetime of a permit after evaluation.
pub const DEFAULT_PERMIT_DEADLINE_SECONDS: i64 = 5;

/// A quota window required by the usage policy.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageWindow {
    /// The host's short session or rolling window.
    Session,
    /// The host's weekly quota window.
    Weekly,
}

/// Whether a local sensor asserts that a measurement is suitable for enforcement.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceConfidence {
    /// The sensor considers the measurement trustworthy for enforcement.
    High,
    /// The sensor has partial confidence, which is insufficient for enforcement.
    Medium,
    /// The sensor cannot establish a trustworthy measurement.
    Low,
}

/// A provider-neutral quota measurement from a side-effect-free local sensor.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct UsageSnapshot {
    /// Snapshot contract version. Only version 1 is currently accepted.
    pub schema_version: u32,
    /// Stable identifier of the local sensor implementation.
    pub sensor_id: String,
    /// Version reported by the local sensor implementation.
    pub sensor_version: String,
    /// Subscription host to which this measurement applies.
    pub host_scope: String,
    /// Non-reversible digest identifying the measured subscription account.
    pub account_scope_digest: String,
    /// Quota window represented by this measurement.
    pub quota_window: UsageWindow,
    /// Remaining quota percentage in the inclusive range `0..=100`.
    pub remaining_percent: f64,
    /// Unix timestamp at which the measurement was taken.
    pub measured_at_unix_seconds: i64,
    /// Unix timestamp after which the measurement must not be used.
    pub expires_at_unix_seconds: i64,
    /// Unix timestamp of the quota window reset.
    pub resets_at_unix_seconds: i64,
    /// Sensor confidence asserted for this measurement.
    pub source_confidence: SourceConfidence,
}

/// Fixed provider-neutral requirements for automatic usage enforcement.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UsagePolicy {
    required_sensor_id: String,
    required_sensor_version: String,
    required_host_scope: String,
    required_account_scope_digest: String,
    stop_threshold_percent: u8,
}

impl UsagePolicy {
    /// Create the default fail-closed policy for a sensor, host, and account.
    #[must_use]
    pub fn new(
        sensor_id: impl Into<String>,
        sensor_version: impl Into<String>,
        host_scope: impl Into<String>,
        account_scope_digest: impl Into<String>,
    ) -> Self {
        Self {
            required_sensor_id: sensor_id.into(),
            required_sensor_version: sensor_version.into(),
            required_host_scope: host_scope.into(),
            required_account_scope_digest: account_scope_digest.into(),
            stop_threshold_percent: 10,
        }
    }

    /// Override the stop threshold with a setup-approved percentage.
    ///
    /// # Errors
    ///
    /// Returns [`UsagePolicyError::StopThresholdOutOfRange`] unless the
    /// percentage is in the inclusive range `1..=99`.
    pub fn with_stop_remaining_percent(
        mut self,
        stop_threshold_percent: u8,
    ) -> Result<Self, UsagePolicyError> {
        if !(1..=99).contains(&stop_threshold_percent) {
            return Err(UsagePolicyError::StopThresholdOutOfRange {
                value: stop_threshold_percent,
            });
        }
        self.stop_threshold_percent = stop_threshold_percent;
        Ok(self)
    }

    /// Sensor identifier required by this policy.
    #[must_use]
    pub fn required_sensor_id(&self) -> &str {
        &self.required_sensor_id
    }

    /// Sensor version required by this policy.
    #[must_use]
    pub fn required_sensor_version(&self) -> &str {
        &self.required_sensor_version
    }

    /// Host scope required by this policy.
    #[must_use]
    pub fn required_host_scope(&self) -> &str {
        &self.required_host_scope
    }

    /// Account digest required by this policy.
    #[must_use]
    pub fn required_account_scope_digest(&self) -> &str {
        &self.required_account_scope_digest
    }

    /// Remaining percentage that blocks new automatic work.
    #[must_use]
    pub fn stop_threshold_percent(&self) -> f64 {
        f64::from(self.stop_threshold_percent)
    }

    /// Maximum accepted measurement skew into the future.
    #[must_use]
    pub const fn max_future_skew_seconds(&self) -> i64 {
        DEFAULT_MAX_FUTURE_SKEW_SECONDS
    }

    /// Maximum permit lifetime after evaluation.
    #[must_use]
    pub const fn permit_deadline_seconds(&self) -> i64 {
        DEFAULT_PERMIT_DEADLINE_SECONDS
    }
}

/// Invalid policy configuration rejected before usage evaluation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UsagePolicyError {
    /// The setup-approved stop threshold must be in `1..=99`.
    StopThresholdOutOfRange { value: u8 },
}

/// A typed reason that automatic continuation cannot safely be decided.
#[derive(Debug, Clone, PartialEq)]
pub enum UsageUnknownReason {
    /// The snapshot schema is not the supported version.
    UnsupportedSchemaVersion { actual: u32 },
    /// Neither the preferred session nor fallback weekly window is available.
    MissingWindow { window: UsageWindow },
    /// A quota window has more than one current measurement.
    DuplicateWindow { window: UsageWindow },
    /// The measurement came from a different sensor.
    SensorIdMismatch { expected: String, actual: String },
    /// The measurement came from an unapproved sensor version.
    SensorVersionMismatch { expected: String, actual: String },
    /// The measurement applies to a different host.
    HostScopeMismatch { expected: String, actual: String },
    /// The measurement applies to a different account.
    AccountScopeDigestMismatch { expected: String, actual: String },
    /// The sensor did not assert enforcement-grade confidence.
    UntrustedSource { window: UsageWindow },
    /// The remaining percentage was NaN or infinite.
    NonFiniteRemaining { window: UsageWindow },
    /// The remaining percentage was outside `0..=100`.
    RemainingOutOfRange {
        window: UsageWindow,
        remaining_percent: f64,
    },
    /// The measurement had already expired at evaluation time.
    StaleMeasurement {
        window: UsageWindow,
        expires_at_unix_seconds: i64,
        evaluated_at_unix_seconds: i64,
    },
    /// The measurement was farther in the future than the allowed clock skew.
    FutureMeasurement {
        window: UsageWindow,
        measured_at_unix_seconds: i64,
        evaluated_at_unix_seconds: i64,
    },
    /// The sensor's measurement timestamp moved backward.
    MeasurementRegression {
        window: UsageWindow,
        previous_unix_seconds: i64,
        current_unix_seconds: i64,
    },
    /// The sensor's reset timestamp moved backward.
    ResetRegression {
        window: UsageWindow,
        previous_unix_seconds: i64,
        current_unix_seconds: i64,
    },
    /// Remaining quota increased without a new reset.
    SameResetRemainingIncrease {
        window: UsageWindow,
        previous_percent: f64,
        current_percent: f64,
        reset_at_unix_seconds: i64,
    },
}

/// Details of a valid measurement that blocks automatic work.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UsageBlock {
    /// Window whose remaining quota reached the threshold.
    pub window: UsageWindow,
    /// Valid measured remaining percentage.
    pub remaining_percent: f64,
    /// Inclusive stop threshold applied by the policy.
    pub threshold_percent: f64,
}

/// A non-cloneable, one-shot authorization for one automatic dispatch.
#[derive(Debug)]
pub struct UsagePermit {
    expires_at_unix_seconds: i64,
    consumed: bool,
}

impl UsagePermit {
    /// Unix timestamp at which this permit expires.
    #[must_use]
    pub const fn expires_at_unix_seconds(&self) -> i64 {
        self.expires_at_unix_seconds
    }

    /// Consume the permit for exactly one automatic dispatch.
    ///
    /// # Errors
    ///
    /// Returns [`UsagePermitError::AlreadyConsumed`] after a successful use,
    /// or [`UsagePermitError::Expired`] at or after the permit deadline.
    pub fn consume(&mut self, now_unix_seconds: i64) -> Result<(), UsagePermitError> {
        if self.consumed {
            return Err(UsagePermitError::AlreadyConsumed);
        }
        if now_unix_seconds >= self.expires_at_unix_seconds {
            return Err(UsagePermitError::Expired {
                expires_at_unix_seconds: self.expires_at_unix_seconds,
                attempted_at_unix_seconds: now_unix_seconds,
            });
        }
        self.consumed = true;
        Ok(())
    }
}

/// Failure to consume a one-shot usage permit.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UsagePermitError {
    /// The permit already authorized a dispatch.
    AlreadyConsumed,
    /// The permit deadline passed before dispatch.
    Expired {
        expires_at_unix_seconds: i64,
        attempted_at_unix_seconds: i64,
    },
}

/// Result of evaluating the selected usage window.
#[derive(Debug)]
pub enum UsageDecision {
    /// Every currently applicable window is valid and above the stop threshold.
    Allow(UsagePermit),
    /// Every applicable window is valid, but at least one reached the threshold.
    Block(UsageBlock),
    /// Enforcement cannot safely determine whether a dispatch is allowed.
    Unknown(UsageUnknownReason),
}

/// Evaluate current snapshots against the fixed provider-neutral policy.
///
/// `previous_snapshots` should contain the last accepted measurement for each
/// available window. They are used only for monotonicity checks; an empty slice
/// is valid for the first observation. A session window takes precedence when
/// present; otherwise the weekly window is the required fallback.
#[must_use]
pub fn evaluate_usage(
    policy: &UsagePolicy,
    current_snapshots: &[UsageSnapshot],
    previous_snapshots: &[UsageSnapshot],
    now_unix_seconds: i64,
) -> UsageDecision {
    let mut current = Vec::with_capacity(2);
    for window in [UsageWindow::Session, UsageWindow::Weekly] {
        let mut matching = current_snapshots
            .iter()
            .filter(|snapshot| snapshot.quota_window == window);
        let snapshot = matching.next();
        if matching.next().is_some() {
            return UsageDecision::Unknown(UsageUnknownReason::DuplicateWindow { window });
        }
        if let Some(snapshot) = snapshot {
            current.push(snapshot);
        }
    }

    let Some(snapshot) = current
        .iter()
        .find(|snapshot| snapshot.quota_window == UsageWindow::Session)
        .or_else(|| {
            current
                .iter()
                .find(|snapshot| snapshot.quota_window == UsageWindow::Weekly)
        })
    else {
        return UsageDecision::Unknown(UsageUnknownReason::MissingWindow {
            window: UsageWindow::Weekly,
        });
    };

    if let Some(reason) = validate_snapshot(policy, snapshot, now_unix_seconds) {
        return UsageDecision::Unknown(reason);
    }

    let mut previous_matching = previous_snapshots
        .iter()
        .filter(|previous| previous.quota_window == snapshot.quota_window);
    if let Some(previous) = previous_matching.next() {
        if previous_matching.next().is_some() {
            return UsageDecision::Unknown(UsageUnknownReason::DuplicateWindow {
                window: snapshot.quota_window,
            });
        }
        if let Some(reason) = validate_monotonicity(snapshot, previous) {
            return UsageDecision::Unknown(reason);
        }
    }

    if snapshot.remaining_percent <= policy.stop_threshold_percent() {
        return UsageDecision::Block(UsageBlock {
            window: snapshot.quota_window,
            remaining_percent: snapshot.remaining_percent,
            threshold_percent: policy.stop_threshold_percent(),
        });
    }

    let policy_expiry = now_unix_seconds.saturating_add(policy.permit_deadline_seconds());
    UsageDecision::Allow(UsagePermit {
        expires_at_unix_seconds: snapshot.expires_at_unix_seconds.min(policy_expiry),
        consumed: false,
    })
}

fn validate_snapshot(
    policy: &UsagePolicy,
    snapshot: &UsageSnapshot,
    now_unix_seconds: i64,
) -> Option<UsageUnknownReason> {
    if snapshot.schema_version != 1 {
        return Some(UsageUnknownReason::UnsupportedSchemaVersion {
            actual: snapshot.schema_version,
        });
    }
    if snapshot.sensor_id != policy.required_sensor_id {
        return Some(UsageUnknownReason::SensorIdMismatch {
            expected: policy.required_sensor_id.clone(),
            actual: snapshot.sensor_id.clone(),
        });
    }
    if snapshot.sensor_version != policy.required_sensor_version {
        return Some(UsageUnknownReason::SensorVersionMismatch {
            expected: policy.required_sensor_version.clone(),
            actual: snapshot.sensor_version.clone(),
        });
    }
    if snapshot.host_scope != policy.required_host_scope {
        return Some(UsageUnknownReason::HostScopeMismatch {
            expected: policy.required_host_scope.clone(),
            actual: snapshot.host_scope.clone(),
        });
    }
    if snapshot.account_scope_digest != policy.required_account_scope_digest {
        return Some(UsageUnknownReason::AccountScopeDigestMismatch {
            expected: policy.required_account_scope_digest.clone(),
            actual: snapshot.account_scope_digest.clone(),
        });
    }
    if snapshot.source_confidence != SourceConfidence::High {
        return Some(UsageUnknownReason::UntrustedSource {
            window: snapshot.quota_window,
        });
    }
    if !snapshot.remaining_percent.is_finite() {
        return Some(UsageUnknownReason::NonFiniteRemaining {
            window: snapshot.quota_window,
        });
    }
    if !(0.0..=100.0).contains(&snapshot.remaining_percent) {
        return Some(UsageUnknownReason::RemainingOutOfRange {
            window: snapshot.quota_window,
            remaining_percent: snapshot.remaining_percent,
        });
    }
    if snapshot.expires_at_unix_seconds <= now_unix_seconds {
        return Some(UsageUnknownReason::StaleMeasurement {
            window: snapshot.quota_window,
            expires_at_unix_seconds: snapshot.expires_at_unix_seconds,
            evaluated_at_unix_seconds: now_unix_seconds,
        });
    }
    if snapshot.measured_at_unix_seconds
        > now_unix_seconds.saturating_add(policy.max_future_skew_seconds())
    {
        return Some(UsageUnknownReason::FutureMeasurement {
            window: snapshot.quota_window,
            measured_at_unix_seconds: snapshot.measured_at_unix_seconds,
            evaluated_at_unix_seconds: now_unix_seconds,
        });
    }
    None
}

fn validate_monotonicity(
    current: &UsageSnapshot,
    previous: &UsageSnapshot,
) -> Option<UsageUnknownReason> {
    if current.measured_at_unix_seconds < previous.measured_at_unix_seconds {
        return Some(UsageUnknownReason::MeasurementRegression {
            window: current.quota_window,
            previous_unix_seconds: previous.measured_at_unix_seconds,
            current_unix_seconds: current.measured_at_unix_seconds,
        });
    }
    if current.resets_at_unix_seconds < previous.resets_at_unix_seconds {
        return Some(UsageUnknownReason::ResetRegression {
            window: current.quota_window,
            previous_unix_seconds: previous.resets_at_unix_seconds,
            current_unix_seconds: current.resets_at_unix_seconds,
        });
    }
    if current.resets_at_unix_seconds == previous.resets_at_unix_seconds
        && current.remaining_percent > previous.remaining_percent
    {
        return Some(UsageUnknownReason::SameResetRemainingIncrease {
            window: current.quota_window,
            previous_percent: previous.remaining_percent,
            current_percent: current.remaining_percent,
            reset_at_unix_seconds: current.resets_at_unix_seconds,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_usage, SourceConfidence, UsageDecision, UsagePermitError, UsagePolicy,
        UsagePolicyError, UsageSnapshot, UsageUnknownReason, UsageWindow,
    };

    const NOW: i64 = 1_000;

    fn policy() -> UsagePolicy {
        UsagePolicy::new("local-sensor", "1.2.3", "codex", "sha256:account")
    }

    fn snapshot(window: UsageWindow, remaining_percent: f64) -> UsageSnapshot {
        UsageSnapshot {
            schema_version: 1,
            sensor_id: "local-sensor".to_owned(),
            sensor_version: "1.2.3".to_owned(),
            host_scope: "codex".to_owned(),
            account_scope_digest: "sha256:account".to_owned(),
            quota_window: window,
            remaining_percent,
            measured_at_unix_seconds: NOW,
            expires_at_unix_seconds: NOW + 30,
            resets_at_unix_seconds: NOW + 3_600,
            source_confidence: SourceConfidence::High,
        }
    }

    fn pair(session: f64, weekly: f64) -> Vec<UsageSnapshot> {
        vec![
            snapshot(UsageWindow::Session, session),
            snapshot(UsageWindow::Weekly, weekly),
        ]
    }

    fn assert_unknown(
        current: &[UsageSnapshot],
        previous: &[UsageSnapshot],
        expected: impl FnOnce(&UsageUnknownReason) -> bool,
    ) {
        let UsageDecision::Unknown(reason) = evaluate_usage(&policy(), current, previous, NOW)
        else {
            panic!("expected an unknown decision");
        };
        assert!(expected(&reason), "unexpected reason: {reason:?}");
    }

    #[test]
    fn threshold_is_inclusive_and_values_above_it_are_allowed() {
        for remaining in [9.99, 10.0] {
            let UsageDecision::Block(block) =
                evaluate_usage(&policy(), &pair(remaining, 57.0), &[], NOW)
            else {
                panic!("remaining {remaining} should block");
            };
            assert_eq!(block.remaining_percent.to_bits(), remaining.to_bits());
            assert_eq!(block.threshold_percent.to_bits(), 10.0_f64.to_bits());
        }

        for remaining in [10.01, 57.0] {
            assert!(
                matches!(
                    evaluate_usage(&policy(), &pair(remaining, 57.0), &[], NOW),
                    UsageDecision::Allow(_)
                ),
                "remaining {remaining} should allow"
            );
        }
    }

    #[test]
    fn policy_accepts_only_setup_supported_threshold_overrides() {
        for threshold in [1, 57, 99] {
            let configured = policy()
                .with_stop_remaining_percent(threshold)
                .expect("supported threshold should be accepted");
            assert_eq!(
                configured.stop_threshold_percent().to_bits(),
                f64::from(threshold).to_bits()
            );
        }
        for threshold in [0, 100] {
            assert_eq!(
                policy().with_stop_remaining_percent(threshold),
                Err(UsagePolicyError::StopThresholdOutOfRange { value: threshold })
            );
        }
    }

    #[test]
    fn session_takes_precedence_when_both_windows_exist() {
        assert!(matches!(
            evaluate_usage(&policy(), &pair(57.0, 9.99), &[], NOW),
            UsageDecision::Allow(_)
        ));
        let UsageDecision::Block(block) = evaluate_usage(&policy(), &pair(9.99, 57.0), &[], NOW)
        else {
            panic!("a low session window should block");
        };
        assert_eq!(block.window, UsageWindow::Session);
    }

    #[test]
    fn missing_and_duplicate_windows_are_unknown() {
        let session = snapshot(UsageWindow::Session, 57.0);
        assert!(matches!(
            evaluate_usage(&policy(), std::slice::from_ref(&session), &[], NOW),
            UsageDecision::Allow(_)
        ));
        assert_unknown(&[], &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::MissingWindow {
                    window: UsageWindow::Weekly
                }
            )
        });

        assert_unknown(
            &[
                session.clone(),
                session,
                snapshot(UsageWindow::Weekly, 57.0),
            ],
            &[],
            |reason| {
                matches!(
                    reason,
                    UsageUnknownReason::DuplicateWindow {
                        window: UsageWindow::Session
                    }
                )
            },
        );
    }

    #[test]
    fn stale_and_future_measurements_are_unknown() {
        let mut stale = pair(57.0, 57.0);
        stale[0].expires_at_unix_seconds = NOW;
        assert_unknown(&stale, &[], |reason| {
            matches!(reason, UsageUnknownReason::StaleMeasurement { .. })
        });

        let mut at_skew = pair(57.0, 57.0);
        at_skew[0].measured_at_unix_seconds = NOW + 5;
        assert!(matches!(
            evaluate_usage(&policy(), &at_skew, &[], NOW),
            UsageDecision::Allow(_)
        ));

        let mut future = pair(57.0, 57.0);
        future[0].measured_at_unix_seconds = NOW + 6;
        assert_unknown(&future, &[], |reason| {
            matches!(reason, UsageUnknownReason::FutureMeasurement { .. })
        });
    }

    #[test]
    fn weekly_only_is_allowed_while_the_host_disables_the_session_limit() {
        assert!(matches!(
            evaluate_usage(&policy(), &[snapshot(UsageWindow::Weekly, 51.0)], &[], NOW),
            UsageDecision::Allow(_)
        ));
        assert!(matches!(
            evaluate_usage(&policy(), &[snapshot(UsageWindow::Weekly, 10.0)], &[], NOW),
            UsageDecision::Block(_)
        ));
    }

    #[test]
    fn unsupported_schema_version_is_unknown() {
        let mut unsupported = pair(57.0, 57.0);
        unsupported[0].schema_version = 2;
        assert_unknown(&unsupported, &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::UnsupportedSchemaVersion { actual: 2 }
            )
        });
    }

    #[test]
    fn sensor_host_and_account_mismatches_are_typed_unknowns() {
        let mut sensor_id = pair(57.0, 57.0);
        sensor_id[0].sensor_id = "other".to_owned();
        assert_unknown(&sensor_id, &[], |reason| {
            matches!(reason, UsageUnknownReason::SensorIdMismatch { .. })
        });

        let mut sensor_version = pair(57.0, 57.0);
        sensor_version[0].sensor_version = "9.9.9".to_owned();
        assert_unknown(&sensor_version, &[], |reason| {
            matches!(reason, UsageUnknownReason::SensorVersionMismatch { .. })
        });

        let mut host = pair(57.0, 57.0);
        host[0].host_scope = "claude".to_owned();
        assert_unknown(&host, &[], |reason| {
            matches!(reason, UsageUnknownReason::HostScopeMismatch { .. })
        });

        let mut account = pair(57.0, 57.0);
        account[0].account_scope_digest = "sha256:other".to_owned();
        assert_unknown(&account, &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::AccountScopeDigestMismatch { .. }
            )
        });
    }

    #[test]
    fn untrusted_nonfinite_and_out_of_range_values_are_unknown() {
        let mut untrusted = pair(57.0, 57.0);
        untrusted[0].source_confidence = SourceConfidence::Medium;
        assert_unknown(&untrusted, &[], |reason| {
            matches!(reason, UsageUnknownReason::UntrustedSource { .. })
        });
        let mut low_confidence = pair(57.0, 57.0);
        low_confidence[0].source_confidence = SourceConfidence::Low;
        assert_unknown(&low_confidence, &[], |reason| {
            matches!(reason, UsageUnknownReason::UntrustedSource { .. })
        });

        for remaining in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let nonfinite = pair(remaining, 57.0);
            assert_unknown(&nonfinite, &[], |reason| {
                matches!(reason, UsageUnknownReason::NonFiniteRemaining { .. })
            });
        }

        for remaining in [-0.01, 100.01] {
            let out_of_range = pair(remaining, 57.0);
            assert_unknown(&out_of_range, &[], |reason| {
                matches!(reason, UsageUnknownReason::RemainingOutOfRange { .. })
            });
        }
    }

    #[test]
    fn measurement_reset_and_same_reset_increase_regressions_are_unknown() {
        let previous = pair(57.0, 57.0);

        let mut measurement = pair(56.0, 56.0);
        measurement[0].measured_at_unix_seconds = NOW - 1;
        assert_unknown(&measurement, &previous, |reason| {
            matches!(reason, UsageUnknownReason::MeasurementRegression { .. })
        });

        let mut reset = pair(56.0, 56.0);
        reset[0].resets_at_unix_seconds = NOW + 3_599;
        assert_unknown(&reset, &previous, |reason| {
            matches!(reason, UsageUnknownReason::ResetRegression { .. })
        });

        let increase = pair(58.0, 57.0);
        assert_unknown(&increase, &previous, |reason| {
            matches!(
                reason,
                UsageUnknownReason::SameResetRemainingIncrease { .. }
            )
        });
    }

    #[test]
    fn a_new_reset_allows_remaining_quota_to_increase() {
        let previous = pair(11.0, 11.0);
        let mut current = pair(57.0, 57.0);
        for snapshot in &mut current {
            snapshot.measured_at_unix_seconds = NOW + 1;
            snapshot.resets_at_unix_seconds = NOW + 7_200;
        }

        assert!(matches!(
            evaluate_usage(&policy(), &current, &previous, NOW),
            UsageDecision::Allow(_)
        ));
    }

    #[test]
    fn permit_uses_the_earliest_expiry_and_can_only_be_consumed_once() {
        let mut current = pair(57.0, 57.0);
        current[0].expires_at_unix_seconds = NOW + 3;
        let UsageDecision::Allow(mut permit) = evaluate_usage(&policy(), &current, &[], NOW) else {
            panic!("valid measurements should permit dispatch");
        };

        assert_eq!(permit.expires_at_unix_seconds(), NOW + 3);
        assert_eq!(permit.consume(NOW + 2), Ok(()));
        assert_eq!(
            permit.consume(NOW + 2),
            Err(UsagePermitError::AlreadyConsumed)
        );
    }

    #[test]
    fn permit_is_capped_at_five_seconds_and_expires_at_the_deadline() {
        let current = pair(57.0, 57.0);
        let UsageDecision::Allow(mut permit) = evaluate_usage(&policy(), &current, &[], NOW) else {
            panic!("valid measurements should permit dispatch");
        };

        assert_eq!(permit.expires_at_unix_seconds(), NOW + 5);
        assert_eq!(
            permit.consume(NOW + 5),
            Err(UsagePermitError::Expired {
                expires_at_unix_seconds: NOW + 5,
                attempted_at_unix_seconds: NOW + 5,
            })
        );
    }
}
