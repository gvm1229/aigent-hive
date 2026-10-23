//! Recover a lost completion reply through validation only, without repeating activation.
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
) -> Result<ActionResult, SetupError> {
    let bytes = serde_json::to_vec(&progress.answers)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let (user_root, root_cap) = crate::user_install::open_canonical_user_root_for_setup(root)
        .map_err(SetupError::Conflict)?;
    let _lease = crate::user_install::lock_user_setup(&root_cap).map_err(SetupError::Conflict)?;
    let store = ProgressStore::open(root, None)?;
    let arguments = user_setup::Arguments {
        answers: "saved:setup-recovery".into(),
        mode: user_setup::SetupMode::Validate,
        user_root,
        root_cap,
    };
    let validated = user_setup::execute_answers(&arguments, &bytes, user_setup::SetupBase::Current);
    if validated.is_err() {
        return Ok(super::interactive::pending(
            language,
            "hive.setup-recovery-required",
        ));
    }
    store.record_completion(progress)?;
    let mut result = validated.expect("validated recovery");
    result.code = "hive.user-setup-complete";
    result.message = match language {
        InterfaceLanguage::Ko => {
            "저장된 설정의 적용·검증을 확인했습니다. 설치를 반복하지 않았습니다."
        }
        InterfaceLanguage::En => {
            "Saved setup was applied and verified. Installation was not repeated."
        }
    }
    .into();
    result.data = Some(
        serde_json::json!({"operation_id":progress.operation_id,"recovered":true,"reapplied":false,"model_calls":0}),
    );
    Ok(result)
}
