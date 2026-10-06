//! Sanitized subscription quota callbacks. No model or provider calls.
use super::{
    bind_session, portable_relative_path, read_installed_config, ActionResult, AdapterError,
    CaptureArguments, Evidence, ParsedBinding, PinnedTarget, SessionBinding, MAX_CAPTURE_BYTES,
    MAX_CONTROL_BYTES,
};
use crate::usage::{self, NormalizedSnapshot, NormalizedWindow, SensorError};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: &str = "1.1.18";
const POOLS: [&str; 2] = ["gemini-weekly", "3p-weekly"];
const MAX_AGE: u64 = 120;

#[derive(Deserialize)]
struct Input {
    conversation_id: String,
    #[serde(default)]
    session_id: Option<String>,
    version: String,
    email: String,
    #[serde(default)]
    workspace: Option<super::ClaudeWorkspace>,
    quota: BTreeMap<String, Bucket>,
}

#[derive(Deserialize)]
struct Bucket {
    remaining_fraction: f64,
    reset_time: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    schema_version: u32,
    sensor_version: String,
    session_digest: String,
    account_digest: String,
    received_at: u64,
    received_at_millis: u128,
    expires_at: u64,
    pools: BTreeMap<String, StoredBucket>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredBucket {
    remaining_percent: f64,
    resets_at: u64,
}

fn normalize(input: &Input, now: u64, millis: u128) -> Result<Capture, SensorError> {
    if input.version != VERSION {
        return Err(SensorError::UnsupportedVersion);
    }
    if input.conversation_id.is_empty()
        || input.conversation_id.len() > 256
        || input.conversation_id.chars().any(char::is_control)
        || input
            .session_id
            .as_ref()
            .is_some_and(|id| id != &input.conversation_id)
    {
        return Err(SensorError::WrongSession);
    }
    let email = input.email.trim();
    if email.is_empty()
        || email.len() > 320
        || !email.contains('@')
        || email.chars().any(char::is_whitespace)
        || email.chars().any(char::is_control)
    {
        return Err(SensorError::MissingIdentity);
    }
    if input.quota.len() != POOLS.len() || POOLS.iter().any(|id| !input.quota.contains_key(*id)) {
        return Err(SensorError::WrongWindows);
    }
    let mut pools = BTreeMap::new();
    let mut expires_at = now.saturating_add(MAX_AGE);
    for (id, bucket) in &input.quota {
        if !bucket.remaining_fraction.is_finite()
            || !(0.0..=1.0).contains(&bucket.remaining_fraction)
        {
            return Err(SensorError::Malformed);
        }
        let resets_at = usage::parse_iso8601_z(&bucket.reset_time)?;
        if resets_at <= now {
            return Err(SensorError::Stale);
        }
        expires_at = expires_at.min(resets_at);
        pools.insert(
            id.clone(),
            StoredBucket {
                remaining_percent: bucket.remaining_fraction * 100.0,
                resets_at,
            },
        );
    }
    Ok(Capture {
        schema_version: 1,
        sensor_version: input.version.clone(),
        session_digest: bind_session(
            &ParsedBinding {
                session_id: input.conversation_id.clone(),
                process_id: 1,
            },
            "antigravity",
        )
        .session_digest,
        account_digest: sha256_digest(email.as_bytes()),
        received_at: now,
        received_at_millis: millis,
        expires_at,
        pools,
    })
}

fn path(session_digest: &str) -> Result<PathBuf, AdapterError> {
    if !super::is_sha256_digest(session_digest) {
        return Err(AdapterError::Input(
            "invalid Antigravity session digest".to_owned(),
        ));
    }
    Ok(PathBuf::from(format!(
        ".hive/runtime/usage-guard/antigravity/{}.json",
        &session_digest[7..]
    )))
}

pub(super) fn capture(arguments: &CaptureArguments) -> Result<ActionResult, AdapterError> {
    let mut bytes = Vec::new();
    io::stdin()
        .take((MAX_CAPTURE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| AdapterError::Input("cannot read Antigravity status input".to_owned()))?;
    if bytes.len() > MAX_CAPTURE_BYTES {
        return Err(AdapterError::Input(
            "Antigravity input exceeds 1 MiB".to_owned(),
        ));
    }
    let input: Input = serde_json::from_value(
        usage::parse_strict_native_json(&bytes)
            .map_err(|_| AdapterError::Input("malformed Antigravity status input".to_owned()))?,
    )
    .map_err(|_| AdapterError::Input("missing Antigravity status fields".to_owned()))?;
    let target_path = if arguments.target_from_stdin {
        let workspace = input
            .workspace
            .as_ref()
            .ok_or_else(|| AdapterError::Input("Antigravity workspace is absent".to_owned()))?;
        let value = workspace
            .project_dir
            .as_ref()
            .or(workspace.current_dir.as_ref())
            .ok_or_else(|| {
                AdapterError::Input("Antigravity workspace target is absent".to_owned())
            })?;
        if value.len() > 4096
            || value.chars().any(char::is_control)
            || !PathBuf::from(value).is_absolute()
        {
            return Err(AdapterError::Input(
                "Antigravity target must be an absolute path".to_owned(),
            ));
        }
        PathBuf::from(value)
    } else {
        arguments
            .target
            .clone()
            .ok_or_else(|| AdapterError::Input("capture target is absent".to_owned()))?
    };
    let target = PinnedTarget::open(&target_path)?;
    if read_installed_config(&target)?.primary_host != "antigravity" {
        return Err(AdapterError::Safety(
            "Antigravity capture requires an Antigravity-primary harness".to_owned(),
        ));
    }
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AdapterError::Input("invalid capture clock".to_owned()))?;
    let captured = normalize(&input, time.as_secs(), time.as_millis())
        .map_err(|error| AdapterError::Input(format!("Antigravity capture rejected: {error}")))?;
    let relative = path(&captured.session_digest)?;
    let desired = serde_json::to_vec(&captured)
        .map_err(|_| AdapterError::Internal("cannot encode capture".to_owned()))?;
    let changed = super::publish_capture::<Capture>(
        &target,
        &relative,
        &desired,
        captured.received_at_millis,
        |old| old.received_at_millis,
    )?;
    Ok(ActionResult {
        schema_version: 1,
        action: "CaptureUsage",
        status: "success",
        exit_code: 0,
        code: if changed {
            "hive.usage-capture-recorded"
        } else {
            "hive.usage-capture-current"
        },
        message: "sanitized Antigravity subscription quota captured".to_owned(),
        changed_paths: changed
            .then(|| portable_relative_path(&relative))
            .into_iter()
            .collect(),
        evidence: vec![Evidence {
            kind: "report",
            locator: "antigravity-statusline-capture:sanitized".to_owned(),
            digest: sha256_digest(&desired),
        }],
        next_action: None,
        data: Some(
            json!({"host_scope":"antigravity","quota_pools":POOLS,"raw_input_persisted":false,"periodic_observation_verified":false}),
        ),
    })
}

fn snapshot(
    c: Capture,
    binding: &SessionBinding,
    account: Option<&str>,
    now: u64,
) -> Result<NormalizedSnapshot, SensorError> {
    if c.schema_version != 1 || c.sensor_version != VERSION {
        return Err(SensorError::UnsupportedVersion);
    }
    if c.session_digest != binding.session_digest || binding.host_scope != "antigravity" {
        return Err(SensorError::WrongSession);
    }
    if !super::is_sha256_digest(&c.account_digest) || account.is_some_and(|a| a != c.account_digest)
    {
        return Err(SensorError::AccountNotFound);
    }
    if c.pools.len() != POOLS.len() || POOLS.iter().any(|id| !c.pools.contains_key(*id)) {
        return Err(SensorError::WrongWindows);
    }
    let expected_expiry = c
        .pools
        .values()
        .fold(c.received_at.saturating_add(MAX_AGE), |a, b| {
            a.min(b.resets_at)
        });
    if c.received_at > now.saturating_add(5)
        || c.received_at_millis / 1000 != u128::from(c.received_at)
        || now >= c.expires_at
        || c.expires_at != expected_expiry
    {
        return Err(SensorError::Stale);
    }
    let mut windows = Vec::new();
    for id in POOLS {
        let pool = &c.pools[id];
        if !pool.remaining_percent.is_finite() || !(0.0..=100.0).contains(&pool.remaining_percent) {
            return Err(SensorError::Malformed);
        }
        if pool.resets_at <= now {
            return Err(SensorError::Stale);
        }
        windows.push(NormalizedWindow {
            name: "provider",
            quota_pool: Some(id),
            window_minutes: None,
            remaining_percent: pool.remaining_percent,
            resets_at: pool.resets_at,
        });
    }
    Ok(NormalizedSnapshot {
        sensor_id: "antigravity-statusline".to_owned(),
        sensor_version: c.sensor_version,
        provider: "antigravity".to_owned(),
        account_digest: c.account_digest,
        measured_at: c.received_at,
        expires_at: c.expires_at,
        source_confidence: "local".to_owned(),
        windows,
    })
}

pub(super) fn read(
    target: &PinnedTarget,
    binding: &SessionBinding,
    account: Option<&str>,
    sampled_at: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    let relative = path(&binding.session_digest).map_err(|_| SensorError::FilesystemSafety)?;
    let bytes = target
        .read_optional(&relative, MAX_CONTROL_BYTES)
        .map_err(|_| SensorError::FilesystemSafety)?
        .ok_or(SensorError::Unavailable)?;
    let c = serde_json::from_value(usage::parse_strict_native_json(&bytes)?)
        .map_err(|_| SensorError::Malformed)?;
    let now = sampled_at
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SensorError::ClockInvalid)?
        .as_secs();
    snapshot(c, binding, account, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Input {
        serde_json::from_value(json!({"conversation_id":"test-session","version":"1.1.18","email":"test@example.invalid",
            "quota":{"gemini-weekly":{"remaining_fraction":0.8,"reset_time":"2030-01-01T00:00:00Z"},
            "3p-weekly":{"remaining_fraction":0.01,"reset_time":"2030-01-01T00:00:00Z"}}})).unwrap()
    }
    fn binding() -> SessionBinding {
        bind_session(
            &ParsedBinding {
                session_id: "test-session".to_owned(),
                process_id: 3,
            },
            "antigravity",
        )
    }
    #[test]
    fn captures_both_pools_without_identity_or_duration_inference() {
        let c = normalize(&input(), 100, 100_000).unwrap();
        let bytes = String::from_utf8(serde_json::to_vec(&c).unwrap()).unwrap();
        assert!(!bytes.contains("test@example.invalid"));
        assert!(!bytes.contains("test-session"));
        let s = snapshot(c, &binding(), None, 110).unwrap();
        assert_eq!(s.windows.len(), 2);
        assert!((s.windows[1].remaining_percent - 1.0).abs() < f64::EPSILON);
        assert!(s.windows.iter().all(|w| w.window_minutes.is_none()));
        assert!(s.core_snapshots().iter().all(|s| s.schema_version == 2));
    }
    #[test]
    fn rejects_missing_unknown_invalid_and_unqualified_input() {
        let mut i = input();
        i.quota.remove("3p-weekly");
        assert!(normalize(&i, 100, 100_000).is_err());
        let mut i = input();
        i.quota.insert(
            "other".to_owned(),
            Bucket {
                remaining_fraction: 0.5,
                reset_time: "2030-01-01T00:00:00Z".to_owned(),
            },
        );
        assert!(normalize(&i, 100, 100_000).is_err());
        let mut i = input();
        i.version = "1.1.19".to_owned();
        assert!(matches!(
            normalize(&i, 100, 100_000),
            Err(SensorError::UnsupportedVersion)
        ));
        let mut i = input();
        i.quota.get_mut("gemini-weekly").unwrap().remaining_fraction = 1.01;
        assert!(normalize(&i, 100, 100_000).is_err());
        let mut i = input();
        i.session_id = Some("other".to_owned());
        assert!(normalize(&i, 100, 100_000).is_err());
    }
    #[test]
    fn rejects_stale_cross_account_and_tampered_capture() {
        assert!(matches!(
            snapshot(
                normalize(&input(), 100, 100_000).unwrap(),
                &binding(),
                None,
                220
            ),
            Err(SensorError::Stale)
        ));
        assert!(matches!(
            snapshot(
                normalize(&input(), 100, 100_000).unwrap(),
                &binding(),
                Some(&sha256_digest(b"other")),
                110
            ),
            Err(SensorError::AccountNotFound)
        ));
        let mut c = normalize(&input(), 100, 100_000).unwrap();
        c.expires_at += 1;
        assert!(snapshot(c, &binding(), None, 110).is_err());
    }
    #[test]
    fn expiry_cannot_outlive_quota_reset() {
        let i = input();
        let reset = usage::parse_iso8601_z("2030-01-01T00:00:00Z").unwrap();
        let c = normalize(&i, reset - 10, u128::from(reset - 10) * 1000).unwrap();
        assert_eq!(c.expires_at, reset);
        assert!(matches!(
            snapshot(c, &binding(), None, reset),
            Err(SensorError::Stale)
        ));
    }
}
