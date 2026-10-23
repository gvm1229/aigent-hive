//! A configured opt-out is a completed policy evaluation, never permission to execute work.
use super::{
    effective_policy_digest, ActionResult, AdapterError, EnforcementTrace, Evidence,
    InstalledUsageConfig, SessionBinding,
};
use hive_core::sha256_digest;
use serde_json::json;

pub(super) fn trace(config: &InstalledUsageConfig) -> Result<EnforcementTrace, AdapterError> {
    Ok(EnforcementTrace {
        normalized: None,
        policy_digest: effective_policy_digest(config)?,
        control_digest: "not-required".to_owned(),
        halt_digest: "not-required".to_owned(),
    })
}

pub(super) fn result(
    action: &'static str,
    binding: &SessionBinding,
    config: &InstalledUsageConfig,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code: "hive.usage-disabled",
        message: if config.interface_language == "ko" {
            "사용량 보호 비활성 정책 확인 완료"
        } else {
            "Usage protection is disabled by the current policy"
        }
        .to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "file",
            locator: config.config_locator.clone(),
            digest: sha256_digest(&config.bytes),
        }],
        next_action: None,
        data: Some(
            json!({"policy_evaluated":true,"enforcement_enabled":false,"enforced":false,
            "observation_required":false,"guard_enabled":false,"zero_quota_guard_enabled":false,
            "quota_reset_guard_enabled":false,"monitor_registration_required":false,
            "host_scope":binding.host_scope,"session_id_digest":binding.session_digest,"process_id":binding.process_id,
            "threshold_remaining_percent":config.threshold,"global_threshold_remaining_percent":config.global_threshold,
            "project_threshold_remaining_percent":config.project_threshold,"scope":"automatic-dispatch-preflight",
            "authorizes_dispatch":false,"warnings_enabled":config.policy.as_ref().is_some_and(|policy| policy.warnings_enabled)}),
        ),
    }
}
