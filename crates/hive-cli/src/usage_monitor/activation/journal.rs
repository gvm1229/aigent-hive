//! Recoverable, scope-bound writes for one confirmed monitor activation.
use super::super::{error, serialize, AdapterError, Configuration, PinnedTarget, CONFIG, STATE};
use super::{digest, root_identity, Binding};
use crate::user_install::{
    host_policy_mode, inspect_setup_write_claim, read_user_setup_file, recover_user_setup_file,
    replace_user_setup_file, setup_file_mode,
};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(super) const PATH: &str = ".hive/config/setup-progress/monitor-activation.json";
const WRITE: &str = ".hive/config/setup-progress/monitor-activation-write.json";
const LIMIT: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Phase {
    Prepared,
    BinaryStaged,
    Pausing,
    Stopped,
    StatePrepared,
    Configured,
    Registering,
    Registered,
    Starting,
    Checking,
    Complete,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    pub schema_version: u8,
    pub operation_id: String,
    pub approval: String,
    pub binding: Binding,
    pub desired: Configuration,
    pub previous_config: Option<Vec<u8>>,
    pub phase: Phase,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub state_before_digest: Option<String>,
    pub prepared_state: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WriteIntent {
    schema_version: u8,
    root_identity: String,
    disable_digest: String,
    path: String,
    before: String,
    after: String,
    before_mode: Option<u32>,
}

pub(super) struct Store<'a> {
    root: &'a PinnedTarget,
    expected: Option<Vec<u8>>,
}

impl<'a> Store<'a> {
    pub fn open(root: &'a PinnedTarget) -> Result<(Self, Option<Record>), AdapterError> {
        let mut store = Self {
            root,
            expected: None,
        };
        store.recover_write()?;
        store.expected = store.read(PATH, LIMIT)?;
        let record = store
            .expected
            .as_deref()
            .map(|bytes| store.parse_record(bytes))
            .transpose()?;
        Ok((store, record))
    }

    fn read(&self, path: &str, limit: u64) -> Result<Option<Vec<u8>>, AdapterError> {
        read_user_setup_file(self.root.target_dir(), Path::new(path), limit)
            .map_err(AdapterError::Conflict)
    }

    fn parse_record(&self, bytes: &[u8]) -> Result<Record, AdapterError> {
        let record: Record = serde_json::from_slice(bytes)
            .map_err(|_| error("invalid monitor activation record"))?;
        if record.schema_version != 1
            || record.operation_id.len() != 64
            || !record
                .operation_id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !crate::is_sha256_digest(&record.approval)
            || record.binding.root_identity_digest != root_identity(self.root)?
            || digest(record.previous_config.as_deref()) != record.binding.previous.config_digest
            || record.binding.previous.configured != record.previous_config.is_some()
            || record.binding.previous.enabled
            || !record.desired.enabled
            || record.desired.schema_version != 1
            || !valid_digest(&record.binding.disable_digest)
            || !crate::is_sha256_digest(&record.desired.executable_digest)
            || record
                .state_before_digest
                .as_ref()
                .is_some_and(|hash| !valid_digest(hash))
            || record
                .prepared_state
                .as_ref()
                .is_some_and(|bytes| bytes.len() > 65536)
        {
            return Err(error("monitor activation record scope or payload changed"));
        }
        super::super::upgrade::authenticate_ownership(
            self.root,
            &record.desired,
            &super::super::monitor_task_name(&super::super::monitor_root_path(
                self.root.requested_path(),
            )?),
        )?;
        if let Some(bytes) = &record.previous_config {
            let previous: Configuration = serde_json::from_slice(bytes)
                .map_err(|_| error("invalid previous monitor configuration"))?;
            super::super::upgrade::authenticate_ownership(
                self.root,
                &previous,
                &record.desired.task_name,
            )?;
            if previous.enabled {
                return Err(error("activation cannot replace an enabled selection"));
            }
        }
        if let Some(bytes) = &record.prepared_state {
            let mut state: super::super::MonitorState = serde_json::from_slice(bytes)
                .map_err(|_| error("invalid prepared monitor state"))?;
            state.migrate()?;
        }
        Ok(record)
    }

    pub fn save(&mut self, record: &Record) -> Result<(), AdapterError> {
        let bytes = serialize(record)?;
        self.parse_record(&bytes)?;
        if bytes.len()
            > usize::try_from(LIMIT).expect("activation record limit fits supported targets")
        {
            return Err(error("monitor activation record exceeds its limit"));
        }
        self.write_bound(
            PATH,
            self.expected.as_deref(),
            &bytes,
            &record.binding.disable_digest,
        )?;
        self.expected = Some(bytes);
        Ok(())
    }

    pub fn publish(
        &self,
        path: &str,
        before: Option<&[u8]>,
        after: &[u8],
        record: &Record,
    ) -> Result<(), AdapterError> {
        if !matches!(path, CONFIG | STATE) || after.len() > 65536 {
            return Err(error("monitor activation write escaped its scope"));
        }
        self.write_bound(path, before, after, &record.binding.disable_digest)
    }

    fn parse_intent(&self, bytes: &[u8]) -> Result<WriteIntent, AdapterError> {
        let intent: WriteIntent = serde_json::from_slice(bytes)
            .map_err(|_| error("invalid monitor activation write intent"))?;
        if intent.schema_version != 1
            || intent.root_identity != root_identity(self.root)?
            || !matches!(intent.path.as_str(), PATH | CONFIG | STATE)
            || !valid_digest(&intent.before)
            || !valid_digest(&intent.after)
            || !valid_digest(&intent.disable_digest)
            || intent.before_mode.is_some_and(|mode| mode > 0o7777)
        {
            return Err(error("monitor activation write intent changed"));
        }
        Ok(intent)
    }

    fn recover_intent(&self) -> Result<(), AdapterError> {
        let path = Path::new(WRITE);
        let Some(claim) = inspect_setup_write_claim(self.root.target_dir(), path, LIMIT)
            .map_err(AdapterError::Conflict)?
        else {
            return Ok(());
        };
        let current = self.read(WRITE, LIMIT)?;
        if let Some(before) = &claim.before {
            self.parse_intent(before)?;
            if claim.staged.is_some() || current.as_ref().is_some_and(|value| value != before) {
                return Err(error("monitor write intent removal changed"));
            }
            recover_user_setup_file(
                self.root.target_dir(),
                path,
                &sha256_digest(before),
                "absent",
                claim.before_mode,
            )
            .map_err(AdapterError::Conflict)?;
        } else {
            let candidate = current.as_ref().or(claim.staged.as_ref());
            if let Some(bytes) = candidate {
                self.parse_intent(bytes)?;
            }
            if current
                .as_ref()
                .zip(claim.staged.as_ref())
                .is_some_and(|(a, b)| a != b)
            {
                return Err(error("monitor write intent publication changed"));
            }
            recover_user_setup_file(
                self.root.target_dir(),
                path,
                "absent",
                &digest(candidate.map(Vec::as_slice)),
                None,
            )
            .map_err(AdapterError::Conflict)?;
        }
        Ok(())
    }

    fn recover_write(&self) -> Result<(), AdapterError> {
        self.recover_intent()?;
        let Some(bytes) = self.read(WRITE, LIMIT)? else {
            return Ok(());
        };
        let intent = self.parse_intent(&bytes)?;
        if intent.path != PATH
            && (self.disable_digest()? != intent.disable_digest
                || super::super::usage_policy_disabled(self.root)?)
        {
            return Err(error("a newer disable selection prevents activation write recovery; retained data preserved"));
        }
        let path = Path::new(&intent.path);
        let current = self.read(&intent.path, LIMIT)?;
        if let Some(value) = &current {
            self.validate_payload(&intent.path, value)?;
            let hash = sha256_digest(value);
            let mode =
                host_policy_mode(self.root.target_dir(), path).map_err(AdapterError::Conflict)?;
            if !((hash == intent.before && mode == intent.before_mode)
                || (hash == intent.after && mode == setup_file_mode(path)))
            {
                return Err(error("monitor activation data changed during recovery"));
            }
        }
        if let Some(claim) = inspect_setup_write_claim(self.root.target_dir(), path, LIMIT)
            .map_err(AdapterError::Conflict)?
        {
            if let Some(before) = &claim.before {
                if sha256_digest(before) != intent.before || claim.before_mode != intent.before_mode
                {
                    return Err(error("retained monitor data changed"));
                }
                self.validate_payload(&intent.path, before)?;
            }
            if let Some(after) = &claim.staged {
                if sha256_digest(after) != intent.after {
                    return Err(error("staged monitor data changed"));
                }
                self.validate_payload(&intent.path, after)?;
            }
            recover_user_setup_file(
                self.root.target_dir(),
                path,
                &intent.before,
                &intent.after,
                intent.before_mode,
            )
            .map_err(AdapterError::Conflict)?;
        } else if ![intent.before.as_str(), intent.after.as_str()]
            .contains(&digest(current.as_deref()).as_str())
        {
            return Err(error("monitor activation write no longer matches"));
        }
        replace_user_setup_file(self.root.target_dir(), Path::new(WRITE), Some(&bytes), None)
            .map_err(AdapterError::Conflict)
    }

    fn validate_payload(&self, path: &str, bytes: &[u8]) -> Result<(), AdapterError> {
        if path == PATH {
            self.parse_record(bytes).map(|_| ())
        } else if path == CONFIG {
            let config: Configuration = serde_json::from_slice(bytes)
                .map_err(|_| error("invalid retained monitor configuration"))?;
            super::super::upgrade::authenticate_ownership(
                self.root,
                &config,
                &super::super::monitor_task_name(&super::super::monitor_root_path(
                    self.root.requested_path(),
                )?),
            )
        } else {
            let mut state: super::super::MonitorState = serde_json::from_slice(bytes)
                .map_err(|_| error("invalid retained monitor state"))?;
            state.migrate()
        }
    }

    pub fn disable_digest(&self) -> Result<String, AdapterError> {
        Ok(digest(
            self.read(super::super::transition::JOURNAL, 65536)?
                .as_deref(),
        ))
    }

    fn write_bound(
        &self,
        path: &str,
        before: Option<&[u8]>,
        after: &[u8],
        disable: &str,
    ) -> Result<(), AdapterError> {
        self.root.verify_current()?;
        self.validate_payload(path, after)?;
        if path != PATH && self.disable_digest()? != disable {
            return Err(error("monitor selection changed before activation write"));
        }
        let relative = Path::new(path);
        let mode = before
            .map(|_| host_policy_mode(self.root.target_dir(), relative))
            .transpose()
            .map_err(AdapterError::Conflict)?
            .flatten();
        if before.is_some() && mode != setup_file_mode(relative) {
            return Err(error(
                "monitor file permissions differ; existing permissions preserved",
            ));
        }
        let intent = WriteIntent {
            schema_version: 1,
            root_identity: root_identity(self.root)?,
            disable_digest: disable.into(),
            path: path.into(),
            before: digest(before),
            after: sha256_digest(after),
            before_mode: mode,
        };
        let encoded = serialize(&intent)?;
        replace_user_setup_file(
            self.root.target_dir(),
            Path::new(WRITE),
            None,
            Some(&encoded),
        )
        .map_err(AdapterError::Conflict)?;
        replace_user_setup_file(self.root.target_dir(), relative, before, Some(after))
            .map_err(AdapterError::Conflict)?;
        replace_user_setup_file(
            self.root.target_dir(),
            Path::new(WRITE),
            Some(&encoded),
            None,
        )
        .map_err(AdapterError::Conflict)
    }
}

fn valid_digest(value: &str) -> bool {
    value == "absent" || crate::is_sha256_digest(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    fn record(root: &PinnedTarget, approval: String) -> Record {
        Record {
            schema_version: 1,
            operation_id: "a".repeat(64),
            approval,
            binding: Binding::capture(root).unwrap(),
            desired: super::super::super::configuration(root.requested_path()).unwrap(),
            previous_config: None,
            phase: Phase::Prepared,
            started_at: 1,
            completed_at: None,
            state_before_digest: None,
            prepared_state: None,
        }
    }
    fn intent(root: &PinnedTarget, path: &str, before: Option<&[u8]>, after: &[u8]) -> WriteIntent {
        WriteIntent {
            schema_version: 1,
            root_identity: root_identity(root).unwrap(),
            disable_digest: "absent".into(),
            path: path.into(),
            before: digest(before),
            after: sha256_digest(after),
            before_mode: before
                .and_then(|_| host_policy_mode(root.target_dir(), Path::new(path)).unwrap()),
        }
    }
    fn save_intent(root: &PinnedTarget, intent: &WriteIntent) {
        replace_user_setup_file(
            root.target_dir(),
            Path::new(WRITE),
            None,
            Some(&serialize(intent).unwrap()),
        )
        .unwrap();
    }

    #[test]
    fn activation_record_recovers_a_claimed_or_published_phase_without_other_writes() {
        for published in [false, true] {
            let (temp, root, approval) = super::super::runner::tests::fixture();
            let (mut store, _) = Store::open(&root).unwrap();
            let mut record = record(&root, approval);
            store.save(&record).unwrap();
            let before = root
                .read_required(Path::new(PATH), usize::try_from(LIMIT).unwrap())
                .unwrap();
            record.phase = Phase::BinaryStaged;
            let after = serialize(&record).unwrap();
            save_intent(&root, &intent(&root, PATH, Some(&before), &after));
            crate::user_install::interrupt_setup_replace(
                root.target_dir(),
                Path::new(PATH),
                &before,
                &after,
                published,
            );
            let (_, restored) = Store::open(&root).unwrap();
            assert_eq!(
                restored.unwrap().phase,
                if published {
                    Phase::BinaryStaged
                } else {
                    Phase::Prepared
                }
            );
            assert!(!temp.path().join(WRITE).exists());
            assert!(!temp.path().join(CONFIG).exists());
            assert!(!temp.path().join(STATE).exists());
            assert!(
                inspect_setup_write_claim(root.target_dir(), Path::new(PATH), LIMIT)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn a_later_disable_never_rolls_a_retained_active_configuration_forward() {
        let (_temp, root, approval) = super::super::runner::tests::fixture();
        let (mut store, _) = Store::open(&root).unwrap();
        let record = record(&root, approval);
        store.save(&record).unwrap();
        let mut inactive = record.desired.clone();
        inactive.enabled = false;
        let before = serialize(&inactive).unwrap();
        let after = serialize(&record.desired).unwrap();
        replace_user_setup_file(root.target_dir(), Path::new(CONFIG), None, Some(&before)).unwrap();
        save_intent(&root, &intent(&root, CONFIG, Some(&before), &after));
        crate::user_install::interrupt_setup_replace(
            root.target_dir(),
            Path::new(CONFIG),
            &before,
            &after,
            true,
        );
        super::super::super::transition::disable(
            &root,
            |_, _| Ok(json!({"removed":true})),
            Duration::ZERO,
        )
        .unwrap();
        let off = root.read_required(Path::new(CONFIG), 65536).unwrap();
        let marker = root
            .read_required(Path::new(super::super::super::transition::JOURNAL), 65536)
            .unwrap();
        assert!(Store::open(&root).is_err());
        assert_eq!(root.read_required(Path::new(CONFIG), 65536).unwrap(), off);
        assert_eq!(
            root.read_required(Path::new(super::super::super::transition::JOURNAL), 65536)
                .unwrap(),
            marker
        );
        assert!(
            inspect_setup_write_claim(root.target_dir(), Path::new(CONFIG), LIMIT)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn changed_or_out_of_scope_activation_write_intents_are_preserved() {
        let (temp, root, approval) = super::super::runner::tests::fixture();
        let record = record(&root, approval);
        let after = serialize(&record).unwrap();
        let mut outside = intent(&root, PATH, None, &after);
        outside.path = ".hive/config/user-setup.yml".into();
        save_intent(&root, &outside);
        let original = std::fs::read(temp.path().join(".hive/config/user-setup.yml")).unwrap();
        assert!(Store::open(&root).is_err());
        assert_eq!(
            std::fs::read(temp.path().join(".hive/config/user-setup.yml")).unwrap(),
            original
        );
        assert!(temp.path().join(WRITE).exists());
    }
}
