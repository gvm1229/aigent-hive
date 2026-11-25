//! A failed observation is not a block when the user selected the v2 warning policy, and a
//! missing measurement is never a block (OV160-013 U1, maintainer decision 2026-11-21).
use super::{
    capture_failure::CaptureFailure, effective_policy_digest, missing_window_labels,
    portable_relative_path, ActionResult, AdapterError, EnforcementTrace, Evidence, HaltMarker,
    InstalledUsageConfig, LoadedControl, LoadedHalt, OverrideState, PinnedTarget, SessionBinding,
    TurnObservation,
};
use hive_core::sha256_digest;
use serde_json::json;

pub(super) fn is_current(
    config: &InstalledUsageConfig,
    halt: &LoadedHalt,
) -> Result<bool, AdapterError> {
    let policy = effective_policy_digest(config)?;
    Ok(halt.state == OverrideState::Current
        && halt.marker.as_ref().is_some_and(|marker| {
            marker.schema_version == 2
                && marker.decision == "usage-warning"
                && marker.policy_digest.as_deref() == Some(&policy)
        }))
}

pub(super) fn result(
    root: &PinnedTarget,
    config: &InstalledUsageConfig,
    binding: &SessionBinding,
    control: &LoadedControl,
    previous: &LoadedHalt,
    observation: &TurnObservation,
    trace: &mut EnforcementTrace,
) -> Result<ActionResult, AdapterError> {
    if matches!(control.state, OverrideState::Stale | OverrideState::Damaged) {
        return Err(AdapterError::Safety(
            "usage session control must be valid before warning-policy dispatch".to_owned(),
        ));
    }
    let baseline = previous
        .marker
        .as_ref()
        .filter(|_| previous.state == OverrideState::Current);
    let marker = HaltMarker {
        schema_version: 2,
        host_scope: binding.host_scope.clone(),
        session_id_digest: binding.session_digest.clone(),
        process_id: binding.process_id,
        decision: "usage-warning".to_owned(),
        selected_window: "unknown".to_owned(),
        threshold_remaining_percent: config.threshold,
        policy_digest: Some(trace.policy_digest.clone()),
        remaining_percent: None,
        // The failed attempt is not a fresh measurement. Keep the last accepted reset baseline.
        measured_at: baseline.map_or(0, |old| old.measured_at),
        evidence_digest: observation.evidence_digest.clone(),
        run_id: None,
        request_summary: None,
        progress: None,
        snapshots: baseline.map_or_else(Vec::new, |old| old.snapshots.clone()),
        revision: baseline.map_or(1, |old| old.revision.saturating_add(1)),
    };
    let bytes =
        serde_json::to_vec(&marker).map_err(|error| AdapterError::Internal(error.to_string()))?;
    let changed = root.publish_runtime(&previous.relative, &previous.snapshot, &bytes)?;
    trace.halt_digest = sha256_digest(&bytes);
    trace.normalized = None;
    Ok(ActionResult {
        schema_version: 1,
        action: "CheckUsage",
        status: "success",
        exit_code: 0,
        code: "hive.usage-observation-warning",
        message: if config.interface_language == "ko" {
            "선택한 사용량 정책 확인 완료"
        } else {
            "Selected usage policy evaluated"
        }
        .to_owned(),
        changed_paths: changed
            .then(|| portable_relative_path(&previous.relative))
            .into_iter()
            .collect(),
        evidence: vec![Evidence {
            kind: "file",
            locator: portable_relative_path(&previous.relative),
            digest: trace.halt_digest.clone(),
        }],
        next_action: None,
        data: Some(
            json!({"policy_schema_version":2,"policy_evaluated":true,"enforcement_enabled":true,
            "enforced":false,"observation_required":true,"observation_available":false,"observation_failure":"warn",
            "attempted_at":observation.measured_at,"host_scope":binding.host_scope,"session_id_digest":binding.session_digest,
            "process_id":binding.process_id,"threshold_remaining_percent":config.threshold,"policy_digest":trace.policy_digest,
            "scope":"automatic-dispatch-preflight","authorizes_dispatch":false}),
        ),
    })
}

/// What a warning reported, for the optional Discord notice.
pub(super) struct WarningSummary {
    pub(super) missing_windows: Vec<&'static str>,
    pub(super) cause: &'static str,
}

/// Name the missing windows, the cause and the fix in the warning result (OV160-013 U1/U3/U5).
pub(super) fn describe(
    result: &mut ActionResult,
    config: &InstalledUsageConfig,
    observation: &TurnObservation,
    failure: Option<&CaptureFailure>,
    shown: bool,
) -> WarningSummary {
    let korean = config.interface_language == "ko";
    let missing_windows = if observation.missing_windows.is_empty() {
        missing_window_labels(config, None)
    } else {
        observation.missing_windows.clone()
    };
    // A rejected status-line payload explains an absent or old Claude measurement.
    let cause = if failure.is_some()
        && matches!(
            observation.cause,
            "no-measurement" | "measurement-stale" | "window-missing"
        ) {
        "capture-failed"
    } else {
        observation.cause
    };
    let windows = missing_windows
        .iter()
        .map(|window| crate::discord::window_display(window, korean))
        .collect::<Vec<_>>()
        .join(", ");
    let cause_text = crate::discord::observation_cause_display(
        cause,
        &config.primary_host,
        failure.map(|failure| failure.reason.as_str()),
        korean,
    );
    let fix = crate::discord::observation_fix_command(&config.primary_host);
    let message = if korean {
        format!("사용량 측정 없음({windows}): {cause_text}. 경고 후 작업 계속. 해결: `{fix}`")
    } else {
        format!(
            "Usage is not measured ({windows}): {cause_text}. Work continues with a warning. Fix: `{fix}`"
        )
    };
    result.message.clone_from(&message);
    result.next_action.clone_from(&observation.next_action);
    if let Some(data) = result.data.as_mut() {
        data["missing_windows"] = json!(missing_windows);
        data["observation_cause"] = json!(cause);
        data["missing_measurement"] = json!(observation.missing_measurement);
        data["capture_failure"] = failure.map_or(serde_json::Value::Null, CaptureFailure::to_json);
        data["fix_command"] = json!(fix);
        data["notice"] = if shown {
            json!({"kind": "observation-unavailable", "message": message})
        } else {
            serde_json::Value::Null
        };
    }
    WarningSummary {
        missing_windows,
        cause,
    }
}
