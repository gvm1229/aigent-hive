//! Preserve the monitor choice and history while replacing its version-bound executable/task.
use super::*;

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
    let relative = config
        .executable
        .strip_prefix(root.requested_path())
        .map_err(|_| error("upgraded monitor executable escaped user root"))?;
    let before = root.snapshot_bounded(relative, 128 * 1024 * 1024)?;
    root.publish_runtime(relative, &before, &bytes)?;
    Ok(())
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
    if old.schema_version != 1
        || old.task_name != next.task_name
        || !old
            .executable
            .starts_with(root.requested_path().join(BASE).join("bin"))
    {
        return Err(error(
            "existing monitor ownership is not authenticated; no task changed",
        ));
    }
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
                // Exact stored action/description must match; never stop by process name.
                call(&task_request(&old, "unregister"), Duration::from_secs(15))?;
                journal["phase"] = json!("old-task-removed");
                write(root, JOURNAL, &journal)?;
            }
            write(root, CONFIG, &next)?;
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
            && status["running_monitor_version"] == env!("CARGO_PKG_VERSION")
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
    use super::*;
    use std::cell::RefCell;

    fn fixture(enabled: bool) -> (tempfile::TempDir, PinnedTarget) {
        let directory = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(directory.path()).unwrap();
        let mut old = configuration(directory.path()).unwrap();
        old.enabled = enabled;
        old.version = "0.13.0".into();
        old.executable = directory.path().join(BASE).join("bin/hive-old.exe");
        old.arguments = "historical-owned-action".into();
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
            calls.borrow()[1]["arguments"],
            calls.borrow()[2]["arguments"]
        );
        assert_eq!(result["version"], env!("CARGO_PKG_VERSION"));
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
