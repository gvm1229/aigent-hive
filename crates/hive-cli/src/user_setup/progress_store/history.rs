//! Durable, bounded lifecycle evidence. Answers and free-form text are represented by hashes.
use super::{ProgressPhase, ProgressStore, SetupError, SetupProgress, TargetBinding};
use crate::user_install::read_user_setup_file;
use cap_fs_ext::DirExt;
use hive_core::preferences::declaration_digest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DIRECTORY: &str = ".hive/history/lifecycle";
const LIMIT: u64 = 16384;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    operation_id: String,
    operation: String,
    product_version: String,
    binding: TargetBinding,
    before_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    before_mode: Option<u32>,
    candidate_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    intent_digest: Option<String>,
    after_digest: Option<String>,
    question_revision: u32,
    catalog_digest: String,
    phase: ProgressPhase,
    started_at_unix_nanos: u64,
    completed_at_unix_nanos: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    projection_digest: Option<String>,
}

fn path(id: &str) -> Result<PathBuf, SetupError> {
    if id.len() != 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SetupError::Verification(
            "invalid lifecycle operation identity".into(),
        ));
    }
    Ok(Path::new(DIRECTORY).join(format!("{id}.json")))
}

pub(super) fn is_record_path(relative: &Path) -> bool {
    relative.file_name().and_then(|name| name.to_str()).and_then(|name| name.strip_suffix(".json"))
        .and_then(|id| path(id).ok()).as_deref() == Some(relative)
}

pub(super) fn validate_scope(store: &ProgressStore, relative: &Path, bytes: &[u8]) -> Result<(), SetupError> {
    let receipt: Receipt = serde_json::from_slice(bytes)
        .map_err(|_| SetupError::Verification("invalid retained lifecycle record".into()))?;
    if receipt.schema_version != 1 || receipt.operation != "setup" || receipt.binding != store.binding
        || path(&receipt.operation_id)?.as_path() != relative {
        return Err(SetupError::Conflict("retained lifecycle record scope changed".into()));
    }
    Ok(())
}

fn intent_digest(progress: &SetupProgress) -> Result<String, SetupError> {
    // Bind every persisted execution input, including migration/monitor metadata.
    // Phase changes during recovery; the confirmed intent must remain unchanged.
    let mut intent =
        serde_json::to_value(progress).map_err(|error| SetupError::Internal(error.to_string()))?;
    intent
        .as_object_mut()
        .expect("setup progress is an object")
        .remove("phase");
    declaration_digest(&intent).map_err(SetupError::Internal)
}

pub(super) fn confirm_completion(
    store: &ProgressStore,
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    store.verify_directory_binding()?;
    let bytes = store.read_state(&path(&progress.operation_id)?, LIMIT)?
        .ok_or_else(|| SetupError::Verification("setup completion record is missing".into()))?;
    let receipt: Receipt = serde_json::from_slice(&bytes)
        .map_err(|_| SetupError::Verification("invalid setup completion record".into()))?;
    if receipt.schema_version != 1
        || receipt.operation != "setup"
        || receipt.product_version != env!("CARGO_PKG_VERSION")
        || receipt.operation_id != progress.operation_id
        || receipt.binding != store.binding
        || receipt.before_digest != progress.base_config_digest
        || receipt.before_mode != progress.base_config_mode
        || receipt.candidate_digest
            != declaration_digest(&progress.answers).map_err(SetupError::Internal)?
        || receipt.intent_digest.as_ref() != Some(&intent_digest(progress)?)
        || receipt.catalog_digest != progress.catalog_digest
        || receipt.question_revision != progress.question_revision
        || receipt.phase != ProgressPhase::Complete
        || receipt.after_digest.as_ref() != Some(&store.base_config_digest)
        || receipt.completed_at_unix_nanos.is_none()
    {
        return Err(SetupError::Conflict(
            "setup completion no longer matches".into(),
        ));
    }
    Ok(())
}

pub(super) fn record(
    store: &ProgressStore,
    progress: &SetupProgress,
    phase: ProgressPhase,
) -> Result<(), SetupError> {
    store.verify_directory_binding()?;
    let relative = path(&progress.operation_id)?;
    let before = store.read_state(&relative, LIMIT)?;
    let mut receipt = if let Some(bytes) = &before {
        serde_json::from_slice::<Receipt>(bytes)
            .map_err(|_| SetupError::Verification("invalid lifecycle record".into()))?
    } else {
        if phase != ProgressPhase::Applying {
            return Err(SetupError::Verification(
                "setup apply intent was not recorded".into(),
            ));
        }
        Receipt {
            schema_version: 1,
            operation_id: progress.operation_id.clone(),
            operation: "setup".into(),
            product_version: env!("CARGO_PKG_VERSION").into(),
            binding: progress.binding.clone(),
            before_digest: progress.base_config_digest.clone(),
            before_mode: progress.base_config_mode,
            candidate_digest: declaration_digest(&progress.answers)
                .map_err(SetupError::Internal)?,
            intent_digest: Some(intent_digest(progress)?),
            after_digest: None,
            question_revision: progress.question_revision,
            catalog_digest: progress.catalog_digest.clone(),
            phase,
            started_at_unix_nanos: recorded_now()?,
            completed_at_unix_nanos: None,
            projection_digest: None,
        }
    };
    if receipt.schema_version != 1
        || receipt.operation != "setup"
        || receipt.product_version != env!("CARGO_PKG_VERSION")
        || receipt.binding != store.binding
        || receipt.operation_id != progress.operation_id
        || receipt.before_digest != progress.base_config_digest
        || receipt.before_mode != progress.base_config_mode
        || receipt.candidate_digest
            != declaration_digest(&progress.answers).map_err(SetupError::Internal)?
        || receipt.intent_digest.as_ref() != Some(&intent_digest(progress)?)
        || receipt.catalog_digest != progress.catalog_digest
        || receipt.question_revision != progress.question_revision
        || (receipt.phase == ProgressPhase::Complete && phase != ProgressPhase::Complete)
    {
        return Err(SetupError::Conflict(
            "lifecycle operation changed; evidence retained".into(),
        ));
    }
    receipt.phase = phase;
    if phase == ProgressPhase::Complete {
        receipt.after_digest = Some(store.base_config_digest.clone());
        if receipt.completed_at_unix_nanos.is_none() {
            receipt.completed_at_unix_nanos = Some(recorded_now()?);
        }
    }
    let bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    if bytes.len() > usize::try_from(LIMIT).expect("history limit fits usize") {
        return Err(SetupError::Input(
            "lifecycle record exceeds its size limit".into(),
        ));
    }
    store.write_state(&relative, before.as_deref(), Some(&bytes))
}

pub(super) fn projection_binding(
    store: &ProgressStore,
    progress: &SetupProgress,
    desired: Option<&str>,
) -> Result<Option<String>, SetupError> {
    store.verify_directory_binding()?;
    let relative = path(&progress.operation_id)?;
    let before = store.read_state(&relative, LIMIT)?
        .ok_or_else(|| SetupError::Verification("setup intent record is missing".into()))?;
    let mut receipt: Receipt = serde_json::from_slice(&before)
        .map_err(|_| SetupError::Verification("invalid setup intent record".into()))?;
    if receipt.schema_version != 1
        || receipt.operation != "setup"
        || receipt.product_version != env!("CARGO_PKG_VERSION")
        || receipt.binding != store.binding
        || receipt.operation_id != progress.operation_id
        || receipt.before_digest != progress.base_config_digest
        || receipt.before_mode != progress.base_config_mode
        || receipt.candidate_digest
            != declaration_digest(&progress.answers).map_err(SetupError::Internal)?
        || receipt.intent_digest.as_ref() != Some(&intent_digest(progress)?)
        || receipt.catalog_digest != progress.catalog_digest
        || receipt.question_revision != progress.question_revision
    {
        return Err(SetupError::Conflict(
            "setup projection intent changed".into(),
        ));
    }
    if let Some(digest) = desired {
        if !super::valid_hash(digest)
            || receipt.phase == ProgressPhase::Complete
            || receipt
                .projection_digest
                .as_deref()
                .is_some_and(|prior| prior != digest)
        {
            return Err(SetupError::Conflict(
                "setup projection journal changed".into(),
            ));
        }
        if receipt.projection_digest.is_none() {
            receipt.projection_digest = Some(digest.into());
            let bytes = serde_json::to_vec_pretty(&receipt)
                .map_err(|error| SetupError::Internal(error.to_string()))?;
            store.write_state(&relative, Some(&before), Some(&bytes))?;
        }
    }
    Ok(receipt.projection_digest)
}

pub(super) fn retain_completed(store: &ProgressStore, current: &str) -> Result<(), SetupError> {
    store.verify_directory_binding()?;
    let mut directory = store
        .root
        .try_clone()
        .map_err(|error| SetupError::Conflict(error.to_string()))?;
    for component in Path::new(DIRECTORY).components() {
        directory = directory
            .open_dir_nofollow(component.as_os_str())
            .map_err(|error| SetupError::Conflict(error.to_string()))?;
    }
    let entries = directory
        .entries()
        .map_err(|error| SetupError::Conflict(error.to_string()))?;
    let mut completed = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| SetupError::Conflict(error.to_string()))?;
        if !entry
            .file_type()
            .map_err(|error| SetupError::Conflict(error.to_string()))?
            .is_file()
        {
            continue;
        }
        let name = entry.file_name();
        let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
            continue;
        };
        let Ok(relative) = path(id) else { continue };
        // Inspect scope before considering recovery; another project's write is not ours.
        let Some(bytes) = read_user_setup_file(&store.root, &relative, LIMIT).map_err(SetupError::Conflict)?
        else {
            continue;
        };
        let Ok(receipt) = serde_json::from_slice::<Receipt>(&bytes) else {
            continue;
        };
        if receipt.schema_version == 1
            && receipt.operation == "setup"
            && receipt.operation_id == id
            && receipt.binding == store.binding
            && receipt.phase == ProgressPhase::Complete
        {
            if let Some(time) = receipt.completed_at_unix_nanos {
                completed.push((id == current, time, relative, bytes));
            }
        }
    }
    completed
        .sort_by(|left, right| (&right.0, &right.1, &right.2).cmp(&(&left.0, &left.1, &left.2)));
    for (_, _, relative, bytes) in completed.into_iter().skip(100) {
        store.write_state(&relative, Some(&bytes), None)?;
    }
    Ok(())
}

fn recorded_now() -> Result<u64, SetupError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_nanos()).ok())
        .ok_or_else(|| SetupError::Internal("lifecycle clock is unavailable".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn applying(store: &ProgressStore, text: &str) -> SetupProgress {
        let snapshot = store.load().unwrap();
        let mut progress = store
            .new_progress(
                "persona".into(),
                json!({"persona":{"custom_description":text}}),
            )
            .unwrap();
        progress.phase = ProgressPhase::Applying;
        store.save(&snapshot, &progress).unwrap();
        store.record_applying(&progress).unwrap();
        progress
    }

    #[test]
    fn completion_keeps_hashes_and_removes_free_form_answers_from_progress() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join(super::super::LEGACY_PROGRESS);
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(
            &legacy,
            "schema_version: 1\nstep: persona\nanswers:\n  interface_language: ko\n",
        )
        .unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let progress = applying(&store, "private preference body");
        let receipt_path = root.path().join(path(&progress.operation_id).unwrap());
        let before = fs::read_to_string(&receipt_path).unwrap();
        assert!(!before.contains("private preference body"));
        assert!(before.contains("applying"));
        store.record_completion(&progress).unwrap();
        let completed = store.load().unwrap().progress.unwrap();
        assert_eq!(completed.answers, json!({}));
        assert_eq!(completed.phase, ProgressPhase::Complete);
        assert!(!legacy.exists());
        assert!(!fs::read_to_string(root.path().join(&store.relative))
            .unwrap()
            .contains("private preference body"));
        let receipt: Receipt = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
        assert_eq!(receipt.phase, ProgressPhase::Complete);
        assert_eq!(receipt.after_digest, Some(store.base_config_digest.clone()));
        assert!(receipt.completed_at_unix_nanos.is_some());
    }

    #[test]
    fn retention_keeps_one_hundred_per_scope_and_never_removes_incomplete_or_foreign_records() {
        let root = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let other = ProgressStore::open(root.path(), Some(project.path())).unwrap();
        let other_progress = applying(&other, "other");
        other.record_completion(&other_progress).unwrap();
        let incomplete = applying(&store, "unfinished");
        store.record_failure(&incomplete).unwrap();
        let foreign = root.path().join(DIRECTORY).join("foreign.json");
        fs::write(&foreign, "foreign bytes").unwrap();
        let mut ids = Vec::new();
        for index in 0..102 {
            let progress = applying(&store, &format!("choice {index}"));
            store.record_completion(&progress).unwrap();
            ids.push(progress.operation_id);
        }
        for (index, id) in ids.iter().enumerate() {
            assert_eq!(root.path().join(path(id).unwrap()).exists(), index >= 2);
        }
        assert!(root
            .path()
            .join(path(&incomplete.operation_id).unwrap())
            .exists());
        assert!(root
            .path()
            .join(path(&other_progress.operation_id).unwrap())
            .exists());
        assert_eq!(fs::read_to_string(foreign).unwrap(), "foreign bytes");
        assert_eq!(
            fs::read_dir(root.path().join(DIRECTORY)).unwrap().count(),
            103
        );
    }

    #[test]
    fn another_operation_cannot_finish_or_replace_the_current_receipt() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let progress = applying(&store, "first");
        let path = root.path().join(path(&progress.operation_id).unwrap());
        let bytes = fs::read(&path).unwrap();
        let mut changed = progress.clone();
        changed.answers = json!({"interface_language":"en"});
        assert!(store.record_applying(&changed).is_err());
        assert!(store.record_completion(&changed).is_err());
        assert_eq!(fs::read(path).unwrap(), bytes);
    }

    #[test]
    fn edited_execution_metadata_cannot_resume_apply_projection_or_completion() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress("usage-enforcement".into(), json!({}))
            .unwrap();
        progress.legacy_migration = Some(super::super::LegacyMigration {
            migration_id: hive_core::sha256_digest(b"migration"),
            monitor_choice: crate::usage_monitor::SavedMonitorChoice {
                configured: true,
                enabled: true,
                config_digest: hive_core::sha256_digest(b"monitor"),
            },
            from_schema: 1,
            questions: vec!["usage-enforcement".into(), "usage-warnings".into()],
        });
        progress.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_applying(&progress).unwrap();
        let receipt_path = root.path().join(path(&progress.operation_id).unwrap());
        let receipt = fs::read(&receipt_path).unwrap();
        for (pointer, value) in [
            ("/legacy_migration/monitor_choice/enabled", json!(false)),
            (
                "/legacy_migration/monitor_choice/config_digest",
                json!(hive_core::sha256_digest(b"other")),
            ),
            ("/legacy_migration/from_schema", json!(2)),
            ("/legacy_migration/questions", json!([])),
            (
                "/legacy_migration/migration_id",
                json!(hive_core::sha256_digest(b"other")),
            ),
            ("/legacy_migration", serde_json::Value::Null),
            ("/step", json!("usage-warnings")),
        ] {
            let mut altered = serde_json::to_value(&progress).unwrap();
            *altered.pointer_mut(pointer).unwrap() = value;
            let changed: SetupProgress = serde_json::from_value(altered).unwrap();
            store.save(&store.load().unwrap(), &changed).unwrap();
            assert!(
                store
                    .require_applying(&changed.operation_id, &changed.answers)
                    .is_err(),
                "{pointer}"
            );
            assert!(store.record_failure(&changed).is_err(), "{pointer}");
            assert!(
                store.projection_binding(&changed, None).is_err(),
                "{pointer}"
            );
            assert!(store.record_completion(&changed).is_err(), "{pointer}");
            assert_eq!(fs::read(&receipt_path).unwrap(), receipt);
        }
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_failure(&progress).unwrap();
        store.record_applying(&progress).unwrap();
        store.record_completion(&progress).unwrap();
        store.record_completion(&progress).unwrap();
    }

    #[test]
    fn missing_intent_or_different_product_cannot_authorize_recovery() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let progress = applying(&store, "same answers");
        let receipt_path = root.path().join(path(&progress.operation_id).unwrap());
        let receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        for field in ["intent_digest", "product_version"] {
            let mut changed = receipt.clone();
            if field == "intent_digest" {
                changed.as_object_mut().unwrap().remove(field);
            } else {
                changed[field] = json!("0.0.0");
            }
            let bytes = serde_json::to_vec(&changed).unwrap();
            fs::write(&receipt_path, &bytes).unwrap();
            assert!(store.record_applying(&progress).is_err());
            assert!(store.projection_binding(&progress, None).is_err());
            assert!(store.record_completion(&progress).is_err());
            assert_eq!(fs::read(&receipt_path).unwrap(), bytes);
        }
    }
}
