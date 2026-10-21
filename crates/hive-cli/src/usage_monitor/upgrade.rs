//! Preserve the monitor choice and history while replacing its version-bound executable/task.
use super::{
    configuration, error, load, lock, monitor_root_path, monitor_status, monitor_task_name, now,
    platform, require_app_safe_control, require_desktop_version, task_request, with_state, write,
    Configuration, BASE, CONFIG,
};
use crate::run::{AdapterError, PinnedTarget};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};

pub(super) const JOURNAL: &str = ".hive/runtime/usage-monitor/upgrade.json";

pub(crate) fn upgrade_existing(user_root: &Path) -> Result<Option<Value>, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    match load::<Configuration>(&root, CONFIG)? {
        None => return Ok(None),
        Some(config) if config.enabled => require_app_safe_control()?,
        Some(_) => {}
    }
    upgrade_with(&root, platform::call, await_ready).map(Some)
}

fn stage(root: &PinnedTarget, config: &Configuration) -> Result<(), AdapterError> {
    let bytes = std::fs::read(std::env::current_exe().map_err(|_| error("unknown executable"))?)
        .map_err(|_| error("cannot stage upgraded monitor executable"))?;
    if sha256_digest(&bytes) != config.executable_digest {
        return Err(error("monitor executable changed during update"));
    }
    let relative = owned_binary_relative(root, &config.executable)?;
    let before = root.snapshot_bounded(&relative, 128 * 1024 * 1024)?;
    root.publish_runtime(&relative, &before, &bytes)?;
    Ok(())
}

fn owned_binary_relative(
    root: &PinnedTarget,
    executable: &Path,
) -> Result<std::path::PathBuf, AdapterError> {
    let normal = monitor_root_path(root.requested_path())?;
    for spelling in [normal.as_path(), root.requested_path()] {
        if let Ok(name) = executable.strip_prefix(spelling.join(BASE).join("bin")) {
            let mut components = name.components();
            if matches!(components.next(), Some(std::path::Component::Normal(_)))
                && components.next().is_none()
            {
                return Ok(Path::new(BASE).join("bin").join(name));
            }
        }
    }
    Err(error("monitor executable escaped its owned bin directory"))
}

fn upgrade_with(
    root: &PinnedTarget,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    ready: impl Fn(&PinnedTarget, &Configuration) -> Result<Value, AdapterError>,
) -> Result<Value, AdapterError> {
    let mut next = configuration(root.requested_path())?;
    let old: Configuration =
        load(root, CONFIG)?.ok_or_else(|| error("monitor configuration missing"))?;
    next.enabled = old.enabled;
    authenticate_previous(root, &old, &next)?;
    let manifest: Value = load(root, ".hive/install/codex.json")?
        .ok_or_else(|| error("matching Codex installation required for monitor update"))?;
    if manifest["product_version"] != env!("CARGO_PKG_VERSION") {
        return Err(error(
            "Codex installation and upgraded monitor versions differ",
        ));
    }
    let mut journal = json!({"schema_version":1,"from":old,"to":next,
        "phase":"preparing","started_at":now(),"error":null,"completed_at":null});
    let operation: Result<Value, AdapterError> = (|| {
        {
            let _control = lock(root, "control.lock")?;
            write(root, JOURNAL, &journal)?;
            stage(root, &next)?;
            if old.enabled {
                require_desktop_version(&call(
                    &json!({"operation":"inspect"}),
                    Duration::from_secs(5),
                )?)?;
            }
            let previous_lease = pause_previous(root, &old, Duration::from_secs(15))?;
            if old.enabled {
                // Exact stored action/description must match; never stop by process name.
                if let Err(failure) =
                    call(&task_request(&old, "unregister"), Duration::from_secs(15))
                {
                    write(root, CONFIG, &old)?;
                    return Err(failure);
                }
                journal["phase"] = json!("old-task-removed");
            }
            if let Err(failure) = write(root, CONFIG, &next) {
                write(root, CONFIG, &old)?;
                return Err(failure);
            }
            if old.enabled {
                with_state(root, |state| {
                    // Preserve baseline, incidents, deduplication and manual-resume receipts.
                    state.observation_count = 0;
                    state.controller_pid = None;
                    state.controller_identity = None;
                    state.control_connected = false;
                    state.heartbeat = 0;
                    state.heartbeat_uptime_ms = None;
                    Ok(())
                })?;
                drop(previous_lease);
                call(&task_request(&next, "register"), Duration::from_secs(15))?;
                call(&task_request(&next, "start"), Duration::from_secs(10))?;
            }
            journal["phase"] = json!(if old.enabled {
                "checking"
            } else {
                "disabled-preserved"
            });
            write(root, JOURNAL, &journal)?;
        } // Release control lock so the user can still disable while health is checked.
        let health = if old.enabled {
            ready(root, &next)?
        } else {
            json!({"enabled":false})
        };
        journal["phase"] = json!("complete");
        journal["completed_at"] = json!(now());
        write(root, JOURNAL, &journal)?;
        Ok(
            json!({"enabled":next.enabled,"version":next.version,"executable":next.executable,
            "task_replaced":old.enabled,"history_preserved":true,"health":health,
            "changed_paths":[CONFIG,JOURNAL,next.executable]}),
        )
    })();
    if let Err(failure) = &operation {
        journal["phase"] = json!("incomplete");
        journal["error"] = json!(failure.message());
        let _ = write(root, JOURNAL, &journal);
        // Never enable a disabled choice, discard evidence, or restart an older unsafe binary.
    }
    operation
}

fn authenticate_previous(
    root: &PinnedTarget,
    previous: &Configuration,
    next: &Configuration,
) -> Result<(), AdapterError> {
    let root_text = previous
        .arguments
        .strip_prefix("usage monitor run --user-root \"")
        .and_then(|argument| argument.strip_suffix("\" --output json"))
        .filter(|argument| !argument.contains(['\"', '\r', '\n']))
        .ok_or_else(|| {
            error("existing monitor arguments are not authenticated; no task changed")
        })?;
    let prior_path = Path::new(root_text);
    if !prior_path.is_absolute()
        || monitor_root_path(prior_path)? != monitor_root_path(root.requested_path())?
    {
        return Err(error("existing monitor root differs; no task changed"));
    }
    let prior = PinnedTarget::open_usage(prior_path)?;
    #[cfg(windows)]
    {
        use cap_fs_ext::MetadataExt;
        let before = root
            .target_dir()
            .dir_metadata()
            .map_err(|_| error("monitor root identity unavailable"))?;
        let after = prior
            .target_dir()
            .dir_metadata()
            .map_err(|_| error("prior monitor root identity unavailable"))?;
        if (before.dev(), before.ino()) != (after.dev(), after.ino()) {
            return Err(error(
                "existing monitor root identity differs; no task changed",
            ));
        }
    }
    if previous.schema_version != 1
        || (previous.task_name != next.task_name
            && previous.task_name != monitor_task_name(prior_path))
        || owned_binary_relative(&prior, &previous.executable).is_err()
    {
        return Err(error(
            "existing monitor ownership is not authenticated; no task changed",
        ));
    }
    Ok(())
}

fn pause_previous(
    root: &PinnedTarget,
    previous: &Configuration,
    timeout: Duration,
) -> Result<std::fs::File, AdapterError> {
    let mut paused = previous.clone();
    paused.enabled = false;
    write(root, CONFIG, &paused)?;
    let began = Instant::now();
    loop {
        match lock(root, "controller.lock") {
            Ok(lease) => return Ok(lease),
            Err(AdapterError::Conflict(_)) if began.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(failure) => {
                // No state migration has occurred; keep the saved opt-in and old reader format.
                write(root, CONFIG, previous)?;
                return Err(failure);
            }
        }
    }
}

fn await_ready(root: &PinnedTarget, expected: &Configuration) -> Result<Value, AdapterError> {
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(70) {
        let config: Configuration =
            load(root, CONFIG)?.ok_or_else(|| error("monitor update configuration missing"))?;
        if !config.enabled || config.executable_digest != expected.executable_digest {
            return Err(error(
                "monitor choice changed during update; no automatic reactivation",
            ));
        }
        let status = monitor_status(root)?;
        if status["controller_alive"] == true
            && status["controller_heartbeat_fresh"] == true
            && status["running_monitor_version"] == env!("HIVE_PACKAGE_VERSION")
        {
            if status["sensor_health"] == "idle" {
                return Ok(
                    json!({"controller_verified":true,"state":"idle-awaiting-Codex","fresh_observations":0}),
                );
            }
            if status["monitoring_verified"] == true
                && status["state"]["observation_count"].as_u64().unwrap_or(0) >= 4
            {
                return Ok(
                    json!({"controller_verified":true,"state":"healthy","fresh_observations":status["state"]["observation_count"],"query_source":status["last_query_source"]}),
                );
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    Err(error("monitor update incomplete: new controller and four fresh observations were not verified; choice and history preserved"))
}

#[cfg(test)]
mod tests {
    use super::super::{MonitorState, DESKTOP_VERSION, STATE};
    use super::*;
    use std::cell::RefCell;

    fn fixture(enabled: bool) -> (tempfile::TempDir, PinnedTarget) {
        let directory = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(directory.path()).unwrap();
        let mut old = configuration(root.requested_path()).unwrap();
        old.enabled = enabled;
        old.version = "0.13.0".into();
        old.executable = root.requested_path().join(BASE).join("bin/hive-old.exe");
        old.arguments = format!(
            "usage monitor run --user-root \"{}\" --output json",
            root.requested_path().display()
        );
        if let Some(launcher) = &mut old.launcher {
            launcher.arguments = "historical-owned-action".into();
        }
        write(&root, CONFIG, &old).unwrap();
        let manifest = directory.path().join(".hive/install/codex.json");
        std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        std::fs::write(
            manifest,
            serde_json::to_vec(&json!({"product_version":env!("CARGO_PKG_VERSION")})).unwrap(),
        )
        .unwrap();
        with_state(&root, |state| {
            state.halt("usage-reset", 100);
            state.incident.as_mut().unwrap().stop_dispatched_at = Some(100);
            Ok(())
        })
        .unwrap();
        (directory, root)
    }

    #[test]
    fn disabled_choice_is_updated_without_registering_or_starting() {
        let (_directory, root) = fixture(false);
        let before: MonitorState = load(&root, STATE).unwrap().unwrap();
        let result = upgrade_with(
            &root,
            |_, _| panic!("disabled monitor cannot call scheduler"),
            |_, _| panic!("no live check for disabled monitor"),
        )
        .unwrap();
        assert_eq!(result["enabled"], false);
        assert_eq!(result["task_replaced"], false);
        assert_eq!(
            load::<MonitorState>(&root, STATE)
                .unwrap()
                .unwrap()
                .incident
                .unwrap()
                .digest,
            before.incident.unwrap().digest
        );
        assert!(
            !load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn previous_reader_releases_lease_before_state_format_changes() {
        let (_directory, root) = fixture(true);
        let old: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let old_state = json!({"schema_version":2,"legacy_only_field":"keep exactly"});
        write(&root, STATE, &old_state).unwrap();
        let lease = lock(&root, "controller.lock").unwrap();
        let path = root.requested_path().to_path_buf();
        let reader = thread::spawn(move || {
            let pinned = PinnedTarget::open_usage(&path).unwrap();
            let began = Instant::now();
            while load::<Configuration>(&pinned, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
            {
                assert!(began.elapsed() < Duration::from_secs(3));
                thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(load::<Value>(&pinned, STATE).unwrap().unwrap(), old_state);
            drop(lease);
        });
        let acquired = pause_previous(&root, &old, Duration::from_secs(4)).unwrap();
        reader.join().unwrap();
        assert!(lock(&root, "controller.lock").is_err());
        drop(acquired);
        assert!(lock(&root, "controller.lock").is_ok());
    }

    #[test]
    fn stuck_previous_reader_preserves_choice_and_original_state() {
        let (_directory, root) = fixture(true);
        let old: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let original: Value = load(&root, STATE).unwrap().unwrap();
        let _lease = lock(&root, "controller.lock").unwrap();
        assert!(pause_previous(&root, &old, Duration::ZERO).is_err());
        assert!(
            load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(load::<Value>(&root, STATE).unwrap().unwrap(), original);
    }

    #[test]
    fn scheduler_refusal_restores_old_choice_without_migrating_state() {
        let (_directory, root) = fixture(true);
        let original: Value = load(&root, STATE).unwrap().unwrap();
        assert!(upgrade_with(
            &root,
            |request, _| {
                if request["operation"] == "unregister" {
                    assert!(!load::<Configuration>(&root, CONFIG)?.unwrap().enabled);
                    assert!(lock(&root, "controller.lock").is_err());
                    return Err(error("injected scheduler refusal"));
                }
                Ok(json!({"desktop_package_version":DESKTOP_VERSION}))
            },
            |_, _| panic!("failed replacement cannot start health checks"),
        )
        .is_err());
        assert!(
            load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(load::<Value>(&root, STATE).unwrap().unwrap(), original);
    }

    #[cfg(windows)]
    #[test]
    fn public_plain_root_and_verbatim_update_share_the_same_owned_task() {
        let (_directory, root) = fixture(false);
        let normal = monitor_root_path(root.requested_path()).unwrap();
        let extended = std::fs::canonicalize(&normal).unwrap();
        assert_eq!(
            configuration(&normal).unwrap().task_name,
            configuration(&extended).unwrap().task_name
        );
        let mut old: Configuration = load(&root, CONFIG).unwrap().unwrap();
        old.executable = normal.join(BASE).join("bin/hive-old.exe");
        let task = old.task_name.clone();
        write(&root, CONFIG, &old).unwrap();
        upgrade_with(
            &root,
            |_, _| panic!("disabled choice"),
            |_, _| panic!("disabled choice"),
        )
        .unwrap();
        let current: Configuration = load(&root, CONFIG).unwrap().unwrap();
        assert_eq!(current.task_name, task);
        assert!(current.executable.starts_with(&normal));
        assert!(!current.executable.to_string_lossy().contains('/'));
    }

    #[cfg(windows)]
    #[test]
    fn prior_extended_task_is_removed_by_its_exact_name_before_normal_registration() {
        let (_directory, root) = fixture(true);
        let mut old: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let extended = std::fs::canonicalize(root.requested_path()).unwrap();
        old.task_name = monitor_task_name(&extended);
        old.arguments = format!(
            "usage monitor run --user-root \"{}\" --output json",
            extended.display()
        );
        old.executable = extended.join(BASE).join("bin/hive-old.exe");
        let previous = old.task_name.clone();
        write(&root, CONFIG, &old).unwrap();
        let calls = RefCell::new(Vec::new());
        upgrade_with(
            &root,
            |request, _| {
                calls.borrow_mut().push(request.clone());
                Ok(json!({"desktop_package_version":DESKTOP_VERSION}))
            },
            |_, _| Ok(json!({"fresh_observations":4})),
        )
        .unwrap();
        assert_eq!(calls.borrow()[1]["task_name"], previous);
        assert_eq!(
            calls.borrow()[2]["task_name"],
            configuration(root.requested_path()).unwrap().task_name
        );
        assert_ne!(
            calls.borrow()[1]["task_name"],
            calls.borrow()[2]["task_name"]
        );
    }

    #[test]
    fn foreign_names_and_escaping_binaries_never_reach_the_scheduler() {
        let (_directory, root) = fixture(false);
        let original: Configuration = load(&root, CONFIG).unwrap().unwrap();
        for (name, executable) in [
            ("unrelated-task".to_owned(), original.executable.clone()),
            (
                original.task_name.clone(),
                root.requested_path().join(BASE).join("bin/../foreign.exe"),
            ),
            (
                original.task_name.clone(),
                root.requested_path().join("other/hive.exe"),
            ),
        ] {
            let mut changed = original.clone();
            changed.task_name = name;
            changed.executable = executable;
            write(&root, CONFIG, &changed).unwrap();
            assert!(upgrade_with(
                &root,
                |_, _| panic!("foreign task"),
                |_, _| panic!("foreign task")
            )
            .is_err());
        }
    }

    #[test]
    fn previous_arguments_must_name_the_same_root_with_the_known_command() {
        let (_directory, root) = fixture(false);
        let foreign = tempfile::tempdir().unwrap();
        let original: Configuration = load(&root, CONFIG).unwrap().unwrap();
        for arguments in [
            "unrelated-command".into(),
            format!(
                "usage monitor run --user-root \"{}\" --output json",
                foreign.path().display()
            ),
        ] {
            let mut previous = original.clone();
            previous.arguments = arguments;
            write(&root, CONFIG, &previous).unwrap();
            assert!(upgrade_with(
                &root,
                |_, _| panic!("untrusted arguments"),
                |_, _| panic!("untrusted arguments")
            )
            .is_err());
        }
    }

    #[test]
    fn enabled_update_uses_exact_old_and_new_owned_actions_and_preserves_history() {
        let (_directory, root) = fixture(true);
        let calls = RefCell::new(Vec::new());
        let result = upgrade_with(
            &root,
            |request, _| {
                calls.borrow_mut().push(request.clone());
                Ok(json!({"desktop_package_version":DESKTOP_VERSION}))
            },
            |root, next| {
                let state: MonitorState = load(root, STATE)?.unwrap();
                assert!(state.incident.unwrap().stop_dispatched_at.is_some());
                assert!(next.enabled);
                Ok(json!({"fresh_observations":4}))
            },
        )
        .unwrap();
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|value| value["operation"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["inspect", "unregister", "register", "start"]
        );
        assert_ne!(
            (
                calls.borrow()[1]["executable"].clone(),
                calls.borrow()[1]["arguments"].clone()
            ),
            (
                calls.borrow()[2]["executable"].clone(),
                calls.borrow()[2]["arguments"].clone()
            )
        );
        assert_eq!(result["version"], env!("HIVE_PACKAGE_VERSION"));
        assert_eq!(
            load::<Value>(&root, JOURNAL).unwrap().unwrap()["phase"],
            "complete"
        );
    }

    #[test]
    fn failed_transition_is_incomplete_and_does_not_change_opt_in_or_replay_old_incident() {
        let (_directory, root) = fixture(true);
        assert!(upgrade_with(
            &root,
            |_, _| Ok(json!({"desktop_package_version":DESKTOP_VERSION})),
            |_, _| Err(error("injected fresh observation failure"))
        )
        .is_err());
        assert!(
            load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(
            load::<Value>(&root, JOURNAL).unwrap().unwrap()["phase"],
            "incomplete"
        );
        assert!(load::<MonitorState>(&root, STATE)
            .unwrap()
            .unwrap()
            .incident
            .unwrap()
            .stop_dispatched_at
            .is_some());
    }
}
