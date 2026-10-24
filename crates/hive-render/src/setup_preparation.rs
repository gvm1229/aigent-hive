//! The lifecycle journal consumes the exact, already-validated setup changes.
use super::{
    execute_setup_with_target, open_target_capability, read_target_optional, RenderError,
    SetupChange, SetupMode, SetupOutcome, SetupRequest,
};
use cap_std::fs::Dir;
use hive_core::{ensure_consumer_target, sha256_digest};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Read-only setup preparation. No target file has been changed.
#[derive(Debug)]
pub struct PreparedSetup {
    /// The same outcome returned by a normal setup dry run.
    pub outcome: SetupOutcome,
    /// Private recovery payloads; do not include these bytes in ordinary CLI output.
    pub changes: Vec<PreparedSetupChange>,
}

/// One exact change from the renderer's validated ownership and consent checks.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSetupChange {
    /// Normalized path relative to the explicitly selected consumer target.
    pub path: PathBuf,
    /// Original bytes, or absence for a new file.
    pub before: Option<Vec<u8>>,
    /// Candidate bytes, or absence for a validated deletion.
    pub after: Option<Vec<u8>>,
}

/// Limit recovery to managed setup files and exact new canonical seeds or declared roles.
/// Existing canonical knowledge, role bodies, runs, and foreign files are never replaced here.
///
/// # Errors
/// Rejects unsafe paths or an invalid compiled ownership manifest.
pub fn setup_recovery_change_is_owned(
    change: &PreparedSetupChange,
    declared_roles: &std::collections::BTreeSet<String>,
) -> Result<bool, RenderError> {
    if super::is_shared_marker_path(&change.path) {
        let Some(after) = change.after.as_deref() else {
            return Ok(false);
        };
        let marker = super::extract_exact_marker(&change.path, after)?;
        return Ok(
            super::merge_shared_marker_bytes(change.before.clone(), &change.path, &marker)?
                == after,
        );
    }
    if super::update_path_is_owned(&change.path)? {
        return Ok(true);
    }
    if change.before.is_some() {
        return Ok(false);
    }
    let Some(after) = change.after.as_deref() else {
        return Ok(false);
    };
    let mut seeds = BTreeMap::new();
    super::insert_static_files(&mut seeds);
    if seeds
        .get(&change.path)
        .is_some_and(|bytes| bytes.as_slice() == after)
    {
        return Ok(true);
    }
    Ok(declared_roles.iter().any(|role| {
        super::valid_role_id(role)
            && change.path == Path::new(".hive/team/roles").join(format!("{role}.md"))
    }))
}

/// Permit a confirmed role definition update only while preserving its user body and work state.
///
/// # Errors
/// Rejects malformed role profiles.
pub fn setup_role_update_is_preserving(
    change: &PreparedSetupChange,
    approved: &std::collections::BTreeSet<String>,
) -> Result<bool, RenderError> {
    let Some(id) = approved.iter().find(|id| {
        super::valid_role_id(id)
            && change.path == Path::new(".hive/team/roles").join(format!("{id}.md"))
    }) else {
        return Ok(false);
    };
    let (Some(before), Some(after)) = (change.before.as_deref(), change.after.as_deref()) else {
        return Ok(false);
    };
    let (old, old_body) = super::parse_role(before)?;
    let (new, new_body) = super::parse_role(after)?;
    Ok(old.role_id == *id
        && new.role_id == *id
        && old_body == new_body
        && old.current_assignment == new.current_assignment
        && old.handoff_path == new.handoff_path)
}

/// Prepare recoverable byte changes through the ordinary setup dry-run engine.
/// Callers must still bind the target and compare the original bytes before writing.
///
/// # Errors
/// Returns the ordinary setup validation, ownership, consent, or conflict error.
/// A target changed during capture is rejected without publishing a partial plan.
pub fn prepare_setup(request: &SetupRequest<'_>) -> Result<PreparedSetup, RenderError> {
    ensure_consumer_target(request.target)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let target = open_target_capability(request.target)?;
    let request = SetupRequest {
        target: request.target,
        answers: request.answers,
        capabilities: request.capabilities,
        mode: SetupMode::DryRun,
        reconfigure_roles: request.reconfigure_roles.clone(),
        global_preferences: request.global_preferences.clone(),
    };
    let mut changes = Vec::new();
    let outcome =
        execute_setup_with_target(&request, &target, true, None, None, Some(&mut changes))?;
    Ok(PreparedSetup { outcome, changes })
}

pub(super) fn capture(
    target: &Dir,
    changes: &[SetupChange],
    planned: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<Vec<PreparedSetupChange>, RenderError> {
    changes
        .iter()
        .map(|change| {
            let path = Path::new(&change.path);
            let before = read_target_optional(target, path)?;
            let after = planned.get(path).cloned();
            if before.as_deref().map(sha256_digest) != change.before_digest
                || after.as_deref().map(sha256_digest) != change.after_digest
            {
                return Err(RenderError::Conflict(format!(
                    "setup target changed during preparation: {}",
                    change.path
                )));
            }
            Ok(PreparedSetupChange {
                path: path.to_owned(),
                before,
                after,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, fs};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/setup")
            .join(name)
    }

    #[test]
    fn prepared_bytes_match_normal_dry_run_and_apply_without_early_writes() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().canonicalize().unwrap();
        fs::write(target.join("AGENTS.md"), b"user text\r\n").unwrap();
        let answers = fixture("answers-base.yml");
        let caps = fixture("capabilities-codex-host-native.json");
        let mut request = SetupRequest {
            target: &target,
            answers: &answers,
            capabilities: &caps,
            mode: SetupMode::DryRun,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        };
        let prepared = prepare_setup(&request).unwrap();
        let declared_roles = super::super::load_answers(&answers)
            .unwrap()
            .0
            .persistent_roles
            .iter()
            .map(|role| role.role_id.clone())
            .collect();
        for change in &prepared.changes {
            assert!(
                setup_recovery_change_is_owned(change, &declared_roles).unwrap(),
                "{}",
                change.path.display()
            );
        }
        assert_eq!(
            prepared.outcome,
            super::super::execute_setup(&request).unwrap()
        );
        assert_eq!(prepared.changes.len(), prepared.outcome.changed_paths.len());
        assert_eq!(
            fs::read(target.join("AGENTS.md")).unwrap(),
            b"user text\r\n"
        );
        assert!(!target.join(".hive").exists());
        request.mode = SetupMode::Apply;
        super::super::execute_setup(&request).unwrap();
        for change in prepared.changes {
            assert_eq!(fs::read(target.join(change.path)).ok(), change.after);
        }
        request.mode = SetupMode::Validate;
        super::super::execute_setup(&request).unwrap();
        assert!(prepare_setup(&request).unwrap().changes.is_empty());
    }

    #[test]
    fn preparation_preserves_foreign_collision_and_rejects_changed_capture() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().canonicalize().unwrap();
        let path = target.join(".agents/directives/00-project-harness.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"foreign directive").unwrap();
        let answers = fixture("answers-base.yml");
        let caps = fixture("capabilities-codex-host-native.json");
        let request = SetupRequest {
            target: &target,
            answers: &answers,
            capabilities: &caps,
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        };
        assert!(prepare_setup(&request).is_err());
        assert_eq!(fs::read(path).unwrap(), b"foreign directive");
        assert!(!target.join(".hive").exists());
        let cap = open_target_capability(&target).unwrap();
        let changed = SetupChange {
            path: "AGENTS.md".into(),
            before_digest: Some(sha256_digest(b"old")),
            after_digest: None,
            foreign_before_digest: None,
            foreign_after_digest: None,
        };
        fs::write(target.join("AGENTS.md"), b"new").unwrap();
        assert!(matches!(
            capture(&cap, &[changed], &BTreeMap::new()),
            Err(RenderError::Conflict(_))
        ));
        assert_eq!(fs::read(target.join("AGENTS.md")).unwrap(), b"new");
    }

    #[test]
    fn recovery_never_replaces_canonical_content_or_foreign_marker_bytes() {
        let declared = std::collections::BTreeSet::from(["reviewer".into()]);
        let role = PreparedSetupChange {
            path: ".hive/team/roles/reviewer.md".into(),
            before: None,
            after: Some(b"declared role".to_vec()),
        };
        assert!(setup_recovery_change_is_owned(&role, &declared).unwrap());
        assert!(
            !setup_recovery_change_is_owned(&role, &std::collections::BTreeSet::new()).unwrap()
        );
        assert!(!setup_recovery_change_is_owned(
            &PreparedSetupChange {
                before: Some(b"user body".to_vec()),
                ..role
            },
            &declared
        )
        .unwrap());
        let shared = PreparedSetupChange {
            path: "AGENTS.md".into(),
            before: Some(b"user bytes\r\n".to_vec()),
            after: Some(
                b"replacement\n\n<!-- AIGENT-HIVE:START -->\nnew\n<!-- AIGENT-HIVE:END -->\n"
                    .to_vec(),
            ),
        };
        assert!(!setup_recovery_change_is_owned(&shared, &declared).unwrap());
        let foreign = PreparedSetupChange {
            path: "README.md".into(),
            before: None,
            after: Some(b"foreign".to_vec()),
        };
        assert!(!setup_recovery_change_is_owned(&foreign, &declared).unwrap());
    }
}
