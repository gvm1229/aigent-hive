//! The setup CLI reuses the authenticated release merger, then its shared write journal.
use super::{
    is_shared, open_target_capability, prepare_candidate, read_full_optional, render_error,
    UpdateError,
};
use hive_core::sha256_digest;
use hive_render::{PreparedSetup, PreparedSetupChange, SetupChange, SetupOutcome, SetupRequest};
use std::path::Path;

pub(crate) fn prepare(request: &SetupRequest<'_>) -> Result<PreparedSetup, UpdateError> {
    hive_core::ensure_consumer_target(request.target)
        .map_err(|error| UpdateError::Input(error.to_string()))?;
    let target = open_target_capability(request.target)?;
    let candidate =
        hive_render::project_reconfiguration_candidate(request).map_err(render_error)?;
    let plugin = request
        .global_preferences
        .as_ref()
        .ok_or_else(|| UpdateError::Input("global project preferences are missing".into()))?;
    let canonical = candidate.files;
    let mut tree = canonical.clone();
    let plan = prepare_candidate(
        &target,
        &plugin.codex_plugin_files,
        None,
        Some(candidate.projection),
    )?;
    // Unchanged merged files can still contain preserved local additions.
    for report in &plan.reports {
        let bytes = match plan.final_files.get(&report.path) {
            Some(bytes) => bytes.clone(),
            None => read_full_optional(&target, Path::new(&report.path))?,
        };
        if let Some(bytes) = bytes {
            tree.insert(report.path.clone().into(), bytes);
        } else {
            tree.remove(Path::new(&report.path));
        }
    }
    for (path, bytes) in &plan.final_files {
        if let Some(bytes) = bytes {
            tree.insert(path.into(), bytes.clone());
        } else {
            tree.remove(Path::new(path));
        }
    }
    hive_render::validate_reconfiguration_tree(request, &tree, &canonical).map_err(render_error)?;
    let mut changes = Vec::new();
    let mut evidence = Vec::new();
    for path in &plan.changed_paths {
        let before = plan.expected_before[path].bytes.clone();
        let after = plan.final_files[path].clone();
        let foreign = |bytes: Option<&[u8]>| -> Result<Option<String>, UpdateError> {
            if is_shared(path) {
                Ok(Some(
                    hive_render::shared_marker_foreign_digest_for_path(
                        Path::new(path),
                        bytes.unwrap_or_default(),
                    )
                    .map_err(render_error)?,
                ))
            } else {
                Ok(None)
            }
        };
        evidence.push(SetupChange {
            path: path.clone(),
            before_digest: before.as_deref().map(sha256_digest),
            after_digest: after.as_deref().map(sha256_digest),
            foreign_before_digest: foreign(before.as_deref())?,
            foreign_after_digest: foreign(after.as_deref())?,
        });
        changes.push(PreparedSetupChange {
            path: path.into(),
            before,
            after,
        });
    }
    Ok(PreparedSetup {
        outcome: SetupOutcome {
            changed_paths: plan.changed_paths,
            resolved_owner: candidate.resolved_owner,
            tree_digest: hive_render::reconfiguration_tree_digest(&tree),
            changes: evidence,
            effective_preferences: Some(candidate.effective_preferences),
        },
        changes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::{collections::BTreeSet, fs, path::PathBuf};

    fn global(root: &Path) -> hive_render::GlobalProjectPreferences {
        let path = root.join(".hive/config/user-setup.yml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path,serde_json::to_vec(&json!({"schema_version":1,"interface_language":"en","wiki":{"enabled":true,"language":"both"},
            "profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},"selected_hosts":["codex"],"skills":{"mode":"all"},
            "usage_guard":{"enabled":false,"stop_remaining_percent":20,"codexbar_fallback_enabled":false,"discord":{"enabled":false}}})).unwrap()).unwrap();
        crate::user_setup::project_preferences(root).unwrap()
    }

    fn fixtures() -> (PathBuf, PathBuf) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/setup");
        (
            root.join("answers-base.yml"),
            root.join("capabilities-codex-host-native.json"),
        )
    }

    #[test]
    fn settings_merge_preserves_local_text_and_prepares_a_valid_new_configuration() {
        let user = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let mut global = global(user.path());
        let (answers, caps) = fixtures();
        fs::write(target.path().join("AGENTS.md"), b"User-owned preface\r\n").unwrap();
        let initial = SetupRequest {
            target: target.path(),
            answers: &answers,
            capabilities: &caps,
            mode: hive_render::SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(global.clone()),
        };
        hive_render::execute_setup(&initial).unwrap();
        let paths = [
            "AGENTS.md",
            ".agents/directives/00-project-harness.md",
            ".agents/skills/ship/SKILL.md",
        ];
        let mut saved = Vec::new();
        for path in paths {
            let original = fs::read(target.path().join(path)).unwrap();
            let changed = if path == "AGENTS.md" {
                String::from_utf8(original)
                    .unwrap()
                    .replace(
                        "<!-- AIGENT-HIVE:END -->",
                        "- User-local addition.\n<!-- AIGENT-HIVE:END -->",
                    )
                    .into_bytes()
            } else {
                [original.as_slice(), b"\nUser-local addition.\n"].concat()
            };
            fs::write(target.path().join(path), &changed).unwrap();
            saved.push(changed);
        }
        let mut value: Value = serde_yaml::from_slice(&fs::read(&answers).unwrap()).unwrap();
        value["project_name"] = json!("renamed project");
        let changed = user.path().join("requested.yml");
        fs::write(&changed, serde_json::to_vec(&value).unwrap()).unwrap();
        global.interface_language = "ko".into();
        let request = SetupRequest {
            answers: &changed,
            global_preferences: Some(global),
            ..initial
        };
        let prepared = prepare(&request).unwrap();
        for (path, bytes) in paths.iter().zip(&saved) {
            assert_eq!(fs::read(target.path().join(path)).unwrap(), *bytes);
        }
        let cap = crate::user_install::open_user_root_for_setup(target.path()).unwrap();
        for change in prepared.changes {
            crate::user_install::replace_user_setup_file(
                &cap,
                &change.path,
                change.before.as_deref(),
                change.after.as_deref(),
            )
            .unwrap();
        }
        let validation = SetupRequest {
            mode: hive_render::SetupMode::Validate,
            ..request
        };
        hive_render::execute_setup(&validation).unwrap();
        let agents = fs::read_to_string(target.path().join("AGENTS.md")).unwrap();
        assert!(agents.starts_with("User-owned preface\r\n"));
        assert!(agents.contains("renamed project"));
        assert!(agents.contains("User-local addition."));
        for (path, bytes) in paths.iter().zip(&saved).skip(1) {
            assert_eq!(fs::read(target.path().join(path)).unwrap(), *bytes);
        }
        assert!(
            fs::read_to_string(target.path().join(".hive/config/harness.toml"))
                .unwrap()
                .contains("interface_language = \"ko\"")
        );
    }

    #[test]
    fn removing_a_modified_feature_defers_the_settings_change_without_deleting_local_content() {
        let user = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let global = global(user.path());
        let (answers, caps) = fixtures();
        let initial = SetupRequest {
            target: target.path(),
            answers: &answers,
            capabilities: &caps,
            mode: hive_render::SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(global),
        };
        hive_render::execute_setup(&initial).unwrap();
        let local = target.path().join(".agents/skills/ship/SKILL.md");
        let mut bytes = fs::read(&local).unwrap();
        bytes.extend_from_slice(b"\nUser custom workflow.\n");
        fs::write(&local, &bytes).unwrap();
        let before = fs::read(target.path().join(".hive/setup-answers.yml")).unwrap();
        let mut value: Value = serde_yaml::from_slice(&fs::read(&answers).unwrap()).unwrap();
        value["setup_mode"] = json!("custom");
        value["interface_language"] = json!("en");
        value["wiki"] = json!({"enabled":true,"language":"both"});
        value["persona"] = json!({"id":"balanced"});
        value["skills"] = json!({"mode":"individual","selected":[]});
        let changed = user.path().join("requested.yml");
        fs::write(&changed, serde_json::to_vec(&value).unwrap()).unwrap();
        let request = SetupRequest {
            answers: &changed,
            ..initial
        };
        assert!(
            matches!(prepare(&request),Err(UpdateError::Conflict(message)) if message.contains(".agents/skills/ship/SKILL.md"))
        );
        assert_eq!(fs::read(local).unwrap(), bytes);
        assert_eq!(
            fs::read(target.path().join(".hive/setup-answers.yml")).unwrap(),
            before
        );
    }
}
