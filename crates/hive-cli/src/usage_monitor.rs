//! Independent Windows Codex interruption. Never starts model work or persists transcripts.
mod desktop;
mod platform;
mod state;

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
const HELP: &str = "hive usage monitor preview|enable|disable|status|acknowledge --user-root <dir> [--confirm <preview-digest> --confirm-session-interruption] [--incident <digest>] --output json\n";

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
    root.read_optional(Path::new(path), 65536)?
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
    if state.schema_version != 1 {
        return Err(error("unsupported monitor state"));
    }
    let result = apply(&mut state)?;
    write(root, STATE, &state)?;
    Ok(result)
}

/// Called by both initial enforce and the final authorization check. No bypass by a new chat.
pub(crate) fn check_dispatch(user_root: Option<&Path>) -> Result<(), AdapterError> {
    let default_root =
        std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let Some(path) = user_root.or(default_root.as_deref()) else {
        return Ok(());
    };
    let root = PinnedTarget::open_usage(path)?;
    let Some(config): Option<Configuration> = load(&root, CONFIG)? else {
        return Ok(());
    };
    let state: MonitorState = load(&root, STATE)?.unwrap_or_else(MonitorState::new);
    if config.schema_version != 1
        || state.schema_version != 1
        || state.incident.is_some()
        || state.heartbeat > now()
        || state.last_success.is_some_and(|at| at > now())
    {
        return Err(error(
            "shared usage monitor halt requires exact incident acknowledgement",
        ));
    }
    if config.enabled && (!state.control_connected || state.control_stopping) {
        return Err(error(
            "usage monitor desktop control has not released the session",
        ));
    }
    if config.enabled
        && (state
            .last_success
            .is_none_or(|at| now().saturating_sub(at) >= BLIND_SECONDS)
            || now().saturating_sub(state.heartbeat) >= BLIND_SECONDS)
    {
        return Err(error("usage monitor has no fresh observation or heartbeat"));
    }
    Ok(())
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
        && matches!(
            args.first().map(String::as_str),
            Some("enable" | "disable" | "acknowledge")
        )
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
    if !matches!(
        action,
        "preview" | "enable" | "disable" | "status" | "acknowledge" | "run"
    ) {
        return Err(AdapterError::Input(HELP.into()));
    }
    let mut root = None;
    let mut confirm = None;
    let mut incident = None;
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
            "--incident" if incident.is_none() => incident = Some(value.as_str()),
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
        || (action == "acknowledge") != incident.is_some()
    {
        return Err(error("monitor confirmation does not match action"));
    }
    let root_path = root.ok_or_else(|| AdapterError::Input("--user-root is required".into()))?;
    let root = PinnedTarget::open_usage(&root_path)?;
    root.read_required(Path::new(".hive/config/user-setup.yml"), 65536)?;
    match action {
        "preview" => preview(root.requested_path()),
        "status" => {
            let configuration: Option<Configuration> = load(&root, CONFIG)?;
            let state: MonitorState = load(&root, STATE)?.unwrap_or_else(MonitorState::new);
            Ok(
                json!({"configuration":configuration,"state":state,"observation_gap_seconds":state.last_success.map(|at| now().saturating_sub(at)),
                "monitoring_verified": configuration.as_ref().is_some_and(|c| c.enabled) && now().saturating_sub(state.heartbeat) < BLIND_SECONDS
                    && state.last_success.is_some_and(|at| now().saturating_sub(at) < BLIND_SECONDS) && state.incident.is_none() && state.control_connected && state.unknown_sessions == 0,
                "scope":"local-windows-codex","server_billing_interruption_verified":false}),
            )
        }
        "enable" => enable(&root, confirm.unwrap_or_default()),
        "disable" => disable(&root),
        "acknowledge" => acknowledge(&root, incident.unwrap_or_default()),
        "run" => controller(&root),
        _ => unreachable!(),
    }
}

fn configuration(root: &Path) -> Result<Configuration, AdapterError> {
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
    let task_key = sha256_digest(root_text.to_lowercase().as_bytes());
    let installed_executable = root.join(BASE).join("bin").join(format!(
        "hive-{}-{}.exe",
        env!("CARGO_PKG_VERSION"),
        &digest[7..23]
    ));
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
        version: env!("CARGO_PKG_VERSION").into(),
        executable: installed_executable,
        executable_digest: digest,
        task_name: format!("AigentHive-UsageMonitor-{}", &task_key[7..23]),
        arguments: format!("usage monitor run --user-root \"{root_text}\" --output json"),
        launcher,
    })
}
fn preview(root: &Path) -> Result<Value, AdapterError> {
    let config = configuration(root)?;
    let mut data = json!({"configuration":config,"adapter_digest":sha256_digest(platform::SCRIPT.as_bytes()),
        "poll_seconds":POLL_SECONDS,"sample_timeout_seconds":5,"retry_seconds":2,"unresponsive_seconds":BLIND_SECONDS,
        "scope":"all-local-chats-in-current-user-OpenAI.Codex-package","emergency_termination":false,"session_interrupt":true,"app_preserved":true,"qualified_desktop_package_version":"26.1007.9446.0",
        "changed_paths":[CONFIG,STATE,config.executable],"scheduled_task_operation":"register-current-user-at-logon-and-start",
        "remove":"hive usage monitor disable --user-root <user-root> --output json",
        "supports_current_os":cfg!(windows),"remote_execution_supported":false});
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
fn enable(root: &PinnedTarget, confirm: &str) -> Result<Value, AdapterError> {
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
        s.observation_count = 0;
        s.control_connected = false;
        s.heartbeat = 0;
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
fn acknowledge(root: &PinnedTarget, incident: &str) -> Result<Value, AdapterError> {
    // Lock covers sampling too: no concurrent reset can be cleared by this acknowledgement.
    let mut receipt = with_state(root, |state| {
        if state.incident.as_ref().map(|i| i.digest.as_str()) != Some(incident) {
            return Err(error("wrong incident digest"));
        }
        let snapshots = crate::usage::monitor_snapshot()
            .map_err(|_| error("fresh usage observation failed; halt retained"))?
            .core_snapshots();
        state.acknowledge(incident, &snapshots, now())?;
        Ok(
            json!({"acknowledged":true,"auto_resume":false,"generation":state.generation,"changed_paths":[STATE]}),
        )
    })?;
    if load::<Configuration>(root, CONFIG)?.is_some_and(|c| c.enabled) {
        let deadline = Instant::now() + Duration::from_secs(3);
        let released = loop {
            let state: MonitorState =
                load(root, STATE)?.ok_or_else(|| error("monitor state disappeared"))?;
            if state.control_connected && !state.control_stopping {
                break true;
            }
            if Instant::now() >= deadline {
                break false;
            }
            thread::sleep(Duration::from_millis(50));
        };
        receipt["controller_release_verified"] = json!(released);
    }
    Ok(receipt)
}

type SampleResult = Result<Vec<UsageSnapshot>, crate::usage::SensorError>;
fn sensor_worker() -> mpsc::SyncSender<mpsc::SyncSender<SampleResult>> {
    let (tx, rx) = mpsc::sync_channel::<mpsc::SyncSender<SampleResult>>(1);
    thread::spawn(move || {
        let mut sensor = crate::usage::MonitorSensor::new();
        while let Ok(reply) = rx.recv() {
            let _ = reply.send(sensor.read().map(|s| s.core_snapshots()));
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
    if initial["caller_within_app"] == true {
        return Err(error(
            "monitor must run independently through its scheduled task",
        ));
    }
    let active = initial["app_count"].as_u64().is_none_or(|count| count > 0);
    let desktop = desktop::Desktop::start();
    let mut fault_notice = None;
    loop {
        if let Ok(value) = controller_loop(root, active, &desktop) {
            return Ok(value);
        }
        let incident = with_state(root, |s| {
            s.halt("monitor-fault", now());
            Ok(s.incident.as_ref().map(|i| i.digest.clone()))
        })
        .ok()
        .flatten()
        .unwrap_or_else(|| sha256_digest(b"monitor-state-repair-required"));
        desktop.stop(&incident);
        if fault_notice.as_deref() != Some(incident.as_str()) {
            let _notice =
                platform::notify("monitor-fault", false, &incident, root.requested_path());
            fault_notice = Some(incident);
        }
        // Keep session cancellation alive even when persistent state cannot be read.
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
        s.control_stopping = packet["control_stopping"] == true;
        s.app_alive = active;
        if !control_connected {
            if let Some(i) = s.incident.as_mut() {
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

fn record_sample(root: &PinnedTarget, measured: SampleResult, elapsed: u64) -> bool {
    with_state(root, |state| {
        state.query_elapsed_ms = Some(elapsed);
        let outcome = measured.map_err(|e| e.to_string()).and_then(|snapshots| {
            state
                .observe(&snapshots, now())
                .map_err(|e| e.message().to_owned())
        });
        state.last_query_error = outcome.as_ref().err().cloned();
        Ok(outcome.is_ok())
    })
    .unwrap_or(false)
}

fn claim_halt(root: &PinnedTarget, digest: &str) -> Result<Option<bool>, AdapterError> {
    with_state(root, |s| {
        let Some(current) = s.incident.as_mut().filter(|i| i.digest == digest) else {
            return Ok(None);
        };
        let notify = current.notification == "pending";
        if notify {
            current.notification = "requested".into();
        }
        Ok(Some(notify))
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
    if let Some(incident) = incident {
        let Some(should_notify) = claim_halt(root, &incident.digest)? else {
            return Ok(());
        };
        if sent_incident.as_deref() != Some(incident.digest.as_str()) {
            desktop.stop(&incident.digest);
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

fn controller_loop(
    root: &PinnedTarget,
    mut active: bool,
    desktop: &desktop::Desktop,
) -> Result<Value, AdapterError> {
    let sensor = sensor_worker();
    let started = Instant::now();
    let mut cadence = state::Cadence::new();
    let mut sample: Option<SampleWorker> = None;
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
            control_connected = packet["connected"] == true;
            if control_connected {
                last_control = tick;
                active = true;
            }
            record_control(root, &packet, active, control_connected)?;
        }
        if last_control.elapsed() >= Duration::from_secs(3) {
            control_connected = false;
        }
        if !active {
            cadence.idle(tick.duration_since(started));
        }
        if active && sample.is_none() && tick.duration_since(started) >= cadence.next_sample {
            let (tx, rx) = mpsc::sync_channel(1);
            let _ = sensor.try_send(tx);
            sample = Some((tick, rx));
            cadence.begin(tick.duration_since(started));
        }
        if let Some((began, rx)) = &sample {
            let result = rx.try_recv();
            if result.is_ok() || began.elapsed() >= Duration::from_secs(5) {
                let elapsed = u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX);
                let measured = result.unwrap_or(Err(crate::usage::SensorError::Timeout));
                let success = record_sample(root, measured, elapsed);
                cadence.complete(tick.duration_since(started), success);
                sample = None;
            }
        }
        if active && cadence.blind(tick.duration_since(started)) {
            with_state(root, |state| {
                state.halt("sensor-unresponsive", now());
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
                s.control_connected = control_connected;
                s.app_alive = active;
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
    #[cfg(windows)]
    #[test]
    #[ignore = "read-only qualification against the currently running Windows desktop"]
    fn live_desktop_connection_observes_active_sessions_without_stopping_them() {
        let desktop = desktop::Desktop::start();
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let packet = desktop
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap();
            if packet["connected"] == true
                && packet["unknown_sessions"] == 0
                && packet["known_sessions"].as_u64().unwrap_or_default() > 0
            {
                println!(
                    "qualified desktop: sessions={}, active={}, app_alive={}",
                    packet["known_sessions"], packet["active_turns"], packet["app_alive"]
                );
                assert_eq!(packet["app_alive"], true);
                break;
            }
        }
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "four real native observations, fifteen seconds between starts"]
    fn live_monitor_sensor_sustains_four_fresh_observations() {
        let mut sensor = crate::usage::MonitorSensor::new();
        for index in 0..4 {
            let began = Instant::now();
            let snapshot = sensor.read().expect("live bounded native observation");
            let elapsed = began.elapsed();
            assert!(elapsed < Duration::from_secs(5));
            assert!(!snapshot.core_snapshots().is_empty());
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
    fn acknowledgement_lock_and_stale_incident_cannot_create_another_halt() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let incident = with_state(&root, |s| {
            s.halt("usage-reset", 100);
            Ok(s.incident.as_ref().unwrap().digest.clone())
        })
        .unwrap();
        let held = lock(&root, "state.lock").unwrap();
        assert!(claim_halt(&root, &incident).unwrap().is_none());
        assert!(record_control(&root, &json!({"control_stopping":false}), true, true).is_ok());
        drop(held);
        assert_eq!(claim_halt(&root, &incident).unwrap(), Some(true));
        assert_eq!(claim_halt(&root, &incident).unwrap(), Some(false));
        with_state(&root, |s| {
            s.incident = None;
            Ok(())
        })
        .unwrap();
        assert!(claim_halt(&root, &incident).unwrap().is_none());
        assert!(load::<MonitorState>(&root, STATE)
            .unwrap()
            .unwrap()
            .incident
            .is_none());
    }
    #[test]
    fn shared_incident_blocks_even_when_monitor_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let mut config = configuration(root.requested_path()).unwrap();
        config.enabled = false;
        write(&root, CONFIG, &config).unwrap();
        with_state(&root, |state| {
            state.halt("usage-reset", 100);
            Ok(())
        })
        .unwrap();
        assert!(check_dispatch(Some(temp.path())).is_err());
    }
    #[test]
    fn duplicate_controllers_cannot_hold_same_lease() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let lease = lock(&root, "controller.lock").unwrap();
        assert!(lock(&root, "controller.lock").is_err());
        drop(lease);
        assert!(lock(&root, "controller.lock").is_ok());
    }

    #[test]
    fn enabling_against_an_older_install_cannot_register_a_task_or_stage_a_binary() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        std::fs::create_dir_all(temp.path().join(".hive/install")).unwrap();
        std::fs::write(
            temp.path().join(".hive/install/codex.json"),
            br#"{"product_version":"0.12.0"}"#,
        )
        .unwrap();
        let preview = preview(root.requested_path()).unwrap();
        assert!(enable(&root, preview["preview_digest"].as_str().unwrap()).is_err());
        assert!(!temp.path().join(CONFIG).exists());
        assert!(!temp.path().join(BASE).join("bin").exists());
    }
}
