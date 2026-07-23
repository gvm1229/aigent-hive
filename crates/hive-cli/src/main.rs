use hive_core::usage_guard::{evaluate_usage, UsageDecision, UsagePolicy};
use hive_core::{
    ensure_consumer_target, ensure_no_symlink_ancestors, sha256_digest, source_marker_path,
    validate_project_relative,
};
use hive_render::{
    authorize_hook, execute_setup, HookAuthorization, RenderError, SetupMode, SetupRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

mod usage;

const USAGE: &str = "\
Aigent Hive

USAGE:
    hive doctor
    hive check-target <path>
    hive setup --help
    hive setup --target <dir> --answers <yml> --capabilities <json> (--dry-run|--apply|--validate) [--reconfigure-role <role-id>]... --output json
    hive hook --capability <name> --event <event> [--input <json>] --output json
    hive usage check --account-digest <sha256:...> [--threshold <1..99>] --output json
";

const SETUP_USAGE: &str = "\
Create or validate an installed Aigent Hive consumer harness.

USAGE:
    hive setup --target <dir> --answers <yml> --capabilities <json> (--dry-run|--apply|--validate) [--reconfigure-role <role-id>]... --output json

MODES:
    --dry-run    Render and validate in staging without changing the target
    --apply      Render, validate, and apply only manifest-owned changes
    --validate   Validate the installed harness without changing the target
";

const STALE_MARKER: &[u8] = b"{\"schema_version\":1,\"stale\":true}\n";

#[derive(Serialize)]
struct ActionResult {
    schema_version: u32,
    action: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
    changed_paths: Vec<String>,
    evidence: Vec<Evidence>,
    next_action: Option<String>,
}

#[derive(Serialize)]
struct Evidence {
    kind: &'static str,
    locator: String,
    digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HookInput {
    schema_version: u32,
    event: String,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    operation: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    dry_run: Option<bool>,
    #[serde(default)]
    backup_present: Option<bool>,
    #[serde(default)]
    staging_validated: Option<bool>,
    #[serde(default)]
    checkpoint_present: Option<bool>,
    #[serde(default)]
    status_path: Option<String>,
}

#[derive(Serialize)]
struct HookResult {
    schema_version: u32,
    decision: &'static str,
    active: bool,
    code: &'static str,
    message: String,
}

struct SetupArguments {
    target: PathBuf,
    answers: PathBuf,
    capabilities: PathBuf,
    mode: SetupMode,
    reconfigure_roles: BTreeSet<String>,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("setup") if is_help_request(&arguments[1..]) => {
            print!("{SETUP_USAGE}");
            ExitCode::SUCCESS
        }
        Some("setup") => run_setup(&arguments[1..]),
        Some("hook") => run_hook(&arguments[1..]),
        Some("usage") => run_usage(&arguments[1..]),
        _ if wants_json(&arguments) => {
            let command = arguments.first().map_or("<missing>", String::as_str);
            let result = ActionResult {
                schema_version: 1,
                action: "UnknownAction",
                status: "error",
                exit_code: 2,
                code: "hive.unknown-action",
                message: format!("unknown top-level action: {command}"),
                changed_paths: Vec::new(),
                evidence: Vec::new(),
                next_action: None,
            };
            emit_json_result(&result);
            eprintln!("error: {}", result.message);
            ExitCode::from(2)
        }
        _ => match run_human(arguments.into_iter()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::from(2)
            }
        },
    }
}

struct UsageArguments {
    account_digest: String,
    threshold: u8,
}

fn run_usage(arguments: &[String]) -> ExitCode {
    let result = match parse_usage(arguments) {
        Ok(arguments) => check_usage(&arguments),
        Err(message) => ActionResult {
            schema_version: 1,
            action: "CheckUsage",
            status: "error",
            exit_code: 2,
            code: "hive.invalid-input",
            message,
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
        },
    };
    emit_json_result(&result);
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
    ExitCode::from(result.exit_code)
}

fn parse_usage(arguments: &[String]) -> Result<UsageArguments, String> {
    if arguments.first().map(String::as_str) != Some("check") {
        return Err("usage requires the check action".to_owned());
    }
    let mut account_digest = None;
    let mut threshold = 10_u8;
    let mut threshold_seen = false;
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        let value = arguments
            .get(index)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--account-digest" if account_digest.is_none() => {
                account_digest = Some(value.clone());
            }
            "--threshold" if !threshold_seen => {
                threshold = value
                    .parse::<u8>()
                    .ok()
                    .filter(|value| (1..=99).contains(value))
                    .ok_or_else(|| "threshold must be an integer from 1 through 99".to_owned())?;
                threshold_seen = true;
            }
            "--output" if output.is_none() => output = Some(value.clone()),
            "--account-digest" | "--threshold" | "--output" => {
                return Err(format!("duplicate usage option: {option}"));
            }
            _ => return Err(format!("unknown usage option: {option}")),
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err("usage check requires --output json".to_owned());
    }
    let account_digest =
        account_digest.ok_or_else(|| "missing required option --account-digest".to_owned())?;
    if !is_sha256_digest(&account_digest) {
        return Err("account digest must be sha256 followed by 64 lowercase hex digits".to_owned());
    }
    Ok(UsageArguments {
        account_digest,
        threshold,
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

fn check_usage(arguments: &UsageArguments) -> ActionResult {
    let now = SystemTime::now();
    let now_unix_seconds = now
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok());
    let Some(now_unix_seconds) = now_unix_seconds else {
        return usage_unknown_result("system clock cannot be used for usage enforcement", None);
    };
    let snapshot =
        match usage::check_with_runner(&usage::SystemCommandRunner, &arguments.account_digest, now)
        {
            Ok(snapshot) => snapshot,
            Err(error) => return usage_unknown_result(&error.to_string(), None),
        };
    let evidence_digest = snapshot.evidence_digest();
    let Ok(policy) = UsagePolicy::new("codexbar", "0.45.2", "codex", &arguments.account_digest)
        .with_stop_remaining_percent(arguments.threshold)
    else {
        return usage_unknown_result("usage threshold policy is invalid", None);
    };
    match evaluate_usage(&policy, &snapshot.core_snapshots(), &[], now_unix_seconds) {
        UsageDecision::Allow(_) => ActionResult {
            schema_version: 1,
            action: "CheckUsage",
            status: "success",
            exit_code: 0,
            code: "hive.usage-allowed",
            message: "subscription usage permits a new automatic dispatch".to_owned(),
            changed_paths: Vec::new(),
            evidence: vec![Evidence {
                kind: "report",
                locator: "usage-snapshots:normalized".to_owned(),
                digest: evidence_digest,
            }],
            next_action: None,
        },
        UsageDecision::Block(_) => ActionResult {
            schema_version: 1,
            action: "CheckUsage",
            status: "blocked",
            exit_code: 3,
            code: "hive.usage-limited",
            message: format!(
                "subscription usage is at or below the {}% remaining threshold",
                arguments.threshold
            ),
            changed_paths: Vec::new(),
            evidence: vec![Evidence {
                kind: "report",
                locator: "usage-snapshots:normalized".to_owned(),
                digest: evidence_digest,
            }],
            next_action: None,
        },
        UsageDecision::Unknown(_) => usage_unknown_result(
            "subscription usage could not be verified safely",
            Some(evidence_digest),
        ),
    }
}

fn usage_unknown_result(message: &str, evidence_digest: Option<String>) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "CheckUsage",
        status: "blocked",
        exit_code: 3,
        code: "hive.usage-unknown",
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence: evidence_digest
            .map(|digest| {
                vec![Evidence {
                    kind: "report",
                    locator: "usage-snapshots:normalized".to_owned(),
                    digest,
                }]
            })
            .unwrap_or_default(),
        next_action: None,
    }
}

fn is_help_request(arguments: &[String]) -> bool {
    matches!(arguments, [argument] if argument == "-h" || argument == "--help")
}

fn wants_json(arguments: &[String]) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == "--output" && pair[1] == "json")
}

fn run_setup(arguments: &[String]) -> ExitCode {
    let parsed = parse_setup(arguments);
    let result = match parsed {
        Ok(arguments) => {
            let request = SetupRequest {
                target: &arguments.target,
                answers: &arguments.answers,
                capabilities: &arguments.capabilities,
                mode: arguments.mode,
                reconfigure_roles: arguments.reconfigure_roles,
            };
            match execute_setup(&request) {
                Ok(outcome) => ActionResult {
                    schema_version: 1,
                    action: "SetupHarness",
                    status: "success",
                    exit_code: 0,
                    code: match arguments.mode {
                        SetupMode::DryRun => "hive.setup-dry-run-complete",
                        SetupMode::Apply => "hive.setup-complete",
                        SetupMode::Validate => "hive.setup-valid",
                    },
                    message: match arguments.mode {
                        SetupMode::DryRun => "setup dry run completed".to_owned(),
                        SetupMode::Apply => "consumer harness setup completed".to_owned(),
                        SetupMode::Validate => "installed consumer harness is valid".to_owned(),
                    },
                    changed_paths: outcome.changed_paths,
                    evidence: vec![
                        Evidence {
                            kind: "report",
                            locator: format!("orchestration-owner:{}", outcome.resolved_owner),
                            digest: sha256_digest(outcome.resolved_owner.as_bytes()),
                        },
                        Evidence {
                            kind: "report",
                            locator: "render-tree:normalized".to_owned(),
                            digest: outcome.tree_digest,
                        },
                    ],
                    next_action: None,
                },
                Err(error) => failure_result(&error),
            }
        }
        Err(error) => failure_result(&error),
    };
    emit_json_result(&result);
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
    ExitCode::from(result.exit_code)
}

fn parse_setup(arguments: &[String]) -> Result<SetupArguments, RenderError> {
    let mut target = None;
    let mut answers = None;
    let mut capabilities = None;
    let mut mode = None;
    let mut output = None;
    let mut reconfigure_roles = BTreeSet::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--target" => target = Some(next_value(arguments, &mut index, "--target")?),
            "--answers" => answers = Some(next_value(arguments, &mut index, "--answers")?),
            "--capabilities" => {
                capabilities = Some(next_value(arguments, &mut index, "--capabilities")?);
            }
            "--reconfigure-role" => {
                reconfigure_roles.insert(next_value(arguments, &mut index, "--reconfigure-role")?);
            }
            "--output" => output = Some(next_value(arguments, &mut index, "--output")?),
            "--dry-run" => set_mode(&mut mode, SetupMode::DryRun)?,
            "--apply" => set_mode(&mut mode, SetupMode::Apply)?,
            "--validate" => set_mode(&mut mode, SetupMode::Validate)?,
            option => {
                return Err(RenderError::Input(format!(
                    "unknown setup option: {option}"
                )));
            }
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err(RenderError::Input(
            "setup requires --output json".to_owned(),
        ));
    }
    Ok(SetupArguments {
        target: PathBuf::from(required(target, "--target")?),
        answers: PathBuf::from(required(answers, "--answers")?),
        capabilities: PathBuf::from(required(capabilities, "--capabilities")?),
        mode: mode.ok_or_else(|| {
            RenderError::Input("choose exactly one of --dry-run, --apply, or --validate".to_owned())
        })?,
        reconfigure_roles,
    })
}

fn next_value(
    arguments: &[String],
    index: &mut usize,
    option: &str,
) -> Result<String, RenderError> {
    *index += 1;
    arguments
        .get(*index)
        .cloned()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| RenderError::Input(format!("missing value for {option}")))
}

fn set_mode(mode: &mut Option<SetupMode>, value: SetupMode) -> Result<(), RenderError> {
    if mode.replace(value).is_some() {
        return Err(RenderError::Input(
            "choose exactly one of --dry-run, --apply, or --validate".to_owned(),
        ));
    }
    Ok(())
}

fn required(value: Option<String>, option: &str) -> Result<String, RenderError> {
    value.ok_or_else(|| RenderError::Input(format!("missing required option {option}")))
}

fn failure_result(error: &RenderError) -> ActionResult {
    failure_result_for("SetupHarness", error, Vec::new())
}

fn failure_result_for(
    action: &'static str,
    error: &RenderError,
    changed_paths: Vec<String>,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.to_string(),
        changed_paths,
        evidence: Vec::new(),
        next_action: None,
    }
}

fn emit_json_result(result: &ActionResult) {
    match serde_json::to_string(result) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            println!(
                "{{\"schema_version\":1,\"action\":\"SetupHarness\",\"status\":\"error\",\"exit_code\":10,\"code\":\"hive.internal-error\",\"message\":\"JSON serialization failed\",\"changed_paths\":[],\"evidence\":[],\"next_action\":null}}"
            );
            eprintln!("error: {error}");
        }
    }
}

fn run_hook(arguments: &[String]) -> ExitCode {
    let stop_requested = requested_event(arguments).as_deref() == Some("Stop");
    if stop_requested {
        println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
        return ExitCode::SUCCESS;
    }
    match parse_hook(arguments) {
        Ok((capability, event, input)) => {
            let target = match env::current_dir() {
                Ok(target) => target,
                Err(error) => {
                    eprintln!("diagnostic: inactive fallback hook: {error}");
                    println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
                    return ExitCode::SUCCESS;
                }
            };
            let authorization = authorize_hook(&target, &capability, &event);
            match authorization {
                Ok(HookAuthorization::Authorized) => {
                    match read_hook_input(input.as_deref(), &event) {
                        Ok(input) => match execute_hook_capability(&target, &capability, &input) {
                            Ok(result) => {
                                let exit_code = if result.decision == "block" { 3 } else { 0 };
                                emit_hook_result(&result);
                                ExitCode::from(exit_code)
                            }
                            Err(error) => {
                                eprintln!("diagnostic: inactive fallback hook: {error}");
                                println!(
                                "{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}"
                            );
                                ExitCode::SUCCESS
                            }
                        },
                        Err(error) => {
                            eprintln!("diagnostic: inactive fallback hook: {error}");
                            println!(
                                "{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}"
                            );
                            ExitCode::SUCCESS
                        }
                    }
                }
                Ok(HookAuthorization::Inert) => {
                    println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("diagnostic: inactive fallback hook: {error}");
                    println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
                    ExitCode::SUCCESS
                }
            }
        }
        Err(message) => {
            eprintln!("diagnostic: inactive fallback hook: {message}");
            println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
            ExitCode::SUCCESS
        }
    }
}

fn read_hook_input(path: Option<&Path>, event: &str) -> Result<HookInput, RenderError> {
    let mut bytes = Vec::new();
    match path {
        Some(path) => {
            bytes = std::fs::read(path)
                .map_err(|error| RenderError::Input(format!("cannot read hook input: {error}")))?;
        }
        None => {
            io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|error| RenderError::Input(format!("cannot read hook input: {error}")))?;
        }
    }
    let input: HookInput = serde_json::from_slice(&bytes)
        .map_err(|error| RenderError::Input(format!("invalid hook input: {error}")))?;
    if input.schema_version != 1 {
        return Err(RenderError::Input(format!(
            "unsupported hook input schema version: {}",
            input.schema_version
        )));
    }
    if input.event != event {
        return Err(RenderError::Input(format!(
            "hook input event {} does not match requested event {event}",
            input.event
        )));
    }
    Ok(input)
}

fn execute_hook_capability(
    target: &Path,
    capability: &str,
    input: &HookInput,
) -> Result<HookResult, RenderError> {
    match capability {
        "protect-hive-owned-state" => protect_hive_owned_state(target, input),
        "update-integrity-guard" => update_integrity_guard(input),
        "derived-state-invalidation" => derived_state_invalidation(target, input),
        "checkpoint-reminder" => checkpoint_reminder(target, input),
        _ => Err(RenderError::Unsupported(format!(
            "fallback hook capability is unsupported: {capability}"
        ))),
    }
}

fn protect_hive_owned_state(target: &Path, input: &HookInput) -> Result<HookResult, RenderError> {
    if input.event != "PreToolUse" {
        return Err(RenderError::Unsupported(
            "protect-hive-owned-state requires PreToolUse".to_owned(),
        ));
    }
    let operation = input
        .operation
        .as_deref()
        .ok_or_else(|| RenderError::Input("hook input is missing operation".to_owned()))?;
    let path = input
        .path
        .as_deref()
        .ok_or_else(|| RenderError::Input("hook input is missing path".to_owned()))?;
    let relative_path = normalize_hook_path(target, path)?;
    let destructive = matches!(
        operation.to_ascii_lowercase().as_str(),
        "delete"
            | "remove"
            | "unlink"
            | "rmdir"
            | "truncate"
            | "overwrite"
            | "rename"
            | "move"
            | "chmod"
    );
    let protected = relative_path.as_deref().is_some_and(is_protected_hive_path);
    if destructive && protected {
        return Ok(HookResult {
            schema_version: 1,
            decision: "block",
            active: true,
            code: "hive.hook-protected-state-mutation",
            message: format!(
                "refused {operation} of protected Hive path {path}; use a Hive-owned operation"
            ),
        });
    }
    Ok(HookResult {
        schema_version: 1,
        decision: "allow",
        active: true,
        code: "hive.hook-owned-state-checked",
        message: format!(
            "checked {} operation {operation} for protected Hive ownership",
            input.tool.as_deref().unwrap_or("tool")
        ),
    })
}

fn normalize_hook_path(target: &Path, value: &str) -> Result<Option<PathBuf>, RenderError> {
    let path = PathBuf::from(value);
    #[cfg(windows)]
    let relative = if path.is_absolute() {
        match windows_target_relative(target, &path) {
            Some(relative) => relative,
            None => return Ok(None),
        }
    } else {
        path
    };
    #[cfg(not(windows))]
    let relative = if path.is_absolute() {
        match path.strip_prefix(target) {
            Ok(relative) => relative.to_path_buf(),
            Err(_) => return Ok(None),
        }
    } else {
        path
    };
    validate_project_relative(&relative)
        .map_err(|error| RenderError::Input(format!("unsafe hook input path {value}: {error}")))?;
    Ok(Some(relative))
}

#[cfg(windows)]
fn windows_target_relative(target: &Path, path: &Path) -> Option<PathBuf> {
    let target_text = windows_portable_path(target)?;
    let path_text = windows_portable_path(path)?;
    if let Some(relative) = windows_portable_relative(&target_text, &path_text) {
        return Some(relative);
    }
    let canonical_target = windows_portable_path(&target.canonicalize().ok()?)?;
    if let Some(relative) = windows_portable_relative(&canonical_target, &path_text) {
        return Some(relative);
    }
    let canonical_path = windows_portable_path(&path.canonicalize().ok()?)?;
    windows_portable_relative(&canonical_target, &canonical_path)
}

#[cfg(windows)]
fn windows_portable_relative(target: &str, path: &str) -> Option<PathBuf> {
    let target = target.trim_end_matches('/');
    let prefix = path.get(..target.len())?;
    if !prefix.eq_ignore_ascii_case(target) {
        return None;
    }
    let relative = path.get(target.len()..)?.strip_prefix('/')?;
    if relative.is_empty() {
        return None;
    }
    Some(PathBuf::from(relative))
}

#[cfg(windows)]
fn windows_portable_path(path: &Path) -> Option<String> {
    let mut value = path.to_str()?.replace('\\', "/");
    if value
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("//?/UNC/"))
    {
        value.replace_range(..8, "//");
    } else if value
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("//?/"))
    {
        value.replace_range(..4, "");
    }
    Some(value)
}

fn is_protected_hive_path(path: &Path) -> bool {
    [
        ".hive/.gitignore",
        ".hive/LICENSE-AIGENT-HIVE.txt",
        ".hive/README.md",
        ".hive/setup-answers.yml",
    ]
    .iter()
    .any(|owned| path == Path::new(owned))
        || [
            ".hive/config",
            ".hive/hooks",
            ".hive/knowledge",
            ".hive/team",
            ".hive/runs",
        ]
        .iter()
        .any(|prefix| path == Path::new(prefix) || path.starts_with(prefix))
}

fn update_integrity_guard(input: &HookInput) -> Result<HookResult, RenderError> {
    if input.event != "PreToolUse" {
        return Err(RenderError::Unsupported(
            "update-integrity-guard requires PreToolUse".to_owned(),
        ));
    }
    let action = input
        .action
        .as_deref()
        .ok_or_else(|| RenderError::Input("hook input is missing action".to_owned()))?;
    if !matches!(action, "update" | "migrate") {
        return Err(RenderError::Input(format!(
            "update-integrity-guard does not accept action {action}"
        )));
    }
    let gates_passed = input.dry_run == Some(true)
        && input.backup_present == Some(true)
        && input.staging_validated == Some(true);
    Ok(HookResult {
        schema_version: 1,
        decision: if gates_passed { "allow" } else { "block" },
        active: true,
        code: if gates_passed {
            "hive.hook-update-gates-verified"
        } else {
            "hive.hook-update-gate-missing"
        },
        message: if gates_passed {
            format!("{action} integrity gates verified")
        } else {
            format!("{action} requires dry-run, backup, and validated staging")
        },
    })
}

fn derived_state_invalidation(target: &Path, input: &HookInput) -> Result<HookResult, RenderError> {
    if input.event != "PostToolUse" {
        return Err(RenderError::Unsupported(
            "derived-state-invalidation requires PostToolUse".to_owned(),
        ));
    }
    let path = input
        .path
        .as_deref()
        .ok_or_else(|| RenderError::Input("hook input is missing path".to_owned()))?;
    let relative_path = normalize_hook_path(target, path)?;
    if !relative_path.as_deref().is_some_and(is_canonical_hive_path) {
        return Ok(HookResult {
            schema_version: 1,
            decision: "allow",
            active: true,
            code: "hive.hook-derived-state-current",
            message: "changed path is not canonical Hive data".to_owned(),
        });
    }
    mark_derived_state_stale(target)?;
    Ok(HookResult {
        schema_version: 1,
        decision: "allow",
        active: true,
        code: "hive.hook-derived-state-invalidated",
        message: format!("marked derived state stale after canonical change at {path}"),
    })
}

fn mark_derived_state_stale(target: &Path) -> Result<(), RenderError> {
    let marker_relative = Path::new(".hive/index/.stale");
    ensure_no_symlink_ancestors(target, marker_relative)
        .map_err(|error| RenderError::Safety(error.to_string()))?;
    let index_directory = target.join(".hive/index");
    std::fs::create_dir_all(index_directory).map_err(|error| {
        RenderError::Internal(format!("cannot create derived index path: {error}"))
    })?;
    let marker = target.join(marker_relative);
    if std::fs::read(&marker).ok().as_deref() != Some(STALE_MARKER) {
        std::fs::write(&marker, STALE_MARKER).map_err(|error| {
            RenderError::Internal(format!("cannot mark derived state stale: {error}"))
        })?;
    }
    Ok(())
}

fn is_canonical_hive_path(path: &Path) -> bool {
    [".hive/knowledge", ".hive/team", ".hive/runs"]
        .iter()
        .any(|prefix| path == Path::new(prefix) || path.starts_with(prefix))
}

fn checkpoint_reminder(target: &Path, input: &HookInput) -> Result<HookResult, RenderError> {
    if input.event != "PreCompact" {
        return Err(RenderError::Unsupported(
            "checkpoint-reminder is active only for PreCompact; Stop is always neutral".to_owned(),
        ));
    }
    let status_path = input
        .status_path
        .as_deref()
        .ok_or_else(|| RenderError::Input("hook input is missing status_path".to_owned()))?;
    let relative_status_path = normalize_hook_path(target, status_path)?;
    if !relative_status_path
        .as_deref()
        .is_some_and(|path| path.starts_with(".hive/runs") && path.ends_with("STATUS.md"))
    {
        return Err(RenderError::Input(format!(
            "checkpoint status path is outside .hive/runs: {status_path}"
        )));
    }
    let checkpoint_present = input
        .checkpoint_present
        .ok_or_else(|| RenderError::Input("hook input is missing checkpoint_present".to_owned()))?;
    Ok(HookResult {
        schema_version: 1,
        decision: "allow",
        active: true,
        code: if checkpoint_present {
            "hive.hook-checkpoint-present"
        } else {
            "hive.hook-checkpoint-missing"
        },
        message: if checkpoint_present {
            format!("durable checkpoint is present at {status_path}")
        } else {
            format!("durable checkpoint is missing at {status_path}")
        },
    })
}

fn emit_hook_result(result: &HookResult) {
    match serde_json::to_string(result) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("diagnostic: inactive fallback hook: {error}");
            println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
        }
    }
}

fn requested_event(arguments: &[String]) -> Option<String> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--event")
        .map(|pair| pair[1].clone())
}

fn parse_hook(arguments: &[String]) -> Result<(String, String, Option<PathBuf>), String> {
    let mut capability = None;
    let mut event = None;
    let mut input = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        let value = arguments
            .get(index)
            .cloned()
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--capability" => capability = Some(value),
            "--event" => event = Some(value),
            "--input" => input = Some(PathBuf::from(value)),
            "--output" => output = Some(value),
            _ => return Err(format!("unknown hook option: {option}")),
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err("hook requires --output json".to_owned());
    }
    Ok((
        capability.ok_or_else(|| "missing --capability".to_owned())?,
        event.ok_or_else(|| "missing --event".to_owned())?,
        input,
    ))
}

fn run_human(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    match arguments.next().as_deref() {
        Some("doctor") => {
            reject_extra_arguments(arguments)?;
            doctor()
        }
        Some("check-target") => {
            let target = arguments
                .next()
                .ok_or_else(|| format!("missing target path\n\n{USAGE}"))?;
            reject_extra_arguments(arguments)?;
            check_target(Path::new(&target))
        }
        Some("-V" | "--version") => {
            println!("hive {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("-h" | "--help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(command) => Err(format!("unknown command: {command}\n\n{USAGE}")),
    }
}

fn reject_extra_arguments(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    if let Some(argument) = arguments.next() {
        return Err(format!("unexpected argument: {argument}\n\n{USAGE}"));
    }
    Ok(())
}

fn doctor() -> Result<(), String> {
    let current_directory =
        env::current_dir().map_err(|error| format!("cannot read current directory: {error}"))?;
    let marker = source_marker_path(&current_directory);

    println!("aigent-hive {}", env!("CARGO_PKG_VERSION"));
    println!("workspace: {}", current_directory.display());
    println!(
        "source workspace: {}",
        if marker.is_file() { "yes" } else { "no" }
    );
    println!("model API client: disabled by architecture");
    println!("setup renderer: available");
    Ok(())
}

fn check_target(target: &Path) -> Result<(), String> {
    ensure_consumer_target(target).map_err(|error| error.to_string())?;
    println!("target accepted: {}", target.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        execute_hook_capability, failure_result_for, is_help_request, mark_derived_state_stale,
        normalize_hook_path, parse_hook, parse_setup, run_human, wants_json, ActionResult,
        HookInput, SETUP_USAGE,
    };
    use hive_render::RenderError;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "aigent-hive-cli-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn help_is_the_default_command() {
        assert_eq!(run_human(std::iter::empty()), Ok(()));
    }

    #[test]
    fn setup_help_has_the_exact_supported_invocation() {
        assert!(is_help_request(&["--help".to_owned()]));
        assert!(SETUP_USAGE.contains(
            "hive setup --target <dir> --answers <yml> --capabilities <json> \
             (--dry-run|--apply|--validate)"
        ));
        assert!(SETUP_USAGE.contains("--output json"));
    }

    #[test]
    fn hook_preview_command_uses_json_stdin_when_input_path_is_omitted() {
        let arguments = [
            "--capability".to_owned(),
            "protect-hive-owned-state".to_owned(),
            "--event".to_owned(),
            "PreToolUse".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let (_, _, input) = parse_hook(&arguments).expect("preview command should parse");
        assert!(input.is_none());
    }

    #[test]
    fn json_output_is_detected_for_unknown_top_level_actions() {
        assert!(wants_json(&[
            "unknown-action".to_owned(),
            "--output".to_owned(),
            "json".to_owned()
        ]));
    }

    #[test]
    fn unknown_action_result_uses_the_explicit_failure_sentinel() {
        let result = ActionResult {
            schema_version: 1,
            action: "UnknownAction",
            status: "error",
            exit_code: 2,
            code: "hive.unknown-action",
            message: "unknown top-level action: fixture".to_owned(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
        };
        let json = serde_json::to_value(result).expect("result should serialize");
        assert_eq!(json["action"], "UnknownAction");
        assert_eq!(json["exit_code"], 2);
        assert_eq!(json["changed_paths"], serde_json::json!([]));
    }

    #[test]
    fn unknown_commands_fail() {
        let error = run_human(["unknown".to_owned()].into_iter()).expect_err("command should fail");
        assert!(error.contains("unknown command"));
    }

    #[test]
    fn conflicting_setup_modes_fail() {
        let arguments = vec![
            "--target".to_owned(),
            "target".to_owned(),
            "--answers".to_owned(),
            "answers".to_owned(),
            "--capabilities".to_owned(),
            "capabilities".to_owned(),
            "--dry-run".to_owned(),
            "--apply".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert!(parse_setup(&arguments).is_err());
    }

    #[test]
    fn action_result_helper_covers_every_stable_failure_exit() {
        let cases = [
            (RenderError::Input("input".to_owned()), 2, "error"),
            (RenderError::Safety("safety".to_owned()), 3, "blocked"),
            (RenderError::Conflict("conflict".to_owned()), 3, "conflict"),
            (
                RenderError::Unsupported("unsupported".to_owned()),
                4,
                "unsupported",
            ),
            (
                RenderError::Verification("verification".to_owned()),
                5,
                "verification-failed",
            ),
            (RenderError::Internal("internal".to_owned()), 10, "error"),
            (RenderError::Rollback("rollback".to_owned()), 10, "error"),
        ];
        for (error, exit_code, status) in cases {
            let result = failure_result_for("SetupHarness", &error, Vec::new());
            assert_eq!(result.exit_code, exit_code);
            assert_eq!(result.status, status);
            let json = serde_json::to_value(&result).expect("result should serialize");
            assert_eq!(json["action"], "SetupHarness");
            assert_eq!(json["changed_paths"], serde_json::json!([]));
        }
    }

    #[test]
    fn protected_state_hook_blocks_destructive_mutation() {
        let input = HookInput {
            schema_version: 1,
            event: "PreToolUse".to_owned(),
            tool: Some("filesystem-write".to_owned()),
            operation: Some("delete".to_owned()),
            path: Some(".hive/config/harness.toml".to_owned()),
            action: None,
            dry_run: None,
            backup_present: None,
            staging_validated: None,
            checkpoint_present: None,
            status_path: None,
        };
        let result =
            execute_hook_capability(Path::new("/consumer"), "protect-hive-owned-state", &input)
                .expect("approved hook should execute");
        assert!(result.active);
        assert_eq!(result.decision, "block");
        assert_eq!(result.code, "hive.hook-protected-state-mutation");
    }

    #[test]
    fn protected_state_hook_allows_non_destructive_read() {
        let input = HookInput {
            schema_version: 1,
            event: "PreToolUse".to_owned(),
            tool: Some("filesystem-read".to_owned()),
            operation: Some("read".to_owned()),
            path: Some(".hive/config/harness.toml".to_owned()),
            action: None,
            dry_run: None,
            backup_present: None,
            staging_validated: None,
            checkpoint_present: None,
            status_path: None,
        };
        let result =
            execute_hook_capability(Path::new("/consumer"), "protect-hive-owned-state", &input)
                .expect("approved hook should execute");
        assert!(result.active);
        assert_eq!(result.decision, "allow");
    }

    #[test]
    fn update_integrity_hook_blocks_missing_safety_gate() {
        let input = HookInput {
            schema_version: 1,
            event: "PreToolUse".to_owned(),
            tool: None,
            operation: None,
            path: None,
            action: Some("update".to_owned()),
            dry_run: Some(true),
            backup_present: Some(false),
            staging_validated: Some(true),
            checkpoint_present: None,
            status_path: None,
        };
        let result =
            execute_hook_capability(Path::new("/consumer"), "update-integrity-guard", &input)
                .expect("approved hook should execute");
        assert!(result.active);
        assert_eq!(result.decision, "block");
        assert_eq!(result.code, "hive.hook-update-gate-missing");
    }

    #[test]
    fn derived_state_marker_is_idempotent() {
        let target = temporary_directory("derived-state");
        fs::create_dir_all(&target).expect("temporary target should exist");

        mark_derived_state_stale(&target).expect("first invalidation should succeed");
        let marker = target.join(".hive/index/.stale");
        let first = fs::read(&marker).expect("stale marker should exist");
        mark_derived_state_stale(&target).expect("second invalidation should succeed");
        let second = fs::read(&marker).expect("stale marker should remain");

        fs::remove_dir_all(&target).expect("temporary target should be removed");
        assert_eq!(first, second);
        assert_eq!(first, b"{\"schema_version\":1,\"stale\":true}\n");
    }

    #[test]
    fn precompact_checkpoint_hook_reports_missing_checkpoint_without_blocking() {
        let input = HookInput {
            schema_version: 1,
            event: "PreCompact".to_owned(),
            tool: None,
            operation: None,
            path: None,
            action: None,
            dry_run: None,
            backup_present: None,
            staging_validated: None,
            checkpoint_present: Some(false),
            status_path: Some(".hive/runs/run/STATUS.md".to_owned()),
        };
        let result = execute_hook_capability(Path::new("/consumer"), "checkpoint-reminder", &input)
            .expect("approved hook should execute");
        assert!(result.active);
        assert_eq!(result.decision, "allow");
        assert_eq!(result.code, "hive.hook-checkpoint-missing");
    }

    #[test]
    fn hook_path_normalization_accepts_target_relative_and_inside_absolute_paths() {
        let target = temporary_directory("path-target");
        let inside = target.join(".hive/config/harness.toml");
        assert_eq!(
            normalize_hook_path(&target, ".hive/config/harness.toml")
                .expect("relative path should validate"),
            Some(PathBuf::from(".hive/config/harness.toml"))
        );
        assert_eq!(
            normalize_hook_path(
                &target,
                inside.to_str().expect("temporary path should be UTF-8")
            )
            .expect("inside absolute path should validate"),
            Some(PathBuf::from(".hive/config/harness.toml"))
        );
    }

    #[test]
    fn hook_path_normalization_ignores_outside_absolute_paths() {
        let target = temporary_directory("path-target");
        let outside = temporary_directory("path-outside").join(".hive/config/harness.toml");
        assert_eq!(
            normalize_hook_path(
                &target,
                outside.to_str().expect("temporary path should be UTF-8")
            )
            .expect("outside absolute path should be classified"),
            None
        );
    }

    #[test]
    fn hook_path_normalization_rejects_traversal_and_foreign_syntax() {
        let target = Path::new("/consumer");
        for path in ["../.hive/config/harness.toml", ".hive/../outside"] {
            assert!(
                normalize_hook_path(target, path).is_err(),
                "{path} should be rejected"
            );
        }

        #[cfg(not(windows))]
        assert!(
            normalize_hook_path(target, r"C:\consumer\.hive\config\harness.toml").is_err(),
            "foreign Windows syntax should be rejected"
        );

        #[cfg(windows)]
        assert_eq!(
            normalize_hook_path(
                Path::new(r"D:\consumer"),
                r"C:\consumer\.hive\config\harness.toml"
            )
            .expect("outside native absolute path should be classified"),
            None
        );
    }

    #[cfg(windows)]
    #[test]
    fn hook_path_normalization_matches_verbatim_current_directory() {
        let normalized = normalize_hook_path(
            Path::new(r"\\?\C:\Users\RUNNER~1\Consumer"),
            r"c:\users\runner~1\consumer\.hive\config\harness.toml",
        )
        .expect("inside absolute path should validate")
        .expect("inside absolute path should be classified");
        assert!(super::is_protected_hive_path(&normalized));
    }
}
