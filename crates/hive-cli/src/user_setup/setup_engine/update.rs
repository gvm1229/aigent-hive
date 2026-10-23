//! The newly activated executable completes settings and harness work on the selected input channel.
use super::{interactive, legacy, recovery, terminal::Terminal};
use crate::user_setup::{
    self,
    progress_store::{ProgressPhase, ProgressStore},
    InterfaceLanguage, SetupError, UserSetupConfig,
};
use crate::ActionResult;
use hive_core::setup::{InteractionMode, InteractionRequest};
use serde_json::json;
use std::{
    io::{self, BufRead, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

pub(in crate::user_setup) fn run(arguments: &[String]) -> ExitCode {
    let mut result = parse(arguments)
        .and_then(|(root, request)| {
            let mode = request
                .resolve(io::stdin().is_terminal(), io::stderr().is_terminal())
                .map_err(SetupError::Input)?;
            let mut terminal = Terminal {
                reader: io::stdin().lock(),
                writer: io::stderr().lock(),
                language: InterfaceLanguage::En,
            };
            complete(&root, mode, &mut terminal)
        })
        .unwrap_or_else(|error| user_setup::failure(&error));
    result.action = "UpdateHiveUser";
    crate::emit_action_result(&result)
}

fn parse(arguments: &[String]) -> Result<(PathBuf, InteractionRequest), SetupError> {
    let mut root = None;
    let mut mode = None;
    if arguments.len() != 4 {
        return Err(SetupError::Input(
            "invalid update completion arguments".into(),
        ));
    }
    for pair in arguments.chunks_exact(2) {
        match pair[0].as_str() {
            "--user-root" if root.is_none() => root = Some(PathBuf::from(&pair[1])),
            "--interaction" if mode.is_none() => mode = Some(pair[1].as_str()),
            _ => {
                return Err(SetupError::Input(
                    "unknown or duplicate update completion argument".into(),
                ))
            }
        }
    }
    let root = root
        .filter(|root| root.is_absolute())
        .ok_or_else(|| SetupError::Input("absolute update user root required".into()))?;
    let request = match mode {
        Some("terminal") => InteractionRequest {
            interactive: true,
            ..InteractionRequest::default()
        },
        Some("non-interactive") => InteractionRequest {
            non_interactive: true,
            ..InteractionRequest::default()
        },
        _ => return Err(SetupError::Input("invalid update interaction mode".into())),
    };
    Ok((root, request))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Route {
    Questions,
    Recover,
    PreserveDraft,
    ApplySaved,
    NeedsInput,
}

fn route(
    config: Option<&UserSetupConfig>,
    phase: Option<ProgressPhase>,
    transition: bool,
    mode: InteractionMode,
) -> Route {
    if mode == InteractionMode::NeedsInput {
        return Route::NeedsInput;
    }
    if phase.is_some_and(|phase| {
        matches!(
            phase,
            ProgressPhase::Applying | ProgressPhase::RecoverableFailure
        )
    }) {
        return Route::Recover;
    }
    if phase.is_some_and(|phase| phase != ProgressPhase::Complete) {
        if mode == InteractionMode::Terminal && (transition || config.is_none()) {
            return Route::Questions;
        }
        return if config.is_some() {
            Route::PreserveDraft
        } else {
            Route::NeedsInput
        };
    }
    if config.is_none_or(|config| !user_setup::migration::pending_questions(config).is_empty()) {
        if mode == InteractionMode::Terminal {
            return Route::Questions;
        }
        if config.is_none() {
            return Route::NeedsInput;
        }
    }
    Route::ApplySaved
}

pub(super) fn complete<R: BufRead, W: Write>(
    root: &Path,
    mode: InteractionMode,
    terminal: &mut Terminal<R, W>,
) -> Result<ActionResult, SetupError> {
    let (root, cap) = crate::user_install::open_canonical_user_root_for_setup(root)
        .map_err(SetupError::Conflict)?;
    let existing = crate::user_install::read_user_setup_file(
        &cap,
        Path::new(user_setup::USER_SETUP_RELATIVE),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let config = existing
        .as_deref()
        .map(user_setup::parse_and_validate_installed_config)
        .transpose()?;
    if let Some(config) = &config {
        terminal.language = config.interface_language;
    }
    let store = ProgressStore::open(&root, None)?;
    let snapshot = store.load()?;
    let selected = route(
        config.as_ref(),
        snapshot.progress.as_ref().map(|progress| progress.phase),
        snapshot
            .progress
            .as_ref()
            .is_some_and(|progress| progress.legacy_migration.is_some()),
        mode,
    );
    let mut result = match selected {
        Route::NeedsInput => interactive::pending(terminal.language, "hive.update-input-pending"),
        Route::Questions => interactive::execute_update(&root, mode, terminal)?,
        Route::Recover => recovery::resume(
            &root,
            snapshot.progress.as_ref().expect("recoverable progress"),
            terminal.language,
            false,
        )?,
        Route::PreserveDraft => preserve_draft(
            &root,
            cap,
            existing.as_deref().expect("saved configuration"),
        )?,
        Route::ApplySaved => apply_saved(
            &root,
            cap,
            &store,
            &snapshot,
            config.as_ref().expect("saved configuration"),
            existing.as_deref(),
        )?,
    };
    result.action = "UpdateHiveUser";
    if result.exit_code != 0 {
        result.next_action = Some(crate::update_bridge::resume_command(&root));
    }
    if result.exit_code == 0 {
        let cap =
            crate::user_install::open_user_root_for_setup(&root).map_err(SetupError::Conflict)?;
        let saved = user_setup::load_operational_config(&cap)?
            .ok_or_else(|| SetupError::Verification("updated settings are missing".into()))?;
        terminal.language = saved.interface_language;
        let pending = user_setup::migration::pending_questions(&saved);
        if selected == Route::Recover && mode == InteractionMode::Terminal && !pending.is_empty() {
            return complete(&root, mode, terminal);
        }
        result.code = if pending.is_empty() {
            "hive.user-update-complete"
        } else {
            "hive.user-update-complete-pending-input"
        };
        result.data = Some(
            json!({"pending_questions":pending,"settings_schema":saved.schema_version,
            "interface_language":saved.interface_language,"selected_hosts":saved.selected_hosts,"stored_usage_policy":saved.usage_guard,
            "settings_draft_preserved":selected==Route::PreserveDraft,"model_calls":0}),
        );
        result.message = match terminal.language {
            InterfaceLanguage::Ko => if pending.is_empty() {"사용자 설정과 지침 갱신·검증 완료"} else {"사용자 지침 갱신·검증 완료. 기존 보호 선택을 보존했으며 선택 질문은 보류했습니다."},
            InterfaceLanguage::En => if pending.is_empty() {"User settings and guidance updated and verified"} else {"User guidance updated and verified. Existing protection was preserved; optional questions remain."},
        }.into();
    }
    Ok(result)
}

fn apply_saved(
    root: &Path,
    cap: cap_std::fs::Dir,
    store: &ProgressStore,
    snapshot: &user_setup::progress_store::ProgressSnapshot,
    config: &UserSetupConfig,
    existing: Option<&[u8]>,
) -> Result<ActionResult, SetupError> {
    let _lease = crate::user_install::lock_user_setup(&cap).map_err(SetupError::Conflict)?;
    let mut progress = legacy::transition_progress(root, store, config)?;
    progress.phase = ProgressPhase::Applying;
    store.save(snapshot, &progress)?;
    store.record_applying(&progress)?;
    let bytes = serde_json::to_vec(&progress.answers)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let arguments = user_setup::Arguments {
        answers: "saved:update".into(),
        mode: user_setup::SetupMode::Apply,
        user_root: root.to_owned(),
        root_cap: cap,
    };
    let applied = legacy::apply(&arguments, &bytes, existing, &progress);
    match applied {
        Ok(applied) => {
            ProgressStore::open(root, None)?.record_completion(&progress)?;
            Ok(applied)
        }
        Err(error) => {
            store.record_failure(&progress)?;
            Err(error)
        }
    }
}

fn preserve_draft(
    root: &Path,
    cap: cap_std::fs::Dir,
    existing: &[u8],
) -> Result<ActionResult, SetupError> {
    let _lease = crate::user_install::lock_user_setup(&cap).map_err(SetupError::Conflict)?;
    let mut arguments = user_setup::Arguments {
        answers: "saved:update-preserve-draft".into(),
        mode: user_setup::SetupMode::Validate,
        user_root: root.to_owned(),
        root_cap: cap,
    };
    let applied = if let Ok(validated) = user_setup::execute_answers(
        &arguments,
        existing,
        user_setup::SetupBase::Expected(Some(existing)),
    ) {
        validated
    } else {
        arguments.mode = user_setup::SetupMode::Apply;
        let applied = user_setup::execute_answers(
            &arguments,
            existing,
            user_setup::SetupBase::Expected(Some(existing)),
        )?;
        arguments.mode = user_setup::SetupMode::Validate;
        user_setup::execute_answers(&arguments, existing, user_setup::SetupBase::Current)?;
        applied
    };
    crate::usage_monitor::upgrade_existing(root)
        .map_err(|failure| SetupError::Verification(failure.message().into()))?;
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn existing() -> UserSetupConfig {
        user_setup::parse_and_validate_installed_config(&serde_json::to_vec(&json!({"schema_version":1,"interface_language":"ko",
            "wiki":{"language":"ko"},"profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},
            "selected_hosts":["codex"],"skills":{"mode":"all"},"usage_guard":{"enabled":false,"stop_remaining_percent":5}})).unwrap()).unwrap()
    }

    #[test]
    fn machine_updates_preserve_existing_choices_and_unconfirmed_drafts() {
        let old = existing();
        assert_eq!(
            route(Some(&old), None, false, InteractionMode::NonInteractive),
            Route::ApplySaved
        );
        assert_eq!(
            route(Some(&old), None, false, InteractionMode::Terminal),
            Route::Questions
        );
        assert_eq!(
            route(
                Some(&old),
                Some(ProgressPhase::AwaitingAnswers),
                true,
                InteractionMode::NonInteractive
            ),
            Route::PreserveDraft
        );
        assert_eq!(
            route(
                Some(&old),
                Some(ProgressPhase::AwaitingAnswers),
                true,
                InteractionMode::Terminal
            ),
            Route::Questions
        );
        assert_eq!(
            route(
                Some(&old),
                Some(ProgressPhase::Applying),
                true,
                InteractionMode::NonInteractive
            ),
            Route::Recover
        );
    }

    #[test]
    fn fresh_noninteractive_completion_returns_input_needed_without_writes_or_reads() {
        let root = tempfile::tempdir().unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new(b"yes\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        let result = complete(root.path(), InteractionMode::NonInteractive, &mut terminal).unwrap();
        assert_eq!(result.code, "hive.update-input-pending");
        assert_eq!(terminal.reader.position(), 0);
        assert!(terminal.writer.is_empty());
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn completion_rejects_ambiguous_or_relative_input_modes() {
        assert!(parse(&[
            "--user-root".into(),
            "relative".into(),
            "--interaction".into(),
            "terminal".into()
        ])
        .is_err());
        assert!(parse(&[
            "--interaction".into(),
            "terminal".into(),
            "--interaction".into(),
            "non-interactive".into()
        ])
        .is_err());
        assert_eq!(
            route(
                None,
                Some(ProgressPhase::Applying),
                false,
                InteractionMode::NonInteractive
            ),
            Route::Recover
        );
        assert_eq!(
            route(None, None, false, InteractionMode::Terminal),
            Route::Questions
        );
    }
}
