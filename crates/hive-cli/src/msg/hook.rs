//! `hive msg hook`: the command an approved project hook runs for one official host event.
//!
//! It reads the hook stdin, writes one event record under `.hive/mediation/events/`, and prints
//! exactly one host-native JSON document. It never waits for a worker, never claims or
//! dispatches a message, never launches Codex, Gemini or any other process, and never reads the
//! transcript. Every path exits 0 with nothing on stderr, so a normal return is never shown as a
//! host "hook error"; a refused event prints `{}` and is kept as a quarantined record.
//!
//! An acknowledged result for the bound role is handed back once: its delivery record is
//! published no-clobber and names this event. A byte-identical replay of the same event prints
//! the stored response again, so it receives the same receipt with no new delivery, claim or
//! model call. A later different event of any session gets no second copy of that result.
use super::hooks::{self, HookIntent};
use super::{
    list_entries, load, option, publish_new, record_bytes, Context, EntryKind, MsgError, Published,
};
use crate::run::{parse_options, required};
use hive_core::hook_events::{
    self, AllowedEvent, DeliveredResult, DeliveryRecord, EventDisposition, HookEventRecord,
    HookHost, NormalizedPayload, Notice, ReturnShape, HOOK_EVENT_SCHEMA_VERSION,
    MAX_HOOK_INPUT_BYTES, MAX_RESULTS_PER_RESPONSE,
};
use hive_core::mediation::{self, MessageState, MAX_RESULT_BYTES, MEDIATION_ROOT, UNTRUSTED_DATA};
use hive_core::sha256_digest;
use std::io::{Read, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const NEUTRAL: &[u8] = b"{}\n";
const MAX_RESPONSE_BYTES: usize = 128 * 1024;

pub(super) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Hook process entry. Always exit 0, always one JSON document, never stderr.
pub(super) fn run(arguments: &[String]) -> ExitCode {
    let received_at_ms = now_ms();
    let mut input = Vec::new();
    let read = std::io::stdin()
        .take(MAX_HOOK_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut input);
    let response = if read.is_ok() {
        handle(arguments, &input, received_at_ms).map_or_else(|_| NEUTRAL.to_vec(), |o| o.response)
    } else {
        NEUTRAL.to_vec()
    };
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(&response);
    let _ = stdout.flush();
    ExitCode::SUCCESS
}

/// What one hook call did; the response bytes are exactly what the host reads. The other
/// fields are the test-visible record of the call; the process entry prints only `response`.
#[derive(Debug)]
#[cfg_attr(not(test), allow(dead_code))]
pub(super) struct Outcome {
    pub(super) response: Vec<u8>,
    pub(super) event_id: String,
    pub(super) disposition: EventDisposition,
    pub(super) replayed: bool,
    pub(super) record_path: String,
}

struct Call<'a> {
    context: Context,
    host: HookHost,
    event: &'a str,
    role: Option<String>,
    task: Option<String>,
    binding: Option<String>,
    input: &'a [u8],
    received_at_ms: u64,
}

pub(super) fn handle(
    arguments: &[String],
    input: &[u8],
    received_at_ms: u64,
) -> Result<Outcome, MsgError> {
    let options = parse_options(
        arguments,
        &[
            "--host",
            "--event",
            "--role",
            "--task",
            "--binding",
            "--now",
            "--target",
            "--user-root",
        ],
    )?;
    let host = HookHost::parse(required(&options, "--host")?).map_err(MsgError::input)?;
    let event = required(&options, "--event")?;
    let context = Context::open(&options)?;
    let mut call = Call {
        context,
        host,
        event,
        role: None,
        task: None,
        binding: option(&options, "--binding").map(str::to_owned),
        input,
        received_at_ms,
    };
    let Some(allowed) = hook_events::allowed_event(host, event) else {
        return quarantine(
            &call,
            None,
            "event is not in the official allowlist for this host",
        );
    };
    // Role and task come only from the approved command, resolved through the roster.
    match option(&options, "--role").map(|role| call.context.role(role)) {
        Some(Ok(role)) => call.role = Some(role),
        _ => return quarantine(&call, None, "role does not resolve through the roster"),
    }
    match option(&options, "--task") {
        Some(task) if mediation::is_valid_id(task) => call.task = Some(task.to_owned()),
        Some(_) => return quarantine(&call, None, "task id is not a plain id"),
        None => {}
    }
    let payload = match hook_events::normalize_payload(allowed, input) {
        Ok(payload) => payload,
        Err(reason) => return quarantine(&call, None, &reason),
    };
    let intent = match authority(&call, &options) {
        Ok(intent) => intent,
        Err(reason) => return quarantine(&call, Some(&payload), &reason),
    };
    respond(&call, allowed, &payload, &intent)
}

/// The command's binding must name the current approved receipt for this exact scope.
fn authority(call: &Call<'_>, options: &[(&str, &str)]) -> Result<HookIntent, String> {
    let intent = hooks::installed(&call.context, call.host)
        .map_err(|error| error.message)?
        .filter(|intent| intent.operation == "install")
        .ok_or("no approved project mediation hooks for this host")?;
    let user_root = hooks::user_root(options).map_err(|error| error.message)?;
    let expected = hook_events::binding_digest(
        call.host,
        &intent.role,
        intent.task.as_deref(),
        intent.stop_shape,
        &hooks::path_digest(call.context.target.requested_path()),
        &hooks::path_digest(&user_root),
    )?;
    if call.binding.as_deref() != Some(intent.binding_digest.as_str())
        || intent.binding_digest != expected
        || call.role.as_deref() != Some(intent.role.as_str())
        || call.task != intent.task
    {
        return Err("hook binding does not match the approved project hook configuration".into());
    }
    Ok(intent)
}

fn quarantine(
    call: &Call<'_>,
    payload: Option<&NormalizedPayload>,
    reason: &str,
) -> Result<Outcome, MsgError> {
    let payload_digest = sha256_digest(call.input);
    let session = payload.map(|payload| payload.session_digest.clone());
    let event_id = hook_events::event_id_for(
        call.host,
        call.event,
        session.as_deref().unwrap_or("no-session"),
        &payload_digest,
    );
    // Unknown event names are not echoed into the record verbatim.
    let event = if mediation::is_valid_id(call.event) {
        call.event.to_owned()
    } else {
        "unrecognized".to_owned()
    };
    let record = HookEventRecord {
        schema_version: HOOK_EVENT_SCHEMA_VERSION,
        event_id: event_id.clone(),
        host: call.host,
        event,
        role: call.role.clone(),
        task: call.task.clone(),
        binding_digest: call
            .binding
            .clone()
            .filter(|binding| hive_core::policy::valid_digest(binding)),
        session_digest: session,
        payload_digest,
        payload_bytes: call.input.len() as u64,
        stop_hook_active: payload.is_some_and(|payload| payload.stop_hook_active),
        disposition: EventDisposition::Quarantined,
        reason: Some(reason.chars().take(240).collect()),
        received_at_ms: call.received_at_ms,
        recorded_at_ms: now_ms().max(call.received_at_ms),
        delivered: Vec::new(),
        pending: Vec::new(),
        withheld: Vec::new(),
        redispatched: false,
    };
    let record_path = hook_events::event_path(record.session_digest.as_deref(), &event_id);
    if record.validate().is_ok() {
        publish_new(&call.context.target, &record_path, &record_bytes(&record)?)?;
    }
    Ok(Outcome {
        response: NEUTRAL.to_vec(),
        event_id,
        disposition: EventDisposition::Quarantined,
        replayed: false,
        record_path,
    })
}

fn respond(
    call: &Call<'_>,
    allowed: &AllowedEvent,
    payload: &NormalizedPayload,
    intent: &HookIntent,
) -> Result<Outcome, MsgError> {
    let target = &call.context.target;
    let event_id = hook_events::event_id_for(
        call.host,
        call.event,
        &payload.session_digest,
        &payload.payload_digest,
    );
    let record_path = hook_events::event_path(Some(&payload.session_digest), &event_id);
    let response_path = hook_events::response_path(&event_id);
    // Replay of the same event: the stored bytes, nothing new.
    if let Some(stored) = target.read_optional(Path::new(&response_path), MAX_RESPONSE_BYTES)? {
        return Ok(Outcome {
            response: stored,
            event_id,
            disposition: EventDisposition::Recorded,
            replayed: true,
            record_path,
        });
    }
    let shape = match (allowed.shape, intent.stop_shape) {
        (ReturnShape::StopContext | ReturnShape::StopBlock, Some(chosen))
            if chosen == allowed.shape || allowed.alternatives.contains(&chosen) =>
        {
            chosen
        }
        (shape, _) => shape,
    };
    let (disposition, notice) = if payload.stop_hook_active {
        // The host is already continuing because of a hook: answer neutrally, deliver nothing.
        (EventDisposition::Recursive, Notice::default())
    } else if shape == ReturnShape::RecordOnly {
        (EventDisposition::Recorded, Notice::default())
    } else {
        (
            EventDisposition::Recorded,
            collect(call, &event_id, &payload.session_digest)?,
        )
    };
    let record = HookEventRecord {
        schema_version: HOOK_EVENT_SCHEMA_VERSION,
        event_id: event_id.clone(),
        host: call.host,
        event: allowed.event.to_owned(),
        role: call.role.clone(),
        task: call.task.clone(),
        binding_digest: Some(intent.binding_digest.clone()),
        session_digest: Some(payload.session_digest.clone()),
        payload_digest: payload.payload_digest.clone(),
        payload_bytes: payload.payload_bytes,
        stop_hook_active: payload.stop_hook_active,
        disposition,
        reason: None,
        received_at_ms: call.received_at_ms,
        recorded_at_ms: now_ms().max(call.received_at_ms),
        delivered: notice
            .delivered
            .iter()
            .map(|item| item.message_id.clone())
            .collect(),
        pending: notice.pending.clone(),
        withheld: notice.withheld.iter().map(|(id, _)| id.clone()).collect(),
        redispatched: false,
    };
    record.validate().map_err(MsgError::input)?;
    // An interrupted earlier call may already hold this record; keep the first bytes.
    publish_new(target, &record_path, &record_bytes(&record)?)?;
    let mut response =
        serde_json::to_vec(&hook_events::render_response(shape, allowed.event, &notice)).map_err(
            |error| MsgError::new("error", 10, "hive.internal-error", error.to_string()),
        )?;
    response.push(b'\n');
    let response = match publish_new(target, &response_path, &response)? {
        Published::Created | Published::SameBytes => response,
        Published::Different(existing) => existing,
    };
    Ok(Outcome {
        response,
        event_id,
        disposition,
        replayed: false,
        record_path,
    })
}

/// Messages sent by the bound role (and task): deliver acknowledged results once, report
/// queued or claimed ones as pending, and withhold everything that is not deliverable.
fn collect(call: &Call<'_>, event_id: &str, session_digest: &str) -> Result<Notice, MsgError> {
    let context = &call.context;
    let role = call.role.as_deref().unwrap_or_default();
    let mut notice = Notice {
        inbox: format!("{MEDIATION_ROOT}/inbox/"),
        ..Notice::default()
    };
    let inbox = Path::new(MEDIATION_ROOT).join("inbox");
    let mut found = Vec::new();
    for (recipient, kind) in list_entries(&context.target, &inbox)?.unwrap_or_default() {
        if kind != EntryKind::Dir || !mediation::is_valid_id(&recipient) {
            continue;
        }
        for (name, kind) in
            list_entries(&context.target, &inbox.join(&recipient))?.unwrap_or_default()
        {
            let Some(message_id) = name.strip_suffix(".json") else {
                continue;
            };
            if kind != EntryKind::File || !mediation::is_valid_id(message_id) {
                continue;
            }
            let loaded = load(context, &recipient, message_id)?;
            let Some(envelope) = loaded.parsed.clone() else {
                continue;
            };
            if envelope.sender.role != role
                || call
                    .task
                    .as_ref()
                    .is_some_and(|task| envelope.recipient_task.as_ref() != Some(task))
            {
                continue;
            }
            found.push((envelope.sequence, message_id.to_owned(), loaded));
        }
    }
    found.sort_by_key(|(sequence, _, _)| *sequence);
    for (_, message_id, loaded) in found {
        // Inside an active stop nothing is handed back, so the host is never asked to continue.
        if loaded.stopped_by.is_some() {
            notice.withheld.push((message_id, "stopped".to_owned()));
            continue;
        }
        let (state, _) = loaded.state(context.now);
        match state {
            MessageState::Queued | MessageState::Claimed => notice.pending.push(message_id),
            MessageState::Expired
            | MessageState::DispatchUncertain
            | MessageState::Quarantined
            | MessageState::CancelRequested
            | MessageState::Cancelled => {
                notice
                    .withheld
                    .push((message_id, state.as_str().to_owned()));
            }
            MessageState::Acknowledged => {
                if notice.delivered.len() >= MAX_RESULTS_PER_RESPONSE {
                    continue;
                }
                if let Some(result) = deliver(call, &message_id, event_id, session_digest)? {
                    notice.delivered.push(result);
                }
            }
        }
    }
    Ok(notice)
}

/// Publish the one delivery record for an acknowledged message. `None` when another event
/// already received it.
fn deliver(
    call: &Call<'_>,
    message_id: &str,
    event_id: &str,
    session_digest: &str,
) -> Result<Option<DeliveredResult>, MsgError> {
    let target = &call.context.target;
    let receipt_file = mediation::receipt_path(message_id);
    let result_file = mediation::result_path(message_id);
    let Some(receipt) = target.read_optional(Path::new(&receipt_file), 64 * 1024)? else {
        return Ok(None);
    };
    let Some(result) = target.read_optional(Path::new(&result_file), MAX_RESULT_BYTES)? else {
        return Ok(None);
    };
    let record = DeliveryRecord {
        schema_version: HOOK_EVENT_SCHEMA_VERSION,
        message_id: message_id.to_owned(),
        event_id: event_id.to_owned(),
        host: call.host,
        event: call.event.to_owned(),
        session_digest: session_digest.to_owned(),
        receipt_file: receipt_file.clone(),
        receipt_digest: sha256_digest(&receipt),
        result_file: result_file.clone(),
        result_digest: sha256_digest(&result),
        delivered_at_ms: now_ms(),
        trust: UNTRUSTED_DATA.to_owned(),
    };
    record.validate().map_err(MsgError::input)?;
    let path = hook_events::delivery_path(message_id);
    let ours = match publish_new(target, &path, &record_bytes(&record)?)? {
        Published::Created | Published::SameBytes => true,
        // Resuming an interrupted call of this same event keeps its delivery.
        Published::Different(existing) => serde_json::from_slice::<DeliveryRecord>(&existing)
            .is_ok_and(|existing| existing.event_id == event_id),
    };
    if !ours {
        return Ok(None);
    }
    Ok(Some(DeliveredResult {
        message_id: message_id.to_owned(),
        receipt_file,
        result_file,
        result_digest: record.result_digest,
        text: std::str::from_utf8(&result).ok().map(str::to_owned),
    }))
}
