//! `hive msg flow`: builder → reviewer → advisor flows over the file inbox (OV160-004).
//!
//! A flow publishes its brief and an immutable definition under `.hive/mediation/flows/<id>/`,
//! then one ordinary inbox message per step. Step `n + 1` is created only after step `n` has an
//! acknowledged receipt (`msg ack` advances the flow; `msg flow advance` repeats that after a
//! restart). Step messages use the deterministic key `flow:<id>:<step>:<attempt>`, so repeated
//! advances never create a second message. A dispatch-uncertain or expired step blocks the flow
//! until `msg flow requeue --confirm <blocked envelope digest>` creates exactly one new attempt;
//! a quarantined step stops the flow. The status screen reads only Hive records: the profile
//! frozen at start and the message state derived from claim and receipt files. Hive never
//! launches a host or worker and never reads a CLI screen.
use super::{
    checked_id, containment, deliver, duration, failure, list_entries, load, option, publish_new,
    read_record, record_bytes, stop, success, Context, EntryKind, MsgError, Published, SendRequest,
};
use crate::run::{parse_options, read_explicit_file, required};
use crate::ActionResult;
use hive_core::flow::{
    self, check_cross_provider, derive_flow, status_line, FlowKind, FlowMessageLink, FlowRecord,
    FlowState, FlowStep, HostMode, PreviousResult, ReplacedMessage, StepDisplay, StepKind,
    StepLink, StepProfile, FLOWS_ROOT, FLOW_SCHEMA_VERSION, MAX_ATTEMPTS,
};
use hive_core::mediation::{
    self, MessageReceipt, MessageState, MAX_BODY_BYTES, MAX_RESULT_BYTES, UNTRUSTED_DATA,
};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

const ALLOW_SAME_PROVIDER: &str = "--allow-same-provider";
const MAX_INPUT_BYTES: usize = 1024 * 1024;

pub(super) fn run(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let rest = arguments.get(1..).unwrap_or_default();
    match arguments.first().map(String::as_str) {
        Some("start") => start(rest),
        Some("advance") => advance_command(rest),
        Some("status") => status_command(rest),
        Some("requeue") => requeue(rest),
        Some(other) => Err(MsgError::input(format!(
            "unknown msg flow command: {other}; use start, advance, status or requeue"
        ))),
        None => Err(MsgError::input("missing msg flow command")),
    }
}

// ---------------------------------------------------------------------------------------------
// Observation.

struct Attempt {
    link: Result<StepLink, String>,
    number: u32,
    state: MessageState,
    reason: Option<String>,
    envelope_digest: Option<String>,
    receipt: Option<MessageReceipt>,
}

/// Every attempt of one step, oldest first; the latest decides the step state.
struct StepObservation {
    attempts: Vec<Attempt>,
}

impl StepObservation {
    fn latest(&self) -> Option<&Attempt> {
        self.attempts.last()
    }
    fn state(&self) -> Option<MessageState> {
        self.latest().map(|attempt| attempt.state)
    }
}

fn load_flow(context: &Context, flow_id: &str) -> Result<FlowRecord, MsgError> {
    let flow_id = checked_id(flow_id, "--flow")?;
    let record: FlowRecord = match read_record(&context.target, &flow::flow_path(&flow_id))? {
        None => {
            return Err(MsgError::new(
                "error",
                2,
                "hive.msg-flow-not-found",
                format!("flow not found: {flow_id}"),
            ))
        }
        Some(Err(reason)) => return Err(MsgError::blocked("hive.msg-flow-quarantined", reason)),
        Some(Ok(record)) => record,
    };
    let invalid = |reason: String| MsgError::blocked("hive.msg-flow-quarantined", reason);
    record.validate().map_err(invalid)?;
    if record.flow_id != flow_id {
        return Err(invalid(
            "flow record is stored outside its own folder".into(),
        ));
    }
    let brief = context
        .target
        .read_optional(Path::new(&record.brief_file), MAX_BODY_BYTES)?
        .ok_or_else(|| invalid("flow brief is missing".into()))?;
    if brief.len() as u64 != record.brief_bytes || sha256_digest(&brief) != record.brief_digest {
        return Err(invalid("flow brief does not match its record".into()));
    }
    Ok(record)
}

fn observe(context: &Context, record: &FlowRecord) -> Result<Vec<StepObservation>, MsgError> {
    let mut observed: Vec<StepObservation> = record
        .steps
        .iter()
        .map(|_| StepObservation {
            attempts: Vec::new(),
        })
        .collect();
    let directory = flow::steps_dir(&record.flow_id);
    let mut names: Vec<(u32, u32, String)> = list_entries(&context.target, Path::new(&directory))?
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, kind)| *kind == EntryKind::File)
        .filter_map(|(name, _)| {
            flow::parse_step_link_name(&name).map(|(step, attempt)| (step, attempt, name))
        })
        .collect();
    names.sort();
    for (step, attempt, name) in names {
        let Some(slot) = observed.get_mut(step as usize) else {
            continue;
        };
        let path = format!("{directory}/{name}");
        let link = match read_record::<StepLink>(&context.target, &path)? {
            Some(Ok(link)) => link
                .validate(record)
                .and_then(|()| {
                    if link.step_index == step && link.attempt == attempt {
                        Ok(())
                    } else {
                        Err("step link is stored under another step or attempt".into())
                    }
                })
                .and_then(|()| {
                    let previous = slot.attempts.last().and_then(|a| a.link.as_ref().ok());
                    let chained = match (&link.replaces, previous) {
                        (None, None) => attempt == 1,
                        (Some(replaced), Some(previous)) => {
                            replaced.message_id == previous.message_id
                                && attempt == previous.attempt + 1
                        }
                        _ => false,
                    };
                    if chained {
                        Ok(link)
                    } else {
                        Err("step attempts are not an explicit requeue chain".into())
                    }
                }),
            Some(Err(reason)) => Err(reason),
            None => Err("step link disappeared".into()),
        };
        slot.attempts.push(observe_attempt(context, link, attempt)?);
    }
    Ok(observed)
}

fn observe_attempt(
    context: &Context,
    link: Result<StepLink, String>,
    attempt: u32,
) -> Result<Attempt, MsgError> {
    let quarantined = |link, reason: String| Attempt {
        link,
        number: attempt,
        state: MessageState::Quarantined,
        reason: Some(reason),
        envelope_digest: None,
        receipt: None,
    };
    let valid = match &link {
        Ok(valid) => valid.clone(),
        Err(reason) => {
            let reason = reason.clone();
            return Ok(quarantined(link, reason));
        }
    };
    let envelope = mediation::envelope_path(&valid.recipient_role, &valid.message_id);
    if context
        .target
        .read_optional(Path::new(&envelope), super::MAX_RECORD_BYTES)?
        .is_none()
    {
        return Ok(quarantined(link, "linked step message is missing".into()));
    }
    let loaded = load(context, &valid.recipient_role, &valid.message_id)?;
    let (state, reason) = loaded.state(context.now);
    Ok(Attempt {
        link,
        number: attempt,
        state,
        reason,
        envelope_digest: loaded.parsed.as_ref().map(|e| e.envelope_digest.clone()),
        receipt: loaded.receipt.and_then(Result::ok),
    })
}

fn derive(observed: &[StepObservation]) -> (FlowState, Option<usize>) {
    let states: Vec<Option<MessageState>> = observed.iter().map(StepObservation::state).collect();
    derive_flow(&states)
}

// ---------------------------------------------------------------------------------------------
// Profiles.

/// Roster-resolved profile; the god is the named god or the first god mapped to the role.
fn resolve_profile(
    context: &Context,
    name: &str,
    flag: &str,
    provider: Option<&str>,
) -> Result<StepProfile, MsgError> {
    checked_id(name, flag)?;
    let resolution = provider.map_or_else(
        || context.roster.resolve_named(name),
        |provider| context.roster.resolve_named_for(name, provider),
    );
    let (role, provider, resolved) =
        resolution.map_err(|error| MsgError::input(format!("{flag}: unknown role: {error}")))?;
    let god = hive_core::gods::find(name)
        .map(|god| god.name.to_owned())
        .or_else(|| {
            hive_core::gods::GODS
                .iter()
                .find(|god| context.roster.gods.get(god.name).is_some_and(|r| r == role))
                .map(|god| god.name.to_owned())
        });
    Ok(StepProfile {
        role: role.to_owned(),
        god,
        provider: provider.to_owned(),
        model: resolved.model.unwrap_or_default(),
        effort: resolved.effort.unwrap_or_default(),
    })
}

/// The recorded profile still matches the current roster resolution of its role.
fn profile_current(context: &Context, profile: &StepProfile) -> bool {
    context
        .roster
        .resolve_named_for(&profile.role, &profile.provider)
        .is_ok_and(|(_, provider, role)| {
            provider == profile.provider
                && role.model.as_deref() == Some(profile.model.as_str())
                && role.effort.as_deref() == Some(profile.effort.as_str())
        })
}

fn selected_provider_names(user_root: &str, except: &str) -> Result<Vec<String>, MsgError> {
    let root = crate::user_install::open_user_root_for_setup(Path::new(user_root))
        .map_err(MsgError::input)?;
    let Some(config) = crate::user_setup::load_operational_config(&root)
        .map_err(|error| MsgError::input(error.message().to_owned()))?
    else {
        return Ok(Vec::new());
    };
    Ok(config
        .selected_hosts
        .iter()
        .map(|host| match host.as_str() {
            "antigravity" => "gemini",
            provider => provider,
        })
        .filter(|provider| *provider != except)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

fn provider_command(options: &[(&str, &str)], providers: &[String]) -> String {
    let mut base = String::from("hive msg flow start");
    for (option, value) in options {
        if *option != "--reviewer-provider" {
            base.push(' ');
            base.push_str(option);
            base.push_str(" \"");
            base.push_str(&value.replace('"', "\\\""));
            base.push('"');
        }
    }
    let commands = if providers.is_empty() {
        format!("{base} --allow-same-provider")
    } else {
        providers
            .iter()
            .map(|provider| format!("{base} --reviewer-provider {provider}"))
            .collect::<Vec<_>>()
            .join(" OR ")
    };
    format!("Choose a reviewer provider and run: {commands}")
}

type StepInput = (String, Option<String>, Vec<u8>, Option<PreviousResult>);

// ---------------------------------------------------------------------------------------------
// Step creation.

/// Sender, body and previous-result reference of one step: the brief for the builder, the
/// previous acknowledged result (as data) for later steps.
fn step_input(
    context: &Context,
    record: &FlowRecord,
    observed: &[StepObservation],
    index: usize,
) -> Result<StepInput, MsgError> {
    let step = &record.steps[index];
    if index == 0 {
        let brief = context
            .target
            .read_optional(Path::new(&record.brief_file), MAX_BODY_BYTES)?
            .ok_or_else(|| MsgError::blocked("hive.msg-flow-quarantined", "brief is missing"))?;
        Ok((record.initiator_role.clone(), None, brief, None))
    } else {
        let previous_step = &record.steps[index - 1];
        let latest = observed[index - 1]
            .latest()
            .filter(|attempt| attempt.state == MessageState::Acknowledged);
        let (Some(latest), Some(receipt)) = (latest, latest.and_then(|a| a.receipt.as_ref()))
        else {
            return Err(MsgError::conflict(
                "hive.msg-flow-not-ready",
                "the previous step has no acknowledged receipt yet",
            ));
        };
        let bytes = context
            .target
            .read_optional(Path::new(&receipt.result_file), MAX_RESULT_BYTES)?
            .filter(|bytes| sha256_digest(bytes) == receipt.result_digest)
            .ok_or_else(|| {
                MsgError::blocked(
                    "hive.msg-flow-quarantined",
                    "the previous result file does not match its receipt",
                )
            })?;
        let previous = PreviousResult {
            message_id: latest
                .link
                .as_ref()
                .map_or_else(|_| String::new(), |link| link.message_id.clone()),
            result_file: receipt.result_file.clone(),
            result_digest: receipt.result_digest.clone(),
            result_bytes: receipt.result_bytes,
        };
        let body = flow::step_body(record, step, previous_step, &previous, &bytes);
        Ok((
            previous_step.profile.role.clone(),
            Some(previous_step.profile.provider.clone()),
            body,
            Some(previous),
        ))
    }
}

/// Publish one step attempt: message (idempotent key), then step link, then message link.
fn create_step(
    context: &Context,
    record: &FlowRecord,
    observed: &[StepObservation],
    index: usize,
    attempt: u32,
    replaces: Option<ReplacedMessage>,
) -> Result<(String, Vec<String>), MsgError> {
    // OV160-005: no step inside an active stop, and none after a pinned check changed.
    stop::gate(context, Some(&record.flow_id), Some(&record.task))?;
    containment::verify_checks(context, &record.flow_id)?;
    let step = &record.steps[index];
    if !profile_current(context, &step.profile) {
        return Err(MsgError::blocked(
            "hive.msg-flow-profile-changed",
            format!(
                "role {} no longer resolves to the recorded provider, model and effort; start a new flow",
                step.profile.role
            ),
        ));
    }
    let step_index = u32::try_from(index).unwrap_or(u32::MAX);
    let (sender_role, sender_provider, body, previous) =
        step_input(context, record, observed, index)?;
    let request = SendRequest {
        sender: match sender_provider {
            Some(provider) => context.sender_for_provider(&sender_role, &provider)?,
            None => context.sender(&sender_role)?,
        },
        recipient: step.profile.role.clone(),
        task: Some(record.task.clone()),
        body,
    };
    let key = flow::step_key(&record.flow_id, step_index, attempt);
    let delivered = deliver(context, request, &key, record.message_ttl_seconds)?;
    let mut changed = delivered.changed;
    let link = StepLink {
        schema_version: FLOW_SCHEMA_VERSION,
        flow_id: record.flow_id.clone(),
        step_index,
        attempt,
        message_id: delivered.message_id.clone(),
        recipient_role: step.profile.role.clone(),
        previous,
        replaces,
        trust: UNTRUSTED_DATA.into(),
    };
    link.validate(record).map_err(MsgError::input)?;
    let reverse = FlowMessageLink {
        schema_version: FLOW_SCHEMA_VERSION,
        message_id: delivered.message_id.clone(),
        flow_id: record.flow_id.clone(),
        step_index,
        attempt,
    };
    for (path, bytes) in [
        (
            flow::step_link_path(&record.flow_id, step_index, attempt),
            record_bytes(&link)?,
        ),
        (
            flow::message_link_path(&delivered.message_id),
            record_bytes(&reverse)?,
        ),
    ] {
        match publish_new(&context.target, &path, &bytes)? {
            Published::Created => changed.push(path),
            Published::SameBytes => {}
            Published::Different(_) => {
                return Err(MsgError::conflict(
                    "hive.msg-flow-conflict",
                    format!("{path} already holds a different step link"),
                ))
            }
        }
    }
    Ok((delivered.message_id, changed))
}

/// Observations after the call, the message created (if any) and the files written.
type Advanced = (Vec<StepObservation>, Option<String>, Vec<String>);

/// Create the next step when the flow is `ready`; otherwise change nothing.
fn advance(context: &Context, record: &FlowRecord) -> Result<Advanced, MsgError> {
    let observed = observe(context, record)?;
    let (state, current) = derive(&observed);
    let (FlowState::Ready, Some(index)) = (state, current) else {
        return Ok((observed, None, Vec::new()));
    };
    let (message_id, changed) = create_step(context, record, &observed, index, 1, None)?;
    Ok((observe(context, record)?, Some(message_id), changed))
}

// ---------------------------------------------------------------------------------------------
// Views.

fn flow_state_label(state: FlowState, korean: bool) -> &'static str {
    match (state, korean) {
        (FlowState::Ready, true) => "다음 단계 준비",
        (FlowState::Waiting, true) => "대기",
        (FlowState::Active, true) => "진행 중",
        (FlowState::Blocked, true) => "확인 불가, 자동 재개 없음",
        (FlowState::Stopped, true) => "격리·취소로 중단",
        (FlowState::Completed, true) => "완료",
        (FlowState::Ready, false) => "next step ready",
        (FlowState::Waiting, false) => "waiting",
        (FlowState::Active, false) => "in progress",
        (FlowState::Blocked, false) => "uncertain, no automatic resume",
        (FlowState::Stopped, false) => "stopped by quarantine or cancel",
        (FlowState::Completed, false) => "completed",
    }
}

fn step_view(context: &Context, step: &FlowStep, observed: &StepObservation) -> Value {
    let profile = &step.profile;
    let state = observed.state();
    let display = StepDisplay::from_state(state);
    let latest = observed.latest();
    let attempts: Vec<Value> = observed
        .attempts
        .iter()
        .map(|attempt| {
            json!({"attempt": attempt.number,
                "message_id": attempt.link.as_ref().ok().map(|l| l.message_id.clone()),
                "state": attempt.state, "reason": attempt.reason,
                "envelope_digest": attempt.envelope_digest})
        })
        .collect();
    json!({
        "index": step.index,
        "kind": step.kind,
        "role": profile.role,
        "god": profile.god,
        "god_display": profile.god_display(),
        "provider": profile.provider,
        "provider_display": flow::provider_display_name(&profile.provider),
        "model": profile.model,
        "model_display": flow::model_display_name(&profile.model),
        "effort": profile.effort,
        "host_mode": HostMode::for_provider(&profile.provider),
        "verified": profile.verified(),
        "roster_changed": !profile_current(context, profile),
        "state": state.map_or("not-created", MessageState::as_str),
        "display_state": display.as_str(),
        "label": {"ko": display.label(step.kind, true), "en": display.label(step.kind, false)},
        "display": {
            "ko": status_line(profile, step.kind, state, true),
            "en": status_line(profile, step.kind, state, false),
        },
        "attempt": latest.map(|a| a.number),
        "message_id": latest.and_then(|a| a.link.as_ref().ok()).map(|l| l.message_id.clone()),
        "envelope_digest": latest.and_then(|a| a.envelope_digest.clone()),
        "reason": latest.and_then(|a| a.reason.clone()),
        "attempts": attempts,
    })
}

/// Flow state with containment: a recorded or live check drift stops the flow.
fn effective(
    context: &Context,
    record: &FlowRecord,
    observed: &[StepObservation],
) -> Result<(FlowState, Option<usize>, Value), MsgError> {
    let (state, current) = derive(observed);
    let checks = containment::checks_view(context, &record.flow_id)?;
    if checks["drift"] == true && state != FlowState::Completed {
        return Ok((FlowState::Stopped, current, checks));
    }
    Ok((state, current, checks))
}

fn flow_view(
    context: &Context,
    record: &FlowRecord,
    observed: &[StepObservation],
) -> Result<Value, MsgError> {
    let (state, current, checks) = effective(context, record, observed)?;
    let stopped_by = stop::stops(context)?.covering(Some(&record.flow_id), Some(&record.task));
    let steps: Vec<Value> = record
        .steps
        .iter()
        .zip(observed)
        .map(|(step, observed)| step_view(context, step, observed))
        .collect();
    Ok(json!({
        "kind": "flow",
        "flow_id": record.flow_id,
        "flow_kind": record.kind,
        "task": record.task,
        "initiator_role": record.initiator_role,
        "state": state.as_str(),
        "state_label": {"ko": flow_state_label(state, true), "en": flow_state_label(state, false)},
        "current_step": current,
        "allow_same_provider": record.allow_same_provider,
        "brief": {"path": record.brief_file, "digest": record.brief_digest, "bytes": record.brief_bytes},
        "steps": steps,
        "checks": checks,
        "stopped_by": stopped_by,
        "source": "hive-records",
        "workers_launched": 0,
        "auto_redispatch": false,
        "trust": UNTRUSTED_DATA,
    }))
}

/// Status result for an unchanged flow; blocked and stopped flows say how to continue.
fn status_result(
    context: &Context,
    record: &FlowRecord,
    observed: &[StepObservation],
) -> Result<ActionResult, MsgError> {
    let (state, current, checks) = effective(context, record, observed)?;
    let view = flow_view(context, record, observed)?;
    Ok(match state {
        FlowState::Stopped if checks["drift"] == true => success(
            "hive.msg-flow-stopped",
            "a pinned score or check input changed after the flow started; the flow is quarantined and never continues",
            json!({"flow": view, "redispatched": false, "waited": false}),
        ),
        FlowState::Blocked => {
            let digest = current
                .and_then(|index| observed[index].latest())
                .and_then(|attempt| attempt.envelope_digest.clone())
                .unwrap_or_default();
            let mut result = success(
                "hive.msg-flow-blocked",
                "the current step is dispatch-uncertain or expired; Hive will not resume it automatically",
                json!({"flow": view, "redispatched": false, "waited": false}),
            );
            result.next_action = Some(format!(
                "Inspect the claimant host's own record, then run: hive msg flow requeue --flow {} --confirm {digest}",
                record.flow_id
            ));
            result
        }
        FlowState::Stopped => success(
            "hive.msg-flow-stopped",
            "a quarantined or cancelled step stopped the flow; it never continues",
            json!({"flow": view, "redispatched": false, "waited": false}),
        ),
        _ => success(
            "hive.msg-flow-status",
            format!("flow state: {}", state.as_str()),
            json!({"flow": view, "waited": false}),
        ),
    })
}

// ---------------------------------------------------------------------------------------------
// Commands.

/// One linear publication script: profiles, brief, definition, first step. Kept in one place on
/// purpose, like `msg send`.
#[allow(clippy::too_many_lines)]
fn start(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let allow_same_provider = arguments.iter().any(|a| a == ALLOW_SAME_PROVIDER);
    let arguments: Vec<String> = arguments
        .iter()
        .filter(|a| *a != ALLOW_SAME_PROVIDER)
        .cloned()
        .collect();
    let options = parse_options(
        &arguments,
        &[
            "--kind",
            "--builder",
            "--reviewer",
            "--reviewer-provider",
            "--advisor",
            "--task",
            "--file",
            "--from",
            "--key",
            "--ttl",
            "--check",
            "--worktree",
            "--now",
            "--target",
            "--user-root",
        ],
    )?;
    let context = Context::open(&options)?;
    if option(&options, "--worktree").is_some() && option(&options, "--check").is_none() {
        return Err(MsgError::input("--worktree is used only with --check"));
    }
    let kind =
        FlowKind::parse(option(&options, "--kind").unwrap_or("review")).map_err(MsgError::input)?;
    let builder = resolve_profile(
        &context,
        required(&options, "--builder")?,
        "--builder",
        None,
    )?;
    let reviewer_name = required(&options, "--reviewer")?;
    let reviewer_provider =
        if let Some(provider) = option(&options, "--reviewer-provider") {
            if !matches!(provider, "codex" | "claude" | "gemini") {
                return Err(MsgError::input(
                    "--reviewer-provider must be codex, claude or gemini",
                ));
            }
            Some(provider.to_owned())
        } else if allow_same_provider {
            None
        } else {
            let others =
                selected_provider_names(required(&options, "--user-root")?, &builder.provider)?;
            let roster_reviewer = resolve_profile(&context, reviewer_name, "--reviewer", None)?;
            let same_provider = check_cross_provider(&builder, &roster_reviewer, false).err();
            if same_provider.is_none() {
                // The roster already puts the reviewer on another provider: keep test.3 behavior.
                None
            } else if others.len() == 1 {
                Some(others[0].clone())
            } else {
                let reason = same_provider.unwrap_or_default();
                return Err(MsgError::blocked("hive.msg-flow-same-provider", reason)
                    .with_next_action(provider_command(
                        &options,
                        &context
                            .roster
                            .providers
                            .keys()
                            .filter(|provider| *provider != &builder.provider)
                            .cloned()
                            .collect::<Vec<_>>(),
                    )));
            }
        };
    let reviewer = resolve_profile(
        &context,
        reviewer_name,
        "--reviewer",
        reviewer_provider.as_deref(),
    )?;
    let advisor = option(&options, "--advisor")
        .map(|name| resolve_profile(&context, name, "--advisor", None))
        .transpose()?;
    check_cross_provider(&builder, &reviewer, allow_same_provider)
        .map_err(|reason| MsgError::blocked("hive.msg-flow-same-provider", reason))?;
    let task = checked_id(required(&options, "--task")?, "--task")?;
    let initiator = match option(&options, "--from") {
        Some(name) => {
            checked_id(name, "--from")?;
            context.sender(name)?.role
        }
        None => builder.role.clone(),
    };
    let brief = read_explicit_file(Path::new(required(&options, "--file")?), MAX_INPUT_BYTES)?;
    if brief.is_empty() || brief.len() > MAX_BODY_BYTES {
        return Err(MsgError::new(
            "error",
            2,
            "hive.msg-too-large",
            format!(
                "flow brief must contain 1..={MAX_BODY_BYTES} bytes; got {}",
                brief.len()
            ),
        ));
    }
    if std::str::from_utf8(&brief).is_err() {
        return Err(MsgError::input("flow brief must be UTF-8 text"));
    }
    let brief_digest = sha256_digest(&brief);
    let mut profiles = vec![builder, reviewer];
    profiles.extend(advisor);
    let roles: Vec<&str> = profiles.iter().map(|p| p.role.as_str()).collect();
    // A scored flow names its pinned check in the automatic key, so a changed check is a new flow.
    let check_digest = option(&options, "--check")
        .map(|check| {
            read_explicit_file(Path::new(check), containment::MAX_CHECK_BYTES)
                .map(|bytes| sha256_digest(&bytes))
        })
        .transpose()?;
    let key = option(&options, "--key").map_or_else(
        || {
            let key = flow::default_flow_key(kind, &task, &initiator, &roles, &brief_digest);
            check_digest
                .as_ref()
                .map_or_else(|| key.clone(), |digest| format!("{key}:check:{digest}"))
        },
        str::to_owned,
    );
    let flow_id = flow::flow_id_for(&mediation::idempotency_digest(&key).map_err(MsgError::input)?);
    // Inside an active stop nothing is written, not even the brief.
    stop::gate(&context, Some(&flow_id), Some(&task))?;
    let pins = option(&options, "--check")
        .map(|check| {
            containment::pin_check(&context, &flow_id, check, option(&options, "--worktree"))
        })
        .transpose()?;
    let kinds = [StepKind::Builder, StepKind::Reviewer, StepKind::Advisor];
    let record = FlowRecord {
        schema_version: FLOW_SCHEMA_VERSION,
        brief_file: flow::brief_path(&flow_id),
        flow_id: flow_id.clone(),
        kind,
        task,
        initiator_role: initiator,
        steps: profiles
            .into_iter()
            .zip(kinds)
            .zip(0_u32..)
            .map(|((profile, kind), index)| FlowStep {
                index,
                kind,
                profile,
            })
            .collect(),
        allow_same_provider,
        brief_digest,
        brief_bytes: brief.len() as u64,
        message_ttl_seconds: duration(&options, "--ttl", mediation::DEFAULT_MESSAGE_TTL_SECONDS)?,
        created_at: context.now,
        trust: UNTRUSTED_DATA.into(),
        flow_digest: String::new(),
    }
    .seal()
    .map_err(MsgError::input)?;
    let mut changed = Vec::new();
    match publish_new(&context.target, &record.brief_file, &brief)? {
        Published::Created => changed.push(record.brief_file.clone()),
        Published::SameBytes => {}
        Published::Different(_) => {
            return Err(MsgError::conflict(
                "hive.msg-key-conflict",
                "this flow key already names a different brief; use a new --key",
            ))
        }
    }
    let path = flow::flow_path(&flow_id);
    let duplicate = match publish_new(&context.target, &path, &record_bytes(&record)?)? {
        Published::Created => {
            changed.push(path);
            false
        }
        Published::SameBytes => true,
        Published::Different(existing) => {
            let same = serde_json::from_slice::<FlowRecord>(&existing)
                .is_ok_and(|existing| existing.same_definition(&record));
            if !same {
                return Err(MsgError::conflict(
                    "hive.msg-key-conflict",
                    "this flow key already names a different flow; use a new --key",
                ));
            }
            true
        }
    };
    if let Some(pins) = pins {
        let path = hive_core::containment::check_pin_path(&flow_id);
        match publish_new(&context.target, &path, &record_bytes(&pins)?)? {
            Published::Created => changed.push(path),
            Published::SameBytes => {}
            Published::Different(_) => {
                return Err(MsgError::conflict(
                    "hive.msg-key-conflict",
                    "this flow already pinned different check inputs; use a new --key",
                ))
            }
        }
    }
    let record = load_flow(&context, &flow_id)?;
    let (observed, created, step_changed) = advance(&context, &record)?;
    changed.extend(step_changed);
    let (code, message) = if duplicate {
        (
            "hive.msg-flow-duplicate",
            "same flow already exists; returned its current state and created no duplicate step",
        )
    } else {
        (
            "hive.msg-flow-started",
            "flow started; the builder step is queued and nothing waited or launched",
        )
    };
    let mut result = success(
        code,
        message,
        json!({"flow": flow_view(&context, &record, &observed)?, "duplicate": duplicate,
            "created_message": created, "waited": false}),
    );
    result.changed_paths = changed;
    Ok(result)
}

fn flow_options(
    arguments: &[String],
    extra: &[&'static str],
) -> Result<Vec<(String, String)>, MsgError> {
    let mut allowed = vec!["--flow", "--now", "--target", "--user-root"];
    allowed.extend_from_slice(extra);
    Ok(parse_options(arguments, &allowed)?
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect())
}

fn borrowed(options: &[(String, String)]) -> Vec<(&str, &str)> {
    options
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect()
}

fn advance_command(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let owned = flow_options(arguments, &[])?;
    let options = borrowed(&owned);
    let context = Context::open(&options)?;
    let record = load_flow(&context, required(&options, "--flow")?)?;
    let (observed, created, changed) = advance(&context, &record)?;
    let Some(created) = created else {
        return status_result(&context, &record, &observed);
    };
    let mut result = success(
        "hive.msg-flow-advanced",
        "the previous step is acknowledged; the next step message is queued",
        json!({"flow": flow_view(&context, &record, &observed)?, "created_message": created,
            "waited": false}),
    );
    result.changed_paths = changed;
    Ok(result)
}

fn status_command(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let owned = flow_options(arguments, &[])?;
    let options = borrowed(&owned);
    let context = Context::open(&options)?;
    let record = load_flow(&context, required(&options, "--flow")?)?;
    let observed = observe(&context, &record)?;
    status_result(&context, &record, &observed)
}

/// Explicit resume of a blocked step: one new attempt for the confirmed blocked envelope.
fn requeue(arguments: &[String]) -> Result<ActionResult, MsgError> {
    let owned = flow_options(arguments, &["--confirm"])?;
    let options = borrowed(&owned);
    let context = Context::open(&options)?;
    let record = load_flow(&context, required(&options, "--flow")?)?;
    let confirm = required(&options, "--confirm")?;
    let observed = observe(&context, &record)?;
    let already = observed
        .iter()
        .flat_map(|step| &step.attempts)
        .any(|attempt| {
            attempt
                .link
                .as_ref()
                .ok()
                .and_then(|link| link.replaces.as_ref())
                .is_some_and(|replaced| replaced.envelope_digest == confirm)
        });
    if already {
        return Ok(success(
            "hive.msg-flow-requeue-duplicate",
            "this blocked message was already requeued; nothing new was created",
            json!({"flow": flow_view(&context, &record, &observed)?, "duplicate": true,
                "redispatched": false, "waited": false}),
        ));
    }
    let view = flow_view(&context, &record, &observed)?;
    let refused = |message: &str| {
        Ok(failure(
            MsgError::conflict("hive.msg-flow-requeue-refused", message),
            Some(json!({"flow": view.clone()})),
        ))
    };
    let (state, current) = derive(&observed);
    let (FlowState::Blocked, Some(index)) = (state, current) else {
        return refused(if state == FlowState::Stopped {
            "a quarantined step stops the flow; start a new flow instead"
        } else {
            "only a dispatch-uncertain or expired step can be requeued"
        });
    };
    let Some(latest) = observed[index].latest() else {
        return refused("the blocked step has no message");
    };
    let (Ok(link), Some(digest)) = (&latest.link, latest.envelope_digest.as_deref()) else {
        return refused("the blocked step link is unreadable");
    };
    if digest != confirm {
        return refused("--confirm must be the blocked message's envelope digest from status");
    }
    if latest.number >= MAX_ATTEMPTS {
        return refused("this step has no attempts left; start a new flow");
    }
    let replaces = ReplacedMessage {
        message_id: link.message_id.clone(),
        envelope_digest: digest.to_owned(),
    };
    let (created, changed) = create_step(
        &context,
        &record,
        &observed,
        index,
        latest.number + 1,
        Some(replaces),
    )?;
    let observed = observe(&context, &record)?;
    let mut result = success(
        "hive.msg-flow-requeued",
        "a new message for the blocked step is queued; the uncertain message stays as evidence",
        json!({"flow": flow_view(&context, &record, &observed)?, "created_message": created,
            "redispatched": false, "waited": false}),
    );
    result.changed_paths = changed;
    Ok(result)
}

/// After a receipt exists, advance the message's flow. Errors stay in the data; the receipt
/// itself is already recorded.
pub(super) fn after_ack(
    context: &Context,
    message_id: &str,
    result: &mut ActionResult,
) -> Result<(), MsgError> {
    let Some(link) =
        read_record::<FlowMessageLink>(&context.target, &flow::message_link_path(message_id))?
    else {
        return Ok(());
    };
    let outcome = link
        .map_err(|reason| MsgError::blocked("hive.msg-flow-quarantined", reason))
        .and_then(|link| {
            let record = load_flow(context, &link.flow_id)?;
            let (observed, created, changed) = advance(context, &record)?;
            Ok((flow_view(context, &record, &observed)?, created, changed))
        });
    let data = result.data.get_or_insert_with(|| json!({}));
    match outcome {
        Ok((view, created, changed)) => {
            data["flow"] = view;
            data["created_message"] = json!(created);
            result.changed_paths.extend(changed);
        }
        Err(error) => {
            data["flow_error"] = json!({"code": error.code, "message": error.message});
        }
    }
    Ok(())
}

/// Unfinished flows and their current step line for `msg inbox`.
pub(super) fn inbox_entries(context: &Context, role: Option<&str>) -> Result<Vec<Value>, MsgError> {
    let mut entries = Vec::new();
    for (name, kind) in list_entries(&context.target, Path::new(FLOWS_ROOT))?.unwrap_or_default() {
        if kind != EntryKind::Dir || !mediation::is_valid_id(&name) {
            continue;
        }
        let record = match load_flow(context, &name) {
            Ok(record) => record,
            Err(error) => {
                entries.push(
                    json!({"flow_id": name, "state": "quarantined", "reason": error.message}),
                );
                continue;
            }
        };
        let observed = observe(context, &record)?;
        let (state, current) = derive(&observed);
        let Some(index) = current else {
            continue;
        };
        let step = &record.steps[index];
        if role.is_some_and(|role| role != step.profile.role) {
            continue;
        }
        entries.push(json!({
            "flow_id": record.flow_id,
            "task": record.task,
            "state": state.as_str(),
            "current_step": step_view(context, step, &observed[index]),
        }));
    }
    Ok(entries)
}
