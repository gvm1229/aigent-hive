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
}
