//! Durable provider-neutral orchestration CLI.
//!
//! The adapter persists declarative events and cooperative host receipts. It
//! never launches a host process or calls a model provider.

use super::{emit_action_result, ActionResult, Evidence};
use crate::run::{
    parse_options, portable_relative_path, read_explicit_file, required, run_path, AdapterError,
    FileSnapshot, PinnedTarget,
};
use hive_core::orchestration::{
    replay_events, ActionAuthority, AuthorityAction, AuthorityExpectation, EventHead, EventKind,
    HostReceipt, OrchestrationEvent, OrchestrationTrustRoot, ReceiptKind, ReducerState,
};
use hive_core::sha256_digest;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_REPLAY_EVENTS: u64 = 4096;
const ORCHESTRATION_USAGE: &str = "\
Manage provider-neutral durable orchestration state without spawning a host process.

USAGE:
    hive orchestration status --target <dir> --run <id> --output json
    hive orchestration plan|dispatch|cancel|recover --target <dir> --run <id> --event <event.toml|json> --expected-head <none|sequence:sha256> --control-epoch <n> --authority <authority.toml|json> --trust-root <external-root.toml|json> --request-digest <sha256> --now <ordered-time> --output json
    hive orchestration receipt --target <dir> --run <id> --event <event.toml|json> --receipt <receipt.json> --expected-head <none|sequence:sha256> --control-epoch <n> --authority <authority.toml|json> --trust-root <external-root.toml|json> --request-digest <sha256> --now <ordered-time> --output json
    hive orchestration authority issue|revoke --help
    hive orchestration migrate --help
";

#[derive(Debug)]
enum CliError {
    Adapter(AdapterError),
    Core(String),
}

impl From<AdapterError> for CliError {
    fn from(error: AdapterError) -> Self {
        Self::Adapter(error)
    }
}

impl CliError {
    fn status(&self) -> &'static str {
        match self {
            Self::Adapter(error) => error.status(),
            Self::Core(_) => "verification-failed",
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Adapter(error) => error.exit_code(),
            Self::Core(_) => 5,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Adapter(error) => error.code(),
            Self::Core(_) => "hive.orchestration-verification-failed",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Adapter(error) => error.message(),
            Self::Core(message) => message,
        }
    }
}

#[derive(Debug)]
struct MutationArguments {
    target: PathBuf,
    run_id: String,
    event: PathBuf,
    expected_head: String,
    control_epoch: u64,
    authority: PathBuf,
    trust_root: PathBuf,
    request_digest: String,
    now: String,
    receipt: Option<PathBuf>,
}

struct ValidatedPayload {
    event: OrchestrationEvent,
    receipt: Option<(HostReceipt, Vec<u8>)>,
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.is_empty()
        || arguments
            .iter()
            .any(|argument| argument == "--help" || argument == "-h")
    {
        print!("{ORCHESTRATION_USAGE}");
        return ExitCode::SUCCESS;
    }
    let action = action_name(arguments.first().map(String::as_str).unwrap_or_default());
    let result = match execute(arguments) {
        Ok(result) => result,
        Err(error) => ActionResult {
            schema_version: 1,
            action,
            status: error.status(),
            exit_code: error.exit_code(),
            code: error.code(),
            message: error.message().to_owned(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
            data: None,
        },
    };
    emit_action_result(&result)
}

fn execute(arguments: &[String]) -> Result<ActionResult, CliError> {
    match arguments.first().map(String::as_str) {
        Some("status") => status(&arguments[1..]),
        Some(command @ ("plan" | "dispatch" | "receipt" | "cancel" | "recover")) => {
            let parsed = parse_mutation(&arguments[1..], command == "receipt")?;
            mutate(command, &parsed)
        }
        Some("authority") => Err(AdapterError::Unsupported(
            "authority issue and revoke are not enabled before the signed-ledger stage".to_owned(),
        )
        .into()),
        Some("migrate") => Err(AdapterError::Unsupported(
            "legacy migration is not enabled before the staging-recovery stage".to_owned(),
        )
        .into()),
        Some(other) => {
            Err(AdapterError::Input(format!("unknown orchestration action: {other}")).into())
        }
        None => Err(AdapterError::Input("missing orchestration action".to_owned()).into()),
    }
}

fn status(arguments: &[String]) -> Result<ActionResult, CliError> {
    let options = parse_options(arguments, &["--target", "--run"])?;
    let target = PinnedTarget::open(Path::new(required(&options, "--target")?))?;
    let run_id = required(&options, "--run")?;
    let loaded = load_chain(&target, run_id)?;
    let head = loaded
        .head
        .as_ref()
        .map_or_else(|| "none".to_owned(), EventHead::binding);
    let state = loaded
        .state
        .state
        .map(|value| format!("{value:?}").to_ascii_lowercase());
    let mut evidence = Vec::new();
    if let (Some(head), Some(bytes)) = (loaded.head.as_ref(), loaded.head_bytes.as_ref()) {
        evidence.push(Evidence {
            kind: "orchestration-head",
            locator: portable_relative_path(&run_path(run_id, "EVENT-CURRENT.toml")?),
            digest: sha256_digest(bytes),
        });
        evidence.push(Evidence {
            kind: "orchestration-event",
            locator: portable_relative_path(&event_path(run_id, head.sequence)?),
            digest: head.event_digest.clone(),
        });
    }
    Ok(success(
        "OrchestrationStatus",
        "hive.orchestration-status",
        "canonical orchestration event chain verified",
        Vec::new(),
        evidence,
        json!({
            "run_id": run_id,
            "head": head,
            "control_epoch": loaded.state.control_epoch,
            "state": state,
            "event_count": loaded.events.len(),
            "target_digest": target_digest(&target),
            "activation": "off",
            "host_process_spawned": false
        }),
    ))
}

fn parse_mutation(
    arguments: &[String],
    needs_receipt: bool,
) -> Result<MutationArguments, CliError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--run",
            "--event",
            "--expected-head",
            "--control-epoch",
            "--authority",
            "--trust-root",
            "--request-digest",
            "--now",
            "--receipt",
        ],
    )?;
    let receipt = option(&options, "--receipt").map(PathBuf::from);
    if needs_receipt != receipt.is_some() {
        return Err(AdapterError::Input(if needs_receipt {
            "receipt mutation requires --receipt".to_owned()
        } else {
            "--receipt is only valid for the receipt action".to_owned()
        })
        .into());
    }
    let expected_head = required(&options, "--expected-head")?.to_owned();
    validate_head(&expected_head)?;
    let request_digest = required(&options, "--request-digest")?.to_owned();
    validate_digest(&request_digest, "request digest")?;
    let control_epoch = required(&options, "--control-epoch")?
        .parse::<u64>()
        .map_err(|_| AdapterError::Input("control epoch must be unsigned".to_owned()))?;
    Ok(MutationArguments {
        target: PathBuf::from(required(&options, "--target")?),
        run_id: required(&options, "--run")?.to_owned(),
        event: PathBuf::from(required(&options, "--event")?),
        expected_head,
        control_epoch,
        authority: PathBuf::from(required(&options, "--authority")?),
        trust_root: PathBuf::from(required(&options, "--trust-root")?),
        request_digest,
        now: required(&options, "--now")?.to_owned(),
        receipt,
    })
}

fn mutate(command: &str, arguments: &MutationArguments) -> Result<ActionResult, CliError> {
    let target = PinnedTarget::open(&arguments.target)?;
    validate_external_root(&target, &arguments.trust_root)?;
    let mut loaded = load_chain(&target, &arguments.run_id)?;
    let actual_head = loaded
        .head
        .as_ref()
        .map_or_else(|| "none".to_owned(), EventHead::binding);
    if arguments.expected_head != actual_head
        || arguments.control_epoch != loaded.state.control_epoch
    {
        return Err(AdapterError::Conflict(format!(
            "stale expectation; current head is {actual_head} at epoch {}",
            loaded.state.control_epoch
        ))
        .into());
    }

    let payload =
        validate_mutation_payload(command, arguments, &target, &mut loaded, &actual_head)?;
    let event_digest = loaded
        .state
        .apply_event(&payload.event)
        .map_err(|error| CliError::Core(error.to_string()))?;
    let receipt_change = payload
        .receipt
        .as_ref()
        .map(|(receipt, bytes)| publish_receipt(&target, &arguments.run_id, receipt, bytes))
        .transpose()?;
    commit_event(
        command,
        arguments,
        &target,
        loaded,
        &payload.event,
        event_digest,
        receipt_change,
    )
}

fn validate_mutation_payload(
    command: &str,
    arguments: &MutationArguments,
    target: &PinnedTarget,
    loaded: &mut LoadedChain,
    actual_head: &str,
) -> Result<ValidatedPayload, CliError> {
    let event = parse_event(&read_explicit_file(&arguments.event, MAX_INPUT_BYTES)?)?;
    validate_command_event(command, &event)?;
    let event_epoch = if command == "cancel" {
        arguments.control_epoch.saturating_add(1)
    } else {
        arguments.control_epoch
    };
    if event.run_id != arguments.run_id
        || event.request_digest != arguments.request_digest
        || event.control_epoch != event_epoch
        || event.sequence != loaded.state.sequence.saturating_add(1)
        || event.predecessor_digest != loaded.state.event_digest
        || event.from_state != loaded.state.state
    {
        return Err(AdapterError::Verification(
            "event does not match the exact run, request, predecessor, sequence, state, or epoch"
                .to_owned(),
        )
        .into());
    }

    let authority = parse_authority(&read_explicit_file(&arguments.authority, MAX_INPUT_BYTES)?)?;
    let root = parse_root(&read_explicit_file(&arguments.trust_root, MAX_INPUT_BYTES)?)?;
    let target_binding = target_digest(target);
    authority
        .verify(
            &root,
            AuthorityExpectation {
                action: authority_action(command),
                target_digest: &target_binding,
                head: &arguments.expected_head,
                control_epoch: arguments.control_epoch,
                request_digest: &arguments.request_digest,
                now: &arguments.now,
            },
        )
        .map_err(|error| CliError::Core(error.to_string()))?;
    if event.authority_id != authority.authority_id {
        return Err(AdapterError::Verification("event authority id mismatch".to_owned()).into());
    }

    let mut receipt_payload = None;
    if let Some(path) = arguments.receipt.as_deref() {
        let bytes = read_explicit_file(path, MAX_INPUT_BYTES)?;
        let receipt =
            HostReceipt::parse_json(&bytes).map_err(|error| CliError::Core(error.to_string()))?;
        validate_receipt(&receipt, &event, actual_head, arguments.control_epoch)?;
        loaded
            .state
            .bind_receipt(&receipt)
            .map_err(|error| CliError::Core(error.to_string()))?;
        receipt_payload = Some((receipt, bytes));
    }
    Ok(ValidatedPayload {
        event,
        receipt: receipt_payload,
    })
}

fn commit_event(
    command: &str,
    arguments: &MutationArguments,
    target: &PinnedTarget,
    mut loaded: LoadedChain,
    event: &OrchestrationEvent,
    event_digest: String,
    receipt_change: Option<PathBuf>,
) -> Result<ActionResult, CliError> {
    let (event_relative, event_bytes) = render_event(event)?;
    publish_immutable(
        target,
        &run_path(&arguments.run_id, "events")?,
        &event_relative,
        &event_bytes,
        "event revision",
    )?;

    let head = EventHead {
        schema_version: 1,
        generation: loaded.head.as_ref().map_or(1, |value| value.generation + 1),
        sequence: event.sequence,
        event_digest: event_digest.clone(),
        control_epoch: event.control_epoch,
    };
    let head_relative = run_path(&arguments.run_id, "EVENT-CURRENT.toml")?;
    let expected = loaded
        .head_bytes
        .take()
        .map_or(FileSnapshot::Missing, FileSnapshot::File);
    let head_bytes = toml::to_string(&head)
        .map_err(|error| CliError::Core(format!("cannot serialize event head: {error}")))?
        .into_bytes();
    target.publish(&head_relative, &expected, &head_bytes)?;

    let mut changed = vec![
        portable_relative_path(&event_relative),
        portable_relative_path(&head_relative),
    ];
    if let Some(relative) = receipt_change {
        changed.push(portable_relative_path(&relative));
    }
    if let Some(relative) = materialize_control(target, event, &head)? {
        changed.push(portable_relative_path(&relative));
    }
    Ok(success(
        action_name(command),
        "hive.orchestration-committed",
        "immutable event committed by exact event-head CAS",
        changed,
        vec![Evidence {
            kind: "orchestration-event",
            locator: portable_relative_path(&event_relative),
            digest: event_digest,
        }],
        json!({
            "run_id": arguments.run_id,
            "action_id": event.action_id,
            "head": head.binding(),
            "control_epoch": head.control_epoch,
            "state": event.to_state,
            "host_process_spawned": false
        }),
    ))
}

fn validate_command_event(command: &str, event: &OrchestrationEvent) -> Result<(), CliError> {
    let valid = match command {
        "plan" => matches!(event.kind, EventKind::Reserve | EventKind::Prepare),
        "dispatch" => matches!(
            event.kind,
            EventKind::Claim | EventKind::MarkDispatchUncertain
        ),
        "receipt" => matches!(
            event.kind,
            EventKind::Claim
                | EventKind::Acknowledge
                | EventKind::Heartbeat
                | EventKind::ReceiveResult
                | EventKind::Quarantine
        ),
        "cancel" => event.kind == EventKind::RequestCancel,
        "recover" => event.kind == EventKind::Recover,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AdapterError::Input(format!(
            "event kind is not permitted for orchestration {command}"
        ))
        .into())
    }
}

fn validate_receipt(
    receipt: &HostReceipt,
    event: &OrchestrationEvent,
    head: &str,
    epoch: u64,
) -> Result<(), CliError> {
    let kind = match receipt.kind {
        ReceiptKind::Claim => EventKind::Claim,
        ReceiptKind::LaunchAck => EventKind::Acknowledge,
        ReceiptKind::Heartbeat => EventKind::Heartbeat,
        ReceiptKind::FinalResult => EventKind::ReceiveResult,
        ReceiptKind::Lookup | ReceiptKind::NonLaunchProof | ReceiptKind::CancelAck => {
            EventKind::Quarantine
        }
    };
    if event.kind == kind
        && receipt.run_id == event.run_id
        && receipt.action_id == event.action_id
        && receipt.event_head == head
        && receipt.control_epoch == epoch
    {
        Ok(())
    } else {
        Err(AdapterError::Verification(
            "receipt does not match the exact event, head, or epoch".to_owned(),
        )
        .into())
    }
}

fn publish_receipt(
    target: &PinnedTarget,
    run_id: &str,
    receipt: &HostReceipt,
    source: &[u8],
) -> Result<PathBuf, CliError> {
    let relative = run_path(run_id, &format!("receipts/{}.md", receipt.receipt_id))?;
    let body = format!(
        "+++\n{}+++\n\n# Host receipt\n\nSource digest: {}\n",
        toml::to_string(receipt)
            .map_err(|error| CliError::Core(format!("cannot serialize receipt: {error}")))?,
        sha256_digest(source)
    )
    .into_bytes();
    publish_immutable(
        target,
        &run_path(run_id, "receipts")?,
        &relative,
        &body,
        "receipt",
    )?;
    Ok(relative)
}

fn publish_immutable(
    target: &PinnedTarget,
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    label: &str,
) -> Result<(), CliError> {
    target.ensure_owned_parent(relative, root)?;
    match target.snapshot(relative)? {
        FileSnapshot::Missing => {
            target.publish(relative, &FileSnapshot::Missing, bytes)?;
        }
        FileSnapshot::File(existing) if existing == bytes => {}
        FileSnapshot::File(_) => {
            return Err(AdapterError::Conflict(format!(
                "immutable {label} already has different bytes: {}",
                relative.display()
            ))
            .into());
        }
    }
    Ok(())
}

fn materialize_control(
    target: &PinnedTarget,
    event: &OrchestrationEvent,
    head: &EventHead,
) -> Result<Option<PathBuf>, CliError> {
    if event.kind != EventKind::RequestCancel {
        return Ok(None);
    }
    let relative = run_path(&event.run_id, "CONTROL.md")?;
    let bytes = format!(
        "+++\nschema_version = 1\ncontrol_epoch = {}\nevent_head = \"{}\"\ndesired_state = \"cancelled\"\nauthority_id = \"{}\"\nrequest_digest = \"{}\"\n+++\n\n# Orchestration control\n\nCancellation is committed at the event head above.\n",
        head.control_epoch,
        head.binding(),
        event.authority_id,
        event.request_digest
    )
    .into_bytes();
    let expected = target.snapshot(&relative)?;
    target.publish(&relative, &expected, &bytes)?;
    Ok(Some(relative))
}

struct LoadedChain {
    events: Vec<OrchestrationEvent>,
    state: hive_core::orchestration::ReducerState,
    head: Option<EventHead>,
    head_bytes: Option<Vec<u8>>,
}

fn load_chain(target: &PinnedTarget, run_id: &str) -> Result<LoadedChain, CliError> {
    let head_relative = run_path(run_id, "EVENT-CURRENT.toml")?;
    let head_bytes = target.read_optional(&head_relative, MAX_INPUT_BYTES)?;
    let Some(bytes) = head_bytes.as_ref() else {
        return Ok(LoadedChain {
            events: Vec::new(),
            state: ReducerState::default(),
            head: None,
            head_bytes: None,
        });
    };
    let head: EventHead = toml::from_str(
        std::str::from_utf8(bytes)
            .map_err(|_| AdapterError::Verification("event head is not UTF-8".to_owned()))?,
    )
    .map_err(|error| AdapterError::Verification(format!("invalid event head: {error}")))?;
    if head.schema_version != 1 || head.sequence == 0 || head.sequence > MAX_REPLAY_EVENTS {
        return Err(
            AdapterError::Verification("event head is outside replay bounds".to_owned()).into(),
        );
    }
    let mut events = Vec::with_capacity(usize::try_from(head.sequence).unwrap_or(0));
    for sequence in 1..=head.sequence {
        let relative = event_path(run_id, sequence)?;
        let event = parse_event_markdown(&target.read_required(&relative, MAX_INPUT_BYTES)?)?;
        if event.sequence != sequence || event.run_id != run_id {
            return Err(AdapterError::Verification(format!(
                "event revision {sequence} has the wrong sequence or run id"
            ))
            .into());
        }
        events.push(event);
    }
    let state = replay_events(&events).map_err(|error| CliError::Core(error.to_string()))?;
    if state.sequence != head.sequence
        || state.event_digest.as_deref() != Some(head.event_digest.as_str())
        || state.control_epoch != head.control_epoch
    {
        return Err(AdapterError::Verification(
            "event head does not match deterministic replay".to_owned(),
        )
        .into());
    }
    Ok(LoadedChain {
        events,
        state,
        head: Some(head),
        head_bytes,
    })
}

fn parse_event(bytes: &[u8]) -> Result<OrchestrationEvent, CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AdapterError::Input("event input is not UTF-8".to_owned()))?;
    if text.trim_start().starts_with('{') {
        serde_json::from_str(text)
            .map_err(|error| AdapterError::Input(format!("invalid event JSON: {error}")).into())
    } else {
        toml::from_str(text)
            .map_err(|error| AdapterError::Input(format!("invalid event TOML: {error}")).into())
    }
}

fn parse_event_markdown(bytes: &[u8]) -> Result<OrchestrationEvent, CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AdapterError::Verification("event revision is not UTF-8".to_owned()))?;
    let frontmatter = text
        .strip_prefix("+++\n")
        .and_then(|rest| rest.split_once("+++\n"))
        .map(|(frontmatter, _)| frontmatter)
        .ok_or_else(|| {
            AdapterError::Verification("event revision lacks TOML frontmatter".to_owned())
        })?;
    toml::from_str(frontmatter).map_err(|error| {
        AdapterError::Verification(format!("invalid event revision: {error}")).into()
    })
}

fn parse_authority(bytes: &[u8]) -> Result<ActionAuthority, CliError> {
    if first_byte(bytes) == Some(b'{') {
        ActionAuthority::parse_json(bytes).map_err(|error| CliError::Core(error.to_string()))
    } else {
        toml::from_str(
            std::str::from_utf8(bytes)
                .map_err(|_| AdapterError::Input("authority is not UTF-8".to_owned()))?,
        )
        .map_err(|error| AdapterError::Input(format!("invalid authority TOML: {error}")).into())
    }
}

fn parse_root(bytes: &[u8]) -> Result<OrchestrationTrustRoot, CliError> {
    if first_byte(bytes) == Some(b'{') {
        return OrchestrationTrustRoot::parse_json(bytes)
            .map_err(|error| CliError::Core(error.to_string()));
    }
    let root: OrchestrationTrustRoot = toml::from_str(
        std::str::from_utf8(bytes)
            .map_err(|_| AdapterError::Input("trust root is not UTF-8".to_owned()))?,
    )
    .map_err(|error| AdapterError::Input(format!("invalid trust root TOML: {error}")))?;
    root.validate()
        .map_err(|error| CliError::Core(error.to_string()))?;
    Ok(root)
}

fn validate_external_root(target: &PinnedTarget, root: &Path) -> Result<(), CliError> {
    let absolute = fs::canonicalize(root)
        .map_err(|error| AdapterError::Input(format!("cannot resolve trust root: {error}")))?;
    if absolute.starts_with(target.requested_path()) {
        return Err(AdapterError::Safety(
            "orchestration trust root must be outside the consumer target".to_owned(),
        )
        .into());
    }
    let metadata = fs::metadata(&absolute)
        .map_err(|error| AdapterError::Input(format!("cannot inspect trust root: {error}")))?;
    if !metadata.is_file() || !metadata.permissions().readonly() {
        return Err(AdapterError::Safety(
            "orchestration trust root must be a read-only regular file".to_owned(),
        )
        .into());
    }
    Ok(())
}

fn render_event(event: &OrchestrationEvent) -> Result<(PathBuf, Vec<u8>), CliError> {
    let relative = event_path(&event.run_id, event.sequence)?;
    let frontmatter = toml::to_string(event)
        .map_err(|error| CliError::Core(format!("cannot serialize event: {error}")))?;
    Ok((
        relative,
        format!(
            "+++\n{frontmatter}+++\n\n# Orchestration event {}\n\nImmutable event revision.\n",
            event.sequence
        )
        .into_bytes(),
    ))
}

fn event_path(run_id: &str, sequence: u64) -> Result<PathBuf, CliError> {
    Ok(run_path(
        run_id,
        &format!("events/revisions/{sequence:020}.md"),
    )?)
}

fn target_digest(target: &PinnedTarget) -> String {
    sha256_digest(target.requested_path().to_string_lossy().as_bytes())
}

fn authority_action(command: &str) -> AuthorityAction {
    match command {
        "plan" => AuthorityAction::Plan,
        "dispatch" => AuthorityAction::Dispatch,
        "receipt" => AuthorityAction::Receipt,
        "cancel" => AuthorityAction::Cancel,
        "recover" => AuthorityAction::Recover,
        _ => unreachable!("validated command"),
    }
}

fn action_name(command: &str) -> &'static str {
    match command {
        "status" => "OrchestrationStatus",
        "plan" => "OrchestrationPlan",
        "dispatch" => "OrchestrationDispatch",
        "receipt" => "OrchestrationReceipt",
        "cancel" => "OrchestrationCancel",
        "recover" => "OrchestrationRecover",
        "authority" => "OrchestrationAuthority",
        "migrate" => "OrchestrationMigrate",
        _ => "Orchestration",
    }
}

fn validate_head(value: &str) -> Result<(), CliError> {
    if value == "none" {
        return Ok(());
    }
    let Some((sequence, digest)) = value.split_once(':') else {
        return Err(AdapterError::Input("invalid expected head".to_owned()).into());
    };
    sequence
        .parse::<u64>()
        .map_err(|_| AdapterError::Input("invalid expected head sequence".to_owned()))?;
    validate_digest(digest, "expected head digest")
}

fn validate_digest(value: &str, label: &str) -> Result<(), CliError> {
    if value.len() != 71
        || !value.starts_with("sha256:")
        || !value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(AdapterError::Input(format!("invalid {label}")).into());
    }
    Ok(())
}

fn first_byte(bytes: &[u8]) -> Option<u8> {
    bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn option<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(option, value)| (*option == name).then_some(*value))
}

fn success(
    action: &'static str,
    code: &'static str,
    message: impl Into<String>,
    changed_paths: Vec<String>,
    evidence: Vec<Evidence>,
    data: serde_json::Value,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.into(),
        changed_paths,
        evidence,
        next_action: None,
        data: Some(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::orchestration::DispatchState;

    #[test]
    fn head_binding_is_closed() {
        assert!(validate_head("none").is_ok());
        assert!(validate_head(&format!("1:sha256:{}", "a".repeat(64))).is_ok());
        assert!(validate_head("1:sha256:ABC").is_err());
    }

    #[test]
    fn command_event_kinds_are_closed() {
        let event = OrchestrationEvent {
            schema_version: 1,
            event_id: "event-1".to_owned(),
            run_id: "run-1".to_owned(),
            action_id: "action-1".to_owned(),
            sequence: 1,
            predecessor_digest: None,
            control_epoch: 0,
            kind: EventKind::Reserve,
            from_state: None,
            to_state: DispatchState::Reserved,
            authority_id: "authority-1".to_owned(),
            request_digest: format!("sha256:{}", "1".repeat(64)),
            payload_digest: format!("sha256:{}", "2".repeat(64)),
            occurred_at: "2026-08-12T00:00:00Z".to_owned(),
        };
        assert!(validate_command_event("plan", &event).is_ok());
        assert!(validate_command_event("cancel", &event).is_err());
    }
}
