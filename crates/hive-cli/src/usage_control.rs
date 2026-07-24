use super::{emit_action_result, ActionResult, Evidence};
use crate::run::{portable_relative_path, AdapterError, FileSnapshot, PinnedTarget};
use crate::usage;
use hive_core::sha256_digest;
use hive_core::usage_guard::{evaluate_usage, UsageDecision, UsagePolicy};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_CONFIG_BYTES: usize = 64 * 1024;
const MAX_CONTROL_BYTES: usize = 16 * 1024;
const CONFIG_PATH: &str = ".hive/config/harness.toml";
const USAGE_CONTROL: &str = "\
Inspect or change the installed usage safeguard.

USAGE:
    hive usage enforce --target <dir> --session-id <id> --process-id <positive-u32> [--account-digest <sha256:...>] --output json
    hive usage status --target <dir> --session-id <id> --process-id <positive-u32> --output json
    hive usage threshold --target <dir> --remaining-percent <1..99> --output json
    hive usage session --target <dir> --session-id <id> --process-id <positive-u32> --action enable|disable|toggle [--confirm-session-disable] --output json
";

#[derive(Debug)]
struct StatusArguments {
    target: PathBuf,
    binding: ParsedBinding,
}

#[derive(Debug)]
struct EnforceArguments {
    target: PathBuf,
    binding: ParsedBinding,
    account_digest: Option<String>,
}

#[derive(Debug)]
struct ThresholdArguments {
    target: PathBuf,
    remaining_percent: u8,
}

#[derive(Debug)]
struct SessionArguments {
    target: PathBuf,
    binding: ParsedBinding,
    action: SessionAction,
    confirm_disable: bool,
}

#[derive(Clone, Debug)]
struct SessionBinding {
    host_scope: String,
    session_digest: String,
    process_id: u32,
}

#[derive(Clone, Debug)]
struct ParsedBinding {
    session_id: String,
    process_id: u32,
}

struct InstalledUsageConfig {
    threshold: u8,
    primary_host: String,
    bytes: Vec<u8>,
}

struct TurnObservation {
    decision: Option<&'static str>,
    selected_window: &'static str,
    measured_at: u64,
    evidence_digest: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SessionAction {
    Enable,
    Disable,
    Toggle,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SessionControl {
    schema_version: u32,
    host_scope: String,
    session_id_digest: String,
    process_id: u32,
    guard_enabled: bool,
    revision: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HaltMarker {
    schema_version: u32,
    host_scope: String,
    session_id_digest: String,
    process_id: u32,
    decision: String,
    selected_window: String,
    threshold_remaining_percent: u8,
    measured_at: u64,
    evidence_digest: String,
    revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OverrideState {
    Absent,
    Current,
    Stale,
}

struct LoadedControl {
    relative: PathBuf,
    snapshot: FileSnapshot,
    control: Option<SessionControl>,
    state: OverrideState,
}

struct LoadedHalt {
    relative: PathBuf,
    snapshot: FileSnapshot,
    bytes: Option<Vec<u8>>,
    marker: Option<HaltMarker>,
    state: OverrideState,
}

pub(crate) fn run_usage_control(arguments: &[String]) -> ExitCode {
    if arguments == ["enforce", "--help"]
        || arguments == ["status", "--help"]
        || arguments == ["threshold", "--help"]
        || arguments == ["session", "--help"]
    {
        print!("{USAGE_CONTROL}");
        return ExitCode::SUCCESS;
    }
    let (action, result) = match arguments.first().map(String::as_str) {
        Some("enforce") => (
            "CheckUsage",
            parse_enforce(&arguments[1..]).and_then(|parsed| enforce(&parsed)),
        ),
        Some("status") => (
            "ShowUsageStatus",
            parse_status(&arguments[1..]).and_then(|parsed| status(&parsed)),
        ),
        Some("threshold") => (
            "SetUsageThreshold",
            parse_threshold(&arguments[1..]).and_then(|parsed| set_threshold(&parsed)),
        ),
        Some("session") => (
            "ControlUsageSession",
            parse_session(&arguments[1..]).and_then(|parsed| control_session(&parsed)),
        ),
        Some(other) => (
            "CheckUsage",
            Err(AdapterError::Input(format!(
                "unknown usage action: {other}"
            ))),
        ),
        None => (
            "CheckUsage",
            Err(AdapterError::Input("usage requires an action".to_owned())),
        ),
    };
    let result = result.unwrap_or_else(|error| failure_result(action, &error));
    emit_action_result(&result)
}

fn parse_status(arguments: &[String]) -> Result<StatusArguments, AdapterError> {
    let options = parse_key_value_options(
        arguments,
        &["--target", "--session-id", "--process-id"],
        false,
    )?;
    Ok(StatusArguments {
        target: PathBuf::from(required(&options, "--target")?),
        binding: parse_binding(&options)?,
    })
}

fn parse_enforce(arguments: &[String]) -> Result<EnforceArguments, AdapterError> {
    let options = parse_key_value_options(
        arguments,
        &[
            "--target",
            "--session-id",
            "--process-id",
            "--account-digest",
        ],
        false,
    )?;
    let account_digest = optional(&options, "--account-digest").map(str::to_owned);
    if account_digest
        .as_deref()
        .is_some_and(|value| !is_sha256_digest(value))
    {
        return Err(AdapterError::Input(
            "--account-digest must be sha256 followed by 64 lowercase hex digits".to_owned(),
        ));
    }
    Ok(EnforceArguments {
        target: PathBuf::from(required(&options, "--target")?),
        binding: parse_binding(&options)?,
        account_digest,
    })
}

fn parse_threshold(arguments: &[String]) -> Result<ThresholdArguments, AdapterError> {
    let options = parse_key_value_options(arguments, &["--target", "--remaining-percent"], false)?;
    let raw = required(&options, "--remaining-percent")?;
    let remaining_percent = raw
        .parse::<u8>()
        .ok()
        .filter(|value| (1..=99).contains(value) && value.to_string() == raw)
        .ok_or_else(|| {
            AdapterError::Input(
                "--remaining-percent must be an integer from 1 through 99".to_owned(),
            )
        })?;
    Ok(ThresholdArguments {
        target: PathBuf::from(required(&options, "--target")?),
        remaining_percent,
    })
}

fn parse_session(arguments: &[String]) -> Result<SessionArguments, AdapterError> {
    let options = parse_key_value_options(
        arguments,
        &["--target", "--session-id", "--process-id", "--action"],
        true,
    )?;
    let action = match required(&options, "--action")? {
        "enable" => SessionAction::Enable,
        "disable" => SessionAction::Disable,
        "toggle" => SessionAction::Toggle,
        _ => {
            return Err(AdapterError::Input(
                "--action must be enable, disable, or toggle".to_owned(),
            ));
        }
    };
    let confirm_disable = options
        .iter()
        .any(|(option, _)| *option == "--confirm-session-disable");
    if action == SessionAction::Disable && !confirm_disable {
        return Err(AdapterError::Input(
            "disabling the current-session usage safeguard requires --confirm-session-disable"
                .to_owned(),
        ));
    }
    Ok(SessionArguments {
        target: PathBuf::from(required(&options, "--target")?),
        binding: parse_binding(&options)?,
        action,
        confirm_disable,
    })
}

fn parse_key_value_options<'a>(
    arguments: &'a [String],
    allowed: &[&str],
    allow_confirmation: bool,
) -> Result<Vec<(&'a str, &'a str)>, AdapterError> {
    let mut options = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--confirm-session-disable" {
            if !allow_confirmation {
                return Err(AdapterError::Input(format!("unknown option: {option}")));
            }
            if options.iter().any(|(existing, _)| *existing == option) {
                return Err(AdapterError::Input(format!("duplicate option: {option}")));
            }
            options.push((option, "true"));
            index += 1;
            continue;
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| AdapterError::Input(format!("missing value for {option}")))?;
        if option == "--output" {
            if value != "json" {
                return Err(AdapterError::Input(
                    "usage control commands require --output json".to_owned(),
                ));
            }
        } else if !allowed.contains(&option) {
            return Err(AdapterError::Input(format!("unknown option: {option}")));
        }
        if options.iter().any(|(existing, _)| *existing == option) {
            return Err(AdapterError::Input(format!("duplicate option: {option}")));
        }
        options.push((option, value.as_str()));
        index += 2;
    }
    if optional(&options, "--output") != Some("json") {
        return Err(AdapterError::Input(
            "usage control commands require --output json".to_owned(),
        ));
    }
    Ok(options)
}

fn parse_binding(options: &[(&str, &str)]) -> Result<ParsedBinding, AdapterError> {
    let session_id = required(options, "--session-id")?;
    if session_id.is_empty() || session_id.len() > 256 || session_id.chars().any(char::is_control) {
        return Err(AdapterError::Input(
            "--session-id must contain 1 through 256 non-control characters".to_owned(),
        ));
    }
    let raw_process_id = required(options, "--process-id")?;
    let process_id = raw_process_id
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0 && value.to_string() == raw_process_id)
        .ok_or_else(|| {
            AdapterError::Input("--process-id must be a positive decimal u32".to_owned())
        })?;
    Ok(ParsedBinding {
        session_id: session_id.to_owned(),
        process_id,
    })
}

fn bind_session(parsed: &ParsedBinding, primary_host: &str) -> SessionBinding {
    let mut scoped = Vec::with_capacity(primary_host.len() + parsed.session_id.len() + 1);
    scoped.extend_from_slice(primary_host.as_bytes());
    scoped.push(0);
    scoped.extend_from_slice(parsed.session_id.as_bytes());
    SessionBinding {
        host_scope: primary_host.to_owned(),
        session_digest: sha256_digest(&scoped),
        process_id: parsed.process_id,
    }
}

fn required<'a>(options: &[(&'a str, &'a str)], name: &str) -> Result<&'a str, AdapterError> {
    optional(options, name)
        .ok_or_else(|| AdapterError::Input(format!("missing required option {name}")))
}

fn optional<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(option, value)| (*option == name).then_some(*value))
}

fn status(arguments: &StatusArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let config = read_installed_config(&target)?;
    let binding = bind_session(&arguments.binding, &config.primary_host);
    let loaded = load_control(&target, &binding)?;
    let halt = load_halt(&target, &binding)?;
    let guard_enabled = effective_enabled(&loaded);
    let override_state_name = override_name(loaded.state);
    let halted = guard_enabled && halt.state == OverrideState::Current;
    let mut evidence = vec![Evidence {
        kind: "file",
        locator: CONFIG_PATH.to_owned(),
        digest: sha256_digest(&config.bytes),
    }];
    if let Some(bytes) = halt.bytes.as_deref() {
        evidence.push(Evidence {
            kind: "file",
            locator: portable_relative_path(&halt.relative),
            digest: sha256_digest(bytes),
        });
    }
    Ok(ActionResult {
        schema_version: 1,
        action: "ShowUsageStatus",
        status: if halted { "blocked" } else { "success" },
        exit_code: if halted { 3 } else { 0 },
        code: if halted {
            "hive.usage-session-halted"
        } else {
            "hive.usage-status"
        },
        message: if halted {
            "the current session remains halted until an explicit session disable".to_owned()
        } else {
            "installed usage safeguard status is available".to_owned()
        },
        changed_paths: Vec::new(),
        evidence,
        next_action: None,
        data: Some(json!({
            "threshold_remaining_percent": config.threshold,
            "host_scope": binding.host_scope,
            "guard_enabled": guard_enabled,
            "session_override": override_state_name,
            "halt_marker": override_name(halt.state),
            "halt_decision": halt.marker.as_ref().map(|marker| marker.decision.as_str()),
            "session_id_digest": binding.session_digest,
            "process_id": binding.process_id,
        })),
    })
}

fn enforce(arguments: &EnforceArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let config = read_installed_config(&target)?;
    let binding = bind_session(&arguments.binding, &config.primary_host);
    let loaded = load_control(&target, &binding)?;
    if !effective_enabled(&loaded) {
        return Ok(ActionResult {
            schema_version: 1,
            action: "CheckUsage",
            status: "success",
            exit_code: 0,
            code: "hive.usage-session-bypassed",
            message: "usage safeguard is explicitly disabled for this session binding".to_owned(),
            changed_paths: Vec::new(),
            evidence: vec![Evidence {
                kind: "file",
                locator: CONFIG_PATH.to_owned(),
                digest: sha256_digest(&config.bytes),
            }],
            next_action: None,
            data: Some(json!({
                "guard_enabled": false,
                "host_scope": binding.host_scope,
                "session_id_digest": binding.session_digest,
                "process_id": binding.process_id,
                "threshold_remaining_percent": config.threshold,
            })),
        });
    }

    let halt = load_halt(&target, &binding)?;
    if halt.state == OverrideState::Current {
        return Ok(halted_result(&binding, &halt, false));
    }
    if halt.state == OverrideState::Stale {
        return Err(AdapterError::Safety(
            "session halt marker belongs to a different process binding".to_owned(),
        ));
    }

    let observation = observe_usage(&config, arguments.account_digest.as_deref());
    let Some(decision) = observation.decision else {
        return Ok(allowed_result(&binding, &config, &observation));
    };

    let marker = HaltMarker {
        schema_version: 1,
        host_scope: binding.host_scope.clone(),
        session_id_digest: binding.session_digest.clone(),
        process_id: binding.process_id,
        decision: decision.to_owned(),
        selected_window: observation.selected_window.to_owned(),
        threshold_remaining_percent: config.threshold,
        measured_at: observation.measured_at,
        evidence_digest: observation.evidence_digest,
        revision: 1,
    };
    let desired = serde_json::to_vec(&marker)
        .map_err(|error| AdapterError::Internal(format!("cannot encode halt marker: {error}")))?
        .into_iter()
        .chain(std::iter::once(b'\n'))
        .collect::<Vec<_>>();
    let changed = target.publish_runtime(&halt.relative, &halt.snapshot, &desired)?;
    let published = load_halt(&target, &binding)?;
    if published.state != OverrideState::Current {
        return Err(AdapterError::Verification(
            "published halt marker did not bind to the current session".to_owned(),
        ));
    }
    Ok(halted_result(&binding, &published, changed))
}

fn observe_usage(config: &InstalledUsageConfig, account_digest: Option<&str>) -> TurnObservation {
    let sampled_at = SystemTime::now();
    let sampled_at_unix = sampled_at
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok());
    let snapshot = if config.primary_host == "codex" {
        match account_digest {
            Some(account_digest) => {
                usage::check_with_runner(&usage::SystemCommandRunner, account_digest, sampled_at)
            }
            None => usage::check_unique_with_runner(&usage::SystemCommandRunner, sampled_at),
        }
    } else {
        Err(usage::SensorError::WrongProvider)
    };
    let Ok(snapshot) = snapshot else {
        let error = snapshot.expect_err("the failed sensor result was matched");
        return TurnObservation {
            decision: Some("usage-unknown"),
            selected_window: "unknown",
            measured_at: sampled_at_unix
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(0),
            evidence_digest: sha256_digest(format!("codexbar-error:{error}").as_bytes()),
        };
    };
    let decision = match sampled_at_unix.and_then(|now| {
        UsagePolicy::new("codexbar", "0.45.2", "codex", &snapshot.account_digest)
            .with_stop_remaining_percent(config.threshold)
            .ok()
            .map(|policy| evaluate_usage(&policy, &snapshot.core_snapshots(), &[], now))
    }) {
        Some(UsageDecision::Allow(_)) => None,
        Some(UsageDecision::Block(_)) => Some("halted"),
        Some(UsageDecision::Unknown(_)) | None => Some("usage-unknown"),
    };
    TurnObservation {
        decision,
        selected_window: snapshot
            .windows
            .first()
            .map_or("unknown", |window| window.name),
        measured_at: snapshot.measured_at,
        evidence_digest: snapshot.evidence_digest(),
    }
}

fn allowed_result(
    binding: &SessionBinding,
    config: &InstalledUsageConfig,
    observation: &TurnObservation,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "CheckUsage",
        status: "success",
        exit_code: 0,
        code: "hive.usage-allowed",
        message: "subscription usage permits the current host turn".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![
            Evidence {
                kind: "file",
                locator: CONFIG_PATH.to_owned(),
                digest: sha256_digest(&config.bytes),
            },
            Evidence {
                kind: "report",
                locator: "usage-snapshots:normalized".to_owned(),
                digest: observation.evidence_digest.clone(),
            },
        ],
        next_action: None,
        data: Some(json!({
            "guard_enabled": true,
            "host_scope": binding.host_scope,
            "session_id_digest": binding.session_digest,
            "process_id": binding.process_id,
            "selected_window": observation.selected_window,
            "threshold_remaining_percent": config.threshold,
            "measured_at": observation.measured_at,
        })),
    }
}

fn halted_result(binding: &SessionBinding, halt: &LoadedHalt, changed: bool) -> ActionResult {
    let marker = halt
        .marker
        .as_ref()
        .expect("a current halt state always has a validated marker");
    let locator = portable_relative_path(&halt.relative);
    ActionResult {
        schema_version: 1,
        action: "CheckUsage",
        status: "blocked",
        exit_code: 3,
        code: if marker.decision == "halted" {
            "hive.usage-limited"
        } else {
            "hive.usage-unknown"
        },
        message: if marker.decision == "halted" {
            format!(
                "subscription usage is at or below the {}% remaining threshold",
                marker.threshold_remaining_percent
            )
        } else {
            "subscription usage could not be verified safely".to_owned()
        },
        changed_paths: changed.then(|| locator.clone()).into_iter().collect(),
        evidence: halt
            .bytes
            .as_ref()
            .map(|bytes| Evidence {
                kind: "file",
                locator,
                digest: sha256_digest(bytes),
            })
            .into_iter()
            .collect(),
        next_action: None,
        data: Some(json!({
            "guard_enabled": true,
            "host_scope": binding.host_scope,
            "session_id_digest": binding.session_digest,
            "process_id": binding.process_id,
            "decision": marker.decision,
            "selected_window": marker.selected_window,
            "threshold_remaining_percent": marker.threshold_remaining_percent,
            "measured_at": marker.measured_at,
            "evidence_digest": marker.evidence_digest,
            "revision": marker.revision,
        })),
    }
}

fn set_threshold(arguments: &ThresholdArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let relative = Path::new(CONFIG_PATH);
    let snapshot = target.snapshot_bounded(relative, MAX_CONFIG_BYTES)?;
    let bytes = snapshot.bytes().ok_or_else(|| {
        AdapterError::Safety("installed .hive/config/harness.toml is required".to_owned())
    })?;
    let config = parse_installed_config(bytes.to_vec())?;
    let (current, range) = threshold_range(bytes)?;
    debug_assert_eq!(current, config.threshold);
    let desired = if current == arguments.remaining_percent {
        bytes.to_vec()
    } else {
        let mut desired = Vec::with_capacity(bytes.len() + 2);
        desired.extend_from_slice(&bytes[..range.start]);
        desired.extend_from_slice(arguments.remaining_percent.to_string().as_bytes());
        desired.extend_from_slice(&bytes[range.end..]);
        desired
    };
    let changed = target.publish(relative, &snapshot, &desired)?;
    let code = if changed {
        "hive.usage-threshold-updated"
    } else {
        "hive.usage-threshold-unchanged"
    };
    Ok(ActionResult {
        schema_version: 1,
        action: "SetUsageThreshold",
        status: "success",
        exit_code: 0,
        code,
        message: if changed {
            "installed usage threshold was updated atomically".to_owned()
        } else {
            "installed usage threshold already matched the requested value".to_owned()
        },
        changed_paths: changed
            .then(|| CONFIG_PATH.to_owned())
            .into_iter()
            .collect(),
        evidence: vec![Evidence {
            kind: "file",
            locator: CONFIG_PATH.to_owned(),
            digest: sha256_digest(&desired),
        }],
        next_action: None,
        data: Some(json!({
            "previous_remaining_percent": current,
            "threshold_remaining_percent": arguments.remaining_percent,
        })),
    })
}

fn control_session(arguments: &SessionArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let config = read_installed_config(&target)?;
    let binding = bind_session(&arguments.binding, &config.primary_host);
    let loaded = load_control(&target, &binding)?;
    let currently_enabled = effective_enabled(&loaded);
    let desired_enabled = match arguments.action {
        SessionAction::Enable => true,
        SessionAction::Disable => false,
        SessionAction::Toggle => !currently_enabled,
    };
    if !desired_enabled && !arguments.confirm_disable {
        return Err(AdapterError::Input(
            "disabling the current-session usage safeguard requires --confirm-session-disable"
                .to_owned(),
        ));
    }
    let revision = loaded
        .control
        .as_ref()
        .filter(|_| loaded.state == OverrideState::Current)
        .map_or(1, |control| control.revision.saturating_add(1));
    let desired = SessionControl {
        schema_version: 1,
        host_scope: binding.host_scope.clone(),
        session_id_digest: binding.session_digest.clone(),
        process_id: binding.process_id,
        guard_enabled: desired_enabled,
        revision,
    };
    let desired_bytes = serde_json::to_vec(&desired)
        .map_err(|error| AdapterError::Internal(format!("cannot encode session control: {error}")))?
        .into_iter()
        .chain(std::iter::once(b'\n'))
        .collect::<Vec<_>>();
    let changed = target.publish_runtime(&loaded.relative, &loaded.snapshot, &desired_bytes)?;
    let code = if desired_enabled {
        "hive.usage-session-enabled"
    } else {
        "hive.usage-session-disabled"
    };
    let locator = portable_relative_path(&loaded.relative);
    Ok(ActionResult {
        schema_version: 1,
        action: "ControlUsageSession",
        status: "success",
        exit_code: 0,
        code,
        message: if desired_enabled {
            "usage safeguard is enabled for the current session binding".to_owned()
        } else {
            "usage safeguard is disabled only for the current session binding".to_owned()
        },
        changed_paths: changed.then(|| locator.clone()).into_iter().collect(),
        evidence: vec![Evidence {
            kind: "file",
            locator,
            digest: sha256_digest(&desired_bytes),
        }],
        next_action: None,
        data: Some(json!({
            "guard_enabled": desired_enabled,
            "session_override": "current",
            "host_scope": binding.host_scope,
            "session_id_digest": binding.session_digest,
            "process_id": binding.process_id,
            "revision": revision,
        })),
    })
}

fn read_installed_config(target: &PinnedTarget) -> Result<InstalledUsageConfig, AdapterError> {
    let bytes = target
        .read_optional(Path::new(CONFIG_PATH), MAX_CONFIG_BYTES)?
        .ok_or_else(|| {
            AdapterError::Safety("installed .hive/config/harness.toml is required".to_owned())
        })?;
    parse_installed_config(bytes)
}

fn parse_installed_config(bytes: Vec<u8>) -> Result<InstalledUsageConfig, AdapterError> {
    validate_config(&bytes)?;
    let (threshold, _) = threshold_range(&bytes)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| AdapterError::Safety("installed harness.toml must be UTF-8".to_owned()))?;
    let table = toml::from_str::<toml::Table>(text).map_err(|error| {
        AdapterError::Safety(format!("installed harness.toml is invalid: {error}"))
    })?;
    let primary_host = table
        .get("primary_host")
        .and_then(toml::Value::as_str)
        .filter(|value| matches!(*value, "codex" | "claude" | "antigravity"))
        .ok_or_else(|| {
            AdapterError::Safety(
                "installed harness.toml primary_host must be codex, claude, or antigravity"
                    .to_owned(),
            )
        })?
        .to_owned();
    Ok(InstalledUsageConfig {
        threshold,
        primary_host,
        bytes,
    })
}

fn validate_config(bytes: &[u8]) -> Result<(), AdapterError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AdapterError::Safety("installed harness.toml must be UTF-8".to_owned()))?;
    let table = toml::from_str::<toml::Table>(text).map_err(|error| {
        AdapterError::Safety(format!("installed harness.toml is invalid: {error}"))
    })?;
    if table
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        != Some(1)
    {
        return Err(AdapterError::Safety(
            "installed harness.toml schema_version must equal 1".to_owned(),
        ));
    }
    let cli_major = semver_major(env!("CARGO_PKG_VERSION"))
        .ok_or_else(|| AdapterError::Internal("Hive CLI package version is invalid".to_owned()))?;
    for key in ["harness_version", "source_release_version"] {
        let version = table
            .get(key)
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                AdapterError::Safety(format!("installed harness.toml is missing {key}"))
            })?;
        let installed_major = semver_major(version).ok_or_else(|| {
            AdapterError::Safety(format!("installed {key} is not a valid semantic version"))
        })?;
        if installed_major != cli_major {
            return Err(AdapterError::Safety(format!(
                "installed {key} is not same-major compatible with this Hive CLI"
            )));
        }
    }
    Ok(())
}

fn semver_major(version: &str) -> Option<u64> {
    let mut components = version.split('.');
    let major = components.next()?.parse().ok()?;
    let minor = components.next()?.parse::<u64>().ok()?;
    let patch_and_suffix = components.next()?;
    if components.next().is_some()
        || patch_and_suffix
            .split_once(['-', '+'])
            .map_or(patch_and_suffix, |(patch, _)| patch)
            .parse::<u64>()
            .is_err()
    {
        return None;
    }
    let _ = minor;
    Some(major)
}

fn threshold_range(bytes: &[u8]) -> Result<(u8, std::ops::Range<usize>), AdapterError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AdapterError::Safety("installed harness.toml must be UTF-8".to_owned()))?;
    let mut offset = 0;
    let mut found = None;
    let mut entered_table = false;
    for line in text.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let trimmed = content.trim();
        if trimmed.starts_with('[') {
            entered_table = true;
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some((raw_key, raw_value)) = content.split_once('=') {
                if raw_key.trim() == "usage_stop_remaining_percent" {
                    if entered_table || found.is_some() {
                        return Err(AdapterError::Safety(
                            "installed usage threshold must be one unique root key".to_owned(),
                        ));
                    }
                    let value_start_in_line = raw_key.len() + 1;
                    let whitespace = raw_value.len() - raw_value.trim_start().len();
                    let digits = raw_value.trim_start();
                    let digit_len = digits.bytes().take_while(u8::is_ascii_digit).count();
                    let remainder = &digits[digit_len..];
                    if digit_len == 0
                        || !remainder.trim().is_empty()
                        || digits[..digit_len].starts_with('0')
                    {
                        return Err(AdapterError::Safety(
                            "installed usage threshold must be a plain integer from 1 to 99"
                                .to_owned(),
                        ));
                    }
                    let raw = &digits[..digit_len];
                    let value = raw
                        .parse::<u8>()
                        .ok()
                        .filter(|value| (1..=99).contains(value))
                        .ok_or_else(|| {
                            AdapterError::Safety(
                                "installed usage threshold must be an integer from 1 to 99"
                                    .to_owned(),
                            )
                        })?;
                    let start = offset + value_start_in_line + whitespace;
                    found = Some((value, start..start + digit_len));
                }
            }
        }
        offset += line.len();
    }
    found.ok_or_else(|| {
        AdapterError::Safety(
            "installed harness.toml is missing usage_stop_remaining_percent".to_owned(),
        )
    })
}

fn control_path(binding: &SessionBinding) -> PathBuf {
    let digest = binding
        .session_digest
        .strip_prefix("sha256:")
        .unwrap_or(&binding.session_digest);
    Path::new(".hive/runtime/usage-guard/sessions")
        .join(digest)
        .join("control.json")
}

fn halt_path(binding: &SessionBinding) -> PathBuf {
    control_path(binding).with_file_name("halt.json")
}

fn load_control(
    target: &PinnedTarget,
    binding: &SessionBinding,
) -> Result<LoadedControl, AdapterError> {
    let relative = control_path(binding);
    let snapshot = target.snapshot(&relative)?;
    let Some(bytes) = snapshot.bytes() else {
        return Ok(LoadedControl {
            relative,
            snapshot,
            control: None,
            state: OverrideState::Absent,
        });
    };
    if bytes.len() > MAX_CONTROL_BYTES {
        return Err(AdapterError::Safety(
            "session control exceeds the bounded runtime size".to_owned(),
        ));
    }
    let control: SessionControl = serde_json::from_slice(bytes)
        .map_err(|error| AdapterError::Safety(format!("session control is malformed: {error}")))?;
    if control.schema_version != 1
        || control.revision == 0
        || control.host_scope != binding.host_scope
        || control.session_id_digest != binding.session_digest
    {
        return Err(AdapterError::Safety(
            "session control binding is invalid".to_owned(),
        ));
    }
    let state = if control.process_id == binding.process_id {
        OverrideState::Current
    } else {
        OverrideState::Stale
    };
    Ok(LoadedControl {
        relative,
        snapshot,
        control: Some(control),
        state,
    })
}

fn load_halt(target: &PinnedTarget, binding: &SessionBinding) -> Result<LoadedHalt, AdapterError> {
    let relative = halt_path(binding);
    let snapshot = target.snapshot(&relative)?;
    let Some(bytes) = snapshot.bytes().map(<[u8]>::to_vec) else {
        return Ok(LoadedHalt {
            relative,
            snapshot,
            bytes: None,
            marker: None,
            state: OverrideState::Absent,
        });
    };
    if bytes.len() > MAX_CONTROL_BYTES {
        return Err(AdapterError::Safety(
            "session halt marker exceeds the bounded runtime size".to_owned(),
        ));
    }
    let marker: HaltMarker = serde_json::from_slice(&bytes).map_err(|error| {
        AdapterError::Safety(format!("session halt marker is malformed: {error}"))
    })?;
    if marker.schema_version != 1
        || marker.host_scope != binding.host_scope
        || marker.session_id_digest != binding.session_digest
        || !is_sha256_digest(&marker.evidence_digest)
        || !matches!(marker.decision.as_str(), "halted" | "usage-unknown")
        || !matches!(
            marker.selected_window.as_str(),
            "session" | "weekly" | "unknown"
        )
        || !(1..=99).contains(&marker.threshold_remaining_percent)
        || marker.revision == 0
    {
        return Err(AdapterError::Safety(
            "session halt marker binding is invalid".to_owned(),
        ));
    }
    let state = if marker.process_id == binding.process_id {
        OverrideState::Current
    } else {
        OverrideState::Stale
    };
    Ok(LoadedHalt {
        relative,
        snapshot,
        bytes: Some(bytes),
        marker: Some(marker),
        state,
    })
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn effective_enabled(loaded: &LoadedControl) -> bool {
    if loaded.state != OverrideState::Current {
        return true;
    }
    loaded
        .control
        .as_ref()
        .is_none_or(|control| control.guard_enabled)
}

const fn override_name(state: OverrideState) -> &'static str {
    match state {
        OverrideState::Absent => "absent",
        OverrideState::Current => "current",
        OverrideState::Stale => "stale",
    }
}

fn failure_result(action: &'static str, error: &AdapterError) -> ActionResult {
    let code = match error {
        AdapterError::Input(_) => "hive.invalid-input",
        AdapterError::Safety(_)
        | AdapterError::UpdateRecoveryRequired(_)
        | AdapterError::OwnerBlocked(_) => "hive.usage-control-blocked",
        AdapterError::Conflict(_) => "hive.usage-control-conflict",
        AdapterError::Unsupported(_) | AdapterError::OwnerUnsupported(_) => {
            "hive.usage-control-unsupported"
        }
        AdapterError::Verification(_) => "hive.usage-control-verification-failed",
        AdapterError::Internal(_) | AdapterError::Rollback(_) => "hive.internal-error",
    };
    ActionResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code,
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}
