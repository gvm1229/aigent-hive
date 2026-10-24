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
fn optional_features_require_confirmation_and_resume_without_repeating_review() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    std::fs::create_dir(target.path().join("vendor")).unwrap();
    std::fs::write(
        target.path().join("vendor/SKILL.md"),
        b"---\nname: local-check\ndescription: Read local fixture.\n---\n\n# Local check\n",
    )
    .unwrap();
    let mut options = options(root.path(), target.path());
    options.capabilities = Some(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/setup/capabilities-codex-host-native-hooks.json"),
    );
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut answers = questions::defaults(target.path(), &global, "codex");
    answers["setup_mode"] = json!("custom");
    questions::seed_overrides(&mut answers, &global);
    answers["project_identity"] = answers["project_name"].clone();
    let mut progress = store
        .new_progress("project-extras".into(), answers)
        .unwrap();
    let draft = json!({"skills":[{"name":"local-check","source":"path:vendor/SKILL.md","requested_capabilities":["filesystem-read"],"approved_capabilities":["filesystem-read"]}],
        "hooks":[{"capability":"checkpoint-reminder","event":"Stop"}]});
    let mut first = Terminal {
        reader: Cursor::new(format!("yes\n{draft}\nno\n")),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        !extras::configure(&options, &store, &mut snapshot, &mut progress, &mut first).unwrap()
    );
    assert_eq!(progress.answers["approved_optional_skills"], json!([]));
    assert!(progress.extras_draft.is_some());
    assert!(!target.path().join(".hive").exists());
    let mut second = Terminal {
        reader: Cursor::new(b"\nyes\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        extras::configure(&options, &store, &mut snapshot, &mut progress, &mut second).unwrap()
    );
    assert!(!String::from_utf8(second.writer)
        .unwrap()
        .contains("Review optional features"));
    assert!(progress.extras_draft.is_none());
    assert_extra_review_binding(&options, &store, &mut snapshot, &mut progress);
    assert_eq!(
        progress.answers["approved_optional_skills"][0]["name"],
        "local-check"
    );
    assert_eq!(
        progress.answers["preference_decisions"]["/approved_optional_skills"]["origin"],
        "explicit"
    );
    let mut apply_terminal = Terminal {
        reader: Cursor::new(b"yes\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert_eq!(
        apply(
            &options,
            &store,
            &mut snapshot,
            &mut progress,
            &global,
            &mut apply_terminal
        )
        .unwrap()
        .exit_code,
        0
    );
    assert!(target
        .path()
        .join(".agents/skills/local-check/SKILL.md")
        .is_file());
    assert!(target
        .path()
        .join(".hive/hooks/checkpoint-reminder")
        .is_file());
}

fn assert_extra_review_binding(
    options: &Options,
    store: &ProgressStore,
    snapshot: &mut user_setup::progress_store::ProgressSnapshot,
    progress: &mut user_setup::progress_store::SetupProgress,
) {
    let mut no_input = Terminal {
        reader: Cursor::new(b""),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(extras::configure(options, store, snapshot, progress, &mut no_input).unwrap());
    assert!(no_input.writer.is_empty());
    let original = progress.clone();
    progress.answers["approved_optional_skills"] = json!([]);
    assert!(!extras::configure(options, store, snapshot, progress, &mut no_input).unwrap());
    assert!(String::from_utf8(no_input.writer)
        .unwrap()
        .contains("Review optional features"));
    *progress = original;
    *snapshot = store.save(snapshot, progress).unwrap();
}

#[test]
fn optional_source_change_after_preview_preserves_unapproved_answers() {
    struct EditAtConsent {
        path: PathBuf,
        text: String,
        edited: bool,
    }
    impl Write for EditAtConsent {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.text.push_str(&String::from_utf8_lossy(bytes));
            if !self.edited && self.text.contains("Approve the displayed sources") {
                std::fs::write(
                    &self.path,
                    b"---\nname: local-check\ndescription: Changed.\n---\nChanged source\n",
                )?;
                self.edited = true;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let path = target.path().join("SKILL.md");
    std::fs::write(
        &path,
        b"---\nname: local-check\ndescription: Original.\n---\nOriginal source\n",
    )
    .unwrap();
    let mut options = options(root.path(), target.path());
    options.capabilities = Some(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/setup/capabilities-codex-host-native.json"),
    );
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut answers = questions::defaults(target.path(), &global, "codex");
    answers["setup_mode"] = json!("custom");
    questions::seed_overrides(&mut answers, &global);
    let mut progress = store
        .new_progress("project-extras".into(), answers)
        .unwrap();
    let draft = json!({"skills":[{"name":"local-check","source":"path:SKILL.md","requested_capabilities":[],"approved_capabilities":[]}],"hooks":[]});
    let mut terminal = Terminal {
        reader: Cursor::new(format!("yes\n{draft}\nyes\n")),
        writer: EditAtConsent {
            path,
            text: String::new(),
            edited: false,
        },
        language: InterfaceLanguage::En,
    };
    let result = extras::configure(
        &options,
        &store,
        &mut snapshot,
        &mut progress,
        &mut terminal,
    );
    assert!(terminal.writer.edited);
    assert!(
        matches!(result, Err(SetupError::Conflict(message)) if message.contains("source changed"))
    );
    assert_eq!(progress.answers["approved_optional_skills"], json!([]));
    assert!(progress.extras_approval_digest.is_none());
    assert!(progress.extras_draft.is_some());
    assert!(!target.path().join(".hive").exists());
}

#[test]
fn declining_optional_review_keeps_choices_and_eof_does_not_answer() {
    for bytes in [b"".as_slice(), b"no\n".as_slice()] {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let global = global(root.path());
        let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
        let mut snapshot = store.load().unwrap();
        let mut answers = questions::defaults(target.path(), &global, "codex");
        answers["setup_mode"] = json!("custom");
        questions::seed_overrides(&mut answers, &global);
        let mut progress = store
            .new_progress("project-extras".into(), answers.clone())
            .unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new(bytes),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert_eq!(
            extras::configure(
                &options(root.path(), target.path()),
                &store,
                &mut snapshot,
                &mut progress,
                &mut terminal
            )
            .unwrap(),
            !bytes.is_empty()
        );
        assert_eq!(progress.answers, answers);
        assert_eq!(
            progress.answered_questions.contains_key("project-extras"),
            !bytes.is_empty()
        );
        assert!(!target.path().join(".hive").exists());
    }
}

#[test]
fn custom_inputs_retry_only_invalid_context_and_bind_sorted_sharing_choices() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let mut global = global(root.path());
    global.wiki_enabled = false;
    global.usage_guard_enabled = true;
    global.usage_stop_remaining_percent = 45;
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut progress = store
        .new_progress(
            "project-mode".into(),
            questions::defaults(target.path(), &global, "codex"),
        )
        .unwrap();
    let mut terminal = Terminal {
        reader: Cursor::new(b"2\n2\nyes\nno\n\n\n\n\n\n0\n45\n\n../outside\ndocs\n\n3,1\n1\n2\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap(),
        "{}",
        String::from_utf8_lossy(&terminal.writer)
    );
    let output = String::from_utf8(terminal.writer).unwrap();
    assert!(output.contains("Enable global knowledge storage"));
    assert!(output.contains("below the global threshold"));
    assert!(output.contains("relative to this project"));
    assert!(output.contains("cannot also be confidential"));
    assert_eq!(output.matches("Enter the project name.").count(), 1);
    assert_eq!(
        progress.answers["root_knowledge_promotion_categories"],
        json!(["fact", "workflow"])
    );
    assert_eq!(
        progress.answers["confidential_knowledge_categories"],
        json!(["preference"])
    );
    let mut binding = Terminal {
        reader: Cursor::new(b""),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(existing::bind_knowledge(
        &options(root.path(), target.path()),
        &store,
        &mut snapshot,
        &mut progress,
        &mut binding
    )
    .unwrap());
    assert!(binding.writer.is_empty());
    assert!(progress.answers["user_store_binding"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    validate_answers(&progress.answers);
}

#[test]
fn changing_an_existing_sharing_destination_requires_an_answer_and_keeps_the_old_binding_on_no() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut answers = questions::defaults(target.path(), &global, "codex");
    answers["root_knowledge_promotion_categories"] = json!(["fact"]);
    let old = hive_core::sha256_digest(b"previous user store");
    answers["user_store_binding"] = json!(old);
    let mut progress = store
        .new_progress("project-knowledge-promotion".into(), answers)
        .unwrap();
    let options = options(root.path(), target.path());
    for input in [b"".as_slice(), b"no\n".as_slice()] {
        let mut terminal = Terminal {
            reader: Cursor::new(input),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(!existing::bind_knowledge(
            &options,
            &store,
            &mut snapshot,
            &mut progress,
            &mut terminal
        )
        .unwrap());
        assert_eq!(progress.answers["user_store_binding"], old);
    }
    let mut terminal = Terminal {
        reader: Cursor::new(b"yes\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(existing::bind_knowledge(
        &options,
        &store,
        &mut snapshot,
        &mut progress,
        &mut terminal
    )
    .unwrap());
    assert_ne!(progress.answers["user_store_binding"], old);
    assert!(!target.path().join(".hive").exists());
    progress.answers["user_store_binding"] = json!(old);
    let mut no = Terminal {
        reader: Cursor::new(b"no\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        !existing::bind_knowledge(&options, &store, &mut snapshot, &mut progress, &mut no).unwrap()
    );
    let mut revise = Terminal {
        reader: Cursor::new(b"\n\n\n\n[]\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut revise).unwrap()
    );
    assert_eq!(
        progress.answers["root_knowledge_promotion_categories"],
        json!([])
    );
}

#[test]
fn existing_project_answers_keep_legacy_values_without_inventing_original_choice_times() {
    let bytes = include_bytes!("../../../../../../tests/fixtures/setup/answers-base.yml");
    let original: Value = serde_yaml::from_slice(bytes).unwrap();
    let normalized = existing::answers(bytes).unwrap();
    assert_eq!(normalized["schema_version"], 2);
    assert_eq!(normalized["persistent_roles"], original["persistent_roles"]);
    assert_eq!(
        normalized["knowledge_include_paths"],
        original["knowledge_include_paths"]
    );
    assert_eq!(
        normalized["preference_decisions"]["/persistent_roles"]["origin"],
        "legacy-unknown"
    );
    assert_eq!(
        normalized["preference_decisions"]["/persistent_roles"]["changed_in_version"],
        "unknown"
    );
    assert!(normalized["preference_decisions"]["/persistent_roles"]["decided_at"].is_null());
    assert_eq!(
        normalized["preference_decisions"]["/root_knowledge_promotion_categories"]["origin"],
        "default"
    );
    validate_answers(&normalized);
}

#[test]
fn choosing_custom_preferences_replaces_the_previous_quick_inheritance_records() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut answers = questions::defaults(target.path(), &global, "codex");
    preferences::record_defaults(&mut answers).unwrap();
    let mut progress = store.new_progress("project-mode".into(), answers).unwrap();
    let mut terminal = Terminal {
        reader: Cursor::new(b"2\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap()
    );
    assert_eq!(progress.answers["setup_mode"], "custom");
    assert!(progress.answers["preference_decisions"]
        .as_object()
        .unwrap()
        .values()
        .all(|record| record["origin"] != "inherited"));
    assert_eq!(
        progress.answers["preference_decisions"]["/wiki/enabled"]["input_channel"],
        "terminal"
    );
    validate_answers(&progress.answers);
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
fn custom_project_resume_and_inspection_reuse_its_language_instead_of_the_global_language() {
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
        reader: Cursor::new(b"2\n2\n:cancel\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        !questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap()
    );
    let pending_inspection = preferences::inspect(
        &options(root.path(), target.path()),
        &store,
        None,
        InterfaceLanguage::Ko,
    )
    .unwrap();
    assert_eq!(pending_inspection.message, "Project setup inspection");
    let mut resumed_options = options(root.path(), target.path());
    resumed_options.resume = true;
    let pending_message =
        resume_without_input(&resumed_options, &store, InterfaceLanguage::Ko).unwrap();
    assert!(pending_message
        .message
        .starts_with("Project setup is pending"));
    let mut snapshot = store.load().unwrap();
    let mut progress = snapshot.progress.clone().unwrap();
    let mut terminal = Terminal {
        reader: Cursor::new(b"\n\n\n\n\n\n\n\n\n\n\n\n\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::Ko,
    };
    assert!(
        questions::collect(&store, &mut snapshot, &mut progress, &global, &mut terminal).unwrap()
    );
    assert_eq!(terminal.language, InterfaceLanguage::En);
    let output = String::from_utf8(terminal.writer).unwrap();
    assert!(output.contains("Save knowledge in local documents?"));
    assert!(!output.contains("로컬 문서"));
    assert!(!output.contains("Choose a language"));
    let bytes = serde_json::to_vec(&progress.answers).unwrap();
    let mut options = options(root.path(), target.path());
    options.show = true;
    let shown =
        preferences::inspect(&options, &store, Some(&bytes), InterfaceLanguage::Ko).unwrap();
    assert_eq!(shown.message, "Project setup inspection");
    assert_eq!(
        questions::language(
            &json!({"setup_mode":"expedited","interface_language":"en"}),
            InterfaceLanguage::Ko
        ),
        InterfaceLanguage::Ko
    );
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

#[test]
fn existing_role_changes_require_a_saved_confirmation_and_preserve_the_user_body() {
    let root = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let global = global(root.path());
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/setup");
    let original = fixtures.join("answers-base.yml");
    let caps = fixtures.join("capabilities-codex-host-native.json");
    let initial = hive_render::SetupRequest {
        target: target.path(),
        answers: &original,
        capabilities: &caps,
        mode: hive_render::SetupMode::Apply,
        reconfigure_roles: BTreeSet::new(),
        global_preferences: Some(global.clone()),
    };
    hive_render::execute_setup(&initial).unwrap();
    let role_path = target.path().join(".hive/team/roles/reviewer.md");
    let mut role = std::fs::read(&role_path).unwrap();
    role.extend_from_slice(b"\nUser handoff notes.\r\n");
    std::fs::write(&role_path, &role).unwrap();
    let mut answers =
        existing::answers(&std::fs::read(target.path().join(".hive/setup-answers.yml")).unwrap())
            .unwrap();
    answers["persistent_roles"][0]["responsibilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("Review CLI settings"));
    let store = ProgressStore::open(root.path(), Some(target.path())).unwrap();
    let mut snapshot = store.load().unwrap();
    let mut progress = store.new_progress("project-roles".into(), answers).unwrap();
    let mut options = options(root.path(), target.path());
    let mut no = Terminal {
        reader: Cursor::new(b"no\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        !existing::confirm_roles(&options, &store, &mut snapshot, &mut progress, &mut no).unwrap()
    );
    assert!(progress.reconfigure_roles.is_empty());
    assert_eq!(std::fs::read(&role_path).unwrap(), role);
    let mut yes = Terminal {
        reader: Cursor::new(b"yes\n"),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        existing::confirm_roles(&options, &store, &mut snapshot, &mut progress, &mut yes).unwrap()
    );
    assert_eq!(
        progress.reconfigure_roles,
        BTreeSet::from(["reviewer".into()])
    );
    let mut no_input = Terminal {
        reader: Cursor::new(b""),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(existing::confirm_roles(
        &options,
        &store,
        &mut snapshot,
        &mut progress,
        &mut no_input
    )
    .unwrap());
    assert!(no_input.writer.is_empty());
    assert_changed_role_needs_approval(&options, &store, &mut snapshot, &progress);
    options.reconfigure_roles = progress.reconfigure_roles.clone();
    let input = write_answers(&progress.answers).unwrap();
    let prepared = prepare_settings(&options, input.path(), &caps, &global).unwrap();
    activation::bind_context(
        &mut progress,
        &global,
        &std::fs::read(&caps).unwrap(),
        &prepared.outcome,
    )
    .unwrap();
    progress.phase = ProgressPhase::Applying;
    store.save(&snapshot, &progress).unwrap();
    store.record_applying(&progress).unwrap();
    activation::apply(&options, &store, &progress, &global, Some(prepared)).unwrap();
    assert!(std::fs::read(&role_path)
        .unwrap()
        .ends_with(b"\nUser handoff notes.\r\n"));
    ProgressStore::open(root.path(), Some(target.path()))
        .unwrap()
        .record_completion(&progress)
        .unwrap();
}

fn assert_changed_role_needs_approval(
    options: &Options,
    store: &ProgressStore,
    snapshot: &mut user_setup::progress_store::ProgressSnapshot,
    original: &user_setup::progress_store::SetupProgress,
) {
    let mut changed = original.clone();
    changed.answers["persistent_roles"][0]["display_name"] = json!("Changed after approval");
    let mut terminal = Terminal {
        reader: Cursor::new(b""),
        writer: Vec::new(),
        language: InterfaceLanguage::En,
    };
    assert!(
        !existing::confirm_roles(options, store, snapshot, &mut changed, &mut terminal).unwrap()
    );
    assert!(String::from_utf8(terminal.writer)
        .unwrap()
        .contains("Approve definition changes"));
    *snapshot = store.save(snapshot, original).unwrap();
}
