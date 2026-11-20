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
    WorkerPolicy, WorkerSlot, CONTAINMENT_SCHEMA_VERSION,
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
        let Ok(Some(Ok(claim))) = read_record::<mediation::ClaimRecord>(
            &context.target,
            &mediation::claim_path(message_id),
        ) else {
            continue;
        };
        if !mediation::is_valid_id(&claim.claimant_role) {
            continue;
        }
        let envelope = mediation::envelope_path(&claim.claimant_role, message_id);
        if !matches!(
            context
                .target
                .read_optional(Path::new(&envelope), super::MAX_RECORD_BYTES),
            Ok(Some(_))
        ) {
            continue;
        }
        if holds_worker(context, &claim.claimant_role, message_id, None) {
            running.push(message_id.to_owned());
        }
    }
    Ok(running)
}

/// The claim of `message_id` (with `token`, when given) is claimed or cancel-requested inside
/// its lease. A message whose records cannot be read holds nothing.
fn holds_worker(context: &Context, role: &str, message_id: &str, token: Option<&str>) -> bool {
    let Ok(loaded) = load(context, role, message_id) else {
        return false;
    };
    let same_claim = match (token, loaded.claim.as_ref()) {
        (None, _) => true,
        (Some(token), Some(Ok(claim))) => claim.claim_token == token,
        (Some(_), _) => false,
    };
    same_claim
        && matches!(
            loaded.state(context.now).0,
            MessageState::Claimed | MessageState::CancelRequested
        )
}

/// Whether the holder of one slot generation still runs. A reservation whose claim was never
/// published is held until its lease ends (the claimant may still be publishing it); one whose
/// message has another claim lost the claim race and is free at once.
fn slot_held(context: &Context, slot: &WorkerSlot) -> bool {
    // An unreadable claim quarantines its message: it runs nothing.
    let Ok(claim) = read_record::<mediation::ClaimRecord>(
        &context.target,
        &mediation::claim_path(&slot.message_id),
    ) else {
        return false;
    };
    match claim {
        None => context.now < slot.lease_until,
        Some(Ok(claim)) if claim.claim_token == slot.claim_token => holds_worker(
            context,
            &claim.claimant_role,
            &slot.message_id,
            Some(&slot.claim_token),
        ),
        Some(_) => false,
    }
}

fn already_claimed() -> MsgError {
    MsgError::conflict(
        "hive.msg-already-claimed",
        "another consumer claimed this message first",
    )
}

/// Reserve one `max_workers` slot for `claim` before the claim is published (S4).
///
/// Each slot `0..max_workers` is a directory of contiguous generations. A slot is free when it
/// has no generation yet or its newest generation's holder is released; the claimant then
/// publishes the next generation no-clobber, so of two racers exactly one wins that slot.
/// Restart-safe: nothing is ever deleted, and a crashed claimant's reservation ends with its
/// lease. Returns the published path and a view.
pub(super) fn reserve_slot(
    context: &Context,
    claim: &mediation::ClaimRecord,
) -> Result<(String, Value), MsgError> {
    let limits = worker_limits(context)?;
    for slot in 0..limits.max_workers {
        let directory = containment::slot_dir(slot);
        let generations: Vec<u64> = list_entries(&context.target, Path::new(&directory))?
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, kind)| *kind == EntryKind::File)
            .filter_map(|(name, _)| name.strip_suffix(".json")?.parse().ok())
            .collect();
        let next = match generations.iter().max() {
            None => 0,
            Some(&newest) => {
                let path = containment::slot_path(slot, newest);
                let held = match read_record::<WorkerSlot>(&context.target, &path)? {
                    Some(Ok(holder)) if holder.validate().is_ok() && holder.slot == slot => {
                        let held = slot_held(context, &holder);
                        // Another claimant of this same message is publishing its claim: this
                        // claimant lost the claim race, not the worker limit.
                        if held
                            && holder.message_id == claim.message_id
                            && holder.claim_token != claim.claim_token
                        {
                            return Err(already_claimed());
                        }
                        held
                    }
                    // A damaged reservation fails closed until its slot is inspected.
                    _ => true,
                };
                if held {
                    continue;
                }
                newest + 1
            }
        };
        let record = WorkerSlot {
            schema_version: CONTAINMENT_SCHEMA_VERSION,
            slot,
            generation: next,
            message_id: claim.message_id.clone(),
            claim_token: claim.claim_token.clone(),
            reserved_at: claim.claimed_at,
            lease_until: claim.lease_until,
        };
        record.validate().map_err(MsgError::input)?;
        let path = containment::slot_path(slot, next);
        if let Published::Created = publish_new(&context.target, &path, &record_bytes(&record)?)? {
            return Ok((
                path.clone(),
                json!({"slot": slot, "generation": next, "path": path,
                    "max_workers": limits.max_workers}),
            ));
        }
        // Another claimant won this generation. If it claims this same message, this claimant
        // lost the claim race; otherwise try the next slot.
        if let Some(Ok(winner)) = read_record::<WorkerSlot>(&context.target, &path)? {
            if winner.message_id == claim.message_id && winner.claim_token != claim.claim_token {
                return Err(already_claimed());
            }
        }
    }
    // Every slot is held. If this message was claimed meanwhile, report the lost claim race.
    if read_record::<mediation::ClaimRecord>(
        &context.target,
        &mediation::claim_path(&claim.message_id),
    )?
    .is_some()
    {
        return Err(already_claimed());
    }
    Err(MsgError::blocked(
        "hive.msg-worker-limit",
        format!(
            "every worker slot is held and max_workers = {}; acknowledge or wait for one to end",
            limits.max_workers
        ),
    ))
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
