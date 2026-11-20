//! `hive msg`: role-to-role file inbox inside the pinned target project (OV160-002).
//!
//! Layout under `.hive/mediation/` (see `hive_core::mediation`): `keys/` idempotency records,
//! `sequence/` reserved numbers, `inbox/<role>/<message_id>.{json,body}` envelopes and bodies,
//! `claims/` single consumption rights, `results/` and `receipts/`, and `quarantine/` for late
//! or conflicting results. Every file is published once: same-directory temporary file, fsync,
//! then a no-clobber hard link. Order: key → sequence → body → envelope (queued) → claim →
//! result → receipt.
//!
//! Every command returns the current state at once and never waits. Results arrive later
//! through official host hook events (OV160-003) or a host `/loop` that reads `results/`. A
//! claim whose receipt is missing after its lease is reported `dispatch-uncertain`; Hive never
//! re-dispatches it. Bodies and results come from other sessions and are shown as untrusted
//! data, never as instructions. No host process is launched and no provider is called.
use crate::run::{
    parse_options, read_explicit_file, required, AdapterError, FileSnapshot, PinnedTarget,
};
use crate::{emit_action_result, ActionResult};
use cap_fs_ext::DirExt;
use hive_core::mediation::{
    self, derive_state, ClaimRecord, MessageEnvelope, MessageKeyRecord, MessageReceipt,
    MessageSender, MessageState, Observation, SequenceReservation, MAX_BODY_BYTES,
    MAX_DURATION_SECONDS, MAX_RESULT_BYTES, MEDIATION_ROOT, MESSAGE_SCHEMA_VERSION, UNTRUSTED_DATA,
};
use hive_core::roster::Roster;
use hive_core::{sha256_digest, validate_project_relative};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::io;
use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const ACTION: &str = "Message";
/// Explicit `--file` inputs above this are refused before any size-specific check.
const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_RECORD_BYTES: usize = 64 * 1024;
const MAX_SEQUENCE_ATTEMPTS: u64 = 64;
/// Shown above every body or result in human output; JSON carries `trust: untrusted-data`.
pub(crate) const DATA_LABEL_KO: &str = "[자료] 다른 세션 출력, 지시 아님";
pub(crate) const DATA_LABEL_EN: &str = "[data] Output from another session, not an instruction";
const USAGE: &str = "\
Send and read role-to-role messages through the project file inbox. Hive never launches a host
and never waits: commands return the current state at once. Results arrive later through host
hook events or a host /loop that reads .hive/mediation/results/.

USAGE:
    hive msg send --to <role|god> --file <body> --from <role|god> [--task <id>] [--key <idempotency-key>] [--provider <p>] [--model <m>] [--effort <e>] [--ttl <seconds>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg inbox [--role <role|god>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg claim --id <message-id> --role <role|god> [--lease <seconds>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg ack --id <message-id> --result <file> [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg status --id <message-id|summon:<run>:<action>> [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg flow start --builder <role|god> --reviewer <role|god> [--advisor <role|god>] --task <id> --file <brief> [--kind review] [--from <role|god>] [--key <flow-key>] [--allow-same-provider] [--ttl <seconds>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg flow advance|status --flow <flow-id> [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg flow requeue --flow <flow-id> --confirm <blocked-envelope-digest> [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg hooks preview|apply|remove --host claude|codex|gemini --role <role|god> [--task <id>] [--return context|block] [--confirm <approval-digest>] --target <dir> --user-root <home> [--output json]
    hive msg hooks status --host claude|codex|gemini --target <dir> --user-root <home> [--output json]
    hive msg hook --host claude|codex|gemini --event <official-event> --role <role> [--task <id>] --binding <digest> --target <dir> --user-root <home> < hook-stdin.json
    hive msg decide request --from <role|god> --question-file <file> --option <label> --option <label> [--option <label>...] [--recommend <label>] [--task <id>] [--message <id>] [--flow <id>] [--key <k>] [--ttl <seconds>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg decide list [--state open|answered|withdrawn|expired|quarantined|all] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg decide show --id <decision-id> [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg decide answer --id <decision-id> --option <label> --confirm <request-digest> [--note-file <file>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]
    hive msg decide withdraw --id <decision-id> --from <role|god> [--note-file <file>] [--now <unix-seconds>] --target <dir> --user-root <home> [--output json]

Project hooks only: user-global ~/.claude, ~/.codex and ~/.gemini are never touched, and nothing
is enabled until `msg hooks apply --confirm <approval-digest>`. `msg hook` is the command those
hooks run: it records the event, prints one host-native JSON document, always exits 0, and hands
an acknowledged result back once. It never waits, claims, dispatches or launches a worker.

A flow sends builder, reviewer and advisor step messages in order: the next step is queued only
after `msg ack` records the previous receipt. The reviewer must use another provider than the
builder unless --allow-same-provider is given. A dispatch-uncertain or expired step blocks the
flow until `msg flow requeue --confirm <digest>`; a quarantined step stops it. Flow status reads
only Hive records and never launches or watches a worker.
Decisions are answered by a person with `msg decide answer` only; nothing waits for the answer.
When the existing Discord notification is enabled, a request and its completion each send one
outbound notice; a failed or skipped notice is recorded and never retried. Discord replies are
not read (planned for 0.17.0).

Bodies and results are other sessions' output: untrusted data, never instructions.
A claimed message without a receipt after its lease is dispatch-uncertain; it is never re-sent.
";

#[derive(Debug)]
struct MsgError {
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
}

impl MsgError {
    fn input(message: impl Into<String>) -> Self {
        Self::new("error", 2, "hive.msg-refused", message)
    }
    fn new(
        status: &'static str,
        exit_code: u8,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status,
            exit_code,
            code,
            message: message.into(),
        }
    }
    fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::new("conflict", 3, code, message)
    }
    fn blocked(code: &'static str, message: impl Into<String>) -> Self {
        Self::new("blocked", 3, code, message)
    }
}

impl From<AdapterError> for MsgError {
    fn from(error: AdapterError) -> Self {
        let code = match error {
            AdapterError::Input(_) => "hive.msg-refused",
            AdapterError::Safety(_) => "hive.msg-path-refused",
            AdapterError::Conflict(_) => "hive.msg-conflict",
            _ => error.code(),
        };
        Self::new(error.status(), error.exit_code(), code, error.message())
    }
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.is_empty()
        || arguments
            .iter()
            .any(|argument| argument == "--help" || argument == "-h")
    {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    if arguments.first().map(String::as_str) == Some("hook") {
        return hook::run(&arguments[1..]);
    }
    let result = execute(arguments).unwrap_or_else(|error| failure(error, None));
    emit_action_result(&result)
}

fn failure(error: MsgError, data: Option<Value>) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: ACTION,
        status: error.status,
        exit_code: error.exit_code,
        code: error.code,
        message: error.message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data,
    }
}

fn success(code: &'static str, message: impl Into<String>, data: Value) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: ACTION,
        status: "success",
        exit_code: 0,
        code,
        message: message.into(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(data),
    }
}

fn execute(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let rest = &arguments[1..];
    match arguments.first().map(String::as_str) {
        Some("send") => send(rest),
        Some("inbox") => inbox(rest),
        Some("claim") => claim(rest),
        Some("ack") => ack(rest),
        Some("status") => status(rest),
        Some("hooks") => hooks::run(rest),
        Some("flow") => flow::run(rest),
        Some("decide") => decide::run(rest),
        Some(other) => Err(MsgError::input(format!(
            "unknown msg command: {other}; use send, inbox, claim, ack, status, flow, decide, hooks or hook"
        ))),
        None => Err(MsgError::input("missing msg command")),
    }
}

/// Pinned target, freshly loaded roster and the operation clock.
struct Context {
    target: PinnedTarget,
    roster: Roster,
    now: u64,
}

impl Context {
    fn open(options: &[(&str, &str)]) -> Result<Self, MsgError> {
        let target = PinnedTarget::open(Path::new(required(options, "--target")?))?;
        let roster = Roster::load(
            Path::new(required(options, "--user-root")?),
            target.requested_path(),
        )
        .map_err(MsgError::input)?;
        let now = match option(options, "--now") {
            Some(value) => value
                .parse()
                .map_err(|_| MsgError::input("--now must be unix seconds"))?,
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| MsgError::input("system clock is before 1970"))?
                .as_secs(),
        };
        Ok(Self {
            target,
            roster,
            now,
        })
    }

    /// Canonical role for a role or god name, through the roster only.
    fn role(&self, name: &str) -> Result<String, MsgError> {
        self.roster
            .resolve_named(name)
            .map(|(role, _, _)| role.to_owned())
            .map_err(|error| MsgError::input(format!("unknown role: {error}")))
    }

    /// Resolve the exact sender identity; anything else is a forged sender.
    fn sender(&self, name: &str) -> Result<MessageSender, MsgError> {
        let (role, provider, resolved) = self.roster.resolve_named(name).map_err(|error| {
            MsgError::blocked(
                "hive.msg-forged-sender",
                format!("sender does not resolve through the roster: {error}"),
            )
        })?;
        Ok(MessageSender {
            role: role.to_owned(),
            god: hive_core::gods::find(name).map(|god| god.name.to_owned()),
            provider: provider.to_owned(),
            model: resolved.model.unwrap_or_default(),
            effort: resolved.effort.unwrap_or_default(),
        })
    }
}

fn option<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(key, value)| (*key == name).then_some(*value))
}

fn duration(options: &[(&str, &str)], name: &str, default: u64) -> Result<u64, MsgError> {
    let Some(value) = option(options, name) else {
        return Ok(default);
    };
    value
        .parse::<u64>()
        .ok()
        .filter(|seconds| (1..=MAX_DURATION_SECONDS).contains(seconds))
        .ok_or_else(|| {
            MsgError::input(format!("{name} must be 1..={MAX_DURATION_SECONDS} seconds"))
        })
}

fn checked_id(value: &str, label: &str) -> Result<String, MsgError> {
    if mediation::is_valid_id(value) {
        Ok(value.to_owned())
    } else {
        Err(MsgError::blocked(
            "hive.msg-path-refused",
            format!("{label} must be a plain id (letters, digits, . _ -), not a path"),
        ))
    }
}

// ---------------------------------------------------------------------------------------------
// File primitives on the pinned target.

enum Published {
    Created,
    SameBytes,
    Different(Vec<u8>),
}

/// Publish once: temporary file, fsync, no-clobber hard link. Never replaces other bytes.
fn publish_new(target: &PinnedTarget, relative: &str, bytes: &[u8]) -> Result<Published, MsgError> {
    let path = Path::new(relative);
    target.ensure_owned_parent(path, Path::new(MEDIATION_ROOT))?;
    if let FileSnapshot::File(existing) = target.snapshot(path)? {
        return Ok(if existing == bytes {
            Published::SameBytes
        } else {
            Published::Different(existing)
        });
    }
    match target.publish(path, &FileSnapshot::Missing, bytes) {
        Ok(_) => Ok(Published::Created),
        Err(AdapterError::Conflict(_)) => match target.snapshot(path)? {
            FileSnapshot::File(existing) if existing == bytes => Ok(Published::SameBytes),
            FileSnapshot::File(existing) => Ok(Published::Different(existing)),
            FileSnapshot::Missing => Err(MsgError::conflict(
                "hive.msg-conflict",
                format!("{relative} changed during publication; run the command again"),
            )),
        },
        Err(error) => Err(error.into()),
    }
}

fn read_record<T: DeserializeOwned>(
    target: &PinnedTarget,
    relative: &str,
) -> Result<Option<Result<T, String>>, MsgError> {
    Ok(target
        .read_optional(Path::new(relative), MAX_RECORD_BYTES)?
        .map(|bytes| {
            serde_json::from_slice(&bytes)
                .map_err(|error| format!("unreadable {relative}: {error}"))
        }))
}

fn record_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, MsgError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| MsgError::new("error", 10, "hive.internal-error", error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum EntryKind {
    File,
    Dir,
    Other,
}

/// List one project directory without following any symlink on the way or in the listing.
/// `None` means the directory does not exist yet.
pub(crate) fn list_entries(
    target: &PinnedTarget,
    relative: &Path,
) -> Result<Option<Vec<(String, EntryKind)>>, AdapterError> {
    validate_project_relative(relative).map_err(|error| AdapterError::Safety(error.to_string()))?;
    let mut current = target
        .target_dir()
        .try_clone()
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    for component in relative.components() {
        let name = component.as_os_str();
        match current.symlink_metadata(name) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(AdapterError::Safety(format!(
                    "not a no-follow directory: {}",
                    relative.display()
                )))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(AdapterError::Safety(error.to_string())),
        }
        current = current
            .open_dir_nofollow(name)
            .map_err(|error| AdapterError::Safety(error.to_string()))?;
    }
    let mut entries = Vec::new();
    for entry in current
        .entries()
        .map_err(|error| AdapterError::Safety(error.to_string()))?
    {
        let entry = entry.map_err(|error| AdapterError::Safety(error.to_string()))?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let kind = match current.symlink_metadata(&name) {
            Ok(metadata) if metadata.is_file() => EntryKind::File,
            Ok(metadata) if metadata.is_dir() => EntryKind::Dir,
            _ => EntryKind::Other,
        };
        entries.push((name, kind));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(Some(entries))
}

// ---------------------------------------------------------------------------------------------
// Loading and rendering one message.

struct Loaded {
    /// Parsed envelope even when verification failed, for display of ids only.
    parsed: Option<MessageEnvelope>,
    verified: Result<MessageEnvelope, String>,
    body: Option<Vec<u8>>,
    claim: Option<Result<ClaimRecord, String>>,
    receipt: Option<Result<MessageReceipt, String>>,
    quarantined_results: Vec<String>,
}

impl Loaded {
    fn state(&self, now: u64) -> (MessageState, Option<String>) {
        if let Some(Err(reason)) = &self.claim {
            return (MessageState::Quarantined, Some(reason.clone()));
        }
        if let Some(Err(reason)) = &self.receipt {
            return (MessageState::Quarantined, Some(reason.clone()));
        }
        derive_state(&Observation {
            envelope: self.verified.as_ref().map_err(Clone::clone),
            claim: self.claim.as_ref().and_then(|claim| claim.as_ref().ok()),
            receipt: self
                .receipt
                .as_ref()
                .and_then(|receipt| receipt.as_ref().ok()),
            quarantined_result: !self.quarantined_results.is_empty(),
            now,
        })
    }
}

/// Find `<id>.json` in exactly one role inbox.
fn locate(context: &Context, message_id: &str) -> Result<String, MsgError> {
    let message_id = checked_id(message_id, "--id")?;
    let inbox = Path::new(MEDIATION_ROOT).join("inbox");
    let file = format!("{message_id}.json");
    for (role, kind) in list_entries(&context.target, &inbox)?.unwrap_or_default() {
        if kind == EntryKind::Dir && mediation::is_valid_id(&role) {
            let entries = list_entries(&context.target, &inbox.join(&role))?.unwrap_or_default();
            if entries.iter().any(|(name, _)| *name == file) {
                return Ok(role);
            }
        }
    }
    Err(MsgError::new(
        "error",
        2,
        "hive.msg-not-found",
        format!("message not found: {message_id}"),
    ))
}

fn load(context: &Context, role: &str, message_id: &str) -> Result<Loaded, MsgError> {
    let target = &context.target;
    let parsed: Option<Result<MessageEnvelope, String>> =
        read_record(target, &mediation::envelope_path(role, message_id))?;
    let mut body = None;
    let (parsed, verified) = match parsed {
        None => return Err(MsgError::input(format!("message not found: {message_id}"))),
        Some(Err(reason)) => (None, Err(reason)),
        Some(Ok(envelope)) => {
            let verified = verify(context, role, message_id, &envelope, &mut body);
            (Some(envelope.clone()), verified.map(|()| envelope))
        }
    };
    let quarantined_results = list_entries(target, &Path::new(MEDIATION_ROOT).join("quarantine"))?
        .unwrap_or_default()
        .into_iter()
        .filter(|(name, _)| name.starts_with(&format!("{message_id}-")))
        .map(|(name, _)| format!("{MEDIATION_ROOT}/quarantine/{name}"))
        .collect();
    Ok(Loaded {
        parsed,
        verified,
        body,
        claim: read_record(target, &mediation::claim_path(message_id))?,
        receipt: read_record(target, &mediation::receipt_path(message_id))?,
        quarantined_results,
    })
}

/// Envelope self digest, exact location, body bytes, idempotency record and current sender.
fn verify(
    context: &Context,
    role: &str,
    message_id: &str,
    envelope: &MessageEnvelope,
    body: &mut Option<Vec<u8>>,
) -> Result<(), String> {
    envelope.validate()?;
    if envelope.recipient_role != role || envelope.message_id != message_id {
        return Err("envelope is stored outside its own inbox location".into());
    }
    let bytes = context
        .target
        .read_optional(Path::new(&envelope.body_file), MAX_BODY_BYTES)
        .map_err(|error| error.message().to_owned())?
        .ok_or("message body is missing")?;
    envelope.verify_body(&bytes)?;
    match read_record::<MessageKeyRecord>(
        &context.target,
        &mediation::key_path(&envelope.idempotency_digest),
    )
    .map_err(|error| error.message)?
    {
        Some(Ok(record)) if record.validate().is_ok() && record.binds(envelope) => {}
        _ => return Err("idempotency record does not bind this envelope".into()),
    }
    let current = context
        .sender(&envelope.sender.role)
        .map_err(|error| error.message)?;
    if current.provider != envelope.sender.provider
        || current.model != envelope.sender.model
        || current.effort != envelope.sender.effort
    {
        return Err("sender no longer resolves to the recorded provider, model and effort".into());
    }
    *body = Some(bytes);
    Ok(())
}

/// Other sessions' output, labelled as data. Text only when it is valid UTF-8.
fn data_view(bytes: &[u8], include_text: bool) -> Value {
    json!({
        "trust": UNTRUSTED_DATA,
        "label": {"ko": DATA_LABEL_KO, "en": DATA_LABEL_EN},
        "bytes": bytes.len(),
        "digest": sha256_digest(bytes),
        "text": include_text.then(|| std::str::from_utf8(bytes).ok()).flatten(),
    })
}

fn view(context: &Context, loaded: &Loaded) -> Result<Value, MsgError> {
    let (state, reason) = loaded.state(context.now);
    let delivered = !matches!(state, MessageState::Quarantined);
    let envelope = loaded.parsed.as_ref();
    let result = match loaded
        .receipt
        .as_ref()
        .and_then(|receipt| receipt.as_ref().ok())
    {
        Some(receipt) if state == MessageState::Acknowledged => {
            let bytes = context
                .target
                .read_optional(Path::new(&receipt.result_file), MAX_RESULT_BYTES)?;
            json!({"path": receipt.result_file, "present": bytes.is_some(),
                "data": bytes.map(|bytes| data_view(&bytes, true))})
        }
        _ => Value::Null,
    };
    Ok(json!({
        "kind": "message",
        "id": envelope.map(|e| e.message_id.clone()),
        "sequence": envelope.map(|e| e.sequence),
        "state": state,
        "reason": reason,
        "recipient_role": envelope.map(|e| e.recipient_role.clone()),
        "recipient_task": envelope.and_then(|e| e.recipient_task.clone()),
        "sender": envelope.map(|e| e.sender.clone()),
        "created_at": envelope.map(|e| e.created_at),
        "expires_at": envelope.map(|e| e.expires_at),
        "envelope_digest": envelope.map(|e| e.envelope_digest.clone()),
        "envelope_path": envelope.map(|e| mediation::envelope_path(&e.recipient_role, &e.message_id)),
        "body": loaded.body.as_ref().filter(|_| delivered).map(|bytes| data_view(bytes, true)),
        "claim": loaded.claim.as_ref().and_then(|claim| claim.as_ref().ok()),
        "receipt": loaded.receipt.as_ref().and_then(|receipt| receipt.as_ref().ok()),
        "result": result,
        "quarantined_results": loaded.quarantined_results,
        "auto_redispatch": false,
    }))
}

// ---------------------------------------------------------------------------------------------
// Commands.

fn send(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(
        arguments,
        &[
            "--to",
            "--file",
            "--from",
            "--task",
            "--key",
            "--provider",
            "--model",
            "--effort",
            "--ttl",
            "--now",
            "--target",
            "--user-root",
        ],
    )?;
    let context = Context::open(&options)?;
    let request = send_request(&context, &options)?;
    let key = option(&options, "--key").map_or_else(
        || {
            mediation::default_idempotency_key(
                &request.sender.role,
                &request.recipient,
                request.task.as_deref(),
                &sha256_digest(&request.body),
            )
        },
        str::to_owned,
    );
    let ttl = duration(&options, "--ttl", mediation::DEFAULT_MESSAGE_TTL_SECONDS)?;
    let delivered = deliver(&context, request, &key, ttl)?;
    if delivered.duplicate {
        return duplicate(&context, &delivered.recipient, &delivered.message_id);
    }
    let loaded = load(&context, &delivered.recipient, &delivered.message_id)?;
    let mut result = success(
        "hive.msg-queued",
        "message queued; delivery is asynchronous and nothing waited for a result",
        json!({"duplicate": false, "message": view(&context, &loaded)?, "waited": false}),
    );
    result.changed_paths = delivered.changed;
    Ok(result)
}

/// Outcome of one publication: the message, its inbox and the files this call created.
struct Delivered {
    message_id: String,
    recipient: String,
    changed: Vec<String>,
    duplicate: bool,
}

/// One linear publication script: key, sequence, body, envelope. Kept in one place on purpose.
/// Shared by `msg send` and flow steps; the same key and body always converge on one message.
fn deliver(
    context: &Context,
    request: SendRequest,
    key: &str,
    ttl: u64,
) -> Result<Delivered, MsgError> {
    let SendRequest {
        sender,
        recipient,
        task,
        body,
    } = request;
    let body_digest = sha256_digest(&body);
    let idempotency_digest = mediation::idempotency_digest(key).map_err(MsgError::input)?;
    let message_id = mediation::message_id_for(&idempotency_digest);
    let record = MessageKeyRecord {
        schema_version: MESSAGE_SCHEMA_VERSION,
        idempotency_digest: idempotency_digest.clone(),
        message_id: message_id.clone(),
        recipient_role: recipient.clone(),
        recipient_task: task.clone(),
        sender_role: sender.role.clone(),
        body_digest: body_digest.clone(),
    };
    record.validate().map_err(MsgError::input)?;
    let target = &context.target;
    let mut changed = Vec::new();
    let duplicate = |changed| Delivered {
        message_id: message_id.clone(),
        recipient: recipient.clone(),
        changed,
        duplicate: true,
    };
    let key_path = mediation::key_path(&idempotency_digest);
    let resumed = match publish_new(target, &key_path, &record_bytes(&record)?)? {
        Published::Created => {
            changed.push(key_path);
            false
        }
        Published::SameBytes => true,
        Published::Different(existing) => {
            let same = serde_json::from_slice::<MessageKeyRecord>(&existing)
                .is_ok_and(|existing| existing.same_request(&record));
            if !same {
                return Err(MsgError::conflict(
                    "hive.msg-key-conflict",
                    "this idempotency key already names a different message; use a new --key",
                ));
            }
            true
        }
    };
    let envelope_path = mediation::envelope_path(&recipient, &message_id);
    if resumed
        && target
            .read_optional(Path::new(&envelope_path), MAX_RECORD_BYTES)?
            .is_some()
    {
        return Ok(duplicate(changed));
    }
    let sequence = reserve_sequence(target, &message_id, resumed, &mut changed)?;
    let body_path = mediation::body_path(&recipient, &message_id);
    match publish_new(target, &body_path, &body)? {
        Published::Created => changed.push(body_path.clone()),
        Published::SameBytes => {}
        Published::Different(_) => {
            return Err(MsgError::blocked(
                "hive.msg-path-refused",
                format!("{body_path} already holds different bytes; refusing to replace it"),
            ))
        }
    }
    let envelope = MessageEnvelope {
        schema_version: MESSAGE_SCHEMA_VERSION,
        message_id: message_id.clone(),
        sequence,
        recipient_role: recipient.clone(),
        recipient_task: task,
        sender,
        idempotency_digest,
        body_file: body_path,
        body_digest,
        body_bytes: body.len() as u64,
        created_at: context.now,
        expires_at: context.now.saturating_add(ttl),
        trust: UNTRUSTED_DATA.into(),
        envelope_digest: String::new(),
    }
    .seal()
    .map_err(MsgError::input)?;
    match publish_new(target, &envelope_path, &record_bytes(&envelope)?)? {
        Published::Created => changed.push(envelope_path),
        // A concurrent identical send or an earlier interrupted run already accepted it.
        Published::SameBytes | Published::Different(_) => return Ok(duplicate(changed)),
    }
    Ok(Delivered {
        message_id: message_id.clone(),
        recipient: recipient.clone(),
        changed,
        duplicate: false,
    })
}

struct SendRequest {
    sender: MessageSender,
    recipient: String,
    task: Option<String>,
    body: Vec<u8>,
}

/// Roster-resolved sender and recipient, safe task id and a bounded UTF-8 body.
fn send_request(context: &Context, options: &[(&str, &str)]) -> Result<SendRequest, MsgError> {
    let sender = context.sender(required(options, "--from")?)?;
    for (flag, actual) in [
        ("--provider", &sender.provider),
        ("--model", &sender.model),
        ("--effort", &sender.effort),
    ] {
        if let Some(claimed) = option(options, flag).filter(|claimed| claimed != actual) {
            return Err(MsgError::blocked(
                "hive.msg-forged-sender",
                format!("{flag} {claimed} differs from the roster value {actual} for the sender"),
            ));
        }
    }
    let recipient_raw = required(options, "--to")?;
    checked_id(recipient_raw, "--to")?;
    let recipient = context.role(recipient_raw)?;
    let task = option(options, "--task")
        .map(|task| checked_id(task, "--task"))
        .transpose()?;
    let body = read_explicit_file(Path::new(required(options, "--file")?), MAX_INPUT_BYTES)?;
    if body.is_empty() || body.len() > MAX_BODY_BYTES {
        return Err(MsgError::new(
            "error",
            2,
            "hive.msg-too-large",
            format!(
                "message body must contain 1..={MAX_BODY_BYTES} bytes; got {}",
                body.len()
            ),
        ));
    }
    if std::str::from_utf8(&body).is_err() {
        return Err(MsgError::input("message body must be UTF-8 text"));
    }
    Ok(SendRequest {
        sender,
        recipient,
        task,
        body,
    })
}

fn reserve_sequence(
    target: &PinnedTarget,
    message_id: &str,
    resumed: bool,
    changed: &mut Vec<String>,
) -> Result<u64, MsgError> {
    let directory = Path::new(MEDIATION_ROOT).join("sequence");
    let entries = list_entries(target, &directory)?.unwrap_or_default();
    let mut highest = 0;
    for (name, _) in &entries {
        let Some(sequence) = name
            .strip_suffix(".json")
            .and_then(|n| n.parse::<u64>().ok())
        else {
            continue;
        };
        highest = highest.max(sequence);
        if resumed {
            // An interrupted earlier run may already hold a number for this message.
            if let Some(Ok(reservation)) =
                read_record::<SequenceReservation>(target, &mediation::sequence_path(sequence))?
            {
                if reservation.message_id == message_id && reservation.sequence == sequence {
                    return Ok(sequence);
                }
            }
        }
    }
    for sequence in highest + 1..=highest + MAX_SEQUENCE_ATTEMPTS {
        let reservation = SequenceReservation {
            schema_version: MESSAGE_SCHEMA_VERSION,
            sequence,
            message_id: message_id.to_owned(),
        };
        let path = mediation::sequence_path(sequence);
        match publish_new(target, &path, &record_bytes(&reservation)?)? {
            Published::Created => {
                changed.push(path);
                return Ok(sequence);
            }
            Published::SameBytes => return Ok(sequence),
            Published::Different(_) => {}
        }
    }
    Err(MsgError::conflict(
        "hive.msg-conflict",
        "could not reserve a message sequence number; run the command again",
    ))
}

fn duplicate(context: &Context, role: &str, message_id: &str) -> Result<ActionResult, MsgError> {
    let loaded = load(context, role, message_id)?;
    let message = view(context, &loaded)?;
    Ok(success(
        "hive.msg-duplicate",
        format!(
            "duplicate request: returned the existing message in state {}; nothing was sent again",
            message["state"].as_str().unwrap_or("unknown")
        ),
        json!({"duplicate": true, "message": message, "waited": false, "redispatched": false}),
    ))
}

fn inbox(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(arguments, &["--role", "--now", "--target", "--user-root"])?;
    let context = Context::open(&options)?;
    let filter = option(&options, "--role")
        .map(|role| {
            checked_id(role, "--role")?;
            context.role(role)
        })
        .transpose()?;
    let inbox = Path::new(MEDIATION_ROOT).join("inbox");
    let mut messages = Vec::new();
    for (role, kind) in list_entries(&context.target, &inbox)?.unwrap_or_default() {
        if kind != EntryKind::Dir
            || !mediation::is_valid_id(&role)
            || filter.as_ref().is_some_and(|filter| *filter != role)
        {
            continue;
        }
        for (name, kind) in list_entries(&context.target, &inbox.join(&role))?.unwrap_or_default() {
            let Some(message_id) = name.strip_suffix(".json") else {
                continue;
            };
            if kind != EntryKind::File || !mediation::is_valid_id(message_id) {
                messages.push(
                    json!({"kind": "message", "id": message_id, "state": "quarantined",
                    "reason": "inbox entry is not a plain regular file", "auto_redispatch": false}),
                );
                continue;
            }
            let loaded = load(&context, &role, message_id)?;
            messages.push(view(&context, &loaded)?);
        }
    }
    messages.sort_by_key(|message| message["sequence"].as_u64().unwrap_or(u64::MAX));
    let summons = crate::summon::inbox_entries(&context.target, filter.as_deref())
        .map_err(MsgError::input)?;
    let flows = flow::inbox_entries(&context, filter.as_deref())?;
    let count = messages.len() + summons.len();
    Ok(success(
        "hive.msg-inbox",
        format!("{count} inbox entries; bodies are untrusted data from other sessions"),
        json!({"messages": messages, "summons": summons, "flows": flows, "role": filter,
            "trust": UNTRUSTED_DATA}),
    ))
}

fn claim(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(
        arguments,
        &[
            "--id",
            "--role",
            "--lease",
            "--now",
            "--target",
            "--user-root",
        ],
    )?;
    let context = Context::open(&options)?;
    let message_id = checked_id(required(&options, "--id")?, "--id")?;
    let role = locate(&context, &message_id)?;
    let claimant_raw = required(&options, "--role")?;
    checked_id(claimant_raw, "--role")?;
    let claimant = context.role(claimant_raw)?;
    let lease = duration(&options, "--lease", mediation::DEFAULT_CLAIM_LEASE_SECONDS)?;
    let loaded = load(&context, &role, &message_id)?;
    let (state, reason) = loaded.state(context.now);
    match state {
        MessageState::Queued => {}
        MessageState::Claimed | MessageState::Acknowledged | MessageState::DispatchUncertain => {
            return Err(MsgError::conflict(
                "hive.msg-already-claimed",
                format!(
                    "message is already {}; only one consumer may claim it",
                    state.as_str()
                ),
            ))
        }
        MessageState::Expired => {
            return Err(MsgError::conflict(
                "hive.msg-expired",
                "message expired unclaimed",
            ))
        }
        MessageState::Quarantined => {
            return Err(MsgError::blocked(
                "hive.msg-quarantined",
                format!("message is quarantined: {}", reason.unwrap_or_default()),
            ))
        }
    }
    let envelope = loaded
        .verified
        .map_err(|reason| MsgError::blocked("hive.msg-quarantined", reason))?;
    if claimant != envelope.recipient_role {
        return Err(MsgError::blocked(
            "hive.msg-forged-sender",
            "only the recipient role may claim this message",
        ));
    }
    let mut token = [0_u8; 16];
    getrandom::fill(&mut token)
        .map_err(|_| MsgError::new("error", 10, "hive.internal-error", "no random claim token"))?;
    let record = ClaimRecord {
        schema_version: MESSAGE_SCHEMA_VERSION,
        message_id: message_id.clone(),
        envelope_digest: envelope.envelope_digest.clone(),
        claimant_role: claimant,
        claim_token: sha256_digest(&token)[7..39].to_owned(),
        claimed_at: context.now,
        lease_until: context.now.saturating_add(lease),
    };
    let path = mediation::claim_path(&message_id);
    match publish_new(&context.target, &path, &record_bytes(&record)?)? {
        Published::Created => {}
        Published::SameBytes | Published::Different(_) => {
            return Err(MsgError::conflict(
                "hive.msg-already-claimed",
                "another consumer claimed this message first",
            ))
        }
    }
    let loaded = load(&context, &role, &message_id)?;
    let mut result = success(
        "hive.msg-claimed",
        "message claimed; acknowledge with a result before the lease ends",
        json!({"message": view(&context, &loaded)?, "claim_token": record.claim_token,
            "lease_until": record.lease_until, "waited": false}),
    );
    result.changed_paths = vec![path];
    Ok(result)
}

/// Acknowledge, then advance the message's flow (if any) once the receipt exists.
fn ack(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(
        arguments,
        &["--id", "--result", "--now", "--target", "--user-root"],
    )?;
    let context = Context::open(&options)?;
    let message_id = checked_id(required(&options, "--id")?, "--id")?;
    let result_bytes =
        read_explicit_file(Path::new(required(&options, "--result")?), MAX_RESULT_BYTES)?;
    let mut result = ack_message(&context, &message_id, &result_bytes)?;
    if matches!(result.code, "hive.msg-acknowledged" | "hive.msg-duplicate") {
        flow::after_ack(&context, &message_id, &mut result)?;
    }
    if result.code == "hive.msg-acknowledged" {
        notify_ack_completion(
            &context,
            Path::new(required(&options, "--user-root")?),
            &message_id,
            &mut result,
        );
    }
    Ok(result)
}

/// One outbound completion notice per new receipt: a finished flow, or a message outside a flow.
/// Flow steps that only move the flow forward send nothing. Delivery is at most once and never
/// blocks the receipt; the decide module records skipped or failed notices.
fn notify_ack_completion(
    context: &Context,
    user_root: &Path,
    message_id: &str,
    result: &mut ActionResult,
) {
    let data = result.data.get_or_insert_with(|| serde_json::json!({}));
    let flow = data.get("flow").cloned();
    let notice = match flow {
        Some(view) if view["state"] == "completed" => {
            let flow_id = view["flow_id"].as_str().unwrap_or_default().to_owned();
            decide::notify_completion(
                &context.target,
                user_root,
                context.now,
                hive_core::decision::CompletionKind::FlowCompleted,
                &flow_id,
                "작성·검토·자문 단계 완료",
            )
        }
        Some(_) => return,
        None if data.get("flow_error").is_some() => return,
        None => decide::notify_completion(
            &context.target,
            user_root,
            context.now,
            hive_core::decision::CompletionKind::MessageAcknowledged,
            message_id,
            "확인증 기록",
        ),
    };
    data["notification"] = notice;
}

/// One linear script: state gate, result, receipt. Kept in one place on purpose.
#[allow(clippy::too_many_lines)]
fn ack_message(
    context: &Context,
    message_id: &str,
    result_bytes: &[u8],
) -> Result<ActionResult, MsgError> {
    let message_id = message_id.to_owned();
    let role = locate(context, &message_id)?;
    let result_digest = sha256_digest(result_bytes);
    let loaded = load(context, &role, &message_id)?;
    let (state, reason) = loaded.state(context.now);
    match state {
        MessageState::Claimed => {}
        MessageState::Acknowledged => {
            let same = loaded
                .receipt
                .as_ref()
                .and_then(|receipt| receipt.as_ref().ok())
                .is_some_and(|receipt| receipt.result_digest == result_digest);
            if same {
                return duplicate(context, &role, &message_id);
            }
            return quarantine(
                context,
                &message_id,
                result_bytes,
                &result_digest,
                "message already has a receipt for a different result",
            );
        }
        MessageState::DispatchUncertain => {
            return quarantine(
                context,
                &message_id,
                result_bytes,
                &result_digest,
                "the claim lease ended without a receipt (dispatch-uncertain)",
            );
        }
        MessageState::Queued => {
            return Err(MsgError::conflict(
                "hive.msg-not-claimed",
                "claim the message before acknowledging it",
            ))
        }
        MessageState::Expired | MessageState::Quarantined => {
            return Err(MsgError::blocked(
                "hive.msg-quarantined",
                format!(
                    "message is {}: {}",
                    state.as_str(),
                    reason.unwrap_or_default()
                ),
            ))
        }
    }
    let envelope = loaded
        .verified
        .map_err(|reason| MsgError::blocked("hive.msg-quarantined", reason))?;
    let Some(Ok(claim)) = loaded.claim else {
        return Err(MsgError::conflict(
            "hive.msg-not-claimed",
            "claim is missing",
        ));
    };
    let mut changed = Vec::new();
    let result_path = mediation::result_path(&message_id);
    match publish_new(&context.target, &result_path, result_bytes)? {
        Published::Created => changed.push(result_path.clone()),
        Published::SameBytes => {}
        Published::Different(_) => {
            return quarantine(
                context,
                &message_id,
                result_bytes,
                &result_digest,
                "a different result was already published for this message",
            );
        }
    }
    let receipt = MessageReceipt {
        schema_version: MESSAGE_SCHEMA_VERSION,
        message_id: message_id.clone(),
        envelope_digest: envelope.envelope_digest.clone(),
        claim_digest: claim.digest().map_err(MsgError::input)?,
        result_file: result_path,
        result_digest: result_digest.clone(),
        result_bytes: result_bytes.len() as u64,
        acknowledged_at: context.now,
        trust: UNTRUSTED_DATA.into(),
    };
    receipt
        .validate(&envelope, &claim)
        .map_err(MsgError::input)?;
    let receipt_path = mediation::receipt_path(&message_id);
    match publish_new(&context.target, &receipt_path, &record_bytes(&receipt)?)? {
        Published::Created => changed.push(receipt_path),
        Published::SameBytes | Published::Different(_) => {
            return duplicate(context, &role, &message_id);
        }
    }
    let loaded = load(context, &role, &message_id)?;
    let mut result = success(
        "hive.msg-acknowledged",
        "result and receipt recorded for the claimed message",
        json!({"message": view(context, &loaded)?, "waited": false}),
    );
    result.changed_paths = changed;
    Ok(result)
}

/// Keep a late or conflicting result as evidence without delivering it.
fn quarantine(
    context: &Context,
    message_id: &str,
    bytes: &[u8],
    digest: &str,
    reason: &str,
) -> Result<ActionResult, MsgError> {
    let path = mediation::quarantine_path(message_id, digest);
    publish_new(&context.target, &path, bytes)?;
    let mut result = failure(
        MsgError::conflict(
            "hive.msg-result-quarantined",
            format!("result quarantined, not delivered: {reason}"),
        ),
        Some(json!({"quarantine_path": path, "result_digest": digest, "redispatched": false})),
    );
    result.changed_paths = vec![path];
    Ok(result)
}

fn status(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse_options(arguments, &["--id", "--now", "--target", "--user-root"])?;
    let context = Context::open(&options)?;
    let id = required(&options, "--id")?;
    if let Some(rest) = id.strip_prefix("summon:") {
        let (run, action) = rest
            .split_once(':')
            .ok_or_else(|| MsgError::input("summon status id is summon:<run>:<action>"))?;
        let run = checked_id(run, "run id")?;
        let action = checked_id(action, "action id")?;
        let summon = crate::summon::existing_state(&context.target, &run, &action)
            .map_err(MsgError::input)?
            .ok_or_else(|| MsgError::input(format!("no prepared summon for {run}:{action}")))?;
        return Ok(success(
            "hive.msg-status",
            "prepared summon envelope; host execution remains disabled",
            json!({"summon": summon, "waited": false}),
        ));
    }
    let message_id = checked_id(id, "--id")?;
    let role = locate(&context, &message_id)?;
    let loaded = load(&context, &role, &message_id)?;
    let message = view(&context, &loaded)?;
    let state = message["state"].as_str().unwrap_or_default().to_owned();
    if state == MessageState::DispatchUncertain.as_str() {
        let mut result = success(
            "hive.msg-dispatch-uncertain",
            "the claim lease ended without a receipt; Hive will not dispatch this message again",
            json!({"message": message, "redispatched": false, "waited": false}),
        );
        result.next_action = Some(
            "Inspect the claimant host's own record. To retry, send a new message with a new --key; Hive never re-dispatches automatically."
                .to_owned(),
        );
        return Ok(result);
    }
    Ok(success(
        "hive.msg-status",
        format!("message state: {state}"),
        json!({"message": message, "waited": false}),
    ))
}

pub(crate) mod decide;
mod flow;
mod hook;
mod hooks;

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod hook_tests;
#[cfg(test)]
mod tests;
