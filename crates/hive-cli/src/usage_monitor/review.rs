//! Read-only withdrawal scope and the consent carried by the existing disable journal.
use super::{activation, error, load, transition, AdapterError, PinnedTarget, CONFIG, STATE};
use hive_core::{preferences::PreferenceInput, sha256_digest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{path::Path, time::Duration};

const ACTIVATION: &str = ".hive/config/setup-progress/monitor-activation.json";
const WRITE: &str = ".hive/config/setup-progress/monitor-activation-write.json";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Withdrawal {
    pub operation_id: String,
    pub intent_digest: String,
    pub config_digest: String,
    pub preview_digest: String,
    pub question_id: String,
    pub question_revision: u32,
    pub channel: PreferenceInput,
    pub answered_at: u64,
    pub scope: Value,
}

pub(crate) fn scope(user_root: &Path) -> Result<Value, AdapterError> {
    capture(&PinnedTarget::open_usage(user_root)?)
}

pub(super) fn capture(root: &PinnedTarget) -> Result<Value, AdapterError> {
    let mut files = serde_json::Map::new();
    for path in [CONFIG, ACTIVATION, WRITE, transition::JOURNAL] {
        let bytes = root.read_optional(Path::new(path), 1024 * 1024)?;
        files.insert(
            path.into(),
            json!(bytes
                .as_deref()
                .map_or_else(|| "absent".into(), sha256_digest)),
        );
    }
    let activation: Option<Value> = load(root, ACTIVATION)?;
    let mut claims = serde_json::Map::new();
    for path in [CONFIG, STATE, ACTIVATION, WRITE] {
        let claim = crate::user_install::inspect_setup_write_claim(
            root.target_dir(),
            Path::new(path),
            1024 * 1024,
        )
        .map_err(AdapterError::Conflict)?;
        claims.insert(
            path.into(),
            claim.map_or(Value::Null, |claim| {
                json!({
            "before":claim.before.as_deref().map(sha256_digest), "before_mode":claim.before_mode,
            "staged":claim.staged.as_deref().map(sha256_digest)})
            }),
        );
    }
    root.verify_current()?;
    Ok(
        json!({"root":activation::root_identity(root)?, "files":files, "claims":claims,
        "activation_id":activation.as_ref().and_then(|record| record["operation_id"].as_str())}),
    )
}

pub(crate) fn pending(user_root: &Path) -> Result<Option<Withdrawal>, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    let journal: Option<Value> = load(&root, transition::JOURNAL)?;
    journal
        .as_ref()
        .and_then(|j| j.get("setup_review"))
        .map(|value| {
            let consent: Withdrawal = serde_json::from_value(value.clone())
                .map_err(|_| error("invalid monitor withdrawal receipt"))?;
            validate(&root, &consent)?;
            Ok(consent)
        })
        .transpose()
}

fn validate(root: &PinnedTarget, consent: &Withdrawal) -> Result<(), AdapterError> {
    if consent.operation_id.len() != 64
        || !consent.operation_id.bytes().all(|b| b.is_ascii_hexdigit())
        || !crate::is_sha256_digest(&consent.intent_digest)
        || (consent.config_digest != "absent" && !crate::is_sha256_digest(&consent.config_digest))
        || !crate::is_sha256_digest(&consent.preview_digest)
        || consent.question_id != "user-monitor-withdrawal"
        || consent.question_revision != 1
        || consent.channel == PreferenceInput::Migration
        || consent.scope["root"] != activation::root_identity(root)?
    {
        return Err(error("monitor withdrawal scope changed; preserved"));
    }
    Ok(())
}

pub(super) fn check(root: &PinnedTarget, consent: &Withdrawal) -> Result<(), AdapterError> {
    validate(root, consent)?;
    let current = capture(root)?;
    let journal: Option<Value> = load(root, transition::JOURNAL)?;
    let saved = serde_json::to_value(consent).map_err(|_| error("cannot encode withdrawal"))?;
    if let Some(journal) = journal.filter(|j| j.get("setup_review") == Some(&saved)) {
        let config = &current["files"][CONFIG];
        if config != &journal["before_digest"] && config != &journal["after_digest"] {
            return Err(error(
                "monitor configuration changed after withdrawal; preserved",
            ));
        }
        if current["files"][ACTIVATION] != consent.scope["files"][ACTIVATION]
            || current["files"][WRITE] != consent.scope["files"][WRITE]
            || current["claims"] != consent.scope["claims"]
        {
            let activation: Option<Value> = load(root, ACTIVATION)?;
            if !activation.is_some_and(|record| {
                record["phase"] == "cancelled"
                    && record["operation_id"] == consent.scope["activation_id"]
            }) || current["files"][WRITE] != "absent"
                || current["claims"]
                    .as_object()
                    .is_none_or(|claims| claims.values().any(|c| !c.is_null()))
            {
                return Err(error(
                    "monitor activation changed after withdrawal; preserved",
                ));
            }
        }
    } else if current != consent.scope {
        return Err(error("monitor withdrawal preview changed; preserved"));
    }
    Ok(())
}

pub(crate) fn stop(
    user_root: &Path,
    consent: &Withdrawal,
) -> Result<transition::StoppedReader, AdapterError> {
    stop_using(
        user_root,
        consent,
        super::platform::call,
        Duration::from_secs(15),
    )
}

fn stop_using(
    user_root: &Path,
    consent: &Withdrawal,
    call: impl Fn(&Value, Duration) -> Result<Value, AdapterError>,
    timeout: Duration,
) -> Result<transition::StoppedReader, AdapterError> {
    let root = PinnedTarget::open_usage(user_root)?;
    let stopped = transition::quiesce_review(&root, call, timeout, consent)?;
    activation::retire_disabled(&root)?;
    Ok(stopped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consent(root: &Path) -> Withdrawal {
        Withdrawal {
            operation_id: "a".repeat(64),
            intent_digest: sha256_digest(b"intent"),
            config_digest: sha256_digest(b"settings"),
            preview_digest: sha256_digest(b"preview"),
            question_id: "user-monitor-withdrawal".into(),
            question_revision: 1,
            channel: PreferenceInput::AnswersFile,
            answered_at: 42,
            scope: scope(root).unwrap(),
        }
    }

    #[test]
    fn withdrawal_scope_is_read_only_and_does_not_bind_live_observations() {
        let dir = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(dir.path()).unwrap();
        let first = scope(dir.path()).unwrap();
        assert!(!dir.path().join(".hive").exists());
        super::super::write(&root, STATE, &json!({"observation":"changed"})).unwrap();
        assert_eq!(scope(dir.path()).unwrap(), first);
        crate::user_install::replace_user_setup_file(
            root.target_dir(),
            Path::new(WRITE),
            None,
            Some(br#"{"write":"changed"}"#),
        )
        .unwrap();
        assert_ne!(scope(dir.path()).unwrap(), first);
    }

    #[test]
    fn withdrawal_failure_keeps_consent_and_only_same_scope_can_resume() {
        let dir = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(dir.path()).unwrap();
        let config = super::super::configuration(dir.path()).unwrap();
        super::super::write(&root, CONFIG, &config).unwrap();
        let approved = consent(dir.path());
        assert!(stop_using(
            dir.path(),
            &approved,
            |_, _| Err(error("scheduler refused")),
            Duration::ZERO
        )
        .is_err());
        assert_eq!(pending(dir.path()).unwrap(), Some(approved.clone()));
        let failed: Value = load(&root, transition::JOURNAL).unwrap().unwrap();
        assert_eq!(failed["phase"], "incomplete");
        let mut changed = approved.clone();
        changed.preview_digest = sha256_digest(b"other preview");
        assert!(stop_using(
            dir.path(),
            &changed,
            |_, _| panic!("changed consent applied"),
            Duration::ZERO
        )
        .is_err());
        let stopped = stop_using(
            dir.path(),
            &approved,
            |request, _| {
                assert_eq!(request["operation"], "unregister");
                Ok(json!({"removed":true}))
            },
            Duration::ZERO,
        )
        .unwrap();
        assert!(super::super::lock(&root, "control.lock").is_err());
        stopped.verify_root(dir.path()).unwrap();
        let other = tempfile::tempdir().unwrap();
        assert!(stopped.verify_root(other.path()).is_err());
        drop(stopped);
        transition::verify_stopped(&root).unwrap();
        let completed: Value = load(&root, transition::JOURNAL).unwrap().unwrap();
        assert_eq!(completed["setup_review"]["channel"], "answers-file");
    }

    #[test]
    fn a_changed_activation_or_scope_refuses_before_any_scheduler_call() {
        let dir = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(dir.path()).unwrap();
        let approved = consent(dir.path());
        crate::user_install::replace_user_setup_file(
            root.target_dir(),
            Path::new(ACTIVATION),
            None,
            Some(br#"{"operation_id":"another"}"#),
        )
        .unwrap();
        assert!(stop_using(
            dir.path(),
            &approved,
            |_, _| panic!("stale preview applied"),
            Duration::ZERO
        )
        .is_err());
        assert!(load::<Value>(&root, transition::JOURNAL).unwrap().is_none());
    }
}
