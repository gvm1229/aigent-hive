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
    if authenticated_config(root)?.is_none() {
        return Ok(json!({"enabled":false,"configured":false,"controller_stopped":null}));
    }
    let _control = lock(root, "control.lock")?;
    let mut config =
        authenticated_config(root)?.ok_or_else(|| error("monitor configuration changed"))?;
    let mut journal = json!({"schema_version":1,"operation":"disable","started_at":now(),
        "before_digest":sha256_digest(&super::serialize(&config)?),"phase":"stopping",
        "completed_at":null,"error":null});
    write(root, JOURNAL, &journal)?;
    // Persist the requested stop before touching the scheduler. A retry must never re-enable it.
    config.enabled = false;
    write(root, CONFIG, &config)?;
    let result = (|| {
        if call(&task_request(&config, "unregister"), timeout)?["removed"] != true {
            return Err(error("monitor task removal was not confirmed"));
        }
        let began = Instant::now();
        let _controller = loop {
            match lock(root, "controller.lock") {
                Ok(lease) => break lease,
                Err(AdapterError::Conflict(_)) if began.elapsed() < timeout => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(failure) => return Err(failure),
            }
        };
        root.verify_current()?;
        let current = authenticated_config(root)?
            .ok_or_else(|| error("monitor configuration removed during stop"))?;
        if current.enabled || super::serialize(&current)? != super::serialize(&config)? {
            return Err(error("monitor configuration changed during stop"));
        }
        journal["phase"] = json!("complete");
        journal["completed_at"] = json!(now());
        write(root, JOURNAL, &journal)?;
        Ok(
            json!({"enabled":false,"configured":true,"controller_stopped":true,
            "incident_preserved":true,"changed_paths":[CONFIG,JOURNAL]}),
        )
    })();
    if let Err(failure) = &result {
        journal["phase"] = json!("incomplete");
        journal["error"] = json!(failure.message());
        let _ = write(root, JOURNAL, &journal);
    }
    result
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
