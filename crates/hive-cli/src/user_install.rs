use super::{emit_action_result, ActionResult, Evidence};
use crate::usage::{CommandRunner, QualifiedExecutable, SystemCommandRunner};
use cap_fs_ext::{DirExt, OpenOptionsFollowExt};
use cap_primitives::fs::FollowSymlinks;
#[cfg(unix)]
use cap_primitives::fs::PermissionsExt as CapPermissionsExt;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use hive_core::sha256_digest;
use hive_projection::{
    compile_user_projection_localized, DescriptorLanguage, Host as ProjectionHost,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::{OsStr, OsString};
#[cfg(test)]
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const USER_MARKER_START: &[u8] = b"<!-- AIGENT-HIVE:USER:START -->";
const USER_MARKER_END: &[u8] = b"<!-- AIGENT-HIVE:USER:END -->";
const MAX_USER_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DERIVED_INDEX_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ANTIGRAVITY_STAGE_FILES: usize = 4_096;
const MAX_ANTIGRAVITY_STAGE_DIRECTORIES: usize = 4_096;
const MAX_ANTIGRAVITY_STAGE_DEPTH: usize = 64;
const MAX_ANTIGRAVITY_STAGE_BYTES: u64 = 64 * 1024 * 1024;
const COMMAND_TIMEOUT: Duration = Duration::from_mins(2);
const COMMAND_OUTPUT_LIMIT: usize = 1024 * 1024;
const ROOT_INDEX_RELATIVE: &str = ".hive/index/hive.sqlite3";
const ANTIGRAVITY_SOURCE_RELATIVE: &str = ".hive/marketplaces/antigravity/plugins/aigent-hive";
const ANTIGRAVITY_STAGE_RELATIVE: &str = ".gemini/config/plugins/aigent-hive";
const LEGACY_ANTIGRAVITY_070_SOURCE_DIGEST: &str =
    "sha256:0f39dbdccd2a50e49ea3a123a9ca2cd99823485b8c8e5eada8becf8495b7238c";
const LEGACY_ANTIGRAVITY_070_MANIFEST_DIGEST: &str =
    "sha256:2eeb1a2cb0d4f2c616443e1b5844b1e10551457f78f1cb96ff76afb223495e86";
const USER_070_CODEX_PLUGIN_DIGEST: &str =
    "sha256:4ecb63663a94ffd939aacb9e5179660c46cb72ee9ee379d7bea04279eae35e9e";
const USER_070_CLAUDE_PLUGIN_DIGEST: &str =
    "sha256:5590976481b90ab7c11c4109f2abca3282f641d9e86aa4155ce40ad140c38242";
const USER_070_CLAUDE_CAPTURE_DIGEST: &str =
    "sha256:d347a2eff52a7908b16d048e50f69abc0696699ad7b02b0d4a1bedb425a5c93b";
const USER_070_CODEX_MARKETPLACE_DIGEST: &str =
    "sha256:5fc994474640b82960f5c015c6fdcb50a01df010441b065c991a114e96f84620";
const USER_070_CLAUDE_MARKETPLACE_DIGEST: &str =
    "sha256:52681102bb7c8c9c0bb2ce332602e8b4cb418ebfbe0fe1fd0d0cc34ee3aa5af9";
const PRE_SCOPE_ROUTING_SETUP_HIVE_DIGEST: &str =
    "sha256:891b6af921dfdbe390df51a6ee69874bc16ca29adc5b483b18b7cb4d5bb66a57";
const PRE_SCOPE_ROUTING_SETUP_HARNESS_DIGEST: &str =
    "sha256:8e0d1e2bc964eefbfedc24b462d657552250fb976c18f534c6601328c0b7451c";
const TEST3_SETUP_HIVE_DIGEST: &str =
    "sha256:5a729db9b80c9e3c008ecc5b24d927219015ba44dd72a60564afba4476c07bb4";
const USER_070_CODEX_ONBOARDING_SOURCE_DIGEST: &str =
    "sha256:b9e0364aedca5b56a7a8a189570c054ceeb00c5ec4f58361901ce513a85fa371";
const USER_070_CODEX_ONBOARDING_GUIDANCE_DIGEST: &str =
    "sha256:b36972eeb3905d23421fb6d604b48369300c93be9a63cdeb19bf347b2628f430";
const USER_070_CODEX_ONBOARDING_RAW_README_DIGEST: &str =
    "sha256:eb57ad2af4196916dd0bb9e899723fe822b618b08e920f74b8f3f1110cd2eb82";
const USER_070_CODEX_ONBOARDING_SCHEMA_DIGEST: &str =
    "sha256:411b45789a64cef406b4fe0de87e85f8fb03649bcce62a0f647774c3ffebf396";
const USER_070_CODEX_ONBOARDING_WIKI_INDEX_DIGEST: &str =
    "sha256:4982a7080053b989bf26ef87076c818b89c7b9b65591ae30af1e51296b9489b6";
const USER_070_CODEX_ONBOARDING_WIKI_LOG_DIGEST: &str =
    "sha256:73f63a7979e80ef74aa5181f214e22ca173b56300b1b5af5b2cd2eb716a27031";
const USER_070_CODEX_ONBOARDING_SUPPRESSION_DIGEST: &str =
    "sha256:4511250f4407bf8e3d4c66deaea468602e8faf5734b4f9b5a57d93bf54ab68cc";
const USER_070_CODEX_ONBOARDING_AUTO_SETUP_DIGEST: &str =
    "sha256:3336c86d6cf338e84f2440c4434d06b7d517714a07a0ac9c0c5163f695148b7e";
const USER_070_CODEX_ONBOARDING_KNOWLEDGE_CAPTURE_DIGEST: &str =
    "sha256:dd05e3e1065cf1fbf0ca04f0ef6f9acfcfdcdf91fd8362071665ac28542a4f1b";
const USER_070_CODEX_ONBOARDING_KNOWLEDGE_MAINTENANCE_DIGEST: &str =
    "sha256:de7484f35d262c2c44d52a60f2c6d698f1ccb5d278f10930f412a4b1ad8be90e";
const USER_070_CODEX_ONBOARDING_KNOWLEDGE_QUERY_DIGEST: &str =
    "sha256:3e0354c7b90bf5cc06ce4bce55d604d007ebd4f4fe03581d48e0cdefbf206b60";
const USER_070_CODEX_ONBOARDING_SETUP_HARNESS_DIGEST: &str =
    "sha256:2cd7d8f9318a8acd96a709dd98e9cb8c0d9ff15fb02383097959d45463645a61";
const USER_070_CODEX_ONBOARDING_SETUP_HIVE_DIGEST: &str =
    "sha256:4ee3a621dd2149a3fde6aa1df8c36cef3ab249404ac4d00934197596340e39e6";
const LEGACY_ANTIGRAVITY_070_SKILLS: &[(&str, &str)] = &[
    (
        "hive-judge-package",
        "sha256:a87f46c6a21f94944fc465d85b74bf5e8aef308bfbc1b7a08df56e6078f3890c",
    ),
    (
        "hive-knowledge-capture",
        "sha256:bff118dd34504091478c636470c4ae113e6eef0ef488eac406c1f2dd79970d91",
    ),
    (
        "hive-knowledge-maintenance",
        "sha256:eb196fd8c66722928ba4f691543e199e09947241face5a51eb6b329c391fdd3f",
    ),
    (
        "hive-knowledge-promote",
        "sha256:3239382ad2461e31b99a89d32ee69b6bb6fd26faf1a91e50f52aaa9ed21b5159",
    ),
    (
        "hive-knowledge-query",
        "sha256:57544a98a992cab44cf81b5a2badf1efd6bba68f6a1785e678b44bf099a4353e",
    ),
    (
        "hive-migrate",
        "sha256:a48df71fcc1d5eb901487f86e807f6acdebf62e0250a564d99fa734f9512ea32",
    ),
    (
        "hive-project-upgrade",
        "sha256:c673f3fca88085c25d044944bd17e692eafa0105d0eaf31ac53f14737a15ace4",
    ),
    (
        "hive-prompt-refine",
        "sha256:167fe625b59f020f200f167ac380875b77e79aac111dfa03163f94718834eb42",
    ),
    (
        "hive-role-handoff",
        "sha256:d59b0866ed27c85bd16f8a8cb9fb8a278df0d4cd5f9ca9262b676f4b513b2eaa",
    ),
    (
        "hive-run-checkpoint",
        "sha256:e44b6b4456bc02996c44cdab64c25fd4335f360509685f592248e2c342401383",
    ),
    (
        "hive-run-resume",
        "sha256:fbec4961baef77e9e33a73f2f6caa47d46f093133fe142005067dde8143389b7",
    ),
    (
        "hive-simple-question",
        "sha256:716be2df49d27e0cbaf28bbd428b01b0a716e1baf6ebca7fab2fd26e91b3fa9a",
    ),
    (
        "hive-update",
        "sha256:e390663900b8e362ca64d066c35ac7c63ce8a6c88398c35210825e8056d212df",
    ),
    (
        "hive-usage-guard",
        "sha256:7e18146d1bb6becce19c1bed6d86d6ece488cffc7793d3bfb4a591d5a3ce7a3c",
    ),
    (
        "setup-harness",
        "sha256:11316ba100022cbf18713712eaf6d50325ea489dc410a6a4c1a9b8fa2e1a7f0e",
    ),
];
const CODEX_PLUGIN_MANIFEST: &[u8] =
    include_bytes!("../../../harness/plugins/aigent-hive/.codex-plugin/plugin.json");
const CLAUDE_PLUGIN_MANIFEST: &[u8] =
    include_bytes!("../../../harness/plugins/aigent-hive/.claude-plugin/plugin.json");
const ANTIGRAVITY_PLUGIN_MANIFEST: &[u8] =
    include_bytes!("../../../harness/plugins/aigent-hive/plugin.json");
const CLAUDE_USAGE_CAPTURE: &[u8] =
    include_bytes!("../../../harness/plugins/aigent-hive/bin/hive-claude-usage-capture");
const USER_080_CODEX_PLUGIN_MANIFEST: &[u8] = include_bytes!(
    "../../../harness/user-bases/0.8.0/plugins/aigent-hive/.codex-plugin/plugin.json"
);
const USER_080_CLAUDE_PLUGIN_MANIFEST: &[u8] = include_bytes!(
    "../../../harness/user-bases/0.8.0/plugins/aigent-hive/.claude-plugin/plugin.json"
);
const USER_080_ANTIGRAVITY_PLUGIN_MANIFEST: &[u8] =
    include_bytes!("../../../harness/user-bases/0.8.0/plugins/aigent-hive/plugin.json");
const USER_080_CLAUDE_USAGE_CAPTURE: &[u8] = include_bytes!(
    "../../../harness/user-bases/0.8.0/plugins/aigent-hive/bin/hive-claude-usage-capture"
);
const USER_080_CODEX_MARKETPLACE: &[u8] =
    include_bytes!("../../../harness/user-bases/0.8.0/adapters/codex-marketplace.json");
const USER_080_CLAUDE_MARKETPLACE: &[u8] =
    include_bytes!("../../../harness/user-bases/0.8.0/adapters/claude-marketplace.json");

macro_rules! frozen_user_skill_0_8 {
    ($name:literal) => {
        (
            $name,
            include_bytes!(concat!(
                "../../../harness/user-bases/0.8.0/plugins/aigent-hive/skills/",
                $name,
                "/SKILL.md"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../../../harness/user-bases/0.8.0/plugins/aigent-hive/skills/",
                $name,
                "/agents/openai.yaml"
            ))
            .as_slice(),
        )
    };
}

const USER_080_SKILLS: [(&str, &[u8], &[u8]); 17] = [
    frozen_user_skill_0_8!("auto-setup-harness"),
    frozen_user_skill_0_8!("hive-judge-package"),
    frozen_user_skill_0_8!("hive-knowledge-capture"),
    frozen_user_skill_0_8!("hive-knowledge-maintenance"),
    frozen_user_skill_0_8!("hive-knowledge-promote"),
    frozen_user_skill_0_8!("hive-knowledge-query"),
    frozen_user_skill_0_8!("hive-migrate"),
    frozen_user_skill_0_8!("hive-project-upgrade"),
    frozen_user_skill_0_8!("hive-prompt-refine"),
    frozen_user_skill_0_8!("hive-role-handoff"),
    frozen_user_skill_0_8!("hive-run-checkpoint"),
    frozen_user_skill_0_8!("hive-run-resume"),
    frozen_user_skill_0_8!("hive-simple-question"),
    frozen_user_skill_0_8!("hive-update"),
    frozen_user_skill_0_8!("hive-usage-guard"),
    frozen_user_skill_0_8!("setup-harness"),
    frozen_user_skill_0_8!("setup-hive"),
];
const ROOT_RAW_README: &[u8] =
    include_bytes!("../../../harness/template/.hive/knowledge/Raw/README.md");
const ROOT_SCHEMA: &[u8] =
    include_bytes!("../../../harness/template/.hive/knowledge/Schema/schema.md");
const ROOT_WIKI_INDEX: &[u8] =
    include_bytes!("../../../harness/template/.hive/knowledge/Wiki/index.md");
const ROOT_WIKI_LOG: &[u8] =
    include_bytes!("../../../harness/template/.hive/knowledge/Wiki/log.md");
const ROOT_SUPPRESSION: &[u8] =
    include_bytes!("../../../harness/template/.hive/knowledge/suppression.yml");
const ROOT_DISCORD_USAGE_GUIDE: &[u8] =
    include_bytes!("../../../harness/template/.hive/guides/discord-usage-notifications.html");
const USER_070_SETUP_REVIEW: &[u8] = b"schema_version: 1\nsource_version: 0.7.0\nsetup_required: true\nwiki_markdown_preserved: true\nlegacy_skill_projection: all-built-ins\n";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum UserHost {
    Codex,
    Claude,
    Antigravity,
}

impl UserHost {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Antigravity => "antigravity",
        }
    }

    const fn projection_host(self) -> ProjectionHost {
        match self {
            Self::Codex => ProjectionHost::Codex,
            Self::Claude => ProjectionHost::Claude,
            Self::Antigravity => ProjectionHost::Antigravity,
        }
    }

    const fn version_range(self) -> &'static str {
        match self {
            Self::Codex => ">=0.145.0 <1.0.0",
            Self::Claude => ">=2.1.0 <3.0.0",
            Self::Antigravity => ">=1.1.7 <1.2.0",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UserMode {
    DryRun,
    Apply,
    Validate,
    Recover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UserOperation {
    Install,
    Update,
}

struct UserArguments {
    host: UserHost,
    mode: UserMode,
    user_root: PathBuf,
    root_cap: Dir,
    setup_override: Option<(crate::user_setup::UserSetupConfig, Vec<String>)>,
}

#[derive(Debug)]
enum InstallError {
    Input(String),
    Conflict(String),
    Unsupported(String),
    Verification(String),
    Internal(String),
}

impl InstallError {
    const fn status(&self) -> &'static str {
        match self {
            Self::Input(_) | Self::Internal(_) => "error",
            Self::Conflict(_) => "conflict",
            Self::Unsupported(_) => "unsupported",
            Self::Verification(_) => "verification-failed",
        }
    }

    const fn exit_code(&self) -> u8 {
        match self {
            Self::Input(_) => 2,
            Self::Conflict(_) => 3,
            Self::Unsupported(_) => 4,
            Self::Verification(_) => 5,
            Self::Internal(_) => 10,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Input(message)
            | Self::Conflict(message)
            | Self::Unsupported(message)
            | Self::Verification(message)
            | Self::Internal(message) => message,
        }
    }
}

#[derive(Clone)]
struct PlannedFile {
    bytes: Vec<u8>,
    executable: bool,
    ownership: &'static str,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RegularTree {
    directories: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, Vec<u8>>,
}

struct UserPlan {
    files: BTreeMap<PathBuf, PlannedFile>,
    retired_files: BTreeMap<PathBuf, RetiredFile>,
    changed_paths: Vec<String>,
    plan_digest: String,
    manifest_relative: PathBuf,
    marketplace_root: Option<PathBuf>,
    expected_before: BTreeMap<PathBuf, Option<Vec<u8>>>,
    expected_permissions: BTreeMap<PathBuf, Option<FilePermissions>>,
    qualified_host_version: Option<String>,
    prior_antigravity_activation_source: bool,
    expected_antigravity_stage: Option<RegularTree>,
}

#[derive(Clone)]
struct RetiredFile {
    bytes: Vec<u8>,
    #[cfg(unix)]
    permissions: FilePermissions,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserOwnershipManifest {
    schema_version: u32,
    product_version: String,
    host: UserHost,
    host_version_range: String,
    source_release_digest: String,
    plan_digest: String,
    last_backup: Option<String>,
    guidance_path: String,
    entries: Vec<UserOwnershipEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserOwnershipEntry {
    path: String,
    digest: String,
    executable: bool,
    #[serde(default)]
    unix_mode: Option<u32>,
    ownership: String,
}

#[derive(Clone)]
struct AuthenticatedUserInventory {
    product_version: String,
    host: UserHost,
    host_version_range: String,
    source_release_digest: String,
    guidance_path: String,
    entries: Vec<UserOwnershipEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserBackupManifest {
    schema_version: u32,
    host: UserHost,
    plan_digest: String,
    #[serde(default)]
    host_mutations: Vec<HostMutation>,
    #[serde(default)]
    index_existed: bool,
    #[serde(default)]
    codex_state_before: Option<CodexHostState>,
    #[serde(default)]
    claude_state_before: Option<ClaudeHostState>,
    #[serde(default)]
    antigravity_state_before: Option<AntigravityHostState>,
    #[serde(default)]
    host_owned_state: Option<HostStateSnapshot>,
    #[serde(default)]
    pending_host_transition: Option<PendingHostTransition>,
    #[serde(default)]
    codex_plugin_was_latent_before_marketplace_add: bool,
    entries: Vec<UserBackupEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserBackupEntry {
    path: String,
    existed: bool,
    digest: Option<String>,
    #[serde(default)]
    installed_digest: Option<String>,
    #[serde(default)]
    installed_executable: Option<bool>,
    #[serde(default)]
    installed_unix_mode: Option<u32>,
    #[serde(default)]
    executable: bool,
    #[serde(default)]
    unix_mode: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FilePermissions {
    executable: bool,
    unix_mode: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum HostMutation {
    CodexMarketplaceAdded,
    CodexPluginAdded,
    CodexPluginRefreshed,
    ClaudeMarketplaceAdded,
    ClaudePluginInstalled,
    ClaudeMarketplaceRefreshed,
    ClaudePluginRefreshed,
    AntigravityPluginInstalled,
    AntigravityPluginRefreshed,
}

type ActivationCommand<'a> = (Vec<&'a str>, Option<HostMutation>);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "host", content = "state", rename_all = "lowercase")]
enum HostStateSnapshot {
    Codex(CodexHostState),
    Claude(ClaudeHostState),
    Antigravity(AntigravityHostState),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum HostTransitionPhase {
    Forward,
    Compensation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PendingHostTransition {
    mutation: HostMutation,
    phase: HostTransitionPhase,
    before: HostStateSnapshot,
    after: HostStateSnapshot,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct CodexHostState {
    marketplace: Option<CodexMarketplaceState>,
    plugin: Option<CodexPluginState>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct CodexMarketplaceState {
    root: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct CodexPluginState {
    version: String,
    enabled: bool,
    source_path: String,
    marketplace_source: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClaudeHostState {
    marketplace: Option<ClaudeMarketplaceState>,
    plugin: Option<ClaudePluginState>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClaudeMarketplaceState {
    source: String,
    path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClaudePluginState {
    version: String,
    enabled: bool,
    scope: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct AntigravityHostState {
    plugin: Option<AntigravityPluginState>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct AntigravityPluginState {
    source: String,
    components: Vec<String>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostCompensationSurface {
    CodexStructuredJson,
    ClaudeStructuredJson,
    AntigravityStructuredOutput,
}

#[cfg(test)]
fn compensation_surface(mutation: HostMutation) -> HostCompensationSurface {
    match mutation {
        HostMutation::CodexMarketplaceAdded
        | HostMutation::CodexPluginAdded
        | HostMutation::CodexPluginRefreshed => HostCompensationSurface::CodexStructuredJson,
        HostMutation::ClaudeMarketplaceAdded
        | HostMutation::ClaudePluginInstalled
        | HostMutation::ClaudeMarketplaceRefreshed
        | HostMutation::ClaudePluginRefreshed => HostCompensationSurface::ClaudeStructuredJson,
        HostMutation::AntigravityPluginInstalled | HostMutation::AntigravityPluginRefreshed => {
            HostCompensationSurface::AntigravityStructuredOutput
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UserTransactionJournal {
    schema_version: u32,
    host: UserHost,
    plan_digest: String,
    backup: String,
}

struct UserTransaction {
    backup_relative: PathBuf,
    journal_relative: PathBuf,
    backup: UserBackupManifest,
}

type PlannedSnapshot = (PathBuf, Option<Vec<u8>>, FilePermissions);

pub(crate) fn run_install(arguments: &[String]) -> ExitCode {
    run(UserOperation::Install, arguments)
}

pub(crate) fn run_update(arguments: &[String]) -> ExitCode {
    run(UserOperation::Update, arguments)
}

const USER_UNINSTALL_USAGE: &str = "\
Remove the user-scope Aigent Hive installation while preserving saved preferences and knowledge.

USAGE:
    hive uninstall [--user-root <absolute-dir>] [--output json]

RESULT:
    Remove Hive-managed host activation, projections, packages, indexes, backups, and runtime state. Always preserve `.hive/knowledge/` and saved user preferences.
";

struct UserUninstallArguments {
    root_cap: Dir,
}

pub(crate) fn run_uninstall(arguments: &[String]) -> ExitCode {
    if arguments.len() == 1 && arguments[0] == "--help" {
        print!("{USER_UNINSTALL_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = parse_uninstall(arguments)
        .and_then(|arguments| execute_uninstall(&arguments, &SystemCommandRunner))
        .unwrap_or_else(|error| failure("UninstallHiveUser", &error));
    emit_action_result(&result)
}

fn parse_uninstall(arguments: &[String]) -> Result<UserUninstallArguments, InstallError> {
    let mut user_root = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--output" => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| InstallError::Input("missing value for --output".to_owned()))?;
                if value != "json" {
                    return Err(InstallError::Input(
                        "uninstall --output must be json".to_owned(),
                    ));
                }
                index += 2;
            }
            "--user-root" => {
                let value = arguments.get(index + 1).ok_or_else(|| {
                    InstallError::Input("missing value for --user-root".to_owned())
                })?;
                if user_root.replace(value.clone()).is_some() {
                    return Err(InstallError::Input(
                        "duplicate option: --user-root".to_owned(),
                    ));
                }
                index += 2;
            }
            option => return Err(InstallError::Input(format!("unknown option: {option}"))),
        }
    }
    let requested_user_root =
        user_root.map_or_else(resolve_user_root, |value| Ok(PathBuf::from(value)))?;
    let (_, root_cap) = open_canonical_user_root(&requested_user_root)?;
    Ok(UserUninstallArguments { root_cap })
}

fn execute_uninstall(
    arguments: &UserUninstallArguments,
    runner: &impl CommandRunner,
) -> Result<ActionResult, InstallError> {
    let mut changed_paths = Vec::new();
    let mut removed_hosts = Vec::new();
    for host in [UserHost::Codex, UserHost::Claude, UserHost::Antigravity] {
        let manifest_relative = PathBuf::from(format!(".hive/install/{}.json", host.as_str()));
        let manifest = read_installed_manifest(&arguments.root_cap, &manifest_relative, host)?;
        let host_evidence = manifest.is_some()
            || owned_path_exists(&arguments.root_cap, host_uninstall_root(host))?;
        if host_evidence {
            remove_host_activation(host, runner)?;
            removed_hosts.push(host.as_str());
        }
        if let Some(manifest) = manifest {
            for entry in manifest.entries {
                let path = PathBuf::from(entry.path);
                if path.starts_with(".hive/knowledge") {
                    continue;
                }
                if entry.ownership == "shared-marker" {
                    remove_hive_guidance_marker(&arguments.root_cap, &path, &mut changed_paths)?;
                } else {
                    remove_owned_regular(&arguments.root_cap, &path, &mut changed_paths)?;
                }
            }
        }
        remove_owned_regular(&arguments.root_cap, &manifest_relative, &mut changed_paths)?;
    }
    for path in crate::user_setup::uninstall_projection_paths(&arguments.root_cap)
        .map_err(|error| InstallError::Conflict(error.message().to_owned()))?
    {
        remove_owned_regular(&arguments.root_cap, &path, &mut changed_paths)?;
    }
    for relative in [
        ".hive/install",
        ".hive/install-transactions",
        ".hive/backups",
        ".hive/marketplaces",
        ".hive/plugins",
        ".hive/index",
        ".hive/runtime",
        ".hive/claims",
        ".hive/guides",
    ] {
        remove_owned_tree(&arguments.root_cap, Path::new(relative), &mut changed_paths)?;
    }
    for relative in [
        ROOT_INDEX_RELATIVE,
        ".hive/config/user-active-skills.yml",
        ".hive/config/user-setup-progress.yml",
        ".hive/config/user-setup-review.yml",
        ".hive/config/projects.yml",
    ] {
        remove_owned_regular(&arguments.root_cap, Path::new(relative), &mut changed_paths)?;
    }
    for relative in [
        ".hive/config",
        ".hive",
        ".agents/skills",
        ".agents/directives",
        ".agents",
    ] {
        remove_owned_empty_dir(&arguments.root_cap, Path::new(relative))?;
    }
    changed_paths.sort();
    changed_paths.dedup();
    Ok(ActionResult {
        schema_version: 1,
        action: "UninstallHiveUser",
        status: "success",
        exit_code: 0,
        code: "hive.user-uninstall-complete",
        message: "user-scope Hive installation removed; saved preferences and knowledge preserved"
            .to_owned(),
        changed_paths,
        evidence: Vec::new(),
        next_action: Some(
            "run hive install --scope user --host <saved-host> --apply --output json; saved preferences are reused without setup questions".to_owned(),
        ),
        data: Some(json!({
            "removed_hosts": removed_hosts,
            "preserved": [".hive/knowledge", ".hive/config/user-setup.yml", ".hive/config/user-preferences.json"],
        })),
    })
}

fn host_uninstall_root(host: UserHost) -> &'static Path {
    match host {
        UserHost::Codex => Path::new(".hive/marketplaces/codex"),
        UserHost::Claude => Path::new(".hive/marketplaces/claude"),
        UserHost::Antigravity => Path::new(ANTIGRAVITY_SOURCE_RELATIVE),
    }
}

fn owned_path_exists(root: &Dir, relative: &Path) -> Result<bool, InstallError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(false);
    };
    match parent.symlink_metadata(&name) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(InstallError::Conflict(format!(
            "Hive-owned uninstall path is a symlink: {}",
            relative.display()
        ))),
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_internal("inspect uninstall path", relative, error)),
    }
}

fn remove_host_activation(host: UserHost, runner: &impl CommandRunner) -> Result<(), InstallError> {
    let program = match host {
        UserHost::Codex => "codex",
        UserHost::Claude => "claude",
        UserHost::Antigravity => "agy",
    };
    let executable = runner.qualify(program).map_err(|_| {
        InstallError::Unsupported(format!(
            "{program} executable is unavailable; Hive host activation was not removed"
        ))
    })?;
    probe_supported_host_version(host, &executable, runner)?;
    let commands: &[&[&str]] = match host {
        UserHost::Codex => &[
            &["plugin", "remove", "aigent-hive@aigent-hive", "--json"],
            &["plugin", "marketplace", "remove", "aigent-hive", "--json"],
        ],
        UserHost::Claude => &[
            &[
                "plugin",
                "uninstall",
                "aigent-hive@aigent-hive",
                "--scope",
                "user",
            ],
            &[
                "plugin",
                "marketplace",
                "remove",
                "aigent-hive",
                "--scope",
                "user",
            ],
        ],
        UserHost::Antigravity => &[&["plugin", "uninstall", "aigent-hive"]],
    };
    for command in commands {
        let _ = runner.run(&executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT);
    }
    let state = probe_host_snapshot(host, &executable, runner)?;
    let absent = match state {
        HostStateSnapshot::Codex(state) => state.marketplace.is_none() && state.plugin.is_none(),
        HostStateSnapshot::Claude(state) => state.marketplace.is_none() && state.plugin.is_none(),
        HostStateSnapshot::Antigravity(state) => state.plugin.is_none(),
    };
    if absent {
        Ok(())
    } else {
        Err(InstallError::Verification(format!(
            "{} still reports a Hive-owned activation after uninstall",
            host.as_str()
        )))
    }
}

fn remove_hive_guidance_marker(
    root: &Dir,
    relative: &Path,
    changed_paths: &mut Vec<String>,
) -> Result<(), InstallError> {
    let Some(existing) = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)? else {
        return Ok(());
    };
    let starts = find_all(&existing, USER_MARKER_START);
    let ends = find_all(&existing, USER_MARKER_END);
    let replacement = match (starts.as_slice(), ends.as_slice()) {
        ([], []) => return Ok(()),
        ([start], [end]) if start < end => {
            let mut end = end + USER_MARKER_END.len();
            if existing.get(end) == Some(&b'\n') {
                end += 1;
            }
            let mut bytes = Vec::with_capacity(existing.len());
            bytes.extend_from_slice(&existing[..*start]);
            bytes.extend_from_slice(&existing[end..]);
            bytes
        }
        _ => {
            return Err(InstallError::Conflict(format!(
                "Hive guidance marker is malformed: {}",
                relative.display()
            )))
        }
    };
    let permissions = file_permissions(root, relative)?;
    if replacement.is_empty() {
        remove_regular_if_exists(root, relative, Some(&existing), Some(permissions))?;
    } else {
        write_atomic(
            root,
            relative,
            &replacement,
            false,
            Some(&existing),
            Some(permissions),
        )?;
    }
    changed_paths.push(portable(relative));
    Ok(())
}

fn remove_owned_regular(
    root: &Dir,
    relative: &Path,
    changed_paths: &mut Vec<String>,
) -> Result<(), InstallError> {
    let Some(existing) = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)? else {
        return Ok(());
    };
    let permissions = file_permissions(root, relative)?;
    remove_regular_if_exists(root, relative, Some(&existing), Some(permissions))?;
    changed_paths.push(portable(relative));
    Ok(())
}

fn remove_owned_tree(
    root: &Dir,
    relative: &Path,
    changed_paths: &mut Vec<String>,
) -> Result<(), InstallError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(());
    };
    let metadata = match parent.symlink_metadata(&name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_internal("inspect uninstall tree", relative, error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(InstallError::Conflict(format!(
            "Hive-owned uninstall path is not a no-follow directory: {}",
            relative.display()
        )));
    }
    let directory = parent.open_dir_nofollow(&name).map_err(|error| {
        InstallError::Conflict(format!(
            "cannot pin Hive-owned uninstall directory {}: {error}",
            relative.display()
        ))
    })?;
    remove_owned_dir_contents(&directory, relative, changed_paths)?;
    drop(directory);
    parent
        .remove_dir(&name)
        .map_err(|error| io_internal("remove uninstall directory", relative, error))?;
    Ok(())
}

fn remove_owned_dir_contents(
    directory: &Dir,
    prefix: &Path,
    changed_paths: &mut Vec<String>,
) -> Result<(), InstallError> {
    let entries = directory.entries().map_err(|error| {
        InstallError::Internal(format!(
            "cannot enumerate Hive-owned uninstall directory: {error}"
        ))
    })?;
    for entry in entries {
        let name = entry
            .map_err(|error| {
                InstallError::Internal(format!("cannot read uninstall entry: {error}"))
            })?
            .file_name();
        let relative = prefix.join(&name);
        let metadata = directory
            .symlink_metadata(&name)
            .map_err(|error| io_internal("inspect uninstall entry", &relative, error))?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let child = directory.open_dir_nofollow(&name).map_err(|error| {
                InstallError::Conflict(format!(
                    "cannot pin Hive-owned uninstall directory {}: {error}",
                    relative.display()
                ))
            })?;
            remove_owned_dir_contents(&child, &relative, changed_paths)?;
            drop(child);
            directory
                .remove_dir(&name)
                .map_err(|error| io_internal("remove uninstall directory", &relative, error))?;
        } else if metadata.is_file() && !metadata.file_type().is_symlink() {
            directory
                .remove_file(&name)
                .map_err(|error| io_internal("remove uninstall file", &relative, error))?;
            changed_paths.push(portable(&relative));
        } else {
            return Err(InstallError::Conflict(format!(
                "Hive-owned uninstall directory contains a non-regular entry: {}",
                relative.display()
            )));
        }
    }
    Ok(())
}

fn remove_owned_empty_dir(root: &Dir, relative: &Path) -> Result<(), InstallError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(());
    };
    let directory = match parent.open_dir_nofollow(&name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_internal("open uninstall directory", relative, error)),
    };
    if directory
        .entries()
        .map_err(|error| io_internal("enumerate uninstall directory", relative, error))?
        .next()
        .is_none()
    {
        drop(directory);
        parent
            .remove_dir(&name)
            .map_err(|error| io_internal("remove empty uninstall directory", relative, error))?;
    }
    Ok(())
}

pub(crate) fn apply_configured_host(
    user_root: &Path,
    selected_host: crate::user_setup::SelectedHost,
    config: &crate::user_setup::UserSetupConfig,
    resolved_skills: &[String],
) -> Result<ActionResult, String> {
    configured_host(
        user_root,
        selected_host,
        config,
        resolved_skills,
        UserMode::Apply,
    )
}

pub(crate) fn preview_configured_host(
    user_root: &Path,
    selected_host: crate::user_setup::SelectedHost,
    config: &crate::user_setup::UserSetupConfig,
    resolved_skills: &[String],
) -> Result<ActionResult, String> {
    configured_host(
        user_root,
        selected_host,
        config,
        resolved_skills,
        UserMode::DryRun,
    )
}

pub(crate) fn validate_configured_host(
    user_root: &Path,
    selected_host: crate::user_setup::SelectedHost,
    config: &crate::user_setup::UserSetupConfig,
    resolved_skills: &[String],
) -> Result<ActionResult, String> {
    configured_host(
        user_root,
        selected_host,
        config,
        resolved_skills,
        UserMode::Validate,
    )
}

fn configured_host(
    user_root: &Path,
    selected_host: crate::user_setup::SelectedHost,
    config: &crate::user_setup::UserSetupConfig,
    resolved_skills: &[String],
    mode: UserMode,
) -> Result<ActionResult, String> {
    let host = match selected_host {
        crate::user_setup::SelectedHost::Codex => UserHost::Codex,
        crate::user_setup::SelectedHost::Claude => UserHost::Claude,
        crate::user_setup::SelectedHost::Antigravity => UserHost::Antigravity,
    };
    let (user_root, root_cap) =
        open_canonical_user_root(user_root).map_err(|error| error.message().to_owned())?;
    let arguments = UserArguments {
        host,
        mode,
        user_root,
        root_cap,
        setup_override: Some((config.clone(), resolved_skills.to_vec())),
    };
    if mode == UserMode::DryRun
        && has_recoverable_dangling_codex_marketplace(&arguments)
            .map_err(|error| error.message().to_owned())?
    {
        let plan = build_plan(&arguments).map_err(|error| error.message().to_owned())?;
        let mut result = success_result(
            UserOperation::Install,
            &arguments,
            &plan,
            "hive.user-install-dry-run-recovery-planned",
            "user installation dry run completed with Hive-owned recovery planned",
            None,
        );
        result.next_action = Some(format!(
            "run with --host {} --apply; Hive will recover its incomplete marketplace activation before setup",
            arguments.host.as_str()
        ));
        return Ok(result);
    }
    if mode == UserMode::Apply
        && has_open_transaction(&arguments).map_err(|error| error.message().to_owned())?
    {
        let (recovery_root, recovery_cap) = open_canonical_user_root(&arguments.user_root)
            .map_err(|error| error.message().to_owned())?;
        let recovery = UserArguments {
            host: arguments.host,
            mode: UserMode::Recover,
            user_root: recovery_root,
            root_cap: recovery_cap,
            setup_override: None,
        };
        execute(UserOperation::Install, &recovery, &SystemCommandRunner)
            .map_err(|error| error.message().to_owned())?;
    }
    execute(UserOperation::Install, &arguments, &SystemCommandRunner)
        .map_err(|error| error.message().to_owned())
}

pub(crate) fn recover_configured_host(
    user_root: &Path,
    selected_host: crate::user_setup::SelectedHost,
) -> Result<ActionResult, String> {
    let host = match selected_host {
        crate::user_setup::SelectedHost::Codex => UserHost::Codex,
        crate::user_setup::SelectedHost::Claude => UserHost::Claude,
        crate::user_setup::SelectedHost::Antigravity => UserHost::Antigravity,
    };
    let (user_root, root_cap) =
        open_canonical_user_root(user_root).map_err(|error| error.message().to_owned())?;
    let arguments = UserArguments {
        host,
        mode: UserMode::Recover,
        user_root,
        root_cap,
        setup_override: None,
    };
    execute(UserOperation::Install, &arguments, &SystemCommandRunner)
        .map_err(|error| error.message().to_owned())
}

fn run(operation: UserOperation, arguments: &[String]) -> ExitCode {
    let action = match operation {
        UserOperation::Install => "InstallHiveUser",
        UserOperation::Update => "UpdateHiveUser",
    };
    let result = parse(arguments)
        .and_then(|arguments| execute(operation, &arguments, &SystemCommandRunner))
        .unwrap_or_else(|error| failure(action, &error));
    emit_action_result(&result)
}

fn parse(arguments: &[String]) -> Result<UserArguments, InstallError> {
    let mut scope = None;
    let mut host = None;
    let mut mode = None;
    let mut output = None;
    let mut user_root = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dry-run" if mode.is_none() => {
                mode = Some(UserMode::DryRun);
                index += 1;
            }
            "--apply" if mode.is_none() => {
                mode = Some(UserMode::Apply);
                index += 1;
            }
            "--validate" if mode.is_none() => {
                mode = Some(UserMode::Validate);
                index += 1;
            }
            "--recover" if mode.is_none() => {
                mode = Some(UserMode::Recover);
                index += 1;
            }
            "--dry-run" | "--apply" | "--validate" | "--recover" => {
                return Err(InstallError::Input(
                    "exactly one install/update mode is required".to_owned(),
                ));
            }
            option @ ("--scope" | "--host" | "--output" | "--user-root") => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| InstallError::Input(format!("missing value for {option}")))?;
                let slot = match option {
                    "--scope" => &mut scope,
                    "--host" => &mut host,
                    "--output" => &mut output,
                    "--user-root" => &mut user_root,
                    _ => unreachable!(),
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(InstallError::Input(format!("duplicate option: {option}")));
                }
                index += 2;
            }
            option => return Err(InstallError::Input(format!("unknown option: {option}"))),
        }
    }
    if scope.as_deref() != Some("user") {
        return Err(InstallError::Input(
            "user installation requires --scope user".to_owned(),
        ));
    }
    if output.as_deref() != Some("json") {
        return Err(InstallError::Input(
            "user installation requires --output json".to_owned(),
        ));
    }
    let host = match host.as_deref() {
        Some("codex") => UserHost::Codex,
        Some("claude") => UserHost::Claude,
        Some("antigravity") => UserHost::Antigravity,
        Some(_) => {
            return Err(InstallError::Input(
                "--host must be codex, claude, or antigravity".to_owned(),
            ));
        }
        None => {
            return Err(InstallError::Input(
                "missing required option --host".to_owned(),
            ))
        }
    };
    let mode = mode.ok_or_else(|| InstallError::Input("missing install/update mode".to_owned()))?;
    let requested_user_root =
        user_root.map_or_else(resolve_user_root, |value| Ok(PathBuf::from(value)))?;
    let (user_root, root_cap) = open_canonical_user_root(&requested_user_root)?;
    Ok(UserArguments {
        host,
        mode,
        user_root,
        root_cap,
        setup_override: None,
    })
}

fn resolve_user_root() -> Result<PathBuf, InstallError> {
    let value = if cfg!(windows) {
        env::var_os("USERPROFILE")
    } else {
        env::var_os("HOME")
    }
    .ok_or_else(|| InstallError::Input("cannot resolve the user home directory".to_owned()))?;
    Ok(PathBuf::from(value))
}

pub(crate) fn resolve_user_root_path() -> Result<PathBuf, String> {
    resolve_user_root().map_err(|error| error.message().to_owned())
}

fn open_user_root(root: &Path) -> Result<Dir, InstallError> {
    if !root.is_absolute() {
        return Err(InstallError::Input(
            "--user-root must be an absolute directory".to_owned(),
        ));
    }
    let parent = root
        .parent()
        .ok_or_else(|| InstallError::Input("user root has no parent directory".to_owned()))?;
    let name = root
        .file_name()
        .ok_or_else(|| InstallError::Input("user root has no directory name".to_owned()))?;
    let parent_cap = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|error| {
        InstallError::Input(format!(
            "cannot open user root parent {}: {error}",
            parent.display()
        ))
    })?;
    parent_cap.open_dir_nofollow(name).map_err(|error| {
        InstallError::Conflict(format!(
            "user root must be a no-follow directory {}: {error}",
            root.display()
        ))
    })
}

fn open_canonical_user_root(root: &Path) -> Result<(PathBuf, Dir), InstallError> {
    let root_cap = open_user_root(root)?;
    let canonical = root.canonicalize().map_err(|error| {
        InstallError::Conflict(format!(
            "cannot resolve user root {} after no-follow validation: {error}",
            root.display()
        ))
    })?;
    Ok((canonical, root_cap))
}

pub(crate) fn open_user_root_for_setup(root: &Path) -> Result<Dir, String> {
    open_user_root(root).map_err(|error| error.message().to_owned())
}

pub(crate) fn open_canonical_user_root_for_setup(root: &Path) -> Result<(PathBuf, Dir), String> {
    open_canonical_user_root(root).map_err(|error| error.message().to_owned())
}

pub(crate) fn read_user_setup_file(
    root: &Dir,
    relative: &Path,
    maximum: u64,
) -> Result<Option<Vec<u8>>, String> {
    read_optional_regular(root, relative, maximum).map_err(|error| error.message().to_owned())
}

pub(crate) fn replace_user_setup_file(
    root: &Dir,
    relative: &Path,
    expected: Option<&[u8]>,
    desired: Option<&[u8]>,
) -> Result<(), String> {
    let expected_permissions = expected
        .map(|_| file_permissions(root, relative))
        .transpose()
        .map_err(|error| error.message().to_owned())?;
    cas_activate(
        root,
        relative,
        expected.map(|bytes| ExpectedFile {
            bytes,
            permissions: expected_permissions
                .expect("existing setup file requires a permission token"),
        }),
        desired,
        FilePermissions {
            executable: false,
            unix_mode: None,
        },
    )
    .map_err(|error| error.message().to_owned())
}

fn execute(
    operation: UserOperation,
    arguments: &UserArguments,
    runner: &impl CommandRunner,
) -> Result<ActionResult, InstallError> {
    if arguments.mode == UserMode::Recover {
        return recover(arguments, runner);
    }
    let mut plan = build_plan(arguments)?;
    match arguments.mode {
        UserMode::DryRun => {
            let executable = qualify_host(arguments, &plan, runner)?;
            let executable = executable.as_ref().ok_or_else(|| {
                InstallError::Internal("qualified host executable is missing".to_owned())
            })?;
            let host_version = probe_supported_host_version(arguments.host, executable, runner)?;
            bind_host_version(&mut plan, &host_version);
            let codex = probe_codex_state_if_required(arguments, Some(executable), runner)?;
            let claude = probe_claude_state_if_required(arguments, Some(executable), runner)?;
            let antigravity =
                probe_antigravity_state_if_required(arguments, Some(executable), runner)?;
            validate_codex_prestate(arguments, codex.as_ref())?;
            validate_claude_prestate(arguments, claude.as_ref())?;
            validate_antigravity_prestate(arguments, &plan, antigravity.as_ref())?;
            Ok(success_result(
                operation,
                arguments,
                &plan,
                "hive.user-install-dry-run-complete",
                "user installation dry run completed",
                None,
            ))
        }
        UserMode::Validate => {
            if !plan.changed_paths.is_empty() {
                return Err(InstallError::Verification(format!(
                    "user installation drift detected at {} path(s)",
                    plan.changed_paths.len()
                )));
            }
            let executable = qualify_host(arguments, &plan, runner)?;
            let executable = executable.as_ref().ok_or_else(|| {
                InstallError::Internal("qualified host executable is missing".to_owned())
            })?;
            let host_version = probe_supported_host_version(arguments.host, executable, runner)?;
            bind_host_version(&mut plan, &host_version);
            validate_installed_host(arguments, &plan, executable, runner)?;
            Ok(success_result(
                operation,
                arguments,
                &plan,
                "hive.user-install-valid",
                "user installation is valid",
                None,
            ))
        }
        UserMode::Apply => execute_apply(operation, arguments, runner, &mut plan),
        UserMode::Recover => unreachable!("recovery returns before plan construction"),
    }
}

fn execute_apply(
    operation: UserOperation,
    arguments: &UserArguments,
    runner: &impl CommandRunner,
    plan: &mut UserPlan,
) -> Result<ActionResult, InstallError> {
    let host_executable = qualify_host(arguments, plan, runner)?;
    let host_version = host_executable
        .as_ref()
        .map(|executable| probe_supported_host_version(arguments.host, executable, runner))
        .transpose()?;
    if let Some(host_version) = host_version.as_deref() {
        bind_host_version(plan, host_version);
    }
    let codex_before = probe_codex_state_if_required(arguments, host_executable.as_ref(), runner)?;
    let claude_before =
        probe_claude_state_if_required(arguments, host_executable.as_ref(), runner)?;
    let antigravity_before =
        probe_antigravity_state_if_required(arguments, host_executable.as_ref(), runner)?;
    validate_codex_prestate(arguments, codex_before.as_ref())?;
    validate_claude_prestate(arguments, claude_before.as_ref())?;
    validate_antigravity_prestate(arguments, plan, antigravity_before.as_ref())?;
    let applied_changed_paths = plan.changed_paths.clone();
    let mut transaction = apply_plan(
        arguments,
        plan,
        codex_before,
        claude_before,
        antigravity_before,
    )?;
    let activated = activate_host(
        arguments,
        plan,
        &mut transaction,
        host_executable.as_ref(),
        runner,
    )
    .and_then(|()| validate_codex_activation(arguments, host_executable.as_ref(), runner))
    .and_then(|()| validate_claude_activation(arguments, host_executable.as_ref(), runner))
    .and_then(|()| {
        validate_antigravity_activation(arguments, plan, host_executable.as_ref(), runner)
    })
    .and_then(|()| validate_plugin_package(arguments, plan))
    .and_then(|()| rebuild_root_index(arguments))
    .and_then(|()| {
        crate::user_setup::restore_saved_projection_after_uninstall(&arguments.root_cap).map_err(
            |error| {
                InstallError::Verification(format!(
                    "saved user preferences could not be restored after installation: {}",
                    error.message()
                ))
            },
        )
    })
    .and_then(|_| validate_applied_bytes(arguments));
    let mut refreshed = activated.map_err(|primary| {
        rollback_after_failure(
            arguments,
            &mut transaction,
            host_executable.as_ref(),
            runner,
            primary,
        )
    })?;
    if let Some(host_version) = host_version.as_deref() {
        bind_host_version(&mut refreshed, host_version);
    }
    remove_transaction_journal(arguments, &transaction.journal_relative).map_err(|primary| {
        rollback_after_failure(
            arguments,
            &mut transaction,
            host_executable.as_ref(),
            runner,
            primary,
        )
    })?;
    refreshed.changed_paths = applied_changed_paths;
    Ok(success_result(
        operation,
        arguments,
        &refreshed,
        match operation {
            UserOperation::Install => "hive.user-install-complete",
            UserOperation::Update => "hive.user-update-complete",
        },
        match operation {
            UserOperation::Install => "user-scope Hive installation completed",
            UserOperation::Update => "user-scope Hive update completed",
        },
        Some(&portable(&transaction.backup_relative)),
    ))
}

fn rebuild_root_index(arguments: &UserArguments) -> Result<(), InstallError> {
    let installed = crate::user_setup::load_operational_config(&arguments.root_cap)
        .map_err(|error| InstallError::Conflict(error.message().to_owned()))?;
    let setup = arguments
        .setup_override
        .as_ref()
        .map(|(config, _)| config)
        .or(installed.as_ref());
    if setup.as_ref().is_none_or(|config| !config.wiki.enabled) {
        return remove_disposable_root_index(&arguments.root_cap);
    }
    if setup.is_some_and(|config| config.wiki.backend == crate::user_setup::WikiBackend::Notion) {
        return Ok(());
    }
    hive_wiki::shared::ensure_project_registry(&arguments.user_root)
        .and_then(|_| hive_wiki::shared::rebuild_shared_index(&arguments.user_root))
        .map(|_| ())
        .map_err(|error| {
            InstallError::Verification(format!(
                "shared knowledge index rebuild failed after installation: {error}"
            ))
        })
}

fn validate_applied_bytes(arguments: &UserArguments) -> Result<UserPlan, InstallError> {
    let refreshed = build_plan(arguments)?;
    if refreshed.changed_paths.is_empty() {
        Ok(refreshed)
    } else {
        Err(InstallError::Verification(
            "user installation failed post-activation byte validation".to_owned(),
        ))
    }
}

struct DesiredUserFiles {
    files: BTreeMap<PathBuf, PlannedFile>,
    guidance_relative: PathBuf,
    marketplace_root: Option<PathBuf>,
}

#[allow(clippy::too_many_lines)]
fn build_desired_user_files(
    arguments: &UserArguments,
    operational: Option<&(crate::user_setup::UserSetupConfig, Vec<String>)>,
) -> Result<DesiredUserFiles, InstallError> {
    let mut files = BTreeMap::new();
    let guidance_relative = guidance_path(arguments.host, &arguments.root_cap)?;
    let guidance_existing =
        read_optional_regular(&arguments.root_cap, &guidance_relative, MAX_USER_FILE_BYTES)?
            .unwrap_or_default();
    let guidance = render_user_guidance(arguments.host, operational.map(|(config, _)| config));
    validate_operational_guidance(&guidance, operational.map(|(config, _)| config))?;
    files.insert(
        guidance_relative.clone(),
        PlannedFile {
            bytes: merge_user_marker(&guidance_existing, &guidance)?,
            executable: false,
            ownership: "shared-marker",
        },
    );
    if operational.is_some_and(|(config, _)| {
        config.wiki.enabled && config.wiki.backend == crate::user_setup::WikiBackend::Markdown
    }) {
        seed_root_knowledge(&arguments.root_cap, &mut files)?;
    }

    let selected_skills = operational.map_or_else(
        || vec!["product-update".to_owned(), "user-setup".to_owned()],
        |(_, skills)| skills.clone(),
    );
    let language = operational.map_or(DescriptorLanguage::En, |(config, _)| {
        match config.interface_language {
            crate::user_setup::InterfaceLanguage::En => DescriptorLanguage::En,
            crate::user_setup::InterfaceLanguage::Ko => DescriptorLanguage::Ko,
        }
    });
    let projection = compile_user_projection_localized(
        arguments.host.projection_host(),
        &selected_skills,
        &[],
        language,
    )
    .map_err(|error| InstallError::Internal(error.to_string()))?;
    let marketplace_root = match arguments.host {
        UserHost::Codex | UserHost::Claude => {
            let relative = PathBuf::from(format!(".hive/marketplaces/{}", arguments.host.as_str()));
            let plugin_relative = relative.join("plugins/aigent-hive");
            for (path, bytes) in projection.files {
                let Some(skill_path) = strip_project_skill_prefix(&path) else {
                    continue;
                };
                files.insert(
                    plugin_relative.join(skill_path),
                    PlannedFile {
                        bytes,
                        executable: false,
                        ownership: "immutable-plugin-package",
                    },
                );
            }
            files.insert(
                plugin_relative.join(".codex-plugin/plugin.json"),
                PlannedFile {
                    bytes: CODEX_PLUGIN_MANIFEST.to_vec(),
                    executable: false,
                    ownership: "immutable-plugin-package",
                },
            );
            files.insert(
                plugin_relative.join(".claude-plugin/plugin.json"),
                PlannedFile {
                    bytes: CLAUDE_PLUGIN_MANIFEST.to_vec(),
                    executable: false,
                    ownership: "immutable-plugin-package",
                },
            );
            files.insert(
                plugin_relative.join("bin/hive-claude-usage-capture"),
                PlannedFile {
                    bytes: CLAUDE_USAGE_CAPTURE.to_vec(),
                    executable: true,
                    ownership: "immutable-plugin-package",
                },
            );
            let marketplace = match arguments.host {
                UserHost::Codex => render_codex_marketplace(),
                UserHost::Claude => render_claude_marketplace(),
                UserHost::Antigravity => unreachable!(),
            };
            let marketplace_relative = match arguments.host {
                UserHost::Codex => relative.join(".agents/plugins/marketplace.json"),
                UserHost::Claude => relative.join(".claude-plugin/marketplace.json"),
                UserHost::Antigravity => unreachable!(),
            };
            files.insert(
                marketplace_relative,
                PlannedFile {
                    bytes: marketplace,
                    executable: false,
                    ownership: "host-adapter-metadata",
                },
            );
            Some(relative)
        }
        UserHost::Antigravity => {
            let plugin_relative = PathBuf::from(ANTIGRAVITY_SOURCE_RELATIVE);
            for (path, bytes) in projection.files {
                let Some(skill_path) = strip_project_skill_prefix(&path) else {
                    continue;
                };
                files.insert(
                    plugin_relative.join(&skill_path),
                    PlannedFile {
                        bytes: bytes.clone(),
                        executable: false,
                        ownership: "immutable-plugin-package",
                    },
                );
                files.insert(
                    Path::new(".gemini/config").join(skill_path),
                    PlannedFile {
                        bytes,
                        executable: false,
                        ownership: "host-skill-projection",
                    },
                );
            }
            files.insert(
                plugin_relative.join("plugin.json"),
                PlannedFile {
                    bytes: ANTIGRAVITY_PLUGIN_MANIFEST.to_vec(),
                    executable: false,
                    ownership: "immutable-plugin-package",
                },
            );
            Some(plugin_relative)
        }
    };
    Ok(DesiredUserFiles {
        files,
        guidance_relative,
        marketplace_root,
    })
}

fn authenticated_current_inventory(
    arguments: &UserArguments,
    desired: &DesiredUserFiles,
) -> AuthenticatedUserInventory {
    let entries = ownership_entries(&desired.files);
    AuthenticatedUserInventory {
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        host: arguments.host,
        host_version_range: arguments.host.version_range().to_owned(),
        source_release_digest: source_release_digest_from_entries(&entries),
        guidance_path: portable(&desired.guidance_relative),
        entries,
    }
}

#[allow(clippy::too_many_lines)]
fn build_plan(arguments: &UserArguments) -> Result<UserPlan, InstallError> {
    let installed_operational = crate::user_setup::resolved_operational_skills(&arguments.root_cap)
        .map_err(|error| InstallError::Conflict(error.message().to_owned()))?;
    let operational = arguments
        .setup_override
        .as_ref()
        .or(installed_operational.as_ref());
    if let Some((config, _)) = operational.as_ref() {
        let selected = config
            .selected_hosts
            .iter()
            .any(|host| host.as_str() == arguments.host.as_str());
        if !selected {
            return Err(InstallError::Conflict(format!(
                "{} is not selected in the operational user setup",
                arguments.host.as_str()
            )));
        }
    }
    let manifest_relative =
        PathBuf::from(format!(".hive/install/{}.json", arguments.host.as_str()));
    let prior_manifest =
        read_installed_manifest(&arguments.root_cap, &manifest_relative, arguments.host)?;
    let DesiredUserFiles {
        mut files,
        guidance_relative,
        marketplace_root,
    } = build_desired_user_files(arguments, operational)?;
    let owns_setup_review = prior_manifest.as_ref().is_some_and(|manifest| {
        manifest.entries.iter().any(|entry| {
            entry.path == ".hive/config/user-setup-review.yml"
                && matches!(
                    entry.ownership.as_str(),
                    "migration-state" | "shared-migration-state"
                )
        })
    });
    let requires_setup_review = prior_manifest.as_ref().is_some_and(|manifest| {
        manifest.product_version == "0.7.0"
            && !manifest
                .entries
                .iter()
                .any(|entry| entry.path.contains("/skills/setup-hive/"))
    });
    if operational.is_none() && (owns_setup_review || requires_setup_review) {
        insert_user_setup_review(&mut files);
    }
    let authenticated_prior = if operational == installed_operational.as_ref() {
        None
    } else {
        let mut desired_prior =
            build_desired_user_files(arguments, installed_operational.as_ref())?;
        if owns_setup_review {
            insert_user_setup_review(&mut desired_prior.files);
        }
        Some(authenticated_current_inventory(arguments, &desired_prior))
    };
    let source_release_digest = source_release_digest(&files);
    let entries = ownership_entries(&files);
    let plan_digest = inventory_digest(
        arguments.host,
        env!("CARGO_PKG_VERSION"),
        arguments.host.version_range(),
        &guidance_relative,
        &source_release_digest,
        &entries,
    );
    let retired_files = validate_prior_ownership(
        &arguments.root_cap,
        &files,
        &guidance_relative,
        &source_release_digest,
        &entries,
        prior_manifest.as_ref(),
        authenticated_prior.as_ref(),
    )?;
    let prior_antigravity_activation_source = arguments.host == UserHost::Antigravity
        && prior_manifest.as_ref().is_some_and(|manifest| {
            manifest
                .entries
                .iter()
                .any(|entry| entry.path == format!("{ANTIGRAVITY_SOURCE_RELATIVE}/plugin.json"))
        });
    let expected_antigravity_stage =
        authenticated_antigravity_stage_tree(&arguments.root_cap, prior_manifest.as_ref())?;
    let last_backup = prior_manifest
        .as_ref()
        .and_then(|manifest| manifest.last_backup.clone());
    let manifest = UserOwnershipManifest {
        schema_version: 1,
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        host: arguments.host,
        host_version_range: arguments.host.version_range().to_owned(),
        source_release_digest,
        plan_digest: plan_digest.clone(),
        last_backup,
        guidance_path: portable(&guidance_relative),
        entries,
    };
    let manifest_bytes = json_line(&manifest)?;
    files.insert(
        manifest_relative.clone(),
        PlannedFile {
            bytes: manifest_bytes,
            executable: false,
            ownership: "user-install-manifest",
        },
    );
    let expected_before = snapshot_operation_paths(&arguments.root_cap, &files, &retired_files)?;
    let expected_permissions =
        snapshot_operation_permissions(&arguments.root_cap, &expected_before, &retired_files)?;
    let changed_paths = changed_paths(
        &expected_before,
        &arguments.root_cap,
        &files,
        &retired_files,
    )?;
    Ok(UserPlan {
        files,
        retired_files,
        changed_paths,
        plan_digest,
        manifest_relative,
        marketplace_root,
        expected_before,
        expected_permissions,
        qualified_host_version: None,
        prior_antigravity_activation_source,
        expected_antigravity_stage,
    })
}

fn insert_user_setup_review(files: &mut BTreeMap<PathBuf, PlannedFile>) {
    files.insert(
        PathBuf::from(".hive/config/user-setup-review.yml"),
        PlannedFile {
            bytes: USER_070_SETUP_REVIEW.to_vec(),
            executable: false,
            ownership: "shared-migration-state",
        },
    );
}

fn guidance_path(host: UserHost, root: &Dir) -> Result<PathBuf, InstallError> {
    match host {
        UserHost::Codex => {
            let override_path = Path::new(".codex/AGENTS.override.md");
            let bytes = read_optional_regular(root, override_path, MAX_USER_FILE_BYTES)?;
            if bytes.as_deref().is_some_and(|value| !value.is_empty()) {
                Ok(override_path.to_path_buf())
            } else {
                Ok(PathBuf::from(".codex/AGENTS.md"))
            }
        }
        UserHost::Claude => Ok(PathBuf::from(".claude/CLAUDE.md")),
        UserHost::Antigravity => Ok(PathBuf::from(".gemini/GEMINI.md")),
    }
}

fn validate_prior_ownership(
    root: &Dir,
    files: &BTreeMap<PathBuf, PlannedFile>,
    guidance_path: &Path,
    expected_source_release_digest: &str,
    expected_entries: &[UserOwnershipEntry],
    prior: Option<&UserOwnershipManifest>,
    authenticated_prior: Option<&AuthenticatedUserInventory>,
) -> Result<BTreeMap<PathBuf, RetiredFile>, InstallError> {
    let authenticated = if let Some(manifest) = prior {
        let recomputed = inventory_digest(
            manifest.host,
            &manifest.product_version,
            &manifest.host_version_range,
            Path::new(&manifest.guidance_path),
            &manifest.source_release_digest,
            &manifest.entries,
        );
        if manifest.plan_digest != recomputed {
            return Err(InstallError::Conflict(
                "installed ownership manifest is not internally reproducible".to_owned(),
            ));
        }
        let authenticated = authenticated_user_inventory(
            manifest.host,
            &InventoryAuthentication {
                product_version: &manifest.product_version,
                installed_host_version_range: &manifest.host_version_range,
                source_release_digest: &manifest.source_release_digest,
                installed_entries: &manifest.entries,
                installed_guidance_path: Path::new(&manifest.guidance_path),
                current_guidance_path: guidance_path,
                current_source_release_digest: expected_source_release_digest,
                current_entries: expected_entries,
                authenticated_prior,
            },
        )
        .ok_or_else(|| {
            InstallError::Conflict(
                "installed ownership manifest does not match an authenticated Hive release"
                    .to_owned(),
            )
        })?;
        validate_manifest_against_authenticated_inventory(manifest, &authenticated)?;
        validate_authenticated_installed_bytes(root, &authenticated)?;
        Some(authenticated)
    } else {
        None
    };
    let authenticated_entries = authenticated
        .as_ref()
        .map(|inventory| {
            inventory
                .entries
                .iter()
                .map(|entry| (entry.path.as_str(), entry))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    for (relative, planned) in files {
        if read_optional_regular(root, relative, MAX_USER_FILE_BYTES)?.is_none() {
            continue;
        }
        let path = portable(relative);
        if matches!(
            planned.ownership,
            "shared-marker" | "shared-migration-state" | "canonical-data-protected"
        ) {
            continue;
        }
        let Some(entry) = authenticated_entries.get(path.as_str()) else {
            return Err(InstallError::Conflict(format!(
                "occupied user installation path lacks exact prior Hive ownership: {path}"
            )));
        };
        if entry.ownership != planned.ownership {
            return Err(InstallError::Conflict(format!(
                "occupied user installation path has incompatible prior Hive ownership: {path}"
            )));
        }
    }
    let mut retired = BTreeMap::new();
    if let Some(authenticated) = authenticated {
        for entry in authenticated.entries.iter().filter(|entry| {
            is_managed_ownership(&entry.ownership)
                && !files.contains_key(Path::new(entry.path.as_str()))
        }) {
            let relative = PathBuf::from(&entry.path);
            let bytes =
                read_optional_regular(root, &relative, MAX_USER_FILE_BYTES)?.ok_or_else(|| {
                    InstallError::Conflict(format!(
                        "authenticated prior Hive inventory names a missing managed path: {}",
                        entry.path
                    ))
                })?;
            retired.insert(
                relative,
                RetiredFile {
                    bytes,
                    #[cfg(unix)]
                    permissions: FilePermissions {
                        executable: entry.executable,
                        unix_mode: entry.unix_mode,
                    },
                },
            );
        }
    }
    Ok(retired)
}

fn authenticated_antigravity_stage_tree(
    root: &Dir,
    prior: Option<&UserOwnershipManifest>,
) -> Result<Option<RegularTree>, InstallError> {
    let Some(manifest) = prior.filter(|manifest| manifest.host == UserHost::Antigravity) else {
        return Ok(None);
    };
    let source = Path::new(ANTIGRAVITY_SOURCE_RELATIVE);
    let stage = Path::new(ANTIGRAVITY_STAGE_RELATIVE);
    let prefix = if manifest
        .entries
        .iter()
        .any(|entry| Path::new(&entry.path) == source.join("plugin.json"))
    {
        source
    } else if manifest
        .entries
        .iter()
        .any(|entry| Path::new(&entry.path) == stage.join("plugin.json"))
    {
        stage
    } else {
        return Err(InstallError::Conflict(
            "authenticated prior Antigravity inventory has no plugin package".to_owned(),
        ));
    };
    let mut tree = RegularTree::default();
    for entry in &manifest.entries {
        let path = Path::new(&entry.path);
        let Ok(relative) = path.strip_prefix(prefix) else {
            continue;
        };
        if relative.as_os_str().is_empty() || !is_managed_ownership(&entry.ownership) {
            continue;
        }
        let bytes = read_optional_regular(root, path, MAX_USER_FILE_BYTES)?.ok_or_else(|| {
            InstallError::Conflict(format!(
                "authenticated prior Antigravity package is missing {}",
                entry.path
            ))
        })?;
        insert_regular_tree_file(&mut tree, relative, bytes);
    }
    if !tree.files.contains_key(Path::new("plugin.json")) {
        return Err(InstallError::Conflict(
            "authenticated prior Antigravity package omitted plugin.json".to_owned(),
        ));
    }
    Ok(Some(tree))
}

fn insert_regular_tree_file(tree: &mut RegularTree, relative: &Path, bytes: Vec<u8>) {
    let mut parent = relative.parent();
    while let Some(directory) = parent.filter(|path| !path.as_os_str().is_empty()) {
        tree.directories.insert(directory.to_path_buf());
        parent = directory.parent();
    }
    tree.files.insert(relative.to_path_buf(), bytes);
}

fn validate_manifest_against_authenticated_inventory(
    manifest: &UserOwnershipManifest,
    authenticated: &AuthenticatedUserInventory,
) -> Result<(), InstallError> {
    if manifest.product_version != authenticated.product_version
        || manifest.host != authenticated.host
        || manifest.host_version_range != authenticated.host_version_range
        || manifest.source_release_digest != authenticated.source_release_digest
        || manifest.guidance_path != authenticated.guidance_path
        || manifest.entries.len() != authenticated.entries.len()
    {
        return Err(InstallError::Conflict(
            "installed ownership manifest differs from its authenticated Hive release".to_owned(),
        ));
    }
    for (entry, expected) in manifest.entries.iter().zip(&authenticated.entries) {
        let managed = is_managed_ownership(&expected.ownership);
        if entry.path != expected.path
            || entry.executable != expected.executable
            || entry.unix_mode != expected.unix_mode
            || entry.ownership != expected.ownership
            || (managed && entry.digest != expected.digest)
        {
            return Err(InstallError::Conflict(format!(
                "installed ownership entry differs from its authenticated Hive release: {}",
                entry.path
            )));
        }
    }
    Ok(())
}

fn validate_authenticated_installed_bytes(
    root: &Dir,
    authenticated: &AuthenticatedUserInventory,
) -> Result<(), InstallError> {
    for entry in authenticated
        .entries
        .iter()
        .filter(|entry| is_managed_ownership(&entry.ownership))
    {
        let relative = Path::new(&entry.path);
        let bytes =
            read_optional_regular(root, relative, MAX_USER_FILE_BYTES)?.ok_or_else(|| {
                InstallError::Conflict(format!(
                    "authenticated prior Hive inventory names a missing managed path: {}",
                    entry.path
                ))
            })?;
        let permissions = file_permissions(root, relative)?;
        if sha256_digest(&bytes) != entry.digest
            || permissions.unix_mode != entry.unix_mode
            || !permissions_match_managed_mode(permissions, entry.executable)
        {
            return Err(InstallError::Conflict(format!(
                "installed managed path differs from its authenticated prior Hive release: {}",
                entry.path
            )));
        }
    }
    Ok(())
}

fn is_managed_ownership(ownership: &str) -> bool {
    !matches!(
        ownership,
        "shared-marker" | "shared-migration-state" | "canonical-data-protected"
    )
}

#[cfg(unix)]
fn permissions_match_managed_mode(permissions: FilePermissions, executable: bool) -> bool {
    permissions.unix_mode == Some(if executable { 0o755 } else { 0o644 })
}

#[cfg(not(unix))]
fn permissions_match_managed_mode(_permissions: FilePermissions, _executable: bool) -> bool {
    true
}

fn render_user_guidance(
    host: UserHost,
    setup: Option<&crate::user_setup::UserSetupConfig>,
) -> Vec<u8> {
    let (heading, adapter_label, body, footer) = setup.map_or_else(
        || {
            (
                "# Aigent Hive user directives / 사용자 지침",
                "Active adapter / 활성 adapter",
                "- State / 상태: `setup-required`\n- Ask the user to choose `English` or `한국어` first, then ask for daily update-check consent. / 먼저 `English` 또는 `한국어`를 선택하고 일일 update 확인 동의를 질문.\n- Use the installed `aigent-hive:user-setup` Skill before ordinary Hive Skills. / 일반 Hive Skill보다 설치된 `aigent-hive:user-setup` Skill을 먼저 사용.\n- Before setup completes, only setup, doctor, update, and recover operations are available. / 설정 완료 전 setup, doctor, update, recover만 사용 가능.\n"
                    .to_owned(),
                "- Preserve foreign guidance bytes and modify only exact Hive marker blocks. / Foreign guidance bytes를 보존하고 exact Hive marker block만 변경.\n- Never request provider API credentials or call model-provider APIs on Hive's behalf. / Provider API credential을 요청하거나 Hive를 대신해 model-provider API를 호출하지 않음.\n",
            )
        },
        |config| {
            let hosts = config
                .selected_hosts
                .iter()
                .map(|selected| selected.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let wiki = if config.wiki.enabled {
                config.wiki.backend.as_str()
            } else {
                "disabled"
            };
            let update_check = if config.update_check.enabled {
                "enabled"
            } else {
                "disabled"
            };
            let memory_gate_en = if config.wiki.enabled {
                "- Before every final response, review the current user statement and completed outcome for one safe reusable fact, preference, workflow, decision, convention, project profile, or verified outcome. Resolve `user-root|current-project|named-project` scope explicitly. An unregistered repository's user-global fact stays at `user-root`; ambiguous project-specific scope fails closed.\n- For an explicit safe user-root statement, prefer `hive knowledge remember --user-root <user-root> --user-statement <normalized-fact> --claim-key <stable-key> --kind <preference|workflow|decision|convention|project-profile> --output json` exactly once; use `--request <request.json>` only for reviewed artifacts or another supported scope. Require the canonical Markdown and derived-index receipt before the final response; identical current truth is a no-op.\n- Never record a secret, credential, confidential item without current-action authorization, ephemeral status, ambiguous inference, private path, raw transcript, complete conversation, hook payload, tool output, hidden prompt, cache, database, or runtime state.\n"
            } else {
                "- Global Wiki is disabled: do not write or refresh knowledge.\n"
            };
            let memory_gate_ko = if config.wiki.enabled {
                "- 모든 최종 응답 전 현재 사용자 발화와 완료 결과에서 안전하고 재사용 가능한 사실·선호·작업 방식·결정·규약·프로젝트 특성·검증된 결과 1개를 검토. `user-root|current-project|named-project` 범위를 명시적으로 결정. 미등록 repository의 사용자 전역 사실은 `user-root`에 유지하고, 모호한 project 범위는 안전하게 중단.\n- 안전한 명시적 user-root 사용자 발화에는 `hive knowledge remember --user-root <user-root> --user-statement <normalized-fact> --claim-key <stable-key> --kind <preference|workflow|decision|convention|project-profile> --output json`을 정확히 1회 우선 실행. 검토 artifact 또는 다른 지원 범위에는 `--request <request.json>` 사용. 최종 응답 전 canonical Markdown과 derived-index receipt를 확인하며, 동일한 현재 truth는 no-op.\n- 현재 action 승인 없는 secret·credential·confidential 항목, ephemeral 상태, 모호한 추론, private path, raw transcript, complete conversation, hook payload, tool output, hidden prompt, cache, database, runtime state는 기록 금지.\n"
            } else {
                "- 전역 위키 비활성: knowledge 기록·갱신 금지.\n"
            };
            let memory_gate = match config.interface_language {
                crate::user_setup::InterfaceLanguage::En => memory_gate_en,
                crate::user_setup::InterfaceLanguage::Ko => memory_gate_ko,
            };
            match config.interface_language {
                crate::user_setup::InterfaceLanguage::En => (
                    "# Aigent Hive user directives",
                    "Active adapter",
                    format!(
                        "- State: `operational`\n- Interface language: `en`; use English for every question and response unless the user explicitly requests another language for the current response. A message written in another language does not by itself change this preference. Keep Korean only for exact Korean names, literals, quotations, or text the user asks to preserve.\n- Selected hosts: `{hosts}`\n- Global Wiki: `{wiki}`\n- Daily update check: `{update_check}`.\n{memory_gate}- When enabled, run `hive update --check --user-root <user-root> --output json` before the first Hive task of each host session; never install from a check.\n- Use `aigent-hive:project-setup` for project expedited or custom setup.\n- Project Markdown Wiki remains canonical; the user-root SQLite index is derived and shared.\n- Use `aigent-hive:project-refresh` for project projection upgrades.\n- Offer one optional refinement suggestion for ambiguous or detail-poor ordinary requests; never rewrite automatically.\n- Unless the user explicitly opts out for the current request, write every plan to an appropriate project Markdown file before presenting or executing it. Never mirror the persisted plan one-for-one in the session; reference it with a concise summary and file path, or provide the file path alone for extensive review.\n- Before presenting pending actions, finish every safe, in-scope, automatable task. Present only the remaining user-owned steps as a concise ordered guide with the exact action, expected result, and reason user authority is required. Separate failures or impossible tasks with their causes and recovery paths.\n"
                    ),
                    "- Preserve foreign guidance bytes and modify only exact Hive marker blocks.\n- Never request provider API credentials or call model-provider APIs on Hive's behalf.\n",
                ),
                crate::user_setup::InterfaceLanguage::Ko => (
                    "# Aigent Hive 사용자 지침",
                    "활성 어댑터",
                    format!(
                        "- 상태: `operational`\n- 사용 언어: `ko`; 현재 응답에 다른 언어를 사용하라는 명시적 요청이 없는 한 모든 질문과 응답에 한국어 사용. 다른 언어로 작성된 메시지만으로 이 선호를 변경하지 않음. 고유명사, 제품·패키지 이름, 명령어, 코드 식별자, 경로, 스키마 키, 정확한 화면 문구, 뚜렷한 한국어 대체어가 없는 용어만 영어 유지. 대체 가능한 일반 영어 단어의 한영 혼용 금지.\n- 선택한 호스트: `{hosts}`\n- 전역 위키: `{wiki}`\n- 일일 갱신 확인: `{update_check}`.\n{memory_gate}- 활성화한 경우 각 호스트 세션의 첫 Hive 작업 전에 `hive update --check --user-root <user-root> --output json` 실행. 확인만으로 설치 금지.\n- 프로젝트 빠른 설정 또는 사용자 지정 설정에는 `aigent-hive:project-setup` 사용.\n- 프로젝트 Markdown 위키가 정본이며 사용자 루트 SQLite 색인은 파생·공유 상태.\n- 프로젝트 투영 갱신에는 `aigent-hive:project-refresh` 사용.\n- 모호하거나 핵심 세부가 부족한 일반 요청에는 자동 재작성 없이 선택적 개선 제안 1개만 제공.\n- 현재 요청에서 사용자의 명시적 제외 요청이 없는 모든 계획을 적절한 프로젝트 Markdown 파일에 제시·실행 전 기록. 저장한 계획 전문을 session에 일대일 복제하지 않고 간결한 요약과 파일 경로로 참조하며, 광범위한 검토에는 파일 경로만 제시.\n- 남은 작업 제시 전 범위 안에서 안전하게 자동 처리 가능한 작업을 모두 완료. 사용자 권한이 필요한 단계만 정확한 행동·예상 결과·권한 필요 이유를 포함한 간결한 순서 안내로 제시. 실패·불가능 작업은 원인과 해결 경로를 분리해 제시.\n"
                    ),
                    "- 외부 지침 바이트 보존, 정확한 Hive 표시 블록만 변경.\n- 제공자 API 자격 증명 요청 금지, Hive를 대신한 모델 제공자 API 호출 금지.\n",
                ),
            }
        },
    );
    let body = body.replacen(
        "- Before presenting pending actions, finish every safe, in-scope, automatable task. Present only the remaining user-owned steps as a concise ordered guide with the exact action, expected result, and reason user authority is required. Separate failures or impossible tasks with their causes and recovery paths.\n",
        "- Before presenting pending actions, finish every safe, in-scope, automatable task. Present only the remaining user-owned steps as a concise ordered guide with the exact action, expected result, and reason user authority is required. Separate failures or impossible tasks with their causes and recovery paths.\n\\
- For `all todos`, `until completion`, `do not stop`, or an equivalent terminal request, continue while any in-scope agent-owned inspection, fix, verification, commit, permitted push, CI observation, or authorized publication remains. A progress report naming such work must not end the task. Before a final response, classify every remaining item as `agent-owned`, `awaiting-user-authority`, `awaiting-external-evidence`, or `blocked`; only no `agent-owned` work permits completion.\n",
        1,
    );
    let body = body.replacen(
        "- 남은 작업 제시 전 범위 안에서 안전하게 자동 처리 가능한 작업을 모두 완료. 사용자 권한이 필요한 단계만 정확한 행동·예상 결과·권한 필요 이유를 포함한 간결한 순서 안내로 제시. 실패·불가능 작업은 원인과 해결 경로를 분리해 제시.\n",
        "- 남은 작업 제시 전 범위 안에서 안전하게 자동 처리 가능한 작업을 모두 완료. 사용자 권한이 필요한 단계만 정확한 행동·예상 결과·권한 필요 이유를 포함한 간결한 순서 안내로 제시. 실패·불가능 작업은 원인과 해결 경로를 분리해 제시.\n\\
- `all todos`, `until completion`, `do not stop` 또는 같은 완료 요청: 범위 안 Agent 소유 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업이 남은 동안 계속 진행. 해당 작업이 남았다는 진행 보고로 task 종료 금지. 최종 응답 전 남은 항목을 `agent-owned`, `awaiting-user-authority`, `awaiting-external-evidence`, `blocked`로 분류. `agent-owned` 작업 `0건`일 때만 완료 표기.\n",
        1,
    );
    let explanation_style = setup.map_or(
        "- Explain in simple terms by default. Use concrete examples when they materially improve understanding, but do not force irrelevant examples or weaken technical precision. / 기본 설명은 쉬운 말로 작성. 이해에 도움이 될 때 구체적 예시 사용. 관련 없는 예시 강제 또는 기술적 정확성 약화 금지.\n",
        |config| match config.interface_language {
            crate::user_setup::InterfaceLanguage::En => {
                "- Explain in simple terms by default. Use concrete examples when they materially improve understanding, but do not force irrelevant examples or weaken technical precision.\n"
            }
            crate::user_setup::InterfaceLanguage::Ko => {
                "- 기본 설명은 쉬운 말로 작성. 이해에 도움이 될 때 구체적 예시 사용. 관련 없는 예시 강제 또는 기술적 정확성 약화 금지.\n"
            }
        },
    );
    let result_clarity = setup.map_or(
        "- For every passed, failed, skipped, deferred, unverified, or unsupported item, state the affected scope, exact reason, current host or platform relationship, whether it ran, and what the result does and does not prove. / 통과·실패·건너뜀·연기·미검증·미지원 항목마다 대상 범위, 정확한 이유, 현재 호스트·운영체제와의 관계, 실제 실행 여부, 증명 범위와 미증명 범위를 모두 명시.\n",
        |config| match config.interface_language {
            crate::user_setup::InterfaceLanguage::En => {
                "- For every passed, failed, skipped, deferred, unverified, or unsupported item, state the affected scope, exact reason, current host or platform relationship, whether it ran, and what the result does and does not prove. Never trade those qualifiers for brevity.\n"
            }
            crate::user_setup::InterfaceLanguage::Ko => {
                "- 통과·실패·건너뜀·연기·미검증·미지원 항목마다 대상 범위, 정확한 이유, 현재 호스트·운영체제와의 관계, 실제 실행 여부, 증명하는 범위와 증명하지 못한 범위를 모두 명시. 해석에 필요한 한정어를 간결함을 이유로 생략 금지.\n"
            }
        },
    );
    format!(
        "<!-- AIGENT-HIVE:USER:START -->\n{heading}\n\n- {adapter_label}: `{}`\n{body}{explanation_style}{result_clarity}{footer}<!-- AIGENT-HIVE:USER:END -->\n",
        host.as_str()
    )
    .into_bytes()
}

fn validate_operational_guidance(
    guidance: &[u8],
    setup: Option<&crate::user_setup::UserSetupConfig>,
) -> Result<(), InstallError> {
    let Some(config) = setup else {
        return Ok(());
    };
    let guidance = std::str::from_utf8(guidance).map_err(|_| {
        InstallError::Verification("generated user guidance is not UTF-8".to_owned())
    })?;
    let command = "hive knowledge remember --user-root <user-root> --user-statement <normalized-fact> --claim-key <stable-key>";
    if !config.wiki.enabled {
        return (!guidance.contains(command)).then_some(()).ok_or_else(|| {
            InstallError::Verification(
                "Wiki-disabled user guidance must not contain a knowledge write command".to_owned(),
            )
        });
    }
    let required = match config.interface_language {
        crate::user_setup::InterfaceLanguage::En => [
            "Before every final response, review the current user statement",
            command,
            "canonical Markdown and derived-index receipt",
            "ambiguous project-specific scope fails closed",
        ],
        crate::user_setup::InterfaceLanguage::Ko => [
            "모든 최종 응답 전 현재 사용자 발화와 완료 결과",
            command,
            "canonical Markdown과 derived-index receipt",
            "모호한 project 범위는 안전하게 중단",
        ],
    };
    required
        .iter()
        .all(|fragment| guidance.contains(fragment))
        .then_some(())
        .ok_or_else(|| {
            InstallError::Verification(
                "Wiki-enabled user guidance omitted the mandatory knowledge capture contract"
                    .to_owned(),
            )
        })
}

fn merge_user_marker(existing: &[u8], marker: &[u8]) -> Result<Vec<u8>, InstallError> {
    let starts = find_all(existing, USER_MARKER_START);
    let ends = find_all(existing, USER_MARKER_END);
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => {
            let mut output = existing.to_vec();
            if !output.is_empty() && !output.ends_with(b"\n") {
                output.push(b'\n');
            }
            output.extend_from_slice(marker);
            Ok(output)
        }
        ([start], [end]) if start < end => {
            let end = end + USER_MARKER_END.len();
            let mut output = Vec::with_capacity(existing.len() + marker.len());
            output.extend_from_slice(&existing[..*start]);
            output.extend_from_slice(marker);
            if end < existing.len() && marker.ends_with(b"\n") && existing[end] == b'\n' {
                output.extend_from_slice(&existing[end + 1..]);
            } else {
                output.extend_from_slice(&existing[end..]);
            }
            Ok(output)
        }
        _ => Err(InstallError::Conflict(
            "user guidance contains malformed, duplicate, or nested Aigent Hive user markers"
                .to_owned(),
        )),
    }
}

fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, value)| (value == needle).then_some(index))
        .collect()
}

fn seed_root_knowledge(
    root: &Dir,
    files: &mut BTreeMap<PathBuf, PlannedFile>,
) -> Result<(), InstallError> {
    for (relative, seed) in [
        (".hive/knowledge/Raw/README.md", ROOT_RAW_README),
        (".hive/knowledge/Schema/schema.md", ROOT_SCHEMA),
        (".hive/knowledge/Wiki/index.md", ROOT_WIKI_INDEX),
        (".hive/knowledge/Wiki/log.md", ROOT_WIKI_LOG),
        (".hive/knowledge/suppression.yml", ROOT_SUPPRESSION),
        (
            ".hive/guides/discord-usage-notifications.html",
            ROOT_DISCORD_USAGE_GUIDE,
        ),
    ] {
        let relative = PathBuf::from(relative);
        let bytes = read_optional_regular(root, &relative, MAX_USER_FILE_BYTES)?
            .unwrap_or_else(|| seed.to_vec());
        files.insert(
            relative,
            PlannedFile {
                bytes,
                executable: false,
                ownership: "canonical-data-protected",
            },
        );
    }
    Ok(())
}

fn strip_project_skill_prefix(path: &str) -> Option<PathBuf> {
    path.strip_prefix(".agents/")
        .or_else(|| path.strip_prefix(".claude/"))
        .map(PathBuf::from)
}

fn render_codex_marketplace() -> Vec<u8> {
    br#"{
  "name": "aigent-hive",
  "interface": {
    "displayName": "Aigent Hive"
  },
  "plugins": [
    {
      "name": "aigent-hive",
      "source": {
        "source": "local",
        "path": "./plugins/aigent-hive"
      },
      "policy": {
        "installation": "AVAILABLE",
        "authentication": "ON_INSTALL"
      },
      "category": "Productivity"
    }
  ]
}
"#
    .to_vec()
}

fn render_claude_marketplace() -> Vec<u8> {
    format!(
        "{{\n  \"name\": \"aigent-hive\",\n  \"owner\": {{\n    \"name\": \"Aigent Hive maintainers\"\n  }},\n  \"plugins\": [\n    {{\n      \"name\": \"aigent-hive\",\n      \"source\": \"./plugins/aigent-hive\",\n      \"description\": \"Initialize and maintain project-local Aigent Hive harnesses.\",\n      \"version\": \"{}\"\n    }}\n  ]\n}}\n",
        env!("CARGO_PKG_VERSION")
    )
    .into_bytes()
}

fn source_release_digest(files: &BTreeMap<PathBuf, PlannedFile>) -> String {
    source_release_digest_from_entries(&ownership_entries(files))
}

fn source_release_digest_from_entries(entries: &[UserOwnershipEntry]) -> String {
    let mut bytes = Vec::new();
    for entry in entries.iter().filter(|entry| {
        matches!(
            entry.ownership.as_str(),
            "immutable-plugin-package" | "host-skill-projection" | "host-adapter-metadata"
        )
    }) {
        bytes.extend_from_slice(entry.path.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(entry.digest.as_bytes());
        bytes.push(b'\n');
    }
    sha256_digest(&bytes)
}

fn ownership_entries(files: &BTreeMap<PathBuf, PlannedFile>) -> Vec<UserOwnershipEntry> {
    files
        .iter()
        .map(|(path, file)| UserOwnershipEntry {
            path: portable(path),
            digest: sha256_digest(&file.bytes),
            executable: file.executable,
            unix_mode: installed_unix_mode(file.executable),
            ownership: file.ownership.to_owned(),
        })
        .collect()
}

fn inventory_digest(
    host: UserHost,
    product_version: &str,
    host_version_range: &str,
    guidance_path: &Path,
    source_release_digest: &str,
    entries: &[UserOwnershipEntry],
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(host.as_str().as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(product_version.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(host_version_range.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(portable(guidance_path).as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(source_release_digest.as_bytes());
    bytes.push(b'\n');
    for entry in entries {
        bytes.extend_from_slice(entry.path.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(entry.digest.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(if entry.executable { b"1" } else { b"0" });
        bytes.push(0);
        bytes.extend_from_slice(
            entry
                .unix_mode
                .map_or_else(|| "-".to_owned(), |mode| format!("{mode:o}"))
                .as_bytes(),
        );
        bytes.push(0);
        bytes.extend_from_slice(entry.ownership.as_bytes());
        bytes.push(b'\n');
    }
    sha256_digest(&bytes)
}

struct InventoryAuthentication<'a> {
    product_version: &'a str,
    installed_host_version_range: &'a str,
    source_release_digest: &'a str,
    installed_entries: &'a [UserOwnershipEntry],
    installed_guidance_path: &'a Path,
    current_guidance_path: &'a Path,
    current_source_release_digest: &'a str,
    current_entries: &'a [UserOwnershipEntry],
    authenticated_prior: Option<&'a AuthenticatedUserInventory>,
}

fn authenticated_user_inventory(
    host: UserHost,
    request: &InventoryAuthentication<'_>,
) -> Option<AuthenticatedUserInventory> {
    if request.product_version == env!("CARGO_PKG_VERSION")
        && request.source_release_digest == request.current_source_release_digest
    {
        let host_version_range = if host == UserHost::Antigravity
            && request.installed_host_version_range == ">=1.1.7 <2.0.0"
        {
            request.installed_host_version_range
        } else {
            host.version_range()
        };
        return Some(AuthenticatedUserInventory {
            product_version: request.product_version.to_owned(),
            host,
            host_version_range: host_version_range.to_owned(),
            source_release_digest: request.source_release_digest.to_owned(),
            guidance_path: portable(request.current_guidance_path),
            entries: request.current_entries.to_vec(),
        });
    }
    if let Some(prior) = request.authenticated_prior {
        if prior.host == host
            && prior.product_version == request.product_version
            && prior.host_version_range == request.installed_host_version_range
            && prior.source_release_digest == request.source_release_digest
        {
            return Some(prior.clone());
        }
    }
    if let Some(developer) =
        developer_authenticated_user_inventory(host, request, env!("HIVE_PACKAGE_VERSION"))
    {
        return Some(developer);
    }
    if let Some(historical) = test_three_user_inventory(host, request) {
        return Some(historical);
    }
    if let Some(historical) = pre_scope_routing_test_inventory(host, request) {
        return Some(historical);
    }
    if request.product_version == "0.8.0" {
        if let Some(historical) = historical_080_user_inventory(
            host,
            request.installed_host_version_range,
            request.installed_guidance_path,
            request.installed_entries,
        ) {
            if historical.source_release_digest == request.source_release_digest {
                return Some(historical);
            }
        }
    }
    if request.product_version == "0.7.0" {
        if let Some(historical) = historical_070_codex_onboarding_inventory(
            host,
            request.installed_host_version_range,
            request.installed_guidance_path,
        ) {
            if historical.source_release_digest == request.source_release_digest {
                return Some(historical);
            }
        }
        if let Some(historical) = historical_070_user_inventory(
            host,
            request.installed_host_version_range,
            request.installed_guidance_path,
        ) {
            if historical.source_release_digest == request.source_release_digest {
                return Some(historical);
            }
        }
    }
    let historical = historical_user_inventory(host);
    if historical.product_version == request.product_version
        && historical.source_release_digest == request.source_release_digest
    {
        return Some(historical);
    }
    if host == UserHost::Antigravity {
        let legacy = legacy_antigravity_directory_scan_inventory();
        if legacy.product_version == request.product_version
            && legacy.source_release_digest == request.source_release_digest
        {
            return Some(legacy);
        }
    }
    None
}

/// A local `-dev` binary is intentionally not a public release identity. It may therefore use
/// the currently installed, internally reproducible user manifest as its three-way base, but
/// only after the ordinary manifest and live-byte checks in `validate_prior_ownership` run.
/// Public stable and `-test[.N]` builds never enter this branch.
fn developer_authenticated_user_inventory(
    host: UserHost,
    request: &InventoryAuthentication<'_>,
    package_version: &str,
) -> Option<AuthenticatedUserInventory> {
    if package_version != format!("{}-dev", env!("CARGO_PKG_VERSION"))
        || request.product_version != env!("CARGO_PKG_VERSION")
        || request.installed_host_version_range != host.version_range()
        || portable(request.installed_guidance_path) != portable(request.current_guidance_path)
        || request.installed_entries.is_empty()
        || request.source_release_digest
            != source_release_digest_from_entries(request.installed_entries)
    {
        return None;
    }

    Some(AuthenticatedUserInventory {
        product_version: request.product_version.to_owned(),
        host,
        host_version_range: request.installed_host_version_range.to_owned(),
        source_release_digest: request.source_release_digest.to_owned(),
        guidance_path: portable(request.installed_guidance_path),
        entries: request.installed_entries.to_vec(),
    })
}

fn test_three_user_inventory(
    host: UserHost,
    request: &InventoryAuthentication<'_>,
) -> Option<AuthenticatedUserInventory> {
    if request.product_version != "0.9.0"
        || request.installed_host_version_range != host.version_range()
        || request.current_entries.is_empty()
    {
        return None;
    }

    let mut entries = request.current_entries.to_vec();
    let mut setup_hive = false;
    for entry in &mut entries {
        if is_managed_ownership(&entry.ownership)
            && entry.path.ends_with("/skills/user-setup/SKILL.md")
        {
            TEST3_SETUP_HIVE_DIGEST.clone_into(&mut entry.digest);
            setup_hive = true;
        }
    }
    if !setup_hive {
        return None;
    }
    let source_release_digest = source_release_digest_from_entries(&entries);
    if request.source_release_digest != source_release_digest {
        return None;
    }
    Some(AuthenticatedUserInventory {
        product_version: request.product_version.to_owned(),
        host,
        host_version_range: request.installed_host_version_range.to_owned(),
        source_release_digest,
        guidance_path: portable(request.installed_guidance_path),
        entries,
    })
}

fn historical_070_codex_onboarding_inventory(
    host: UserHost,
    installed_host_version_range: &str,
    guidance_path: &Path,
) -> Option<AuthenticatedUserInventory> {
    if host != UserHost::Codex
        || installed_host_version_range != ">=0.145.0 <1.0.0"
        || portable(guidance_path) != ".codex/AGENTS.md"
    {
        return None;
    }

    let mut entries = historical_070_host_entries(host);
    for (path, digest) in [
        (
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/auto-setup-harness/SKILL.md",
            USER_070_CODEX_ONBOARDING_AUTO_SETUP_DIGEST,
        ),
        (
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/hive-knowledge-capture/SKILL.md",
            USER_070_CODEX_ONBOARDING_KNOWLEDGE_CAPTURE_DIGEST,
        ),
        (
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/hive-knowledge-maintenance/SKILL.md",
            USER_070_CODEX_ONBOARDING_KNOWLEDGE_MAINTENANCE_DIGEST,
        ),
        (
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/hive-knowledge-query/SKILL.md",
            USER_070_CODEX_ONBOARDING_KNOWLEDGE_QUERY_DIGEST,
        ),
        (
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/setup-harness/SKILL.md",
            USER_070_CODEX_ONBOARDING_SETUP_HARNESS_DIGEST,
        ),
        (
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/setup-hive/SKILL.md",
            USER_070_CODEX_ONBOARDING_SETUP_HIVE_DIGEST,
        ),
    ] {
        if let Some(entry) = entries.iter_mut().find(|entry| entry.path == path) {
            digest.clone_into(&mut entry.digest);
        } else {
            entries.push(historical_070_entry(
                path.to_owned(),
                digest,
                false,
                "immutable-plugin-package",
            ));
        }
    }
    entries.extend([
        historical_070_entry(
            ".codex/AGENTS.md".to_owned(),
            USER_070_CODEX_ONBOARDING_GUIDANCE_DIGEST,
            false,
            "shared-marker",
        ),
        historical_070_entry(
            ".hive/knowledge/Raw/README.md".to_owned(),
            USER_070_CODEX_ONBOARDING_RAW_README_DIGEST,
            false,
            "canonical-data-protected",
        ),
        historical_070_entry(
            ".hive/knowledge/Schema/schema.md".to_owned(),
            USER_070_CODEX_ONBOARDING_SCHEMA_DIGEST,
            false,
            "canonical-data-protected",
        ),
        historical_070_entry(
            ".hive/knowledge/Wiki/index.md".to_owned(),
            USER_070_CODEX_ONBOARDING_WIKI_INDEX_DIGEST,
            false,
            "canonical-data-protected",
        ),
        historical_070_entry(
            ".hive/knowledge/Wiki/log.md".to_owned(),
            USER_070_CODEX_ONBOARDING_WIKI_LOG_DIGEST,
            false,
            "canonical-data-protected",
        ),
        historical_070_entry(
            ".hive/knowledge/suppression.yml".to_owned(),
            USER_070_CODEX_ONBOARDING_SUPPRESSION_DIGEST,
            false,
            "canonical-data-protected",
        ),
    ]);
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let source_release_digest = source_release_digest_from_entries(&entries);
    if source_release_digest != USER_070_CODEX_ONBOARDING_SOURCE_DIGEST {
        return None;
    }
    Some(AuthenticatedUserInventory {
        product_version: "0.7.0".to_owned(),
        host,
        host_version_range: installed_host_version_range.to_owned(),
        source_release_digest,
        guidance_path: ".codex/AGENTS.md".to_owned(),
        entries,
    })
}

fn pre_scope_routing_test_inventory(
    host: UserHost,
    request: &InventoryAuthentication<'_>,
) -> Option<AuthenticatedUserInventory> {
    if request.product_version != "0.9.0"
        || request.installed_host_version_range != host.version_range()
        || request.current_entries.is_empty()
    {
        return None;
    }

    let mut entries = request.current_entries.to_vec();
    let mut setup_hive = false;
    for entry in &mut entries {
        if !is_managed_ownership(&entry.ownership) {
            continue;
        }
        if entry.path.ends_with("/skills/user-setup/SKILL.md") {
            PRE_SCOPE_ROUTING_SETUP_HIVE_DIGEST.clone_into(&mut entry.digest);
            setup_hive = true;
        } else if entry.path.ends_with("/skills/project-setup/SKILL.md") {
            PRE_SCOPE_ROUTING_SETUP_HARNESS_DIGEST.clone_into(&mut entry.digest);
        }
    }
    if !setup_hive {
        return None;
    }
    let source_release_digest = source_release_digest_from_entries(&entries);
    if request.source_release_digest != source_release_digest {
        return None;
    }
    Some(AuthenticatedUserInventory {
        product_version: request.product_version.to_owned(),
        host,
        host_version_range: request.installed_host_version_range.to_owned(),
        source_release_digest,
        guidance_path: portable(request.installed_guidance_path),
        entries,
    })
}

#[allow(clippy::too_many_lines)]
fn historical_080_user_inventory(
    host: UserHost,
    installed_host_version_range: &str,
    guidance_path: &Path,
    installed_entries: &[UserOwnershipEntry],
) -> Option<AuthenticatedUserInventory> {
    const KNOWLEDGE_PATHS: [&str; 5] = [
        ".hive/knowledge/Raw/README.md",
        ".hive/knowledge/Schema/schema.md",
        ".hive/knowledge/Wiki/index.md",
        ".hive/knowledge/Wiki/log.md",
        ".hive/knowledge/suppression.yml",
    ];
    if installed_host_version_range != host.version_range()
        || !installed_entries
            .windows(2)
            .all(|pair| pair[0].path < pair[1].path)
        || installed_entries.iter().any(|entry| {
            !valid_sha256(&entry.digest)
                || entry.executable && entry.unix_mode != installed_unix_mode(true)
                || !entry.executable && entry.unix_mode != installed_unix_mode(false)
        })
    {
        return None;
    }
    let guidance = portable(guidance_path);
    let valid_guidance = match host {
        UserHost::Codex => matches!(
            guidance.as_str(),
            ".codex/AGENTS.md" | ".codex/AGENTS.override.md"
        ),
        UserHost::Claude => guidance == ".claude/CLAUDE.md",
        UserHost::Antigravity => guidance == ".gemini/GEMINI.md",
    };
    if !valid_guidance {
        return None;
    }

    let expected_managed = ownership_entries(&historical_080_managed_files(host));
    let expected_by_path = expected_managed
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let installed_paths = installed_entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    for entry in installed_entries
        .iter()
        .filter(|entry| is_managed_ownership(&entry.ownership))
    {
        let expected = expected_by_path.get(entry.path.as_str())?;
        if entry.digest != expected.digest
            || entry.executable != expected.executable
            || entry.unix_mode != expected.unix_mode
            || entry.ownership != expected.ownership
        {
            return None;
        }
    }
    if historical_080_required_paths(host)
        .iter()
        .any(|path| !installed_paths.contains(path.as_str()))
    {
        return None;
    }
    for (name, _, _) in USER_080_SKILLS {
        let paths = historical_080_skill_paths(host, name);
        let present = paths
            .iter()
            .filter(|path| installed_paths.contains(path.as_str()))
            .count();
        if (present != 0 && present != paths.len()) || (name == "setup-hive" && present == 0) {
            return None;
        }
    }

    let mut guidance_count = 0;
    let mut knowledge_count = 0;
    let mut review_count = 0;
    for entry in installed_entries
        .iter()
        .filter(|entry| !is_managed_ownership(&entry.ownership))
    {
        if entry.executable {
            return None;
        }
        match entry.ownership.as_str() {
            "shared-marker" if entry.path == guidance => guidance_count += 1,
            "canonical-data-protected" if KNOWLEDGE_PATHS.contains(&entry.path.as_str()) => {
                knowledge_count += 1;
            }
            "shared-migration-state" if entry.path == ".hive/config/user-setup-review.yml" => {
                review_count += 1;
            }
            _ => return None,
        }
    }
    if guidance_count != 1
        || !matches!(knowledge_count, 0 | 5)
        || review_count > 1
        || installed_entries.len()
            != installed_entries
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<BTreeSet<_>>()
                .len()
    {
        return None;
    }

    Some(AuthenticatedUserInventory {
        product_version: "0.8.0".to_owned(),
        host,
        host_version_range: installed_host_version_range.to_owned(),
        source_release_digest: source_release_digest_from_entries(installed_entries),
        guidance_path: guidance,
        entries: installed_entries.to_vec(),
    })
}

#[allow(clippy::too_many_lines)]
fn historical_080_managed_files(host: UserHost) -> BTreeMap<PathBuf, PlannedFile> {
    let mut files = BTreeMap::new();
    match host {
        UserHost::Codex | UserHost::Claude => {
            let root = PathBuf::from(format!(
                ".hive/marketplaces/{}/plugins/aigent-hive",
                host.as_str()
            ));
            for &(name, skill, metadata) in &USER_080_SKILLS {
                files.insert(
                    root.join(format!("skills/{name}/SKILL.md")),
                    PlannedFile {
                        bytes: skill.to_vec(),
                        executable: false,
                        ownership: "immutable-plugin-package",
                    },
                );
                if host == UserHost::Codex {
                    files.insert(
                        root.join(format!("skills/{name}/agents/openai.yaml")),
                        PlannedFile {
                            bytes: metadata.to_vec(),
                            executable: false,
                            ownership: "immutable-plugin-package",
                        },
                    );
                }
            }
            for (relative, bytes, executable) in [
                (
                    ".codex-plugin/plugin.json",
                    USER_080_CODEX_PLUGIN_MANIFEST,
                    false,
                ),
                (
                    ".claude-plugin/plugin.json",
                    USER_080_CLAUDE_PLUGIN_MANIFEST,
                    false,
                ),
                (
                    "bin/hive-claude-usage-capture",
                    USER_080_CLAUDE_USAGE_CAPTURE,
                    true,
                ),
            ] {
                files.insert(
                    root.join(relative),
                    PlannedFile {
                        bytes: bytes.to_vec(),
                        executable,
                        ownership: "immutable-plugin-package",
                    },
                );
            }
            let marketplace = PathBuf::from(format!(
                ".hive/marketplaces/{}/{}",
                host.as_str(),
                if host == UserHost::Codex {
                    ".agents/plugins/marketplace.json"
                } else {
                    ".claude-plugin/marketplace.json"
                }
            ));
            files.insert(
                marketplace,
                PlannedFile {
                    bytes: if host == UserHost::Codex {
                        USER_080_CODEX_MARKETPLACE.to_vec()
                    } else {
                        USER_080_CLAUDE_MARKETPLACE.to_vec()
                    },
                    executable: false,
                    ownership: "host-adapter-metadata",
                },
            );
        }
        UserHost::Antigravity => {
            for &(name, skill, metadata) in &USER_080_SKILLS {
                for (relative, bytes, ownership) in [
                    (
                        PathBuf::from(ANTIGRAVITY_SOURCE_RELATIVE)
                            .join(format!("skills/{name}/SKILL.md")),
                        skill,
                        "immutable-plugin-package",
                    ),
                    (
                        PathBuf::from(ANTIGRAVITY_SOURCE_RELATIVE)
                            .join(format!("skills/{name}/agents/openai.yaml")),
                        metadata,
                        "immutable-plugin-package",
                    ),
                    (
                        PathBuf::from(format!(".gemini/config/skills/{name}/SKILL.md")),
                        skill,
                        "host-skill-projection",
                    ),
                    (
                        PathBuf::from(format!(".gemini/config/skills/{name}/agents/openai.yaml")),
                        metadata,
                        "host-skill-projection",
                    ),
                ] {
                    files.insert(
                        relative,
                        PlannedFile {
                            bytes: bytes.to_vec(),
                            executable: false,
                            ownership,
                        },
                    );
                }
            }
            files.insert(
                PathBuf::from(ANTIGRAVITY_SOURCE_RELATIVE).join("plugin.json"),
                PlannedFile {
                    bytes: USER_080_ANTIGRAVITY_PLUGIN_MANIFEST.to_vec(),
                    executable: false,
                    ownership: "immutable-plugin-package",
                },
            );
        }
    }
    files
}

fn historical_080_skill_paths(host: UserHost, name: &str) -> Vec<String> {
    match host {
        UserHost::Codex => vec![
            format!(".hive/marketplaces/codex/plugins/aigent-hive/skills/{name}/SKILL.md"),
            format!(
                ".hive/marketplaces/codex/plugins/aigent-hive/skills/{name}/agents/openai.yaml"
            ),
        ],
        UserHost::Claude => vec![format!(
            ".hive/marketplaces/claude/plugins/aigent-hive/skills/{name}/SKILL.md"
        )],
        UserHost::Antigravity => vec![
            format!("{ANTIGRAVITY_SOURCE_RELATIVE}/skills/{name}/SKILL.md"),
            format!("{ANTIGRAVITY_SOURCE_RELATIVE}/skills/{name}/agents/openai.yaml"),
            format!(".gemini/config/skills/{name}/SKILL.md"),
            format!(".gemini/config/skills/{name}/agents/openai.yaml"),
        ],
    }
}

fn historical_080_required_paths(host: UserHost) -> Vec<String> {
    match host {
        UserHost::Codex | UserHost::Claude => {
            let host_name = host.as_str();
            let root = format!(".hive/marketplaces/{host_name}");
            let plugin = format!("{root}/plugins/aigent-hive");
            vec![
                format!("{plugin}/.codex-plugin/plugin.json"),
                format!("{plugin}/.claude-plugin/plugin.json"),
                format!("{plugin}/bin/hive-claude-usage-capture"),
                format!(
                    "{root}/{}",
                    if host == UserHost::Codex {
                        ".agents/plugins/marketplace.json"
                    } else {
                        ".claude-plugin/marketplace.json"
                    }
                ),
            ]
        }
        UserHost::Antigravity => {
            vec![format!("{ANTIGRAVITY_SOURCE_RELATIVE}/plugin.json")]
        }
    }
}

fn historical_070_user_inventory(
    host: UserHost,
    installed_host_version_range: &str,
    guidance_path: &Path,
) -> Option<AuthenticatedUserInventory> {
    let supported_range = match host {
        UserHost::Codex => installed_host_version_range == ">=0.145.0 <1.0.0",
        UserHost::Claude => installed_host_version_range == ">=2.1.0 <3.0.0",
        UserHost::Antigravity => matches!(
            installed_host_version_range,
            ">=1.1.7 <1.2.0" | ">=1.1.7 <2.0.0"
        ),
    };
    if !supported_range {
        return None;
    }
    let guidance = portable(guidance_path);
    let valid_guidance = match host {
        UserHost::Codex => matches!(
            guidance.as_str(),
            ".codex/AGENTS.md" | ".codex/AGENTS.override.md"
        ),
        UserHost::Claude => guidance == ".claude/CLAUDE.md",
        UserHost::Antigravity => guidance == ".gemini/GEMINI.md",
    };
    if !valid_guidance {
        return None;
    }
    let mut entries = historical_070_host_entries(host);
    entries.push(historical_070_entry(
        guidance.clone(),
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        false,
        "shared-marker",
    ));
    for path in [
        ".hive/knowledge/Raw/README.md",
        ".hive/knowledge/Schema/schema.md",
        ".hive/knowledge/Wiki/index.md",
        ".hive/knowledge/Wiki/log.md",
        ".hive/knowledge/suppression.yml",
    ] {
        entries.push(historical_070_entry(
            path.to_owned(),
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            false,
            "canonical-data-protected",
        ));
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Some(AuthenticatedUserInventory {
        product_version: "0.7.0".to_owned(),
        host,
        host_version_range: installed_host_version_range.to_owned(),
        source_release_digest: source_release_digest_from_entries(&entries),
        guidance_path: guidance,
        entries,
    })
}

fn historical_070_entry(
    path: String,
    digest: &str,
    executable: bool,
    ownership: &str,
) -> UserOwnershipEntry {
    UserOwnershipEntry {
        path,
        digest: digest.to_owned(),
        executable,
        unix_mode: installed_unix_mode(executable),
        ownership: ownership.to_owned(),
    }
}

fn historical_070_host_entries(host: UserHost) -> Vec<UserOwnershipEntry> {
    let mut entries = Vec::new();
    match host {
        UserHost::Codex | UserHost::Claude => {
            let host_name = host.as_str();
            let root = format!(".hive/marketplaces/{host_name}");
            let plugin = format!("{root}/plugins/aigent-hive");
            for (skill, digest) in LEGACY_ANTIGRAVITY_070_SKILLS {
                entries.push(historical_070_entry(
                    format!("{plugin}/skills/{skill}/SKILL.md"),
                    digest,
                    false,
                    "immutable-plugin-package",
                ));
            }
            entries.extend([
                historical_070_entry(
                    format!("{plugin}/.codex-plugin/plugin.json"),
                    USER_070_CODEX_PLUGIN_DIGEST,
                    false,
                    "immutable-plugin-package",
                ),
                historical_070_entry(
                    format!("{plugin}/.claude-plugin/plugin.json"),
                    USER_070_CLAUDE_PLUGIN_DIGEST,
                    false,
                    "immutable-plugin-package",
                ),
                historical_070_entry(
                    format!("{plugin}/bin/hive-claude-usage-capture"),
                    USER_070_CLAUDE_CAPTURE_DIGEST,
                    true,
                    "immutable-plugin-package",
                ),
                historical_070_entry(
                    match host {
                        UserHost::Codex => format!("{root}/.agents/plugins/marketplace.json"),
                        UserHost::Claude => {
                            format!("{root}/.claude-plugin/marketplace.json")
                        }
                        UserHost::Antigravity => unreachable!(),
                    },
                    match host {
                        UserHost::Codex => USER_070_CODEX_MARKETPLACE_DIGEST,
                        UserHost::Claude => USER_070_CLAUDE_MARKETPLACE_DIGEST,
                        UserHost::Antigravity => unreachable!(),
                    },
                    false,
                    "host-adapter-metadata",
                ),
            ]);
        }
        UserHost::Antigravity => {
            for (skill, digest) in LEGACY_ANTIGRAVITY_070_SKILLS {
                entries.push(historical_070_entry(
                    format!("{ANTIGRAVITY_SOURCE_RELATIVE}/skills/{skill}/SKILL.md"),
                    digest,
                    false,
                    "immutable-plugin-package",
                ));
                entries.push(historical_070_entry(
                    format!(".gemini/config/skills/{skill}/SKILL.md"),
                    digest,
                    false,
                    "host-skill-projection",
                ));
            }
            entries.push(historical_070_entry(
                format!("{ANTIGRAVITY_SOURCE_RELATIVE}/plugin.json"),
                LEGACY_ANTIGRAVITY_070_MANIFEST_DIGEST,
                false,
                "immutable-plugin-package",
            ));
        }
    }
    entries
}

fn historical_user_inventory(host: UserHost) -> AuthenticatedUserInventory {
    const HISTORICAL_VERSION: &str = "0.6.0";
    let guidance_path = match host {
        UserHost::Codex => ".codex/AGENTS.md",
        UserHost::Claude => ".claude/CLAUDE.md",
        UserHost::Antigravity => ".gemini/GEMINI.md",
    };
    let entries = ownership_entries(&historical_user_files(host));
    AuthenticatedUserInventory {
        product_version: HISTORICAL_VERSION.to_owned(),
        host,
        host_version_range: match host {
            UserHost::Antigravity => ">=2.3.1 <3.0.0",
            UserHost::Codex | UserHost::Claude => host.version_range(),
        }
        .to_owned(),
        source_release_digest: source_release_digest_from_entries(&entries),
        guidance_path: guidance_path.to_owned(),
        entries,
    }
}

fn legacy_antigravity_directory_scan_inventory() -> AuthenticatedUserInventory {
    const VERSION: &str = "0.7.0";
    let entry = |path: String, digest: &str, ownership: &str| UserOwnershipEntry {
        path,
        digest: digest.to_owned(),
        executable: false,
        unix_mode: installed_unix_mode(false),
        ownership: ownership.to_owned(),
    };
    let mut entries = vec![entry(
        ".gemini/config/plugins/aigent-hive/plugin.json".to_owned(),
        LEGACY_ANTIGRAVITY_070_MANIFEST_DIGEST,
        "immutable-plugin-package",
    )];
    for (skill, digest) in LEGACY_ANTIGRAVITY_070_SKILLS {
        entries.push(entry(
            format!(".gemini/config/plugins/aigent-hive/skills/{skill}/SKILL.md"),
            digest,
            "immutable-plugin-package",
        ));
        entries.push(entry(
            format!(".gemini/config/skills/{skill}/SKILL.md"),
            digest,
            "host-skill-projection",
        ));
    }
    for (path, ownership) in [
        (".gemini/GEMINI.md", "shared-marker"),
        (".hive/knowledge/Raw/README.md", "canonical-data-protected"),
        (
            ".hive/knowledge/Schema/schema.md",
            "canonical-data-protected",
        ),
        (".hive/knowledge/Wiki/index.md", "canonical-data-protected"),
        (".hive/knowledge/Wiki/log.md", "canonical-data-protected"),
        (
            ".hive/knowledge/suppression.yml",
            "canonical-data-protected",
        ),
    ] {
        entries.push(entry(
            path.to_owned(),
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            ownership,
        ));
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let source_release_digest = source_release_digest_from_entries(&entries);
    debug_assert_eq!(source_release_digest, LEGACY_ANTIGRAVITY_070_SOURCE_DIGEST);
    AuthenticatedUserInventory {
        product_version: VERSION.to_owned(),
        host: UserHost::Antigravity,
        host_version_range: ">=2.3.1 <3.0.0".to_owned(),
        source_release_digest,
        guidance_path: ".gemini/GEMINI.md".to_owned(),
        entries,
    }
}

fn historical_user_files(host: UserHost) -> BTreeMap<PathBuf, PlannedFile> {
    const HISTORICAL_PLUGIN_MANIFEST: &[u8] = b"{\"name\":\"aigent-hive\",\"version\":\"0.6.0\"}\n";
    const HISTORICAL_RETIRED_SKILL: &[u8] =
        b"#!/bin/sh\nprintf '%s\\n' 'retired Hive 0.6.0 skill'\n";
    let (plugin_path, retired_skill_path) = match host {
        UserHost::Codex => (
            ".hive/marketplaces/codex/plugins/aigent-hive/.codex-plugin/plugin.json",
            ".hive/marketplaces/codex/plugins/aigent-hive/bin/hive-retired-skill",
        ),
        UserHost::Claude => (
            ".hive/marketplaces/claude/plugins/aigent-hive/.claude-plugin/plugin.json",
            ".hive/marketplaces/claude/plugins/aigent-hive/bin/hive-retired-skill",
        ),
        UserHost::Antigravity => (
            ".gemini/config/plugins/aigent-hive/plugin.json",
            ".gemini/config/plugins/aigent-hive/bin/hive-retired-skill",
        ),
    };
    BTreeMap::from([
        (
            PathBuf::from(plugin_path),
            PlannedFile {
                bytes: HISTORICAL_PLUGIN_MANIFEST.to_vec(),
                executable: false,
                ownership: "immutable-plugin-package",
            },
        ),
        (
            PathBuf::from(retired_skill_path),
            PlannedFile {
                bytes: HISTORICAL_RETIRED_SKILL.to_vec(),
                executable: true,
                ownership: "immutable-plugin-package",
            },
        ),
        (
            PathBuf::from(".hive/historical-shared-marker.md"),
            PlannedFile {
                bytes: b"historical shared marker\n".to_vec(),
                executable: false,
                ownership: "shared-marker",
            },
        ),
        (
            PathBuf::from(".hive/knowledge/historical-protected.md"),
            PlannedFile {
                bytes: b"historical protected knowledge\n".to_vec(),
                executable: false,
                ownership: "canonical-data-protected",
            },
        ),
    ])
}

#[cfg(unix)]
#[allow(clippy::unnecessary_wraps)]
const fn installed_unix_mode(executable: bool) -> Option<u32> {
    Some(if executable { 0o755 } else { 0o644 })
}

#[cfg(not(unix))]
const fn installed_unix_mode(_executable: bool) -> Option<u32> {
    None
}

#[cfg(unix)]
const fn installed_file_permissions(executable: bool) -> FilePermissions {
    FilePermissions {
        executable,
        unix_mode: installed_unix_mode(executable),
    }
}

#[cfg(not(unix))]
const fn installed_file_permissions(_executable: bool) -> FilePermissions {
    FilePermissions {
        executable: true,
        unix_mode: None,
    }
}

fn changed_paths(
    current: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    root: &Dir,
    files: &BTreeMap<PathBuf, PlannedFile>,
    retired_files: &BTreeMap<PathBuf, RetiredFile>,
) -> Result<Vec<String>, InstallError> {
    let mut changed = Vec::new();
    for (relative, planned) in files {
        let bytes = current.get(relative).ok_or_else(|| {
            InstallError::Internal(format!(
                "planned snapshot is missing: {}",
                relative.display()
            ))
        })?;
        if bytes.as_deref() != Some(planned.bytes.as_slice())
            || (planned.executable && !is_executable(root, relative)?)
        {
            changed.push(portable(relative));
        }
    }
    changed.extend(retired_files.keys().map(|relative| portable(relative)));
    changed.sort();
    Ok(changed)
}

fn snapshot_operation_paths(
    root: &Dir,
    files: &BTreeMap<PathBuf, PlannedFile>,
    retired_files: &BTreeMap<PathBuf, RetiredFile>,
) -> Result<BTreeMap<PathBuf, Option<Vec<u8>>>, InstallError> {
    let mut paths = files
        .keys()
        .map(|relative| {
            read_optional_regular(root, relative, MAX_USER_FILE_BYTES)
                .map(|bytes| (relative.clone(), bytes))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    paths.extend(
        retired_files
            .iter()
            .map(|(relative, retired)| (relative.clone(), Some(retired.bytes.clone()))),
    );
    Ok(paths)
}

fn snapshot_operation_permissions(
    root: &Dir,
    paths: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    retired_files: &BTreeMap<PathBuf, RetiredFile>,
) -> Result<BTreeMap<PathBuf, Option<FilePermissions>>, InstallError> {
    #[cfg(unix)]
    let mut permissions = snapshot_permissions(root, paths)?;
    #[cfg(not(unix))]
    let permissions = snapshot_permissions(root, paths)?;
    #[cfg(unix)]
    permissions.extend(
        retired_files
            .iter()
            .map(|(relative, retired)| (relative.clone(), Some(retired.permissions))),
    );
    #[cfg(not(unix))]
    let _ = retired_files;
    Ok(permissions)
}

fn snapshot_permissions(
    root: &Dir,
    paths: &BTreeMap<PathBuf, Option<Vec<u8>>>,
) -> Result<BTreeMap<PathBuf, Option<FilePermissions>>, InstallError> {
    paths
        .iter()
        .map(|(relative, bytes)| {
            bytes
                .as_ref()
                .map(|_| file_permissions(root, relative))
                .transpose()
                .map(|permissions| (relative.clone(), permissions))
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
fn apply_plan(
    arguments: &UserArguments,
    plan: &mut UserPlan,
    codex_state_before: Option<CodexHostState>,
    claude_state_before: Option<ClaudeHostState>,
    antigravity_state_before: Option<AntigravityHostState>,
) -> Result<UserTransaction, InstallError> {
    let journal_relative = transaction_journal_relative(arguments.host);
    ensure_no_open_transaction(arguments, &journal_relative)?;
    let backup_relative = PathBuf::from(format!(
        ".hive/backups/user-install/{}/{}-{}",
        arguments.host.as_str(),
        unix_seconds()?,
        std::process::id()
    ));
    let snapshots = preflight_plan(&arguments.root_cap, plan)?;
    let index_existed = root_index_exists(&arguments.root_cap)?;
    let mut backup_entries = Vec::new();
    let mut activated_snapshots = Vec::new();
    for (relative, existing, permissions) in &snapshots {
        if let Some(bytes) = existing.as_ref() {
            let backup_path = backup_relative.join("files").join(relative);
            write_atomic_with_permissions(
                &arguments.root_cap,
                &backup_path,
                bytes,
                *permissions,
                None,
                None,
            )?;
        }
        backup_entries.push(UserBackupEntry {
            path: portable(relative),
            existed: existing.is_some(),
            digest: existing.as_ref().map(|bytes| sha256_digest(bytes)),
            installed_digest: plan
                .files
                .get(relative)
                .map(|planned| sha256_digest(&planned.bytes)),
            installed_executable: plan
                .files
                .get(relative)
                .map(|planned| installed_file_permissions(planned.executable).executable),
            installed_unix_mode: plan
                .files
                .get(relative)
                .and_then(|planned| installed_file_permissions(planned.executable).unix_mode),
            executable: permissions.executable,
            unix_mode: permissions.unix_mode,
        });
    }
    let mut backup = UserBackupManifest {
        schema_version: 2,
        host: arguments.host,
        plan_digest: plan.plan_digest.clone(),
        host_mutations: Vec::new(),
        index_existed,
        codex_state_before,
        claude_state_before,
        antigravity_state_before,
        host_owned_state: None,
        pending_host_transition: None,
        codex_plugin_was_latent_before_marketplace_add: false,
        entries: backup_entries,
    };
    persist_backup(arguments, &backup_relative, &backup)?;
    let journal = UserTransactionJournal {
        schema_version: 1,
        host: arguments.host,
        plan_digest: plan.plan_digest.clone(),
        backup: portable(&backup_relative),
    };
    write_atomic(
        &arguments.root_cap,
        &journal_relative,
        &json_line(&journal)?,
        false,
        None,
        None,
    )?;

    let backup_text = portable(&backup_relative);
    if let Some(manifest_file) = plan.files.get_mut(&plan.manifest_relative) {
        let mut manifest: UserOwnershipManifest = serde_json::from_slice(&manifest_file.bytes)
            .map_err(|error| {
                InstallError::Internal(format!("cannot decode planned ownership manifest: {error}"))
            })?;
        manifest.last_backup = Some(backup_text.clone());
        manifest_file.bytes = json_line(&manifest)?;
        if let Some(entry) = backup
            .entries
            .iter_mut()
            .find(|entry| entry.path == portable(&plan.manifest_relative))
        {
            entry.installed_digest = Some(sha256_digest(&manifest_file.bytes));
        }
        persist_backup(arguments, &backup_relative, &backup)?;
    }
    for (relative, existing, permissions) in &snapshots {
        let expected = existing.as_deref();
        let expected_permissions = existing.as_ref().map(|_| *permissions);
        let result = if let Some(file) = plan.files.get(relative) {
            write_atomic(
                &arguments.root_cap,
                relative,
                &file.bytes,
                file.executable,
                expected,
                expected_permissions,
            )
        } else if plan.retired_files.contains_key(relative) {
            remove_regular_if_exists(
                &arguments.root_cap,
                relative,
                expected,
                expected_permissions,
            )
        } else {
            Err(InstallError::Internal(format!(
                "planned snapshot has no activation operation: {}",
                relative.display()
            )))
        };
        if let Err(error) = result {
            let rollback_scope = if matches!(error, InstallError::Conflict(_)) {
                &activated_snapshots
            } else {
                &snapshots
            };
            if let Err(rollback) =
                rollback_snapshots(&arguments.root_cap, rollback_scope, &plan.files)
            {
                return Err(InstallError::Internal(format!(
                    "{}; filesystem rollback also failed: {}",
                    error.message(),
                    rollback.message()
                )));
            }
            if let Err(cleanup) = remove_transaction_journal(arguments, &journal_relative) {
                return Err(InstallError::Internal(format!(
                    "{}; rollback journal cleanup failed: {}",
                    error.message(),
                    cleanup.message()
                )));
            }
            return Err(error);
        }
        activated_snapshots.push((relative.clone(), existing.clone(), *permissions));
    }
    prune_retired_antigravity_source_directories(arguments, plan);
    Ok(UserTransaction {
        backup_relative,
        journal_relative,
        backup,
    })
}

fn prune_retired_antigravity_source_directories(arguments: &UserArguments, plan: &UserPlan) {
    if arguments.host != UserHost::Antigravity {
        return;
    }
    let source = Path::new(ANTIGRAVITY_SOURCE_RELATIVE);
    let mut directories = BTreeSet::new();
    for path in plan.retired_files.keys() {
        let Ok(relative) = path.strip_prefix(source) else {
            continue;
        };
        let mut parent = relative.parent();
        while let Some(directory) = parent.filter(|directory| !directory.as_os_str().is_empty()) {
            directories.insert(source.join(directory));
            parent = directory.parent();
        }
    }
    let mut directories = directories.into_iter().collect::<Vec<_>>();
    directories.sort_by(|left, right| {
        right
            .components()
            .count()
            .cmp(&left.components().count())
            .then_with(|| right.cmp(left))
    });
    for directory in directories {
        let _ = arguments.root_cap.remove_dir(&directory);
    }
}

fn preflight_plan(root: &Dir, plan: &UserPlan) -> Result<Vec<PlannedSnapshot>, InstallError> {
    let mut snapshots = Vec::with_capacity(plan.files.len() + plan.retired_files.len());
    for relative in plan.files.keys().chain(plan.retired_files.keys()) {
        validate_relative(relative)?;
        let existing = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)?;
        let permissions = existing
            .as_ref()
            .map(|_| file_permissions(root, relative))
            .transpose()?;
        if plan.expected_before.get(relative) != Some(&existing)
            || plan.expected_permissions.get(relative) != Some(&permissions)
        {
            return Err(InstallError::Conflict(format!(
                "user installation path changed after planning: {}",
                relative.display()
            )));
        }
        snapshots.push((relative.clone(), existing, permissions.unwrap_or_default()));
    }
    Ok(snapshots)
}

fn ensure_no_open_transaction(
    arguments: &UserArguments,
    journal_relative: &Path,
) -> Result<(), InstallError> {
    if read_optional_regular(&arguments.root_cap, journal_relative, MAX_USER_FILE_BYTES)?.is_some()
    {
        return Err(InstallError::Conflict(format!(
            "unfinished {} user installation requires --recover",
            arguments.host.as_str()
        )));
    }
    Ok(())
}

fn has_open_transaction(arguments: &UserArguments) -> Result<bool, InstallError> {
    read_optional_regular(
        &arguments.root_cap,
        &transaction_journal_relative(arguments.host),
        MAX_USER_FILE_BYTES,
    )
    .map(|journal| journal.is_some())
}

fn has_recoverable_dangling_codex_marketplace(
    arguments: &UserArguments,
) -> Result<bool, InstallError> {
    if arguments.host != UserHost::Codex || !has_open_transaction(arguments)? {
        return Ok(false);
    }
    let journal_relative = transaction_journal_relative(arguments.host);
    let Some(journal) = read_transaction_journal(arguments, &journal_relative)? else {
        return Ok(false);
    };
    let backup_relative = PathBuf::from(journal.backup);
    let backup_bytes = read_optional_regular(
        &arguments.root_cap,
        &backup_relative.join("manifest.json"),
        MAX_USER_FILE_BYTES,
    )?
    .ok_or_else(|| InstallError::Verification("user backup manifest is missing".to_owned()))?;
    let backup: UserBackupManifest = serde_json::from_slice(&backup_bytes)
        .map_err(|_| InstallError::Verification("user backup manifest is malformed".to_owned()))?;
    if journal.plan_digest != backup.plan_digest {
        return Err(InstallError::Verification(
            "user transaction journal does not match its backup".to_owned(),
        ));
    }
    is_recoverable_dangling_codex_marketplace(arguments, &backup)
}

fn rollback_snapshots(
    root: &Dir,
    snapshots: &[(PathBuf, Option<Vec<u8>>, FilePermissions)],
    installed: &BTreeMap<PathBuf, PlannedFile>,
) -> Result<(), InstallError> {
    for (relative, bytes, permissions) in snapshots.iter().rev() {
        let installed_file = installed.get(relative);
        let installed_bytes = installed_file.map(|file| file.bytes.as_slice());
        let installed_permissions = installed_file.map(|file| FilePermissions {
            executable: file.executable,
            unix_mode: installed_unix_mode(file.executable),
        });
        let current = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)?;
        let current_permissions = current
            .as_ref()
            .map(|_| file_permissions(root, relative))
            .transpose()?;
        let prior_permissions = bytes.as_ref().map(|_| *permissions);
        if current == *bytes && current_permissions == prior_permissions {
            continue;
        }
        if current.as_deref() != installed_bytes || current_permissions != installed_permissions {
            return Err(InstallError::Conflict(format!(
                "rollback preserved concurrently changed user path: {}",
                relative.display()
            )));
        }
        match bytes {
            Some(bytes) => {
                write_atomic_with_permissions(
                    root,
                    relative,
                    bytes,
                    *permissions,
                    installed_bytes,
                    installed_permissions,
                )?;
            }
            None => {
                remove_regular_if_exists(root, relative, installed_bytes, installed_permissions)?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn recover(
    arguments: &UserArguments,
    runner: &impl CommandRunner,
) -> Result<ActionResult, InstallError> {
    let manifest_relative =
        PathBuf::from(format!(".hive/install/{}.json", arguments.host.as_str()));
    let journal_relative = transaction_journal_relative(arguments.host);
    let journal = read_transaction_journal(arguments, &journal_relative)?;
    let backup_relative = if let Some(journal) = journal.as_ref() {
        PathBuf::from(&journal.backup)
    } else {
        let manifest =
            read_installed_manifest(&arguments.root_cap, &manifest_relative, arguments.host)?
                .ok_or_else(|| {
                    InstallError::Input("no user installation is available to recover".to_owned())
                })?;
        manifest
            .last_backup
            .as_deref()
            .ok_or_else(|| {
                InstallError::Input("user installation has no recoverable backup".to_owned())
            })
            .map(PathBuf::from)?
    };
    validate_relative(&backup_relative)?;
    let backup_bytes = read_optional_regular(
        &arguments.root_cap,
        &backup_relative.join("manifest.json"),
        MAX_USER_FILE_BYTES,
    )?
    .ok_or_else(|| InstallError::Verification("user backup manifest is missing".to_owned()))?;
    let mut backup: UserBackupManifest = serde_json::from_slice(&backup_bytes)
        .map_err(|_| InstallError::Verification("user backup manifest is malformed".to_owned()))?;
    if !(1..=2).contains(&backup.schema_version) || backup.host != arguments.host {
        return Err(InstallError::Verification(
            "user backup manifest binding is invalid".to_owned(),
        ));
    }
    if let Some(journal) = journal.as_ref() {
        if journal.plan_digest != backup.plan_digest {
            return Err(InstallError::Verification(
                "user transaction journal does not match its backup".to_owned(),
            ));
        }
    }
    validate_host_recovery_preconditions(arguments, &backup)?;
    let executable = qualify_recovery_host(arguments, &backup, runner)?;
    if journal.is_none()
        && (!backup.host_mutations.is_empty() || backup.pending_host_transition.is_some())
    {
        let recovery_journal = UserTransactionJournal {
            schema_version: 1,
            host: arguments.host,
            plan_digest: backup.plan_digest.clone(),
            backup: portable(&backup_relative),
        };
        write_atomic(
            &arguments.root_cap,
            &journal_relative,
            &json_line(&recovery_journal)?,
            false,
            None,
            None,
        )?;
    }
    if backup.pending_host_transition.is_some() {
        let executable = executable.as_ref().ok_or_else(|| {
            InstallError::Internal("qualified recovery host executable is missing".to_owned())
        })?;
        resolve_pending_host_transition(
            arguments,
            &backup_relative,
            &mut backup,
            executable,
            runner,
            true,
        )?;
    }
    let changed = restore_backup_files(arguments, &backup_relative, &backup)?;
    reconcile_root_index_after_rollback(arguments, backup.index_existed)?;
    compensate_host_mutations(
        arguments,
        &backup_relative,
        &mut backup,
        executable.as_ref(),
        runner,
        true,
    )?;
    let recovered_backup_bytes = read_optional_regular(
        &arguments.root_cap,
        &backup_relative.join("manifest.json"),
        MAX_USER_FILE_BYTES,
    )?
    .ok_or_else(|| {
        InstallError::Verification("user backup manifest disappeared during recovery".to_owned())
    })?;
    remove_transaction_journal(arguments, &journal_relative)?;
    let digest = sha256_digest(&recovered_backup_bytes);
    Ok(ActionResult {
        schema_version: 1,
        action: "RecoverHiveUser",
        status: "success",
        exit_code: 0,
        code: "hive.user-install-recovered",
        message: "user-scope Hive installation recovered from the latest backup".to_owned(),
        changed_paths: changed,
        evidence: vec![Evidence {
            kind: "file",
            locator: portable(&backup_relative.join("manifest.json")),
            digest,
        }],
        next_action: Some(format!(
            "run hive install --scope user --host {} --validate --output json",
            arguments.host.as_str()
        )),
        data: Some(json!({"host": arguments.host.as_str()})),
    })
}

fn validate_host_recovery_preconditions(
    arguments: &UserArguments,
    backup: &UserBackupManifest,
) -> Result<(), InstallError> {
    if backup.host_mutations.is_empty() && backup.pending_host_transition.is_none() {
        return Ok(());
    }
    match arguments.host {
        UserHost::Codex if backup.codex_state_before.is_some() => Ok(()),
        UserHost::Codex => Err(InstallError::Verification(
            "Codex transaction backup omitted the pre-mutation structured state".to_owned(),
        )),
        UserHost::Claude if backup.claude_state_before.is_some() => Ok(()),
        UserHost::Claude => Err(InstallError::Verification(
            "Claude transaction backup omitted the pre-mutation structured state".to_owned(),
        )),
        UserHost::Antigravity if backup.antigravity_state_before.is_some() => Ok(()),
        UserHost::Antigravity => Err(InstallError::Verification(
            "Antigravity transaction backup omitted the pre-mutation structured state".to_owned(),
        )),
    }
}

fn transaction_journal_relative(host: UserHost) -> PathBuf {
    PathBuf::from(format!(".hive/install-transactions/{}.json", host.as_str()))
}

fn persist_backup(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &UserBackupManifest,
) -> Result<(), InstallError> {
    let manifest_relative = backup_relative.join("manifest.json");
    let expected =
        read_optional_regular(&arguments.root_cap, &manifest_relative, MAX_USER_FILE_BYTES)?;
    let expected_permissions = expected
        .as_ref()
        .map(|_| file_permissions(&arguments.root_cap, &manifest_relative))
        .transpose()?;
    write_atomic(
        &arguments.root_cap,
        &manifest_relative,
        &json_line(backup)?,
        false,
        expected.as_deref(),
        expected_permissions,
    )
}

fn read_transaction_journal(
    arguments: &UserArguments,
    journal_relative: &Path,
) -> Result<Option<UserTransactionJournal>, InstallError> {
    let Some(bytes) =
        read_optional_regular(&arguments.root_cap, journal_relative, MAX_USER_FILE_BYTES)?
    else {
        return Ok(None);
    };
    let journal: UserTransactionJournal = serde_json::from_slice(&bytes).map_err(|_| {
        InstallError::Verification("user transaction journal is malformed".to_owned())
    })?;
    let backup = PathBuf::from(&journal.backup);
    if journal.schema_version != 1 || journal.host != arguments.host {
        return Err(InstallError::Verification(
            "user transaction journal binding is invalid".to_owned(),
        ));
    }
    validate_relative(&backup)?;
    Ok(Some(journal))
}

fn restore_backup_files(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &UserBackupManifest,
) -> Result<Vec<String>, InstallError> {
    let mut changed = Vec::new();
    for entry in backup.entries.iter().rev() {
        let relative = PathBuf::from(&entry.path);
        validate_relative(&relative)?;
        reconcile_retained_claim(&arguments.root_cap, &relative, entry)?;
        if entry.existed {
            let bytes = read_optional_regular(
                &arguments.root_cap,
                &backup_relative.join("files").join(&relative),
                MAX_USER_FILE_BYTES,
            )?
            .ok_or_else(|| {
                InstallError::Verification(format!("backup bytes are missing for {}", entry.path))
            })?;
            if entry.digest.as_deref() != Some(sha256_digest(&bytes).as_str()) {
                return Err(InstallError::Verification(format!(
                    "backup digest mismatch for {}",
                    entry.path
                )));
            }
            let expected = current_installed_state(&arguments.root_cap, &relative, entry)?;
            write_atomic_with_permissions(
                &arguments.root_cap,
                &relative,
                &bytes,
                FilePermissions {
                    executable: entry.executable,
                    unix_mode: entry.unix_mode,
                },
                expected.bytes.as_deref(),
                expected.permissions,
            )?;
        } else {
            let expected = current_installed_state(&arguments.root_cap, &relative, entry)?;
            remove_regular_if_exists(
                &arguments.root_cap,
                &relative,
                expected.bytes.as_deref(),
                expected.permissions,
            )?;
        }
        changed.push(entry.path.clone());
    }
    Ok(changed)
}

#[allow(clippy::too_many_lines)]
fn reconcile_retained_claim(
    root: &Dir,
    relative: &Path,
    entry: &UserBackupEntry,
) -> Result<(), InstallError> {
    let Some((parent, destination)) = capability_parent(root, relative, false)? else {
        return Ok(());
    };
    let claim_name = claim_name(relative);
    let claim = match parent.open_dir_nofollow(&claim_name) {
        Ok(claim) => claim,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(InstallError::Verification(format!(
                "cannot open retained user installation claim at {}: {error}",
                recovery_locator(relative).display()
            )))
        }
    };
    let mut names = claim
        .entries()
        .map_err(|error| retained_claim_error(relative, "enumerate retained claim", error))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name())
                .map_err(|error| retained_claim_error(relative, "read retained claim entry", error))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    if names.iter().any(|name| {
        name != OsStr::new("claimed.bin")
            && name != OsStr::new("replacement.bin")
            && name != OsStr::new("published.bin")
    }) {
        return Err(InstallError::Conflict(format!(
            "retained user installation claim contains foreign entries at {}",
            recovery_locator(relative).display()
        )));
    }
    let prior_permissions = prior_backup_permissions(entry)?;
    let installed_permissions = installed_backup_permissions(entry)?;
    if names.iter().any(|name| name == OsStr::new("published.bin")) {
        validate_retained_object(
            &claim,
            Path::new("published.bin"),
            entry.installed_digest.as_deref(),
            installed_permissions,
            relative,
            "published",
        )?;
        return Err(InstallError::Conflict(format!(
            "retained user installation claim has an unresolved published object at {}",
            recovery_locator(relative).display()
        )));
    }
    if names
        .iter()
        .any(|name| name == OsStr::new("replacement.bin"))
    {
        validate_retained_object(
            &claim,
            Path::new("replacement.bin"),
            entry.installed_digest.as_deref(),
            installed_permissions,
            relative,
            "replacement",
        )?;
    }
    let current = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)?;
    let current_digest = current.as_deref().map(sha256_digest);
    let current_permissions = current
        .as_ref()
        .map(|_| file_permissions(root, relative))
        .transpose()?;
    if entry.existed {
        let claimed = read_optional_regular(&claim, Path::new("claimed.bin"), MAX_USER_FILE_BYTES)?
            .ok_or_else(|| {
                InstallError::Verification(format!(
                    "retained user installation claim omitted prior bytes at {}",
                    recovery_locator(relative).display()
                ))
            })?;
        let claimed_permissions = file_permissions(&claim, Path::new("claimed.bin"))?;
        if entry.digest.as_deref() != Some(sha256_digest(&claimed).as_str())
            || Some(claimed_permissions) != prior_permissions
        {
            return Err(InstallError::Verification(format!(
                "retained user installation prior byte or mode mismatch at {}",
                recovery_locator(relative).display()
            )));
        }
        if current.is_none() {
            claim
                .hard_link("claimed.bin", &parent, &destination)
                .map_err(|error| retained_claim_error(relative, "restore prior bytes", error))?;
        } else if current_digest == entry.installed_digest
            && current_permissions == installed_permissions
        {
            parent
                .rename(&destination, &claim, OsStr::new("published.bin"))
                .map_err(|error| retained_claim_error(relative, "claim published bytes", error))?;
            let published =
                read_optional_regular(&claim, Path::new("published.bin"), MAX_USER_FILE_BYTES)?
                    .ok_or_else(|| {
                        retained_claim_error(relative, "verify published bytes", "missing")
                    })?;
            let published_permissions = file_permissions(&claim, Path::new("published.bin"))?;
            if Some(sha256_digest(&published)) != entry.installed_digest
                || Some(published_permissions) != installed_permissions
            {
                return Err(retained_claim_error(
                    relative,
                    "verify published object",
                    "bytes or permissions changed",
                ));
            }
            claim
                .hard_link("claimed.bin", &parent, &destination)
                .map_err(|error| retained_claim_error(relative, "restore prior bytes", error))?;
            claim
                .remove_file("published.bin")
                .map_err(|error| retained_claim_error(relative, "clean published bytes", error))?;
        } else if current_digest != entry.digest || current_permissions != prior_permissions {
            return Err(InstallError::Conflict(format!(
                "recovery preserved a foreign destination beside retained claim: {}",
                relative.display()
            )));
        }
        claim
            .remove_file("claimed.bin")
            .map_err(|error| retained_claim_error(relative, "clean retained prior bytes", error))?;
    } else {
        if names.iter().any(|name| name == OsStr::new("claimed.bin")) {
            return Err(InstallError::Verification(format!(
                "creation claim unexpectedly retained prior bytes at {}",
                recovery_locator(relative).display()
            )));
        }
        if current_digest == entry.installed_digest && current_permissions == installed_permissions
        {
            parent
                .rename(&destination, &claim, OsStr::new("published.bin"))
                .map_err(|error| retained_claim_error(relative, "claim created bytes", error))?;
            let published =
                read_optional_regular(&claim, Path::new("published.bin"), MAX_USER_FILE_BYTES)?
                    .ok_or_else(|| {
                        retained_claim_error(relative, "verify created bytes", "missing")
                    })?;
            let published_permissions = file_permissions(&claim, Path::new("published.bin"))?;
            if Some(sha256_digest(&published)) != entry.installed_digest
                || Some(published_permissions) != installed_permissions
            {
                return Err(retained_claim_error(
                    relative,
                    "verify created object",
                    "bytes or permissions changed",
                ));
            }
            claim
                .remove_file("published.bin")
                .map_err(|error| retained_claim_error(relative, "remove created bytes", error))?;
        } else if current.is_some() {
            return Err(InstallError::Conflict(format!(
                "recovery preserved a foreign destination beside creation claim: {}",
                relative.display()
            )));
        }
    }
    if names
        .iter()
        .any(|name| name == OsStr::new("replacement.bin"))
    {
        let replacement =
            read_optional_regular(&claim, Path::new("replacement.bin"), MAX_USER_FILE_BYTES)?
                .ok_or_else(|| {
                    retained_claim_error(relative, "verify staged replacement", "missing")
                })?;
        let replacement_permissions = file_permissions(&claim, Path::new("replacement.bin"))?;
        if Some(sha256_digest(&replacement)) != entry.installed_digest
            || Some(replacement_permissions) != installed_permissions
        {
            return Err(retained_claim_error(
                relative,
                "verify staged replacement",
                "bytes or permissions changed",
            ));
        }
        claim
            .remove_file("replacement.bin")
            .map_err(|error| retained_claim_error(relative, "clean staged replacement", error))?;
    }
    drop(claim);
    parent
        .remove_dir(&claim_name)
        .map_err(|error| retained_claim_error(relative, "clean retained claim directory", error))
}

fn validate_retained_object(
    claim: &Dir,
    name: &Path,
    expected_digest: Option<&str>,
    expected_permissions: Option<FilePermissions>,
    relative: &Path,
    label: &str,
) -> Result<(), InstallError> {
    let bytes = read_optional_regular(claim, name, MAX_USER_FILE_BYTES)?.ok_or_else(|| {
        retained_claim_error(
            relative,
            &format!("verify retained {label} object"),
            "missing",
        )
    })?;
    let permissions = file_permissions(claim, name)?;
    if Some(sha256_digest(&bytes).as_str()) != expected_digest
        || Some(permissions) != expected_permissions
    {
        return Err(retained_claim_error(
            relative,
            &format!("verify retained {label} object"),
            "bytes or permissions changed",
        ));
    }
    Ok(())
}

struct AuthorizedFileState {
    bytes: Option<Vec<u8>>,
    permissions: Option<FilePermissions>,
}

fn current_installed_state(
    root: &Dir,
    relative: &Path,
    entry: &UserBackupEntry,
) -> Result<AuthorizedFileState, InstallError> {
    let current = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)?;
    let permissions = current
        .as_ref()
        .map(|_| file_permissions(root, relative))
        .transpose()?;
    let digest = current.as_deref().map(sha256_digest);
    let prior_permissions = prior_backup_permissions(entry)?;
    let installed_permissions = installed_backup_permissions(entry)?;
    let prior_matches = digest == entry.digest && permissions == prior_permissions;
    let installed_matches =
        digest == entry.installed_digest && permissions == installed_permissions;
    if prior_matches || installed_matches {
        return Ok(AuthorizedFileState {
            bytes: current,
            permissions,
        });
    }
    Err(InstallError::Conflict(format!(
        "recovery preserved concurrently changed user bytes or permissions: {}",
        relative.display()
    )))
}

#[cfg_attr(not(unix), allow(clippy::unnecessary_wraps))]
fn prior_backup_permissions(
    entry: &UserBackupEntry,
) -> Result<Option<FilePermissions>, InstallError> {
    if !entry.existed {
        return Ok(None);
    }
    #[cfg(unix)]
    if entry.unix_mode.is_none() {
        return Err(InstallError::Verification(format!(
            "backup lacks prior Unix mode authority for {}",
            entry.path
        )));
    }
    Ok(Some(FilePermissions {
        executable: entry.executable,
        unix_mode: entry.unix_mode,
    }))
}

fn installed_backup_permissions(
    entry: &UserBackupEntry,
) -> Result<Option<FilePermissions>, InstallError> {
    if entry.installed_digest.is_none() {
        if entry.installed_executable.is_some() || entry.installed_unix_mode.is_some() {
            return Err(InstallError::Verification(format!(
                "backup has permission authority without installed bytes for {}",
                entry.path
            )));
        }
        return Ok(None);
    }
    let executable = entry.installed_executable.ok_or_else(|| {
        InstallError::Verification(format!(
            "backup lacks installed permission authority for {}",
            entry.path
        ))
    })?;
    #[cfg(unix)]
    if entry.installed_unix_mode.is_none() {
        return Err(InstallError::Verification(format!(
            "backup lacks installed Unix mode authority for {}",
            entry.path
        )));
    }
    Ok(Some(FilePermissions {
        executable,
        unix_mode: entry.installed_unix_mode,
    }))
}

fn remove_regular_if_exists(
    root: &Dir,
    relative: &Path,
    expected: Option<&[u8]>,
    expected_permissions: Option<FilePermissions>,
) -> Result<(), InstallError> {
    cas_activate(
        root,
        relative,
        expected.map(|bytes| ExpectedFile {
            bytes,
            permissions: expected_permissions.expect("existing file requires permission token"),
        }),
        None,
        FilePermissions::default(),
    )
}

fn root_index_exists(root: &Dir) -> Result<bool, InstallError> {
    match capability_metadata(root, Path::new(ROOT_INDEX_RELATIVE))? {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(InstallError::Conflict(format!(
                "root knowledge index must be a regular file: {ROOT_INDEX_RELATIVE}"
            )))
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(InstallError::Internal(format!(
            "cannot inspect root knowledge index: {error}"
        ))),
    }
}

fn remove_disposable_root_index(root: &Dir) -> Result<(), InstallError> {
    for relative in [
        ROOT_INDEX_RELATIVE,
        ".hive/index/hive.sqlite3-wal",
        ".hive/index/hive.sqlite3-shm",
        ".hive/index/hive.sqlite3-journal",
        hive_wiki::store::RAG_MANIFEST_RELATIVE,
        hive_wiki::store::RAG_TRUST_RELATIVE,
        hive_wiki::store::RAG_DIRTY_RELATIVE,
        ".hive/index/.stale",
    ] {
        let maximum = match relative {
            ROOT_INDEX_RELATIVE
            | ".hive/index/hive.sqlite3-wal"
            | ".hive/index/hive.sqlite3-shm"
            | ".hive/index/hive.sqlite3-journal" => MAX_DERIVED_INDEX_BYTES,
            _ => MAX_USER_FILE_BYTES,
        };
        let expected = read_optional_regular(root, Path::new(relative), maximum)?;
        let expected_permissions = expected
            .as_ref()
            .map(|_| file_permissions(root, Path::new(relative)))
            .transpose()?;
        remove_regular_if_exists(
            root,
            Path::new(relative),
            expected.as_deref(),
            expected_permissions,
        )?;
    }
    Ok(())
}

fn reconcile_root_index_after_rollback(
    arguments: &UserArguments,
    index_existed: bool,
) -> Result<(), InstallError> {
    if !index_existed {
        return remove_disposable_root_index(&arguments.root_cap);
    }
    hive_wiki::shared::ensure_project_registry(&arguments.user_root)
        .and_then(|_| hive_wiki::shared::rebuild_shared_index(&arguments.user_root))
        .map(|_| ())
        .map_err(|error| {
            InstallError::Verification(format!(
                "root knowledge index reconstruction failed during rollback; existing index was preserved: {error}"
            ))
        })
}

fn qualify_recovery_host(
    arguments: &UserArguments,
    backup: &UserBackupManifest,
    runner: &impl CommandRunner,
) -> Result<Option<QualifiedExecutable>, InstallError> {
    if backup.host_mutations.is_empty() && backup.pending_host_transition.is_none() {
        return Ok(None);
    }
    let program = match arguments.host {
        UserHost::Codex => "codex",
        UserHost::Claude => "claude",
        UserHost::Antigravity => "agy",
    };
    let executable = runner.qualify(program).map_err(|error| {
        InstallError::Unsupported(format!(
            "cannot compensate {} host state because {program} is unavailable: {error}",
            arguments.host.as_str()
        ))
    })?;
    probe_supported_host_version(arguments.host, &executable, runner)?;
    Ok(Some(executable))
}

fn codex_compensation_command(mutation: HostMutation) -> &'static [&'static str] {
    match mutation {
        HostMutation::CodexMarketplaceAdded => {
            &["plugin", "marketplace", "remove", "aigent-hive", "--json"]
        }
        HostMutation::CodexPluginAdded => {
            &["plugin", "remove", "aigent-hive@aigent-hive", "--json"]
        }
        HostMutation::CodexPluginRefreshed => {
            &["plugin", "add", "aigent-hive@aigent-hive", "--json"]
        }
        HostMutation::ClaudeMarketplaceAdded
        | HostMutation::ClaudePluginInstalled
        | HostMutation::ClaudeMarketplaceRefreshed
        | HostMutation::ClaudePluginRefreshed
        | HostMutation::AntigravityPluginInstalled
        | HostMutation::AntigravityPluginRefreshed => unreachable!("Codex mutation required"),
    }
}

fn probe_codex_state_if_required(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<Option<CodexHostState>, InstallError> {
    if arguments.host != UserHost::Codex {
        return Ok(None);
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Codex executable is missing".to_owned())
    })?;
    probe_codex_state(executable, runner).map(Some)
}

fn probe_codex_state(
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<CodexHostState, InstallError> {
    let marketplace_command = ["plugin", "marketplace", "list", "--json"];
    let plugin_command = ["plugin", "list", "--json"];
    let marketplaces = run_codex_probe(executable, &marketplace_command, runner)?;
    let plugins = run_codex_probe(executable, &plugin_command, runner)?;
    Ok(CodexHostState {
        marketplace: parse_codex_marketplace_state(&marketplaces)?,
        plugin: parse_codex_plugin_state(&plugins)?,
    })
}

fn run_codex_probe(
    executable: &QualifiedExecutable,
    command: &[&str],
    runner: &impl CommandRunner,
) -> Result<Vec<u8>, InstallError> {
    let output = runner
        .run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "Codex structured state probe `{}` failed: {error}",
                command.join(" ")
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "Codex structured state probe exited unsuccessfully: {}",
            sanitized_command_diagnostic(command, &output.stdout)
        )));
    }
    Ok(output.stdout)
}

fn probe_claude_state_if_required(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<Option<ClaudeHostState>, InstallError> {
    if arguments.host != UserHost::Claude {
        return Ok(None);
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Claude executable is missing".to_owned())
    })?;
    probe_claude_state(executable, runner).map(Some)
}

fn probe_claude_state(
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<ClaudeHostState, InstallError> {
    let marketplace_command = ["plugin", "marketplace", "list", "--json"];
    let plugin_command = ["plugin", "list", "--json"];
    let marketplaces = run_claude_probe(executable, &marketplace_command, runner)?;
    let plugins = run_claude_probe(executable, &plugin_command, runner)?;
    Ok(ClaudeHostState {
        marketplace: parse_claude_marketplace_state(&marketplaces)?,
        plugin: parse_claude_plugin_state(&plugins)?,
    })
}

fn probe_antigravity_state_if_required(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<Option<AntigravityHostState>, InstallError> {
    if arguments.host != UserHost::Antigravity {
        return Ok(None);
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Antigravity executable is missing".to_owned())
    })?;
    probe_antigravity_state(executable, runner).map(Some)
}

fn probe_antigravity_state(
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<AntigravityHostState, InstallError> {
    let command = ["plugin", "list"];
    let output = runner
        .run(executable, &command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "Antigravity structured state probe `{}` failed: {error}",
                command.join(" ")
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "Antigravity structured state probe exited unsuccessfully: {}",
            sanitized_command_diagnostic(&command, &output.stdout)
        )));
    }
    parse_antigravity_plugin_state(&output.stdout).map(|plugin| AntigravityHostState { plugin })
}

fn parse_antigravity_plugin_state(
    bytes: &[u8],
) -> Result<Option<AntigravityPluginState>, InstallError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        InstallError::Verification(
            "Antigravity plugin state probe returned non-UTF-8 output".to_owned(),
        )
    })?;
    if text.trim() == "No imported plugins." {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| {
        InstallError::Verification(
            "Antigravity plugin state probe returned malformed structured output".to_owned(),
        )
    })?;
    let entries = value
        .get("imports")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            InstallError::Verification(
                "Antigravity plugin state probe omitted imports[]".to_owned(),
            )
        })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("name").and_then(serde_json::Value::as_str) == Some("aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Antigravity plugin state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let source = entry.get("source").and_then(serde_json::Value::as_str);
            let imported_at = entry.get("importedAt").and_then(serde_json::Value::as_str);
            let components = entry
                .get("components")
                .and_then(serde_json::Value::as_array)
                .and_then(|components| {
                    components
                        .iter()
                        .map(serde_json::Value::as_str)
                        .map(|component| component.map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                });
            match (source, imported_at, components) {
                (Some(source), Some(_), Some(components)) => Ok(AntigravityPluginState {
                    source: source.to_owned(),
                    components,
                }),
                _ => Err(InstallError::Verification(
                    "Antigravity aigent-hive plugin state omitted required fields".to_owned(),
                )),
            }
        })
        .transpose()
}

fn expected_antigravity_plugin_state() -> AntigravityPluginState {
    AntigravityPluginState {
        source: "antigravity".to_owned(),
        components: vec!["skills".to_owned()],
    }
}

fn run_claude_probe(
    executable: &QualifiedExecutable,
    command: &[&str],
    runner: &impl CommandRunner,
) -> Result<Vec<u8>, InstallError> {
    let output = runner
        .run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "Claude structured state probe `{}` failed: {error}",
                command.join(" ")
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "Claude structured state probe exited unsuccessfully: {}",
            sanitized_command_diagnostic(command, &output.stdout)
        )));
    }
    Ok(output.stdout)
}

fn parse_claude_marketplace_state(
    bytes: &[u8],
) -> Result<Option<ClaudeMarketplaceState>, InstallError> {
    let entries: Vec<serde_json::Value> = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification(
            "Claude marketplace state probe returned malformed JSON".to_owned(),
        )
    })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("name").and_then(serde_json::Value::as_str) == Some("aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Claude marketplace state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let source = entry.get("source").and_then(serde_json::Value::as_str);
            let path = entry.get("path").and_then(serde_json::Value::as_str);
            match (source, path) {
                (Some(source), Some(path)) => Ok(ClaudeMarketplaceState {
                    source: source.to_owned(),
                    path: normalize_host_path(path),
                }),
                _ => Err(InstallError::Verification(
                    "Claude aigent-hive marketplace state omitted source or path".to_owned(),
                )),
            }
        })
        .transpose()
}

fn parse_claude_plugin_state(bytes: &[u8]) -> Result<Option<ClaudePluginState>, InstallError> {
    let entries: Vec<serde_json::Value> = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification("Claude plugin state probe returned malformed JSON".to_owned())
    })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("id").and_then(serde_json::Value::as_str) == Some("aigent-hive@aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Claude plugin state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let version = entry.get("version").and_then(serde_json::Value::as_str);
            let enabled = entry.get("enabled").and_then(serde_json::Value::as_bool);
            let scope = entry.get("scope").and_then(serde_json::Value::as_str);
            match (version, enabled, scope) {
                (Some(version), Some(enabled), Some(scope)) => Ok(ClaudePluginState {
                    version: version.to_owned(),
                    enabled,
                    scope: scope.to_owned(),
                }),
                _ => Err(InstallError::Verification(
                    "Claude aigent-hive plugin state omitted version, enabled, or scope".to_owned(),
                )),
            }
        })
        .transpose()
}

fn parse_codex_marketplace_state(
    bytes: &[u8],
) -> Result<Option<CodexMarketplaceState>, InstallError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification(
            "Codex marketplace state probe returned malformed JSON".to_owned(),
        )
    })?;
    let entries = value
        .get("marketplaces")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            InstallError::Verification(
                "Codex marketplace state probe omitted marketplaces[]".to_owned(),
            )
        })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("name").and_then(serde_json::Value::as_str) == Some("aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Codex marketplace state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            entry
                .get("root")
                .and_then(serde_json::Value::as_str)
                .map(|root| CodexMarketplaceState {
                    root: normalize_host_path(root),
                })
                .ok_or_else(|| {
                    InstallError::Verification(
                        "Codex aigent-hive marketplace state omitted root".to_owned(),
                    )
                })
        })
        .transpose()
}

fn parse_codex_plugin_state(bytes: &[u8]) -> Result<Option<CodexPluginState>, InstallError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification("Codex plugin state probe returned malformed JSON".to_owned())
    })?;
    let entries = value
        .get("installed")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            InstallError::Verification("Codex plugin state probe omitted installed[]".to_owned())
        })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("pluginId").and_then(serde_json::Value::as_str) == Some("aigent-hive@aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Codex plugin state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let version = entry.get("version").and_then(serde_json::Value::as_str);
            let enabled = entry.get("enabled").and_then(serde_json::Value::as_bool);
            let source_path = entry
                .get("source")
                .and_then(|source| source.get("path"))
                .and_then(serde_json::Value::as_str);
            let marketplace_source = entry
                .get("marketplaceSource")
                .and_then(|source| source.get("source"))
                .and_then(serde_json::Value::as_str);
            match (version, enabled, source_path, marketplace_source) {
                (Some(version), Some(enabled), Some(source_path), Some(marketplace_source)) => {
                    Ok(CodexPluginState {
                        version: version.to_owned(),
                        enabled,
                        source_path: normalize_host_path(source_path),
                        marketplace_source: normalize_host_path(marketplace_source),
                    })
                }
                _ => Err(InstallError::Verification(
                    "Codex aigent-hive plugin state omitted required fields".to_owned(),
                )),
            }
        })
        .transpose()
}

fn expected_codex_marketplace_root(arguments: &UserArguments) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(".hive")
        .join("marketplaces")
        .join("codex")
        .to_str()
        .map(normalize_host_path)
        .ok_or_else(|| {
            InstallError::Unsupported("Codex marketplace path is not valid UTF-8".to_owned())
        })
}

fn expected_codex_plugin_source_path(arguments: &UserArguments) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(".hive")
        .join("marketplaces")
        .join("codex")
        .join("plugins")
        .join("aigent-hive")
        .to_str()
        .map(normalize_host_path)
        .ok_or_else(|| {
            InstallError::Unsupported("Codex plugin source path is not valid UTF-8".to_owned())
        })
}

fn expected_claude_marketplace_path(arguments: &UserArguments) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(".hive")
        .join("marketplaces")
        .join("claude")
        .to_str()
        .map(normalize_host_path)
        .ok_or_else(|| {
            InstallError::Unsupported("Claude marketplace path is not valid UTF-8".to_owned())
        })
}

fn normalize_host_path(path: &str) -> String {
    #[cfg(windows)]
    {
        let normalized = path.replace('/', "\\");
        if let Some(rest) = normalized.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{rest}")
        } else if let Some(rest) = normalized.strip_prefix(r"\\?\") {
            rest.to_owned()
        } else {
            normalized
        }
    }
    #[cfg(not(windows))]
    {
        path.to_owned()
    }
}

fn expected_antigravity_source_path(arguments: &UserArguments) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(ANTIGRAVITY_SOURCE_RELATIVE)
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| {
            InstallError::Unsupported(
                "Antigravity plugin source path is not valid UTF-8".to_owned(),
            )
        })
}

fn validate_codex_prestate(
    arguments: &UserArguments,
    state: Option<&CodexHostState>,
) -> Result<(), InstallError> {
    let Some(state) = state else {
        return Ok(());
    };
    let expected_marketplace = expected_codex_marketplace_root(arguments)?;
    if state.plugin.is_some() && state.marketplace.is_none() {
        return Err(InstallError::Conflict(
            "Codex aigent-hive plugin exists without its structured marketplace state".to_owned(),
        ));
    }
    if state
        .marketplace
        .as_ref()
        .is_some_and(|marketplace| marketplace.root != expected_marketplace)
    {
        return Err(InstallError::Conflict(
            "Codex aigent-hive marketplace is bound to an unexpected root".to_owned(),
        ));
    }
    if state
        .plugin
        .as_ref()
        .is_some_and(|plugin| plugin.marketplace_source != expected_marketplace || !plugin.enabled)
    {
        return Err(InstallError::Conflict(
            "Codex aigent-hive plugin source or enabled state cannot be restored exactly"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_codex_activation(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Codex {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Codex executable is missing".to_owned())
    })?;
    let state = probe_codex_state(executable, runner)?;
    let marketplace_root = expected_codex_marketplace_root(arguments)?;
    let plugin_source = expected_codex_plugin_source_path(arguments)?;
    let marketplace_valid = state
        .marketplace
        .as_ref()
        .is_some_and(|marketplace| marketplace.root == marketplace_root);
    let plugin_valid = state.plugin.as_ref().is_some_and(|plugin| {
        plugin.version == env!("CARGO_PKG_VERSION")
            && plugin.enabled
            && plugin.source_path == plugin_source
            && plugin.marketplace_source == marketplace_root
    });
    if marketplace_valid && plugin_valid {
        Ok(())
    } else {
        Err(InstallError::Verification(
            "Codex structured state probe did not confirm activated aigent-hive state".to_owned(),
        ))
    }
}

fn validate_claude_prestate(
    arguments: &UserArguments,
    state: Option<&ClaudeHostState>,
) -> Result<(), InstallError> {
    let Some(state) = state else {
        return Ok(());
    };
    let expected_path = expected_claude_marketplace_path(arguments)?;
    if state.plugin.is_some() && state.marketplace.is_none() {
        return Err(InstallError::Conflict(
            "Claude aigent-hive plugin exists without its structured marketplace state".to_owned(),
        ));
    }
    if state.marketplace.as_ref().is_some_and(|marketplace| {
        marketplace.source != "directory" || marketplace.path != expected_path
    }) {
        return Err(InstallError::Conflict(
            "Claude aigent-hive marketplace is bound to an unexpected source".to_owned(),
        ));
    }
    if state
        .plugin
        .as_ref()
        .is_some_and(|plugin| plugin.scope != "user")
    {
        return Err(InstallError::Conflict(
            "Claude aigent-hive plugin is not installed at user scope".to_owned(),
        ));
    }
    Ok(())
}

fn validate_claude_activation(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Claude {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Claude executable is missing".to_owned())
    })?;
    let state = probe_claude_state(executable, runner)?;
    let expected_path = expected_claude_marketplace_path(arguments)?;
    let marketplace_valid = state.marketplace.as_ref().is_some_and(|marketplace| {
        marketplace.source == "directory" && marketplace.path == expected_path
    });
    let plugin_valid = state.plugin.as_ref().is_some_and(|plugin| {
        plugin.version == env!("CARGO_PKG_VERSION") && plugin.enabled && plugin.scope == "user"
    });
    if marketplace_valid && plugin_valid {
        Ok(())
    } else {
        Err(InstallError::Verification(
            "Claude structured state probe did not confirm activated aigent-hive state".to_owned(),
        ))
    }
}

fn validate_antigravity_prestate(
    arguments: &UserArguments,
    plan: &UserPlan,
    state: Option<&AntigravityHostState>,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Antigravity {
        return Ok(());
    }
    let plugin = state.and_then(|state| state.plugin.as_ref());
    if plugin.is_some_and(|plugin| plugin != &expected_antigravity_plugin_state()) {
        return Err(InstallError::Conflict(
            "Antigravity aigent-hive plugin has an unsupported native registration shape"
                .to_owned(),
        ));
    }
    let observed_stage =
        read_optional_regular_tree(&arguments.root_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))?;
    match (&plan.expected_antigravity_stage, &observed_stage) {
        (None, None) if plugin.is_none() => Ok(()),
        (None, _) => Err(InstallError::Conflict(
            "Antigravity aigent-hive namespace is occupied without authenticated Hive ownership"
                .to_owned(),
        )),
        (Some(expected), Some(observed)) if expected == observed => {
            if plugin.is_some() && !plan.prior_antigravity_activation_source {
                return Err(InstallError::Conflict(
                    "Antigravity aigent-hive plugin is registered without an authenticated Hive source bundle"
                        .to_owned(),
                ));
            }
            Ok(())
        }
        (Some(expected), observed)
            if plugin.is_none()
                && observed.as_ref().is_none_or(|observed| {
                    observed.files.is_empty()
                        && observed.directories.is_subset(&expected.directories)
                }) =>
        {
            Ok(())
        }
        (Some(_), None) => Err(InstallError::Conflict(
            "Antigravity aigent-hive plugin is registered without its authenticated host stage"
                .to_owned(),
        )),
        (Some(_), Some(_)) => Err(InstallError::Conflict(
            "Antigravity aigent-hive host stage differs from its authenticated prior package"
                .to_owned(),
        )),
    }
}

fn validate_installed_host(
    arguments: &UserArguments,
    plan: &UserPlan,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    match arguments.host {
        UserHost::Codex => validate_codex_activation(arguments, Some(executable), runner),
        UserHost::Claude => validate_claude_activation(arguments, Some(executable), runner),
        UserHost::Antigravity => {
            validate_antigravity_activation(arguments, plan, Some(executable), runner)
        }
    }
}

fn validate_antigravity_activation(
    arguments: &UserArguments,
    plan: &UserPlan,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Antigravity {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Antigravity executable is missing".to_owned())
    })?;
    let state = probe_antigravity_state(executable, runner)?;
    if state.plugin.as_ref() != Some(&expected_antigravity_plugin_state()) {
        return Err(InstallError::Verification(
            "Antigravity structured state probe did not confirm activated aigent-hive state"
                .to_owned(),
        ));
    }
    validate_antigravity_stage(arguments, plan)
}

fn planned_antigravity_stage(plan: &UserPlan) -> RegularTree {
    let source = Path::new(ANTIGRAVITY_SOURCE_RELATIVE);
    let mut expected = RegularTree::default();
    for (path, planned) in &plan.files {
        let Ok(relative) = path.strip_prefix(source) else {
            continue;
        };
        insert_regular_tree_file(&mut expected, relative, planned.bytes.clone());
    }
    expected
}

fn validate_antigravity_stage(
    arguments: &UserArguments,
    plan: &UserPlan,
) -> Result<(), InstallError> {
    let expected = planned_antigravity_stage(plan);
    let observed =
        read_optional_regular_tree(&arguments.root_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))?
            .ok_or_else(|| {
                InstallError::Verification(
                    "Antigravity native plugin staging directory is missing".to_owned(),
                )
            })?;
    if observed != expected {
        return Err(InstallError::Verification(
            "Antigravity native plugin staging differs from the authenticated source tree"
                .to_owned(),
        ));
    }
    Ok(())
}

fn sanitized_command_diagnostic(command: &[&str], stdout: &[u8]) -> String {
    format!(
        "argv=`{}`; output-bytes={}; output-digest={}",
        command.join(" "),
        stdout.len(),
        sha256_digest(stdout)
    )
}

fn compensate_host_mutations(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
    allow_dangling_codex_recovery: bool,
) -> Result<(), InstallError> {
    if backup.pending_host_transition.is_some() {
        let executable = executable.ok_or_else(|| {
            InstallError::Internal("qualified recovery host executable is missing".to_owned())
        })?;
        resolve_pending_host_transition(
            arguments,
            backup_relative,
            backup,
            executable,
            runner,
            allow_dangling_codex_recovery,
        )?;
    }
    if backup.host_mutations.is_empty() {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified recovery host executable is missing".to_owned())
    })?;
    match arguments.host {
        UserHost::Codex => {
            let desired = backup.codex_state_before.clone().ok_or_else(|| {
                InstallError::Verification(
                    "Codex transaction backup omitted the pre-mutation structured state".to_owned(),
                )
            })?;
            reconcile_codex_state(
                arguments,
                backup_relative,
                backup,
                &desired,
                executable,
                runner,
            )?;
        }
        UserHost::Claude => {
            let desired = backup.claude_state_before.clone().ok_or_else(|| {
                InstallError::Verification(
                    "Claude transaction backup omitted the pre-mutation structured state"
                        .to_owned(),
                )
            })?;
            reconcile_claude_state(
                arguments,
                backup_relative,
                backup,
                &desired,
                executable,
                runner,
            )?;
        }
        UserHost::Antigravity => {
            let desired = backup.antigravity_state_before.clone().ok_or_else(|| {
                InstallError::Verification(
                    "Antigravity transaction backup omitted the pre-mutation structured state"
                        .to_owned(),
                )
            })?;
            reconcile_antigravity_state(
                arguments,
                backup_relative,
                backup,
                &desired,
                executable,
                runner,
            )?;
        }
    }
    backup.host_mutations.clear();
    backup.host_owned_state = None;
    backup.pending_host_transition = None;
    backup.codex_plugin_was_latent_before_marketplace_add = false;
    persist_backup(arguments, backup_relative, backup)?;
    Ok(())
}

fn resolve_pending_host_transition(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
    allow_dangling_codex_recovery: bool,
) -> Result<(), InstallError> {
    let pending = backup
        .pending_host_transition
        .as_ref()
        .ok_or_else(|| InstallError::Internal("pending host transition is missing".to_owned()))?;
    if allow_dangling_codex_recovery
        && is_recoverable_dangling_codex_marketplace(arguments, backup)?
    {
        let probe_error = probe_host_snapshot(arguments.host, executable, runner)
            .err()
            .ok_or_else(|| {
                InstallError::Conflict(
                    "pending Codex marketplace transition still has a structured host state; external state was preserved"
                        .to_owned(),
                )
            })?;
        if !is_dangling_codex_marketplace_probe(&probe_error) {
            return Err(InstallError::Conflict(format!(
                "unresolved {:?} host transition {:?} cannot be attributed during recovery; external state was preserved",
                pending.phase, pending.mutation
            )));
        }
        let command = codex_compensation_command(HostMutation::CodexMarketplaceAdded);
        let output = runner
            .run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
            .map_err(|error| {
                InstallError::Unsupported(format!(
                    "Codex Hive marketplace recovery command failed: {error}"
                ))
            })?;
        if !output.success {
            return Err(InstallError::Unsupported(format!(
                "Codex Hive marketplace recovery command exited unsuccessfully: {}",
                sanitized_command_diagnostic(command, &output.stdout)
            )));
        }
        let observed = probe_host_snapshot(arguments.host, executable, runner)?;
        if observed != pending.before {
            return Err(InstallError::Conflict(
                "Codex Hive marketplace recovery did not restore the authenticated pre-transaction state; external state was preserved"
                    .to_owned(),
            ));
        }
        backup.host_mutations.clear();
        backup.host_owned_state = None;
        backup.pending_host_transition = None;
        backup.codex_plugin_was_latent_before_marketplace_add = false;
        persist_backup(arguments, backup_relative, backup)?;
        return Ok(());
    }
    Err(InstallError::Conflict(format!(
        "unresolved {:?} host transition {:?} cannot be attributed during recovery; external state was preserved",
        pending.phase, pending.mutation
    )))
}

fn is_recoverable_dangling_codex_marketplace(
    arguments: &UserArguments,
    backup: &UserBackupManifest,
) -> Result<bool, InstallError> {
    let Some(pending) = backup.pending_host_transition.as_ref() else {
        return Ok(false);
    };
    if arguments.host != UserHost::Codex
        || pending.phase != HostTransitionPhase::Forward
        || pending.mutation != HostMutation::CodexMarketplaceAdded
        || !backup.host_mutations.is_empty()
        || backup.host_owned_state.is_some()
    {
        return Ok(false);
    }
    let Some(before) = backup.codex_state_before.as_ref() else {
        return Ok(false);
    };
    if before.marketplace.is_some() || before.plugin.is_some() {
        return Ok(false);
    }
    let expected_before = HostStateSnapshot::Codex(before.clone());
    let expected_after = expected_host_state_after(
        arguments,
        HostMutation::CodexMarketplaceAdded,
        &expected_before,
    )?;
    if pending.before != expected_before || pending.after != expected_after {
        return Ok(false);
    }
    let manifest = Path::new(".hive/marketplaces/codex/.agents/plugins/marketplace.json");
    Ok(read_optional_regular(&arguments.root_cap, manifest, MAX_USER_FILE_BYTES)?.is_none())
}

fn is_dangling_codex_marketplace_probe(error: &InstallError) -> bool {
    matches!(error, InstallError::Unsupported(message)
        if message.starts_with("Codex structured state probe exited unsuccessfully:")
            && message.contains("argv=`plugin marketplace list --json`"))
}

struct CompensationContext<'a, R: CommandRunner> {
    arguments: &'a UserArguments,
    backup_relative: &'a Path,
    backup: &'a mut UserBackupManifest,
    executable: &'a QualifiedExecutable,
    runner: &'a R,
}

fn reconcile_codex_state(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    desired: &CodexHostState,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let owned = backup.host_owned_state.clone().ok_or_else(|| {
        InstallError::Verification(
            "confirmed host mutations omitted their exact owned state".to_owned(),
        )
    })?;
    let observed = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed != owned {
        return Err(InstallError::Conflict(
            "host state drifted after Hive's confirmed transition; recovery preserved external state"
                .to_owned(),
        ));
    }
    let HostStateSnapshot::Codex(mut current) = owned else {
        return Err(InstallError::Internal(
            "Codex compensation received non-Codex state".to_owned(),
        ));
    };
    let latent_plugin = backup.codex_plugin_was_latent_before_marketplace_add;
    let mut context = CompensationContext {
        arguments,
        backup_relative,
        backup,
        executable,
        runner,
    };
    if current.marketplace != desired.marketplace {
        if current.plugin.is_some() && !latent_plugin {
            let mut after = current.clone();
            after.plugin = None;
            current = run_codex_reconciliation_step(
                &mut context,
                codex_compensation_command(HostMutation::CodexPluginAdded),
                HostMutation::CodexPluginAdded,
                &current,
                &after,
            )?;
        }
        if current.marketplace.is_some() {
            let mut after = current.clone();
            after.marketplace = None;
            if latent_plugin {
                after.plugin = None;
            }
            current = run_codex_reconciliation_step(
                &mut context,
                codex_compensation_command(HostMutation::CodexMarketplaceAdded),
                HostMutation::CodexMarketplaceAdded,
                &current,
                &after,
            )?;
        }
        if let Some(marketplace) = desired.marketplace.as_ref() {
            let command = [
                "plugin",
                "marketplace",
                "add",
                marketplace.root.as_str(),
                "--json",
            ];
            let mut after = current.clone();
            after.marketplace = Some(marketplace.clone());
            current = run_codex_reconciliation_step(
                &mut context,
                &command,
                HostMutation::CodexMarketplaceAdded,
                &current,
                &after,
            )?;
        }
    }
    if current.plugin != desired.plugin {
        let command = if desired.plugin.is_some() {
            codex_compensation_command(HostMutation::CodexPluginRefreshed)
        } else {
            codex_compensation_command(HostMutation::CodexPluginAdded)
        };
        let mut after = current.clone();
        after.plugin.clone_from(&desired.plugin);
        current = run_codex_reconciliation_step(
            &mut context,
            command,
            HostMutation::CodexPluginRefreshed,
            &current,
            &after,
        )?;
    }
    if current == *desired {
        Ok(())
    } else {
        Err(InstallError::Verification(format!(
            "Codex compensation did not restore the pre-mutation structured state for {}",
            arguments.host.as_str()
        )))
    }
}

fn reconcile_claude_state(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    desired: &ClaudeHostState,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let owned = backup.host_owned_state.clone().ok_or_else(|| {
        InstallError::Verification(
            "confirmed host mutations omitted their exact owned state".to_owned(),
        )
    })?;
    let observed = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed != owned {
        return Err(InstallError::Conflict(
            "host state drifted after Hive's confirmed transition; recovery preserved external state"
                .to_owned(),
        ));
    }
    let HostStateSnapshot::Claude(mut current) = owned else {
        return Err(InstallError::Internal(
            "Claude compensation received non-Claude state".to_owned(),
        ));
    };
    let mut context = CompensationContext {
        arguments,
        backup_relative,
        backup,
        executable,
        runner,
    };
    current = reconcile_claude_marketplace(&mut context, current, desired)?;
    if current.plugin != desired.plugin {
        if current.plugin.is_some() {
            let mut after = current.clone();
            after.plugin = None;
            current = run_claude_reconciliation_step(
                &mut context,
                &[
                    "plugin",
                    "uninstall",
                    "aigent-hive@aigent-hive",
                    "--scope",
                    "user",
                ],
                HostMutation::ClaudePluginInstalled,
                &current,
                &after,
            )?;
        }
        if let Some(plugin) = desired.plugin.as_ref() {
            let mut after = current.clone();
            after.plugin = Some(ClaudePluginState {
                enabled: true,
                ..plugin.clone()
            });
            current = run_claude_reconciliation_step(
                &mut context,
                &[
                    "plugin",
                    "install",
                    "aigent-hive@aigent-hive",
                    "--scope",
                    "user",
                ],
                HostMutation::ClaudePluginInstalled,
                &current,
                &after,
            )?;
            if !plugin.enabled {
                let mut after = current.clone();
                after.plugin = Some(plugin.clone());
                current = run_claude_reconciliation_step(
                    &mut context,
                    &[
                        "plugin",
                        "disable",
                        "aigent-hive@aigent-hive",
                        "--scope",
                        "user",
                    ],
                    HostMutation::ClaudePluginRefreshed,
                    &current,
                    &after,
                )?;
            }
        }
    }
    if current == *desired {
        Ok(())
    } else {
        Err(InstallError::Verification(format!(
            "Claude compensation did not restore the pre-mutation structured state for {}",
            arguments.host.as_str()
        )))
    }
}

fn reconcile_claude_marketplace(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    mut current: ClaudeHostState,
    desired: &ClaudeHostState,
) -> Result<ClaudeHostState, InstallError> {
    if current.marketplace == desired.marketplace {
        return Ok(current);
    }
    if current.plugin.is_some() {
        let mut after = current.clone();
        after.plugin = None;
        current = run_claude_reconciliation_step(
            context,
            &[
                "plugin",
                "uninstall",
                "aigent-hive@aigent-hive",
                "--scope",
                "user",
            ],
            HostMutation::ClaudePluginInstalled,
            &current,
            &after,
        )?;
    }
    if current.marketplace.is_some() {
        let mut after = current.clone();
        after.marketplace = None;
        current = run_claude_reconciliation_step(
            context,
            &[
                "plugin",
                "marketplace",
                "remove",
                "aigent-hive",
                "--scope",
                "user",
            ],
            HostMutation::ClaudeMarketplaceAdded,
            &current,
            &after,
        )?;
    }
    if let Some(marketplace) = desired.marketplace.as_ref() {
        let command = [
            "plugin",
            "marketplace",
            "add",
            marketplace.path.as_str(),
            "--scope",
            "user",
        ];
        let mut after = current.clone();
        after.marketplace = Some(marketplace.clone());
        current = run_claude_reconciliation_step(
            context,
            &command,
            HostMutation::ClaudeMarketplaceAdded,
            &current,
            &after,
        )?;
    }
    Ok(current)
}

fn reconcile_antigravity_state(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    desired: &AntigravityHostState,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let owned = backup.host_owned_state.clone().ok_or_else(|| {
        InstallError::Verification(
            "confirmed Antigravity mutation omitted its exact owned state".to_owned(),
        )
    })?;
    let observed = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed != owned {
        return Err(InstallError::Conflict(
            "Antigravity state drifted after Hive's confirmed transition; recovery preserved external state"
                .to_owned(),
        ));
    }
    let HostStateSnapshot::Antigravity(mut current) = owned else {
        return Err(InstallError::Internal(
            "Antigravity compensation received non-Antigravity state".to_owned(),
        ));
    };
    let mut context = CompensationContext {
        arguments,
        backup_relative,
        backup,
        executable,
        runner,
    };
    if desired.plugin.is_none() && current.plugin.is_some() {
        let mut after = current.clone();
        after.plugin = None;
        current = run_antigravity_reconciliation_step(
            &mut context,
            &["plugin", "uninstall", "aigent-hive"],
            HostMutation::AntigravityPluginInstalled,
            &current,
            &after,
        )?;
    } else if desired.plugin.is_some() {
        let source_path = expected_antigravity_source_path(arguments)?;
        let validate_command = ["plugin", "validate", source_path.as_str()];
        let output = runner
            .run(
                executable,
                &validate_command,
                COMMAND_TIMEOUT,
                COMMAND_OUTPUT_LIMIT,
            )
            .map_err(|error| {
                InstallError::Internal(format!(
                    "Antigravity compensation validation failed: {error}"
                ))
            })?;
        if !output.success {
            return Err(InstallError::Internal(format!(
                "Antigravity compensation validation returned a non-success result: {}",
                sanitized_command_diagnostic(&validate_command, &output.stdout)
            )));
        }
        let after = desired.clone();
        current = run_antigravity_reconciliation_step(
            &mut context,
            &["plugin", "install", source_path.as_str()],
            HostMutation::AntigravityPluginRefreshed,
            &current,
            &after,
        )?;
        validate_antigravity_recovered_stage(arguments, context.backup)?;
    }
    if current == *desired {
        Ok(())
    } else {
        Err(InstallError::Verification(
            "Antigravity compensation did not restore the pre-mutation structured state".to_owned(),
        ))
    }
}

fn run_antigravity_reconciliation_step(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &AntigravityHostState,
    after: &AntigravityHostState,
) -> Result<AntigravityHostState, InstallError> {
    let before = HostStateSnapshot::Antigravity(before.clone());
    let observed = run_compensation_transition(
        context,
        command,
        mutation,
        &before,
        HostStateSnapshot::Antigravity(after.clone()),
    )?;
    match observed {
        HostStateSnapshot::Antigravity(state) => Ok(state),
        HostStateSnapshot::Codex(_) | HostStateSnapshot::Claude(_) => Err(InstallError::Internal(
            "Antigravity compensation observed another host state".to_owned(),
        )),
    }
}

fn validate_antigravity_recovered_stage(
    arguments: &UserArguments,
    backup: &UserBackupManifest,
) -> Result<(), InstallError> {
    let source = Path::new(ANTIGRAVITY_SOURCE_RELATIVE);
    let mut expected = RegularTree::default();
    for entry in &backup.entries {
        if !entry.existed {
            continue;
        }
        let source_path = Path::new(&entry.path);
        let Ok(relative) = source_path.strip_prefix(source) else {
            continue;
        };
        let source_bytes =
            read_optional_regular(&arguments.root_cap, source_path, MAX_USER_FILE_BYTES)?
                .ok_or_else(|| {
                    InstallError::Verification(format!(
                        "restored Antigravity source omitted {}",
                        relative.display()
                    ))
                })?;
        insert_regular_tree_file(&mut expected, relative, source_bytes);
    }
    if expected.files.is_empty() {
        return Err(InstallError::Verification(
            "Antigravity recovery has no authenticated prior source bundle".to_owned(),
        ));
    }
    let observed =
        read_optional_regular_tree(&arguments.root_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))?
            .ok_or_else(|| {
                InstallError::Verification(
                    "restored Antigravity staging directory is missing".to_owned(),
                )
            })?;
    if observed != expected {
        return Err(InstallError::Verification(
            "restored Antigravity staging differs from the authenticated prior source tree"
                .to_owned(),
        ));
    }
    Ok(())
}

fn run_claude_reconciliation_step(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &ClaudeHostState,
    after: &ClaudeHostState,
) -> Result<ClaudeHostState, InstallError> {
    let before = HostStateSnapshot::Claude(before.clone());
    let observed = run_compensation_transition(
        context,
        command,
        mutation,
        &before,
        HostStateSnapshot::Claude(after.clone()),
    )?;
    match observed {
        HostStateSnapshot::Claude(state) => Ok(state),
        HostStateSnapshot::Codex(_) | HostStateSnapshot::Antigravity(_) => Err(
            InstallError::Internal("Claude compensation observed Codex state".to_owned()),
        ),
    }
}

fn run_codex_reconciliation_step(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &CodexHostState,
    after: &CodexHostState,
) -> Result<CodexHostState, InstallError> {
    let before = HostStateSnapshot::Codex(before.clone());
    let observed = run_compensation_transition(
        context,
        command,
        mutation,
        &before,
        HostStateSnapshot::Codex(after.clone()),
    )?;
    match observed {
        HostStateSnapshot::Codex(state) => Ok(state),
        HostStateSnapshot::Claude(_) | HostStateSnapshot::Antigravity(_) => Err(
            InstallError::Internal("Codex compensation observed Claude state".to_owned()),
        ),
    }
}

fn run_compensation_transition(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &HostStateSnapshot,
    after: HostStateSnapshot,
) -> Result<HostStateSnapshot, InstallError> {
    let observed_before =
        probe_host_snapshot(context.arguments.host, context.executable, context.runner)?;
    if observed_before != *before || context.backup.host_owned_state.as_ref() != Some(before) {
        return Err(InstallError::Conflict(
            "host state drifted immediately before Hive compensation; external state was preserved"
                .to_owned(),
        ));
    }
    context.backup.pending_host_transition = Some(PendingHostTransition {
        mutation,
        phase: HostTransitionPhase::Compensation,
        before: before.clone(),
        after: after.clone(),
    });
    persist_backup(context.arguments, context.backup_relative, context.backup)?;
    let command_result = context.runner.run(
        context.executable,
        command,
        COMMAND_TIMEOUT,
        COMMAND_OUTPUT_LIMIT,
    );
    let observed_after =
        probe_host_snapshot(context.arguments.host, context.executable, context.runner)?;
    if matches!(&command_result, Ok(output) if output.success) && observed_after == after {
        context.backup.host_owned_state = Some(after.clone());
        context.backup.pending_host_transition = None;
        persist_backup(context.arguments, context.backup_relative, context.backup)?;
        return Ok(after);
    }
    match command_result {
        Ok(output) if output.success => Err(InstallError::Internal(format!(
            "{} reconciliation command did not reach its exact structured target state: {}",
            context.arguments.host.as_str(),
            sanitized_command_diagnostic(command, &output.stdout)
        ))),
        Ok(output) => Err(InstallError::Internal(format!(
            "{} reconciliation command returned a non-success result and remains unresolved: {}",
            context.arguments.host.as_str(),
            sanitized_command_diagnostic(command, &output.stdout)
        ))),
        Err(error) => Err(InstallError::Internal(format!(
            "{} reconciliation command `{}` failed before its exact structured target state was observed: {error}",
            context.arguments.host.as_str(),
            command.join(" ")
        ))),
    }
}

fn remove_transaction_journal(
    arguments: &UserArguments,
    journal_relative: &Path,
) -> Result<(), InstallError> {
    let expected =
        read_optional_regular(&arguments.root_cap, journal_relative, MAX_USER_FILE_BYTES)?;
    let expected_permissions = expected
        .as_ref()
        .map(|_| file_permissions(&arguments.root_cap, journal_relative))
        .transpose()?;
    remove_regular_if_exists(
        &arguments.root_cap,
        journal_relative,
        expected.as_deref(),
        expected_permissions,
    )
}

fn rollback_after_failure(
    arguments: &UserArguments,
    transaction: &mut UserTransaction,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
    primary: InstallError,
) -> InstallError {
    if let Err(error) =
        restore_backup_files(arguments, &transaction.backup_relative, &transaction.backup)
    {
        return InstallError::Internal(format!(
            "{}; filesystem rollback failed: {}",
            primary.message(),
            error.message()
        ));
    }
    if let Err(error) =
        reconcile_root_index_after_rollback(arguments, transaction.backup.index_existed)
    {
        return InstallError::Internal(format!(
            "{}; root index rollback failed: {}",
            primary.message(),
            error.message()
        ));
    }
    if let Err(error) = compensate_host_mutations(
        arguments,
        &transaction.backup_relative,
        &mut transaction.backup,
        executable,
        runner,
        false,
    ) {
        return InstallError::Internal(format!("{}; {}", primary.message(), error.message()));
    }
    if let Err(cleanup) = remove_transaction_journal(arguments, &transaction.journal_relative) {
        return InstallError::Internal(format!(
            "{}; rollback journal cleanup failed: {}",
            primary.message(),
            cleanup.message()
        ));
    }
    primary
}

fn qualify_host(
    arguments: &UserArguments,
    _plan: &UserPlan,
    runner: &impl CommandRunner,
) -> Result<Option<QualifiedExecutable>, InstallError> {
    let executable_name = match arguments.host {
        UserHost::Codex => "codex",
        UserHost::Claude => "claude",
        UserHost::Antigravity => "agy",
    };
    runner.qualify(executable_name).map(Some).map_err(|_| {
        InstallError::Unsupported(format!(
            "{executable_name} executable is unavailable; user installation was not changed"
        ))
    })
}

fn probe_supported_host_version(
    host: UserHost,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<String, InstallError> {
    let output = runner
        .run(
            executable,
            &["--version"],
            COMMAND_TIMEOUT,
            COMMAND_OUTPUT_LIMIT,
        )
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "{} fixed-argv version probe failed: {error}",
                host.as_str()
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "{} fixed-argv version probe exited unsuccessfully",
            host.as_str()
        )));
    }
    let raw = std::str::from_utf8(&output.stdout)
        .map_err(|_| InstallError::Unsupported("host version output is not UTF-8".to_owned()))?
        .trim();
    let version = match host {
        UserHost::Codex => raw.strip_prefix("codex-cli "),
        UserHost::Claude => raw
            .strip_suffix(" (Claude Code)")
            .or_else(|| raw.strip_prefix("claude ")),
        UserHost::Antigravity => Some(raw),
    }
    .ok_or_else(|| {
        InstallError::Unsupported(format!(
            "{} version output has an unsupported shape",
            host.as_str()
        ))
    })?;
    let parsed = parse_three_part_version(version)?;
    let (minimum, maximum) = match host {
        UserHost::Codex => ((0, 145, 0), (1, 0, 0)),
        UserHost::Claude => ((2, 1, 0), (3, 0, 0)),
        UserHost::Antigravity => ((1, 1, 7), (1, 2, 0)),
    };
    if parsed < minimum || parsed >= maximum {
        return Err(InstallError::Unsupported(format!(
            "{} version {version} is outside supported range {}",
            host.as_str(),
            host.version_range()
        )));
    }
    Ok(version.to_owned())
}

fn parse_three_part_version(value: &str) -> Result<(u64, u64, u64), InstallError> {
    let mut parts = value.split('.');
    let parse = |part: Option<&str>| {
        let part = part.ok_or_else(|| {
            InstallError::Unsupported("host version must contain three numeric parts".to_owned())
        })?;
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(InstallError::Unsupported(
                "host version must contain three numeric parts".to_owned(),
            ));
        }
        part.parse::<u64>().map_err(|_| {
            InstallError::Unsupported("host version component is out of range".to_owned())
        })
    };
    let parsed = (
        parse(parts.next())?,
        parse(parts.next())?,
        parse(parts.next())?,
    );
    if parts.next().is_some() {
        return Err(InstallError::Unsupported(
            "host version must contain exactly three numeric parts".to_owned(),
        ));
    }
    Ok(parsed)
}

fn bind_host_version(plan: &mut UserPlan, version: &str) {
    let prior = plan.plan_digest.clone();
    plan.plan_digest = sha256_digest(format!("{prior}\0{version}").as_bytes());
    plan.qualified_host_version = Some(version.to_owned());
}

fn activate_host(
    arguments: &UserArguments,
    plan: &UserPlan,
    transaction: &mut UserTransaction,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let Some(marketplace_relative) = plan.marketplace_root.as_ref() else {
        return Ok(());
    };
    let executable = executable
        .ok_or_else(|| InstallError::Internal("qualified host executable is missing".to_owned()))?;
    let marketplace = arguments.user_root.join(marketplace_relative);
    let marketplace_text = marketplace.to_str().ok_or_else(|| {
        InstallError::Unsupported("marketplace path is not valid UTF-8".to_owned())
    })?;
    let command_sets = activation_commands(arguments.host, &transaction.backup, marketplace_text)?;
    for (command, mutation) in command_sets {
        if let Some(mutation) = mutation {
            if arguments.host == UserHost::Codex
                && matches!(
                    mutation,
                    HostMutation::CodexPluginAdded | HostMutation::CodexPluginRefreshed
                )
                && transaction
                    .backup
                    .codex_plugin_was_latent_before_marketplace_add
            {
                continue;
            }
            execute_forward_host_transition(
                arguments,
                plan,
                transaction,
                executable,
                runner,
                &command,
                mutation,
            )?;
            continue;
        }
        let output = runner
            .run(executable, &command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
            .map_err(|error| {
                InstallError::Unsupported(format!(
                    "{} native plugin command `{}` failed: {error}",
                    arguments.host.as_str(),
                    command.join(" ")
                ))
            })?;
        if !output.success {
            return Err(InstallError::Unsupported(format!(
                "{} native plugin command exited unsuccessfully: {}",
                arguments.host.as_str(),
                sanitized_command_diagnostic(&command, &output.stdout)
            )));
        }
    }
    Ok(())
}

fn execute_forward_host_transition(
    arguments: &UserArguments,
    plan: &UserPlan,
    transaction: &mut UserTransaction,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
    command: &[&str],
    mutation: HostMutation,
) -> Result<(), InstallError> {
    let expected_before = transaction
        .backup
        .host_owned_state
        .clone()
        .or_else(|| initial_host_snapshot(&transaction.backup))
        .ok_or_else(|| InstallError::Internal("host prestate is missing".to_owned()))?;
    let observed_before = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed_before != expected_before {
        return Err(InstallError::Conflict(
            "host state drifted before Hive could issue its next native mutation".to_owned(),
        ));
    }
    if let HostStateSnapshot::Antigravity(state) = &observed_before {
        validate_antigravity_prestate(arguments, plan, Some(state))?;
    }
    let expected_after = expected_host_state_after(arguments, mutation, &expected_before)?;
    transaction.backup.pending_host_transition = Some(PendingHostTransition {
        mutation,
        phase: HostTransitionPhase::Forward,
        before: expected_before.clone(),
        after: expected_after.clone(),
    });
    persist_backup(arguments, &transaction.backup_relative, &transaction.backup)?;
    let command_result = runner.run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT);
    let mut observed_after = probe_host_snapshot(arguments.host, executable, runner)?;
    if matches!(mutation, HostMutation::CodexMarketplaceAdded)
        && codex_marketplace_add_revealed_stale_hive_plugin(
            arguments,
            &expected_after,
            &observed_after,
        )?
    {
        let remove = codex_compensation_command(HostMutation::CodexPluginAdded);
        let output = runner
            .run(executable, remove, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
            .map_err(|error| {
                InstallError::Unsupported(format!(
                    "Codex Hive stale plugin recovery command failed: {error}"
                ))
            })?;
        if !output.success {
            return Err(InstallError::Unsupported(format!(
                "Codex Hive stale plugin recovery command exited unsuccessfully: {}",
                sanitized_command_diagnostic(remove, &output.stdout)
            )));
        }
        observed_after = probe_host_snapshot(arguments.host, executable, runner)?;
        if observed_after != expected_after {
            return Err(InstallError::Verification(
                "Codex Hive stale plugin recovery did not restore the exact marketplace transition"
                    .to_owned(),
            ));
        }
    }
    let codex_latent_plugin = matches!(mutation, HostMutation::CodexMarketplaceAdded)
        && codex_marketplace_add_revealed_expected_plugin(
            arguments,
            &expected_after,
            &observed_after,
        )?;
    if matches!(&command_result, Ok(output) if output.success)
        && (observed_after == expected_after || codex_latent_plugin)
    {
        let owned_state = if codex_latent_plugin {
            transaction
                .backup
                .codex_plugin_was_latent_before_marketplace_add = true;
            observed_after.clone()
        } else {
            expected_after.clone()
        };
        transaction.backup.host_mutations.push(mutation);
        transaction.backup.host_owned_state = Some(owned_state);
        transaction.backup.pending_host_transition = None;
        persist_backup(arguments, &transaction.backup_relative, &transaction.backup)?;
        return Ok(());
    }
    match command_result {
        Ok(output) if output.success => Err(InstallError::Verification(format!(
            "{} native plugin command did not produce its exact structured transition: {}",
            arguments.host.as_str(),
            sanitized_command_diagnostic(command, &output.stdout)
        ))),
        Ok(output) => Err(InstallError::Unsupported(format!(
            "{} native plugin command returned a non-success result and remains unresolved: {}",
            arguments.host.as_str(),
            sanitized_command_diagnostic(command, &output.stdout)
        ))),
        Err(error) => Err(InstallError::Unsupported(format!(
            "{} native plugin command `{}` failed before its exact structured transition was observed: {error}",
            arguments.host.as_str(),
            command.join(" ")
        ))),
    }
}

fn codex_marketplace_add_revealed_expected_plugin(
    arguments: &UserArguments,
    expected_after: &HostStateSnapshot,
    observed_after: &HostStateSnapshot,
) -> Result<bool, InstallError> {
    let (HostStateSnapshot::Codex(expected), HostStateSnapshot::Codex(observed)) =
        (expected_after, observed_after)
    else {
        return Ok(false);
    };
    if expected.plugin.is_some() || observed.marketplace != expected.marketplace {
        return Ok(false);
    }
    let HostStateSnapshot::Codex(expected_with_plugin) =
        expected_host_state_after(arguments, HostMutation::CodexPluginAdded, expected_after)?
    else {
        return Err(InstallError::Internal(
            "Codex plugin transition produced a non-Codex state".to_owned(),
        ));
    };
    Ok(observed.plugin == expected_with_plugin.plugin)
}

fn codex_marketplace_add_revealed_stale_hive_plugin(
    arguments: &UserArguments,
    expected_after: &HostStateSnapshot,
    observed_after: &HostStateSnapshot,
) -> Result<bool, InstallError> {
    let (HostStateSnapshot::Codex(expected), HostStateSnapshot::Codex(observed)) =
        (expected_after, observed_after)
    else {
        return Ok(false);
    };
    let Some(observed_plugin) = observed.plugin.as_ref() else {
        return Ok(false);
    };
    if expected.plugin.is_some() || observed.marketplace != expected.marketplace {
        return Ok(false);
    }
    let HostStateSnapshot::Codex(expected_with_plugin) =
        expected_host_state_after(arguments, HostMutation::CodexPluginAdded, expected_after)?
    else {
        return Err(InstallError::Internal(
            "Codex plugin transition produced a non-Codex state".to_owned(),
        ));
    };
    let Some(expected_plugin) = expected_with_plugin.plugin.as_ref() else {
        return Err(InstallError::Internal(
            "Codex plugin transition did not produce a plugin state".to_owned(),
        ));
    };
    Ok(observed_plugin.enabled
        && observed_plugin.source_path == expected_plugin.source_path
        && observed_plugin.marketplace_source == expected_plugin.marketplace_source
        && observed_plugin.version != expected_plugin.version)
}

fn initial_host_snapshot(backup: &UserBackupManifest) -> Option<HostStateSnapshot> {
    match backup.host {
        UserHost::Codex => backup
            .codex_state_before
            .clone()
            .map(HostStateSnapshot::Codex),
        UserHost::Claude => backup
            .claude_state_before
            .clone()
            .map(HostStateSnapshot::Claude),
        UserHost::Antigravity => backup
            .antigravity_state_before
            .clone()
            .map(HostStateSnapshot::Antigravity),
    }
}

fn probe_host_snapshot(
    host: UserHost,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<HostStateSnapshot, InstallError> {
    match host {
        UserHost::Codex => probe_codex_state(executable, runner).map(HostStateSnapshot::Codex),
        UserHost::Claude => probe_claude_state(executable, runner).map(HostStateSnapshot::Claude),
        UserHost::Antigravity => {
            probe_antigravity_state(executable, runner).map(HostStateSnapshot::Antigravity)
        }
    }
}

fn expected_host_state_after(
    arguments: &UserArguments,
    mutation: HostMutation,
    before: &HostStateSnapshot,
) -> Result<HostStateSnapshot, InstallError> {
    match (mutation, before) {
        (HostMutation::CodexMarketplaceAdded, HostStateSnapshot::Codex(state)) => {
            let mut state = state.clone();
            state.marketplace = Some(CodexMarketplaceState {
                root: expected_codex_marketplace_root(arguments)?,
            });
            Ok(HostStateSnapshot::Codex(state))
        }
        (
            HostMutation::CodexPluginAdded | HostMutation::CodexPluginRefreshed,
            HostStateSnapshot::Codex(state),
        ) => {
            let root = expected_codex_marketplace_root(arguments)?;
            let mut state = state.clone();
            state.plugin = Some(CodexPluginState {
                version: env!("CARGO_PKG_VERSION").to_owned(),
                enabled: true,
                source_path: expected_codex_plugin_source_path(arguments)?,
                marketplace_source: root,
            });
            Ok(HostStateSnapshot::Codex(state))
        }
        (
            HostMutation::ClaudeMarketplaceAdded | HostMutation::ClaudeMarketplaceRefreshed,
            HostStateSnapshot::Claude(state),
        ) => {
            let mut state = state.clone();
            state.marketplace = Some(ClaudeMarketplaceState {
                source: "directory".to_owned(),
                path: expected_claude_marketplace_path(arguments)?,
            });
            Ok(HostStateSnapshot::Claude(state))
        }
        (
            HostMutation::ClaudePluginInstalled | HostMutation::ClaudePluginRefreshed,
            HostStateSnapshot::Claude(state),
        ) => {
            let mut state = state.clone();
            state.plugin = Some(ClaudePluginState {
                version: env!("CARGO_PKG_VERSION").to_owned(),
                enabled: true,
                scope: "user".to_owned(),
            });
            Ok(HostStateSnapshot::Claude(state))
        }
        (
            HostMutation::AntigravityPluginInstalled | HostMutation::AntigravityPluginRefreshed,
            HostStateSnapshot::Antigravity(state),
        ) => {
            let mut state = state.clone();
            state.plugin = Some(expected_antigravity_plugin_state());
            Ok(HostStateSnapshot::Antigravity(state))
        }
        _ => Err(InstallError::Internal(
            "host mutation does not match the structured host state".to_owned(),
        )),
    }
}

fn activation_commands<'a>(
    host: UserHost,
    backup: &UserBackupManifest,
    marketplace_text: &'a str,
) -> Result<Vec<ActivationCommand<'a>>, InstallError> {
    match host {
        UserHost::Codex => {
            let mut commands = Vec::new();
            let before = backup.codex_state_before.as_ref().ok_or_else(|| {
                InstallError::Internal(
                    "Codex transaction omitted the pre-mutation structured state".to_owned(),
                )
            })?;
            if before.marketplace.is_none() {
                commands.push((
                    vec!["plugin", "marketplace", "add", marketplace_text, "--json"],
                    Some(HostMutation::CodexMarketplaceAdded),
                ));
            }
            commands.push((
                vec!["plugin", "add", "aigent-hive@aigent-hive", "--json"],
                Some(if before.plugin.is_some() {
                    HostMutation::CodexPluginRefreshed
                } else {
                    HostMutation::CodexPluginAdded
                }),
            ));
            Ok(commands)
        }
        UserHost::Claude => {
            let mut commands = vec![(vec!["plugin", "validate", marketplace_text], None)];
            let before = backup.claude_state_before.as_ref().ok_or_else(|| {
                InstallError::Internal(
                    "Claude transaction omitted the pre-mutation structured state".to_owned(),
                )
            })?;
            if before.marketplace.is_some() {
                commands.push((
                    vec!["plugin", "marketplace", "update", "aigent-hive"],
                    Some(HostMutation::ClaudeMarketplaceRefreshed),
                ));
            } else {
                commands.push((
                    vec![
                        "plugin",
                        "marketplace",
                        "add",
                        marketplace_text,
                        "--scope",
                        "user",
                    ],
                    Some(HostMutation::ClaudeMarketplaceAdded),
                ));
            }
            if before.plugin.is_some() {
                commands.push((
                    vec![
                        "plugin",
                        "update",
                        "aigent-hive@aigent-hive",
                        "--scope",
                        "user",
                    ],
                    Some(HostMutation::ClaudePluginRefreshed),
                ));
            } else {
                commands.push((
                    vec![
                        "plugin",
                        "install",
                        "aigent-hive@aigent-hive",
                        "--scope",
                        "user",
                    ],
                    Some(HostMutation::ClaudePluginInstalled),
                ));
            }
            Ok(commands)
        }
        UserHost::Antigravity => {
            let before = backup.antigravity_state_before.as_ref().ok_or_else(|| {
                InstallError::Internal(
                    "Antigravity transaction omitted the pre-mutation structured state".to_owned(),
                )
            })?;
            Ok(vec![
                (vec!["plugin", "validate", marketplace_text], None),
                (
                    vec!["plugin", "install", marketplace_text],
                    Some(if before.plugin.is_some() {
                        HostMutation::AntigravityPluginRefreshed
                    } else {
                        HostMutation::AntigravityPluginInstalled
                    }),
                ),
            ])
        }
    }
}

fn validate_plugin_package(arguments: &UserArguments, plan: &UserPlan) -> Result<(), InstallError> {
    let root = match arguments.host {
        UserHost::Codex | UserHost::Claude => plan
            .marketplace_root
            .as_ref()
            .ok_or_else(|| InstallError::Internal("plugin marketplace root is missing".to_owned()))?
            .join("plugins/aigent-hive"),
        UserHost::Antigravity => plan
            .marketplace_root
            .as_ref()
            .ok_or_else(|| {
                InstallError::Internal("Antigravity plugin source root is missing".to_owned())
            })?
            .clone(),
    };
    let manifest = match arguments.host {
        UserHost::Codex => ".codex-plugin/plugin.json",
        UserHost::Claude => ".claude-plugin/plugin.json",
        UserHost::Antigravity => "plugin.json",
    };
    for relative in [manifest, "skills/user-setup/SKILL.md"] {
        let path = root.join(relative);
        if read_optional_regular(&arguments.root_cap, &path, MAX_USER_FILE_BYTES)?.is_none() {
            return Err(InstallError::Verification(format!(
                "installed plugin package is missing {relative}"
            )));
        }
    }
    Ok(())
}

fn success_result(
    operation: UserOperation,
    arguments: &UserArguments,
    plan: &UserPlan,
    code: &'static str,
    message: &'static str,
    backup: Option<&str>,
) -> ActionResult {
    let action = match arguments.mode {
        UserMode::Validate => "ValidateHiveUser",
        _ => match operation {
            UserOperation::Install => "InstallHiveUser",
            UserOperation::Update => "UpdateHiveUser",
        },
    };
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths: plan.changed_paths.clone(),
        evidence: vec![Evidence {
            kind: "report",
            locator: format!("user-install-plan:{}", arguments.host.as_str()),
            digest: plan.plan_digest.clone(),
        }],
        next_action: match arguments.mode {
            UserMode::DryRun => Some(format!(
                "run with --host {} --apply to activate this exact user installation plan",
                arguments.host.as_str()
            )),
            _ => None,
        },
        data: Some(json!({
            "host": arguments.host.as_str(),
            "scope": "user",
            "product_version": env!("CARGO_PKG_VERSION"),
            "host_version_range": arguments.host.version_range(),
            "qualified_host_version": plan.qualified_host_version,
            "backup": backup,
            "foreign_guidance_preserved": true,
            "provider_credentials_requested": false
        })),
    }
}

fn failure(action: &'static str, error: &InstallError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code: match error {
            InstallError::Input(_) => "hive.user-install-invalid-input",
            InstallError::Conflict(_) => "hive.user-install-conflict",
            InstallError::Unsupported(_) => "hive.user-install-unsupported",
            InstallError::Verification(_) => "hive.user-install-verification-failed",
            InstallError::Internal(_) => "hive.internal-error",
        },
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn read_installed_manifest(
    root: &Dir,
    relative: &Path,
    host: UserHost,
) -> Result<Option<UserOwnershipManifest>, InstallError> {
    let Some(bytes) = read_optional_regular(root, relative, MAX_USER_FILE_BYTES)? else {
        return Ok(None);
    };
    let manifest = serde_json::from_slice::<UserOwnershipManifest>(&bytes).map_err(|_| {
        InstallError::Conflict(format!(
            "installed ownership manifest is malformed: {}",
            relative.display()
        ))
    })?;
    if manifest.schema_version != 1
        || manifest.host != host
        || !recognized_host_version_range(host, &manifest.host_version_range)
        || !valid_sha256(&manifest.source_release_digest)
        || !valid_sha256(&manifest.plan_digest)
        || manifest.entries.is_empty()
        || validate_relative(Path::new(&manifest.guidance_path)).is_err()
        || !permissions_match_managed_mode(file_permissions(root, relative)?, false)
    {
        return Err(InstallError::Conflict(format!(
            "installed ownership manifest binding is invalid: {}",
            relative.display()
        )));
    }
    let mut previous = None;
    for entry in &manifest.entries {
        let path = Path::new(&entry.path);
        validate_relative(path)?;
        if previous.is_some_and(|value: &str| value >= entry.path.as_str())
            || !valid_sha256(&entry.digest)
            || entry.unix_mode != installed_unix_mode(entry.executable)
            || entry.ownership.is_empty()
        {
            return Err(InstallError::Conflict(format!(
                "installed ownership manifest inventory is invalid: {}",
                relative.display()
            )));
        }
        previous = Some(entry.path.as_str());
    }
    Ok(Some(manifest))
}

fn recognized_host_version_range(host: UserHost, range: &str) -> bool {
    range == host.version_range()
        || (host == UserHost::Antigravity && matches!(range, ">=1.1.7 <2.0.0" | ">=2.3.1 <3.0.0"))
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn read_optional_regular(
    root: &Dir,
    relative: &Path,
    maximum: u64,
) -> Result<Option<Vec<u8>>, InstallError> {
    validate_relative(relative)?;
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(None);
    };
    let metadata = match parent.symlink_metadata(&name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_internal("inspect", relative, error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return Err(InstallError::Conflict(format!(
            "user installation path must be a bounded no-follow regular file: {}",
            relative.display()
        )));
    }
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(&name, &options)
        .map_err(|error| io_internal("open", relative, error))?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    Read::by_ref(&mut file)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| io_internal("read", relative, error))?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > maximum {
        return Err(InstallError::Conflict(format!(
            "user installation path changed during no-follow read: {}",
            relative.display()
        )));
    }
    Ok(Some(bytes))
}

fn read_optional_regular_tree(
    root: &Dir,
    relative: &Path,
) -> Result<Option<RegularTree>, InstallError> {
    validate_relative(relative)?;
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(None);
    };
    let metadata = match parent.symlink_metadata(&name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_internal("inspect tree", relative, error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(InstallError::Conflict(format!(
            "Antigravity plugin stage must be a no-follow directory: {}",
            relative.display()
        )));
    }
    let directory = parent.open_dir_nofollow(&name).map_err(|error| {
        InstallError::Conflict(format!(
            "cannot pin Antigravity plugin stage {}: {error}",
            relative.display()
        ))
    })?;
    let mut tree = RegularTree::default();
    let mut total_bytes = 0_u64;
    read_regular_tree(&directory, Path::new(""), 0, &mut tree, &mut total_bytes)?;
    Ok(Some(tree))
}

fn read_regular_tree(
    directory: &Dir,
    prefix: &Path,
    depth: usize,
    tree: &mut RegularTree,
    total_bytes: &mut u64,
) -> Result<(), InstallError> {
    let entries = directory.entries().map_err(|error| {
        InstallError::Internal(format!(
            "cannot enumerate Antigravity plugin stage: {error}"
        ))
    })?;
    for entry in entries {
        let name = entry
            .map_err(|error| {
                InstallError::Internal(format!(
                    "cannot read Antigravity plugin stage entry: {error}"
                ))
            })?
            .file_name();
        let relative = prefix.join(&name);
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            InstallError::Internal(format!(
                "cannot inspect Antigravity plugin stage entry {}: {error}",
                relative.display()
            ))
        })?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            if tree.directories.len() >= MAX_ANTIGRAVITY_STAGE_DIRECTORIES {
                return Err(InstallError::Conflict(format!(
                    "Antigravity plugin stage exceeds {MAX_ANTIGRAVITY_STAGE_DIRECTORIES} directories"
                )));
            }
            if depth >= MAX_ANTIGRAVITY_STAGE_DEPTH {
                return Err(InstallError::Conflict(format!(
                    "Antigravity plugin stage exceeds depth {MAX_ANTIGRAVITY_STAGE_DEPTH}"
                )));
            }
            tree.directories.insert(relative.clone());
            let child = directory.open_dir_nofollow(&name).map_err(|error| {
                InstallError::Conflict(format!(
                    "cannot pin Antigravity plugin stage directory {}: {error}",
                    relative.display()
                ))
            })?;
            read_regular_tree(&child, &relative, depth + 1, tree, total_bytes)?;
            continue;
        }
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(InstallError::Conflict(format!(
                "Antigravity plugin stage contains a non-regular entry: {}",
                relative.display()
            )));
        }
        if tree.files.len() >= MAX_ANTIGRAVITY_STAGE_FILES {
            return Err(InstallError::Conflict(format!(
                "Antigravity plugin stage exceeds {MAX_ANTIGRAVITY_STAGE_FILES} files"
            )));
        }
        let bytes = read_optional_regular(directory, Path::new(&name), MAX_USER_FILE_BYTES)?
            .ok_or_else(|| {
                InstallError::Conflict(format!(
                    "Antigravity plugin stage changed during inspection: {}",
                    relative.display()
                ))
            })?;
        *total_bytes = total_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| InstallError::Conflict("Antigravity stage size overflow".to_owned()))?;
        if *total_bytes > MAX_ANTIGRAVITY_STAGE_BYTES {
            return Err(InstallError::Conflict(format!(
                "Antigravity plugin stage exceeds {MAX_ANTIGRAVITY_STAGE_BYTES} bytes"
            )));
        }
        tree.files.insert(relative, bytes);
    }
    Ok(())
}

fn write_atomic(
    root: &Dir,
    relative: &Path,
    bytes: &[u8],
    executable: bool,
    expected: Option<&[u8]>,
    expected_permissions: Option<FilePermissions>,
) -> Result<(), InstallError> {
    cas_activate(
        root,
        relative,
        expected.map(|bytes| ExpectedFile {
            bytes,
            permissions: expected_permissions.expect("existing file requires permission token"),
        }),
        Some(bytes),
        FilePermissions {
            executable,
            unix_mode: None,
        },
    )
}

fn write_atomic_with_permissions(
    root: &Dir,
    relative: &Path,
    bytes: &[u8],
    permissions: FilePermissions,
    expected: Option<&[u8]>,
    expected_permissions: Option<FilePermissions>,
) -> Result<(), InstallError> {
    cas_activate(
        root,
        relative,
        expected.map(|bytes| ExpectedFile {
            bytes,
            permissions: expected_permissions.expect("existing file requires permission token"),
        }),
        Some(bytes),
        permissions,
    )
}

fn capability_parent(
    root: &Dir,
    relative: &Path,
    create: bool,
) -> Result<Option<(Dir, OsString)>, InstallError> {
    validate_relative(relative)?;
    let name = relative
        .file_name()
        .ok_or_else(|| InstallError::Input("user path has no file name".to_owned()))?
        .to_os_string();
    let mut current = root
        .try_clone()
        .map_err(|error| InstallError::Internal(format!("cannot clone user root: {error}")))?;
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let component = component.as_os_str();
            match current.symlink_metadata(component) {
                Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
                Ok(_) => {
                    return Err(InstallError::Conflict(format!(
                        "user installation ancestor is not a no-follow directory: {}",
                        relative.display()
                    )))
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound && create => {
                    current.create_dir(component).map_err(|error| {
                        InstallError::Conflict(format!(
                            "cannot create user installation ancestor {}: {error}",
                            relative.display()
                        ))
                    })?;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(io_internal("inspect ancestor", relative, error)),
            }
            current = current.open_dir_nofollow(component).map_err(|error| {
                InstallError::Conflict(format!(
                    "cannot pin user installation ancestor {}: {error}",
                    relative.display()
                ))
            })?;
        }
    }
    Ok(Some((current, name)))
}

fn validate_relative(path: &Path) -> Result<(), InstallError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(InstallError::Input(format!(
            "unsafe user-relative path: {}",
            path.display()
        )));
    }
    Ok(())
}

fn cas_activate(
    root: &Dir,
    relative: &Path,
    expected: Option<ExpectedFile<'_>>,
    desired: Option<&[u8]>,
    permissions: FilePermissions,
) -> Result<(), InstallError> {
    cas_activate_with_barrier(root, relative, expected, desired, permissions, None)
}

#[derive(Clone, Copy)]
struct ExpectedFile<'a> {
    bytes: &'a [u8],
    permissions: FilePermissions,
}

type CasBarrier<'a> = Option<&'a dyn Fn(&Dir, &OsStr, &Dir)>;

fn cas_activate_with_barrier(
    root: &Dir,
    relative: &Path,
    expected: Option<ExpectedFile<'_>>,
    desired: Option<&[u8]>,
    permissions: FilePermissions,
    after_claim: CasBarrier<'_>,
) -> Result<(), InstallError> {
    if expected.is_none() {
        return match desired {
            Some(bytes) => create_new_exclusive(root, relative, bytes, permissions),
            None => match read_optional_regular(root, relative, MAX_USER_FILE_BYTES)? {
                None => Ok(()),
                Some(_) => Err(InstallError::Conflict(format!(
                    "user installation path appeared before exclusive deletion: {}",
                    relative.display()
                ))),
            },
        };
    }
    let expected = expected.expect("checked");
    let (parent, name) = capability_parent(root, relative, false)?.ok_or_else(|| {
        InstallError::Conflict(format!(
            "user installation path disappeared before atomic claim: {}",
            relative.display()
        ))
    })?;
    let quarantine_name = claim_name(relative);
    parent.create_dir(&quarantine_name).map_err(|error| {
        InstallError::Conflict(format!(
            "prior user installation claim requires recovery at {}: {error}",
            recovery_locator(relative).display()
        ))
    })?;
    let quarantine = parent
        .open_dir_nofollow(&quarantine_name)
        .map_err(|error| {
            InstallError::Internal(format!("cannot open user installation claim: {error}"))
        })?;
    if let Err(error) = parent.rename(&name, &quarantine, OsStr::new("claimed.bin")) {
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return Err(InstallError::Conflict(format!(
            "user installation path changed before atomic claim at {}: {error}",
            relative.display()
        )));
    }
    if let Some(barrier) = after_claim {
        barrier(&parent, &name, &quarantine);
    }
    let claimed_maximum = u64::try_from(expected.bytes.len())
        .unwrap_or(u64::MAX)
        .max(MAX_USER_FILE_BYTES);
    let claimed = read_optional_regular(&quarantine, Path::new("claimed.bin"), claimed_maximum)?
        .ok_or_else(|| {
            InstallError::Verification(format!(
                "claimed user installation object is missing at {}",
                recovery_locator(relative).display()
            ))
        })?;
    let claimed_permissions = file_permissions(&quarantine, Path::new("claimed.bin"))?;
    if claimed != expected.bytes || claimed_permissions != expected.permissions {
        restore_claim(&parent, &name, quarantine, &quarantine_name, relative)?;
        return Err(InstallError::Conflict(format!(
            "user installation claimed concurrently changed bytes or permissions: {}",
            relative.display()
        )));
    }
    match desired {
        Some(bytes) => {
            stage_file(&quarantine, "replacement.bin", bytes, permissions, relative).map_err(
                |error| {
                    InstallError::Verification(format!(
                        "{}; exact prior object retained at {}",
                        error.message(),
                        recovery_locator(relative).display()
                    ))
                },
            )?;
            quarantine
                .hard_link("replacement.bin", &parent, &name)
                .map_err(|error| retained_claim_error(relative, "publish replacement", error))?;
            quarantine.remove_file("replacement.bin").map_err(|error| {
                retained_claim_error(relative, "clean staged replacement", error)
            })?;
        }
        None => match parent.symlink_metadata(&name) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(retained_claim_error(
                    relative,
                    "publish deletion because destination was reoccupied",
                    "destination occupied",
                ))
            }
            Err(error) => return Err(retained_claim_error(relative, "inspect deletion", error)),
        },
    }
    quarantine
        .remove_file("claimed.bin")
        .map_err(|error| retained_claim_error(relative, "clean prior bytes", error))?;
    drop(quarantine);
    parent
        .remove_dir(&quarantine_name)
        .map_err(|error| retained_claim_error(relative, "clean claim directory", error))
}

fn create_new_exclusive(
    root: &Dir,
    relative: &Path,
    bytes: &[u8],
    permissions: FilePermissions,
) -> Result<(), InstallError> {
    let (parent, name) = capability_parent(root, relative, true)?.ok_or_else(|| {
        InstallError::Internal("created user installation parent disappeared".to_owned())
    })?;
    let quarantine_name = claim_name(relative);
    parent.create_dir(&quarantine_name).map_err(|error| {
        InstallError::Conflict(format!(
            "prior user installation claim requires recovery at {}: {error}",
            recovery_locator(relative).display()
        ))
    })?;
    let quarantine = parent
        .open_dir_nofollow(&quarantine_name)
        .map_err(|error| {
            InstallError::Internal(format!("cannot open user installation claim: {error}"))
        })?;
    if let Err(error) = stage_file(&quarantine, "replacement.bin", bytes, permissions, relative) {
        let _ = quarantine.remove_file("replacement.bin");
        drop(quarantine);
        if let Err(cleanup) = parent.remove_dir(&quarantine_name) {
            return Err(InstallError::Verification(format!(
                "{}; empty creation claim cleanup failed at {}: {cleanup}",
                error.message(),
                recovery_locator(relative).display()
            )));
        }
        return Err(error);
    }
    if let Err(error) = quarantine.hard_link("replacement.bin", &parent, &name) {
        let _ = quarantine.remove_file("replacement.bin");
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return Err(InstallError::Conflict(format!(
            "user installation destination appeared before exclusive activation at {}: {error}",
            relative.display()
        )));
    }
    quarantine
        .remove_file("replacement.bin")
        .map_err(|error| retained_claim_error(relative, "clean staged creation", error))?;
    drop(quarantine);
    parent
        .remove_dir(&quarantine_name)
        .map_err(|error| retained_claim_error(relative, "clean creation claim", error))
}

fn stage_file(
    directory: &Dir,
    name: &str,
    bytes: &[u8],
    permissions: FilePermissions,
    relative: &Path,
) -> Result<(), InstallError> {
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = directory
        .open_with(name, &options)
        .map_err(|error| io_internal("stage", relative, error))?;
    set_file_permissions(&file, permissions, relative)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| io_internal("write staged", relative, error))
}

fn restore_claim(
    parent: &Dir,
    name: &OsStr,
    quarantine: Dir,
    quarantine_name: &OsStr,
    relative: &Path,
) -> Result<(), InstallError> {
    quarantine
        .hard_link("claimed.bin", parent, name)
        .map_err(|error| retained_claim_error(relative, "restore claimed bytes", error))?;
    quarantine
        .remove_file("claimed.bin")
        .map_err(|error| retained_claim_error(relative, "clean restored claim", error))?;
    drop(quarantine);
    parent
        .remove_dir(quarantine_name)
        .map_err(|error| retained_claim_error(relative, "clean restored claim directory", error))
}

fn claim_name(relative: &Path) -> OsString {
    let digest = sha256_digest(portable(relative).as_bytes());
    OsString::from(format!(
        ".hive-user-claim-{}",
        digest.trim_start_matches("sha256:")
    ))
}

fn recovery_locator(relative: &Path) -> PathBuf {
    relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(claim_name(relative))
        .join("claimed.bin")
}

fn retained_claim_error(
    relative: &Path,
    operation: &str,
    error: impl std::fmt::Display,
) -> InstallError {
    InstallError::Verification(format!(
        "cannot {operation} for {}; exact prior object retained at {}: {error}",
        relative.display(),
        recovery_locator(relative).display()
    ))
}

fn capability_metadata(
    root: &Dir,
    relative: &Path,
) -> Result<Result<cap_std::fs::Metadata, io::Error>, InstallError> {
    let Some((parent, name)) = capability_parent(root, relative, false)? else {
        return Ok(Err(io::Error::from(io::ErrorKind::NotFound)));
    };
    Ok(parent.symlink_metadata(name))
}

fn io_internal(operation: &str, relative: &Path, error: impl std::fmt::Display) -> InstallError {
    InstallError::Internal(format!(
        "cannot {operation} user installation path {}: {error}",
        relative.display()
    ))
}

#[cfg(unix)]
fn set_file_permissions(
    file: &cap_std::fs::File,
    permissions: FilePermissions,
    relative: &Path,
) -> Result<(), InstallError> {
    let mode = permissions
        .unix_mode
        .unwrap_or(if permissions.executable { 0o755 } else { 0o644 });
    file.set_permissions(cap_primitives::fs::Permissions::from_mode(mode))
        .map_err(|error| io_internal("set permissions on", relative, error))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn set_file_permissions(
    _file: &cap_std::fs::File,
    _permissions: FilePermissions,
    _relative: &Path,
) -> Result<(), InstallError> {
    Ok(())
}

#[cfg(unix)]
fn is_executable(root: &Dir, relative: &Path) -> Result<bool, InstallError> {
    match capability_metadata(root, relative)? {
        Ok(metadata) => Ok(metadata.permissions().mode() & 0o111 != 0),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(InstallError::Internal(format!(
            "cannot inspect permissions for {}: {error}",
            relative.display()
        ))),
    }
}

#[cfg(unix)]
fn file_permissions(root: &Dir, relative: &Path) -> Result<FilePermissions, InstallError> {
    capability_metadata(root, relative)?
        .map(|metadata| {
            let mode = metadata.permissions().mode() & 0o7777;
            FilePermissions {
                executable: mode & 0o111 != 0,
                unix_mode: Some(mode),
            }
        })
        .map_err(|error| {
            InstallError::Internal(format!(
                "cannot inspect permissions for {}: {error}",
                relative.display()
            ))
        })
}

#[cfg(not(unix))]
fn is_executable(root: &Dir, relative: &Path) -> Result<bool, InstallError> {
    Ok(capability_metadata(root, relative)?.is_ok())
}

#[cfg(not(unix))]
fn file_permissions(root: &Dir, relative: &Path) -> Result<FilePermissions, InstallError> {
    Ok(FilePermissions {
        executable: is_executable(root, relative)?,
        unix_mode: None,
    })
}

fn portable(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn json_line(value: &impl Serialize) -> Result<Vec<u8>, InstallError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| InstallError::Internal(format!("cannot encode JSON: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn unix_seconds() -> Result<u64, InstallError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| InstallError::Internal("system clock is before Unix epoch".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::{CommandOutput, QualifiedExecutable, SensorError};
    use crate::user_setup::DiscordGuardPreferences;
    use std::sync::Mutex;
    use tempfile::tempdir;

    struct FakeRunner {
        calls: Mutex<Vec<String>>,
    }

    impl FakeRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    struct UnavailableRunner;

    struct VersionRunner {
        stdout: Vec<u8>,
        success: bool,
    }

    impl CommandRunner for VersionRunner {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            Ok(QualifiedExecutable::synthetic(program))
        }

        fn run(
            &self,
            _program: &QualifiedExecutable,
            arguments: &[&str],
            _timeout: Duration,
            _output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            assert_eq!(arguments, ["--version"]);
            Ok(CommandOutput {
                success: self.success,
                stdout: self.stdout.clone(),
            })
        }
    }

    impl CommandRunner for UnavailableRunner {
        fn qualify(&self, _program: &str) -> Result<QualifiedExecutable, SensorError> {
            Err(SensorError::Unavailable)
        }

        fn run(
            &self,
            _program: &QualifiedExecutable,
            _arguments: &[&str],
            _timeout: Duration,
            _output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            panic!("unavailable runner must not execute a command")
        }
    }

    #[derive(Clone, Copy)]
    enum HostSabotage {
        None,
        LatentCodexPluginActivation,
        StaleCodexHivePluginActivation,
        FailBeforeMarketplaceMutation,
        FailAfterMarketplaceMutation,
        FailBeforePluginMutation,
        FailAfterPluginMutation,
        DeleteInstalledSkill,
        TamperGuidanceAfterActivation,
        CrashAfterPluginInverse,
        FailAfterPluginMutationAndCompensation,
        DriftBeforeFirstMutation,
        DriftBeforeSecondMutation,
        DriftBeforeLaterCompensation,
        ForeignAfterFailedForward,
        ForeignAfterFailedCompensation,
        DanglingCodexMarketplace,
    }

    struct StatefulHostRunner {
        root: PathBuf,
        calls: Mutex<Vec<String>>,
        qualified_host: Mutex<String>,
        marketplace_installed: Mutex<bool>,
        plugin_installed: Mutex<bool>,
        plugin_stale: Mutex<bool>,
        plugin_enabled: Mutex<bool>,
        marketplace_probe_count: Mutex<usize>,
        compensation_failures: Mutex<usize>,
        sabotage: HostSabotage,
    }

    impl StatefulHostRunner {
        fn new(root: &Path, sabotage: HostSabotage) -> Self {
            Self {
                root: root.canonicalize().expect("canonical fake host root"),
                calls: Mutex::new(Vec::new()),
                qualified_host: Mutex::new(String::new()),
                marketplace_installed: Mutex::new(false),
                plugin_installed: Mutex::new(matches!(
                    sabotage,
                    HostSabotage::LatentCodexPluginActivation
                        | HostSabotage::StaleCodexHivePluginActivation
                )),
                plugin_stale: Mutex::new(matches!(
                    sabotage,
                    HostSabotage::StaleCodexHivePluginActivation
                )),
                plugin_enabled: Mutex::new(true),
                marketplace_probe_count: Mutex::new(0),
                compensation_failures: Mutex::new(usize::from(matches!(
                    sabotage,
                    HostSabotage::FailAfterPluginMutationAndCompensation
                ))),
                sabotage,
            }
        }

        fn external_state(&self) -> (bool, bool) {
            (
                *self.marketplace_installed.lock().expect("marketplace"),
                *self.plugin_installed.lock().expect("plugin"),
            )
        }

        fn seed_installed_codex_state(&self) {
            *self.marketplace_installed.lock().expect("marketplace") = true;
            *self.plugin_installed.lock().expect("plugin") = true;
        }

        fn clear_codex_state(&self) {
            *self.marketplace_installed.lock().expect("marketplace") = false;
            *self.plugin_installed.lock().expect("plugin") = false;
        }

        fn seed_marketplace_only(&self) {
            *self.marketplace_installed.lock().expect("marketplace") = true;
            *self.plugin_installed.lock().expect("plugin") = false;
        }

        fn inject_probe_drift(&self, command: &str) {
            if command == "plugin marketplace list --json" {
                let mut count = self.marketplace_probe_count.lock().expect("probe count");
                *count += 1;
                if matches!(self.sabotage, HostSabotage::DriftBeforeFirstMutation) && *count == 2 {
                    *self.marketplace_installed.lock().expect("marketplace") = true;
                }
                if matches!(self.sabotage, HostSabotage::DriftBeforeSecondMutation) && *count == 4 {
                    *self.plugin_installed.lock().expect("plugin") = true;
                }
                if matches!(self.sabotage, HostSabotage::DriftBeforeLaterCompensation)
                    && *count == 10
                {
                    *self.plugin_installed.lock().expect("plugin") = true;
                }
            }
        }

        #[allow(clippy::too_many_lines)]
        fn probe_output(&self, command: &str) -> Option<CommandOutput> {
            let claude = self.qualified_host.lock().expect("host").as_str() == "claude";
            self.inject_probe_drift(command);
            if command == "plugin marketplace list --json"
                && !claude
                && matches!(self.sabotage, HostSabotage::DanglingCodexMarketplace)
                && *self.marketplace_installed.lock().expect("marketplace")
            {
                return Some(CommandOutput {
                    success: false,
                    stdout: Vec::new(),
                });
            }
            match (command, claude) {
                ("plugin marketplace list --json", true) => {
                    let entries = if *self.marketplace_installed.lock().expect("marketplace") {
                        vec![json!({
                            "name": "aigent-hive",
                            "source": "directory",
                            "path": self.root.join(".hive/marketplaces/claude"),
                            "installLocation": self.root.join(".claude/plugins/marketplaces/aigent-hive")
                        })]
                    } else {
                        Vec::new()
                    };
                    Some(CommandOutput {
                        success: true,
                        stdout: serde_json::to_vec(&entries).expect("Claude marketplace JSON"),
                    })
                }
                ("plugin list --json", true) => {
                    let entries = if *self.plugin_installed.lock().expect("plugin") {
                        vec![json!({
                            "id": "aigent-hive@aigent-hive",
                            "version": env!("CARGO_PKG_VERSION"),
                            "scope": "user",
                            "enabled": *self.plugin_enabled.lock().expect("enabled"),
                            "status": "enabled"
                        })]
                    } else {
                        Vec::new()
                    };
                    Some(CommandOutput {
                        success: true,
                        stdout: serde_json::to_vec(&entries).expect("Claude plugin JSON"),
                    })
                }
                ("plugin marketplace list --json", false) => {
                    let entries = if *self.marketplace_installed.lock().expect("marketplace") {
                        vec![json!({
                            "name": "aigent-hive",
                            "root": self.root.join(".hive/marketplaces/codex"),
                            "marketplaceSource": {
                                "sourceType": "local",
                                "source": self.root.join(".hive/marketplaces/codex")
                            }
                        })]
                    } else {
                        Vec::new()
                    };
                    Some(CommandOutput {
                        success: true,
                        stdout: serde_json::to_vec(&json!({"marketplaces": entries}))
                            .expect("Codex marketplace JSON"),
                    })
                }
                ("plugin list --json", false) => {
                    let plugin_visible = *self.plugin_installed.lock().expect("plugin")
                        && (*self.marketplace_installed.lock().expect("marketplace")
                            || !matches!(
                                self.sabotage,
                                HostSabotage::LatentCodexPluginActivation
                                    | HostSabotage::StaleCodexHivePluginActivation
                            ));
                    let entries = if plugin_visible {
                        vec![json!({
                            "pluginId": "aigent-hive@aigent-hive",
                            "name": "aigent-hive",
                            "marketplaceName": "aigent-hive",
                            "version": if *self.plugin_stale.lock().expect("stale plugin") {
                                "0.7.0"
                            } else {
                                env!("CARGO_PKG_VERSION")
                            },
                            "installed": true,
                            "enabled": true,
                            "source": {
                                "source": "local",
                                "path": self.root.join(
                                    ".hive/marketplaces/codex/plugins/aigent-hive"
                                )
                            },
                            "marketplaceSource": {
                                "sourceType": "local",
                                "source": self.root.join(".hive/marketplaces/codex")
                            }
                        })]
                    } else {
                        Vec::new()
                    };
                    Some(CommandOutput {
                        success: true,
                        stdout: serde_json::to_vec(&json!({
                            "installed": entries,
                            "available": []
                        }))
                        .expect("Codex plugins JSON"),
                    })
                }
                _ => None,
            }
        }

        fn run_codex_plugin_add(&self) -> Result<CommandOutput, SensorError> {
            if matches!(self.sabotage, HostSabotage::FailBeforePluginMutation) {
                return Err(SensorError::Failed);
            }
            *self.plugin_installed.lock().expect("plugin") = true;
            match self.sabotage {
                HostSabotage::FailAfterPluginMutation => {
                    return Err(SensorError::Failed);
                }
                HostSabotage::DeleteInstalledSkill
                | HostSabotage::CrashAfterPluginInverse
                | HostSabotage::FailAfterPluginMutationAndCompensation => {
                    fs::remove_file(self.root.join(
                        ".hive/marketplaces/codex/plugins/aigent-hive/skills/user-setup/SKILL.md",
                    ))
                    .expect("delete installed skill");
                }
                HostSabotage::TamperGuidanceAfterActivation => {
                    fs::write(self.root.join(".codex/AGENTS.md"), b"tampered guidance\n")
                        .expect("tamper guidance");
                }
                HostSabotage::None
                | HostSabotage::LatentCodexPluginActivation
                | HostSabotage::StaleCodexHivePluginActivation
                | HostSabotage::FailBeforeMarketplaceMutation
                | HostSabotage::FailAfterMarketplaceMutation
                | HostSabotage::FailBeforePluginMutation
                | HostSabotage::DriftBeforeFirstMutation
                | HostSabotage::DriftBeforeSecondMutation
                | HostSabotage::DriftBeforeLaterCompensation
                | HostSabotage::ForeignAfterFailedForward
                | HostSabotage::ForeignAfterFailedCompensation
                | HostSabotage::DanglingCodexMarketplace => {}
            }
            Ok(CommandOutput {
                success: true,
                stdout: Vec::new(),
            })
        }

        fn remove_plugin(&self) -> Result<CommandOutput, SensorError> {
            let mut failures = self.compensation_failures.lock().expect("failures");
            if *failures > 0 {
                *failures -= 1;
                return Ok(CommandOutput {
                    success: false,
                    stdout: Vec::new(),
                });
            }
            let mut installed = self.plugin_installed.lock().expect("plugin");
            if !*installed {
                return Ok(CommandOutput {
                    success: false,
                    stdout: Vec::new(),
                });
            }
            *installed = false;
            *self.plugin_stale.lock().expect("stale plugin") = false;
            if matches!(
                self.sabotage,
                HostSabotage::CrashAfterPluginInverse
                    | HostSabotage::ForeignAfterFailedCompensation
            ) {
                return Err(SensorError::Failed);
            }
            Ok(successful_output())
        }
    }

    impl CommandRunner for FakeRunner {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            self.calls
                .lock()
                .expect("calls")
                .push(format!("qualify:{program}"));
            Ok(QualifiedExecutable::synthetic(program))
        }

        fn run(
            &self,
            _program: &QualifiedExecutable,
            arguments: &[&str],
            _timeout: Duration,
            _output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            self.calls.lock().expect("calls").push(arguments.join(" "));
            if arguments == ["--version"] {
                let stdout = match self
                    .calls
                    .lock()
                    .expect("calls")
                    .iter()
                    .find_map(|call| call.strip_prefix("qualify:"))
                {
                    Some("codex") => b"codex-cli 0.145.0\n".to_vec(),
                    Some("claude") => b"2.1.0 (Claude Code)\n".to_vec(),
                    Some("agy") => b"1.1.7\n".to_vec(),
                    _ => Vec::new(),
                };
                return Ok(CommandOutput {
                    success: true,
                    stdout,
                });
            }
            Ok(CommandOutput {
                success: true,
                stdout: Vec::new(),
            })
        }
    }

    struct AntigravityRunner {
        root: PathBuf,
        calls: Mutex<Vec<String>>,
        plugin_installed: Mutex<bool>,
        plugin_probe_count: Mutex<usize>,
        stage_drift_on_probe: Option<usize>,
    }

    impl AntigravityRunner {
        fn new(root: &Path) -> Self {
            Self {
                root: root
                    .canonicalize()
                    .expect("canonical fake Antigravity root"),
                calls: Mutex::new(Vec::new()),
                plugin_installed: Mutex::new(false),
                plugin_probe_count: Mutex::new(0),
                stage_drift_on_probe: None,
            }
        }

        fn with_stage_drift_on_probe(root: &Path, probe: usize) -> Self {
            Self {
                stage_drift_on_probe: Some(probe),
                ..Self::new(root)
            }
        }

        fn copy_tree(source: &Path, destination: &Path) {
            if destination.exists() {
                fs::remove_dir_all(destination).expect("remove prior Antigravity staging");
            }
            fs::create_dir_all(destination).expect("create Antigravity staging");
            for entry in fs::read_dir(source).expect("read Antigravity source") {
                let entry = entry.expect("Antigravity source entry");
                let source_path = entry.path();
                let destination_path = destination.join(entry.file_name());
                if entry.file_type().expect("Antigravity source type").is_dir() {
                    Self::copy_tree(&source_path, &destination_path);
                } else {
                    fs::copy(&source_path, &destination_path)
                        .expect("copy Antigravity source file");
                }
            }
        }
    }

    impl CommandRunner for AntigravityRunner {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            assert_eq!(program, "agy");
            self.calls
                .lock()
                .expect("calls")
                .push(format!("qualify:{program}"));
            Ok(QualifiedExecutable::synthetic(program))
        }

        fn run(
            &self,
            _program: &QualifiedExecutable,
            arguments: &[&str],
            _timeout: Duration,
            _output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            let command = arguments.join(" ");
            self.calls.lock().expect("calls").push(command.clone());
            if arguments == ["--version"] {
                return Ok(CommandOutput {
                    success: true,
                    stdout: b"1.1.7\n".to_vec(),
                });
            }
            if arguments == ["plugin", "list"] {
                let mut probe_count = self.plugin_probe_count.lock().expect("probe count");
                *probe_count += 1;
                if self.stage_drift_on_probe == Some(*probe_count) {
                    let foreign = self
                        .root
                        .join(ANTIGRAVITY_STAGE_RELATIVE)
                        .join("foreign-race.txt");
                    fs::create_dir_all(foreign.parent().expect("foreign stage parent"))
                        .expect("foreign stage parent");
                    fs::write(foreign, b"foreign racing stage bytes\n")
                        .expect("foreign racing stage");
                }
                let installed = *self.plugin_installed.lock().expect("plugin");
                return Ok(CommandOutput {
                    success: true,
                    stdout: if installed {
                        serde_json::to_vec(&json!({
                            "imports": [{
                                "name": "aigent-hive",
                                "source": "antigravity",
                                "importedAt": "2026-07-26T00:00:00Z",
                                "components": ["skills"]
                            }]
                        }))
                        .expect("Antigravity import JSON")
                    } else {
                        b"No imported plugins.\n".to_vec()
                    },
                });
            }
            if arguments.len() == 3 && arguments[0] == "plugin" && arguments[1] == "validate" {
                return Ok(successful_output());
            }
            if arguments.len() == 3 && arguments[0] == "plugin" && arguments[1] == "install" {
                Self::copy_tree(
                    Path::new(arguments[2]),
                    &self.root.join(ANTIGRAVITY_STAGE_RELATIVE),
                );
                *self.plugin_installed.lock().expect("plugin") = true;
                return Ok(successful_output());
            }
            if arguments == ["plugin", "uninstall", "aigent-hive"] {
                let stage = self.root.join(ANTIGRAVITY_STAGE_RELATIVE);
                if stage.exists() {
                    fs::remove_dir_all(stage).expect("remove Antigravity staging");
                }
                *self.plugin_installed.lock().expect("plugin") = false;
                return Ok(successful_output());
            }
            panic!("unexpected Antigravity command: {command}")
        }
    }

    impl CommandRunner for StatefulHostRunner {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            *self.qualified_host.lock().expect("host") = program.to_owned();
            Ok(QualifiedExecutable::synthetic(program))
        }

        fn run(
            &self,
            _program: &QualifiedExecutable,
            arguments: &[&str],
            _timeout: Duration,
            _output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            let command = arguments.join(" ");
            self.calls.lock().expect("calls").push(command.clone());
            if command == "--version" {
                let stdout = match self.qualified_host.lock().expect("host").as_str() {
                    "codex" => b"codex-cli 0.145.0\n".to_vec(),
                    "claude" => b"2.1.0 (Claude Code)\n".to_vec(),
                    "agy" => b"1.1.7\n".to_vec(),
                    _ => Vec::new(),
                };
                return Ok(CommandOutput {
                    success: true,
                    stdout,
                });
            }
            if let Some(output) = self.probe_output(&command) {
                return Ok(output);
            }
            match command.as_str() {
                command if command.starts_with("plugin marketplace add ") => {
                    if matches!(self.sabotage, HostSabotage::ForeignAfterFailedForward) {
                        *self.marketplace_installed.lock().expect("marketplace") = true;
                        return Err(SensorError::Failed);
                    }
                    if matches!(self.sabotage, HostSabotage::FailBeforeMarketplaceMutation) {
                        return Err(SensorError::Failed);
                    }
                    *self.marketplace_installed.lock().expect("marketplace") = true;
                    if matches!(self.sabotage, HostSabotage::FailAfterMarketplaceMutation) {
                        return Err(SensorError::Failed);
                    }
                }
                "plugin marketplace update aigent-hive" => {
                    if matches!(self.sabotage, HostSabotage::FailBeforeMarketplaceMutation) {
                        return Err(SensorError::Failed);
                    }
                    if matches!(self.sabotage, HostSabotage::FailAfterMarketplaceMutation) {
                        return Err(SensorError::Failed);
                    }
                }
                "plugin install aigent-hive@aigent-hive --scope user" => {
                    if matches!(self.sabotage, HostSabotage::FailBeforePluginMutation) {
                        return Err(SensorError::Failed);
                    }
                    *self.plugin_installed.lock().expect("plugin") = true;
                    if matches!(
                        self.sabotage,
                        HostSabotage::FailAfterPluginMutation
                            | HostSabotage::CrashAfterPluginInverse
                    ) {
                        return Err(SensorError::Failed);
                    }
                }
                "plugin update aigent-hive@aigent-hive --scope user" => {
                    if matches!(self.sabotage, HostSabotage::FailBeforePluginMutation) {
                        return Err(SensorError::Failed);
                    }
                    if matches!(self.sabotage, HostSabotage::FailAfterPluginMutation) {
                        return Err(SensorError::Failed);
                    }
                }
                "plugin add aigent-hive@aigent-hive --json" => {
                    return self.run_codex_plugin_add();
                }
                "plugin remove aigent-hive@aigent-hive --json"
                | "plugin uninstall aigent-hive@aigent-hive --scope user" => {
                    return self.remove_plugin();
                }
                "plugin marketplace remove aigent-hive --json"
                | "plugin marketplace remove aigent-hive --scope user" => {
                    let mut installed = self.marketplace_installed.lock().expect("marketplace");
                    if !*installed {
                        return Ok(CommandOutput {
                            success: false,
                            stdout: Vec::new(),
                        });
                    }
                    *installed = false;
                }
                "plugin disable aigent-hive@aigent-hive --scope user" => {
                    *self.plugin_enabled.lock().expect("enabled") = false;
                }
                _ => {}
            }
            Ok(successful_output())
        }
    }

    fn successful_output() -> CommandOutput {
        CommandOutput {
            success: true,
            stdout: Vec::new(),
        }
    }

    fn args(root: &Path, host: UserHost, mode: UserMode) -> UserArguments {
        let user_root = root.canonicalize().expect("canonical user root");
        UserArguments {
            host,
            mode,
            root_cap: open_user_root(&user_root).expect("pinned user root"),
            user_root,
            setup_override: None,
        }
    }

    fn uninstall_args(root: &Path) -> UserUninstallArguments {
        let user_root = root.canonicalize().expect("canonical user root");
        UserUninstallArguments {
            root_cap: open_user_root(&user_root).expect("pinned user root"),
        }
    }

    #[test]
    fn cli_parse_uses_the_physical_user_root_after_no_follow_validation() {
        let temporary = tempdir().expect("tempdir");
        let requested = temporary.path().to_path_buf();
        let expected = requested.canonicalize().expect("canonical user root");
        let arguments = vec![
            "--scope".to_owned(),
            "user".to_owned(),
            "--host".to_owned(),
            "codex".to_owned(),
            "--user-root".to_owned(),
            requested.display().to_string(),
            "--dry-run".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let parsed = parse(&arguments).expect("parse user installation arguments");

        assert_eq!(parsed.user_root, expected);
    }

    #[test]
    fn uninstall_refuses_removed_destructive_flags() {
        for flag in ["--full", "-f"] {
            assert!(matches!(
                parse_uninstall(&[flag.to_owned()]),
                Err(InstallError::Input(_))
            ));
        }
    }

    #[test]
    fn uninstall_preserves_saved_setup_and_knowledge_then_reinstalls_without_setup_questions() {
        let temporary = tempdir().expect("tempdir");
        write_operational_setup(temporary.path(), &["codex"]);
        let setup = temporary.path().join(".hive/config/user-setup.yml");
        let saved_setup = fs::read(&setup).expect("saved setup");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
        let install = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        execute(UserOperation::Install, &install, &runner).expect("install");
        let knowledge = temporary.path().join(".hive/knowledge/Wiki/user-note.md");
        fs::write(
            &knowledge,
            b"---\nschema_version: 1\nid: user-note\nkind: concept\nsummary: user note\ntags: [test]\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: 2026-08-10T00:00:00Z\nupdated_at: 2026-08-10T00:00:00Z\n---\n\nUser knowledge\n",
        )
        .expect("knowledge note");

        let removed = execute_uninstall(&uninstall_args(temporary.path()), &runner)
            .expect("preserving uninstall");

        assert_eq!(removed.code, "hive.user-uninstall-complete");
        assert_eq!(fs::read(&setup).expect("saved setup retained"), saved_setup);
        assert!(knowledge.is_file());
        assert!(!temporary.path().join(".hive/install/codex.json").exists());
        assert!(!temporary.path().join(".hive/marketplaces/codex").exists());
        assert!(!temporary.path().join(".codex/AGENTS.md").exists());
        assert_eq!(runner.external_state(), (false, false));

        execute(UserOperation::Install, &install, &runner).expect("saved preference reinstall");
        assert_eq!(
            fs::read(&setup).expect("saved setup unchanged"),
            saved_setup
        );
        assert!(temporary.path().join(".hive/install/codex.json").is_file());
        assert!(temporary
            .path()
            .join(".hive/install/user-projection.json")
            .is_file());
        assert_eq!(runner.external_state(), (true, true));
    }

    fn write_operational_setup(root: &Path, selected_hosts: &[&str]) {
        let path = root.join(".hive/config/user-setup.yml");
        fs::create_dir_all(path.parent().expect("setup parent")).expect("setup parent");
        let hosts = selected_hosts
            .iter()
            .map(|host| format!("  - {host}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(
            path,
            format!(
                "schema_version: 1\ninterface_language: en\nwiki:\n  enabled: true\n  language: both\nprofile:\n  id: web-developer\npersona:\n  id: balanced\nselected_hosts:\n{hosts}\nskills:\n  mode: individual\n  selected:\n    - setup-hive\n    - hive-update\nusage_guard:\n  enabled: false\n  stop_remaining_percent: 20\n  codexbar_fallback_enabled: false\n"
            ),
        )
        .expect("user setup");
    }

    fn seed_historical_user_install(root: &Path, host: UserHost) -> UserOwnershipManifest {
        let historical = historical_user_inventory(host);
        let mut manifest = UserOwnershipManifest {
            schema_version: 1,
            product_version: historical.product_version,
            host,
            host_version_range: historical.host_version_range,
            source_release_digest: historical.source_release_digest,
            plan_digest: String::new(),
            last_backup: None,
            guidance_path: historical.guidance_path,
            entries: historical.entries,
        };
        manifest.plan_digest = inventory_digest(
            host,
            &manifest.product_version,
            &manifest.host_version_range,
            Path::new(&manifest.guidance_path),
            &manifest.source_release_digest,
            &manifest.entries,
        );
        for (relative, file) in historical_user_files(host) {
            let target = root.join(relative);
            fs::create_dir_all(target.parent().expect("historical parent"))
                .expect("historical parent");
            fs::write(&target, &file.bytes).expect("historical bytes");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &target,
                    fs::Permissions::from_mode(if file.executable { 0o755 } else { 0o644 }),
                )
                .expect("historical mode");
            }
        }
        let manifest_path = root.join(format!(".hive/install/{}.json", host.as_str()));
        fs::create_dir_all(manifest_path.parent().expect("manifest parent"))
            .expect("manifest parent");
        fs::write(
            &manifest_path,
            json_line(&manifest).expect("historical manifest JSON"),
        )
        .expect("historical manifest");
        manifest
    }

    fn seed_historical_080_user_install(root: &Path, host: UserHost) -> UserOwnershipManifest {
        let files = historical_080_managed_files(host);
        let guidance_path = match host {
            UserHost::Codex => ".codex/AGENTS.md",
            UserHost::Claude => ".claude/CLAUDE.md",
            UserHost::Antigravity => ".gemini/GEMINI.md",
        };
        let guidance_bytes = b"foreign prefix\n\n<!-- AIGENT-HIVE:USER:START -->\nfrozen 0.8 marker\n<!-- AIGENT-HIVE:USER:END -->\n";
        let mut entries = ownership_entries(&files);
        entries.push(UserOwnershipEntry {
            path: guidance_path.to_owned(),
            digest: sha256_digest(guidance_bytes),
            executable: false,
            unix_mode: installed_unix_mode(false),
            ownership: "shared-marker".to_owned(),
        });
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        let source_release_digest = source_release_digest_from_entries(&entries);
        let mut manifest = UserOwnershipManifest {
            schema_version: 1,
            product_version: "0.8.0".to_owned(),
            host,
            host_version_range: host.version_range().to_owned(),
            source_release_digest,
            plan_digest: String::new(),
            last_backup: None,
            guidance_path: guidance_path.to_owned(),
            entries,
        };
        manifest.plan_digest = inventory_digest(
            host,
            &manifest.product_version,
            &manifest.host_version_range,
            Path::new(&manifest.guidance_path),
            &manifest.source_release_digest,
            &manifest.entries,
        );
        for (relative, file) in files {
            let target = root.join(relative);
            fs::create_dir_all(target.parent().expect("0.8 historical parent"))
                .expect("0.8 historical parent");
            fs::write(&target, &file.bytes).expect("0.8 historical bytes");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &target,
                    fs::Permissions::from_mode(if file.executable { 0o755 } else { 0o644 }),
                )
                .expect("0.8 historical mode");
            }
        }
        let guidance = root.join(guidance_path);
        fs::create_dir_all(guidance.parent().expect("0.8 guidance parent"))
            .expect("0.8 guidance parent");
        fs::write(guidance, guidance_bytes).expect("0.8 guidance bytes");
        let manifest_path = root.join(format!(".hive/install/{}.json", host.as_str()));
        fs::create_dir_all(manifest_path.parent().expect("0.8 manifest parent"))
            .expect("0.8 manifest parent");
        fs::write(
            manifest_path,
            json_line(&manifest).expect("0.8 historical manifest JSON"),
        )
        .expect("0.8 historical manifest");
        manifest
    }

    fn assert_user_entries_equal(actual: &[UserOwnershipEntry], expected: &[UserOwnershipEntry]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.path, expected.path);
            assert_eq!(actual.digest, expected.digest);
            assert_eq!(actual.executable, expected.executable);
            assert_eq!(actual.unix_mode, expected.unix_mode);
            assert_eq!(actual.ownership, expected.ownership);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn seed_legacy_antigravity_directory_scan_install(root: &Path) {
        const LEGACY_MANIFEST: &[u8] = b"{\n  \"name\": \"aigent-hive\"\n}\n";
        const LEGACY_SKILLS: &[(&str, &[u8])] = &[
            (
                "hive-judge-package",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-judge-package/SKILL.md"),
            ),
            (
                "hive-knowledge-capture",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-knowledge-capture/SKILL.md"),
            ),
            (
                "hive-knowledge-maintenance",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-knowledge-maintenance/SKILL.md"),
            ),
            (
                "hive-knowledge-promote",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-knowledge-promote/SKILL.md"),
            ),
            (
                "hive-knowledge-query",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-knowledge-query/SKILL.md"),
            ),
            (
                "hive-migrate",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-migrate/SKILL.md"),
            ),
            (
                "hive-project-upgrade",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-project-upgrade/SKILL.md"),
            ),
            (
                "hive-prompt-refine",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-prompt-refine/SKILL.md"),
            ),
            (
                "hive-role-handoff",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-role-handoff/SKILL.md"),
            ),
            (
                "hive-run-checkpoint",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-run-checkpoint/SKILL.md"),
            ),
            (
                "hive-run-resume",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-run-resume/SKILL.md"),
            ),
            (
                "hive-simple-question",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-simple-question/SKILL.md"),
            ),
            (
                "hive-update",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-update/SKILL.md"),
            ),
            (
                "hive-usage-guard",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-usage-guard/SKILL.md"),
            ),
            (
                "setup-harness",
                include_bytes!("../../../harness/project-bases/0.7.0/skills/setup-harness/SKILL.md"),
            ),
        ];
        let inventory = legacy_antigravity_directory_scan_inventory();
        let mut bytes_by_path = BTreeMap::new();
        bytes_by_path.insert(
            PathBuf::from(".gemini/config/plugins/aigent-hive/plugin.json"),
            LEGACY_MANIFEST.to_vec(),
        );
        for (name, bytes) in LEGACY_SKILLS {
            let skill_path = PathBuf::from(format!("skills/{name}/SKILL.md"));
            bytes_by_path.insert(
                Path::new(".gemini/config/plugins/aigent-hive").join(&skill_path),
                bytes.to_vec(),
            );
            bytes_by_path.insert(Path::new(".gemini/config").join(skill_path), bytes.to_vec());
        }
        for entry in inventory
            .entries
            .iter()
            .filter(|entry| is_managed_ownership(&entry.ownership))
        {
            let relative = PathBuf::from(&entry.path);
            let bytes = bytes_by_path
                .get(&relative)
                .expect("legacy authenticated bytes");
            assert_eq!(sha256_digest(bytes), entry.digest);
            let target = root.join(&relative);
            fs::create_dir_all(target.parent().expect("legacy parent")).expect("legacy parent");
            fs::write(&target, bytes).expect("legacy bytes");
        }
        let mut manifest = UserOwnershipManifest {
            schema_version: 1,
            product_version: inventory.product_version,
            host: inventory.host,
            host_version_range: inventory.host_version_range,
            source_release_digest: inventory.source_release_digest,
            plan_digest: String::new(),
            last_backup: None,
            guidance_path: inventory.guidance_path,
            entries: inventory.entries,
        };
        manifest.plan_digest = inventory_digest(
            manifest.host,
            &manifest.product_version,
            &manifest.host_version_range,
            Path::new(&manifest.guidance_path),
            &manifest.source_release_digest,
            &manifest.entries,
        );
        let manifest_path = root.join(".hive/install/antigravity.json");
        fs::create_dir_all(manifest_path.parent().expect("legacy manifest parent"))
            .expect("legacy manifest parent");
        fs::write(
            manifest_path,
            json_line(&manifest).expect("legacy manifest JSON"),
        )
        .expect("legacy manifest");
    }

    fn transaction_backup(root: &Path, host: UserHost) -> UserBackupManifest {
        let journal: UserTransactionJournal = serde_json::from_slice(
            &fs::read(root.join(format!(".hive/install-transactions/{}.json", host.as_str())))
                .expect("transaction journal"),
        )
        .expect("journal JSON");
        serde_json::from_slice(
            &fs::read(root.join(journal.backup).join("manifest.json")).expect("backup manifest"),
        )
        .expect("backup JSON")
    }

    fn snapshot_user_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
        fn visit(root: &Path, current: &Path, output: &mut BTreeMap<String, Vec<u8>>) {
            let mut entries = fs::read_dir(current)
                .expect("read snapshot directory")
                .collect::<Result<Vec<_>, _>>()
                .expect("snapshot entries");
            entries.sort_by_key(std::fs::DirEntry::file_name);
            for entry in entries {
                let path = entry.path();
                let relative = path.strip_prefix(root).expect("relative snapshot path");
                let metadata = fs::symlink_metadata(&path).expect("snapshot metadata");
                if metadata.is_dir() {
                    output.insert(format!("{}/", portable(relative)), Vec::new());
                    visit(root, &path, output);
                } else if metadata.is_file() {
                    output.insert(portable(relative), fs::read(&path).expect("snapshot file"));
                } else {
                    output.insert(format!("{}:nonregular", portable(relative)), Vec::new());
                }
            }
        }
        let mut output = BTreeMap::new();
        visit(root, root, &mut output);
        output
    }

    #[test]
    fn user_marker_append_and_replace_preserve_foreign_bytes() {
        let foreign = b"before\r\n<!-- omx:block -->\r\nafter";
        let first = merge_user_marker(foreign, &render_user_guidance(UserHost::Codex, None))
            .expect("append");
        assert!(first.starts_with(foreign));
        let second = merge_user_marker(&first, &render_user_guidance(UserHost::Claude, None))
            .expect("replace");
        let outside = [&second[..foreign.len()]];
        assert_eq!(outside[0], foreign);
        assert_eq!(find_all(&second, USER_MARKER_START).len(), 1);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn operational_user_guidance_keeps_the_selected_language_consistent() {
        use crate::user_setup::{
            CatalogSelection, InterfaceLanguage, SelectedHost, SkillPreferences,
            SkillSelectionMode, UpdateCheckPreferences, UsageGuardPreferences, UserProfile,
            UserSetupConfig, WikiBackend, WikiLanguage, WikiPreferences,
        };

        let config = |interface_language| UserSetupConfig {
            schema_version: 1,
            interface_language,
            wiki: WikiPreferences {
                enabled: true,
                language: WikiLanguage::Both,
                backend: WikiBackend::Markdown,
                notion: None,
            },
            profile: UserProfile {
                contexts: vec!["web-developer".to_owned()],
                description: None,
            },
            persona: CatalogSelection {
                id: "balanced".to_owned(),
                custom_description: None,
            },
            selected_hosts: vec![SelectedHost::Codex],
            skills: SkillPreferences {
                mode: SkillSelectionMode::Individual,
                selected: vec!["setup-hive".to_owned()],
            },
            update_check: UpdateCheckPreferences::default(),
            usage_guard: UsageGuardPreferences {
                enabled: false,
                stop_remaining_percent: 20,
                codexbar_fallback_enabled: false,
                discord: DiscordGuardPreferences::default(),
                project_overrides: BTreeMap::new(),
            },
        };

        let english = String::from_utf8(render_user_guidance(
            UserHost::Codex,
            Some(&config(InterfaceLanguage::En)),
        ))
        .expect("English guidance");
        assert!(english.contains(
            "use English for every question and response unless the user explicitly requests another language for the current response"
        ));
        assert!(english.contains(
            "A message written in another language does not by itself change this preference"
        ));
        assert!(english.contains("Explain in simple terms by default"));
        assert!(english.contains("do not force irrelevant examples or weaken technical precision"));
        assert!(english.contains("For every passed, failed, skipped, deferred"));
        assert!(english.contains("Before every final response, review the current user statement"));
        assert!(english.contains("--user-statement <normalized-fact> --claim-key <stable-key>"));
        assert!(english.contains("canonical Markdown and derived-index receipt"));
        assert!(!english.contains("질문과 응답"));

        let korean = String::from_utf8(render_user_guidance(
            UserHost::Codex,
            Some(&config(InterfaceLanguage::Ko)),
        ))
        .expect("Korean guidance");
        assert!(korean.contains("명시적 요청이 없는 한 모든 질문과 응답에 한국어 사용"));
        assert!(korean.contains("다른 언어로 작성된 메시지만으로 이 선호를 변경하지 않음"));
        assert!(korean.contains("대체 가능한 일반 영어 단어의 한영 혼용 금지"));
        assert!(korean.contains("기본 설명은 쉬운 말로 작성"));
        assert!(korean.contains("관련 없는 예시 강제 또는 기술적 정확성 약화 금지"));
        assert!(korean.contains("통과·실패·건너뜀·연기·미검증·미지원"));
        assert!(korean.contains("모든 최종 응답 전 현재 사용자 발화와 완료 결과"));
        assert!(korean.contains("--user-statement <normalized-fact> --claim-key <stable-key>"));
        assert!(korean.contains("canonical Markdown과 derived-index receipt"));
        for avoidable_mixture in [
            "활성 adapter",
            "Interface language",
            "선택 host",
            "Global Wiki",
            "일일 update",
            "Enabled이면",
            "Project expedited",
            "optional refine",
            "Foreign guidance bytes",
        ] {
            assert!(!korean.contains(avoidable_mixture));
        }

        let mut disabled = config(InterfaceLanguage::En);
        disabled.wiki.enabled = false;
        let disabled = String::from_utf8(render_user_guidance(UserHost::Codex, Some(&disabled)))
            .expect("disabled guidance");
        assert!(disabled.contains("Global Wiki is disabled: do not write or refresh knowledge"));
        assert!(!disabled.contains("hive knowledge remember --user-root"));

        let broken = english.replacen(
            "--user-statement <normalized-fact> --claim-key <stable-key>",
            "knowledge write removed",
            1,
        );
        assert!(matches!(
            validate_operational_guidance(broken.as_bytes(), Some(&config(InterfaceLanguage::En))),
            Err(InstallError::Verification(_))
        ));

        for host in [UserHost::Codex, UserHost::Claude, UserHost::Antigravity] {
            let guidance = render_user_guidance(host, Some(&config(InterfaceLanguage::En)));
            validate_operational_guidance(&guidance, Some(&config(InterfaceLanguage::En)))
                .expect("every host must retain the mandatory capture contract");
        }
    }

    #[test]
    fn malformed_user_markers_fail_closed() {
        let malformed = b"<!-- AIGENT-HIVE:USER:START -->\nmissing end";
        assert!(matches!(
            merge_user_marker(malformed, &render_user_guidance(UserHost::Codex, None)),
            Err(InstallError::Conflict(_))
        ));
    }

    #[test]
    fn codex_plan_uses_supported_marketplace_manifest_location() {
        let temporary = tempdir().expect("tempdir");
        let plan =
            build_plan(&args(temporary.path(), UserHost::Codex, UserMode::DryRun)).expect("plan");
        let marketplace = Path::new(".hive/marketplaces/codex/.agents/plugins/marketplace.json");
        let bytes = &plan
            .files
            .get(marketplace)
            .expect("Codex marketplace manifest")
            .bytes;
        let value: serde_json::Value =
            serde_json::from_slice(bytes).expect("Codex marketplace JSON");
        assert_eq!(value["name"], "aigent-hive");
        assert_eq!(
            value["plugins"][0]["source"]["path"],
            "./plugins/aigent-hive"
        );
        assert!(!plan
            .files
            .contains_key(Path::new(".hive/marketplaces/codex/marketplace.json")));
    }

    #[test]
    fn user_plugin_plan_keeps_only_the_selected_host_skill_projection() {
        for host in [UserHost::Codex, UserHost::Claude, UserHost::Antigravity] {
            let temporary = tempdir().expect("tempdir");
            let plan =
                build_plan(&args(temporary.path(), host, UserMode::DryRun)).expect("host plan");
            let metadata = PathBuf::from(format!(
                ".hive/marketplaces/{}/plugins/aigent-hive/skills/user-setup/agents/openai.yaml",
                host.as_str()
            ));
            if host == UserHost::Claude {
                assert!(
                    !plan.files.contains_key(&metadata),
                    "Claude should not receive Codex metadata"
                );
            } else {
                let text = std::str::from_utf8(
                    &plan
                        .files
                        .get(&metadata)
                        .expect("host setup metadata")
                        .bytes,
                )
                .expect("host setup metadata should be UTF-8");
                assert!(
                    text.contains("allow_implicit_invocation: true"),
                    "{host:?} should preserve user-plugin invocation metadata"
                );
            }
        }
    }

    #[test]
    fn supported_host_version_ranges_enforce_floor_ceiling_and_shape() {
        for (host, floor, in_range, ceiling, malformed) in [
            (
                UserHost::Codex,
                "codex-cli 0.145.0\n",
                "codex-cli 0.999.9\n",
                "codex-cli 1.0.0\n",
                "codex-cli 0.145\n",
            ),
            (
                UserHost::Claude,
                "2.1.0 (Claude Code)\n",
                "2.99.1 (Claude Code)\n",
                "3.0.0 (Claude Code)\n",
                "Claude Code 2.1.0\n",
            ),
            (
                UserHost::Antigravity,
                "1.1.7\n",
                "1.1.99\n",
                "1.2.0\n",
                "agy 1.1.7\n",
            ),
        ] {
            let executable = QualifiedExecutable::synthetic(host.as_str());
            for accepted in [floor, in_range] {
                let runner = VersionRunner {
                    stdout: accepted.as_bytes().to_vec(),
                    success: true,
                };
                assert!(
                    probe_supported_host_version(host, &executable, &runner).is_ok(),
                    "{host:?} should accept {accepted:?}"
                );
            }
            for rejected in [ceiling, malformed] {
                let runner = VersionRunner {
                    stdout: rejected.as_bytes().to_vec(),
                    success: true,
                };
                assert!(
                    matches!(
                        probe_supported_host_version(host, &executable, &runner),
                        Err(InstallError::Unsupported(_))
                    ),
                    "{host:?} should reject {rejected:?}"
                );
            }
        }
    }

    #[test]
    fn antigravity_version_probe_accepts_actual_agy_shape() {
        let executable = QualifiedExecutable::synthetic("agy");
        let runner = VersionRunner {
            stdout: b"1.1.7\n".to_vec(),
            success: true,
        };
        assert_eq!(
            probe_supported_host_version(UserHost::Antigravity, &executable, &runner)
                .expect("supported agy"),
            "1.1.7"
        );
    }

    #[test]
    fn unsupported_host_version_stops_before_any_user_filesystem_mutation() {
        for (host, stdout) in [
            (UserHost::Codex, b"codex-cli 1.0.0\n".as_slice()),
            (UserHost::Claude, b"3.0.0 (Claude Code)\n".as_slice()),
            (UserHost::Antigravity, b"1.2.0\n".as_slice()),
        ] {
            let temporary = tempdir().expect("tempdir");
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let runner = VersionRunner {
                stdout: stdout.to_vec(),
                success: true,
            };
            let Err(error) = execute(UserOperation::Install, &arguments, &runner) else {
                panic!("unsupported version");
            };
            assert!(matches!(error, InstallError::Unsupported(_)));
            assert_eq!(
                fs::read_dir(temporary.path()).expect("root").count(),
                0,
                "{host:?} changed the user root before version rejection"
            );
        }
    }

    #[test]
    fn codex_uses_nonempty_override_without_touching_base_guidance() {
        let temporary = tempdir().expect("tempdir");
        fs::create_dir(temporary.path().join(".codex")).expect("codex dir");
        fs::write(
            temporary.path().join(".codex/AGENTS.override.md"),
            b"temporary override\n",
        )
        .expect("override");
        fs::write(
            temporary.path().join(".codex/AGENTS.md"),
            b"base guidance\n",
        )
        .expect("base");
        let plan =
            build_plan(&args(temporary.path(), UserHost::Codex, UserMode::DryRun)).expect("plan");
        assert!(plan
            .files
            .contains_key(Path::new(".codex/AGENTS.override.md")));
        assert!(!plan.files.contains_key(Path::new(".codex/AGENTS.md")));
        assert_eq!(
            fs::read(temporary.path().join(".codex/AGENTS.md")).expect("base"),
            b"base guidance\n"
        );
    }

    #[test]
    fn first_install_refuses_foreign_managed_projection_for_every_host() {
        for (host, relative) in [
            (
                UserHost::Codex,
                ".hive/marketplaces/codex/plugins/aigent-hive/skills/user-setup/SKILL.md",
            ),
            (
                UserHost::Claude,
                ".hive/marketplaces/claude/plugins/aigent-hive/skills/user-setup/SKILL.md",
            ),
            (
                UserHost::Antigravity,
                ".hive/marketplaces/antigravity/plugins/aigent-hive/skills/user-setup/SKILL.md",
            ),
        ] {
            let temporary = tempdir().expect("tempdir");
            let foreign = temporary.path().join(relative);
            fs::create_dir_all(foreign.parent().expect("foreign parent")).expect("foreign parent");
            fs::write(&foreign, b"foreign managed bytes\n").expect("foreign bytes");
            let error = build_plan(&args(temporary.path(), host, UserMode::DryRun))
                .err()
                .expect("foreign path must conflict");
            assert!(matches!(error, InstallError::Conflict(_)));
            assert_eq!(
                fs::read(foreign).expect("foreign bytes preserved"),
                b"foreign managed bytes\n"
            );
        }
    }

    #[test]
    fn occupied_manifest_without_complete_prior_ledger_is_not_ownership_proof() {
        let temporary = tempdir().expect("tempdir");
        let manifest = temporary.path().join(".hive/install/antigravity.json");
        fs::create_dir_all(manifest.parent().expect("manifest parent")).expect("manifest parent");
        fs::write(
            &manifest,
            br#"{
  "schema_version": 1,
  "product_version": "0.7.0",
  "host": "antigravity",
  "host_version_range": ">=2.3.1 <3.0.0",
  "source_release_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "plan_digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "last_backup": null,
  "guidance_path": ".gemini/GEMINI.md",
  "entries": []
}
"#,
        )
        .expect("foreign manifest");
        let Err(error) = build_plan(&args(
            temporary.path(),
            UserHost::Antigravity,
            UserMode::DryRun,
        )) else {
            panic!("incomplete ledger must conflict");
        };
        assert!(matches!(error, InstallError::Conflict(_)));
    }

    #[test]
    fn forged_nonempty_ledgers_matching_foreign_bytes_fail_for_every_host() {
        for (host, relative, ownership) in [
            (
                UserHost::Codex,
                ".hive/marketplaces/codex/plugins/aigent-hive/skills/setup-hive/SKILL.md",
                "immutable-plugin-package",
            ),
            (
                UserHost::Claude,
                ".hive/marketplaces/claude/plugins/aigent-hive/skills/setup-hive/SKILL.md",
                "immutable-plugin-package",
            ),
            (
                UserHost::Antigravity,
                ".hive/marketplaces/antigravity/plugins/aigent-hive/skills/setup-hive/SKILL.md",
                "immutable-plugin-package",
            ),
        ] {
            let temporary = tempdir().expect("tempdir");
            let foreign = temporary.path().join(relative);
            fs::create_dir_all(foreign.parent().expect("foreign parent")).expect("foreign parent");
            fs::write(&foreign, b"foreign bytes with forged proof\n").expect("foreign");
            let manifest_relative = PathBuf::from(format!(".hive/install/{}.json", host.as_str()));
            let manifest_path = temporary.path().join(&manifest_relative);
            fs::create_dir_all(manifest_path.parent().expect("manifest parent"))
                .expect("manifest parent");
            let mut forged = UserOwnershipManifest {
                schema_version: 1,
                product_version: env!("CARGO_PKG_VERSION").to_owned(),
                host,
                host_version_range: host.version_range().to_owned(),
                source_release_digest:
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .to_owned(),
                plan_digest: String::new(),
                last_backup: None,
                guidance_path: portable(&match host {
                    UserHost::Codex => PathBuf::from(".codex/AGENTS.md"),
                    UserHost::Claude => PathBuf::from(".claude/CLAUDE.md"),
                    UserHost::Antigravity => PathBuf::from(".gemini/GEMINI.md"),
                }),
                entries: vec![UserOwnershipEntry {
                    path: relative.to_owned(),
                    digest: sha256_digest(b"foreign bytes with forged proof\n"),
                    executable: false,
                    unix_mode: installed_unix_mode(false),
                    ownership: ownership.to_owned(),
                }],
            };
            forged.plan_digest = inventory_digest(
                host,
                &forged.product_version,
                &forged.host_version_range,
                Path::new(&forged.guidance_path),
                &forged.source_release_digest,
                &forged.entries,
            );
            fs::write(&manifest_path, json_line(&forged).expect("forged JSON"))
                .expect("forged manifest");
            let Err(error) = build_plan(&args(temporary.path(), host, UserMode::DryRun)) else {
                panic!("forged manifest must conflict");
            };
            assert!(matches!(error, InstallError::Conflict(_)));
            assert_eq!(
                fs::read(&foreign).expect("foreign preserved"),
                b"foreign bytes with forged proof\n"
            );
        }
    }

    #[test]
    fn authenticated_historical_inventory_upgrades_to_current_release_for_every_host() {
        assert_eq!(
            historical_user_inventory(UserHost::Antigravity).host_version_range,
            ">=2.3.1 <3.0.0"
        );
        for host in [UserHost::Codex, UserHost::Claude, UserHost::Antigravity] {
            let temporary = tempdir().expect("tempdir");
            let historical = historical_user_inventory(host);
            let historical_files = historical_user_files(host);
            let retired = historical_files
                .iter()
                .find(|(_, file)| file.executable)
                .map(|(path, _)| path.clone())
                .expect("retired executable");
            let manifest_relative = PathBuf::from(format!(".hive/install/{}.json", host.as_str()));
            seed_historical_user_install(temporary.path(), host);
            let manifest_path = temporary.path().join(&manifest_relative);

            let arguments = args(temporary.path(), host, UserMode::Apply);
            let plan = build_plan(&arguments).expect("historical upgrade plan");
            assert!(plan.retired_files.contains_key(&retired));
            assert!(plan.changed_paths.contains(&portable(&retired)));
            match host {
                UserHost::Codex | UserHost::Claude => {
                    let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
                    execute(UserOperation::Update, &arguments, &runner)
                        .expect("historical upgrade");
                }
                UserHost::Antigravity => {
                    execute(
                        UserOperation::Update,
                        &arguments,
                        &AntigravityRunner::new(temporary.path()),
                    )
                    .expect("historical upgrade");
                }
            }

            let upgraded: UserOwnershipManifest =
                serde_json::from_slice(&fs::read(&manifest_path).expect("upgraded manifest"))
                    .expect("upgraded manifest JSON");
            assert_eq!(upgraded.product_version, env!("CARGO_PKG_VERSION"));
            assert_ne!(
                upgraded.source_release_digest,
                historical.source_release_digest
            );
            assert_ne!(upgraded.entries.len(), historical.entries.len());
            assert!(!temporary.path().join(retired).exists());
            assert_eq!(
                fs::read(temporary.path().join(".hive/historical-shared-marker.md"))
                    .expect("shared marker preserved"),
                b"historical shared marker\n"
            );
            assert_eq!(
                fs::read(
                    temporary
                        .path()
                        .join(".hive/knowledge/historical-protected.md")
                )
                .expect("protected knowledge preserved"),
                b"historical protected knowledge\n"
            );
        }
    }

    #[test]
    fn historical_070_user_inventory_is_exact_and_rejects_wrong_bindings() {
        for (host, range, expected_entries) in [
            (UserHost::Codex, ">=0.145.0 <1.0.0", 25),
            (UserHost::Claude, ">=2.1.0 <3.0.0", 25),
            (UserHost::Antigravity, ">=1.1.7 <1.2.0", 37),
        ] {
            let guidance = Path::new(match host {
                UserHost::Codex => ".codex/AGENTS.md",
                UserHost::Claude => ".claude/CLAUDE.md",
                UserHost::Antigravity => ".gemini/GEMINI.md",
            });
            let inventory = historical_070_user_inventory(host, range, guidance)
                .expect("authenticated 0.7 inventory");
            assert_eq!(inventory.product_version, "0.7.0");
            assert_eq!(inventory.entries.len(), expected_entries);
            assert!(valid_sha256(&inventory.source_release_digest));
            assert!(inventory
                .entries
                .windows(2)
                .all(|pair| pair[0].path < pair[1].path));
            assert!(!inventory
                .entries
                .iter()
                .any(|entry| entry.path.contains("/skills/setup-hive/")));
            assert!(historical_070_user_inventory(host, ">=9.0.0 <10.0.0", guidance).is_none());
            assert!(
                historical_070_user_inventory(host, range, Path::new(".hive/foreign.md")).is_none()
            );
        }
    }

    #[test]
    fn historical_080_user_inventory_authenticates_exact_snapshots_for_every_host() {
        for (host, expected_entries) in [
            (UserHost::Codex, 39),
            (UserHost::Claude, 22),
            (UserHost::Antigravity, 70),
        ] {
            let temporary = tempdir().expect("tempdir");
            let manifest = seed_historical_080_user_install(temporary.path(), host);
            let inventory = historical_080_user_inventory(
                host,
                &manifest.host_version_range,
                Path::new(&manifest.guidance_path),
                &manifest.entries,
            )
            .expect("authenticated 0.8 inventory");
            assert_eq!(inventory.product_version, "0.8.0");
            assert_eq!(inventory.entries.len(), expected_entries);
            assert_user_entries_equal(&inventory.entries, &manifest.entries);
            assert_eq!(
                inventory.source_release_digest,
                manifest.source_release_digest
            );

            let authenticated = authenticated_user_inventory(
                host,
                &InventoryAuthentication {
                    product_version: "0.8.0",
                    installed_host_version_range: &manifest.host_version_range,
                    source_release_digest: &manifest.source_release_digest,
                    installed_entries: &manifest.entries,
                    installed_guidance_path: Path::new(&manifest.guidance_path),
                    current_guidance_path: Path::new(&manifest.guidance_path),
                    current_source_release_digest:
                        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                    current_entries: &[],
                    authenticated_prior: None,
                },
            )
            .expect("0.8 branch should authenticate independently of current bytes");
            assert_user_entries_equal(&authenticated.entries, &manifest.entries);

            let mut tampered = manifest.entries.clone();
            let managed = tampered
                .iter_mut()
                .find(|entry| is_managed_ownership(&entry.ownership))
                .expect("managed entry");
            managed.digest = format!("sha256:{}", "f".repeat(64));
            assert!(historical_080_user_inventory(
                host,
                &manifest.host_version_range,
                Path::new(&manifest.guidance_path),
                &tampered,
            )
            .is_none());
            assert!(historical_080_user_inventory(
                host,
                ">=9.0.0 <10.0.0",
                Path::new(&manifest.guidance_path),
                &manifest.entries,
            )
            .is_none());
        }
    }

    #[test]
    fn pre_scope_routing_test_inventory_accepts_only_the_exact_test_predecessor() {
        let temporary = tempdir().expect("tempdir");
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::DryRun);
        let desired = build_desired_user_files(&arguments, None).expect("current desired files");
        let current = authenticated_current_inventory(&arguments, &desired);
        let mut predecessor_entries = current.entries.clone();
        let setup_hive = predecessor_entries
            .iter_mut()
            .find(|entry| entry.path.ends_with("/skills/user-setup/SKILL.md"))
            .expect("user-setup projection");
        setup_hive.digest = PRE_SCOPE_ROUTING_SETUP_HIVE_DIGEST.to_owned();
        let predecessor_digest = source_release_digest_from_entries(&predecessor_entries);
        let request = InventoryAuthentication {
            product_version: "0.9.0",
            installed_host_version_range: UserHost::Codex.version_range(),
            source_release_digest: &predecessor_digest,
            installed_entries: &predecessor_entries,
            installed_guidance_path: Path::new(".codex/AGENTS.md"),
            current_guidance_path: Path::new(".codex/AGENTS.md"),
            current_source_release_digest: &current.source_release_digest,
            current_entries: &current.entries,
            authenticated_prior: None,
        };

        let authenticated = authenticated_user_inventory(UserHost::Codex, &request)
            .expect("exact pre-routing test inventory");
        assert_user_entries_equal(&authenticated.entries, &predecessor_entries);
        assert_eq!(authenticated.source_release_digest, predecessor_digest);

        let forged_digest = format!("sha256:{}", "0".repeat(64));
        let forged = InventoryAuthentication {
            source_release_digest: &forged_digest,
            ..request
        };
        assert!(authenticated_user_inventory(UserHost::Codex, &forged).is_none());
    }

    #[test]
    fn test_three_inventory_accepts_only_the_published_test_three_predecessor() {
        let temporary = tempdir().expect("tempdir");
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::DryRun);
        let desired = build_desired_user_files(&arguments, None).expect("current desired files");
        let current = authenticated_current_inventory(&arguments, &desired);
        let mut predecessor_entries = current.entries.clone();
        let setup_hive = predecessor_entries
            .iter_mut()
            .find(|entry| entry.path.ends_with("/skills/user-setup/SKILL.md"))
            .expect("user-setup projection");
        setup_hive.digest = TEST3_SETUP_HIVE_DIGEST.to_owned();
        let predecessor_digest = source_release_digest_from_entries(&predecessor_entries);
        let request = InventoryAuthentication {
            product_version: "0.9.0",
            installed_host_version_range: UserHost::Codex.version_range(),
            source_release_digest: &predecessor_digest,
            installed_entries: &predecessor_entries,
            installed_guidance_path: Path::new(".codex/AGENTS.md"),
            current_guidance_path: Path::new(".codex/AGENTS.md"),
            current_source_release_digest: &current.source_release_digest,
            current_entries: &current.entries,
            authenticated_prior: None,
        };

        let authenticated =
            test_three_user_inventory(UserHost::Codex, &request).expect("exact test.3 inventory");
        assert_user_entries_equal(&authenticated.entries, &predecessor_entries);

        let forged_digest = format!("sha256:{}", "0".repeat(64));
        let forged = InventoryAuthentication {
            source_release_digest: &forged_digest,
            ..request
        };
        assert!(test_three_user_inventory(UserHost::Codex, &forged).is_none());
    }

    #[test]
    fn developer_build_accepts_only_an_internally_reproducible_prior_inventory() {
        let temporary = tempdir().expect("tempdir");
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::DryRun);
        let desired = build_desired_user_files(&arguments, None).expect("current desired files");
        let current = authenticated_current_inventory(&arguments, &desired);
        let request = InventoryAuthentication {
            product_version: "0.9.0",
            installed_host_version_range: UserHost::Codex.version_range(),
            source_release_digest: &current.source_release_digest,
            installed_entries: &current.entries,
            installed_guidance_path: Path::new(".codex/AGENTS.md"),
            current_guidance_path: Path::new(".codex/AGENTS.md"),
            current_source_release_digest:
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            current_entries: &[],
            authenticated_prior: None,
        };

        let authenticated =
            developer_authenticated_user_inventory(UserHost::Codex, &request, "0.9.0-dev")
                .expect("developer base");
        assert_user_entries_equal(&authenticated.entries, &current.entries);
        assert!(
            developer_authenticated_user_inventory(UserHost::Codex, &request, "0.9.0-test.4")
                .is_none()
        );

        let forged_digest = format!("sha256:{}", "0".repeat(64));
        let forged = InventoryAuthentication {
            source_release_digest: &forged_digest,
            ..request
        };
        assert!(
            developer_authenticated_user_inventory(UserHost::Codex, &forged, "0.9.0-dev").is_none()
        );
    }

    #[test]
    fn historical_070_codex_onboarding_inventory_authenticates_only_its_frozen_snapshot() {
        let inventory = historical_070_codex_onboarding_inventory(
            UserHost::Codex,
            ">=0.145.0 <1.0.0",
            Path::new(".codex/AGENTS.md"),
        )
        .expect("frozen Codex onboarding inventory");
        assert_eq!(
            inventory.source_release_digest,
            USER_070_CODEX_ONBOARDING_SOURCE_DIGEST
        );
        assert_eq!(inventory.entries.len(), 27);

        let request = InventoryAuthentication {
            product_version: "0.7.0",
            installed_host_version_range: ">=0.145.0 <1.0.0",
            source_release_digest: &inventory.source_release_digest,
            installed_entries: &inventory.entries,
            installed_guidance_path: Path::new(".codex/AGENTS.md"),
            current_guidance_path: Path::new(".codex/AGENTS.md"),
            current_source_release_digest:
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            current_entries: &[],
            authenticated_prior: None,
        };
        let authenticated =
            authenticated_user_inventory(UserHost::Codex, &request).expect("frozen 0.7 snapshot");
        assert_user_entries_equal(&authenticated.entries, &inventory.entries);
        assert!(historical_070_codex_onboarding_inventory(
            UserHost::Codex,
            ">=9.0.0 <10.0.0",
            Path::new(".codex/AGENTS.md"),
        )
        .is_none());
    }

    #[test]
    fn historical_080_install_supports_retirement_and_rejects_byte_tamper() {
        for host in [UserHost::Codex, UserHost::Claude, UserHost::Antigravity] {
            let temporary = tempdir().expect("tempdir");
            seed_historical_080_user_install(temporary.path(), host);
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let plan = build_plan(&arguments).expect("authenticated 0.8 update plan");
            for (name, _, _) in USER_080_SKILLS {
                for path in historical_080_skill_paths(host, name) {
                    assert!(
                        plan.retired_files.contains_key(Path::new(&path)),
                        "authenticated 0.8 retired Skill path must be planned: {path}"
                    );
                }
            }
            match host {
                UserHost::Codex | UserHost::Claude => {
                    execute(
                        UserOperation::Update,
                        &arguments,
                        &StatefulHostRunner::new(temporary.path(), HostSabotage::None),
                    )
                    .expect("authenticated 0.8 upgrade");
                }
                UserHost::Antigravity => {
                    execute(
                        UserOperation::Update,
                        &arguments,
                        &AntigravityRunner::new(temporary.path()),
                    )
                    .expect("authenticated 0.8 upgrade");
                }
            }
            for (name, _, _) in USER_080_SKILLS {
                for path in historical_080_skill_paths(host, name) {
                    assert!(
                        !temporary.path().join(path).exists(),
                        "authenticated retired Skill path must be removed"
                    );
                }
            }

            let tampered = tempdir().expect("tampered tempdir");
            let manifest = seed_historical_080_user_install(tampered.path(), host);
            let tampered_arguments = args(tampered.path(), host, UserMode::Apply);
            let tampered_path = manifest
                .entries
                .iter()
                .find(|entry| is_managed_ownership(&entry.ownership))
                .map(|entry| tampered.path().join(&entry.path))
                .expect("managed 0.8 path");
            fs::write(&tampered_path, b"tampered frozen 0.8 bytes\n").expect("tamper frozen path");
            let Err(error) = build_plan(&tampered_arguments) else {
                panic!("tampered 0.8 bytes must fail closed");
            };
            assert!(matches!(error, InstallError::Conflict(_)));
        }
    }

    #[test]
    fn authenticated_antigravity_directory_scan_install_migrates_to_native_registry() {
        let temporary = tempdir().expect("tempdir");
        seed_legacy_antigravity_directory_scan_install(temporary.path());
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let plan = build_plan(&arguments).expect("legacy directory-scan migration plan");
        assert!(!plan.prior_antigravity_activation_source);
        assert!(plan
            .retired_files
            .contains_key(Path::new(".gemini/config/plugins/aigent-hive/plugin.json")));

        let runner = AntigravityRunner::new(temporary.path());
        execute(UserOperation::Update, &arguments, &runner).expect("native registry migration");
        assert_eq!(
            probe_antigravity_state(&QualifiedExecutable::synthetic("agy"), &runner)
                .expect("registered state")
                .plugin,
            Some(expected_antigravity_plugin_state())
        );
        let manifest: UserOwnershipManifest = serde_json::from_slice(
            &fs::read(temporary.path().join(".hive/install/antigravity.json")).expect("manifest"),
        )
        .expect("manifest JSON");
        assert_eq!(manifest.host_version_range, ">=1.1.7 <1.2.0");
        assert!(manifest
            .entries
            .iter()
            .any(|entry| entry.path == format!("{ANTIGRAVITY_SOURCE_RELATIVE}/plugin.json")));
        assert!(!manifest
            .entries
            .iter()
            .any(|entry| entry.path.starts_with(ANTIGRAVITY_STAGE_RELATIVE)));
    }

    #[test]
    fn antigravity_legacy_migration_preserves_foreign_stage_entries() {
        let temporary = tempdir().expect("tempdir");
        seed_legacy_antigravity_directory_scan_install(temporary.path());
        let foreign = temporary
            .path()
            .join(ANTIGRAVITY_STAGE_RELATIVE)
            .join("foreign.txt");
        fs::write(&foreign, b"foreign stage bytes\n").expect("foreign stage");
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let runner = AntigravityRunner::new(temporary.path());

        let error = execute(UserOperation::Update, &arguments, &runner)
            .err()
            .expect("foreign legacy stage must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(
            fs::read(foreign).expect("foreign stage preserved"),
            b"foreign stage bytes\n"
        );
        assert!(!runner
            .calls
            .lock()
            .expect("calls")
            .iter()
            .any(|call| call.starts_with("plugin install ")));
    }

    #[test]
    fn modified_authenticated_prior_only_path_fails_closed_and_is_preserved() {
        let temporary = tempdir().expect("tempdir");
        seed_historical_user_install(temporary.path(), UserHost::Antigravity);
        let retired = historical_user_files(UserHost::Antigravity)
            .into_iter()
            .find(|(_, file)| file.executable)
            .map(|(path, _)| path)
            .expect("retired executable");
        let target = temporary.path().join(&retired);
        fs::write(&target, b"locally modified retired skill\n").expect("local modification");

        let error = build_plan(&args(
            temporary.path(),
            UserHost::Antigravity,
            UserMode::DryRun,
        ))
        .err()
        .expect("modified prior-only path must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(
            fs::read(target).expect("modified bytes preserved"),
            b"locally modified retired skill\n"
        );
    }

    #[test]
    fn reoccupied_authenticated_prior_only_path_fails_preflight_and_is_preserved() {
        let temporary = tempdir().expect("tempdir");
        seed_historical_user_install(temporary.path(), UserHost::Antigravity);
        let retired = historical_user_files(UserHost::Antigravity)
            .into_iter()
            .find(|(_, file)| file.executable)
            .map(|(path, _)| path)
            .expect("retired executable");
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("historical upgrade plan");
        let target = temporary.path().join(&retired);
        fs::write(&target, b"foreign reoccupation after planning\n").expect("reoccupation");

        let error = apply_plan(&arguments, &mut plan, None, None, None)
            .err()
            .expect("reoccupied prior-only path must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(
            fs::read(target).expect("foreign bytes preserved"),
            b"foreign reoccupation after planning\n"
        );
        assert!(!temporary
            .path()
            .join(".hive/backups/user-install/antigravity")
            .exists());
        assert!(!temporary
            .path()
            .join(".hive/install-transactions/antigravity.json")
            .exists());
    }

    #[test]
    fn failed_host_activation_rolls_back_authenticated_prior_only_deletion() {
        let temporary = tempdir().expect("tempdir");
        let historical = seed_historical_user_install(temporary.path(), UserHost::Codex);
        let retired = historical_user_files(UserHost::Codex)
            .into_iter()
            .find(|(_, file)| file.executable)
            .map(|(path, file)| (path, file.bytes))
            .expect("retired executable");
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let runner = StatefulHostRunner::new(
            temporary.path(),
            HostSabotage::FailBeforeMarketplaceMutation,
        );

        let _error = execute(UserOperation::Update, &arguments, &runner)
            .err()
            .expect("host activation failure");
        assert_eq!(
            fs::read(temporary.path().join(&retired.0)).expect("retired file restored"),
            retired.1
        );
        let restored: UserOwnershipManifest = serde_json::from_slice(
            &fs::read(temporary.path().join(".hive/install/codex.json"))
                .expect("restored manifest"),
        )
        .expect("restored manifest JSON");
        assert_eq!(restored.product_version, historical.product_version);
        assert_eq!(
            restored.source_release_digest,
            historical.source_release_digest
        );
    }

    #[test]
    fn recovery_restores_authenticated_prior_only_file_deleted_by_interrupted_upgrade() {
        let temporary = tempdir().expect("tempdir");
        seed_historical_user_install(temporary.path(), UserHost::Antigravity);
        let retired = historical_user_files(UserHost::Antigravity)
            .into_iter()
            .find(|(_, file)| file.executable)
            .map(|(path, file)| (path, file.bytes))
            .expect("retired executable");
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("historical upgrade plan");
        apply_plan(&arguments, &mut plan, None, None, None).expect("interrupted filesystem apply");
        assert!(!temporary.path().join(&retired.0).exists());

        recover(&arguments, &FakeRunner::new()).expect("recover interrupted upgrade");
        assert_eq!(
            fs::read(temporary.path().join(&retired.0)).expect("retired file restored"),
            retired.1
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(temporary.path().join(&retired.0))
                    .expect("retired metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o755
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn pinned_user_root_and_nofollow_ancestors_preserve_swap_targets() {
        use std::os::unix::fs::symlink;

        let container = tempdir().expect("container");
        let user = container.path().join("user");
        let moved = container.path().join("pinned-user");
        let outside = container.path().join("outside");
        fs::create_dir(&user).expect("user");
        fs::create_dir(&outside).expect("outside");
        fs::write(outside.join("sentinel"), b"outside bytes").expect("sentinel");
        let arguments = args(&user, UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("plan");
        fs::rename(&user, &moved).expect("retarget root");
        symlink(&outside, &user).expect("root symlink");
        apply_plan(&arguments, &mut plan, None, None, None).expect("pinned apply");
        assert!(moved
            .join(".hive/marketplaces/antigravity/plugins/aigent-hive/plugin.json")
            .is_file());
        assert_eq!(
            fs::read(outside.join("sentinel")).expect("outside"),
            b"outside bytes"
        );
        assert!(!outside.join(".gemini").exists());

        let second = tempdir().expect("second");
        let outside = tempdir().expect("outside");
        fs::create_dir_all(second.path().join(".gemini")).expect("gemini");
        let arguments = args(second.path(), UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("plan");
        symlink(outside.path(), second.path().join(".gemini/config")).expect("ancestor symlink");
        let error = apply_plan(&arguments, &mut plan, None, None, None)
            .err()
            .expect("ancestor swap conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert!(!outside.path().join("plugins").exists());
    }

    #[test]
    fn late_sorted_drift_fails_before_any_backup_or_journal_write() {
        let temporary = tempdir().expect("tempdir");
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("plan");
        let late = plan
            .files
            .keys()
            .next_back()
            .expect("late planned path")
            .clone();
        let path = temporary.path().join(&late);
        fs::create_dir_all(path.parent().expect("late parent")).expect("late parent");
        fs::write(&path, b"late concurrent bytes\n").expect("late drift");
        let before = snapshot_user_tree(temporary.path());
        let error = apply_plan(&arguments, &mut plan, None, None, None)
            .err()
            .expect("late drift conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(snapshot_user_tree(temporary.path()), before);
        assert!(!temporary
            .path()
            .join(".hive/backups/user-install/antigravity")
            .exists());
        assert!(!temporary
            .path()
            .join(".hive/install-transactions/antigravity.json")
            .exists());
    }

    #[cfg(unix)]
    #[test]
    fn mode_only_drift_fails_before_any_backup_or_journal_write() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempdir().expect("tempdir");
        let guidance = temporary.path().join(".gemini/GEMINI.md");
        fs::create_dir_all(guidance.parent().expect("guidance parent")).expect("guidance parent");
        fs::write(&guidance, b"foreign guidance\n").expect("guidance");
        fs::set_permissions(&guidance, fs::Permissions::from_mode(0o644)).expect("initial mode");
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("plan");
        fs::set_permissions(&guidance, fs::Permissions::from_mode(0o600)).expect("mode drift");
        let error = apply_plan(&arguments, &mut plan, None, None, None)
            .err()
            .expect("mode drift conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(
            fs::metadata(&guidance)
                .expect("guidance metadata")
                .permissions()
                .mode()
                & 0o7777,
            0o600
        );
        assert!(!temporary
            .path()
            .join(".hive/backups/user-install/antigravity")
            .exists());
        assert!(!temporary
            .path()
            .join(".hive/install-transactions/antigravity.json")
            .exists());
    }

    #[test]
    fn destination_reoccupation_preserves_foreign_and_claimed_prior_bytes() {
        let temporary = tempdir().expect("tempdir");
        let root = open_user_root(temporary.path()).expect("root capability");
        let relative = Path::new(".gemini/config/plugins/aigent-hive/plugin.json");
        create_new_exclusive(
            &root,
            relative,
            b"prior owned bytes\n",
            FilePermissions::default(),
        )
        .expect("seed prior");
        let barrier = |parent: &Dir, name: &OsStr, _claim: &Dir| {
            let mut options = OpenOptions::new();
            options
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            let mut file = parent.open_with(name, &options).expect("foreign claim");
            file.write_all(b"foreign concurrent bytes\n")
                .expect("foreign bytes");
            file.sync_all().expect("foreign sync");
        };
        let error = cas_activate_with_barrier(
            &root,
            relative,
            Some(ExpectedFile {
                bytes: b"prior owned bytes\n",
                permissions: file_permissions(&root, relative).expect("prior permissions"),
            }),
            Some(b"incoming bytes\n"),
            FilePermissions::default(),
            Some(&barrier),
        )
        .expect_err("destination conflict");
        assert!(matches!(error, InstallError::Verification(_)));
        assert_eq!(
            read_optional_regular(&root, relative, MAX_USER_FILE_BYTES)
                .expect("read foreign")
                .expect("foreign exists"),
            b"foreign concurrent bytes\n"
        );
        assert_eq!(
            read_optional_regular(&root, &recovery_locator(relative), MAX_USER_FILE_BYTES)
                .expect("read recovery")
                .expect("recovery exists"),
            b"prior owned bytes\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn permission_change_after_inode_claim_fails_closed_and_preserves_third_party_mode() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempdir().expect("tempdir");
        let root = open_user_root(temporary.path()).expect("root capability");
        let relative = Path::new(".gemini/config/plugins/aigent-hive/plugin.json");
        let expected_permissions = FilePermissions {
            executable: false,
            unix_mode: Some(0o644),
        };
        create_new_exclusive(
            &root,
            relative,
            b"prior owned bytes\n",
            expected_permissions,
        )
        .expect("seed prior");
        let barrier = |_parent: &Dir, _name: &OsStr, claim: &Dir| {
            let file = claim.open("claimed.bin").expect("open claimed inode");
            file.set_permissions(cap_primitives::fs::Permissions::from_mode(0o600))
                .expect("third-party chmod");
        };

        let error = cas_activate_with_barrier(
            &root,
            relative,
            Some(ExpectedFile {
                bytes: b"prior owned bytes\n",
                permissions: expected_permissions,
            }),
            Some(b"incoming bytes\n"),
            FilePermissions {
                executable: false,
                unix_mode: Some(0o644),
            },
            Some(&barrier),
        )
        .expect_err("permission race must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(
            fs::read(temporary.path().join(relative)).expect("preserved bytes"),
            b"prior owned bytes\n"
        );
        assert_eq!(
            fs::metadata(temporary.path().join(relative))
                .expect("preserved metadata")
                .permissions()
                .mode()
                & 0o7777,
            0o600
        );
        assert!(!temporary
            .path()
            .join(".hive/install-transactions/antigravity.json")
            .exists());
        assert!(!temporary.path().join(recovery_locator(relative)).exists());
    }

    #[test]
    fn recovery_reconciles_published_replacement_with_retained_prior_claim() {
        let temporary = tempdir().expect("tempdir");
        let root = open_user_root(temporary.path()).expect("root capability");
        let relative = Path::new(".gemini/config/plugins/aigent-hive/plugin.json");
        let permissions = installed_file_permissions(false);
        create_new_exclusive(&root, relative, b"prior owned bytes\n", permissions)
            .expect("seed prior");
        let (parent, destination) = capability_parent(&root, relative, false)
            .expect("parent")
            .expect("existing parent");
        let claim_name = claim_name(relative);
        parent.create_dir(&claim_name).expect("claim");
        let claim = parent.open_dir_nofollow(&claim_name).expect("claim dir");
        parent
            .rename(&destination, &claim, OsStr::new("claimed.bin"))
            .expect("claim prior");
        stage_file(
            &claim,
            "replacement.bin",
            b"installed bytes\n",
            permissions,
            relative,
        )
        .expect("replacement");
        claim
            .hard_link("replacement.bin", &parent, &destination)
            .expect("publish");
        drop(claim);
        let entry = UserBackupEntry {
            path: portable(relative),
            existed: true,
            digest: Some(sha256_digest(b"prior owned bytes\n")),
            installed_digest: Some(sha256_digest(b"installed bytes\n")),
            installed_executable: Some(permissions.executable),
            installed_unix_mode: permissions.unix_mode,
            executable: permissions.executable,
            unix_mode: permissions.unix_mode,
        };
        reconcile_retained_claim(&root, relative, &entry).expect("reconcile");
        assert_eq!(
            read_optional_regular(&root, relative, MAX_USER_FILE_BYTES)
                .expect("prior read")
                .expect("prior exists"),
            b"prior owned bytes\n"
        );
        assert!(
            read_optional_regular(&root, &recovery_locator(relative), MAX_USER_FILE_BYTES)
                .expect("claim read")
                .is_none()
        );
    }

    #[test]
    fn recovery_reconciles_retained_claim_from_interrupted_deletion() {
        let temporary = tempdir().expect("tempdir");
        let root = open_user_root(temporary.path()).expect("root capability");
        let relative = Path::new(".gemini/config/plugins/aigent-hive/bin/retired");
        create_new_exclusive(
            &root,
            relative,
            b"prior executable bytes\n",
            FilePermissions {
                executable: true,
                unix_mode: installed_unix_mode(true),
            },
        )
        .expect("seed prior");
        let (parent, destination) = capability_parent(&root, relative, false)
            .expect("parent")
            .expect("existing parent");
        let claim_name = claim_name(relative);
        parent.create_dir(&claim_name).expect("claim directory");
        let claim = parent.open_dir_nofollow(&claim_name).expect("open claim");
        parent
            .rename(&destination, &claim, OsStr::new("claimed.bin"))
            .expect("claim prior");
        drop(claim);
        let entry = UserBackupEntry {
            path: portable(relative),
            existed: true,
            digest: Some(sha256_digest(b"prior executable bytes\n")),
            installed_digest: None,
            installed_executable: None,
            installed_unix_mode: None,
            executable: true,
            unix_mode: installed_unix_mode(true),
        };

        reconcile_retained_claim(&root, relative, &entry).expect("reconcile deletion claim");
        assert_eq!(
            read_optional_regular(&root, relative, MAX_USER_FILE_BYTES)
                .expect("read restored")
                .expect("restored prior"),
            b"prior executable bytes\n"
        );
        assert!(
            read_optional_regular(&root, &recovery_locator(relative), MAX_USER_FILE_BYTES)
                .expect("claim read")
                .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    #[allow(clippy::too_many_lines)]
    fn recovery_authority_rejects_chmod_only_drift_for_every_file_state() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempdir().expect("tempdir");
        let root = open_user_root(temporary.path()).expect("root capability");
        let cases = [
            (
                "existing-prior",
                true,
                b"prior bytes\n".as_slice(),
                Some(b"installed bytes\n".as_slice()),
                0o644,
                Some(0o755),
                b"prior bytes\n".as_slice(),
                0o600,
            ),
            (
                "existing-installed",
                true,
                b"prior bytes\n".as_slice(),
                Some(b"installed bytes\n".as_slice()),
                0o644,
                Some(0o755),
                b"installed bytes\n".as_slice(),
                0o644,
            ),
            (
                "created-executable",
                false,
                b"".as_slice(),
                Some(b"created executable\n".as_slice()),
                0o644,
                Some(0o755),
                b"created executable\n".as_slice(),
                0o644,
            ),
            (
                "retired-prior",
                true,
                b"retired executable\n".as_slice(),
                None,
                0o755,
                None,
                b"retired executable\n".as_slice(),
                0o644,
            ),
        ];
        for (name, existed, prior, installed, prior_mode, installed_mode, live, drifted_mode) in
            cases
        {
            let relative = PathBuf::from(format!(".hive/recovery-mode-cases/{name}"));
            let target = temporary.path().join(&relative);
            fs::create_dir_all(target.parent().expect("case parent")).expect("case parent");
            fs::write(&target, live).expect("live bytes");
            fs::set_permissions(&target, fs::Permissions::from_mode(drifted_mode))
                .expect("drifted mode");
            let entry = UserBackupEntry {
                path: portable(&relative),
                existed,
                digest: existed.then(|| sha256_digest(prior)),
                installed_digest: installed.map(sha256_digest),
                installed_executable: installed_mode.map(|mode| mode & 0o111 != 0),
                installed_unix_mode: installed_mode,
                executable: prior_mode & 0o111 != 0,
                unix_mode: existed.then_some(prior_mode),
            };

            let error = current_installed_state(&root, &relative, &entry)
                .err()
                .expect("chmod-only drift must conflict");
            assert!(matches!(error, InstallError::Conflict(_)), "{name}");
            assert_eq!(fs::read(&target).expect("preserved bytes"), live, "{name}");
            assert_eq!(
                fs::metadata(&target)
                    .expect("preserved metadata")
                    .permissions()
                    .mode()
                    & 0o7777,
                drifted_mode,
                "{name}"
            );
        }
        let legacy_relative = Path::new(".hive/recovery-mode-cases/legacy-installed");
        fs::write(
            temporary.path().join(legacy_relative),
            b"legacy installed bytes\n",
        )
        .expect("legacy bytes");
        fs::set_permissions(
            temporary.path().join(legacy_relative),
            fs::Permissions::from_mode(0o644),
        )
        .expect("legacy mode");
        let legacy = UserBackupEntry {
            path: portable(legacy_relative),
            existed: false,
            digest: None,
            installed_digest: Some(sha256_digest(b"legacy installed bytes\n")),
            installed_executable: None,
            installed_unix_mode: None,
            executable: false,
            unix_mode: None,
        };
        assert!(matches!(
            current_installed_state(&root, legacy_relative, &legacy),
            Err(InstallError::Verification(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn recovery_preserves_chmod_drift_on_retained_claim_object() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempdir().expect("tempdir");
        let root = open_user_root(temporary.path()).expect("root capability");
        let relative = Path::new(".gemini/config/plugins/aigent-hive/bin/retired-mode-drift");
        create_new_exclusive(
            &root,
            relative,
            b"prior executable bytes\n",
            FilePermissions {
                executable: true,
                unix_mode: Some(0o755),
            },
        )
        .expect("seed prior");
        let (parent, destination) = capability_parent(&root, relative, false)
            .expect("parent")
            .expect("existing parent");
        let claim_name = claim_name(relative);
        parent.create_dir(&claim_name).expect("claim directory");
        let claim = parent.open_dir_nofollow(&claim_name).expect("open claim");
        parent
            .rename(&destination, &claim, OsStr::new("claimed.bin"))
            .expect("claim prior");
        let claimed = claim.open("claimed.bin").expect("claimed object");
        claimed
            .set_permissions(cap_primitives::fs::Permissions::from_mode(0o600))
            .expect("claim chmod");
        drop(claimed);
        drop(claim);
        let entry = UserBackupEntry {
            path: portable(relative),
            existed: true,
            digest: Some(sha256_digest(b"prior executable bytes\n")),
            installed_digest: None,
            installed_executable: None,
            installed_unix_mode: None,
            executable: true,
            unix_mode: Some(0o755),
        };

        let error = reconcile_retained_claim(&root, relative, &entry)
            .expect_err("claim mode drift must conflict");
        assert!(matches!(error, InstallError::Verification(_)));
        assert!(!temporary.path().join(relative).exists());
        let claim_path = temporary.path().join(recovery_locator(relative));
        assert_eq!(
            fs::read(&claim_path).expect("claimed bytes preserved"),
            b"prior executable bytes\n"
        );
        assert_eq!(
            fs::metadata(claim_path)
                .expect("claimed metadata")
                .permissions()
                .mode()
                & 0o7777,
            0o600
        );
    }

    #[test]
    fn antigravity_uses_agy_native_registration_and_preserves_foreign_guidance() {
        let temporary = tempdir().expect("tempdir");
        fs::create_dir(temporary.path().join(".gemini")).expect("gemini");
        fs::write(
            temporary.path().join(".gemini/GEMINI.md"),
            b"foreign guidance\n",
        )
        .expect("guidance");
        let runner = AntigravityRunner::new(temporary.path());
        let apply = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let result = execute(UserOperation::Install, &apply, &runner).expect("apply");
        assert_eq!(
            result
                .data
                .as_ref()
                .expect("data")
                .get("qualified_host_version"),
            Some(&serde_json::Value::String("1.1.7".to_owned()))
        );
        assert_eq!(
            result
                .data
                .as_ref()
                .expect("data")
                .get("host_version_range"),
            Some(&serde_json::Value::String(">=1.1.7 <1.2.0".to_owned()))
        );
        let manifest: UserOwnershipManifest = serde_json::from_slice(
            &fs::read(temporary.path().join(".hive/install/antigravity.json")).expect("manifest"),
        )
        .expect("manifest JSON");
        assert_ne!(
            manifest.source_release_digest,
            sha256_digest(&[]),
            "Antigravity plugin and Skill projections must bind the source release"
        );
        assert!(temporary
            .path()
            .join(".hive/marketplaces/antigravity/plugins/aigent-hive/plugin.json")
            .is_file());
        assert!(temporary
            .path()
            .join(".gemini/config/plugins/aigent-hive/skills/user-setup/SKILL.md")
            .is_file());
        assert!(runner
            .calls
            .lock()
            .expect("calls")
            .iter()
            .any(|call| call.starts_with("plugin install ")));
        let validate = args(temporary.path(), UserHost::Antigravity, UserMode::Validate);
        execute(UserOperation::Install, &validate, &runner).expect("validate");
        let recover_args = args(temporary.path(), UserHost::Antigravity, UserMode::Recover);
        recover(&recover_args, &runner).expect("recover");
        assert_eq!(
            fs::read(temporary.path().join(".gemini/GEMINI.md")).expect("guidance"),
            b"foreign guidance\n"
        );
    }

    #[test]
    fn antigravity_fresh_install_preserves_occupied_host_stage() {
        for foreign_relative in ["foreign.txt", "empty/placeholder.txt"] {
            let temporary = tempdir().expect("tempdir");
            let foreign = temporary
                .path()
                .join(ANTIGRAVITY_STAGE_RELATIVE)
                .join(foreign_relative);
            fs::create_dir_all(foreign.parent().expect("foreign parent")).expect("foreign parent");
            fs::write(&foreign, b"foreign stage bytes\n").expect("foreign stage");
            let runner = AntigravityRunner::new(temporary.path());
            let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);

            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("occupied fresh stage must conflict");
            assert!(matches!(error, InstallError::Conflict(_)));
            assert_eq!(
                fs::read(foreign).expect("foreign stage preserved"),
                b"foreign stage bytes\n"
            );
            assert!(!temporary
                .path()
                .join(".hive/install/antigravity.json")
                .exists());
        }
    }

    #[test]
    fn antigravity_fresh_install_preserves_empty_occupied_host_stage() {
        let temporary = tempdir().expect("tempdir");
        let stage = temporary.path().join(ANTIGRAVITY_STAGE_RELATIVE);
        fs::create_dir_all(&stage).expect("empty foreign stage");
        let runner = AntigravityRunner::new(temporary.path());
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);

        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("empty occupied stage must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert!(stage.is_dir());
    }

    #[test]
    fn antigravity_refresh_preserves_tampered_or_extended_host_stage() {
        for case in ["tampered", "extended"] {
            let temporary = tempdir().expect("tempdir");
            let runner = AntigravityRunner::new(temporary.path());
            let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
            execute(UserOperation::Install, &arguments, &runner).expect("initial install");
            let stage = temporary.path().join(ANTIGRAVITY_STAGE_RELATIVE);
            let protected = if case == "tampered" {
                stage.join("plugin.json")
            } else {
                stage.join("foreign.txt")
            };
            fs::write(&protected, b"locally protected stage bytes\n").expect("stage drift");

            let error = execute(UserOperation::Update, &arguments, &runner)
                .err()
                .expect("drifted host stage must conflict");
            assert!(matches!(error, InstallError::Conflict(_)));
            assert_eq!(
                fs::read(protected).expect("stage drift preserved"),
                b"locally protected stage bytes\n"
            );
            assert_eq!(
                runner
                    .calls
                    .lock()
                    .expect("calls")
                    .iter()
                    .filter(|call| call.starts_with("plugin install "))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn antigravity_rechecks_stage_at_the_native_mutation_boundary() {
        let temporary = tempdir().expect("tempdir");
        let runner = AntigravityRunner::with_stage_drift_on_probe(temporary.path(), 2);
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);

        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("racing stage drift must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(
            fs::read(
                temporary
                    .path()
                    .join(ANTIGRAVITY_STAGE_RELATIVE)
                    .join("foreign-race.txt")
            )
            .expect("foreign racing stage preserved"),
            b"foreign racing stage bytes\n"
        );
        assert!(!runner
            .calls
            .lock()
            .expect("calls")
            .iter()
            .any(|call| call.starts_with("plugin install ")));
        assert!(!temporary
            .path()
            .join(".hive/install/antigravity.json")
            .exists());
    }

    #[test]
    fn antigravity_stage_reader_caps_directory_count_and_depth() {
        let depth_root = tempdir().expect("depth tempdir");
        let mut nested = depth_root.path().join(ANTIGRAVITY_STAGE_RELATIVE);
        for index in 0..=MAX_ANTIGRAVITY_STAGE_DEPTH {
            nested.push(format!("d{index}"));
        }
        fs::create_dir_all(&nested).expect("deep stage");
        let depth_cap = open_user_root(depth_root.path()).expect("depth root");
        let error = read_optional_regular_tree(&depth_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))
            .expect_err("deep stage must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));

        let count_root = tempdir().expect("count tempdir");
        let stage = count_root.path().join(ANTIGRAVITY_STAGE_RELATIVE);
        fs::create_dir_all(&stage).expect("stage");
        for index in 0..=MAX_ANTIGRAVITY_STAGE_DIRECTORIES {
            fs::create_dir(stage.join(format!("d{index}"))).expect("stage directory");
        }
        let count_cap = open_user_root(count_root.path()).expect("count root");
        let error = read_optional_regular_tree(&count_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))
            .expect_err("wide stage must conflict");
        assert!(matches!(error, InstallError::Conflict(_)));
    }

    #[test]
    fn validate_checks_native_registration_and_exact_antigravity_stage() {
        let temporary = tempdir().expect("tempdir");
        let runner = AntigravityRunner::new(temporary.path());
        let apply = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        execute(UserOperation::Install, &apply, &runner).expect("initial install");
        let validate = args(temporary.path(), UserHost::Antigravity, UserMode::Validate);

        *runner.plugin_installed.lock().expect("plugin") = false;
        let error = execute(UserOperation::Install, &validate, &runner)
            .err()
            .expect("missing native registration must fail validation");
        assert!(matches!(error, InstallError::Verification(_)));

        *runner.plugin_installed.lock().expect("plugin") = true;
        let stage = temporary.path().join(ANTIGRAVITY_STAGE_RELATIVE);
        fs::remove_dir_all(&stage).expect("remove stage");
        let error = execute(UserOperation::Install, &validate, &runner)
            .err()
            .expect("missing host stage must fail validation");
        assert!(matches!(error, InstallError::Verification(_)));

        AntigravityRunner::copy_tree(&temporary.path().join(ANTIGRAVITY_SOURCE_RELATIVE), &stage);
        let foreign = stage.join("foreign.txt");
        fs::write(&foreign, b"foreign stage bytes\n").expect("foreign stage");
        let error = execute(UserOperation::Install, &validate, &runner)
            .err()
            .expect("foreign host stage must fail validation");
        assert!(matches!(error, InstallError::Verification(_)));
        assert_eq!(
            fs::read(foreign).expect("foreign stage preserved"),
            b"foreign stage bytes\n"
        );
    }

    #[test]
    fn validate_checks_codex_and_claude_native_registration() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
            let apply = args(temporary.path(), host, UserMode::Apply);
            execute(UserOperation::Install, &apply, &runner).expect("initial install");
            *runner.plugin_installed.lock().expect("plugin") = false;
            let validate = args(temporary.path(), host, UserMode::Validate);
            let error = execute(UserOperation::Install, &validate, &runner)
                .err()
                .expect("missing native registration must fail validation");
            assert!(matches!(error, InstallError::Verification(_)));
        }
    }

    #[test]
    fn antigravity_refresh_and_recovery_restore_registered_prior_bundle() {
        let temporary = tempdir().expect("tempdir");
        let runner = AntigravityRunner::new(temporary.path());
        let install = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        execute(UserOperation::Install, &install, &runner).expect("initial install");
        execute(UserOperation::Update, &install, &runner).expect("native refresh");

        let install_calls = runner
            .calls
            .lock()
            .expect("calls")
            .iter()
            .filter(|call| call.starts_with("plugin install "))
            .count();
        assert_eq!(install_calls, 2);

        let recover_args = args(temporary.path(), UserHost::Antigravity, UserMode::Recover);
        recover(&recover_args, &runner).expect("recover refreshed plugin");
        assert_eq!(
            probe_antigravity_state(&QualifiedExecutable::synthetic("agy"), &runner,)
                .expect("registered state")
                .plugin,
            Some(expected_antigravity_plugin_state())
        );
        assert_eq!(
            fs::read(
                temporary
                    .path()
                    .join(ANTIGRAVITY_SOURCE_RELATIVE)
                    .join("skills/user-setup/SKILL.md")
            )
            .expect("source Skill"),
            fs::read(
                temporary
                    .path()
                    .join(ANTIGRAVITY_STAGE_RELATIVE)
                    .join("skills/user-setup/SKILL.md")
            )
            .expect("staged Skill")
        );
    }

    #[test]
    fn antigravity_probe_parser_accepts_cli_shapes_and_rejects_duplicates() {
        assert_eq!(
            parse_antigravity_plugin_state(b"No imported plugins.\n").expect("empty"),
            None
        );
        let valid = serde_json::to_vec(&json!({
            "imports": [{
                "name": "aigent-hive",
                "source": "antigravity",
                "importedAt": "2026-07-26T00:00:00Z",
                "components": ["skills"]
            }]
        }))
        .expect("valid JSON");
        assert_eq!(
            parse_antigravity_plugin_state(&valid).expect("valid state"),
            Some(expected_antigravity_plugin_state())
        );
        let duplicate = serde_json::to_vec(&json!({
            "imports": [
                {
                    "name": "aigent-hive",
                    "source": "antigravity",
                    "importedAt": "2026-07-26T00:00:00Z",
                    "components": ["skills"]
                },
                {
                    "name": "aigent-hive",
                    "source": "antigravity",
                    "importedAt": "2026-07-26T00:00:01Z",
                    "components": ["skills"]
                }
            ]
        }))
        .expect("duplicate JSON");
        assert!(matches!(
            parse_antigravity_plugin_state(&duplicate),
            Err(InstallError::Verification(_))
        ));
    }

    #[test]
    fn codex_activation_uses_only_native_fixed_argv() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        execute(UserOperation::Install, &arguments, &runner).expect("install");
        let calls = runner.calls.lock().expect("calls");
        let marketplace = arguments
            .user_root
            .join(".hive/marketplaces/codex")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            *calls,
            [
                "--version".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                format!("plugin marketplace add {marketplace} --json"),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin add aigent-hive@aigent-hive --json".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
            ]
        );
    }

    #[test]
    fn codex_marketplace_add_preserves_a_valid_latent_plugin_activation() {
        let temporary = tempdir().expect("tempdir");
        let runner =
            StatefulHostRunner::new(temporary.path(), HostSabotage::LatentCodexPluginActivation);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);

        execute(UserOperation::Install, &arguments, &runner).expect("install");
        assert_eq!(runner.external_state(), (true, true));
        let calls = runner.calls.lock().expect("calls");
        assert!(calls
            .iter()
            .any(|call| call.starts_with("plugin marketplace add ")));
        assert!(!calls
            .iter()
            .any(|call| call == "plugin add aigent-hive@aigent-hive --json"));
        drop(calls);

        recover(&arguments, &runner).expect("recover");
        assert_eq!(runner.external_state(), (false, true));
        let calls = runner.calls.lock().expect("calls");
        assert!(!calls
            .iter()
            .any(|call| call == "plugin remove aigent-hive@aigent-hive --json"));
        assert!(calls
            .iter()
            .any(|call| call == "plugin marketplace remove aigent-hive --json"));
    }

    #[test]
    fn codex_marketplace_add_reinstalls_an_exact_stale_hive_plugin() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(
            temporary.path(),
            HostSabotage::StaleCodexHivePluginActivation,
        );
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);

        execute(UserOperation::Install, &arguments, &runner)
            .expect("reinstall exact stale Hive plugin");

        assert_eq!(runner.external_state(), (true, true));
        let calls = runner.calls.lock().expect("calls");
        let marketplace_add = calls
            .iter()
            .position(|call| call.starts_with("plugin marketplace add "))
            .expect("marketplace add");
        let stale_remove = calls
            .iter()
            .position(|call| call == "plugin remove aigent-hive@aigent-hive --json")
            .expect("stale plugin removal");
        let plugin_add = calls
            .iter()
            .position(|call| call == "plugin add aigent-hive@aigent-hive --json")
            .expect("plugin reinstall");
        assert!(marketplace_add < stale_remove && stale_remove < plugin_add);
    }

    #[test]
    fn claude_activation_uses_only_native_fixed_argv() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
        let arguments = args(temporary.path(), UserHost::Claude, UserMode::Apply);
        execute(UserOperation::Install, &arguments, &runner).expect("Claude install");
        let marketplace = arguments
            .user_root
            .join(".hive/marketplaces/claude")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            *runner.calls.lock().expect("calls"),
            [
                "--version".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                format!("plugin validate {marketplace}"),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                format!("plugin marketplace add {marketplace} --scope user"),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin install aigent-hive@aigent-hive --scope user".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
                "plugin marketplace list --json".to_owned(),
                "plugin list --json".to_owned(),
            ]
        );
    }

    #[test]
    fn unavailable_host_cli_is_detected_before_user_files_change() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &UnavailableRunner)
                .err()
                .expect("missing host CLI");
            assert!(matches!(error, InstallError::Unsupported(_)));
            assert_eq!(fs::read_dir(temporary.path()).expect("root").count(), 0);
        }
    }

    #[test]
    fn failed_native_activation_rolls_back_active_user_files() {
        let temporary = tempdir().expect("tempdir");
        let runner =
            StatefulHostRunner::new(temporary.path(), HostSabotage::FailBeforePluginMutation);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("activation failure");
        assert!(matches!(error, InstallError::Internal(_)));
        assert!(error
            .message()
            .contains("failed before its exact structured transition"));
        assert!(!temporary.path().join(".codex/AGENTS.md").exists());
        assert!(!temporary.path().join(".hive/install/codex.json").exists());
        assert_eq!(runner.external_state(), (true, false));
        assert!(temporary
            .path()
            .join(".hive/install-transactions/codex.json")
            .is_file());
        assert!(
            temporary
                .path()
                .join(".hive/backups/user-install/codex")
                .is_dir(),
            "recoverable backup remains after automatic rollback"
        );
    }

    #[test]
    fn repeated_install_skips_duplicate_marketplace_registration() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        execute(UserOperation::Install, &arguments, &runner).expect("first install");
        let first_call_count = runner.calls.lock().expect("calls").len();
        execute(UserOperation::Install, &arguments, &runner).expect("second install");
        let calls = runner.calls.lock().expect("calls");
        let repeated = &calls[first_call_count..];
        assert!(!repeated
            .iter()
            .any(|call| call.starts_with("plugin marketplace add ")));
        assert!(repeated
            .iter()
            .any(|call| call == "plugin add aigent-hive@aigent-hive --json"));
    }

    #[cfg(unix)]
    #[test]
    fn backup_and_recovery_preserve_exact_unix_modes() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempdir().expect("tempdir");
        write_operational_setup(temporary.path(), &["antigravity"]);
        fs::create_dir(temporary.path().join(".gemini")).expect("gemini");
        let guidance = temporary.path().join(".gemini/GEMINI.md");
        fs::write(&guidance, b"foreign executable guidance\n").expect("guidance");
        fs::set_permissions(&guidance, fs::Permissions::from_mode(0o700)).expect("guidance mode");
        let suppression = temporary.path().join(".hive/knowledge/suppression.yml");
        fs::create_dir_all(suppression.parent().expect("suppression parent"))
            .expect("knowledge directory");
        fs::write(&suppression, ROOT_SUPPRESSION).expect("suppression");
        fs::set_permissions(&suppression, fs::Permissions::from_mode(0o640))
            .expect("suppression mode");
        let runner = AntigravityRunner::new(temporary.path());
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        execute(UserOperation::Install, &arguments, &runner).expect("apply");
        let manifest: UserOwnershipManifest = serde_json::from_slice(
            &fs::read(temporary.path().join(".hive/install/antigravity.json")).expect("manifest"),
        )
        .expect("manifest JSON");
        let backup = PathBuf::from(manifest.last_backup.expect("backup"));
        assert_eq!(
            fs::metadata(
                temporary
                    .path()
                    .join(&backup)
                    .join("files/.gemini/GEMINI.md")
            )
            .expect("backup guidance")
            .permissions()
            .mode()
                & 0o7777,
            0o700
        );
        assert_eq!(
            fs::metadata(
                temporary
                    .path()
                    .join(&backup)
                    .join("files/.hive/knowledge/suppression.yml")
            )
            .expect("backup suppression")
            .permissions()
            .mode()
                & 0o7777,
            0o640
        );
        recover(&arguments, &runner).expect("recover");
        assert_eq!(
            fs::metadata(&guidance)
                .expect("guidance")
                .permissions()
                .mode()
                & 0o7777,
            0o700
        );
        assert_eq!(
            fs::metadata(&suppression)
                .expect("suppression")
                .permissions()
                .mode()
                & 0o7777,
            0o640
        );
        assert_eq!(
            fs::read(guidance).expect("recovered guidance"),
            b"foreign executable guidance\n"
        );
    }

    #[test]
    fn fresh_install_crash_recovers_from_journal_without_installed_manifest() {
        let temporary = tempdir().expect("tempdir");
        fs::create_dir(temporary.path().join(".gemini")).expect("gemini");
        fs::write(
            temporary.path().join(".gemini/GEMINI.md"),
            b"foreign guidance\n",
        )
        .expect("guidance");
        let arguments = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        let mut plan = build_plan(&arguments).expect("plan");
        apply_plan(&arguments, &mut plan, None, None, None).expect("partial apply");
        fs::remove_file(temporary.path().join(".hive/install/antigravity.json"))
            .expect("simulate crash before installed manifest activation");
        recover(&arguments, &FakeRunner::new()).expect("journal recovery");
        assert_eq!(
            fs::read(temporary.path().join(".gemini/GEMINI.md")).expect("guidance"),
            b"foreign guidance\n"
        );
        assert!(!temporary
            .path()
            .join(".hive/install-transactions/antigravity.json")
            .exists());
    }

    #[test]
    fn ambiguous_host_mutations_are_not_compensated() {
        let temporary = tempdir().expect("tempdir");
        let runner =
            StatefulHostRunner::new(temporary.path(), HostSabotage::FailBeforePluginMutation);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("activation failure");
        assert!(matches!(error, InstallError::Internal(_)));
        assert_eq!(runner.external_state(), (true, false));
        let calls = runner.calls.lock().expect("calls");
        assert!(!calls
            .iter()
            .any(|call| call == "plugin marketplace remove aigent-hive --json"));
        assert!(!calls
            .iter()
            .any(|call| call == "plugin remove aigent-hive@aigent-hive --json"));
    }

    #[test]
    fn every_mutation_has_an_explicit_qualified_compensation_surface() {
        for mutation in [
            HostMutation::CodexMarketplaceAdded,
            HostMutation::CodexPluginAdded,
            HostMutation::CodexPluginRefreshed,
        ] {
            assert_eq!(
                compensation_surface(mutation),
                HostCompensationSurface::CodexStructuredJson
            );
        }
        for mutation in [
            HostMutation::ClaudeMarketplaceAdded,
            HostMutation::ClaudePluginInstalled,
            HostMutation::ClaudeMarketplaceRefreshed,
            HostMutation::ClaudePluginRefreshed,
        ] {
            assert_eq!(
                compensation_surface(mutation),
                HostCompensationSurface::ClaudeStructuredJson
            );
        }
    }

    #[test]
    fn codex_probe_parsers_accept_real_shapes_and_reject_ambiguous_duplicates() {
        let marketplace = br#"{
          "marketplaces": [{
            "name": "aigent-hive",
            "root": "/tmp/.hive/marketplaces/codex",
            "marketplaceSource": {"sourceType": "local", "source": "/tmp/source"}
          }]
        }"#;
        assert_eq!(
            parse_codex_marketplace_state(marketplace).expect("marketplace state"),
            Some(CodexMarketplaceState {
                root: normalize_host_path("/tmp/.hive/marketplaces/codex")
            })
        );
        let plugin = br#"{
          "installed": [{
            "pluginId": "aigent-hive@aigent-hive",
            "name": "aigent-hive",
            "marketplaceName": "aigent-hive",
            "version": "0.7.0",
            "installed": true,
            "enabled": true,
            "source": {"source": "local", "path": "/tmp/plugin"},
            "marketplaceSource": {"sourceType": "local", "source": "/tmp/source"}
          }],
          "available": []
        }"#;
        assert_eq!(
            parse_codex_plugin_state(plugin).expect("plugin state"),
            Some(CodexPluginState {
                version: "0.7.0".to_owned(),
                enabled: true,
                source_path: normalize_host_path("/tmp/plugin"),
                marketplace_source: normalize_host_path("/tmp/source")
            })
        );
        let duplicate = br#"{"marketplaces":[
          {"name":"aigent-hive","root":"/tmp/a"},
          {"name":"aigent-hive","root":"/tmp/b"}
        ]}"#;
        assert!(parse_codex_marketplace_state(duplicate).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn codex_probe_normalizes_windows_verbatim_paths() {
        assert_eq!(
            normalize_host_path(r"\\?\C:\Users\hive\marketplace"),
            r"C:\Users\hive\marketplace"
        );
        assert_eq!(
            normalize_host_path(r"\\?\UNC\server\share\marketplace"),
            r"\\server\share\marketplace"
        );
    }

    #[test]
    fn claude_probe_parsers_accept_documented_shapes_and_reject_ambiguous_duplicates() {
        let marketplace = br#"[{
          "name": "aigent-hive",
          "source": "directory",
          "path": "/tmp/.hive/marketplaces/claude",
          "installLocation": "/tmp/.claude/plugins/marketplaces/aigent-hive"
        }]"#;
        assert_eq!(
            parse_claude_marketplace_state(marketplace).expect("marketplace state"),
            Some(ClaudeMarketplaceState {
                source: "directory".to_owned(),
                path: normalize_host_path("/tmp/.hive/marketplaces/claude")
            })
        );
        let plugin = br#"[{
          "id": "aigent-hive@aigent-hive",
          "version": "0.7.0",
          "scope": "user",
          "enabled": true,
          "status": "enabled"
        }]"#;
        assert_eq!(
            parse_claude_plugin_state(plugin).expect("plugin state"),
            Some(ClaudePluginState {
                version: "0.7.0".to_owned(),
                enabled: true,
                scope: "user".to_owned()
            })
        );
        let duplicate = br#"[
          {"name":"aigent-hive","source":"directory","path":"/tmp/a"},
          {"name":"aigent-hive","source":"directory","path":"/tmp/b"}
        ]"#;
        assert!(parse_claude_marketplace_state(duplicate).is_err());
    }

    #[test]
    fn crashes_before_forward_mutation_remain_pending() {
        for sabotage in [
            HostSabotage::FailBeforeMarketplaceMutation,
            HostSabotage::FailBeforePluginMutation,
        ] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(temporary.path(), sabotage);
            let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("forward failure");
            assert!(matches!(error, InstallError::Internal(_)));
            assert!(temporary
                .path()
                .join(".hive/install-transactions/codex.json")
                .is_file());
        }
    }

    #[test]
    fn concurrent_drift_before_first_host_mutation_is_preserved_for_both_hosts() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner =
                StatefulHostRunner::new(temporary.path(), HostSabotage::DriftBeforeFirstMutation);
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("pre-command drift");
            assert!(matches!(error, InstallError::Conflict(_)));
            assert_eq!(runner.external_state(), (true, false));
            let calls = runner.calls.lock().expect("calls");
            assert!(!calls.iter().any(|call| {
                call.starts_with("plugin marketplace add ")
                    || call == "plugin add aigent-hive@aigent-hive --json"
                    || call == "plugin install aigent-hive@aigent-hive --scope user"
            }));
            assert!(!temporary
                .path()
                .join(format!(".hive/install-transactions/{}.json", host.as_str()))
                .exists());
        }
    }

    #[test]
    fn concurrent_drift_before_later_host_mutation_blocks_safe_recovery() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner =
                StatefulHostRunner::new(temporary.path(), HostSabotage::DriftBeforeSecondMutation);
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("later-command drift");
            assert!(matches!(error, InstallError::Internal(_)));
            assert_eq!(runner.external_state(), (true, true));
            let journal = temporary
                .path()
                .join(format!(".hive/install-transactions/{}.json", host.as_str()));
            assert!(journal.is_file());
            let before_recovery = runner.calls.lock().expect("calls").len();
            let recovery_error = recover(&arguments, &runner)
                .err()
                .expect("recovery must preserve drift");
            assert!(matches!(recovery_error, InstallError::Conflict(_)));
            assert_eq!(runner.external_state(), (true, true));
            let calls = runner.calls.lock().expect("calls");
            assert!(!calls[before_recovery..].iter().any(|call| {
                call.contains(" remove ")
                    || call.starts_with("plugin remove ")
                    || call.starts_with("plugin uninstall ")
            }));
            assert!(journal.is_file());
        }
    }

    #[test]
    fn concurrent_drift_before_later_compensation_preserves_external_state_and_evidence() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(
                temporary.path(),
                HostSabotage::DriftBeforeLaterCompensation,
            );
            let arguments = args(temporary.path(), host, UserMode::Apply);
            execute(UserOperation::Install, &arguments, &runner).expect("install");
            let ownership: UserOwnershipManifest = serde_json::from_slice(
                &fs::read(
                    temporary
                        .path()
                        .join(format!(".hive/install/{}.json", host.as_str())),
                )
                .expect("ownership manifest"),
            )
            .expect("ownership JSON");
            let backup_relative = PathBuf::from(ownership.last_backup.expect("backup"));
            let before_recovery = runner.calls.lock().expect("calls").len();
            let error = recover(&arguments, &runner)
                .err()
                .expect("compensation drift");
            assert!(matches!(error, InstallError::Conflict(_)));
            assert_eq!(runner.external_state(), (true, true));
            let calls = runner.calls.lock().expect("calls");
            assert!(!calls[before_recovery..]
                .iter()
                .any(|call| call.contains("marketplace remove")));
            let backup: UserBackupManifest = serde_json::from_slice(
                &fs::read(temporary.path().join(backup_relative).join("manifest.json"))
                    .expect("backup manifest"),
            )
            .expect("backup JSON");
            assert!(backup.pending_host_transition.is_none());
            match backup.host_owned_state.expect("owned state evidence") {
                HostStateSnapshot::Codex(state) => assert!(state.plugin.is_none()),
                HostStateSnapshot::Claude(state) => assert!(state.plugin.is_none()),
                HostStateSnapshot::Antigravity(_) => panic!("unexpected Antigravity state"),
            }
        }
    }

    #[test]
    fn foreign_exact_after_transport_failure_is_never_attributed() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner =
                StatefulHostRunner::new(temporary.path(), HostSabotage::ForeignAfterFailedForward);
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("ambiguous foreign transition");
            assert!(matches!(error, InstallError::Internal(_)));
            assert_eq!(runner.external_state(), (true, false));
            let calls = runner.calls.lock().expect("calls");
            assert!(!calls.iter().any(|call| call.contains("marketplace remove")));
            assert!(temporary
                .path()
                .join(format!(".hive/install-transactions/{}.json", host.as_str()))
                .is_file());
            let backup = transaction_backup(temporary.path(), host);
            assert!(backup.host_owned_state.is_none());
            assert!(backup.pending_host_transition.is_some());
        }
    }

    #[test]
    fn recover_removes_only_a_dangling_hive_codex_marketplace_after_a_failed_probe() {
        let temporary = tempdir().expect("tempdir");
        let knowledge = temporary.path().join(".hive/knowledge/Wiki/user-note.md");
        let preferences = temporary.path().join(".hive/config/user-preferences.json");
        fs::create_dir_all(knowledge.parent().expect("knowledge parent"))
            .expect("knowledge parent");
        fs::create_dir_all(preferences.parent().expect("preferences parent"))
            .expect("preferences parent");
        fs::write(&knowledge, b"# user knowledge\n").expect("knowledge");
        fs::write(&preferences, b"{\"persona\":\"strict\"}\n").expect("preferences");
        let runner =
            StatefulHostRunner::new(temporary.path(), HostSabotage::DanglingCodexMarketplace);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);

        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("failed Codex probe");
        assert!(matches!(error, InstallError::Internal(_)));
        let journal = temporary
            .path()
            .join(".hive/install-transactions/codex.json");
        assert!(journal.is_file());
        assert_eq!(runner.external_state(), (true, false));
        assert!(!temporary
            .path()
            .join(".hive/marketplaces/codex/.agents/plugins/marketplace.json")
            .exists());

        let recovered = recover(&arguments, &runner).expect("recover dangling marketplace");

        assert_eq!(recovered.code, "hive.user-install-recovered");
        assert_eq!(runner.external_state(), (false, false));
        assert!(!journal.exists());
        assert_eq!(
            fs::read(&knowledge).expect("knowledge retained"),
            b"# user knowledge\n"
        );
        assert_eq!(
            fs::read(&preferences).expect("preferences retained"),
            b"{\"persona\":\"strict\"}\n"
        );
        let calls = runner.calls.lock().expect("calls");
        assert!(calls
            .iter()
            .any(|call| call == "plugin marketplace remove aigent-hive --json"));
        assert!(!calls
            .iter()
            .any(|call| call == "plugin remove aigent-hive@aigent-hive --json"));
    }

    #[test]
    fn crash_before_command_then_foreign_exact_state_stays_unresolved() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(
                temporary.path(),
                HostSabotage::FailBeforeMarketplaceMutation,
            );
            let arguments = args(temporary.path(), host, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("command crash");
            assert!(matches!(error, InstallError::Internal(_)));
            runner.seed_marketplace_only();
            let before_recovery = runner.calls.lock().expect("calls").len();
            let recovery = recover(&arguments, &runner)
                .err()
                .expect("pending transition cannot be inferred");
            assert!(matches!(recovery, InstallError::Conflict(_)));
            assert_eq!(runner.external_state(), (true, false));
            let calls = runner.calls.lock().expect("calls");
            assert!(!calls[before_recovery..]
                .iter()
                .any(|call| call.contains("marketplace remove")));
            assert!(temporary
                .path()
                .join(format!(".hive/install-transactions/{}.json", host.as_str()))
                .is_file());
            let backup = transaction_backup(temporary.path(), host);
            assert!(backup.host_owned_state.is_none());
            assert!(backup.pending_host_transition.is_some());
        }
    }

    #[test]
    fn foreign_exact_after_ambiguous_compensation_is_never_authorized() {
        for host in [UserHost::Codex, UserHost::Claude] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(
                temporary.path(),
                HostSabotage::ForeignAfterFailedCompensation,
            );
            let arguments = args(temporary.path(), host, UserMode::Apply);
            execute(UserOperation::Install, &arguments, &runner).expect("install");
            let error = recover(&arguments, &runner)
                .err()
                .expect("ambiguous inverse");
            assert!(matches!(error, InstallError::Internal(_)));
            assert_eq!(runner.external_state(), (true, false));
            let before_retry = runner.calls.lock().expect("calls").len();
            let retry = recover(&arguments, &runner)
                .err()
                .expect("pending inverse remains manual");
            assert!(matches!(retry, InstallError::Conflict(_)));
            let calls = runner.calls.lock().expect("calls");
            assert!(!calls[before_retry..]
                .iter()
                .any(|call| call.contains("marketplace remove")));
            assert!(temporary
                .path()
                .join(format!(".hive/install-transactions/{}.json", host.as_str()))
                .is_file());
            let backup = transaction_backup(temporary.path(), host);
            assert!(backup.pending_host_transition.is_some());
            match backup
                .host_owned_state
                .expect("success-confirmed ownership")
            {
                HostStateSnapshot::Codex(state) => assert!(state.plugin.is_some()),
                HostStateSnapshot::Claude(state) => assert!(state.plugin.is_some()),
                HostStateSnapshot::Antigravity(_) => panic!("unexpected Antigravity state"),
            }
        }
    }

    #[test]
    fn codex_refresh_ambiguity_remains_unresolved_despite_equal_probe() {
        for sabotage in [
            HostSabotage::FailBeforePluginMutation,
            HostSabotage::FailAfterPluginMutation,
        ] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(temporary.path(), sabotage);
            runner.seed_installed_codex_state();
            let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
            let error = execute(UserOperation::Update, &arguments, &runner)
                .err()
                .expect("ambiguous refresh remains unresolved");
            assert!(matches!(error, InstallError::Internal(_)));
            assert_eq!(runner.external_state(), (true, true));
            assert!(temporary
                .path()
                .join(".hive/install-transactions/codex.json")
                .is_file());
        }
    }

    #[test]
    fn codex_recovery_preserves_unowned_post_install_drift() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
        runner.seed_installed_codex_state();
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        execute(UserOperation::Update, &arguments, &runner).expect("update");
        let manifest: UserOwnershipManifest = serde_json::from_slice(
            &fs::read(temporary.path().join(".hive/install/codex.json")).expect("manifest"),
        )
        .expect("manifest JSON");
        let backup = manifest.last_backup.expect("backup");
        runner.clear_codex_state();
        let error = recover(&arguments, &runner)
            .err()
            .expect("unowned drift must block recovery");
        assert!(matches!(error, InstallError::Conflict(_)));
        assert_eq!(runner.external_state(), (false, false));
        assert!(temporary
            .path()
            .join(backup)
            .join("manifest.json")
            .is_file());
    }

    #[test]
    fn claude_all_four_ambiguous_mutation_windows_remain_unresolved() {
        for sabotage in [
            HostSabotage::FailAfterMarketplaceMutation,
            HostSabotage::FailAfterPluginMutation,
        ] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(temporary.path(), sabotage);
            let arguments = args(temporary.path(), UserHost::Claude, UserMode::Apply);
            let error = execute(UserOperation::Install, &arguments, &runner)
                .err()
                .expect("fresh ambiguity remains unresolved");
            assert!(matches!(error, InstallError::Internal(_)));
            assert!(temporary
                .path()
                .join(".hive/install-transactions/claude.json")
                .is_file());
        }
        for sabotage in [
            HostSabotage::FailAfterMarketplaceMutation,
            HostSabotage::FailAfterPluginMutation,
        ] {
            let temporary = tempdir().expect("tempdir");
            let runner = StatefulHostRunner::new(temporary.path(), sabotage);
            runner.seed_installed_codex_state();
            let arguments = args(temporary.path(), UserHost::Claude, UserMode::Apply);
            let error = execute(UserOperation::Update, &arguments, &runner)
                .err()
                .expect("refresh ambiguity remains unresolved");
            assert!(matches!(error, InstallError::Internal(_)));
            assert_eq!(runner.external_state(), (true, true));
            assert!(temporary
                .path()
                .join(".hive/install-transactions/claude.json")
                .is_file());
        }
    }

    #[test]
    fn crash_after_inverse_success_remains_unresolved_without_command_success() {
        let temporary = tempdir().expect("tempdir");
        let runner =
            StatefulHostRunner::new(temporary.path(), HostSabotage::CrashAfterPluginInverse);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("inverse crash");
        assert!(matches!(error, InstallError::Internal(_)));
        assert_eq!(runner.external_state(), (true, false));
        assert!(temporary
            .path()
            .join(".hive/install-transactions/codex.json")
            .is_file());
        let retry = recover(&arguments, &runner)
            .err()
            .expect("pending inverse remains unresolved");
        assert!(matches!(retry, InstallError::Conflict(_)));
    }

    #[test]
    fn post_activation_package_validation_failure_rolls_back_every_boundary() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::DeleteInstalledSkill);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("validation failure");
        assert!(matches!(error, InstallError::Verification(_)));
        assert_eq!(runner.external_state(), (false, false));
        assert!(!temporary.path().join(".codex/AGENTS.md").exists());
        assert!(!temporary.path().join(".hive/install/codex.json").exists());
    }

    #[test]
    fn concurrent_guidance_tamper_is_preserved_with_recovery_evidence() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(
            temporary.path(),
            HostSabotage::TamperGuidanceAfterActivation,
        );
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("index failure");
        assert!(matches!(error, InstallError::Internal(_)));
        assert_eq!(
            fs::read(temporary.path().join(".codex/AGENTS.md")).expect("foreign guidance"),
            b"tampered guidance\n"
        );
        assert!(temporary
            .path()
            .join(".hive/install-transactions/codex.json")
            .is_file());
    }

    #[test]
    fn preexisting_index_and_foreign_tamper_remain_recoverable() {
        let temporary = tempdir().expect("tempdir");
        write_operational_setup(temporary.path(), &["codex", "antigravity"]);
        let initial = args(temporary.path(), UserHost::Antigravity, UserMode::Apply);
        execute(
            UserOperation::Install,
            &initial,
            &AntigravityRunner::new(temporary.path()),
        )
        .expect("initial install");
        let before = hive_wiki::shared::rebuild_shared_index(&initial.user_root)
            .expect("initial shared index");
        let runner = StatefulHostRunner::new(
            temporary.path(),
            HostSabotage::TamperGuidanceAfterActivation,
        );
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("post-rebuild validation failure");
        assert!(matches!(error, InstallError::Internal(_)));
        assert_eq!(
            fs::read(temporary.path().join(".codex/AGENTS.md")).expect("foreign guidance"),
            b"tampered guidance\n"
        );
        hive_wiki::shared::validate_shared_index(&arguments.user_root)
            .expect("rollback left a current shared index");
        let after = hive_wiki::shared::rebuild_shared_index(&arguments.user_root)
            .expect("rebuilt rollback shared index");
        assert_eq!(before.logical_digest, after.logical_digest);
    }

    #[test]
    fn wiki_disable_removes_every_disposable_rag_artifact() {
        let temporary = tempdir().expect("tempdir");
        let index = temporary.path().join(".hive/index");
        fs::create_dir_all(&index).expect("index directory");
        fs::create_dir_all(temporary.path().join(".hive/config")).expect("config directory");
        let relatives = [
            ROOT_INDEX_RELATIVE,
            ".hive/index/hive.sqlite3-wal",
            ".hive/index/hive.sqlite3-shm",
            ".hive/index/hive.sqlite3-journal",
            hive_wiki::store::RAG_MANIFEST_RELATIVE,
            hive_wiki::store::RAG_TRUST_RELATIVE,
            hive_wiki::store::RAG_DIRTY_RELATIVE,
            ".hive/index/.stale",
        ];
        for relative in relatives {
            fs::write(temporary.path().join(relative), b"derived").expect("derived artifact");
        }
        let root =
            Dir::open_ambient_dir(temporary.path(), ambient_authority()).expect("pinned user root");

        remove_disposable_root_index(&root).expect("remove disposable RAG state");

        for relative in relatives {
            assert!(!temporary.path().join(relative).exists(), "{relative}");
        }
    }

    #[test]
    fn wiki_disable_removes_large_bounded_sqlite_state() {
        let temporary = tempdir().expect("tempdir");
        let index = temporary.path().join(".hive/index");
        fs::create_dir_all(&index).expect("index directory");
        for relative in [
            ROOT_INDEX_RELATIVE,
            ".hive/index/hive.sqlite3-wal",
            ".hive/index/hive.sqlite3-shm",
        ] {
            fs::File::create(temporary.path().join(relative))
                .expect("derived artifact")
                .set_len(MAX_USER_FILE_BYTES + 1)
                .expect("large sparse artifact");
        }
        let root =
            Dir::open_ambient_dir(temporary.path(), ambient_authority()).expect("pinned user root");

        remove_disposable_root_index(&root).expect("remove large disposable RAG state");

        for relative in [
            ROOT_INDEX_RELATIVE,
            ".hive/index/hive.sqlite3-wal",
            ".hive/index/hive.sqlite3-shm",
        ] {
            assert!(!temporary.path().join(relative).exists(), "{relative}");
        }
    }

    #[test]
    fn rollback_reconstruction_failure_preserves_preexisting_index_and_surfaces_error() {
        let temporary = tempdir().expect("tempdir");
        let index = temporary.path().join(ROOT_INDEX_RELATIVE);
        fs::create_dir_all(index.parent().expect("index parent")).expect("index parent");
        fs::write(&index, b"prior index bytes\n").expect("prior index");
        fs::create_dir_all(temporary.path().join(".hive/config/projects.yml"))
            .expect("invalid registry directory");
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);

        let error = reconcile_root_index_after_rollback(&arguments, true)
            .expect_err("reconstruction failure");

        assert!(matches!(error, InstallError::Verification(_)));
        assert!(error
            .message()
            .contains("root knowledge index reconstruction failed during rollback"));
        assert_eq!(
            fs::read(index).expect("preserved index"),
            b"prior index bytes\n"
        );
    }

    #[test]
    fn compensation_failure_retains_primary_and_compensation_context() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(
            temporary.path(),
            HostSabotage::FailAfterPluginMutationAndCompensation,
        );
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        let error = execute(UserOperation::Install, &arguments, &runner)
            .err()
            .expect("compensation failure");
        assert!(matches!(error, InstallError::Internal(_)));
        assert!(error.message().contains("reconciliation command"));
        assert!(error.message().contains("remains unresolved"));
        assert!(temporary
            .path()
            .join(".hive/install-transactions/codex.json")
            .is_file());
        let retry = recover(&arguments, &runner)
            .err()
            .expect("retry remains manual");
        assert!(matches!(retry, InstallError::Conflict(_)));
        assert_eq!(runner.external_state(), (true, true));
        assert!(temporary
            .path()
            .join(".hive/install-transactions/codex.json")
            .is_file());
    }

    #[test]
    fn explicit_recovery_restores_external_host_state() {
        let temporary = tempdir().expect("tempdir");
        let runner = StatefulHostRunner::new(temporary.path(), HostSabotage::None);
        let arguments = args(temporary.path(), UserHost::Codex, UserMode::Apply);
        execute(UserOperation::Install, &arguments, &runner).expect("apply");
        assert_eq!(runner.external_state(), (true, true));
        let recovered = recover(&arguments, &runner).expect("recover");
        assert_eq!(runner.external_state(), (false, false));
        let evidence = recovered.evidence.first().expect("backup evidence");
        assert_eq!(
            evidence.digest,
            sha256_digest(
                &fs::read(temporary.path().join(&evidence.locator)).expect("backup manifest")
            )
        );
    }
}
