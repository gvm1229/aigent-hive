//! `hive stop` and `hive msg cancel-ack` (OV160-005).
//!
//! `hive stop` publishes one cancel event (`.hive/mediation/stops/<stop-id>.json`, no-clobber).
//! While it has no clear record, no new envelope, flow step, requeue, claim or hook continuation
//! is created in its scope. Claimed work is never interrupted: it is shown `cancel-requested`
//! and ends only by the host's official cancel receipt (`msg cancel-ack`) or by its lease
//! (`dispatch-uncertain`). Hosts without a known official cancel are shown `unsupported`.
//! Hive sends no keys, kills no process and never claims a forced cancel. Resuming needs an
//! explicit `hive stop clear --confirm <stop-digest>`.
use super::{
    check_claim, checked_id, failure, list_entries, load, locate, option, publish_new, read_record,
    record_bytes, success, view, Context, EntryKind, MsgError, Published,
};
use crate::run::{parse_options, read_explicit_file, required, PinnedTarget};
use crate::{emit_action_result, ActionResult};
use hive_core::containment::{
    self, host_cancel_support, CancelReceipt, CancelSupport, StopClear, StopRecord, StopScope,
    CONTAINMENT_SCHEMA_VERSION, STOPS_ROOT,
};
use hive_core::flow::{self, FlowMessageLink};
use hive_core::mediation::{self, MessageEnvelope, MessageState};
use serde_json::{json, Value};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "\
Stop new mediated work. A stop is a cancel event: no new message, flow step, requeue, claim or
hook continuation is created in its scope until it is cleared. Claimed work is not interrupted:
it shows cancel-requested and ends only by the host's official cancel receipt or by its lease.
Hive sends no keys, kills no process and never claims a forced cancel.

USAGE:
    hive stop [--flow <flow-id> | --task <task-id>] [--reason <text>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive stop status [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive stop clear [--flow <flow-id> | --task <task-id>] [--confirm <stop-digest>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg cancel-ack --id <message-id> --claim <claim-token|claim-digest> --receipt <host-cancel-receipt.json> [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]

Without a scope the whole project stops. `stop clear` without --confirm only previews the
digest to confirm. A host with no official cancel is shown `unsupported`; its claimed work
stays cancel-requested until its lease ends, and any later result is quarantined.
";

/// `hive stop ...` process entry.
pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = execute(arguments).unwrap_or_else(|error| failure(error, None));
    emit_action_result(&result)
}

pub(super) fn execute(arguments: &[String]) -> Result<ActionResult, MsgError> {
    match arguments.first().map(String::as_str) {
        Some("status") => status(&arguments[1..]),
        Some("clear") => clear(&arguments[1..]),
        Some("request") => request(&arguments[1..]),
        Some(flag) if flag.starts_with("--") => request(arguments),
        None => request(arguments),
        Some(other) => Err(MsgError::input(format!(
            "unknown stop command: {other}; use status or clear, or give only options to stop"
        ))),
    }
}

// ---------------------------------------------------------------------------------------------
// Reading stops.

/// Every stop record on disk. Unreadable records fail closed: they cover everything.
pub(super) struct Stops {
    pub(super) active: Vec<StopRecord>,
    pub(super) cleared: Vec<StopRecord>,
    pub(super) invalid: Vec<(String, String)>,
}

impl Stops {
    /// First active stop covering `flow`/`task`; an invalid record covers everything.
    pub(super) fn covering(&self, flow: Option<&str>, task: Option<&str>) -> Option<String> {
        if let Some((name, _)) = self.invalid.first() {
            return Some(name.clone());
        }
        self.active
            .iter()
            .find(|stop| stop.covers(flow, task))
            .map(|stop| stop.stop_id.clone())
    }

    fn has_flow_scope(&self) -> bool {
        self.active.iter().any(|stop| stop.scope == StopScope::Flow)
    }
}

pub(super) fn stops(context: &Context) -> Result<Stops, MsgError> {
    stops_in(&context.target)
}

/// Active project-wide (or unreadable) stop for callers outside `msg`, such as `hive summon`.
pub(crate) fn project_stop(target: &PinnedTarget) -> Result<Option<String>, String> {
    stops_in(target)
        .map(|stops| stops.covering(None, None))
        .map_err(|error| error.message)
}

fn stops_in(target: &PinnedTarget) -> Result<Stops, MsgError> {
    let mut found = Stops {
        active: Vec::new(),
        cleared: Vec::new(),
        invalid: Vec::new(),
    };
    for (name, kind) in list_entries(target, Path::new(STOPS_ROOT))?.unwrap_or_default() {
        if name.ends_with(".clear.json") {
            continue;
        }
        let Some(stop_id) = name.strip_suffix(".json") else {
            continue;
        };
        if kind != EntryKind::File || !mediation::is_valid_id(stop_id) {
            found
                .invalid
                .push((name.clone(), "stop entry is not a plain file".into()));
            continue;
        }
        let record = read_record::<StopRecord>(target, &containment::stop_path(stop_id))?
            .unwrap_or_else(|| Err("stop record disappeared".into()))
            .and_then(|record| {
                record.validate()?;
                if record.stop_id == stop_id {
                    Ok(record)
                } else {
                    Err("stop record is stored under another id".into())
                }
            });
        let record = match record {
            Ok(record) => record,
            Err(reason) => {
                found.invalid.push((stop_id.to_owned(), reason));
                continue;
            }
        };
        match read_record::<StopClear>(target, &containment::stop_clear_path(stop_id))? {
            None => found.active.push(record),
            Some(Ok(clear)) if clear.validate(&record).is_ok() => found.cleared.push(record),
            Some(_) => found.invalid.push((
                stop_id.to_owned(),
                "stop clear does not bind its stop".into(),
            )),
        }
    }
    Ok(found)
}

/// Refuse any new envelope, step, requeue or claim inside an active stop.
pub(super) fn gate(
    context: &Context,
    flow: Option<&str>,
    task: Option<&str>,
) -> Result<(), MsgError> {
    if let Some(stop_id) = stops(context)?.covering(flow, task) {
        return Err(MsgError::blocked(
            "hive.msg-stopped",
            format!(
                "stop {stop_id} is active: no new message, step, requeue or claim is created; clear it with `hive stop clear --confirm <stop-digest>`"
            ),
        ));
    }
    Ok(())
}

/// Active stop covering one message: the project, its task or its flow.
pub(super) fn covering_message(
    context: &Context,
    envelope: &MessageEnvelope,
) -> Result<Option<String>, MsgError> {
    let stops = stops(context)?;
    if stops.active.is_empty() && stops.invalid.is_empty() {
        return Ok(None);
    }
    let flow = if stops.has_flow_scope() {
        read_record::<FlowMessageLink>(
            &context.target,
            &flow::message_link_path(&envelope.message_id),
        )?
        .and_then(Result::ok)
        .map(|link| link.flow_id)
    } else {
        None
    };
    Ok(stops.covering(flow.as_deref(), envelope.recipient_task.as_deref()))
}

// ---------------------------------------------------------------------------------------------
// Views.

fn scope(
    context: &Context,
    options: &[(&str, &str)],
) -> Result<(StopScope, Option<String>), MsgError> {
    match (option(options, "--flow"), option(options, "--task")) {
        (Some(_), Some(_)) => Err(MsgError::input("use only one of --flow or --task")),
        (Some(flow_id), None) => {
            let flow_id = checked_id(flow_id, "--flow")?;
            if context
                .target
                .read_optional(
                    Path::new(&flow::flow_path(&flow_id)),
                    super::MAX_RECORD_BYTES,
                )?
                .is_none()
            {
                return Err(MsgError::new(
                    "error",
                    2,
                    "hive.msg-flow-not-found",
                    format!("flow not found: {flow_id}"),
                ));
            }
            Ok((StopScope::Flow, Some(flow_id)))
        }
        (None, Some(task)) => Ok((StopScope::Task, Some(checked_id(task, "--task")?))),
        (None, None) => Ok((StopScope::Project, None)),
    }
}

/// Claimed work inside a stop, with what the host can and cannot do about it.
fn in_progress(context: &Context, stop: &StopRecord) -> Result<Vec<Value>, MsgError> {
    let mut entries = Vec::new();
    for message_id in super::containment::running_claims(context)? {
        let role = locate(context, &message_id)?;
        let loaded = load(context, &role, &message_id)?;
        let Some(envelope) = loaded.parsed.as_ref() else {
            continue;
        };
        if loaded.stopped_by.as_deref() != Some(stop.stop_id.as_str()) {
            continue;
        }
        let provider = context
            .sender(&role)
            .map_or_else(|_| "unknown".to_owned(), |sender| sender.provider);
        let (support, reason) = host_cancel_support(&provider);
        entries.push(json!({
            "message_id": message_id,
            "role": role,
            "provider": provider,
            "state": loaded.state(context.now).0,
            "envelope_digest": envelope.envelope_digest,
            "claim_digest": loaded.claim.as_ref().and_then(|c| c.as_ref().ok()).and_then(|c| c.digest().ok()),
            "lease_until": loaded.claim.as_ref().and_then(|c| c.as_ref().ok()).map(|c| c.lease_until),
            "host_cancel": support.as_str(),
            "host_cancel_reason": reason,
            "interrupted_by_hive": false,
            "next": match support {
                CancelSupport::Unsupported => "Hive cannot cancel this work; it stays cancel-requested until its lease ends, and any later result is quarantined",
                CancelSupport::Unverified => "the host may cancel with its own official control, then record the receipt with `hive msg cancel-ack`",
            },
        }));
    }
    Ok(entries)
}

fn stop_view(context: &Context, stop: &StopRecord) -> Result<Value, MsgError> {
    let in_progress = in_progress(context, stop)?;
    let unsupported = in_progress
        .iter()
        .filter(|entry| entry["host_cancel"] == "unsupported")
        .count();
    Ok(json!({
        "record": stop,
        "path": containment::stop_path(&stop.stop_id),
        "stop_digest": stop.record_digest,
        "new_envelopes": "refused",
        "in_progress": in_progress,
        "unsupported_cancel": unsupported,
        "forced": false,
        "workers_interrupted": 0,
        "keystrokes_sent": 0,
    }))
}

// ---------------------------------------------------------------------------------------------
// Commands.

const OPTIONS: &[&str] = &["--flow", "--task", "--now", "--target", "--user-root"];

fn request(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let mut allowed = OPTIONS.to_vec();
    allowed.push("--reason");
    let options = parse_options(arguments, &allowed)?;
    let context = Context::open(&options)?;
    let (scope, scope_id) = scope(&context, &options)?;
    let found = stops(&context)?;
    if let Some(existing) = found
        .active
        .iter()
        .find(|stop| stop.scope == scope && stop.scope_id == scope_id)
    {
        return Ok(success(
            "hive.stop-already-active",
            "this scope is already stopped; nothing new was written",
            json!({"stop": stop_view(&context, existing)?, "duplicate": true}),
        ));
    }
    let generation = found
        .cleared
        .iter()
        .filter(|stop| stop.scope == scope && stop.scope_id == scope_id)
        .map(|stop| stop.generation)
        .max()
        .unwrap_or(0)
        + 1;
    let record = StopRecord {
        schema_version: CONTAINMENT_SCHEMA_VERSION,
        stop_id: containment::stop_id_for(scope, scope_id.as_deref(), generation),
        scope,
        scope_id,
        generation,
        reason: option(&options, "--reason").map(str::to_owned),
        requested_at: context.now,
        forced: false,
        record_digest: String::new(),
    }
    .seal()
    .map_err(MsgError::input)?;
    let path = containment::stop_path(&record.stop_id);
    let mut changed = Vec::new();
    match publish_new(&context.target, &path, &record_bytes(&record)?)? {
        Published::Created => changed.push(path),
        Published::SameBytes => {}
        Published::Different(_) => {
            return Err(MsgError::conflict(
                "hive.msg-conflict",
                "a concurrent stop of this scope was written first; run `hive stop status`",
            ))
        }
    }
    let view = stop_view(&context, &record)?;
    let unsupported = view["unsupported_cancel"].as_u64().unwrap_or(0);
    let running = view["in_progress"].as_array().map_or(0, Vec::len);
    let mut result = success(
        "hive.stop-requested",
        format!(
            "stop recorded: new envelopes are refused; {running} claimed message(s) are cancel-requested and were not interrupted ({unsupported} on hosts whose official cancel is unsupported)"
        ),
        json!({"stop": view, "duplicate": false}),
    );
    result.changed_paths = changed;
    result.next_action = Some(format!(
        "Resume only with: hive stop clear --confirm {}",
        record.record_digest
    ));
    Ok(result)
}

fn status(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(arguments, &["--now", "--target", "--user-root"])?;
    let context = Context::open(&options)?;
    let found = stops(&context)?;
    let active = found
        .active
        .iter()
        .map(|stop| stop_view(&context, stop))
        .collect::<Result<Vec<_>, _>>()?;
    let invalid: Vec<Value> = found
        .invalid
        .iter()
        .map(
            |(name, reason)| json!({"entry": name, "reason": reason, "treated_as": "project stop"}),
        )
        .collect();
    let stopped = !active.is_empty() || !invalid.is_empty();
    Ok(success(
        "hive.stop-status",
        if stopped {
            format!(
                "{} active stop(s); new envelopes in their scope are refused",
                active.len() + invalid.len()
            )
        } else {
            "no active stop".to_owned()
        },
        json!({"stopped": stopped, "active": active, "invalid": invalid,
            "cleared": found.cleared.len()}),
    ))
}

fn clear(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let mut allowed = OPTIONS.to_vec();
    allowed.push("--confirm");
    let options = parse_options(arguments, &allowed)?;
    let context = Context::open(&options)?;
    let (scope, scope_id) = scope(&context, &options)?;
    let found = stops(&context)?;
    let Some(stop) = found
        .active
        .iter()
        .find(|stop| stop.scope == scope && stop.scope_id == scope_id)
    else {
        return Err(MsgError::conflict(
            "hive.stop-clear-refused",
            "no active stop for this scope",
        ));
    };
    let Some(confirm) = option(&options, "--confirm") else {
        let mut result = success(
            "hive.stop-clear-preview",
            "clear preview only; nothing changed",
            json!({"stop": stop_view(&context, stop)?, "confirm": stop.record_digest}),
        );
        result.next_action = Some(format!(
            "To resume, run: hive stop clear --confirm {}",
            stop.record_digest
        ));
        return Ok(result);
    };
    if confirm != stop.record_digest {
        return Err(MsgError::conflict(
            "hive.stop-clear-refused",
            "--confirm must be the active stop digest from `hive stop status`",
        ));
    }
    let clear = StopClear {
        schema_version: CONTAINMENT_SCHEMA_VERSION,
        stop_id: stop.stop_id.clone(),
        stop_digest: stop.record_digest.clone(),
        cleared_at: context.now.max(stop.requested_at),
    };
    let path = containment::stop_clear_path(&stop.stop_id);
    let mut changed = Vec::new();
    if let Published::Created = publish_new(&context.target, &path, &record_bytes(&clear)?)? {
        changed.push(path);
    }
    let mut result = success(
        "hive.stop-cleared",
        "stop cleared; new messages may be created again. Nothing was re-sent automatically",
        json!({"stop_id": stop.stop_id, "clear": clear, "redispatched": false}),
    );
    result.changed_paths = changed;
    Ok(result)
}

/// Record a host's official cancel receipt for one cancel-requested message.
pub(super) fn cancel_ack(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(
        arguments,
        &[
            "--id",
            "--claim",
            "--receipt",
            "--now",
            "--target",
            "--user-root",
        ],
    )?;
    let context = Context::open(&options)?;
    let message_id = checked_id(required(&options, "--id")?, "--id")?;
    let presented = required(&options, "--claim")?;
    let role = locate(&context, &message_id)?;
    let bytes = read_explicit_file(Path::new(required(&options, "--receipt")?), 64 * 1024)?;
    let loaded = load(&context, &role, &message_id)?;
    check_claim(&loaded, presented)?;
    let (state, _) = loaded.state(context.now);
    let refused = |message: String| MsgError::conflict("hive.msg-cancel-refused", message);
    match state {
        MessageState::CancelRequested => {}
        MessageState::Cancelled => {
            return Ok(success(
                "hive.msg-cancel-recorded",
                "the host cancel receipt was already recorded; nothing changed",
                json!({"message": view(&context, &loaded)?, "duplicate": true, "forced": false}),
            ))
        }
        MessageState::Claimed => {
            return Err(refused(
                "no active stop covers this message; run `hive stop` first".into(),
            ))
        }
        other => {
            return Err(refused(format!(
                "only cancel-requested work can be cancelled; the message is {}",
                other.as_str()
            )))
        }
    }
    let provider = context.sender(&role)?.provider;
    let (support, reason) = host_cancel_support(&provider);
    if support == CancelSupport::Unsupported {
        return Err(MsgError::blocked(
            "hive.msg-cancel-unsupported",
            format!("official cancel is unsupported for {provider}: {reason}. Hive records no cancel; the claim ends by its lease"),
        ));
    }
    let receipt: CancelReceipt = serde_json::from_slice(&bytes)
        .map_err(|error| refused(format!("invalid host cancel receipt: {error}")))?;
    let envelope = loaded
        .verified
        .as_ref()
        .map_err(|reason| MsgError::blocked("hive.msg-quarantined", reason.clone()))?;
    let Some(Ok(claim)) = loaded.claim.as_ref() else {
        return Err(refused("claim is missing".into()));
    };
    receipt.validate(envelope, claim).map_err(refused)?;
    if Some(receipt.stop_id.as_str()) != loaded.stopped_by.as_deref() || receipt.host != provider {
        return Err(refused(
            "the receipt must name the active stop and the claimant's provider".into(),
        ));
    }
    let path = containment::cancel_receipt_path(&message_id);
    let mut changed = Vec::new();
    match publish_new(&context.target, &path, &record_bytes(&receipt)?)? {
        Published::Created => changed.push(path),
        Published::SameBytes => {}
        Published::Different(_) => {
            return Err(refused(
                "a different cancel receipt is already recorded".into(),
            ))
        }
    }
    let loaded = load(&context, &role, &message_id)?;
    let mut result = success(
        "hive.msg-cancel-recorded",
        format!("host official cancel receipt recorded ({}; Hive interrupted nothing); any later result is quarantined", support.as_str()),
        json!({"message": view(&context, &loaded)?, "duplicate": false, "forced": false,
            "host_cancel": support.as_str(), "interrupted_by_hive": false}),
    );
    result.changed_paths = changed;
    Ok(result)
}

#[cfg(test)]
pub(super) fn test_execute(arguments: &[String]) -> ActionResult {
    execute(arguments).unwrap_or_else(|error| failure(error, None))
}
