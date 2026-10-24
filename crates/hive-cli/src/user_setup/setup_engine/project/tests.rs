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

#[test]
fn inherited_fingerprints_follow_the_declaration_and_show_the_current_parent() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let mut answers = questions::defaults(target.path(), &global, "codex");
    answers["project_identity"] = json!("test");
    preferences::record_defaults(&mut answers).unwrap();
    validate_answers(&answers);
    let bytes = serde_json::to_vec(&answers).unwrap();
    let path = target.path().join(".hive/setup-answers.yml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let mut options = options(root.path(), target.path());
    options.show = true;
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let first = preferences::inspect(&options, &store, Some(&bytes), InterfaceLanguage::Ko)
        .unwrap()
        .data
        .unwrap();
    assert_eq!(first["effective"]["interface_language"], "ko");
    let parent_path = root.path().join(user_setup::USER_SETUP_RELATIVE);
    let mut parent: Value = serde_json::from_slice(&std::fs::read(&parent_path).unwrap()).unwrap();
    parent["interface_language"] = json!("en");
    parent["usage_guard"]["stop_remaining_percent"] = json!(45);
    std::fs::write(&parent_path, serde_json::to_vec(&parent).unwrap()).unwrap();
    let second = preferences::inspect(&options, &store, Some(&bytes), InterfaceLanguage::En)
        .unwrap()
        .data
        .unwrap();
    assert_eq!(second["effective"]["interface_language"], "en");
    assert_eq!(second["effective"]["usage_stop_remaining_percent"], 45);
    assert_eq!(second["stored"]["usage_stop_remaining_percent"], 20);
    assert_eq!(first["sources"], second["sources"]);
    assert_eq!(
        second["sources"]["/interface_language"]["origin"],
        "inherited"
    );
    assert_ne!(
        first["parent"]["settings_digest"],
        second["parent"]["settings_digest"]
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert!(!root.path().join(".hive/config/setup-progress").exists());
    options.describe = true;
    options.section = Some("language".into());
    let described = preferences::inspect(&options, &store, Some(&bytes), InterfaceLanguage::En)
        .unwrap()
        .data
        .unwrap();
    assert_eq!(described["questions"][0]["source"], "inherited");
    assert_eq!(described["questions"][0]["effective"], "en");
}

#[test]
fn preview_acceptance_distinguishes_visible_defaults_and_external_edits() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let mut answers = questions::defaults(target.path(), &global, "codex");
    answers["project_identity"] = json!("test");
    preferences::record_defaults(&mut answers).unwrap();
    assert_eq!(
        answers["preference_decisions"]["/persistent_roles"]["origin"],
        "default"
    );
    assert!(answers["preference_decisions"]["/persistent_roles"]["decided_at"].is_null());
    preferences::accept_preview_defaults(&mut answers).unwrap();
    assert_eq!(
        answers["preference_decisions"]["/persistent_roles"]["origin"],
        "accepted-default"
    );
    assert_eq!(
        answers["preference_decisions"]["/elevated_judge_quorum"]["origin"],
        "default"
    );
    assert_eq!(
        answers["preference_decisions"]["/interface_language"]["origin"],
        "inherited"
    );
    answers["project_name"] = json!("manual change");
    let (records, _) =
        hive_render::inspect_project_preferences(&serde_json::to_vec(&answers).unwrap(), &global)
            .unwrap();
    assert_eq!(
        records["/project_name"].origin,
        hive_core::preferences::DecisionOrigin::ExternalChange
    );
    assert!(records["/project_name"].decided_at.is_none());
    assert_eq!(
        answers["preference_decisions"]["/project_name"]["origin"],
        "accepted-default"
    );
}

#[test]
fn changed_global_preferences_after_preview_do_not_apply_the_project() {
    struct EditOnConfirmation {
        path: PathBuf,
        output: Vec<u8>,
        changed: bool,
    }
    impl Write for EditOnConfirmation {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.output.extend_from_slice(bytes);
            if !self.changed
                && String::from_utf8_lossy(&self.output).contains("이 변경을 적용할까요?")
            {
                let mut value: Value = serde_json::from_slice(&std::fs::read(&self.path)?)?;
                value["interface_language"] = json!("en");
                std::fs::write(&self.path, serde_json::to_vec(&value)?)?;
                self.changed = true;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    global(root.path());
    let caps = root.path().join("capabilities.json");
    std::fs::write(
        &caps,
        serde_json::to_vec(&capabilities::unverified("codex", "unverified").unwrap()).unwrap(),
    )
    .unwrap();
    let mut options = options(root.path(), target.path());
    options.capabilities = Some(caps);
    let mut terminal = Terminal {
        reader: Cursor::new(b"\n\n\n\nyes\n"),
        writer: EditOnConfirmation {
            path: root.path().join(user_setup::USER_SETUP_RELATIVE),
            output: Vec::new(),
            changed: false,
        },
        language: InterfaceLanguage::En,
    };
    let result = execute(&options, InteractionMode::Terminal, &mut terminal);
    assert!(
        matches!(result,Err(SetupError::Conflict(message)) if message.contains("global settings changed after the preview"))
    );
    assert!(terminal.writer.changed);
    assert!(!target.path().join(".hive").exists());
    let progress = ProgressStore::open(root.path(), Some(target.path()))
        .unwrap()
        .load()
        .unwrap()
        .progress
        .unwrap();
    assert_eq!(progress.phase, ProgressPhase::Previewed);
    assert_eq!(
        progress.answers["preference_decisions"]["/persistent_roles"]["origin"],
        "default"
    );
}

#[test]
fn confirmed_project_recovery_rejects_global_drift_and_never_trusts_a_missing_payload_alone() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let options = options(root.path(), target.path());
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut answers = questions::defaults(target.path(), &global, "codex");
    answers["project_identity"] = answers["project_name"].clone();
    preferences::record_defaults(&mut answers).unwrap();
    preferences::accept_preview_defaults(&mut answers).unwrap();
    let input = write_answers(&answers).unwrap();
    let matrix = capabilities::unverified("codex", "unverified").unwrap();
    let caps = write_answers(&matrix).unwrap();
    let prepared = hive_render::prepare_setup(&request(
        &options,
        input.path(),
        caps.path(),
        &global,
        hive_render::SetupMode::DryRun,
    ))
    .unwrap();
    let mut progress = store.new_progress("project-mode".into(), answers).unwrap();
    activation::bind_context(
        &mut progress,
        &global,
        &serde_json::to_vec(&matrix).unwrap(),
        &prepared.outcome,
    )
    .unwrap();
    progress.phase = ProgressPhase::Applying;
    store.save(&store.load().unwrap(), &progress).unwrap();
    store.record_applying(&progress).unwrap();
    let mut changed = global.clone();
    changed.interface_language = "en".into();
    assert!(activation::apply(&options, &store, &progress, &changed, None).is_err());
    assert!(!target.path().join(".hive").exists());
    activation::apply(&options, &store, &progress, &global, Some(prepared)).unwrap();
    let user = crate::user_install::open_user_root_for_setup(root.path()).unwrap();
    let digest = store.projection_binding(&progress, None).unwrap();
    crate::user_setup::projection_journal::cleanup(&user, &progress, digest.as_deref()).unwrap();
    let path = target.path().join(".hive/config/harness.toml");
    std::fs::write(&path, b"user changed the harness").unwrap();
    assert!(activation::apply(&options, &store, &progress, &global, None).is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"user changed the harness");
    let pending = ProgressStore::open(root.path(), Some(target.path()))
        .unwrap()
        .load()
        .unwrap()
        .progress
        .unwrap();
    assert_eq!(pending.phase, ProgressPhase::Applying);
    let mut tampered = progress;
    tampered.project_context.as_mut().unwrap().plan_digest =
        hive_core::sha256_digest(b"different plan");
    assert!(store.projection_binding(&tampered, None).is_err());
}
