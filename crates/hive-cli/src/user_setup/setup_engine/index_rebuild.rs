//! Operation-scoped consent for restoring an index that includes external documents.
use super::{
    catalog,
    interactive::{persist, Conversation},
    terminal::Terminal,
};
use crate::user_setup::{
    self,
    progress_store::{ProgressStore, SetupProgress},
    Arguments, SetupError,
};
use cap_std::fs::Dir;
use hive_core::setup::AnswerInput;
use hive_wiki::store::{RagStore, SetupIndexPreview};
use serde_json::json;
use std::{
    io::{BufRead, Write},
    path::Path,
};

fn open(root: &Path, cap: &Dir) -> Result<RagStore, SetupError> {
    RagStore::open_bound(root, cap).map_err(|error| SetupError::Conflict(error.to_string()))
}

fn preview(root: &Path, cap: &Dir, enabled: bool) -> Result<Option<SetupIndexPreview>, SetupError> {
    if !enabled {
        return Ok(None);
    }
    open(root, cap)?
        .preview_setup_index_rebuild()
        .map_err(|error| SetupError::Conflict(error.to_string()))
}

pub(super) fn collect<R: BufRead, W: Write>(
    root: &Path,
    cap: &Dir,
    store: &ProgressStore,
    progress: &mut Conversation,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    let Some(preview) = preview(root, cap, progress.answers["wiki"]["enabled"] == true)? else {
        progress.index_rebuild_scope = None;
        return Ok(true);
    };
    let question = catalog()?
        .questions
        .into_iter()
        .find(|q| q.id == "user-index-rebuild")
        .ok_or_else(|| SetupError::Internal("missing index rebuild question".into()))?;
    if progress.index_rebuild_scope.as_ref() == Some(&preview.scope_digest)
        && progress.answered_questions.get(&question.id) == Some(&question.revision)
        && progress.question_bindings.get(&question.id) == Some(&(&question).into())
    {
        return Ok(true);
    }
    progress.index_rebuild_scope = None;
    progress.step.clone_from(&question.id);
    progress.answered_questions.remove(&question.id);
    persist(store, progress)?;
    terminal.say("검색 색인을 복구할 범위", "Search index sources")?;
    terminal.say(
        "사용자 폴더의 지식 문서와 다음에 등록된 폴더를 읽습니다:",
        "Read knowledge documents in the user folder and these registered folders:",
    )?;
    for source in &preview.sources {
        // JSON quoting prevents a registered path or alias from injecting terminal controls.
        writeln!(
            terminal.writer,
            "- {}: {}",
            json!(source.aliases),
            json!(source.path)
        )
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    }
    let AnswerInput::Answered { value, .. } = terminal.ask(&question, None)? else {
        return Ok(false);
    };
    if value != json!(true) {
        return Ok(false);
    }
    let current = self::preview(root, cap, true)?;
    if current.as_ref().map(|p| &p.scope_digest) != Some(&preview.scope_digest) {
        return Err(SetupError::Conflict(
            "index sources changed during confirmation; review the saved answers again".into(),
        ));
    }
    progress.index_rebuild_scope = Some(preview.scope_digest);
    progress
        .answered_questions
        .insert(question.id.clone(), question.revision);
    progress
        .question_bindings
        .insert(question.id.clone(), (&question).into());
    persist(store, progress)?;
    Ok(true)
}

pub(super) fn describe(
    arguments: &Arguments,
    config: &user_setup::UserSetupConfig,
    result: &mut crate::ActionResult,
) -> Result<(), SetupError> {
    if let Some(preview) = preview(
        &arguments.user_root,
        &arguments.root_cap,
        config.wiki.enabled,
    )? {
        let sources: Vec<_> = preview
            .sources
            .iter()
            .map(|s| {
                json!({
                    "collection_id":s.collection_id,"aliases":s.aliases,"path":s.path
                })
            })
            .collect();
        let data = result.data.get_or_insert_with(|| json!({}));
        data["index_rebuild"] = json!({"approval_required":true,"scope_digest":preview.scope_digest,
            "includes_user_root":true,"sources":sources,"approval_option":"--index-rebuild-scope"});
    }
    Ok(())
}

pub(super) fn bind_file(
    arguments: &Arguments,
    progress: &mut SetupProgress,
    requested: Option<&str>,
) -> Result<(), SetupError> {
    let Some(preview) = preview(
        &arguments.user_root,
        &arguments.root_cap,
        progress.answers["wiki"]["enabled"] == true,
    )?
    else {
        if let Some(digest) = requested {
            if progress.answers["wiki"]["enabled"] != true {
                return Err(SetupError::Input(
                    "disabled knowledge cannot approve an index rebuild".into(),
                ));
            }
            open(&arguments.user_root, &arguments.root_cap)?
                .validate_setup_index_scope(digest)
                .map_err(|error| SetupError::Conflict(error.to_string()))?;
            progress.index_rebuild_scope = Some(digest.to_owned());
            return Ok(());
        }
        progress.index_rebuild_scope = None;
        return Ok(());
    };
    let approval = requested.or(progress.index_rebuild_scope.as_deref());
    if approval != Some(preview.scope_digest.as_str()) {
        return Err(SetupError::Input("index rebuild requires the scope shown by --dry-run; pass --index-rebuild-scope <digest> with --apply, or use interactive setup".into()));
    }
    progress.index_rebuild_scope = Some(preview.scope_digest);
    Ok(())
}

pub(super) fn apply(arguments: &Arguments, progress: &SetupProgress) -> Result<(), SetupError> {
    if progress.answers["wiki"]["enabled"] != true {
        return Ok(());
    }
    let store = open(&arguments.user_root, &arguments.root_cap)?;
    if let Some(digest) = &progress.index_rebuild_scope {
        store
            .rebuild_setup_index(digest)
            .map_err(|error| SetupError::Conflict(error.to_string()))?;
    } else if store
        .preview_setup_index_rebuild()
        .map_err(|error| SetupError::Conflict(error.to_string()))?
        .is_some()
    {
        return Err(SetupError::Conflict(
            "index rebuild has no approved source scope".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::{InterfaceLanguage, SetupMode};
    use hive_wiki::shared::{
        KnowledgeLanguage, KnowledgeVisibility, RegisteredProject, SHARED_INDEX_RELATIVE,
    };

    fn fixture() -> (tempfile::TempDir, tempfile::TempDir, Arguments) {
        let root = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let wiki = RagStore::open(root.path()).unwrap();
        wiki.ensure_registry().unwrap();
        wiki.register_project_atomic(RegisteredProject {
            id: "selected-project".into(),
            root: project.path().to_owned(),
            enabled: true,
            language: KnowledgeLanguage::En,
            visibility: KnowledgeVisibility::Shared,
        })
        .unwrap();
        std::fs::remove_file(root.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        let (user_root, root_cap) =
            crate::user_install::open_canonical_user_root_for_setup(root.path()).unwrap();
        let args = Arguments {
            answers: "saved:test".into(),
            mode: SetupMode::Apply,
            user_root,
            root_cap,
        };
        (root, project, args)
    }

    fn conversation(args: &Arguments) -> (ProgressStore, Conversation) {
        let store = ProgressStore::open(&args.user_root, None).unwrap();
        let record = store
            .new_progress(
                "test".into(),
                super::super::default_answers(Some(InterfaceLanguage::Ko)),
            )
            .unwrap();
        let snapshot = store.load().unwrap();
        (
            store,
            Conversation {
                record,
                snapshot,
                fresh: false,
            },
        )
    }

    #[test]
    fn index_consent_preserves_declined_and_interrupted_drafts_and_reuses_exact_approval() {
        let (root, _project, args) = fixture();
        let (store, mut progress) = conversation(&args);
        for input in ["", "아니요\n"] {
            let mut terminal = Terminal {
                reader: input.as_bytes(),
                writer: Vec::new(),
                language: InterfaceLanguage::Ko,
            };
            assert!(!collect(
                &args.user_root,
                &args.root_cap,
                &store,
                &mut progress,
                &mut terminal
            )
            .unwrap());
            assert!(progress.index_rebuild_scope.is_none());
            assert_eq!(
                store.load().unwrap().progress.unwrap().answers,
                progress.answers
            );
            assert!(!root.path().join(SHARED_INDEX_RELATIVE).exists());
        }
        let mut terminal = Terminal {
            reader: "예\n".as_bytes(),
            writer: Vec::new(),
            language: InterfaceLanguage::Ko,
        };
        assert!(collect(
            &args.user_root,
            &args.root_cap,
            &store,
            &mut progress,
            &mut terminal
        )
        .unwrap());
        assert!(progress.index_rebuild_scope.is_some());
        assert!(!root.path().join(SHARED_INDEX_RELATIVE).exists());
        let mut resumed = Terminal {
            reader: "".as_bytes(),
            writer: Vec::new(),
            language: InterfaceLanguage::Ko,
        };
        assert!(collect(
            &args.user_root,
            &args.root_cap,
            &store,
            &mut progress,
            &mut resumed
        )
        .unwrap());
        assert!(resumed.writer.is_empty());
        apply(&args, &progress).unwrap();
        assert!(root.path().join(SHARED_INDEX_RELATIVE).is_file());
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
        let digest = progress.index_rebuild_scope.clone().unwrap();
        let original = std::fs::read(root.path().join(SHARED_INDEX_RELATIVE)).unwrap();
        bind_file(&args, &mut progress, Some(&digest)).unwrap();
        apply(&args, &progress).unwrap();
        assert_eq!(
            std::fs::read(root.path().join(SHARED_INDEX_RELATIVE)).unwrap(),
            original
        );
    }

    #[test]
    fn index_file_application_requires_exact_scope_and_preserves_changes_after_approval() {
        let (root, _project, args) = fixture();
        let (_, mut progress) = conversation(&args);
        assert!(bind_file(&args, &mut progress, None).is_err());
        assert!(apply(&args, &progress).is_err());
        let preview = preview(&args.user_root, &args.root_cap, true)
            .unwrap()
            .unwrap();
        bind_file(&args, &mut progress, Some(&preview.scope_digest)).unwrap();
        let registry = root
            .path()
            .join(hive_wiki::shared::PROJECT_REGISTRY_RELATIVE);
        let mut changed = std::fs::read(&registry).unwrap();
        changed.extend_from_slice(b"\n# changed\n");
        std::fs::write(&registry, &changed).unwrap();
        assert!(apply(&args, &progress).is_err());
        assert_eq!(std::fs::read(registry).unwrap(), changed);
        assert!(!root.path().join(SHARED_INDEX_RELATIVE).exists());
    }
}
