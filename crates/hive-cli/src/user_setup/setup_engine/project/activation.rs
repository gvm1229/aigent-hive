//! Apply a confirmed render through the existing scoped recovery journal.
use super::{execute_core, request, write_answers, Options, ProgressStore, SetupError};
use crate::user_setup::{
    progress_store::{ProjectContext, SetupProgress},
    AppliedProjection, ProjectionChange,
};
use hive_core::preferences::declaration_digest;
use hive_render::{GlobalProjectPreferences, PreparedSetup, SetupMode, SetupOutcome};
use std::io::Write;

fn global_digest(global: &GlobalProjectPreferences) -> Result<String, SetupError> {
    declaration_digest(
        &serde_json::to_value(global).map_err(|error| SetupError::Internal(error.to_string()))?,
    )
    .map_err(SetupError::Internal)
}

pub(super) fn bind_context(
    progress: &mut SetupProgress,
    global: &GlobalProjectPreferences,
    capabilities: &[u8],
    prepared: &SetupOutcome,
) -> Result<(), SetupError> {
    hive_core::run::CapabilityResolution::parse_json(capabilities)
        .map_err(|error| SetupError::Verification(error.to_string()))?;
    progress.project_context = Some(ProjectContext {
        global_digest: global_digest(global)?,
        plan_digest: plan_digest(prepared)?,
        capabilities: serde_json::from_slice(capabilities)
            .map_err(|error| SetupError::Verification(error.to_string()))?,
    });
    Ok(())
}

fn plan_digest(prepared: &SetupOutcome) -> Result<String, SetupError> {
    let changes = prepared
        .changes
        .iter()
        .map(|change| {
            serde_json::json!({
                "path":change.path,"before":change.before_digest,"after":change.after_digest
            })
        })
        .collect::<Vec<_>>();
    declaration_digest(&serde_json::json!({"tree":prepared.tree_digest,"changes":changes}))
        .map_err(SetupError::Internal)
}

pub(super) fn verify_preview(
    preview: &SetupOutcome,
    prepared: &SetupOutcome,
) -> Result<(), SetupError> {
    if preview.changes.len() != prepared.changes.len()
        || preview
            .changes
            .iter()
            .zip(&prepared.changes)
            .any(|(before, after)| {
                before.path != after.path
                    || before.before_digest != after.before_digest
                    || (before.path != ".hive/setup-answers.yml"
                        && before.after_digest != after.after_digest)
            })
    {
        return Err(SetupError::Conflict(
            "project files changed after the preview; review the current changes".into(),
        ));
    }
    Ok(())
}

pub(super) fn apply(
    options: &Options,
    store: &ProgressStore,
    progress: &SetupProgress,
    global: &GlobalProjectPreferences,
    prepared: Option<PreparedSetup>,
) -> Result<usize, SetupError> {
    let context = progress
        .project_context
        .as_ref()
        .ok_or_else(|| SetupError::Conflict("confirmed project context is missing".into()))?;
    if global_digest(global)? != context.global_digest {
        return Err(SetupError::Conflict(
            "global settings changed since confirmation; the project recovery data was preserved"
                .into(),
        ));
    }
    let root = crate::user_install::open_user_root_for_setup(&options.root)
        .map_err(SetupError::Conflict)?;
    let output = crate::user_install::open_user_root_for_setup(&options.target)
        .map_err(SetupError::Conflict)?;
    let answers = write_answers(&progress.answers)?;
    let mut caps =
        tempfile::NamedTempFile::new().map_err(|error| SetupError::Internal(error.to_string()))?;
    caps.write_all(
        &serde_json::to_vec(&context.capabilities)
            .map_err(|error| SetupError::Internal(error.to_string()))?,
    )
    .map_err(|error| SetupError::Internal(error.to_string()))?;
    #[cfg(debug_assertions)]
    if std::env::var("HIVE_PROJECT_SETUP_INTERRUPT_AFTER").as_deref() == Ok("intent") {
        std::process::exit(87);
    }
    let validation = request(
        options,
        answers.path(),
        caps.path(),
        global,
        SetupMode::Validate,
    );
    if crate::user_setup::projection_journal::bound_payload_is_absent(
        &root, &output, store, progress,
    )? {
        // Completion may have removed recovery bytes before saving its final receipt.
        // Only fresh validation of both project files and registry can close that gap.
        execute_core(&validation, &options.root, global)?;
        return Ok(0);
    }
    let applied =
        crate::user_setup::projection_journal::apply_to(&root, &output, store, progress, || {
            let prepared = match prepared {
                Some(prepared) => prepared,
                None => hive_render::prepare_setup(&request(
                    options,
                    answers.path(),
                    caps.path(),
                    global,
                    SetupMode::DryRun,
                ))
                .map_err(|error| SetupError::Verification(error.to_string()))?,
            };
            if plan_digest(&prepared.outcome)? != context.plan_digest {
                return Err(SetupError::Conflict(
                    "confirmed project preparation changed; recovery data was preserved".into(),
                ));
            }
            Ok(AppliedProjection {
                changes: prepared
                    .changes
                    .into_iter()
                    .map(|change| ProjectionChange {
                        path: change.path,
                        before: change.before,
                        after: change.after,
                    })
                    .collect(),
                ..AppliedProjection::default()
            })
        })?;
    let outcome = hive_render::execute_setup(&validation)
        .map_err(|error| SetupError::Verification(error.to_string()))?;
    let preferences = outcome.effective_preferences.ok_or_else(|| {
        SetupError::Verification("project preferences missing after apply".into())
    })?;
    let mut registry_changes = Vec::new();
    crate::reconcile_project_registry(
        &options.root,
        &options.target,
        &preferences,
        SetupMode::Apply,
        global.wiki_enabled,
        &mut registry_changes,
    )
    .map_err(|error| SetupError::Verification(error.to_string()))?;
    #[cfg(debug_assertions)]
    if std::env::var("HIVE_PROJECT_SETUP_INTERRUPT_AFTER").as_deref() == Ok("registry") {
        std::process::exit(87);
    }
    execute_core(&validation, &options.root, global)?;
    Ok(applied.changed_paths.len() + registry_changes.len())
}
