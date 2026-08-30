use super::{emit_action_result, ActionResult, Evidence};
use cap_std::fs::Dir;
use hive_core::native_workflow::{JudgeInvocationPolicy, JudgeRoute};
use hive_core::sha256_digest;
use hive_projection::{
    canonical_builtin_skill_name, compile_user_projection_localized, embedded_catalog,
    historical_builtin_skills, retired_builtin_skill_names, DescriptorLanguage,
    Host as ProjectionHost,
};
use hive_render::GlobalProjectPreferences;
use hive_update::{three_way_merge, three_way_merge_hive_directive, MergeDisposition};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USER_SETUP_RELATIVE: &str = ".hive/config/user-setup.yml";
const USER_SETUP_PROGRESS_RELATIVE: &str = ".hive/config/user-setup-progress.yml";
const USER_FEATURE_ANSWERS_RELATIVE: &str = ".hive/config/user-feature-answers.yml";
const USER_PROJECTION_MANIFEST_RELATIVE: &str = ".hive/install/user-projection.json";
const LEGACY_USER_SETUP_REVIEW_RELATIVE: &str = ".hive/config/user-setup-review.yml";
const LEGACY_USER_SETUP_REVIEW: &[u8] = b"schema_version: 1\nsource_version: 0.7.0\nsetup_required: true\nwiki_markdown_preserved: true\nlegacy_skill_projection: all-built-ins\n";
const USER_SETUP_SCHEMA: &str = include_str!("../../../schemas/user-setup.schema.json");
const USER_SETUP_CATALOG_SCHEMA: &str =
    include_str!("../../../schemas/user-setup-catalog.schema.json");
const USER_SETUP_CATALOG: &str = include_str!("../../../harness/user-setup/catalog.yml");
const MAX_ANSWERS_BYTES: u64 = 1024 * 1024;
const MAX_USER_SETUP_BYTES: u64 = 1024 * 1024;
const EXPEDITED_DEFAULT_USAGE_THRESHOLD: u8 = 20;
/// Historical 0.8.x preferences omitted this setting. This compatibility value is never offered
/// as a new-setup default: every new setup answer selects its own threshold.
const LEGACY_080_USAGE_THRESHOLD: u8 = 20;
const USER_PROJECTION_090_TEST3_SETUP_HIVE: &[u8] =
    include_bytes!("../../../harness/user-bases/0.9.0-test.3/skills/setup-hive/SKILL.md");
const HISTORICAL_SKILL_RELEASES: [&str; 14] = [
    "0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0", "0.7.0", "0.8.0", "0.9.0", "0.9.1",
    "0.9.2", "0.9.3", "0.9.4", "0.9.5",
];

const USER_SETUP_USAGE: &str = "\
Configure or validate Aigent Hive user preferences.

USAGE:
    hive setup --scope user --describe --output json
    hive setup --scope user (--answers|--quick-answers) <yml> (--dry-run|--apply|--validate) [--user-root <dir>] --output json
    hive setup feature status|claim|prompt --id vector-search [--user-root <dir>] --output json
    hive setup feature answer --id vector-search --answer yes|no [--user-root <dir>] --output json
    hive setup --progress save --scope user --step <step> (--answers|--quick-answers) <yml> [--user-root <dir>] --output json
    hive setup --progress status|clear --scope user [--user-root <dir>] --output json

MODES:
    --dry-run    Validate answers and preview the owned user configuration change
    --apply      Validate and atomically apply the owned user configuration
    --validate   Validate the installed operational user configuration
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SetupMode {
    DryRun,
    Apply,
    Validate,
}

#[derive(Debug)]
struct Arguments {
    answers: PathBuf,
    mode: SetupMode,
    user_root: PathBuf,
    root_cap: Dir,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum UserSetupState {
    Bootstrap,
    SetupRequired,
    Operational,
}

impl UserSetupState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::SetupRequired => "setup-required",
            Self::Operational => "operational",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum InterfaceLanguage {
    En,
    Ko,
}

impl InterfaceLanguage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
        }
    }

    const fn descriptor_language(self) -> DescriptorLanguage {
        match self {
            Self::En => DescriptorLanguage::En,
            Self::Ko => DescriptorLanguage::Ko,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WikiLanguage {
    En,
    Ko,
    Both,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WikiBackend {
    #[default]
    Markdown,
    Notion,
}

impl WikiBackend {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Notion => "notion",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NotionWikiPreferences {
    pub(crate) workspace_id: String,
    pub(crate) scope_id: String,
    pub(crate) local_index_consent: bool,
}

impl WikiLanguage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
            Self::Both => "both",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WikiPreferences {
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    pub(crate) language: WikiLanguage,
    #[serde(default)]
    pub(crate) backend: WikiBackend,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) notion: Option<NotionWikiPreferences>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CatalogSelection {
    pub(crate) id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) custom_description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UserProfile {
    pub(crate) contexts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SelectedHost {
    Codex,
    Claude,
    Antigravity,
}

impl SelectedHost {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Antigravity => "antigravity",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SkillSelectionMode {
    All,
    Individual,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillPreferences {
    pub(crate) mode: SkillSelectionMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) selected: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DiscordGuardPreferences {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) webhook_url_env: Option<String>,
    #[serde(default)]
    pub(crate) request_privacy: DiscordRequestPrivacy,
    #[serde(default = "default_discord_message_fields")]
    pub(crate) message_fields: Vec<DiscordMessageField>,
}

impl Default for DiscordGuardPreferences {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url_env: None,
            request_privacy: DiscordRequestPrivacy::Summary,
            message_fields: default_discord_message_fields(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DiscordRequestPrivacy {
    #[default]
    Summary,
    RawPrompt,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DiscordMessageField {
    RemainingUsage,
    Project,
    Request,
    Progress,
    Host,
    Resume,
    MeasuredAt,
    Evidence,
}

impl DiscordMessageField {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::RemainingUsage => "remaining-usage",
            Self::Project => "project",
            Self::Request => "request",
            Self::Progress => "progress",
            Self::Host => "host",
            Self::Resume => "resume",
            Self::MeasuredAt => "measured-at",
            Self::Evidence => "evidence",
        }
    }
}

fn default_discord_message_fields() -> Vec<DiscordMessageField> {
    vec![
        DiscordMessageField::RemainingUsage,
        DiscordMessageField::Project,
        DiscordMessageField::Request,
        DiscordMessageField::Progress,
        DiscordMessageField::Host,
        DiscordMessageField::Resume,
    ]
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UsageGuardPreferences {
    #[serde(default)]
    pub(crate) enabled: bool,
    pub(crate) stop_remaining_percent: u8,
    #[serde(default)]
    pub(crate) codexbar_fallback_enabled: bool,
    #[serde(default)]
    pub(crate) discord: DiscordGuardPreferences,
    /// Stable registered project identity to an earlier-stop threshold. The key is never a path.
    #[serde(default)]
    pub(crate) project_overrides: BTreeMap<String, u8>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateCheckPreferences {
    #[serde(default)]
    pub(crate) enabled: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum JudgeInvocation {
    #[default]
    Explicit,
    Implicit,
}

impl JudgeInvocation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Implicit => "implicit",
        }
    }

    const fn policy(self) -> JudgeInvocationPolicy {
        match self {
            Self::Explicit => JudgeInvocationPolicy::Explicit,
            Self::Implicit => JudgeInvocationPolicy::Implicit,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UserSetupConfig {
    pub(crate) schema_version: u32,
    pub(crate) interface_language: InterfaceLanguage,
    pub(crate) wiki: WikiPreferences,
    pub(crate) profile: UserProfile,
    pub(crate) persona: CatalogSelection,
    pub(crate) selected_hosts: Vec<SelectedHost>,
    pub(crate) skills: SkillPreferences,
    #[serde(default)]
    pub(crate) update_check: UpdateCheckPreferences,
    #[serde(default)]
    pub(crate) judge_invocation: JudgeInvocation,
    pub(crate) usage_guard: UsageGuardPreferences,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum VectorFeatureAnswer {
    Yes,
    No,
}

/// Read the saved vector-search preference without changing setup or feature-answer state.
pub(crate) fn vector_search_enabled(user_root: &Path) -> Result<bool, String> {
    let root = super::user_install::open_user_root_for_setup(user_root)?;
    let (_, answers) = load_feature_answers(&root).map_err(|error| format!("{error:?}"))?;
    Ok(answers.vector_search == Some(VectorFeatureAnswer::Yes))
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct UserFeatureAnswers {
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vector_search: Option<VectorFeatureAnswer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    introduced_in: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserSetupProgress {
    schema_version: u32,
    step: String,
    answers: JsonValue,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalizedText {
    en: String,
    ko: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogChoice {
    id: String,
    display_name: LocalizedText,
    description: LocalizedText,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillDependency {
    skill: String,
    requires: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserSetupCatalog {
    schema_version: u32,
    profiles: Vec<CatalogChoice>,
    personas: Vec<CatalogChoice>,
    mandatory_skills: Vec<String>,
    skill_dependencies: Vec<SkillDependency>,
    optional_third_party_skills: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserProjectionManifest {
    schema_version: u32,
    product_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    package_version: Option<String>,
    setup_digest: String,
    entries: Vec<UserProjectionEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    base_entries: Vec<UserProjectionBaseEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserProjectionEntry {
    path: String,
    digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserProjectionBaseEntry {
    path: String,
    digest: String,
    content: String,
}

struct AppliedProjection {
    changes: Vec<ProjectionChange>,
    changed_paths: Vec<String>,
    reports: Vec<UserProjectionPathReport>,
}

struct ProjectionChange {
    path: PathBuf,
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
struct UserProjectionPathReport {
    path: String,
    base_digest: Option<String>,
    local_digest: Option<String>,
    incoming_digest: Option<String>,
    final_digest: Option<String>,
    disposition: MergeDisposition,
    omitted_incoming_hunks: usize,
    local_priority: bool,
}

const fn default_true() -> bool {
    true
}

#[derive(Debug)]
pub(crate) enum SetupError {
    Input(String),
    Conflict(String),
    Verification(String),
    Internal(String),
}

impl SetupError {
    const fn status(&self) -> &'static str {
        match self {
            Self::Input(_) | Self::Internal(_) => "error",
            Self::Conflict(_) => "conflict",
            Self::Verification(_) => "verification-failed",
        }
    }

    const fn exit_code(&self) -> u8 {
        match self {
            Self::Input(_) => 2,
            Self::Conflict(_) => 3,
            Self::Verification(_) => 5,
            Self::Internal(_) => 10,
        }
    }

    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Input(message)
            | Self::Conflict(message)
            | Self::Verification(message)
            | Self::Internal(message) => message,
        }
    }
}

pub(crate) fn print_help() {
    print!("{USER_SETUP_USAGE}");
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.iter().any(|argument| argument == "--describe") {
        return run_describe(arguments);
    }
    if arguments.first().map(String::as_str) == Some("--progress") {
        return run_progress(&arguments[1..]);
    }
    if arguments.first().map(String::as_str) == Some("feature") {
        return run_feature(&arguments[1..]);
    }
    let result = parse(arguments)
        .and_then(|arguments| execute(&arguments))
        .unwrap_or_else(|error| failure(&error));
    emit_action_result(&result)
}

#[derive(Debug)]
enum FeatureAction {
    Status,
    Claim,
    Answer(VectorFeatureAnswer),
    Prompt,
}

#[derive(Debug)]
struct FeatureArguments {
    action: FeatureAction,
    root_cap: Dir,
}

fn run_feature(arguments: &[String]) -> ExitCode {
    let result = parse_feature(arguments)
        .and_then(|arguments| execute_feature(&arguments))
        .unwrap_or_else(|error| failure(&error));
    emit_action_result(&result)
}

fn parse_feature(arguments: &[String]) -> Result<FeatureArguments, SetupError> {
    let action = arguments.first().ok_or_else(|| {
        SetupError::Input("setup feature requires status, claim, answer, or prompt".to_owned())
    })?;
    let mut id = None;
    let mut answer = None;
    let mut output = None;
    let mut user_root = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| SetupError::Input(format!("missing value for {option}")))?;
        let slot = match option {
            "--id" => &mut id,
            "--answer" => &mut answer,
            "--output" => &mut output,
            "--user-root" => &mut user_root,
            _ => {
                return Err(SetupError::Input(format!(
                    "unknown setup feature option: {option}"
                )))
            }
        };
        if slot.replace(value.clone()).is_some() {
            return Err(SetupError::Input(format!("duplicate option: {option}")));
        }
        index += 2;
    }
    if id.as_deref() != Some("vector-search") || output.as_deref() != Some("json") {
        return Err(SetupError::Input(
            "setup feature requires --id vector-search and --output json".to_owned(),
        ));
    }
    let action = match action.as_str() {
        "status" if answer.is_none() => FeatureAction::Status,
        "claim" if answer.is_none() => FeatureAction::Claim,
        "prompt" if answer.is_none() => FeatureAction::Prompt,
        "answer" => match answer.as_deref() {
            Some("yes") => FeatureAction::Answer(VectorFeatureAnswer::Yes),
            Some("no") => FeatureAction::Answer(VectorFeatureAnswer::No),
            _ => {
                return Err(SetupError::Input(
                    "vector-search answer requires yes or no".to_owned(),
                ))
            }
        },
        _ => {
            return Err(SetupError::Input(
                "invalid setup feature action or answer".to_owned(),
            ))
        }
    };
    let user_root = user_root.map_or_else(resolve_user_root, |value| Ok(PathBuf::from(value)))?;
    let root_cap =
        super::user_install::open_user_root_for_setup(&user_root).map_err(SetupError::Conflict)?;
    Ok(FeatureArguments { action, root_cap })
}

fn load_feature_answers(root: &Dir) -> Result<(Option<Vec<u8>>, UserFeatureAnswers), SetupError> {
    let existing = super::user_install::read_user_setup_file(
        root,
        Path::new(USER_FEATURE_ANSWERS_RELATIVE),
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let answers = match existing.as_deref() {
        Some(bytes) => serde_yaml::from_slice(bytes).map_err(|error| {
            SetupError::Verification(format!("invalid user feature answers: {error}"))
        })?,
        None => UserFeatureAnswers {
            schema_version: 1,
            vector_search: None,
            introduced_in: None,
        },
    };
    if answers.schema_version != 1 {
        return Err(SetupError::Verification(
            "unsupported user feature answers schema".to_owned(),
        ));
    }
    Ok((existing, answers))
}

fn execute_feature(arguments: &FeatureArguments) -> Result<ActionResult, SetupError> {
    let (existing, mut answers) = load_feature_answers(&arguments.root_cap)?;
    let language = load_operational_config(&arguments.root_cap)?
        .map_or(InterfaceLanguage::En, |config| config.interface_language);
    let prior = answers.vector_search;
    let (code, message, changed_paths, prompt) = match arguments.action {
        FeatureAction::Status => (
            "hive.user-feature-status",
            "vector-search feature status inspected".to_owned(),
            Vec::new(),
            None,
        ),
        FeatureAction::Claim => (
            if prior.is_some() {
                "hive.user-feature-already-answered"
            } else {
                "hive.user-feature-question-claimed"
            },
            "vector-search onboarding question evaluated".to_owned(),
            Vec::new(),
            None,
        ),
        FeatureAction::Answer(answer) => {
            answers.vector_search = Some(answer);
            answers.introduced_in = Some(env!("CARGO_PKG_VERSION").to_owned());
            let desired = serde_yaml::to_string(&answers)
                .map_err(|error| {
                    SetupError::Internal(format!("cannot serialize user feature answers: {error}"))
                })?
                .into_bytes();
            super::user_install::replace_user_setup_file(
                &arguments.root_cap,
                Path::new(USER_FEATURE_ANSWERS_RELATIVE),
                existing.as_deref(),
                Some(&desired),
            )
            .map_err(SetupError::Conflict)?;
            (
                "hive.user-feature-answer-saved",
                "vector-search feature answer saved".to_owned(),
                vec![USER_FEATURE_ANSWERS_RELATIVE.to_owned()],
                None,
            )
        }
        FeatureAction::Prompt => {
            if prior != Some(VectorFeatureAnswer::Yes) {
                return Err(SetupError::Conflict(
                    "vector-search setup prompt requires a saved yes answer".to_owned(),
                ));
            }
            (
                "hive.user-feature-prompt-ready",
                "vector-search setup prompt prepared".to_owned(),
                Vec::new(),
                Some(vector_setup_prompt(language).to_owned()),
            )
        }
    };
    let answer = answers.vector_search.map(|value| match value {
        VectorFeatureAnswer::Yes => "yes",
        VectorFeatureAnswer::No => "no",
    });
    Ok(ActionResult {
        schema_version: 1,
        action: "ManageHiveUserFeature",
        status: "success",
        exit_code: 0,
        code,
        message,
        changed_paths,
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({
            "id":"vector-search",
            "answer":answer,
            "question_required":answer.is_none(),
            "prompt":prompt,
            "actual_runtime_or_index_state":"separate; inspect with hive knowledge vector status",
        })),
    })
}

fn vector_setup_prompt(language: InterfaceLanguage) -> &'static str {
    match language {
        InterfaceLanguage::En => "Set up Aigent Hive semantic search for my user-root knowledge and currently registered shared collections only. Do not inspect or include project-private, confidential, or newly discovered collections. First run the Hive vector preview, show its exact downloads, storage paths, Python requirement, and consent digest. If a supported existing Python is unavailable, explain the manual prerequisite and stop without changing my answer. If the preview is valid, use its exact consent digest to enable vector search, refresh existing knowledge, rebuild only the approved collections with resumable time limits, then verify semantic search, canonical citations, and FTS fallback. Do not install Python, use provider APIs or credentials, create a background service, or change canonical Markdown.",
        InterfaceLanguage::Ko => "내 사용자 전역 지식과 현재 등록된 공유 모음에만 Aigent Hive 의미 검색을 설정해줘. 프로젝트 비공개·기밀·새로 발견한 모음은 읽거나 포함하지 마. 먼저 Hive 벡터 미리보기를 실행해 정확한 다운로드, 저장 경로, Python 조건, 동의 지문을 보여줘. 지원되는 기존 Python이 없으면 수동 준비 방법을 설명하고 내 답변은 바꾸지 말고 멈춰줘. 미리보기가 유효하면 정확한 동의 지문으로 벡터 검색을 활성화하고, 기존 지식을 갱신한 뒤 승인된 모음만 시간 제한을 두고 재개 가능하게 생성해줘. 마지막으로 의미 검색, 정본 인용, FTS 대체 경로를 확인해줘. Python 자동 설치, 제공자 API·자격 증명 사용, 상시 서비스 생성, 정본 Markdown 변경은 하지 마.",
    }
}

fn run_describe(arguments: &[String]) -> ExitCode {
    let valid = arguments.len() == 5
        && arguments.iter().any(|value| value == "--scope")
        && arguments.iter().any(|value| value == "user")
        && arguments.iter().any(|value| value == "--describe")
        && arguments.iter().any(|value| value == "--output")
        && arguments.iter().any(|value| value == "json");
    let result = if valid {
        describe_result().unwrap_or_else(|error| failure(&error))
    } else {
        failure(&SetupError::Input(
            "setup describe requires --scope user --describe --output json".to_owned(),
        ))
    };
    emit_action_result(&result)
}

fn describe_result() -> Result<ActionResult, SetupError> {
    let schema: JsonValue = serde_json::from_str(USER_SETUP_SCHEMA).map_err(|error| {
        SetupError::Internal(format!("invalid embedded user setup schema: {error}"))
    })?;
    let catalog: JsonValue = serde_yaml::from_str(USER_SETUP_CATALOG).map_err(|error| {
        SetupError::Internal(format!("invalid embedded user setup catalog: {error}"))
    })?;
    let question_order = json!([
        "interface-language",
        "daily-update-check",
        "setup-mode",
        "wiki",
        "user-contexts",
        "persona",
        "hosts",
        "judge-invocation",
        "skills",
        "usage-guard",
        "discord"
    ]);
    let example = json!({
        "schema_version": 1,
        "interface_language": "en",
        "wiki": { "enabled": true, "language": "en", "backend": "markdown" },
        "profile": { "contexts": ["non-developer"] },
        "persona": { "id": "balanced" },
        "selected_hosts": ["codex"],
        "skills": { "mode": "all" },
        "update_check": { "enabled": false },
        "judge_invocation": "explicit",
        "usage_guard": { "enabled": true,
            "stop_remaining_percent": "<user-chosen-integer-1-to-99>",
            "codexbar_fallback_enabled": false,
            "discord": { "enabled": false, "request_privacy": "summary",
                "message_fields": default_discord_message_fields().iter().map(|field| field.as_str()).collect::<Vec<_>>() }
        }
    });
    let expedited_defaults = json!({
        "schema_version": 1,
        "interface_language": "en",
        "wiki": { "enabled": true, "language": "en", "backend": "markdown" },
        "profile": { "contexts": ["non-developer"] },
        "persona": { "id": "strict" },
        "selected_hosts": ["codex"],
        "skills": { "mode": "all" },
        "update_check": { "enabled": false },
        "judge_invocation": "explicit",
        "usage_guard": { "enabled": true,
            "stop_remaining_percent": EXPEDITED_DEFAULT_USAGE_THRESHOLD,
            "codexbar_fallback_enabled": false,
            "discord": { "enabled": false, "request_privacy": "summary",
                "message_fields": default_discord_message_fields().iter().map(|field| field.as_str()).collect::<Vec<_>>() }
        }
    });
    Ok(ActionResult {
        schema_version: 1,
        action: "DescribeHiveUserSetup",
        status: "success",
        exit_code: 0,
        code: "hive.user-setup-described",
        message: "user setup contract described".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "user-setup-catalog",
            locator: "harness/user-setup/catalog.yml".to_owned(),
            digest: sha256_digest(USER_SETUP_CATALOG.as_bytes()),
        }],
        next_action: None,
        data: Some(json!({
            "contract_digest": sha256_digest(format!("{USER_SETUP_SCHEMA}\n{USER_SETUP_CATALOG}").as_bytes()),
            "schema": schema,
            "catalog": catalog,
            "answer_template": example,
            "answer_template_notice": "Usage protection is recommended. In Custom setup, replace <user-chosen-integer-1-to-99> with the user's own value before validation; Expedited setup uses 20% remaining.",
            "expedited_defaults": expedited_defaults,
            "question_order": question_order,
        })),
    })
}

fn run_progress(arguments: &[String]) -> ExitCode {
    let result = parse_progress(arguments)
        .and_then(execute_progress)
        .unwrap_or_else(|error| failure(&error));
    emit_action_result(&result)
}

#[derive(Debug)]
enum ProgressAction {
    Save { step: String, answers: PathBuf },
    Status,
    Clear,
}

#[derive(Debug)]
struct ProgressArguments {
    action: ProgressAction,
    root_cap: Dir,
}

fn parse_progress(arguments: &[String]) -> Result<ProgressArguments, SetupError> {
    let action = arguments.first().ok_or_else(|| {
        SetupError::Input("setup progress requires save, status, or clear".to_owned())
    })?;
    let mut scope = None;
    let mut answers = None;
    let mut step = None;
    let mut output = None;
    let mut user_root = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| SetupError::Input(format!("missing value for {option}")))?;
        let slot = match option {
            "--scope" => &mut scope,
            "--answers" | "--quick-answers" => &mut answers,
            "--step" => &mut step,
            "--output" => &mut output,
            "--user-root" => &mut user_root,
            _ => {
                return Err(SetupError::Input(format!(
                    "unknown setup progress option: {option}"
                )))
            }
        };
        if slot.replace(value.clone()).is_some() {
            return Err(SetupError::Input(format!("duplicate option: {option}")));
        }
        index += 2;
    }
    if scope.as_deref() != Some("user") || output.as_deref() != Some("json") {
        return Err(SetupError::Input(
            "setup progress requires --scope user and --output json".to_owned(),
        ));
    }
    let action = match action.as_str() {
        "save" => {
            let step = step.ok_or_else(|| {
                SetupError::Input("setup progress save requires --step".to_owned())
            })?;
            if step.is_empty()
                || step.len() > 80
                || !step
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(SetupError::Input(
                    "setup progress step must use lowercase letters, digits, or hyphens".to_owned(),
                ));
            }
            ProgressAction::Save {
                step,
                answers: PathBuf::from(answers.ok_or_else(|| {
                    SetupError::Input(
                        "setup progress save requires --answers or --quick-answers".to_owned(),
                    )
                })?),
            }
        }
        "status" => {
            if answers.is_some() || step.is_some() {
                return Err(SetupError::Input(
                    "setup progress status accepts no --answers or --step".to_owned(),
                ));
            }
            ProgressAction::Status
        }
        "clear" => {
            if answers.is_some() || step.is_some() {
                return Err(SetupError::Input(
                    "setup progress clear accepts no --answers or --step".to_owned(),
                ));
            }
            ProgressAction::Clear
        }
        _ => {
            return Err(SetupError::Input(
                "setup progress requires save, status, or clear".to_owned(),
            ))
        }
    };
    let user_root = user_root.map_or_else(resolve_user_root, |value| Ok(PathBuf::from(value)))?;
    let root_cap =
        super::user_install::open_user_root_for_setup(&user_root).map_err(SetupError::Conflict)?;
    Ok(ProgressArguments { action, root_cap })
}

fn execute_progress(arguments: ProgressArguments) -> Result<ActionResult, SetupError> {
    let relative = Path::new(USER_SETUP_PROGRESS_RELATIVE);
    let existing = super::user_install::read_user_setup_file(
        &arguments.root_cap,
        relative,
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    match arguments.action {
        ProgressAction::Save { step, answers } => {
            let answer_bytes = read_bounded_regular(&answers, MAX_ANSWERS_BYTES)?;
            let answers: JsonValue = serde_yaml::from_slice(&answer_bytes).map_err(|error| {
                SetupError::Input(format!("invalid user setup progress YAML: {error}"))
            })?;
            validate_progress_answers(&answers)?;
            let progress = UserSetupProgress {
                schema_version: 1,
                step,
                answers,
            };
            let desired = serde_yaml::to_string(&progress)
                .map_err(|error| {
                    SetupError::Internal(format!("cannot serialize setup progress: {error}"))
                })?
                .into_bytes();
            super::user_install::replace_user_setup_file(
                &arguments.root_cap,
                relative,
                existing.as_deref(),
                Some(&desired),
            )
            .map_err(SetupError::Conflict)?;
            Ok(progress_result(
                "SaveUserSetupProgress",
                "success",
                0,
                "hive.user-setup-progress-saved",
                "user setup progress saved",
                vec![USER_SETUP_PROGRESS_RELATIVE.to_owned()],
                Some(&progress),
            ))
        }
        ProgressAction::Status => {
            let progress = existing.as_deref().map(parse_progress_file).transpose()?;
            Ok(progress_result(
                "InspectUserSetupProgress",
                "success",
                0,
                "hive.user-setup-progress-status",
                "user setup progress inspected",
                Vec::new(),
                progress.as_ref(),
            ))
        }
        ProgressAction::Clear => {
            if existing.is_some() {
                super::user_install::replace_user_setup_file(
                    &arguments.root_cap,
                    relative,
                    existing.as_deref(),
                    None,
                )
                .map_err(SetupError::Conflict)?;
            }
            Ok(progress_result(
                "ClearUserSetupProgress",
                "success",
                0,
                "hive.user-setup-progress-cleared",
                "user setup progress cleared",
                existing
                    .map(|_| vec![USER_SETUP_PROGRESS_RELATIVE.to_owned()])
                    .unwrap_or_default(),
                None,
            ))
        }
    }
}

fn parse_progress_file(bytes: &[u8]) -> Result<UserSetupProgress, SetupError> {
    let progress: UserSetupProgress = serde_yaml::from_slice(bytes).map_err(|error| {
        SetupError::Verification(format!("invalid user setup progress: {error}"))
    })?;
    if progress.schema_version != 1 {
        return Err(SetupError::Verification(
            "unsupported user setup progress schema".to_owned(),
        ));
    }
    if progress.step.is_empty()
        || progress.step.len() > 80
        || !progress
            .step
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(SetupError::Verification(
            "invalid user setup progress step".to_owned(),
        ));
    }
    validate_progress_answers(&progress.answers)?;
    Ok(progress)
}

fn progress_result(
    action: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: &'static str,
    changed_paths: Vec<String>,
    progress: Option<&UserSetupProgress>,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status,
        exit_code,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({
            "pending": progress.is_some(),
            "step": progress.map(|value| value.step.as_str()),
            "answers": progress.map(|value| &value.answers),
        })),
    }
}

fn validate_progress_answers(answers: &JsonValue) -> Result<(), SetupError> {
    let object = answers.as_object().ok_or_else(|| {
        SetupError::Verification("user setup progress answers must be a YAML object".to_owned())
    })?;
    let allowed = [
        "schema_version",
        "interface_language",
        "wiki",
        "profile",
        "persona",
        "selected_hosts",
        "skills",
        "update_check",
        "usage_guard",
    ];
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(SetupError::Verification(
            "user setup progress contains an unknown setting".to_owned(),
        ));
    }
    if progress_contains_secret(answers) {
        return Err(SetupError::Verification(
            "user setup progress cannot contain a webhook URL, token, or secret".to_owned(),
        ));
    }
    Ok(())
}

fn progress_contains_secret(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(object) => object.iter().any(|(key, value)| {
            let normalized = key.to_ascii_lowercase();
            normalized.contains("token")
                || normalized.contains("secret")
                || (normalized == "webhook_url"
                    || (normalized == "url" && key != "webhook_url_env"))
                || progress_contains_secret(value)
        }),
        JsonValue::Array(values) => values.iter().any(progress_contains_secret),
        JsonValue::String(value) => value.starts_with("https://") || value.starts_with("http://"),
        _ => false,
    }
}

fn parse(arguments: &[String]) -> Result<Arguments, SetupError> {
    let mut scope = None;
    let mut answers = None;
    let mut mode = None;
    let mut output = None;
    let mut user_root = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dry-run" if mode.is_none() => {
                mode = Some(SetupMode::DryRun);
                index += 1;
            }
            "--apply" if mode.is_none() => {
                mode = Some(SetupMode::Apply);
                index += 1;
            }
            "--validate" if mode.is_none() => {
                mode = Some(SetupMode::Validate);
                index += 1;
            }
            "--dry-run" | "--apply" | "--validate" => {
                return Err(SetupError::Input(
                    "choose exactly one of --dry-run, --apply, or --validate".to_owned(),
                ));
            }
            option @ ("--scope" | "--answers" | "--quick-answers" | "--output" | "--user-root") => {
                let value = arguments
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| SetupError::Input(format!("missing value for {option}")))?;
                let slot = match option {
                    "--scope" => &mut scope,
                    "--answers" | "--quick-answers" => &mut answers,
                    "--output" => &mut output,
                    "--user-root" => &mut user_root,
                    _ => unreachable!(),
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(SetupError::Input(format!("duplicate option: {option}")));
                }
                index += 2;
            }
            option => return Err(SetupError::Input(format!("unknown setup option: {option}"))),
        }
    }
    if scope.as_deref() != Some("user") {
        return Err(SetupError::Input(
            "user setup requires --scope user".to_owned(),
        ));
    }
    if output.as_deref() != Some("json") {
        return Err(SetupError::Input(
            "user setup requires --output json".to_owned(),
        ));
    }
    let requested_user_root =
        user_root.map_or_else(resolve_user_root, |value| Ok(PathBuf::from(value)))?;
    let (user_root, root_cap) =
        super::user_install::open_canonical_user_root_for_setup(&requested_user_root)
            .map_err(SetupError::Conflict)?;
    Ok(Arguments {
        answers: PathBuf::from(
            answers.ok_or_else(|| SetupError::Input("missing --answers".to_owned()))?,
        ),
        mode: mode
            .ok_or_else(|| SetupError::Input("choose exactly one user setup mode".to_owned()))?,
        user_root,
        root_cap,
    })
}

fn resolve_user_root() -> Result<PathBuf, SetupError> {
    let value = if cfg!(windows) {
        env::var_os("USERPROFILE")
    } else {
        env::var_os("HOME")
    }
    .ok_or_else(|| SetupError::Input("cannot resolve the user home directory".to_owned()))?;
    Ok(PathBuf::from(value))
}

#[allow(clippy::too_many_lines)]
fn execute(arguments: &Arguments) -> Result<ActionResult, SetupError> {
    let answer_bytes = read_bounded_regular(&arguments.answers, MAX_ANSWERS_BYTES)?;
    let relative = Path::new(USER_SETUP_RELATIVE);
    let existing = super::user_install::read_user_setup_file(
        &arguments.root_cap,
        relative,
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
    let legacy_answers = existing.as_deref() == Some(answer_bytes.as_slice())
        && has_legacy_recommended_skill_selection(&answer_bytes)?;
    let config = if legacy_answers {
        parse_and_validate_installed_config(&answer_bytes)?
    } else {
        parse_and_validate_config(&answer_bytes)?
    };
    validate_registered_project_overrides(&config, &arguments.user_root)?;
    let catalog = parse_and_validate_catalog()?;
    let resolved_skills = resolve_skills(&config, &catalog)?;
    let desired = canonical_config(&config)?;
    let before_state = detect_state(&arguments.root_cap)?;
    let changed = existing.as_deref() != Some(desired.as_slice());
    if matches!(arguments.mode, SetupMode::DryRun | SetupMode::Apply) {
        reject_host_deselection(existing.as_deref(), &config)?;
    }

    match arguments.mode {
        SetupMode::DryRun => {
            let projection =
                plan_user_projection(&arguments.root_cap, &config, &resolved_skills, &desired)?;
            let mut changed_paths = projection.changed_paths;
            for host in &config.selected_hosts {
                let result = super::user_install::preview_configured_host(
                    &arguments.user_root,
                    *host,
                    &config,
                    &resolved_skills,
                )
                .map_err(|error| {
                    SetupError::Verification(format!(
                        "{} host preview failed after user setup validation: {error}",
                        host.as_str()
                    ))
                })?;
                changed_paths.extend(result.changed_paths);
            }
            if changed {
                changed_paths.push(USER_SETUP_RELATIVE.to_owned());
            }
            changed_paths.sort();
            changed_paths.dedup();
            Ok(success(
                arguments,
                &config,
                &resolved_skills,
                before_state,
                changed_paths,
                "hive.user-setup-dry-run-complete",
                "user setup dry run completed",
                &answer_bytes,
                &desired,
                &projection.reports,
            ))
        }
        SetupMode::Validate => {
            let installed = existing.ok_or_else(|| {
                SetupError::Verification("user setup is required before validation".to_owned())
            })?;
            let installed_config =
                parse_and_validate_installed_config(&installed).map_err(|error| {
                    SetupError::Verification(format!(
                        "installed user setup config is invalid: {}",
                        error.message()
                    ))
                })?;
            if installed_config != config {
                return Err(SetupError::Verification(
                    "installed user setup differs from the supplied answers".to_owned(),
                ));
            }
            validate_user_projection(&arguments.root_cap, &config, &resolved_skills, &installed)?;
            for host in &config.selected_hosts {
                super::user_install::validate_configured_host(
                    &arguments.user_root,
                    *host,
                    &config,
                    &resolved_skills,
                )
                .map_err(|error| {
                    SetupError::Verification(format!(
                        "{} host validation failed: {error}",
                        host.as_str()
                    ))
                })?;
            }
            Ok(success(
                arguments,
                &config,
                &resolved_skills,
                UserSetupState::Operational,
                Vec::new(),
                "hive.user-setup-valid",
                "installed user setup is valid",
                &answer_bytes,
                &desired,
                &[],
            ))
        }
        SetupMode::Apply => {
            let projection =
                apply_user_projection(&arguments.root_cap, &config, &resolved_skills, &desired)?;
            let mut host_changed_paths = Vec::new();
            let mut activated_hosts = Vec::new();
            for host in &config.selected_hosts {
                match super::user_install::apply_configured_host(
                    &arguments.user_root,
                    *host,
                    &config,
                    &resolved_skills,
                ) {
                    Ok(result) => {
                        host_changed_paths.extend(result.changed_paths);
                        activated_hosts.push(*host);
                    }
                    Err(primary) => {
                        let host_rollback =
                            rollback_activated_hosts(&arguments.user_root, &activated_hosts);
                        let projection_rollback =
                            rollback_user_projection(&arguments.root_cap, &projection);
                        if let Err(rollback) = host_rollback {
                            return Err(SetupError::Conflict(format!(
                                "host activation failed ({primary}); prior host rollback also failed ({rollback})"
                            )));
                        }
                        if let Err(rollback) = projection_rollback {
                            return Err(SetupError::Conflict(format!(
                                "host activation failed ({primary}); user projection rollback also failed ({rollback})"
                            )));
                        }
                        return Err(SetupError::Verification(format!(
                            "{} host activation failed after user setup validation: {primary}",
                            host.as_str()
                        )));
                    }
                }
            }
            if changed {
                if let Err(primary) = super::user_install::replace_user_setup_file(
                    &arguments.root_cap,
                    relative,
                    existing.as_deref(),
                    Some(&desired),
                ) {
                    let host_rollback =
                        rollback_activated_hosts(&arguments.user_root, &activated_hosts);
                    let projection_rollback =
                        rollback_user_projection(&arguments.root_cap, &projection);
                    if let Err(rollback) = host_rollback {
                        return Err(SetupError::Conflict(format!(
                            "setup activation failed ({primary}); host rollback also failed ({rollback})"
                        )));
                    }
                    if let Err(rollback) = projection_rollback {
                        return Err(SetupError::Conflict(format!(
                            "setup activation failed ({primary}); user projection rollback also failed ({rollback})"
                        )));
                    }
                    return Err(SetupError::Conflict(format!(
                        "setup activation failed: {primary}"
                    )));
                }
            }
            let retired_legacy_review = finish_applied_setup(
                &arguments.root_cap,
                &arguments.user_root,
                existing.as_deref(),
                &desired,
                changed,
                &activated_hosts,
                &projection,
                || {
                    let installed =
                        load_operational_config(&arguments.root_cap)?.ok_or_else(|| {
                            SetupError::Verification(
                                "user setup did not become operational after apply".to_owned(),
                            )
                        })?;
                    if installed != config {
                        return Err(SetupError::Verification(
                            "applied user setup failed byte validation".to_owned(),
                        ));
                    }
                    remove_completed_legacy_setup_review(&arguments.root_cap)
                },
            )?;
            if retired_legacy_review {
                host_changed_paths.push(LEGACY_USER_SETUP_REVIEW_RELATIVE.to_owned());
            }
            if changed {
                host_changed_paths.push(USER_SETUP_RELATIVE.to_owned());
            }
            host_changed_paths.extend(projection.changed_paths);
            host_changed_paths.sort();
            host_changed_paths.dedup();
            Ok(success(
                arguments,
                &config,
                &resolved_skills,
                UserSetupState::Operational,
                host_changed_paths,
                "hive.user-setup-complete",
                "user setup completed",
                &answer_bytes,
                &desired,
                &projection.reports,
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_applied_setup<F>(
    root: &Dir,
    user_root: &Path,
    prior_config: Option<&[u8]>,
    desired_config: &[u8],
    config_changed: bool,
    activated_hosts: &[SelectedHost],
    projection: &AppliedProjection,
    post_commit: F,
) -> Result<bool, SetupError>
where
    F: FnOnce() -> Result<bool, SetupError>,
{
    match post_commit() {
        Ok(retired_legacy_review) => Ok(retired_legacy_review),
        Err(primary) => {
            let rollback = rollback_applied_setup(
                root,
                user_root,
                prior_config,
                desired_config,
                config_changed,
                activated_hosts,
                projection,
            );
            match rollback {
                Ok(()) => Err(primary),
                Err(rollback) => Err(SetupError::Conflict(format!(
                    "post-commit setup validation failed ({}); rollback also failed ({rollback})",
                    primary.message()
                ))),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn rollback_applied_setup(
    root: &Dir,
    user_root: &Path,
    prior_config: Option<&[u8]>,
    desired_config: &[u8],
    config_changed: bool,
    activated_hosts: &[SelectedHost],
    projection: &AppliedProjection,
) -> Result<(), String> {
    let mut failures = Vec::new();
    if config_changed {
        if let Err(error) = super::user_install::replace_user_setup_file(
            root,
            Path::new(USER_SETUP_RELATIVE),
            Some(desired_config),
            prior_config,
        ) {
            failures.push(format!("user setup config: {error}"));
        }
    }
    if let Err(error) = rollback_activated_hosts(user_root, activated_hosts) {
        failures.push(format!("configured hosts: {error}"));
    }
    if let Err(error) = rollback_user_projection(root, projection) {
        failures.push(format!("user projection: {error}"));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn remove_completed_legacy_setup_review(root: &Dir) -> Result<bool, SetupError> {
    let relative = Path::new(LEGACY_USER_SETUP_REVIEW_RELATIVE);
    let existing = super::user_install::read_user_setup_file(root, relative, MAX_USER_SETUP_BYTES)
        .map_err(SetupError::Conflict)?;
    if existing.as_deref() != Some(LEGACY_USER_SETUP_REVIEW) {
        return Ok(false);
    }
    super::user_install::replace_user_setup_file(
        root,
        relative,
        Some(LEGACY_USER_SETUP_REVIEW),
        None,
    )
    .map_err(|error| {
        SetupError::Conflict(format!(
            "operational setup could not retire the completed legacy setup review: {error}"
        ))
    })?;
    Ok(true)
}

fn rollback_activated_hosts(
    user_root: &Path,
    activated_hosts: &[SelectedHost],
) -> Result<(), String> {
    let mut failures = Vec::new();
    for host in activated_hosts.iter().rev() {
        if let Err(error) = super::user_install::recover_configured_host(user_root, *host) {
            failures.push(format!("{}: {error}", host.as_str()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn parse_and_validate_config(bytes: &[u8]) -> Result<UserSetupConfig, SetupError> {
    parse_and_validate_config_inner(bytes, false)
}

fn parse_and_validate_installed_config(bytes: &[u8]) -> Result<UserSetupConfig, SetupError> {
    parse_and_validate_config_inner(bytes, true)
}

fn parse_and_validate_config_inner(
    bytes: &[u8],
    allow_legacy_recommended: bool,
) -> Result<UserSetupConfig, SetupError> {
    let mut value: JsonValue = serde_yaml::from_slice(bytes)
        .map_err(|error| SetupError::Input(format!("invalid user setup YAML: {error}")))?;
    let migrated_skills = migrate_legacy_recommended_skill_selection(&mut value)?;
    if migrated_skills && !allow_legacy_recommended {
        return Err(SetupError::Input(
            "recommended Skill suites are no longer accepted; choose mode all or individual"
                .to_owned(),
        ));
    }
    migrate_legacy_skill_names(&mut value)?;
    migrate_legacy_single_profile(&mut value);
    if allow_legacy_recommended {
        migrate_legacy_missing_usage_threshold(&mut value);
    }
    validate_schema(USER_SETUP_SCHEMA, &value, "user setup")?;
    let config: UserSetupConfig = serde_json::from_value(value)
        .map_err(|error| SetupError::Input(format!("invalid user setup values: {error}")))?;
    validate_config_semantics(&config)?;
    Ok(config)
}

fn migrate_legacy_missing_usage_threshold(value: &mut JsonValue) {
    let Some(usage_guard) = value
        .get_mut("usage_guard")
        .and_then(JsonValue::as_object_mut)
    else {
        return;
    };
    usage_guard
        .entry("stop_remaining_percent".to_owned())
        .or_insert_with(|| JsonValue::from(LEGACY_080_USAGE_THRESHOLD));
}

fn migrate_legacy_skill_names(value: &mut JsonValue) -> Result<(), SetupError> {
    let Some(selected) = value
        .get_mut("skills")
        .and_then(JsonValue::as_object_mut)
        .and_then(|skills| skills.get_mut("selected"))
        .and_then(JsonValue::as_array_mut)
    else {
        return Ok(());
    };
    let mut names = BTreeSet::new();
    for item in selected.iter() {
        let name = item.as_str().ok_or_else(|| {
            SetupError::Input("selected Skills must contain only names".to_owned())
        })?;
        let canonical = canonical_builtin_skill_name(name)
            .map_err(|error| SetupError::Internal(error.to_string()))?
            .unwrap_or_else(|| name.to_owned());
        names.insert(canonical);
    }
    *selected = names.into_iter().map(JsonValue::String).collect();
    Ok(())
}

fn has_legacy_recommended_skill_selection(bytes: &[u8]) -> Result<bool, SetupError> {
    let value: JsonValue = serde_yaml::from_slice(bytes)
        .map_err(|error| SetupError::Input(format!("invalid user setup YAML: {error}")))?;
    Ok(value
        .get("skills")
        .and_then(JsonValue::as_object)
        .and_then(|skills| skills.get("mode"))
        .is_some_and(|mode| mode == "recommended"))
}

fn migrate_legacy_recommended_skill_selection(value: &mut JsonValue) -> Result<bool, SetupError> {
    let Some(skills) = value.get_mut("skills").and_then(JsonValue::as_object_mut) else {
        return Ok(false);
    };
    if skills.get("mode") != Some(&JsonValue::String("recommended".to_owned())) {
        return Ok(false);
    }
    let suite = skills
        .get("recommended_suite")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SetupError::Input(
                "legacy recommended Skill selection requires recommended_suite".to_owned(),
            )
        })?;
    if skills.contains_key("selected") {
        return Err(SetupError::Input(
            "legacy recommended Skill selection must not include selected".to_owned(),
        ));
    }
    let selected = legacy_recommended_skill_set(suite).ok_or_else(|| {
        SetupError::Input(format!("unknown legacy recommended Skill suite: {suite}"))
    })?;
    skills.insert(
        "mode".to_owned(),
        JsonValue::String("individual".to_owned()),
    );
    skills.remove("recommended_suite");
    skills.insert(
        "selected".to_owned(),
        JsonValue::Array(
            selected
                .iter()
                .map(|name| JsonValue::String((*name).to_owned()))
                .collect(),
        ),
    );
    Ok(true)
}

fn migrate_legacy_single_profile(value: &mut JsonValue) -> bool {
    let Some(profile) = value.get_mut("profile").and_then(JsonValue::as_object_mut) else {
        return false;
    };
    if profile.contains_key("contexts") {
        return false;
    }
    let Some(id) = profile.get("id").and_then(JsonValue::as_str) else {
        return false;
    };
    let description = profile.get("custom_description").cloned();
    let contexts = match id {
        "web-developer" | "game-developer" | "non-developer" => {
            vec![JsonValue::String(id.to_owned())]
        }
        "custom" => Vec::new(),
        _ => return false,
    };
    profile.clear();
    profile.insert("contexts".to_owned(), JsonValue::Array(contexts));
    if let Some(description) = description {
        profile.insert("description".to_owned(), description);
    }
    true
}

fn legacy_recommended_skill_set(suite: &str) -> Option<&'static [&'static str]> {
    match suite {
        "web-developer" => Some(&[
            "user-setup",
            "project-setup",
            "project-setup",
            "code-polish",
            "research-best-practices",
            "verified-workflow",
            "prompt-refine",
            "knowledge-maintain",
            "knowledge-capture",
            "knowledge-recall",
            "knowledge-maintain",
            "knowledge-scan",
            "run-checkpoint",
            "run-resume",
            "usage-guard",
            "product-update",
            "project-refresh",
        ]),
        "game-developer" => Some(&[
            "user-setup",
            "project-setup",
            "project-setup",
            "code-polish",
            "research-best-practices",
            "verified-workflow",
            "prompt-refine",
            "knowledge-maintain",
            "knowledge-capture",
            "knowledge-recall",
            "knowledge-maintain",
            "knowledge-scan",
            "run-checkpoint",
            "run-resume",
            "run-handoff",
            "package-review",
            "usage-guard",
            "product-update",
            "project-refresh",
        ]),
        "non-developer" => Some(&[
            "user-setup",
            "project-setup",
            "project-setup",
            "research-best-practices",
            "quick-answer",
            "prompt-refine",
            "knowledge-maintain",
            "knowledge-capture",
            "knowledge-recall",
            "knowledge-maintain",
            "knowledge-scan",
            "usage-guard",
            "product-update",
        ]),
        _ => None,
    }
}

fn parse_and_validate_catalog() -> Result<UserSetupCatalog, SetupError> {
    let value: JsonValue = serde_yaml::from_str(USER_SETUP_CATALOG).map_err(|error| {
        SetupError::Internal(format!("invalid embedded setup catalog: {error}"))
    })?;
    validate_schema(
        USER_SETUP_CATALOG_SCHEMA,
        &value,
        "embedded user setup catalog",
    )
    .map_err(|error| SetupError::Internal(error.message().to_owned()))?;
    let catalog: UserSetupCatalog = serde_json::from_value(value).map_err(|error| {
        SetupError::Internal(format!("invalid embedded setup catalog values: {error}"))
    })?;
    validate_catalog_semantics(&catalog)?;
    Ok(catalog)
}

fn validate_schema(schema_source: &str, value: &JsonValue, label: &str) -> Result<(), SetupError> {
    let schema: JsonValue = serde_json::from_str(schema_source)
        .map_err(|error| SetupError::Internal(format!("invalid {label} schema: {error}")))?;
    jsonschema::meta::validate(&schema)
        .map_err(|error| SetupError::Internal(format!("invalid {label} metaschema: {error}")))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .map_err(|error| SetupError::Internal(format!("cannot compile {label} schema: {error}")))?;
    if let Err(error) = validator.validate(value) {
        return Err(SetupError::Input(format!(
            "{label} schema validation failed at {}: {error}",
            error.instance_path()
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn validate_config_semantics(config: &UserSetupConfig) -> Result<(), SetupError> {
    if config.schema_version != 1 {
        return Err(SetupError::Input(
            "user setup schema_version must be 1".to_owned(),
        ));
    }
    validate_user_profile(&config.profile)?;
    validate_custom_selection(&config.persona, "persona")?;
    validate_sorted_unique_hosts(&config.selected_hosts)?;
    match (
        config.wiki.enabled,
        config.wiki.backend,
        config.wiki.notion.as_ref(),
    ) {
        (_, WikiBackend::Markdown, None) => {}
        (true, WikiBackend::Notion, Some(notion)) => {
            validate_notion_id("workspace_id", &notion.workspace_id)?;
            validate_notion_id("scope_id", &notion.scope_id)?;
            if !notion.local_index_consent {
                return Err(SetupError::Input(
                    "Notion Wiki requires explicit consent to store derived content in local SQLite"
                        .to_owned(),
                ));
            }
        }
        (false, WikiBackend::Notion, _) => {
            return Err(SetupError::Input(
                "disabled Wiki must use the markdown backend without Notion configuration"
                    .to_owned(),
            ));
        }
        (_, WikiBackend::Markdown, Some(_)) => {
            return Err(SetupError::Input(
                "markdown Wiki backend must not include Notion configuration".to_owned(),
            ));
        }
        (true, WikiBackend::Notion, None) => {
            return Err(SetupError::Input(
                "notion Wiki backend requires workspace_id, scope_id, and local_index_consent"
                    .to_owned(),
            ));
        }
    }
    if !(1..=99).contains(&config.usage_guard.stop_remaining_percent) {
        return Err(SetupError::Input(
            "usage guard stop_remaining_percent must be between 1 and 99".to_owned(),
        ));
    }
    for (project_identity, threshold) in &config.usage_guard.project_overrides {
        if project_identity.trim().is_empty()
            || project_identity.trim() != project_identity
            || project_identity.len() > 160
            || project_identity.contains(['\r', '\n', '\0'])
        {
            return Err(SetupError::Input(
                "usage guard project override requires a stable non-path project identity"
                    .to_owned(),
            ));
        }
        if !(1..=99).contains(threshold) {
            return Err(SetupError::Input(
                "usage guard project override must be between 1 and 99".to_owned(),
            ));
        }
        if *threshold < config.usage_guard.stop_remaining_percent {
            return Err(SetupError::Input(
                "usage guard project override cannot be lower than the global threshold".to_owned(),
            ));
        }
    }
    if !config.usage_guard.enabled && config.usage_guard.codexbar_fallback_enabled {
        return Err(SetupError::Input(
            "codexbar_fallback_enabled must be false when the usage guard is disabled".to_owned(),
        ));
    }
    let discord = &config.usage_guard.discord;
    match (
        config.usage_guard.enabled,
        discord.enabled,
        discord.webhook_url_env.as_deref(),
    ) {
        (false, true, _) => {
            return Err(SetupError::Input(
                "Discord usage notification requires the usage guard to be enabled".to_owned(),
            ));
        }
        (_, true, Some(name)) if valid_environment_name(name) => {}
        (_, true, _) => {
            return Err(SetupError::Input(
                "Discord usage notification requires a valid webhook_url_env name".to_owned(),
            ));
        }
        (_, false, None) => {}
        (_, false, Some(_)) => {
            return Err(SetupError::Input(
                "Discord webhook_url_env must be absent while Discord notification is disabled"
                    .to_owned(),
            ));
        }
    }
    if discord.message_fields.is_empty()
        || discord
            .message_fields
            .iter()
            .map(|field| field.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != discord.message_fields.len()
    {
        return Err(SetupError::Input(
            "Discord message_fields must be a non-empty list without duplicates".to_owned(),
        ));
    }
    Ok(())
}

fn validate_registered_project_overrides(
    config: &UserSetupConfig,
    user_root: &Path,
) -> Result<(), SetupError> {
    if config.usage_guard.project_overrides.is_empty() {
        return Ok(());
    }
    let registry = hive_wiki::shared::load_project_registry(user_root).map_err(|error| {
        SetupError::Input(format!(
            "usage guard project overrides require a valid registered-project list: {error}"
        ))
    })?;
    for project_identity in config.usage_guard.project_overrides.keys() {
        if !registry
            .projects
            .iter()
            .any(|project| project.id == *project_identity)
        {
            return Err(SetupError::Input(format!(
                "usage guard project override is not a registered project identity: {project_identity}"
            )));
        }
    }
    Ok(())
}

fn valid_environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some('A'..='Z' | '_'))
        && characters.all(|character| matches!(character, 'A'..='Z' | '0'..='9' | '_'))
        && value.len() <= 128
}

fn validate_notion_id(label: &str, value: &str) -> Result<(), SetupError> {
    if value.trim() != value
        || value.is_empty()
        || value.len() > 500
        || value.chars().any(char::is_control)
        || value.contains(['/', '\\'])
        || value == "."
        || value == ".."
    {
        return Err(SetupError::Input(format!("invalid Notion {label}")));
    }
    Ok(())
}

fn validate_custom_selection(selection: &CatalogSelection, label: &str) -> Result<(), SetupError> {
    match (
        selection.id.as_str(),
        selection.custom_description.as_deref(),
    ) {
        ("custom", Some(value))
            if !value.trim().is_empty()
                && !value.contains('\r')
                && !value.contains('\n')
                && value.chars().count() <= 500 =>
        {
            Ok(())
        }
        ("custom", Some(value)) if value.chars().count() > 500 => Err(SetupError::Input(format!(
            "custom {label} custom_description must not exceed 500 Unicode scalar values"
        ))),
        ("custom", _) => Err(SetupError::Input(format!(
            "custom {label} requires a nonblank custom_description"
        ))),
        (_, None) => Ok(()),
        _ => Err(SetupError::Input(format!(
            "non-custom {label} must not include custom_description"
        ))),
    }
}

fn validate_user_profile(profile: &UserProfile) -> Result<(), SetupError> {
    let mut contexts = BTreeSet::new();
    for context in &profile.contexts {
        if !contexts.insert(context.as_str()) {
            return Err(SetupError::Input(
                "user profile contexts must not contain duplicates".to_owned(),
            ));
        }
    }
    match profile.description.as_deref() {
        Some(value)
            if !value.trim().is_empty()
                && !value.contains('\r')
                && !value.contains('\n')
                && value.chars().count() <= 500 => {}
        Some(value) if value.chars().count() > 500 => {
            return Err(SetupError::Input(
                "user profile description must not exceed 500 Unicode scalar values".to_owned(),
            ));
        }
        Some(_) => {
            return Err(SetupError::Input(
                "user profile description must be a nonblank single line".to_owned(),
            ));
        }
        None => {}
    }
    if profile.contexts.is_empty() && profile.description.is_none() {
        return Err(SetupError::Input(
            "user profile requires at least one context or a description".to_owned(),
        ));
    }
    Ok(())
}

fn reject_host_deselection(
    installed: Option<&[u8]>,
    desired: &UserSetupConfig,
) -> Result<(), SetupError> {
    let Some(installed) = installed else {
        return Ok(());
    };
    let installed = parse_and_validate_installed_config(installed).map_err(|error| {
        SetupError::Conflict(format!(
            "installed user setup is invalid: {}",
            error.message()
        ))
    })?;
    if installed.wiki.backend != desired.wiki.backend {
        return Err(SetupError::Conflict(
            "Wiki backend changes require a separate previewed migration; user setup cannot switch backends"
                .to_owned(),
        ));
    }
    let desired_hosts = desired
        .selected_hosts
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let removed = installed
        .selected_hosts
        .iter()
        .copied()
        .filter(|host| !desired_hosts.contains(host))
        .map(SelectedHost::as_str)
        .collect::<Vec<_>>();
    if removed.is_empty() {
        Ok(())
    } else {
        Err(SetupError::Conflict(format!(
            "cannot remove configured hosts until transactional host deactivation is available; keep these selected_hosts entries: {}",
            removed.join(", ")
        )))
    }
}

fn validate_sorted_unique_hosts(hosts: &[SelectedHost]) -> Result<(), SetupError> {
    if hosts.is_empty() {
        return Err(SetupError::Input(
            "selected_hosts must contain at least one host".to_owned(),
        ));
    }
    let mut unique = BTreeSet::new();
    for host in hosts {
        if !unique.insert(*host) {
            return Err(SetupError::Input(
                "selected_hosts must not contain duplicates".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_catalog_semantics(catalog: &UserSetupCatalog) -> Result<(), SetupError> {
    if catalog.schema_version != 1 {
        return Err(SetupError::Internal(
            "embedded user setup catalog schema_version must be 1".to_owned(),
        ));
    }
    validate_catalog_choices(
        &catalog.profiles,
        &["game-developer", "non-developer", "web-developer"],
        "user context",
    )?;
    validate_catalog_choices(
        &catalog.personas,
        &["balanced", "custom", "friendly", "strict"],
        "persona",
    )?;
    let skill_catalog =
        embedded_catalog().map_err(|error| SetupError::Internal(error.to_string()))?;
    let built_ins: BTreeSet<&str> = skill_catalog
        .skills
        .iter()
        .filter(|entry| {
            matches!(
                entry.availability,
                hive_projection::Availability::Implemented
            )
        })
        .map(|entry| entry.name.as_str())
        .collect();
    if catalog.mandatory_skills != ["user-setup"] {
        return Err(SetupError::Internal(
            "embedded mandatory skill set must be exactly user-setup".to_owned(),
        ));
    }
    if !catalog.optional_third_party_skills.is_empty() {
        return Err(SetupError::Internal(
            "optional third-party Skills are unsupported until a signed consent contract is available"
                .to_owned(),
        ));
    }
    validate_skill_names(&catalog.mandatory_skills, &built_ins, "mandatory skills")?;
    let mut dependency_keys = BTreeSet::new();
    for dependency in &catalog.skill_dependencies {
        if !dependency_keys.insert(dependency.skill.as_str())
            || !built_ins.contains(dependency.skill.as_str())
        {
            return Err(SetupError::Internal(
                "embedded skill dependency key is invalid or duplicate".to_owned(),
            ));
        }
        validate_skill_names(&dependency.requires, &built_ins, "skill dependency")?;
    }
    if dependency_keys != built_ins {
        return Err(SetupError::Internal(
            "embedded Skill dependencies must cover every built-in Skill exactly".to_owned(),
        ));
    }
    Ok(())
}

fn validate_catalog_choices(
    choices: &[CatalogChoice],
    expected: &[&str],
    label: &str,
) -> Result<(), SetupError> {
    let mut ids = BTreeSet::new();
    for choice in choices {
        validate_localized(&choice.display_name, "catalog display_name")?;
        validate_localized(&choice.description, "catalog description")?;
        if !ids.insert(choice.id.as_str()) {
            return Err(SetupError::Internal(format!(
                "duplicate embedded {label} identifier"
            )));
        }
    }
    if ids != expected.iter().copied().collect() {
        return Err(SetupError::Internal(format!(
            "embedded {label} catalog coverage is not exact"
        )));
    }
    Ok(())
}

fn validate_localized(value: &LocalizedText, label: &str) -> Result<(), SetupError> {
    if value.en.trim().is_empty() || value.ko.trim().is_empty() {
        return Err(SetupError::Internal(format!(
            "embedded {label} must contain English and Korean text"
        )));
    }
    Ok(())
}

fn validate_skill_names(
    names: &[String],
    built_ins: &BTreeSet<&str>,
    label: &str,
) -> Result<(), SetupError> {
    let mut unique = BTreeSet::new();
    for name in names {
        if !built_ins.contains(name.as_str()) || !unique.insert(name.as_str()) {
            return Err(SetupError::Internal(format!(
                "embedded {label} contains an unknown or duplicate Skill: {name}"
            )));
        }
    }
    Ok(())
}

fn resolve_skills(
    config: &UserSetupConfig,
    catalog: &UserSetupCatalog,
) -> Result<Vec<String>, SetupError> {
    for context in &config.profile.contexts {
        if !catalog.profiles.iter().any(|entry| entry.id == *context) {
            return Err(SetupError::Input(format!(
                "unknown user context: {context}",
            )));
        }
    }
    if !catalog
        .personas
        .iter()
        .any(|entry| entry.id == config.persona.id)
    {
        return Err(SetupError::Input(format!(
            "unknown agent persona: {}",
            config.persona.id
        )));
    }
    let mut selected: BTreeSet<String> = match config.skills.mode {
        SkillSelectionMode::All => {
            if !config.skills.selected.is_empty() {
                return Err(SetupError::Input(
                    "all Skill mode must not include individual selections".to_owned(),
                ));
            }
            embedded_catalog()
                .map_err(|error| SetupError::Internal(error.to_string()))?
                .skills
                .into_iter()
                .filter(|entry| {
                    matches!(
                        entry.availability,
                        hive_projection::Availability::Implemented
                    )
                })
                .map(|entry| entry.name)
                .collect()
        }
        SkillSelectionMode::Individual => {
            if config.skills.selected.is_empty() {
                return Err(SetupError::Input(
                    "individual Skill mode requires selected".to_owned(),
                ));
            }
            config.skills.selected.iter().cloned().collect()
        }
    };
    selected.extend(catalog.mandatory_skills.iter().cloned());
    if !config.wiki.enabled {
        selected.retain(|name| {
            !matches!(
                name.as_str(),
                "knowledge-capture"
                    | "knowledge-recall"
                    | "knowledge-promote"
                    | "knowledge-maintain"
                    | "knowledge-scan"
            )
        });
    }
    if config.usage_guard.enabled {
        selected.insert("usage-guard".to_owned());
    }
    let dependencies: BTreeMap<&str, &[String]> = catalog
        .skill_dependencies
        .iter()
        .map(|entry| (entry.skill.as_str(), entry.requires.as_slice()))
        .collect();
    loop {
        let before = selected.len();
        let current: Vec<String> = selected.iter().cloned().collect();
        for name in current {
            if let Some(required) = dependencies.get(name.as_str()) {
                selected.extend(required.iter().cloned());
            }
        }
        if selected.len() == before {
            break;
        }
    }
    let available: BTreeSet<String> = embedded_catalog()
        .map_err(|error| SetupError::Internal(error.to_string()))?
        .skills
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.availability,
                hive_projection::Availability::Implemented
            )
        })
        .map(|entry| entry.name)
        .collect();
    if let Some(unknown) = selected.iter().find(|name| !available.contains(*name)) {
        return Err(SetupError::Input(format!(
            "selected Skill is not available in this release: {unknown}"
        )));
    }
    Ok(selected.into_iter().collect())
}

fn canonical_config(config: &UserSetupConfig) -> Result<Vec<u8>, SetupError> {
    let mut bytes = serde_yaml::to_string(config)
        .map_err(|error| SetupError::Internal(format!("cannot serialize user setup: {error}")))?
        .into_bytes();
    if !bytes.ends_with(b"\n") {
        bytes.push(b'\n');
    }
    Ok(bytes)
}

#[allow(clippy::too_many_lines)]
fn plan_user_projection(
    root: &Dir,
    config: &UserSetupConfig,
    resolved_skills: &[String],
    setup_bytes: &[u8],
) -> Result<AppliedProjection, SetupError> {
    let files = user_projection_files(config, resolved_skills)?;
    let manifest_relative = Path::new(USER_PROJECTION_MANIFEST_RELATIVE);
    let prior_bytes =
        super::user_install::read_user_setup_file(root, manifest_relative, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
    let prior = prior_bytes
        .as_deref()
        .map(parse_projection_manifest)
        .transpose()?;
    let mut base_files = projection_base_files(root, prior.as_ref())?;
    let retired_files = authenticated_retired_user_skill_files(root)?;
    base_files.extend(retired_files.clone());
    let mut prior_owned_paths = prior
        .as_ref()
        .map(|manifest| {
            manifest
                .entries
                .iter()
                .map(|entry| PathBuf::from(&entry.path))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    prior_owned_paths.extend(retired_files.keys().cloned());

    let mut paths = base_files.keys().cloned().collect::<BTreeSet<_>>();
    paths.extend(files.keys().cloned());
    let mut planned = AppliedProjection {
        changes: Vec::new(),
        changed_paths: Vec::new(),
        reports: Vec::new(),
    };
    for path in paths {
        let before = super::user_install::read_user_setup_file(root, &path, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        let base = base_files.get(&path);
        let incoming = files.get(&path);
        if prior.is_some()
            && base.is_none()
            && before.is_some()
            && before.as_deref() != incoming.map(Vec::as_slice)
        {
            return Err(SetupError::Conflict(format!(
                "Hive cannot safely refresh this setup file because its release original is unavailable: {}. No files were changed.",
                path.display()
            )));
        }
        if before.is_some() && !prior_owned_paths.contains(&path) {
            return Err(SetupError::Conflict(format!(
                "user projection path exists without Hive ownership proof: {}",
                path.display()
            )));
        }
        let merged = if path.starts_with(".agents/directives/") {
            three_way_merge_hive_directive(
                &path,
                base.map(Vec::as_slice),
                before.as_deref(),
                incoming.map(Vec::as_slice),
            )
        } else {
            three_way_merge(
                &path,
                base.map(Vec::as_slice),
                before.as_deref(),
                incoming.map(Vec::as_slice),
            )
        }
        .map_err(|error| SetupError::Conflict(error.to_string()))?;
        let after = merged.bytes;
        planned.reports.push(UserProjectionPathReport {
            path: portable(&path),
            base_digest: base.map(|bytes| sha256_digest(bytes)),
            local_digest: before.as_deref().map(sha256_digest),
            incoming_digest: incoming.map(|bytes| sha256_digest(bytes)),
            final_digest: after.as_deref().map(sha256_digest),
            disposition: merged.disposition,
            omitted_incoming_hunks: merged.omitted_incoming_hunks,
            local_priority: merged.local_priority,
        });
        if before != after {
            planned.changed_paths.push(portable(&path));
            planned.changes.push(ProjectionChange {
                path,
                before,
                after,
            });
        }
    }

    let manifest_bytes = render_projection_manifest(setup_bytes, &files)?;
    let manifest_before =
        super::user_install::read_user_setup_file(root, manifest_relative, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
    if manifest_before.as_deref() != Some(manifest_bytes.as_slice()) {
        planned
            .changed_paths
            .push(USER_PROJECTION_MANIFEST_RELATIVE.to_owned());
        planned.changes.push(ProjectionChange {
            path: manifest_relative.to_path_buf(),
            before: manifest_before,
            after: Some(manifest_bytes),
        });
    }
    planned.changed_paths.sort();
    planned.changed_paths.dedup();
    planned
        .reports
        .sort_by(|left, right| left.path.cmp(&right.path));
    Ok(planned)
}

fn authenticated_retired_user_skill_files(
    root: &Dir,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, SetupError> {
    let retired = retired_builtin_skill_names()
        .map_err(|error| SetupError::Internal(error.message().to_owned()))?;
    let mut historical_digests = BTreeMap::<String, BTreeSet<String>>::new();
    for version in HISTORICAL_SKILL_RELEASES {
        for skill in historical_builtin_skills(version)
            .map_err(|error| SetupError::Internal(error.message().to_owned()))?
        {
            if retired.contains_key(&skill.name) {
                historical_digests
                    .entry(skill.name)
                    .or_default()
                    .insert(skill.content_digest);
            }
        }
    }

    let mut files = BTreeMap::new();
    for (name, digests) in historical_digests {
        let path = PathBuf::from(".agents/skills").join(name).join("SKILL.md");
        let Some(bytes) =
            super::user_install::read_user_setup_file(root, &path, MAX_USER_SETUP_BYTES)
                .map_err(SetupError::Conflict)?
        else {
            continue;
        };
        if digests.contains(&sha256_digest(&bytes)) {
            files.insert(path, bytes);
        }
    }
    Ok(files)
}

fn user_projection_files(
    config: &UserSetupConfig,
    resolved_skills: &[String],
) -> Result<BTreeMap<PathBuf, Vec<u8>>, SetupError> {
    let mut files = BTreeMap::<PathBuf, Vec<u8>>::new();
    let projection = compile_user_projection_localized(
        ProjectionHost::Codex,
        resolved_skills,
        &[],
        config.interface_language.descriptor_language(),
    )
    .map_err(|error| SetupError::Internal(error.to_string()))?;
    for (path, bytes) in projection.files {
        if path.starts_with(".agents/skills/") {
            files.insert(PathBuf::from(path), bytes);
        } else if path == ".hive/config/active-skills.yml" {
            files.insert(PathBuf::from(".hive/config/user-active-skills.yml"), bytes);
        }
    }
    files.insert(
        PathBuf::from(".agents/directives/00-hive-user.md"),
        render_user_directive(config, resolved_skills),
    );
    Ok(files)
}

fn legacy_070_projection_files(
    config: &UserSetupConfig,
    resolved_skills: &[String],
) -> Result<BTreeMap<PathBuf, Vec<u8>>, SetupError> {
    Ok(user_projection_files(config, resolved_skills)?
        .into_iter()
        .filter(|(path, _)| !path.ends_with("agents/openai.yaml"))
        .collect())
}

fn render_projection_manifest(
    setup_bytes: &[u8],
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<Vec<u8>, SetupError> {
    let entries = files
        .iter()
        .map(|(path, bytes)| UserProjectionEntry {
            path: portable(path),
            digest: sha256_digest(bytes),
        })
        .collect::<Vec<_>>();
    let base_entries = files
        .iter()
        .map(|(path, bytes)| {
            let content = String::from_utf8(bytes.clone()).map_err(|_| {
                SetupError::Internal(format!("user projection must be UTF-8: {}", path.display()))
            })?;
            Ok(UserProjectionBaseEntry {
                path: portable(path),
                digest: sha256_digest(bytes),
                content,
            })
        })
        .collect::<Result<Vec<_>, SetupError>>()?;
    let manifest = UserProjectionManifest {
        schema_version: 2,
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        package_version: Some(env!("HIVE_PACKAGE_VERSION").to_owned()),
        setup_digest: sha256_digest(setup_bytes),
        entries,
        base_entries,
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| {
        SetupError::Internal(format!(
            "cannot serialize user projection manifest: {error}"
        ))
    })?;
    manifest_bytes.push(b'\n');
    Ok(manifest_bytes)
}

fn projection_base_files(
    root: &Dir,
    prior: Option<&UserProjectionManifest>,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, SetupError> {
    let Some(prior) = prior else {
        return Ok(BTreeMap::new());
    };
    if prior.schema_version == 2 {
        return prior
            .base_entries
            .iter()
            .map(|entry| {
                Ok((
                    PathBuf::from(&entry.path),
                    entry.content.as_bytes().to_vec(),
                ))
            })
            .collect();
    }
    if prior.product_version == "0.7.0" {
        return legacy_070_projection_base(root, prior);
    }
    legacy_test3_projection_base(root, prior)
}

fn legacy_070_projection_base(
    root: &Dir,
    prior: &UserProjectionManifest,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, SetupError> {
    let installed = super::user_install::read_user_setup_file(
        root,
        Path::new(USER_SETUP_RELATIVE),
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    .ok_or_else(|| {
        SetupError::Conflict(
            "Hive cannot identify the release original because the saved global preferences are missing. No files were changed."
                .to_owned(),
        )
    })?;
    if sha256_digest(&installed) != prior.setup_digest {
        return Err(SetupError::Conflict(
            "Hive cannot identify the release original because the saved global preferences changed outside Hive. No files were changed."
                .to_owned(),
        ));
    }
    let config = parse_and_validate_installed_config(&installed).map_err(|_| {
        SetupError::Conflict(
            "Hive cannot read the saved global preferences needed for a safe refresh. No files were changed."
                .to_owned(),
        )
    })?;
    let catalog = parse_and_validate_catalog()
        .map_err(|error| SetupError::Internal(error.message().to_owned()))?;
    let skills = resolve_skills(&config, &catalog)?;
    let expected_paths = legacy_070_projection_files(&config, &skills)?
        .keys()
        .map(|path| portable(path))
        .collect::<Vec<_>>();
    let recorded_paths = prior
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    if expected_paths != recorded_paths {
        return Err(SetupError::Conflict(
            "Hive cannot safely refresh this older setup because its recorded Hive file list does not match the supported 0.7.0 installation. No files were changed."
                .to_owned(),
        ));
    }
    let mut base = BTreeMap::new();
    for entry in &prior.entries {
        let path = PathBuf::from(&entry.path);
        let bytes = super::user_install::read_user_setup_file(root, &path, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?
            .ok_or_else(|| {
                SetupError::Conflict(
                    "Hive cannot safely refresh this older setup because a recorded Hive file is missing. No files were changed."
                        .to_owned(),
                )
            })?;
        if sha256_digest(&bytes) != entry.digest {
            return Err(SetupError::Conflict(
                "Hive cannot safely refresh this older setup because some Hive files changed after that installation. No files were changed."
                    .to_owned(),
            ));
        }
        base.insert(path, bytes);
    }
    Ok(base)
}

fn legacy_test3_projection_base(
    root: &Dir,
    prior: &UserProjectionManifest,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, SetupError> {
    if prior.product_version != "0.9.0" {
        return Err(SetupError::Conflict(
            "Hive cannot identify the release original for this setup. No files were changed."
                .to_owned(),
        ));
    }
    let installed = super::user_install::read_user_setup_file(
        root,
        Path::new(USER_SETUP_RELATIVE),
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    .ok_or_else(|| {
        SetupError::Conflict(
            "Hive cannot identify the release original because the saved global preferences are missing. No files were changed."
                .to_owned(),
        )
    })?;
    if sha256_digest(&installed) != prior.setup_digest {
        return Err(SetupError::Conflict(
            "Hive cannot identify the release original because the saved global preferences changed outside Hive. No files were changed."
                .to_owned(),
        ));
    }
    let config = parse_and_validate_installed_config(&installed).map_err(|_| {
        SetupError::Conflict(
            "Hive cannot read the saved global preferences needed for a safe refresh. No files were changed."
                .to_owned(),
        )
    })?;
    let catalog = parse_and_validate_catalog()
        .map_err(|error| SetupError::Internal(error.message().to_owned()))?;
    let skills = resolve_skills(&config, &catalog)?;
    let mut base = user_projection_files(&config, &skills)?;
    base.insert(
        PathBuf::from(".agents/skills/user-setup/SKILL.md"),
        USER_PROJECTION_090_TEST3_SETUP_HIVE.to_vec(),
    );
    let expected = base
        .iter()
        .map(|(path, bytes)| (portable(path), sha256_digest(bytes)))
        .collect::<Vec<_>>();
    let recorded = prior
        .entries
        .iter()
        .map(|entry| (entry.path.clone(), entry.digest.clone()))
        .collect::<Vec<_>>();
    if expected != recorded {
        return Err(SetupError::Conflict(
            "Hive cannot authenticate the release original for this older setup. No files were changed."
                .to_owned(),
        ));
    }
    Ok(base)
}

fn apply_user_projection(
    root: &Dir,
    config: &UserSetupConfig,
    resolved_skills: &[String],
    setup_bytes: &[u8],
) -> Result<AppliedProjection, SetupError> {
    let AppliedProjection {
        changes,
        changed_paths,
        reports,
    } = plan_user_projection(root, config, resolved_skills, setup_bytes)?;
    let mut applied = AppliedProjection {
        changes: Vec::new(),
        changed_paths,
        reports,
    };
    for change in changes {
        if let Err(primary) = super::user_install::replace_user_setup_file(
            root,
            &change.path,
            change.before.as_deref(),
            change.after.as_deref(),
        ) {
            let rollback = rollback_user_projection(root, &applied);
            if let Err(rollback) = rollback {
                return Err(SetupError::Conflict(format!(
                    "user projection activation failed ({primary}); rollback failed ({rollback})"
                )));
            }
            return Err(SetupError::Conflict(format!(
                "user projection activation failed: {primary}"
            )));
        }
        applied.changes.push(change);
    }
    let removed_skill_paths = applied
        .changes
        .iter()
        .filter(|change| change.after.is_none() && change.path.starts_with(".agents/skills/"))
        .map(|change| change.path.as_path())
        .collect::<Vec<_>>();
    if let Err(primary) = super::user_install::prune_user_setup_empty_ancestors(
        root,
        removed_skill_paths.iter().copied(),
    ) {
        let rollback = rollback_user_projection(root, &applied);
        return match rollback {
            Ok(()) => Err(SetupError::Conflict(format!(
                "user projection cleanup failed: {primary}"
            ))),
            Err(rollback) => Err(SetupError::Conflict(format!(
                "user projection cleanup failed ({primary}); rollback failed ({rollback})"
            ))),
        };
    }
    Ok(applied)
}

/// Recreate the Hive-owned global projection when a preserving uninstall kept
/// the validated preferences but removed the projection files. This is an
/// internal reinstall path: it never asks for preferences and refuses to
/// replace a path that cannot be proven Hive-owned.
pub(crate) fn restore_saved_projection_after_uninstall(root: &Dir) -> Result<bool, SetupError> {
    let manifest_relative = Path::new(USER_PROJECTION_MANIFEST_RELATIVE);
    if super::user_install::read_user_setup_file(root, manifest_relative, MAX_USER_SETUP_BYTES)
        .map_err(SetupError::Conflict)?
        .is_some()
    {
        return Ok(false);
    }
    let Some(setup_bytes) = super::user_install::read_user_setup_file(
        root,
        Path::new(USER_SETUP_RELATIVE),
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    else {
        return Ok(false);
    };
    let Some((config, resolved_skills)) = resolved_operational_skills(root)? else {
        return Ok(false);
    };
    let projection = apply_user_projection(root, &config, &resolved_skills, &setup_bytes)?;
    Ok(!projection.changed_paths.is_empty())
}

fn validate_user_projection(
    root: &Dir,
    config: &UserSetupConfig,
    resolved_skills: &[String],
    setup_bytes: &[u8],
) -> Result<(), SetupError> {
    let planned = plan_user_projection(root, config, resolved_skills, setup_bytes)?;
    let mut drifted_paths = planned
        .reports
        .iter()
        .filter(|report| report.local_digest != report.incoming_digest)
        .map(|report| report.path.clone())
        .collect::<Vec<_>>();
    drifted_paths.extend(planned.changed_paths);
    drifted_paths.sort();
    drifted_paths.dedup();
    if drifted_paths.is_empty() {
        Ok(())
    } else {
        Err(SetupError::Conflict(format!(
            "installed user projection differs at: {}",
            drifted_paths.join(", ")
        )))
    }
}

fn rollback_user_projection(root: &Dir, applied: &AppliedProjection) -> Result<(), String> {
    for change in applied.changes.iter().rev() {
        super::user_install::replace_user_setup_file(
            root,
            &change.path,
            change.after.as_deref(),
            change.before.as_deref(),
        )?;
    }
    Ok(())
}

fn parse_projection_manifest(bytes: &[u8]) -> Result<UserProjectionManifest, SetupError> {
    let manifest: UserProjectionManifest = serde_json::from_slice(bytes).map_err(|error| {
        SetupError::Conflict(format!(
            "installed user projection manifest is invalid: {error}"
        ))
    })?;
    if !matches!(manifest.schema_version, 1 | 2)
        || manifest.product_version.is_empty()
        || !valid_sha256(&manifest.setup_digest)
    {
        return Err(SetupError::Conflict(
            "installed user projection manifest binding is invalid".to_owned(),
        ));
    }
    if manifest.schema_version == 1
        && (manifest.package_version.is_some() || !manifest.base_entries.is_empty())
    {
        return Err(SetupError::Conflict(
            "installed user projection manifest binding is invalid".to_owned(),
        ));
    }
    if manifest.schema_version == 2 {
        if manifest
            .package_version
            .as_deref()
            .is_none_or(str::is_empty)
            || manifest.base_entries.len() != manifest.entries.len()
        {
            return Err(SetupError::Conflict(
                "installed user projection manifest binding is invalid".to_owned(),
            ));
        }
        for (entry, base) in manifest.entries.iter().zip(&manifest.base_entries) {
            if entry.path != base.path
                || entry.digest != base.digest
                || sha256_digest(base.content.as_bytes()) != base.digest
            {
                return Err(SetupError::Conflict(
                    "installed user projection base inventory is invalid".to_owned(),
                ));
            }
        }
    }
    let mut previous = None;
    for entry in &manifest.entries {
        let path = Path::new(&entry.path);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
            || !valid_sha256(&entry.digest)
            || previous.is_some_and(|value: &str| value >= entry.path.as_str())
        {
            return Err(SetupError::Conflict(
                "installed user projection manifest inventory is invalid".to_owned(),
            ));
        }
        previous = Some(entry.path.as_str());
    }
    Ok(manifest)
}

pub(crate) fn uninstall_projection_paths(root: &Dir) -> Result<Vec<PathBuf>, SetupError> {
    let manifest_path = Path::new(USER_PROJECTION_MANIFEST_RELATIVE);
    let Some(bytes) =
        super::user_install::read_user_setup_file(root, manifest_path, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?
    else {
        return Ok(Vec::new());
    };
    let mut paths = parse_projection_manifest(&bytes)
        .map(|manifest| {
            manifest
                .entries
                .into_iter()
                .map(|entry| PathBuf::from(entry.path))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let absent_codex_metadata = paths
        .iter()
        .filter(|path| {
            path.starts_with(".agents/skills/")
                && path.file_name().is_some_and(|name| name == "SKILL.md")
        })
        .filter_map(|path| {
            path.parent()
                .map(|parent| parent.join("agents/openai.yaml"))
        })
        .collect::<Vec<_>>();
    paths.extend(absent_codex_metadata);
    paths.push(manifest_path.to_path_buf());
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn render_user_directive(config: &UserSetupConfig, resolved_skills: &[String]) -> Vec<u8> {
    let hosts = config
        .selected_hosts
        .iter()
        .map(|host| host.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let wiki = if config.wiki.enabled {
        format!(
            "enabled (backend={}, language={})",
            config.wiki.backend.as_str(),
            config.wiki.language.as_str()
        )
    } else {
        "disabled".to_owned()
    };
    let profile = render_user_profile(&config.profile);
    let persona = render_catalog_selection(&config.persona);
    let update_check = if config.update_check.enabled {
        "enabled"
    } else {
        "disabled"
    };
    let judge_invocation = config.judge_invocation.as_str();
    let judge_policy = config.judge_invocation.policy();
    let judge_policy_line = match judge_policy {
        JudgeInvocationPolicy::Explicit => {
            debug_assert!(!judge_policy.permits(JudgeRoute::MaterialRisk));
            (
            "- The configured `explicit` Judge policy requires a Judge only for terminal acceptance of iterative, team, or multi-goal criteria. Never invoke a Judge for material-risk work unless the policy is explicitly reconfigured to `implicit`, or for simple questions, read-only or format-only work, ticks, heartbeats, retries, deterministic failures, or an unsupported or unattested host.\n",
            "- 설정된 `explicit` Judge 정책은 iterative·team·multi-goal criterion의 terminal acceptance에만 Judge를 요구. 정책을 `implicit`으로 명시 재설정하지 않은 material-risk 작업, 단순 질문, read-only·format-only 작업, tick, heartbeat, retry, 결정적 실패, unsupported 또는 attestation 없는 host에는 Judge 호출 금지.\n",
            )
        }
        JudgeInvocationPolicy::Implicit => {
            debug_assert!(judge_policy.permits(JudgeRoute::MaterialRisk));
            (
            "- The configured `implicit` Judge policy requires a Judge for terminal acceptance of iterative, team, or multi-goal criteria and permits an additional strict material-risk route. Never invoke a Judge for simple questions, read-only or format-only work, ticks, heartbeats, retries, deterministic failures, or an unsupported or unattested host.\n",
            "- 설정된 `implicit` Judge 정책은 iterative·team·multi-goal criterion의 terminal acceptance에 Judge를 요구하고 strict material-risk route를 추가 허용. 단순 질문, read-only·format-only 작업, tick, heartbeat, retry, 결정적 실패, unsupported 또는 attestation 없는 host에는 Judge 호출 금지.\n",
            )
        }
    };
    let mut rendered = match config.interface_language {
        InterfaceLanguage::En => {
            let capture = if config.wiki.enabled {
                "- Before every final response, review the current user statement and completed outcome for one safe reusable fact, preference, workflow, decision, convention, project profile, or verified outcome. This user-level gate applies immediately after installation in every selected-host folder; project setup, a Hive harness, a project marker, or an attached collection is not required. Resolve `user-root|current-project|named-project` scope explicitly; an unregistered repository's user-global fact stays at `user-root`, and ambiguous project-specific scope fails closed.\n- For an explicit safe user-root statement, prefer `hive knowledge remember --user-root <user-root> --user-statement <normalized-fact> --claim-key <stable-key> --kind <preference|workflow|decision|convention|project-profile> --output json` exactly once; use `--request <request.json>` only for reviewed artifacts or another supported scope. Require the canonical Markdown and derived-index receipt before the final response; identical current truth is a no-op.\n- After a successful knowledge write, use `hive source-wiki lint --target <source-root> --output json` only for a valid source workspace. Otherwise use `hive knowledge lint --target <current-project-root> --user-root <user-root> --output json` for an enabled registered project, or `hive knowledge lint --target <user-root> --user-root <user-root> --output json` when the current project is unregistered. Missing project setup, a project marker, or an attached collection never skips lint.\n- Before knowledge-dependent work, run one bounded `hive knowledge retrieve --user-root <user-root> --target <current-project-root> --scope auto --query <query> --top-k 5 --byte-budget 16384 --output json`. An unregistered target falls back to user-root and shared knowledge while excluding project-private knowledge; missing project setup or collection is not a reason to skip retrieval.\n- Never record a secret, credential, confidential item without current-action authorization, ephemeral status, ambiguous inference, private path, raw transcript, complete conversation, hook payload, tool output, hidden prompt, cache, database, or runtime state.\n"
            } else {
                "- Global Wiki is disabled: do not write or refresh knowledge; preserve canonical Markdown until an explicit deletion request.\n"
            };
            format!(
                "# Aigent Hive user preferences\n\n- Setup state: `operational`\n- Interface language: `en`\n- User contexts: {profile}\n- Agent persona: {persona}\n- Selected hosts: `{hosts}`\n- Global Wiki: `{wiki}`\n- Daily update check: `{update_check}`\n- Judge invocation: `{judge_invocation}`\n- Active Skills: `{}`\n{capture}- User contexts inform only the global background. They never select a project workflow, implementation approach, delivery priority, or active Skill set.\n- When daily update check is enabled, run `hive update --check --user-root <user-root> --output json` before the first Hive task of each host session. A check may notify but must never install.\n{}- Use English for every question and response unless the user explicitly requests another language for the current response. A message written in another language does not by itself change this preference.\n- The response-language rule does not control a prompt that Hive authors or refines. Unless the user explicitly requests another language for the current prompt, write the prompt in English.\n- For ambiguous or detail-poor ordinary prompts, offer one concise optional refine suggestion without automatic rewrite.\n- Never request provider credentials or call model-provider APIs on Hive's behalf.\n",
                resolved_skills.join(", "), judge_policy_line.0
            )
        }
        InterfaceLanguage::Ko => {
            let capture = if config.wiki.enabled {
                "- 모든 최종 응답 전 현재 사용자 발화와 완료 결과에서 안전하고 재사용 가능한 사실·선호·작업 방식·결정·규약·프로젝트 특성·검증된 결과 1개를 검토. 이 사용자 범위 절차는 설치 직후 선택 호스트의 모든 폴더에 적용하며 프로젝트 설정·Hive harness·project marker·연결 collection을 전제하지 않음. `user-root|current-project|named-project` 범위를 명시적으로 결정. 미등록 repository의 사용자 전역 사실은 `user-root`에 유지하고, 모호한 project 범위는 안전하게 중단.\n- 안전한 명시적 user-root 사용자 발화에는 `hive knowledge remember --user-root <user-root> --user-statement <normalized-fact> --claim-key <stable-key> --kind <preference|workflow|decision|convention|project-profile> --output json`을 정확히 1회 우선 실행. 검토 artifact 또는 다른 지원 범위에는 `--request <request.json>` 사용. 최종 응답 전 canonical Markdown과 derived-index receipt를 확인하며, 동일한 현재 truth는 no-op.\n- 지식 기록 성공 뒤 유효한 source workspace에서만 `hive source-wiki lint --target <source-root> --output json` 실행. 그 밖에는 등록된 project에서 `hive knowledge lint --target <current-project-root> --user-root <user-root> --output json`, 미등록 project에서 `hive knowledge lint --target <user-root> --user-root <user-root> --output json` 실행. 프로젝트 설정·project marker·연결 collection 부재를 lint 건너뜀 사유로 사용 금지.\n- 지식이 필요한 작업 전 `hive knowledge retrieve --user-root <user-root> --target <current-project-root> --scope auto --query <query> --top-k 5 --byte-budget 16384 --output json`을 제한된 범위에서 1회 실행. 미등록 target은 project-private 지식을 제외한 user-root·shared 지식으로 폴백하며, 프로젝트 설정 또는 collection 부재만으로 조회를 건너뛰지 않음.\n- 현재 action 승인 없는 secret·credential·confidential 항목, ephemeral 상태, 모호한 추론, private path, raw transcript, complete conversation, hook payload, tool output, hidden prompt, cache, database, runtime state는 기록 금지.\n"
            } else {
                "- 전역 위키 비활성: knowledge 기록·갱신 금지. 명시적 삭제 요청 전까지 canonical Markdown을 보존.\n"
            };
            format!(
                "# Aigent Hive 사용자 설정\n\n- 설정 상태: `operational`\n- Interface language: `ko`\n- 사용자 기본 맥락: {profile}\n- 에이전트 페르소나: {persona}\n- 선택 호스트: `{hosts}`\n- Global Wiki: `{wiki}`\n- 일일 update 확인: `{update_check}`\n- Judge 호출: `{judge_invocation}`\n- 활성 Skill: `{}`\n{capture}- 사용자 기본 맥락은 전역 배경 정보만 제공하며 프로젝트 작업 흐름, 구현 방식, 작업 우선순위, 활성 Skill을 정하지 않음.\n- 일일 update 확인이 enabled이면 각 host session의 첫 Hive 작업 전에 `hive update --check --user-root <user-root> --output json` 실행. 확인은 알림만 가능하며 설치 금지.\n{}- 현재 응답에 다른 언어를 사용하라는 명시적 요청이 없는 한 모든 질문과 응답에 한국어 사용. 다른 언어로 작성된 메시지만으로 이 선호를 변경하지 않음.\n- 응답 언어 규칙은 Hive가 작성·개선하는 프롬프트에 적용하지 않음. 현재 프롬프트 언어를 사용자가 명시하지 않은 경우 프롬프트는 영어로 작성.\n- 모호하거나 핵심 세부가 부족한 일반 prompt에는 자동 rewrite 없이 간결한 optional refine 제안 1개만 제공.\n- Provider credential을 요청하거나 Hive를 대신해 model-provider API를 호출하지 않음.\n",
                resolved_skills.join(", "), judge_policy_line.1
            )
        }
    };
    let directive_language = match config.interface_language {
        InterfaceLanguage::En => crate::user_directives::UserDirectiveLanguage::En,
        InterfaceLanguage::Ko => crate::user_directives::UserDirectiveLanguage::Ko,
    };
    rendered.push_str(match config.interface_language {
        InterfaceLanguage::En => "- Before the first ordinary Hive task after this instruction is installed, run `hive setup feature claim --id vector-search --user-root <user-root> --output json`. When `question_required` is true, ask once whether the user wants semantic search. Explain that exact search remains available, initial setup can take time, and Windows measured runtime needs about 376MB. For yes, save `--answer yes`, then run `hive setup feature prompt --id vector-search --user-root <user-root> --output json` and provide its prompt for a new session. For no, save `--answer no` and do not ask again unless vector setup is explicitly requested. Never treat no answer or cancellation as no.\n",
        InterfaceLanguage::Ko => "- 이 지침 설치 뒤 첫 일반 Hive 작업 전 `hive setup feature claim --id vector-search --user-root <user-root> --output json` 실행. `question_required`가 true이면 의미 검색 사용 의사를 한 번 질문. 기존 정확 검색은 유지되고 처음 준비에는 시간이 걸릴 수 있으며 Windows 실측 실행 환경은 약 376MB라고 설명. 예면 `hive setup feature answer --id vector-search --answer yes --user-root <user-root> --output json` 뒤 `hive setup feature prompt --id vector-search --user-root <user-root> --output json`의 새 세션 안내문 제공. 아니요면 `--answer no`로 저장하고 사용자가 벡터 설정을 명시 요청하기 전 재질문 금지. 무응답·취소를 아니요로 기록 금지.\n",
    });
    rendered.push_str(crate::user_directives::work_completion_block(
        directive_language,
    ));
    rendered.into_bytes()
}

fn render_catalog_selection(selection: &CatalogSelection) -> String {
    selection.custom_description.as_ref().map_or_else(
        || format!("`{}`", selection.id),
        |description| format!("`{}` — {}", selection.id, markdown_code_span(description)),
    )
}

fn render_user_profile(profile: &UserProfile) -> String {
    let contexts = profile
        .contexts
        .iter()
        .map(|context| format!("`{context}`"))
        .collect::<Vec<_>>()
        .join(", ");
    match (&contexts[..], profile.description.as_deref()) {
        ("", Some(description)) => markdown_code_span(description),
        (contexts, Some(description)) => {
            format!("{contexts} — {}", markdown_code_span(description))
        }
        (contexts, None) => contexts.to_owned(),
    }
}

fn markdown_code_span(value: &str) -> String {
    let longest_run = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest_run.saturating_add(1));
    if value.starts_with(' ')
        || value.starts_with('`')
        || value.ends_with(' ')
        || value.ends_with('`')
    {
        format!("{fence} {value} {fence}")
    } else {
        format!("{fence}{value}{fence}")
    }
}

fn portable(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub(crate) fn load_operational_config(root: &Dir) -> Result<Option<UserSetupConfig>, SetupError> {
    let Some(bytes) = super::user_install::read_user_setup_file(
        root,
        Path::new(USER_SETUP_RELATIVE),
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?
    else {
        return Ok(None);
    };
    parse_and_validate_installed_config(&bytes)
        .map(Some)
        .map_err(|error| {
            SetupError::Conflict(format!(
                "installed user setup is invalid: {}",
                error.message()
            ))
        })
}

pub(crate) struct UsageThresholdUpdate {
    pub(crate) previous: u8,
    pub(crate) current: u8,
    pub(crate) changed_paths: Vec<String>,
    pub(crate) config_bytes: Vec<u8>,
}

/// Update only the authenticated global usage threshold while keeping the user projection
/// manifest bound to the new canonical preferences. Project-local thresholds are intentionally
/// outside this path.
pub(crate) fn set_operational_usage_threshold(
    user_root: &Path,
    remaining_percent: u8,
) -> Result<UsageThresholdUpdate, String> {
    if !(1..=99).contains(&remaining_percent) {
        return Err("global usage threshold must be between 1 and 99".to_owned());
    }
    let (_canonical_root, root) =
        super::user_install::open_canonical_user_root_for_setup(user_root)?;
    let prior_bytes = super::user_install::read_user_setup_file(
        &root,
        Path::new(USER_SETUP_RELATIVE),
        MAX_USER_SETUP_BYTES,
    )?
    .ok_or_else(|| {
        "global Hive setup is required before changing its usage threshold".to_owned()
    })?;
    let mut config = parse_and_validate_installed_config(&prior_bytes)
        .map_err(|error| error.message().to_owned())?;
    let previous = config.usage_guard.stop_remaining_percent;
    if previous == remaining_percent {
        return Ok(UsageThresholdUpdate {
            previous,
            current: remaining_percent,
            changed_paths: Vec::new(),
            config_bytes: prior_bytes,
        });
    }
    if let Some((project, threshold)) = config
        .usage_guard
        .project_overrides
        .iter()
        .find(|(_, threshold)| **threshold < remaining_percent)
    {
        return Err(format!(
            "global threshold {remaining_percent} would exceed the saved project override for {project} ({threshold})"
        ));
    }
    config.usage_guard.stop_remaining_percent = remaining_percent;
    validate_config_semantics(&config).map_err(|error| error.message().to_owned())?;
    let catalog = parse_and_validate_catalog().map_err(|error| error.message().to_owned())?;
    let resolved_skills =
        resolve_skills(&config, &catalog).map_err(|error| error.message().to_owned())?;
    let desired = canonical_config(&config).map_err(|error| error.message().to_owned())?;
    let projection = apply_user_projection(&root, &config, &resolved_skills, &desired)
        .map_err(|error| error.message().to_owned())?;
    if let Err(primary) = super::user_install::replace_user_setup_file(
        &root,
        Path::new(USER_SETUP_RELATIVE),
        Some(&prior_bytes),
        Some(&desired),
    ) {
        let rollback = rollback_user_projection(&root, &projection);
        return Err(match rollback {
            Ok(()) => format!("global threshold activation failed: {primary}"),
            Err(rollback) => format!(
                "global threshold activation failed ({primary}); projection rollback failed ({rollback})"
            ),
        });
    }
    let validation = load_operational_config(&root)
        .map_err(|error| error.message().to_owned())?
        .ok_or_else(|| "global Hive setup disappeared after threshold activation".to_owned())
        .and_then(|installed| {
            if installed == config {
                validate_user_projection(&root, &config, &resolved_skills, &desired)
                    .map_err(|error| error.message().to_owned())
            } else {
                Err("global Hive setup changed after threshold activation".to_owned())
            }
        });
    if let Err(primary) = validation {
        let config_rollback = super::user_install::replace_user_setup_file(
            &root,
            Path::new(USER_SETUP_RELATIVE),
            Some(&desired),
            Some(&prior_bytes),
        );
        let projection_rollback = rollback_user_projection(&root, &projection);
        return Err(match (config_rollback, projection_rollback) {
            (Ok(()), Ok(())) => primary,
            (config, projection) => format!(
                "global threshold validation failed ({primary}); rollback failed (config: {}; projection: {})",
                config.err().unwrap_or_else(|| "ok".to_owned()),
                projection.err().unwrap_or_else(|| "ok".to_owned())
            ),
        });
    }
    let mut changed_paths = projection.changed_paths;
    changed_paths.push(USER_SETUP_RELATIVE.to_owned());
    changed_paths.sort();
    changed_paths.dedup();
    Ok(UsageThresholdUpdate {
        previous,
        current: remaining_percent,
        changed_paths,
        config_bytes: desired,
    })
}

pub(crate) fn resolved_operational_skills(
    root: &Dir,
) -> Result<Option<(UserSetupConfig, Vec<String>)>, SetupError> {
    let Some(config) = load_operational_config(root)? else {
        return Ok(None);
    };
    let catalog = parse_and_validate_catalog()?;
    let skills = resolve_skills(&config, &catalog)?;
    Ok(Some((config, skills)))
}

/// Return the validated global Wiki mode without exposing setup-file bytes.
pub(crate) fn operational_wiki_preferences(user_root: &Path) -> Result<WikiPreferences, String> {
    let root = super::user_install::open_user_root_for_setup(user_root)?;
    load_operational_config(&root)
        .map_err(|error| error.message().to_owned())?
        .map(|config| config.wiki)
        .ok_or_else(|| "global Hive setup is required for shared knowledge operations".to_owned())
}

pub(crate) fn project_preferences(user_root: &Path) -> Result<GlobalProjectPreferences, String> {
    let root = super::user_install::open_user_root_for_setup(user_root)?;
    let (config, mut selected_project_skills) = resolved_operational_skills(&root)
        .map_err(|error| error.message().to_owned())?
        .ok_or_else(|| {
            "global Hive setup is required before project expedited or custom setup".to_owned()
        })?;
    selected_project_skills.retain(|name| name != "user-setup");
    selected_project_skills.sort();
    selected_project_skills.dedup();
    Ok(GlobalProjectPreferences {
        interface_language: config.interface_language.as_str().to_owned(),
        wiki_enabled: config.wiki.enabled,
        wiki_backend: config.wiki.backend.as_str().to_owned(),
        wiki_language: config.wiki.language.as_str().to_owned(),
        persona_id: config.persona.id,
        persona_custom_description: config.persona.custom_description,
        selected_project_skills,
        usage_guard_enabled: config.usage_guard.enabled,
        codexbar_fallback_enabled: config.usage_guard.codexbar_fallback_enabled,
        discord_guard_enabled: config.usage_guard.discord.enabled,
        discord_webhook_url_env: config.usage_guard.discord.webhook_url_env,
        discord_message_fields: config
            .usage_guard
            .discord
            .message_fields
            .into_iter()
            .map(DiscordMessageField::as_str)
            .map(str::to_owned)
            .collect(),
        usage_stop_remaining_percent: config.usage_guard.stop_remaining_percent,
    })
}

fn detect_state(root: &Dir) -> Result<UserSetupState, SetupError> {
    if load_operational_config(root)?.is_some() {
        return Ok(UserSetupState::Operational);
    }
    let install = Path::new(".hive/install");
    let metadata = match root.symlink_metadata(install) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(UserSetupState::Bootstrap);
        }
        Err(error) => {
            return Err(SetupError::Internal(format!(
                "cannot inspect user installation state: {error}"
            )));
        }
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        Ok(UserSetupState::SetupRequired)
    } else {
        Err(SetupError::Conflict(
            "user installation state path is not a no-follow directory".to_owned(),
        ))
    }
}

fn read_bounded_regular(path: &Path, maximum: u64) -> Result<Vec<u8>, SetupError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        SetupError::Input(format!(
            "cannot inspect setup answers {}: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return Err(SetupError::Input(format!(
            "setup answers must be a bounded no-follow regular file: {}",
            path.display()
        )));
    }
    let mut file = fs::File::open(path).map_err(|error| {
        SetupError::Input(format!(
            "cannot open setup answers {}: {error}",
            path.display()
        ))
    })?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    Read::by_ref(&mut file)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| {
            SetupError::Input(format!(
                "cannot read setup answers {}: {error}",
                path.display()
            ))
        })?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > maximum {
        return Err(SetupError::Conflict(format!(
            "setup answers changed during read: {}",
            path.display()
        )));
    }
    Ok(bytes)
}

#[allow(clippy::too_many_arguments)]
fn success(
    arguments: &Arguments,
    config: &UserSetupConfig,
    resolved_skills: &[String],
    state: UserSetupState,
    changed_paths: Vec<String>,
    code: &'static str,
    message: &'static str,
    answer_bytes: &[u8],
    desired: &[u8],
    projection_reports: &[UserProjectionPathReport],
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "SetupHiveUser",
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![
            Evidence {
                kind: "file",
                locator: arguments.answers.display().to_string(),
                digest: sha256_digest(answer_bytes),
            },
            Evidence {
                kind: "release",
                locator: "harness/user-setup/catalog.yml".to_owned(),
                digest: sha256_digest(USER_SETUP_CATALOG.as_bytes()),
            },
            Evidence {
                kind: "report",
                locator: USER_SETUP_RELATIVE.to_owned(),
                digest: sha256_digest(desired),
            },
        ],
        next_action: None,
        data: Some(json!({
            "setup_state": state.as_str(),
            "user_root": arguments.user_root,
            "interface_language": config.interface_language,
            "wiki": config.wiki,
            "profile": config.profile,
            "persona": config.persona,
            "selected_hosts": config.selected_hosts,
            "resolved_skills": resolved_skills,
            "update_check": config.update_check,
            "judge_invocation": config.judge_invocation,
            "usage_guard": config.usage_guard,
            "user_projection": {
                "paths": projection_reports,
            },
        })),
    }
}

fn failure(error: &SetupError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "SetupHiveUser",
        status: error.status(),
        exit_code: error.exit_code(),
        code: match error {
            SetupError::Input(_) => "hive.invalid-input",
            SetupError::Conflict(_) => "hive.user-setup-conflict",
            SetupError::Verification(_) => "hive.user-setup-verification-failed",
            SetupError::Internal(_) => "hive.internal-error",
        },
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> UserSetupConfig {
        parse_and_validate_config(
            br"
schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: both
profile:
  id: web-developer
persona:
  id: balanced
selected_hosts:
  - codex
  - claude
skills:
  mode: individual
  selected:
    - user-setup
usage_guard:
  enabled: false
  stop_remaining_percent: 20
  codexbar_fallback_enabled: false
",
        )
        .expect("valid config")
    }

    #[test]
    fn legacy_user_setup_defaults_to_explicit_judge_invocation() {
        assert_eq!(valid_config().judge_invocation, JudgeInvocation::Explicit);
    }

    #[test]
    fn judge_invocation_accepts_only_the_two_persisted_modes() {
        for (value, expected) in [
            ("explicit", JudgeInvocation::Explicit),
            ("implicit", JudgeInvocation::Implicit),
        ] {
            let answers =
                String::from_utf8(canonical_config(&valid_config()).expect("config bytes"))
                    .expect("UTF-8 config")
                    .replace(
                        "judge_invocation: explicit",
                        &format!("judge_invocation: {value}"),
                    );
            let config =
                parse_and_validate_config(answers.as_bytes()).expect("configured Judge invocation");
            assert_eq!(config.judge_invocation, expected);
            assert!(
                String::from_utf8(canonical_config(&config).expect("config bytes"))
                    .expect("UTF-8 config")
                    .contains(&format!("judge_invocation: {value}\n"))
            );
        }

        let invalid = String::from_utf8(canonical_config(&valid_config()).expect("config bytes"))
            .expect("UTF-8 config")
            .replace("judge_invocation: explicit", "judge_invocation: automatic");
        assert!(parse_and_validate_config(invalid.as_bytes()).is_err());
    }

    #[test]
    fn persisted_judge_invocation_maps_to_the_closed_core_policy() {
        assert_eq!(
            JudgeInvocation::Explicit.policy(),
            JudgeInvocationPolicy::Explicit
        );
        assert_eq!(
            JudgeInvocation::Implicit.policy(),
            JudgeInvocationPolicy::Implicit
        );
    }

    #[test]
    fn describe_places_judge_invocation_before_skill_selection() {
        let described = describe_result().expect("describe result");
        let data = described.data.expect("describe data");
        let question_order = data["question_order"]
            .as_array()
            .expect("question order")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();

        assert_eq!(
            question_order,
            vec![
                "interface-language",
                "daily-update-check",
                "setup-mode",
                "wiki",
                "user-contexts",
                "persona",
                "hosts",
                "judge-invocation",
                "skills",
                "usage-guard",
                "discord",
            ]
        );
        assert_eq!(data["answer_template"]["judge_invocation"], "explicit");
        assert_eq!(
            data["schema"]["$defs"]["judge_invocation"]["enum"],
            serde_json::json!(["explicit", "implicit"])
        );
    }

    #[test]
    fn cli_parse_uses_the_physical_user_root_after_no_follow_validation() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let requested = temporary.path().to_path_buf();
        let expected = requested.canonicalize().expect("canonical user root");
        let answers = requested.join("answers.yml");
        let arguments = vec![
            "--scope".to_owned(),
            "user".to_owned(),
            "--answers".to_owned(),
            answers.display().to_string(),
            "--user-root".to_owned(),
            requested.display().to_string(),
            "--dry-run".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let parsed = parse(&arguments).expect("parse user setup arguments");

        assert_eq!(parsed.user_root, expected);
    }

    fn write_projection_manifest(root: &Path, manifest: &UserProjectionManifest) {
        let path = root.join(USER_PROJECTION_MANIFEST_RELATIVE);
        fs::create_dir_all(path.parent().expect("manifest parent")).expect("manifest parent");
        let mut bytes = serde_json::to_vec_pretty(manifest).expect("manifest JSON");
        bytes.push(b'\n');
        fs::write(path, bytes).expect("projection manifest");
    }

    fn schema_two_manifest(path: &Path, base: &[u8]) -> UserProjectionManifest {
        let entry = UserProjectionEntry {
            path: portable(path),
            digest: sha256_digest(base),
        };
        UserProjectionManifest {
            schema_version: 2,
            product_version: env!("CARGO_PKG_VERSION").to_owned(),
            package_version: Some(env!("HIVE_PACKAGE_VERSION").to_owned()),
            setup_digest: sha256_digest(b"fixture setup\n"),
            entries: vec![entry.clone()],
            base_entries: vec![UserProjectionBaseEntry {
                path: entry.path,
                digest: entry.digest,
                content: String::from_utf8(base.to_vec()).expect("UTF-8 base"),
            }],
        }
    }

    fn seed_legacy_070_projection(
        root: &Path,
        config: &UserSetupConfig,
    ) -> (Vec<String>, Vec<u8>, usize) {
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(config, &catalog).expect("skill closure");
        let old_files = legacy_070_projection_files(config, &skills).expect("legacy files");
        for (relative, bytes) in &old_files {
            let full = root.join(relative);
            fs::create_dir_all(full.parent().expect("projection parent"))
                .expect("projection parent");
            fs::write(full, bytes).expect("legacy projection bytes");
        }
        let answers = canonical_config(config).expect("answers");
        let answer_path = root.join(USER_SETUP_RELATIVE);
        fs::create_dir_all(answer_path.parent().expect("answers parent")).expect("answers parent");
        fs::write(answer_path, &answers).expect("saved answers");
        let manifest = UserProjectionManifest {
            schema_version: 1,
            product_version: "0.7.0".to_owned(),
            package_version: None,
            setup_digest: sha256_digest(&answers),
            entries: old_files
                .iter()
                .map(|(relative, bytes)| UserProjectionEntry {
                    path: portable(relative),
                    digest: sha256_digest(bytes),
                })
                .collect(),
            base_entries: Vec::new(),
        };
        write_projection_manifest(root, &manifest);
        (skills, answers, old_files.len())
    }

    fn seeded_projection_plan(
        base: &[u8],
        local: Vec<u8>,
    ) -> (AppliedProjection, Vec<u8>, PathBuf) {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let files = user_projection_files(&config, &skills).expect("desired files");
        let path = PathBuf::from(".agents/skills/user-setup/SKILL.md");
        let incoming = files.get(&path).expect("user-setup source").clone();
        let full = temporary.path().join(&path);
        fs::create_dir_all(full.parent().expect("skill parent")).expect("skill parent");
        fs::write(&full, local).expect("local Skill");
        write_projection_manifest(temporary.path(), &schema_two_manifest(&path, base));
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");
        let planned = plan_user_projection(
            &root,
            &config,
            &skills,
            &canonical_config(&config).expect("answers"),
        )
        .expect("projection plan");
        (planned, incoming, path)
    }

    #[test]
    fn user_setup_help_describes_all_modes() {
        assert!(USER_SETUP_USAGE.contains("--scope user"));
        assert!(USER_SETUP_USAGE.contains("--dry-run|--apply|--validate"));
        assert!(USER_SETUP_USAGE.contains("--output json"));
    }

    #[test]
    fn validation_uses_installed_projection_binding_for_equivalent_answers() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let answers = canonical_config(&config).expect("canonical answers");
        let installed = [b"# locally preserved formatting\n".as_slice(), &answers].concat();
        let setup_path = temporary.path().join(USER_SETUP_RELATIVE);
        fs::create_dir_all(setup_path.parent().expect("setup parent")).expect("setup parent");
        fs::write(&setup_path, &installed).expect("installed setup");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");
        apply_user_projection(&root, &config, &skills, &installed).expect("installed projection");
        let manifest_path = temporary.path().join(USER_PROJECTION_MANIFEST_RELATIVE);
        let manifest_before = fs::read(&manifest_path).expect("installed manifest");

        assert!(validate_user_projection(&root, &config, &skills, &answers).is_err());
        validate_user_projection(&root, &config, &skills, &installed)
            .expect("installed binding validates without rewriting the receipt");
        assert_eq!(
            fs::read(&manifest_path).expect("manifest after validation"),
            manifest_before
        );
    }

    #[test]
    fn validation_keeps_modified_or_malformed_projection_fail_closed() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let answers = canonical_config(&config).expect("answers");
        let setup_path = temporary.path().join(USER_SETUP_RELATIVE);
        fs::create_dir_all(setup_path.parent().expect("setup parent")).expect("setup parent");
        fs::write(&setup_path, &answers).expect("installed setup");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");
        apply_user_projection(&root, &config, &skills, &answers).expect("installed projection");
        let answer_path = temporary.path().join("answers.yml");
        fs::write(&answer_path, &answers).expect("answers");
        let manifest_path = temporary.path().join(USER_PROJECTION_MANIFEST_RELATIVE);
        let manifest = fs::read(&manifest_path).expect("installed manifest");
        let projection_path = temporary.path().join(".agents/skills/user-setup/SKILL.md");
        let projection = fs::read(&projection_path).expect("installed projection");
        fs::write(&projection_path, b"foreign local change\n").expect("local change");

        let arguments = Arguments {
            answers: answer_path.clone(),
            mode: SetupMode::Validate,
            user_root: temporary
                .path()
                .canonicalize()
                .expect("canonical user root"),
            root_cap: super::super::user_install::open_user_root_for_setup(temporary.path())
                .expect("validation root"),
        };
        let Err(error) = execute(&arguments) else {
            panic!("local projection change must fail");
        };
        assert_eq!(error.status(), "conflict");
        assert!(error
            .message()
            .contains(".agents/skills/user-setup/SKILL.md"));

        fs::write(
            temporary.path().join(USER_PROJECTION_MANIFEST_RELATIVE),
            b"not a projection manifest\n",
        )
        .expect("malformed manifest");
        let arguments = Arguments {
            answers: answer_path.clone(),
            mode: SetupMode::Validate,
            user_root: temporary
                .path()
                .canonicalize()
                .expect("canonical user root"),
            root_cap: super::super::user_install::open_user_root_for_setup(temporary.path())
                .expect("validation root"),
        };
        let Err(error) = execute(&arguments) else {
            panic!("malformed receipt must fail");
        };
        assert_eq!(error.status(), "conflict");
        assert!(error.message().contains("manifest is invalid"));

        fs::write(&manifest_path, manifest).expect("restore manifest");
        fs::write(&projection_path, projection).expect("restore projection");
        fs::write(&setup_path, b"schema_version: not-a-number\n")
            .expect("corrupted installed setup");
        let arguments = Arguments {
            answers: answer_path,
            mode: SetupMode::Validate,
            user_root: temporary
                .path()
                .canonicalize()
                .expect("canonical user root"),
            root_cap: super::super::user_install::open_user_root_for_setup(temporary.path())
                .expect("validation root"),
        };
        let Err(error) = execute(&arguments) else {
            panic!("corrupted installed setup must fail");
        };
        assert_eq!(error.status(), "conflict");
        assert!(error.message().contains("installed user setup is invalid"));
    }

    #[test]
    fn user_projection_replaces_an_authenticated_vanilla_base() {
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let files = user_projection_files(&config, &skills).expect("desired files");
        let path = PathBuf::from(".agents/skills/user-setup/SKILL.md");
        let incoming = files.get(&path).expect("user-setup source").clone();
        let mut base = incoming.clone();
        let replaced = String::from_utf8(base.clone())
            .expect("UTF-8 Skill")
            .replace("# Setup Hive", "# Earlier Setup Hive");
        base = replaced.into_bytes();

        let (planned, expected, target) = seeded_projection_plan(&base, base.clone());
        let report = planned
            .reports
            .iter()
            .find(|report| report.path == portable(&target))
            .expect("target report");

        assert_eq!(expected, incoming);
        assert_eq!(report.disposition, MergeDisposition::IncomingReplace);
        assert!(!report.local_priority);
        assert!(planned
            .changes
            .iter()
            .any(|change| change.path == target
                && change.after.as_deref() == Some(incoming.as_slice())));
    }

    #[test]
    fn user_projection_merges_disjoint_local_edits_and_retains_overlaps() {
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let files = user_projection_files(&config, &skills).expect("desired files");
        let path = PathBuf::from(".agents/skills/user-setup/SKILL.md");
        let incoming = files.get(&path).expect("user-setup source").clone();
        let base = String::from_utf8(incoming)
            .expect("UTF-8 Skill")
            .replace("# Setup Hive", "# Earlier Setup Hive")
            .into_bytes();

        let mut disjoint_local = base.clone();
        disjoint_local.extend_from_slice(b"\n<!-- local note -->\n");
        let (merged, _, target) = seeded_projection_plan(&base, disjoint_local);
        let merged_report = merged
            .reports
            .iter()
            .find(|report| report.path == portable(&target))
            .expect("merged report");
        assert_eq!(merged_report.disposition, MergeDisposition::Merged);
        assert!(merged_report.local_priority);
        assert_eq!(merged_report.omitted_incoming_hunks, 0);
        assert!(merged
            .changes
            .iter()
            .find(|change| change.path == target)
            .and_then(|change| change.after.as_deref())
            .is_some_and(|bytes| bytes.ends_with(b"<!-- local note -->\n")));

        let overlap_local = String::from_utf8(base.clone())
            .expect("UTF-8 base")
            .replace("# Earlier Setup Hive", "# Local Setup Hive")
            .into_bytes();
        let (overlap, _, target) = seeded_projection_plan(&base, overlap_local);
        let overlap_report = overlap
            .reports
            .iter()
            .find(|report| report.path == portable(&target))
            .expect("overlap report");
        assert!(overlap_report.local_priority);
        assert!(overlap_report.omitted_incoming_hunks > 0);
        assert!(!overlap.changes.iter().any(|change| change.path == target));
    }

    #[test]
    fn user_projection_removes_an_authenticated_retired_global_skill_and_empty_leaf() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let answers = canonical_config(&config).expect("answers");
        let retired = PathBuf::from(".agents/skills/hive-knowledge-capture/SKILL.md");
        let historical = include_bytes!(
            "../../../harness/project-bases/0.7.0/skills/hive-knowledge-capture/SKILL.md"
        );
        let full = temporary.path().join(&retired);
        fs::create_dir_all(full.parent().expect("retired parent")).expect("retired parent");
        fs::write(&full, historical).expect("retired Hive Skill");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        let planned = plan_user_projection(&root, &config, &skills, &answers)
            .expect("authenticated retired cleanup preview");
        assert!(planned.changes.iter().any(|change| {
            change.path == retired
                && change.before.as_deref() == Some(historical.as_slice())
                && change.after.is_none()
        }));

        apply_user_projection(&root, &config, &skills, &answers)
            .expect("authenticated retired cleanup apply");
        assert!(!full.exists());
        assert!(!temporary
            .path()
            .join(".agents/skills/hive-knowledge-capture")
            .exists());
    }

    #[test]
    fn user_projection_preserves_a_modified_or_foreign_retired_name() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let answers = canonical_config(&config).expect("answers");
        let retired = PathBuf::from(".agents/skills/hive-knowledge-capture/SKILL.md");
        let full = temporary.path().join(&retired);
        fs::create_dir_all(full.parent().expect("retired parent")).expect("retired parent");
        let foreign = b"---\nname: hive-knowledge-capture\n---\n# User Skill\n";
        fs::write(&full, foreign).expect("foreign Skill");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        let planned = plan_user_projection(&root, &config, &skills, &answers)
            .expect("foreign retired-name preview");
        assert!(!planned.changes.iter().any(|change| change.path == retired));
        assert_eq!(fs::read(full).expect("foreign Skill retained"), foreign);
    }

    #[test]
    fn legacy_user_projection_without_an_authenticated_base_stays_unchanged() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let path = PathBuf::from(".agents/skills/user-setup/SKILL.md");
        let full = temporary.path().join(&path);
        fs::create_dir_all(full.parent().expect("skill parent")).expect("skill parent");
        let local = b"local-only setup instructions\n".to_vec();
        fs::write(&full, &local).expect("local Skill");
        let manifest = UserProjectionManifest {
            schema_version: 1,
            product_version: "0.8.0".to_owned(),
            package_version: None,
            setup_digest: sha256_digest(b"fixture setup\n"),
            entries: vec![UserProjectionEntry {
                path: portable(&path),
                digest: sha256_digest(&local),
            }],
            base_entries: Vec::new(),
        };
        write_projection_manifest(temporary.path(), &manifest);
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");

        let Err(error) = plan_user_projection(
            &root,
            &config,
            &skills,
            &canonical_config(&config).expect("answers"),
        ) else {
            panic!("unknown base must stop before writes");
        };

        assert_eq!(error.status(), "conflict");
        assert!(error.message().contains("No files were changed"));
        assert_eq!(fs::read(full).expect("unchanged local Skill"), local);
    }

    #[test]
    fn test_three_global_projection_has_an_authenticated_vanilla_upgrade_base() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("skill closure");
        let mut old_files = user_projection_files(&config, &skills).expect("old files");
        let path = PathBuf::from(".agents/skills/user-setup/SKILL.md");
        old_files.insert(path.clone(), USER_PROJECTION_090_TEST3_SETUP_HIVE.to_vec());
        for (relative, bytes) in &old_files {
            let full = temporary.path().join(relative);
            fs::create_dir_all(full.parent().expect("projection parent"))
                .expect("projection parent");
            fs::write(full, bytes).expect("old projection bytes");
        }
        let answers = canonical_config(&config).expect("answers");
        let answer_path = temporary.path().join(USER_SETUP_RELATIVE);
        fs::create_dir_all(answer_path.parent().expect("answers parent")).expect("answers parent");
        fs::write(answer_path, &answers).expect("saved answers");
        let manifest = UserProjectionManifest {
            schema_version: 1,
            product_version: "0.9.0".to_owned(),
            package_version: None,
            setup_digest: sha256_digest(&answers),
            entries: old_files
                .iter()
                .map(|(relative, bytes)| UserProjectionEntry {
                    path: portable(relative),
                    digest: sha256_digest(bytes),
                })
                .collect(),
            base_entries: Vec::new(),
        };
        write_projection_manifest(temporary.path(), &manifest);
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        let planned = plan_user_projection(&root, &config, &skills, &answers)
            .expect("test.3 vanilla upgrade preview");
        let report = planned
            .reports
            .iter()
            .find(|report| report.path == portable(&path))
            .expect("user-setup report");

        assert_eq!(report.disposition, MergeDisposition::IncomingReplace);
        assert!(!report.local_priority);
    }

    #[test]
    fn legacy_070_global_projection_has_an_authenticated_vanilla_upgrade_base() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let (skills, answers, old_file_count) =
            seed_legacy_070_projection(temporary.path(), &config);
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        let planned = plan_user_projection(&root, &config, &skills, &answers)
            .expect("0.7.0 vanilla upgrade preview");
        let manifest = planned
            .changes
            .iter()
            .find(|change| change.path == Path::new(USER_PROJECTION_MANIFEST_RELATIVE))
            .and_then(|change| change.after.as_deref())
            .expect("schema-2 manifest update");
        let upgraded: UserProjectionManifest =
            serde_json::from_slice(manifest).expect("upgraded projection manifest");

        assert_eq!(upgraded.schema_version, 2);
        assert_eq!(upgraded.product_version, env!("CARGO_PKG_VERSION"));
        assert!(upgraded.package_version.is_some());
        let catalog = parse_and_validate_catalog().expect("catalog");
        let current_skills = resolve_skills(&config, &catalog).expect("skill closure");
        let current_files = user_projection_files(&config, &current_skills).expect("current files");
        assert!(current_files.len() > old_file_count);
        assert_eq!(upgraded.base_entries.len(), current_files.len());
        assert!(planned.changes.iter().any(|change| {
            change
                .path
                .to_string_lossy()
                .ends_with("/agents/openai.yaml")
                && change.before.is_none()
                && change.after.is_some()
        }));
    }

    #[test]
    fn legacy_070_global_projection_rejects_a_local_edit_before_schema_two_upgrade() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let (skills, answers, _) = seed_legacy_070_projection(temporary.path(), &config);
        let path = PathBuf::from(".agents/skills/user-setup/SKILL.md");
        let full = temporary.path().join(&path);
        let mut local = fs::read(&full).expect("legacy user-setup");
        local.extend_from_slice(b"\n<!-- local note -->\n");
        fs::write(&full, &local).expect("local edit");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        let Err(error) = plan_user_projection(&root, &config, &skills, &answers) else {
            panic!("legacy local edit must stop before schema-2 upgrade");
        };

        assert_eq!(error.status(), "conflict");
        assert!(error.message().contains("No files were changed"));
        assert_eq!(fs::read(full).expect("local edit retained"), local);
    }

    #[test]
    fn legacy_070_global_projection_rejects_an_unknown_inventory() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = valid_config();
        let (skills, answers, _) = seed_legacy_070_projection(temporary.path(), &config);
        let manifest_path = temporary.path().join(USER_PROJECTION_MANIFEST_RELATIVE);
        let mut manifest: UserProjectionManifest =
            serde_json::from_slice(&fs::read(&manifest_path).expect("legacy manifest"))
                .expect("legacy manifest JSON");
        manifest.entries.pop();
        write_projection_manifest(temporary.path(), &manifest);
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        let Err(error) = plan_user_projection(&root, &config, &skills, &answers) else {
            panic!("unknown 0.7.0 inventory must stop");
        };

        assert_eq!(error.status(), "conflict");
        assert!(error.message().contains("No files were changed"));
    }

    #[test]
    fn embedded_user_setup_catalog_is_valid() {
        let catalog = parse_and_validate_catalog().expect("catalog");
        assert_eq!(catalog.schema_version, 1);
        assert_eq!(catalog.mandatory_skills, ["user-setup"]);
        assert!(catalog.optional_third_party_skills.is_empty());
    }

    #[test]
    fn all_skill_mode_resolves_every_implemented_builtin() {
        let mut config = valid_config();
        config.skills = SkillPreferences {
            mode: SkillSelectionMode::All,
            selected: Vec::new(),
        };
        let catalog = parse_and_validate_catalog().expect("catalog");
        let expected = embedded_catalog()
            .expect("built-in catalog")
            .skills
            .into_iter()
            .filter(|entry| {
                matches!(
                    entry.availability,
                    hive_projection::Availability::Implemented
                )
            })
            .map(|entry| entry.name)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            resolve_skills(&config, &catalog)
                .expect("all built-ins")
                .into_iter()
                .collect::<BTreeSet<_>>(),
            expected
        );
    }

    #[test]
    fn legacy_recommended_selection_requires_saved_config_and_preserves_its_closure() {
        let legacy = br"
schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: both
profile:
  id: web-developer
persona:
  id: balanced
selected_hosts:
  - codex
skills:
  mode: recommended
  recommended_suite: web-developer
usage_guard:
  stop_remaining_percent: 20
";
        let error = parse_and_validate_config(legacy).expect_err("new quick-answer rejects suite");
        assert!(error.message().contains("no longer accepted"));

        let migrated =
            parse_and_validate_installed_config(legacy).expect("existing setting migration");
        assert_eq!(migrated.skills.mode, SkillSelectionMode::Individual);
        assert!(migrated.skills.selected.contains(&"usage-guard".to_owned()));
        assert!(!migrated
            .skills
            .selected
            .contains(&"quick-answer".to_owned()));
        let canonical = String::from_utf8(canonical_config(&migrated).expect("new format"))
            .expect("UTF-8 config");
        assert!(canonical.contains("mode: individual"));
        assert!(!canonical.contains("recommended_suite"));
    }

    #[test]
    fn legacy_individual_skill_names_migrate_to_short_public_names() {
        let legacy = br"
schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: both
profile:
  id: web-developer
persona:
  id: balanced
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - setup-hive
    - hive-knowledge-capture
    - ai-slop-cleaner
usage_guard:
  stop_remaining_percent: 20
";
        let config = parse_and_validate_config(legacy).expect("legacy individual selection");

        assert_eq!(
            config.skills.selected,
            ["code-polish", "knowledge-capture", "user-setup"]
        );
        let canonical = String::from_utf8(canonical_config(&config).expect("canonical config"))
            .expect("UTF-8 canonical config");
        assert!(canonical.contains("- code-polish"));
        assert!(!canonical.contains("ai-slop-cleaner"));
    }

    #[test]
    fn legacy_single_profile_migrates_to_user_contexts_without_losing_description() {
        let migrated = parse_and_validate_config(
            r"
schema_version: 1
interface_language: ko
wiki:
  language: both
profile:
  id: custom
  custom_description: 웹과 게임을 함께 만듦
persona:
  id: balanced
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - user-setup
usage_guard:
  stop_remaining_percent: 20
"
            .as_bytes(),
        )
        .expect("legacy profile migration");

        assert!(migrated.profile.contexts.is_empty());
        assert_eq!(
            migrated.profile.description.as_deref(),
            Some("웹과 게임을 함께 만듦")
        );
        let canonical = String::from_utf8(canonical_config(&migrated).expect("canonical config"))
            .expect("UTF-8 config");
        assert!(canonical.contains("contexts: []"));
        assert!(canonical.contains("description: 웹과 게임을 함께 만듦"));
        assert!(!canonical.contains("custom_description"));
    }

    #[test]
    fn user_profile_accepts_multiple_contexts_without_affecting_skill_resolution() {
        let config = parse_and_validate_config(
            r"
schema_version: 1
interface_language: ko
wiki:
  language: ko
profile:
  contexts:
    - web-developer
    - game-developer
  description: 웹과 게임을 함께 만듦
persona:
  id: strict
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - quick-answer
usage_guard:
  stop_remaining_percent: 20
"
            .as_bytes(),
        )
        .expect("multiple contexts");
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("skill resolution"),
            ["quick-answer", "user-setup"]
        );
    }

    #[test]
    fn legacy_defaults_enable_wiki_and_keep_guard_native_first() {
        let config = parse_and_validate_installed_config(
            br"
schema_version: 1
interface_language: en
wiki:
  language: both
profile:
  id: non-developer
persona:
  id: friendly
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - user-setup
usage_guard:
  stop_remaining_percent: 20
",
        )
        .expect("defaults");

        assert!(config.wiki.enabled);
        assert!(!config.usage_guard.enabled);
        assert_eq!(config.usage_guard.stop_remaining_percent, 20);
        assert!(!config.usage_guard.codexbar_fallback_enabled);
        assert!(!config.usage_guard.discord.enabled);
        assert!(config.usage_guard.discord.webhook_url_env.is_none());
        assert_eq!(
            config.usage_guard.discord.request_privacy,
            DiscordRequestPrivacy::Summary
        );
        assert_eq!(
            config.usage_guard.discord.message_fields,
            default_discord_message_fields()
        );
    }

    #[test]
    fn new_setup_requires_a_user_selected_usage_threshold() {
        let error = parse_and_validate_config(
            br"
schema_version: 1
interface_language: en
wiki:
  language: both
profile:
  contexts:
    - non-developer
persona:
  id: friendly
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - user-setup
usage_guard: {}
",
        )
        .expect_err("new setup rejects an omitted threshold");

        assert!(error.message().contains("stop_remaining_percent"));
    }

    #[test]
    fn progress_preserves_only_nonsecret_answers_and_the_next_step() {
        let user_root = tempfile::tempdir().expect("temporary user root");
        let answers = user_root.path().join("answers.yml");
        std::fs::write(
            &answers,
            "interface_language: ko\nusage_guard:\n  enabled: true\n  stop_remaining_percent: 37\n",
        )
        .expect("write answers");
        let save = parse_progress(&[
            "save".to_owned(),
            "--scope".to_owned(),
            "user".to_owned(),
            "--step".to_owned(),
            "discord-test".to_owned(),
            "--answers".to_owned(),
            answers.display().to_string(),
            "--user-root".to_owned(),
            user_root.path().display().to_string(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("progress save arguments");
        let saved = execute_progress(save).expect("progress save");
        assert_eq!(saved.code, "hive.user-setup-progress-saved");

        let status = parse_progress(&[
            "status".to_owned(),
            "--scope".to_owned(),
            "user".to_owned(),
            "--user-root".to_owned(),
            user_root.path().display().to_string(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("progress status arguments");
        let inspected = execute_progress(status).expect("progress status");
        assert_eq!(inspected.code, "hive.user-setup-progress-status");
        assert_eq!(inspected.data.expect("status data")["step"], "discord-test");

        std::fs::write(
            &answers,
            "usage_guard:\n  webhook_url: https://discord.example/webhook\n",
        )
        .expect("write forbidden answers");
        let rejected = parse_progress(&[
            "save".to_owned(),
            "--scope".to_owned(),
            "user".to_owned(),
            "--step".to_owned(),
            "discord-test".to_owned(),
            "--answers".to_owned(),
            answers.display().to_string(),
            "--user-root".to_owned(),
            user_root.path().display().to_string(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .and_then(execute_progress);
        let Err(rejected) = rejected else {
            panic!("progress accepts a webhook URL");
        };
        assert!(rejected.message().contains("webhook URL"));
    }

    #[test]
    fn project_override_requires_a_registered_project_identity() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let mut config = valid_config();
        config
            .usage_guard
            .project_overrides
            .insert("not-registered".to_owned(), 30);

        let error = validate_registered_project_overrides(&config, temporary.path())
            .expect_err("unregistered project override");

        assert!(error.message().contains("registered-project"));
    }

    #[test]
    fn discord_usage_notification_requires_guard_and_environment_name() {
        let mut config = valid_config();
        config.usage_guard.enabled = true;
        config.usage_guard.discord.enabled = true;
        config.usage_guard.discord.webhook_url_env = Some("HIVE_DISCORD_WEBHOOK_URL".to_owned());
        config.usage_guard.discord.request_privacy = DiscordRequestPrivacy::RawPrompt;
        validate_config_semantics(&config).expect("enabled Discord notification");

        config.usage_guard.discord.webhook_url_env = Some("discord_webhook".to_owned());
        let error = validate_config_semantics(&config).expect_err("lowercase environment rejected");
        assert!(error.message().contains("webhook_url_env"));

        config.usage_guard.discord.webhook_url_env = Some("HIVE_DISCORD_WEBHOOK_URL".to_owned());
        config.usage_guard.enabled = false;
        let error = validate_config_semantics(&config).expect_err("guard dependency rejected");
        assert!(error.message().contains("usage guard"));

        config.usage_guard.enabled = true;
        config.usage_guard.discord.message_fields = vec![
            DiscordMessageField::Project,
            DiscordMessageField::RemainingUsage,
        ];
        validate_config_semantics(&config).expect("ordered Discord fields accepted");
        let canonical = String::from_utf8(canonical_config(&config).expect("canonical config"))
            .expect("UTF-8 config");
        assert!(canonical.contains("message_fields:\n    - project\n    - remaining-usage"));

        config.usage_guard.discord.message_fields.clear();
        let error = validate_config_semantics(&config).expect_err("empty fields rejected");
        assert!(error.message().contains("message_fields"));
    }

    #[test]
    fn notion_wiki_is_rejected_by_the_v0_9_user_setup_schema() {
        let error = parse_and_validate_config(
            br"
schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: both
  backend: notion
  notion:
    workspace_id: workspace-a
    scope_id: scope-a
    local_index_consent: true
profile:
  id: non-developer
persona:
  id: balanced
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - user-setup
usage_guard:
  stop_remaining_percent: 20
",
        )
        .expect_err("v0.9 must not accept a Notion user setup");
        assert!(error.message().contains("/wiki/backend"));
    }

    #[test]
    fn disabled_wiki_removes_knowledge_skills_from_operational_projection() {
        let config = parse_and_validate_config(
            br"
schema_version: 1
interface_language: ko
wiki:
  enabled: false
  language: ko
profile:
  id: web-developer
persona:
  id: strict
selected_hosts:
  - codex
skills:
  mode: individual
  selected:
    - knowledge-capture
    - knowledge-maintain
usage_guard:
  enabled: true
  stop_remaining_percent: 20
",
        )
        .expect("config");
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            ["usage-guard", "user-setup"]
        );
    }

    #[test]
    fn enabled_wiki_resolves_knowledge_skill_dependency_closure() {
        let mut config = valid_config();
        config.skills.selected = vec!["knowledge-capture".to_owned()];
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            ["knowledge-capture", "knowledge-recall", "user-setup",]
        );
    }

    #[test]
    fn enabled_wiki_skill_resolves_the_complete_reused_knowledge_stack() {
        let mut config = valid_config();
        config.skills.selected = vec!["knowledge-maintain".to_owned()];
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            ["knowledge-maintain", "knowledge-recall", "user-setup",]
        );
    }

    #[test]
    fn disabled_usage_guard_preserves_an_explicitly_selected_control_skill() {
        let mut config = valid_config();
        config.skills.selected = vec!["usage-guard".to_owned()];
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            ["usage-guard", "user-setup"]
        );
    }

    #[test]
    fn reconfigure_rejects_host_deselection_with_actionable_conflict() {
        let installed = valid_config();
        let installed_bytes = canonical_config(&installed).expect("installed bytes");
        let mut desired = installed;
        desired.selected_hosts = vec![SelectedHost::Codex];

        let error = reject_host_deselection(Some(&installed_bytes), &desired)
            .expect_err("host deselection must fail closed");

        assert_eq!(error.status(), "conflict");
        assert_eq!(
            error.message(),
            "cannot remove configured hosts until transactional host deactivation is available; keep these selected_hosts entries: claude"
        );
    }

    #[test]
    fn reconfigure_allows_host_addition_without_deselection() {
        let mut installed = valid_config();
        installed.selected_hosts = vec![SelectedHost::Codex];
        let installed_bytes = canonical_config(&installed).expect("installed bytes");
        let mut desired = installed;
        desired.selected_hosts = vec![SelectedHost::Codex, SelectedHost::Claude];

        reject_host_deselection(Some(&installed_bytes), &desired).expect("host addition");
    }

    #[test]
    fn disabled_usage_guard_rejects_codexbar_fallback() {
        let mut config = valid_config();
        config.usage_guard.codexbar_fallback_enabled = true;

        let error =
            validate_config_semantics(&config).expect_err("disabled guard cannot use fallback");

        assert_eq!(
            error.message(),
            "codexbar_fallback_enabled must be false when the usage guard is disabled"
        );
    }

    #[test]
    fn user_profile_description_limit_counts_unicode_scalars() {
        let accepted = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("한".repeat(500)),
        };
        validate_custom_selection(&accepted, "persona").expect("500 Unicode scalars");

        let rejected = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("한".repeat(501)),
        };
        let error =
            validate_custom_selection(&rejected, "persona").expect_err("501 Unicode scalars");
        assert_eq!(
            error.message(),
            "custom persona custom_description must not exceed 500 Unicode scalar values"
        );
    }

    #[test]
    fn generic_user_directive_safely_includes_custom_descriptions() {
        let mut config = valid_config();
        config.profile = UserProfile {
            contexts: vec!["game-developer".to_owned(), "web-developer".to_owned()],
            description: Some("웹과 게임".to_owned()),
        };
        config.persona = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("friendly `but strict`".to_owned()),
        };

        let rendered =
            String::from_utf8(render_user_directive(&config, &["user-setup".to_owned()]))
                .expect("UTF-8 guidance");

        assert!(
            rendered.contains("- User contexts: `game-developer`, `web-developer` — `웹과 게임`")
        );
        assert!(rendered.contains("- Agent persona: `custom` — `` friendly `but strict` ``"));
    }

    #[test]
    fn user_directive_uses_the_selected_interface_language() {
        let mut config = valid_config();
        let english = String::from_utf8(render_user_directive(&config, &["user-setup".to_owned()]))
            .expect("English guidance");
        assert!(english.contains("# Aigent Hive user preferences"));
        assert!(english.contains(
            "Use English for every question and response unless the user explicitly requests another language for the current response"
        ));
        assert!(english.contains(
            "A message written in another language does not by itself change this preference"
        ));
        assert!(english.contains(
            "Unless the user explicitly requests another language for the current prompt, write the prompt in English"
        ));
        assert!(english.contains("For every passed, failed, skipped, deferred"));
        assert!(english.contains("a progress report is not closure"));
        assert!(!english.contains("# Aigent Hive 사용자 설정"));

        config.interface_language = InterfaceLanguage::Ko;
        let korean = String::from_utf8(render_user_directive(&config, &["user-setup".to_owned()]))
            .expect("Korean guidance");
        assert!(korean.contains("# Aigent Hive 사용자 설정"));
        assert!(korean.contains("명시적 요청이 없는 한 모든 질문과 응답에 한국어 사용"));
        assert!(korean.contains("다른 언어로 작성된 메시지만으로 이 선호를 변경하지 않음"));
        assert!(korean
            .contains("현재 프롬프트 언어를 사용자가 명시하지 않은 경우 프롬프트는 영어로 작성"));
        assert!(korean.contains("통과·실패·건너뜀·연기·미검증·미지원"));
        assert!(korean.contains("진행 보고를 closure로 사용 금지"));
        assert!(!korean.contains("# Aigent Hive user preferences"));
    }

    #[test]
    fn update_check_is_opt_in_and_projects_a_no_install_session_command() {
        let mut config = valid_config();
        assert!(!config.update_check.enabled);
        config.update_check.enabled = true;

        let rendered =
            String::from_utf8(render_user_directive(&config, &["user-setup".to_owned()]))
                .expect("English guidance");

        assert!(rendered.contains("- Daily update check: `enabled`"));
        assert!(rendered.contains("hive update --check --user-root <user-root> --output json"));
        assert!(rendered.contains("must never install"));
    }

    #[test]
    fn vector_onboarding_guidance_is_localized_and_one_time() {
        let mut config = valid_config();
        let english = String::from_utf8(render_user_directive(&config, &["user-setup".to_owned()]))
            .expect("English guidance");
        assert!(english.contains("hive setup feature claim --id vector-search"));
        assert!(english.contains("Never treat no answer or cancellation as no"));
        assert!(vector_setup_prompt(InterfaceLanguage::En)
            .contains("registered shared collections only"));

        config.interface_language = InterfaceLanguage::Ko;
        let korean = String::from_utf8(render_user_directive(&config, &["user-setup".to_owned()]))
            .expect("Korean guidance");
        assert!(korean.contains("무응답·취소를 아니요로 기록 금지"));
        assert!(vector_setup_prompt(InterfaceLanguage::Ko).contains("현재 등록된 공유 모음"));
    }

    #[test]
    fn user_directive_gates_task_fact_autocapture_on_wiki_enablement() {
        let mut config = valid_config();
        let enabled = String::from_utf8(render_user_directive(
            &config,
            &["knowledge-capture".to_owned()],
        ))
        .expect("enabled guidance");
        assert!(enabled.contains("Before every final response, review the current user statement"));
        assert!(enabled
            .contains("applies immediately after installation in every selected-host folder"));
        assert!(
            enabled.contains("An unregistered target falls back to user-root and shared knowledge")
        );
        assert!(enabled.contains("--user-statement <normalized-fact> --claim-key <stable-key>"));
        assert!(enabled.contains("canonical Markdown and derived-index receipt"));
        assert!(enabled.contains(
            "hive knowledge lint --target <user-root> --user-root <user-root> --output json"
        ));
        assert!(enabled.contains(
            "Missing project setup, a project marker, or an attached collection never skips lint"
        ));
        assert!(enabled.contains("raw transcript"));

        config.interface_language = InterfaceLanguage::Ko;
        let korean = String::from_utf8(render_user_directive(
            &config,
            &["knowledge-capture".to_owned()],
        ))
        .expect("Korean enabled guidance");
        assert!(korean.contains("모든 최종 응답 전 현재 사용자 발화와 완료 결과"));
        assert!(korean.contains("설치 직후 선택 호스트의 모든 폴더에 적용"));
        assert!(korean.contains(
            "미등록 target은 project-private 지식을 제외한 user-root·shared 지식으로 폴백"
        ));
        assert!(korean.contains("--user-statement <normalized-fact> --claim-key <stable-key>"));
        assert!(korean.contains("canonical Markdown과 derived-index receipt"));
        assert!(korean.contains(
            "hive knowledge lint --target <user-root> --user-root <user-root> --output json"
        ));
        assert!(korean.contains("lint 건너뜀 사유로 사용 금지"));

        config.interface_language = InterfaceLanguage::En;
        config.wiki.enabled = false;
        let disabled = String::from_utf8(render_user_directive(
            &config,
            &["knowledge-capture".to_owned()],
        ))
        .expect("disabled guidance");
        assert!(!disabled.contains("hive knowledge remember --user-root"));
        assert!(disabled.contains("Global Wiki is disabled: do not write or refresh knowledge"));
    }

    #[test]
    fn project_preferences_bridge_preserves_codexbar_fallback_consent() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let mut config = valid_config();
        config.usage_guard.enabled = true;
        config.usage_guard.codexbar_fallback_enabled = true;
        let relative = temporary.path().join(USER_SETUP_RELATIVE);
        fs::create_dir_all(relative.parent().expect("config parent")).expect("config directory");
        fs::write(&relative, canonical_config(&config).expect("config bytes"))
            .expect("operational config");

        let preferences = project_preferences(temporary.path()).expect("project preferences");

        assert!(preferences.usage_guard_enabled);
        assert!(preferences.codexbar_fallback_enabled);
    }

    #[test]
    fn global_usage_threshold_update_keeps_projection_binding_current() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let mut config = valid_config();
        config.usage_guard.enabled = true;
        config.usage_guard.stop_remaining_percent = 20;
        let bytes = canonical_config(&config).expect("config bytes");
        let config_path = temporary.path().join(USER_SETUP_RELATIVE);
        fs::create_dir_all(config_path.parent().expect("config parent")).expect("config directory");
        fs::write(&config_path, &bytes).expect("operational config");
        let root = super::super::user_install::open_user_root_for_setup(temporary.path())
            .expect("user root");
        let catalog = parse_and_validate_catalog().expect("catalog");
        let skills = resolve_skills(&config, &catalog).expect("resolved skills");
        apply_user_projection(&root, &config, &skills, &bytes).expect("initial projection");

        let update =
            set_operational_usage_threshold(temporary.path(), 5).expect("global threshold update");

        assert_eq!(update.previous, 20);
        assert_eq!(update.current, 5);
        assert!(update
            .changed_paths
            .iter()
            .any(|path| path == USER_SETUP_RELATIVE));
        let installed = load_operational_config(&root)
            .expect("load updated config")
            .expect("updated config");
        assert_eq!(installed.usage_guard.stop_remaining_percent, 5);
        validate_user_projection(&root, &installed, &skills, &update.config_bytes)
            .expect("updated projection binding");
    }

    #[test]
    fn completed_legacy_setup_review_is_removed_only_on_exact_bytes() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let review = temporary.path().join(LEGACY_USER_SETUP_REVIEW_RELATIVE);
        fs::create_dir_all(review.parent().expect("review parent")).expect("review directory");
        fs::write(&review, LEGACY_USER_SETUP_REVIEW).expect("canonical legacy review");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");

        assert!(remove_completed_legacy_setup_review(&root).expect("exact cleanup"));
        assert!(!review.exists());

        fs::write(&review, b"maintainer review notes\n").expect("foreign review bytes");
        assert!(!remove_completed_legacy_setup_review(&root).expect("preserve foreign"));
        assert_eq!(
            fs::read(&review).expect("preserved review"),
            b"maintainer review notes\n"
        );
    }

    #[test]
    fn post_commit_failures_restore_prior_config_and_projection() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = temporary.path().join(USER_SETUP_RELATIVE);
        let projection_path = PathBuf::from(".agents/directives/00-hive-user.md");
        let projection_file = temporary.path().join(&projection_path);
        fs::create_dir_all(config.parent().expect("config parent")).expect("config directory");
        fs::create_dir_all(projection_file.parent().expect("projection parent"))
            .expect("projection directory");
        let prior_config = b"prior config\n";
        let desired_config = b"desired config\n";
        let prior_projection = b"prior projection\n";
        let desired_projection = b"desired projection\n";
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");
        let projection = AppliedProjection {
            changes: vec![ProjectionChange {
                path: projection_path,
                before: Some(prior_projection.to_vec()),
                after: Some(desired_projection.to_vec()),
            }],
            changed_paths: Vec::new(),
            reports: Vec::new(),
        };
        let failures = [
            SetupError::Verification("injected operational config load failure".to_owned()),
            SetupError::Verification("applied user setup failed byte validation".to_owned()),
            SetupError::Conflict("injected legacy setup review removal failure".to_owned()),
        ];

        for primary in failures {
            fs::write(&config, desired_config).expect("committed config");
            fs::write(&projection_file, desired_projection).expect("activated projection");
            let expected_message = primary.message().to_owned();

            let error = finish_applied_setup(
                &root,
                temporary.path(),
                Some(prior_config),
                desired_config,
                true,
                &[],
                &projection,
                || Err(primary),
            )
            .expect_err("post-commit failure");

            assert_eq!(error.message(), expected_message);
            assert_eq!(fs::read(&config).expect("restored config"), prior_config);
            assert_eq!(
                fs::read(&projection_file).expect("restored projection"),
                prior_projection
            );
        }
    }

    #[test]
    fn post_commit_rollback_reports_all_independent_failures() {
        let temporary = tempfile::tempdir().expect("temporary user root");
        let config = temporary.path().join(USER_SETUP_RELATIVE);
        let projection_path = PathBuf::from(".agents/directives/00-hive-user.md");
        let projection_file = temporary.path().join(&projection_path);
        fs::create_dir_all(config.parent().expect("config parent")).expect("config directory");
        fs::create_dir_all(projection_file.parent().expect("projection parent"))
            .expect("projection directory");
        fs::write(&config, b"racing config\n").expect("racing config");
        fs::write(&projection_file, b"racing projection\n").expect("racing projection");
        let root =
            super::super::user_install::open_user_root_for_setup(temporary.path()).expect("root");
        let projection = AppliedProjection {
            changes: vec![ProjectionChange {
                path: projection_path,
                before: Some(b"prior projection\n".to_vec()),
                after: Some(b"desired projection\n".to_vec()),
            }],
            changed_paths: Vec::new(),
            reports: Vec::new(),
        };

        let error = finish_applied_setup(
            &root,
            temporary.path(),
            Some(b"prior config\n"),
            b"desired config\n",
            true,
            &[],
            &projection,
            || {
                Err(SetupError::Verification(
                    "injected validation failure".to_owned(),
                ))
            },
        )
        .expect_err("rollback must report every failure");

        assert_eq!(error.status(), "conflict");
        assert!(error.message().contains("user setup config:"));
        assert!(error.message().contains("user projection:"));
        assert_eq!(
            fs::read(&config).expect("preserved racing config"),
            b"racing config\n"
        );
        assert_eq!(
            fs::read(&projection_file).expect("preserved racing projection"),
            b"racing projection\n"
        );
    }
}
