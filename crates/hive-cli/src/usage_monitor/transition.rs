//! Quiesce the exact owned controller before changing configuration reader formats.
use super::{
    error, load, lock, monitor_root_path, monitor_task_name, now, task_launcher, task_request,
    upgrade, write, Configuration, CONFIG,
};
use crate::run::{AdapterError, PinnedTarget};
use hive_core::sha256_digest;
use serde_json::{json, Value};
use std::{
    thread,
    time::{Duration, Instant},
};

pub(super) const JOURNAL: &str = ".hive/runtime/usage-monitor/disable.json";

fn selection_id() -> Result<String, AdapterError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| error("cannot identify monitor selection"))?;
    Ok(sha256_digest(&bytes))
}

/// Missing configuration is distinct from malformed or unverifiable configuration.
pub(super) fn selection(root: &PinnedTarget) -> Result<Option<bool>, AdapterError> {
    let Some(config) = authenticated_config(root)? else {
        return Ok(None);
    };
    if config.enabled {
        let binary = upgrade::owned_binary_relative(root, &config.executable)?;
        let bytes = root.read_required(&binary, 128 * 1024 * 1024)?;
        if sha256_digest(&bytes) != config.executable_digest {
            return Err(error("monitor executable does not match its saved choice"));
        }
    }
    Ok(Some(config.enabled))
}

fn authenticated_config(root: &PinnedTarget) -> Result<Option<Configuration>, AdapterError> {
    let Some(config): Option<Configuration> = load(root, CONFIG)? else {
        for path in [super::STATE, super::STARTUP] {
            if root
                .read_optional(std::path::Path::new(path), 65536)?
                .is_some()
            {
                return Err(error("monitor history exists without its saved choice"));
            }
        }
        return Ok(None);
    };
    upgrade::authenticate_ownership(
        root,
        &config,
        &monitor_task_name(&monitor_root_path(root.requested_path())?),
    )?;
    let root_text = config
        .arguments
        .strip_prefix("usage monitor run --user-root \"")
        .and_then(|text| text.strip_suffix("\" --output json"))
        .ok_or_else(|| error("invalid saved monitor arguments"))?;
    if config.launcher.is_some() && config.launcher != task_launcher(&config.executable, root_text)
    {
        return Err(error("saved monitor launcher is not a Hive action"));
    }
    if !crate::is_sha256_digest(&config.executable_digest) || !valid_version(&config.version) {
        return Err(error("saved monitor version or digest is invalid"));
    }
    Ok(Some(config))
}

fn valid_version(version: &str) -> bool {
    let (product, suffix) = version.split_once('-').unwrap_or((version, ""));
    product.parse::<hive_update::SemVersion>().is_ok()
        && (suffix.is_empty()
            || suffix == "test"
            || suffix.strip_prefix("test.").is_some_and(|revision| {
                !revision.starts_with('0') && revision.parse::<u64>().is_ok_and(|number| number > 0)
            }))
}

pub(super) fn disable(
    root: &PinnedTarget,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    timeout: Duration,
) -> Result<Value, AdapterError> {
    quiesce(root, call, timeout, None).map(|stopped| {
        let mut result = stopped.result.clone();
        match super::activation::retire_disabled(root) {
            Ok(retired) => result["activation_retirement"] = json!(if retired { "cancelled" } else { "not-needed" }),
            Err(error) => {
                result["activation_retirement"] = json!("pending");
                result["activation_retirement_error"] = json!(error.message());
            }
        }
        result
    })
}

pub(super) fn verify_stopped(root: &PinnedTarget) -> Result<(), AdapterError> {
    if authenticated_config(root)?.is_none() {
        return Err(error("monitor stop configuration is missing"));
    }
    let _control = super::existing_lock(root, "control.lock")?;
    let _controller = super::existing_lock(root, "controller.lock")?;
    let config = authenticated_config(root)?.ok_or_else(|| error("monitor stop configuration is missing"))?;
    let journal: Value = load(root, JOURNAL)?.ok_or_else(|| error("monitor stop receipt is missing"))?;
    let bytes = root.read_required(std::path::Path::new(CONFIG), 65536)?;
    if config.enabled || journal["schema_version"] != 1 || journal["operation"] != "disable"
        || journal["phase"] != "complete" || journal["completed_at"].as_u64().is_none()
        || journal["after_digest"] != sha256_digest(&bytes)
        || super::serialize(&config)? != bytes {
        return Err(error("monitor stop is incomplete or its saved configuration changed"));
    }
    root.verify_current()
}

pub(crate) struct StoppedReader {
    _control: std::fs::File,
    _controller: std::fs::File,
    result: Value,
}

pub(super) fn quiesce(
    root: &PinnedTarget,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    timeout: Duration,
    expected_digest: Option<&str>,
) -> Result<StoppedReader, AdapterError> {
    quiesce_operation(root, call, timeout, expected_digest, None)
}

pub(super) fn quiesce_operation(
    root: &PinnedTarget,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    timeout: Duration,
    expected_digest: Option<&str>,
    operation_id: Option<&str>,
) -> Result<StoppedReader, AdapterError> {
    let control = lock(root, "control.lock")?;
    let before = root.read_optional(std::path::Path::new(CONFIG), 65536)?;
    let digest = before
        .as_deref()
        .map_or_else(|| "absent".into(), sha256_digest);
    if expected_digest.is_some_and(|expected| expected != digest) {
        let prior: Option<Value> = load(root, JOURNAL)?;
        if !prior.is_some_and(|journal| {
            operation_id.is_some_and(|id| journal["operation_id"] == id)
                && journal["before_digest"] == expected_digest.unwrap_or_default()
                && journal["after_digest"] == digest
        }) {
            return Err(error("monitor choice changed since setup preview"));
        }
    }
    let candidate = if before.is_none() && expected_digest.is_none() && operation_id.is_none() {
        super::activation::pending_disabled_configuration(root)?
    } else { None };
    let config = match candidate { Some(config) => Some(config), None => authenticated_config(root)? };
    let Some(mut config) = config else {
        let controller = lock(root, "controller.lock")?;
        // A later explicit disable must cancel a pending activation even if no configuration
        // has been written yet. This records intent, not an ownership or execution grant.
        write(root, JOURNAL, &json!({"schema_version":1,"operation":"disable",
            "selection_id":selection_id()?,"operation_id":operation_id,
            "before_digest":"absent","after_digest":"absent","configured":false,
            "phase":"complete","started_at":now(),"completed_at":now(),"error":null}))?;
        return Ok(StoppedReader {
            _control: control,
            _controller: controller,
            result: json!({"enabled":false,"configured":false,"controller_stopped":null,"changed_paths":[JOURNAL]}),
        });
    };
    // Persist the requested stop before touching the scheduler. A retry must never re-enable it.
    config.enabled = false;
    let mut journal = json!({"schema_version":1,"operation":"disable","started_at":now(),
        "selection_id":selection_id()?,
        "operation_id":operation_id,"before_digest":expected_digest.unwrap_or(&digest),
        "after_digest":sha256_digest(&super::serialize(&config)?),"phase":"stopping",
        "completed_at":null,"error":null});
    write(root, JOURNAL, &journal)?;
    write(root, CONFIG, &config)?;
    let result = (|| {
        if call(&task_request(&config, "unregister"), timeout)?["removed"] != true {
            return Err(error("monitor task removal was not confirmed"));
        }
        let controller = await_controller(root, timeout)?;
        root.verify_current()?;
        let current = authenticated_config(root)?
            .ok_or_else(|| error("monitor configuration removed during stop"))?;
        if current.enabled || super::serialize(&current)? != super::serialize(&config)? {
            return Err(error("monitor configuration changed during stop"));
        }
        journal["phase"] = json!("complete");
        journal["completed_at"] = json!(now());
        write(root, JOURNAL, &journal)?;
        Ok(StoppedReader {
            _control: control,
            _controller: controller,
            result: json!({"enabled":false,"configured":true,"controller_stopped":true,
                "incident_preserved":true,"changed_paths":[CONFIG,JOURNAL]}),
        })
    })();
    if let Err(failure) = &result {
        journal["phase"] = json!("incomplete");
        journal["error"] = json!(failure.message());
        let _ = write(root, JOURNAL, &journal);
    }
    result
}

fn await_controller(root: &PinnedTarget, timeout: Duration) -> Result<std::fs::File, AdapterError> {
    let began = Instant::now();
    loop {
        match lock(root, "controller.lock") {
            Ok(lease) => return Ok(lease),
            Err(AdapterError::Conflict(_)) if began.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(failure) => return Err(failure),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture() -> (tempfile::TempDir, PinnedTarget) {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let mut config = super::super::configuration(root.requested_path()).unwrap();
        config.executable_digest = sha256_digest(b"old binary");
        let path = upgrade::owned_binary_relative(&root, &config.executable).unwrap();
        root.ensure_owned_parent(&path, Path::new(".hive")).unwrap();
        std::fs::write(root.requested_path().join(path), b"old binary").unwrap();
        write(&root, CONFIG, &config).unwrap();
        (temp, root)
    }

    #[test]
    fn repeated_disable_has_a_new_identity_even_without_configuration_or_value_changes() {
        let temp = tempfile::tempdir().unwrap();
        let empty = PinnedTarget::open_usage(temp.path()).unwrap();
        let (_configured, configured) = fixture();
        for root in [&empty, &configured] {
            let expected_configured = selection(root).unwrap().is_some();
            let call = |_: &Value, _: Duration| {
                assert!(expected_configured, "no scheduler access without an owned configuration");
                Ok(json!({"removed":true}))
            };
            disable(root, call, Duration::ZERO).unwrap();
            let first: Value = load(root, JOURNAL).unwrap().unwrap();
            let configuration = root.read_optional(std::path::Path::new(CONFIG), 65536).unwrap();
            disable(root, call, Duration::ZERO).unwrap();
            let second: Value = load(root, JOURNAL).unwrap().unwrap();
            assert_ne!(first["selection_id"], second["selection_id"]);
            assert!(crate::is_sha256_digest(second["selection_id"].as_str().unwrap()));
            assert_eq!(first["after_digest"], second["after_digest"]);
            assert_eq!(second["phase"], "complete");
            assert_eq!(root.read_optional(std::path::Path::new(CONFIG), 65536).unwrap(), configuration);
            assert_eq!(selection(root).unwrap(), expected_configured.then_some(false));
        }
    }

    #[test]
    fn reader_lease_holds_both_locks_until_the_settings_writer_finishes() {
        let (_temp, root) = fixture();
        let before = root.read_required(Path::new(CONFIG), 65536).unwrap();
        let stopped = quiesce(
            &root,
            |_, _| Ok(json!({"removed":true})),
            Duration::ZERO,
            Some(&sha256_digest(&before)),
        )
        .unwrap();
        assert_eq!(stopped.result["controller_stopped"], true);
        assert!(lock(&root, "controller.lock").is_err());
        assert!(lock(&root, "control.lock").is_err());
        assert!(
            !load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
        drop(stopped);
        assert!(lock(&root, "controller.lock").is_ok());
        assert!(lock(&root, "control.lock").is_ok());
    }

    #[test]
    fn changed_monitor_choice_is_rejected_before_stop_or_rewrite() {
        let (_temp, root) = fixture();
        let before = root.read_required(Path::new(CONFIG), 65536).unwrap();
        assert!(quiesce(
            &root,
            |_, _| panic!("changed choice stopped"),
            Duration::ZERO,
            Some(&sha256_digest(b"different preview"))
        )
        .is_err());
        assert_eq!(
            root.read_required(Path::new(CONFIG), 65536).unwrap(),
            before
        );
        assert!(root
            .read_optional(Path::new(JOURNAL), 65536)
            .unwrap()
            .is_none());
    }

    #[test]
    fn only_the_same_setup_operation_can_resume_its_partial_stop() {
        let (_temp, root) = fixture();
        let before = root.read_required(Path::new(CONFIG), 65536).unwrap();
        let digest = sha256_digest(&before);
        assert!(quiesce_operation(
            &root,
            |_, _| Err(error("scheduler refused")),
            Duration::ZERO,
            Some(&digest),
            Some("setup-a")
        )
        .is_err());
        assert!(quiesce_operation(
            &root,
            |_, _| panic!("another setup stopped it"),
            Duration::ZERO,
            Some(&digest),
            Some("setup-b")
        )
        .is_err());
        let stopped = quiesce_operation(
            &root,
            |_, _| Ok(json!({"removed":true})),
            Duration::ZERO,
            Some(&digest),
            Some("setup-a"),
        )
        .unwrap();
        assert_eq!(stopped.result["controller_stopped"], true);
        assert_eq!(
            load::<Value>(&root, JOURNAL).unwrap().unwrap()["before_digest"],
            digest
        );
    }

    #[test]
    fn saved_selection_distinguishes_absent_disabled_enabled_and_damaged() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        assert_eq!(selection(&root).unwrap(), None);
        let (_temp, root) = fixture();
        assert_eq!(selection(&root).unwrap(), Some(true));
        let mut config: Configuration = load(&root, CONFIG).unwrap().unwrap();
        config.enabled = false;
        write(&root, CONFIG, &config).unwrap();
        assert_eq!(selection(&root).unwrap(), Some(false));
        config.task_name = "foreign-task".into();
        write(&root, CONFIG, &config).unwrap();
        assert!(selection(&root).is_err());
        std::fs::write(root.requested_path().join(CONFIG), b"broken").unwrap();
        assert!(selection(&root).is_err());
    }

    #[test]
    fn forged_launcher_or_active_binary_is_not_a_verified_selection() {
        let (_temp, root) = fixture();
        let mut config: Configuration = load(&root, CONFIG).unwrap().unwrap();
        config.executable_digest = sha256_digest(b"different");
        write(&root, CONFIG, &config).unwrap();
        assert!(selection(&root).is_err());
        config.enabled = false;
        config.launcher = Some(super::super::TaskLauncher {
            executable: "foreign.exe".into(),
            arguments: "run".into(),
        });
        write(&root, CONFIG, &config).unwrap();
        assert!(disable(&root, |_, _| panic!("unowned task touched"), Duration::ZERO).is_err());
    }

    #[test]
    fn disable_waits_for_exact_controller_and_preserves_history_on_retry() {
        let (_temp, root) = fixture();
        let state = root.requested_path().join(super::super::STATE);
        std::fs::write(&state, b"opaque old controller state").unwrap();
        let lease = lock(&root, "controller.lock").unwrap();
        assert!(disable(
            &root,
            |request, _| {
                assert_eq!(request["operation"], "unregister");
                Ok(json!({"removed":true}))
            },
            Duration::ZERO
        )
        .is_err());
        assert!(
            !load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(
            load::<Value>(&root, JOURNAL).unwrap().unwrap()["phase"],
            "incomplete"
        );
        drop(lease);
        let result = disable(&root, |_, _| Ok(json!({"removed":true})), Duration::ZERO).unwrap();
        assert_eq!(result["controller_stopped"], true);
        assert_eq!(
            std::fs::read(state).unwrap(),
            b"opaque old controller state"
        );
        assert_eq!(
            load::<Value>(&root, JOURNAL).unwrap().unwrap()["phase"],
            "complete"
        );
    }

    #[test]
    fn scheduler_failure_keeps_stop_intent_without_claiming_completion() {
        let (_temp, root) = fixture();
        assert!(disable(
            &root,
            |_, _| Err(error("scheduler unavailable")),
            Duration::ZERO
        )
        .is_err());
        assert_eq!(selection(&root).unwrap(), Some(false));
        assert_eq!(
            load::<Value>(&root, JOURNAL).unwrap().unwrap()["phase"],
            "incomplete"
        );
        assert!(verify_stopped(&root).is_err());
    }

    #[test]
    fn setup_stop_verification_requires_a_matching_receipt_and_no_live_controller() {
        let (_temp, root) = fixture();
        assert!(verify_stopped(&root).is_err());
        disable(&root, |_, _| Ok(json!({"removed":true})), Duration::ZERO).unwrap();
        verify_stopped(&root).unwrap();
        let lease = lock(&root, "controller.lock").unwrap();
        assert!(verify_stopped(&root).is_err());
        drop(lease);
        let mut journal: Value = load(&root, JOURNAL).unwrap().unwrap();
        journal["after_digest"] = json!(sha256_digest(b"another configuration"));
        write(&root, JOURNAL, &journal).unwrap();
        assert!(verify_stopped(&root).is_err());
    }

    #[test]
    fn orphan_history_and_unconfirmed_task_removal_are_incomplete() {
        let (_temp, root) = fixture();
        assert!(disable(&root, |_, _| Ok(json!({"removed":false})), Duration::ZERO).is_err());
        std::fs::write(root.requested_path().join(super::super::STATE), b"history").unwrap();
        std::fs::remove_file(root.requested_path().join(CONFIG)).unwrap();
        assert!(selection(&root).is_err());
        assert!(disable(&root, |_, _| panic!("unowned action"), Duration::ZERO).is_err());
    }
}
