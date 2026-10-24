//! Render new project choices without treating the current user edits as replaceable bytes.
use super::{ProjectUpgradeCandidate, RenderError, ResolvedProjectPreferences, SetupRequest};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Incoming configuration for a separately authenticated, preserving project merge.
/// The caller must compare the installed base before applying any of these bytes.
pub struct ProjectReconfigurationCandidate {
    /// Canonical incoming projection and support data for the existing upgrade merger.
    pub projection: ProjectUpgradeCandidate,
    /// Complete rendered tree, including original bytes outside exact Hive marker blocks.
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    /// Current resolved global and project choices used by registry reconciliation.
    pub effective_preferences: ResolvedProjectPreferences,
    /// Host-owned or explicitly pinned orchestration owner.
    pub resolved_owner: String,
}

/// Render and stage-validate new project preferences without changing the target.
/// Existing roles keep their body, assignment, and handoff; changing a definition
/// still requires the request's explicit role id.
///
/// # Errors
/// Rejects source workspaces, unsafe paths, invalid answers, missing global settings,
/// incompatible capabilities or consent, and unapproved role changes.
pub fn project_reconfiguration_candidate(
    request: &SetupRequest<'_>,
) -> Result<ProjectReconfigurationCandidate, RenderError> {
    super::ensure_consumer_target(request.target)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let target = super::open_target_capability(request.target)?;
    let source = super::read_installed_harness(&target)?;
    let (answers, value) = super::load_answers(request.answers)?;
    let resolution = super::load_resolution(request.capabilities)?;
    super::validate_answers(&answers)?;
    super::validate_resolution(&answers, &resolution)?;
    if super::derived_owner(&resolution)? != source.resolved_owner {
        return Err(RenderError::Conflict("project settings cannot switch the installed orchestration owner; use an explicit owner migration".into()));
    }
    super::validate_skill_approvals(&answers.approved_optional_skills)?;
    super::validate_hook_approvals(&answers.approved_fallback_hooks, &resolution)?;
    super::validate_schema_instance(super::SETUP_SCHEMA, &value, "setup answers")?;
    let global = request.global_preferences.as_ref().ok_or_else(|| {
        RenderError::Input("project reconfiguration requires global preferences".into())
    })?;
    let effective = super::resolve_effective_project_preferences(&answers, Some(global))?;
    let files = super::render_tree_with_preferences(
        &target,
        &answers,
        &resolution,
        &request.reconfigure_roles,
        effective.as_ref(),
    )?;
    let desired = super::validate_desired_projection_ownership(&files, &answers)?;
    super::validate_owned_paths(files.keys(), &desired, None)?;
    super::stage_and_validate(
        Some(request.target),
        &files,
        &answers,
        &resolution,
        super::staging_corruption_from_environment(),
    )?;
    let mut baseline_effective = effective.clone();
    if let Some(preferences) = baseline_effective.as_mut() {
        preferences.codex_plugin_files.clear();
    }
    let baseline = super::render_tree_with_preferences(
        &target,
        &answers,
        &resolution,
        &request.reconfigure_roles,
        baseline_effective.as_ref(),
    )?;
    let mut mergeable = super::project_upgrade_files(&baseline)?;
    mergeable.retain(|path, _| files.contains_key(Path::new(path)));
    let base_ledger = files
        .get(Path::new(".hive/config/project-base.json"))
        .cloned()
        .ok_or_else(|| RenderError::Internal("reconfiguration omitted its project base".into()))?;
    let support = files
        .iter()
        .filter(|(path, _)| {
            !mergeable.contains_key(&path.to_string_lossy().into_owned())
                && path.as_path() != Path::new(".hive/config/project-base.json")
        })
        .map(|(path, bytes)| (path.to_string_lossy().into_owned(), bytes.clone()))
        .collect();
    let effective_preferences = super::resolved_preferences(&answers, effective.as_ref())
        .ok_or_else(|| {
            RenderError::Internal("reconfiguration omitted global preferences".into())
        })?;
    super::verify_target_capability_current(request.target, &target)?;
    Ok(ProjectReconfigurationCandidate {
        projection: ProjectUpgradeCandidate {
            product_version: env!("CARGO_PKG_VERSION").into(),
            source_version: source.harness_version,
            migration_id: "project-settings-v2".into(),
            normalized_fields: Vec::new(),
            skill_merges: Vec::new(),
            files: mergeable,
            support_files: support,
            base_ledger,
        },
        files,
        effective_preferences,
        resolved_owner: super::derived_owner(&resolution)?.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GlobalProjectPreferences;
    use std::{collections::BTreeSet, fs};
    fn global() -> GlobalProjectPreferences {
        GlobalProjectPreferences {
            codex_plugin_files: BTreeMap::new(),
            interface_language: "en".into(),
            wiki_enabled: true,
            wiki_backend: "markdown".into(),
            wiki_language: "both".into(),
            persona_id: "balanced".into(),
            persona_custom_description: None,
            selected_project_skills: hive_projection::project_default_skills().unwrap(),
            usage_guard_enabled: false,
            codexbar_fallback_enabled: false,
            discord_guard_enabled: false,
            discord_webhook_url_env: None,
            discord_message_fields: super::super::default_discord_message_fields(),
            usage_stop_remaining_percent: 20,
        }
    }

    #[test]
    fn reconfiguration_refuses_source_before_reading_any_setup_inputs() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("hive-source.json"), b"{}").unwrap();
        let missing = root.path().join("missing-input");
        let request = SetupRequest {
            target: root.path(),
            answers: &missing,
            capabilities: &missing,
            mode: super::super::SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(global()),
        };
        assert!(
            matches!(project_reconfiguration_candidate(&request), Err(RenderError::Input(message)) if message.contains("source"))
        );
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn incoming_settings_keep_existing_role_body_and_do_not_replace_any_target_file() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().canonicalize().unwrap();
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/setup");
        let source = fixtures.join("answers-base.yml");
        let caps = fixtures.join("capabilities-codex-host-native.json");
        let initial = SetupRequest {
            target: &target,
            answers: &source,
            capabilities: &caps,
            mode: super::super::SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(global()),
        };
        super::super::execute_setup(&initial).unwrap();
        let role_path = target.join(".hive/team/roles/reviewer.md");
        let (mut assigned, body) =
            super::super::parse_role(&fs::read(&role_path).unwrap()).unwrap();
        assigned.current_assignment = Some("review-running-work".into());
        assigned.handoff_path = Some(".hive/runs/review/handoff.md".into());
        let mut role = super::super::encode_role(&assigned, &body).unwrap();
        role.extend_from_slice(b"\nUser assignment notes remain here.\r\n");
        fs::write(&role_path, &role).unwrap();
        let before = super::super::installed_tree_digest(&target).unwrap();
        let (mut answers, _) = super::super::load_answers(&source).unwrap();
        answers.project_name = "new name".into();
        let changed = root.path().join("requested.yml");
        fs::write(
            &changed,
            super::super::render_setup_answers(&answers).unwrap(),
        )
        .unwrap();
        let request = SetupRequest {
            answers: &changed,
            ..initial
        };
        let candidate = project_reconfiguration_candidate(&request).unwrap();
        assert_eq!(
            candidate.files[Path::new(".hive/team/roles/reviewer.md")],
            role
        );
        assert!(
            String::from_utf8_lossy(&candidate.files[Path::new(".hive/config/harness.toml")])
                .contains("new name")
        );
        assert_eq!(
            super::super::installed_tree_digest(&target).unwrap(),
            before
        );
        assert_eq!(fs::read(&role_path).unwrap(), role);
        let external = fixtures.join("capabilities-codex-omx.json");
        let switched = SetupRequest {
            target: &target,
            answers: &changed,
            capabilities: &external,
            mode: super::super::SetupMode::DryRun,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(global()),
        };
        assert!(
            matches!(project_reconfiguration_candidate(&switched),Err(RenderError::Conflict(message)) if message.contains("orchestration owner"))
        );
        answers.persistent_roles[0]
            .responsibilities
            .push("Review changed settings".into());
        fs::write(
            &changed,
            super::super::render_setup_answers(&answers).unwrap(),
        )
        .unwrap();
        assert!(project_reconfiguration_candidate(&request).is_err());
        let approved = SetupRequest {
            reconfigure_roles: BTreeSet::from(["reviewer".into()]),
            ..request
        };
        let candidate = project_reconfiguration_candidate(&approved).unwrap();
        let (_, old_body) = super::super::parse_role(&role).unwrap();
        let (new_profile, new_body) =
            super::super::parse_role(&candidate.files[Path::new(".hive/team/roles/reviewer.md")])
                .unwrap();
        assert_eq!(new_body, old_body);
        assert_eq!(new_profile.current_assignment, assigned.current_assignment);
        assert_eq!(new_profile.handoff_path, assigned.handoff_path);
        assert_eq!(fs::read(role_path).unwrap(), role);
    }
}
