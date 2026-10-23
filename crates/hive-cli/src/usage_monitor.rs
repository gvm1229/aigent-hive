//! Independent Windows Codex interruption. Never starts model work or persists transcripts.
mod desktop;
mod platform;
mod preferences;
#[cfg(debug_assertions)]
mod qualification;
mod sensor;
mod state;
mod threshold;
mod upgrade;

#[cfg(debug_assertions)]
pub(crate) use qualification::run_qualification;
#[cfg(debug_assertions)]
pub(crate) use sensor::run_supervisor;
pub(crate) use sensor::run_worker;
pub(crate) use upgrade::upgrade_existing;

use crate::run::{AdapterError, PinnedTarget};
use crate::{emit_action_result, ActionResult};
use base64::Engine;
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::OpenOptions;
use hive_core::sha256_digest;
use hive_core::usage_guard::UsageSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use state::{MonitorState, BLIND_SECONDS, POLL_SECONDS};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BASE: &str = ".hive/runtime/usage-monitor";
const CONFIG: &str = ".hive/runtime/usage-monitor/config.json";
const STATE: &str = ".hive/runtime/usage-monitor/state.json";
const STARTUP: &str = ".hive/runtime/usage-monitor/startup.json";

const HELP: &str = "hive usage monitor preview|enable|disable|status --user-root <dir> [--confirm <preview-digest> --confirm-session-interruption] --output json\n";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    schema_version: u8,
    enabled: bool,
    version: String,
    executable: PathBuf,
    executable_digest: String,
    task_name: String,
    arguments: String,
    #[serde(default)]
    launcher: Option<TaskLauncher>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TaskLauncher {
    executable: PathBuf,
    arguments: String,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn error(message: &str) -> AdapterError {
    AdapterError::Safety(message.into())
}
fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, AdapterError> {
    serde_json::to_vec_pretty(value).map_err(|_| error("monitor state serialization failed"))
}
fn load<T: serde::de::DeserializeOwned>(
    root: &PinnedTarget,
    path: &str,
) -> Result<Option<T>, AdapterError> {
    let mut observed = root.read_optional(Path::new(path), 65536);
    for _ in 0..2 {
        if matches!(&observed, Ok(Some(_))) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
        observed = root.read_optional(Path::new(path), 65536);
    }
    observed?
        .map(|bytes| {
            serde_json::from_value(
                crate::usage::parse_strict_native_json(&bytes)
                    .map_err(|_| error("monitor state is malformed"))?,
            )
            .map_err(|_| error("monitor state is invalid"))
        })
        .transpose()
}
fn write<T: Serialize>(root: &PinnedTarget, path: &str, value: &T) -> Result<(), AdapterError> {
    let path = Path::new(path);
    let old = root.snapshot(path)?;
    root.publish_runtime(path, &old, &serialize(value)?)?;
    Ok(())
}
fn lock(root: &PinnedTarget, name: &str) -> Result<File, AdapterError> {
    let path = Path::new(BASE).join(name);
    root.ensure_owned_parent(&path, Path::new(".hive"))?;
    let mut parent = root
        .target_dir()
        .try_clone()
        .map_err(|_| error("cannot pin lock parent"))?;
    for component in Path::new(BASE).components() {
        parent = parent
            .open_dir_nofollow(component.as_os_str())
            .map_err(|_| error("unsafe monitor lock ancestor"))?;
    }
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(error("monitor lock is not a no-follow file")),
        Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => {
            let mut create = OpenOptions::new();
            create
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            if let Err(failure) = parent.open_with(name, &create) {
                if failure.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(error("cannot create monitor lock"));
                }
            }
        }
        Err(_) => return Err(error("cannot inspect monitor lock")),
    }
    // Do not read a lock file: Windows byte-range locks correctly reject such reads.
    let mut options = OpenOptions::new();
    options.read(true).write(true).follow(FollowSymlinks::No);
    let file = parent
        .open_with(name, &options)
        .map_err(|_| error("cannot pin monitor lock"))?
        .into_std();
    file.try_lock()
        .map_err(|_| AdapterError::Conflict("monitor operation already running".into()))?;
    Ok(file)
}

fn with_state<T>(
    root: &PinnedTarget,
    apply: impl FnOnce(&mut MonitorState) -> Result<T, AdapterError>,
) -> Result<T, AdapterError> {
    let _lock = lock(root, "state.lock")?;
    let mut state: MonitorState = load(root, STATE)?.unwrap_or_else(MonitorState::new);
    state.migrate()?;
    let generation = state.generation;
    let result = apply(&mut state)?;
    if state.generation != generation {
        if let Some(incident) = state.incident.as_mut() {
            incident.detected_at_ms = Some(
                u64::try_from(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                )
                .unwrap_or(u64::MAX),
            );
        }
    }
    write(root, STATE, &state)?;
    Ok(result)
}

/// Deduplicate an observed refill; this never grants permission or blocks a new request.
pub(crate) fn reset_already_dispatched(
    root: &PinnedTarget,
    current: &[UsageSnapshot],
    previous: &[UsageSnapshot],
) -> bool {
    load::<MonitorState>(root, STATE)
        .ok()
        .flatten()
        .is_some_and(|state| {
            matches!(state.schema_version, 2 | 3)
                && state.reset_already_dispatched(current, previous)
        })
}

pub(crate) fn monitor_enabled(root: &PinnedTarget) -> bool {
    load::<Configuration>(root, CONFIG)
        .ok()
        .flatten()
        .is_some_and(|configuration| configuration.enabled)
}

pub(crate) fn threshold_already_handled(
    root: &PinnedTarget,
    policy: &crate::usage_control::MonitorPolicy,
    snapshots: &[UsageSnapshot],
    automatic: bool,
) -> bool {
    !automatic
        && cfg!(windows)
        && monitor_enabled(root)
        && load::<MonitorState>(root, STATE)
            .ok()
            .flatten()
            .is_some_and(|state| {
                state.schema_version == 3 && threshold::handled(&state, policy, snapshots)
            })
}

pub(crate) fn session_process_identity(pid: u32) -> Option<String> {
    use crate::usage::CommandRunner;
    let executable = crate::usage::SystemCommandRunner.qualify("codex").ok()?;
    platform::call(
        &json!({"operation":"session-identity","pid":pid,"executable":executable.path()}),
        Duration::from_secs(5),
    )
    .ok()?
    .get("digest")?
    .as_str()
    .map(str::to_owned)
}

pub(crate) fn run(args: &[String]) -> ExitCode {
    if args.is_empty() || args == ["--help"] {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let result = parse_and_run(args);
    let mut result = match result {
        Ok(data) => response(
            "success",
            0,
            "hive.usage-monitor-complete",
            "usage monitor command completed",
            Some(data),
        ),
        Err(e) => response(
            e.status(),
            e.exit_code(),
            "hive.usage-monitor-blocked",
            e.message(),
            None,
        ),
    };
    if result.exit_code == 0
        && matches!(args.first().map(String::as_str), Some("enable" | "disable"))
    {
        result.changed_paths = result
            .data
            .as_ref()
            .and_then(|v| v["changed_paths"].as_array())
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|p| p.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
    }
    let exit = result.exit_code;
    emit_action_result(&result);
    ExitCode::from(exit)
}

fn response(
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: &str,
    data: Option<Value>,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "ManageUsageMonitor",
        status,
        exit_code,
        code,
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data,
    }
}

fn parse_and_run(args: &[String]) -> Result<Value, AdapterError> {
    let action = args.first().map(String::as_str).unwrap_or_default();
    if !matches!(action, "preview" | "enable" | "disable" | "status" | "run") {
        return Err(AdapterError::Input(HELP.into()));
    }
    let mut root = None;
    let mut confirm = None;
    let mut terminate = false;
    let mut output = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index].as_str();
        if option == "--confirm-session-interruption" {
            if terminate {
                return Err(error("duplicate confirmation"));
            }
            terminate = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| AdapterError::Input("missing monitor option value".into()))?;
        match option {
            "--user-root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--confirm" if confirm.is_none() => confirm = Some(value.as_str()),
            "--output" if !output && value == "json" => output = true,
            _ => {
                return Err(AdapterError::Input(
                    "unknown or duplicate monitor option".into(),
                ))
            }
        }
        index += 2;
    }
    if !output {
        return Err(AdapterError::Input("--output json is required".into()));
    }
    if (action == "enable") != (confirm.is_some() && terminate)
        || (action != "enable" && (confirm.is_some() || terminate))
    {
        return Err(error("monitor confirmation does not match action"));
    }
    let root_path = root.ok_or_else(|| AdapterError::Input("--user-root is required".into()))?;
    let root = PinnedTarget::open_usage(&root_path)?;
    root.read_required(Path::new(".hive/config/user-setup.yml"), 65536)?;
    match action {
        "preview" => preview(root.requested_path()),
        "status" => monitor_status(&root),
        "enable" => enable(&root, confirm.unwrap_or_default()),
        "disable" => disable(&root),
        "run" => controller(&root),
        _ => unreachable!(),
    }
}

fn monitor_status(root: &PinnedTarget) -> Result<Value, AdapterError> {
    let configuration: Option<Configuration> = load(root, CONFIG)?;
    let mut state: MonitorState = load(root, STATE)?.unwrap_or_else(MonitorState::new);
    let probe = configuration
        .as_ref()
        .zip(state.controller_pid)
        .and_then(|(config, pid)| {
            platform::call(
                &json!({"operation":"session-identity","pid":pid,"executable":config.executable}),
                Duration::from_secs(5),
            )
            .ok()
        });
    let alive = probe
        .as_ref()
        .and_then(|value| value["digest"].as_str())
        .is_some_and(|digest| state.controller_identity.as_deref() == Some(digest));
    let uptime = probe.as_ref().and_then(|value| value["uptime_ms"].as_u64());
    let gap = state.observation_gap_monotonic(state.app_alive, now(), uptime);
    let heartbeat_fresh = uptime.zip(state.heartbeat_uptime_ms).map_or_else(
        || now().saturating_sub(state.heartbeat) < 5,
        |(tick, last)| tick >= last && tick - last < 5000,
    );
    let observed = state.observation_count > 0
        && state.last_success.is_some()
        && gap.as_secs() < BLIND_SECONDS;
    let sensor_health = if !alive {
        "stopped"
    } else if !state.app_alive {
        "idle"
    } else if !observed {
        "blind"
    } else if state.last_query_error.is_some() {
        "recovering"
    } else {
        "healthy"
    };
    Ok(
        json!({"configuration":configuration,"state":state,"startup":load::<Value>(root, STARTUP).ok().flatten(),"observation_gap_seconds":gap.as_secs(),"controller_alive":alive,"controller_heartbeat_fresh":heartbeat_fresh,"sensor_health":sensor_health,
        "configured_monitor_version":configuration.as_ref().map(|config| &config.version),"running_monitor_version":alive.then_some(&state.runtime_version),
        "last_query_source":state.last_query_source,"last_primary_error":state.primary_query_error,"last_query_error":state.last_query_error,"consecutive_query_failures":state.consecutive_query_failures,"last_stop_confirmed":state.incident.as_ref().map(|incident| incident.stop_confirmed),
        "automatic_control_available":cfg!(windows),"compatibility_policy":"protocol","monitoring_verified": require_app_safe_control().is_ok() && configuration.as_ref().is_some_and(|c| c.enabled) && alive && heartbeat_fresh
            && observed && state.last_query_error.is_none() && state.control_compatibility.as_deref() == Some("compatible") && state.control_connected && !state.control_stopping && state.unknown_sessions == 0 && state.threshold_policy_errors == 0,
        "desktop_compatibility":{"actual_version":state.desktop_version,"state":if !alive || !heartbeat_fresh {"unavailable"} else {state.control_compatibility.as_deref().unwrap_or("pending")},"stage":state.control_stage,"error":state.control_error},
        "upgrade":load::<Value>(root, upgrade::JOURNAL)?,
        "scope":"local-windows-codex","resume_lock":false,"server_billing_interruption_verified":false}),
    )
}

fn configuration(root: &Path) -> Result<Configuration, AdapterError> {
    let normal = monitor_root_path(root)?;
    let root = normal.as_path();
    let executable =
        std::env::current_exe().map_err(|_| error("cannot identify monitor executable"))?;
    let digest = sha256_digest(
        &std::fs::read(&executable).map_err(|_| error("cannot fingerprint executable"))?,
    );
    let root_text = root
        .to_str()
        .ok_or_else(|| error("monitor root must be Unicode"))?;
    if root_text.contains(['"', '\r', '\n']) {
        return Err(error("invalid monitor root"));
    }
    let installed_executable: PathBuf = root
        .join(BASE)
        .join("bin")
        .join(format!(
            "hive-{}-{}.exe",
            env!("CARGO_PKG_VERSION"),
            &digest[7..23]
        ))
        .components()
        .collect();
    let launch = format!(
        "& '{}' usage monitor run --user-root '{}' --output json\nexit $LASTEXITCODE",
        installed_executable.to_string_lossy().replace('\'', "''"),
        root_text.replace('\'', "''")
    );
    let encoded = base64::engine::general_purpose::STANDARD.encode(
        launch
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    let launcher = std::env::var_os("SystemRoot").map(|path| TaskLauncher {
        executable: Path::new(&path).join("System32/WindowsPowerShell/v1.0/powershell.exe"),
        arguments: format!(
            "-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand {encoded}"
        ),
    });
    Ok(Configuration {
        schema_version: 1,
        enabled: true,
        version: env!("HIVE_PACKAGE_VERSION").into(),
        executable: installed_executable,
        executable_digest: digest,
        task_name: monitor_task_name(root),
        arguments: format!("usage monitor run --user-root \"{root_text}\" --output json"),
        launcher,
    })
}

fn monitor_task_name(root: &Path) -> String {
    let key = sha256_digest(root.to_string_lossy().to_lowercase().as_bytes());
    format!("AigentHive-UsageMonitor-{}", &key[7..23])
}

/// Normalize process-facing Windows names only after comparing pinned directory identities.
#[cfg_attr(not(windows), allow(clippy::unnecessary_wraps))]
fn monitor_root_path(root: &Path) -> Result<PathBuf, AdapterError> {
    #[cfg(windows)]
    {
        use cap_fs_ext::MetadataExt;
        use std::path::{Component, Prefix};
        let canonical =
            std::fs::canonicalize(root).map_err(|_| error("monitor root identity unavailable"))?;
        let mut components = canonical.components();
        let normal = match components.next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::VerbatimDisk(drive) => {
                    Some(PathBuf::from(format!("{}:\\", char::from(drive))))
                }
                Prefix::VerbatimUNC(server, share) => {
                    Some(PathBuf::from("\\\\").join(server).join(share))
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(mut normal) = normal {
            for component in components {
                if component != Component::RootDir {
                    normal.push(component.as_os_str());
                }
            }
            let expected = PinnedTarget::open_usage(root)?;
            let actual = PinnedTarget::open_usage(&normal)?;
            let before = expected
                .target_dir()
                .dir_metadata()
                .map_err(|_| error("monitor root identity unavailable"))?;
            let after = actual
                .target_dir()
                .dir_metadata()
                .map_err(|_| error("normalized monitor root identity unavailable"))?;
            if (before.dev(), before.ino()) != (after.dev(), after.ino()) {
                return Err(error(
                    "normalized monitor root identifies another directory",
                ));
            }
            return Ok(normal);
        }
    }
    Ok(root.to_owned())
}
fn preview(root: &Path) -> Result<Value, AdapterError> {
    let mut config = configuration(root)?;
    let available = cfg!(windows);
    config.enabled = available;
    let changed = if available {
        vec![json!(CONFIG), json!(STATE), json!(config.executable)]
    } else {
        vec![]
    };
    let mut data = json!({"configuration":config,"adapter_digest":sha256_digest(platform::SCRIPT.as_bytes()),
        "poll_seconds":POLL_SECONDS,"sample_timeout_seconds":5,"retry_seconds":2,"unresponsive_seconds":BLIND_SECONDS,
        "scope":"all-local-chats-in-current-user-OpenAI.Codex-package","emergency_termination":false,"session_interrupt":available,"app_preserved":available,"compatibility_policy":"protocol","automatic_control_available":available,
        "changed_paths":changed,"scheduled_task_operation":if available {"register-current-user-login"} else {"none-unsupported-os"},
        "remove":"hive usage monitor disable --user-root <user-root> --output json",
        "supports_current_os":available,"remote_execution_supported":false});
    let digest = sha256_digest(&serialize(&data)?);
    data["preview_digest"] = json!(digest);
    Ok(data)
}
fn task_request(config: &Configuration, operation: &str) -> Value {
    let (executable, arguments) = config
        .launcher
        .as_ref()
        .map_or((&config.executable, &config.arguments), |l| {
            (&l.executable, &l.arguments)
        });
    json!({"operation":operation,"task_name":config.task_name,"executable":executable,"arguments":arguments})
}
fn require_app_safe_control() -> Result<(), AdapterError> {
    if cfg!(windows) {
        Ok(())
    } else {
        Err(AdapterError::Unsupported(
            "session interruption supports Windows Codex only".into(),
        ))
    }
}
fn require_desktop_inspection(inspection: &Value) -> Result<(), AdapterError> {
    if inspection["available"] == true && inspection["app_count"].as_u64().is_some() {
        Ok(())
    } else {
        Err(error("cannot inspect the current-user Codex desktop"))
    }
}
fn prepare_activation(state: &mut MonitorState) {
    state.suppress_pending();
    // Only an explicit enable starts a new observation period. A controller crash
    // or upgrade must retain its prior gap and must not gain sixty extra seconds.
    state.observation_count = 0;
    state.observation_watch_started_at = None;
    state.watch_started_uptime_ms = None;
    state.last_success_uptime_ms = None;
    state.sensor_outage_stopped_at = None;
    state.observation_outage_noticed_at = None;
    state.control_connected = false;
    state.heartbeat = 0;
    state.heartbeat_uptime_ms = None;
}

fn usage_policy_disabled(root: &PinnedTarget) -> Result<bool, AdapterError> {
    Ok(!preferences::load(root)?.checks.enabled)
}

fn enable(root: &PinnedTarget, confirm: &str) -> Result<Value, AdapterError> {
    if usage_policy_disabled(root)? {
        return Err(error(
            "enable usage enforcement and explicitly select midway monitoring in Hive setup first",
        ));
    }
    require_app_safe_control()?;
    let _control = lock(root, "control.lock")?;
    if preview(root.requested_path())?["preview_digest"].as_str() != Some(confirm) {
        return Err(error("preview digest changed"));
    }
    if load::<Configuration>(root, CONFIG)?.is_some_and(|c| c.enabled) {
        return Err(error("monitor already enabled"));
    }
    let manifest: Value = load(root, ".hive/install/codex.json")?.ok_or_else(|| {
        error("install the matching Hive Codex version before enabling its monitor")
    })?;
    if manifest["product_version"] != env!("CARGO_PKG_VERSION") {
        return Err(error(
            "installed Hive Codex version differs; update it before enabling the monitor",
        ));
    }
    require_desktop_inspection(&platform::call(
        &json!({"operation":"inspect"}),
        Duration::from_secs(5),
    )?)?;
    let config = configuration(root.requested_path())?;
    let bytes = std::fs::read(std::env::current_exe().map_err(|_| error("unknown executable"))?)
        .map_err(|_| error("cannot stage monitor executable"))?;
    if sha256_digest(&bytes) != config.executable_digest {
        return Err(error("monitor executable changed after preview"));
    }
    let relative = config
        .executable
        .strip_prefix(root.requested_path())
        .map_err(|_| error("monitor executable escaped root"))?;
    let before = root.snapshot_bounded(relative, 128 * 1024 * 1024)?;
    root.publish_runtime(relative, &before, &bytes)?;
    platform::call(&task_request(&config, "register"), Duration::from_secs(15))?;
    with_state(root, |s| {
        prepare_activation(s);
        Ok(())
    })?;
    if let Err(e) = write(root, CONFIG, &config) {
        let _ = platform::call(
            &task_request(&config, "unregister"),
            Duration::from_secs(15),
        );
        return Err(e);
    }
    if let Err(e) = platform::call(&task_request(&config, "start"), Duration::from_secs(10)) {
        platform::call(
            &task_request(&config, "unregister"),
            Duration::from_secs(15),
        )?;
        let mut inactive = config.clone();
        inactive.enabled = false;
        write(root, CONFIG, &inactive)?;
        return Err(e);
    }
    Ok(
        json!({"enabled":true,"activation":"awaiting-first-observation","changed_paths":[CONFIG],"task_name":config.task_name}),
    )
}
fn disable(root: &PinnedTarget) -> Result<Value, AdapterError> {
    let _control = lock(root, "control.lock")?;
    let Some(mut config): Option<Configuration> = load(root, CONFIG)? else {
        return Ok(json!({"enabled":false}));
    };
    platform::call(
        &task_request(&config, "unregister"),
        Duration::from_secs(15),
    )?;
    config.enabled = false;
    write(root, CONFIG, &config)?;
    Ok(json!({"enabled":false,"incident_preserved":true,"changed_paths":[CONFIG]}))
}
type SampleResult = sensor::Sample;
fn sensor_worker() -> mpsc::SyncSender<(bool, mpsc::SyncSender<SampleResult>)> {
    let (tx, rx) = mpsc::sync_channel::<(bool, mpsc::SyncSender<SampleResult>)>(1);
    thread::spawn(move || {
        let mut sensor = sensor::Reader::new();
        while let Ok((fresh, reply)) = rx.recv() {
            let _ = reply.send(sensor.read(fresh));
        }
    });
    tx
}
fn inspect_async() -> Receiver<Result<Value, ()>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(
            platform::call(
                &json!({"operation":"inspect","caller_pid":std::process::id()}),
                Duration::from_secs(5),
            )
            .map_err(|_| ()),
        );
    });
    rx
}

fn controller(root: &PinnedTarget) -> Result<Value, AdapterError> {
    if usage_policy_disabled(root)? {
        return Ok(json!({"stopped":true,"reason":"usage-policy-disabled"}));
    }
    let mut report = json!({"started_at":now(),"pid":std::process::id(),
        "version":env!("HIVE_PACKAGE_VERSION"),"error":null});
    write(root, STARTUP, &report)?;
    let result = controller_scoped(root, None);
    if let Err(failure) = &result {
        report["error"] = json!(failure.message());
        let _ = write(root, STARTUP, &report);
    }
    result
}

fn controller_scoped(
    root: &PinnedTarget,
    qualification: Option<String>,
) -> Result<Value, AdapterError> {
    if usage_policy_disabled(root)? {
        return Ok(json!({"stopped":true,"reason":"usage-policy-disabled"}));
    }
    require_app_safe_control()?;
    let _lease = lock(root, "controller.lock")?;
    let config: Configuration =
        load(root, CONFIG)?.ok_or_else(|| error("monitor has not been enabled"))?;
    if !config.enabled
        || config.schema_version != 1
        || config.executable != std::env::current_exe().map_err(|_| error("unknown executable"))?
        || config.executable_digest != configuration(root.requested_path())?.executable_digest
    {
        return Err(error("monitor executable or configuration changed"));
    }
    let initial = platform::call(
        &json!({"operation":"inspect","caller_pid":std::process::id()}),
        Duration::from_secs(5),
    )?;
    require_desktop_inspection(&initial)?;
    if let Some(tick) = initial["uptime_ms"].as_u64() {
        sensor::seed_uptime(tick);
    }
    if initial["caller_within_app"] == true {
        return Err(error(
            "monitor must run independently through its scheduled task",
        ));
    }
    let pid = std::process::id();
    let identity = platform::call(
        &json!({"operation":"session-identity","pid":pid,"executable":config.executable}),
        Duration::from_secs(5),
    )?;
    with_state(root, |state| {
        state.controller_pid = Some(pid);
        state.controller_identity = identity["digest"].as_str().map(str::to_owned);
        state.runtime_version = Some(env!("HIVE_PACKAGE_VERSION").to_owned());
        Ok(())
    })?;
    let active = initial["app_count"].as_u64().is_none_or(|count| count > 0);
    let desktop = desktop::Desktop::start_scoped(qualification);
    let mut fault_notice = None;
    loop {
        if usage_policy_disabled(root)? {
            return Ok(json!({"stopped":true,"reason":"usage-policy-disabled"}));
        }
        if load::<Configuration>(root, CONFIG)?.is_some_and(|current| !current.enabled) {
            return Ok(json!({"stopped":true}));
        }
        if let Ok(value) = controller_loop(root, active, &desktop) {
            return Ok(value);
        }
        if usage_policy_disabled(root)? {
            return Ok(json!({"stopped":true,"reason":"usage-policy-disabled"}));
        }
        let selected = preferences::load(root)?;
        let incident = match with_state(root, |s| {
            if selected.stop_on_failure() {
                Ok(s.controller_fault(now()))
            } else {
                s.controller_warning(now());
                Ok(None)
            }
        }) {
            Ok(value) => value,
            Err(_) if selected.stop_on_failure() => fault_notice
                .is_none()
                .then(|| sha256_digest(b"monitor-state-repair-required")),
            Err(_) => None,
        };
        if let Some(incident) = incident {
            desktop.stop(&incident, Some(now().saturating_mul(1000)));
            if selected.notify() {
                let _notice =
                    platform::notify("monitor-fault", false, &incident, root.requested_path());
            }
            fault_notice = Some(incident);
        }
        // Recover observation without repeatedly stopping user-resumed work.
        thread::sleep(Duration::from_secs(2));
    }
}

fn collect_notice(root: &PinnedTarget, notification: &mut Option<(String, Receiver<bool>)>) {
    if let Some((digest, rx)) = &*notification {
        if let Ok(delivered) = rx.try_recv() {
            let _ = with_state(root, |s| {
                if let Some(i) = s.incident.as_mut().filter(|i| &i.digest == digest) {
                    i.notification = if delivered { "delivered" } else { "failed" }.into();
                }
                Ok(())
            });
            *notification = None;
        }
    }
}

fn record_control(
    root: &PinnedTarget,
    packet: &Value,
    active: bool,
    control_connected: bool,
) -> Result<(), AdapterError> {
    with_state(root, |s| {
        s.control_connected = control_connected;
        if let Some(version) = packet["desktop_version"].as_str().filter(|v| v.len() <= 64) {
            s.desktop_version = Some(version.into());
        }
        s.control_compatibility = Some(
            if control_connected && packet["compatibility"] == "compatible" {
                "compatible"
            } else if control_connected && packet["compatibility"] == "incompatible" {
                "incompatible"
            } else if control_connected {
                "pending"
            } else {
                "unavailable"
            }
            .into(),
        );
        s.control_stage = packet["compatibility_stage"]
            .as_str()
            .filter(|v| matches!(*v, "identity" | "initialize" | "observe" | "interrupt"))
            .map(str::to_owned);
        if let Some(reason) = packet["control_error"].as_str().filter(|v| {
            v.starts_with("desktop-")
                && v.len() <= 64
                && v.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
        }) {
            s.control_error = Some(reason.into());
        } else if control_connected {
            s.control_error = None;
        } else if s.control_error.is_none() {
            s.control_error = Some("desktop-control-disconnected".into());
        }
        s.control_stopping = packet["control_stopping"] == true;
        s.activity_observed_at =
            (control_connected && packet["activity_observed"] == true).then(now);
        s.app_alive = active;
        if !control_connected {
            if let Some(i) = s.incident.as_mut().filter(|i| i.stop_finished_at.is_none()) {
                i.stop_confirmed = false;
                i.stop_error = Some("desktop-control-unavailable".into());
            }
        }
        if control_connected {
            s.active_turns = packet["active_turns"].as_u64().unwrap_or_default();
            s.known_sessions = packet["known_sessions"].as_u64().unwrap_or_default();
            s.unknown_sessions = packet["unknown_sessions"].as_u64().unwrap_or_default();
            if let Some(i) = s.incident.as_mut() {
                if packet["stop"]["incident"].as_str() == Some(i.digest.as_str()) {
                    i.record_session_stop(&packet["stop"], now());
                    if let Some(confirmed) = packet["stop"]["confirmed_sessions"].as_array() {
                        for receipt in &mut s.threshold_receipts {
                            if i.threshold_details.iter().any(|detail| {
                                detail.session_digest == receipt.session_digest
                                    && detail.policy_digest == receipt.policy_digest
                                    && detail.turn_digest == receipt.turn_digest
                            }) && confirmed.iter().any(|value| {
                                value.as_str() == Some(receipt.session_digest.as_str())
                            }) {
                                receipt.confirmed = true;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    })
    .or_else(|failure| {
        if matches!(failure, AdapterError::Conflict(_)) {
            Ok(())
        } else {
            Err(failure)
        }
    })
}

fn record_sample(
    root: &PinnedTarget,
    measured: SampleResult,
    elapsed: u64,
    policies: &[crate::usage_control::MonitorPolicy],
) -> (bool, bool) {
    let Ok(selected) = preferences::load(root) else {
        return (false, false);
    };
    #[cfg(debug_assertions)]
    let measured = qualification::observation(root.requested_path(), measured);
    with_state(root, |state| {
        state.query_elapsed_ms = Some(elapsed);
        let result = state.record_query(measured, now());
        selected.filter_pending(state);
        if result.0 {
            threshold::observe(state, policies, &state.baseline.clone(), now());
        }
        selected.filter_pending(state);
        Ok(result)
    })
    .unwrap_or((false, false))
}

fn claim_halt(
    root: &PinnedTarget,
    digest: &str,
    selected: &preferences::Preferences,
) -> Result<Option<(state::Incident, bool)>, AdapterError> {
    with_state(root, |s| {
        selected.filter_pending(s);
        let Some(current) = s.incident.as_mut().filter(|i| i.digest == digest) else {
            return Ok(None);
        };
        if current.stop_dispatched_at.is_some() || current.policy_suppressed {
            return Ok(None);
        }
        current.stop_dispatched_at = Some(now());
        if current.reason == "usage-reset" {
            s.last_reset_dispatched_at = Some(current.detected_at);
        }
        let notify = current.notification == "pending" && selected.notify();
        if notify {
            current.notification = "requested".into();
        } else if current.notification == "pending" {
            current.notification = "suppressed".into();
        }
        Ok(Some((current.clone(), notify)))
    })
    .or_else(|failure| {
        if matches!(failure, AdapterError::Conflict(_)) {
            Ok(None)
        } else {
            Err(failure)
        }
    })
}

fn dispatch_halt(
    root: &PinnedTarget,
    desktop: &desktop::Desktop,
    incident: Option<state::Incident>,
    sent_incident: &mut Option<String>,
    notification: &mut Option<(String, Receiver<bool>)>,
) -> Result<(), AdapterError> {
    let selected = preferences::load(root)?;
    if let Some(incident) = incident {
        let Some((incident, should_notify)) = claim_halt(root, &incident.digest, &selected)? else {
            return Ok(());
        };
        if sent_incident.as_deref() != Some(incident.digest.as_str()) {
            desktop.stop_scoped(
                &incident.digest,
                Some(
                    incident
                        .detected_at_ms
                        .unwrap_or_else(|| incident.detected_at.saturating_mul(1000)),
                ),
                incident.target_sessions.as_deref(),
            );
            *sent_incident = Some(incident.digest.clone());
        }
        if should_notify {
            *notification = Some((
                incident.digest.clone(),
                platform::notify(
                    &incident.reason,
                    incident.stop_confirmed,
                    &incident.digest,
                    root.requested_path(),
                ),
            ));
        }
    } else if sent_incident.take().is_some() {
        desktop.clear();
    }
    Ok(())
}

type SampleWorker = (Instant, Receiver<SampleResult>);

type PolicyJob = (
    Vec<crate::usage::MonitorThread>,
    Vec<threshold::ActiveSession>,
    mpsc::SyncSender<(Vec<crate::usage_control::MonitorPolicy>, u64)>,
);

fn policy_worker(root: PathBuf) -> mpsc::SyncSender<PolicyJob> {
    let (send, receive) = mpsc::sync_channel::<PolicyJob>(1);
    thread::spawn(move || {
        while let Ok((contexts, active, reply)) = receive.recv() {
            let _ = reply.send(threshold::policies(&root, &contexts, &active));
        }
    });
    send
}

#[allow(clippy::too_many_lines)]
fn controller_loop(
    root: &PinnedTarget,
    mut active: bool,
    desktop: &desktop::Desktop,
) -> Result<Value, AdapterError> {
    let sensor = sensor_worker();
    let policy_worker = policy_worker(root.requested_path().to_owned());
    let mut contexts: Vec<crate::usage::MonitorThread> = Vec::new();
    let mut sessions: Vec<threshold::ActiveSession> = Vec::new();
    let mut policies = Vec::new();
    let mut policy_job = None;
    let mut next_policy = Instant::now();
    let mut policies_at = Instant::now();
    let started = Instant::now();
    let gap = with_state(root, |state| {
        Ok(state.observation_gap_monotonic(active, now(), sensor::uptime_ms()))
    })?;
    let mut cadence = state::Cadence::with_gap(gap);
    let mut sample: Option<SampleWorker> = None;
    let mut independent = false;
    let mut inspect: Option<Receiver<Result<Value, ()>>> = None;
    let mut next_inspect = started;
    let mut last_control = started
        .checked_sub(Duration::from_secs(30))
        .unwrap_or(started);
    let mut control_connected = false;
    let mut sent_incident = None;
    let mut notification: Option<(String, Receiver<bool>)> = None;
    let mut last_publish = started
        .checked_sub(Duration::from_secs(2))
        .unwrap_or(started);
    loop {
        let tick = Instant::now();
        if usage_policy_disabled(root)? {
            return Ok(json!({"stopped":true,"reason":"usage-policy-disabled"}));
        }
        collect_notice(root, &mut notification);
        let Some(current): Option<Configuration> = load(root, CONFIG)? else {
            return Err(error("monitor configuration disappeared"));
        };
        if !current.enabled {
            return Ok(json!({"stopped":true}));
        }
        if inspect.is_none() && tick >= next_inspect {
            inspect = Some(inspect_async());
            next_inspect = tick + Duration::from_secs(5);
        }
        if let Some(rx) = &inspect {
            if let Ok(result) = rx.try_recv() {
                if let Ok(value) = result {
                    if value["caller_within_app"] == true {
                        return Err(error(
                            "monitor must run independently through its scheduled task",
                        ));
                    }
                    active = value["app_count"].as_u64().is_none_or(|count| count > 0);
                } else {
                    active = true;
                }
                inspect = None;
            }
        }
        while let Ok(packet) = desktop.events.try_recv() {
            if packet["type"] == "catalog" {
                contexts = serde_json::from_value(packet["contexts"].clone()).unwrap_or_default();
                #[cfg(debug_assertions)]
                qualification::scope_contexts(root.requested_path(), &mut contexts);
                next_policy = tick;
                continue;
            }
            control_connected = packet["connected"] == true;
            if control_connected {
                last_control = tick;
                active = true;
                sessions =
                    serde_json::from_value(packet["active_sessions"].clone()).unwrap_or_default();
            }
            record_control(root, &packet, active, control_connected)?;
        }
        if policy_job.is_none() && tick >= next_policy {
            let (send, receive) = mpsc::sync_channel(1);
            let _ = policy_worker.try_send((contexts.clone(), sessions.clone(), send));
            policy_job = Some((tick, receive));
            next_policy = tick + Duration::from_secs(2);
        }
        if let Some((began, receive)) = &policy_job {
            if let Ok((updated, errors)) = receive.try_recv() {
                policies = updated;
                policies_at = tick;
                let selected = preferences::load(root)?;
                with_state(root, |state| {
                    state.threshold_policy_errors = errors;
                    selected.filter_pending(state);
                    if state
                        .last_success
                        .is_some_and(|at| now().saturating_sub(at) <= 5)
                    {
                        threshold::observe(state, &policies, &state.baseline.clone(), now());
                    }
                    selected.filter_pending(state);
                    Ok(())
                })?;
                policy_job = None;
            } else if began.elapsed() >= Duration::from_secs(5) {
                policies.clear();
                with_state(root, |state| {
                    state.threshold_policy_errors =
                        u64::try_from(sessions.len()).unwrap_or(u64::MAX);
                    Ok(())
                })?;
                policy_job = None;
            }
        }
        if policies_at.elapsed() > Duration::from_secs(20) {
            policies.clear();
        }
        if last_control.elapsed() >= Duration::from_secs(3) {
            control_connected = false;
        }
        if !active {
            cadence.idle(tick.duration_since(started));
        }
        if active && sample.is_none() && tick.duration_since(started) >= cadence.next_sample {
            let (tx, rx) = mpsc::sync_channel(1);
            let _ = sensor.try_send((independent, tx));
            sample = Some((tick, rx));
            cadence.begin(tick.duration_since(started));
        }
        if let Some((began, rx)) = &sample {
            let result = rx.try_recv();
            if result.is_ok() || began.elapsed() >= Duration::from_secs(11) {
                let elapsed = u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX);
                let measured = result.unwrap_or(sensor::Sample {
                    snapshots: Err("observation-supervisor-timeout".into()),
                    source: "unavailable",
                    primary_error: None,
                });
                let (success, needs_independent) =
                    record_sample(root, measured, elapsed, &policies);
                independent = needs_independent;
                cadence.complete(tick.duration_since(started), success);
                sample = None;
            }
        }
        if active && cadence.blind(tick.duration_since(started)) {
            let selected = preferences::load(root)?;
            with_state(root, |state| {
                if selected.stop_on_failure() {
                    state.observation_blind(now());
                } else {
                    state.observation_warning(now());
                }
                selected.filter_pending(state);
                Ok(())
            })?;
        }
        let state: MonitorState = load(root, STATE)?.unwrap_or_else(MonitorState::new);
        dispatch_halt(
            root,
            desktop,
            state.incident,
            &mut sent_incident,
            &mut notification,
        )?;
        if last_publish.elapsed() >= Duration::from_secs(1) {
            let _ = with_state(root, |s| {
                s.heartbeat = now();
                s.heartbeat_uptime_ms = sensor::uptime_ms();
                s.control_connected = control_connected;
                if !control_connected {
                    s.control_compatibility = Some("unavailable".into());
                }
                s.app_alive = active;
                let _ = s.observation_gap_monotonic(active, now(), sensor::uptime_ms());
                Ok(())
            });
            last_publish = tick;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opted_in_user_policy() -> Value {
        let mut guard = serde_json::to_value(hive_core::preferences::UsageProtection::from_legacy(
            true, true, true,
        ))
        .unwrap();
        guard["stop_remaining_percent"] = json!(20);
        json!({"schema_version":2,"interface_language":"en","wiki":{"language":"en"},
            "profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},
            "selected_hosts":["codex"],"skills":{"mode":"all"},"usage_guard":guard})
    }

    #[test]
    fn explicit_master_disable_prevents_monitor_start_or_registration() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(".hive/config/user-setup.yml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut config = opted_in_user_policy();
        let root = PinnedTarget::open_usage(directory.path()).unwrap();
        for (master, midway) in [(false, true), (true, false)] {
            config["usage_guard"]["enforcement_enabled"] = json!(master);
            config["usage_guard"]["midway"]["enabled"] = json!(midway);
            std::fs::write(&path, serde_yaml::to_string(&config).unwrap()).unwrap();
            assert!(usage_policy_disabled(&root).unwrap());
            assert_eq!(
                controller(&root).unwrap()["reason"],
                "usage-policy-disabled"
            );
            assert!(enable(&root, "unconfirmed").is_err());
            assert!(!directory.path().join(".hive/runtime").exists());
        }
        config["usage_guard"]["midway"]["enabled"] = json!(true);
        std::fs::write(&path, serde_yaml::to_string(&config).unwrap()).unwrap();
        assert!(!usage_policy_disabled(&root).unwrap());
        std::fs::write(&path, "broken: policy").unwrap();
        assert!(usage_policy_disabled(&root).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(usage_policy_disabled(&root).is_err());
    }
    #[test]
    fn live_contract_failure_and_disconnect_never_reuse_compatible_state() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        record_control(&root, &json!({"desktop_version":"99.0.0.0","compatibility":"compatible","compatibility_stage":"observe","activity_observed":true}), true, true).unwrap();
        let state: MonitorState = load(&root, STATE).unwrap().unwrap();
        assert_eq!(state.control_compatibility.as_deref(), Some("compatible"));
        assert!(state.incident.is_none());
        record_control(&root, &json!({"compatibility":"incompatible","compatibility_stage":"interrupt","control_error":"desktop-interrupt-protocol-mismatch"}), true, true).unwrap();
        let state: MonitorState = load(&root, STATE).unwrap().unwrap();
        assert_eq!(state.control_compatibility.as_deref(), Some("incompatible"));
        record_control(&root, &json!({"connected":false}), true, false).unwrap();
        let state: MonitorState = load(&root, STATE).unwrap().unwrap();
        assert_eq!(state.control_compatibility.as_deref(), Some("unavailable"));
        assert_eq!(
            state.control_error.as_deref(),
            Some("desktop-interrupt-protocol-mismatch")
        );
        assert!(state.activity_observed_at.is_none());
    }
    #[test]
    fn explicit_reenable_does_not_reuse_a_disabled_observation_gap() {
        let mut state = MonitorState::new();
        state.last_success = Some(100);
        state.observation_count = 3;
        state.watch_started_uptime_ms = Some(90_000);
        state.last_success_uptime_ms = Some(100_000);
        state.observation_blind(101);
        let incident = serde_json::to_value(&state.incident).unwrap();
        prepare_activation(&mut state);
        assert_eq!(state.observation_count, 0);
        assert_eq!(state.last_success, Some(100));
        assert_eq!(
            serde_json::to_value(&state.incident).unwrap()["digest"],
            incident["digest"]
        );
        assert!(state.incident.as_ref().unwrap().policy_suppressed);
        assert_eq!(
            state.observation_gap_monotonic(true, 900, Some(900_000)),
            Duration::ZERO
        );
        assert_eq!(
            state.observation_gap_monotonic(true, 959, Some(959_999)),
            Duration::from_millis(59_999)
        );
        assert_eq!(
            state.observation_gap_monotonic(true, 960, Some(960_000)),
            Duration::from_secs(BLIND_SECONDS)
        );
        state.observation_blind(960);
        assert_eq!(state.generation, 2);
        state.observation_blind(961);
        assert_eq!(state.generation, 2);
    }

    #[test]
    fn configured_monitor_identifies_the_actual_package_build() {
        let temporary = tempfile::tempdir().unwrap();
        let config = configuration(temporary.path()).unwrap();
        assert_eq!(config.version, env!("HIVE_PACKAGE_VERSION"));
    }

    #[test]
    fn enabled_configuration_is_not_reported_as_a_running_observer() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let mut config = configuration(temporary.path()).unwrap();
        config.enabled = true;
        write(&root, CONFIG, &config).unwrap();
        let status = monitor_status(&root).unwrap();
        assert_eq!(status["configuration"]["enabled"], true);
        assert_eq!(status["controller_alive"], false);
        assert_eq!(status["sensor_health"], "stopped");
        assert_eq!(status["monitoring_verified"], false);
        assert!(status["running_monitor_version"].is_null());
    }
    #[test]
    fn rejected_startup_is_visible_without_publishing_a_stop_or_changing_choice() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let user_policy = temporary.path().join(".hive/config/user-setup.yml");
        std::fs::create_dir_all(user_policy.parent().unwrap()).unwrap();
        std::fs::write(
            user_policy,
            serde_yaml::to_string(&opted_in_user_policy()).unwrap(),
        )
        .unwrap();
        let mut config = configuration(temporary.path()).unwrap();
        config.enabled = false;
        write(&root, CONFIG, &config).unwrap();
        assert!(controller(&root).is_err());
        let status = monitor_status(&root).unwrap();
        assert!(status["startup"]["error"].is_string());
        assert_eq!(status["startup"]["version"], env!("HIVE_PACKAGE_VERSION"));
        assert_eq!(status["configuration"]["enabled"], false);
        assert!(!temporary.path().join(STATE).exists());
        assert_eq!(status["monitoring_verified"], false);
    }
    #[test]
    fn preview_is_nonmutating_and_activation_requires_matching_installation() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let view = preview(temporary.path()).unwrap();
        assert_eq!(view["automatic_control_available"], cfg!(windows));
        assert_eq!(view["session_interrupt"], cfg!(windows));
        assert_eq!(view["configuration"]["enabled"], cfg!(windows));
        assert_eq!(view["supports_current_os"], cfg!(windows));
        assert_eq!(view["emergency_termination"], false);
        assert!(!temporary.path().join(".hive").exists());
        assert!(enable(&root, view["preview_digest"].as_str().unwrap()).is_err());
        assert!(controller(&root).is_err());
        assert!(!temporary.path().join(CONFIG).exists());
    }
    #[test]
    fn desktop_inspection_does_not_gate_on_package_version() {
        for version in ["26.1013.4894.0", "26.1016.8120.0", "99.0.0.0"] {
            assert!(require_desktop_inspection(
                &json!({"available":true,"app_count":1,"desktop_package_version":version})
            )
            .is_ok());
        }
        assert!(
            require_desktop_inspection(&json!({"desktop_package_version":"26.1013.4894.0"}))
                .is_err()
        );
        assert!(require_desktop_inspection(&json!({"available":false,"app_count":1})).is_err());
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "four real native observations, fifteen seconds between starts"]
    fn live_monitor_sensor_sustains_four_fresh_observations() {
        let mut sensor = crate::usage::MonitorSensor::new();
        let mut state = MonitorState::new();
        for index in 0..4 {
            let began = Instant::now();
            let snapshot = sensor.read().expect("live bounded native observation");
            let elapsed = began.elapsed();
            assert!(elapsed < Duration::from_secs(5));
            let current = snapshot.core_snapshots();
            println!(
                "observation timestamps: {:?}",
                current
                    .iter()
                    .map(|s| (s.measured_at_unix_seconds, s.resets_at_unix_seconds))
                    .collect::<Vec<_>>()
            );
            state
                .observe(&current, now())
                .expect("live observation accepted by monitor state");
            println!("native observation {index}: {} ms", elapsed.as_millis());
            if index < 3 {
                thread::sleep(Duration::from_secs(15).saturating_sub(began.elapsed()));
            }
        }
    }
    #[test]
    fn unsupported_and_malformed_commands_cannot_enable_termination() {
        for args in [
            vec!["enable"],
            vec!["run", "--confirm-session-interruption"],
            vec!["status", "--output", "json", "--incident", "x"],
        ] {
            assert!(
                parse_and_run(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err()
            );
        }
    }
    #[test]
    fn dispatch_claim_is_atomic_and_never_replays_the_same_event() {
        let selected = preferences::Preferences {
            checks: hive_core::preferences::UsageProtection::from_legacy(true, true, true).midway,
            popup_enabled: true,
        };
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let incident = with_state(&root, |s| {
            s.halt("usage-reset", 100);
            Ok(s.incident.as_ref().unwrap().digest.clone())
        })
        .unwrap();
        let held = lock(&root, "state.lock").unwrap();
        assert!(claim_halt(&root, &incident, &selected)
            .unwrap()
            .map(|(_, notify)| notify)
            .is_none());
        assert!(record_control(&root, &json!({"control_stopping":false}), true, true).is_ok());
        drop(held);
        assert_eq!(
            claim_halt(&root, &incident, &selected)
                .unwrap()
                .map(|(_, notify)| notify),
            Some(true)
        );
        assert_eq!(
            claim_halt(&root, &incident, &selected)
                .unwrap()
                .map(|(_, notify)| notify),
            None
        );
        with_state(&root, |s| {
            s.incident = None;
            Ok(())
        })
        .unwrap();
        assert!(claim_halt(&root, &incident, &selected)
            .unwrap()
            .map(|(_, notify)| notify)
            .is_none());
        assert!(load::<MonitorState>(&root, STATE)
            .unwrap()
            .unwrap()
            .incident
            .is_none());
    }

    #[test]
    fn dispatch_claim_uses_filtered_targets_and_requires_popup_choice() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let mut selected = preferences::Preferences {
            checks: hive_core::preferences::UsageProtection::from_legacy(true, true, true).midway,
            popup_enabled: false,
        };
        let digest = with_state(&root, |state| {
            state.halt("usage-reset", 100);
            Ok(state.incident.as_ref().unwrap().digest.clone())
        })
        .unwrap();
        let (claimed, notify) = claim_halt(&root, &digest, &selected).unwrap().unwrap();
        assert!(!notify);
        assert_eq!(claimed.notification, "suppressed");
        assert!(claimed.stop_dispatched_at.is_some());
        selected.checks.reset_enabled = false;
        selected.popup_enabled = true;
        let digest = with_state(&root, |state| {
            state.halt("usage-reset", 101);
            Ok(state.incident.as_ref().unwrap().digest.clone())
        })
        .unwrap();
        assert!(claim_halt(&root, &digest, &selected).unwrap().is_none());
        let history = load::<MonitorState>(&root, STATE)
            .unwrap()
            .unwrap()
            .incident
            .unwrap();
        assert_eq!(history.digest, digest);
        assert!(history.policy_suppressed);
        assert_eq!(history.stop_dispatched_at, None);
    }
    #[test]
    fn no_monitor_event_is_a_dispatch_permission_or_resume_lock() {
        let source = include_str!("usage_control.rs");
        assert!(!source.contains("usage_monitor::check_dispatch"));
        assert!(!HELP.contains("acknowledge"));
        for args in [vec!["acknowledge"], vec!["status", "--incident", "old"]] {
            assert!(
                parse_and_run(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err()
            );
        }
    }
}
