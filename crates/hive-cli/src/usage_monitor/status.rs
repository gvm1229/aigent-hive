//! A bounded, read-only setup view separates saved intent from verified runtime facts.
use super::{monitor_status, transition, AdapterError, PinnedTarget};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn setup_status(user_root: &Path) -> Value {
    inspect(user_root).unwrap_or_else(|error| {
        json!({
            "configuration":"unverified","saved_enabled":null,"controller_state":"unverified",
            "monitoring_verified":false,"error":error.message(),"sensor_queries":0
        })
    })
}

fn inspect(user_root: &Path) -> Result<Value, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    let status = monitor_status(&root)?;
    let stopped =
        status["verified_saved_selection"] == false && transition::verify_stopped(&root).is_ok();
    root.verify_current()?;
    Ok(summarize(&status, stopped))
}

fn summarize(status: &Value, stopped: bool) -> Value {
    let verified = status["saved_selection_verified"] == true;
    let absent = status["configuration"].is_null() && verified;
    let running = verified && status["controller_alive"] == true;
    let state = if running {
        "running"
    } else if stopped {
        "stopped"
    } else if absent {
        "not-configured"
    } else {
        "unverified"
    };
    json!({
        "configuration":if absent {"absent"} else if verified {"verified"} else {"unverified"},
        "saved_enabled":status["verified_saved_selection"],"controller_state":state,
        "stop_verified":stopped,"monitoring_verified":status["monitoring_verified"],
        "configured_version":status["configured_monitor_version"],"running_version":status["running_monitor_version"],
        "heartbeat_fresh":status["controller_heartbeat_fresh"],
        "observation":{"count":status["state"]["observation_count"],"last_success":status["state"]["last_success"],
            "health":if state=="unverified" {json!("unverified")} else {status["sensor_health"].clone()}},
        "control":{"supported_on_this_os":status["automatic_control_available"],
            "compatibility":status["desktop_compatibility"]["state"],"connected":status["state"]["control_connected"]},
        "sensor_queries":0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn setup_status_keeps_absent_disabled_and_verified_stopped_distinct_without_writes() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        assert_eq!(
            setup_status(temp.path())["controller_state"],
            "not-configured"
        );
        assert!(!temp.path().join(".hive").exists());
        let mut configuration = super::super::configuration(temp.path()).unwrap();
        configuration.enabled = false;
        super::super::write(&root, super::super::CONFIG, &configuration).unwrap();
        let before = std::fs::read(temp.path().join(super::super::CONFIG)).unwrap();
        let unknown = setup_status(temp.path());
        assert_eq!(unknown["saved_enabled"], false);
        assert_eq!(unknown["controller_state"], "unverified");
        assert!(!temp
            .path()
            .join(super::super::BASE)
            .join("control.lock")
            .exists());
        super::super::transition::disable(
            &root,
            |_, _| Ok(json!({"removed":true})),
            Duration::ZERO,
        )
        .unwrap();
        let receipt = std::fs::read(temp.path().join(super::super::transition::JOURNAL)).unwrap();
        let stopped = setup_status(temp.path());
        assert_eq!(stopped["controller_state"], "stopped");
        assert_eq!(stopped["stop_verified"], true);
        assert_eq!(stopped["sensor_queries"], 0);
        assert_eq!(
            std::fs::read(temp.path().join(super::super::CONFIG)).unwrap(),
            before
        );
        assert_eq!(
            std::fs::read(temp.path().join(super::super::transition::JOURNAL)).unwrap(),
            receipt
        );
        let lease = super::super::existing_lock(&root, "controller.lock").unwrap();
        assert_eq!(setup_status(temp.path())["controller_state"], "unverified");
        drop(lease);
        std::fs::write(temp.path().join(super::super::CONFIG), b"broken").unwrap();
        let damaged = setup_status(temp.path());
        assert_eq!(damaged["configuration"], "unverified");
        assert!(damaged["saved_enabled"].is_null());
        assert_eq!(
            std::fs::read(temp.path().join(super::super::CONFIG)).unwrap(),
            b"broken"
        );
    }

    #[test]
    fn running_controller_does_not_imply_verified_observation_or_control() {
        let data = summarize(
            &json!({"configuration":{},"saved_selection_verified":true,
            "verified_saved_selection":true,"controller_alive":true,"monitoring_verified":false,
            "state":{"observation_count":0,"control_connected":false}}),
            false,
        );
        assert_eq!(data["controller_state"], "running");
        assert_eq!(data["monitoring_verified"], false);
        assert_eq!(data["observation"]["count"], 0);
        assert_eq!(data["control"]["connected"], false);
    }
}
