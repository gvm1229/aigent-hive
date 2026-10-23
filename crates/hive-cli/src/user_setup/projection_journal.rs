//! Approved projection writes survive interruption; drift is preserved instead of overwritten.
use super::{
    progress_store::{ProgressStore, SetupProgress},
    AppliedProjection, SetupError,
};
use crate::user_install::{read_user_setup_file, replace_user_setup_file};
use cap_std::fs::Dir;
use hive_core::{preferences::declaration_digest, sha256_digest};
use serde::{Deserialize, Serialize};
use std::path::{Component, PathBuf};

const LIMIT: u64 = 32 * 1024 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema_version: u8,
    package_version: String,
    operation_id: String,
    candidate_digest: String,
    projection: AppliedProjection,
    before_modes: std::collections::BTreeMap<PathBuf, Option<u32>>,
}

fn path(progress: &SetupProgress) -> Result<PathBuf, SetupError> {
    Ok(super::progress_store::recovery_path(progress)?.with_extension("projection.json"))
}

fn validate(journal: &Journal, progress: &SetupProgress) -> Result<(), SetupError> {
    if journal.schema_version != 1
        || journal.package_version != env!("HIVE_PACKAGE_VERSION")
        || journal.operation_id != progress.operation_id
        || journal.candidate_digest
            != declaration_digest(&progress.answers).map_err(SetupError::Internal)?
    {
        return Err(SetupError::Conflict(
            "projection recovery belongs to different answers".into(),
        ));
    }
    let mut paths = std::collections::BTreeSet::new();
    for change in &journal.projection.changes {
        let allowed = matches!(
            change.path.to_str(),
            Some(
                ".agents/directives/00-hive-user.md"
                    | ".hive/config/user-active-skills.yml"
                    | ".hive/install/user-projection.json"
                    | ".agents/skills/user-setup/references/workflow.md"
                    | ".agents/skills/user-setup/references/questions.md"
                    | ".agents/skills/user-setup/references/reconfiguration.md"
                    | ".agents/skills/user-setup/references/recovery.md"
                    | ".agents/skills/user-setup/references/language.md"
                    | ".agents/skills/user-setup/scripts/resolve-hive.ps1"
            )
        ) || (change.path.starts_with(".agents/skills")
            && hive_core::validate_hive_skill_projection_relative(&change.path).is_ok());
        if !allowed
            || change
                .path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            || !paths.insert(&change.path)
            || !journal.before_modes.contains_key(&change.path)
            || change
                .before
                .iter()
                .chain(change.after.iter())
                .any(|bytes| {
                    u64::try_from(bytes.len()).unwrap_or(u64::MAX) > super::MAX_USER_SETUP_BYTES
                })
        {
            return Err(SetupError::Conflict(
                "invalid owned projection recovery path or payload".into(),
            ));
        }
    }
    if paths.len() != journal.before_modes.len() {
        return Err(SetupError::Conflict(
            "projection permission inventory changed".into(),
        ));
    }
    Ok(())
}

fn load(
    root: &Dir,
    store: &ProgressStore,
    progress: &SetupProgress,
) -> Result<Option<Journal>, SetupError> {
    let bytes =
        read_user_setup_file(root, &path(progress)?, LIMIT).map_err(SetupError::Conflict)?;
    let binding = store.projection_binding(progress, None)?;
    let Some(bytes) = bytes else {
        if binding.is_some() {
            return Err(SetupError::Conflict(
                "projection recovery journal is missing".into(),
            ));
        }
        return Ok(None);
    };
    let journal: Journal = serde_json::from_slice(&bytes)
        .map_err(|_| SetupError::Verification("invalid projection recovery journal".into()))?;
    validate(&journal, progress)?;
    let digest = sha256_digest(&bytes);
    if binding.as_deref().is_some_and(|prior| prior != digest) {
        return Err(SetupError::Conflict(
            "projection recovery journal changed".into(),
        ));
    }
    // A crash between journal creation and receipt binding cannot have applied any file.
    for change in &journal.projection.changes {
        if binding.is_some() {
            crate::user_install::recover_user_setup_file(
                root,
                &change.path,
                &change
                    .before
                    .as_deref()
                    .map_or_else(|| "absent".into(), sha256_digest),
                &change
                    .after
                    .as_deref()
                    .map_or_else(|| "absent".into(), sha256_digest),
                journal.before_modes[&change.path],
            )
            .map_err(SetupError::Conflict)?;
        }
        let current = read_user_setup_file(root, &change.path, super::MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        if current != change.before && (binding.is_none() || current != change.after) {
            return Err(SetupError::Conflict(format!(
                "projection changed during recovery: {}",
                change.path.display()
            )));
        }
        verify_mode(
            root,
            change,
            current.as_deref(),
            journal.before_modes[&change.path],
        )?;
    }
    if binding.is_none() {
        store.projection_binding(progress, Some(&digest))?;
    }
    Ok(Some(journal))
}

pub(super) fn preflight(
    root: &Dir,
    store: &ProgressStore,
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    load(root, store, progress).map(|_| ())
}

pub(super) fn apply(
    root: &Dir,
    store: &ProgressStore,
    progress: &SetupProgress,
    prepare: impl FnOnce() -> Result<AppliedProjection, SetupError>,
) -> Result<AppliedProjection, SetupError> {
    let journal = if let Some(journal) = load(root, store, progress)? {
        journal
    } else {
        let projection = prepare()?;
        let mut before_modes = std::collections::BTreeMap::new();
        for change in &projection.changes {
            before_modes.insert(
                change.path.clone(),
                if change.before.is_some() {
                    crate::user_install::host_policy_mode(root, &change.path)
                        .map_err(SetupError::Conflict)?
                } else {
                    None
                },
            );
        }
        let journal = Journal {
            schema_version: 1,
            package_version: env!("HIVE_PACKAGE_VERSION").into(),
            operation_id: progress.operation_id.clone(),
            candidate_digest: declaration_digest(&progress.answers)
                .map_err(SetupError::Internal)?,
            projection,
            before_modes,
        };
        validate(&journal, progress)?;
        let bytes = serde_json::to_vec(&journal)
            .map_err(|error| SetupError::Internal(error.to_string()))?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > LIMIT {
            return Err(SetupError::Input(
                "projection recovery exceeds its size limit".into(),
            ));
        }
        replace_user_setup_file(root, &path(progress)?, None, Some(&bytes))
            .map_err(SetupError::Conflict)?;
        store.projection_binding(progress, Some(&sha256_digest(&bytes)))?;
        journal
    };
    let removed_skills: Vec<_> = journal
        .projection
        .changes
        .iter()
        .filter(|change| change.after.is_none() && change.path.starts_with(".agents/skills"))
        .map(|change| change.path.clone())
        .collect();
    let mut applied = AppliedProjection {
        reports: journal.projection.reports,
        ..AppliedProjection::default()
    };
    for change in journal.projection.changes {
        let current = read_user_setup_file(root, &change.path, super::MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        verify_mode(
            root,
            &change,
            current.as_deref(),
            journal.before_modes[&change.path],
        )?;
        if current == change.after {
            continue;
        }
        if current != change.before {
            return Err(SetupError::Conflict(
                "projection changed before recovery write".into(),
            ));
        }
        replace_user_setup_file(
            root,
            &change.path,
            change.before.as_deref(),
            change.after.as_deref(),
        )
        .map_err(SetupError::Conflict)?;
        applied
            .changed_paths
            .push(change.path.to_string_lossy().into_owned());
        applied.changes.push(change);
    }
    crate::user_install::prune_user_setup_empty_ancestors(
        root,
        removed_skills.iter().map(PathBuf::as_path),
    )
    .map_err(SetupError::Conflict)?;
    Ok(applied)
}

fn verify_mode(
    root: &Dir,
    change: &super::ProjectionChange,
    current: Option<&[u8]>,
    before_mode: Option<u32>,
) -> Result<(), SetupError> {
    if current.is_some() {
        let expected = if current == change.before.as_deref() {
            before_mode
        } else if cfg!(unix) {
            Some(0o644)
        } else {
            None
        };
        if crate::user_install::host_policy_mode(root, &change.path)
            .map_err(SetupError::Conflict)?
            != expected
        {
            return Err(SetupError::Conflict(
                "projection permissions changed during recovery".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn cleanup(
    root: &Dir,
    progress: &SetupProgress,
    digest: Option<&str>,
) -> Result<(), SetupError> {
    let relative = path(progress)?;
    let Some(bytes) = read_user_setup_file(root, &relative, LIMIT).map_err(SetupError::Conflict)?
    else {
        return Ok(());
    };
    if digest != Some(sha256_digest(&bytes).as_str()) {
        return Err(SetupError::Conflict(
            "projection recovery changed before cleanup".into(),
        ));
    }
    replace_user_setup_file(root, &relative, Some(&bytes), None).map_err(SetupError::Conflict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::{progress_store::ProgressPhase, ProjectionChange};
    use std::path::Path;

    fn fixture() -> (
        tempfile::TempDir,
        Dir,
        ProgressStore,
        SetupProgress,
        AppliedProjection,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let (_, root) =
            crate::user_install::open_canonical_user_root_for_setup(temp.path()).unwrap();
        let store = ProgressStore::open(temp.path(), None).unwrap();
        let mut progress = store
            .new_progress(
                "user-language".into(),
                serde_json::json!({"interface_language":"ko"}),
            )
            .unwrap();
        progress.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_applying(&progress).unwrap();
        let mut projection = AppliedProjection::default();
        for name in [
            ".agents/directives/00-hive-user.md",
            ".hive/config/user-active-skills.yml",
        ] {
            replace_user_setup_file(&root, Path::new(name), None, Some(b"before")).unwrap();
            projection.changes.push(ProjectionChange {
                path: name.into(),
                before: Some(b"before".to_vec()),
                after: Some(b"after".to_vec()),
            });
        }
        (temp, root, store, progress, projection)
    }

    fn save_intent(
        root: &Dir,
        store: &ProgressStore,
        progress: &SetupProgress,
        projection: AppliedProjection,
        bind: bool,
    ) -> Vec<u8> {
        let journal = Journal {
            schema_version: 1,
            package_version: env!("HIVE_PACKAGE_VERSION").into(),
            operation_id: progress.operation_id.clone(),
            candidate_digest: declaration_digest(&progress.answers).unwrap(),
            before_modes: projection
                .changes
                .iter()
                .map(|change| {
                    (
                        change.path.clone(),
                        crate::user_install::host_policy_mode(root, &change.path).unwrap(),
                    )
                })
                .collect(),
            projection,
        };
        let bytes = serde_json::to_vec(&journal).unwrap();
        replace_user_setup_file(root, &path(progress).unwrap(), None, Some(&bytes)).unwrap();
        if bind {
            store
                .projection_binding(progress, Some(&sha256_digest(&bytes)))
                .unwrap();
        }
        bytes
    }

    #[test]
    fn partial_projection_resumes_only_unfinished_files_then_removes_recovery_bytes() {
        let (temp, root, store, progress, projection) = fixture();
        save_intent(&root, &store, &progress, projection.clone(), true);
        let first = &projection.changes[0];
        replace_user_setup_file(
            &root,
            &first.path,
            first.before.as_deref(),
            first.after.as_deref(),
        )
        .unwrap();
        let result = apply(&root, &store, &progress, || panic!("must use saved plan")).unwrap();
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].path, projection.changes[1].path);
        for change in projection.changes {
            assert_eq!(
                read_user_setup_file(&root, &change.path, 1024).unwrap(),
                change.after
            );
        }
        store.record_completion(&progress).unwrap();
        assert!(!temp.path().join(path(&progress).unwrap()).exists());
        assert_eq!(
            store.load().unwrap().progress.unwrap().phase,
            ProgressPhase::Complete
        );
    }

    #[test]
    fn changed_file_or_journal_never_overwrites_external_data() {
        for tamper_record in [false, true] {
            let (temp, root, store, progress, projection) = fixture();
            let mut bytes = save_intent(&root, &store, &progress, projection.clone(), true);
            if tamper_record {
                bytes.push(b' ');
                std::fs::write(temp.path().join(path(&progress).unwrap()), bytes).unwrap();
            } else {
                std::fs::write(
                    temp.path().join(&projection.changes[1].path),
                    b"external edit",
                )
                .unwrap();
            }
            assert!(apply(&root, &store, &progress, || panic!(
                "cannot replace recovery plan"
            ))
            .is_err());
            assert_eq!(
                read_user_setup_file(&root, &projection.changes[0].path, 1024).unwrap(),
                Some(b"before".to_vec())
            );
            if !tamper_record {
                assert_eq!(
                    read_user_setup_file(&root, &projection.changes[1].path, 1024).unwrap(),
                    Some(b"external edit".to_vec())
                );
            }
        }
    }

    #[test]
    fn a_journal_saved_before_its_binding_can_resume_only_before_any_file_write() {
        for already_written in [false, true] {
            let (_temp, root, store, progress, projection) = fixture();
            save_intent(&root, &store, &progress, projection.clone(), false);
            if already_written {
                let first = &projection.changes[0];
                replace_user_setup_file(
                    &root,
                    &first.path,
                    first.before.as_deref(),
                    first.after.as_deref(),
                )
                .unwrap();
            }
            assert_eq!(
                apply(&root, &store, &progress, || panic!("saved plan exists")).is_ok(),
                !already_written
            );
        }
    }

    #[test]
    fn unowned_paths_are_rejected_before_a_recovery_plan_is_saved() {
        let (temp, root, store, progress, mut projection) = fixture();
        projection.changes[0].path = "foreign.txt".into();
        assert!(apply(&root, &store, &progress, || Ok(projection)).is_err());
        assert!(!temp.path().join(path(&progress).unwrap()).exists());
        assert!(!temp.path().join("foreign.txt").exists());
    }

    #[test]
    fn cleanup_resumes_after_a_retired_skill_file_was_already_removed() {
        let (temp, root, store, progress, _) = fixture();
        let relative = Path::new(".agents/skills/usage-guard/SKILL.md");
        replace_user_setup_file(&root, relative, None, Some(b"old skill")).unwrap();
        let projection = AppliedProjection {
            changes: vec![ProjectionChange {
                path: relative.into(),
                before: Some(b"old skill".to_vec()),
                after: None,
            }],
            ..AppliedProjection::default()
        };
        save_intent(&root, &store, &progress, projection, true);
        replace_user_setup_file(&root, relative, Some(b"old skill"), None).unwrap();
        assert!(temp.path().join(relative.parent().unwrap()).exists());
        let result = apply(&root, &store, &progress, || {
            panic!("saved deletion intent exists")
        })
        .unwrap();
        assert!(result.changes.is_empty());
        assert!(!temp.path().join(relative.parent().unwrap()).exists());
    }
}
