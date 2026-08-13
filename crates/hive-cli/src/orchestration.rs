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
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_REPLAY_EVENTS: u64 = 4096;
const SNAPSHOT_INTERVAL: u64 = 64;
const SEGMENT_INTERVAL: u64 = 256;
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
    grant: Option<PathBuf>,
    subject_authority: Option<String>,
    emergency_reason_digest: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MigrationMode {
    DryRun,
    Apply,
    Recover,
}

struct MigrationArguments {
    target: PathBuf,
    from_run: String,
    mode: MigrationMode,
    expected_head: Option<String>,
    control_epoch: Option<u64>,
    authority: Option<PathBuf>,
    trust_root: Option<PathBuf>,
    request_digest: Option<String>,
    now: Option<String>,
}

struct ValidatedPayload {
    event: OrchestrationEvent,
    receipt: Option<(HostReceipt, Vec<u8>)>,
    side_artifact: Option<SideArtifact>,
}

struct SideArtifact {
    root: PathBuf,
    relative: PathBuf,
    bytes: Vec<u8>,
    label: &'static str,
}

#[derive(Serialize)]
struct AuthorityRevocation<'a> {
    schema_version: u32,
    authority_id: &'a str,
    authority_digest: &'a str,
    revocation_event_id: &'a str,
    request_digest: &'a str,
    occurred_at: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EmergencyCancel {
    schema_version: u32,
    run_id: String,
    expected_head: String,
    corrupt_head_digest: String,
    control_epoch: u64,
    request_digest: String,
    authority_digest: String,
    reason_digest: String,
    event_digest: String,
    promoted_head: Option<String>,
}

#[derive(Serialize)]
struct ReplaySnapshot<'a> {
    schema_version: u32,
    sequence: u64,
    event_head: String,
    control_epoch: u64,
    state: hive_core::orchestration::DispatchState,
    source_event_digest: &'a str,
}

#[derive(Serialize)]
struct EventSegment<'a> {
    schema_version: u32,
    start_sequence: u64,
    end_sequence: u64,
    start_event_digest: &'a str,
    end_event_digest: &'a str,
}

#[derive(Debug, Serialize)]
struct MigrationEntry {
    locator: String,
    digest: String,
    bytes: usize,
    disposition: &'static str,
}

#[derive(Debug, Serialize)]
struct MigrationInventory {
    schema_version: u32,
    source_run_id: String,
    target_run_id: String,
    entries: Vec<MigrationEntry>,
    source_digest: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MigrationRecovery {
    schema_version: u32,
    source_run_id: String,
    source_digest: String,
    target_run_id: String,
    request_digest: String,
    authority_digest: String,
    event_digest: String,
    state: String,
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
            let parsed = parse_mutation(&arguments[1..], command)?;
            mutate(command, &parsed)
        }
        Some("authority") => {
            let operation = arguments
                .get(1)
                .map(String::as_str)
                .ok_or_else(|| AdapterError::Input("missing authority operation".to_owned()))?;
            if !matches!(operation, "issue" | "revoke") {
                return Err(AdapterError::Input(format!(
                    "unknown authority operation: {operation}"
                ))
                .into());
            }
            let command = if operation == "issue" {
                "authority-issue"
            } else {
                "authority-revoke"
            };
            let parsed = parse_authority_mutation(&arguments[2..], operation)?;
            mutate(command, &parsed)
        }
        Some("migrate") => migrate(&arguments[1..]),
        Some(other) => {
            Err(AdapterError::Input(format!("unknown orchestration action: {other}")).into())
        }
        None => Err(AdapterError::Input("missing orchestration action".to_owned()).into()),
    }
}

fn migrate(arguments: &[String]) -> Result<ActionResult, CliError> {
    let parsed = parse_migration(arguments)?;
    let target = PinnedTarget::open(&parsed.target)?;
    let inventory = migration_inventory(&target, &parsed.from_run)?;
    if parsed.mode == MigrationMode::DryRun {
        return Ok(success(
            "OrchestrationMigrate",
            "hive.orchestration-migration-preview",
            "legacy run inventory verified without mutation",
            Vec::new(),
            inventory_evidence(&inventory),
            json!({
                "mode": "dry-run",
                "source_run_id": inventory.source_run_id,
                "target_run_id": inventory.target_run_id,
                "source_digest": inventory.source_digest,
                "entries": inventory.entries,
                "mutation_count": 0
            }),
        ));
    }
    apply_migration(&target, &parsed, &inventory)
}

fn parse_migration(arguments: &[String]) -> Result<MigrationArguments, CliError> {
    let mut mode = None;
    let mut filtered = Vec::new();
    for argument in arguments {
        let candidate = match argument.as_str() {
            "--dry-run" => Some(MigrationMode::DryRun),
            "--apply" => Some(MigrationMode::Apply),
            "--recover" => Some(MigrationMode::Recover),
            _ => None,
        };
        if let Some(candidate) = candidate {
            if mode.replace(candidate).is_some() {
                return Err(
                    AdapterError::Input("migration requires exactly one mode".to_owned()).into(),
                );
            }
        } else {
            filtered.push(argument.clone());
        }
    }
    let mode = mode.ok_or_else(|| AdapterError::Input("migration mode is required".to_owned()))?;
    let options = parse_options(
        &filtered,
        &[
            "--target",
            "--from-run",
            "--expected-head",
            "--control-epoch",
            "--authority",
            "--trust-root",
            "--request-digest",
            "--now",
        ],
    )?;
    let expected_head = option(&options, "--expected-head").map(str::to_owned);
    let control_epoch = option(&options, "--control-epoch")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| AdapterError::Input("control epoch must be unsigned".to_owned()))
        })
        .transpose()?;
    let authority = option(&options, "--authority").map(PathBuf::from);
    let trust_root = option(&options, "--trust-root").map(PathBuf::from);
    let request_digest = option(&options, "--request-digest").map(str::to_owned);
    let now = option(&options, "--now").map(str::to_owned);
    let mutation_fields = [
        expected_head.is_some(),
        control_epoch.is_some(),
        authority.is_some(),
        trust_root.is_some(),
        request_digest.is_some(),
        now.is_some(),
    ];
    if mode == MigrationMode::DryRun && mutation_fields.iter().any(|present| *present) {
        return Err(AdapterError::Input(
            "migration dry-run does not accept mutation authority fields".to_owned(),
        )
        .into());
    }
    if mode != MigrationMode::DryRun && mutation_fields.iter().any(|present| !present) {
        return Err(AdapterError::Input(
            "migration apply and recover require every authority binding field".to_owned(),
        )
        .into());
    }
    if let Some(digest) = request_digest.as_deref() {
        validate_digest(digest, "request digest")?;
    }
    Ok(MigrationArguments {
        target: PathBuf::from(required(&options, "--target")?),
        from_run: required(&options, "--from-run")?.to_owned(),
        mode,
        expected_head,
        control_epoch,
        authority,
        trust_root,
        request_digest,
        now,
    })
}

fn migration_inventory(
    target: &PinnedTarget,
    from_run: &str,
) -> Result<MigrationInventory, CliError> {
    let mut entries = Vec::new();
    for name in [
        "PLAN.md",
        "STATUS.md",
        "HANDOFF.md",
        "LOOP-CURRENT.md",
        "CONTROL.md",
        "AUTHORITY.md",
    ] {
        let relative = run_path(from_run, name)?;
        if let Some(bytes) = target.read_optional(&relative, MAX_INPUT_BYTES)? {
            entries.push(MigrationEntry {
                locator: portable_relative_path(&relative),
                digest: sha256_digest(&bytes),
                bytes: bytes.len(),
                disposition: "preserved-read-only-provenance",
            });
        }
    }
    if entries.is_empty() {
        return Err(AdapterError::Input(
            "legacy run has no supported canonical artifacts".to_owned(),
        )
        .into());
    }
    let digest_payload = serde_json_canonicalizer::to_vec(&entries)
        .map_err(|error| CliError::Core(format!("cannot digest migration inventory: {error}")))?;
    let source_digest = sha256_digest(&digest_payload);
    let target_run_id = format!("native-migration-{}", &source_digest[7..23]);
    Ok(MigrationInventory {
        schema_version: 1,
        source_run_id: from_run.to_owned(),
        target_run_id,
        entries,
        source_digest,
    })
}

fn apply_migration(
    target: &PinnedTarget,
    arguments: &MigrationArguments,
    inventory: &MigrationInventory,
) -> Result<ActionResult, CliError> {
    let expected_head = arguments
        .expected_head
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration expected head".to_owned()))?;
    let epoch = arguments
        .control_epoch
        .ok_or_else(|| AdapterError::Input("missing migration control epoch".to_owned()))?;
    if expected_head != "none" || epoch != 0 {
        return Err(AdapterError::Input(
            "new native migration requires expected head none at epoch 0".to_owned(),
        )
        .into());
    }
    let trust_root_path = arguments
        .trust_root
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration trust root".to_owned()))?;
    validate_external_root(target, trust_root_path)?;
    let authority_path = arguments
        .authority
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration authority".to_owned()))?;
    let authority_bytes = read_explicit_file(authority_path, MAX_INPUT_BYTES)?;
    let authority = parse_authority(&authority_bytes)?;
    let root = parse_root(&read_explicit_file(trust_root_path, MAX_INPUT_BYTES)?)?;
    let request_digest = arguments
        .request_digest
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration request digest".to_owned()))?;
    let now = arguments
        .now
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration time".to_owned()))?;
    let target_binding = target_digest(target);
    authority
        .verify(
            &root,
            AuthorityExpectation {
                action: AuthorityAction::Migrate,
                target_digest: &target_binding,
                head: expected_head,
                control_epoch: epoch,
                request_digest,
                now,
            },
        )
        .map_err(|error| CliError::Core(error.to_string()))?;
    publish_migration(target, arguments, inventory, &authority, &authority_bytes)
}

fn publish_migration(
    target: &PinnedTarget,
    arguments: &MigrationArguments,
    inventory: &MigrationInventory,
    authority: &ActionAuthority,
    authority_bytes: &[u8],
) -> Result<ActionResult, CliError> {
    let request_digest = arguments
        .request_digest
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration request digest".to_owned()))?;
    let now = arguments
        .now
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing migration time".to_owned()))?;
    let event = OrchestrationEvent {
        schema_version: 1,
        event_id: format!("migration-{}", &inventory.source_digest[7..23]),
        run_id: inventory.target_run_id.clone(),
        action_id: format!("migration-{}", &inventory.source_digest[7..23]),
        sequence: 1,
        predecessor_digest: None,
        control_epoch: 0,
        kind: EventKind::Migrate,
        from_state: None,
        to_state: hive_core::orchestration::DispatchState::Reserved,
        authority_id: authority.authority_id.clone(),
        request_digest: request_digest.to_owned(),
        payload_digest: inventory.source_digest.clone(),
        occurred_at: now.to_owned(),
    };
    let mut reducer = ReducerState::default();
    let event_digest = reducer
        .apply_event(&event)
        .map_err(|error| CliError::Core(error.to_string()))?;
    let recovery = MigrationRecovery {
        schema_version: 1,
        source_run_id: inventory.source_run_id.clone(),
        source_digest: inventory.source_digest.clone(),
        target_run_id: inventory.target_run_id.clone(),
        request_digest: request_digest.to_owned(),
        authority_digest: sha256_digest(authority_bytes),
        event_digest: event_digest.clone(),
        state: "staged".to_owned(),
    };
    let recovery_relative = run_path(&inventory.target_run_id, "RECOVERY.toml")?;
    let recovery_bytes = toml::to_string(&recovery)
        .map_err(|error| CliError::Core(format!("cannot serialize recovery: {error}")))?
        .into_bytes();
    let recovery_snapshot = target.snapshot(&recovery_relative)?;
    publish_or_validate_recovery(
        target,
        &inventory.target_run_id,
        &recovery_relative,
        &recovery_snapshot,
        &recovery,
        &recovery_bytes,
    )?;
    let source_before = migration_inventory(target, &inventory.source_run_id)?;
    let changed = publish_migration_generation(target, inventory, &event, &event_digest)?;
    let source_after = migration_inventory(target, &inventory.source_run_id)?;
    if source_before.source_digest != source_after.source_digest {
        return Err(AdapterError::Conflict(
            "legacy source bytes changed during migration and were not modified by Hive".to_owned(),
        )
        .into());
    }
    let committed_recovery = MigrationRecovery {
        state: "committed".to_owned(),
        ..recovery
    };
    let committed_bytes = toml::to_string(&committed_recovery)
        .map_err(|error| CliError::Core(format!("cannot serialize recovery: {error}")))?
        .into_bytes();
    let current_recovery = target.snapshot(&recovery_relative)?;
    target.publish(&recovery_relative, &current_recovery, &committed_bytes)?;
    target.restore(&recovery_relative, &FileSnapshot::Missing, &committed_bytes)?;
    Ok(success(
        "OrchestrationMigrate",
        "hive.orchestration-migration-committed",
        "legacy bytes were preserved and a separate native run generation was committed",
        changed,
        inventory_evidence(inventory),
        json!({
            "mode": if arguments.mode == MigrationMode::Recover { "recover" } else { "apply" },
            "source_run_id": inventory.source_run_id,
            "target_run_id": inventory.target_run_id,
            "source_digest": inventory.source_digest,
            "source_bytes_modified": false,
            "recovery_marker_cleaned": true
        }),
    ))
}

fn publish_or_validate_recovery(
    target: &PinnedTarget,
    run_id: &str,
    relative: &Path,
    snapshot: &FileSnapshot,
    expected: &MigrationRecovery,
    bytes: &[u8],
) -> Result<(), CliError> {
    target.ensure_owned_parent(relative, &run_path(run_id, "")?)?;
    match snapshot {
        FileSnapshot::Missing => {
            target.publish(relative, &FileSnapshot::Missing, bytes)?;
        }
        FileSnapshot::File(existing) => {
            let record: MigrationRecovery =
                toml::from_str(std::str::from_utf8(existing).map_err(|_| {
                    AdapterError::Verification("migration recovery is not UTF-8".to_owned())
                })?)
                .map_err(|error| {
                    AdapterError::Verification(format!("invalid migration recovery: {error}"))
                })?;
            if record.schema_version != expected.schema_version
                || record.source_run_id != expected.source_run_id
                || record.source_digest != expected.source_digest
                || record.target_run_id != expected.target_run_id
                || record.request_digest != expected.request_digest
                || record.authority_digest != expected.authority_digest
                || record.event_digest != expected.event_digest
                || !matches!(record.state.as_str(), "staged" | "committed")
            {
                return Err(AdapterError::Conflict(
                    "migration recovery belongs to a different generation".to_owned(),
                )
                .into());
            }
        }
    }
    Ok(())
}

fn publish_migration_generation(
    target: &PinnedTarget,
    inventory: &MigrationInventory,
    event: &OrchestrationEvent,
    event_digest: &str,
) -> Result<Vec<String>, CliError> {
    let migration_relative = run_path(&inventory.target_run_id, "MIGRATION.md")?;
    let migration_bytes = markdown_toml("Legacy migration provenance", inventory)?;
    publish_immutable(
        target,
        &run_path(&inventory.target_run_id, "")?,
        &migration_relative,
        &migration_bytes,
        "migration provenance",
    )?;
    let (event_relative, event_bytes) = render_event(event)?;
    publish_immutable(
        target,
        &run_path(&inventory.target_run_id, "events")?,
        &event_relative,
        &event_bytes,
        "migration event",
    )?;
    let head = EventHead {
        schema_version: 1,
        generation: 1,
        sequence: 1,
        event_digest: event_digest.to_owned(),
        control_epoch: 0,
    };
    let head_relative = run_path(&inventory.target_run_id, "EVENT-CURRENT.toml")?;
    let head_bytes = toml::to_string(&head)
        .map_err(|error| CliError::Core(format!("cannot serialize migration head: {error}")))?
        .into_bytes();
    match target.snapshot(&head_relative)? {
        FileSnapshot::Missing => {
            target.publish(&head_relative, &FileSnapshot::Missing, &head_bytes)?;
        }
        FileSnapshot::File(existing) if existing == head_bytes => {}
        FileSnapshot::File(_) => {
            return Err(AdapterError::Conflict(
                "native migration target already has a different event head".to_owned(),
            )
            .into());
        }
    }
    Ok(vec![
        portable_relative_path(&migration_relative),
        portable_relative_path(&event_relative),
        portable_relative_path(&head_relative),
    ])
}

fn inventory_evidence(inventory: &MigrationInventory) -> Vec<Evidence> {
    vec![Evidence {
        kind: "legacy-run-inventory",
        locator: format!(".hive/runs/{}", inventory.source_run_id),
        digest: inventory.source_digest.clone(),
    }]
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

fn parse_mutation(arguments: &[String], command: &str) -> Result<MutationArguments, CliError> {
    let needs_receipt = command == "receipt";
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
            "--emergency",
        ],
    )?;
    let receipt = option(&options, "--receipt").map(PathBuf::from);
    let emergency_reason_digest = option(&options, "--emergency").map(str::to_owned);
    if needs_receipt != receipt.is_some() {
        return Err(AdapterError::Input(if needs_receipt {
            "receipt mutation requires --receipt".to_owned()
        } else {
            "--receipt is only valid for the receipt action".to_owned()
        })
        .into());
    }
    if let Some(digest) = emergency_reason_digest.as_deref() {
        if command != "cancel" {
            return Err(AdapterError::Input(
                "--emergency requires a sha256 reason digest on cancel".to_owned(),
            )
            .into());
        }
        validate_digest(digest, "emergency reason digest")?;
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
        grant: None,
        subject_authority: None,
        emergency_reason_digest,
    })
}

fn parse_authority_mutation(
    arguments: &[String],
    operation: &str,
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
            "--grant",
            "--subject-authority",
        ],
    )?;
    let expected_head = required(&options, "--expected-head")?.to_owned();
    validate_head(&expected_head)?;
    let request_digest = required(&options, "--request-digest")?.to_owned();
    validate_digest(&request_digest, "request digest")?;
    let control_epoch = required(&options, "--control-epoch")?
        .parse::<u64>()
        .map_err(|_| AdapterError::Input("control epoch must be unsigned".to_owned()))?;
    let grant = option(&options, "--grant").map(PathBuf::from);
    let subject_authority = option(&options, "--subject-authority").map(str::to_owned);
    if (operation == "issue") != grant.is_some()
        || (operation == "revoke") != subject_authority.is_some()
    {
        return Err(AdapterError::Input(
            "authority issue requires only --grant; revoke requires only --subject-authority"
                .to_owned(),
        )
        .into());
    }
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
        receipt: None,
        grant,
        subject_authority,
        emergency_reason_digest: None,
    })
}

fn mutate(command: &str, arguments: &MutationArguments) -> Result<ActionResult, CliError> {
    let target = PinnedTarget::open(&arguments.target)?;
    validate_external_root(&target, &arguments.trust_root)?;
    if arguments.emergency_reason_digest.is_some() {
        return emergency_cancel(arguments, &target);
    }
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
    let side_change = payload
        .side_artifact
        .as_ref()
        .map(|artifact| {
            publish_immutable(
                &target,
                &artifact.root,
                &artifact.relative,
                &artifact.bytes,
                artifact.label,
            )
            .map(|()| artifact.relative.clone())
        })
        .transpose()?;
    let artifact_changes = receipt_change.into_iter().chain(side_change).collect();
    commit_event(
        command,
        arguments,
        &target,
        loaded,
        &payload.event,
        event_digest,
        artifact_changes,
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
    if !(matches!(command, "authority-issue" | "authority-revoke")
        || command == "plan" && event.kind == EventKind::Reserve)
    {
        ensure_authority_usable(target, &event.run_id, &authority, &loaded.events)?;
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
    let side_artifact = authority_side_artifact(command, arguments, target, &event, &root, loaded)?;
    Ok(ValidatedPayload {
        event,
        receipt: receipt_payload,
        side_artifact,
    })
}

fn commit_event(
    command: &str,
    arguments: &MutationArguments,
    target: &PinnedTarget,
    mut loaded: LoadedChain,
    event: &OrchestrationEvent,
    event_digest: String,
    artifact_changes: Vec<PathBuf>,
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
    for relative in artifact_changes {
        changed.push(portable_relative_path(&relative));
    }
    if let Some(relative) = materialize_control(target, event, &head)? {
        changed.push(portable_relative_path(&relative));
    }
    if let Some(relative) = materialize_authority(target, event, &head)? {
        changed.push(portable_relative_path(&relative));
    }
    for relative in materialize_replay_artifacts(target, event, &head)? {
        changed.push(portable_relative_path(&relative));
    }
    if event.kind == EventKind::RebuildProjection {
        for relative in rebuild_projections(target, &event.run_id)? {
            if !changed
                .iter()
                .any(|existing| existing == &portable_relative_path(&relative))
            {
                changed.push(portable_relative_path(&relative));
            }
        }
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
        "recover" => matches!(
            event.kind,
            EventKind::Recover | EventKind::RebuildProjection
        ),
        "authority-issue" => event.kind == EventKind::IssueAuthority,
        "authority-revoke" => event.kind == EventKind::RevokeAuthority,
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

fn authority_side_artifact(
    command: &str,
    arguments: &MutationArguments,
    target: &PinnedTarget,
    event: &OrchestrationEvent,
    root: &OrchestrationTrustRoot,
    loaded: &LoadedChain,
) -> Result<Option<SideArtifact>, CliError> {
    match command {
        "authority-issue" => issue_artifact(arguments, target, event, root, loaded),
        "authority-revoke" => revoke_artifact(arguments, target, event, loaded),
        _ => Ok(None),
    }
}

fn issue_artifact(
    arguments: &MutationArguments,
    target: &PinnedTarget,
    event: &OrchestrationEvent,
    root: &OrchestrationTrustRoot,
    loaded: &LoadedChain,
) -> Result<Option<SideArtifact>, CliError> {
    let grant_path = arguments
        .grant
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing authority grant".to_owned()))?;
    let grant = parse_authority(&read_explicit_file(grant_path, MAX_INPUT_BYTES)?)?;
    let target_binding = target_digest(target);
    grant
        .verify(
            root,
            AuthorityExpectation {
                action: grant.action,
                target_digest: &target_binding,
                head: &grant.expected_head,
                control_epoch: grant.control_epoch,
                request_digest: &grant.request_digest,
                now: &arguments.now,
            },
        )
        .map_err(|error| CliError::Core(error.to_string()))?;
    if loaded.events.iter().any(|prior| {
        prior.kind == EventKind::IssueAuthority && prior.action_id == grant.authority_id
    }) {
        return Err(AdapterError::Conflict(
            "authority id was already issued in the committed event chain".to_owned(),
        )
        .into());
    }
    let bytes = toml::to_string(&grant)
        .map_err(|error| CliError::Core(format!("cannot serialize authority: {error}")))?
        .into_bytes();
    if event.action_id != grant.authority_id
        || event.payload_digest != sha256_digest(&bytes)
        || event.from_state != Some(event.to_state)
    {
        return Err(AdapterError::Verification(
            "authority issue event does not bind the grant id, digest, or unchanged state"
                .to_owned(),
        )
        .into());
    }
    Ok(Some(SideArtifact {
        root: run_path(&event.run_id, "events/authorities")?,
        relative: authority_path(&event.run_id, &grant.authority_id)?,
        bytes,
        label: "authority grant",
    }))
}

fn revoke_artifact(
    arguments: &MutationArguments,
    target: &PinnedTarget,
    event: &OrchestrationEvent,
    loaded: &LoadedChain,
) -> Result<Option<SideArtifact>, CliError> {
    let subject = arguments
        .subject_authority
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing authority revocation subject".to_owned()))?;
    let subject_relative = authority_path(&event.run_id, subject)?;
    let subject_bytes = target.read_required(&subject_relative, MAX_INPUT_BYTES)?;
    let subject_authority = parse_authority(&subject_bytes)?;
    ensure_authority_usable(target, &event.run_id, &subject_authority, &loaded.events)?;
    let subject_digest = sha256_digest(&subject_bytes);
    if event.action_id != subject
        || event.payload_digest != subject_digest
        || event.from_state != Some(event.to_state)
    {
        return Err(AdapterError::Verification(
            "authority revoke event does not bind the subject, digest, or unchanged state"
                .to_owned(),
        )
        .into());
    }
    let record = AuthorityRevocation {
        schema_version: 1,
        authority_id: subject,
        authority_digest: &subject_digest,
        revocation_event_id: &event.event_id,
        request_digest: &event.request_digest,
        occurred_at: &event.occurred_at,
    };
    let bytes = toml::to_string(&record)
        .map_err(|error| CliError::Core(format!("cannot serialize revocation: {error}")))?
        .into_bytes();
    Ok(Some(SideArtifact {
        root: run_path(&event.run_id, "events/authorities")?,
        relative: revocation_path(&event.run_id, subject)?,
        bytes,
        label: "authority revocation",
    }))
}

fn ensure_authority_usable(
    target: &PinnedTarget,
    run_id: &str,
    authority: &ActionAuthority,
    events: &[OrchestrationEvent],
) -> Result<(), CliError> {
    let stored_bytes = target.read_required(
        &authority_path(run_id, &authority.authority_id)?,
        MAX_INPUT_BYTES,
    )?;
    let stored = parse_authority(&stored_bytes)?;
    let digest = sha256_digest(&stored_bytes);
    let issued = events.iter().any(|event| {
        event.kind == EventKind::IssueAuthority
            && event.action_id == authority.authority_id
            && event.payload_digest == digest
    });
    let revoked = events.iter().any(|event| {
        event.kind == EventKind::RevokeAuthority && event.action_id == authority.authority_id
    });
    if stored != *authority || !issued || revoked {
        return Err(AdapterError::Verification(
            "authority is not an exact active grant in the committed event chain".to_owned(),
        )
        .into());
    }
    Ok(())
}

fn authority_path(run_id: &str, authority_id: &str) -> Result<PathBuf, CliError> {
    Ok(run_path(
        run_id,
        &format!("events/authorities/{authority_id}.toml"),
    )?)
}

fn revocation_path(run_id: &str, authority_id: &str) -> Result<PathBuf, CliError> {
    Ok(run_path(
        run_id,
        &format!("events/authorities/{authority_id}.revoked.toml"),
    )?)
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

fn materialize_authority(
    target: &PinnedTarget,
    event: &OrchestrationEvent,
    head: &EventHead,
) -> Result<Option<PathBuf>, CliError> {
    let operation = match event.kind {
        EventKind::IssueAuthority => "issued",
        EventKind::RevokeAuthority => "revoked",
        _ => return Ok(None),
    };
    let relative = run_path(&event.run_id, "AUTHORITY.md")?;
    let bytes = format!(
        "+++\nschema_version = 1\nevent_head = \"{}\"\ncontrol_epoch = {}\nlast_operation = \"{}\"\nsubject_authority_id = \"{}\"\npayload_digest = \"{}\"\n+++\n\n# Orchestration authority\n\nThis projection is rebuilt from the committed immutable event chain.\n",
        head.binding(),
        head.control_epoch,
        operation,
        event.action_id,
        event.payload_digest
    )
    .into_bytes();
    let expected = target.snapshot(&relative)?;
    target.publish(&relative, &expected, &bytes)?;
    Ok(Some(relative))
}

fn materialize_replay_artifacts(
    target: &PinnedTarget,
    event: &OrchestrationEvent,
    head: &EventHead,
) -> Result<Vec<PathBuf>, CliError> {
    let force = event.kind == EventKind::RebuildProjection;
    let mut changed = Vec::new();
    if force || head.sequence.is_multiple_of(SNAPSHOT_INTERVAL) {
        let snapshot = ReplaySnapshot {
            schema_version: 1,
            sequence: head.sequence,
            event_head: head.binding(),
            control_epoch: head.control_epoch,
            state: event.to_state,
            source_event_digest: &head.event_digest,
        };
        let relative = run_path(
            &event.run_id,
            &format!("events/snapshots/{:020}.md", head.sequence),
        )?;
        let bytes = markdown_toml("Orchestration replay snapshot", &snapshot)?;
        publish_immutable(
            target,
            &run_path(&event.run_id, "events/snapshots")?,
            &relative,
            &bytes,
            "replay snapshot",
        )?;
        changed.push(relative);
    }
    if head.sequence.is_multiple_of(SEGMENT_INTERVAL) {
        let start_sequence = head.sequence - SEGMENT_INTERVAL + 1;
        let start_event = parse_event_markdown(
            &target.read_required(&event_path(&event.run_id, start_sequence)?, MAX_INPUT_BYTES)?,
        )?;
        let start_digest = start_event
            .digest()
            .map_err(|error| CliError::Core(error.to_string()))?;
        let segment = EventSegment {
            schema_version: 1,
            start_sequence,
            end_sequence: head.sequence,
            start_event_digest: &start_digest,
            end_event_digest: &head.event_digest,
        };
        let relative = run_path(
            &event.run_id,
            &format!(
                "events/segments/{start_sequence:020}-{:020}.md",
                head.sequence
            ),
        )?;
        let bytes = markdown_toml("Orchestration event segment", &segment)?;
        publish_immutable(
            target,
            &run_path(&event.run_id, "events/segments")?,
            &relative,
            &bytes,
            "event segment",
        )?;
        changed.push(relative);
    }
    Ok(changed)
}

fn rebuild_projections(target: &PinnedTarget, run_id: &str) -> Result<Vec<PathBuf>, CliError> {
    let loaded = load_chain(target, run_id)?;
    let head = loaded.head.as_ref().ok_or_else(|| {
        AdapterError::Verification("cannot rebuild an empty event chain".to_owned())
    })?;
    let mut changed = Vec::new();
    if let Some(cancel) = loaded
        .events
        .iter()
        .rev()
        .find(|event| event.kind == EventKind::RequestCancel)
    {
        if let Some(relative) = materialize_control(target, cancel, head)? {
            changed.push(relative);
        }
    }
    if let Some(authority) = loaded.events.iter().rev().find(|event| {
        matches!(
            event.kind,
            EventKind::IssueAuthority | EventKind::RevokeAuthority
        )
    }) {
        if let Some(relative) = materialize_authority(target, authority, head)? {
            changed.push(relative);
        }
    }
    Ok(changed)
}

fn markdown_toml<T: Serialize>(title: &str, value: &T) -> Result<Vec<u8>, CliError> {
    let frontmatter = toml::to_string(value)
        .map_err(|error| CliError::Core(format!("cannot serialize projection: {error}")))?;
    Ok(format!("+++\n{frontmatter}+++\n\n# {title}\n").into_bytes())
}

fn emergency_cancel(
    arguments: &MutationArguments,
    target: &PinnedTarget,
) -> Result<ActionResult, CliError> {
    let reason_digest = arguments
        .emergency_reason_digest
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing emergency reason digest".to_owned()))?;
    let predecessor = parse_expected_event_head(&arguments.expected_head, arguments.control_epoch)?;
    let (events, state) = replay_to_head(target, &arguments.run_id, &predecessor)?;
    let mut loaded = LoadedChain {
        events,
        state,
        head: Some(predecessor),
        head_bytes: None,
    };
    let payload = validate_mutation_payload(
        "cancel",
        arguments,
        target,
        &mut loaded,
        &arguments.expected_head,
    )?;
    if payload.receipt.is_some() || payload.side_artifact.is_some() {
        return Err(AdapterError::Verification(
            "emergency cancel cannot carry a receipt or authority ledger artifact".to_owned(),
        )
        .into());
    }
    let event_digest = loaded
        .state
        .apply_event(&payload.event)
        .map_err(|error| CliError::Core(error.to_string()))?;
    let desired_head = EventHead {
        schema_version: 1,
        generation: loaded
            .head
            .as_ref()
            .map_or(1, |head| head.generation.saturating_add(1)),
        sequence: payload.event.sequence,
        event_digest: event_digest.clone(),
        control_epoch: payload.event.control_epoch,
    };
    let head_relative = run_path(&arguments.run_id, "EVENT-CURRENT.toml")?;
    let current_head = target.snapshot(&head_relative)?;
    reject_unnecessary_emergency(target, &arguments.run_id, &desired_head)?;

    let authority_bytes = read_explicit_file(&arguments.authority, MAX_INPUT_BYTES)?;
    let marker_relative = run_path(&arguments.run_id, "EMERGENCY-CANCEL.toml")?;
    let marker_snapshot = target.snapshot(&marker_relative)?;
    let marker = load_or_create_emergency_marker(
        arguments,
        &payload.event,
        reason_digest,
        &authority_bytes,
        &event_digest,
        &current_head,
        &marker_snapshot,
    )?;
    let marker_bytes = toml::to_string(&marker)
        .map_err(|error| CliError::Core(format!("cannot serialize emergency marker: {error}")))?
        .into_bytes();
    if matches!(marker_snapshot, FileSnapshot::Missing) {
        target.publish(&marker_relative, &FileSnapshot::Missing, &marker_bytes)?;
    }

    let (event_relative, event_bytes) = render_event(&payload.event)?;
    publish_immutable(
        target,
        &run_path(&arguments.run_id, "events")?,
        &event_relative,
        &event_bytes,
        "event revision",
    )?;
    let desired_head_bytes = toml::to_string(&desired_head)
        .map_err(|error| CliError::Core(format!("cannot serialize event head: {error}")))?
        .into_bytes();
    if current_head.bytes() != Some(desired_head_bytes.as_slice()) {
        if snapshot_digest(&current_head) != marker.corrupt_head_digest {
            return Err(AdapterError::Conflict(
                "event head changed after emergency intent was recorded".to_owned(),
            )
            .into());
        }
        target.publish(&head_relative, &current_head, &desired_head_bytes)?;
    }
    materialize_control(target, &payload.event, &desired_head)?;
    let promoted = EmergencyCancel {
        promoted_head: Some(desired_head.binding()),
        ..marker
    };
    let promoted_bytes = toml::to_string(&promoted)
        .map_err(|error| CliError::Core(format!("cannot serialize promoted marker: {error}")))?
        .into_bytes();
    let expected_marker = target.snapshot(&marker_relative)?;
    target.publish(&marker_relative, &expected_marker, &promoted_bytes)?;
    target.restore(&marker_relative, &FileSnapshot::Missing, &promoted_bytes)?;
    emergency_result(
        arguments,
        &payload.event,
        &desired_head,
        &event_relative,
        event_digest,
    )
}

fn emergency_result(
    arguments: &MutationArguments,
    event: &OrchestrationEvent,
    head: &EventHead,
    event_relative: &Path,
    event_digest: String,
) -> Result<ActionResult, CliError> {
    Ok(success(
        "OrchestrationCancel",
        "hive.orchestration-emergency-cancel-promoted",
        "authenticated emergency intent was promoted to a normal cancel event",
        vec![
            portable_relative_path(event_relative),
            portable_relative_path(&run_path(&arguments.run_id, "EVENT-CURRENT.toml")?),
            portable_relative_path(&run_path(&arguments.run_id, "CONTROL.md")?),
        ],
        vec![Evidence {
            kind: "orchestration-event",
            locator: portable_relative_path(event_relative),
            digest: event_digest,
        }],
        json!({
            "run_id": arguments.run_id,
            "head": head.binding(),
            "control_epoch": head.control_epoch,
            "state": event.to_state,
            "emergency_intent_cleaned": true,
            "host_process_spawned": false
        }),
    ))
}

fn parse_expected_event_head(value: &str, control_epoch: u64) -> Result<EventHead, CliError> {
    let (sequence, digest) = value.split_once(':').ok_or_else(|| {
        AdapterError::Input("emergency cancel requires a non-empty expected head".to_owned())
    })?;
    let sequence = sequence
        .parse::<u64>()
        .map_err(|_| AdapterError::Input("invalid expected head sequence".to_owned()))?;
    validate_digest(digest, "expected head digest")?;
    if sequence == 0 || sequence > MAX_REPLAY_EVENTS {
        return Err(AdapterError::Input(
            "emergency predecessor is outside replay bounds".to_owned(),
        )
        .into());
    }
    Ok(EventHead {
        schema_version: 1,
        generation: sequence,
        sequence,
        event_digest: digest.to_owned(),
        control_epoch,
    })
}

fn reject_unnecessary_emergency(
    target: &PinnedTarget,
    run_id: &str,
    desired: &EventHead,
) -> Result<(), CliError> {
    if let Ok(current) = load_chain(target, run_id) {
        let binding = current
            .head
            .as_ref()
            .map_or_else(|| "none".to_owned(), EventHead::binding);
        if binding != desired.binding() {
            return Err(AdapterError::Safety(
                "emergency cancel is forbidden while the current event head is valid".to_owned(),
            )
            .into());
        }
    }
    Ok(())
}

fn load_or_create_emergency_marker(
    arguments: &MutationArguments,
    event: &OrchestrationEvent,
    reason_digest: &str,
    authority_bytes: &[u8],
    event_digest: &str,
    current_head: &FileSnapshot,
    snapshot: &FileSnapshot,
) -> Result<EmergencyCancel, CliError> {
    let expected = EmergencyCancel {
        schema_version: 1,
        run_id: arguments.run_id.clone(),
        expected_head: arguments.expected_head.clone(),
        corrupt_head_digest: snapshot_digest(current_head),
        control_epoch: event.control_epoch,
        request_digest: arguments.request_digest.clone(),
        authority_digest: sha256_digest(authority_bytes),
        reason_digest: reason_digest.to_owned(),
        event_digest: event_digest.to_owned(),
        promoted_head: None,
    };
    let FileSnapshot::File(bytes) = snapshot else {
        return Ok(expected);
    };
    let existing: EmergencyCancel = toml::from_str(
        std::str::from_utf8(bytes)
            .map_err(|_| AdapterError::Verification("emergency marker is not UTF-8".to_owned()))?,
    )
    .map_err(|error| {
        AdapterError::Verification(format!("invalid emergency cancel marker: {error}"))
    })?;
    if existing.schema_version != expected.schema_version
        || existing.run_id != expected.run_id
        || existing.expected_head != expected.expected_head
        || existing.control_epoch != expected.control_epoch
        || existing.request_digest != expected.request_digest
        || existing.authority_digest != expected.authority_digest
        || existing.reason_digest != expected.reason_digest
        || existing.event_digest != expected.event_digest
    {
        return Err(AdapterError::Conflict(
            "a different emergency cancel intent already exists".to_owned(),
        )
        .into());
    }
    Ok(existing)
}

fn snapshot_digest(snapshot: &FileSnapshot) -> String {
    match snapshot {
        FileSnapshot::Missing => sha256_digest(b"missing"),
        FileSnapshot::File(bytes) => sha256_digest(bytes),
    }
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
    let (events, state) = replay_to_head(target, run_id, &head)?;
    Ok(LoadedChain {
        events,
        state,
        head: Some(head),
        head_bytes,
    })
}

fn replay_to_head(
    target: &PinnedTarget,
    run_id: &str,
    head: &EventHead,
) -> Result<(Vec<OrchestrationEvent>, ReducerState), CliError> {
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
    Ok((events, state))
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
        "authority-issue" => AuthorityAction::IssueAuthority,
        "authority-revoke" => AuthorityAction::RevokeAuthority,
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
        "authority-issue" => "OrchestrationAuthorityIssue",
        "authority-revoke" => "OrchestrationAuthorityRevoke",
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
    use ed25519_dalek::{Signer, SigningKey};
    use hive_core::orchestration::DispatchState;
    use hive_core::orchestration::{
        AuthorityKeyStatus, OrchestrationTrustRoot, TrustedAuthorityKey,
    };
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

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

    #[test]
    fn migration_preview_inventory_is_read_only() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target_path = temporary.path().join("consumer");
        std::fs::create_dir_all(target_path.join(".hive/runs/legacy-run")).expect("legacy run");
        std::fs::write(
            target_path.join(".hive/runs/legacy-run/PLAN.md"),
            b"# Legacy plan\n",
        )
        .expect("legacy plan");
        let target = PinnedTarget::open(&target_path).expect("target");
        let inventory = migration_inventory(&target, "legacy-run").expect("inventory");
        assert_eq!(inventory.entries.len(), 1);
        assert!(inventory.target_run_id.starts_with("native-migration-"));
        assert!(!target_path
            .join(".hive/runs")
            .join(&inventory.target_run_id)
            .exists());
    }

    #[test]
    fn emergency_marker_rejects_unknown_fields() {
        let marker = EmergencyCancel {
            schema_version: 1,
            run_id: "run-1".to_owned(),
            expected_head: format!("1:sha256:{}", "1".repeat(64)),
            corrupt_head_digest: format!("sha256:{}", "2".repeat(64)),
            control_epoch: 1,
            request_digest: format!("sha256:{}", "3".repeat(64)),
            authority_digest: format!("sha256:{}", "4".repeat(64)),
            reason_digest: format!("sha256:{}", "5".repeat(64)),
            event_digest: format!("sha256:{}", "6".repeat(64)),
            promoted_head: None,
        };
        let mut encoded = toml::to_string(&marker).expect("marker");
        encoded.push_str("unknown = true\n");
        assert!(toml::from_str::<EmergencyCancel>(&encoded).is_err());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn signed_authority_issue_is_committed_and_then_required() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target_path = temporary.path().join("consumer");
        std::fs::create_dir_all(target_path.join(".hive/runs/run-1")).expect("run directory");
        let target = PinnedTarget::open(&target_path).expect("target");
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let all_actions = [
            AuthorityAction::Plan,
            AuthorityAction::Dispatch,
            AuthorityAction::Receipt,
            AuthorityAction::Cancel,
            AuthorityAction::Recover,
            AuthorityAction::Migrate,
            AuthorityAction::IssueAuthority,
            AuthorityAction::RevokeAuthority,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        let mut root = OrchestrationTrustRoot {
            schema_version: 1,
            trust_root_id: "root-1".to_owned(),
            revision: 1,
            issued_at: "2026-08-12T00:00:00Z".to_owned(),
            keys: vec![TrustedAuthorityKey {
                key_id: "key-1".to_owned(),
                principal_id: "principal-1".to_owned(),
                algorithm: "ed25519".to_owned(),
                public_key: format!(
                    "ed25519:{}",
                    hex_bytes(signing_key.verifying_key().as_bytes())
                ),
                status: AuthorityKeyStatus::Active,
                valid_from: "2026-08-12T00:00:00Z".to_owned(),
                valid_until: "2026-08-13T00:00:00Z".to_owned(),
                allowed_actions: all_actions,
            }],
            root_digest: String::new(),
        };
        root.root_digest = root.computed_digest().expect("root digest");
        let root_path = temporary.path().join("root.toml");
        std::fs::write(&root_path, toml::to_string(&root).expect("root")).expect("write root");
        let mut permissions = std::fs::metadata(&root_path)
            .expect("root metadata")
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&root_path, permissions).expect("readonly root");

        let request_digest = format!("sha256:{}", "1".repeat(64));
        let reserve_authority = signed_authority(
            &signing_key,
            "reserve-authority",
            AuthorityAction::Plan,
            target_digest(&target),
            "none",
            0,
            &request_digest,
        );
        let reserve_authority_path = temporary.path().join("reserve.toml");
        std::fs::write(
            &reserve_authority_path,
            toml::to_string(&reserve_authority).expect("reserve authority"),
        )
        .expect("write authority");
        let reserve = OrchestrationEvent {
            schema_version: 1,
            event_id: "event-reserve".to_owned(),
            run_id: "run-1".to_owned(),
            action_id: "action-1".to_owned(),
            sequence: 1,
            predecessor_digest: None,
            control_epoch: 0,
            kind: EventKind::Reserve,
            from_state: None,
            to_state: DispatchState::Reserved,
            authority_id: reserve_authority.authority_id.clone(),
            request_digest: request_digest.clone(),
            payload_digest: format!("sha256:{}", "2".repeat(64)),
            occurred_at: "2026-08-12T00:00:01Z".to_owned(),
        };
        let reserve_path = temporary.path().join("reserve-event.toml");
        std::fs::write(
            &reserve_path,
            toml::to_string(&reserve).expect("reserve event"),
        )
        .expect("write reserve");
        let reserve_arguments = MutationArguments {
            target: target_path.clone(),
            run_id: "run-1".to_owned(),
            event: reserve_path,
            expected_head: "none".to_owned(),
            control_epoch: 0,
            authority: reserve_authority_path,
            trust_root: root_path.clone(),
            request_digest: request_digest.clone(),
            now: "2026-08-12T00:00:02Z".to_owned(),
            receipt: None,
            grant: None,
            subject_authority: None,
            emergency_reason_digest: None,
        };
        assert_eq!(
            mutate("plan", &reserve_arguments).expect("reserve").status,
            "success"
        );

        let reserve_head = reserve.digest().expect("reserve digest");
        let grant_request = format!("sha256:{}", "3".repeat(64));
        let grant = signed_authority(
            &signing_key,
            "grant-1",
            AuthorityAction::Dispatch,
            target_digest(&target),
            &format!("1:{reserve_head}"),
            0,
            &grant_request,
        );
        let grant_bytes = toml::to_string(&grant).expect("grant").into_bytes();
        let grant_path = temporary.path().join("grant.toml");
        std::fs::write(&grant_path, &grant_bytes).expect("write grant");
        let issue_request = format!("sha256:{}", "4".repeat(64));
        let issue_authority = signed_authority(
            &signing_key,
            "issuer-1",
            AuthorityAction::IssueAuthority,
            target_digest(&target),
            &format!("1:{reserve_head}"),
            0,
            &issue_request,
        );
        let issue_authority_path = temporary.path().join("issuer.toml");
        std::fs::write(
            &issue_authority_path,
            toml::to_string(&issue_authority).expect("issuer"),
        )
        .expect("write issuer");
        let issue = OrchestrationEvent {
            schema_version: 1,
            event_id: "event-issue".to_owned(),
            run_id: "run-1".to_owned(),
            action_id: grant.authority_id.clone(),
            sequence: 2,
            predecessor_digest: Some(reserve_head.clone()),
            control_epoch: 0,
            kind: EventKind::IssueAuthority,
            from_state: Some(DispatchState::Reserved),
            to_state: DispatchState::Reserved,
            authority_id: issue_authority.authority_id.clone(),
            request_digest: issue_request.clone(),
            payload_digest: sha256_digest(&grant_bytes),
            occurred_at: "2026-08-12T00:00:03Z".to_owned(),
        };
        let issue_path = temporary.path().join("issue-event.toml");
        std::fs::write(&issue_path, toml::to_string(&issue).expect("issue event"))
            .expect("write issue");
        let issue_arguments = MutationArguments {
            target: target_path,
            run_id: "run-1".to_owned(),
            event: issue_path,
            expected_head: format!("1:{reserve_head}"),
            control_epoch: 0,
            authority: issue_authority_path,
            trust_root: root_path,
            request_digest: issue_request,
            now: "2026-08-12T00:00:04Z".to_owned(),
            receipt: None,
            grant: Some(grant_path),
            subject_authority: None,
            emergency_reason_digest: None,
        };
        assert_eq!(
            mutate("authority-issue", &issue_arguments)
                .expect("issue")
                .status,
            "success"
        );
        assert!(temporary
            .path()
            .join("consumer/.hive/runs/run-1/events/authorities/grant-1.toml")
            .is_file());
    }

    fn signed_authority(
        key: &SigningKey,
        authority_id: &str,
        action: AuthorityAction,
        target_digest: String,
        expected_head: &str,
        control_epoch: u64,
        request_digest: &str,
    ) -> ActionAuthority {
        let mut authority = ActionAuthority {
            schema_version: 1,
            authority_id: authority_id.to_owned(),
            trust_root_id: "root-1".to_owned(),
            principal_id: "principal-1".to_owned(),
            role_id: "role-1".to_owned(),
            action,
            target_digest,
            expected_head: expected_head.to_owned(),
            control_epoch,
            request_digest: request_digest.to_owned(),
            nonce: format!("nonce-{authority_id}"),
            issued_at: "2026-08-12T00:00:00Z".to_owned(),
            valid_until: "2026-08-13T00:00:00Z".to_owned(),
            key_id: "key-1".to_owned(),
            signature: format!("ed25519:{}", "0".repeat(128)),
        };
        let signature = key.sign(&authority.signing_message().expect("signing message"));
        authority.signature = format!("ed25519:{}", hex_bytes(&signature.to_bytes()));
        authority
    }

    fn hex_bytes(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut encoded, byte| {
            write!(encoded, "{byte:02x}").expect("write hex");
            encoded
        })
    }
}
