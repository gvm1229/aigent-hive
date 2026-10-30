//! A caught project apply failure restores the journal's exact prior bytes.
use super::{AppliedProjection, Dir, Journal, PathBuf, PreferenceScope, SetupError, SetupProgress};
use std::{collections::BTreeSet, io::ErrorKind, path::Component};

pub(super) fn new_directories(
    output: &Dir,
    progress: &SetupProgress,
    projection: &AppliedProjection,
) -> Result<BTreeSet<PathBuf>, SetupError> {
    let mut directories = BTreeSet::new();
    if progress.binding.scope != PreferenceScope::Project {
        return Ok(directories);
    }
    for change in &projection.changes {
        for parent in change
            .path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            match output.symlink_metadata(parent) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(SetupError::Conflict(
                        "project parent is not a directory".into(),
                    ))
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    directories.insert(parent.to_owned());
                }
                Err(error) => return Err(SetupError::Internal(error.to_string())),
            }
        }
    }
    Ok(directories)
}

pub(super) fn validate_directories(
    journal: &Journal,
    progress: &SetupProgress,
) -> Result<(), SetupError> {
    for path in &journal.new_directories {
        if progress.binding.scope != PreferenceScope::Project
            || path.as_os_str().is_empty()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            || !journal
                .projection
                .changes
                .iter()
                .any(|change| change.path != *path && change.path.starts_with(path))
        {
            return Err(SetupError::Conflict(
                "invalid project rollback directory".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn inject_failure(
    progress: &SetupProgress,
    operations: usize,
) -> Result<(), SetupError> {
    #[cfg(debug_assertions)]
    if progress.binding.scope == PreferenceScope::Project {
        if let Ok(value) = std::env::var("HIVE_TEST_ACTIVATION_FAIL_AFTER") {
            let (scope, count) = value
                .rsplit_once('@')
                .map_or((None, value.as_str()), |(s, n)| (Some(s), n));
            if scope.is_none_or(|s| s == format!("{:?}", std::thread::current().id()))
                && count.parse::<usize>().ok().filter(|n| *n <= 4096) == Some(operations)
            {
                return Err(SetupError::Internal(
                    "injected project activation failure".into(),
                ));
            }
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (progress, operations);
    Ok(())
}

pub(super) fn on_failure(
    output: &Dir,
    progress: &SetupProgress,
    journal: &Journal,
    error: SetupError,
) -> SetupError {
    if progress.binding.scope != PreferenceScope::Project {
        return error;
    }
    match restore(output, journal) {
        Ok(()) => match error {
            SetupError::Internal(message) => {
                SetupError::Internal(format!("project activation rolled back: {message}"))
            }
            other => other,
        },
        Err(rollback) => SetupError::Internal(format!(
            "hive.activation-rollback-failed: {rollback}; original failure: {}",
            error.message()
        )),
    }
}

fn restore(output: &Dir, journal: &Journal) -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var("HIVE_TEST_ROLLBACK_FAIL").as_deref() == Ok("1") {
        return Err("injected rollback failure".into());
    }
    for change in journal.projection.changes.iter().rev() {
        let current = super::read_user_setup_file(
            output,
            &change.path,
            crate::user_setup::MAX_USER_SETUP_BYTES,
        )?;
        if current == change.before {
            continue;
        }
        if current != change.after {
            return Err(format!(
                "changed project file preserved: {}",
                change.path.display()
            ));
        }
        super::verify_mode(
            output,
            change,
            current.as_deref(),
            journal.before_modes[&change.path],
        )
        .map_err(|e| e.message().to_owned())?;
        crate::user_install::restore_setup_file(
            output,
            &change.path,
            change.after.as_deref(),
            change.before.as_deref(),
            journal.before_modes[&change.path],
        )?;
    }
    // Deepest paths first; only parents absent before this operation are eligible.
    let mut directories: Vec<_> = journal.new_directories.iter().collect();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for path in directories {
        match output.remove_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "project directory preserved: {}: {error}",
                    path.display()
                ))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::ProjectionChange;

    fn journal(changes: Vec<ProjectionChange>, mode: Option<u32>) -> Journal {
        Journal {
            schema_version: 1,
            package_version: env!("HIVE_PACKAGE_VERSION").into(),
            operation_id: "fixture".into(),
            candidate_digest: "fixture".into(),
            before_modes: changes
                .iter()
                .map(|c| (c.path.clone(), c.before.as_ref().and(mode)))
                .collect(),
            projection: AppliedProjection {
                changes,
                ..AppliedProjection::default()
            },
            new_directories: BTreeSet::new(),
        }
    }

    #[test]
    fn restore_replacements_deletions_and_new_files_preserves_original_modes_and_empty_parents() {
        let temp = crate::test_tempdir();
        let (_, root) =
            crate::user_install::open_canonical_user_root_for_setup(temp.path()).unwrap();
        root.create_dir_all(".hive/config").unwrap();
        root.create_dir_all(".agents/skills/quick-answer").unwrap();
        root.write(".hive/config/approved-skills.yml", b"after")
            .unwrap();
        root.write(".agents/skills/quick-answer/SKILL.md", b"new")
            .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                temp.path().join(".hive/config/approved-skills.yml"),
                std::fs::Permissions::from_mode(0o640),
            )
            .unwrap();
        }
        let mode = cfg!(unix).then_some(0o640);
        let mut journal = journal(
            vec![
                ProjectionChange {
                    path: ".hive/config/approved-skills.yml".into(),
                    before: Some(b"before".to_vec()),
                    after: Some(b"after".to_vec()),
                },
                ProjectionChange {
                    path: ".hive/config/approved-hooks.yml".into(),
                    before: Some(b"deleted".to_vec()),
                    after: None,
                },
                ProjectionChange {
                    path: ".agents/skills/quick-answer/SKILL.md".into(),
                    before: None,
                    after: Some(b"new".to_vec()),
                },
            ],
            mode,
        );
        journal.new_directories = [".agents/skills", ".agents/skills/quick-answer"]
            .into_iter()
            .map(PathBuf::from)
            .collect();
        restore(&root, &journal).unwrap();
        assert_eq!(
            root.read(".hive/config/approved-skills.yml").unwrap(),
            b"before"
        );
        assert_eq!(
            root.read(".hive/config/approved-hooks.yml").unwrap(),
            b"deleted"
        );
        for name in ["approved-skills.yml", "approved-hooks.yml"] {
            assert_eq!(
                crate::user_install::host_policy_mode(
                    &root,
                    &PathBuf::from(".hive/config").join(name)
                )
                .unwrap(),
                mode
            );
        }
        assert!(root.is_dir(".agents"));
        assert!(!root.exists(".agents/skills"));
    }

    #[test]
    fn restore_never_overwrites_a_foreign_edit() {
        let temp = crate::test_tempdir();
        let (_, root) =
            crate::user_install::open_canonical_user_root_for_setup(temp.path()).unwrap();
        root.create_dir_all(".hive/config").unwrap();
        let path = PathBuf::from(".hive/config/approved-skills.yml");
        root.write(&path, b"foreign").unwrap();
        let journal = journal(
            vec![ProjectionChange {
                path: path.clone(),
                before: Some(b"before".to_vec()),
                after: Some(b"after".to_vec()),
            }],
            cfg!(unix).then_some(0o644),
        );
        assert!(restore(&root, &journal).unwrap_err().contains("preserved"));
        assert_eq!(root.read(path).unwrap(), b"foreign");
    }
}
