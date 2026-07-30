use super::{emit_action_result, ActionResult, Evidence};
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use hive_projection::{compile_user_projection, embedded_catalog, Host as ProjectionHost};
use hive_render::GlobalProjectPreferences;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USER_SETUP_RELATIVE: &str = ".hive/config/user-setup.yml";
const USER_PROJECTION_MANIFEST_RELATIVE: &str = ".hive/install/user-projection.json";
const LEGACY_USER_SETUP_REVIEW_RELATIVE: &str = ".hive/config/user-setup-review.yml";
const LEGACY_USER_SETUP_REVIEW: &[u8] = b"schema_version: 1\nsource_version: 0.7.0\nsetup_required: true\nwiki_markdown_preserved: true\nlegacy_skill_projection: all-built-ins\n";
const USER_SETUP_SCHEMA: &str = include_str!("../../../schemas/user-setup.schema.json");
const USER_SETUP_CATALOG_SCHEMA: &str =
    include_str!("../../../schemas/user-setup-catalog.schema.json");
const USER_SETUP_CATALOG: &str = include_str!("../../../harness/user-setup/catalog.yml");
const MAX_ANSWERS_BYTES: u64 = 1024 * 1024;
const MAX_USER_SETUP_BYTES: u64 = 1024 * 1024;

const USER_SETUP_USAGE: &str = "\
Configure or validate Aigent Hive user preferences.

USAGE:
    hive setup --scope user --answers <yml> (--dry-run|--apply|--validate) [--user-root <dir>] --output json

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
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WikiLanguage {
    En,
    Ko,
    Both,
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CatalogSelection {
    pub(crate) id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) custom_description: Option<String>,
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
    Recommended,
    Individual,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillPreferences {
    pub(crate) mode: SkillSelectionMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) recommended_suite: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) selected: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UsageGuardPreferences {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default = "default_usage_threshold")]
    pub(crate) stop_remaining_percent: u8,
    #[serde(default)]
    pub(crate) codexbar_fallback_enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateCheckPreferences {
    #[serde(default)]
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UserSetupConfig {
    pub(crate) schema_version: u32,
    pub(crate) interface_language: InterfaceLanguage,
    pub(crate) wiki: WikiPreferences,
    pub(crate) profile: CatalogSelection,
    pub(crate) persona: CatalogSelection,
    pub(crate) selected_hosts: Vec<SelectedHost>,
    pub(crate) skills: SkillPreferences,
    #[serde(default)]
    pub(crate) update_check: UpdateCheckPreferences,
    pub(crate) usage_guard: UsageGuardPreferences,
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
struct SkillSuite {
    id: String,
    display_name: LocalizedText,
    description: LocalizedText,
    skills: Vec<String>,
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
    recommended_skill_suites: Vec<SkillSuite>,
    mandatory_skills: Vec<String>,
    skill_dependencies: Vec<SkillDependency>,
    optional_third_party_skills: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserProjectionManifest {
    schema_version: u32,
    product_version: String,
    setup_digest: String,
    entries: Vec<UserProjectionEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserProjectionEntry {
    path: String,
    digest: String,
}

struct AppliedProjection {
    changes: Vec<ProjectionChange>,
    changed_paths: Vec<String>,
}

struct ProjectionChange {
    path: PathBuf,
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
}

const fn default_true() -> bool {
    true
}

const fn default_usage_threshold() -> u8 {
    20
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
    let result = parse(arguments)
        .and_then(|arguments| execute(&arguments))
        .unwrap_or_else(|error| failure(&error));
    emit_action_result(&result)
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
            option @ ("--scope" | "--answers" | "--output" | "--user-root") => {
                let value = arguments
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| SetupError::Input(format!("missing value for {option}")))?;
                let slot = match option {
                    "--scope" => &mut scope,
                    "--answers" => &mut answers,
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
    let user_root = user_root.map_or_else(resolve_user_root, |value| Ok(PathBuf::from(value)))?;
    let root_cap =
        super::user_install::open_user_root_for_setup(&user_root).map_err(SetupError::Conflict)?;
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
    let config = parse_and_validate_config(&answer_bytes)?;
    let catalog = parse_and_validate_catalog()?;
    let resolved_skills = resolve_skills(&config, &catalog)?;
    let desired = canonical_config(&config)?;
    let relative = Path::new(USER_SETUP_RELATIVE);
    let existing = super::user_install::read_user_setup_file(
        &arguments.root_cap,
        relative,
        MAX_USER_SETUP_BYTES,
    )
    .map_err(SetupError::Conflict)?;
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
            ))
        }
        SetupMode::Validate => {
            let installed = existing.ok_or_else(|| {
                SetupError::Verification("user setup is required before validation".to_owned())
            })?;
            let installed_config = parse_and_validate_config(&installed).map_err(|error| {
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
            validate_user_projection(&arguments.root_cap, &config, &resolved_skills, &desired)?;
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
    let value: JsonValue = serde_yaml::from_slice(bytes)
        .map_err(|error| SetupError::Input(format!("invalid user setup YAML: {error}")))?;
    validate_schema(USER_SETUP_SCHEMA, &value, "user setup")?;
    let config: UserSetupConfig = serde_json::from_value(value)
        .map_err(|error| SetupError::Input(format!("invalid user setup values: {error}")))?;
    validate_config_semantics(&config)?;
    Ok(config)
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

fn validate_config_semantics(config: &UserSetupConfig) -> Result<(), SetupError> {
    if config.schema_version != 1 {
        return Err(SetupError::Input(
            "user setup schema_version must be 1".to_owned(),
        ));
    }
    validate_custom_selection(&config.profile, "profile")?;
    validate_custom_selection(&config.persona, "persona")?;
    validate_sorted_unique_hosts(&config.selected_hosts)?;
    if !(1..=99).contains(&config.usage_guard.stop_remaining_percent) {
        return Err(SetupError::Input(
            "usage guard stop_remaining_percent must be between 1 and 99".to_owned(),
        ));
    }
    if !config.usage_guard.enabled && config.usage_guard.codexbar_fallback_enabled {
        return Err(SetupError::Input(
            "codexbar_fallback_enabled must be false when the usage guard is disabled".to_owned(),
        ));
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

fn reject_host_deselection(
    installed: Option<&[u8]>,
    desired: &UserSetupConfig,
) -> Result<(), SetupError> {
    let Some(installed) = installed else {
        return Ok(());
    };
    let installed = parse_and_validate_config(installed).map_err(|error| {
        SetupError::Conflict(format!(
            "installed user setup is invalid: {}",
            error.message()
        ))
    })?;
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
        &["custom", "game-developer", "non-developer", "web-developer"],
        "profile",
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
    if catalog.mandatory_skills != ["setup-hive"] {
        return Err(SetupError::Internal(
            "embedded mandatory skill set must be exactly setup-hive".to_owned(),
        ));
    }
    if !catalog.optional_third_party_skills.is_empty() {
        return Err(SetupError::Internal(
            "optional third-party Skills are unsupported until a signed consent contract is available"
                .to_owned(),
        ));
    }
    let mut suites = BTreeSet::new();
    for suite in &catalog.recommended_skill_suites {
        validate_localized(&suite.display_name, "suite display_name")?;
        validate_localized(&suite.description, "suite description")?;
        if !suites.insert(suite.id.as_str()) || suite.skills.is_empty() {
            return Err(SetupError::Internal(
                "embedded recommended suite identifiers and skills must be unique".to_owned(),
            ));
        }
        validate_skill_names(&suite.skills, &built_ins, "recommended suite")?;
    }
    if suites != BTreeSet::from(["game-developer", "non-developer", "web-developer"]) {
        return Err(SetupError::Internal(
            "embedded recommended suite coverage is not exact".to_owned(),
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
    if !catalog
        .profiles
        .iter()
        .any(|entry| entry.id == config.profile.id)
    {
        return Err(SetupError::Input(format!(
            "unknown user profile: {}",
            config.profile.id
        )));
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
        SkillSelectionMode::Recommended => {
            if !config.skills.selected.is_empty() {
                return Err(SetupError::Input(
                    "recommended Skill mode must not include individual selections".to_owned(),
                ));
            }
            let suite = config.skills.recommended_suite.as_deref().ok_or_else(|| {
                SetupError::Input("recommended Skill mode requires recommended_suite".to_owned())
            })?;
            let entry = catalog
                .recommended_skill_suites
                .iter()
                .find(|entry| entry.id == suite)
                .ok_or_else(|| SetupError::Input(format!("unknown recommended suite: {suite}")))?;
            entry.skills.iter().cloned().collect()
        }
        SkillSelectionMode::Individual => {
            if config.skills.recommended_suite.is_some() || config.skills.selected.is_empty() {
                return Err(SetupError::Input(
                    "individual Skill mode requires selected and forbids recommended_suite"
                        .to_owned(),
                ));
            }
            config.skills.selected.iter().cloned().collect()
        }
    };
    selected.extend(catalog.mandatory_skills.iter().cloned());
    if !config.wiki.enabled {
        selected.retain(|name| !name.starts_with("hive-knowledge-"));
    }
    if config.usage_guard.enabled {
        selected.insert("hive-usage-guard".to_owned());
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
    let mut files = BTreeMap::<PathBuf, Vec<u8>>::new();
    let projection = compile_user_projection(ProjectionHost::Codex, resolved_skills, &[])
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

    let manifest_relative = Path::new(USER_PROJECTION_MANIFEST_RELATIVE);
    let prior_bytes =
        super::user_install::read_user_setup_file(root, manifest_relative, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
    let prior = prior_bytes
        .as_deref()
        .map(parse_projection_manifest)
        .transpose()?;
    authenticate_projection(root, prior.as_ref())?;

    let entries = files
        .iter()
        .map(|(path, bytes)| UserProjectionEntry {
            path: portable(path),
            digest: sha256_digest(bytes),
        })
        .collect::<Vec<_>>();
    let manifest = UserProjectionManifest {
        schema_version: 1,
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        setup_digest: sha256_digest(setup_bytes),
        entries,
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| {
        SetupError::Internal(format!(
            "cannot serialize user projection manifest: {error}"
        ))
    })?;
    manifest_bytes.push(b'\n');
    files.insert(manifest_relative.to_path_buf(), manifest_bytes);

    let prior_paths = prior
        .as_ref()
        .map(|manifest| {
            manifest
                .entries
                .iter()
                .map(|entry| PathBuf::from(&entry.path))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let prior_owned_paths = prior_paths.clone();
    let desired_paths = files.keys().cloned().collect::<BTreeSet<_>>();
    let mut operations = Vec::new();
    for path in prior_paths.difference(&desired_paths) {
        operations.push((path.clone(), None));
    }
    for (path, bytes) in files {
        operations.push((path, Some(bytes)));
    }
    operations.sort_by(|left, right| left.0.cmp(&right.0));

    let mut planned = AppliedProjection {
        changes: Vec::new(),
        changed_paths: Vec::new(),
    };
    for (path, after) in operations {
        let before = super::user_install::read_user_setup_file(root, &path, MAX_USER_SETUP_BYTES)
            .map_err(SetupError::Conflict)?;
        if before == after {
            continue;
        }
        if before.is_some() && path != manifest_relative && !prior_owned_paths.contains(&path) {
            return Err(SetupError::Conflict(format!(
                "user projection path exists without Hive ownership proof: {}",
                path.display()
            )));
        }
        planned.changed_paths.push(portable(&path));
        planned.changes.push(ProjectionChange {
            path,
            before,
            after,
        });
    }
    planned.changed_paths.sort();
    planned.changed_paths.dedup();
    Ok(planned)
}

fn apply_user_projection(
    root: &Dir,
    config: &UserSetupConfig,
    resolved_skills: &[String],
    setup_bytes: &[u8],
) -> Result<AppliedProjection, SetupError> {
    let planned = plan_user_projection(root, config, resolved_skills, setup_bytes)?;
    let mut applied = AppliedProjection {
        changes: Vec::new(),
        changed_paths: planned.changed_paths,
    };
    for change in planned.changes {
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
    Ok(applied)
}

fn validate_user_projection(
    root: &Dir,
    config: &UserSetupConfig,
    resolved_skills: &[String],
    setup_bytes: &[u8],
) -> Result<(), SetupError> {
    let planned = plan_user_projection(root, config, resolved_skills, setup_bytes)?;
    if planned.changed_paths.is_empty() {
        Ok(())
    } else {
        Err(SetupError::Verification(format!(
            "installed user projection differs at: {}",
            planned.changed_paths.join(", ")
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
    if manifest.schema_version != 1
        || manifest.product_version.is_empty()
        || !valid_sha256(&manifest.setup_digest)
    {
        return Err(SetupError::Conflict(
            "installed user projection manifest binding is invalid".to_owned(),
        ));
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

fn authenticate_projection(
    root: &Dir,
    prior: Option<&UserProjectionManifest>,
) -> Result<(), SetupError> {
    let Some(prior) = prior else {
        return Ok(());
    };
    for entry in &prior.entries {
        let bytes = super::user_install::read_user_setup_file(
            root,
            Path::new(&entry.path),
            MAX_USER_SETUP_BYTES,
        )
        .map_err(SetupError::Conflict)?
        .ok_or_else(|| {
            SetupError::Conflict(format!(
                "owned user projection path is missing: {}",
                entry.path
            ))
        })?;
        if sha256_digest(&bytes) != entry.digest {
            return Err(SetupError::Conflict(format!(
                "owned user projection path has local modifications: {}",
                entry.path
            )));
        }
    }
    Ok(())
}

fn render_user_directive(config: &UserSetupConfig, resolved_skills: &[String]) -> Vec<u8> {
    let hosts = config
        .selected_hosts
        .iter()
        .map(|host| host.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let wiki = if config.wiki.enabled {
        format!("enabled ({})", config.wiki.language.as_str())
    } else {
        "disabled".to_owned()
    };
    let profile = render_catalog_selection(&config.profile);
    let persona = render_catalog_selection(&config.persona);
    let update_check = if config.update_check.enabled {
        "enabled"
    } else {
        "disabled"
    };
    match config.interface_language {
        InterfaceLanguage::En => {
            let capture = if config.wiki.enabled {
                "- Before the final response for material work, run agent-reviewed task-fact \
autocapture into the enabled global Wiki. Record the bounded outcome, tool or project, criteria, \
and originating request summary from current authorized artifacts; never ingest a raw transcript, \
hook payload, tool output, hidden prompt, or runtime state.\n"
            } else {
                "- Wiki capture is disabled. Do not capture or refresh the knowledge index; \
preserve canonical Markdown until an explicit deletion request.\n"
            };
            format!(
                "# Aigent Hive user preferences\n\n- Setup state: `operational`\n- Interface language: `en`\n- User profile: {profile}\n- Agent persona: {persona}\n- Selected hosts: `{hosts}`\n- Global Wiki: `{wiki}`\n- Daily update check: `{update_check}`\n- Active Skills: `{}`\n{capture}- When daily update check is enabled, run `hive update --check --user-root <user-root> --output json` before the first Hive task of each host session. A check may notify but must never install.\n- Ask and answer in English.\n- For ambiguous or detail-poor ordinary prompts, offer one concise optional refine suggestion without automatic rewrite.\n- Never request provider credentials or call model-provider APIs on Hive's behalf.\n",
                resolved_skills.join(", ")
            )
        }
        InterfaceLanguage::Ko => {
            let capture = if config.wiki.enabled {
                "- 중요한 작업의 최종 응답 전 enabled global Wiki에 agent-reviewed task-fact를 \
기록. 현재 승인된 artifact에서 bounded outcome, tool·project, criteria, originating request \
summary만 사용하고 raw transcript, hook payload, tool output, hidden prompt, runtime state는 \
수집하지 않음.\n"
            } else {
                "- Wiki capture 비활성. 명시적 삭제 요청 전까지 canonical Markdown을 보존하고 \
knowledge index를 capture·refresh하지 않음.\n"
            };
            format!(
                "# Aigent Hive 사용자 설정\n\n- 설정 상태: `operational`\n- Interface language: `ko`\n- 사용자 profile: {profile}\n- Agent persona: {persona}\n- 선택 host: `{hosts}`\n- Global Wiki: `{wiki}`\n- 일일 update 확인: `{update_check}`\n- 활성 Skill: `{}`\n{capture}- 일일 update 확인이 enabled이면 각 host session의 첫 Hive 작업 전에 `hive update --check --user-root <user-root> --output json` 실행. 확인은 알림만 가능하며 설치 금지.\n- 질문과 응답은 한국어 사용.\n- 모호하거나 핵심 세부가 부족한 일반 prompt에는 자동 rewrite 없이 간결한 optional refine 제안 1개만 제공.\n- Provider credential을 요청하거나 Hive를 대신해 model-provider API를 호출하지 않음.\n",
                resolved_skills.join(", ")
            )
        }
    }
    .into_bytes()
}

fn render_catalog_selection(selection: &CatalogSelection) -> String {
    selection.custom_description.as_ref().map_or_else(
        || format!("`{}`", selection.id),
        |description| format!("`{}` — {}", selection.id, markdown_code_span(description)),
    )
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
    parse_and_validate_config(&bytes)
        .map(Some)
        .map_err(|error| {
            SetupError::Conflict(format!(
                "installed user setup is invalid: {}",
                error.message()
            ))
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

pub(crate) fn operational_wiki_enabled(user_root: &Path) -> Result<bool, String> {
    let root = super::user_install::open_user_root_for_setup(user_root)?;
    load_operational_config(&root)
        .map_err(|error| error.message().to_owned())?
        .map(|config| config.wiki.enabled)
        .ok_or_else(|| "global Hive setup is required for shared knowledge operations".to_owned())
}

pub(crate) fn project_preferences(user_root: &Path) -> Result<GlobalProjectPreferences, String> {
    let root = super::user_install::open_user_root_for_setup(user_root)?;
    let (config, mut selected_project_skills) = resolved_operational_skills(&root)
        .map_err(|error| error.message().to_owned())?
        .ok_or_else(|| {
            "global Hive setup is required before project expedited or custom setup".to_owned()
        })?;
    selected_project_skills.retain(|name| name != "setup-hive");
    selected_project_skills.sort();
    selected_project_skills.dedup();
    Ok(GlobalProjectPreferences {
        interface_language: config.interface_language.as_str().to_owned(),
        wiki_enabled: config.wiki.enabled,
        wiki_language: config.wiki.language.as_str().to_owned(),
        persona_id: config.persona.id,
        persona_custom_description: config.persona.custom_description,
        selected_project_skills,
        usage_guard_enabled: config.usage_guard.enabled,
        codexbar_fallback_enabled: config.usage_guard.codexbar_fallback_enabled,
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
            "usage_guard": config.usage_guard,
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
    - setup-hive
usage_guard:
  enabled: false
  stop_remaining_percent: 20
  codexbar_fallback_enabled: false
",
        )
        .expect("valid config")
    }

    #[test]
    fn user_setup_help_describes_all_modes() {
        assert!(USER_SETUP_USAGE.contains("--scope user"));
        assert!(USER_SETUP_USAGE.contains("--dry-run|--apply|--validate"));
        assert!(USER_SETUP_USAGE.contains("--output json"));
    }

    #[test]
    fn embedded_user_setup_catalog_is_valid() {
        let catalog = parse_and_validate_catalog().expect("catalog");
        assert_eq!(catalog.schema_version, 1);
        assert_eq!(catalog.mandatory_skills, ["setup-hive"]);
        assert!(catalog.optional_third_party_skills.is_empty());
    }

    #[test]
    fn setup_defaults_enable_wiki_and_keep_guard_native_first() {
        let config = parse_and_validate_config(
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
    - setup-hive
usage_guard: {}
",
        )
        .expect("defaults");

        assert!(config.wiki.enabled);
        assert!(!config.usage_guard.enabled);
        assert_eq!(config.usage_guard.stop_remaining_percent, 20);
        assert!(!config.usage_guard.codexbar_fallback_enabled);
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
    - hive-knowledge-capture
usage_guard:
  enabled: true
",
        )
        .expect("config");
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            ["hive-usage-guard", "setup-hive"]
        );
    }

    #[test]
    fn enabled_wiki_resolves_knowledge_skill_dependency_closure() {
        let mut config = valid_config();
        config.skills.selected = vec!["hive-knowledge-capture".to_owned()];
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            [
                "hive-knowledge-capture",
                "hive-knowledge-query",
                "setup-hive",
            ]
        );
    }

    #[test]
    fn disabled_usage_guard_preserves_an_explicitly_selected_control_skill() {
        let mut config = valid_config();
        config.skills.selected = vec!["hive-usage-guard".to_owned()];
        let catalog = parse_and_validate_catalog().expect("catalog");

        assert_eq!(
            resolve_skills(&config, &catalog).expect("closure"),
            ["hive-usage-guard", "setup-hive"]
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
    fn custom_description_limit_counts_unicode_scalars() {
        let accepted = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("한".repeat(500)),
        };
        validate_custom_selection(&accepted, "profile").expect("500 Unicode scalars");

        let rejected = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("한".repeat(501)),
        };
        let error =
            validate_custom_selection(&rejected, "profile").expect_err("501 Unicode scalars");
        assert_eq!(
            error.message(),
            "custom profile custom_description must not exceed 500 Unicode scalar values"
        );
    }

    #[test]
    fn generic_user_directive_safely_includes_custom_descriptions() {
        let mut config = valid_config();
        config.profile = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("웹과 게임".to_owned()),
        };
        config.persona = CatalogSelection {
            id: "custom".to_owned(),
            custom_description: Some("friendly `but strict`".to_owned()),
        };

        let rendered =
            String::from_utf8(render_user_directive(&config, &["setup-hive".to_owned()]))
                .expect("UTF-8 guidance");

        assert!(rendered.contains("- User profile: `custom` — `웹과 게임`"));
        assert!(rendered.contains("- Agent persona: `custom` — `` friendly `but strict` ``"));
    }

    #[test]
    fn user_directive_uses_the_selected_interface_language() {
        let mut config = valid_config();
        let english = String::from_utf8(render_user_directive(&config, &["setup-hive".to_owned()]))
            .expect("English guidance");
        assert!(english.contains("# Aigent Hive user preferences"));
        assert!(english.contains("- Ask and answer in English."));
        assert!(!english.contains("# Aigent Hive 사용자 설정"));

        config.interface_language = InterfaceLanguage::Ko;
        let korean = String::from_utf8(render_user_directive(&config, &["setup-hive".to_owned()]))
            .expect("Korean guidance");
        assert!(korean.contains("# Aigent Hive 사용자 설정"));
        assert!(korean.contains("- 질문과 응답은 한국어 사용."));
        assert!(!korean.contains("# Aigent Hive user preferences"));
    }

    #[test]
    fn update_check_is_opt_in_and_projects_a_no_install_session_command() {
        let mut config = valid_config();
        assert!(!config.update_check.enabled);
        config.update_check.enabled = true;

        let rendered =
            String::from_utf8(render_user_directive(&config, &["setup-hive".to_owned()]))
                .expect("English guidance");

        assert!(rendered.contains("- Daily update check: `enabled`"));
        assert!(rendered.contains("hive update --check --user-root <user-root> --output json"));
        assert!(rendered.contains("must never install"));
    }

    #[test]
    fn user_directive_gates_task_fact_autocapture_on_wiki_enablement() {
        let mut config = valid_config();
        let enabled = String::from_utf8(render_user_directive(
            &config,
            &["hive-knowledge-capture".to_owned()],
        ))
        .expect("enabled guidance");
        assert!(enabled.contains("agent-reviewed task-fact autocapture"));
        assert!(enabled.contains("originating request"));
        assert!(enabled.contains("raw transcript"));

        config.wiki.enabled = false;
        let disabled = String::from_utf8(render_user_directive(
            &config,
            &["hive-knowledge-capture".to_owned()],
        ))
        .expect("disabled guidance");
        assert!(!disabled.contains("agent-reviewed task-fact autocapture"));
        assert!(disabled.contains("Wiki capture is disabled"));
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
