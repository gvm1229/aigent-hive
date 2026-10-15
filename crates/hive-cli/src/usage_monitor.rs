//! Independent Windows Codex interruption. Never starts model work or reads transcripts.
mod platform;
mod state;

use crate::run::{AdapterError, FileSnapshot, PinnedTarget};
use crate::{emit_action_result, ActionResult};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
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
const HELP: &str = "hive usage monitor preview|enable|disable|status|acknowledge --user-root <dir> [--confirm <preview-digest> --confirm-app-termination] [--incident <digest>] --output json\n";

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
    if matches!(root.snapshot(&path)?, FileSnapshot::Missing) {
        root.publish_runtime(&path, &FileSnapshot::Missing, b"monitor-lock")?;
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).follow(FollowSymlinks::No);
    let file = root
        .target_dir()
        .open_with(path, &options)
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
        if option == "--confirm-app-termination" {
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
                    && state.last_success.is_some_and(|at| now().saturating_sub(at) < BLIND_SECONDS),
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
    Ok(Configuration {
        schema_version: 1,
        enabled: true,
        version: env!("CARGO_PKG_VERSION").into(),
        executable: installed_executable,
        executable_digest: digest,
        task_name: format!("AigentHive-UsageMonitor-{}", &task_key[7..23]),
        arguments: format!("usage monitor run --user-root \"{root_text}\" --output json"),
    })
}
fn preview(root: &Path) -> Result<Value, AdapterError> {
    let config = configuration(root)?;
    let mut data = json!({"configuration":config,"adapter_digest":sha256_digest(platform::SCRIPT.as_bytes()),
        "poll_seconds":POLL_SECONDS,"sample_timeout_seconds":5,"retry_seconds":2,"unresponsive_seconds":BLIND_SECONDS,
        "scope":"all-local-chats-in-current-user-OpenAI.Codex-package","emergency_termination":true,
        "changed_paths":[CONFIG,STATE,config.executable],"scheduled_task_operation":"register-current-user-at-logon-and-start",
        "remove":"hive usage monitor disable --user-root <user-root> --output json",
        "supports_current_os":cfg!(windows),"remote_execution_supported":false});
    let digest = sha256_digest(&serialize(&data)?);
    data["preview_digest"] = json!(digest);
    Ok(data)
}
fn task_request(config: &Configuration, operation: &str) -> Value {
    json!({"operation":operation,"task_name":config.task_name,"executable":config.executable,"arguments":config.arguments})
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
    with_state(root, |state| {
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
    })
}

fn sample_async() -> Receiver<Result<Vec<UsageSnapshot>, ()>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(
            crate::usage::monitor_snapshot()
                .map(|s| s.core_snapshots())
                .map_err(|_| ()),
        );
    });
    rx
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
    let mut roots = initial["roots"].clone();
    let outcome = controller_loop(root, &mut roots, active);
    if outcome.is_err() {
        // Once armed, corrupt state or I/O failure must not silently leave inference running.
        let _ = with_state(root, |s| {
            s.halt("monitor-fault", now());
            Ok(())
        });
        let result = platform::call(
            &json!({"operation":"halt","roots":roots}),
            Duration::from_secs(5),
        );
        let verified = result.ok().is_some_and(|v| v["verified"] == true);
        let incident = load::<MonitorState>(root, STATE)
            .ok()
            .flatten()
            .and_then(|s| s.incident)
            .map_or_else(|| "state-repair-required".into(), |i| i.digest);
        let _notice = platform::notify("monitor-fault", verified, &incident);
        // Let the independent notification start before task recovery restarts this controller.
        thread::sleep(Duration::from_millis(100));
    }
    outcome
}

fn stop_incident(
    root: &PinnedTarget,
    roots: &Value,
    incident: &state::Incident,
) -> Result<(bool, bool), AdapterError> {
    stop_incident_with(root, roots, incident, |request| {
        platform::call(request, Duration::from_secs(5))
    })
}

fn stop_incident_with(
    root: &PinnedTarget,
    roots: &Value,
    incident: &state::Incident,
    halt: impl FnOnce(&Value) -> Result<Value, AdapterError>,
) -> Result<(bool, bool), AdapterError> {
    with_state(root, |state| {
        let Some(current) = state
            .incident
            .as_mut()
            .filter(|i| i.digest == incident.digest)
        else {
            return Ok((false, false));
        };
        let scope_digest = sha256_digest(&serialize(roots)?);
        // A process identity that has exited cannot return. Ignore an inspection completed
        // before our stop; a restarted app has a different PID/creation-time scope.
        if current.termination_verified
            && current.last_verified_scope.as_deref() == Some(scope_digest.as_str())
        {
            return Ok((false, true));
        }
        let outcome = if roots.as_array().is_some_and(|r| !r.is_empty()) {
            halt(&json!({"operation":"halt","roots":roots}))
        } else {
            Err(error(
                "no authenticated app process identities; halt unverified",
            ))
        };
        let verified = outcome
            .as_ref()
            .ok()
            .and_then(|v| v["verified"].as_bool())
            .unwrap_or(false);
        let elapsed = outcome.as_ref().ok().and_then(|v| v["elapsed_ms"].as_u64());
        let targeted = outcome.as_ref().ok().and_then(|v| v["targeted"].as_u64());
        current.record_termination(scope_digest, verified, targeted, elapsed, now());
        let notify = current.notification == "pending";
        current.stop_method = "windows-emergency".into();
        if notify {
            current.notification = "requested".into();
        }
        state.heartbeat = now();
        Ok((notify, current.termination_verified))
    })
}

type SampleWorker = (Instant, Receiver<Result<Vec<UsageSnapshot>, ()>>);

fn controller_loop(
    root: &PinnedTarget,
    roots: &mut Value,
    mut active: bool,
) -> Result<Value, AdapterError> {
    let started = Instant::now();
    let mut cadence = state::Cadence::new();
    let mut sample: Option<SampleWorker> = None;
    let mut inspect: Option<Receiver<Result<Value, ()>>> = None;
    let mut next_inspect = started;
    let mut notification: Option<(String, Receiver<bool>)> = None;
    let mut last_publish = started
        .checked_sub(Duration::from_secs(2))
        .unwrap_or(started);
    loop {
        let tick = Instant::now();
        if let Some((digest, rx)) = &notification {
            if let Ok(delivered) = rx.try_recv() {
                let _ = with_state(root, |s| {
                    if let Some(i) = s.incident.as_mut().filter(|i| &i.digest == digest) {
                        i.notification = if delivered { "delivered" } else { "failed" }.into();
                    }
                    Ok(())
                });
                notification = None;
            }
        }
        let Some(current): Option<Configuration> = load(root, CONFIG)? else {
            return Err(error("monitor configuration disappeared"));
        };
        if !current.enabled {
            return Ok(json!({"stopped":true}));
        }
        if inspect.is_none() && tick >= next_inspect {
            inspect = Some(inspect_async());
            next_inspect = tick + Duration::from_secs(2);
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
                    *roots = value["roots"].clone();
                } else {
                    active = true;
                }
                inspect = None;
            }
        }
        if !active {
            cadence.idle(tick.duration_since(started));
        }
        if active && sample.is_none() && tick.duration_since(started) >= cadence.next_sample {
            sample = Some((tick, sample_async()));
            cadence.begin(tick.duration_since(started));
        }
        if let Some((began, rx)) = &sample {
            let result = rx.try_recv();
            if result.is_ok() || began.elapsed() >= Duration::from_secs(5) {
                let success = if let Ok(Ok(snapshots)) = result {
                    with_state(root, |state| state.observe(&snapshots, now())).is_ok()
                } else {
                    false
                };
                cadence.complete(tick.duration_since(started), success);
                // Native queries are bounded; a timed out query cannot hold this controller.
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
        if let Some(incident) = state.incident {
            if active {
                let (should_notify, verified) = stop_incident(root, roots, &incident)?;
                if should_notify {
                    notification = Some((
                        incident.digest.clone(),
                        platform::notify(&incident.reason, verified, &incident.digest),
                    ));
                }
                active = false;
                next_inspect = Instant::now() + Duration::from_secs(1);
            }
        }
        if last_publish.elapsed() >= Duration::from_secs(1) {
            // Another command may hold the state lock briefly. Never overwrite its generation.
            let _ = with_state(root, |s| {
                s.heartbeat = now();
                Ok(())
            });
            last_publish = tick;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_and_malformed_commands_cannot_enable_termination() {
        for args in [
            vec!["enable"],
            vec!["run", "--confirm-app-termination"],
            vec!["status", "--output", "json", "--incident", "x"],
        ] {
            assert!(
                parse_and_run(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err()
            );
        }
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

    #[test]
    fn usage_monitor_exit_receipt_survives_stale_inspection_but_restart_is_stopped_again() {
        let temporary = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temporary.path()).unwrap();
        let incident = with_state(&root, |s| {
            s.halt("usage-reset", 100);
            Ok(s.incident.clone().unwrap())
        })
        .unwrap();
        let scope = json!([{"pid":1,"created_ticks":100}]);
        let result = stop_incident_with(&root, &scope, &incident, |_| {
            Ok(json!({"verified":true,"targeted":3,"elapsed_ms":200}))
        })
        .unwrap();
        assert_eq!(result, (true, true));
        let duplicate = stop_incident_with(&root, &scope, &incident, |_| {
            panic!("must not retry an exited identity")
        })
        .unwrap();
        assert_eq!(duplicate, (false, true));
        let restarted = json!([{"pid":1,"created_ticks":200}]);
        let failed = stop_incident_with(&root, &restarted, &incident, |_| {
            Ok(json!({"verified":false,"targeted":1,"elapsed_ms":400}))
        })
        .unwrap();
        assert_eq!(failed, (false, false));
        let saved: MonitorState = load(&root, STATE).unwrap().unwrap();
        let saved = saved.incident.unwrap();
        assert!(!saved.termination_verified);
        assert_eq!(saved.confirmed_termination.unwrap().targeted_processes, 3);
    }
}
