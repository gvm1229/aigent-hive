//! `hive msg decide`: local decision queue with opt-in outbound Discord notices (OV160-006).
//!
//! Layout under `.hive/mediation/` (see `hive_core::decision`): `decisions/requests/<id>.json`
//! is the acceptance point, `decisions/resolutions/<id>.json` the single answer or withdrawal,
//! and `notified/<event>.json` plus `notified/<event>.outcome.json` make each notice one attempt.
//! Every file is published once with the inbox's no-clobber helper; the state is recomputed from
//! those files on every read, so a restart loses nothing.
//!
//! Commands return at once and never wait for an answer. A person answers with the local CLI;
//! Discord is outbound only (replies are 0.17.0). A failed or skipped notice never blocks the
//! command and is never retried automatically.
use super::{
    checked_id, data_view, duration, option, publish_new, read_record, record_bytes, success,
    Context, EntryKind, MsgError, Published,
};
use crate::run::{read_explicit_file, required, PinnedTarget};
use crate::ActionResult;
use hive_core::decision::{
    self, answer_command, completion_notice_ko, derive_decision_state, notice_claim_path,
    notice_event_id, notice_outcome_path, request_notice_ko, CompletionDetail, CompletionKind,
    DecisionRequest, DecisionResolution, DecisionState, NoticeClaim, NoticeKind, NoticeOutcome,
    ResolutionKind, DECISION_SCHEMA_VERSION, LOCAL_CLI, MAX_DECISION_TTL_SECONDS, MAX_NOTE_BYTES,
    MAX_QUESTION_BYTES,
};
use hive_core::mediation::{self, UNTRUSTED_DATA};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::Path;

const INPUT_LIMIT: usize = 64 * 1024;

/// One delivery attempt for a prepared notice. Production uses the existing Discord path;
/// tests use a fake. Returns `(outcome, reason)` with outcome `sent`, `notify-failed` or
/// `skipped` and an opaque reason that never contains a URL or an environment value.
pub(crate) trait Transport {
    fn deliver(
        &mut self,
        target: &PinnedTarget,
        user_root: &Path,
        content: &str,
    ) -> (&'static str, &'static str);
}

/// The existing opt-in usage-guard Discord webhook, resolved from the project's effective
/// configuration. Unconfigured or disabled means `skipped`.
pub(crate) struct DiscordTransport;

impl Transport for DiscordTransport {
    fn deliver(
        &mut self,
        target: &PinnedTarget,
        user_root: &Path,
        content: &str,
    ) -> (&'static str, &'static str) {
        use crate::discord::NotificationOutcome as O;
        let Ok(config) = crate::usage_control::read_effective_config(target, Some(user_root), None)
        else {
            return ("skipped", "discord-config-unavailable");
        };
        match crate::discord::notify_content(
            config.discord_guard_enabled,
            config.discord_webhook_url_env.as_deref(),
            content,
        ) {
            O::Sent => ("sent", "sent"),
            O::Disabled => ("skipped", "disabled"),
            O::MissingWebhookEnvironment => ("skipped", "missing-webhook-environment"),
            O::InvalidWebhookUrl => ("notify-failed", "invalid-webhook-url"),
            O::DeliveryFailed => ("notify-failed", "delivery-failed"),
        }
    }
}

pub(super) fn run(arguments: &[String]) -> Result<ActionResult, MsgError> {
    run_with(arguments, &mut DiscordTransport)
}

fn run_with(arguments: &[String], transport: &mut dyn Transport) -> Result<ActionResult, MsgError> {
    let rest = arguments.get(1..).unwrap_or_default();
    match arguments.first().map(String::as_str) {
        Some("request") => request(rest, transport),
        Some("list") => list(rest),
        Some("show") => show(rest),
        Some("answer") => answer(rest, transport),
        Some("withdraw") => withdraw(rest, transport),
        Some(other) => Err(MsgError::input(format!(
            "unknown msg decide command: {other}; use request, list, show, answer or withdraw"
        ))),
        None => Err(MsgError::input(
            "missing msg decide command: request, list, show, answer or withdraw",
        )),
    }
}

/// Announce one completed event (message acknowledgement, finished flow, ...) at most once
/// through the existing Discord path. Never fails or blocks the caller; the returned report
/// says `sent`, `notify-failed`, `skipped`, or the earlier outcome when already recorded.
#[allow(dead_code)] // Wired by the flow and ack owners; covered by tests here.
pub(crate) fn notify_completion(
    target: &PinnedTarget,
    user_root: &Path,
    now: u64,
    kind: CompletionKind,
    subject_id: &str,
    summary: &str,
    detail: &CompletionDetail,
) -> Value {
    notify_completion_with(
        target,
        user_root,
        now,
        &mut DiscordTransport,
        kind,
        subject_id,
        summary,
        detail,
    )
}

#[allow(clippy::too_many_arguments)] // One notice: where, when, how, what, and its detail.
fn notify_completion_with(
    target: &PinnedTarget,
    user_root: &Path,
    now: u64,
    transport: &mut dyn Transport,
    kind: CompletionKind,
    subject_id: &str,
    summary: &str,
    detail: &CompletionDetail,
) -> Value {
    let content = completion_notice_ko(kind, subject_id, summary, &project_name(target), detail);
    notify_once(
        target,
        user_root,
        now,
        transport,
        NoticeKind::Completion(kind),
        subject_id,
        &content,
    )
}

// ---------------------------------------------------------------------------------------------
// Options: like the inbox parser, but `--option` may repeat for `request`.

fn parse<'a>(
    arguments: &'a [String],
    allowed: &[&str],
    repeatable: &[&str],
) -> Result<Vec<(&'a str, &'a str)>, MsgError> {
    let mut options = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let name = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| MsgError::input(format!("missing value for {name}")))?;
        if name == "--output" {
            if value != "json" {
                return Err(MsgError::input("--output accepts only json"));
            }
        } else if !allowed.contains(&name) {
            return Err(MsgError::input(format!("unknown option: {name}")));
        } else if !repeatable.contains(&name) && options.iter().any(|(seen, _)| *seen == name) {
            return Err(MsgError::input(format!("duplicate option: {name}")));
        }
        options.push((name, value.as_str()));
        index += 2;
    }
    Ok(options)
}

fn invalid(message: impl Into<String>) -> MsgError {
    MsgError::new("error", 2, "hive.decision-invalid", message)
}

fn decision_id(options: &[(&str, &str)]) -> Result<String, MsgError> {
    let id = checked_id(required(options, "--id")?, "--id")?;
    if id.starts_with("dec-") {
        Ok(id)
    } else {
        Err(invalid("--id must be a decision id (dec-...)"))
    }
}

fn utf8_file(path: &str, max: usize, label: &str) -> Result<String, MsgError> {
    let bytes = read_explicit_file(Path::new(path), INPUT_LIMIT)?;
    if bytes.len() > max {
        return Err(invalid(format!("{label} must be at most {max} bytes")));
    }
    String::from_utf8(bytes).map_err(|_| invalid(format!("{label} must be UTF-8 text")))
}

fn project_name(target: &PinnedTarget) -> String {
    target
        .requested_path()
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project")
        .to_owned()
}

// ---------------------------------------------------------------------------------------------
// Loading and rendering.

struct Loaded {
    parsed: Option<DecisionRequest>,
    verified: Result<DecisionRequest, String>,
    resolution: Option<Result<DecisionResolution, String>>,
}

impl Loaded {
    fn state(&self, now: u64) -> (DecisionState, Option<String>) {
        derive_decision_state(
            self.verified.as_ref().map_err(Clone::clone),
            self.resolution
                .as_ref()
                .map(|resolution| resolution.as_ref().map_err(Clone::clone)),
            now,
        )
    }
}

fn load(target: &PinnedTarget, id: &str) -> Result<Option<Loaded>, MsgError> {
    let Some(parsed) = read_record::<DecisionRequest>(target, &decision::request_path(id))? else {
        return Ok(None);
    };
    let (parsed, verified) = match parsed {
        Err(reason) => (None, Err(reason)),
        Ok(request) => {
            let verified = request.validate().and_then(|()| {
                if request.decision_id == id {
                    Ok(request.clone())
                } else {
                    Err("decision request is stored outside its own location".to_owned())
                }
            });
            (Some(request), verified)
        }
    };
    Ok(Some(Loaded {
        parsed,
        verified,
        resolution: read_record(target, &decision::resolution_path(id))?,
    }))
}

fn load_existing(target: &PinnedTarget, id: &str) -> Result<Loaded, MsgError> {
    load(target, id)?.ok_or_else(|| {
        MsgError::new(
            "error",
            2,
            "hive.decision-not-found",
            format!("decision not found: {id}"),
        )
    })
}

fn notice_status(target: &PinnedTarget, kind: NoticeKind, subject: &str) -> Value {
    let event_id = notice_event_id(kind, subject);
    let claimed = target
        .read_optional(Path::new(&notice_claim_path(&event_id)), 4096)
        .ok()
        .flatten()
        .is_some();
    let outcome = read_record::<NoticeOutcome>(target, &notice_outcome_path(&event_id))
        .ok()
        .flatten()
        .and_then(Result::ok)
        .map(|record| record.outcome);
    let state = match (claimed, outcome) {
        (false, _) => "none".to_owned(),
        (true, None) => "notify-uncertain".to_owned(),
        (true, Some(outcome)) => outcome,
    };
    json!({"event_id": event_id, "kind": kind.as_str(), "outcome": state})
}

fn view(target: &PinnedTarget, id: &str, loaded: &Loaded, now: u64) -> Value {
    let (state, reason) = loaded.state(now);
    let request = loaded.parsed.as_ref();
    let resolution = loaded
        .resolution
        .as_ref()
        .and_then(|resolution| resolution.as_ref().ok())
        .filter(|_| matches!(state, DecisionState::Answered | DecisionState::Withdrawn));
    let completion = match state {
        DecisionState::Answered => Some(CompletionKind::DecisionAnswered),
        DecisionState::Withdrawn => Some(CompletionKind::DecisionWithdrawn),
        _ => None,
    };
    let delivered = state != DecisionState::Quarantined;
    json!({
        "kind": "decision",
        "id": id,
        "state": state,
        "reason": reason,
        "requester_role": request.map(|r| r.requester_role.clone()),
        "requester_god": request.and_then(|r| r.requester_god.clone()),
        "question": request.filter(|_| delivered).map(|r| data_view(r.question.as_bytes(), true)),
        "options": request.map(|r| r.options.clone()),
        "recommended": request.and_then(|r| r.recommended.clone()),
        "task": request.and_then(|r| r.task.clone()),
        "message": request.and_then(|r| r.message.clone()),
        "flow": request.and_then(|r| r.flow.clone()),
        "created_at": request.map(|r| r.created_at),
        "expires_at": request.map(|r| r.expires_at),
        "request_digest": request.map(|r| r.request_digest.clone()),
        "request_path": decision::request_path(id),
        "resolution": resolution,
        "answer_command": loaded.verified.as_ref().ok()
            .filter(|_| state == DecisionState::Open)
            .map(answer_command),
        "notifications": {
            "request": notice_status(target, NoticeKind::DecisionRequested, id),
            "completion": completion.map(|kind| notice_status(target, NoticeKind::Completion(kind), id)),
        },
        "answer_channel": LOCAL_CLI,
        "discord_inbound": false,
    })
}

// ---------------------------------------------------------------------------------------------
// One notice attempt per event.

fn notify_once(
    target: &PinnedTarget,
    user_root: &Path,
    now: u64,
    transport: &mut dyn Transport,
    kind: NoticeKind,
    subject: &str,
    content: &str,
) -> Value {
    let event_id = notice_event_id(kind, subject);
    try_notify(
        target, user_root, now, transport, kind, subject, content, &event_id,
    )
    .unwrap_or_else(|error| {
        json!({"event_id": event_id, "kind": kind.as_str(), "outcome": "notify-failed",
            "reason": "notice-record-unavailable", "detail": error.message,
            "attempted_now": false, "auto_retry": false})
    })
}

#[allow(clippy::too_many_arguments)]
fn try_notify(
    target: &PinnedTarget,
    user_root: &Path,
    now: u64,
    transport: &mut dyn Transport,
    kind: NoticeKind,
    subject: &str,
    content: &str,
    event_id: &str,
) -> Result<Value, MsgError> {
    let report = |outcome: &str, reason: &str, attempted: bool| {
        json!({"event_id": event_id, "kind": kind.as_str(), "outcome": outcome,
            "reason": reason, "attempted_now": attempted, "auto_retry": false,
            "claim_path": notice_claim_path(event_id),
            "outcome_path": notice_outcome_path(event_id)})
    };
    let claim = NoticeClaim {
        schema_version: DECISION_SCHEMA_VERSION,
        event_id: event_id.to_owned(),
        kind: kind.as_str().to_owned(),
        subject_id: subject.to_owned(),
        content_digest: sha256_digest(content.as_bytes()),
        content_chars: content.chars().count() as u64,
        claimed_at: now,
    };
    match publish_new(target, &notice_claim_path(event_id), &record_bytes(&claim)?)? {
        Published::Created => {}
        // Another run already owns this notice: report its outcome, never send again.
        Published::SameBytes | Published::Different(_) => {
            let previous = read_record::<NoticeOutcome>(target, &notice_outcome_path(event_id))?
                .and_then(Result::ok);
            return Ok(match previous {
                Some(record) => report(&record.outcome, &record.reason, false),
                None => report("notify-uncertain", "claimed-without-outcome", false),
            });
        }
    }
    let (outcome, reason) = transport.deliver(target, user_root, content);
    let record = NoticeOutcome {
        schema_version: DECISION_SCHEMA_VERSION,
        event_id: event_id.to_owned(),
        outcome: outcome.to_owned(),
        reason: reason.to_owned(),
        recorded_at: now,
    };
    publish_new(
        target,
        &notice_outcome_path(event_id),
        &record_bytes(&record)?,
    )?;
    Ok(report(outcome, reason, true))
}

fn notice_message(report: &Value) -> String {
    let outcome = report["outcome"].as_str().unwrap_or("unknown");
    let reason = report["reason"].as_str().unwrap_or("-");
    match outcome {
        "sent" if report["attempted_now"] == true => "Discord notice sent".to_owned(),
        "skipped" => format!("Discord notice skipped ({reason})"),
        "notify-failed" => {
            format!("Discord notice failed ({reason}); recorded, not retried")
        }
        _ => format!("Discord notice already recorded: {outcome}"),
    }
}

// ---------------------------------------------------------------------------------------------
// Commands.

/// One linear script: validate, publish the request once, then one notice attempt.
#[allow(clippy::too_many_lines)]
fn request(arguments: &[String], transport: &mut dyn Transport) -> Result<ActionResult, MsgError> {
    let options = parse(
        arguments,
        &[
            "--from",
            "--question-file",
            "--option",
            "--recommend",
            "--task",
            "--message",
            "--flow",
            "--key",
            "--ttl",
            "--now",
            "--target",
            "--user-root",
        ],
        &["--option"],
    )?;
    let context = Context::open(&options)?;
    let user_root = Path::new(required(&options, "--user-root")?);
    let from = required(&options, "--from")?;
    checked_id(from, "--from")?;
    let requester_role = context.role(from)?;
    let question = utf8_file(
        required(&options, "--question-file")?,
        MAX_QUESTION_BYTES,
        "question",
    )?;
    decision::validate_question(&question).map_err(invalid)?;
    let choices: Vec<String> = options
        .iter()
        .filter(|(name, _)| *name == "--option")
        .map(|(_, value)| (*value).to_owned())
        .collect();
    let recommended = option(&options, "--recommend").map(str::to_owned);
    decision::validate_options(&choices, recommended.as_deref()).map_err(invalid)?;
    let related = |name: &str| {
        option(&options, name)
            .map(|value| checked_id(value, name))
            .transpose()
    };
    let (task, message, flow) = (
        related("--task")?,
        related("--message")?,
        related("--flow")?,
    );
    let question_digest = sha256_digest(question.as_bytes());
    let key = option(&options, "--key").map_or_else(
        || {
            decision::default_decision_key(
                &requester_role,
                task.as_deref(),
                &question_digest,
                &choices,
            )
        },
        str::to_owned,
    );
    let idempotency_digest = mediation::idempotency_digest(&key).map_err(invalid)?;
    let id = decision::decision_id_for(&idempotency_digest);
    let ttl = duration(&options, "--ttl", decision::DEFAULT_DECISION_TTL_SECONDS)?
        .min(MAX_DECISION_TTL_SECONDS);
    let request = DecisionRequest {
        schema_version: DECISION_SCHEMA_VERSION,
        decision_id: id.clone(),
        idempotency_digest,
        requester_role,
        requester_god: hive_core::gods::find(from).map(|god| god.name.to_owned()),
        question,
        question_digest,
        options: choices,
        recommended,
        task,
        message,
        flow,
        created_at: context.now,
        expires_at: context.now.saturating_add(ttl),
        trust: UNTRUSTED_DATA.into(),
        request_digest: String::new(),
    }
    .seal()
    .map_err(invalid)?;
    let target = &context.target;
    let path = decision::request_path(&id);
    let duplicate = match publish_new(target, &path, &record_bytes(&request)?)? {
        Published::Created => false,
        Published::SameBytes => true,
        Published::Different(existing) => {
            let same = serde_json::from_slice::<DecisionRequest>(&existing)
                .is_ok_and(|existing| existing.same_request(&request));
            if !same {
                return Err(MsgError::conflict(
                    "hive.decision-key-conflict",
                    "this idempotency key already names a different decision; use a new --key",
                ));
            }
            true
        }
    };
    let loaded = load_existing(target, &id)?;
    let (state, _) = loaded.state(context.now);
    // A duplicate resumes an interrupted notice once; a recorded one is never sent again.
    let notification = match (&loaded.verified, state) {
        (Ok(stored), DecisionState::Open) => notify_once(
            target,
            user_root,
            context.now,
            transport,
            NoticeKind::DecisionRequested,
            &id,
            &request_notice_ko(stored, &project_name(target)),
        ),
        _ => json!({"event_id": notice_event_id(NoticeKind::DecisionRequested, &id),
            "kind": NoticeKind::DecisionRequested.as_str(), "outcome": "skipped",
            "reason": format!("decision-{}", state.as_str()), "attempted_now": false,
            "auto_retry": false}),
    };
    let decision = view(target, &id, &loaded, context.now);
    let (code, summary) = if duplicate {
        (
            "hive.decision-duplicate",
            format!(
                "duplicate request: returned decision {id} in state {}; nothing was queued again",
                state.as_str()
            ),
        )
    } else {
        (
            "hive.decision-requested",
            format!("decision {id} is open; a person answers it locally and nothing waited"),
        )
    };
    let mut result = success(
        code,
        format!("{summary}. {}", notice_message(&notification)),
        json!({"duplicate": duplicate, "decision": decision, "notification": notification,
            "waited": false}),
    );
    if !duplicate {
        result.changed_paths = vec![path];
    }
    result.next_action = decision["answer_command"].as_str().map(str::to_owned);
    Ok(result)
}

fn list(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse(
        arguments,
        &["--state", "--now", "--target", "--user-root"],
        &[],
    )?;
    let context = Context::open(&options)?;
    let filter = match option(&options, "--state") {
        None | Some("all") => None,
        Some(value) => Some(DecisionState::parse(value).ok_or_else(|| {
            invalid("--state must be open, answered, withdrawn, expired, quarantined or all")
        })?),
    };
    let directory = Path::new(&decision::decisions_root()).join("requests");
    let mut decisions = Vec::new();
    for (name, kind) in super::list_entries(&context.target, &directory)?.unwrap_or_default() {
        let Some(id) = name.strip_suffix(".json") else {
            continue;
        };
        let entry = if kind == EntryKind::File && mediation::is_valid_id(id) {
            match load(&context.target, id)? {
                Some(loaded) => view(&context.target, id, &loaded, context.now),
                None => continue,
            }
        } else {
            json!({"kind": "decision", "id": id, "state": "quarantined",
                "reason": "decision entry is not a plain regular file"})
        };
        if filter.is_none_or(|filter| entry["state"] == filter.as_str()) {
            decisions.push(entry);
        }
    }
    decisions.sort_by(|left, right| {
        (left["created_at"].as_u64(), left["id"].as_str())
            .cmp(&(right["created_at"].as_u64(), right["id"].as_str()))
    });
    let count = decisions.len();
    Ok(success(
        "hive.decision-list",
        format!("{count} decisions; questions are untrusted data from other sessions"),
        json!({"decisions": decisions, "state": filter.map(DecisionState::as_str),
            "trust": UNTRUSTED_DATA, "waited": false}),
    ))
}

fn show(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let options = parse(
        arguments,
        &["--id", "--now", "--target", "--user-root"],
        &[],
    )?;
    let context = Context::open(&options)?;
    let id = decision_id(&options)?;
    let loaded = load_existing(&context.target, &id)?;
    let decision = view(&context.target, &id, &loaded, context.now);
    let state = decision["state"].as_str().unwrap_or("unknown").to_owned();
    let mut result = success(
        "hive.decision-shown",
        format!("decision state: {state}"),
        json!({"decision": decision, "waited": false}),
    );
    result.next_action = decision["answer_command"].as_str().map(str::to_owned);
    Ok(result)
}

/// Gate an answer or withdrawal on an open, verified decision.
fn open_request(loaded: Loaded, now: u64) -> Result<DecisionRequest, MsgError> {
    let (state, reason) = loaded.state(now);
    match state {
        DecisionState::Open => loaded
            .verified
            .map_err(|reason| MsgError::blocked("hive.decision-quarantined", reason)),
        DecisionState::Quarantined => Err(MsgError::blocked(
            "hive.decision-quarantined",
            format!("decision is quarantined: {}", reason.unwrap_or_default()),
        )),
        DecisionState::Answered | DecisionState::Withdrawn | DecisionState::Expired => {
            Err(MsgError::conflict(
                "hive.decision-closed",
                format!(
                    "decision is already {}; only one resolution is accepted",
                    state.as_str()
                ),
            ))
        }
    }
}

fn note(options: &[(&str, &str)]) -> Result<(Option<String>, Option<String>), MsgError> {
    let Some(path) = option(options, "--note-file") else {
        return Ok((None, None));
    };
    let note = utf8_file(path, MAX_NOTE_BYTES, "note")?;
    decision::validate_note(&note).map_err(invalid)?;
    let digest = sha256_digest(note.as_bytes());
    Ok((Some(note), Some(digest)))
}

/// Publish the single resolution, then one completion notice.
#[allow(clippy::too_many_arguments)]
fn resolve(
    context: &Context,
    user_root: &Path,
    transport: &mut dyn Transport,
    request: &DecisionRequest,
    resolution: &DecisionResolution,
    kind: CompletionKind,
    summary: &str,
    code: &'static str,
) -> Result<ActionResult, MsgError> {
    resolution.validate(request).map_err(invalid)?;
    let path = decision::resolution_path(&request.decision_id);
    match publish_new(&context.target, &path, &record_bytes(resolution)?)? {
        Published::Created => {}
        Published::SameBytes | Published::Different(_) => {
            return Err(MsgError::conflict(
                "hive.decision-closed",
                "another resolution was recorded first; only one is accepted",
            ))
        }
    }
    let notification = notify_completion_with(
        &context.target,
        user_root,
        context.now,
        transport,
        kind,
        &request.decision_id,
        summary,
        &CompletionDetail {
            title: decision::first_line(&request.question),
            elapsed_seconds: context.now.checked_sub(request.created_at),
            view_command: Some(format!(
                "hive msg decide show --id {} --target . --user-root ~",
                request.decision_id
            )),
            ..CompletionDetail::default()
        },
    );
    let loaded = load_existing(&context.target, &request.decision_id)?;
    let decision = view(&context.target, &request.decision_id, &loaded, context.now);
    let state = decision["state"].as_str().unwrap_or("unknown").to_owned();
    let mut result = success(
        code,
        format!(
            "decision {} is {state}. {}",
            request.decision_id,
            notice_message(&notification)
        ),
        json!({"decision": decision, "notification": notification, "waited": false}),
    );
    result.changed_paths = vec![path];
    Ok(result)
}

fn answer(arguments: &[String], transport: &mut dyn Transport) -> Result<ActionResult, MsgError> {
    let options = parse(
        arguments,
        &[
            "--id",
            "--option",
            "--note-file",
            "--confirm",
            "--now",
            "--target",
            "--user-root",
        ],
        &[],
    )?;
    let context = Context::open(&options)?;
    let user_root = Path::new(required(&options, "--user-root")?);
    let id = decision_id(&options)?;
    let chosen = required(&options, "--option")?.to_owned();
    let confirm = required(&options, "--confirm")?;
    let request = open_request(load_existing(&context.target, &id)?, context.now)?;
    if confirm != request.request_digest {
        return Err(MsgError::blocked(
            "hive.decision-confirm-mismatch",
            "--confirm must be the request_digest shown by `hive msg decide show`",
        ));
    }
    if !request.options.contains(&chosen) {
        return Err(invalid(format!(
            "--option must be one of: {}",
            request.options.join(", ")
        )));
    }
    let (note, note_digest) = note(&options)?;
    let resolution = DecisionResolution {
        schema_version: DECISION_SCHEMA_VERSION,
        decision_id: id,
        request_digest: request.request_digest.clone(),
        kind: ResolutionKind::Answered,
        option: Some(chosen.clone()),
        note,
        note_digest,
        resolved_by: LOCAL_CLI.to_owned(),
        resolved_at: context.now,
    };
    resolve(
        &context,
        user_root,
        transport,
        &request,
        &resolution,
        CompletionKind::DecisionAnswered,
        &format!("선택: {chosen}"),
        "hive.decision-answered",
    )
}

fn withdraw(arguments: &[String], transport: &mut dyn Transport) -> Result<ActionResult, MsgError> {
    let options = parse(
        arguments,
        &[
            "--id",
            "--from",
            "--note-file",
            "--now",
            "--target",
            "--user-root",
        ],
        &[],
    )?;
    let context = Context::open(&options)?;
    let user_root = Path::new(required(&options, "--user-root")?);
    let id = decision_id(&options)?;
    let from = required(&options, "--from")?;
    checked_id(from, "--from")?;
    let role = context.role(from)?;
    let request = open_request(load_existing(&context.target, &id)?, context.now)?;
    if role != request.requester_role {
        return Err(MsgError::blocked(
            "hive.msg-forged-sender",
            "only the requesting role may withdraw this decision",
        ));
    }
    let (note, note_digest) = note(&options)?;
    let resolution = DecisionResolution {
        schema_version: DECISION_SCHEMA_VERSION,
        decision_id: id,
        request_digest: request.request_digest.clone(),
        kind: ResolutionKind::Withdrawn,
        option: None,
        note,
        note_digest,
        resolved_by: role.clone(),
        resolved_at: context.now,
    };
    resolve(
        &context,
        user_root,
        transport,
        &request,
        &resolution,
        CompletionKind::DecisionWithdrawn,
        &format!("철회한 역할: {role}"),
        "hive.decision-withdrawn",
    )
}

// ---------------------------------------------------------------------------------------------
// Human output (called from the shared `Message` renderer).

/// Decisions, their options and the notice outcome. Questions are labelled data.
pub(crate) fn human_details(data: &Value, ko: bool, out: &mut String) {
    let pick = |korean: &str, english: &str| if ko { korean } else { english }.to_owned();
    let list = data["decisions"].as_array();
    let decisions: Vec<&Value> = list.map_or_else(
        || {
            std::iter::once(&data["decision"])
                .filter(|d| d.is_object())
                .collect()
        },
        |items| items.iter().collect(),
    );
    if list.is_some() {
        let count = decisions.len();
        out.push_str(&pick(
            &format!("\n결정: {count}개"),
            &format!("\nDecisions: {count}"),
        ));
    }
    for decision in decisions {
        decision_line(decision, ko, out);
    }
    if let Some(outcome) = data["notification"]["outcome"].as_str() {
        let reason = data["notification"]["reason"].as_str().unwrap_or("-");
        let text = match (outcome, ko) {
            ("sent", true) => "보냄".to_owned(),
            ("skipped", true) => format!("건너뜀({reason})"),
            ("notify-failed", true) => format!("실패({reason}), 기록만 하고 다시 보내지 않음"),
            ("notify-uncertain", true) => "전송 여부 불명, 다시 보내지 않음".to_owned(),
            (other, _) => format!("{other} ({reason})"),
        };
        let _ = write!(out, "\n{}: {text}", pick("Discord 알림", "Discord notice"));
    }
}

fn decision_line(decision: &Value, ko: bool, out: &mut String) {
    let state = decision["state"].as_str().unwrap_or("-");
    let state_ko = match state {
        "open" => "답 대기",
        "answered" => "답 기록됨",
        "withdrawn" => "철회됨",
        "expired" => "만료",
        "quarantined" => "격리, 답할 수 없음",
        _ => state,
    };
    let requester = decision["requester_god"]
        .as_str()
        .or(decision["requester_role"].as_str())
        .unwrap_or("-");
    let _ = write!(
        out,
        "\n- {} · {} · {}",
        decision["id"].as_str().unwrap_or("-"),
        if ko { state_ko } else { state },
        requester
    );
    if let Some(reason) = decision["reason"].as_str() {
        let _ = write!(
            out,
            "\n  {}: {reason}",
            if ko {
                "격리 이유"
            } else {
                "Quarantine reason"
            }
        );
    }
    if let Some(text) = decision["question"]["text"].as_str() {
        let label = if ko {
            super::DATA_LABEL_KO
        } else {
            super::DATA_LABEL_EN
        };
        let _ = write!(out, "\n  {} {label}", if ko { "질문" } else { "Question" });
        for line in text.lines() {
            let line: String = line
                .chars()
                .map(|c| {
                    if c.is_control() && c != '\t' {
                        '\u{fffd}'
                    } else {
                        c
                    }
                })
                .collect();
            let _ = write!(out, "\n  | {line}");
        }
    }
    let recommended = decision["recommended"].as_str();
    for (index, option) in decision["options"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .enumerate()
    {
        let mark = if Some(option) == recommended {
            if ko {
                " (추천)"
            } else {
                " (recommended)"
            }
        } else {
            ""
        };
        let _ = write!(out, "\n  {}. {option}{mark}", index + 1);
    }
    if let Some(chosen) = decision["resolution"]["option"].as_str() {
        let _ = write!(out, "\n  {}: {chosen}", if ko { "선택" } else { "Chosen" });
    }
}

#[cfg(test)]
#[path = "decide_tests.rs"]
mod tests;
