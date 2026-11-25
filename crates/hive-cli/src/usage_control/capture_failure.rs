//! The last rejected Claude status-line capture (OV160-013 U3).
//!
//! The status-line wrapper must never break the host display, so a capture error has no visible
//! output there. This record keeps the reason in the user root, where `usage enforce` and
//! `usage status` can name it as the cause of a missing measurement. Raw stdin is never kept.
use super::{AdapterError, PinnedTarget, USER_CONFIG_PATH};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const RECORD_PATH: &str = ".hive/runtime/usage-guard/claude/capture-failure.json";
const MAX_RECORD_BYTES: usize = 4096;
const MAX_MESSAGE_CHARS: usize = 300;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaptureFailure {
    schema_version: u32,
    pub(super) failed_at_unix_seconds: u64,
    pub(super) reason: String,
    pub(super) message: String,
    /// Target digest of the rejected payload; absent when the folder was not resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_scope: Option<String>,
}

impl CaptureFailure {
    pub(super) fn to_json(&self) -> serde_json::Value {
        json!({
            "reason": self.reason,
            "message": self.message,
            "failed_at_unix_seconds": self.failed_at_unix_seconds,
        })
    }
}

/// Only an existing global Hive setup receives a record; this never creates the user root.
fn open_root(user_root: Option<&Path>) -> Option<PinnedTarget> {
    let user_root = user_root?;
    if !user_root.join(USER_CONFIG_PATH).is_file() {
        return None;
    }
    PinnedTarget::open_usage(user_root).ok()
}

/// Stable reason code for one capture error. The message keeps the exact local detail.
pub(super) fn reason_code(error: &AdapterError) -> &'static str {
    let message = error.message();
    if message.contains("rate_limits are unavailable") {
        "rate-limits-unavailable"
    } else if message.contains("rate-limit window is stale") {
        "rate-limits-stale"
    } else if message.contains("workspace path") {
        "workspace-unavailable"
    } else if message.contains("JSON is malformed") || message.contains("exceeds 1 MiB") {
        "statusline-input-invalid"
    } else if message.contains("--user-root") || message.contains("global Hive setup is required") {
        "user-setup-missing"
    } else if message.contains("not enabled by the global Hive setup")
        || message.contains("Claude-primary")
        || message.contains("does not match the installed project host")
    {
        "host-not-enabled"
    } else if matches!(error, AdapterError::Safety(_)) {
        "capture-refused"
    } else {
        "capture-failed"
    }
}

pub(super) fn record(user_root: Option<&Path>, scope: Option<&str>, error: &AdapterError) {
    let Some(root) = open_root(user_root) else {
        return;
    };
    let failure = CaptureFailure {
        schema_version: 1,
        failed_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs()),
        reason: reason_code(error).to_owned(),
        message: error
            .message()
            .chars()
            .filter(|character| !character.is_control())
            .take(MAX_MESSAGE_CHARS)
            .collect(),
        target_scope: scope.map(str::to_owned),
    };
    let Ok(mut bytes) = serde_json::to_vec(&failure) else {
        return;
    };
    bytes.push(b'\n');
    let path = Path::new(RECORD_PATH);
    if let Ok(snapshot) = root.snapshot_bounded(path, MAX_RECORD_BYTES) {
        let _ = root.publish_runtime(path, &snapshot, &bytes);
    }
}

/// A successful capture clears a failure for the same folder or an unresolved folder.
pub(super) fn clear(user_root: Option<&Path>, scope: Option<&str>) {
    let Some(root) = open_root(user_root) else {
        return;
    };
    let path = Path::new(RECORD_PATH);
    let Ok(snapshot) = root.snapshot_bounded(path, MAX_RECORD_BYTES) else {
        return;
    };
    let matches = snapshot
        .bytes()
        .and_then(|bytes| serde_json::from_slice::<CaptureFailure>(bytes).ok())
        .is_none_or(|old| old.target_scope.is_none() || old.target_scope.as_deref() == scope);
    if matches {
        let _ = root.remove_runtime(path, &snapshot);
    }
}

/// The last failure that applies to `scope`, if any.
pub(super) fn read(user_root: Option<&Path>, scope: &str) -> Option<CaptureFailure> {
    let root = open_root(user_root)?;
    let bytes = root
        .read_optional(Path::new(RECORD_PATH), MAX_RECORD_BYTES)
        .ok()??;
    let failure = serde_json::from_slice::<CaptureFailure>(&bytes).ok()?;
    (failure.schema_version == 1
        && failure
            .target_scope
            .as_deref()
            .is_none_or(|stored| stored == scope))
    .then_some(failure)
}
