//! Provider-neutral subscription usage policy evaluation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Remaining percentage at or below which automatic work must stop.
pub const DEFAULT_STOP_THRESHOLD_PERCENT: f64 = 10.0;
/// Backward-compatible quota pool for providers with one shared limit.
pub const DEFAULT_QUOTA_POOL: &str = "default";
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
    /// A provider-defined quota whose cadence is not asserted by the sensor.
    Provider,
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
    /// Snapshot contract version. Version 1 is unpooled; version 2 is pooled.
    pub schema_version: u32,
    /// Stable identifier of the local sensor implementation.
    pub sensor_id: String,
    /// Version reported by the local sensor implementation.
    pub sensor_version: String,
    /// Subscription host to which this measurement applies.
    pub host_scope: String,
    /// Non-reversible digest identifying the measured subscription account.
    pub account_scope_digest: String,
    /// Stable provider-local quota pool. Present only in pooled schema version 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_pool: Option<String>,
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
    /// A provider-local quota pool is missing, malformed, or attached to v1.
    InvalidQuotaPool { actual: Option<String> },
    /// The quota window is not valid for the snapshot schema version.
    InvalidQuotaWindow {
        schema_version: u32,
        window: UsageWindow,
    },
    /// Provider-defined and cadence-defined windows coexist in one quota pool.
    ConflictingWindowKinds { quota_pool: String },
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
/// present; otherwise the weekly window is the required fallback. A
/// provider-defined window is accepted only when no cadence-defined window
/// coexists in the same pool.
#[must_use]
pub fn evaluate_usage(
    policy: &UsagePolicy,
    current_snapshots: &[UsageSnapshot],
    previous_snapshots: &[UsageSnapshot],
    now_unix_seconds: i64,
) -> UsageDecision {
    let mut scopes = BTreeSet::new();
    for snapshot in current_snapshots.iter().chain(previous_snapshots) {
        scopes.insert(raw_quota_pool(snapshot));
    }
    if scopes.is_empty() {
        return UsageDecision::Unknown(UsageUnknownReason::MissingWindow {
            window: UsageWindow::Weekly,
        });
    }

    let mut selected = Vec::with_capacity(scopes.len());
    for scope in scopes {
        let provider = match unique_snapshot(current_snapshots, scope, UsageWindow::Provider) {
            Ok(snapshot) => snapshot,
            Err(reason) => return UsageDecision::Unknown(reason),
        };
        let has_cadence_window = current_snapshots.iter().any(|snapshot| {
            raw_quota_pool(snapshot) == scope
                && matches!(
                    snapshot.quota_window,
                    UsageWindow::Session | UsageWindow::Weekly
                )
        });
        if provider.is_some() && has_cadence_window {
            return UsageDecision::Unknown(UsageUnknownReason::ConflictingWindowKinds {
                quota_pool: scope.to_owned(),
            });
        }
        let snapshot = if let Some(provider) = provider {
            provider
        } else {
            match unique_snapshot(current_snapshots, scope, UsageWindow::Session) {
                Ok(Some(snapshot)) => snapshot,
                Ok(None) => match unique_snapshot(current_snapshots, scope, UsageWindow::Weekly) {
                    Ok(Some(snapshot)) => snapshot,
                    Ok(None) => {
                        return UsageDecision::Unknown(UsageUnknownReason::MissingWindow {
                            window: UsageWindow::Weekly,
                        });
                    }
                    Err(reason) => return UsageDecision::Unknown(reason),
                },
                Err(reason) => return UsageDecision::Unknown(reason),
            }
        };
        if let Some(reason) = validate_snapshot(policy, snapshot, now_unix_seconds) {
            return UsageDecision::Unknown(reason);
        }
        let mut previous_matching = previous_snapshots.iter().filter(|previous| {
            raw_quota_pool(previous) == raw_quota_pool(snapshot)
                && previous.quota_window == snapshot.quota_window
        });
        if let Some(previous) = previous_matching.next() {
            if previous_matching.next().is_some() {
                return UsageDecision::Unknown(UsageUnknownReason::DuplicateWindow {
                    window: snapshot.quota_window,
                });
            }
            if let Some(reason) = validate_snapshot_contract(policy, previous) {
                return UsageDecision::Unknown(reason);
            }
            if let Some(reason) = validate_monotonicity(snapshot, previous) {
                return UsageDecision::Unknown(reason);
            }
        }
        selected.push(snapshot);
    }

    if let Some(snapshot) = selected
        .iter()
        .copied()
        .filter(|snapshot| snapshot.remaining_percent <= policy.stop_threshold_percent())
        .min_by(|left, right| left.remaining_percent.total_cmp(&right.remaining_percent))
    {
        return UsageDecision::Block(UsageBlock {
            window: snapshot.quota_window,
            remaining_percent: snapshot.remaining_percent,
            threshold_percent: policy.stop_threshold_percent(),
        });
    }

    let policy_expiry = now_unix_seconds.saturating_add(policy.permit_deadline_seconds());
    UsageDecision::Allow(UsagePermit {
        expires_at_unix_seconds: selected
            .iter()
            .map(|snapshot| snapshot.expires_at_unix_seconds)
            .min()
            .unwrap_or(policy_expiry)
            .min(policy_expiry),
        consumed: false,
    })
}

fn unique_snapshot<'a>(
    snapshots: &'a [UsageSnapshot],
    scope: &str,
    window: UsageWindow,
) -> Result<Option<&'a UsageSnapshot>, UsageUnknownReason> {
    let mut matching = snapshots
        .iter()
        .filter(|snapshot| raw_quota_pool(snapshot) == scope && snapshot.quota_window == window);
    let snapshot = matching.next();
    if matching.next().is_some() {
        return Err(UsageUnknownReason::DuplicateWindow { window });
    }
    Ok(snapshot)
}

fn raw_quota_pool(snapshot: &UsageSnapshot) -> &str {
    snapshot.quota_pool.as_deref().unwrap_or(DEFAULT_QUOTA_POOL)
}

fn valid_quota_pool(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().enumerate().all(|(index, byte)| match byte {
            b'a'..=b'z' | b'0'..=b'9' => true,
            b'-' => index > 0 && index + 1 < value.len(),
            _ => false,
        })
}

fn validate_snapshot(
    policy: &UsagePolicy,
    snapshot: &UsageSnapshot,
    now_unix_seconds: i64,
) -> Option<UsageUnknownReason> {
    if let Some(reason) = validate_snapshot_contract(policy, snapshot) {
        return Some(reason);
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

fn validate_snapshot_contract(
    policy: &UsagePolicy,
    snapshot: &UsageSnapshot,
) -> Option<UsageUnknownReason> {
    if !matches!(snapshot.schema_version, 1 | 2) {
        return Some(UsageUnknownReason::UnsupportedSchemaVersion {
            actual: snapshot.schema_version,
        });
    }
    match (snapshot.schema_version, snapshot.quota_pool.as_deref()) {
        (1, None) => {}
        (2, Some(pool)) if valid_quota_pool(pool) => {}
        _ => {
            return Some(UsageUnknownReason::InvalidQuotaPool {
                actual: snapshot.quota_pool.clone(),
            });
        }
    }
    if snapshot.schema_version == 1 && snapshot.quota_window == UsageWindow::Provider {
        return Some(UsageUnknownReason::InvalidQuotaWindow {
            schema_version: snapshot.schema_version,
            window: snapshot.quota_window,
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
            quota_pool: None,
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

    fn scoped_snapshot(scope: &str, window: UsageWindow, remaining_percent: f64) -> UsageSnapshot {
        let mut snapshot = snapshot(window, remaining_percent);
        snapshot.schema_version = 2;
        snapshot.quota_pool = Some(scope.to_owned());
        snapshot
    }

    #[test]
    fn version_one_snapshot_serializes_to_legacy_bytes() {
        let encoded = serde_json::to_string(&snapshot(UsageWindow::Session, 57.0))
            .expect("version-one snapshot should serialize");

        assert_eq!(
            encoded,
            concat!(
                r#"{"schema_version":1,"sensor_id":"local-sensor","#,
                r#""sensor_version":"1.2.3","host_scope":"codex","#,
                r#""account_scope_digest":"sha256:account","#,
                r#""quota_window":"session","remaining_percent":57.0,"#,
                r#""measured_at_unix_seconds":1000,"expires_at_unix_seconds":1030,"#,
                r#""resets_at_unix_seconds":4600,"source_confidence":"high"}"#
            )
        );
        assert!(!encoded.contains("quota_pool"));
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
    fn session_decision_ignores_duplicate_or_invalid_weekly_snapshots() {
        let duplicate_weekly = vec![
            snapshot(UsageWindow::Session, 57.0),
            snapshot(UsageWindow::Weekly, 9.99),
            snapshot(UsageWindow::Weekly, 57.0),
        ];
        assert!(matches!(
            evaluate_usage(&policy(), &duplicate_weekly, &[], NOW),
            UsageDecision::Allow(_)
        ));

        let mut invalid_weekly = snapshot(UsageWindow::Weekly, 57.0);
        invalid_weekly.schema_version = 2;
        let current = vec![snapshot(UsageWindow::Session, 10.0), invalid_weekly];
        let UsageDecision::Block(block) = evaluate_usage(&policy(), &current, &[], NOW) else {
            panic!("the valid session snapshot should determine the decision");
        };
        assert_eq!(block.window, UsageWindow::Session);
        assert_eq!(block.remaining_percent.to_bits(), 10.0_f64.to_bits());
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
    fn every_quota_pool_must_pass_its_selected_window() {
        let current = [
            scoped_snapshot("antigravity-gemini", UsageWindow::Provider, 57.0),
            scoped_snapshot("antigravity-claude-gpt", UsageWindow::Provider, 9.99),
        ];
        let UsageDecision::Block(block) = evaluate_usage(&policy(), &current, &[], NOW) else {
            panic!("a limited provider pool must block automatic dispatch");
        };
        assert_eq!(block.window, UsageWindow::Provider);
        assert_eq!(block.remaining_percent.to_bits(), 9.99_f64.to_bits());

        let allowed = [
            scoped_snapshot("antigravity-gemini", UsageWindow::Provider, 57.0),
            scoped_snapshot("antigravity-claude-gpt", UsageWindow::Provider, 51.0),
        ];
        assert!(matches!(
            evaluate_usage(&policy(), &allowed, &[], NOW),
            UsageDecision::Allow(_)
        ));
    }

    #[test]
    fn provider_and_cadence_windows_cannot_coexist_in_one_pool() {
        let current = [
            scoped_snapshot("gemini", UsageWindow::Provider, 57.0),
            scoped_snapshot("gemini", UsageWindow::Weekly, 57.0),
        ];
        assert_unknown(&current, &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::ConflictingWindowKinds { quota_pool }
                    if quota_pool == "gemini"
            )
        });
    }

    #[test]
    fn session_precedence_and_duplicates_are_scoped_per_pool() {
        let current = [
            scoped_snapshot("gemini", UsageWindow::Session, 57.0),
            scoped_snapshot("gemini", UsageWindow::Weekly, 9.99),
            scoped_snapshot("claude-gpt", UsageWindow::Weekly, 57.0),
        ];
        assert!(matches!(
            evaluate_usage(&policy(), &current, &[], NOW),
            UsageDecision::Allow(_)
        ));

        let duplicates = [
            scoped_snapshot("gemini", UsageWindow::Weekly, 57.0),
            scoped_snapshot("gemini", UsageWindow::Weekly, 51.0),
            scoped_snapshot("claude-gpt", UsageWindow::Weekly, 57.0),
        ];
        assert_unknown(&duplicates, &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::DuplicateWindow {
                    window: UsageWindow::Weekly
                }
            )
        });
    }

    #[test]
    fn pool_history_is_matched_by_scope_and_missing_prior_pools_fail_closed() {
        let previous = [
            scoped_snapshot("gemini", UsageWindow::Weekly, 57.0),
            scoped_snapshot("claude-gpt", UsageWindow::Weekly, 57.0),
        ];
        let current = [
            scoped_snapshot("gemini", UsageWindow::Weekly, 56.0),
            scoped_snapshot("claude-gpt", UsageWindow::Weekly, 55.0),
        ];
        assert!(matches!(
            evaluate_usage(&policy(), &current, &previous, NOW),
            UsageDecision::Allow(_)
        ));

        assert_unknown(&current[..1], &previous, |reason| {
            matches!(reason, UsageUnknownReason::MissingWindow { .. })
        });

        let legacy = [snapshot(UsageWindow::Weekly, 57.0)];
        let pooled_default = [scoped_snapshot("default", UsageWindow::Weekly, 56.0)];
        assert!(matches!(
            evaluate_usage(&policy(), &pooled_default, &legacy, NOW),
            UsageDecision::Allow(_)
        ));
    }

    #[test]
    fn malformed_or_misversioned_quota_pool_is_unknown() {
        let mut invalid = snapshot(UsageWindow::Weekly, 57.0);
        invalid.schema_version = 2;
        invalid.quota_pool = Some("Gemini Pool".to_owned());
        assert_unknown(&[invalid], &[], |reason| {
            matches!(reason, UsageUnknownReason::InvalidQuotaPool { .. })
        });

        let mut v1_with_pool = snapshot(UsageWindow::Weekly, 57.0);
        v1_with_pool.quota_pool = Some("gemini".to_owned());
        assert_unknown(&[v1_with_pool], &[], |reason| {
            matches!(reason, UsageUnknownReason::InvalidQuotaPool { .. })
        });

        let mut v2_without_pool = snapshot(UsageWindow::Weekly, 57.0);
        v2_without_pool.schema_version = 2;
        assert_unknown(&[v2_without_pool], &[], |reason| {
            matches!(reason, UsageUnknownReason::InvalidQuotaPool { .. })
        });

        let v1_provider = snapshot(UsageWindow::Provider, 57.0);
        assert_unknown(&[v1_provider], &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::InvalidQuotaWindow {
                    schema_version: 1,
                    window: UsageWindow::Provider
                }
            )
        });
    }

    #[test]
    fn unsupported_schema_version_is_unknown() {
        let mut unsupported = pair(57.0, 57.0);
        unsupported[0].schema_version = 3;
        assert_unknown(&unsupported, &[], |reason| {
            matches!(
                reason,
                UsageUnknownReason::UnsupportedSchemaVersion { actual: 3 }
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
        let mut current = vec![
            scoped_snapshot("gemini", UsageWindow::Weekly, 57.0),
            scoped_snapshot("claude-gpt", UsageWindow::Weekly, 57.0),
        ];
        current[1].expires_at_unix_seconds = NOW + 3;
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
