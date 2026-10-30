//! A failed observation is not a block when the user selected the v2 warning policy.
use super::{
    effective_policy_digest, portable_relative_path, ActionResult, AdapterError, EnforcementTrace,
    Evidence, HaltMarker, InstalledUsageConfig, LoadedControl, LoadedHalt, OverrideState,
    PinnedTarget, SessionBinding, TurnObservation,
};
use hive_core::sha256_digest;
use serde_json::json;

pub(super) fn is_current(
    config: &InstalledUsageConfig,
    halt: &LoadedHalt,
) -> Result<bool, AdapterError> {
    let policy = effective_policy_digest(config)?;
    Ok(config.warn_on_observation_failure()
        && halt.state == OverrideState::Current
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
