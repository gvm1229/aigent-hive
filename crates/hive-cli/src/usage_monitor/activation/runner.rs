//! Deterministic activation; an uncertain native call is reconciled by its exact owned task.
use super::super::{
    configuration, error, load, lock, now, platform, prepare_activation, preview, preview_bound,
    require_app_safe_control, require_desktop_inspection, serialize, task_request,
    usage_policy_disabled, AdapterError, Configuration, MonitorState, PinnedTarget, CONFIG, STATE,
};
use super::{
    digest,
    journal::{Phase, Record, Store},
    Binding,
};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(in crate::usage_monitor) fn enable(
    root: &PinnedTarget,
    approval: &str,
) -> Result<Value, AdapterError> {
    require_app_safe_control()?;
    if usage_policy_disabled(root)? {
        return Err(error(
            "enable usage enforcement and select midway monitoring in setup first",
        ));
    }
    run_with(
        root,
        approval,
        platform::call,
        super::super::upgrade::await_ready,
    )
}

// Keep the persisted phase order visible beside each native effect.
#[allow(clippy::too_many_lines)]
fn run_with(
    root: &PinnedTarget,
    approval: &str,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    ready: impl Fn(&PinnedTarget, &Configuration) -> Result<Value, AdapterError>,
) -> Result<Value, AdapterError> {
    if usage_policy_disabled(root)? {
        return Err(error("monitor activation policy is disabled"));
    }
    let manifest: Value = load(root, ".hive/install/codex.json")?
        .ok_or_else(|| error("matching Codex installation required for monitor activation"))?;
    if manifest["product_version"] != env!("CARGO_PKG_VERSION") {
        return Err(error(
            "Codex installation differs from this monitor version",
        ));
    }
    let control = lock(root, "control.lock")?;
    let (mut store, saved) = Store::open(root)?;
    let mut record = match saved {
        Some(record) if record.phase == Phase::Cancelled && record.approval == approval => return Err(error("this monitor activation consent was cancelled; request a fresh preview")),
        Some(record) if record.approval==approval => record,
        Some(record) if !matches!(record.phase, Phase::Complete | Phase::Cancelled) => return Err(error("another monitor activation is incomplete; resume its recorded preview or keep it disabled")),
        _ => {
            if preview(root.requested_path())?["preview_digest"] != approval { return Err(error("monitor preview changed")); }
            let binding=Binding::capture(root)?;
            if binding.previous.enabled { return Err(error("monitor already enabled")); }
            let mut nonce=[0_u8;16]; getrandom::fill(&mut nonce).map_err(|_|error("cannot identify monitor activation"))?;
            let record=Record{schema_version:1,operation_id:hive_core::sha256_digest(&nonce)[7..].into(),approval:approval.into(),binding,
                desired:configuration(root.requested_path())?,previous_config:root.read_optional(Path::new(CONFIG),65536)?,
                phase:Phase::Prepared,started_at:now(),completed_at:None,state_before_digest:None,prepared_state:None};
            store.save(&record)?;fault(Phase::Prepared)?;record
        }
    };
    verify(root, &store, &record)?;
    if record.phase == Phase::Prepared {
        super::super::upgrade::stage(root, &record.desired)?;
        advance(&mut store, &mut record, Phase::BinaryStaged)?;
    }
    if record.phase == Phase::BinaryStaged {
        let mut paused = previous(&record)?.unwrap_or_else(|| record.desired.clone());
        paused.enabled = false;
        let desired = serialize(&paused)?;
        let current = root.read_optional(Path::new(CONFIG), 65536)?;
        if current.as_deref() != record.previous_config.as_deref()
            && current.as_deref() != Some(desired.as_slice())
        {
            return Err(error(
                "monitor configuration changed before activation pause",
            ));
        }
        store.publish(CONFIG, current.as_deref(), &desired, &record)?;
        advance(&mut store, &mut record, Phase::Pausing)?;
    }
    let mut controller = None;
    if matches!(
        record.phase,
        Phase::Pausing | Phase::Stopped | Phase::StatePrepared | Phase::Configured
    ) {
        controller = Some(await_controller(root)?);
        // A controller may have held the lease while an external edit arrived.
        verify(root, &store, &record)?;
    }
    if record.phase == Phase::Pausing {
        advance(&mut store, &mut record, Phase::Stopped)?;
    }
    if record.phase == Phase::Stopped {
        if record.prepared_state.is_none() {
            let current = root.read_optional(Path::new(STATE), 65536)?;
            let mut state = current
                .as_deref()
                .map(serde_json::from_slice::<MonitorState>)
                .transpose()
                .map_err(|_| error("invalid monitor state before activation"))?
                .unwrap_or_else(MonitorState::new);
            state.migrate()?;
            prepare_activation(&mut state);
            record.state_before_digest = Some(digest(current.as_deref()));
            record.prepared_state = Some(serialize(&state)?);
            store.save(&record)?;
        }
        let current = root.read_optional(Path::new(STATE), 65536)?;
        let desired = record.prepared_state.as_ref().expect("prepared state");
        if digest(current.as_deref())
            != *record.state_before_digest.as_ref().expect("state baseline")
            && current.as_deref() != Some(desired.as_slice())
        {
            return Err(error("monitor history changed during activation"));
        }
        store.publish(STATE, current.as_deref(), desired, &record)?;
        record.prepared_state = None;
        advance(&mut store, &mut record, Phase::StatePrepared)?;
    }
    if record.phase == Phase::StatePrepared {
        let mut paused = previous(&record)?.unwrap_or_else(|| record.desired.clone());
        paused.enabled = false;
        let current = root.read_optional(Path::new(CONFIG), 65536)?;
        let desired = serialize(&record.desired)?;
        if current.as_deref() != Some(serialize(&paused)?.as_slice())
            && current.as_deref() != Some(desired.as_slice())
        {
            return Err(error("monitor choice changed before activation"));
        }
        store.publish(CONFIG, current.as_deref(), &desired, &record)?;
        advance(&mut store, &mut record, Phase::Configured)?;
    }
    if record.phase == Phase::Configured {
        verify(root, &store, &record)?;
        verify_config(root, &record)?;
        require_desktop_inspection(&call(&json!({"operation":"inspect"}), timeout(5))?)?;
        if let Some(old) = previous(&record)? {
            if (old.task_name != record.desired.task_name
                || old.executable != record.desired.executable
                || old.launcher != record.desired.launcher)
                && call(&task_request(&old, "unregister"), timeout(15))?["removed"] != true
            {
                return Err(error("previous owned monitor task removal is unconfirmed"));
            }
        }
        advance(&mut store, &mut record, Phase::Registering)?;
    }
    if record.phase == Phase::Registering {
        verify(root, &store, &record)?;
        verify_config(root, &record)?;
        let status = call(&task_request(&record.desired, "inspect-task"), timeout(5))?;
        if status["present"].as_bool().is_none() {
            return Err(error("monitor registration state is unknown"));
        }
        if status["present"] == true && (status["owned"] != true || status["enabled"] != true) {
            return Err(error(
                "existing monitor task differs or was disabled; preserved",
            ));
        }
        if status["present"] != true {
            let mut request = task_request(&record.desired, "register");
            request["allow_existing_exact"] = json!(true);
            if call(&request, timeout(15))?["registered"] != true {
                return Err(error("monitor registration outcome is unconfirmed"));
            }
        }
        advance(&mut store, &mut record, Phase::Registered)?;
    }
    if record.phase == Phase::Registered {
        advance(&mut store, &mut record, Phase::Starting)?;
    }
    drop(controller);
    if record.phase == Phase::Starting {
        verify(root, &store, &record)?;
        verify_config(root, &record)?;
        if call(&task_request(&record.desired, "start"), timeout(10))?["started"] != true {
            return Err(error("monitor startup outcome is unconfirmed"));
        }
        advance(&mut store, &mut record, Phase::Checking)?;
    }
    if !matches!(record.phase, Phase::Checking | Phase::Complete) {
        return Err(error("monitor activation phase is incomplete"));
    }
    verify_config(root, &record)?;
    drop(control);
    let health = ready(root, &record.desired)?;
    if health["controller_verified"] != true
        || !matches!(
            health["state"].as_str(),
            Some("healthy" | "idle-awaiting-Codex")
        )
    {
        return Err(error("monitor readiness was not verified"));
    }
    let _control = lock(root, "control.lock")?;
    let (mut latest, saved) = Store::open(root)?;
    let saved = saved.ok_or_else(|| error("monitor activation record disappeared"))?;
    if saved.operation_id != record.operation_id
        || saved.approval != record.approval
        || saved.phase != record.phase
    {
        return Err(error("monitor activation changed during readiness check"));
    }
    verify(root, &latest, &saved)?;
    verify_config(root, &saved)?;
    let task = call(&task_request(&saved.desired, "inspect-task"), timeout(5))?;
    if task["present"] != true || task["owned"] != true || task["enabled"] != true {
        return Err(error(
            "monitor registration changed during readiness check; preserved",
        ));
    }
    if record.phase != Phase::Complete {
        record.completed_at = Some(now());
        advance(&mut latest, &mut record, Phase::Complete)?;
    }
    Ok(
        json!({"enabled":true,"activation":"verified","operation_id":record.operation_id,"health":health,
        "history_preserved":true,"changed_paths":[CONFIG,STATE,super::journal::PATH,record.desired.executable]}),
    )
}

fn previous(record: &Record) -> Result<Option<Configuration>, AdapterError> {
    record
        .previous_config
        .as_deref()
        .map(serde_json::from_slice)
        .transpose()
        .map_err(|_| error("invalid prior monitor configuration"))
}
fn verify(root: &PinnedTarget, store: &Store<'_>, record: &Record) -> Result<(), AdapterError> {
    root.verify_current()?;
    if usage_policy_disabled(root)? || store.disable_digest()? != record.binding.disable_digest {
        return Err(error(
            "a later disable selection cancelled this activation; no automatic reactivation",
        ));
    }
    if record.desired != configuration(root.requested_path())?
        || preview_bound(root, &record.binding)?["preview_digest"] != record.approval
    {
        return Err(error(
            "monitor activation program or scope changed; request a fresh preview",
        ));
    }
    verify_phase_config(root, record)?;
    Ok(())
}
fn verify_phase_config(root: &PinnedTarget, record: &Record) -> Result<(), AdapterError> {
    let current = root.read_optional(Path::new(CONFIG), 65536)?;
    let mut paused = previous(record)?.unwrap_or_else(|| record.desired.clone());
    paused.enabled = false;
    let paused = serialize(&paused)?;
    let desired = serialize(&record.desired)?;
    let matches = match record.phase {
        Phase::Prepared => current == record.previous_config,
        Phase::BinaryStaged => {
            current == record.previous_config || current.as_deref() == Some(paused.as_slice())
        }
        Phase::Pausing | Phase::Stopped => current.as_deref() == Some(paused.as_slice()),
        Phase::StatePrepared => {
            current.as_deref() == Some(paused.as_slice())
                || current.as_deref() == Some(desired.as_slice())
        }
        _ => current.as_deref() == Some(desired.as_slice()),
    };
    if !matches {
        return Err(error(
            "monitor configuration changed during activation; preserved",
        ));
    }
    Ok(())
}
fn verify_config(root: &PinnedTarget, record: &Record) -> Result<(), AdapterError> {
    let current: Configuration =
        load(root, CONFIG)?.ok_or_else(|| error("activated monitor configuration missing"))?;
    if current != record.desired {
        return Err(error("activated monitor choice changed; preserved"));
    }
    Ok(())
}
fn timeout(seconds: u64) -> Duration {
    crate::user_setup::command_timeout(Duration::from_secs(seconds))
}
fn await_controller(root: &PinnedTarget) -> Result<std::fs::File, AdapterError> {
    let until = Instant::now() + timeout(15);
    loop {
        match lock(root, "controller.lock") {
            Ok(file) => return Ok(file),
            Err(AdapterError::Conflict(_)) if Instant::now() < until => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error),
        }
    }
}
fn advance(store: &mut Store<'_>, record: &mut Record, phase: Phase) -> Result<(), AdapterError> {
    record.phase = phase;
    store.save(record)?;
    fault(phase)
}
#[cfg(test)]
thread_local! { static STOP_AT: std::cell::Cell<Option<Phase>> = const { std::cell::Cell::new(None) }; }
#[cfg_attr(not(test), allow(clippy::unnecessary_wraps))]
fn fault(phase: Phase) -> Result<(), AdapterError> {
    #[cfg(test)]
    if STOP_AT.with(|point| {
        if point.get() == Some(phase) {
            point.set(None);
            true
        } else {
            false
        }
    }) {
        return Err(error("fixture activation interruption"));
    }
    #[cfg(all(debug_assertions, not(test)))]
    if std::env::var("HIVE_MONITOR_ACTIVATION_INTERRUPT_AFTER")
        .ok()
        .as_deref()
        == Some(&format!("{phase:?}"))
    {
        std::process::exit(90);
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    pub(in crate::usage_monitor::activation) fn fixture(
    ) -> (tempfile::TempDir, PinnedTarget, String) {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let policy = hive_core::preferences::UsageProtection {
            enforcement_enabled: true,
            midway: hive_core::preferences::MidwayPreferences {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut guard = serde_json::to_value(policy).unwrap();
        guard["stop_remaining_percent"] = json!(20);
        guard["popup_enabled"] = json!(false);
        let settings = json!({"schema_version":2,"interface_language":"en","wiki":{"enabled":false,"language":"en"},
            "profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},"selected_hosts":["codex"],
            "skills":{"mode":"all"},"usage_guard":guard});
        std::fs::create_dir_all(temp.path().join(".hive/config")).unwrap();
        std::fs::create_dir_all(temp.path().join(".hive/install")).unwrap();
        std::fs::write(
            temp.path().join(".hive/config/user-setup.yml"),
            serde_json::to_vec(&settings).unwrap(),
        )
        .unwrap();
        std::fs::write(
            temp.path().join(".hive/install/codex.json"),
            serde_json::to_vec(&json!({"product_version":env!("CARGO_PKG_VERSION")})).unwrap(),
        )
        .unwrap();
        let approval = preview(temp.path()).unwrap()["preview_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        (temp, root, approval)
    }

    #[derive(Default)]
    struct Scheduler {
        present: Cell<bool>,
        running: Cell<bool>,
        effects: RefCell<Vec<String>>,
        fail: Cell<Option<&'static str>>,
    }
    impl Scheduler {
        fn call(&self, request: &Value, _: Duration) -> Result<Value, AdapterError> {
            let operation = request["operation"].as_str().unwrap();
            let result = match operation {
                "inspect" => json!({"available":true,"app_count":0}),
                "inspect-task" => {
                    json!({"present":self.present.get(),"owned":true,"enabled":true,"state":if self.running.get(){"Running"}else{"Ready"}})
                }
                "register" => {
                    if !self.present.replace(true) {
                        self.effects.borrow_mut().push("register".into());
                    }
                    json!({"registered":true})
                }
                "start" => {
                    assert!(self.present.get());
                    if !self.running.replace(true) {
                        self.effects.borrow_mut().push("start".into());
                    }
                    json!({"started":true})
                }
                "unregister" => {
                    self.present.set(false);
                    self.running.set(false);
                    self.effects.borrow_mut().push("unregister".into());
                    json!({"removed":true})
                }
                _ => panic!("unexpected native operation"),
            };
            if self.fail.get() == Some(operation) {
                self.fail.set(None);
                return Err(error("fixture lost native response"));
            }
            Ok(result)
        }
        fn ready(&self, _: &PinnedTarget, _: &Configuration) -> Result<Value, AdapterError> {
            if !self.running.get() {
                return Err(error("fixture controller is not running"));
            }
            Ok(json!({"controller_verified":true,"state":"idle-awaiting-Codex"}))
        }
    }

    pub(in crate::usage_monitor::activation) fn activate_fixture(root: &PinnedTarget, approval: &str) -> Result<Value, AdapterError> {
        let scheduler = Scheduler::default();
        let result = run_with(root, approval, |a,b| scheduler.call(a,b), |a,b| scheduler.ready(a,b))?;
        assert_eq!(*scheduler.effects.borrow(), ["register", "start"]);
        Ok(result)
    }

    #[test]
    fn activation_phase_recovery_preserves_one_registration_and_start() {
        for phase in [
            Phase::Prepared,
            Phase::BinaryStaged,
            Phase::Pausing,
            Phase::Stopped,
            Phase::StatePrepared,
            Phase::Configured,
            Phase::Registering,
            Phase::Registered,
            Phase::Starting,
            Phase::Checking,
            Phase::Complete,
        ] {
            let (_temp, root, approval) = fixture();
            let scheduler = Scheduler::default();
            STOP_AT.with(|point| point.set(Some(phase)));
            assert!(run_with(
                &root,
                &approval,
                |a, b| scheduler.call(a, b),
                |a, b| scheduler.ready(a, b)
            )
            .is_err());
            let resumed = run_with(
                &root,
                &approval,
                |a, b| scheduler.call(a, b),
                |a, b| scheduler.ready(a, b),
            )
            .unwrap();
            assert_eq!(resumed["activation"], "verified");
            assert_eq!(*scheduler.effects.borrow(), ["register", "start"]);
            let before = root.read_required(Path::new(STATE), 65536).unwrap();
            run_with(
                &root,
                &approval,
                |a, b| scheduler.call(a, b),
                |a, b| scheduler.ready(a, b),
            )
            .unwrap();
            assert_eq!(root.read_required(Path::new(STATE), 65536).unwrap(), before);
            assert_eq!(*scheduler.effects.borrow(), ["register", "start"]);
        }
    }

    #[test]
    fn lost_native_responses_are_reconciled_without_duplicate_effects() {
        for operation in ["register", "start"] {
            let (_temp, root, approval) = fixture();
            let scheduler = Scheduler::default();
            scheduler.fail.set(Some(operation));
            assert!(run_with(
                &root,
                &approval,
                |a, b| scheduler.call(a, b),
                |a, b| scheduler.ready(a, b)
            )
            .is_err());
            run_with(
                &root,
                &approval,
                |a, b| scheduler.call(a, b),
                |a, b| scheduler.ready(a, b),
            )
            .unwrap();
            assert_eq!(*scheduler.effects.borrow(), ["register", "start"]);
        }
    }

    #[test]
    fn a_later_disable_cancels_a_prepared_activation_without_configuration() {
        let (_temp, root, approval) = fixture();
        let scheduler = Scheduler::default();
        STOP_AT.with(|point| point.set(Some(Phase::Prepared)));
        assert!(run_with(
            &root,
            &approval,
            |a, b| scheduler.call(a, b),
            |a, b| scheduler.ready(a, b)
        )
        .is_err());
        super::super::super::transition::disable(
            &root,
            |_, _| panic!("no configured task"),
            Duration::ZERO,
        )
        .unwrap();
        let marker = root
            .read_required(Path::new(super::super::super::transition::JOURNAL), 65536)
            .unwrap();
        assert!(run_with(
            &root,
            &approval,
            |_, _| panic!("cancelled activation scheduled"),
            |_, _| panic!("cancelled activation observed")
        )
        .is_err());
        assert!(root
            .read_optional(Path::new(CONFIG), 65536)
            .unwrap()
            .is_none());
        assert_eq!(
            root.read_required(Path::new(super::super::super::transition::JOURNAL), 65536)
                .unwrap(),
            marker
        );
    }

    #[test]
    fn explicit_disable_retires_old_consent_and_only_a_fresh_preview_can_activate() {
        let (_temp, root, approval) = fixture();
        let scheduler = Scheduler::default();
        STOP_AT.with(|point| point.set(Some(Phase::Prepared)));
        assert!(run_with(&root, &approval, |a,b| scheduler.call(a,b), |a,b| scheduler.ready(a,b)).is_err());
        let original = Store::open(&root).unwrap().1.unwrap();
        assert!(run_with(&root, &hive_core::sha256_digest(b"unapproved preview"),
            |_,_| panic!("unapproved schedule"), |_,_| panic!("unapproved observe")).is_err());
        let result = super::super::super::transition::disable(&root,
            |_,_| panic!("no configured task"), Duration::ZERO).unwrap();
        assert_eq!(result["activation_retirement"], "cancelled");
        let retired = Store::open(&root).unwrap().1.unwrap();
        assert_eq!(retired.operation_id, original.operation_id);
        assert_eq!(retired.phase, Phase::Cancelled);
        assert!(retired.completed_at.is_some());
        assert!(run_with(&root, &approval, |_,_| panic!("old consent scheduled"),
            |_,_| panic!("old consent observed")).is_err());
        let fresh = preview(root.requested_path()).unwrap()["preview_digest"].as_str().unwrap().to_owned();
        assert_ne!(approval, fresh);
        let result = run_with(&root, &fresh, |a,b| scheduler.call(a,b), |a,b| scheduler.ready(a,b)).unwrap();
        assert_eq!(result["activation"], "verified");
        let latest = Store::open(&root).unwrap().1.unwrap();
        assert_ne!(latest.operation_id, original.operation_id);
        assert_eq!(latest.phase, Phase::Complete);
        assert_eq!(*scheduler.effects.borrow(), ["register", "start"]);
    }

    #[test]
    fn readiness_retry_preserves_observation_history_and_honors_disable() {
        let (_temp, root, approval) = fixture();
        let scheduler = Scheduler::default();
        assert!(run_with(
            &root,
            &approval,
            |a, b| scheduler.call(a, b),
            |root, _| {
                super::super::super::with_state(root, |state| {
                    state.observation_count = 4;
                    state.observation_watch_started_at = Some(123);
                    Ok(())
                })?;
                Err(error("fixture health timeout"))
            }
        )
        .is_err());
        let state = root.read_required(Path::new(STATE), 65536).unwrap();
        run_with(
            &root,
            &approval,
            |a, b| scheduler.call(a, b),
            |a, b| scheduler.ready(a, b),
        )
        .unwrap();
        assert_eq!(root.read_required(Path::new(STATE), 65536).unwrap(), state);
        super::super::super::transition::disable(
            &root,
            |a, b| scheduler.call(a, b),
            Duration::ZERO,
        )
        .unwrap();
        assert!(run_with(
            &root,
            &approval,
            |_, _| panic!("late disable ignored"),
            |_, _| panic!("late disable ignored")
        )
        .is_err());
        assert!(
            !load::<Configuration>(&root, CONFIG)
                .unwrap()
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn early_external_configuration_edits_preserve_history_and_do_not_schedule() {
        for phase in [
            Phase::BinaryStaged,
            Phase::Pausing,
            Phase::Stopped,
            Phase::StatePrepared,
        ] {
            let (_temp, root, approval) = fixture();
            let scheduler = Scheduler::default();
            STOP_AT.with(|point| point.set(Some(phase)));
            assert!(run_with(
                &root,
                &approval,
                |a, b| scheduler.call(a, b),
                |a, b| scheduler.ready(a, b)
            )
            .is_err());
            let mut changed = configuration(root.requested_path()).unwrap();
            changed.enabled = false;
            changed.task_name.push_str("-external-edit");
            super::super::super::write(&root, CONFIG, &changed).unwrap();
            let changed = root.read_required(Path::new(CONFIG), 65536).unwrap();
            let state = root.read_optional(Path::new(STATE), 65536).unwrap();
            assert!(run_with(
                &root,
                &approval,
                |_, _| panic!("external edit scheduled"),
                |_, _| panic!("external edit observed")
            )
            .is_err());
            assert_eq!(
                root.read_required(Path::new(CONFIG), 65536).unwrap(),
                changed
            );
            assert_eq!(root.read_optional(Path::new(STATE), 65536).unwrap(), state);
            assert!(scheduler.effects.borrow().is_empty());
        }
    }
}
