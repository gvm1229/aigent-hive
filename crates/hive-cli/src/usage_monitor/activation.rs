//! Exact preview inputs for a consented monitor activation.
use super::{
    error, inspect_saved_choice, monitor_root_path, transition, AdapterError, PinnedTarget,
    SavedMonitorChoice, CONFIG,
};
use cap_fs_ext::MetadataExt;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::path::Path;

mod journal;
mod runner;

pub(super) fn pending_disabled_configuration(root: &PinnedTarget) -> Result<Option<super::Configuration>, AdapterError> {
    let (store, record) = journal::Store::open_retiring(root)?;
    record.as_ref().map(|record| store.disabled_candidate(record)).transpose().map(Option::flatten)
}

/// The caller holds the verified disable's control and stopped-controller leases.
pub(super) fn retire_disabled(root: &PinnedTarget) -> Result<bool, AdapterError> {
    let binding = Binding::capture(root)?;
    let stopped: serde_json::Value = super::load(root, transition::JOURNAL)?
        .ok_or_else(|| error("monitor disable receipt is missing"))?;
    if binding.previous.enabled || stopped["phase"] != "complete"
        || stopped["after_digest"] != binding.previous.config_digest
        || stopped["completed_at"].as_u64().is_none() {
        return Err(error("monitor disable has not completed"));
    }
    let (mut store, record) = journal::Store::open_retiring(root)?;
    let Some(mut record) = record else { return Ok(false) };
    if matches!(record.phase, journal::Phase::Complete | journal::Phase::Cancelled) { return Ok(false); }
    if record.binding.root_identity_digest != binding.root_identity_digest
        || record.binding.disable_digest == binding.disable_digest {
        return Err(error("monitor activation has no newer disable selection"));
    }
    store.retire_config_write(&record, &binding)?;
    record.phase = journal::Phase::Cancelled;
    record.completed_at = Some(super::now());
    store.save(&record)?;
    Ok(true)
}
pub(super) use runner::enable;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Binding {
    pub root_identity_digest: String,
    pub previous: SavedMonitorChoice,
    pub disable_digest: String,
}

impl Binding {
    pub(super) fn capture(root: &PinnedTarget) -> Result<Self, AdapterError> {
        let config_before = root.snapshot_bounded(Path::new(CONFIG), 65536)?;
        let disable = root.snapshot_bounded(Path::new(transition::JOURNAL), 65536)?;
        if let Some(bytes) = disable.bytes() {
            let record: serde_json::Value = serde_json::from_slice(bytes)
                .map_err(|_| error("invalid monitor disable selection record"))?;
            if record["schema_version"] != 1 || record["operation"] != "disable" {
                return Err(error("unsupported monitor disable selection record"));
            }
        }
        let previous = inspect_saved_choice(root.requested_path())?;
        let config_after = root.snapshot_bounded(Path::new(CONFIG), 65536)?;
        let disable_after = root.snapshot_bounded(Path::new(transition::JOURNAL), 65536)?;
        root.verify_current()?;
        if config_before.bytes() != config_after.bytes()
            || disable.bytes() != disable_after.bytes()
            || previous.config_digest != digest(config_after.bytes())
        {
            return Err(error("monitor selection changed during preview"));
        }
        Ok(Self {
            root_identity_digest: root_identity(root)?,
            previous,
            disable_digest: digest(disable.bytes()),
        })
    }
}

fn root_identity(root: &PinnedTarget) -> Result<String, AdapterError> {
    let identity = root
        .target_dir()
        .dir_metadata()
        .map_err(|_| error("monitor root identity unavailable"))?;
    let normal = monitor_root_path(root.requested_path())?;
    let encoded = serde_json::to_vec(&(
        "monitor-root-v1",
        identity.dev(),
        identity.ino(),
        normal.as_os_str().as_encoded_bytes(),
    ))
    .map_err(|_| error("cannot encode monitor root identity"))?;
    Ok(sha256_digest(&encoded))
}

fn digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".into(), sha256_digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn activation_preview_is_stable_without_writes_and_changes_for_each_disable_selection() {
        let temp = tempfile::tempdir().unwrap();
        let root = PinnedTarget::open_usage(temp.path()).unwrap();
        let first = super::super::preview(temp.path()).unwrap();
        assert_eq!(first, super::super::preview(temp.path()).unwrap());
        assert!(!temp.path().join(".hive").exists());
        transition::disable(
            &root,
            |_, _| panic!("no scheduler without configuration"),
            Duration::ZERO,
        )
        .unwrap();
        let second = super::super::preview(temp.path()).unwrap();
        assert_ne!(first["preview_digest"], second["preview_digest"]);
        assert_eq!(
            first["activation_binding"]["previous"],
            second["activation_binding"]["previous"]
        );
        transition::disable(
            &root,
            |_, _| panic!("no scheduler without configuration"),
            Duration::ZERO,
        )
        .unwrap();
        let third = super::super::preview(temp.path()).unwrap();
        assert_ne!(second["preview_digest"], third["preview_digest"]);
        assert_eq!(third["supports_current_os"], cfg!(windows));
    }

    #[test]
    fn activation_preview_binds_configuration_and_replacement_at_the_same_path() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("user");
        std::fs::create_dir(&path).unwrap();
        let initial = super::super::preview(&path).unwrap();
        {
            let root = PinnedTarget::open_usage(&path).unwrap();
            let mut config = super::super::configuration(&path).unwrap();
            config.enabled = false;
            super::super::write(&root, CONFIG, &config).unwrap();
            let configured = super::super::preview(&path).unwrap();
            assert_eq!(
                configured["activation_binding"]["previous"]["configured"],
                true
            );
            assert_ne!(initial["preview_digest"], configured["preview_digest"]);
            config.version = "0.13.1".into();
            super::super::write(&root, CONFIG, &config).unwrap();
            assert_ne!(
                configured["preview_digest"],
                super::super::preview(&path).unwrap()["preview_digest"]
            );
        }
        std::fs::rename(&path, temp.path().join("old-user")).unwrap();
        std::fs::create_dir(&path).unwrap();
        let replacement = super::super::preview(&path).unwrap();
        assert_ne!(
            initial["activation_binding"]["root_identity_digest"],
            replacement["activation_binding"]["root_identity_digest"]
        );
        assert_eq!(
            replacement["activation_binding"]["previous"]["config_digest"],
            "absent"
        );
        assert!(!path.join(".hive").exists());
        let root = PinnedTarget::open_usage(&path).unwrap();
        super::super::write(
            &root,
            transition::JOURNAL,
            &json!({"schema_version":99,"operation":"disable"}),
        )
        .unwrap();
        let before = std::fs::read(path.join(transition::JOURNAL)).unwrap();
        assert!(super::super::preview(&path).is_err());
        assert_eq!(
            std::fs::read(path.join(transition::JOURNAL)).unwrap(),
            before
        );
    }
}
