//! Validate completed work first, then recover only an unchanged approved setup intent.
use crate::user_setup::{
    self,
    progress_store::{ProgressStore, SetupProgress},
    InterfaceLanguage, SetupError,
};
use crate::ActionResult;
use std::path::Path;

pub(super) fn resume(
    root: &Path,
    progress: &SetupProgress,
    language: InterfaceLanguage,
    legacy_parent: bool,
) -> Result<ActionResult, SetupError> {
    if legacy_parent {
        return Ok(super::interactive::pending(
            language,
            "hive.update-input-pending",
        ));
    }
    let handoff = ProgressStore::open(root, None)?;
    if handoff.recover_monitor_handoff()? {
        let next = handoff.load()?.progress.ok_or_else(|| {
            SetupError::Conflict("monitor handoff has no replacement setup".into())
        })?;
        return resume(root, &next, language, false);
    }
    if let Some(result) = super::monitor_review::resume(root, progress, language)? {
        return Ok(result);
    }
    let (_, root_cap) = crate::user_install::open_canonical_user_root_for_setup(root)
        .map_err(SetupError::Conflict)?;
    let _lease = crate::user_install::lock_user_setup(&root_cap).map_err(SetupError::Conflict)?;
    let store = ProgressStore::open(root, None)?;
    let application = (|| {
        let (result, repaired) = settings(root, progress, None)?;
        let bytes = serde_json::to_vec(&progress.answers)
            .map_err(|e| SetupError::Internal(e.to_string()))?;
        super::legacy::finish_monitor(root, &bytes, progress)?;
        Ok((result, repaired))
    })();
    let repaired = application.as_ref().is_ok_and(|(_, repaired)| *repaired);
    let mut result =
        super::application::finish(&store, progress, application.map(|(result, _)| result))?;
    result.code = "hive.user-setup-complete";
    result.message = match (language, repaired) {
        (InterfaceLanguage::Ko, true) => "남아 있던 설정 적용과 검증을 마쳤습니다.",
        (InterfaceLanguage::En, true) => "The remaining setup steps were applied and verified.",
        (InterfaceLanguage::Ko, false) => {
            "저장된 설정의 적용·검증을 확인했습니다. 설치를 반복하지 않았습니다."
        }
        (InterfaceLanguage::En, false) => {
            "Saved setup was applied and verified. Installation was not repeated."
        }
    }
    .into();
    result.data = Some(
        serde_json::json!({"operation_id":progress.operation_id,"recovered":true,"remaining_applied":repaired,"model_calls":0}),
    );
    super::monitor_review::report(root, progress, language, &mut result)?;
    Ok(result)
}

/// The caller owns the user setup lease. Withdrawal deliberately omits monitor activation.
pub(super) fn settings(
    root: &Path,
    progress: &SetupProgress,
    stopped: Option<&crate::usage_monitor::StoppedReader>,
) -> Result<(ActionResult, bool), SetupError> {
    let bytes =
        serde_json::to_vec(&progress.answers).map_err(|e| SetupError::Internal(e.to_string()))?;
    let (user_root, root_cap) = crate::user_install::open_canonical_user_root_for_setup(root)
        .map_err(SetupError::Conflict)?;
    let store = ProgressStore::open(root, None)?;
    let mut arguments = user_setup::Arguments {
        answers: "saved:setup-recovery".into(),
        mode: user_setup::SetupMode::Validate,
        user_root,
        root_cap,
    };
    if !store.questions_match(progress) || store.load()?.progress.as_ref() != Some(progress) {
        return Err(SetupError::Conflict("setup recovery inputs changed".into()));
    }
    user_setup::progress_store::recover_config(&arguments.root_cap, &store, progress)?;
    super::vector_choice::apply(&arguments.root_cap, progress)?;
    crate::user_install::deactivation::apply(root, &progress.host_deactivations)
        .map_err(SetupError::Verification)?;
    let validated = user_setup::execute_answers(&arguments, &bytes, user_setup::SetupBase::Current);
    let repaired = validated.is_err();
    let result = if let Ok(result) = validated {
        result
    } else {
        let current = crate::user_install::read_user_setup_file(
            &arguments.root_cap,
            Path::new(user_setup::USER_SETUP_RELATIVE),
            user_setup::MAX_USER_SETUP_BYTES,
        )
        .map_err(SetupError::Conflict)?;
        let desired =
            user_setup::canonical_config(&user_setup::parse_and_validate_config(&bytes)?)?;
        if current.as_deref() != Some(desired.as_slice())
            && user_setup::progress_store::optional_digest(current.as_deref())
                != progress.base_config_digest
        {
            return Err(SetupError::Conflict(
                "settings changed after interrupted apply; changes preserved".into(),
            ));
        }
        arguments.mode = user_setup::SetupMode::Apply;
        super::legacy::apply_settings(&arguments, &bytes, current.as_deref(), progress, stopped)?
    };
    Ok((result, repaired))
}
