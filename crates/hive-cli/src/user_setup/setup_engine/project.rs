//! Project terminal adapter. Rendering, ownership checks, and registry writes stay in the core path.
use super::terminal::Terminal;
use crate::user_setup::{
    self,
    progress_store::{ProgressPhase, ProgressStore},
    InterfaceLanguage, SetupError,
};
use crate::ActionResult;
use hive_core::setup::{InteractionMode, InteractionRequest};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    io::{self, BufRead, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

mod activation;
mod capabilities;
mod existing;
mod extras;
mod input;
mod preferences;
mod questions;
mod file_input;
pub(in crate::user_setup) use file_input::execute as execute_file;
#[cfg(test)]
mod tests;

// CLI flags are validated together before any write or input wait.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
struct Options {
    target: PathBuf,
    root: PathBuf,
    interaction: InteractionRequest,
    resume: bool,
    show: bool,
    dry_run: bool,
    describe: bool,
    section: Option<String>,
    capabilities: Option<PathBuf>,
    reconfigure_roles: BTreeSet<String>,
}

fn parse(arguments: &[String]) -> Result<Options, SetupError> {
    let mut options = Options {
        target: PathBuf::new(),
        root: user_setup::resolve_user_root()?,
        interaction: InteractionRequest::default(),
        resume: false,
        show: false,
        dry_run: false,
        describe: false,
        section: None,
        capabilities: None,
        reconfigure_roles: BTreeSet::new(),
    };
    let mut seen = BTreeSet::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if !seen.insert(option) {
            return Err(SetupError::Input("duplicate project setup option".into()));
        }
        match option {
            "--interactive" => options.interaction.interactive = true,
            "--non-interactive" => options.interaction.non_interactive = true,
            "--resume" => options.resume = true,
            "--show" => options.show = true,
            "--dry-run" => options.dry_run = true,
            "--describe" => options.describe = true,
            "--target" | "--user-root" | "--output" | "--section" | "--capabilities" => {
                let value = arguments
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| {
                        SetupError::Input("missing project setup option value".into())
                    })?;
                match option {
                    "--target" => options.target = PathBuf::from(value),
                    "--user-root" => options.root = PathBuf::from(value),
                    "--output" if value == "json" => options.interaction.json = true,
                    "--section" => options.section = Some(value.clone()),
                    "--capabilities" => options.capabilities = Some(PathBuf::from(value)),
                    _ => {
                        return Err(SetupError::Input(
                            "project setup output must be json".into(),
                        ))
                    }
                }
                index += 1;
            }
            _ => return Err(SetupError::Input("unknown project setup option".into())),
        }
        index += 1;
    }
    options
        .interaction
        .resolve(false, false)
        .map_err(SetupError::Input)?;
    if options.target.as_os_str().is_empty()
        || (options.show && (options.resume || options.dry_run || options.describe))
        || (options.describe && (options.resume || options.dry_run || !options.interaction.json))
        || (options.section.is_some() && !options.describe)
    {
        return Err(SetupError::Input(
            "invalid project setup scope or mode".into(),
        ));
    }
    Ok(options)
}

pub(in crate::user_setup) fn run(arguments: &[String]) -> ExitCode {
    let json_output = arguments
        .windows(2)
        .any(|pair| pair == ["--output", "json"]);
    let mut terminal = Terminal {
        reader: io::stdin().lock(),
        writer: io::stderr().lock(),
        language: super::messages::language(arguments),
    };
    let mut result = parse(arguments)
        .and_then(|mut options| {
            hive_core::ensure_consumer_target(&options.target)
                .map_err(|error| SetupError::Input(error.to_string()))?;
            options.target =
                crate::user_install::open_canonical_user_root_for_setup(&options.target)
                    .map_err(SetupError::Conflict)?
                    .0;
            options.root = crate::user_install::open_canonical_user_root_for_setup(&options.root)
                .map_err(SetupError::Conflict)?
                .0;
            let mode = options
                .interaction
                .resolve(io::stdin().is_terminal(), io::stderr().is_terminal())
                .map_err(SetupError::Input)?;
            let mut result = execute(&options, mode, &mut terminal)
                .unwrap_or_else(|error| super::messages::failure(&error, terminal.language));
            if result.exit_code != 0 {
                result.next_action = Some(resume_command(&options));
            }
            Ok(result)
        })
        .unwrap_or_else(|error| super::messages::failure(&error, terminal.language));
    result.action = "SetupHarness";
    if json_output {
        return crate::emit_action_result(&result);
    }
    let _ = writeln!(terminal.writer, "{}", result.message);
    if let Some(next) = result.next_action {
        let _ = writeln!(terminal.writer, "{next}");
    }
    ExitCode::from(result.exit_code)
}

fn resume_command(options: &Options) -> String {
    let quote = |path: &Path| {
        if cfg!(windows) {
            path.to_string_lossy().replace('\'', "''")
        } else {
            path.to_string_lossy().replace('\'', "'\"'\"'")
        }
    };
    format!(
        "hive setup --target '{}' --user-root '{}' --resume",
        quote(&options.target),
        quote(&options.root)
    )
}

fn pending(language: InterfaceLanguage) -> ActionResult {
    report(
        "needs-input",
        2,
        "hive.project-setup-input-pending",
        language,
        [
            "프로젝트 설정이 아직 끝나지 않았습니다. 터미널에서 이어서 진행해 주세요.",
            "Project setup is pending. Continue in a terminal.",
        ],
        None,
    )
}

fn report(
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    language: InterfaceLanguage,
    message: [&str; 2],
    data: Option<Value>,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "SetupHarness",
        status,
        exit_code,
        code,
        message: message[usize::from(language == InterfaceLanguage::En)].into(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data,
    }
}

fn execute<R: BufRead, W: Write>(
    options: &Options,
    mode: InteractionMode,
    terminal: &mut Terminal<R, W>,
) -> Result<ActionResult, SetupError> {
    hive_core::ensure_consumer_target(&options.target)
        .map_err(|error| SetupError::Input(error.to_string()))?;
    let (target, project) =
        crate::user_install::open_canonical_user_root_for_setup(&options.target)
            .map_err(SetupError::Conflict)?;
    let user = crate::user_install::open_user_root_for_setup(&options.root)
        .map_err(SetupError::Conflict)?;
    let mut global_config = user_setup::load_operational_config(&user)?;
    if let Some(config) = &global_config {
        terminal.language = config.interface_language;
    }
    let saved = existing::read(&project)?;
    let store = ProgressStore::open(&options.root, Some(&target))?;
    if options.show || options.describe {
        return preferences::inspect(options, &store, saved.as_deref(), terminal.language);
    }
    if mode != InteractionMode::Terminal {
        return resume_without_input(options, &store, terminal.language);
    }
    if global_config.is_none() {
        terminal.say(
            "프로젝트에 사용할 전역 설정부터 진행합니다.",
            "Complete global settings for this project first.",
        )?;
        let result = super::interactive::execute_update(&options.root, mode, terminal)?;
        if result.exit_code != 0 {
            return Ok(result);
        }
        global_config = user_setup::load_operational_config(&user)?;
    }
    store.recover_pending_state()?;
    let mut snapshot = store.load()?;
    let global = global_preferences(&options.root)?;
    let host = global_config
        .as_ref()
        .and_then(|config| config.selected_hosts.first())
        .map_or("codex", |host| host.as_str());
    let mut progress = match snapshot.progress.clone() {
        Some(progress) if progress.phase != ProgressPhase::Complete => {
            terminal.language = questions::language(&progress.answers, terminal.language);
            if !options.resume {
                return Ok(pending(terminal.language));
            }
            if matches!(
                progress.phase,
                ProgressPhase::Applying | ProgressPhase::RecoverableFailure
            ) {
                return finish_existing_apply(
                    options,
                    &store,
                    &progress,
                    &global,
                    terminal.language,
                );
            }
            if let Some(refreshed) = store.refresh_draft_questions(&snapshot)? {
                snapshot = refreshed;
            }
            let progress = snapshot.progress.clone().expect("pending record");
            if !store.resume_ready(&progress) {
                return Ok(pending(terminal.language));
            }
            progress
        }
        Some(progress) if options.resume => {
            return completed_resume(options, &progress, &global, terminal.language);
        }
        _ if saved.is_some() => store.new_progress(
            "project-mode".into(),
            existing::answers(saved.as_deref().expect("saved answers"))?,
        )?,
        _ => store.new_progress(
            "project-mode".into(),
            questions::defaults(&target, &global, host),
        )?,
    };
    if !questions::collect(&store, &mut snapshot, &mut progress, &global, terminal)? {
        return Ok(pending(terminal.language));
    }
    if !existing::bind_knowledge(options, &store, &mut snapshot, &mut progress, terminal)?
        || !existing::confirm_roles(options, &store, &mut snapshot, &mut progress, terminal)?
        || !extras::configure(options, &store, &mut snapshot, &mut progress, terminal)?
    {
        return Ok(pending(terminal.language));
    }
    let mut confirmed = options.clone();
    confirmed
        .reconfigure_roles
        .clone_from(&progress.reconfigure_roles);
    apply(
        &confirmed,
        &store,
        &mut snapshot,
        &mut progress,
        &global,
        terminal,
    )
}

fn resume_without_input(
    options: &Options,
    store: &ProgressStore,
    language: InterfaceLanguage,
) -> Result<ActionResult, SetupError> {
    if !options.resume {
        return Ok(pending(language));
    }
    let Some(progress) = store.load()?.progress else {
        return Ok(pending(language));
    };
    let language = questions::language(&progress.answers, language);
    if progress.phase == ProgressPhase::Complete {
        return completed_resume(
            options,
            &progress,
            &global_preferences(&options.root)?,
            language,
        );
    }
    if matches!(
        progress.phase,
        ProgressPhase::Applying | ProgressPhase::RecoverableFailure
    ) && progress.project_context.is_some()
    {
        store.recover_pending_state()?;
        return finish_existing_apply(
            options,
            store,
            &progress,
            &global_preferences(&options.root)?,
            language,
        );
    }
    Ok(pending(language))
}

fn global_preferences(root: &Path) -> Result<hive_render::GlobalProjectPreferences, SetupError> {
    let mut global = user_setup::project_preferences(root).map_err(SetupError::Conflict)?;
    global.codex_plugin_files =
        crate::user_install::validated_codex_plugin_files(root).unwrap_or_default();
    Ok(global)
}

fn prepare_files(
    options: &Options,
    answers: &Value,
) -> Result<(tempfile::NamedTempFile, tempfile::NamedTempFile, Value), SetupError> {
    let answer_file = write_answers(answers)?;
    let (matrix, facts) = if let Some(path) = &options.capabilities {
        let bytes = user_setup::read_bounded_regular(path, user_setup::MAX_USER_SETUP_BYTES)?;
        hive_core::run::CapabilityResolution::parse_json(&bytes)
            .map_err(|e| SetupError::Verification(e.to_string()))?;
        (
            serde_json::from_slice(&bytes)
                .map_err(|_| SetupError::Input("invalid capabilities".into()))?,
            json!({"capabilities":"provided-document"}),
        )
    } else if let Some(matrix) = existing::capabilities(&options.target, answers)? {
        (
            matrix,
            json!({"capabilities":"stored-project-document","execution_capabilities_revalidated":false}),
        )
    } else {
        capabilities::inspect(
            answers["primary_host"]
                .as_str()
                .ok_or_else(|| SetupError::Input("project host missing".into()))?,
        )?
    };
    let mut capability_file =
        tempfile::NamedTempFile::new().map_err(|e| SetupError::Internal(e.to_string()))?;
    serde_json::to_writer(capability_file.as_file_mut(), &matrix)
        .map_err(|e| SetupError::Internal(e.to_string()))?;
    Ok((answer_file, capability_file, facts))
}

fn write_answers(answers: &Value) -> Result<tempfile::NamedTempFile, SetupError> {
    let mut file =
        tempfile::NamedTempFile::new().map_err(|e| SetupError::Internal(e.to_string()))?;
    serde_json::to_writer(file.as_file_mut(), answers)
        .map_err(|e| SetupError::Internal(e.to_string()))?;
    Ok(file)
}

fn request<'a>(
    options: &'a Options,
    answers: &'a Path,
    caps: &'a Path,
    global: &hive_render::GlobalProjectPreferences,
    mode: hive_render::SetupMode,
) -> hive_render::SetupRequest<'a> {
    hive_render::SetupRequest {
        target: &options.target,
        answers,
        capabilities: caps,
        mode,
        reconfigure_roles: options.reconfigure_roles.clone(),
        global_preferences: Some(global.clone()),
    }
}

fn execute_core(
    request: &hive_render::SetupRequest<'_>,
    root: &Path,
    global: &hive_render::GlobalProjectPreferences,
) -> Result<hive_render::SetupOutcome, SetupError> {
    crate::execute_setup_and_registry(request, root, request.target, global.wiki_enabled)
        .map_err(|e| SetupError::Verification(e.to_string()))
}

fn prepare_settings(options:&Options, answers:&Path, caps:&Path, global:&hive_render::GlobalProjectPreferences)
    -> Result<hive_render::PreparedSetup, SetupError> {
    prepare_settings_raw(options,answers,caps,global).map_err(|error| match error {
        hive_render::RenderError::Input(message) => SetupError::Input(message),
        hive_render::RenderError::Conflict(message) | hive_render::RenderError::Safety(message) => SetupError::Conflict(message),
        hive_render::RenderError::Verification(message) | hive_render::RenderError::Unsupported(message) => SetupError::Verification(message),
        hive_render::RenderError::Internal(message) | hive_render::RenderError::Rollback(message) => SetupError::Internal(message),
    })
}
fn prepare_settings_raw(
    options: &Options,
    answers: &Path,
    caps: &Path,
    global: &hive_render::GlobalProjectPreferences,
) -> Result<hive_render::PreparedSetup, hive_render::RenderError> {
    let target = crate::user_install::open_user_root_for_setup(&options.target)
        .map_err(hive_render::RenderError::Conflict)?;
    let existing = crate::user_install::read_user_setup_file(
        &target,
        Path::new(".hive/setup-answers.yml"),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(hive_render::RenderError::Conflict)?;
    let request = request(
        options,
        answers,
        caps,
        global,
        hive_render::SetupMode::DryRun,
    );
    let mut prepared = if existing.is_some() {
        crate::project_upgrade::prepare_reconfiguration(&request)?
    } else {
        hive_render::prepare_setup(&request)?
    };
    if let Some(preferences) = prepared.outcome.effective_preferences.as_ref() {
        crate::reconcile_project_registry(
            &options.root,
            &options.target,
            preferences,
            hive_render::SetupMode::DryRun,
            global.wiki_enabled,
            &mut prepared.outcome.changed_paths,
        )
        .map_err(|error| hive_render::RenderError::Verification(error.to_string()))?;
    }
    Ok(prepared)
}

#[allow(clippy::too_many_arguments)]
fn apply<R: BufRead, W: Write>(
    options: &Options,
    store: &ProgressStore,
    snapshot: &mut user_setup::progress_store::ProgressSnapshot,
    progress: &mut user_setup::progress_store::SetupProgress,
    global: &hive_render::GlobalProjectPreferences,
    terminal: &mut Terminal<R, W>,
) -> Result<ActionResult, SetupError> {
    let (answers, caps, facts) = prepare_files(options, &progress.answers)?;
    let preview = prepare_settings(options, answers.path(), caps.path(), global)?.outcome;
    terminal.say(
        "프로젝트 설정 미리보기: 선택한 대상에만 Hive 파일 작성. 사용자 파일 보존.",
        "Project preview: write Hive files only in the selected target; preserve user files.",
    )?;
    questions::preview(&progress.answers, global, terminal)?;
    writeln!(
        terminal.writer,
        "{}: {}",
        if terminal.language == InterfaceLanguage::Ko {
            "변경 파일 수"
        } else {
            "Changed file count"
        },
        preview.changed_paths.len()
    )
    .map_err(|e| SetupError::Internal(e.to_string()))?;
    if facts["capabilities"] == "stored-project-document" {
        terminal.say(
            "기존 프로젝트의 기능 자료를 사용합니다. 실제 실행 지원을 새로 검증한 결과는 아닙니다.",
            "Using the project's saved capability data. Runtime support was not revalidated.",
        )?;
    } else if options.capabilities.is_none() {
        terminal.say("앱 실행 파일 확인과 실행 기능 지원은 별개입니다. 실행 기능은 아직 검증되지 않았습니다.","Executable presence is separate from execution support. Execution capabilities remain unverified.")?;
    }
    progress.phase = ProgressPhase::Previewed;
    *snapshot = store.save(snapshot, progress)?;
    if options.dry_run || !terminal.confirm()? {
        return Ok(pending(terminal.language));
    }
    let cap = crate::user_install::open_user_root_for_setup(&options.root)
        .map_err(SetupError::Conflict)?;
    let _lease = crate::user_install::lock_user_setup(&cap).map_err(SetupError::Conflict)?;
    if global_preferences(&options.root)? != *global {
        return Err(SetupError::Conflict(
            "global settings changed after the preview; review the current values before applying"
                .into(),
        ));
    }
    preferences::accept_preview_defaults(&mut progress.answers)?;
    let answers = write_answers(&progress.answers)?;
    let capability_bytes =
        user_setup::read_bounded_regular(caps.path(), user_setup::MAX_USER_SETUP_BYTES)?;
    let prepared = prepare_settings(options, answers.path(), caps.path(), global)?;
    activation::verify_preview(&preview, &prepared.outcome)?;
    let outcome = commit_prepared(options,store,snapshot,progress,global,&capability_bytes,prepared)?;
    Ok(report(
        "success",
        0,
        "hive.project-setup-complete",
        terminal.language,
        [
            "프로젝트 설정 적용·검증 완료",
            "Project settings applied and verified",
        ],
        Some(json!({"changed_file_count":outcome.len(),"model_calls":0,"host":facts})),
    ))
}

#[allow(clippy::too_many_arguments)]
fn commit_prepared(options:&Options,store:&ProgressStore,snapshot:&mut user_setup::progress_store::ProgressSnapshot,
    progress:&mut user_setup::progress_store::SetupProgress,global:&hive_render::GlobalProjectPreferences,
    capability_bytes:&[u8],prepared:hive_render::PreparedSetup) -> Result<Vec<String>,SetupError> {
    activation::bind_context(progress, global, capability_bytes, &prepared.outcome)?;
    progress.phase = ProgressPhase::Applying;
    *snapshot = store.save(snapshot, progress)?;
    store.record_applying(progress)?;
    let outcome = activation::apply(options, store, progress, global, Some(prepared));
    let changed = match outcome {
        Ok(value) => value,
        Err(error) => {
            store.record_failure(progress)?;
            return Err(error);
        }
    };
    ProgressStore::open(&options.root, Some(&options.target))?.record_completion(progress)?;
    Ok(changed)
}

fn finish_existing_apply(
    options: &Options,
    store: &ProgressStore,
    progress: &user_setup::progress_store::SetupProgress,
    global: &hive_render::GlobalProjectPreferences,
    language: InterfaceLanguage,
) -> Result<ActionResult, SetupError> {
    let language = questions::language(&progress.answers, language);
    if !store.questions_match(progress) {
        return Ok(pending(language));
    }
    if progress.project_context.is_some() {
        let mut confirmed = options.clone();
        confirmed
            .reconfigure_roles
            .clone_from(&progress.reconfigure_roles);
        let root = crate::user_install::open_user_root_for_setup(&options.root)
            .map_err(SetupError::Conflict)?;
        let _lease = crate::user_install::lock_user_setup(&root).map_err(SetupError::Conflict)?;
        activation::apply(&confirmed, store, progress, global, None)?;
        ProgressStore::open(&options.root, Some(&options.target))?.record_completion(progress)?;
        return Ok(report(
            "success",
            0,
            "hive.project-setup-complete",
            language,
            [
                "남은 프로젝트 설정 적용·검증 완료",
                "Remaining project settings applied and verified",
            ],
            Some(json!({"model_calls":0,"resumed":true})),
        ));
    }
    store.projection_binding(progress, None)?;
    let (answers, caps, facts) = prepare_files(options, &progress.answers)?;
    execute_core(
        &request(
            options,
            answers.path(),
            caps.path(),
            global,
            hive_render::SetupMode::Validate,
        ),
        &options.root,
        global,
    )?;
    ProgressStore::open(&options.root, Some(&options.target))?.record_completion(progress)?;
    Ok(report(
        "success",
        0,
        "hive.project-setup-complete",
        language,
        [
            "저장된 프로젝트 적용·검증을 확인했습니다.",
            "Saved project setup was applied and verified.",
        ],
        Some(json!({"model_calls":0,"host":facts})),
    ))
}

fn completed_resume(
    options: &Options,
    progress: &user_setup::progress_store::SetupProgress,
    global: &hive_render::GlobalProjectPreferences,
    language: InterfaceLanguage,
) -> Result<ActionResult, SetupError> {
    let (_, cap) = crate::user_install::open_canonical_user_root_for_setup(&options.target)
        .map_err(SetupError::Conflict)?;
    let bytes = crate::user_install::read_user_setup_file(
        &cap,
        Path::new(".hive/setup-answers.yml"),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    .ok_or_else(|| SetupError::Verification("completed project settings are missing".into()))?;
    let value = serde_yaml::from_slice(&bytes)
        .map_err(|_| SetupError::Verification("invalid completed project settings".into()))?;
    let language = questions::language(&value, language);
    let answers = write_answers(&value)?;
    let capabilities = crate::user_install::read_user_setup_file(
        &cap,
        Path::new(".hive/config/capability-resolution.yml"),
        user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    .ok_or_else(|| {
        SetupError::Verification("completed project capability data is missing".into())
    })?;
    let matrix: Value = serde_yaml::from_slice(&capabilities)
        .map_err(|_| SetupError::Verification("invalid completed project capabilities".into()))?;
    let caps = write_answers(&matrix)?;
    execute_core(
        &request(
            options,
            answers.path(),
            caps.path(),
            global,
            hive_render::SetupMode::Validate,
        ),
        &options.root,
        global,
    )?;
    Ok(report(
        "success",
        0,
        "hive.project-setup-complete",
        language,
        [
            "이미 적용·검증한 프로젝트 설정입니다.",
            "Project setup was already applied and verified.",
        ],
        Some(
            json!({"operation_id":progress.operation_id,"model_calls":0,"capabilities":"stored-project-document"}),
        ),
    ))
}
