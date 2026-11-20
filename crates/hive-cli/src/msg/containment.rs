//! Worker containment on the file inbox (OV160-005): the least-privilege policy published beside
//! every envelope, the `max_workers` / `max_worker_seconds` limits read from
//! `.hive/config/harness.toml`, and digest-pinned score/check inputs of a flow.
//!
//! Hive never runs the worker. The policy is data the host applies; a profile whose argv would
//! grant more than the policy is refused before any envelope is written.
use super::{
    list_entries, load, publish_new, read_record, record_bytes, Context, EntryKind, MsgError,
    Published,
};
use crate::run::read_explicit_file;
use hive_core::containment::{
    self, CheckDrift, CheckPinRecord, LimitError, MessagePolicyRecord, PinnedCheck, WorkerLimits,
    WorkerPolicy, CONTAINMENT_SCHEMA_VERSION,
};
use hive_core::mediation::{self, MessageState, MEDIATION_ROOT};
use hive_core::roster::Roster;
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const HARNESS_CONFIG: &str = ".hive/config/harness.toml";
/// Largest accepted score or check input.
pub(super) const MAX_CHECK_BYTES: usize = 1024 * 1024;

// ---------------------------------------------------------------------------------------------
// Worker policy.

/// Policy for the recipient role, from its roster profile, after the argv escalation check.
pub(super) fn policy_for(context: &Context, role: &str) -> Result<WorkerPolicy, MsgError> {
    let (role, provider, resolved) = context
        .roster
        .resolve_named(role)
        .map_err(|error| MsgError::input(format!("unknown role: {error}")))?;
    let profile = Roster::worker_profile_resolved(role, provider, &resolved)
        .map_err(|reason| MsgError::blocked("hive.msg-policy-refused", reason))?;
    WorkerPolicy::for_profile(role, &profile)
        .map_err(|reason| MsgError::blocked("hive.msg-policy-refused", reason))
}

/// Publish the policy record before the envelope, so every envelope has one.
pub(super) fn publish_policy(
    context: &Context,
    role: &str,
    message_id: &str,
    changed: &mut Vec<String>,
) -> Result<(), MsgError> {
    let record = MessagePolicyRecord {
        schema_version: CONTAINMENT_SCHEMA_VERSION,
        message_id: message_id.to_owned(),
        policy: policy_for(context, role)?,
    };
    let path = containment::policy_path(message_id);
    match publish_new(&context.target, &path, &record_bytes(&record)?)? {
        Published::Created => changed.push(path),
        // A roster change between two sends of the same key: the first policy stays.
        Published::SameBytes | Published::Different(_) => {}
    }
    Ok(())
}

/// Policy view for one message: the recorded policy bound to the envelope.
pub(super) fn policy_view(
    context: &Context,
    envelope: Option<&mediation::MessageEnvelope>,
) -> Result<Value, MsgError> {
    let Some(envelope) = envelope else {
        return Ok(Value::Null);
    };
    let record: Option<Result<MessagePolicyRecord, String>> = read_record(
        &context.target,
        &containment::policy_path(&envelope.message_id),
    )?;
    Ok(match record {
        None => json!({"present": false}),
        Some(Ok(record)) => match record.validate(envelope) {
            Ok(()) => json!({"present": true, "policy": record.policy, "enforced_by_hive": false}),
            Err(reason) => json!({"present": true, "invalid": reason}),
        },
        Some(Err(reason)) => json!({"present": true, "invalid": reason}),
    })
}

// ---------------------------------------------------------------------------------------------
// Limits.

/// Limits from the project harness config; defaults when the file or keys are absent.
pub(super) fn worker_limits(context: &Context) -> Result<WorkerLimits, MsgError> {
    let Some(bytes) = context
        .target
        .read_optional(Path::new(HARNESS_CONFIG), 64 * 1024)?
    else {
        return Ok(WorkerLimits::default());
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| MsgError::input("installed harness config is not UTF-8"))?;
    containment::worker_limits_from_harness(text).map_err(|error| match error {
        LimitError::Conflict { .. } => {
            MsgError::blocked("hive.msg-limit-conflict", error.to_string())
        }
        LimitError::Invalid(reason) => MsgError::input(reason),
    })
}

/// Claims currently holding a worker slot: claimed or cancel-requested inside their lease.
pub(super) fn running_claims(context: &Context) -> Result<Vec<String>, MsgError> {
    let mut running = Vec::new();
    let claims = Path::new(MEDIATION_ROOT).join("claims");
    for (name, kind) in list_entries(&context.target, &claims)?.unwrap_or_default() {
        let Some(message_id) = name.strip_suffix(".json") else {
            continue;
        };
        if kind != EntryKind::File || !mediation::is_valid_id(message_id) {
            continue;
        }
        let Some(Ok(claim)) = read_record::<mediation::ClaimRecord>(
            &context.target,
            &mediation::claim_path(message_id),
        )?
        else {
            continue;
        };
        if !mediation::is_valid_id(&claim.claimant_role) {
            continue;
        }
        let envelope = mediation::envelope_path(&claim.claimant_role, message_id);
        if context
            .target
            .read_optional(Path::new(&envelope), super::MAX_RECORD_BYTES)?
            .is_none()
        {
            continue;
        }
        let loaded = load(context, &claim.claimant_role, message_id)?;
        if matches!(
            loaded.state(context.now).0,
            MessageState::Claimed | MessageState::CancelRequested
        ) {
            running.push(message_id.to_owned());
        }
    }
    Ok(running)
}

/// Gate one claim: slot count and the per-claim time ceiling. Returns the lease to use.
pub(super) fn admit_claim(
    context: &Context,
    requested: Option<u64>,
) -> Result<(u64, Value), MsgError> {
    let limits = worker_limits(context)?;
    let lease = match requested {
        Some(lease) if lease > limits.max_worker_seconds => {
            return Err(MsgError::blocked(
                "hive.msg-worker-time-limit",
                format!(
                    "--lease {lease} exceeds max_worker_seconds = {}; ask for a shorter lease",
                    limits.max_worker_seconds
                ),
            ))
        }
        Some(lease) => lease,
        None => mediation::DEFAULT_CLAIM_LEASE_SECONDS.min(limits.max_worker_seconds),
    };
    let running = running_claims(context)?;
    if running.len() >= limits.max_workers as usize {
        return Err(MsgError::blocked(
            "hive.msg-worker-limit",
            format!(
                "{} claimed messages already run and max_workers = {}; acknowledge or wait for one to end",
                running.len(),
                limits.max_workers
            ),
        ));
    }
    Ok((
        lease,
        json!({"max_workers": limits.max_workers, "max_worker_seconds": limits.max_worker_seconds,
            "source": limits.source, "running_before": running.len()}),
    ))
}

// ---------------------------------------------------------------------------------------------
// Digest-pinned checks.

fn canonical(path: &Path, label: &str) -> Result<PathBuf, MsgError> {
    std::fs::canonicalize(path).map_err(|error| {
        MsgError::input(format!(
            "{label} {} is not readable: {error}",
            path.display()
        ))
    })
}

/// Pin one score/check file outside the worker's writable worktree (default: the target).
pub(super) fn pin_check(
    context: &Context,
    flow_id: &str,
    check: &str,
    worktree: Option<&str>,
) -> Result<CheckPinRecord, MsgError> {
    let worktree = canonical(
        worktree.map_or_else(|| context.target.requested_path(), Path::new),
        "--worktree",
    )?;
    let path = canonical(Path::new(check), "--check")?;
    if !containment::check_is_outside(&path, &worktree) {
        return Err(MsgError::blocked(
            "hive.msg-check-inside-worktree",
            format!(
                "--check {} lies inside the writable worktree {}; a worker could rewrite it, so it is refused for scoring",
                path.display(),
                worktree.display()
            ),
        ));
    }
    let bytes = read_explicit_file(&path, MAX_CHECK_BYTES)?;
    CheckPinRecord {
        schema_version: CONTAINMENT_SCHEMA_VERSION,
        flow_id: flow_id.to_owned(),
        worktree: worktree.to_string_lossy().into_owned(),
        checks: vec![PinnedCheck {
            path: path.to_string_lossy().into_owned(),
            digest: sha256_digest(&bytes),
            bytes: bytes.len() as u64,
        }],
        record_digest: String::new(),
    }
    .seal()
    .map_err(MsgError::input)
}

/// Pinned checks of a flow, if any. An unreadable record is a drift.
fn pins(
    context: &Context,
    flow_id: &str,
) -> Result<Option<Result<CheckPinRecord, String>>, MsgError> {
    Ok(
        read_record::<CheckPinRecord>(&context.target, &containment::check_pin_path(flow_id))?.map(
            |record| {
                record.and_then(|record| {
                    record.validate()?;
                    if record.flow_id == flow_id {
                        Ok(record)
                    } else {
                        Err("check pins are stored under another flow".into())
                    }
                })
            },
        ),
    )
}

/// Re-read every pinned check. Returns the first drift (path, pinned, observed digest).
fn observe_drift(
    context: &Context,
    flow_id: &str,
) -> Result<Option<(String, String, Option<String>)>, MsgError> {
    match pins(context, flow_id)? {
        None => Ok(None),
        Some(Err(reason)) => Ok(Some((reason, String::new(), None))),
        Some(Ok(record)) => {
            for check in &record.checks {
                let observed = read_explicit_file(Path::new(&check.path), MAX_CHECK_BYTES)
                    .ok()
                    .map(|bytes| sha256_digest(&bytes));
                if observed.as_deref() != Some(check.digest.as_str()) {
                    return Ok(Some((check.path.clone(), check.digest.clone(), observed)));
                }
            }
            Ok(None)
        }
    }
}

/// Fail closed on a changed check: record the drift once (no-clobber) and refuse to continue.
pub(super) fn verify_checks(context: &Context, flow_id: &str) -> Result<(), MsgError> {
    if let Some(drift) = drift_view(context, flow_id)?.filter(Value::is_object) {
        return Err(MsgError::blocked(
            "hive.msg-flow-check-drift",
            format!(
                "pinned check {} changed after the flow started; the flow is quarantined and never continues",
                drift["path"].as_str().unwrap_or("-")
            ),
        ));
    }
    let Some((path, pinned_digest, observed_digest)) = observe_drift(context, flow_id)? else {
        return Ok(());
    };
    let record = CheckDrift {
        schema_version: CONTAINMENT_SCHEMA_VERSION,
        flow_id: flow_id.to_owned(),
        path: path.clone(),
        pinned_digest,
        observed_digest,
        detected_at: context.now,
    };
    publish_new(
        &context.target,
        &containment::check_drift_path(flow_id),
        &record_bytes(&record)?,
    )?;
    Err(MsgError::blocked(
        "hive.msg-flow-check-drift",
        format!(
            "pinned check {path} changed after the flow started; the flow is quarantined and never continues"
        ),
    ))
}

/// Recorded drift of a flow (`None` when none was recorded).
pub(super) fn drift_view(context: &Context, flow_id: &str) -> Result<Option<Value>, MsgError> {
    Ok(
        read_record::<CheckDrift>(&context.target, &containment::check_drift_path(flow_id))?.map(
            |record| match record {
                Ok(record) => json!(record),
                Err(reason) => json!({"path": reason}),
            },
        ),
    )
}

/// Checks view for a flow: pinned inputs, recorded or live drift.
pub(super) fn checks_view(context: &Context, flow_id: &str) -> Result<Value, MsgError> {
    let pinned = match pins(context, flow_id)? {
        None => return Ok(Value::Null),
        Some(Ok(record)) => json!(record.checks),
        Some(Err(reason)) => json!({"invalid": reason}),
    };
    let recorded = drift_view(context, flow_id)?;
    let live = observe_drift(context, flow_id)?;
    Ok(json!({
        "pinned": pinned,
        "read_only_for_worker": true,
        "drift": recorded.is_some() || live.is_some(),
        "drift_record": recorded,
    }))
}
