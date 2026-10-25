//! Localize user guidance without translating or discarding technical diagnostics.
use crate::{
    user_setup::{self, InterfaceLanguage, SetupError},
    ActionResult,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn selected(value: &Value) -> Option<InterfaceLanguage> {
    match value.as_str()? {
        "ko" => Some(InterfaceLanguage::Ko),
        "en" => Some(InterfaceLanguage::En),
        _ => None,
    }
}

fn stored(path: &Path, relative: &str) -> Option<InterfaceLanguage> {
    let root = crate::user_install::open_user_root_for_setup(path).ok()?;
    let bytes = crate::user_install::read_user_setup_file(
        &root,
        Path::new(relative),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .ok()??;
    let value: Value = serde_yaml::from_slice(&bytes).ok()?;
    selected(&value["interface_language"])
}

pub(in crate::user_setup) fn language_at(root: &Path, target: Option<&Path>) -> InterfaceLanguage {
    if target.is_none_or(|target| hive_core::ensure_consumer_target(target).is_ok()) {
        if let Ok(store) = user_setup::progress_store::ProgressStore::open(root, target) {
            if let Ok(snapshot) = store.load() {
                if let Some(language) = snapshot
                    .progress
                    .as_ref()
                    .and_then(|progress| selected(&progress.answers["interface_language"]))
                {
                    return language;
                }
            }
        }
        if let Some(language) = target.and_then(|target| stored(target, ".hive/setup-answers.yml"))
        {
            return language;
        }
    }
    stored(root, user_setup::USER_SETUP_RELATIVE).unwrap_or(InterfaceLanguage::En)
}

pub(in crate::user_setup) fn language(arguments: &[String]) -> InterfaceLanguage {
    let option = |name| {
        arguments
            .windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| PathBuf::from(&pair[1]))
    };
    let Some(root) = option("--user-root").or_else(|| user_setup::resolve_user_root().ok()) else {
        return InterfaceLanguage::En;
    };
    language_at(&root, option("--target").as_deref())
}

pub(in crate::user_setup) fn failure(
    error: &SetupError,
    language: InterfaceLanguage,
) -> ActionResult {
    let mut result = user_setup::failure(error);
    let messages = match error.message() {
        "unknown setup question section" | "unknown setup question section id" =>
            ["알 수 없는 설정 항목입니다. --section 값을 확인하거나 대화형 설정에서 항목을 다시 골라 주세요.", "Unknown settings group. Check --section or choose a group in interactive setup."],
        "interactive mode conflicts with non-interactive or JSON output" =>
            ["대화형 입력과 무입력 또는 JSON 출력을 함께 선택할 수 없습니다.", "Interactive input cannot be combined with non-interactive mode or JSON output."],
        "unknown interactive setup option" | "unknown project setup option" =>
            ["알 수 없는 설정 옵션입니다. hive setup --help로 사용할 옵션을 확인해 주세요.", "Unknown setup option. Run hive setup --help to inspect the available options."],
        "duplicate setup option" | "duplicate project setup option" =>
            ["같은 설정 옵션을 두 번 지정했습니다. 중복된 입력을 지워 주세요.", "A setup option was supplied twice. Remove the duplicate input."],
        "turn off Codex background monitoring before deselecting Codex" =>
            ["Codex의 상시 감시를 먼저 꺼 주세요. 감시 선택을 바꾼 뒤 앱 선택을 해제할 수 있습니다.", "Turn off Codex background monitoring before removing Codex from the selected apps."],
        _ => match error {
            SetupError::Input(_) => ["설정 입력을 확인해 주세요. 자세한 원인은 JSON 출력의 diagnostic 항목에서 확인할 수 있습니다.", "Check the setup input. The diagnostic field in JSON output contains the original cause."],
            SetupError::Conflict(_) => ["설정이나 진행 기록이 예상과 다릅니다. 현재 상태를 확인한 뒤 저장된 진행을 재개해 주세요. 자세한 원인은 JSON 출력의 diagnostic 항목에 있습니다.", "Settings or progress differ from the expected state. Inspect the current state before resuming saved progress. JSON diagnostic contains the original cause."],
            SetupError::Verification(_) => ["설정의 완료를 확인하지 못했습니다. 이미 끝난 단계가 있을 수 있습니다. 저장된 진행과 JSON 출력의 diagnostic 항목을 확인해 주세요.", "Setup completion could not be verified. Some steps may already be complete. Inspect saved progress and the diagnostic field in JSON output."],
            SetupError::Internal(_) => ["설정 처리 중 오류가 발생했습니다. 저장된 진행과 JSON 출력의 diagnostic 항목을 확인해 주세요.", "An error occurred during setup. Inspect saved progress and the diagnostic field in JSON output."],
        },
    };
    result.message = messages[usize::from(language == InterfaceLanguage::En)].into();
    result.data = Some(
        json!({"interface_language":language,"diagnostic":{"original_message":error.message()}}),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn language_lookup_is_read_only_and_uses_pending_project_then_global_choices() {
        let root = crate::test_tempdir();
        let project = crate::test_tempdir();
        assert_eq!(language_at(root.path(), None), InterfaceLanguage::En);
        assert!(!root.path().join(".hive").exists());
        let file = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"interface_language: ko\n").unwrap();
        assert_eq!(language_at(root.path(), None), InterfaceLanguage::Ko);
        let store =
            user_setup::progress_store::ProgressStore::open(root.path(), Some(project.path()))
                .unwrap();
        let progress = store
            .new_progress("project-mode".into(), json!({"interface_language":"en"}))
            .unwrap();
        store.save(&store.load().unwrap(), &progress).unwrap();
        assert_eq!(
            language_at(root.path(), Some(project.path())),
            InterfaceLanguage::En
        );
        assert_eq!(std::fs::read(file).unwrap(), b"interface_language: ko\n");
    }

    #[test]
    fn localized_explanation_preserves_error_codes_and_original_diagnostic() {
        for error in [
            SetupError::Input("unknown setup question section".into()),
            SetupError::Conflict("fixture conflict".into()),
            SetupError::Verification("fixture incomplete".into()),
            SetupError::Internal("fixture internal".into()),
        ] {
            let old = user_setup::failure(&error);
            for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
                let result = failure(&error, language);
                assert_eq!(
                    (result.code, result.status, result.exit_code),
                    (old.code, old.status, old.exit_code)
                );
                assert_eq!(
                    result.data.unwrap()["diagnostic"]["original_message"],
                    error.message()
                );
                assert_ne!(result.message, error.message());
            }
        }
    }
}
