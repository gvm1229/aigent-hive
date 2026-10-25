use super::{catalog, terminal::Terminal};
use crate::user_setup::{
    self,
    progress_store::{ProgressPhase, ProgressSnapshot, ProgressStore, SetupProgress},
    InterfaceLanguage, SetupError,
};
use hive_core::preferences::{
    DecisionOrigin, PreferenceDecision, PreferenceInput, PreferenceScope,
};
use hive_core::setup::{AnswerInput, InteractionMode, InteractionRequest, Question};
use serde_json::{json, Value};
use std::io::{self, BufRead, IsTerminal, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub(super) struct Conversation {
    pub(super) record: SetupProgress,
    pub(super) snapshot: ProgressSnapshot,
    pub(super) fresh: bool,
}
impl std::ops::Deref for Conversation {
    type Target = SetupProgress;
    fn deref(&self) -> &Self::Target {
        &self.record
    }
}
impl std::ops::DerefMut for Conversation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.record
    }
}

pub(in crate::user_setup) fn handles(arguments: &[String]) -> bool {
    !arguments
        .iter()
        .any(|arg| matches!(arg.as_str(), "--answers" | "--quick-answers"))
}

// Independent CLI switches; parse rejects conflicting combinations before any write.
#[allow(clippy::struct_excessive_bools)]
struct Options {
    root: PathBuf,
    interaction: InteractionRequest,
    resume: bool,
    review: bool,
    review_answers: Option<PathBuf>,
    show: bool,
    dry_run: bool,
    section: Option<String>,
    legacy_budget: Option<crate::update_bridge::LegacyBudget>,
    update_hosts: Vec<String>,
}

fn parse(arguments: &[String]) -> Result<Options, SetupError> {
    let mut options = Options {
        root: user_setup::resolve_user_root()?,
        interaction: InteractionRequest::default(),
        resume: false,
        review: false,
        review_answers: None,
        show: false,
        dry_run: false,
        section: None,
        legacy_budget: None,
        update_hosts: Vec::new(),
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut scope = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if !seen.insert(option) {
            return Err(SetupError::Input("duplicate setup option".to_owned()));
        }
        match option {
            "--interactive" => options.interaction.interactive = true,
            "--non-interactive" => options.interaction.non_interactive = true,
            "--resume" => options.resume = true,
            "--review" => options.review = true,
            "--show" => options.show = true,
            "--dry-run" => options.dry_run = true,
            "--scope" | "--user-root" | "--output" | "--section" | "--review-answers" => {
                let value = arguments
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| SetupError::Input("missing setup option value".to_owned()))?;
                match option {
                    "--scope" => scope = Some(value.as_str()),
                    "--user-root" => options.root = value.into(),
                    "--section" => options.section = Some(value.clone()),
                    "--review-answers" => options.review_answers = Some(value.into()),
                    _ if value == "json" => options.interaction.json = true,
                    _ => return Err(SetupError::Input("setup output must be json".to_owned())),
                }
                index += 1;
            }
            _ => {
                return Err(SetupError::Input(
                    "unknown interactive setup option".to_owned(),
                ))
            }
        }
        index += 1;
    }
    if scope != Some("user")
        || (options.show && (options.resume || options.dry_run || options.section.is_some()))
        || (options.resume && options.dry_run && !options.review)
        || (options.review && (!options.resume || options.show || options.section.is_some()))
        || (options.review_answers.is_some() && (!options.review || options.dry_run))
    {
        return Err(SetupError::Input(
            "choose user setup or a separate settings inspection".to_owned(),
        ));
    }
    options
        .interaction
        .resolve(false, false)
        .map_err(SetupError::Input)?;
    Ok(options)
}

pub(in crate::user_setup) fn run(arguments: &[String]) -> ExitCode {
    let json_output = arguments
        .windows(2)
        .any(|pair| pair == ["--output", "json"]);
    let mut terminal = Terminal {
        reader: io::stdin().lock(), writer: io::stderr().lock(),
        language: super::messages::language(arguments),
    };
    let result = parse(arguments)
        .and_then(|options| {
            let mode = options
                .interaction
                .resolve(io::stdin().is_terminal(), io::stderr().is_terminal())
                .map_err(SetupError::Input)?;
            execute(&options, mode, &mut terminal)
        })
        .unwrap_or_else(|error| super::messages::failure(&error, terminal.language));
    if json_output {
        return crate::emit_action_result(&result);
    }
    eprintln!("{}", result.message);
    if let Some(next) = &result.next_action {
        eprintln!("{next}");
    }
    ExitCode::from(result.exit_code)
}

pub(in crate::user_setup) fn resume_pending_update(
    root: &Path,
    interactive: bool,
) -> Result<Option<crate::ActionResult>, SetupError> {
    let store = ProgressStore::open(root, None)?;
    let Some(progress) = store
        .load()?
        .progress
        .filter(|progress| progress.phase != ProgressPhase::Complete)
    else {
        return Ok(None);
    };
    if !matches!(
        progress.phase,
        ProgressPhase::Applying | ProgressPhase::RecoverableFailure
    ) && progress.legacy_migration.is_none()
        && progress.base_config_digest != user_setup::progress_store::optional_digest(None)
    {
        // An unconfirmed personal reconfiguration must not block a software update.
        return Ok(None);
    }
    let mode = if interactive {
        InteractionMode::Terminal
    } else {
        InteractionMode::NonInteractive
    };
    let mut terminal = Terminal {
        reader: io::stdin().lock(),
        writer: io::stderr().lock(),
        language: InterfaceLanguage::En,
    };
    let mut result = super::update::complete(root, mode, &mut terminal)
        .unwrap_or_else(|error| super::messages::failure(&error, terminal.language));
    result.action = "UpdateHiveUser";
    if result.exit_code != 0 {
        result.next_action = Some(crate::update_bridge::resume_command(root));
    }
    Ok(Some(result))
}

pub(super) fn execute_update<R: BufRead, W: Write>(
    root: &Path,
    mode: InteractionMode,
    terminal: &mut Terminal<R, W>,
) -> Result<crate::ActionResult, SetupError> {
    let options = Options {
        root: root.to_owned(),
        interaction: InteractionRequest::default(),
        resume: true,
        review: false,
        review_answers: None,
        show: false,
        dry_run: false,
        section: None,
        legacy_budget: None,
        update_hosts: Vec::new(),
    };
    execute(&options, mode, terminal)
}

#[cfg(any(windows, target_os = "linux"))]
pub(in crate::user_setup) fn update_with_terminal<R: BufRead, W: Write>(
    root: &Path,
    reader: R,
    writer: W,
    budget: crate::update_bridge::LegacyBudget,
    hosts: &[String],
) -> crate::ActionResult {
    let options = Options {
        root: root.to_owned(),
        interaction: InteractionRequest::default(),
        resume: true,
        review: false,
        review_answers: None,
        show: false,
        dry_run: false,
        section: None,
        legacy_budget: Some(budget),
        update_hosts: hosts.to_vec(),
    };
    let mut terminal = Terminal {
        reader,
        writer,
        language: InterfaceLanguage::En,
    };
    let mut result = execute(&options, InteractionMode::Terminal, &mut terminal)
        .unwrap_or_else(|failure| super::messages::failure(&failure, terminal.language));
    if result.exit_code != 0 {
        result.next_action = Some(crate::update_bridge::resume_command(root));
    }
    let _ = writeln!(terminal.writer, "{}", result.message);
    if let Some(next) = &result.next_action {
        let _ = writeln!(terminal.writer, "{next}");
    }
    let _ = terminal.writer.flush();
    result
}

fn execute<R: BufRead, W: Write>(
    options: &Options,
    mode: InteractionMode,
    terminal: &mut Terminal<R, W>,
) -> Result<crate::ActionResult, SetupError> {
    let execution = if options.review {
        terminal.language = super::messages::language_at(&options.root, None);
        super::monitor_review::run(&options.root, mode, options.dry_run,
            options.review_answers.as_deref(), terminal)
    } else { execute_inner(options, mode, terminal) };
    let mut result = match execution {
        Err(error) if options.review => {
            let mut result = super::messages::failure(&error, terminal.language);
            result.next_action = Some("hive setup --scope user --resume".into());
            result
        }
        other => other?,
    };
    if result.next_action.as_deref() == Some("hive setup --scope user --resume") {
        let path = options.root.to_string_lossy();
        let quoted = if cfg!(windows) {
            path.replace('\'', "''")
        } else {
            path.replace('\'', "'\"'\"'")
        };
        result.next_action = Some(format!(
            "hive setup --scope user --resume --user-root '{quoted}'{}",
            if options.review { " --review" } else { "" }
        ));
    }
    Ok(result)
}

fn execute_inner<R: BufRead, W: Write>(
    options: &Options,
    mode: InteractionMode,
    terminal: &mut Terminal<R, W>,
) -> Result<crate::ActionResult, SetupError> {
    let (root_path, root_cap) =
        crate::user_install::open_canonical_user_root_for_setup(&options.root)
            .map_err(SetupError::Conflict)?;
    let existing = crate::user_install::read_user_setup_file(
        &root_cap,
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
    let store = ProgressStore::open(&root_path, None)?;
    if !options.show && !options.dry_run && mode!=InteractionMode::NeedsInput {
        store.recover_monitor_handoff()?;
    }
    if !options.show && mode != InteractionMode::NeedsInput {
        store.recover_pending_state()?;
    }
    if !store.matches_config(existing.as_deref()) {
        return Err(SetupError::Conflict(
            "settings changed before setup could start".to_owned(),
        ));
    }
    let mut snapshot = store.load()?;
    if let Some(progress) = snapshot.progress.as_ref().filter(|p| p.phase != ProgressPhase::Complete) {
        terminal.language = match progress.answers["interface_language"].as_str() {
            Some("ko") => InterfaceLanguage::Ko,
            Some("en") => InterfaceLanguage::En,
            _ => terminal.language,
        };
        super::host_scope::check_request(progress, &options.update_hosts)?;
    }
    if let Some(result) =
        preliminary_result(options, mode, &store, &snapshot, config.as_ref(), terminal)?
    {
        return Ok(result);
    }
    refresh_questions(options, &store, &mut snapshot, terminal)?;
    let record = match snapshot.progress.clone() {
        Some(progress) if progress.phase != ProgressPhase::Complete => {
            if !options.resume {
                return Ok(pending(terminal.language, "hive.setup-resume-required"));
            }
            if matches!(
                progress.phase,
                ProgressPhase::Applying | ProgressPhase::RecoverableFailure
            ) {
                return super::recovery::resume(
                    &root_path,
                    &progress,
                    terminal.language,
                    options.legacy_budget.is_some(),
                );
            }
            if !store.resume_ready(&progress) {
                return Ok(pending(terminal.language, "hive.setup-resume-required"));
            }
            progress
        }
        _ => initial_progress(&root_path, &store, config.as_ref())?,
    };
    let mut progress = Conversation {
        record,
        snapshot,
        fresh: config.is_none(),
    };
    super::host_scope::bind(&mut progress.record, &options.update_hosts, config.is_none(),
        progress.snapshot.progress.as_ref().is_none_or(|p| p.phase == ProgressPhase::Complete))?;
    if progress.selected_section.is_none() {
        progress.selected_section.clone_from(&options.section);
    }
    let questions = catalog()?;
    let collected = if progress.legacy_migration.is_some() {
        super::legacy::collect(&root_path, &store, &mut progress, terminal)?
    } else {
        collect(
            &store,
            &mut progress,
            &questions.questions,
            options,
            terminal,
        )?
    };
    if !collected {
        return Ok(pending(terminal.language, "hive.setup-input-pending"));
    }
    preview_apply(
        options,
        &root_path,
        &root_cap,
        existing.as_deref(),
        &store,
        &mut progress,
        terminal,
    )
}

fn refresh_questions<R: BufRead, W: Write>(
    options: &Options,
    store: &ProgressStore,
    snapshot: &mut ProgressSnapshot,
    terminal: &mut Terminal<R, W>,
) -> Result<(), SetupError> {
    if options.resume {
        if let Some(refreshed) = store.refresh_draft_questions(snapshot)? {
            *snapshot = refreshed;
            terminal.say("질문 목록 변경을 확인했습니다. 기존 값은 보존하고 바뀐 답안만 다시 확인합니다.",
                "The question catalog changed. Saved values are preserved; only affected answers need review.")?;
        }
    }
    Ok(())
}

fn initial_progress(
    root: &Path,
    store: &ProgressStore,
    config: Option<&user_setup::UserSetupConfig>,
) -> Result<SetupProgress, SetupError> {
    if let Some(progress) = super::legacy::initial_progress(root, store, config)? {
        return Ok(progress);
    }
    store.new_progress(
        "user-language".into(),
        config
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| SetupError::Internal(error.to_string()))?
            .unwrap_or_else(fresh_answers),
    )
}

fn preliminary_result<R: BufRead, W: Write>(
    options: &Options,
    mode: InteractionMode,
    store: &ProgressStore,
    snapshot: &ProgressSnapshot,
    config: Option<&user_setup::UserSetupConfig>,
    terminal: &mut Terminal<R, W>,
) -> Result<Option<crate::ActionResult>, SetupError> {
    if mode == InteractionMode::NeedsInput {
        return Ok(Some(pending(terminal.language, "hive.setup-needs-input")));
    }
    if options.resume
        && snapshot
            .progress
            .as_ref()
            .is_some_and(|p| p.phase == ProgressPhase::Complete && store.resume_ready(p))
        && config.is_none_or(|config| user_setup::migration::pending_questions(config).is_empty())
    {
        return Ok(Some(result(
            "success",
            0,
            "hive.user-setup-complete",
            terminal.language,
            "이미 적용·검증을 마친 설정입니다.",
            "This setup was already applied and verified.",
            None,
            None,
        )));
    }
    if options.section.as_ref().is_some_and(|section| {
        catalog().map_or(true, |c| {
            c.section(PreferenceScope::User, section).is_empty()
        })
    }) {
        return Err(SetupError::Input(
            "unknown setup question section".to_owned(),
        ));
    }
    if options.show {
        return inspect_settings(
            &options.root,
            config,
            snapshot,
            terminal,
            options.interaction.json,
        )
        .map(Some);
    }
    let confirmed_resume = options.resume
        && snapshot.progress.as_ref().is_some_and(|p| {
            matches!(p.phase, ProgressPhase::Applying | ProgressPhase::RecoverableFailure)
        });
    if mode != InteractionMode::Terminal && !confirmed_resume {
        return Ok(Some(pending(terminal.language, "hive.setup-needs-input")));
    }
    Ok(None)
}

fn fresh_answers() -> Value {
    super::default_answers(None)
}

fn collect<R: BufRead, W: Write>(
    store: &ProgressStore,
    progress: &mut Conversation,
    questions: &[Question],
    options: &Options,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    let selected = |id: &str| {
        questions
            .iter()
            .find(|q| q.id == id)
            .ok_or_else(|| SetupError::Internal("missing setup question".to_owned()))
    };
    if !choose_section(store, progress, selected("user-section")?, terminal)? { return Ok(false); }
    let needs_language = progress.answers.get("interface_language").is_none()
        || (progress.selected_section.as_deref() == Some("language")
            && !answered(progress, selected("user-language")?));
    if needs_language && !ask(store, progress, selected("user-language")?, terminal)? {
        return Ok(false);
    }
    terminal.language = if progress.answers["interface_language"] == "ko" {
        InterfaceLanguage::Ko
    } else {
        InterfaceLanguage::En
    };
    super::host_scope::announce(progress, terminal)?;
    if progress.fresh && progress.answers["preference_decisions"]["/wiki/language"].is_null() {
        progress.answers["wiki"]["language"] = progress.answers["interface_language"].clone();
    }
    for question in questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::User)
    {
        if let Some(value) = progress.answers.pointer(&question.key).cloned() {
            if progress.answers["preference_decisions"][&question.key].is_null() {
                let decision = PreferenceDecision::new(
                    &value,
                    if progress.fresh {
                        DecisionOrigin::Default
                    } else {
                        DecisionOrigin::LegacyUnknown
                    },
                    PreferenceInput::Terminal,
                    PreferenceScope::User,
                    env!("CARGO_PKG_VERSION"),
                    None,
                )
                .map_err(SetupError::Internal)?;
                progress.answers["preference_decisions"][&question.key] = json!(decision);
            }
        }
    }
    if progress.setup_mode.is_none() {
        if progress.selected_section.is_some() {
            progress.setup_mode = Some("custom".to_owned());
        } else {
            "user-mode".clone_into(&mut progress.step);
            persist(store, progress)?;
            let answer = terminal.ask(selected("user-mode")?, None)?;
            let AnswerInput::Answered { value, .. } = answer else {
                return Ok(false);
            };
            progress.setup_mode = value.as_str().map(str::to_owned);
        }
        persist(store, progress)?;
    }
    let quick = progress.setup_mode.as_deref() == Some("expedited");
    for question in questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::User)
    {
        if matches!(question.id.as_str(), "user-language" | "user-mode" | "user-section" | "user-index-rebuild" | "user-monitor-activation" | "user-vector-search") {
            continue;
        }
        if question.id == "user-hosts" && !progress.update_hosts.is_empty() {
            continue;
        }
        if !selected_question(progress, question, options, quick) {
            continue;
        }
        if !question.applicable(&progress.answers) {
            continue;
        }
        if answered(progress, question) && options.resume {
            continue;
        }
        if !ask(store, progress, question, terminal)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn choose_section<R: BufRead, W: Write>(
    store: &ProgressStore,
    progress: &mut Conversation,
    question: &Question,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    if progress.fresh || progress.selected_section.is_some() || progress.setup_mode.is_some() {
        return Ok(true);
    }
    progress.step.clone_from(&question.id);
    persist(store, progress)?;
    let AnswerInput::Answered { value, .. } = terminal.ask(question, None)? else { return Ok(false) };
    let section = value.as_str().ok_or_else(|| SetupError::Input("invalid setting group".into()))?;
    progress.selected_section = (section != "all").then(|| section.to_owned());
    progress.setup_mode = Some("custom".into());
    progress.answered_questions.insert(question.id.clone(), question.revision);
    progress.question_bindings.insert(question.id.clone(), question.into());
    persist(store, progress)?;
    Ok(true)
}

fn ask<R: BufRead, W: Write>(
    store: &ProgressStore,
    progress: &mut Conversation,
    question: &Question,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    progress.step.clone_from(&question.id);
    persist(store, progress)?;
    let current = progress.answers.pointer(&question.key);
    let (value, origin) = loop {
        let AnswerInput::Answered { value, origin } = terminal.ask(question, current)? else {
            return Ok(false);
        };
        if question.id == "user-context-description" && value == json!(":clear")
            && (origin == DecisionOrigin::Explicit || current.is_none()) {
            if progress.answers["profile"]["contexts"].as_array().is_some_and(|items| !items.is_empty()) {
                break (Value::Null, origin);
            }
            terminal.say("배경 종류를 고르지 않았으므로 설명을 입력해 주세요.",
                "Enter a description because no background is selected.")?;
            continue;
        }
        if config_answer_valid(question, &progress.answers, &value)? {
            if question.id == "usage-project-overrides" && !registered_overrides(store, &progress.answers, &value)? {
                terminal.say("등록된 프로젝트 식별자와 전역 기준 이상의 값을 입력해 주세요. 프로젝트 파일은 조사하지 않습니다.",
                    "Use registered identities and values at or above the global threshold. Project files are not inspected.")?;
                continue;
            }
            break (value, origin);
        }
        terminal.say(
            "입력한 구조가 이 설정의 형식과 맞지 않습니다. 이 항목만 다시 입력해 주세요.",
            "The structure does not match this setting. Enter this answer again.",
        )?;
    };
    if question.id == "user-context-description" && value.is_null() {
        progress.answers["profile"].as_object_mut().expect("profile").remove("description");
    } else {
        set_value(&mut progress.answers, &question.key, value.clone())?;
    }
    if question.id == "discord-enabled" && value == json!(false) {
        progress.answers["usage_guard"]["discord"].as_object_mut()
            .expect("notification preferences").remove("webhook_url_env");
        progress.answers["preference_decisions"].as_object_mut()
            .expect("choice records").remove("/usage_guard/discord/webhook_url_env");
        progress.answered_questions.remove("discord-environment");
    }
    if question.id == "user-wiki" && value == json!(false) {
        progress.index_rebuild_scope = None;
    }
    let mut decision = PreferenceDecision::new(
        &value,
        origin,
        PreferenceInput::Terminal,
        PreferenceScope::User,
        env!("CARGO_PKG_VERSION"),
        Some(user_setup::feature_now_unix()?),
    )
    .map_err(SetupError::Input)?;
    decision.question_id = Some(question.id.clone());
    decision.question_revision = Some(question.revision);
    progress.answers["preference_decisions"][&question.key] = json!(decision);
    progress
        .answered_questions
        .insert(question.id.clone(), question.revision);
    progress.question_bindings.insert(question.id.clone(), question.into());
    persist(store, progress)?;
    Ok(true)
}

pub(super) fn config_answer_valid(
    question: &Question,
    answers: &Value,
    value: &Value,
) -> Result<bool, SetupError> {
    let source = match question.scope {
        PreferenceScope::User => user_setup::USER_SETUP_SCHEMA,
        PreferenceScope::Project => {
            include_str!("../../../../../schemas/setup-answers.schema.json")
        }
    };
    let schema: Value =
        serde_json::from_str(source).map_err(|error| SetupError::Internal(error.to_string()))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let mut proposed = answers.clone();
    set_value(&mut proposed, &question.key, value.clone())?;
    // Other required answers may still be pending. Reject only errors in this answer's subtree.
    let nested = format!("{}/", question.key);
    let invalid = validator.iter_errors(&proposed).any(|error| {
        let path = error.instance_path().to_string();
        path == question.key || path.starts_with(&nested)
    });
    Ok(!invalid)
}

fn registered_overrides(store: &ProgressStore, answers: &Value, value: &Value) -> Result<bool, SetupError> {
    let values = value.as_object().expect("validated threshold map");
    if values.is_empty() { return Ok(true); }
    let threshold = answers["usage_guard"]["stop_remaining_percent"].as_u64()
        .ok_or_else(|| SetupError::Input("global threshold must be selected first".into()))?;
    if values.values().any(|value| value.as_u64().is_none_or(|value| value < threshold)) {
        return Ok(false);
    }
    let registry = hive_wiki::shared::load_project_registry_metadata(store.user_root_path())
        .map_err(|error| SetupError::Input(format!("cannot read registered project identities: {error}")))?;
    Ok(values.keys().all(|id| registry.projects.iter().any(|project| &project.id == id)))
}

pub(super) fn answered(progress: &SetupProgress, question: &Question) -> bool {
    progress.answered_questions.get(&question.id) == Some(&question.revision)
        && progress.question_bindings.get(&question.id).is_some_and(|binding|
            binding.revision == question.revision && binding.key == question.key && binding.input == question.input)
        && (progress.answers.pointer(&question.key).is_some_and(|value|
            question.validate_value(value).is_ok() && config_answer_valid(question, &progress.answers, value).unwrap_or(false))
            || cleared_description(&progress.answers, question))
}

fn cleared_description(answers: &Value, question: &Question) -> bool {
    question.id == "user-context-description" && answers.pointer(&question.key).is_none()
        && answers["profile"]["contexts"].as_array().is_some_and(|items| !items.is_empty())
        && answers["preference_decisions"][&question.key]["value_digest"]
            == json!(hive_core::preferences::declaration_digest(&Value::Null).expect("null fingerprint"))
}

fn selected_question(progress: &SetupProgress, question: &Question, options: &Options, quick: bool) -> bool {
    if options.resume && progress.answered_questions.contains_key(&question.id) && !answered(progress, question) {
        return true;
    }
    progress.selected_section.as_ref().is_none_or(|section| &question.section == section)
        && (!quick || matches!(question.id.as_str(), "user-contexts" | "user-hosts")
            || (question.id == "user-context-description" && progress.answers["profile"]["contexts"].as_array().is_some_and(Vec::is_empty)))
}

pub(super) fn set_value(
    document: &mut Value,
    pointer: &str,
    value: Value,
) -> Result<(), SetupError> {
    let (parent, key) = pointer
        .rsplit_once('/')
        .ok_or_else(|| SetupError::Internal("invalid preference pointer".to_owned()))?;
    let object = document
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| SetupError::Internal("missing preference object".to_owned()))?;
    object.insert(key.replace("~1", "/").replace("~0", "~"), value);
    Ok(())
}

pub(super) fn persist(
    store: &ProgressStore,
    progress: &mut Conversation,
) -> Result<(), SetupError> {
    progress.snapshot = store.save(&progress.snapshot, &progress.record)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn preview_apply<R: BufRead, W: Write>(
    options: &Options,
    root: &Path,
    root_cap: &cap_std::fs::Dir,
    existing: Option<&[u8]>,
    store: &ProgressStore,
    progress: &mut Conversation,
    terminal: &mut Terminal<R, W>,
) -> Result<crate::ActionResult, SetupError> {
    if progress.answers["persona"]["id"] != "custom" {
        progress.answers["persona"]
            .as_object_mut()
            .expect("persona object")
            .remove("custom_description");
    }
    let bytes = serde_json::to_vec(&progress.answers)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let mut input =
        tempfile::NamedTempFile::new().map_err(|error| SetupError::Internal(error.to_string()))?;
    input
        .write_all(&bytes)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    let mut args = user_setup::Arguments {
        answers: input.path().to_owned(),
        mode: user_setup::SetupMode::DryRun,
        user_root: root.to_owned(),
        root_cap: root_cap
            .try_clone()
            .map_err(|error| SetupError::Internal(error.to_string()))?,
    };
    let preview = user_setup::execute_answers(&args, &bytes, user_setup::SetupBase::Expected(existing))?;
    super::host_scope::bind_deactivations(&preview, &mut progress.record)?;
    super::host_scope::display_deactivations(&progress.record, terminal)?;
    if !super::vector_choice::collect(root_cap, store, progress, terminal)? {
        return Ok(pending(terminal.language, "hive.setup-input-pending"));
    }
    if !super::monitor_start::collect(root, existing, store, progress, terminal)? {
        return Ok(pending(terminal.language, "hive.setup-input-pending"));
    }
    super::monitor_stop::bind(root, &mut progress.record)?;
    if progress.monitor_stop.is_some() {
        terminal.say("Hive가 등록한 상시 감시를 해제하고 종료를 확인합니다. 지난 사용량 기록은 보존합니다.",
            "Remove the owned Hive monitor registration and confirm that it stopped. Keep usage history.")?;
    }
    if !super::index_rebuild::collect(root, root_cap, store, progress, terminal)? {
        return Ok(pending(terminal.language, "hive.setup-input-pending"));
    }
    let displayed = display_preview(&progress.answers, terminal)?;
    super::vector_choice::display(&progress.record, terminal)?;
    progress.phase = ProgressPhase::Previewed;
    persist(store, progress)?;
    if options.dry_run || !terminal.confirm()? {
        return Ok(pending(terminal.language, "hive.setup-input-pending"));
    }
    if options
        .legacy_budget
        .as_ref()
        .is_some_and(|budget| budget.apply_deadline().is_none())
    {
        return Ok(pending(terminal.language, "hive.update-input-pending"));
    }
    record_bundle_acceptance(progress, &displayed)?;
    super::monitor_start::preflight(root, &progress.record)?;
    let _lease = if options.legacy_budget.is_none() {
        Some(crate::user_install::lock_user_setup(root_cap).map_err(SetupError::Conflict)?)
    } else {
        None
    };
    let bytes = serde_json::to_vec(&progress.answers)
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    input
        .as_file_mut()
        .set_len(0)
        .and_then(|()| input.rewind())
        .and_then(|()| input.write_all(&bytes))
        .map_err(|error| SetupError::Internal(error.to_string()))?;
    super::application::begin(store, &progress.snapshot, &mut progress.record)?;
    args.mode = user_setup::SetupMode::Apply;
    let application = if let Some(budget) = &options.legacy_budget {
        super::worker::apply(&args, &bytes, existing, &progress.operation_id, budget)
    } else {
        super::legacy::apply(&args, &bytes, existing, &progress.record)
    };
    let applied = super::application::finish(store, &progress.record, application)?;
    let mut completed = result(
        "success",
        0,
        "hive.user-setup-complete",
        terminal.language,
        "Hive 전역 설정 적용·검증 완료",
        "Hive user setup applied and verified",
        Some(
            json!({"operation_id":progress.operation_id,"changed_file_count":applied.changed_paths.len(),"model_calls":0}),
        ),
        None,
    );
    completed.changed_paths = applied.changed_paths;
    Ok(completed)
}

fn record_bundle_acceptance(
    progress: &mut Conversation,
    displayed: &hive_core::setup::QuestionCatalog,
) -> Result<(), SetupError> {
    // The bundle confirmation records acceptance only for choices displayed above.
    let accepted_questions: Vec<_> = displayed
        .questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::User && q.applicable(&progress.answers))
        .collect();
    for question in accepted_questions {
        if progress.answers["preference_decisions"][&question.key]["origin"] == "default" {
            if let Some(value) = progress.answers.pointer(&question.key).cloned() {
                let mut decision = PreferenceDecision::new(
                    &value,
                    DecisionOrigin::AcceptedDefault,
                    PreferenceInput::Terminal,
                    PreferenceScope::User,
                    env!("CARGO_PKG_VERSION"),
                    Some(user_setup::feature_now_unix()?),
                )
                .map_err(SetupError::Internal)?;
                decision.question_id = Some(question.id.clone());
                decision.question_revision = Some(question.revision);
                progress.answers["preference_decisions"][&question.key] = json!(decision);
            }
        }
    }
    Ok(())
}

pub(super) fn display_preview<R: BufRead, W: Write>(
    answers: &Value,
    terminal: &mut Terminal<R, W>,
) -> Result<hive_core::setup::QuestionCatalog, SetupError> {
    terminal.say("적용 전 확인: 선택한 앱의 Hive 설정과 지침 갱신. 기존 사용자 자료 보존.","Preview: update Hive settings and instructions for the selected apps. Preserve existing user data.")?;
    let displayed = catalog()?;
    for question in displayed
        .questions
        .iter()
        .filter(|q| q.scope == PreferenceScope::User && q.applicable(answers))
    {
        if let Some(value) = answers.pointer(&question.key)
            .or_else(|| cleared_description(answers, question).then_some(&Value::Null)) {
            writeln!(
                terminal.writer,
                "{} {}",
                question
                    .prompt
                    .get(terminal.language == InterfaceLanguage::Ko),
                super::terminal::display_value(
                    question,
                    value,
                    terminal.language == InterfaceLanguage::Ko
                )
            )
            .map_err(|error| SetupError::Input(error.to_string()))?;
        }
    }
    Ok(displayed)
}

fn inspect_settings<R: BufRead, W: Write>(
    root: &Path,
    config: Option<&user_setup::UserSetupConfig>,
    snapshot: &ProgressSnapshot,
    terminal: &mut Terminal<R, W>,
    json_output: bool,
) -> Result<crate::ActionResult, SetupError> {
    let manual_commands = config.map(|config| config.selected_hosts.iter()
        .map(|host| crate::usage_control::manual_usage_command(root, host.as_str())).collect::<Vec<_>>())
        .unwrap_or_default();
    let migration = config
        .map(|config| user_setup::migration::inspect(root, config))
        .transpose()?
        .flatten();
    let policy = config
        .and_then(|c| c.usage_guard.policy.as_ref())
        .or_else(|| {
            migration
                .as_ref()
                .and_then(|preview| preview.candidate.usage_guard.policy.as_ref())
        });
    let monitor = crate::usage_monitor::setup_status(root);
    let root_cap = crate::user_install::open_user_root_for_setup(root).map_err(SetupError::Conflict)?;
    let (_, feature_preferences) = user_setup::load_feature_answers(&root_cap)?;
    if !json_output {
        if let Some(config) = migration
            .as_ref()
            .map(|preview| &preview.candidate)
            .or(config)
        {
            let values = serde_json::to_value(config)
                .map_err(|error| SetupError::Internal(error.to_string()))?;
            display_preview(&values, terminal)?;
        } else {
            terminal.say(
                "아직 저장된 설정이 없습니다.",
                "No settings have been saved yet.",
            )?;
        }
        display_monitor(&monitor, terminal)?;
        if !crate::display_options::quiet() && !manual_commands.is_empty() {
            terminal.say("경고는 이미 확인한 사용량을 이용하며 추가로 조회하지 않습니다. 보호를 바꾸지 않고 사용량을 직접 확인하려면 다음 명령을 실행하세요.",
                "Warnings reuse existing measurements without extra queries. To read usage without changing protection, run one of these commands:")?;
            for command in &manual_commands { terminal.say(command, command)?; }
        }
    }
    Ok(result(
        "success",
        0,
        "hive.setup-shown",
        terminal.language,
        "저장된 설정과 미완료 질문 조회 완료",
        "Stored settings and pending questions inspected",
        Some(
            json!({"stored":config,"effective_usage":policy.map(|p| json!({"preflight":p.effective_preflight(),"midway":p.effective_midway()})),
            "pending":snapshot.progress.as_ref().filter(|p| p.phase != ProgressPhase::Complete),"actual_monitor_state":monitor,
            "feature_preferences":feature_preferences,"vector_runtime_verification":"not-performed",
            "manual_usage_commands":manual_commands,
            "migration_preview":migration,"pending_questions":config.map(user_setup::migration::pending_questions).unwrap_or_default()}),
        ),
        None,
    ))
}

pub(super) fn display_monitor<R: BufRead, W: Write>(monitor: &Value, terminal: &mut Terminal<R, W>) -> Result<(), SetupError> {
    let (saved_ko, saved_en) = match monitor["saved_enabled"].as_bool() {
        Some(true) => ("켜짐", "on"), Some(false) => ("꺼짐", "off"),
        None if monitor["configuration"] == "absent" => ("설정 없음", "not configured"),
        None => ("확인 못함", "unverified"),
    };
    let (state_ko, state_en) = match monitor["controller_state"].as_str() {
        Some("running") => ("실행 중", "running"), Some("stopped") => ("종료 확인", "stop confirmed"),
        Some("not-configured") => ("설정 없음", "not configured"), _ => ("확인 못함", "unverified"),
    };
    terminal.say(&format!("감시 저장 선택: {saved_ko}"), &format!("Saved monitoring selection: {saved_en}"))?;
    terminal.say(&format!("확인한 감시 실행 상태: {state_ko}"), &format!("Observed monitor state: {state_en}"))?;
    if monitor["controller_state"] == "running" {
        terminal.say(if monitor["monitoring_verified"] == true {"감시 관측·제어: 정상 확인"} else {"감시 관측·제어: 확인 못함"},
            if monitor["monitoring_verified"] == true {"Observation and control: verified"} else {"Observation and control: unverified"})?;
    }
    Ok(())
}

pub(super) fn pending(language: InterfaceLanguage, code: &'static str) -> crate::ActionResult {
    result(
        "needs-input",
        2,
        code,
        language,
        "설정이 아직 끝나지 않았습니다. 터미널에서 이어서 진행해 주세요.",
        "Setup is pending. Continue in a terminal.",
        None,
        Some("hive setup --scope user --resume".to_owned()),
    )
}

#[allow(clippy::too_many_arguments)]
fn result(
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    language: InterfaceLanguage,
    ko: &str,
    en: &str,
    data: Option<Value>,
    next_action: Option<String>,
) -> crate::ActionResult {
    crate::ActionResult {
        schema_version: 1,
        action: "SetupHiveUser",
        status,
        exit_code,
        code,
        message: if language == InterfaceLanguage::Ko {
            ko
        } else {
            en
        }
        .to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn pending_language_change_controls_input_and_resume_notices_before_config_apply() {
        for (stored, pending) in [(InterfaceLanguage::Ko, InterfaceLanguage::En),
            (InterfaceLanguage::En, InterfaceLanguage::Ko)] {
            let root = tempfile::tempdir().unwrap();
            let file = root.path().join(user_setup::USER_SETUP_RELATIVE);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            let before = serde_json::to_vec(&super::super::default_answers(Some(stored))).unwrap();
            std::fs::write(&file, &before).unwrap();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let progress = store.new_progress("user-language".into(), super::super::default_answers(Some(pending))).unwrap();
            store.save(&store.load().unwrap(), &progress).unwrap();
            for mode in [InteractionMode::NonInteractive, InteractionMode::Terminal] {
                let mut terminal = Terminal { reader: Cursor::new(b""), writer: Vec::new(), language: stored };
                let result = execute(&options(root.path()), mode, &mut terminal).unwrap();
                assert_eq!(result.status, "needs-input");
                assert_eq!(terminal.language, pending);
                assert!(terminal.writer.is_empty());
                assert_eq!(std::fs::read(&file).unwrap(), before);
            }
        }
    }

    #[test]
    fn project_overrides_validate_registered_metadata_without_opening_project_directories() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("unavailable-project");
        let registry = json!({"schema_version":1,"projects":[{"id":"known-project","root":missing,
            "enabled":true,"language":"en","visibility":"shared"}]});
        let path = root.path().join(hive_wiki::shared::PROJECT_REGISTRY_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = serde_json::to_vec(&registry).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        progress.answers = super::super::default_answers(Some(InterfaceLanguage::En));
        let catalog = catalog().unwrap();
        let question = catalog.questions.iter().find(|q| q.id=="usage-project-overrides").unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new(b"{\"missing\":30}\n{\"known-project\":101}\n{\"known-project\":19}\n{\"known-project\":40}\n"),
            writer: Vec::new(), language: InterfaceLanguage::En,
        };
        assert!(ask(&store, &mut progress, question, &mut terminal).unwrap());
        let config = user_setup::parse_and_validate_config(&serde_json::to_vec(&progress.answers).unwrap()).unwrap();
        user_setup::validate_registered_project_overrides(&config, root.path()).unwrap();
        assert_eq!(config.usage_guard.project_overrides["known-project"], 40);
        assert_eq!(String::from_utf8(terminal.writer).unwrap().matches(&question.prompt.en).count(), 4);
        assert_eq!(std::fs::read(path).unwrap(), bytes);
        assert!(!missing.exists());
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
        let empty = tempfile::tempdir().unwrap();
        let empty_store = ProgressStore::open(empty.path(), None).unwrap();
        assert!(!registered_overrides(&empty_store, &progress.answers, &json!({"unknown":30})).unwrap());
        assert!(registered_overrides(&empty_store, &progress.answers, &json!({})).unwrap());
        assert!(!empty.path().join(".hive").exists());
    }

    #[test]
    fn only_confirmed_noninteractive_resume_passes_the_input_gate() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut snapshot = store.load().unwrap();
        let mut record = store.new_progress("file-apply".into(), fresh_answers()).unwrap();
        let mut options = options(root.path());
        options.resume = true;
        let mut terminal = Terminal {
            reader: Cursor::new(b""), writer: Vec::new(), language: InterfaceLanguage::En,
        };
        for (phase, needs_input) in [
            (ProgressPhase::AwaitingAnswers, true), (ProgressPhase::Previewed, true),
            (ProgressPhase::Applying, false), (ProgressPhase::RecoverableFailure, false),
        ] {
            record.phase = phase;
            snapshot.progress = Some(record.clone());
            assert_eq!(preliminary_result(
                &options, InteractionMode::NonInteractive, &store, &snapshot, None, &mut terminal,
            ).unwrap().is_some(), needs_input);
        }
        assert!(parse(&["--scope".into(), "user".into(), "--resume".into(), "--dry-run".into()]).is_err());
        assert!(terminal.writer.is_empty());
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn update_resume_does_not_create_progress_or_consume_noninteractive_answers() {
        let root = tempfile::tempdir().unwrap();
        assert!(resume_pending_update(root.path(), false).unwrap().is_none());
        assert!(!root.path().join(".hive").exists());
        let store = ProgressStore::open(root.path(), None).unwrap();
        let progress = store
            .new_progress("user-language".into(), fresh_answers())
            .unwrap();
        store.save(&store.load().unwrap(), &progress).unwrap();
        let before = std::fs::read(root.path().join(&store.relative)).unwrap();
        let result = resume_pending_update(root.path(), false).unwrap().unwrap();
        assert_eq!(result.action, "UpdateHiveUser");
        assert_eq!(result.code, "hive.update-input-pending");
        assert!(result
            .next_action
            .unwrap()
            .starts_with("hive update --user-root"));
        assert_eq!(
            std::fs::read(root.path().join(&store.relative)).unwrap(),
            before
        );
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
    }

    #[test]
    fn optional_existing_setup_does_not_become_an_update_input_requirement() {
        let root = tempfile::tempdir().unwrap();
        let mut settings = fresh_answers();
        settings["interface_language"] = json!("en");
        let config = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(&config, serde_json::to_vec(&settings).unwrap()).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let progress = store.new_progress("user-persona".into(), settings).unwrap();
        store.save(&store.load().unwrap(), &progress).unwrap();
        assert!(resume_pending_update(root.path(), false).unwrap().is_none());
        assert_eq!(store.load().unwrap().progress.unwrap(), progress);
    }

    fn options(root: &Path) -> Options {
        Options {
            root: root.to_owned(),
            interaction: InteractionRequest::default(),
            resume: false,
            review: false,
            review_answers: None,
            show: false,
            dry_run: false,
            section: None,
            legacy_budget: None,
            update_hosts: Vec::new(),
        }
    }

    fn conversation(store: &ProgressStore) -> Conversation {
        Conversation {
            record: store
                .new_progress("user-language".to_owned(), fresh_answers())
                .unwrap(),
            snapshot: store.load().unwrap(),
            fresh: true,
        }
    }

    #[test]
    fn notification_section_collects_privacy_and_fields_then_clears_a_disabled_reference() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        progress.answers = super::super::default_answers(Some(InterfaceLanguage::En));
        progress.setup_mode = Some("custom".into());
        progress.selected_section = Some("notifications".into());
        let mut terminal = Terminal {
            reader: Cursor::new(b"no\nyes\nHIVE_TEST_WEBHOOK\n2\n1,5\n"),
            writer: Vec::new(), language: InterfaceLanguage::En,
        };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut terminal).unwrap());
        let config = user_setup::parse_and_validate_config(&serde_json::to_vec(&progress.answers).unwrap()).unwrap();
        assert_eq!(config.usage_guard.popup_enabled, Some(false));
        assert!(config.usage_guard.discord.enabled);
        assert_eq!(config.usage_guard.discord.request_privacy, user_setup::DiscordRequestPrivacy::RawPrompt);
        assert_eq!(progress.answers["usage_guard"]["discord"]["message_fields"], json!(["remaining-usage", "host"]));
        assert_eq!(progress.answers["preference_decisions"]["/usage_guard/discord/request_privacy"]["origin"], "explicit");
        assert!(!config.usage_guard.policy.unwrap().enforcement_enabled);
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
        let mut disable = Terminal {
            reader: Cursor::new(b"no\nno\n"), writer: Vec::new(), language: InterfaceLanguage::En,
        };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut disable).unwrap());
        assert!(progress.answers["usage_guard"]["discord"].get("webhook_url_env").is_none());
        assert!(progress.answers["preference_decisions"].get("/usage_guard/discord/webhook_url_env").is_none());
        assert!(!String::from_utf8(disable.writer).unwrap().contains("How much request content"));
        user_setup::parse_and_validate_config(&serde_json::to_vec(&progress.answers).unwrap()).unwrap();
    }

    #[test]
    fn profile_details_can_be_added_cleared_and_resumed_with_a_null_choice_record() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        progress.answers = super::super::default_answers(Some(InterfaceLanguage::En));
        progress.setup_mode = Some("custom".into());
        progress.selected_section = Some("profile".into());
        let mut terminal = Terminal { reader: Cursor::new(b"1\nLocal tools\n"), writer: Vec::new(), language: InterfaceLanguage::En };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut terminal).unwrap());
        assert_eq!(progress.answers["profile"]["description"], "Local tools");
        let mut clear = Terminal { reader: Cursor::new(b"\n:clear\n"), writer: Vec::new(), language: InterfaceLanguage::En };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut clear).unwrap());
        let config = user_setup::parse_and_validate_config(&serde_json::to_vec(&progress.answers).unwrap()).unwrap();
        assert!(config.profile.description.is_none());
        let decision = &config.preference_decisions["/profile/description"];
        assert_eq!(decision.origin, DecisionOrigin::Explicit);
        assert!(decision.matches_declaration(&Value::Null).unwrap());
        let mut preview = Terminal { reader: Cursor::new(b""), writer: Vec::new(), language: InterfaceLanguage::En };
        display_preview(&progress.answers, &mut preview).unwrap();
        assert!(String::from_utf8(preview.writer).unwrap().contains("Describe any additional background. None"));
        let mut resume_options = options(root.path()); resume_options.resume = true;
        let mut resumed = Terminal { reader: Cursor::new(b""), writer: Vec::new(), language: InterfaceLanguage::En };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &resume_options, &mut resumed).unwrap());
        assert!(resumed.writer.is_empty());
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
    }

    #[test]
    fn quick_setup_requires_details_when_no_background_is_selected_and_preserves_eof() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        progress.answers = super::super::default_answers(Some(InterfaceLanguage::En));
        progress.setup_mode = Some("expedited".into());
        let mut first = Terminal { reader: Cursor::new(b"[]\n:clear\n"), writer: Vec::new(), language: InterfaceLanguage::En };
        assert!(!collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut first).unwrap());
        assert!(!progress.answered_questions.contains_key("user-context-description"));
        assert!(String::from_utf8(first.writer).unwrap().contains("no background is selected"));
        let mut resume_options = options(root.path()); resume_options.resume = true;
        let mut resumed = Terminal { reader: Cursor::new(b"Local work\n1\n"), writer: Vec::new(), language: InterfaceLanguage::En };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &resume_options, &mut resumed).unwrap());
        let config = user_setup::parse_and_validate_config(&serde_json::to_vec(&progress.answers).unwrap()).unwrap();
        assert!(config.profile.contexts.is_empty());
        assert_eq!(config.profile.description.as_deref(), Some("Local work"));
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
    }

    #[test]
    fn legacy_show_reports_independent_zero_guard_without_saving_migration_or_reading_input() {
        let root = tempfile::tempdir().unwrap();
        let mut stored = fresh_answers();
        stored["schema_version"] = json!(1);
        stored["interface_language"] = json!("ko");
        stored["usage_guard"] = json!({"enabled":false,"stop_remaining_percent":5});
        let bytes = serde_json::to_vec(&stored).unwrap();
        let path = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let mut options = options(root.path());
        options.show = true;
        options.interaction.json = true;
        let mut terminal = Terminal {
            reader: Cursor::new(b"no\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        let shown = execute(&options, InteractionMode::NonInteractive, &mut terminal).unwrap();
        assert_eq!(shown.code, "hive.setup-shown");
        let data = shown.data.unwrap();
        assert_eq!(data["stored"]["schema_version"], 1);
        assert_eq!(
            data["effective_usage"]["preflight"]["threshold_enabled"],
            false
        );
        assert_eq!(
            data["effective_usage"]["preflight"]["zero_quota_enabled"],
            true
        );
        assert_eq!(data["migration_preview"]["candidate"]["schema_version"], 2);
        assert_eq!(data["actual_monitor_state"]["controller_state"], "not-configured");
        assert_eq!(data["actual_monitor_state"]["sensor_queries"], 0);
        assert_eq!(terminal.reader.position(), 0);
        assert!(terminal.writer.is_empty());
        assert_eq!(std::fs::read(path).unwrap(), bytes);
        assert!(!root.path().join(".hive/runtime").exists());
        assert!(!root.path().join(".hive/config/setup-progress").exists());
    }

    #[test]
    fn monitor_status_text_uses_the_selected_language_without_claiming_readiness() {
        for (language, off, stopped, unknown) in [(InterfaceLanguage::Ko, "꺼짐", "종료 확인", "확인 못함"),
            (InterfaceLanguage::En, "off", "stop confirmed", "unverified")] {
            let mut terminal = Terminal { reader: Cursor::new(b"unused\n"), writer: Vec::new(), language };
            display_monitor(&json!({"saved_enabled":false,"controller_state":"stopped"}), &mut terminal).unwrap();
            let output = String::from_utf8(terminal.writer).unwrap();
            assert!(output.contains(off) && output.contains(stopped));
            terminal.writer = Vec::new();
            display_monitor(&json!({"saved_enabled":true,"controller_state":"running","monitoring_verified":false}), &mut terminal).unwrap();
            assert!(String::from_utf8(terminal.writer).unwrap().contains(unknown));
            assert_eq!(terminal.reader.position(), 0);
        }
    }

    #[test]
    fn quick_answers_are_valid_v2_without_any_model_or_settings_write() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let mut terminal = Terminal {
            reader: Cursor::new("2\n\n1\n1\n".as_bytes()),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(collect(
            &store,
            &mut progress,
            &catalog().unwrap().questions,
            &options(root.path()),
            &mut terminal
        )
        .unwrap());
        let config =
            user_setup::parse_and_validate_config(&serde_json::to_vec(&progress.answers).unwrap())
                .unwrap();
        assert_eq!(config.schema_version, 2);
        assert_eq!(config.interface_language, InterfaceLanguage::En);
        assert!(!config.usage_guard.policy.unwrap().enforcement_enabled);
        assert_eq!(
            config.preference_decisions["/selected_hosts"].origin,
            DecisionOrigin::Explicit
        );
        assert_eq!(
            config.preference_decisions["/usage_guard/enforcement_enabled"].origin,
            DecisionOrigin::Default
        );
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
        assert!(!root.path().join(".hive/install").exists());
    }

    #[test]
    fn interruption_preserves_language_and_resume_does_not_repeat_answered_questions() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let mut terminal = Terminal {
            reader: Cursor::new("1\n\n1\n".as_bytes()),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(!collect(
            &store,
            &mut progress,
            &catalog().unwrap().questions,
            &options(root.path()),
            &mut terminal
        )
        .unwrap());
        assert_eq!(store.load().unwrap().progress.unwrap().step, "user-hosts");
        let mut options = options(root.path());
        options.resume = true;
        let mut resumed = Terminal {
            reader: Cursor::new("1\n".as_bytes()),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(collect(
            &store,
            &mut progress,
            &catalog().unwrap().questions,
            &options,
            &mut resumed
        )
        .unwrap());
        let shown = String::from_utf8(resumed.writer).unwrap();
        assert!(shown.contains("Hive를 사용할 앱"));
        assert!(!shown.contains("사용할 언어"));
        assert!(!shown.contains("배경이나 관심사"));
    }

    #[test]
    fn machine_mode_and_missing_terminal_never_read_stdin_or_create_progress() {
        let root = tempfile::tempdir().unwrap();
        for mode in [InteractionMode::NonInteractive, InteractionMode::NeedsInput] {
            let mut terminal = Terminal {
                reader: Cursor::new(b"yes\n"),
                writer: Vec::new(),
                language: InterfaceLanguage::En,
            };
            let result = execute(&options(root.path()), mode, &mut terminal).unwrap();
            assert_eq!(result.code, "hive.setup-needs-input");
            hive_core::validate_json_schema(
                include_str!("../../../../../schemas/action-result.schema.json"),
                &serde_json::to_value(&result).unwrap(),
                "setup result",
            )
            .unwrap();
            assert_eq!(terminal.reader.position(), 0);
            assert!(terminal.writer.is_empty());
            assert!(!root.path().join(".hive").exists());
        }
    }

    #[test]
    fn frozen_update_keeps_selected_apps_after_language_without_repeating_host_question() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        super::super::host_scope::bind(&mut progress.record, &["antigravity".into()], true, true).unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new(b"1\n\n\n"), writer: Vec::new(), language: InterfaceLanguage::En,
        };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions,
            &options(root.path()), &mut terminal).unwrap());
        let shown = String::from_utf8(terminal.writer).unwrap();
        assert!(shown.contains("앞서 선택한 앱에 갱신을 적용합니다: antigravity"));
        assert!(!shown.contains("Hive를 사용할 앱"));
        assert_eq!(progress.answers["selected_hosts"], json!(["antigravity"]));
        assert!(!progress.answered_questions.contains_key("user-hosts"));
    }

    #[test]
    fn resume_after_catalog_change_reasks_only_the_changed_answer() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let questions = catalog().unwrap().questions;
        let mut initial = Terminal { reader: Cursor::new(b"1\n\n1\n1\n"), writer:Vec::new(), language:InterfaceLanguage::En };
        assert!(collect(&store, &mut progress, &questions, &options(root.path()), &mut initial).unwrap());
        let original_hosts = progress.answers["selected_hosts"].clone();
        progress.catalog_digest = hive_core::sha256_digest(b"older catalog");
        progress.answered_questions.insert("user-contexts".into(), 2);
        progress.question_bindings.get_mut("user-contexts").unwrap().revision = 2;
        std::fs::write(root.path().join(&store.relative), serde_yaml::to_string(&progress.record).unwrap()).unwrap();
        let refreshed = store.refresh_draft_questions(&store.load().unwrap()).unwrap().unwrap();
        progress.record = refreshed.progress.clone().unwrap();
        progress.snapshot = refreshed;
        let mut options = options(root.path());
        options.resume = true;
        let mut resumed = Terminal { reader:Cursor::new(b"3\n"), writer:Vec::new(), language:InterfaceLanguage::Ko };
        assert!(collect(&store, &mut progress, &questions, &options, &mut resumed).unwrap());
        let shown = String::from_utf8(resumed.writer).unwrap();
        assert_eq!(shown.matches("관련된 배경이나 관심사").count(), 1);
        assert!(!shown.contains("사용할 언어"));
        assert!(!shown.contains("Hive를 사용할 앱"));
        assert_eq!(progress.answers["selected_hosts"], original_hosts);
        assert_eq!(progress.answers["profile"]["contexts"], json!(["non-developer"]));
        assert_eq!(progress.answered_questions["user-contexts"], 1);
    }

    #[test]
    fn wording_and_default_changes_do_not_replace_a_valid_answer_but_invalid_choices_need_review() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let mut question = catalog().unwrap().questions.into_iter().find(|q| q.id == "user-hosts").unwrap();
        progress.answered_questions.insert(question.id.clone(), question.revision);
        progress.answers["selected_hosts"] = json!(["codex"]);
        assert!(answered(&progress, &question));
        question.prompt.ko = "다듬은 질문".into();
        question.default = Some(json!(["claude"]));
        assert!(answered(&progress, &question));
        question.options.retain(|choice| choice.value != "codex");
        assert!(!answered(&progress, &question));
        let mut options = options(root.path()); options.resume = true;
        progress.selected_section = Some("persona".into());
        assert!(selected_question(&progress, &question, &options, true));
    }

    #[test]
    fn configured_users_select_a_group_and_language_changes_do_not_change_wiki_or_hosts() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        progress.fresh = false;
        progress.answers["interface_language"] = json!("ko");
        progress.answers["wiki"]["language"] = json!("ko");
        let mut terminal = Terminal { reader:Cursor::new(b"language\n2\n"), writer:Vec::new(), language:InterfaceLanguage::Ko };
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut terminal).unwrap());
        let shown = String::from_utf8(terminal.writer).unwrap();
        assert!(shown.contains("변경할 설정을 골라 주세요"));
        assert!(!shown.contains("설정 방식을 골라 주세요"));
        assert!(!shown.contains("Hive를 사용할 앱을 골라 주세요"));
        assert_eq!(progress.answers["interface_language"], "en");
        assert_eq!(progress.answers["wiki"]["language"], "ko");
        assert_eq!(progress.answers["selected_hosts"], json!(["codex"]));
        assert_eq!(terminal.language, InterfaceLanguage::En);
    }

    #[test]
    fn cancelled_group_selection_preserves_values_and_can_resume_a_single_group() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        progress.fresh = false; progress.answers["interface_language"] = json!("ko");
        let original = progress.answers.clone();
        let mut cancelled = Terminal { reader:Cursor::new(b":cancel\n"), writer:Vec::new(), language:InterfaceLanguage::Ko };
        assert!(!collect(&store, &mut progress, &catalog().unwrap().questions, &options(root.path()), &mut cancelled).unwrap());
        assert_eq!(progress.answers, original);
        assert!(!progress.answered_questions.contains_key("user-section"));
        let mut resumed = Terminal { reader:Cursor::new(b"persona\nfriendly\n"), writer:Vec::new(), language:InterfaceLanguage::Ko };
        let mut options = options(root.path()); options.resume=true;
        assert!(collect(&store, &mut progress, &catalog().unwrap().questions, &options, &mut resumed).unwrap());
        assert_eq!(progress.answers["persona"]["id"], "friendly");
        assert_eq!(progress.selected_section.as_deref(), Some("persona"));
        assert_eq!(progress.answers["selected_hosts"], original["selected_hosts"]);
        assert!(!root.path().join(user_setup::USER_SETUP_RELATIVE).exists());
        let catalog = catalog().unwrap();
        let menu = catalog.questions.iter().find(|q| q.id=="user-section").unwrap();
        for option in &menu.options {
            assert!(option.value=="all" || !catalog.section(PreferenceScope::User, &option.value).is_empty());
        }
    }

    #[test]
    fn invalid_interaction_options_are_rejected_before_opening_settings() {
        let args = |tail: &[&str]| {
            [
                vec!["--scope".to_owned(), "user".to_owned()],
                tail.iter().map(|s| (*s).to_owned()).collect(),
            ]
            .concat()
        };
        assert!(parse(&args(&["--interactive", "--output", "json"])).is_err());
        assert!(parse(&args(&["--interactive", "--non-interactive"])).is_err());
        assert!(parse(&args(&["--show", "--resume"])).is_err());
    }

    #[test]
    fn invalid_structured_choice_repeats_only_that_question_without_saving_it() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let catalog = catalog().unwrap();
        let question = catalog
            .questions
            .iter()
            .find(|q| q.id == "user-skills")
            .unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new(b"{\"mode\":\"invalid\"}\n{\"mode\":\"all\"}\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(ask(&store, &mut progress, question, &mut terminal).unwrap());
        assert_eq!(progress.answers["skills"], json!({"mode":"all"}));
        assert_eq!(progress.answered_questions.len(), 1);
        assert!(String::from_utf8(terminal.writer)
            .unwrap()
            .contains("Enter this answer again"));
    }
}
