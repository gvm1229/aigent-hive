//! Preserve the monitor choice and history while replacing its version-bound executable/task.
use super::{
    configuration, error, load, lock, monitor_root_path, monitor_status, monitor_task_name, now,
    platform, require_app_safe_control, require_desktop_inspection, task_request, with_state,
    write, Configuration, BASE, CONFIG,
};
use crate::run::{AdapterError, PinnedTarget};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

pub(super) const JOURNAL: &str = ".hive/runtime/usage-monitor/upgrade.json";

pub(crate) fn upgrade_existing(user_root: &Path) -> Result<Option<Value>, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    match load::<Configuration>(&root, CONFIG)? {
        None => return Ok(None),
        Some(_) if policy_disables_monitor(&root)? => {
            return super::transition::disable(&root, platform::call, Duration::from_secs(15))
                .map(Some);
        }
        Some(config) if config.enabled => require_app_safe_control()?,
        Some(_) => {}
    }
    upgrade_with(&root, platform::call, await_ready).map(Some)
}

fn policy_disables_monitor(root: &PinnedTarget) -> Result<bool, AdapterError> {
    let Some(bytes) = root.read_optional(Path::new(".hive/config/user-setup.yml"), 1024 * 1024)?
    else {
        return Ok(false);
    };
    let settings = crate::user_setup::parse_and_validate_installed_config(&bytes)
        .map_err(|failure| error(failure.message()))?;
    Ok(settings
        .usage_guard
        .policy
        .is_some_and(|policy| !policy.enforcement_enabled || !policy.midway.enabled))
}

pub(super) fn stage(root: &PinnedTarget, config: &Configuration) -> Result<(), AdapterError> {
    let bytes = std::fs::read(std::env::current_exe().map_err(|_| error("unknown executable"))?)
        .map_err(|_| error("cannot stage upgraded monitor executable"))?;
    stage_bytes(root, config, &bytes)
}

fn stage_bytes(
    root: &PinnedTarget,
    config: &Configuration,
    bytes: &[u8],
) -> Result<(), AdapterError> {
    if sha256_digest(bytes) != config.executable_digest {
        return Err(error("monitor executable changed during update"));
    }
    let relative = owned_binary_relative(root, &config.executable)?;
    let before = root.snapshot_bounded(&relative, 128 * 1024 * 1024)?;
    if let Some(installed) = before.bytes() {
        if installed == bytes {
            return Ok(());
        }
        return Err(error(
            "existing version-bound monitor binary differs; no overwrite",
        ));
    }
    root.publish_runtime(&relative, &before, bytes)?;
    Ok(())
}

pub(super) fn owned_binary_relative(
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

/// Keep only the current and the directly previous version-bound monitor executables.
///
/// Cleanup never fails an update. A file that cannot be read or removed exactly (for example a
/// running or locked executable on Windows) stays in place and is reported as skipped. When the
/// update reused the same executable, the previous record's source executable stays too.
pub(super) fn prune_binaries(
    root: &PinnedTarget,
    current: &Configuration,
    previous: &Configuration,
    prior_source: Option<&Path>,
) -> Value {
    let mut executables = vec![current.executable.as_path(), previous.executable.as_path()];
    if current.executable == previous.executable {
        executables.extend(prior_source);
    }
    match retained_binaries(root, &executables) {
        Ok(keep) => prune_binaries_with(root, &keep, false, PinnedTarget::remove_runtime),
        Err(failure) => json!({"kept":[],"removed":[],"skipped":[],
            "error":format!("cleanup skipped: {}", failure.message())}),
    }
}

/// Resolve every executable that must stay; an unresolvable one stops the cleanup.
fn retained_binaries(
    root: &PinnedTarget,
    executables: &[&Path],
) -> Result<Vec<PathBuf>, AdapterError> {
    executables
        .iter()
        .map(|executable| owned_binary_relative(root, executable))
        .collect()
}

/// Preview or remove monitor executables that neither the saved configuration nor the last
/// update record uses. Missing or unreadable records refuse cleanup instead of guessing.
pub(crate) fn prune_unused(user_root: &Path, confirm: bool) -> Result<Value, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    let bin = Path::new(BASE).join("bin");
    let present = match root.target_dir().read_dir(&bin) {
        Ok(mut entries) => entries.next().is_some(),
        Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => false,
        Err(failure) => {
            return Err(error(&format!(
                "cannot list monitor executables: {failure}"
            )));
        }
    };
    if !present {
        return Ok(json!({"kept":[],"removed":[],"skipped":[],"candidates":[],"error":null}));
    }
    let _control = lock(&root, "control.lock")?;
    let config: Configuration = load(&root, CONFIG)?.ok_or_else(|| {
        error("monitor configuration is missing; no executable cleanup without a current record")
    })?;
    let journal: Option<Value> = load(&root, JOURNAL)?;
    let mut executables = vec![config.executable.clone()];
    if let Some(journal) = &journal {
        for side in ["from", "to"] {
            if let Some(path) = journal[side]["executable"].as_str() {
                executables.push(PathBuf::from(path));
            }
        }
    }
    let borrowed: Vec<&Path> = executables.iter().map(PathBuf::as_path).collect();
    let keep = retained_binaries(&root, &borrowed)?;
    Ok(prune_binaries_with(
        &root,
        &keep,
        !confirm,
        PinnedTarget::remove_runtime,
    ))
}

fn prune_binaries_with(
    root: &PinnedTarget,
    keep: &[PathBuf],
    dry_run: bool,
    remove: impl Fn(&PinnedTarget, &Path, &crate::run::FileSnapshot) -> Result<bool, AdapterError>,
) -> Value {
    let bin = Path::new(BASE).join("bin");
    let entries = match root.target_dir().read_dir(&bin) {
        Ok(entries) => entries,
        Err(failure) => {
            return json!({"kept":keep,"removed":[],"skipped":[],"candidates":[],
                "error":format!("cannot list monitor executables: {failure}")});
        }
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_file = entry.file_type().is_ok_and(|kind| kind.is_file());
        let is_exe = Path::new(&name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));
        if is_file && is_exe && name.starts_with("hive-") {
            names.push(name);
        }
    }
    names.sort();
    let (mut removed, mut skipped, mut candidates) = (Vec::new(), Vec::new(), Vec::new());
    for name in names {
        let relative = bin.join(&name);
        if keep.contains(&relative) {
            continue;
        }
        let snapshot = root.snapshot_bounded(&relative, 128 * 1024 * 1024);
        if dry_run {
            match snapshot {
                Ok(snapshot) => candidates.push(json!({"path":relative,
                    "bytes":snapshot.bytes().map_or(0, <[u8]>::len)})),
                Err(failure) => skipped.push(json!({"path":relative,"reason":failure.message()})),
            }
            continue;
        }
        match snapshot.and_then(|snapshot| remove(root, &relative, &snapshot)) {
            Ok(true) => removed.push(relative),
            Ok(false) => skipped.push(json!({"path":relative,"reason":"changed or missing"})),
            Err(failure) => skipped.push(json!({"path":relative,"reason":failure.message()})),
        }
    }
    json!({"kept":keep,"removed":removed,"skipped":skipped,"candidates":candidates,"error":null})
}

fn upgrade_with(
    root: &PinnedTarget,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    ready: impl Fn(&PinnedTarget, &Configuration) -> Result<Value, AdapterError>,
) -> Result<Value, AdapterError> {
    upgrade_with_choice(root, call, ready, None)
}

pub(crate) fn resume_after_setup(
    user_root: &Path,
    operation_id: &str,
) -> Result<Value, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    if policy_disables_monitor(&root)? {
        return super::transition::disable(&root, platform::call, Duration::from_secs(15));
    }
    let stopped: Value = load(&root, super::transition::JOURNAL)?
        .ok_or_else(|| error("setup monitor stop record is missing"))?;
    let bytes = root.read_required(Path::new(CONFIG), 65536)?;
    if stopped["operation_id"] != operation_id || stopped["phase"] != "complete" {
        return Err(error("monitor choice changed after setup paused it"));
    }
    if stopped["after_digest"] != sha256_digest(&bytes) {
        let prior: Value =
            load(&root, JOURNAL)?.ok_or_else(|| error("monitor changed after setup"))?;
        let current: Configuration =
            serde_json::from_slice(&bytes).map_err(|_| error("invalid monitor configuration"))?;
        if prior["setup_operation"] == operation_id
            && matches!(
                prior["phase"].as_str(),
                Some("complete" | "checking" | "incomplete")
            )
            && prior["to"]
                == serde_json::to_value(&current)
                    .map_err(|_| error("invalid monitor configuration"))?
        {
            return finish_saved_setup_upgrade(
                &root,
                operation_id,
                &stopped,
                &current,
                &prior,
                await_ready,
            );
        }
        return Err(error(
            "monitor changed after setup; no automatic reactivation",
        ));
    }
    require_app_safe_control()?;
    upgrade_with_choice(&root, platform::call, await_ready, Some(operation_id))
}

fn finish_saved_setup_upgrade(
    root: &PinnedTarget,
    operation_id: &str,
    stopped: &Value,
    current: &Configuration,
    prior: &Value,
    ready: impl FnOnce(&PinnedTarget, &Configuration) -> Result<Value, AdapterError>,
) -> Result<Value, AdapterError> {
    if prior["setup_operation"] != operation_id
        || stopped["operation_id"] != operation_id
        || stopped["phase"] != "complete"
        || !matches!(
            prior["phase"].as_str(),
            Some("complete" | "checking" | "incomplete")
        )
        || prior["to"]
            != serde_json::to_value(current)
                .map_err(|_| error("invalid saved monitor configuration"))?
    {
        return Err(error(
            "saved monitor readiness does not match the setup operation",
        ));
    }
    authenticate_ownership(
        root,
        current,
        &monitor_task_name(&monitor_root_path(root.requested_path())?),
    )?;
    if !current.enabled || current.version != env!("HIVE_PACKAGE_VERSION") {
        return Err(error("saved monitor does not match this update"));
    }
    let health = ready(root, current)?;
    let _control = lock(root, "control.lock")?;
    let latest: Configuration = load(root, CONFIG)?
        .ok_or_else(|| error("monitor configuration missing after readiness check"))?;
    let stop: Value = load(root, super::transition::JOURNAL)?
        .ok_or_else(|| error("monitor stop record missing after readiness check"))?;
    let upgrade_snapshot = root.snapshot(Path::new(JOURNAL))?;
    let upgrade: Value = serde_json::from_slice(
        upgrade_snapshot
            .bytes()
            .ok_or_else(|| error("monitor update record missing after readiness check"))?,
    )
    .map_err(|_| error("invalid monitor update record after readiness check"))?;
    if policy_disables_monitor(root)?
        || super::serialize(&latest)? != super::serialize(current)?
        || stop != *stopped
        || upgrade != *prior
        || prior["setup_operation"] != operation_id
    {
        return Err(error(
            "monitor selection changed during readiness check; changes preserved",
        ));
    }
    if prior["phase"] != "complete" {
        let mut complete = prior.clone();
        complete["phase"] = json!("complete");
        complete["completed_at"] = json!(now());
        complete["error"] = Value::Null;
        root.publish_runtime(
            Path::new(JOURNAL),
            &upgrade_snapshot,
            &super::serialize(&complete)?,
        )?;
    }
    Ok(health)
}

fn upgrade_with_choice(
    root: &PinnedTarget,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    ready: impl Fn(&PinnedTarget, &Configuration) -> Result<Value, AdapterError>,
    setup_operation: Option<&str>,
) -> Result<Value, AdapterError> {
    if policy_disables_monitor(root)? {
        return super::transition::disable(root, call, Duration::from_secs(15));
    }
    let mut next = configuration(root.requested_path())?;
    let old: Configuration =
        load(root, CONFIG)?.ok_or_else(|| error("monitor configuration missing"))?;
    // The previous update record names the executable before a same-version re-run.
    let prior_source = load::<Value>(root, JOURNAL)?
        .and_then(|journal| journal["from"]["executable"].as_str().map(PathBuf::from));
    next.enabled = setup_operation.is_some() || old.enabled;
    authenticate_previous(root, &old, &next)?;
    let manifest: Value = load(root, ".hive/install/codex.json")?
        .ok_or_else(|| error("matching Codex installation required for monitor update"))?;
    if manifest["product_version"] != env!("CARGO_PKG_VERSION") {
        return Err(error(
            "Codex installation and upgraded monitor versions differ",
        ));
    }
    let mut journal = json!({"schema_version":1,"from":old,"to":next,"setup_operation":setup_operation,
        "phase":"preparing","started_at":now(),"error":null,"completed_at":null});
    let mut journal_started = false;
    let operation: Result<Value, AdapterError> = (|| {
        {
            let _control = lock(root, "control.lock")?;
            verify_upgrade_inputs(root, &old, setup_operation)?;
            write(root, JOURNAL, &journal)?;
            journal_started = true;
            stage(root, &next)?;
            if next.enabled {
                require_desktop_inspection(&call(
                    &json!({"operation":"inspect"}),
                    Duration::from_secs(5),
                )?)?;
            }
            let previous_lease = pause_previous(root, &old, Duration::from_secs(15))?;
            if old.enabled || next.enabled {
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
            if next.enabled {
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
            journal["phase"] = json!(if next.enabled {
                "checking"
            } else {
                "disabled-preserved"
            });
            write(root, JOURNAL, &journal)?;
        } // Release control lock so the user can still disable while health is checked.
        let health = if next.enabled {
            ready(root, &next)?
        } else {
            json!({"enabled":false})
        };
        // Keep the current and directly previous executables; skip running or locked files.
        let cleanup = prune_binaries(root, &next, &old, prior_source.as_deref());
        journal["phase"] = json!("complete");
        journal["completed_at"] = json!(now());
        journal["binary_cleanup"] = cleanup.clone();
        write(root, JOURNAL, &journal)?;
        Ok(
            json!({"enabled":next.enabled,"version":next.version,"executable":next.executable,
            "task_replaced":old.enabled || next.enabled,"history_preserved":true,"health":health,
            "binary_cleanup":cleanup,"changed_paths":[CONFIG,JOURNAL,next.executable]}),
        )
    })();
    if let Err(failure) = &operation {
        journal["phase"] = json!("incomplete");
        journal["error"] = json!(failure.message());
        if journal_started {
            let _ = write(root, JOURNAL, &journal);
        }
        // Never enable a disabled choice, discard evidence, or restart an older unsafe binary.
    }
    operation
}

fn verify_upgrade_inputs(
    root: &PinnedTarget,
    old: &Configuration,
    setup_operation: Option<&str>,
) -> Result<(), AdapterError> {
    let current: Configuration =
        load(root, CONFIG)?.ok_or_else(|| error("monitor changed before update"))?;
    if super::serialize(&current)? != super::serialize(old)? || policy_disables_monitor(root)? {
        return Err(error("monitor or policy changed before update"));
    }
    if let Some(id) = setup_operation {
        let stopped: Value = load(root, super::transition::JOURNAL)?
            .ok_or_else(|| error("setup stop record missing"))?;
        if stopped["operation_id"] != id
            || stopped["phase"] != "complete"
            || stopped["after_digest"] != sha256_digest(&super::serialize(old)?)
        {
            return Err(error("monitor choice changed before setup resume"));
        }
    }
    Ok(())
}

fn authenticate_previous(
    root: &PinnedTarget,
    previous: &Configuration,
    next: &Configuration,
) -> Result<(), AdapterError> {
    authenticate_ownership(root, previous, &next.task_name)
}

pub(super) fn authenticate_ownership(
    root: &PinnedTarget,
    previous: &Configuration,
    task_name: &str,
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
        || (previous.task_name != task_name && previous.task_name != monitor_task_name(prior_path))
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

pub(super) fn await_ready(
    root: &PinnedTarget,
    expected: &Configuration,
) -> Result<Value, AdapterError> {
    let began = Instant::now();
    let budget = crate::user_setup::command_timeout(Duration::from_secs(70));
    while began.elapsed() < budget {
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
        thread::sleep(Duration::from_millis(500).min(budget.saturating_sub(began.elapsed())));
    }
    Err(error("monitor update incomplete: new controller and four fresh observations were not verified; choice and history preserved"))
}

#[cfg(test)]
mod tests {
    use super::super::{MonitorState, STATE};
    use super::*;
    use std::cell::RefCell;

    fn restore_owned_launcher(config: &mut Configuration) {
        // Keep the saved argument spelling, including legitimate Windows short paths.
        let root = config
            .arguments
            .strip_prefix("usage monitor run --user-root \"")
            .and_then(|value| value.strip_suffix("\" --output json"))
            .unwrap();
        config.launcher = super::super::task_launcher(&config.executable, root);
    }

    fn fixture(enabled: bool) -> (tempfile::TempDir, PinnedTarget) {
        let directory = crate::test_tempdir();
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

    fn seed_binaries(root: &PinnedTarget, names: &[&str]) {
        let bin = root.requested_path().join(BASE).join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        for name in names {
            std::fs::write(bin.join(name), name.as_bytes()).unwrap();
        }
    }

    fn binary_names(root: &PinnedTarget) -> Vec<String> {
        let mut names: Vec<String> =
            std::fs::read_dir(root.requested_path().join(BASE).join("bin"))
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
        names.sort();
        names
    }

    #[test]
    fn large_binary_cleanup_after_upgrade_keeps_current_and_previous() {
        let (_directory, root) = fixture(false);
        let bin = root.requested_path().join(BASE).join("bin");
        let bytes = vec![0x5a; 2 * 1024 * 1024 + 17];
        std::fs::create_dir_all(&bin).unwrap();
        for name in ["hive-a.exe", "hive-b.exe", "hive-old.exe"] {
            std::fs::write(bin.join(name), &bytes).unwrap();
        }
        let result = upgrade_with(
            &root,
            |_, _| panic!("disabled monitor cannot change scheduler"),
            |_, _| panic!("disabled monitor cannot inspect live health"),
        )
        .unwrap();
        let cleanup = &result["binary_cleanup"];
        assert_eq!(cleanup["removed"].as_array().unwrap().len(), 2);
        assert!(cleanup["skipped"].as_array().unwrap().is_empty());
        assert_eq!(binary_names(&root).len(), 2);
        assert_eq!(std::fs::read(bin.join("hive-old.exe")).unwrap(), bytes);
        let current: Configuration = load(&root, CONFIG).unwrap().unwrap();
        assert!(current.executable.is_file());
        assert!(binary_names(&root)
            .iter()
            .any(|name| Some(std::ffi::OsStr::new(name)) == current.executable.file_name()));
        let journal: Value = load(&root, JOURNAL).unwrap().unwrap();
        assert_eq!(journal["phase"], "complete");
        assert_eq!(&journal["binary_cleanup"], cleanup);
        assert!(
            prune_unused(root.requested_path(), false).unwrap()["candidates"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[cfg(windows)]
    #[test]
    fn running_windows_binary_is_restored_and_reported_skipped_after_delete_failure() {
        use std::os::windows::process::CommandExt;
        use std::process::{Child, Command, Stdio};

        struct Running(Child);
        impl Drop for Running {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }

        let (_directory, root) = fixture(false);
        let bin = root.requested_path().join(BASE).join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let name = std::ffi::OsStr::new("hive-running.exe");
        let path = bin.join(name);
        let source =
            PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/ping.exe");
        std::fs::copy(source, &path).unwrap();
        let expected = std::fs::read(&path).unwrap();
        let mut running = Running(
            Command::new(&path)
                .args(["-t", "127.0.0.1"])
                .creation_flags(0x0800_0000)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        assert!(running.0.try_wait().unwrap().is_none());
        let parent =
            cap_std::fs::Dir::open_ambient_dir(&bin, cap_std::ambient_authority()).unwrap();
        let failure = crate::run::remove_parent_file(&parent, name, &expected).unwrap_err();
        let message = failure.to_string();
        assert!(
            message.contains("cannot remove claimed destination"),
            "{message}"
        );
        assert!(message.contains("os error 5"), "{message}");
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        let result = prune_unused(root.requested_path(), true).unwrap();
        assert!(result["removed"].as_array().unwrap().is_empty());
        let skipped = result["skipped"].as_array().unwrap();
        assert_eq!(skipped.len(), 1);
        assert_eq!(
            skipped[0]["path"],
            json!(Path::new(BASE).join("bin").join(name))
        );
        assert!(skipped[0]["reason"]
            .as_str()
            .unwrap()
            .contains("os error 5"));
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        assert_eq!(binary_names(&root), ["hive-running.exe"]);
        assert!(running.0.try_wait().unwrap().is_none());
        drop(running);
        assert_eq!(
            prune_unused(root.requested_path(), true).unwrap()["removed"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(binary_names(&root).is_empty());
    }

    #[test]
    fn manual_prune_refuses_without_a_readable_current_record() {
        let (directory, root) = fixture(false);
        seed_binaries(&root, &["hive-a.exe", "hive-old.exe"]);
        let config = directory.path().join(CONFIG);
        std::fs::write(&config, b"{not json").unwrap();
        assert!(prune_unused(directory.path(), true).is_err());
        std::fs::remove_file(&config).unwrap();
        assert!(prune_unused(directory.path(), true).is_err());
        assert_eq!(binary_names(&root), ["hive-a.exe", "hive-old.exe"]);
    }

    #[test]
    fn manual_prune_previews_then_keeps_recorded_executables() {
        let (directory, root) = fixture(false);
        let bin = root.requested_path().join(BASE).join("bin");
        seed_binaries(
            &root,
            &[
                "hive-a.exe",
                "hive-b.exe",
                "hive-old.exe",
                "hive-prior.exe",
                "hive-next.exe",
            ],
        );
        write(
            &root,
            JOURNAL,
            &json!({"from":{"executable":bin.join("hive-prior.exe")},
                "to":{"executable":bin.join("hive-next.exe")}}),
        )
        .unwrap();
        let before = binary_names(&root);
        let preview = prune_unused(directory.path(), false).unwrap();
        assert_eq!(binary_names(&root), before);
        assert_eq!(preview["candidates"].as_array().unwrap().len(), 2);
        let applied = prune_unused(directory.path(), true).unwrap();
        assert_eq!(applied["removed"].as_array().unwrap().len(), 2);
        assert_eq!(
            binary_names(&root),
            ["hive-next.exe", "hive-old.exe", "hive-prior.exe"]
        );
    }

    #[test]
    fn same_version_rerun_keeps_the_prior_source_executable() {
        let (_directory, root) = fixture(false);
        let current: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let prior = root.requested_path().join(BASE).join("bin/hive-prior.exe");
        seed_binaries(&root, &["hive-a.exe", "hive-old.exe", "hive-prior.exe"]);
        prune_binaries(&root, &current, &current, Some(prior.as_path()));
        assert_eq!(binary_names(&root), ["hive-old.exe", "hive-prior.exe"]);
    }

    #[test]
    fn binary_cleanup_keeps_only_current_and_previous_executables() {
        let (_directory, root) = fixture(false);
        let previous: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let mut current = previous.clone();
        current.executable = root.requested_path().join(BASE).join("bin/hive-new.exe");
        seed_binaries(
            &root,
            &[
                "hive-a.exe",
                "hive-b.exe",
                "hive-c.exe",
                "hive-old.exe",
                "hive-new.exe",
            ],
        );
        std::fs::write(
            root.requested_path().join(BASE).join("bin/notes.txt"),
            b"foreign",
        )
        .unwrap();

        let report = prune_binaries(&root, &current, &previous, None);

        assert_eq!(
            binary_names(&root),
            ["hive-new.exe", "hive-old.exe", "notes.txt"]
        );
        assert_eq!(report["removed"].as_array().unwrap().len(), 3);
        assert_eq!(
            report["skipped"].as_array().unwrap().as_slice(),
            &[] as &[Value]
        );
        assert!(report["error"].is_null());
    }

    #[test]
    fn binary_cleanup_skips_running_or_locked_files_and_reports_them() {
        let (_directory, root) = fixture(false);
        let previous: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let mut current = previous.clone();
        current.executable = root.requested_path().join(BASE).join("bin/hive-new.exe");
        seed_binaries(
            &root,
            &[
                "hive-a.exe",
                "hive-running.exe",
                "hive-old.exe",
                "hive-new.exe",
            ],
        );

        let keep = retained_binaries(&root, &[&current.executable, &previous.executable]).unwrap();
        let report = prune_binaries_with(&root, &keep, false, |root, relative, snapshot| {
            if relative.ends_with("hive-running.exe") {
                return Err(error("file is in use"));
            }
            root.remove_runtime(relative, snapshot)
        });

        assert_eq!(
            binary_names(&root),
            ["hive-new.exe", "hive-old.exe", "hive-running.exe"]
        );
        let skipped = report["skipped"].as_array().unwrap();
        assert_eq!(skipped.len(), 1);
        assert!(skipped[0]["reason"].as_str().unwrap().contains("in use"));
    }

    #[test]
    fn binary_cleanup_failure_leaves_existing_files_unchanged() {
        let (_directory, root) = fixture(false);
        let previous: Configuration = load(&root, CONFIG).unwrap().unwrap();
        let mut current = previous.clone();
        current.executable = root.requested_path().join(BASE).join("bin/hive-new.exe");
        seed_binaries(
            &root,
            &["hive-a.exe", "hive-b.exe", "hive-old.exe", "hive-new.exe"],
        );
        let before = binary_names(&root);

        let keep = retained_binaries(&root, &[&current.executable, &previous.executable]).unwrap();
        let report = prune_binaries_with(&root, &keep, false, |_, _, _| {
            Err(error("injected removal failure"))
        });

        assert_eq!(binary_names(&root), before);
        assert_eq!(report["skipped"].as_array().unwrap().len(), 2);
        assert_eq!(
            report["removed"].as_array().unwrap().as_slice(),
            &[] as &[Value]
        );
        for name in &before {
            assert_eq!(
                std::fs::read(root.requested_path().join(BASE).join("bin").join(name)).unwrap(),
                name.as_bytes()
            );
        }
    }

    #[test]
    fn setup_checking_recovery_only_verifies_readiness_and_preserves_later_choices() {
        for phase in ["checking", "incomplete", "complete"] {
            let (_directory, root) = fixture(false);
            let current = configuration(root.requested_path()).unwrap();
            write(&root, CONFIG, &current).unwrap();
            let stopped = json!({"schema_version":1,"operation":"disable","operation_id":"setup-a",
                "phase":"complete","after_digest":sha256_digest(b"old paused configuration")});
            let prior = json!({"schema_version":1,"setup_operation":"setup-a","phase":phase,
                "to":current,"error":"prior timeout","completed_at":null});
            write(&root, super::super::transition::JOURNAL, &stopped).unwrap();
            write(&root, JOURNAL, &prior).unwrap();
            let before_state = root.read_required(Path::new(STATE), 65536).unwrap();
            let before_config = root.read_required(Path::new(CONFIG), 65536).unwrap();
            assert!(finish_saved_setup_upgrade(
                &root,
                "setup-a",
                &stopped,
                &current,
                &prior,
                |_, _| Err(error("still not ready"))
            )
            .is_err());
            assert_eq!(load::<Value>(&root, JOURNAL).unwrap().unwrap(), prior);
            let health =
                finish_saved_setup_upgrade(&root, "setup-a", &stopped, &current, &prior, |_, _| {
                    Ok(json!({"controller_verified":true,"state":"idle-awaiting-Codex"}))
                })
                .unwrap();
            assert_eq!(health["controller_verified"], true);
            assert_eq!(
                load::<Value>(&root, JOURNAL).unwrap().unwrap()["phase"],
                "complete"
            );
            assert_eq!(
                root.read_required(Path::new(STATE), 65536).unwrap(),
                before_state
            );
            assert_eq!(
                root.read_required(Path::new(CONFIG), 65536).unwrap(),
                before_config
            );
            write(&root, JOURNAL, &prior).unwrap();
            assert!(finish_saved_setup_upgrade(
                &root,
                "setup-a",
                &stopped,
                &current,
                &prior,
                |root, _| {
                    let mut later = stopped.clone();
                    later["operation_id"] = Value::Null;
                    write(root, super::super::transition::JOURNAL, &later)?;
                    Ok(json!({"controller_verified":true}))
                }
            )
            .is_err());
            assert_eq!(load::<Value>(&root, JOURNAL).unwrap().unwrap(), prior);
            assert!(load::<Value>(&root, super::super::transition::JOURNAL)
                .unwrap()
                .unwrap()["operation_id"]
                .is_null());
        }
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
    fn disabled_v2_policy_stops_leftover_registration_without_inspection_or_staging() {
        for (master, midway) in [(false, true), (true, false)] {
            let (_directory, root) = fixture(true);
            let mut old: Configuration = load(&root, CONFIG).unwrap().unwrap();
            restore_owned_launcher(&mut old);
            write(&root, CONFIG, &old).unwrap();
            let policy = hive_core::preferences::UsageProtection {
                enforcement_enabled: master,
                midway: hive_core::preferences::MidwayPreferences {
                    enabled: midway,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut guard = serde_json::to_value(policy).unwrap();
            guard["stop_remaining_percent"] = json!(20);
            guard["popup_enabled"] = json!(false);
            let settings = json!({"schema_version":2,"interface_language":"en","wiki":{"language":"ko"},
                "profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},
                "selected_hosts":["codex"],"skills":{"mode":"all"},"usage_guard":guard});
            let path = root.requested_path().join(".hive/config/user-setup.yml");
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, serde_json::to_vec(&settings).unwrap()).unwrap();
            let before = root.read_required(Path::new(STATE), 65536).unwrap();
            let calls = RefCell::new(Vec::new());
            let result = upgrade_with(
                &root,
                |request, _| {
                    calls
                        .borrow_mut()
                        .push(request["operation"].as_str().unwrap().to_owned());
                    assert_eq!(request["operation"], "unregister");
                    Ok(json!({"removed":true}))
                },
                |_, _| panic!("disabled policy cannot observe health"),
            )
            .unwrap();
            assert_eq!(calls.into_inner(), ["unregister"]);
            assert_eq!(result["controller_stopped"], true);
            assert!(
                !load::<Configuration>(&root, CONFIG)
                    .unwrap()
                    .unwrap()
                    .enabled
            );
            assert_eq!(root.read_required(Path::new(STATE), 65536).unwrap(), before);
            assert!(root
                .read_optional(Path::new(JOURNAL), 65536)
                .unwrap()
                .is_none());
        }
    }

    #[test]
    fn identical_large_monitor_binary_can_be_staged_twice_without_rewriting() {
        let (_directory, root) = fixture(false);
        let bytes = vec![0x6d; 1024 * 1024 + 17];
        let mut next = configuration(root.requested_path()).unwrap();
        next.executable_digest = sha256_digest(&bytes);
        stage_bytes(&root, &next, &bytes).unwrap();
        let relative = owned_binary_relative(&root, &next.executable).unwrap();
        let before = root.snapshot_bounded(&relative, 128 * 1024 * 1024).unwrap();
        assert!(before.bytes().unwrap().len() > 1024 * 1024);
        let modified = std::fs::metadata(&next.executable)
            .unwrap()
            .modified()
            .unwrap();
        stage_bytes(&root, &next, &bytes).unwrap();
        assert_eq!(
            std::fs::metadata(&next.executable)
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
        assert_eq!(
            root.snapshot_bounded(&relative, 128 * 1024 * 1024)
                .unwrap()
                .bytes(),
            before.bytes()
        );
    }

    #[test]
    fn differing_existing_version_bound_binary_is_preserved() {
        let (_directory, root) = fixture(false);
        let next = configuration(root.requested_path()).unwrap();
        let relative = owned_binary_relative(&root, &next.executable).unwrap();
        let absent = root.snapshot(&relative).unwrap();
        root.publish_runtime(&relative, &absent, b"preserve unexpected content")
            .unwrap();
        assert!(stage(&root, &next).is_err());
        assert_eq!(
            root.snapshot(&relative).unwrap().bytes(),
            Some(b"preserve unexpected content".as_slice())
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
                Ok(json!({"available":true,"app_count":1,"desktop_package_version":"99.0.0.0"}))
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
                Ok(json!({"available":true,"app_count":1,"desktop_package_version":"99.0.0.0"}))
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
        let foreign = crate::test_tempdir();
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
                Ok(json!({"available":true,"app_count":1,"desktop_package_version":"99.0.0.0"}))
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
    fn setup_resume_activates_only_the_new_reader_and_honors_a_later_manual_disable() {
        let (_directory, root) = fixture(false);
        let mut old: Configuration = load(&root, CONFIG).unwrap().unwrap();
        restore_owned_launcher(&mut old);
        write(&root, CONFIG, &old).unwrap();
        let before = sha256_digest(&super::super::serialize(&old).unwrap());
        let stopped = super::super::transition::quiesce_operation(
            &root,
            |_, _| Ok(json!({"removed":true})),
            Duration::ZERO,
            Some(&before),
            Some("migration-a"),
        )
        .unwrap();
        drop(stopped);
        verify_upgrade_inputs(&root, &old, Some("migration-a")).unwrap();
        let result = upgrade_with_choice(
            &root,
            |request, _| {
                let current: Configuration = load(&root, CONFIG)?.unwrap();
                if matches!(
                    request["operation"].as_str(),
                    Some("inspect" | "unregister")
                ) {
                    assert!(!current.enabled);
                    assert_eq!(current.version, "0.13.0");
                } else {
                    assert!(current.enabled);
                    assert_eq!(current.version, env!("HIVE_PACKAGE_VERSION"));
                }
                Ok(json!({"available":true,"app_count":1,"desktop_package_version":"99.0.0.0"}))
            },
            |_, _| Ok(json!({"fresh_observations":4})),
            Some("migration-a"),
        )
        .unwrap();
        assert_eq!(result["enabled"], true);
        super::super::transition::disable(
            &root,
            |_, _| Ok(json!({"removed":true})),
            Duration::ZERO,
        )
        .unwrap();
        let disabled: Configuration = load(&root, CONFIG).unwrap().unwrap();
        assert!(!disabled.enabled);
        assert!(verify_upgrade_inputs(&root, &disabled, Some("migration-a")).is_err());
    }

    #[test]
    fn failed_transition_is_incomplete_and_does_not_change_opt_in_or_replay_old_incident() {
        let (_directory, root) = fixture(true);
        assert!(upgrade_with(
            &root,
            |_, _| Ok(json!({"available":true,"app_count":1,"desktop_package_version":"99.0.0.0"})),
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
