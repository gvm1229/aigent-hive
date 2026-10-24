use super::*;
use std::io::Cursor;

fn global(root: &Path) -> hive_render::GlobalProjectPreferences {
    let mut answers = super::super::default_answers(None);
    answers["interface_language"] = json!("ko");
    let path = root.join(user_setup::USER_SETUP_RELATIVE);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec(&answers).unwrap()).unwrap();
    global_preferences(root).unwrap()
}

fn options(root: &Path, target: &Path) -> Options {
    parse(&[
        "--user-root".into(),
        root.display().to_string(),
        "--target".into(),
        target.display().to_string(),
    ])
    .unwrap()
}

#[test]
fn conflicting_modes_and_missing_target_fail_before_input() {
    let root = tempfile::tempdir().unwrap();
    let base = ["--user-root".into(), root.path().display().to_string()];
    assert!(parse(&base).is_err());
    for extra in [
        vec!["--interactive", "--output", "json"],
        vec!["--interactive", "--non-interactive"],
        vec!["--show", "--resume"],
        vec!["--section", "identity"],
        vec!["--target", "duplicate"],
    ] {
        let mut arguments = base.to_vec();
        arguments.extend(["--target".into(), root.path().display().to_string()]);
        arguments.extend(extra.into_iter().map(str::to_owned));
        assert!(parse(&arguments).is_err(), "{arguments:?}");
    }
    assert!(!root.path().join(".hive").exists());
}

#[test]
fn noninteractive_new_project_never_reads_or_creates_settings() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let mut terminal = Terminal {
        reader: Cursor::new(b"yes\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    let result = execute(
        &options(root.path(), target.path()),
        InteractionMode::NonInteractive,
        &mut terminal,
    )
    .unwrap();
    assert_eq!(result.status, "needs-input");
    assert_eq!(terminal.reader.position(), 0);
    assert!(terminal.writer.is_empty());
    assert!(!root.path().join(".hive").exists());
    assert!(!target.path().join(".hive").exists());
}

#[test]
fn quick_project_inherits_preferences_and_does_not_invent_roles_or_knowledge() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut progress = store
        .new_progress(
            "project-mode".into(),
            questions::defaults(target.path(), &global, "codex"),
        )
        .unwrap();
    let mut terminal = Terminal {
        reader: Cursor::new(b"\nMy project\n\n\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap()
    );
    let answers = &progress.answers;
    for key in ["interface_language", "wiki", "persona", "skills"] {
        assert!(answers.get(key).is_none(), "{key}");
    }
    for key in [
        "persistent_roles",
        "knowledge_include_paths",
        "approved_optional_skills",
        "approved_fallback_hooks",
        "root_knowledge_promotion_categories",
    ] {
        assert_eq!(answers[key], json!([]), "{key}");
    }
    assert_eq!(answers["project_identity"], "My project");
    assert_eq!(progress.answered_questions.len(), 4);
    assert!(!target.path().join(".hive").exists());
    validate_answers(answers);
}

fn validate_answers(answers: &Value) {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../../schemas/setup-answers.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors = validator
        .iter_errors(answers)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn custom_project_collects_explicit_preferences_and_repeats_only_invalid_answer() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut progress = store
        .new_progress(
            "project-mode".into(),
            questions::defaults(target.path(), &global, "codex"),
        )
        .unwrap();
    // Mode, language, wiki, wiki language, persona, features, name, kind, host,
    // threshold (invalid then valid), roles, include, exclude, promotion, confidential.
    let mut terminal = Terminal {
        reader: Cursor::new(b"2\n2\n\n\n\n\nNamed\n\n\n100\n25\n[]\ndocs\n\n\n\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    let collected =
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap();
    assert!(collected, "{}", String::from_utf8_lossy(&terminal.writer));
    assert_eq!(terminal.language, InterfaceLanguage::En);
    assert_eq!(progress.answers["interface_language"], "en");
    assert_eq!(progress.answers["usage_stop_remaining_percent"], 25);
    assert_eq!(progress.answers["knowledge_include_paths"], json!(["docs"]));
    assert!(String::from_utf8(terminal.writer)
        .unwrap()
        .contains("Check the answer."));
    validate_answers(&progress.answers);
}

#[test]
fn interrupted_project_resumes_without_repeating_collected_answers() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut progress = store
        .new_progress(
            "project-mode".into(),
            questions::defaults(target.path(), &global, "codex"),
        )
        .unwrap();
    let mut terminal = Terminal {
        reader: Cursor::new(b"\nNamed\n:cancel\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        !questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap()
    );
    let mut snapshot = store.load().unwrap();
    let mut progress = snapshot.progress.clone().unwrap();
    assert!(!progress.answered_questions.contains_key("project-kind"));
    let mut terminal = Terminal {
        reader: Cursor::new(b"\n\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap()
    );
    let output = String::from_utf8(terminal.writer).unwrap();
    assert!(!output.contains("프로젝트 이름"));
    assert!(!output.contains("설정 방식을"));
    assert_eq!(progress.answers["project_name"], "Named");
    validate_answers(&progress.answers);
}
