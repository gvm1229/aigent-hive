use hive_core::usage_guard::{evaluate_usage, UsageDecision, UsagePolicy};
use hive_core::{
    ensure_consumer_target, ensure_no_symlink_ancestors, sha256_digest, source_marker_path,
    validate_project_relative,
};
use hive_projection::{
    prompt_refinement_lifecycle, resolve_route, validate_prompt_refinement, LogicalAction,
    PromptRefinementInput, PromptRefinementResult, PromptRefinementState, Route, RoutingRequest,
};
use hive_render::{
    authorize_hook_with_resolution, execute_setup, execute_setup_with_post_apply,
    HookAuthorization, RenderError, SetupMode, SetupOutcome, SetupRequest,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

mod custom_agent_cli;
mod discord;
mod judge;
mod knowledge;
mod knowledge_scan;
mod loop_engineering;
mod orchestration;
mod project_upgrade;
mod report;
mod role;
mod run;
mod session;
mod source_wiki;
mod update;
mod update_activation;
mod update_discovery;
mod usage;
mod usage_control;
mod usage_install;
mod user_install;
mod user_setup;

const USAGE: &str = "\
Aigent Hive

USAGE:
    hive doctor
    hive install --scope user (--host codex|claude|antigravity)...|--hosts <comma-separated-hosts> (--dry-run|--apply|--validate) [--user-root <dir>] --output json
    hive uninstall [--user-root <dir>] [--output json]
    hive check-target <path>
    hive setup --help
    hive setup --target <dir> --answers <yml> --capabilities <json> --user-root <dir> (--dry-run|--apply|--validate) [--reconfigure-role <role-id>]... --output json
    hive source-wiki lint --target <source-root> --output json
    hive source-wiki index --target <source-root> --output json
    hive source-wiki query --target <source-root> --language en|ko (--text <query>|--tag <tag>) [--limit <1..100>] --output json
    hive update
    hive update --check --user-root <absolute-dir> --output json
    hive knowledge add|authorize-confidential|collection|delete|export|import|ingest|lint|list|promote|query|read|refresh|remember|retrieve|scan|suppress --help
    hive discord inbound --host codex|claude|antigravity --output json
    hive report preview|collect|export --help
    hive project upgrade --target <dir> (--scan|--dry-run|--apply|--validate|--recover) --output json
    hive session begin|check|update --target <dir> --host codex|claude|antigravity --session-id <id> --process-id <positive-u32> --path <project-relative-path> [--path <project-relative-path>]... --output json
    hive session close --target <dir> --host codex|claude|antigravity --session-id <id> --output json
    hive session recover --target <dir> --output json
    hive index rebuild --target <dir> --output json
    hive route --request <json> --output json
    hive prompt validate --request <input.json> --result <result.json> --output json
    hive prompt approve --request <input.json> --result <result.json> --digest <sha256:...> --target-host codex|claude|antigravity --confirm-refined-prompt --output json
    hive hook --capability <name> --event <event> [--capabilities <fresh-json>] [--input <json>] --output json
    hive usage check --account-digest <sha256:...> [--threshold <1..99>] --output json
    hive usage probe-native --host codex|claude|antigravity --output json
    hive usage enforce --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--account-digest <sha256:...>] [--user-root <dir>] --output json
    hive usage status --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--user-root <dir>] --output json
    hive usage threshold (--target <configured-project>|--user-root <user-root>) --remaining-percent <1..99> --output json
    hive usage session --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--user-root <dir>] --action enable|disable|toggle [--confirm-session-disable] --output json
    hive usage capture --host claude (--target <dir>|--target-from-stdin) --stdin-json --output json
    hive usage fallback-install --host codex|claude|antigravity (--dry-run|--apply) [--confirm-install] --output json
    hive role validate --target <dir> --role <role-id> --output json
    hive role handoff --target <dir> --request <request.json> --output json
    hive run checkpoint --target <dir> --request <request.json> --capabilities <fresh-json> --output json
    hive run resume --target <dir> --run <run-id> --capabilities <fresh-json> [--dispatch-intent manual|automatic] [--account-digest <sha256:...>] [--session-id <host-session-id>] [--role <role-id> [--threshold <1..99>]] --output json
    hive loop initialize|validate|checkpoint|steer|prepare|recover --help
    hive orchestration status|plan|dispatch|receipt|cancel|recover|authority|migrate --help
    hive judge package --target <dir> --request <json> --output json
    hive judge quorum --target <dir> --request <json> --output json
    hive release verify --bundle <release-dir> --output json
    hive update --help
";

const SETUP_USAGE: &str = "\
Create or validate an installed Aigent Hive consumer harness.

USAGE:
    hive setup --target <dir> --answers <yml> --capabilities <json> --user-root <dir> (--dry-run|--apply|--validate) [--reconfigure-role <role-id>]... --output json

MODES:
    --dry-run    Render and validate in staging without changing the target
    --apply      Render, validate, and apply only manifest-owned changes
    --validate   Validate the installed harness without changing the target
";

const STALE_MARKER: &[u8] = b"{\"schema_version\":1,\"stale\":true}\n";
const ROUTING_REQUEST_SCHEMA: &str = include_str!("../../../schemas/routing-request.schema.json");
const PROMPT_REFINEMENT_INPUT_SCHEMA: &str =
    include_str!("../../../schemas/prompt-refinement-input.schema.json");
const PROMPT_REFINEMENT_RESULT_SCHEMA: &str =
    include_str!("../../../schemas/prompt-refinement-result.schema.json");
const MAX_CONTRACT_BYTES: u64 = 1024 * 1024;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
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
    user_root: PathBuf,
    mode: SetupMode,
    reconfigure_roles: BTreeSet<String>,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("setup")
            if arguments.iter().any(|argument| argument == "--scope")
                && arguments.iter().any(|argument| argument == "--help") =>
        {
            user_setup::print_help();
            ExitCode::SUCCESS
        }
        Some("setup") if is_help_request(&arguments[1..]) => {
            print!("{SETUP_USAGE}");
            ExitCode::SUCCESS
        }
        Some("setup") if arguments.iter().any(|argument| argument == "--scope") => {
            user_setup::run(&arguments[1..])
        }
        Some("setup") => run_setup(&arguments[1..]),
        Some("install") => user_install::run_install(&arguments[1..]),
        Some("uninstall") => user_install::run_uninstall(&arguments[1..]),
        Some("source-wiki") => source_wiki::run(&arguments[1..]),
        Some("knowledge") => knowledge::run_knowledge(&arguments[1..]),
        Some("discord") => discord::run(&arguments[1..]),
        Some("report") => report::run(&arguments[1..]),
        Some("project") => project_upgrade::run(&arguments[1..]),
        Some("session") => session::run(&arguments[1..]),
        Some("index") => knowledge::run_index(&arguments[1..]),
        Some("route") => run_route(&arguments[1..]),
        Some("prompt") => run_prompt(&arguments[1..]),
        Some("hook") => run_hook(&arguments[1..]),
        Some("usage") => run_usage(&arguments[1..]),
        Some("role") => role::run_role(&arguments[1..]),
        Some("agent") => custom_agent_cli::run(&arguments[1..]),
        Some("run") => run::run_run(&arguments[1..]),
        Some("loop") => loop_engineering::run_loop(&arguments[1..]),
        Some("orchestration") => orchestration::run(&arguments[1..]),
        Some("judge") => judge::run_judge(&arguments[1..]),
        Some("release") => update::run_release(&arguments[1..]),
        Some("update") if arguments.len() == 1 => update_activation::run(),
        Some("update") if arguments.get(1).map(String::as_str) == Some("--check") => {
            update_discovery::run(&arguments[1..])
        }
        Some("update") if arguments.iter().any(|argument| argument == "--scope") => {
            user_install::run_update(&arguments[1..])
        }
        Some("update") => update::run_update(&arguments[1..]),
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
                data: None,
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

enum ParsedUsageArguments {
    Check(UsageArguments),
    ProbeNative(usage::UsageHost),
}

fn run_usage(arguments: &[String]) -> ExitCode {
    if matches!(
        arguments.first().map(String::as_str),
        Some("enforce" | "status" | "threshold" | "session" | "capture")
    ) {
        return usage_control::run_usage_control(arguments);
    }
    if arguments.first().map(String::as_str) == Some("fallback-install") {
        return usage_install::run(arguments);
    }
    let result = match parse_usage(arguments) {
        Ok(ParsedUsageArguments::Check(arguments)) => check_usage(&arguments),
        Ok(ParsedUsageArguments::ProbeNative(host)) => probe_native_usage(host),
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
            data: None,
        },
    };
    emit_json_result(&result);
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
    ExitCode::from(result.exit_code)
}

fn parse_usage(arguments: &[String]) -> Result<ParsedUsageArguments, String> {
    if arguments.first().map(String::as_str) == Some("probe-native") {
        let mut host = None;
        let mut output = None;
        let mut index = 1;
        while index < arguments.len() {
            let option = arguments[index].as_str();
            let value = arguments
                .get(index + 1)
                .ok_or_else(|| format!("missing value for {option}"))?;
            match option {
                "--host" if host.is_none() => host = Some(value.clone()),
                "--output" if output.is_none() => output = Some(value.clone()),
                "--host" | "--output" => {
                    return Err(format!("duplicate usage option: {option}"));
                }
                _ => return Err(format!("unknown usage option: {option}")),
            }
            index += 2;
        }
        if output.as_deref() != Some("json") {
            return Err("usage probe-native requires --output json".to_owned());
        }
        let host = match host.as_deref() {
            Some("codex") => usage::UsageHost::Codex,
            Some("claude") => usage::UsageHost::Claude,
            Some("antigravity") => usage::UsageHost::Antigravity,
            Some(_) => {
                return Err(
                    "usage probe-native --host must be codex, claude, or antigravity".to_owned(),
                );
            }
            None => return Err("missing required option --host".to_owned()),
        };
        return Ok(ParsedUsageArguments::ProbeNative(host));
    }
    if arguments.first().map(String::as_str) != Some("check") {
        return Err("usage requires the check or probe-native action".to_owned());
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
    Ok(ParsedUsageArguments::Check(UsageArguments {
        account_digest,
        threshold,
    }))
}

fn probe_native_usage(host: usage::UsageHost) -> ActionResult {
    if host != usage::UsageHost::Codex {
        return ActionResult {
            schema_version: 1,
            action: "ProbeNativeUsage",
            status: "success",
            exit_code: 0,
            code: "hive.usage-native-probe-deferred",
            message: format!(
                "the {} native usage sensor is checked only from an active host session",
                host.as_str()
            ),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
            data: Some(serde_json::json!({ "host": host.as_str(), "probe": "deferred" })),
        };
    }
    let sampled_at = SystemTime::now();
    match usage::NativeUsageRunner::read_codex_native(&usage::SystemCommandRunner, None, sampled_at)
    {
        Ok(snapshot) => ActionResult {
            schema_version: 1,
            action: "ProbeNativeUsage",
            status: "success",
            exit_code: 0,
            code: "hive.usage-native-available",
            message: "the Codex native usage sensor is available".to_owned(),
            changed_paths: Vec::new(),
            evidence: vec![Evidence {
                kind: "report",
                locator: "usage-snapshots:native".to_owned(),
                digest: snapshot.evidence_digest(),
            }],
            next_action: None,
            data: Some(serde_json::json!({ "host": host.as_str(), "probe": "available" })),
        },
        Err(error) if error.allows_native_fallback() => ActionResult {
            schema_version: 1,
            action: "ProbeNativeUsage",
            status: "blocked",
            exit_code: 3,
            code: "hive.usage-native-fallback-eligible",
            message: error.to_string(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: Some(usage::fallback_install_next_action(host)),
            data: Some(serde_json::json!({ "host": host.as_str(), "probe": "fallback-eligible" })),
        },
        Err(error) => ActionResult {
            schema_version: 1,
            action: "ProbeNativeUsage",
            status: "blocked",
            exit_code: 3,
            code: "hive.usage-native-failed-closed",
            message: error.to_string(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
            data: Some(serde_json::json!({ "host": host.as_str(), "probe": "failed-closed" })),
        },
    }
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
    let snapshot = match usage::check_preferred_with_runners(
        &usage::SystemCommandRunner,
        &usage::SystemCommandRunner,
        &arguments.account_digest,
        now,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let next_action = match error {
                usage::SensorError::FallbackRequired(host) => {
                    Some(usage::fallback_install_next_action(host))
                }
                _ => None,
            };
            return usage_unknown_result_with_next(&error.to_string(), None, next_action);
        }
    };
    let evidence_digest = snapshot.evidence_digest();
    let Ok(policy) = UsagePolicy::new(
        &snapshot.sensor_id,
        &snapshot.sensor_version,
        &snapshot.provider,
        &snapshot.account_digest,
    )
    .with_stop_remaining_percent(arguments.threshold) else {
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
            data: None,
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
            data: None,
        },
        UsageDecision::Unknown(_) => usage_unknown_result(
            "subscription usage could not be verified safely",
            Some(evidence_digest),
        ),
    }
}

fn usage_unknown_result(message: &str, evidence_digest: Option<String>) -> ActionResult {
    usage_unknown_result_with_next(message, evidence_digest, None)
}

fn usage_unknown_result_with_next(
    message: &str,
    evidence_digest: Option<String>,
    next_action: Option<String>,
) -> ActionResult {
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
        next_action,
        data: None,
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
            let global_preferences =
                user_setup::project_preferences(&arguments.user_root).map_err(RenderError::Input);
            let global_preferences = match global_preferences {
                Ok(preferences) => preferences,
                Err(error) => return emit_setup_result(&failure_result(&error)),
            };
            let global_wiki_enabled = global_preferences.wiki_enabled;
            let setup_mode = arguments.mode;
            let user_root = arguments.user_root.clone();
            let target = arguments.target.clone();
            let request = SetupRequest {
                target: &arguments.target,
                answers: &arguments.answers,
                capabilities: &arguments.capabilities,
                mode: setup_mode,
                reconfigure_roles: arguments.reconfigure_roles,
                global_preferences: Some(global_preferences),
            };
            match execute_setup_and_registry(&request, &user_root, &target, global_wiki_enabled) {
                Ok(outcome) => ActionResult {
                    schema_version: 1,
                    action: "SetupHarness",
                    status: "success",
                    exit_code: 0,
                    code: match setup_mode {
                        SetupMode::DryRun => "hive.setup-dry-run-complete",
                        SetupMode::Apply => "hive.setup-complete",
                        SetupMode::Validate => "hive.setup-valid",
                    },
                    message: match setup_mode {
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
                    data: None,
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

fn execute_setup_and_registry(
    request: &SetupRequest<'_>,
    user_root: &Path,
    target: &Path,
    global_wiki_enabled: bool,
) -> Result<SetupOutcome, RenderError> {
    if request.mode != SetupMode::Apply {
        let mut outcome = execute_setup(request)?;
        if let Some(preferences) = outcome.effective_preferences.as_ref() {
            reconcile_project_registry(
                user_root,
                target,
                preferences,
                request.mode,
                global_wiki_enabled,
                &mut outcome.changed_paths,
            )?;
        }
        return Ok(outcome);
    }

    let preview = execute_setup(&SetupRequest {
        target: request.target,
        answers: request.answers,
        capabilities: request.capabilities,
        mode: SetupMode::DryRun,
        reconfigure_roles: request.reconfigure_roles.clone(),
        global_preferences: request.global_preferences.clone(),
    })?;
    let preferences = preview.effective_preferences.ok_or_else(|| {
        RenderError::Verification(
            "connected project setup omitted resolved user preferences".to_owned(),
        )
    })?;
    let registry_changed_paths = RefCell::new(Vec::new());
    let commit = || {
        reconcile_project_registry(
            user_root,
            target,
            &preferences,
            SetupMode::Apply,
            global_wiki_enabled,
            &mut registry_changed_paths.borrow_mut(),
        )
    };
    let mut outcome = execute_setup_with_post_apply(request, &commit)?;
    outcome
        .changed_paths
        .extend(registry_changed_paths.into_inner());
    outcome.changed_paths.sort();
    outcome.changed_paths.dedup();
    Ok(outcome)
}

fn reconcile_project_registry(
    user_root: &Path,
    target: &Path,
    preferences: &hive_render::ResolvedProjectPreferences,
    mode: SetupMode,
    global_wiki_enabled: bool,
    changed_paths: &mut Vec<String>,
) -> Result<(), RenderError> {
    use hive_wiki::shared::{
        KnowledgeLanguage, KnowledgeVisibility, RegisteredProject, PROJECT_REGISTRY_RELATIVE,
        SHARED_INDEX_RELATIVE,
    };

    let root = hive_wiki::shared::canonical_root(target)
        .map_err(|error| RenderError::Verification(error.to_string()))?;
    let root_text = root.to_str().ok_or_else(|| {
        RenderError::Unsupported("registered project root is not valid UTF-8".to_owned())
    })?;
    let digest = sha256_digest(root_text.as_bytes());
    let project_id = format!("project-{}", &digest["sha256:".len()..][..24]);
    let language = match preferences.wiki_language.as_str() {
        "en" => KnowledgeLanguage::En,
        "ko" => KnowledgeLanguage::Ko,
        "both" => KnowledgeLanguage::Both,
        value => {
            return Err(RenderError::Internal(format!(
                "resolved project Wiki language is invalid: {value}"
            )))
        }
    };
    let registration = RegisteredProject {
        id: project_id,
        root,
        enabled: preferences.wiki_enabled,
        language,
        visibility: KnowledgeVisibility::ProjectPrivate,
    };
    match mode {
        SetupMode::DryRun => {
            changed_paths.push(format!("user-root:{PROJECT_REGISTRY_RELATIVE}"));
            if global_wiki_enabled {
                changed_paths.push(format!("user-root:{SHARED_INDEX_RELATIVE}"));
            }
        }
        SetupMode::Apply => {
            if global_wiki_enabled {
                let registered =
                    hive_wiki::shared::register_project_atomic(user_root, registration, true)
                        .map_err(|error| RenderError::Verification(error.to_string()))?;
                changed_paths.extend(
                    registered
                        .registry
                        .changed_paths
                        .into_iter()
                        .map(|path| format!("user-root:{path}")),
                );
                if let Some(rebuilt) = registered.shared_index {
                    changed_paths.extend(
                        rebuilt
                            .changed_paths
                            .into_iter()
                            .map(|path| format!("user-root:{path}")),
                    );
                }
            } else {
                let registered = hive_wiki::shared::register_project_with_shared_index_disabled(
                    user_root,
                    registration,
                )
                .map_err(|error| RenderError::Verification(error.to_string()))?;
                changed_paths.extend(
                    registered
                        .registry
                        .changed_paths
                        .into_iter()
                        .chain(registered.removed_derived_paths)
                        .map(|path| format!("user-root:{path}")),
                );
            }
        }
        SetupMode::Validate => {
            let registry = hive_wiki::shared::load_project_registry(user_root)
                .map_err(|error| RenderError::Verification(error.to_string()))?;
            if !registry
                .projects
                .iter()
                .any(|project| project == &registration)
            {
                return Err(RenderError::Verification(
                    "project registration differs from resolved setup preferences".to_owned(),
                ));
            }
            if global_wiki_enabled {
                hive_wiki::shared::validate_shared_index(user_root)
                    .map_err(|error| RenderError::Verification(error.to_string()))?;
            } else {
                hive_wiki::shared::validate_shared_derived_state_absent(user_root)
                    .map_err(|error| RenderError::Verification(error.to_string()))?;
            }
        }
    }
    changed_paths.sort();
    changed_paths.dedup();
    Ok(())
}

fn emit_setup_result(result: &ActionResult) -> ExitCode {
    emit_json_result(result);
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
    ExitCode::from(result.exit_code)
}

fn parse_setup(arguments: &[String]) -> Result<SetupArguments, RenderError> {
    let mut target = None;
    let mut answers = None;
    let mut capabilities = None;
    let mut user_root = None;
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
            "--user-root" => {
                user_root = Some(next_value(arguments, &mut index, "--user-root")?);
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
        user_root: PathBuf::from(required(user_root, "--user-root")?),
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
        data: None,
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

struct RouteArguments {
    request: PathBuf,
}

fn run_route(arguments: &[String]) -> ExitCode {
    let result = match parse_route(arguments).and_then(|arguments| {
        read_json_contract::<RoutingRequest>(
            &arguments.request,
            ROUTING_REQUEST_SCHEMA,
            "routing request",
        )
        .map(|(request, bytes)| (arguments, request, bytes))
    }) {
        Ok((arguments, request, bytes)) => match resolve_route(&request) {
            Ok(decision) => {
                let blocked = decision.route == Route::Blocked;
                let action = logical_action_name(decision.logical_action);
                let next_action = decision
                    .next_action
                    .map(logical_action_name)
                    .map(str::to_owned);
                match serde_json::to_value(&decision) {
                    Ok(data) => ActionResult {
                        schema_version: 1,
                        action,
                        status: if blocked { "blocked" } else { "success" },
                        exit_code: if blocked { 3 } else { 0 },
                        code: if blocked {
                            "hive.routing-blocked"
                        } else {
                            "hive.routing-resolved"
                        },
                        message: if blocked {
                            "normalized routing facts require an explicit transition".to_owned()
                        } else {
                            "normalized routing facts resolved deterministically".to_owned()
                        },
                        changed_paths: Vec::new(),
                        evidence: vec![file_evidence(&arguments.request, &bytes)],
                        next_action,
                        data: Some(data),
                    },
                    Err(error) => internal_result(
                        action,
                        format!("cannot serialize routing decision: {error}"),
                    ),
                }
            }
            Err(error) if error.code() == "hive.routing-proof-invalid" => ActionResult {
                schema_version: 1,
                action: request
                    .explicit_action
                    .map_or("RunWork", logical_action_name),
                status: "verification-failed",
                exit_code: 5,
                code: error.code(),
                message: error.message().to_owned(),
                changed_paths: Vec::new(),
                evidence: vec![file_evidence(&arguments.request, &bytes)],
                next_action: None,
                data: None,
            },
            Err(error) => invalid_input_result("RunWork", error.to_string()),
        },
        Err(message) => invalid_input_result("RunWork", message),
    };
    emit_action_result(&result)
}

fn parse_route(arguments: &[String]) -> Result<RouteArguments, String> {
    let mut request = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = &arguments[index];
        index += 1;
        let value = arguments
            .get(index)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option.as_str() {
            "--request" if request.is_none() => request = Some(PathBuf::from(value)),
            "--output" if output.is_none() => output = Some(value.clone()),
            "--request" | "--output" => return Err(format!("duplicate route option: {option}")),
            _ => return Err(format!("unknown route option: {option}")),
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err("route requires --output json".to_owned());
    }
    Ok(RouteArguments {
        request: request.ok_or_else(|| "missing required option --request".to_owned())?,
    })
}

struct PromptArguments {
    request: PathBuf,
    result: PathBuf,
}

struct PromptContracts {
    arguments: PromptArguments,
    input: PromptRefinementInput,
    output: PromptRefinementResult,
    input_bytes: Vec<u8>,
    output_bytes: Vec<u8>,
}

fn run_prompt(arguments: &[String]) -> ExitCode {
    match arguments.first().map(String::as_str) {
        Some("validate") => run_prompt_validate(arguments),
        Some("approve") => run_prompt_approve(arguments),
        _ => emit_action_result(&invalid_input_result(
            "RefinePrompt",
            "prompt requires the validate or approve action".to_owned(),
        )),
    }
}

fn run_prompt_validate(arguments: &[String]) -> ExitCode {
    let result = match parse_prompt_validate(arguments).and_then(read_prompt_contracts) {
        Ok(contracts) => {
            if contracts.input.mode == hive_projection::RefineMode::RefineAndRun
                && !contracts.input.explicit_run_intent
            {
                prompt_result(
                    "blocked",
                    3,
                    "hive.refine-run-not-authorized",
                    "refine-and-run requires explicit run intent".to_owned(),
                    &contracts.arguments,
                    &contracts.input_bytes,
                    &contracts.output_bytes,
                )
            } else {
                match validate_prompt_refinement(&contracts.input, &contracts.output) {
                    Ok(()) => prompt_validation_success(&contracts),
                    Err(error) if error.code() == "hive.refine-run-not-authorized" => {
                        prompt_result(
                            "blocked",
                            3,
                            error.code(),
                            error.message().to_owned(),
                            &contracts.arguments,
                            &contracts.input_bytes,
                            &contracts.output_bytes,
                        )
                    }
                    Err(error) => prompt_result(
                        "verification-failed",
                        5,
                        error.code(),
                        error.message().to_owned(),
                        &contracts.arguments,
                        &contracts.input_bytes,
                        &contracts.output_bytes,
                    ),
                }
            }
        }
        Err(message) => invalid_input_result("RefinePrompt", message),
    };
    emit_action_result(&result)
}

fn parse_prompt_validate(arguments: &[String]) -> Result<PromptArguments, String> {
    let mut request = None;
    let mut result = None;
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        index += 1;
        let value = arguments
            .get(index)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option.as_str() {
            "--request" if request.is_none() => request = Some(PathBuf::from(value)),
            "--result" if result.is_none() => result = Some(PathBuf::from(value)),
            "--output" if output.is_none() => output = Some(value.clone()),
            "--request" | "--result" | "--output" => {
                return Err(format!("duplicate prompt option: {option}"));
            }
            _ => return Err(format!("unknown prompt option: {option}")),
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err("prompt validate requires --output json".to_owned());
    }
    Ok(PromptArguments {
        request: request.ok_or_else(|| "missing required option --request".to_owned())?,
        result: result.ok_or_else(|| "missing required option --result".to_owned())?,
    })
}

struct PromptApprovalArguments {
    contracts: PromptArguments,
    digest: String,
    target_host: String,
    confirmed: bool,
}

fn run_prompt_approve(arguments: &[String]) -> ExitCode {
    let result = match parse_prompt_approval(arguments) {
        Ok(arguments) if !arguments.confirmed => prompt_approval_confirmation_required_result(),
        Ok(arguments) => {
            let digest = arguments.digest.clone();
            let target_host = arguments.target_host.clone();
            match read_prompt_contracts(arguments.contracts) {
                Ok(contracts) => {
                    match validate_prompt_refinement(&contracts.input, &contracts.output) {
                        Ok(()) => {
                            let expected_digest =
                                sha256_digest(contracts.output.refined_prompt.as_bytes());
                            if contracts.input.mode != hive_projection::RefineMode::RefineOnly {
                                prompt_result(
                                "blocked",
                                3,
                                "hive.refine-approval-not-required",
                                "only a refine-only result may enter a later approval lifecycle"
                                    .to_owned(),
                                &contracts.arguments,
                                &contracts.input_bytes,
                                &contracts.output_bytes,
                            )
                            } else if digest != expected_digest {
                                prompt_result(
                                    "blocked",
                                    3,
                                    "hive.refine-approval-stale",
                                    "approval digest does not bind the current refined prompt"
                                        .to_owned(),
                                    &contracts.arguments,
                                    &contracts.input_bytes,
                                    &contracts.output_bytes,
                                )
                            } else if !matches!(
                                target_host.as_str(),
                                "codex" | "claude" | "antigravity"
                            ) {
                                invalid_input_result(
                                    "RefinePrompt",
                                    "target host must be codex, claude, or antigravity".to_owned(),
                                )
                            } else if contracts
                                .input
                                .target_host
                                .is_some_and(|host| prompt_host_name(host) != target_host)
                            {
                                prompt_result(
                                    "blocked",
                                    3,
                                    "hive.refine-approval-host-mismatch",
                                    "approval target host differs from the refined prompt contract"
                                        .to_owned(),
                                    &contracts.arguments,
                                    &contracts.input_bytes,
                                    &contracts.output_bytes,
                                )
                            } else {
                                prompt_approval_success(&contracts, &target_host, &expected_digest)
                            }
                        }
                        Err(error) if error.code() == "hive.refine-run-not-authorized" => {
                            prompt_result(
                                "blocked",
                                3,
                                error.code(),
                                error.message().to_owned(),
                                &contracts.arguments,
                                &contracts.input_bytes,
                                &contracts.output_bytes,
                            )
                        }
                        Err(error) => prompt_result(
                            "verification-failed",
                            5,
                            error.code(),
                            error.message().to_owned(),
                            &contracts.arguments,
                            &contracts.input_bytes,
                            &contracts.output_bytes,
                        ),
                    }
                }
                Err(message) => invalid_input_result("RefinePrompt", message),
            }
        }
        Err(message) => invalid_input_result("RefinePrompt", message),
    };
    emit_action_result(&result)
}

fn parse_prompt_approval(arguments: &[String]) -> Result<PromptApprovalArguments, String> {
    let mut request = None;
    let mut result = None;
    let mut digest = None;
    let mut target_host = None;
    let mut output = None;
    let mut confirmed = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        index += 1;
        if option == "--confirm-refined-prompt" {
            if confirmed {
                return Err("duplicate prompt approval option: --confirm-refined-prompt".to_owned());
            }
            confirmed = true;
            continue;
        }
        let value = arguments
            .get(index)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option.as_str() {
            "--request" if request.is_none() => request = Some(PathBuf::from(value)),
            "--result" if result.is_none() => result = Some(PathBuf::from(value)),
            "--digest" if digest.is_none() => digest = Some(value.clone()),
            "--target-host" if target_host.is_none() => target_host = Some(value.clone()),
            "--output" if output.is_none() => output = Some(value.clone()),
            "--request" | "--result" | "--digest" | "--target-host" | "--output" => {
                return Err(format!("duplicate prompt approval option: {option}"));
            }
            _ => return Err(format!("unknown prompt approval option: {option}")),
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err("prompt approve requires --output json".to_owned());
    }
    Ok(PromptApprovalArguments {
        contracts: PromptArguments {
            request: request.ok_or_else(|| "missing required option --request".to_owned())?,
            result: result.ok_or_else(|| "missing required option --result".to_owned())?,
        },
        digest: digest.ok_or_else(|| "missing required option --digest".to_owned())?,
        target_host: target_host
            .ok_or_else(|| "missing required option --target-host".to_owned())?,
        confirmed,
    })
}

fn read_prompt_contracts(arguments: PromptArguments) -> Result<PromptContracts, String> {
    let (input, input_bytes) = read_json_contract(
        &arguments.request,
        PROMPT_REFINEMENT_INPUT_SCHEMA,
        "prompt refinement input",
    )?;
    let (output, output_bytes) = read_json_contract(
        &arguments.result,
        PROMPT_REFINEMENT_RESULT_SCHEMA,
        "prompt refinement result",
    )?;
    Ok(PromptContracts {
        arguments,
        input,
        output,
        input_bytes,
        output_bytes,
    })
}

fn read_json_contract<T: DeserializeOwned>(
    path: &Path,
    schema: &str,
    label: &str,
) -> Result<(T, Vec<u8>), String> {
    let metadata =
        std::fs::metadata(path).map_err(|error| format!("cannot inspect {label}: {error}"))?;
    if !metadata.is_file() || metadata.len() > MAX_CONTRACT_BYTES {
        return Err(format!(
            "{label} must be a regular JSON file no larger than {MAX_CONTRACT_BYTES} bytes"
        ));
    }
    let bytes = std::fs::read(path).map_err(|error| format!("cannot read {label}: {error}"))?;
    let instance: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid {label} JSON: {error}"))?;
    validate_json_schema(schema, &instance, label)?;
    let typed = serde_json::from_value(instance)
        .map_err(|error| format!("{label} does not match the typed contract: {error}"))?;
    Ok((typed, bytes))
}

fn validate_json_schema(
    schema: &str,
    instance: &serde_json::Value,
    label: &str,
) -> Result<(), String> {
    let schema: serde_json::Value = serde_json::from_str(schema)
        .map_err(|error| format!("embedded {label} schema is invalid JSON: {error}"))?;
    jsonschema::meta::validate(&schema)
        .map_err(|error| format!("embedded {label} schema is invalid: {error}"))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| format!("cannot compile embedded {label} schema: {error}"))?;
    validator
        .validate(instance)
        .map_err(|error| format!("{label} violates the JSON Schema contract: {error}"))
}

fn prompt_validation_success(contracts: &PromptContracts) -> ActionResult {
    let lifecycle = prompt_refinement_lifecycle(&contracts.input, &contracts.output);
    let next_action = match lifecycle.state {
        PromptRefinementState::AwaitingApproval => "awaiting-approval",
        PromptRefinementState::Authorized => "host-owned-execution",
    };
    ActionResult {
        schema_version: 1,
        action: "RefinePrompt",
        status: "success",
        exit_code: 0,
        code: "hive.prompt-refinement-valid",
        message: "prompt refinement preserves the normalized contract".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![
            file_evidence(&contracts.arguments.request, &contracts.input_bytes),
            file_evidence(&contracts.arguments.result, &contracts.output_bytes),
        ],
        next_action: Some(next_action.to_owned()),
        data: Some(
            serde_json::to_value(lifecycle).expect("prompt refinement lifecycle must serialize"),
        ),
    }
}

fn prompt_approval_success(
    contracts: &PromptContracts,
    target_host: &str,
    refined_prompt_digest: &str,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "RefinePrompt",
        status: "success",
        exit_code: 0,
        code: "hive.prompt-approved",
        message: "exact refined prompt approval is bound for host-owned execution".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![
            file_evidence(&contracts.arguments.request, &contracts.input_bytes),
            file_evidence(&contracts.arguments.result, &contracts.output_bytes),
        ],
        next_action: Some("host-owned-execution".to_owned()),
        data: Some(serde_json::json!({
            "state": "authorized",
            "refined_prompt_digest": refined_prompt_digest,
            "target_host": target_host,
            "execution_owner": "host-native",
            "result_locator": contracts.arguments.result.display().to_string(),
        })),
    }
}

const fn prompt_host_name(host: hive_projection::Host) -> &'static str {
    match host {
        hive_projection::Host::Codex => "codex",
        hive_projection::Host::Claude => "claude",
        hive_projection::Host::Antigravity => "antigravity",
    }
}

fn prompt_result(
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
    arguments: &PromptArguments,
    input_bytes: &[u8],
    output_bytes: &[u8],
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "RefinePrompt",
        status,
        exit_code,
        code,
        message,
        changed_paths: Vec::new(),
        evidence: vec![
            file_evidence(&arguments.request, input_bytes),
            file_evidence(&arguments.result, output_bytes),
        ],
        next_action: None,
        data: None,
    }
}

fn prompt_approval_confirmation_required_result() -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "RefinePrompt",
        status: "blocked",
        exit_code: 3,
        code: "hive.refine-approval-confirmation-required",
        message: "prompt approval requires --confirm-refined-prompt".to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: Some("awaiting-approval".to_owned()),
        data: None,
    }
}

fn invalid_input_result(action: &'static str, message: String) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "error",
        exit_code: 2,
        code: "hive.invalid-input",
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn internal_result(action: &'static str, message: String) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "error",
        exit_code: 10,
        code: "hive.internal-error",
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn file_evidence(path: &Path, bytes: &[u8]) -> Evidence {
    Evidence {
        kind: "file",
        locator: path.display().to_string(),
        digest: sha256_digest(bytes),
    }
}

fn logical_action_name(action: LogicalAction) -> &'static str {
    match action {
        LogicalAction::AnswerSimpleQuestion => "AnswerSimpleQuestion",
        LogicalAction::RefinePrompt => "RefinePrompt",
        LogicalAction::RunWork => "RunWork",
        LogicalAction::ResumeWork => "ResumeWork",
        LogicalAction::VerifyWork => "VerifyWork",
        LogicalAction::IngestKnowledge => "IngestKnowledge",
        LogicalAction::QueryKnowledge => "QueryKnowledge",
        LogicalAction::UpdateHarness => "UpdateHarness",
    }
}

fn emit_action_result(result: &ActionResult) -> ExitCode {
    emit_json_result(result);
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
    ExitCode::from(result.exit_code)
}

fn run_hook(arguments: &[String]) -> ExitCode {
    let stop_requested = requested_event(arguments).as_deref() == Some("Stop");
    if stop_requested {
        println!("{}", neutral_stop_hook_payload());
        return ExitCode::SUCCESS;
    }
    match parse_hook(arguments) {
        Ok(arguments) => {
            let Some(fresh_capabilities) = arguments.capabilities.as_deref() else {
                println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
                return ExitCode::SUCCESS;
            };
            let target = match env::current_dir() {
                Ok(target) => target,
                Err(error) => {
                    eprintln!("diagnostic: inactive fallback hook: {error}");
                    println!("{{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}}");
                    return ExitCode::SUCCESS;
                }
            };
            let authorization = authorize_hook_with_resolution(
                &target,
                &arguments.capability,
                &arguments.event,
                fresh_capabilities,
            );
            match authorization {
                Ok(HookAuthorization::Authorized) => {
                    match read_hook_input(arguments.input.as_deref(), &arguments.event) {
                        Ok(input) => {
                            match execute_hook_capability(&target, &arguments.capability, &input) {
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
                            }
                        }
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

fn neutral_stop_hook_payload() -> &'static str {
    "{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}"
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

struct HookArguments {
    capability: String,
    event: String,
    capabilities: Option<PathBuf>,
    input: Option<PathBuf>,
}

fn parse_hook(arguments: &[String]) -> Result<HookArguments, String> {
    let mut capability = None;
    let mut event = None;
    let mut capabilities = None;
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
            "--capability" if capability.is_none() => capability = Some(value),
            "--event" if event.is_none() => event = Some(value),
            "--capabilities" if capabilities.is_none() => {
                capabilities = Some(PathBuf::from(value));
            }
            "--input" if input.is_none() => input = Some(PathBuf::from(value)),
            "--output" if output.is_none() => output = Some(value),
            "--capability" | "--event" | "--capabilities" | "--input" | "--output" => {
                return Err(format!("duplicate hook option: {option}"));
            }
            _ => return Err(format!("unknown hook option: {option}")),
        }
        index += 1;
    }
    if output.as_deref() != Some("json") {
        return Err("hook requires --output json".to_owned());
    }
    Ok(HookArguments {
        capability: capability.ok_or_else(|| "missing --capability".to_owned())?,
        event: event.ok_or_else(|| "missing --event".to_owned())?,
        capabilities,
        input,
    })
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
        Some("-v" | "-V" | "--version") => {
            println!("{}", version_output());
            Ok(())
        }
        Some("-h" | "--help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(command) => Err(format!("unknown command: {command}\n\n{USAGE}")),
    }
}

fn version_output() -> String {
    version_output_for(
        env!("CARGO_PKG_VERSION"),
        env!("HIVE_PACKAGE_VERSION"),
        env!("HIVE_PACKAGE_RELEASE_DATE"),
    )
}

fn version_output_for(product: &str, package: &str, release_date: &str) -> String {
    if package == product {
        format!("AIgent Hive v{product} (released {release_date})")
    } else if package == format!("{product}-dev") {
        format!("AIgent Hive v{product}-dev · local developer build (built {release_date})")
    } else if package == format!("{product}-test") {
        format!("AIgent Hive v{product}-test · developer test build (released {release_date})")
    } else {
        let revision = package
            .strip_prefix(&format!("{product}-test."))
            .expect("embedded package version is validated by build.rs");
        format!(
            "AIgent Hive v{product}-test #{revision} · developer test build (released {release_date})"
        )
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
        neutral_stop_hook_payload, normalize_hook_path, parse_hook, parse_setup, parse_usage,
        probe_native_usage, reconcile_project_registry, run_human, version_output_for, wants_json,
        ActionResult, HookInput, ParsedUsageArguments, SETUP_USAGE, USAGE,
    };
    use hive_render::{RenderError, ResolvedProjectPreferences, SetupMode};
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
    fn native_usage_probe_parses_every_host_without_fallback_consent() {
        for (name, expected) in [
            ("codex", crate::usage::UsageHost::Codex),
            ("claude", crate::usage::UsageHost::Claude),
            ("antigravity", crate::usage::UsageHost::Antigravity),
        ] {
            let parsed = parse_usage(&[
                "probe-native".to_owned(),
                "--host".to_owned(),
                name.to_owned(),
                "--output".to_owned(),
                "json".to_owned(),
            ])
            .expect("native probe arguments");
            assert!(matches!(
                parsed,
                ParsedUsageArguments::ProbeNative(host) if host == expected
            ));
        }
    }

    #[test]
    fn inactive_host_native_probe_defers_without_naming_or_invoking_codexbar() {
        for host in [
            crate::usage::UsageHost::Claude,
            crate::usage::UsageHost::Antigravity,
        ] {
            let result = probe_native_usage(host);
            assert_eq!(result.code, "hive.usage-native-probe-deferred");
            assert_eq!(result.exit_code, 0);
            assert!(result.next_action.is_none());
            assert!(!serde_json::to_string(&result)
                .expect("probe result JSON")
                .contains("CodexBar"));
        }
    }

    #[test]
    fn help_is_the_default_command() {
        assert_eq!(run_human(std::iter::empty()), Ok(()));
    }

    #[test]
    fn version_output_surfaces_developer_build_kinds() {
        assert_eq!(
            version_output_for("0.9.0", "0.9.0-dev", "2026-08-07"),
            "AIgent Hive v0.9.0-dev · local developer build (built 2026-08-07)"
        );
        assert_eq!(
            version_output_for("0.9.0", "0.9.0-test.2", "2026-08-06"),
            "AIgent Hive v0.9.0-test #2 · developer test build (released 2026-08-06)"
        );
    }

    #[test]
    fn setup_help_has_the_exact_supported_invocation() {
        assert!(is_help_request(&["--help".to_owned()]));
        assert!(SETUP_USAGE.contains(
            "hive setup --target <dir> --answers <yml> --capabilities <json> \
             --user-root <dir> (--dry-run|--apply|--validate)"
        ));
        assert!(SETUP_USAGE.contains("--output json"));
    }

    #[test]
    fn stored_hook_preview_cannot_activate_without_dynamic_fresh_evidence() {
        let arguments = [
            "--capability".to_owned(),
            "protect-hive-owned-state".to_owned(),
            "--event".to_owned(),
            "PreToolUse".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let arguments = parse_hook(&arguments).expect("preview command should parse");
        assert!(arguments.input.is_none());
        assert!(arguments.capabilities.is_none());
        assert!(USAGE.contains("[--capabilities <fresh-json>]"));
    }

    #[test]
    fn stop_hook_is_neutral_for_one_hundred_repeated_calls() {
        let before = neutral_stop_hook_payload();
        for _ in 0..100 {
            assert_eq!(neutral_stop_hook_payload(), before);
        }
        let payload: serde_json::Value = serde_json::from_str(before).expect("neutral JSON");
        assert_eq!(payload["decision"], "allow");
        assert_eq!(payload["active"], false);
        assert!(payload.get("changed_paths").is_none());
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
            data: None,
        };
        let json = serde_json::to_value(result).expect("result should serialize");
        assert_eq!(json["action"], "UnknownAction");
        assert_eq!(json["exit_code"], 2);
        assert_eq!(json["changed_paths"], serde_json::json!([]));
        assert!(json.get("data").is_none());
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
    fn setup_disables_and_validates_complete_disposable_rag_cleanup() {
        let temporary = temporary_directory("setup-wiki-disabled");
        let user = temporary.join("user");
        let project = temporary.join("project");
        fs::create_dir_all(&user).expect("user root");
        fs::create_dir_all(&project).expect("project root");
        let preferences = ResolvedProjectPreferences {
            setup_mode: "expedited".to_owned(),
            provenance: "test".to_owned(),
            interface_language: "en".to_owned(),
            wiki_enabled: true,
            wiki_backend: "markdown".to_owned(),
            wiki_language: "both".to_owned(),
            persona_id: "balanced".to_owned(),
            persona_custom_description: None,
            selected_project_skills: Vec::new(),
            usage_guard_enabled: false,
            codexbar_fallback_enabled: false,
            discord_guard_enabled: false,
            discord_webhook_url_env: None,
            discord_message_fields: vec![
                "remaining-usage".to_owned(),
                "project".to_owned(),
                "request".to_owned(),
                "progress".to_owned(),
                "host".to_owned(),
                "resume".to_owned(),
            ],
            usage_stop_remaining_percent: 60,
        };
        let mut changed = Vec::new();
        reconcile_project_registry(
            &user,
            &project,
            &preferences,
            SetupMode::Apply,
            true,
            &mut changed,
        )
        .expect("enable Global Wiki");
        for relative in hive_wiki::shared::SHARED_DERIVED_RELATIVES {
            let path = user.join(relative);
            fs::create_dir_all(path.parent().expect("derived parent")).expect("derived parent");
            if !path.exists() {
                fs::write(path, b"derived\n").expect("derived artifact");
            }
        }

        changed.clear();
        reconcile_project_registry(
            &user,
            &project,
            &preferences,
            SetupMode::Apply,
            false,
            &mut changed,
        )
        .expect("disable Global Wiki");

        for relative in hive_wiki::shared::SHARED_DERIVED_RELATIVES {
            assert!(!user.join(relative).exists(), "{relative}");
            assert!(changed.contains(&format!("user-root:{relative}")));
        }
        reconcile_project_registry(
            &user,
            &project,
            &preferences,
            SetupMode::Validate,
            false,
            &mut Vec::new(),
        )
        .expect("validate disabled Global Wiki");

        fs::create_dir_all(user.join(".hive/index")).expect("index directory");
        fs::write(user.join(".hive/index/.stale"), b"stale\n").expect("stale marker");
        let error = reconcile_project_registry(
            &user,
            &project,
            &preferences,
            SetupMode::Validate,
            false,
            &mut Vec::new(),
        )
        .expect_err("disabled validation must reject derived residue");
        assert!(matches!(error, RenderError::Verification(_)));
        fs::remove_dir_all(temporary).expect("temporary target should be removed");
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
