//! Deterministic setup rendering and safe consumer-project activation.

use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapFsMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use hive_core::{
    ensure_consumer_target, ensure_no_symlink_ancestors,
    ensure_no_symlink_ancestors_for_hive_directive_projection,
    ensure_no_symlink_ancestors_for_hive_skill_projection, is_hive_directive_projection_path,
    is_hive_skill_projection_path, sha256_digest, validate_hive_directive_projection_relative,
    validate_hive_skill_projection_relative, validate_project_relative, TargetGuardError,
};
use hive_projection::{
    compile_project_projection, compile_projection, embedded_catalog, historical_builtin_skills,
    ActiveSkills, Availability, Host as ProjectionHost, OptionalSkillConsent, OptionalSkillSource,
    Projection, SkillSourceType,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter, Write as _};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MARKER_START: &str = "<!-- AIGENT-HIVE:START -->";
const MARKER_END: &str = "<!-- AIGENT-HIVE:END -->";
const SETUP_SCHEMA: &str = include_str!("../../../schemas/setup-answers.schema.json");
const ROLE_SCHEMA: &str = include_str!("../../../schemas/role-profile.schema.json");
const CAPABILITY_SCHEMA: &str = include_str!("../../../schemas/capability-matrix.schema.json");
const HOOK_SCHEMA: &str = include_str!("../../../schemas/hook-consent.schema.json");
const KNOWLEDGE_SUPPRESSION_SCHEMA: &str =
    include_str!("../../../schemas/knowledge-suppression.schema.json");
const OWNERSHIP_MANIFEST: &str = include_str!("../../../harness/manifest.toml");
const FRESH_CAPABILITY_RESOLUTION_PATH: &str = ".hive/runtime/current-capability-resolution.json";
const FRESH_CAPABILITY_RESOLUTION_MAX_AGE: Duration = Duration::from_mins(1);
const OPERATIONAL_USER_SETUP_VERSION: (u64, u64, u64) = (0, 8, 0);
static ACTIVATION_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Setup operation selected by the CLI.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SetupMode {
    /// Calculate the deterministic target tree without changing the target.
    DryRun,
    /// Render, validate, and activate the target tree.
    Apply,
    /// Validate the installed tree without changing it.
    Validate,
}

/// Stable setup failure class used to select the CLI exit contract.
#[derive(Debug)]
pub enum RenderError {
    /// Invalid input or an unsafe lexical path.
    Input(String),
    /// An approval or target-safety rule blocked activation.
    Safety(String),
    /// Existing protected/user data conflicts with the requested render.
    Conflict(String),
    /// Installed output failed required verification.
    Verification(String),
    /// The selected host/runtime cannot provide a requested capability.
    Unsupported(String),
    /// An unexpected local I/O or serialization failure occurred.
    Internal(String),
    /// Activation failed and the previous generation could not be restored completely.
    Rollback(String),
}

impl RenderError {
    /// Return the process exit class required by the action contract.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Input(_) => 2,
            Self::Safety(_) | Self::Conflict(_) => 3,
            Self::Verification(_) => 5,
            Self::Unsupported(_) => 4,
            Self::Internal(_) | Self::Rollback(_) => 10,
        }
    }

    /// Return the stable product code for this failure.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Input(_) => "hive.setup-invalid-input",
            Self::Safety(_) => "hive.setup-safety-blocked",
            Self::Conflict(_) => "hive.setup-conflict",
            Self::Verification(_) => "hive.setup-verification-failed",
            Self::Unsupported(_) => "hive.capability-unsupported",
            Self::Internal(_) => "hive.internal-error",
            Self::Rollback(_) => "hive.activation-rollback-failed",
        }
    }

    /// Return the corresponding action status.
    #[must_use]
    pub const fn status(&self) -> &'static str {
        match self {
            Self::Input(_) | Self::Internal(_) | Self::Rollback(_) => "error",
            Self::Safety(_) => "blocked",
            Self::Conflict(_) => "conflict",
            Self::Verification(_) => "verification-failed",
            Self::Unsupported(_) => "unsupported",
        }
    }
}

impl Display for RenderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message)
            | Self::Safety(message)
            | Self::Conflict(message)
            | Self::Verification(message)
            | Self::Unsupported(message)
            | Self::Internal(message)
            | Self::Rollback(message) => formatter.write_str(message),
        }
    }
}

impl Error for RenderError {}

/// Successful setup evidence returned to the CLI.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SetupOutcome {
    /// Project-relative paths whose bytes differ from the active target.
    pub changed_paths: Vec<String>,
    /// Automatically selected orchestration owner.
    pub resolved_owner: String,
    /// Digest of the complete normalized planned tree.
    pub tree_digest: String,
    /// Exact before/after digest plan for durable update journaling.
    pub changes: Vec<SetupChange>,
    /// Resolved project preferences when the user-scope bridge was active.
    pub effective_preferences: Option<ResolvedProjectPreferences>,
}

/// Public effective project preference snapshot for registry and CLI evidence.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResolvedProjectPreferences {
    pub setup_mode: String,
    pub provenance: String,
    pub interface_language: String,
    pub wiki_enabled: bool,
    pub wiki_language: String,
    pub persona_id: String,
    pub persona_custom_description: Option<String>,
    pub selected_project_skills: Vec<String>,
    pub usage_guard_enabled: bool,
    pub codexbar_fallback_enabled: bool,
    pub usage_stop_remaining_percent: u8,
}

/// One exact setup mutation plan entry.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SetupChange {
    /// Project-relative manifest-owned path.
    pub path: String,
    /// Existing digest, or `None` when the path is absent.
    pub before_digest: Option<String>,
    /// Planned digest, or `None` for a deletion.
    pub after_digest: Option<String>,
    /// Digest of bytes outside the Hive marker before the change.
    ///
    /// This is populated only for shared-marker paths so cross-major update
    /// verification can prove that user and third-party bytes remain exact.
    pub foreign_before_digest: Option<String>,
    /// Digest of bytes outside the Hive marker after the planned change.
    pub foreign_after_digest: Option<String>,
}

/// Trusted incoming project projection rendered from this binary.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectUpgradeCandidate {
    /// Embedded product release version.
    pub product_version: String,
    /// Mergeable path to exact incoming bytes. Shared files contain only the
    /// exact Hive marker block, never foreign bytes.
    pub files: BTreeMap<String, Vec<u8>>,
    /// Generated support files replaced exactly after the merge succeeds.
    pub support_files: BTreeMap<String, Vec<u8>>,
    /// Canonical base ledger bytes to activate after a successful merge.
    pub base_ledger: Vec<u8>,
}

/// Exact full project-base registry entry for one historical release.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HistoricalProjectBase {
    /// Release whose complete projection contract these files authenticate.
    pub product_version: String,
    /// Sorted exact file inventory.
    pub files: Vec<HistoricalProjectBaseFile>,
}

/// One exact file in a full historical project-base registry entry.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HistoricalProjectBaseFile {
    /// Portable project-relative path.
    pub path: String,
    /// Stable ownership kind: `skill`, `directive`, or `shared-marker`.
    pub kind: String,
    /// Digest of `content`.
    pub content_digest: String,
    /// Exact base bytes.
    pub content: Vec<u8>,
}

/// Setup request assembled by the CLI.
#[derive(Debug)]
pub struct SetupRequest<'a> {
    /// Consumer project root.
    pub target: &'a Path,
    /// Setup answer YAML file.
    pub answers: &'a Path,
    /// Normalized host capability JSON file.
    pub capabilities: &'a Path,
    /// Requested setup operation.
    pub mode: SetupMode,
    /// Role ids explicitly approved for definition reconfiguration.
    pub reconfigure_roles: BTreeSet<String>,
    /// Validated user-scope preferences used to resolve project defaults.
    ///
    /// `None` preserves the pre-0.8 legacy render contract.
    pub global_preferences: Option<GlobalProjectPreferences>,
}

/// Validated user-scope preferences supplied by the CLI bridge.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GlobalProjectPreferences {
    /// Preferred user-facing interface language.
    pub interface_language: String,
    /// Whether canonical Wiki capture and query are enabled globally.
    pub wiki_enabled: bool,
    /// Canonical Wiki language selection.
    pub wiki_language: String,
    /// Selected agent persona id.
    pub persona_id: String,
    /// Required only for the `custom` persona.
    pub persona_custom_description: Option<String>,
    /// Exact dependency-closed built-in Skill names selected for projects.
    pub selected_project_skills: Vec<String>,
    /// Whether usage guard enforcement is enabled.
    pub usage_guard_enabled: bool,
    /// Whether the user approved the fixed `CodexBar` fallback adapter.
    pub codexbar_fallback_enabled: bool,
    /// Remaining-usage stop threshold inherited by every project.
    pub usage_stop_remaining_percent: u8,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct EffectiveProjectPreferences {
    provenance: &'static str,
    interface_language: String,
    wiki_enabled: bool,
    wiki_language: String,
    persona_id: String,
    persona_custom_description: Option<String>,
    selected_project_skills: Vec<String>,
    usage_guard_enabled: bool,
    codexbar_fallback_enabled: bool,
    usage_stop_remaining_percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SetupAnswers {
    schema_version: u32,
    project_name: String,
    #[serde(default)]
    project_identity: String,
    setup_mode: String,
    project_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    interface_language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    wiki: Option<ProjectWikiPreferences>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    persona: Option<ProjectPreferenceSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skills: Option<ProjectSkillSelection>,
    primary_host: String,
    usage_stop_remaining_percent: u8,
    elevated_judge_quorum: String,
    critical_judge_quorum: String,
    persistent_roles: Vec<RoleSeed>,
    knowledge_include_paths: Vec<String>,
    knowledge_exclude_paths: Vec<String>,
    #[serde(default)]
    root_knowledge_promotion_categories: Vec<String>,
    #[serde(default)]
    confidential_knowledge_categories: Vec<String>,
    #[serde(default)]
    user_store_binding: String,
    approved_optional_skills: Vec<SkillApproval>,
    approved_fallback_hooks: Vec<HookApproval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ProjectWikiPreferences {
    enabled: bool,
    language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ProjectPreferenceSelection {
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ProjectSkillSelection {
    mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recommended_suite: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    selected: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct RoleSeed {
    role_id: String,
    display_name: String,
    responsibilities: Vec<String>,
    non_responsibilities: Vec<String>,
    context_paths: Vec<String>,
    allowed_capabilities: Vec<String>,
    write_scope: Vec<String>,
    verification_duties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SkillApproval {
    consent_version: u32,
    name: String,
    source: String,
    revision: String,
    content_digest: String,
    requested_capabilities: Vec<String>,
    approved_capabilities: Vec<String>,
    approved_at: String,
    consent_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct HookApproval {
    consent_version: u32,
    capability: String,
    event: String,
    path: String,
    command: String,
    content_digest: String,
    approved_at: String,
    consent_digest: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct HookLedger {
    schema_version: u32,
    detection: String,
    resolution_evidence_digest: String,
    hooks: Vec<HookApproval>,
}

#[derive(Debug, Deserialize)]
struct SkillLedger {
    skills: Vec<SkillApproval>,
}

#[derive(Debug, Serialize)]
struct ProjectBaseFile {
    path: String,
    kind: String,
    content_digest: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct InstalledHarness {
    schema_version: u32,
    harness_version: String,
    source_release_version: String,
    project_name: String,
    project_kind: String,
    #[serde(default)]
    setup_mode: Option<String>,
    #[serde(default)]
    preference_provenance: Option<String>,
    #[serde(default)]
    interface_language: Option<String>,
    #[serde(default)]
    wiki_enabled: Option<bool>,
    #[serde(default)]
    wiki_language: Option<String>,
    #[serde(default)]
    persona_id: Option<String>,
    #[serde(default)]
    persona_custom_description: Option<String>,
    #[serde(default)]
    selected_project_skills: Vec<String>,
    #[serde(default)]
    usage_guard_enabled: Option<bool>,
    #[serde(default)]
    codexbar_fallback_enabled: bool,
    primary_host: String,
    external_capability_detection: String,
    resolved_owner: String,
    resolution_evidence_digest: String,
    usage_stop_remaining_percent: u8,
    elevated_judge_quorum: String,
    critical_judge_quorum: String,
    approved_optional_skills_file: String,
    capability_resolution_file: String,
    #[serde(default)]
    approved_fallback_hooks_file: Option<String>,
    role_seed_file: String,
    knowledge_scope_file: String,
}

#[derive(Debug, Serialize)]
struct HookDescriptor<'a> {
    capability: &'a str,
    command: &'a str,
    event: &'a str,
    path: &'a str,
    schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct CapabilityResolution {
    schema_version: u32,
    host: String,
    host_version: String,
    surface: String,
    detection: String,
    external_runtime: Option<String>,
    resolved_owner: String,
    #[serde(default)]
    capabilities: BTreeMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hook_events: Option<BTreeMap<String, JsonValue>>,
    evidence_digest: String,
    evidence: Vec<CapabilityEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct CapabilityEvidence {
    source: String,
    locator: String,
    outcome: String,
    digest: String,
}

/// Result of validating an installed fallback-hook invocation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HookAuthorization {
    /// The exact installed hook approval and descriptor are valid.
    Authorized,
    /// External orchestration is present, so the fallback hook is inert.
    Inert,
}

/// Keep legacy authorization callers inert when they have no fresh evidence.
///
/// Hook activation requires [`authorize_hook_with_resolution`] with the exact
/// fresh capability path. This compatibility entrypoint never reads the target
/// and cannot authorize a fallback hook.
///
/// # Errors
///
/// This compatibility entrypoint does not return an error.
pub fn authorize_hook(
    _target: &Path,
    _capability: &str,
    _event: &str,
) -> Result<HookAuthorization, RenderError> {
    Ok(HookAuthorization::Inert)
}

/// Return the normalized detection derived from a fresh capability matrix.
///
/// The matrix is schema-validated and its JCS evidence digest is recomputed.
/// No target or hook input is read.
///
/// # Errors
///
/// Returns an input error when the matrix is malformed, contradictory, or
/// carries an invalid evidence digest.
pub fn capability_detection(path: &Path) -> Result<String, RenderError> {
    let resolution = load_resolution(path)?;
    validate_resolution_host(&resolution.host, &resolution)?;
    Ok(derive_resolution(&resolution)?.0.to_owned())
}

/// Return the external runtime associated with the installed host.
///
/// This performs read-only installed capability validation and never probes or
/// starts a process.
///
/// # Errors
///
/// Returns a verification error when installed capability evidence is missing
/// or malformed.
pub fn expected_external_runtime(target: &Path) -> Result<Option<&'static str>, RenderError> {
    ensure_consumer_target(target).map_err(|error| RenderError::Verification(error.to_string()))?;
    let resolution = read_installed_resolution(target)?;
    validate_resolution_host(&resolution.host, &resolution).map_err(as_verification)?;
    match resolution.host.as_str() {
        "codex" => Ok(Some("omx")),
        "claude" => Ok(Some("omc")),
        "antigravity" => Ok(None),
        _ => Err(RenderError::Verification(
            "installed capability host is unsupported".to_owned(),
        )),
    }
}

/// Validate an installed fallback hook with required fresh host evidence.
///
/// Every state other than conclusive `absent` makes the fallback hook inert
/// before installed hook authorization proceeds. The caller can therefore
/// perform this check before reading hook event input.
///
/// # Errors
///
/// Returns a safety or verification error when fresh evidence is malformed,
/// addresses another host, or the requested non-inert hook is not exactly
/// approved.
pub fn authorize_hook_with_resolution(
    target: &Path,
    capability: &str,
    event: &str,
    fresh_capabilities: &Path,
) -> Result<HookAuthorization, RenderError> {
    let fresh_path = validate_fresh_capability_resolution_path(target, fresh_capabilities)?;
    let fresh = load_resolution(&fresh_path)?;
    validate_resolution_host(&fresh.host, &fresh)?;
    if derive_resolution(&fresh)?.0 != "absent" {
        return Ok(HookAuthorization::Inert);
    }
    ensure_consumer_target(target).map_err(|error| RenderError::Safety(error.to_string()))?;
    validate_installed(target)?;
    let resolution = read_installed_resolution(target)?;
    if fresh.host != resolution.host {
        return Err(RenderError::Safety(
            "fresh capability evidence does not address the installed host".to_owned(),
        ));
    }
    let (detection, _, _) = derive_resolution(&resolution)?;
    if detection != "absent" {
        return Ok(HookAuthorization::Inert);
    }
    if fresh.evidence_digest != resolution.evidence_digest {
        return Err(RenderError::Safety(
            "fresh capability evidence does not match the installed absent resolution".to_owned(),
        ));
    }
    let bytes = read_target_required(
        target,
        Path::new(".hive/config/approved-hooks.yml"),
        "fallback hook approval ledger",
    )
    .map_err(|error| RenderError::Safety(error.to_string()))?;
    let ledger: HookLedger = serde_yaml::from_slice(&bytes).map_err(|error| {
        RenderError::Safety(format!("fallback hook approval ledger is invalid: {error}"))
    })?;
    if ledger.schema_version != 1
        || ledger.detection != "absent"
        || ledger.resolution_evidence_digest != resolution.evidence_digest
    {
        return Err(RenderError::Safety(
            "fallback hook approval does not bind current absent evidence".to_owned(),
        ));
    }
    let hook = ledger
        .hooks
        .iter()
        .find(|hook| hook.capability == capability && hook.event == event)
        .ok_or_else(|| {
            RenderError::Safety(format!(
                "fallback hook invocation is not approved: {capability}/{event}"
            ))
        })?;
    validate_hook_approvals(std::slice::from_ref(hook), &resolution)?;
    let relative = Path::new(&hook.path);
    ensure_no_symlink_ancestors(target, relative)
        .map_err(|error| RenderError::Safety(error.to_string()))?;
    let expected = hook_descriptor_bytes(hook)?;
    let projected = read_target_required(target, relative, "fallback hook descriptor")
        .map_err(|error| RenderError::Safety(error.to_string()))?;
    if projected != expected || sha256_digest(&projected) != hook.content_digest {
        return Err(RenderError::Safety(
            "fallback hook descriptor bytes do not match approval".to_owned(),
        ));
    }
    Ok(HookAuthorization::Authorized)
}

fn validate_fresh_capability_resolution_path(
    target: &Path,
    path: &Path,
) -> Result<PathBuf, RenderError> {
    if path != Path::new(FRESH_CAPABILITY_RESOLUTION_PATH) {
        return Err(RenderError::Input(format!(
            "fresh capability evidence must use {FRESH_CAPABILITY_RESOLUTION_PATH}"
        )));
    }
    ensure_no_symlink_ancestors(target, path)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let absolute = target.join(path);
    let metadata = fs::symlink_metadata(&absolute).map_err(|error| {
        RenderError::Input(format!(
            "cannot inspect fresh capability evidence at {FRESH_CAPABILITY_RESOLUTION_PATH}: {error}"
        ))
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(RenderError::Input(
            "fresh capability evidence must be a regular non-symlink file".to_owned(),
        ));
    }
    let modified = metadata.modified().map_err(|error| {
        RenderError::Input(format!(
            "cannot inspect fresh capability evidence timestamp: {error}"
        ))
    })?;
    let age = SystemTime::now().duration_since(modified).map_err(|_| {
        RenderError::Input("fresh capability evidence timestamp is in the future".to_owned())
    })?;
    if age > FRESH_CAPABILITY_RESOLUTION_MAX_AGE {
        return Err(RenderError::Input(
            "fresh capability evidence is older than 60 seconds".to_owned(),
        ));
    }
    Ok(absolute)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoleProfile {
    allowed_capabilities: Vec<String>,
    context_paths: Vec<String>,
    current_assignment: Option<String>,
    display_name: String,
    handoff_path: Option<String>,
    non_responsibilities: Vec<String>,
    responsibilities: Vec<String>,
    role_id: String,
    schema_version: u32,
    verification_duties: Vec<String>,
    write_scope: Vec<String>,
}

impl RoleProfile {
    fn from_seed(seed: &RoleSeed) -> Self {
        Self {
            allowed_capabilities: seed.allowed_capabilities.clone(),
            context_paths: seed.context_paths.clone(),
            current_assignment: None,
            display_name: seed.display_name.clone(),
            handoff_path: None,
            non_responsibilities: seed.non_responsibilities.clone(),
            responsibilities: seed.responsibilities.clone(),
            role_id: seed.role_id.clone(),
            schema_version: 1,
            verification_duties: seed.verification_duties.clone(),
            write_scope: seed.write_scope.clone(),
        }
    }

    fn definition_matches(&self, seed: &RoleSeed) -> bool {
        self.role_id == seed.role_id
            && self.display_name == seed.display_name
            && self.responsibilities == seed.responsibilities
            && self.non_responsibilities == seed.non_responsibilities
            && self.context_paths == seed.context_paths
            && self.allowed_capabilities == seed.allowed_capabilities
            && self.write_scope == seed.write_scope
            && self.verification_duties == seed.verification_duties
    }

    fn apply_definition(&mut self, seed: &RoleSeed) {
        self.display_name.clone_from(&seed.display_name);
        self.responsibilities.clone_from(&seed.responsibilities);
        self.non_responsibilities
            .clone_from(&seed.non_responsibilities);
        self.context_paths.clone_from(&seed.context_paths);
        self.allowed_capabilities
            .clone_from(&seed.allowed_capabilities);
        self.write_scope.clone_from(&seed.write_scope);
        self.verification_duties
            .clone_from(&seed.verification_duties);
    }
}

#[derive(Debug, Deserialize)]
struct OwnershipManifest {
    paths: Vec<OwnershipEntry>,
}

#[derive(Debug, Deserialize)]
struct OwnershipEntry {
    pattern: String,
    ownership: String,
    #[serde(default)]
    marker_start: Option<String>,
    #[serde(default)]
    marker_end: Option<String>,
}

/// Execute deterministic setup or installed-tree validation.
///
/// # Errors
///
/// Returns a stable [`RenderError`] before changing the target whenever
/// validation, consent, ownership, marker, or role safety checks fail.
pub fn execute_setup(request: &SetupRequest<'_>) -> Result<SetupOutcome, RenderError> {
    ensure_consumer_target(request.target)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let target_dir = open_target_capability(request.target)?;
    execute_setup_with_target(request, &target_dir, true, None, None)
}

/// Execute setup while retaining project rollback state until a connected
/// external projection has committed.
///
/// # Errors
///
/// Returns a stable [`RenderError`] when setup, the connected commit, or the
/// capability-pinned rollback fails.
pub fn execute_setup_with_post_apply(
    request: &SetupRequest<'_>,
    post_apply: &dyn Fn() -> Result<(), RenderError>,
) -> Result<SetupOutcome, RenderError> {
    if request.mode != SetupMode::Apply {
        return Err(RenderError::Unsupported(
            "connected setup commit is available only for apply".to_owned(),
        ));
    }
    ensure_consumer_target(request.target)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let target_dir = open_target_capability(request.target)?;
    execute_setup_with_target(request, &target_dir, true, None, Some(post_apply))
}

/// Render the current binary's mergeable project projection through an
/// already-pinned consumer target capability.
///
/// # Errors
///
/// Returns an error when the installed answers/capabilities, optional Skill
/// source, shared marker, or embedded projection contract is invalid.
pub fn project_upgrade_candidate_in(
    target_dir: &Dir,
) -> Result<ProjectUpgradeCandidate, RenderError> {
    let answers = read_installed_answers(target_dir)?;
    let answers_value =
        serde_json::to_value(&answers).map_err(|error| RenderError::Internal(error.to_string()))?;
    let resolution = read_installed_resolution(target_dir)?;
    validate_answers(&answers)?;
    validate_resolution(&answers, &resolution)?;
    validate_schema_instance(SETUP_SCHEMA, &answers_value, "setup answers")?;
    let harness = read_installed_harness(target_dir)?;
    let effective_preferences = effective_preferences_from_harness(&harness)?;
    let files = render_tree_with_preferences(
        target_dir,
        &answers,
        &resolution,
        &BTreeSet::new(),
        effective_preferences.as_ref(),
    )?;
    let mergeable = project_upgrade_files(&files)?;
    let support_files = [
        ".hive/setup-answers.yml",
        ".hive/config/harness.toml",
        ".hive/config/knowledge-scope.yml",
        ".hive/config/approved-skills.yml",
        ".hive/config/active-skills.yml",
    ]
    .into_iter()
    .map(|path| {
        files
            .get(Path::new(path))
            .cloned()
            .map(|bytes| (path.to_owned(), bytes))
            .ok_or_else(|| {
                RenderError::Internal(format!("project render omitted support file: {path}"))
            })
    })
    .collect::<Result<BTreeMap<_, _>, _>>()?;
    let base_ledger = files
        .get(Path::new(".hive/config/project-base.json"))
        .cloned()
        .ok_or_else(|| {
            RenderError::Internal("project render omitted its base ledger".to_owned())
        })?;
    Ok(ProjectUpgradeCandidate {
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        files: mergeable,
        support_files,
        base_ledger,
    })
}

/// Render an exact full project-base candidate for a release whose complete
/// projection contract is embedded by this binary.
///
/// Release `0.7.0` is the first full-ledger registry entry. Releases `0.1.0`
/// through `0.6.0` retain their legacy Skill-only authentication contract.
/// Future releases must preserve this branch byte-for-byte when adding a new
/// full-ledger entry.
///
/// # Errors
///
/// Returns unsupported for a release without a full embedded project-base
/// contract, or a validation error when the installed project inputs cannot
/// render that release's exact ledger.
pub fn historical_project_upgrade_candidate_in(
    target_dir: &Dir,
    version: &str,
) -> Result<HistoricalProjectBase, RenderError> {
    let files = match version {
        "0.7.0" => frozen_project_base_0_7(target_dir)?,
        _ => Err(RenderError::Unsupported(format!(
            "historical full project base is not embedded: {version}"
        )))?,
    };
    let files = files
        .into_iter()
        .map(|(path, content)| {
            let kind = if matches!(path.as_str(), "AGENTS.md" | "CLAUDE.md" | "GEMINI.md") {
                "shared-marker"
            } else if is_hive_skill_projection_path(Path::new(&path)) {
                "skill"
            } else if is_hive_directive_projection_path(Path::new(&path)) {
                "directive"
            } else {
                return Err(RenderError::Internal(format!(
                    "historical project base contains an unsupported path: {path}"
                )));
            };
            Ok(HistoricalProjectBaseFile {
                path,
                kind: kind.to_owned(),
                content_digest: sha256_digest(&content),
                content,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HistoricalProjectBase {
        product_version: version.to_owned(),
        files,
    })
}

#[allow(clippy::too_many_lines)]
fn frozen_project_base_0_7(target_dir: &Dir) -> Result<BTreeMap<String, Vec<u8>>, RenderError> {
    #[derive(Deserialize)]
    struct Answers0_7 {
        project_name: String,
        project_kind: String,
        primary_host: String,
        approved_optional_skills: Vec<SkillApproval>,
    }

    #[derive(Deserialize)]
    struct Resolution0_7 {
        resolved_owner: String,
        evidence_digest: String,
    }

    const DIRECTIVES: [(&str, &[u8]); 3] = [
        (
            "00-project-harness.md",
            include_bytes!("../../../harness/project-bases/0.7.0/directives/00-project-harness.md"),
        ),
        (
            "01-project-knowledge.md",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/directives/01-project-knowledge.md"
            ),
        ),
        (
            "02-project-upgrade.md",
            include_bytes!("../../../harness/project-bases/0.7.0/directives/02-project-upgrade.md"),
        ),
    ];
    const SKILLS: [(&str, &[u8]); 15] = [
        (
            "hive-judge-package",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-judge-package/SKILL.md"
            ),
        ),
        (
            "hive-knowledge-capture",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-knowledge-capture/SKILL.md"
            ),
        ),
        (
            "hive-knowledge-maintenance",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-knowledge-maintenance/SKILL.md"
            ),
        ),
        (
            "hive-knowledge-promote",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-knowledge-promote/SKILL.md"
            ),
        ),
        (
            "hive-knowledge-query",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-knowledge-query/SKILL.md"
            ),
        ),
        (
            "hive-migrate",
            include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-migrate/SKILL.md"),
        ),
        (
            "hive-project-upgrade",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-project-upgrade/SKILL.md"
            ),
        ),
        (
            "hive-prompt-refine",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-prompt-refine/SKILL.md"
            ),
        ),
        (
            "hive-role-handoff",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-role-handoff/SKILL.md"
            ),
        ),
        (
            "hive-run-checkpoint",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-run-checkpoint/SKILL.md"
            ),
        ),
        (
            "hive-run-resume",
            include_bytes!("../../../harness/project-bases/0.7.0/skills/hive-run-resume/SKILL.md"),
        ),
        (
            "hive-simple-question",
            include_bytes!(
                "../../../harness/project-bases/0.7.0/skills/hive-simple-question/SKILL.md"
            ),
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
    let answers: Answers0_7 = serde_yaml::from_slice(&read_target_required(
        target_dir,
        Path::new(".hive/setup-answers.yml"),
        "0.7 setup answers",
    )?)
    .map_err(|error| {
        RenderError::Verification(format!("invalid frozen 0.7 setup answers: {error}"))
    })?;
    let resolution: Resolution0_7 = serde_yaml::from_slice(&read_target_required(
        target_dir,
        Path::new(".hive/config/capability-resolution.yml"),
        "0.7 capability resolution",
    )?)
    .map_err(|error| {
        RenderError::Verification(format!("invalid frozen 0.7 capability resolution: {error}"))
    })?;
    if answers.project_name.is_empty()
        || answers.project_kind.is_empty()
        || !matches!(
            answers.primary_host.as_str(),
            "codex" | "claude" | "antigravity"
        )
        || resolution.resolved_owner.is_empty()
        || !valid_digest(&resolution.evidence_digest)
    {
        return Err(RenderError::Verification(
            "frozen 0.7 project-base inputs are invalid".to_owned(),
        ));
    }
    validate_skill_approvals(&answers.approved_optional_skills).map_err(as_verification)?;
    let approved_ledger: SkillLedger = serde_yaml::from_slice(&read_target_required(
        target_dir,
        Path::new(".hive/config/approved-skills.yml"),
        "0.7 approved optional Skill ledger",
    )?)
    .map_err(|error| {
        RenderError::Verification(format!(
            "invalid frozen 0.7 approved optional Skill ledger: {error}"
        ))
    })?;
    if approved_ledger.skills != answers.approved_optional_skills {
        return Err(RenderError::Verification(
            "frozen 0.7 optional Skill ledger differs from setup approval authority".to_owned(),
        ));
    }
    let mut files = BTreeMap::new();
    for (name, content) in DIRECTIVES {
        files.insert(format!(".agents/directives/{name}"), content.to_vec());
    }
    for (name, content) in SKILLS {
        files.insert(format!(".agents/skills/{name}/SKILL.md"), content.to_vec());
        if answers.primary_host == "claude" {
            files.insert(format!(".claude/skills/{name}/SKILL.md"), content.to_vec());
        }
    }
    let built_in_names = SKILLS
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    let mut optional_names = BTreeSet::new();
    let mut optional_files = BTreeMap::new();
    for approval in &answers.approved_optional_skills {
        let Some(source) = approval.source.strip_prefix("path:") else {
            continue;
        };
        if approval.approved_capabilities != approval.requested_capabilities {
            continue;
        }
        if built_in_names.contains(approval.name.as_str())
            || !optional_names.insert(approval.name.as_str())
        {
            return Err(RenderError::Verification(format!(
                "frozen 0.7 optional Skill name collides with another projection: {}",
                approval.name
            )));
        }
        let source = PathBuf::from(source);
        validate_project_relative(&source)
            .map_err(|error| RenderError::Verification(error.to_string()))?;
        if source.file_name() != Some(OsStr::new("SKILL.md"))
            || [".agents", ".claude", ".hive"]
                .iter()
                .any(|root| source.starts_with(root))
            || matches!(
                source.to_str(),
                Some("AGENTS.md" | "CLAUDE.md" | "GEMINI.md")
            )
        {
            return Err(RenderError::Verification(format!(
                "frozen 0.7 optional Skill source is inside a Hive-managed namespace: {}",
                source.display()
            )));
        }
        let skill_md = read_target_required(
            target_dir,
            &source,
            "0.7 approved project-local optional Skill source",
        )?;
        validate_frozen_0_7_optional_source(approval, &skill_md)?;
        optional_files.insert(approval.name.as_str(), skill_md);
    }
    for (name, content) in &optional_files {
        let path = format!(".agents/skills/{name}/SKILL.md");
        files.insert(path, (*content).clone());
    }
    if answers.primary_host == "claude" {
        for (name, content) in &optional_files {
            let path = format!(".claude/skills/{name}/SKILL.md");
            files.insert(path, (*content).clone());
        }
    }
    let marker = include_str!("../../../harness/project-bases/0.7.0/AGENTS.md.template")
        .replace("{{ project_name }}", &answers.project_name)
        .replace("{{ project_kind }}", &answers.project_kind)
        .replace("{{ primary_host }}", &answers.primary_host)
        .replace(
            "{{ capability_resolution.resolved_owner }}",
            &resolution.resolved_owner,
        )
        .replace(
            "{{ capability_resolution.evidence_digest }}",
            &resolution.evidence_digest,
        );
    files.insert("AGENTS.md".to_owned(), marker.into_bytes());
    let adapter = format!("{MARKER_START}\n@AGENTS.md\n{MARKER_END}\n").into_bytes();
    files.insert("CLAUDE.md".to_owned(), adapter.clone());
    files.insert("GEMINI.md".to_owned(), adapter);
    Ok(files)
}

fn validate_frozen_0_7_optional_source(
    approval: &SkillApproval,
    skill_md: &[u8],
) -> Result<(), RenderError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Frontmatter0_7 {
        name: String,
        description: String,
    }

    let projected = PathBuf::from(format!(".agents/skills/{}/SKILL.md", approval.name));
    validate_hive_skill_projection_relative(&projected)
        .map_err(|error| RenderError::Verification(error.to_string()))?;
    let digest = sha256_digest(skill_md);
    if approval.revision != digest || approval.content_digest != digest {
        return Err(RenderError::Verification(format!(
            "frozen 0.7 optional Skill bytes differ from approval: {}",
            approval.name
        )));
    }
    let text = std::str::from_utf8(skill_md).map_err(|_| {
        RenderError::Verification(format!(
            "frozen 0.7 optional Skill is not UTF-8: {}",
            approval.name
        ))
    })?;
    let (rest, delimiter) = if let Some(rest) = text.strip_prefix("---\n") {
        (rest, "\n---\n")
    } else if let Some(rest) = text.strip_prefix("---\r\n") {
        (rest, "\r\n---\r\n")
    } else {
        return Err(RenderError::Verification(format!(
            "frozen 0.7 optional Skill has no frontmatter: {}",
            approval.name
        )));
    };
    let (frontmatter, _) = rest.split_once(delimiter).ok_or_else(|| {
        RenderError::Verification(format!(
            "frozen 0.7 optional Skill has unterminated frontmatter: {}",
            approval.name
        ))
    })?;
    let frontmatter: Frontmatter0_7 = serde_yaml::from_str(frontmatter).map_err(|error| {
        RenderError::Verification(format!(
            "frozen 0.7 optional Skill frontmatter is invalid for {}: {error}",
            approval.name
        ))
    })?;
    if frontmatter.name != approval.name || frontmatter.description.trim().is_empty() {
        return Err(RenderError::Verification(format!(
            "frozen 0.7 optional Skill frontmatter differs from approval: {}",
            approval.name
        )));
    }
    Ok(())
}

/// Execute setup against a caller-pinned consumer target capability.
///
/// This entrypoint never reopens `request.target`; callers that authorize
/// target selection must retain the opened directory capability and pass it
/// through the complete operation.
///
/// # Errors
///
/// Returns a stable [`RenderError`] before changing the pinned target whenever
/// validation, consent, ownership, marker, or role safety checks fail.
pub fn execute_setup_in(
    request: &SetupRequest<'_>,
    target_dir: &Dir,
) -> Result<SetupOutcome, RenderError> {
    if request.mode == SetupMode::Validate {
        return Err(RenderError::Unsupported(
            "pinned setup entrypoint supports only dry-run and apply".to_owned(),
        ));
    }
    execute_setup_with_target(request, target_dir, false, None, None)
}

/// Execute a signed release update against a caller-pinned consumer target.
///
/// This entrypoint is intentionally separate from setup/reconfigure. The
/// caller must supply the exact source version selected by its authenticated
/// release transition. Hive still binds that version to the installed harness
/// and authenticates every existing projected Skill against embedded release
/// history before treating legacy directive paths as absent.
///
/// # Errors
///
/// Returns an error when the supplied source version differs from the
/// installed harness, is not covered by embedded release history, or any
/// existing projection lacks exact ownership proof.
pub fn execute_release_update_in(
    request: &SetupRequest<'_>,
    target_dir: &Dir,
    authenticated_source_version: &str,
) -> Result<SetupOutcome, RenderError> {
    execute_release_update_for_target_in(
        request,
        target_dir,
        authenticated_source_version,
        env!("CARGO_PKG_VERSION"),
    )
}

fn execute_release_update_for_target_in(
    request: &SetupRequest<'_>,
    target_dir: &Dir,
    authenticated_source_version: &str,
    target_version: &str,
) -> Result<SetupOutcome, RenderError> {
    if request.mode == SetupMode::Validate {
        return Err(RenderError::Unsupported(
            "pinned release-update entrypoint supports only dry-run and apply".to_owned(),
        ));
    }
    let installed = read_installed_harness(target_dir)?;
    if installed.harness_version != authenticated_source_version {
        return Err(RenderError::Verification(
            "authenticated release-update source version does not match the installed harness"
                .to_owned(),
        ));
    }
    let replay_preferences = effective_preferences_from_harness(&installed)?.map(|preferences| {
        GlobalProjectPreferences {
            interface_language: preferences.interface_language,
            wiki_enabled: preferences.wiki_enabled,
            wiki_language: preferences.wiki_language,
            persona_id: preferences.persona_id,
            persona_custom_description: preferences.persona_custom_description,
            selected_project_skills: preferences.selected_project_skills,
            usage_guard_enabled: preferences.usage_guard_enabled,
            codexbar_fallback_enabled: preferences.codexbar_fallback_enabled,
            usage_stop_remaining_percent: preferences.usage_stop_remaining_percent,
        }
    });
    require_operational_update_preferences(target_version, replay_preferences.as_ref())?;
    if authenticated_source_version != target_version {
        historical_builtin_skills(authenticated_source_version).map_err(|error| {
            RenderError::Verification(format!(
                "authenticated release-update source is not covered by embedded history: {error}"
            ))
        })?;
    }
    let replay_request = SetupRequest {
        target: request.target,
        answers: request.answers,
        capabilities: request.capabilities,
        mode: request.mode,
        reconfigure_roles: request.reconfigure_roles.clone(),
        global_preferences: replay_preferences,
    };
    execute_setup_with_target(
        &replay_request,
        target_dir,
        false,
        Some(authenticated_source_version),
        None,
    )
}

fn require_operational_update_preferences(
    target_version: &str,
    preferences: Option<&GlobalProjectPreferences>,
) -> Result<(), RenderError> {
    let target_version = parse_release_version(target_version).ok_or_else(|| {
        RenderError::Verification(
            "authenticated release-update target version is invalid".to_owned(),
        )
    })?;
    if target_version >= OPERATIONAL_USER_SETUP_VERSION && preferences.is_none() {
        return Err(RenderError::Unsupported(
            "Hive 0.8+ project update requires validated user setup and transactional shared-registry binding; run `hive setup --scope user` and connected project setup before retrying"
                .to_owned(),
        ));
    }
    Ok(())
}

fn parse_release_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut components = version.split('.');
    let parsed = (
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
    );
    components.next().is_none().then_some(parsed)
}

#[allow(clippy::too_many_lines)]
fn execute_setup_with_target(
    request: &SetupRequest<'_>,
    target_dir: &Dir,
    verify_ambient_target: bool,
    release_source_version: Option<&str>,
    post_apply: Option<&dyn Fn() -> Result<(), RenderError>>,
) -> Result<SetupOutcome, RenderError> {
    let (answers, migrated_answers) = load_answers(request.answers)?;
    let resolution = load_resolution(request.capabilities)?;
    validate_answers(&answers)?;
    validate_resolution(&answers, &resolution)?;
    validate_skill_approvals(&answers.approved_optional_skills)?;
    validate_hook_approvals(&answers.approved_fallback_hooks, &resolution)?;
    validate_schema_instance(SETUP_SCHEMA, &migrated_answers, "setup answers")?;
    let effective_preferences =
        resolve_effective_project_preferences(&answers, request.global_preferences.as_ref())?;
    let planned = render_setup_tree(
        target_dir,
        &answers,
        &resolution,
        &request.reconfigure_roles,
        request.mode,
        effective_preferences.as_ref(),
    )?;
    if request.mode == SetupMode::Validate {
        if verify_ambient_target {
            verify_target_capability_current(request.target, target_dir)
                .map_err(as_verification)?;
        }
        for path in planned.keys() {
            if verify_ambient_target {
                ensure_managed_no_symlink_ancestors(request.target, path)?;
            }
        }
        validate_installed_against(request.target, &answers, &resolution, &planned)?;
        let tree_digest = installed_tree_digest(request.target)?;
        if verify_ambient_target {
            verify_target_capability_current(request.target, target_dir)
                .map_err(as_verification)?;
        }
        return Ok(SetupOutcome {
            changed_paths: Vec::new(),
            resolved_owner: derived_owner(&resolution)?.to_owned(),
            tree_digest,
            changes: Vec::new(),
            effective_preferences: resolved_preferences(&answers, effective_preferences.as_ref()),
        });
    }
    let projection_transition = prepare_operation_projection_transition(
        target_dir,
        &planned,
        &answers,
        release_source_version,
    )?;
    validate_owned_paths(
        planned.keys(),
        &projection_transition.desired,
        projection_transition.previous.as_ref(),
    )?;
    for path in planned.keys() {
        if verify_ambient_target {
            ensure_managed_no_symlink_ancestors(request.target, path)?;
        }
    }
    let mut deletions =
        stale_hook_deletions(target_dir, &answers.approved_fallback_hooks, &resolution)?;
    deletions.extend(projection_transition.deletions.iter().cloned());
    validate_owned_paths(
        deletions.iter(),
        &projection_transition.desired,
        projection_transition.previous.as_ref(),
    )?;
    for path in &deletions {
        if verify_ambient_target {
            ensure_managed_no_symlink_ancestors(request.target, path)?;
        }
    }

    let changed_paths = differing_paths(target_dir, &planned, &deletions)?;
    let changes = setup_changes(target_dir, &changed_paths, &planned)?;
    let tree_digest = digest_tree(&planned);
    match request.mode {
        SetupMode::DryRun => stage_and_validate(
            verify_ambient_target.then_some(request.target),
            &planned,
            &answers,
            &resolution,
            staging_corruption_from_environment(),
        )?,
        SetupMode::Apply if !changed_paths.is_empty() => {
            activate_staged(
                request.target,
                target_dir,
                &planned,
                &deletions,
                &answers,
                &resolution,
                &projection_transition.expected_before,
                verify_ambient_target,
                post_apply,
            )?;
        }
        SetupMode::Apply => {
            if let Some(commit) = post_apply {
                commit()?;
            }
        }
        SetupMode::Validate => {}
    }
    if verify_ambient_target {
        verify_target_capability_current(request.target, target_dir)?;
    }

    Ok(SetupOutcome {
        changed_paths,
        resolved_owner: derived_owner(&resolution)?.to_owned(),
        tree_digest,
        changes,
        effective_preferences: resolved_preferences(&answers, effective_preferences.as_ref()),
    })
}

fn resolved_preferences(
    answers: &SetupAnswers,
    effective: Option<&EffectiveProjectPreferences>,
) -> Option<ResolvedProjectPreferences> {
    effective.map(|preferences| ResolvedProjectPreferences {
        setup_mode: answers.setup_mode.clone(),
        provenance: preferences.provenance.to_owned(),
        interface_language: preferences.interface_language.clone(),
        wiki_enabled: preferences.wiki_enabled,
        wiki_language: preferences.wiki_language.clone(),
        persona_id: preferences.persona_id.clone(),
        persona_custom_description: preferences.persona_custom_description.clone(),
        selected_project_skills: preferences.selected_project_skills.clone(),
        usage_guard_enabled: preferences.usage_guard_enabled,
        codexbar_fallback_enabled: preferences.codexbar_fallback_enabled,
        usage_stop_remaining_percent: preferences.usage_stop_remaining_percent,
    })
}

fn render_setup_tree(
    target_dir: &Dir,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    reconfigure_roles: &BTreeSet<String>,
    mode: SetupMode,
    effective_preferences: Option<&EffectiveProjectPreferences>,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, RenderError> {
    let result = render_tree_with_preferences(
        target_dir,
        answers,
        resolution,
        reconfigure_roles,
        effective_preferences,
    );
    if mode == SetupMode::Validate {
        result.map_err(as_verification)
    } else {
        result
    }
}

fn read_bytes(path: &Path, label: &str) -> Result<Vec<u8>, RenderError> {
    fs::read(path).map_err(|error| {
        RenderError::Input(format!("cannot read {label} {}: {error}", path.display()))
    })
}

trait TargetRead {
    fn read_optional(&self, relative: &Path) -> Result<Option<Vec<u8>>, RenderError>;
}

impl TargetRead for Path {
    fn read_optional(&self, relative: &Path) -> Result<Option<Vec<u8>>, RenderError> {
        ensure_managed_no_symlink_ancestors(self, relative)?;
        let absolute = self.join(relative);
        match fs::symlink_metadata(&absolute) {
            Ok(metadata) if metadata.is_file() => {
                fs::read(&absolute).map(Some).map_err(io_internal)
            }
            Ok(_) => Err(RenderError::Conflict(format!(
                "managed file path is occupied by a non-file: {}",
                absolute.display()
            ))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io_internal(error)),
        }
    }
}

impl TargetRead for PathBuf {
    fn read_optional(&self, relative: &Path) -> Result<Option<Vec<u8>>, RenderError> {
        self.as_path().read_optional(relative)
    }
}

impl TargetRead for Dir {
    fn read_optional(&self, relative: &Path) -> Result<Option<Vec<u8>>, RenderError> {
        read_capability_optional(self, relative)
    }
}

fn read_target_optional<T: TargetRead + ?Sized>(
    target: &T,
    relative: &Path,
) -> Result<Option<Vec<u8>>, RenderError> {
    target.read_optional(relative)
}

fn read_target_required<T: TargetRead + ?Sized>(
    target: &T,
    relative: &Path,
    label: &str,
) -> Result<Vec<u8>, RenderError> {
    read_target_optional(target, relative)?.ok_or_else(|| {
        RenderError::Verification(format!(
            "required installed {label} is missing: {}",
            relative.display()
        ))
    })
}

fn load_answers(path: &Path) -> Result<(SetupAnswers, JsonValue), RenderError> {
    let bytes = read_bytes(path, "setup answers")?;
    let mut value: JsonValue = serde_yaml::from_slice(&bytes)
        .map_err(|error| RenderError::Input(format!("invalid setup answer YAML: {error}")))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| RenderError::Input("setup answers must be an object".to_owned()))?;
    object.remove("orchestration_layer");
    object
        .entry("setup_mode")
        .or_insert_with(|| JsonValue::String("expedited".to_owned()));
    object
        .entry("approved_fallback_hooks")
        .or_insert_with(|| JsonValue::Array(Vec::new()));
    let project_name = object
        .get("project_name")
        .cloned()
        .unwrap_or_else(|| JsonValue::String(String::new()));
    object.entry("project_identity").or_insert(project_name);
    for key in [
        "root_knowledge_promotion_categories",
        "confidential_knowledge_categories",
    ] {
        object
            .entry(key)
            .or_insert_with(|| JsonValue::Array(Vec::new()));
    }
    object
        .entry("user_store_binding")
        .or_insert_with(|| JsonValue::String(String::new()));
    let answers: SetupAnswers = serde_json::from_value(value.clone()).map_err(|error| {
        RenderError::Input(format!("setup answers violate the contract: {error}"))
    })?;
    Ok((answers, value))
}

fn load_resolution(path: &Path) -> Result<CapabilityResolution, RenderError> {
    let bytes = read_bytes(path, "capability evidence")?;
    let value: JsonValue = serde_json::from_slice(&bytes).map_err(|error| {
        RenderError::Input(format!(
            "capability evidence violates the contract: {error}"
        ))
    })?;
    validate_schema_instance(CAPABILITY_SCHEMA, &value, "capability evidence")?;
    serde_json::from_value(value).map_err(|error| {
        RenderError::Input(format!(
            "capability evidence violates the contract: {error}"
        ))
    })
}

fn validate_answers(answers: &SetupAnswers) -> Result<(), RenderError> {
    if answers.schema_version != 1
        || answers.project_name.trim().is_empty()
        || !matches!(answers.setup_mode.as_str(), "expedited" | "custom")
        || !matches!(answers.project_kind.as_str(), "general" | "custom")
        || !matches!(
            answers.primary_host.as_str(),
            "codex" | "claude" | "antigravity"
        )
        || !(1..=99).contains(&answers.usage_stop_remaining_percent)
        || answers.elevated_judge_quorum != "2/3"
        || answers.critical_judge_quorum != "3/3+human"
    {
        return Err(RenderError::Input(
            "setup answers violate required scalar constraints".to_owned(),
        ));
    }
    validate_project_setup_preferences(answers)?;
    let mut ids = BTreeSet::new();
    let mut role_paths = BTreeSet::from([".hive/team/roles/readme.md".to_owned()]);
    for role in &answers.persistent_roles {
        validate_role_seed(role)?;
        if !ids.insert(role.role_id.to_lowercase()) {
            return Err(RenderError::Input(format!(
                "duplicate or case-fold-colliding role id: {}",
                role.role_id
            )));
        }
        let role_path = format!(".hive/team/roles/{}.md", role.role_id).to_lowercase();
        if !role_paths.insert(role_path) {
            return Err(RenderError::Input(format!(
                "persistent role path collides with another managed path: {}",
                role.role_id
            )));
        }
    }
    for path in answers
        .knowledge_include_paths
        .iter()
        .chain(&answers.knowledge_exclude_paths)
    {
        if path.is_empty() || Path::new(path).is_absolute() || contains_parent_component(path) {
            return Err(RenderError::Input(format!(
                "unsafe knowledge scope path: {path}"
            )));
        }
    }
    let allowed_promotion_categories = ["fact", "preference", "workflow"];
    for (name, categories) in [
        (
            "root_knowledge_promotion_categories",
            &answers.root_knowledge_promotion_categories,
        ),
        (
            "confidential_knowledge_categories",
            &answers.confidential_knowledge_categories,
        ),
    ] {
        if categories.windows(2).any(|pair| pair[0] >= pair[1])
            || categories
                .iter()
                .any(|category| !allowed_promotion_categories.contains(&category.as_str()))
        {
            return Err(RenderError::Input(format!(
                "{name} must be a sorted unique subset of fact, preference, and workflow"
            )));
        }
    }
    if answers
        .root_knowledge_promotion_categories
        .iter()
        .any(|category| answers.confidential_knowledge_categories.contains(category))
    {
        return Err(RenderError::Input(
            "root promotion and confidential knowledge categories must not overlap".to_owned(),
        ));
    }
    if (!answers.project_identity.is_empty()
        && (answers.project_identity.trim() != answers.project_identity
            || answers.project_identity.len() > 160
            || answers.project_identity.contains(['\r', '\n', '\0'])))
        || (!answers.user_store_binding.is_empty() && !valid_digest(&answers.user_store_binding))
        || (!answers.root_knowledge_promotion_categories.is_empty()
            && answers.user_store_binding.is_empty())
    {
        return Err(RenderError::Input(
            "project identity or user-store binding is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_project_setup_preferences(answers: &SetupAnswers) -> Result<(), RenderError> {
    match answers.setup_mode.as_str() {
        "expedited"
            if answers.interface_language.is_some()
                || answers.wiki.is_some()
                || answers.persona.is_some()
                || answers.skills.is_some() =>
        {
            return Err(RenderError::Input(
                "expedited project setup must inherit global preferences".to_owned(),
            ));
        }
        "custom"
            if answers.interface_language.is_none()
                || answers.wiki.is_none()
                || answers.persona.is_none()
                || answers.skills.is_none() =>
        {
            return Err(RenderError::Input(
                "custom project setup requires explicit preference overrides".to_owned(),
            ));
        }
        _ => {}
    }
    Ok(())
}

#[derive(Deserialize)]
struct ProjectSuiteCatalog {
    schema_version: u32,
    recommended_skill_suites: Vec<ProjectSuite>,
    mandatory_skills: Vec<String>,
    skill_dependencies: Vec<ProjectSkillDependency>,
}

#[derive(Deserialize)]
struct ProjectSuite {
    id: String,
    skills: Vec<String>,
}

#[derive(Deserialize)]
struct ProjectSkillDependency {
    skill: String,
    requires: Vec<String>,
}

fn resolve_effective_project_preferences(
    answers: &SetupAnswers,
    global: Option<&GlobalProjectPreferences>,
) -> Result<Option<EffectiveProjectPreferences>, RenderError> {
    let Some(global) = global else {
        return Ok(None);
    };
    validate_global_project_preferences(global)?;
    let mut effective = match answers.setup_mode.as_str() {
        "expedited" => EffectiveProjectPreferences {
            provenance: "global-inherited",
            interface_language: global.interface_language.clone(),
            wiki_enabled: global.wiki_enabled,
            wiki_language: global.wiki_language.clone(),
            persona_id: global.persona_id.clone(),
            persona_custom_description: global.persona_custom_description.clone(),
            selected_project_skills: global.selected_project_skills.clone(),
            usage_guard_enabled: global.usage_guard_enabled,
            codexbar_fallback_enabled: global.codexbar_fallback_enabled,
            usage_stop_remaining_percent: global.usage_stop_remaining_percent,
        },
        "custom" => {
            let wiki = answers
                .wiki
                .as_ref()
                .expect("custom preferences were validated");
            if wiki.enabled && !global.wiki_enabled {
                return Err(RenderError::Safety(
                    "project Wiki cannot be enabled while the global Wiki is disabled".to_owned(),
                ));
            }
            let persona = answers
                .persona
                .as_ref()
                .expect("custom preferences were validated");
            EffectiveProjectPreferences {
                provenance: "project-custom",
                interface_language: answers
                    .interface_language
                    .clone()
                    .expect("custom preferences were validated"),
                wiki_enabled: wiki.enabled,
                wiki_language: wiki.language.clone(),
                persona_id: persona.id.clone(),
                persona_custom_description: persona.custom_description.clone(),
                selected_project_skills: resolve_project_skill_selection(
                    answers
                        .skills
                        .as_ref()
                        .expect("custom preferences were validated"),
                )?,
                usage_guard_enabled: global.usage_guard_enabled,
                codexbar_fallback_enabled: global.codexbar_fallback_enabled,
                usage_stop_remaining_percent: global.usage_stop_remaining_percent,
            }
        }
        _ => unreachable!("setup mode was validated"),
    };
    effective.selected_project_skills.sort();
    Ok(Some(effective))
}

fn validate_global_project_preferences(
    global: &GlobalProjectPreferences,
) -> Result<(), RenderError> {
    let custom_persona_valid = global.persona_id == "custom"
        && global
            .persona_custom_description
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty() && !value.contains(['\r', '\n']));
    let standard_persona_valid = matches!(
        global.persona_id.as_str(),
        "strict" | "balanced" | "friendly"
    ) && global.persona_custom_description.is_none();
    let unique_skills = global
        .selected_project_skills
        .iter()
        .collect::<BTreeSet<_>>()
        .len()
        == global.selected_project_skills.len();
    if !matches!(global.interface_language.as_str(), "en" | "ko")
        || !matches!(global.wiki_language.as_str(), "en" | "ko" | "both")
        || !(custom_persona_valid || standard_persona_valid)
        || !unique_skills
        || global
            .selected_project_skills
            .iter()
            .any(|name| name == "setup-hive")
        || (global.codexbar_fallback_enabled && !global.usage_guard_enabled)
        || !(1..=99).contains(&global.usage_stop_remaining_percent)
    {
        return Err(RenderError::Input(
            "global project preferences violate the typed bridge contract".to_owned(),
        ));
    }
    Ok(())
}

fn resolve_project_skill_selection(
    selection: &ProjectSkillSelection,
) -> Result<Vec<String>, RenderError> {
    let catalog = project_skill_catalog()?;
    let available = embedded_catalog()
        .map_err(|error| RenderError::Internal(error.to_string()))?
        .skills
        .into_iter()
        .filter(|entry| entry.availability == Availability::Implemented)
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    validate_project_skill_catalog(&catalog, &available)?;
    let recommended = selection.mode == "recommended";
    let mut selected = match selection.mode.as_str() {
        "individual" => selection
            .selected
            .clone()
            .expect("individual selection was schema-validated")
            .into_iter()
            .collect::<BTreeSet<_>>(),
        "recommended" => {
            let suite_id = selection
                .recommended_suite
                .as_deref()
                .expect("recommended selection was schema-validated");
            catalog
                .recommended_skill_suites
                .iter()
                .find(|suite| suite.id == suite_id)
                .map(|suite| suite.skills.iter().cloned().collect())
                .ok_or_else(|| {
                    RenderError::Internal(format!(
                        "embedded recommended project Skill suite is missing: {suite_id}"
                    ))
                })?
        }
        _ => unreachable!("Skill selection mode was schema-validated"),
    };
    if recommended {
        selected.remove("setup-hive");
    } else if selected.contains("setup-hive") {
        return Err(RenderError::Input(
            "setup-hive is user-scope only and cannot be selected for a project".to_owned(),
        ));
    }
    let dependencies = catalog
        .skill_dependencies
        .iter()
        .map(|entry| (entry.skill.as_str(), entry.requires.as_slice()))
        .collect::<BTreeMap<_, _>>();
    loop {
        let before = selected.len();
        for name in selected.clone() {
            if let Some(required) = dependencies.get(name.as_str()) {
                selected.extend(required.iter().cloned());
            }
        }
        if selected.len() == before {
            break;
        }
    }
    if let Some(name) = selected.iter().find(|name| !available.contains(*name)) {
        return Err(RenderError::Input(format!(
            "selected project Skill is not available in this release: {name}"
        )));
    }
    Ok(selected.into_iter().collect())
}

fn project_skill_catalog() -> Result<ProjectSuiteCatalog, RenderError> {
    serde_yaml::from_str(include_str!("../../../harness/user-setup/catalog.yml")).map_err(|error| {
        RenderError::Internal(format!("embedded user setup catalog is invalid: {error}"))
    })
}

fn validate_project_skill_catalog(
    catalog: &ProjectSuiteCatalog,
    available: &BTreeSet<String>,
) -> Result<(), RenderError> {
    let mut suite_ids = BTreeSet::new();
    let suites_valid = catalog.recommended_skill_suites.iter().all(|suite| {
        suite_ids.insert(suite.id.as_str())
            && !suite.skills.is_empty()
            && project_skill_names_are_valid(&suite.skills, available)
    });
    let mut dependency_keys = BTreeSet::new();
    let dependencies_valid = catalog.skill_dependencies.iter().all(|dependency| {
        dependency_keys.insert(dependency.skill.as_str())
            && available.contains(&dependency.skill)
            && project_skill_names_are_valid(&dependency.requires, available)
    });
    if catalog.schema_version != 1
        || catalog.mandatory_skills != ["setup-hive"]
        || suite_ids != BTreeSet::from(["game-developer", "non-developer", "web-developer"])
        || !suites_valid
        || !dependencies_valid
    {
        return Err(RenderError::Internal(
            "embedded user setup catalog violates project Skill semantics".to_owned(),
        ));
    }
    Ok(())
}

fn project_skill_names_are_valid(names: &[String], available: &BTreeSet<String>) -> bool {
    let mut unique = BTreeSet::new();
    names
        .iter()
        .all(|name| available.contains(name) && unique.insert(name.as_str()))
}

fn validate_role_seed(role: &RoleSeed) -> Result<(), RenderError> {
    if !valid_role_id(&role.role_id)
        || role.display_name.is_empty()
        || role.display_name.contains(['\r', '\n'])
        || role.responsibilities.is_empty()
        || role.verification_duties.is_empty()
        || role
            .responsibilities
            .iter()
            .chain(&role.non_responsibilities)
            .chain(&role.context_paths)
            .chain(&role.write_scope)
            .chain(&role.verification_duties)
            .any(String::is_empty)
    {
        return Err(RenderError::Input(format!(
            "role seed violates the role schema: {}",
            role.role_id
        )));
    }
    let allowed = [
        "filesystem-read",
        "filesystem-write",
        "shell",
        "network",
        "subagents",
        "external-app",
    ];
    if role
        .allowed_capabilities
        .iter()
        .any(|capability| !allowed.contains(&capability.as_str()))
        || !is_unique(&role.allowed_capabilities)
    {
        return Err(RenderError::Input(format!(
            "role capability is invalid: {}",
            role.role_id
        )));
    }
    Ok(())
}

fn validate_resolution(
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    validate_resolution_host(&answers.primary_host, resolution)
}

fn validate_resolution_host(
    expected_host: &str,
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    if resolution.schema_version != 1
        || resolution.host != expected_host
        || resolution.host_version.is_empty()
        || !matches!(
            resolution.surface.as_str(),
            "app" | "cli" | "plugin" | "in-session"
        )
        || !valid_digest(&resolution.evidence_digest)
        || resolution.evidence.is_empty()
    {
        return Err(RenderError::Input(
            "capability evidence does not match setup host or schema".to_owned(),
        ));
    }
    for evidence in &resolution.evidence {
        if !matches!(
            evidence.source.as_str(),
            "host-catalog" | "public-executable"
        ) || evidence.locator.is_empty()
            || !matches!(
                evidence.outcome.as_str(),
                "compatible" | "absent" | "incompatible" | "unavailable"
            )
            || !valid_digest(&evidence.digest)
        {
            return Err(RenderError::Input(
                "capability evidence entry is invalid".to_owned(),
            ));
        }
    }
    let mut digest_payload = serde_json::to_value(resolution).map_err(|error| {
        RenderError::Internal(format!("cannot encode capability evidence: {error}"))
    })?;
    digest_payload
        .as_object_mut()
        .ok_or_else(|| RenderError::Internal("capability evidence is not an object".to_owned()))?
        .remove("evidence_digest");
    let canonical = serde_json_canonicalizer::to_vec(&digest_payload).map_err(|error| {
        RenderError::Internal(format!(
            "cannot canonicalize capability evidence with RFC 8785: {error}"
        ))
    })?;
    if sha256_digest(&canonical) != resolution.evidence_digest {
        return Err(RenderError::Input(
            "capability evidence digest does not bind the full normalized object".to_owned(),
        ));
    }
    let (detection, owner, external_runtime) = derive_resolution(resolution)?;
    if resolution.detection != detection {
        return Err(RenderError::Input(format!(
            "capability detection must be derived from evidence as {detection}"
        )));
    }
    if resolution.resolved_owner != owner {
        return Err(RenderError::Input(format!(
            "capability owner must resolve automatically to {owner}"
        )));
    }
    if resolution.external_runtime.as_deref() != external_runtime {
        return Err(RenderError::Input(
            "external runtime does not match host and evidence".to_owned(),
        ));
    }
    Ok(())
}

fn derived_owner(resolution: &CapabilityResolution) -> Result<&'static str, RenderError> {
    derive_resolution(resolution).map(|(_, owner, _)| owner)
}

fn derive_resolution(
    resolution: &CapabilityResolution,
) -> Result<(&'static str, &'static str, Option<&'static str>), RenderError> {
    let expected_runtime = match resolution.host.as_str() {
        "codex" => Some("omx"),
        "claude" => Some("omc"),
        "antigravity" => None,
        _ => {
            return Err(RenderError::Input(
                "unsupported host in capability evidence".to_owned(),
            ));
        }
    };
    let outcomes: BTreeSet<_> = resolution
        .evidence
        .iter()
        .map(|item| item.outcome.as_str())
        .collect();
    let compatible = outcomes.contains("compatible");
    let incompatible = outcomes.contains("incompatible");
    let absent = outcomes.contains("absent");
    let unavailable = outcomes.contains("unavailable");
    if compatible && (incompatible || absent) || incompatible && absent {
        return Err(RenderError::Input(
            "capability evidence contains contradictory outcomes".to_owned(),
        ));
    }
    if resolution.host == "antigravity" {
        if compatible || incompatible || resolution.external_runtime.is_some() {
            return Err(RenderError::Input(
                "Antigravity must always resolve host-native".to_owned(),
            ));
        }
        if absent {
            let complete_absence = resolution
                .evidence
                .iter()
                .any(|item| item.source == "host-catalog" && item.outcome == "absent")
                && resolution
                    .evidence
                    .iter()
                    .any(|item| item.source == "public-executable" && item.outcome == "absent");
            if complete_absence && !unavailable {
                return Ok(("absent", "host-native", None));
            }
        }
        return Ok(("unknown", "host-native", None));
    }
    if compatible {
        return Ok((
            "available",
            expected_runtime.expect("Codex and Claude have an expected runtime"),
            expected_runtime,
        ));
    }
    if incompatible {
        return Ok(("incompatible", "host-native", expected_runtime));
    }
    let catalog_absent = resolution
        .evidence
        .iter()
        .any(|item| item.source == "host-catalog" && item.outcome == "absent");
    let executable_absent = resolution
        .evidence
        .iter()
        .any(|item| item.source == "public-executable" && item.outcome == "absent");
    if catalog_absent && executable_absent && !unavailable {
        return Ok(("absent", "host-native", None));
    }
    if unavailable || absent {
        return Ok(("unknown", "host-native", None));
    }
    Err(RenderError::Input(
        "capability evidence cannot derive a detection state".to_owned(),
    ))
}

fn validate_skill_approvals(skills: &[SkillApproval]) -> Result<(), RenderError> {
    let allowed_capabilities = [
        "filesystem-read",
        "filesystem-write",
        "shell",
        "network",
        "subagents",
        "external-app",
    ];
    for skill in skills {
        if skill.consent_version != 1
            || skill.name.is_empty()
            || skill.source.is_empty()
            || skill.revision.is_empty()
            || !valid_digest(&skill.content_digest)
            || !valid_timestamp(&skill.approved_at)
            || !strictly_sorted_unique(&skill.requested_capabilities)
            || !strictly_sorted_unique(&skill.approved_capabilities)
            || skill
                .requested_capabilities
                .iter()
                .chain(&skill.approved_capabilities)
                .any(|capability| !allowed_capabilities.contains(&capability.as_str()))
        {
            return Err(RenderError::Input(format!(
                "optional Skill approval is malformed: {}",
                skill.name
            )));
        }
        let requested: BTreeSet<_> = skill.requested_capabilities.iter().collect();
        if skill
            .approved_capabilities
            .iter()
            .any(|capability| !requested.contains(capability))
        {
            return Err(RenderError::Input(format!(
                "approved capabilities exceed requested capabilities: {}",
                skill.name
            )));
        }
        verify_consent_digest(skill, &skill.consent_digest, "optional Skill")?;
    }
    Ok(())
}

fn validate_hook_approvals(
    hooks: &[HookApproval],
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    if !hooks.is_empty() && resolution.detection != "absent" {
        return Err(RenderError::Safety(
            "fallback hooks require conclusive external capability absence".to_owned(),
        ));
    }
    let mut identities = BTreeSet::new();
    for hook in hooks {
        if hook.consent_version != 1
            || !valid_digest(&hook.content_digest)
            || !valid_timestamp(&hook.approved_at)
            || !identities.insert(format!("{}:{}", hook.capability, hook.event))
        {
            return Err(RenderError::Input(
                "fallback hook approval is malformed".to_owned(),
            ));
        }
        validate_project_relative(Path::new(&hook.path))
            .map_err(|error| RenderError::Input(error.to_string()))?;
        let expected_event = match hook.capability.as_str() {
            "protect-hive-owned-state" | "update-integrity-guard" => "PreToolUse",
            "derived-state-invalidation" => "PostToolUse",
            "checkpoint-reminder" if matches!(hook.event.as_str(), "PreCompact" | "Stop") => {
                hook.event.as_str()
            }
            _ => {
                return Err(RenderError::Input(format!(
                    "fallback hook capability/event is not approved: {}/{}",
                    hook.capability, hook.event
                )));
            }
        };
        let expected_path = format!(".hive/hooks/{}", hook.capability);
        let expected_command = format!(
            "hive hook --capability {} --event {expected_event} \
             --capabilities {FRESH_CAPABILITY_RESOLUTION_PATH} --output json",
            hook.capability,
        );
        if hook.event != expected_event
            || hook.path != expected_path
            || hook.command != expected_command
        {
            return Err(RenderError::Input(format!(
                "fallback hook preview changed: {}",
                hook.capability
            )));
        }
        let descriptor = hook_descriptor_bytes(hook)?;
        if sha256_digest(&descriptor) != hook.content_digest {
            return Err(RenderError::Safety(format!(
                "fallback hook descriptor digest changed: {}",
                hook.capability
            )));
        }
        verify_consent_digest(hook, &hook.consent_digest, "fallback hook")?;
    }
    Ok(())
}

fn hook_descriptor_bytes(hook: &HookApproval) -> Result<Vec<u8>, RenderError> {
    let descriptor = HookDescriptor {
        capability: &hook.capability,
        command: &hook.command,
        event: &hook.event,
        path: &hook.path,
        schema_version: 1,
    };
    let mut bytes = serde_json_canonicalizer::to_vec(&descriptor).map_err(|error| {
        RenderError::Internal(format!(
            "cannot canonicalize fallback hook descriptor with RFC 8785: {error}"
        ))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn verify_consent_digest<T: Serialize>(
    approval: &T,
    expected: &str,
    kind: &str,
) -> Result<(), RenderError> {
    if calculate_consent_digest(approval)? != expected {
        return Err(RenderError::Safety(format!(
            "{kind} consent digest does not match the approved payload"
        )));
    }
    Ok(())
}

fn calculate_consent_digest<T: Serialize>(approval: &T) -> Result<String, RenderError> {
    let mut value = serde_json::to_value(approval)
        .map_err(|error| RenderError::Internal(format!("cannot encode consent: {error}")))?;
    value
        .as_object_mut()
        .ok_or_else(|| RenderError::Internal("consent is not an object".to_owned()))?
        .remove("consent_digest");
    let canonical = serde_json_canonicalizer::to_vec(&value).map_err(|error| {
        RenderError::Internal(format!(
            "cannot canonicalize consent with RFC 8785: {error}"
        ))
    })?;
    Ok(sha256_digest(&canonical))
}

#[cfg(test)]
fn render_tree<T: TargetRead + ?Sized>(
    target: &T,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    reconfigure_roles: &BTreeSet<String>,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, RenderError> {
    render_tree_with_preferences(target, answers, resolution, reconfigure_roles, None)
}

fn render_tree_with_preferences<T: TargetRead + ?Sized>(
    target: &T,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    reconfigure_roles: &BTreeSet<String>,
    effective_preferences: Option<&EffectiveProjectPreferences>,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, RenderError> {
    let mut files = BTreeMap::new();
    insert_static_files(&mut files);
    preserve_protected_seeds(target, &mut files)?;
    if effective_preferences.is_some_and(|preferences| !preferences.wiki_enabled) {
        files.remove(Path::new(".hive/knowledge/Wiki/index.md"));
        files.remove(Path::new(".hive/knowledge/Wiki/log.md"));
    }
    let optional_sources = load_optional_skill_sources(target, &answers.approved_optional_skills)?;
    let portable_projection = compile_effective_projection(
        ProjectionHost::Codex,
        &optional_sources,
        effective_preferences,
    )?;
    for (path, bytes) in portable_projection.files {
        files.insert(PathBuf::from(path), bytes);
    }
    if answers.primary_host == "claude" {
        let claude_projection = compile_effective_projection(
            ProjectionHost::Claude,
            &optional_sources,
            effective_preferences,
        )?;
        for (path, bytes) in claude_projection
            .files
            .into_iter()
            .filter(|(path, _)| path.starts_with(".claude/skills/"))
        {
            files.insert(PathBuf::from(path), bytes);
        }
    }
    files.insert(
        PathBuf::from(".hive/setup-answers.yml"),
        render_setup_answers(answers)?,
    );
    files.insert(
        PathBuf::from(".hive/config/role-seeds.yml"),
        render_yaml_projection(
            Some("# Initial persistent-role definitions selected during setup."),
            "roles",
            &answers.persistent_roles,
        )?,
    );
    files.insert(
        PathBuf::from(".hive/config/knowledge-scope.yml"),
        render_knowledge_scope(answers)?,
    );
    files.insert(
        PathBuf::from(".hive/config/approved-skills.yml"),
        render_yaml_projection(
            Some("# Generated from explicit setup approvals."),
            "skills",
            &answers.approved_optional_skills,
        )?,
    );
    files.insert(
        PathBuf::from(".hive/config/capability-resolution.yml"),
        render_capability_resolution(resolution)?,
    );
    if !answers.approved_fallback_hooks.is_empty() {
        files.insert(
            PathBuf::from(".hive/config/approved-hooks.yml"),
            render_hook_ledger(&answers.approved_fallback_hooks, resolution),
        );
        for hook in &answers.approved_fallback_hooks {
            files.insert(PathBuf::from(&hook.path), hook_descriptor_bytes(hook)?);
        }
    }
    files.insert(
        PathBuf::from(".hive/config/harness.toml"),
        render_harness_toml(answers, resolution, effective_preferences).into_bytes(),
    );
    let marker = render_agents_marker(answers, resolution, effective_preferences);
    let merged = merge_shared_marker(target, Path::new("AGENTS.md"), marker.as_bytes())?;
    files.insert(PathBuf::from("AGENTS.md"), merged);
    for adapter in ["CLAUDE.md", "GEMINI.md"] {
        let marker = format!("{MARKER_START}\n@AGENTS.md\n{MARKER_END}\n");
        let merged = merge_shared_marker(target, Path::new(adapter), marker.as_bytes())?;
        files.insert(PathBuf::from(adapter), merged);
    }
    render_roles(target, answers, reconfigure_roles, &mut files)?;
    let base = render_project_base(&files)?;
    files.insert(PathBuf::from(".hive/config/project-base.json"), base);
    Ok(files)
}

fn compile_effective_projection(
    host: ProjectionHost,
    optional_sources: &[OptionalSkillSource],
    effective_preferences: Option<&EffectiveProjectPreferences>,
) -> Result<Projection, RenderError> {
    let result = effective_preferences.map_or_else(
        || compile_projection(host, optional_sources),
        |preferences| {
            compile_project_projection(host, &preferences.selected_project_skills, optional_sources)
        },
    );
    result.map_err(|error| map_projection_error(&error))
}

fn project_upgrade_files(
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<BTreeMap<String, Vec<u8>>, RenderError> {
    let mut mergeable = BTreeMap::new();
    for (path, bytes) in files {
        let content =
            if is_hive_skill_projection_path(path) || is_hive_directive_projection_path(path) {
                bytes.clone()
            } else if matches!(path.to_str(), Some("AGENTS.md" | "CLAUDE.md" | "GEMINI.md")) {
                extract_exact_marker(bytes)?
            } else {
                continue;
            };
        let path = path
            .to_str()
            .ok_or_else(|| RenderError::Internal("upgrade path is not UTF-8".to_owned()))?;
        mergeable.insert(path.to_owned(), content);
    }
    Ok(mergeable)
}

fn render_project_base(files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<Vec<u8>, RenderError> {
    let mergeable = project_upgrade_files(files)?;
    let mut entries = Vec::new();
    for (path, bytes) in mergeable {
        let content = String::from_utf8(bytes.clone())
            .map_err(|_| RenderError::Internal(format!("project base is not UTF-8: {path}")))?;
        let kind = if matches!(path.as_str(), "AGENTS.md" | "CLAUDE.md" | "GEMINI.md") {
            "shared-marker"
        } else if path.contains("/skills/") {
            "skill"
        } else {
            "directive"
        };
        entries.push(ProjectBaseFile {
            path,
            kind: kind.to_owned(),
            content_digest: sha256_digest(&bytes),
            content,
        });
    }
    let mut value = serde_json::json!({
        "schema_version": 1,
        "product_version": env!("CARGO_PKG_VERSION"),
        "files": entries,
    });
    let canonical = serde_json_canonicalizer::to_vec(&value).map_err(|error| {
        RenderError::Internal(format!("cannot canonicalize project base ledger: {error}"))
    })?;
    let digest = sha256_digest(&canonical);
    value
        .as_object_mut()
        .expect("project base payload is an object")
        .insert("ledger_digest".to_owned(), JsonValue::String(digest));
    let mut bytes = serde_json_canonicalizer::to_vec(&value).map_err(|error| {
        RenderError::Internal(format!("cannot canonicalize project base ledger: {error}"))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn extract_exact_marker(bytes: &[u8]) -> Result<Vec<u8>, RenderError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| RenderError::Conflict("shared guidance is not UTF-8".to_owned()))?;
    let start = text.find(MARKER_START).ok_or_else(|| {
        RenderError::Conflict("shared guidance is missing Hive marker".to_owned())
    })?;
    let rest = &text[start + MARKER_START.len()..];
    let end_offset = rest.find(MARKER_END).ok_or_else(|| {
        RenderError::Conflict("shared guidance has an unterminated Hive marker".to_owned())
    })?;
    if rest[end_offset + MARKER_END.len()..].contains(MARKER_START)
        || text[..start].contains(MARKER_END)
    {
        return Err(RenderError::Conflict(
            "shared guidance contains malformed Hive markers".to_owned(),
        ));
    }
    let end = start + MARKER_START.len() + end_offset + MARKER_END.len();
    let mut marker = text.as_bytes()[start..end].to_vec();
    marker.push(b'\n');
    Ok(marker)
}

fn projection_host(host: &str) -> Result<ProjectionHost, RenderError> {
    match host {
        "codex" => Ok(ProjectionHost::Codex),
        "claude" => Ok(ProjectionHost::Claude),
        "antigravity" => Ok(ProjectionHost::Antigravity),
        _ => Err(RenderError::Input(format!(
            "unsupported Skill projection host: {host}"
        ))),
    }
}

fn load_optional_skill_sources<T: TargetRead + ?Sized>(
    target: &T,
    approvals: &[SkillApproval],
) -> Result<Vec<OptionalSkillSource>, RenderError> {
    let mut sources = Vec::new();
    for approval in approvals {
        let Some(relative) = approval.source.strip_prefix("path:") else {
            continue;
        };
        if approval.approved_capabilities != approval.requested_capabilities {
            continue;
        }
        let relative = PathBuf::from(relative);
        validate_project_relative(&relative)
            .map_err(|error| RenderError::Safety(error.to_string()))?;
        let skill_md = read_target_optional(target, &relative)?.ok_or_else(|| {
            RenderError::Safety(format!(
                "approved optional Skill source is missing: {}",
                relative.display()
            ))
        })?;
        sources.push(
            optional_source_from_approval(approval, skill_md).map_err(|error| {
                RenderError::Input(format!(
                    "optional Skill approval cannot be projected: {error}"
                ))
            })?,
        );
    }
    Ok(sources)
}

fn map_projection_error(error: &hive_projection::ProjectionError) -> RenderError {
    match error.code() {
        "hive.optional-skill-inert" => RenderError::Safety(error.to_string()),
        "hive.skill-projection-conflict" => RenderError::Conflict(error.to_string()),
        "hive.skill-catalog-invalid" => RenderError::Internal(error.to_string()),
        _ => RenderError::Input(error.to_string()),
    }
}

fn preserve_protected_seeds<T: TargetRead + ?Sized>(
    target: &T,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    let manifest = ownership_manifest()?;
    for (relative, bytes) in files.iter_mut() {
        let entry = ownership_entry(&manifest, relative)?;
        if entry.ownership == "canonical-data-protected" {
            if let Some(existing) = read_target_optional(target, relative)? {
                *bytes = existing;
            }
        }
    }
    Ok(())
}

fn insert_static_files(files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    const STATIC: &[(&str, &[u8])] = &[
        (
            ".hive/.gitignore",
            include_bytes!("../../../harness/template/.hive/.gitignore"),
        ),
        (
            ".hive/LICENSE-AIGENT-HIVE.txt",
            include_bytes!("../../../harness/LICENSE"),
        ),
        (
            ".hive/README.md",
            include_bytes!("../../../harness/template/.hive/README.md"),
        ),
        (
            ".hive/directives/00-editing-discipline.md",
            include_bytes!("../../../harness/template/.hive/directives/00-editing-discipline.md"),
        ),
        (
            ".agents/directives/00-project-harness.md",
            include_bytes!("../../../harness/directives/00-project-harness.md"),
        ),
        (
            ".agents/directives/01-project-knowledge.md",
            include_bytes!("../../../harness/directives/01-project-knowledge.md"),
        ),
        (
            ".agents/directives/02-project-upgrade.md",
            include_bytes!("../../../harness/directives/02-project-upgrade.md"),
        ),
        (
            ".hive/knowledge/Raw/README.md",
            include_bytes!("../../../harness/template/.hive/knowledge/Raw/README.md"),
        ),
        (
            ".hive/knowledge/Schema/schema.md",
            include_bytes!("../../../harness/template/.hive/knowledge/Schema/schema.md"),
        ),
        (
            ".hive/knowledge/Wiki/index.md",
            include_bytes!("../../../harness/template/.hive/knowledge/Wiki/index.md"),
        ),
        (
            ".hive/knowledge/Wiki/log.md",
            include_bytes!("../../../harness/template/.hive/knowledge/Wiki/log.md"),
        ),
        (
            ".hive/knowledge/suppression.yml",
            include_bytes!("../../../harness/template/.hive/knowledge/suppression.yml"),
        ),
        (
            ".hive/runs/README.md",
            include_bytes!("../../../harness/template/.hive/runs/README.md"),
        ),
        (
            ".hive/team/roles/README.md",
            include_bytes!("../../../harness/template/.hive/team/roles/README.md"),
        ),
    ];
    for (path, bytes) in STATIC {
        files.insert(PathBuf::from(path), bytes.to_vec());
    }
}

fn render_harness_toml(
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    effective_preferences: Option<&EffectiveProjectPreferences>,
) -> String {
    let quoted = |value: &str| {
        serde_json::to_string(value).expect("serializing a string to JSON cannot fail")
    };
    let mut output = format!(
        "schema_version = 1\nharness_version = {version}\nsource_release_version = {version}\nproject_name = {project}\nproject_kind = {kind}\nprimary_host = {host}\nexternal_capability_detection = {detection}\nresolved_owner = {owner}\nresolution_evidence_digest = {digest}\nusage_stop_remaining_percent = {usage}\nelevated_judge_quorum = {elevated}\ncritical_judge_quorum = {critical}\n\n# Optional Skills are inert until each entry is explicitly approved.\napproved_optional_skills_file = \".hive/config/approved-skills.yml\"\ncapability_resolution_file = \".hive/config/capability-resolution.yml\"\n",
        version = quoted(env!("CARGO_PKG_VERSION")),
        project = quoted(&answers.project_name),
        kind = quoted(&answers.project_kind),
        host = quoted(&answers.primary_host),
        detection = quoted(&resolution.detection),
        owner = quoted(&resolution.resolved_owner),
        digest = quoted(&resolution.evidence_digest),
        usage = effective_preferences.map_or(
            answers.usage_stop_remaining_percent,
            |preferences| preferences.usage_stop_remaining_percent,
        ),
        elevated = quoted(&answers.elevated_judge_quorum),
        critical = quoted(&answers.critical_judge_quorum),
    );
    if let Some(preferences) = effective_preferences {
        let selected = preferences
            .selected_project_skills
            .iter()
            .map(|name| quoted(name))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            &mut output,
            "setup_mode = {}\npreference_provenance = {}\ninterface_language = {}\nwiki_enabled = {}\nwiki_language = {}\npersona_id = {}\nusage_guard_enabled = {}\ncodexbar_fallback_enabled = {}\nselected_project_skills = [{selected}]\n",
            quoted(&answers.setup_mode),
            quoted(preferences.provenance),
            quoted(&preferences.interface_language),
            preferences.wiki_enabled,
            quoted(&preferences.wiki_language),
            quoted(&preferences.persona_id),
            preferences.usage_guard_enabled,
            preferences.codexbar_fallback_enabled,
        )
        .expect("writing to String cannot fail");
        if let Some(description) = &preferences.persona_custom_description {
            writeln!(
                &mut output,
                "persona_custom_description = {}",
                quoted(description)
            )
            .expect("writing to String cannot fail");
        }
    }
    if resolution.detection == "absent" && !answers.approved_fallback_hooks.is_empty() {
        output.push_str("approved_fallback_hooks_file = \".hive/config/approved-hooks.yml\"\n");
    }
    output.push_str(
        "role_seed_file = \".hive/config/role-seeds.yml\"\nknowledge_scope_file = \".hive/config/knowledge-scope.yml\"\n",
    );
    output
}

fn render_agents_marker(
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    effective_preferences: Option<&EffectiveProjectPreferences>,
) -> String {
    let preference_summary = effective_preferences.map_or_else(String::new, |preferences| {
        format!(
            "Setup mode: `{}`\nPreference provenance: `{}`\nInterface language: `{}`\nWiki: `{}` (`{}`)\nPersona: `{}`\n",
            answers.setup_mode,
            preferences.provenance,
            preferences.interface_language,
            if preferences.wiki_enabled {
                "enabled"
            } else {
                "disabled"
            },
            preferences.wiki_language,
            preferences.persona_id,
        )
    });
    let marker = format!(
        "{MARKER_START}\n# Aigent Hive\n\nProject: `{}`\nProfile: `{}`\n{}Primary host: `{}`\nResolved orchestration owner: `{}`\nResolution evidence: `{}`\n\n- Read canonical Hive configuration from `.hive/config/harness.toml`.\n- Before editing anything, read `.hive/directives/00-editing-discipline.md` in full. Apply all four sections as the highest-priority editing discipline within the Hive contract; never compact, summarize, omit, or substitute any part. Its literal `# CLAUDE.md` heading is original text, not Claude-only scope: the directive applies identically on Codex, Claude, and Gemini Antigravity. Higher-priority instructions and Hive security, ownership, credential, and production boundaries still control.\n- Immediately before each new automatic dispatch, run `hive usage enforce --target <project-root> --session-id <exact-current-host-session-id> --process-id <exact-current-host-process-id> [--account-digest <active-account-digest>] --output json`. Do not run `enforce` for ordinary answers, manual work, or other non-dispatch actions. The account digest may be omitted only when the qualified local sensor exposes exactly one unambiguous account. A current halt marker takes priority; exit `3` blocks that automatic dispatch. Exit `0` is only a session-bound preflight and never authorizes dispatch. Automatic dispatch requires a separate `hive run resume --dispatch-intent automatic` result with `data.usage_guard.enforced=true`, `outcome=authorized`, one authorization ID, and exactly one dispatch brief. A confirmed current-session disable bypasses this preflight but does not authorize dispatch. Non-Codex automatic dispatch fails closed until a qualified local sensor exists. Never transfer an override or halt marker to another host, session, or process. Recognize obvious threshold/off/on intent by meaning rather than a finite phrase list. A bare continue, resume, finish, urgency, or active run never authorizes bypass.\n- Load only the directives and knowledge required by the current request.\n- For a simple question, do not load project memory, spawn agents, or edit files.\n- Route explicit prompt authoring or improvement intent to `hive-prompt-refine` in `refine-only` mode unless the same request explicitly asks to execute the result.\n- If an ordinary work prompt materially lacks a goal, scope, constraints, acceptance criteria, or output contract, offer one concise optional refinement suggestion without rewriting the prompt, loading the Skill, or executing the suggestion. Do not interrupt sufficiently clear ordinary work or a simple question.\n- Keep durable role identity in `.hive/team/roles/`; the active host owns sessions and subagents.\n- Keep durable knowledge in Markdown. Treat `.hive/index/*.sqlite*` as disposable.\n- Keep the selected interface language consistent throughout every question and response. In Korean, keep English only for proper nouns, product or package names, commands, code identifiers, paths, schema keys, exact UI labels, and terms without a clear Korean equivalent; replace ordinary English nouns with Korean. In English, write the full passage in English except for exact Korean names, literals, quotations, or text the user explicitly asks to preserve.\n- Write human-readable project documents in concise Korean unless the user explicitly requests another language. Prefer short headings, bullets, tables, checklists, and semantic noun phrases.\n- Do not end authored explanatory Korean prose with declarative or conversational forms. `~다`, `~한다`, `~된다`, `~이다`, `~있다`, `~없다`, `~않는다`, `~했다`, `~됐다`, `~합니다`, `~됩니다`, and `~해요` are non-exhaustive prohibited examples.\n- Do not mechanically change those endings to `~음` or the attached `~ㅁ` form. This includes Korean stems, mixed English-Korean forms, state labels followed by a copula, and possibility clauses. Rewrite the full clause: avoid `Release 계약이 구현됐다.` and `Release 계약이 구현됐음.`; use `Release 계약 구현 완료`. Avoid `API key를 요청하거나 저장하지 않는다.` and `API key를 요청하거나 저장하지 않음.`; use `API key 요청·저장 없음`.\n- Exact bad → good examples (not exhaustive): `Aigent Hive는 provider-neutral 로컬 agent harness다.` → `Aigent Hive: provider-neutral 로컬 agent harness`; `Product version은 0.7.0이다.` → `Product version: 0.7.0`; `Release 계약이 구현됐다.` → `Release 계약 구현 완료`; `API key를 요청하거나 저장하지 않는다.` → `API key 요청·저장 없음`; `이 기능을 사용합니다.` → `기능 사용`; `다음 단계에서 검증해요.` → `다음 단계: 검증`; `검증이 필요합니다.` → `검증 필요`; `업데이트가 완료되었습니다.` → `업데이트 완료`; `Release 계약이 구현됐음.` → `Release 계약 구현 완료`; `API key를 요청하거나 저장하지 않음.` → `API key 요청·저장 없음`.\n- Mechanical nounization examples (not exhaustive): `Status는 INDETERMINATE다.` → `Status: INDETERMINATE`; `문서를 읽음.` → `문서 확인`; `작업이 끝남.` → `작업 완료`; `연결이 닫힘.` → `연결 종료`; `설정 값을 가짐.` → `설정 값 보유`; `정책을 따름.` → `정책 준수`; `compile됨.` → `compile 완료`; `검증할 수 있음.` → `검증 가능`; `검증할 수 없음.` → `검증 불가`.\n- Do not use conversational imperative endings such as standalone `~줘` or attached `~해` in authored explanation. Exact user-prompt or UI-prompt samples require the path, line, reason, and exact line digest exception. Examples: `문서를 보여 줘.` → `문서 확인 요청`; `기능을 사용해.` → `기능 사용 요청`.\n- Apply the same rule to authored callouts and blockquotes. Blockquote syntax is not proof of an exact quotation. Preserve narrative-form text only for an exact external quote, UI prompt, protocol sample, fixture payload, or another byte-sensitive literal with an explicit path, line, reason, and exact line digest.\n- Do not call model-provider APIs or request provider API credentials.\n- Require explicit approval before activating optional Skills.\n- Resolve compatible OMX on Codex and compatible OMC on Claude before host-native capability; never ask the user to select an owner or switch owners mid-run.\n- Treat OMX/OMC cancellation output as auxiliary evidence only; it never substitutes for the bound usage halt marker or durable goal/task state.\n- Treat fallback hooks as optional data-integrity guards only. They require conclusive external capability absence plus exact capability, event, path, command, and digest consent.\n- Never use a fallback hook for prompt classification or rewriting, Skill activation, memory ingestion, subagent orchestration, or continuation. A `Stop` hook always returns a neutral allow result.\n- Preserve user text and third-party marker blocks outside this Hive block.\n{MARKER_END}\n",
        answers.project_name,
        answers.project_kind,
        preference_summary,
        answers.primary_host,
        resolution.resolved_owner,
        resolution.evidence_digest,
    );
    let marker = marker.replacen(
        "- Keep durable knowledge in Markdown. Treat `.hive/index/*.sqlite*` as disposable.\n",
        "- Keep durable knowledge in Markdown. Treat `.hive/index/*.sqlite*` as disposable.\n\
- When this marker reports Wiki enabled, run agent-reviewed task-fact autocapture before the final response for material work. Record the bounded outcome, tool or project, criteria, and originating request summary from current authorized artifacts; never ingest a raw transcript, hook payload, tool output, hidden prompt, or runtime state.\n",
        1,
    );
    if effective_preferences.is_some_and(|preferences| !preferences.usage_guard_enabled) {
        let mut disabled = marker
            .lines()
            .map(|line| {
                if line.starts_with("- Immediately before each new automatic dispatch") {
                    "- Usage guard: disabled by installed preference. Do not run `hive usage enforce` or call a native/CodexBar sensor automatically. Automatic resume must report `data.usage_guard.enforced=false`, `outcome=disabled`, one authorization ID, and exactly one dispatch brief."
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        disabled.push('\n');
        disabled
    } else {
        marker
    }
}

fn merge_shared_marker<T: TargetRead + ?Sized>(
    target: &T,
    relative: &Path,
    marker: &[u8],
) -> Result<Vec<u8>, RenderError> {
    let Some(existing) = read_target_optional(target, relative)? else {
        return Ok(marker.to_vec());
    };
    let start = MARKER_START.as_bytes();
    let end = MARKER_END.as_bytes();
    let starts = find_all(&existing, start);
    let ends = find_all(&existing, end);
    if starts.is_empty() && ends.is_empty() {
        let mut merged = existing;
        if !merged.is_empty() && !merged.ends_with(b"\n") {
            merged.push(b'\n');
        }
        if !merged.is_empty() {
            merged.push(b'\n');
        }
        merged.extend_from_slice(marker);
        return Ok(merged);
    }
    if starts.len() != 1 || ends.len() != 1 || starts[0] >= ends[0] {
        return Err(RenderError::Conflict(
            "shared AGENTS.md contains malformed or nested Hive markers".to_owned(),
        ));
    }
    let end_offset = ends[0] + end.len();
    let mut merged = Vec::with_capacity(existing.len() + marker.len());
    merged.extend_from_slice(&existing[..starts[0]]);
    merged.extend_from_slice(marker.strip_suffix(b"\n").unwrap_or(marker));
    merged.extend_from_slice(&existing[end_offset..]);
    Ok(merged)
}

/// Return a digest of the bytes outside the single Hive-owned marker block.
///
/// A marker-free file is entirely foreign. A missing file is represented by
/// the SHA-256 digest of an empty byte string by callers. Malformed or nested
/// markers are rejected with the same conflict semantics as marker merging.
///
/// # Errors
///
/// Returns a conflict when the byte stream contains unmatched, duplicated, or
/// nested Hive marker delimiters.
pub fn shared_marker_foreign_digest(bytes: &[u8]) -> Result<String, RenderError> {
    let start = MARKER_START.as_bytes();
    let end = MARKER_END.as_bytes();
    let starts = find_all(bytes, start);
    let ends = find_all(bytes, end);
    if starts.is_empty() && ends.is_empty() {
        return Ok(sha256_digest(bytes));
    }
    if starts.len() != 1 || ends.len() != 1 || starts[0] >= ends[0] {
        return Err(RenderError::Conflict(
            "shared AGENTS.md contains malformed or nested Hive markers".to_owned(),
        ));
    }
    let mut foreign = Vec::with_capacity(bytes.len());
    foreign.extend_from_slice(&bytes[..starts[0]]);
    foreign.extend_from_slice(&bytes[ends[0] + end.len()..]);
    Ok(sha256_digest(&foreign))
}

fn render_roles<T: TargetRead + ?Sized>(
    target: &T,
    answers: &SetupAnswers,
    reconfigure_roles: &BTreeSet<String>,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    for seed in &answers.persistent_roles {
        let relative = PathBuf::from(format!(".hive/team/roles/{}.md", seed.role_id));
        let bytes = if let Some(existing) = read_target_optional(target, &relative)? {
            let (mut profile, body) = parse_role(&existing)?;
            if profile.role_id != seed.role_id {
                return Err(RenderError::Conflict(format!(
                    "role identity changed at {}",
                    relative.display()
                )));
            }
            if profile.definition_matches(seed) {
                existing
            } else if reconfigure_roles.contains(&seed.role_id) {
                profile.apply_definition(seed);
                encode_role(&profile, &body)?
            } else {
                return Err(RenderError::Conflict(format!(
                    "role definition changed without --reconfigure-role {}",
                    seed.role_id
                )));
            }
        } else {
            let profile = RoleProfile::from_seed(seed);
            let body = format!(
                "# {}\n\n## Current assignment\n\n_Unassigned._\n\n## Handoff\n\n_No handoff yet._\n",
                seed.display_name
            );
            encode_role(&profile, &body)?
        };
        files.insert(relative, bytes);
    }
    Ok(())
}

fn parse_role(bytes: &[u8]) -> Result<(RoleProfile, String), RenderError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| RenderError::Conflict("role file is not UTF-8".to_owned()))?;
    let (remainder, line_ending) = if let Some(remainder) = text.strip_prefix("---\n") {
        (remainder, "\n")
    } else if let Some(remainder) = text.strip_prefix("---\r\n") {
        (remainder, "\r\n")
    } else {
        return Err(RenderError::Conflict(
            "role frontmatter start is missing".to_owned(),
        ));
    };
    let delimiter = format!("{line_ending}---{line_ending}");
    let (frontmatter, body) = remainder
        .split_once(&delimiter)
        .ok_or_else(|| RenderError::Conflict("role frontmatter end is missing".to_owned()))?;
    let value: JsonValue = serde_json::from_str(frontmatter)
        .map_err(|error| RenderError::Conflict(format!("role profile is invalid: {error}")))?;
    validate_schema_instance(ROLE_SCHEMA, &value, "role profile")
        .map_err(|error| RenderError::Conflict(error.to_string()))?;
    let profile: RoleProfile = serde_json::from_value(value)
        .map_err(|error| RenderError::Conflict(format!("role profile is invalid: {error}")))?;
    Ok((profile, body.to_owned()))
}

fn encode_role(profile: &RoleProfile, body: &str) -> Result<Vec<u8>, RenderError> {
    let value = serde_json::to_value(profile)
        .map_err(|error| RenderError::Internal(format!("cannot encode role: {error}")))?;
    validate_schema_instance(ROLE_SCHEMA, &value, "role profile")?;
    let canonical = serde_json_canonicalizer::to_string(profile)
        .map_err(|error| RenderError::Internal(format!("cannot canonicalize role: {error}")))?;
    Ok(format!("---\n{canonical}\n---\n{body}").into_bytes())
}

fn validate_owned_paths<'a>(
    paths: impl Iterator<Item = &'a PathBuf>,
    desired_projection: &ValidatedProjectionOwnership,
    previous_projection: Option<&ValidatedProjectionOwnership>,
) -> Result<(), RenderError> {
    let manifest = ownership_manifest()?;
    for path in paths {
        let entry = ownership_entry(&manifest, path)?;
        if !matches!(
            entry.ownership.as_str(),
            "hive-managed-config"
                | "hive-managed-license"
                | "hive-generated-config"
                | "user-answer-protected"
                | "user-consent-protected"
                | "canonical-data-protected"
                | "rebuildable-runtime"
                | "ephemeral-backup"
                | "ephemeral-runtime"
                | "shared-marker"
                | "hive-skill-projection"
                | "hive-directive-projection"
        ) {
            return Err(RenderError::Internal(format!(
                "unknown ownership class for {}: {}",
                path.display(),
                entry.ownership
            )));
        }
        if matches!(
            entry.ownership.as_str(),
            "hive-skill-projection" | "hive-directive-projection"
        ) {
            validate_exact_projection_relative(path)
                .map_err(|error| RenderError::Input(error.to_string()))?;
            let is_desired = desired_projection.files.contains_key(path);
            let was_previously_proven =
                previous_projection.is_some_and(|ownership| ownership.files.contains_key(path));
            if !is_desired && !was_previously_proven {
                return Err(RenderError::Safety(format!(
                    "Hive projection path matches the manifest shape but lacks exact Hive ownership proof: {}",
                    path.display()
                )));
            }
        } else {
            validate_project_relative(path)
                .map_err(|error| RenderError::Input(error.to_string()))?;
        }
        if entry.ownership == "shared-marker"
            && (entry.marker_start.as_deref() != Some(MARKER_START)
                || entry.marker_end.as_deref() != Some(MARKER_END))
        {
            return Err(RenderError::Internal(
                "shared marker ownership does not match the compiled marker contract".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_managed_relative(path: &Path) -> Result<(), RenderError> {
    if is_exact_projection_path(path) {
        validate_exact_projection_relative(path)
            .map_err(|error| RenderError::Internal(error.to_string()))
    } else {
        validate_project_relative(path).map_err(|error| RenderError::Internal(error.to_string()))
    }
}

fn ensure_managed_no_symlink_ancestors(target: &Path, relative: &Path) -> Result<(), RenderError> {
    let result = if is_hive_skill_projection_path(relative) {
        ensure_no_symlink_ancestors_for_hive_skill_projection(target, relative)
    } else if is_hive_directive_projection_path(relative) {
        ensure_no_symlink_ancestors_for_hive_directive_projection(target, relative)
    } else {
        ensure_no_symlink_ancestors(target, relative)
    };
    result.map_err(|error| RenderError::Conflict(error.to_string()))
}

fn ownership_manifest() -> Result<OwnershipManifest, RenderError> {
    toml::from_str(OWNERSHIP_MANIFEST)
        .map_err(|error| RenderError::Internal(format!("invalid embedded manifest: {error}")))
}

fn ownership_entry<'a>(
    manifest: &'a OwnershipManifest,
    path: &Path,
) -> Result<&'a OwnershipEntry, RenderError> {
    let relative = path
        .to_str()
        .ok_or_else(|| RenderError::Input("output path is not UTF-8".to_owned()))?;
    manifest
        .paths
        .iter()
        .find(|entry| manifest_pattern_matches(&entry.pattern, relative))
        .ok_or_else(|| {
            RenderError::Safety(format!(
                "renderer output is outside the ownership manifest: {relative}"
            ))
        })
}

/// Return whether a path belongs to a renderer-owned update surface.
///
/// Canonical role/run/knowledge data, rebuildable runtime, backups, and foreign
/// namespaces are deliberately excluded. Durable update recovery uses this
/// check before accepting any journal-directed mutation.
///
/// # Errors
///
/// Returns an error when the compiled ownership manifest is invalid or the
/// candidate is not a normalized project-relative path.
pub fn update_path_is_owned(path: &Path) -> Result<bool, RenderError> {
    if is_hive_skill_projection_path(path) {
        validate_hive_skill_projection_relative(path)
            .map_err(|error| RenderError::Input(error.to_string()))?;
    } else if is_hive_directive_projection_path(path) {
        validate_hive_directive_projection_relative(path)
            .map_err(|error| RenderError::Input(error.to_string()))?;
    } else {
        validate_project_relative(path).map_err(|error| RenderError::Input(error.to_string()))?;
    }
    let manifest = ownership_manifest()?;
    let entry = match ownership_entry(&manifest, path) {
        Ok(entry) => entry,
        Err(RenderError::Safety(_)) => return Ok(false),
        Err(error) => return Err(error),
    };
    Ok(matches!(
        entry.ownership.as_str(),
        "hive-managed-config"
            | "hive-managed-license"
            | "hive-generated-config"
            | "user-answer-protected"
            | "user-consent-protected"
            | "shared-marker"
            | "hive-skill-projection"
            | "hive-directive-projection"
    ))
}

fn manifest_pattern_matches(pattern: &str, path: &str) -> bool {
    if pattern
        .strip_suffix("/**")
        .is_some_and(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
        || pattern == path
    {
        return true;
    }
    let pattern_parts = pattern.split('/').collect::<Vec<_>>();
    let path_parts = path.split('/').collect::<Vec<_>>();
    pattern_parts.len() == path_parts.len()
        && pattern_parts
            .iter()
            .zip(path_parts)
            .all(|(expected, actual)| *expected == "*" || *expected == actual)
}

fn differing_paths<T: TargetRead + ?Sized>(
    target: &T,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
) -> Result<Vec<String>, RenderError> {
    let mut changed = Vec::new();
    for (relative, bytes) in files {
        match read_target_optional(target, relative)? {
            Some(current) if current == *bytes => {}
            Some(_) | None => changed.push(path_string(relative)?),
        }
    }
    for relative in deletions {
        if read_target_optional(target, relative)?.is_some() {
            changed.push(path_string(relative)?);
        }
    }
    changed.sort();
    changed.dedup();
    Ok(changed)
}

fn setup_changes<T: TargetRead + ?Sized>(
    target: &T,
    changed_paths: &[String],
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<Vec<SetupChange>, RenderError> {
    changed_paths
        .iter()
        .map(|path| {
            let relative = Path::new(path);
            let before = read_target_optional(target, relative)?;
            let before_digest = before.as_ref().map(|bytes| sha256_digest(bytes));
            let after_digest = files.get(relative).map(|bytes| sha256_digest(bytes));
            let (foreign_before_digest, foreign_after_digest) = if path == "AGENTS.md" {
                let empty_digest = sha256_digest(&[]);
                (
                    Some(match before.as_deref() {
                        Some(bytes) => shared_marker_foreign_digest(bytes)?,
                        None => empty_digest.clone(),
                    }),
                    Some(match files.get(relative) {
                        Some(bytes) => shared_marker_foreign_digest(bytes)?,
                        None => empty_digest,
                    }),
                )
            } else {
                (None, None)
            };
            Ok(SetupChange {
                path: path.clone(),
                before_digest,
                after_digest,
                foreign_before_digest,
                foreign_after_digest,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
struct ValidatedProjectionOwnership {
    files: BTreeMap<PathBuf, Vec<u8>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ProjectionExpectedBefore {
    Absent,
    Exact(Vec<u8>),
}

#[derive(Clone, Copy)]
enum ExactProjectionMutation<'a> {
    Replace(&'a [u8]),
    Delete,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ProjectionCleanupFault {
    Replacement,
    Deletion,
}

#[derive(Debug)]
enum MutationOutcome {
    Unchanged,
    Applied,
    AppliedWithCleanupError(RenderError),
}

struct ProjectionTransition {
    desired: ValidatedProjectionOwnership,
    previous: Option<ValidatedProjectionOwnership>,
    expected_before: BTreeMap<PathBuf, ProjectionExpectedBefore>,
    deletions: BTreeSet<PathBuf>,
}

fn is_exact_projection_path(path: &Path) -> bool {
    is_hive_skill_projection_path(path) || is_hive_directive_projection_path(path)
}

fn validate_exact_projection_relative(path: &Path) -> Result<(), TargetGuardError> {
    if is_hive_skill_projection_path(path) {
        validate_hive_skill_projection_relative(path)
    } else {
        validate_hive_directive_projection_relative(path)
    }
}

fn prepare_projection_transition<T: TargetRead + ?Sized>(
    target: &T,
    desired_files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
) -> Result<ProjectionTransition, RenderError> {
    prepare_projection_transition_with_source(target, desired_files, answers, None)
}

fn prepare_operation_projection_transition<T: TargetRead + ?Sized>(
    target: &T,
    desired_files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    release_source_version: Option<&str>,
) -> Result<ProjectionTransition, RenderError> {
    match release_source_version {
        Some(source_version) => {
            prepare_release_projection_transition(target, desired_files, answers, source_version)
        }
        None => prepare_projection_transition(target, desired_files, answers),
    }
}

fn prepare_release_projection_transition<T: TargetRead + ?Sized>(
    target: &T,
    desired_files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    authenticated_source_version: &str,
) -> Result<ProjectionTransition, RenderError> {
    prepare_projection_transition_with_source(
        target,
        desired_files,
        answers,
        Some(authenticated_source_version),
    )
}

fn prepare_projection_transition_with_source<T: TargetRead + ?Sized>(
    target: &T,
    desired_files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    release_source_version: Option<&str>,
) -> Result<ProjectionTransition, RenderError> {
    let desired = validate_desired_projection_ownership(desired_files, answers)?;
    let active_path = Path::new(".hive/config/active-skills.yml");
    if read_target_optional(target, active_path)?.is_none() {
        if let Some(source_version) = release_source_version {
            let historical = historical_builtin_skills(source_version).map_err(|error| {
                RenderError::Verification(format!(
                    "authenticated release-update source is not covered by embedded history: {error}"
                ))
            })?;
            if !historical.is_empty() {
                return Err(RenderError::Conflict(
                    "existing Hive projection ownership cannot be verified: historical active Skill projection ledger is missing"
                        .to_owned(),
                ));
            }
        }
        let mut expected_before = BTreeMap::new();
        for path in desired.files.keys() {
            if read_target_optional(target, path)?.is_some() {
                return Err(RenderError::Conflict(format!(
                    "Hive projection path is occupied without exact ownership proof: {}",
                    path.display()
                )));
            }
            expected_before.insert(path.clone(), ProjectionExpectedBefore::Absent);
        }
        return Ok(ProjectionTransition {
            desired,
            previous: None,
            expected_before,
            deletions: BTreeSet::new(),
        });
    }

    let installed_answers = read_installed_answers(target).map_err(|error| {
        RenderError::Conflict(format!(
            "existing Hive projection ownership cannot be verified: {error}"
        ))
    })?;
    let previous = match release_source_version {
        Some(source_version) => {
            validate_release_projection_ownership(target, &installed_answers, source_version)
        }
        None => validate_projection_ownership(target, &installed_answers),
    }
    .map_err(|error| {
        RenderError::Conflict(format!(
            "existing Hive projection ownership cannot be verified: {error}"
        ))
    })?;

    let mut expected_before = BTreeMap::new();
    for path in desired.files.keys() {
        if let Some(bytes) = previous.files.get(path) {
            expected_before.insert(path.clone(), ProjectionExpectedBefore::Exact(bytes.clone()));
        } else {
            if read_target_optional(target, path)?.is_some() {
                return Err(RenderError::Conflict(format!(
                    "new Hive projection collides with a foreign file: {}",
                    path.display()
                )));
            }
            expected_before.insert(path.clone(), ProjectionExpectedBefore::Absent);
        }
    }

    let deletions = previous
        .files
        .keys()
        .filter(|path| !desired.files.contains_key(*path))
        .cloned()
        .collect::<BTreeSet<_>>();
    for path in &deletions {
        let bytes = previous
            .files
            .get(path)
            .expect("stale projection path comes from validated previous ownership");
        expected_before.insert(path.clone(), ProjectionExpectedBefore::Exact(bytes.clone()));
    }

    Ok(ProjectionTransition {
        desired,
        previous: Some(previous),
        expected_before,
        deletions,
    })
}

fn validate_desired_projection_ownership(
    desired_files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
) -> Result<ValidatedProjectionOwnership, RenderError> {
    let active_path = Path::new(".hive/config/active-skills.yml");
    let active_bytes = desired_files.get(active_path).cloned().ok_or_else(|| {
        RenderError::Internal("compiled tree omitted its active Skill projection ledger".to_owned())
    })?;
    let harness_bytes = desired_files
        .get(Path::new(".hive/config/harness.toml"))
        .ok_or_else(|| RenderError::Internal("compiled tree omitted harness config".to_owned()))?;
    let harness: InstalledHarness =
        toml::from_str(std::str::from_utf8(harness_bytes).map_err(|_| {
            RenderError::Internal("compiled harness config is not UTF-8".to_owned())
        })?)
        .map_err(|error| RenderError::Internal(format!("compiled harness is invalid: {error}")))?;
    let effective_preferences = effective_preferences_from_harness(&harness)?;
    reproduce_projection_ownership(
        &active_bytes,
        answers,
        env!("CARGO_PKG_VERSION"),
        true,
        effective_preferences
            .as_ref()
            .map(|preferences| preferences.selected_project_skills.as_slice()),
        |relative, label| {
            desired_files.get(relative).cloned().ok_or_else(|| {
                RenderError::Internal(format!(
                    "compiled tree omitted required {label}: {}",
                    relative.display()
                ))
            })
        },
    )
}

fn validate_projection_ownership<T: TargetRead + ?Sized>(
    target: &T,
    answers: &SetupAnswers,
) -> Result<ValidatedProjectionOwnership, RenderError> {
    let active_path = Path::new(".hive/config/active-skills.yml");
    let active_bytes = read_target_required(target, active_path, "active Skill projection ledger")?;
    let harness = read_installed_harness(target)?;
    let effective_preferences = effective_preferences_from_harness(&harness)?;
    reproduce_projection_ownership(
        &active_bytes,
        answers,
        &harness.harness_version,
        true,
        effective_preferences
            .as_ref()
            .map(|preferences| preferences.selected_project_skills.as_slice()),
        |relative, label| read_target_required(target, relative, label),
    )
}

fn validate_release_projection_ownership<T: TargetRead + ?Sized>(
    target: &T,
    answers: &SetupAnswers,
    authenticated_source_version: &str,
) -> Result<ValidatedProjectionOwnership, RenderError> {
    let harness = read_installed_harness(target)?;
    if harness.harness_version != authenticated_source_version {
        return Err(RenderError::Verification(
            "authenticated release-update source version does not match the installed harness"
                .to_owned(),
        ));
    }
    let authenticate_directives = authenticated_source_version == env!("CARGO_PKG_VERSION");
    if !authenticate_directives {
        historical_builtin_skills(authenticated_source_version).map_err(|error| {
            RenderError::Verification(format!(
                "authenticated release-update source is not covered by embedded history: {error}"
            ))
        })?;
    }
    let active_path = Path::new(".hive/config/active-skills.yml");
    let active_bytes = read_target_required(target, active_path, "active Skill projection ledger")?;
    let effective_preferences = effective_preferences_from_harness(&harness)?;
    reproduce_projection_ownership(
        &active_bytes,
        answers,
        authenticated_source_version,
        authenticate_directives,
        effective_preferences
            .as_ref()
            .map(|preferences| preferences.selected_project_skills.as_slice()),
        |relative, label| read_target_required(target, relative, label),
    )
}

fn reproduce_projection_ownership(
    active_bytes: &[u8],
    answers: &SetupAnswers,
    source_version: &str,
    authenticate_directives: bool,
    selected_project_skills: Option<&[String]>,
    mut read_projected: impl FnMut(&Path, &str) -> Result<Vec<u8>, RenderError>,
) -> Result<ValidatedProjectionOwnership, RenderError> {
    let active: ActiveSkills = serde_yaml::from_slice(active_bytes).map_err(|error| {
        RenderError::Verification(format!("invalid active Skill projection ledger: {error}"))
    })?;
    if active.schema_version != 1 {
        return Err(RenderError::Verification(
            "active Skill projection ledger schema_version must be 1".to_owned(),
        ));
    }

    let host = projection_host(&answers.primary_host).map_err(as_verification)?;
    let mut optional_sources = Vec::new();
    let mut optional_names = BTreeSet::new();
    for skill in active
        .skills
        .iter()
        .filter(|skill| skill.source_type == SkillSourceType::ApprovedOptional)
    {
        if !optional_names.insert(skill.name.as_str()) {
            return Err(RenderError::Verification(format!(
                "active Skill projection ledger has duplicate name: {}",
                skill.name
            )));
        }
        let approvals = answers
            .approved_optional_skills
            .iter()
            .filter(|approval| approval.name == skill.name)
            .collect::<Vec<_>>();
        if approvals.len() != 1 {
            return Err(RenderError::Verification(format!(
                "active optional Skill does not have one exact installed approval: {}",
                skill.name
            )));
        }
        let relative = projected_skill_path(host, &skill.name)?;
        let skill_md = read_projected(&relative, "projected optional Skill")?;
        optional_sources.push(optional_source_from_approval(approvals[0], skill_md)?);
    }

    let current_projection = selected_project_skills
        .map_or_else(
            || compile_projection(host, &optional_sources),
            |selected| compile_project_projection(host, selected, &optional_sources),
        )
        .map_err(|error| {
            RenderError::Verification(format!(
                "active Skill projection cannot be reproduced: {error}"
            ))
        })?;
    let portable_projection = if host == ProjectionHost::Claude {
        selected_project_skills
            .map_or_else(
                || compile_projection(ProjectionHost::Codex, &optional_sources),
                |selected| {
                    compile_project_projection(ProjectionHost::Codex, selected, &optional_sources)
                },
            )
            .map_err(|error| {
                RenderError::Verification(format!(
                    "portable Skill projection cannot be reproduced: {error}"
                ))
            })?
    } else {
        current_projection.clone()
    };
    let expected_active = expected_active_skills(source_version, &current_projection)?;
    if expected_active != active {
        return Err(RenderError::Verification(
            "active Skill projection ledger does not match installed approvals and the authenticated release history"
                .to_owned(),
        ));
    }
    let expected_active_bytes = serde_yaml::to_string(&expected_active)
        .map_err(|error| {
            RenderError::Internal(format!(
                "active Skill projection ledger serialization failed: {error}"
            ))
        })?
        .into_bytes();
    if active_bytes != expected_active_bytes {
        return Err(RenderError::Verification(
            "active Skill projection ledger bytes are not canonical".to_owned(),
        ));
    }

    let mut files = authenticate_projected_skill_files(
        host,
        source_version,
        &expected_active,
        &current_projection,
        &portable_projection,
        &mut read_projected,
    )?;
    if authenticate_directives {
        authenticate_projected_directive_files(source_version, &mut read_projected, &mut files)?;
    }
    Ok(ValidatedProjectionOwnership { files })
}

fn authenticate_projected_directive_files(
    source_version: &str,
    read_projected: &mut impl FnMut(&Path, &str) -> Result<Vec<u8>, RenderError>,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    if source_version != env!("CARGO_PKG_VERSION") {
        return Err(RenderError::Verification(format!(
            "historical directive projection cannot be authenticated for release {source_version}"
        )));
    }
    for (relative, expected) in [
        (
            Path::new(".agents/directives/00-project-harness.md"),
            include_bytes!("../../../harness/directives/00-project-harness.md").as_slice(),
        ),
        (
            Path::new(".agents/directives/01-project-knowledge.md"),
            include_bytes!("../../../harness/directives/01-project-knowledge.md").as_slice(),
        ),
        (
            Path::new(".agents/directives/02-project-upgrade.md"),
            include_bytes!("../../../harness/directives/02-project-upgrade.md").as_slice(),
        ),
    ] {
        validate_hive_directive_projection_relative(relative)
            .map_err(|error| RenderError::Verification(error.to_string()))?;
        let installed = read_projected(relative, "projected directive")?;
        if installed != expected {
            return Err(RenderError::Verification(format!(
                "projected directive bytes changed: {}",
                relative.display()
            )));
        }
        files.insert(relative.to_path_buf(), installed);
    }
    Ok(())
}

fn expected_active_skills(
    source_version: &str,
    current_projection: &Projection,
) -> Result<ActiveSkills, RenderError> {
    let mut skills = if source_version == env!("CARGO_PKG_VERSION") {
        current_projection
            .active_skills
            .skills
            .iter()
            .filter(|skill| skill.source_type == SkillSourceType::BuiltIn)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        historical_builtin_skills(source_version).map_err(|error| {
            RenderError::Verification(format!(
                "historical active Skill projection cannot be authenticated: {error}"
            ))
        })?
    };
    skills.extend(
        current_projection
            .active_skills
            .skills
            .iter()
            .filter(|skill| skill.source_type == SkillSourceType::ApprovedOptional)
            .cloned(),
    );
    skills.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(ActiveSkills {
        schema_version: 1,
        skills,
    })
}

fn authenticate_projected_skill_files(
    host: ProjectionHost,
    source_version: &str,
    expected_active: &ActiveSkills,
    current_projection: &Projection,
    portable_projection: &Projection,
    read_projected: &mut impl FnMut(&Path, &str) -> Result<Vec<u8>, RenderError>,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, RenderError> {
    let mut files = BTreeMap::new();
    for skill in &expected_active.skills {
        let primary_relative = projected_skill_path(host, &skill.name)?;
        let current_expected = current_projection
            .files
            .get(primary_relative.to_str().ok_or_else(|| {
                RenderError::Verification("projected Skill path is not portable UTF-8".to_owned())
            })?);
        for relative in projected_skill_paths(host, &skill.name)? {
            validate_hive_skill_projection_relative(&relative)
                .map_err(|error| RenderError::Verification(error.to_string()))?;
            let installed = read_projected(&relative, "projected Skill")?;
            let authenticated = match skill.source_type {
                SkillSourceType::BuiltIn if source_version != env!("CARGO_PKG_VERSION") => {
                    sha256_digest(&installed) == skill.content_digest
                }
                SkillSourceType::BuiltIn | SkillSourceType::ApprovedOptional => {
                    current_expected.is_some_and(|expected| *expected == installed)
                }
            };
            if !authenticated {
                return Err(RenderError::Verification(format!(
                    "projected Skill bytes changed: {}",
                    relative.display()
                )));
            }
            files.insert(relative, installed);
        }
    }
    if source_version == env!("CARGO_PKG_VERSION") {
        for (relative, expected) in &portable_projection.files {
            let relative = Path::new(relative);
            if !relative.ends_with("agents/openai.yaml") {
                continue;
            }
            validate_hive_skill_projection_relative(relative)
                .map_err(|error| RenderError::Verification(error.to_string()))?;
            let installed = read_projected(relative, "projected Skill metadata")?;
            if installed != *expected {
                return Err(RenderError::Verification(format!(
                    "projected Skill metadata bytes changed: {}",
                    relative.display()
                )));
            }
            files.insert(relative.to_path_buf(), installed);
        }
    }
    Ok(files)
}

fn projected_skill_path(host: ProjectionHost, name: &str) -> Result<PathBuf, RenderError> {
    let relative = PathBuf::from(format!("{}/{name}/SKILL.md", host.skill_root()));
    validate_hive_skill_projection_relative(&relative)
        .map_err(|error| RenderError::Verification(error.to_string()))?;
    Ok(relative)
}

fn projected_skill_paths(host: ProjectionHost, name: &str) -> Result<Vec<PathBuf>, RenderError> {
    let mut paths = vec![PathBuf::from(format!(".agents/skills/{name}/SKILL.md"))];
    if host == ProjectionHost::Claude {
        paths.push(PathBuf::from(format!(".claude/skills/{name}/SKILL.md")));
    }
    for path in &paths {
        validate_hive_skill_projection_relative(path)
            .map_err(|error| RenderError::Verification(error.to_string()))?;
    }
    Ok(paths)
}

fn optional_source_from_approval(
    approval: &SkillApproval,
    skill_md: Vec<u8>,
) -> Result<OptionalSkillSource, RenderError> {
    let value = serde_json::to_value(approval).map_err(|error| {
        RenderError::Internal(format!("cannot encode optional Skill approval: {error}"))
    })?;
    let consent: OptionalSkillConsent = serde_json::from_value(value).map_err(|error| {
        RenderError::Verification(format!(
            "invalid projected optional Skill approval: {error}"
        ))
    })?;
    Ok(OptionalSkillSource {
        consent,
        source_locator: approval.source.clone(),
        skill_md,
    })
}

fn stale_hook_deletions<T: TargetRead + ?Sized>(
    target: &T,
    desired: &[HookApproval],
    resolution: &CapabilityResolution,
) -> Result<BTreeSet<PathBuf>, RenderError> {
    let ledger_path = PathBuf::from(".hive/config/approved-hooks.yml");
    let Some(bytes) = read_target_optional(target, &ledger_path)? else {
        return Ok(BTreeSet::new());
    };
    let value: JsonValue = serde_yaml::from_slice(&bytes).map_err(|error| {
        RenderError::Conflict(format!(
            "existing fallback hook ledger is malformed and cannot be safely replaced: {error}"
        ))
    })?;
    validate_schema_instance(HOOK_SCHEMA, &value, "existing fallback hook ledger")
        .map_err(|error| RenderError::Conflict(error.to_string()))?;
    let ledger: HookLedger = serde_json::from_value(value).map_err(|error| {
        RenderError::Conflict(format!(
            "existing fallback hook ledger is malformed and cannot be safely replaced: {error}"
        ))
    })?;
    let desired_paths: BTreeSet<_> = desired
        .iter()
        .map(|hook| PathBuf::from(&hook.path))
        .collect();
    let stale_hooks: Vec<_> = ledger
        .hooks
        .iter()
        .filter(|hook| !desired_paths.contains(Path::new(&hook.path)))
        .collect();
    let remove_ledger = desired.is_empty() || resolution.detection != "absent";
    if stale_hooks.is_empty() && !remove_ledger {
        return Ok(BTreeSet::new());
    }

    validate_revoked_hook_ownership(target, &bytes, &ledger, &stale_hooks)?;

    let mut deletions = BTreeSet::new();
    for hook in stale_hooks {
        let relative = PathBuf::from(&hook.path);
        validate_project_relative(&relative)
            .map_err(|error| RenderError::Conflict(error.to_string()))?;
        deletions.insert(relative);
    }
    if remove_ledger {
        deletions.insert(ledger_path);
    }
    Ok(deletions)
}

fn validate_revoked_hook_ownership<T: TargetRead + ?Sized>(
    target: &T,
    ledger_bytes: &[u8],
    ledger: &HookLedger,
    stale_hooks: &[&HookApproval],
) -> Result<(), RenderError> {
    let conflict = |error: RenderError| {
        RenderError::Conflict(format!(
            "existing fallback hook ownership cannot be verified for revocation: {error}"
        ))
    };
    let installed_answers = read_installed_answers(target).map_err(&conflict)?;
    let installed_resolution = read_installed_resolution(target).map_err(&conflict)?;
    validate_resolution(&installed_answers, &installed_resolution).map_err(&conflict)?;
    if ledger.schema_version != 1
        || ledger.detection != "absent"
        || ledger.resolution_evidence_digest != installed_resolution.evidence_digest
        || ledger.hooks != installed_answers.approved_fallback_hooks
    {
        return Err(RenderError::Conflict(
            "existing fallback hook ledger does not match the installed approval contract"
                .to_owned(),
        ));
    }
    validate_hook_approvals(&ledger.hooks, &installed_resolution).map_err(&conflict)?;
    if ledger_bytes != render_hook_ledger(&ledger.hooks, &installed_resolution) {
        return Err(RenderError::Conflict(
            "existing fallback hook ledger bytes do not match the installed approval contract"
                .to_owned(),
        ));
    }
    for hook in stale_hooks {
        let relative = Path::new(&hook.path);
        let projected = read_target_required(target, relative, "fallback hook descriptor")
            .map_err(&conflict)?;
        let expected = hook_descriptor_bytes(hook).map_err(&conflict)?;
        if projected != expected || sha256_digest(&projected) != hook.content_digest {
            return Err(RenderError::Conflict(format!(
                "fallback hook descriptor ownership cannot be verified for revocation: {}",
                hook.capability
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn activate_staged(
    target: &Path,
    target_dir: &Dir,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    verify_ambient_target: bool,
    post_apply: Option<&dyn Fn() -> Result<(), RenderError>>,
) -> Result<(), RenderError> {
    activate_staged_impl_with_pin(
        target,
        target_dir,
        files,
        deletions,
        answers,
        resolution,
        projection_expected_before,
        activation_fault_from_environment(),
        None,
        None,
        post_apply,
        None,
        verify_ambient_target,
    )
}

#[derive(Debug, Clone, Copy)]
struct ActivationFault {
    #[cfg(debug_assertions)]
    fail_after_operations: usize,
    #[cfg(debug_assertions)]
    fail_rollback: bool,
    projection_cleanup: Option<ProjectionCleanupFault>,
}

#[derive(Debug, Clone, Copy)]
struct ReplacePolicy {
    destination_requires_backup: bool,
    fail_after_backup: bool,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn activate_staged_impl_with_pin(
    target: &Path,
    target_dir: &Dir,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    fault: Option<ActivationFault>,
    after_target_open: Option<&dyn Fn()>,
    before_projection_claim: Option<&dyn Fn(&Path)>,
    post_apply: Option<&dyn Fn() -> Result<(), RenderError>>,
    before_rollback: Option<&dyn Fn()>,
    verify_ambient_target: bool,
) -> Result<(), RenderError> {
    stage_and_validate(
        verify_ambient_target.then_some(target),
        files,
        answers,
        resolution,
        staging_corruption_from_environment(),
    )?;

    if verify_ambient_target {
        verify_target_capability_current(target, target_dir)?;
    }
    if let Some(barrier) = after_target_open {
        barrier();
    }

    let mut previous = BTreeMap::<PathBuf, Option<Vec<u8>>>::new();
    let mut created_directories = Vec::new();
    let operation_paths: BTreeSet<_> = files.keys().chain(deletions.iter()).cloned().collect();
    for relative in &operation_paths {
        previous.insert(
            relative.clone(),
            read_capability_optional(target_dir, relative)?,
        );
    }
    for relative in files.keys() {
        if let Err(error) = capability_parent(target_dir, relative, true, &mut created_directories)
        {
            return activation_failed(
                &error,
                target_dir,
                &previous,
                files,
                &[],
                &created_directories,
                projection_expected_before,
                before_rollback,
                fault,
            );
        }
    }

    let mut applied = Vec::new();
    let mut operation_count = 0;
    for (relative, bytes) in files {
        if should_inject_activation_failure(fault, operation_count) {
            return activation_failed(
                &RenderError::Internal("injected activation I/O failure".to_owned()),
                target_dir,
                &previous,
                files,
                &applied,
                &created_directories,
                projection_expected_before,
                before_rollback,
                fault,
            );
        }
        let result = match projection_expected_before.get(relative) {
            Some(ProjectionExpectedBefore::Absent) => verify_projection_expected_before(
                target_dir,
                relative,
                &ProjectionExpectedBefore::Absent,
            )
            .and_then(|()| {
                create_capability_file_exclusive(target_dir, relative, bytes)
                    .map_err(|error| RenderError::Conflict(error.to_string()))
            })
            .map(|()| MutationOutcome::Applied),
            Some(ProjectionExpectedBefore::Exact(expected)) if expected == bytes => {
                verify_projection_expected_before(
                    target_dir,
                    relative,
                    &ProjectionExpectedBefore::Exact(expected.clone()),
                )
                .map(|()| MutationOutcome::Unchanged)
            }
            Some(ProjectionExpectedBefore::Exact(expected)) => mutate_exact_projection_claimed(
                target_dir,
                relative,
                expected,
                ExactProjectionMutation::Replace(bytes),
                before_projection_claim,
                fault.and_then(|value| value.projection_cleanup),
            ),
            None => replace_capability_file(target_dir, relative, bytes)
                .map_err(|error| RenderError::Internal(error.to_string()))
                .map(|()| MutationOutcome::Applied),
        };
        match result {
            Ok(MutationOutcome::Applied) => applied.push(relative.clone()),
            Ok(MutationOutcome::Unchanged) => {}
            Ok(MutationOutcome::AppliedWithCleanupError(error)) => {
                applied.push(relative.clone());
                return activation_failed(
                    &error,
                    target_dir,
                    &previous,
                    files,
                    &applied,
                    &created_directories,
                    projection_expected_before,
                    before_rollback,
                    fault,
                );
            }
            Err(error) => {
                return activation_failed(
                    &error,
                    target_dir,
                    &previous,
                    files,
                    &applied,
                    &created_directories,
                    projection_expected_before,
                    before_rollback,
                    fault,
                );
            }
        }
        operation_count += 1;
    }
    for relative in deletions {
        if should_inject_activation_failure(fault, operation_count) {
            return activation_failed(
                &RenderError::Internal("injected activation I/O failure".to_owned()),
                target_dir,
                &previous,
                files,
                &applied,
                &created_directories,
                projection_expected_before,
                before_rollback,
                fault,
            );
        }
        if let Some(ProjectionExpectedBefore::Exact(expected)) =
            projection_expected_before.get(relative)
        {
            match mutate_exact_projection_claimed(
                target_dir,
                relative,
                expected,
                ExactProjectionMutation::Delete,
                before_projection_claim,
                fault.and_then(|value| value.projection_cleanup),
            ) {
                Ok(MutationOutcome::Applied) => applied.push(relative.clone()),
                Ok(MutationOutcome::AppliedWithCleanupError(error)) => {
                    applied.push(relative.clone());
                    return activation_failed(
                        &error,
                        target_dir,
                        &previous,
                        files,
                        &applied,
                        &created_directories,
                        projection_expected_before,
                        before_rollback,
                        fault,
                    );
                }
                Ok(MutationOutcome::Unchanged) => {
                    return activation_failed(
                        &RenderError::Internal(
                            "projection deletion unexpectedly reported no live change".to_owned(),
                        ),
                        target_dir,
                        &previous,
                        files,
                        &applied,
                        &created_directories,
                        projection_expected_before,
                        before_rollback,
                        fault,
                    );
                }
                Err(error) => {
                    return activation_failed(
                        &error,
                        target_dir,
                        &previous,
                        files,
                        &applied,
                        &created_directories,
                        projection_expected_before,
                        before_rollback,
                        fault,
                    );
                }
            }
        } else if previous.get(relative).is_some_and(Option::is_some) {
            if let Err(error) = remove_capability_file(target_dir, relative) {
                return activation_failed(
                    &RenderError::Internal(format!("activation deletion failed: {error}")),
                    target_dir,
                    &previous,
                    files,
                    &applied,
                    &created_directories,
                    projection_expected_before,
                    before_rollback,
                    fault,
                );
            }
            applied.push(relative.clone());
        }
        operation_count += 1;
    }
    let validation = validate_capability_activation(target_dir, files, deletions, answers)
        .and_then(|()| {
            if verify_ambient_target {
                verify_target_capability_current(target, target_dir)
            } else {
                Ok(())
            }
        })
        .and_then(|()| match post_apply {
            Some(commit) => commit(),
            None => Ok(()),
        });
    if let Err(error) = validation {
        return activation_failed(
            &error,
            target_dir,
            &previous,
            files,
            &applied,
            &created_directories,
            projection_expected_before,
            before_rollback,
            fault,
        );
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn activate_staged_impl(
    target: &Path,
    target_dir: &Dir,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    fault: Option<ActivationFault>,
    after_target_open: Option<&dyn Fn()>,
    before_projection_claim: Option<&dyn Fn(&Path)>,
    before_rollback: Option<&dyn Fn()>,
) -> Result<(), RenderError> {
    activate_staged_impl_with_pin(
        target,
        target_dir,
        files,
        deletions,
        answers,
        resolution,
        projection_expected_before,
        fault,
        after_target_open,
        before_projection_claim,
        None,
        before_rollback,
        true,
    )
}

fn stage_and_validate(
    target: Option<&Path>,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    corrupt_after_render: bool,
) -> Result<(), RenderError> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".aigent-hive-stage-");
    let staging = match target {
        Some(target) => {
            let parent = target
                .parent()
                .ok_or_else(|| RenderError::Input("target has no parent directory".to_owned()))?;
            builder.tempdir_in(parent).map_err(io_internal)?
        }
        None => builder.tempdir().map_err(io_internal)?,
    };
    let staging_root = staging.path().canonicalize().map_err(io_internal)?;
    for (relative, bytes) in files {
        let staged = staging_root.join(relative);
        if let Some(directory) = staged.parent() {
            fs::create_dir_all(directory).map_err(io_internal)?;
        }
        fs::write(&staged, bytes).map_err(io_internal)?;
    }
    if corrupt_after_render {
        fs::write(
            staging_root.join(".hive/config/harness.toml"),
            b"injected invalid staged bytes\n",
        )
        .map_err(io_internal)?;
    }
    validate_staged(&staging_root, files, answers, resolution)
}

fn validate_staged(
    staging: &Path,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    for (relative, expected) in files {
        let staged = staging.join(relative);
        if !staged.is_file() {
            return Err(RenderError::Verification(format!(
                "staged output is missing: {}",
                relative.display()
            )));
        }
        if fs::read(&staged).map_err(io_internal)? != *expected {
            return Err(RenderError::Verification(format!(
                "staged output bytes changed after render: {}",
                relative.display()
            )));
        }
    }
    let harness =
        fs::read_to_string(staging.join(".hive/config/harness.toml")).map_err(io_internal)?;
    let _: toml::Value = toml::from_str(&harness)
        .map_err(|error| RenderError::Verification(format!("invalid harness TOML: {error}")))?;
    validate_installed_against(staging, answers, resolution, files)
}

fn open_target_capability(target: &Path) -> Result<Dir, RenderError> {
    let parent = target
        .parent()
        .ok_or_else(|| RenderError::Input("activation target has no parent".to_owned()))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = target
        .file_name()
        .ok_or_else(|| RenderError::Input("activation target has no directory name".to_owned()))?;
    let parent_dir = Dir::open_ambient_dir(parent, ambient_authority()).map_err(io_internal)?;
    parent_dir.open_dir_nofollow(name).map_err(|error| {
        RenderError::Conflict(format!(
            "activation target cannot be opened as a no-follow directory {}: {error}",
            target.display()
        ))
    })
}

fn verify_target_capability_current(target: &Path, pinned: &Dir) -> Result<(), RenderError> {
    let current = open_target_capability(target)?;
    let pinned_metadata = pinned.dir_metadata().map_err(io_internal)?;
    let current_metadata = current.dir_metadata().map_err(io_internal)?;
    if CapFsMetadataExt::dev(&pinned_metadata) != CapFsMetadataExt::dev(&current_metadata)
        || CapFsMetadataExt::ino(&pinned_metadata) != CapFsMetadataExt::ino(&current_metadata)
    {
        return Err(RenderError::Conflict(format!(
            "activation target no longer resolves to the pinned directory: {}",
            target.display()
        )));
    }
    Ok(())
}

fn capability_parent(
    target: &Dir,
    relative: &Path,
    create_missing: bool,
    created_directories: &mut Vec<PathBuf>,
) -> Result<Option<(Dir, OsString)>, RenderError> {
    validate_managed_relative(relative)?;
    capability_parent_validated(target, relative, create_missing, created_directories)
}

fn capability_parent_validated(
    target: &Dir,
    relative: &Path,
    create_missing: bool,
    created_directories: &mut Vec<PathBuf>,
) -> Result<Option<(Dir, OsString)>, RenderError> {
    let file_name = relative
        .file_name()
        .ok_or_else(|| RenderError::Internal("managed file has no name".to_owned()))?
        .to_os_string();
    let mut current = target.try_clone().map_err(io_internal)?;
    let mut current_relative = PathBuf::new();
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let component = component.as_os_str();
            current_relative.push(component);
            match current.symlink_metadata(component) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(RenderError::Conflict(format!(
                        "managed path ancestor is not a no-follow directory: {}",
                        current_relative.display()
                    )));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound && !create_missing => {
                    return Ok(None);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    current.create_dir(component).map_err(|error| {
                        RenderError::Internal(format!(
                            "activation directory creation failed at {}: {error}",
                            current_relative.display()
                        ))
                    })?;
                    created_directories.push(current_relative.clone());
                }
                Err(error) => {
                    return Err(RenderError::Conflict(format!(
                        "managed path ancestor cannot be inspected at {}: {error}",
                        current_relative.display()
                    )));
                }
            }
            current = current.open_dir_nofollow(component).map_err(|error| {
                RenderError::Conflict(format!(
                    "managed path ancestor cannot be opened no-follow at {}: {error}",
                    current_relative.display()
                ))
            })?;
        }
    }
    Ok(Some((current, file_name)))
}

fn open_capability_file_nofollow(parent: &Dir, file_name: &OsStr) -> io::Result<cap_std::fs::File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    parent.open_with(file_name, &options)
}

fn read_capability_optional(target: &Dir, relative: &Path) -> Result<Option<Vec<u8>>, RenderError> {
    let mut created = Vec::new();
    let Some((parent, file_name)) = capability_parent(target, relative, false, &mut created)?
    else {
        return Ok(None);
    };
    match parent.symlink_metadata(&file_name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_internal(error)),
        Ok(metadata) if metadata.is_file() => {
            let mut file =
                open_capability_file_nofollow(&parent, &file_name).map_err(io_internal)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(io_internal)?;
            Ok(Some(bytes))
        }
        Ok(_) => Err(RenderError::Conflict(format!(
            "managed file path is occupied by a non-file: {}",
            relative.display()
        ))),
    }
}

fn verify_projection_expected_before(
    target: &Dir,
    relative: &Path,
    expected: &ProjectionExpectedBefore,
) -> Result<(), RenderError> {
    validate_exact_projection_relative(relative)
        .map_err(|error| RenderError::Conflict(error.to_string()))?;
    let current = read_capability_optional(target, relative)?;
    let matches = match (expected, current.as_deref()) {
        (ProjectionExpectedBefore::Absent, None) => true,
        (ProjectionExpectedBefore::Exact(expected), Some(current)) => current == expected,
        (ProjectionExpectedBefore::Absent, Some(_))
        | (ProjectionExpectedBefore::Exact(_), None) => false,
    };
    if matches {
        Ok(())
    } else {
        Err(RenderError::Conflict(format!(
            "Hive projection changed after ownership preflight: {}",
            relative.display()
        )))
    }
}

struct ProjectionClaim {
    parent: Dir,
    quarantine: Option<Dir>,
    quarantine_name: OsString,
    destination_name: OsString,
    recovery_path: PathBuf,
}

fn mutate_exact_projection_claimed(
    target: &Dir,
    relative: &Path,
    expected: &[u8],
    mutation: ExactProjectionMutation<'_>,
    before_claim: Option<&dyn Fn(&Path)>,
    cleanup_fault: Option<ProjectionCleanupFault>,
) -> Result<MutationOutcome, RenderError> {
    verify_projection_expected_before(
        target,
        relative,
        &ProjectionExpectedBefore::Exact(expected.to_vec()),
    )?;
    if let Some(barrier) = before_claim {
        barrier(relative);
    }
    let claim = claim_projection_destination(target, relative)?;
    let claimed = read_claimed_projection(&claim)?;
    if claimed != expected {
        return Err(projection_claim_conflict(
            claim,
            relative,
            "claimed bytes differ from the exact ownership proof",
        ));
    }

    match mutation {
        ExactProjectionMutation::Replace(bytes) => publish_claimed_projection_replacement(
            claim,
            relative,
            bytes,
            cleanup_fault == Some(ProjectionCleanupFault::Replacement),
        ),
        ExactProjectionMutation::Delete => Ok(finish_claimed_projection_deletion(
            claim,
            relative,
            cleanup_fault == Some(ProjectionCleanupFault::Deletion),
        )),
    }
}

fn claim_projection_destination(
    target: &Dir,
    relative: &Path,
) -> Result<ProjectionClaim, RenderError> {
    validate_exact_projection_relative(relative)
        .map_err(|error| RenderError::Conflict(error.to_string()))?;
    let mut created = Vec::new();
    let (parent, destination_name) = capability_parent(target, relative, false, &mut created)?
        .ok_or_else(|| {
            RenderError::Conflict(format!(
                "host Skill projection disappeared before it could be claimed: {}",
                relative.display()
            ))
        })?;
    let (quarantine, quarantine_name) = create_projection_quarantine(&parent).map_err(|error| {
        RenderError::Conflict(format!(
            "cannot allocate a private projection quarantine for {}: {error}",
            relative.display()
        ))
    })?;
    if let Err(error) = parent.rename(
        &destination_name,
        &quarantine,
        OsStr::new("claimed-SKILL.md"),
    ) {
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return Err(RenderError::Conflict(format!(
            "host Skill projection changed before atomic claim at {}: {error}",
            relative.display()
        )));
    }
    let recovery_path = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&quarantine_name)
        .join("claimed-SKILL.md");
    Ok(ProjectionClaim {
        parent,
        quarantine: Some(quarantine),
        quarantine_name,
        destination_name,
        recovery_path,
    })
}

fn create_projection_quarantine(parent: &Dir) -> io::Result<(Dir, OsString)> {
    for _ in 0..128 {
        let counter = ACTIVATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let epoch_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let name = OsString::from(format!(
            ".aigent-hive-claim-{}-{epoch_nanos:x}-{counter:x}",
            std::process::id()
        ));
        match parent.create_dir(&name) {
            Ok(()) => match parent.open_dir_nofollow(&name) {
                Ok(directory) => return Ok((directory, name)),
                Err(error) => {
                    let _ = parent.remove_dir(&name);
                    return Err(error);
                }
            },
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot allocate an exclusive projection quarantine",
    ))
}

fn read_claimed_projection(claim: &ProjectionClaim) -> Result<Vec<u8>, RenderError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle");
    let mut file = open_capability_file_nofollow(quarantine, OsStr::new("claimed-SKILL.md"))
        .map_err(|error| {
            RenderError::Conflict(format!(
                "claimed projection cannot be opened no-follow at {}: {error}",
                claim.recovery_path.display()
            ))
        })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|error| {
        RenderError::Conflict(format!(
            "claimed projection cannot be read at {}: {error}",
            claim.recovery_path.display()
        ))
    })?;
    Ok(bytes)
}

fn projection_claim_conflict(claim: ProjectionClaim, relative: &Path, reason: &str) -> RenderError {
    let recovery_path = claim.recovery_path.clone();
    match restore_projection_claim(claim) {
        Ok(()) => RenderError::Conflict(format!(
            "{reason} at {}; racing bytes were restored without overwrite",
            relative.display()
        )),
        Err(error) => RenderError::Conflict(format!(
            "{reason} at {}; every claimed byte remains recoverable at {} because non-overwriting restore failed: {error}",
            relative.display(),
            recovery_path.display()
        )),
    }
}

fn restore_projection_claim(mut claim: ProjectionClaim) -> io::Result<()> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle");
    quarantine.hard_link(
        OsStr::new("claimed-SKILL.md"),
        &claim.parent,
        &claim.destination_name,
    )?;
    quarantine.remove_file(OsStr::new("claimed-SKILL.md"))?;
    drop(claim.quarantine.take());
    claim.parent.remove_dir(&claim.quarantine_name)
}

fn publish_claimed_projection_replacement(
    mut claim: ProjectionClaim,
    relative: &Path,
    bytes: &[u8],
    inject_cleanup_failure: bool,
) -> Result<MutationOutcome, RenderError> {
    let temporary_name = match create_capability_temporary(
        &claim.parent,
        ".aigent-hive-projection-publish",
        bytes,
    ) {
        Ok(name) => name,
        Err(error) => {
            return Err(projection_claim_conflict(
                claim,
                relative,
                &format!("cannot stage exact projection replacement: {error}"),
            ));
        }
    };
    if let Err(error) =
        claim
            .parent
            .hard_link(&temporary_name, &claim.parent, &claim.destination_name)
    {
        let _ = claim.parent.remove_file(&temporary_name);
        return Err(projection_claim_conflict(
            claim,
            relative,
            &format!("exclusive projection publication was blocked: {error}"),
        ));
    }
    if inject_cleanup_failure {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "injected projection replacement cleanup failure after publication at {}; prior exact bytes remain recoverable at {}",
                relative.display(),
                claim.recovery_path.display()
            )),
        ));
    }
    if let Err(error) = claim.parent.remove_file(&temporary_name) {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "projection replacement published at {}, but private temporary cleanup failed; prior exact bytes remain recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            )),
        ));
    }
    if let Err(error) = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle")
        .remove_file(OsStr::new("claimed-SKILL.md"))
    {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "projection replacement published at {}, while prior exact bytes remain recoverable at {} because quarantine cleanup failed: {error}",
                relative.display(),
                claim.recovery_path.display()
            )),
        ));
    }
    drop(claim.quarantine.take());
    if let Err(error) = claim.parent.remove_dir(&claim.quarantine_name) {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "projection replacement published at {}, but its empty private quarantine could not be removed: {error}",
                relative.display()
            )),
        ));
    }
    Ok(MutationOutcome::Applied)
}

fn finish_claimed_projection_deletion(
    mut claim: ProjectionClaim,
    relative: &Path,
    inject_cleanup_failure: bool,
) -> MutationOutcome {
    if inject_cleanup_failure {
        return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                "injected projection deletion cleanup failure after atomic claim at {}; prior exact bytes remain recoverable at {}",
                relative.display(),
                claim.recovery_path.display()
            )));
    }
    match claim.parent.symlink_metadata(&claim.destination_name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Ok(_) => {
            return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                    "concurrent destination appeared while deleting {}; prior exact bytes remain recoverable at {}",
                    relative.display(),
                    claim.recovery_path.display()
                )));
        }
        Err(error) => {
            return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                    "cannot verify exclusive deletion destination at {}; prior exact bytes remain recoverable at {}: {error}",
                    relative.display(),
                    claim.recovery_path.display()
                )));
        }
    }
    if let Err(error) = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle")
        .remove_file(OsStr::new("claimed-SKILL.md"))
    {
        return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                "projection deletion could not finalize at {}; exact bytes remain recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            )));
    }
    drop(claim.quarantine.take());
    if let Err(error) = claim.parent.remove_dir(&claim.quarantine_name) {
        return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                "projection deletion completed at {}, but its empty private quarantine could not be removed: {error}",
                relative.display()
            )));
    }
    MutationOutcome::Applied
}

fn create_capability_file_exclusive(
    target: &Dir,
    destination: &Path,
    bytes: &[u8],
) -> io::Result<()> {
    let mut created = Vec::new();
    let (parent, file_name) = capability_parent(target, destination, false, &mut created)
        .map_err(|error| render_error_to_io(&error))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "managed parent is missing"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    match parent.open_with(&file_name, &options) {
        Ok(mut file) => {
            if let Err(error) = file
                .write_all(bytes)
                .and_then(|()| file.flush())
                .and_then(|()| file.sync_all())
            {
                drop(file);
                let _ = parent.remove_file(&file_name);
                return Err(error);
            }
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn replace_capability_file(target: &Dir, destination: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_capability_file_impl(
        target,
        destination,
        bytes,
        ReplacePolicy {
            destination_requires_backup: cfg!(windows),
            fail_after_backup: false,
        },
    )
}

fn replace_capability_file_impl(
    target: &Dir,
    destination: &Path,
    bytes: &[u8],
    policy: ReplacePolicy,
) -> io::Result<()> {
    let mut created = Vec::new();
    let (parent, file_name) = capability_parent(target, destination, false, &mut created)
        .map_err(|error| render_error_to_io(&error))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "managed parent is missing"))?;
    let destination_exists = match parent.symlink_metadata(&file_name) {
        Ok(metadata) if metadata.is_file() => true,
        Ok(_) => {
            return Err(io::Error::other(format!(
                "managed destination is occupied by a non-file: {}",
                destination.display()
            )));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    let temporary_name = create_capability_temporary(&parent, ".aigent-hive-activate", bytes)?;
    if !destination_exists || !policy.destination_requires_backup {
        let result = parent.rename(&temporary_name, &parent, &file_name);
        if result.is_err() {
            let _ = parent.remove_file(&temporary_name);
        }
        return result;
    }

    let backup_name = create_capability_temporary(&parent, ".aigent-hive-replace-backup", b"")?;
    if let Err(error) = parent.rename(&file_name, &parent, &backup_name) {
        let _ = parent.remove_file(&temporary_name);
        let _ = parent.remove_file(&backup_name);
        return Err(error);
    }
    if policy.fail_after_backup {
        let error = io::Error::other("injected failure after destination backup");
        let restored =
            restore_capability_replacement_backup(&parent, &file_name, &backup_name, error);
        let _ = parent.remove_file(&temporary_name);
        return Err(restored);
    }
    if let Err(error) = parent.rename(&temporary_name, &parent, &file_name) {
        let restored =
            restore_capability_replacement_backup(&parent, &file_name, &backup_name, error);
        let _ = parent.remove_file(&temporary_name);
        return Err(restored);
    }
    match parent.remove_file(&backup_name) {
        Ok(()) => Ok(()),
        Err(error) => Err(restore_capability_replacement_backup(
            &parent,
            &file_name,
            &backup_name,
            error,
        )),
    }
}

fn create_capability_temporary(parent: &Dir, prefix: &str, bytes: &[u8]) -> io::Result<OsString> {
    for _ in 0..128 {
        let counter = ACTIVATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let epoch_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let name = OsString::from(format!(
            "{prefix}-{}-{epoch_nanos:x}-{counter:x}",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        match parent.open_with(&name, &options) {
            Ok(mut file) => {
                if let Err(error) = file
                    .write_all(bytes)
                    .and_then(|()| file.flush())
                    .and_then(|()| file.sync_all())
                {
                    drop(file);
                    let _ = parent.remove_file(&name);
                    return Err(error);
                }
                return Ok(name);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot allocate an exclusive activation temporary file",
    ))
}

fn restore_capability_replacement_backup(
    parent: &Dir,
    destination: &OsStr,
    backup: &OsStr,
    original: io::Error,
) -> io::Error {
    match parent.remove_file(destination) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return io::Error::other(format!(
                "{original}; replacement cleanup failed before restore: {error}"
            ));
        }
    }
    match parent.rename(backup, parent, destination) {
        Ok(()) => original,
        Err(error) => io::Error::other(format!(
            "{original}; destination backup restore failed: {error}"
        )),
    }
}

fn remove_capability_file(target: &Dir, relative: &Path) -> io::Result<()> {
    let mut created = Vec::new();
    let Some((parent, file_name)) = capability_parent(target, relative, false, &mut created)
        .map_err(|error| render_error_to_io(&error))?
    else {
        return Ok(());
    };
    match parent.remove_file(file_name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_created_capability_directory(
    target: &Dir,
    relative: &Path,
    _projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
) -> io::Result<()> {
    let mut created = Vec::new();
    let Some((parent, directory_name)) =
        capability_parent_validated(target, relative, false, &mut created)
            .map_err(|error| render_error_to_io(&error))?
    else {
        return Ok(());
    };
    match parent.remove_dir(&directory_name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            let directory = parent.open_dir_nofollow(&directory_name)?;
            if directory.entries()?.next().is_some() {
                // A concurrent foreign entry owns the surviving directory. The
                // rollback has removed every Hive-created file, so preserving
                // the non-empty container is the safe, complete outcome.
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

fn validate_capability_activation(
    target: &Dir,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
) -> Result<(), RenderError> {
    for (relative, expected) in planned {
        let current = read_capability_optional(target, relative)?.ok_or_else(|| {
            RenderError::Verification(format!(
                "activated managed output is missing: {}",
                relative.display()
            ))
        })?;
        if current != *expected {
            return Err(RenderError::Verification(format!(
                "activated managed output differs from staged bytes: {}",
                relative.display()
            )));
        }
    }
    for relative in deletions {
        if read_capability_optional(target, relative)?.is_some() {
            return Err(RenderError::Verification(format!(
                "activated deletion remains installed: {}",
                relative.display()
            )));
        }
    }
    let approved_hooks: BTreeSet<_> = answers
        .approved_fallback_hooks
        .iter()
        .map(|hook| PathBuf::from(&hook.path))
        .collect();
    for relative in known_hook_descriptor_paths() {
        if !approved_hooks.contains(&relative)
            && read_capability_optional(target, &relative)?.is_some()
        {
            return Err(RenderError::Verification(format!(
                "unapproved known fallback hook remains installed: {}",
                relative.display()
            )));
        }
    }
    let _ = capability_tree_digest(target)?;
    Ok(())
}

fn capability_tree_digest(target: &Dir) -> Result<String, RenderError> {
    let manifest = ownership_manifest()?;
    let mut entries = BTreeMap::new();
    for entry in manifest.paths {
        if let Some(prefix) = entry.pattern.strip_suffix("/**") {
            collect_capability_owned_files(target, Path::new(prefix), &mut entries)?;
        } else if !entry.pattern.contains('*') {
            let relative = PathBuf::from(entry.pattern);
            if let Some(bytes) = read_capability_optional(target, &relative)? {
                entries.insert(relative, bytes);
            }
        }
    }
    if read_target_optional(target, Path::new(".hive/config/active-skills.yml"))?.is_some() {
        let answers = read_installed_answers(target)?;
        entries.extend(validate_projection_ownership(target, &answers)?.files);
    }
    Ok(digest_tree(&entries))
}

fn collect_capability_owned_files(
    target: &Dir,
    relative: &Path,
    entries: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    validate_project_relative(relative).map_err(|error| RenderError::Safety(error.to_string()))?;
    let mut created = Vec::new();
    let Some((parent, name)) = capability_parent(target, relative, false, &mut created)? else {
        return Ok(());
    };
    let metadata = match parent.symlink_metadata(&name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_internal(error)),
    };
    if metadata.is_file() {
        let bytes = read_capability_optional(target, relative)?.ok_or_else(|| {
            RenderError::Internal(format!(
                "owned file disappeared during activation validation: {}",
                relative.display()
            ))
        })?;
        entries.insert(relative.to_path_buf(), bytes);
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(RenderError::Safety(format!(
            "owned tree contains a non-file, non-directory entry: {}",
            relative.display()
        )));
    }
    let directory = open_capability_child_directory(&parent, &name, relative)?;
    collect_capability_directory_entries(&directory, relative, entries)
}

fn collect_capability_directory_entries(
    directory: &Dir,
    relative: &Path,
    entries: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    let mut children = directory
        .entries()
        .map_err(io_internal)?
        .map(|entry| entry.map(|entry| entry.file_name()).map_err(io_internal))
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    for child in children {
        let child_relative = relative.join(&child);
        validate_project_relative(&child_relative)
            .map_err(|error| RenderError::Safety(error.to_string()))?;
        let metadata = directory.symlink_metadata(&child).map_err(io_internal)?;
        if metadata.is_file() {
            let mut file = open_capability_file_nofollow(directory, &child).map_err(io_internal)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(io_internal)?;
            entries.insert(child_relative, bytes);
        } else if metadata.is_dir() {
            let child_directory =
                open_capability_child_directory(directory, &child, &child_relative)?;
            collect_capability_directory_entries(&child_directory, &child_relative, entries)?;
        } else {
            return Err(RenderError::Safety(format!(
                "owned tree contains a non-file, non-directory entry: {}",
                child_relative.display()
            )));
        }
    }
    Ok(())
}

fn open_capability_child_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
) -> Result<Dir, RenderError> {
    parent.open_dir_nofollow(name).map_err(|error| {
        RenderError::Safety(format!(
            "owned directory cannot be opened no-follow at {}: {error}",
            relative.display()
        ))
    })
}

struct ProjectionRecovery {
    parent: Dir,
    quarantine: Option<Dir>,
    quarantine_name: OsString,
    destination_name: OsString,
    recovery_path: PathBuf,
}

fn stage_projection_recovery(
    target: &Dir,
    relative: &Path,
    bytes: &[u8],
) -> Result<ProjectionRecovery, RenderError> {
    validate_hive_skill_projection_relative(relative)
        .map_err(|error| RenderError::Rollback(error.to_string()))?;
    let mut created = Vec::new();
    let (parent, destination_name) = capability_parent(target, relative, false, &mut created)?
        .ok_or_else(|| {
            RenderError::Rollback(format!(
                "projection rollback parent is missing: {}",
                relative.display()
            ))
        })?;
    let (quarantine, quarantine_name) = create_projection_quarantine(&parent).map_err(|error| {
        RenderError::Rollback(format!(
            "cannot allocate projection rollback recovery for {}: {error}",
            relative.display()
        ))
    })?;
    let recovery_name = OsStr::new("prior-SKILL.md");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = match quarantine.open_with(recovery_name, &options) {
        Ok(file) => file,
        Err(error) => {
            drop(quarantine);
            let _ = parent.remove_dir(&quarantine_name);
            return Err(RenderError::Rollback(format!(
                "cannot create projection rollback recovery for {}: {error}",
                relative.display()
            )));
        }
    };
    if let Err(error) = file
        .write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = quarantine.remove_file(recovery_name);
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return Err(RenderError::Rollback(format!(
            "cannot persist projection rollback recovery for {}: {error}",
            relative.display()
        )));
    }
    drop(file);
    let recovery_path = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&quarantine_name)
        .join(recovery_name);
    Ok(ProjectionRecovery {
        parent,
        quarantine: Some(quarantine),
        quarantine_name,
        destination_name,
        recovery_path,
    })
}

fn cleanup_projection_recovery(mut recovery: ProjectionRecovery) -> io::Result<()> {
    recovery
        .quarantine
        .as_ref()
        .expect("live projection recovery retains its quarantine handle")
        .remove_file(OsStr::new("prior-SKILL.md"))?;
    drop(recovery.quarantine.take());
    recovery.parent.remove_dir(&recovery.quarantine_name)
}

fn rollback_projection(
    target: &Dir,
    relative: &Path,
    prior: Option<&[u8]>,
    published: Option<&[u8]>,
) -> Result<(), RenderError> {
    match (prior, published) {
        (Some(prior), Some(published)) => {
            rollback_replaced_projection(target, relative, prior, published)
        }
        (None, Some(published)) => rollback_created_projection(target, relative, published),
        (Some(prior), None) => rollback_deleted_projection(target, relative, prior),
        (None, None) => Err(RenderError::Rollback(format!(
            "{}: projection rollback has neither prior nor published bytes",
            relative.display()
        ))),
    }
}

fn rollback_replaced_projection(
    target: &Dir,
    relative: &Path,
    prior: &[u8],
    published: &[u8],
) -> Result<(), RenderError> {
    let recovery = stage_projection_recovery(target, relative, prior)?;
    let recovery_path = recovery.recovery_path.clone();
    match mutate_exact_projection_claimed(
        target,
        relative,
        published,
        ExactProjectionMutation::Replace(prior),
        None,
        None,
    ) {
        Ok(MutationOutcome::Applied) => {
            cleanup_projection_recovery(recovery).map_err(|error| {
                RenderError::Rollback(format!(
                    "{}: prior bytes were restored, but private rollback recovery cleanup failed at {}: {error}",
                    relative.display(),
                    recovery_path.display()
                ))
            })
        }
        Ok(MutationOutcome::AppliedWithCleanupError(error)) => {
            let recovery_cleanup = cleanup_projection_recovery(recovery)
                .err()
                .map(|cleanup| format!("; rollback recovery cleanup also failed: {cleanup}"))
                .unwrap_or_default();
            Err(RenderError::Rollback(format!(
                "{}: prior bytes were restored, but reverse projection cleanup remains incomplete: {error}{recovery_cleanup}",
                relative.display()
            )))
        }
        Ok(MutationOutcome::Unchanged) => Err(RenderError::Rollback(format!(
            "{}: projection rollback unexpectedly reported no live change",
            relative.display()
        ))),
        Err(error) => {
            drop(recovery);
            Err(RenderError::Rollback(format!(
                "{}: live post-activation bytes changed before rollback; live/claimed bytes were preserved and prior bytes remain recoverable at {}: {error}",
                relative.display(),
                recovery_path.display()
            )))
        }
    }
}

fn rollback_created_projection(
    target: &Dir,
    relative: &Path,
    published: &[u8],
) -> Result<(), RenderError> {
    mutate_exact_projection_claimed(
        target,
        relative,
        published,
        ExactProjectionMutation::Delete,
        None,
        None,
    )
    .and_then(|outcome| match outcome {
        MutationOutcome::Applied => Ok(()),
        MutationOutcome::AppliedWithCleanupError(error) => Err(RenderError::Rollback(format!(
            "{}: newly created projection was removed, but reverse cleanup remains incomplete: {error}",
            relative.display()
        ))),
        MutationOutcome::Unchanged => Err(RenderError::Rollback(format!(
            "{}: new projection rollback unexpectedly reported no live change",
            relative.display()
        ))),
    })
    .map_err(|error| match error {
        RenderError::Rollback(message) => RenderError::Rollback(message),
        other => RenderError::Rollback(format!(
            "{}: newly created projection changed before rollback and was preserved: {other}",
            relative.display()
        )),
    })
}

fn rollback_deleted_projection(
    target: &Dir,
    relative: &Path,
    prior: &[u8],
) -> Result<(), RenderError> {
    let recovery = stage_projection_recovery(target, relative, prior)?;
    let recovery_path = recovery.recovery_path.clone();
    let quarantine = recovery
        .quarantine
        .as_ref()
        .expect("live projection recovery retains its quarantine handle");
    match quarantine.hard_link(
        OsStr::new("prior-SKILL.md"),
        &recovery.parent,
        &recovery.destination_name,
    ) {
        Ok(()) => cleanup_projection_recovery(recovery).map_err(|error| {
            RenderError::Rollback(format!(
                "{}: deleted projection bytes were restored, but private rollback recovery cleanup failed at {}: {error}",
                relative.display(),
                recovery_path.display()
            ))
        }),
        Err(error) => {
            drop(recovery);
            Err(RenderError::Rollback(format!(
                "{}: rollback refused to overwrite a newly occupied projection path; live bytes remain and prior bytes are recoverable at {}: {error}",
                relative.display(),
                recovery_path.display()
            )))
        }
    }
}

fn rollback(
    target: &Dir,
    previous: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
    applied: &[PathBuf],
    created_directories: &[PathBuf],
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
) -> Result<(), RenderError> {
    let mut errors = Vec::new();
    for relative in applied.iter().rev() {
        if is_exact_projection_path(relative) {
            let prior = previous.get(relative).ok_or_else(|| {
                RenderError::Rollback(format!(
                    "hive.activation-rollback-failed: {}: missing projection activation snapshot",
                    relative.display()
                ))
            })?;
            if let Err(error) = rollback_projection(
                target,
                relative,
                prior.as_deref(),
                planned.get(relative).map(Vec::as_slice),
            ) {
                errors.push(error.to_string());
            }
            continue;
        }
        match previous.get(relative) {
            Some(Some(content)) => {
                if let Err(error) = replace_capability_file(target, relative, content) {
                    errors.push(format!("{}: {error}", relative.display()));
                }
            }
            Some(None) => {
                if let Err(error) = remove_capability_file(target, relative) {
                    errors.push(format!("{}: {error}", relative.display()));
                }
            }
            None => {
                errors.push(format!(
                    "{}: missing activation snapshot",
                    relative.display()
                ));
            }
        }
    }
    for directory in created_directories.iter().rev() {
        if let Err(error) =
            remove_created_capability_directory(target, directory, projection_expected_before)
        {
            errors.push(format!("{}: {error}", directory.display()));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(RenderError::Rollback(format!(
            "hive.activation-rollback-failed: {}",
            errors.join("; ")
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn activation_failed(
    original: &RenderError,
    target: &Dir,
    previous: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
    applied: &[PathBuf],
    created_directories: &[PathBuf],
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    before_rollback: Option<&dyn Fn()>,
    fault: Option<ActivationFault>,
) -> Result<(), RenderError> {
    #[cfg(not(debug_assertions))]
    let _ = fault;
    #[cfg(debug_assertions)]
    if fault.is_some_and(|value| value.fail_rollback) {
        return Err(RenderError::Rollback(
            "hive.activation-rollback-failed: injected rollback failure".to_owned(),
        ));
    }
    if let Some(barrier) = before_rollback {
        barrier();
    }
    rollback(
        target,
        previous,
        planned,
        applied,
        created_directories,
        projection_expected_before,
    )?;
    let message =
        format!("activation failed and was rolled back to the prior generation: {original}");
    match original {
        RenderError::Rollback(_) => Err(RenderError::Rollback(format!(
            "{original}; live paths were rolled back but projection cleanup evidence remains"
        ))),
        RenderError::Conflict(_) => Err(RenderError::Conflict(message)),
        _ => Err(RenderError::Internal(message)),
    }
}

fn render_error_to_io(error: &RenderError) -> io::Error {
    io::Error::other(error.to_string())
}

fn should_inject_activation_failure(fault: Option<ActivationFault>, count: usize) -> bool {
    #[cfg(debug_assertions)]
    {
        fault.is_some_and(|value| value.fail_after_operations == count)
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = (fault, count);
        false
    }
}

fn activation_fault_from_environment() -> Option<ActivationFault> {
    #[cfg(debug_assertions)]
    {
        let value = std::env::var("HIVE_TEST_ACTIVATION_FAIL_AFTER").ok()?;
        let current_thread = format!("{:?}", std::thread::current().id());
        let fail_rollback =
            std::env::var_os("HIVE_TEST_ROLLBACK_FAIL").is_some_and(|value| value == "1");
        activation_fault_from_value(&value, &current_thread, fail_rollback)
    }
    #[cfg(not(debug_assertions))]
    {
        None
    }
}

#[cfg(debug_assertions)]
fn activation_fault_from_value(
    value: &str,
    current_thread: &str,
    fail_rollback: bool,
) -> Option<ActivationFault> {
    let (scope, fail_after_operations) = value
        .rsplit_once('@')
        .map_or((None, value), |(scope, operations)| {
            (Some(scope), operations)
        });
    if scope.is_some_and(|scope| scope != current_thread) {
        return None;
    }
    let fail_after_operations = fail_after_operations
        .parse::<usize>()
        .ok()
        .filter(|value| *value <= 4096)?;
    Some(ActivationFault {
        fail_after_operations,
        fail_rollback,
        projection_cleanup: None,
    })
}

fn staging_corruption_from_environment() -> bool {
    #[cfg(debug_assertions)]
    {
        std::env::var_os("HIVE_TEST_STAGING_CORRUPT_AFTER_RENDER").is_some_and(|value| value == "1")
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

fn validate_installed(target: &Path) -> Result<(), RenderError> {
    ensure_consumer_target(target).map_err(|error| RenderError::Verification(error.to_string()))?;
    let harness_relative = Path::new(".hive/config/harness.toml");
    let harness_bytes = read_target_required(target, harness_relative, "harness config")
        .map_err(as_verification)?;
    let harness_text = std::str::from_utf8(&harness_bytes)
        .map_err(|_| RenderError::Verification("installed harness is not UTF-8".to_owned()))?;
    let harness: InstalledHarness = toml::from_str(harness_text).map_err(|error| {
        RenderError::Verification(format!("invalid installed harness: {error}"))
    })?;
    if harness.schema_version != 1
        || harness.harness_version != env!("CARGO_PKG_VERSION")
        || harness.source_release_version != env!("CARGO_PKG_VERSION")
    {
        return Err(RenderError::Verification(format!(
            "installed harness version parity failed: expected {}",
            env!("CARGO_PKG_VERSION")
        )));
    }

    let installed_answers = read_installed_answers(target)?;
    validate_answers(&installed_answers).map_err(as_verification)?;
    let resolution = read_installed_resolution(target)?;
    validate_resolution(&installed_answers, &resolution).map_err(as_verification)?;
    validate_harness_cross_file(&harness, &installed_answers, &resolution)?;

    let skills_relative = Path::new(".hive/config/approved-skills.yml");
    let bytes = read_target_required(target, skills_relative, "Skill approval ledger")
        .map_err(as_verification)?;
    let ledger: SkillLedger = serde_yaml::from_slice(&bytes).map_err(|error| {
        RenderError::Verification(format!("invalid installed Skill ledger: {error}"))
    })?;
    validate_skill_approvals(&ledger.skills).map_err(as_verification)?;
    if ledger.skills != installed_answers.approved_optional_skills {
        return Err(RenderError::Verification(
            "installed Skill ledger does not match setup approvals".to_owned(),
        ));
    }
    validate_projection_ownership(target, &installed_answers).map_err(as_verification)?;

    let hooks_relative = Path::new(".hive/config/approved-hooks.yml");
    let installed_hook_bytes =
        read_target_optional(target, hooks_relative).map_err(as_verification)?;
    if installed_answers.approved_fallback_hooks.is_empty() {
        if installed_hook_bytes.is_some() {
            return Err(RenderError::Verification(
                "revoked fallback hook ledger is still installed".to_owned(),
            ));
        }
    } else {
        let bytes = installed_hook_bytes.ok_or_else(|| {
            RenderError::Verification(
                "approved fallback hook ledger is missing from the installation".to_owned(),
            )
        })?;
        let hook_value: JsonValue = serde_yaml::from_slice(&bytes).map_err(|error| {
            RenderError::Verification(format!("invalid installed hook ledger: {error}"))
        })?;
        validate_schema_instance(HOOK_SCHEMA, &hook_value, "installed hook ledger")
            .map_err(|error| RenderError::Verification(error.to_string()))?;
        let ledger: HookLedger = serde_json::from_value(hook_value).map_err(|error| {
            RenderError::Verification(format!("invalid installed hook ledger: {error}"))
        })?;
        if resolution.detection != "absent"
            || ledger.schema_version != 1
            || ledger.detection != "absent"
            || ledger.resolution_evidence_digest != resolution.evidence_digest
            || ledger.hooks != installed_answers.approved_fallback_hooks
        {
            return Err(RenderError::Verification(
                "installed hooks do not bind current approvals and absent evidence".to_owned(),
            ));
        }
        validate_hook_approvals(&ledger.hooks, &resolution).map_err(as_verification)?;
        for hook in &ledger.hooks {
            let relative = Path::new(&hook.path);
            let projected = read_target_required(target, relative, "hook descriptor")
                .map_err(as_verification)?;
            let expected = hook_descriptor_bytes(hook).map_err(as_verification)?;
            if projected != expected || sha256_digest(&projected) != hook.content_digest {
                return Err(RenderError::Verification(format!(
                    "installed hook descriptor changed: {}",
                    hook.capability
                )));
            }
        }
    }
    validate_hook_tree(target, &installed_answers.approved_fallback_hooks)?;
    validate_roles(target, &installed_answers.persistent_roles)?;
    validate_protected_contract(target, harness.wiki_enabled.unwrap_or(true))?;
    validate_editing_discipline(target)?;
    validate_installed_marker(target, &installed_answers, &resolution)?;
    Ok(())
}

fn read_installed_harness<T: TargetRead + ?Sized>(
    target: &T,
) -> Result<InstalledHarness, RenderError> {
    let relative = Path::new(".hive/config/harness.toml");
    let bytes =
        read_target_required(target, relative, "harness config").map_err(as_verification)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| RenderError::Verification("installed harness is not UTF-8".to_owned()))?;
    let harness: InstalledHarness = toml::from_str(text).map_err(|error| {
        RenderError::Verification(format!("invalid installed harness: {error}"))
    })?;
    if harness.schema_version != 1
        || harness.harness_version != harness.source_release_version
        || harness.harness_version.is_empty()
    {
        return Err(RenderError::Verification(
            "installed harness release identity is invalid".to_owned(),
        ));
    }
    Ok(harness)
}

fn effective_preferences_from_harness(
    harness: &InstalledHarness,
) -> Result<Option<EffectiveProjectPreferences>, RenderError> {
    let Some(provenance) = harness.preference_provenance.as_deref() else {
        return Ok(None);
    };
    let interface_language = harness.interface_language.clone().ok_or_else(|| {
        RenderError::Verification("installed effective interface language is missing".to_owned())
    })?;
    let wiki_enabled = harness.wiki_enabled.ok_or_else(|| {
        RenderError::Verification("installed effective Wiki state is missing".to_owned())
    })?;
    let wiki_language = harness.wiki_language.clone().ok_or_else(|| {
        RenderError::Verification("installed effective Wiki language is missing".to_owned())
    })?;
    let persona_id = harness.persona_id.clone().ok_or_else(|| {
        RenderError::Verification("installed effective persona is missing".to_owned())
    })?;
    let usage_guard_enabled = harness.usage_guard_enabled.ok_or_else(|| {
        RenderError::Verification("installed effective usage guard state is missing".to_owned())
    })?;
    if !matches!(provenance, "global-inherited" | "project-custom") {
        return Err(RenderError::Verification(
            "installed preference provenance is invalid".to_owned(),
        ));
    }
    let global = GlobalProjectPreferences {
        interface_language: interface_language.clone(),
        wiki_enabled,
        wiki_language: wiki_language.clone(),
        persona_id: persona_id.clone(),
        persona_custom_description: harness.persona_custom_description.clone(),
        selected_project_skills: harness.selected_project_skills.clone(),
        usage_guard_enabled,
        codexbar_fallback_enabled: harness.codexbar_fallback_enabled,
        usage_stop_remaining_percent: harness.usage_stop_remaining_percent,
    };
    validate_global_project_preferences(&global).map_err(as_verification)?;
    Ok(Some(EffectiveProjectPreferences {
        provenance: if provenance == "global-inherited" {
            "global-inherited"
        } else {
            "project-custom"
        },
        interface_language,
        wiki_enabled,
        wiki_language,
        persona_id,
        persona_custom_description: harness.persona_custom_description.clone(),
        selected_project_skills: harness.selected_project_skills.clone(),
        usage_guard_enabled,
        codexbar_fallback_enabled: harness.codexbar_fallback_enabled,
        usage_stop_remaining_percent: harness.usage_stop_remaining_percent,
    }))
}

fn read_installed_answers<T: TargetRead + ?Sized>(target: &T) -> Result<SetupAnswers, RenderError> {
    let relative = Path::new(".hive/setup-answers.yml");
    let bytes = read_target_required(target, relative, "setup answers").map_err(as_verification)?;
    let value: JsonValue = serde_yaml::from_slice(&bytes).map_err(|error| {
        RenderError::Verification(format!("invalid installed setup answers: {error}"))
    })?;
    validate_schema_instance(SETUP_SCHEMA, &value, "installed setup answers")
        .map_err(as_verification)?;
    serde_json::from_value(value).map_err(|error| {
        RenderError::Verification(format!("invalid installed setup answers: {error}"))
    })
}

fn validate_harness_cross_file(
    harness: &InstalledHarness,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    let hook_file = (!answers.approved_fallback_hooks.is_empty())
        .then_some(".hive/config/approved-hooks.yml".to_owned());
    let effective_preferences = effective_preferences_from_harness(harness)?;
    let preferences_match = match effective_preferences.as_ref() {
        None => harness.usage_stop_remaining_percent == answers.usage_stop_remaining_percent,
        Some(preferences) => {
            let expected_provenance = if answers.setup_mode == "expedited" {
                "global-inherited"
            } else {
                "project-custom"
            };
            harness.setup_mode.as_deref() == Some(answers.setup_mode.as_str())
                && preferences.provenance == expected_provenance
                && custom_preferences_match_answers(answers, preferences)?
        }
    };
    if harness.project_name != answers.project_name
        || harness.project_kind != answers.project_kind
        || harness.primary_host != answers.primary_host
        || harness.external_capability_detection != resolution.detection
        || harness.resolved_owner != resolution.resolved_owner
        || harness.resolution_evidence_digest != resolution.evidence_digest
        || !preferences_match
        || harness.elevated_judge_quorum != answers.elevated_judge_quorum
        || harness.critical_judge_quorum != answers.critical_judge_quorum
        || harness.approved_optional_skills_file != ".hive/config/approved-skills.yml"
        || harness.capability_resolution_file != ".hive/config/capability-resolution.yml"
        || harness.approved_fallback_hooks_file != hook_file
        || harness.role_seed_file != ".hive/config/role-seeds.yml"
        || harness.knowledge_scope_file != ".hive/config/knowledge-scope.yml"
    {
        return Err(RenderError::Verification(
            "installed harness config does not match its canonical ledgers".to_owned(),
        ));
    }
    Ok(())
}

fn custom_preferences_match_answers(
    answers: &SetupAnswers,
    effective: &EffectiveProjectPreferences,
) -> Result<bool, RenderError> {
    if answers.setup_mode != "custom" {
        return Ok(true);
    }
    let wiki = answers
        .wiki
        .as_ref()
        .expect("custom preferences were validated");
    let persona = answers
        .persona
        .as_ref()
        .expect("custom preferences were validated");
    let selected = resolve_project_skill_selection(
        answers
            .skills
            .as_ref()
            .expect("custom preferences were validated"),
    )?;
    Ok(effective.interface_language
        == *answers
            .interface_language
            .as_ref()
            .expect("custom preferences were validated")
        && effective.wiki_enabled == wiki.enabled
        && effective.wiki_language == wiki.language
        && effective.persona_id == persona.id
        && effective.persona_custom_description == persona.custom_description
        && effective.selected_project_skills == selected)
}

fn validate_roles(target: &Path, seeds: &[RoleSeed]) -> Result<(), RenderError> {
    for seed in seeds {
        let relative = PathBuf::from(format!(".hive/team/roles/{}.md", seed.role_id));
        let bytes =
            read_target_required(target, &relative, "role profile").map_err(as_verification)?;
        let (profile, _) = parse_role(&bytes).map_err(as_verification)?;
        if !profile.definition_matches(seed) {
            return Err(RenderError::Verification(format!(
                "installed role definition does not match role seed: {}",
                seed.role_id
            )));
        }
    }
    Ok(())
}

fn validate_editing_discipline(target: &Path) -> Result<(), RenderError> {
    const EXPECTED: &[u8] =
        include_bytes!("../../../harness/template/.hive/directives/00-editing-discipline.md");
    let relative = Path::new(".hive/directives/00-editing-discipline.md");
    let current = read_target_required(target, relative, "editing discipline directive")
        .map_err(as_verification)?;
    if current != EXPECTED {
        return Err(RenderError::Verification(
            "installed editing discipline directive differs from the shipped contract".to_owned(),
        ));
    }
    Ok(())
}

fn validate_protected_contract(target: &Path, wiki_enabled: bool) -> Result<(), RenderError> {
    const ALWAYS_REQUIRED: &[&str] = &[
        ".hive/knowledge/Raw/README.md",
        ".hive/knowledge/Schema/schema.md",
        ".hive/knowledge/suppression.yml",
        ".hive/runs/README.md",
        ".hive/team/roles/README.md",
    ];
    for path in ALWAYS_REQUIRED
        .iter()
        .copied()
        .chain(wiki_enabled.then_some(".hive/knowledge/Wiki/index.md"))
        .chain(wiki_enabled.then_some(".hive/knowledge/Wiki/log.md"))
    {
        let bytes = read_target_required(target, Path::new(path), "protected canonical seed")
            .map_err(as_verification)?;
        if bytes.is_empty() {
            return Err(RenderError::Verification(format!(
                "protected canonical seed is empty: {path}"
            )));
        }
    }
    let suppression = read_target_required(
        target,
        Path::new(".hive/knowledge/suppression.yml"),
        "suppression ledger",
    )
    .map_err(as_verification)?;
    let value: JsonValue = serde_yaml::from_slice(&suppression).map_err(|error| {
        RenderError::Verification(format!("invalid suppression ledger: {error}"))
    })?;
    validate_schema_instance(
        KNOWLEDGE_SUPPRESSION_SCHEMA,
        &value,
        "knowledge suppression ledger",
    )
    .map_err(as_verification)?;
    let object = value.as_object().ok_or_else(|| {
        RenderError::Verification("suppression ledger must be an object".to_owned())
    })?;
    if object.get("schema_version") != Some(&JsonValue::from(1))
        || !object.get("entries").is_some_and(JsonValue::is_array)
    {
        return Err(RenderError::Verification(
            "suppression ledger contract is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_installed_marker(
    target: &Path,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    let relative = Path::new("AGENTS.md");
    let current =
        read_target_required(target, relative, "shared AGENTS marker").map_err(as_verification)?;
    let harness = read_installed_harness(target)?;
    let effective_preferences = effective_preferences_from_harness(&harness)?;
    let desired = render_agents_marker(answers, resolution, effective_preferences.as_ref());
    let merged =
        merge_shared_marker(target, relative, desired.as_bytes()).map_err(as_verification)?;
    if current != merged {
        return Err(RenderError::Verification(
            "installed AGENTS.md Hive marker is stale or malformed".to_owned(),
        ));
    }
    Ok(())
}

fn validate_hook_tree(target: &Path, hooks: &[HookApproval]) -> Result<(), RenderError> {
    let mut installed = BTreeMap::new();
    collect_owned_files(target, Path::new(".hive/hooks"), &mut installed)
        .map_err(as_verification)?;
    let expected: BTreeSet<_> = hooks.iter().map(|hook| PathBuf::from(&hook.path)).collect();
    let mut installed_known = BTreeSet::new();
    for relative in known_hook_descriptor_paths() {
        if read_target_optional(target, &relative)
            .map_err(as_verification)?
            .is_some()
        {
            installed_known.insert(relative);
        }
    }
    if expected != installed_known {
        return Err(RenderError::Verification(
            "installed hook tree does not match approved Hive descriptors".to_owned(),
        ));
    }
    Ok(())
}

fn known_hook_descriptor_paths() -> impl Iterator<Item = PathBuf> {
    [
        "protect-hive-owned-state",
        "update-integrity-guard",
        "derived-state-invalidation",
        "checkpoint-reminder",
    ]
    .into_iter()
    .map(|capability| PathBuf::from(format!(".hive/hooks/{capability}")))
}

fn validate_installed_against(
    target: &Path,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    validate_installed(target)?;
    if read_installed_answers(target)? != *answers
        || read_installed_resolution(target)? != *resolution
    {
        return Err(RenderError::Verification(
            "installed setup contract does not match supplied answers and capability evidence"
                .to_owned(),
        ));
    }
    for (relative, expected) in planned {
        let current =
            read_target_required(target, relative, "managed output").map_err(as_verification)?;
        if current != *expected {
            return Err(RenderError::Verification(format!(
                "installed managed output differs from the supplied contract: {}",
                relative.display()
            )));
        }
    }
    let stale = stale_hook_deletions(target, &answers.approved_fallback_hooks, resolution)
        .map_err(as_verification)?;
    for relative in stale {
        if read_target_optional(target, &relative)
            .map_err(as_verification)?
            .is_some()
        {
            return Err(RenderError::Verification(
                "revoked fallback hook artifacts remain installed".to_owned(),
            ));
        }
    }
    let _ = installed_tree_digest(target)?;
    Ok(())
}

fn read_installed_resolution<T: TargetRead + ?Sized>(
    target: &T,
) -> Result<CapabilityResolution, RenderError> {
    let relative = Path::new(".hive/config/capability-resolution.yml");
    let resolution_bytes =
        read_target_required(target, relative, "capability resolution").map_err(as_verification)?;
    let resolution_value: JsonValue =
        serde_yaml::from_slice(&resolution_bytes).map_err(|error| {
            RenderError::Verification(format!("invalid installed capability resolution: {error}"))
        })?;
    validate_schema_instance(
        CAPABILITY_SCHEMA,
        &resolution_value,
        "installed capability resolution",
    )
    .map_err(|error| RenderError::Verification(error.to_string()))?;
    serde_json::from_value(resolution_value).map_err(|error| {
        RenderError::Verification(format!("invalid installed capability resolution: {error}"))
    })
}

fn installed_tree_digest(target: &Path) -> Result<String, RenderError> {
    let manifest = ownership_manifest()?;
    let mut entries = BTreeMap::new();
    for entry in manifest.paths {
        if let Some(prefix) = entry.pattern.strip_suffix("/**") {
            collect_owned_files(target, Path::new(prefix), &mut entries)?;
        } else if !entry.pattern.contains('*') {
            let relative = PathBuf::from(entry.pattern);
            if let Some(bytes) = read_target_optional(target, &relative)? {
                entries.insert(relative, bytes);
            }
        }
    }
    if read_target_optional(target, Path::new(".hive/config/active-skills.yml"))?.is_some() {
        let answers = read_installed_answers(target)?;
        entries.extend(validate_projection_ownership(target, &answers)?.files);
    }
    Ok(digest_tree(&entries))
}

fn collect_owned_files(
    target: &Path,
    relative: &Path,
    entries: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    validate_project_relative(relative).map_err(|error| RenderError::Safety(error.to_string()))?;
    ensure_no_symlink_ancestors(target, relative)
        .map_err(|error| RenderError::Safety(error.to_string()))?;
    let absolute = target.join(relative);
    let metadata = match fs::symlink_metadata(&absolute) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_internal(error)),
    };
    if metadata.is_file() {
        entries.insert(
            relative.to_path_buf(),
            fs::read(&absolute).map_err(io_internal)?,
        );
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(RenderError::Safety(format!(
            "owned tree contains a non-file, non-directory entry: {}",
            relative.display()
        )));
    }
    let mut children = Vec::new();
    for child in fs::read_dir(&absolute).map_err(io_internal)? {
        let child = child.map_err(io_internal)?;
        children.push(child.file_name());
    }
    children.sort();
    for child in children {
        collect_owned_files(target, &relative.join(child), entries)?;
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn as_verification(error: RenderError) -> RenderError {
    RenderError::Verification(error.to_string())
}

fn digest_tree(files: &BTreeMap<PathBuf, Vec<u8>>) -> String {
    let mut bytes = Vec::new();
    for (path, content) in files {
        bytes.extend_from_slice(path.to_string_lossy().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(sha256_digest(content).as_bytes());
        bytes.push(b'\n');
    }
    sha256_digest(&bytes)
}

fn render_setup_answers(answers: &SetupAnswers) -> Result<Vec<u8>, RenderError> {
    let quote = |value: &str| {
        serde_json::to_string(value).expect("serializing a string to JSON cannot fail")
    };
    let mut output = format!(
        "# Generated from setup answers. Change values through a validated Hive reconfigure action.\n\
schema_version: 1\n\
project_name: {}\n\
project_identity: {}\n\
setup_mode: {}\n\
project_kind: {}\n\
primary_host: {}\n\
usage_stop_remaining_percent: {}\n\
elevated_judge_quorum: {}\n\
critical_judge_quorum: {}\n",
        quote(&answers.project_name),
        quote(if answers.project_identity.is_empty() {
            &answers.project_name
        } else {
            &answers.project_identity
        }),
        quote(&answers.setup_mode),
        quote(&answers.project_kind),
        quote(&answers.primary_host),
        answers.usage_stop_remaining_percent,
        quote(&answers.elevated_judge_quorum),
        quote(&answers.critical_judge_quorum),
    );
    if answers.setup_mode == "custom" {
        append_yaml_key(
            &mut output,
            "interface_language",
            answers
                .interface_language
                .as_ref()
                .expect("validated custom setup has interface language"),
        )?;
        append_yaml_key(
            &mut output,
            "wiki",
            answers
                .wiki
                .as_ref()
                .expect("validated custom setup has Wiki preferences"),
        )?;
        append_yaml_key(
            &mut output,
            "persona",
            answers
                .persona
                .as_ref()
                .expect("validated custom setup has persona"),
        )?;
        append_yaml_key(
            &mut output,
            "skills",
            answers
                .skills
                .as_ref()
                .expect("validated custom setup has Skill selection"),
        )?;
    }
    append_yaml_key(&mut output, "persistent_roles", &answers.persistent_roles)?;
    append_yaml_key(
        &mut output,
        "knowledge_include_paths",
        &answers.knowledge_include_paths,
    )?;
    append_yaml_key(
        &mut output,
        "knowledge_exclude_paths",
        &answers.knowledge_exclude_paths,
    )?;
    append_yaml_key(
        &mut output,
        "root_knowledge_promotion_categories",
        &answers.root_knowledge_promotion_categories,
    )?;
    append_yaml_key(
        &mut output,
        "confidential_knowledge_categories",
        &answers.confidential_knowledge_categories,
    )?;
    output.push_str("user_store_binding: ");
    output.push_str(&quote(&answers.user_store_binding));
    output.push('\n');
    append_yaml_key(
        &mut output,
        "approved_optional_skills",
        &answers.approved_optional_skills,
    )?;
    quote_approved_at_scalars(&mut output);
    append_copier_hook_list(
        &mut output,
        "approved_fallback_hooks",
        &answers.approved_fallback_hooks,
    );
    Ok(output.into_bytes())
}

fn quote_approved_at_scalars(output: &mut String) {
    let mut quoted = String::with_capacity(output.len());
    for line in output.lines() {
        let trimmed = line.trim_start();
        if let Some(value) = trimmed.strip_prefix("approved_at: ") {
            let indentation = &line[..line.len() - trimmed.len()];
            let value = serde_json::to_string(value)
                .expect("serializing an approved_at string to JSON cannot fail");
            quoted.push_str(indentation);
            quoted.push_str("approved_at: ");
            quoted.push_str(&value);
        } else {
            quoted.push_str(line);
        }
        quoted.push('\n');
    }
    *output = quoted;
}

fn render_knowledge_scope(answers: &SetupAnswers) -> Result<Vec<u8>, RenderError> {
    let mut output = "schema_version: 1\n".to_owned();
    append_yaml_key(&mut output, "include", &answers.knowledge_include_paths)?;
    append_yaml_key(&mut output, "exclude", &answers.knowledge_exclude_paths)?;
    append_yaml_key(
        &mut output,
        "root_promotion_categories",
        &answers.root_knowledge_promotion_categories,
    )?;
    append_yaml_key(
        &mut output,
        "confidential_categories",
        &answers.confidential_knowledge_categories,
    )?;
    output.push_str("project_identity: ");
    output.push_str(
        &serde_json::to_string(if answers.project_identity.is_empty() {
            &answers.project_name
        } else {
            &answers.project_identity
        })
        .expect("serializing a project identity cannot fail"),
    );
    output.push('\n');
    output.push_str("user_store_binding: ");
    output.push_str(
        &serde_json::to_string(&answers.user_store_binding)
            .expect("serializing a user-store binding cannot fail"),
    );
    output.push('\n');
    Ok(output.into_bytes())
}

fn render_hook_ledger(hooks: &[HookApproval], resolution: &CapabilityResolution) -> Vec<u8> {
    let mut output = "# Generated only from explicit fallback-hook approvals after conclusive external capability absence.\n\
schema_version: 1\n\
detection: absent\n\
resolution_evidence_digest: "
        .to_owned();
    output.push_str(
        &serde_json::to_string(&resolution.evidence_digest)
            .expect("serializing a string to JSON cannot fail"),
    );
    output.push('\n');
    append_copier_hook_list(&mut output, "hooks", hooks);
    output.into_bytes()
}

fn append_copier_hook_list(output: &mut String, key: &str, hooks: &[HookApproval]) {
    output.push_str(key);
    output.push_str(":\n");
    if hooks.is_empty() {
        output.push_str("  []\n");
        return;
    }
    for hook in hooks {
        output.push_str("  -   approved_at: '");
        output.push_str(&hook.approved_at);
        output.push_str("'\n      capability: ");
        output.push_str(&hook.capability);
        output.push('\n');
        output.push_str("      command: ");
        if hook.capability == "derived-state-invalidation" {
            let prefix = hook
                .command
                .strip_suffix(" --output json")
                .expect("validated derived-state command has the fixed suffix");
            output.push_str(prefix);
            output.push_str("\n          --output json\n");
        } else if matches!(
            hook.capability.as_str(),
            "protect-hive-owned-state" | "update-integrity-guard"
        ) {
            let prefix = hook
                .command
                .strip_suffix(" json")
                .expect("validated PreToolUse command has the fixed suffix");
            output.push_str(prefix);
            output.push_str("\n          json\n");
        } else {
            output.push_str(&hook.command);
            output.push('\n');
        }
        output.push_str("      consent_digest: ");
        output.push_str(&hook.consent_digest);
        output.push_str("\n      consent_version: ");
        output.push_str(&hook.consent_version.to_string());
        output.push_str("\n      content_digest: ");
        output.push_str(&hook.content_digest);
        output.push_str("\n      event: ");
        output.push_str(&hook.event);
        output.push_str("\n      path: ");
        output.push_str(&hook.path);
        output.push('\n');
    }
}

fn render_capability_resolution(resolution: &CapabilityResolution) -> Result<Vec<u8>, RenderError> {
    let quote = |value: &str| {
        serde_json::to_string(value).expect("serializing a string to JSON cannot fail")
    };
    let mut output = format!(
        "# Generated from a read-only active-host capability probe.\n\
# evidence_digest binds this normalized object except the evidence_digest field itself.\n\
schema_version: 1\n\
host: {}\n\
host_version: {}\n\
surface: {}\n\
detection: {}\n\
external_runtime: {}\n\
resolved_owner: {}\n\
capabilities:\n",
        quote(&resolution.host),
        quote(&resolution.host_version),
        quote(&resolution.surface),
        quote(&resolution.detection),
        resolution
            .external_runtime
            .as_deref()
            .map_or_else(|| "null".to_owned(), quote),
        quote(&resolution.resolved_owner),
    );
    let capabilities = serde_yaml::to_string(&resolution.capabilities)
        .map_err(|error| RenderError::Internal(format!("cannot render YAML: {error}")))?;
    output.push_str(&indent_yaml(&capabilities, 2));
    if let Some(hook_events) = &resolution.hook_events {
        output.push_str("hook_events:\n");
        let hook_events = serde_yaml::to_string(hook_events)
            .map_err(|error| RenderError::Internal(format!("cannot render YAML: {error}")))?;
        output.push_str(&indent_yaml(&hook_events, 2));
    }
    output.push_str("evidence_digest: ");
    output.push_str(&quote(&resolution.evidence_digest));
    output.push_str("\nevidence:\n");
    for evidence in &resolution.evidence {
        output.push_str("  -   digest: ");
        output.push_str(&yaml_scalar(&evidence.digest)?);
        output.push_str("\n      locator: ");
        output.push_str(&yaml_scalar(&evidence.locator)?);
        output.push_str("\n      outcome: ");
        output.push_str(&yaml_scalar(&evidence.outcome)?);
        output.push_str("\n      source: ");
        output.push_str(&yaml_scalar(&evidence.source)?);
        output.push('\n');
    }
    Ok(output.into_bytes())
}

fn yaml_scalar(value: &str) -> Result<String, RenderError> {
    let encoded = serde_yaml::to_string(value)
        .map_err(|error| RenderError::Internal(format!("cannot render YAML scalar: {error}")))?;
    Ok(encoded.trim_end().to_owned())
}

fn append_yaml_key<T: Serialize>(
    output: &mut String,
    key: &str,
    value: &T,
) -> Result<(), RenderError> {
    let yaml = serde_yaml::to_string(value)
        .map_err(|error| RenderError::Internal(format!("cannot render YAML: {error}")))?;
    output.push_str(key);
    output.push_str(":\n");
    output.push_str(&indent_yaml(&yaml, 2));
    Ok(())
}

fn indent_yaml(value: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    let mut output = String::with_capacity(value.len() + spaces * value.lines().count());
    for line in value.lines() {
        output.push_str(&prefix);
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn render_yaml_projection<T: Serialize>(
    comment: Option<&str>,
    key: &str,
    value: &T,
) -> Result<Vec<u8>, RenderError> {
    let mut output = comment.map_or_else(String::new, |comment| format!("{comment}\n"));
    append_yaml_key(&mut output, key, value)?;
    Ok(output.into_bytes())
}

fn validate_schema_instance(
    schema: &str,
    instance: &JsonValue,
    label: &str,
) -> Result<(), RenderError> {
    let schema_value: JsonValue = serde_json::from_str(schema).map_err(|error| {
        RenderError::Internal(format!("embedded {label} schema is invalid JSON: {error}"))
    })?;
    jsonschema::meta::validate(&schema_value).map_err(|error| {
        RenderError::Internal(format!("embedded {label} schema is invalid: {error}"))
    })?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema_value)
        .map_err(|error| {
            RenderError::Internal(format!("cannot compile embedded {label} schema: {error}"))
        })?;
    validator.validate(instance).map_err(|error| {
        RenderError::Input(format!("{label} violate the JSON Schema contract: {error}"))
    })
}

fn valid_role_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (2..=63).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        }))
    {
        return false;
    }
    let parse = |range: std::ops::Range<usize>| {
        value[range]
            .parse::<u32>()
            .expect("timestamp digit positions were validated")
    };
    let year = parse(0..4);
    let month = parse(5..7);
    let day = parse(8..10);
    let hour = parse(11..13);
    let minute = parse(14..16);
    let second = parse(17..19);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day) && hour < 24 && minute < 60 && second < 60
}

fn strictly_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_unique(values: &[String]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn contains_parent_component(path: &str) -> bool {
    path.split('/').any(|component| component == "..")
}

fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect()
}

fn path_string(path: &Path) -> Result<String, RenderError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| RenderError::Input("managed path is not UTF-8".to_owned()))
}

fn io_internal(error: io::Error) -> RenderError {
    let message = error.to_string();
    drop(error);
    RenderError::Internal(message)
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::execute_setup_in;
    use super::{
        activate_staged_impl, activation_fault_from_value, authorize_hook,
        authorize_hook_with_resolution, calculate_consent_digest, capability_detection,
        derive_resolution, encode_role, execute_release_update_for_target_in,
        execute_release_update_in, execute_setup, execute_setup_with_post_apply,
        expected_external_runtime, historical_project_upgrade_candidate_in, hook_descriptor_bytes,
        installed_tree_digest, load_answers, load_resolution, merge_shared_marker,
        mutate_exact_projection_claimed, open_target_capability, parse_role,
        prepare_projection_transition, project_upgrade_candidate_in, render_agents_marker,
        render_project_base, render_tree, replace_capability_file_impl,
        require_operational_update_preferences, resolve_effective_project_preferences,
        resolve_project_skill_selection, shared_marker_foreign_digest, update_path_is_owned,
        valid_digest, valid_role_id, valid_timestamp, validate_hook_approvals,
        validate_owned_paths, validate_skill_approvals, ActivationFault, ActiveSkills,
        CapabilityEvidence, CapabilityResolution, ExactProjectionMutation,
        GlobalProjectPreferences, HookApproval, HookAuthorization, ProjectSkillSelection,
        ProjectionCleanupFault, RenderError, ReplacePolicy, RoleProfile, RoleSeed, SetupAnswers,
        SetupMode, SetupRequest, SkillApproval, ValidatedProjectionOwnership,
        FRESH_CAPABILITY_RESOLUTION_PATH, MARKER_END, MARKER_START,
    };
    use hive_core::{sha256_digest, validate_project_relative};
    use serde_json::Value as JsonValue;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};
    use std::{fs, io};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase1")
            .join(name)
    }

    #[test]
    fn activation_fault_scope_is_limited_to_the_owning_test_thread() {
        let fault = activation_fault_from_value("ThreadId(7)@2", "ThreadId(7)", true)
            .expect("matching thread scope");
        assert_eq!(fault.fail_after_operations, 2);
        assert!(fault.fail_rollback);
        assert!(activation_fault_from_value("ThreadId(7)@2", "ThreadId(8)", false).is_none());
        assert_eq!(
            activation_fault_from_value("2", "ThreadId(8)", false)
                .expect("legacy process scope")
                .fail_after_operations,
            2
        );
    }

    #[test]
    fn update_recovery_ownership_excludes_canonical_runtime_and_foreign_paths() {
        for owned in [
            ".hive/config/harness.toml",
            ".hive/setup-answers.yml",
            ".agents/skills/hive-update/SKILL.md",
            ".agents/skills/hive-update/agents/openai.yaml",
            "AGENTS.md",
        ] {
            assert!(
                update_path_is_owned(Path::new(owned)).expect("ownership"),
                "{owned}"
            );
        }
        for excluded in [
            ".hive/knowledge/Wiki/page.md",
            ".hive/team/roles/reviewer.md",
            ".hive/runtime/update-journal.json",
            ".hive/backups/txn-a/file",
            "README.md",
        ] {
            assert!(
                !update_path_is_owned(Path::new(excluded)).expect("ownership"),
                "{excluded}"
            );
        }
        assert!(update_path_is_owned(Path::new(".omx/state.json")).is_err());
    }

    fn phase3_fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/phase3")
            .join(name)
    }

    fn apply_fixture(target: &Path, answers: &str, capabilities: &str) {
        let target = target
            .canonicalize()
            .expect("fixture target should have a stable path");
        execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture(answers),
            capabilities: &fixture(capabilities),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect("fixture setup should apply");
    }

    fn snapshot_tree(root: &Path) -> Vec<(String, String, Vec<u8>)> {
        fn collect(root: &Path, current: &Path, snapshot: &mut Vec<(String, String, Vec<u8>)>) {
            let mut entries = fs::read_dir(current)
                .expect("snapshot directory should be readable")
                .collect::<Result<Vec<_>, _>>()
                .expect("snapshot entries should be readable");
            entries.sort_by_key(std::fs::DirEntry::file_name);
            for entry in entries {
                let path = entry.path();
                let relative = path
                    .strip_prefix(root)
                    .expect("snapshot path should stay under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                let metadata =
                    fs::symlink_metadata(&path).expect("snapshot metadata should be readable");
                if metadata.file_type().is_symlink() {
                    snapshot.push((
                        relative,
                        "symlink".to_owned(),
                        fs::read_link(&path)
                            .expect("snapshot symlink should be readable")
                            .to_string_lossy()
                            .into_owned()
                            .into_bytes(),
                    ));
                } else if metadata.is_dir() {
                    snapshot.push((relative, "directory".to_owned(), Vec::new()));
                    collect(root, &path, snapshot);
                } else {
                    snapshot.push((
                        relative,
                        "file".to_owned(),
                        fs::read(&path).expect("snapshot file should be readable"),
                    ));
                }
            }
        }

        let mut snapshot = Vec::new();
        collect(root, root, &mut snapshot);
        snapshot
    }

    #[test]
    fn expedited_setup_inherits_effective_preferences_and_exact_skills() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let preferences = GlobalProjectPreferences {
            interface_language: "ko".to_owned(),
            wiki_enabled: false,
            wiki_language: "both".to_owned(),
            persona_id: "friendly".to_owned(),
            persona_custom_description: None,
            selected_project_skills: vec![
                "setup-harness".to_owned(),
                "hive-prompt-refine".to_owned(),
            ],
            usage_guard_enabled: true,
            codexbar_fallback_enabled: true,
            usage_stop_remaining_percent: 17,
        };

        let outcome = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(preferences.clone()),
        })
        .expect("expedited setup should inherit global preferences");

        let effective = outcome
            .effective_preferences
            .expect("effective preferences should be public evidence");
        assert_eq!(effective.setup_mode, "expedited");
        assert_eq!(effective.provenance, "global-inherited");
        assert!(!effective.wiki_enabled);
        assert!(effective.codexbar_fallback_enabled);
        assert_eq!(effective.usage_stop_remaining_percent, 17);
        assert!(!target.join(".hive/knowledge/Wiki/index.md").exists());
        assert!(!target.join(".hive/knowledge/Wiki/log.md").exists());
        assert!(target
            .join(".agents/skills/setup-harness/SKILL.md")
            .is_file());
        assert!(target
            .join(".agents/skills/hive-prompt-refine/SKILL.md")
            .is_file());
        assert!(!target
            .join(".agents/skills/hive-knowledge-query/SKILL.md")
            .exists());
        let harness =
            fs::read_to_string(target.join(".hive/config/harness.toml")).expect("harness config");
        assert!(harness.contains("preference_provenance = \"global-inherited\""));
        assert!(harness.contains("usage_stop_remaining_percent = 17"));
        assert!(harness.contains("codexbar_fallback_enabled = true"));
        assert!(harness
            .contains("selected_project_skills = [\"hive-prompt-refine\", \"setup-harness\"]"));

        execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Validate,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(preferences),
        })
        .expect("effective preference installation should validate");
    }

    #[test]
    fn connected_commit_failure_rolls_back_the_project_activation() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let error = execute_setup_with_post_apply(
            &SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::Apply,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: Some(GlobalProjectPreferences {
                    interface_language: "ko".to_owned(),
                    wiki_enabled: true,
                    wiki_language: "both".to_owned(),
                    persona_id: "friendly".to_owned(),
                    persona_custom_description: None,
                    selected_project_skills: vec!["setup-harness".to_owned()],
                    usage_guard_enabled: false,
                    codexbar_fallback_enabled: false,
                    usage_stop_remaining_percent: 20,
                }),
            },
            &|| {
                Err(RenderError::Verification(
                    "injected connected commit failure".to_owned(),
                ))
            },
        )
        .expect_err("connected commit failure must abort setup");

        assert!(error
            .to_string()
            .contains("injected connected commit failure"));
        assert_eq!(fs::read_dir(&target).expect("target entries").count(), 0);
    }

    #[test]
    fn release_update_replays_installed_connected_preferences() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let preferences = GlobalProjectPreferences {
            interface_language: "ko".to_owned(),
            wiki_enabled: true,
            wiki_language: "both".to_owned(),
            persona_id: "friendly".to_owned(),
            persona_custom_description: None,
            selected_project_skills: vec![
                "hive-prompt-refine".to_owned(),
                "setup-harness".to_owned(),
            ],
            usage_guard_enabled: false,
            codexbar_fallback_enabled: false,
            usage_stop_remaining_percent: 17,
        };
        let installed = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(preferences),
        })
        .expect("connected setup");
        let harness_before =
            fs::read(target.join(".hive/config/harness.toml")).expect("harness config");
        let skills_before =
            fs::read(target.join(".hive/config/active-skills.yml")).expect("active skills");
        let target_dir = open_target_capability(&target).expect("target capability");

        let updated = execute_release_update_in(
            &SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::Apply,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            },
            &target_dir,
            env!("CARGO_PKG_VERSION"),
        )
        .expect("release update should replay installed preferences");

        assert_eq!(
            updated.effective_preferences,
            installed.effective_preferences
        );
        assert!(updated.changed_paths.is_empty());
        assert_eq!(
            fs::read(target.join(".hive/config/harness.toml")).expect("harness config"),
            harness_before
        );
        assert_eq!(
            fs::read(target.join(".hive/config/active-skills.yml")).expect("active skills"),
            skills_before
        );
        assert!(!target
            .join(".agents/skills/hive-knowledge-query/SKILL.md")
            .exists());
    }

    #[test]
    fn operational_release_update_requires_connected_preferences() {
        let preferences = GlobalProjectPreferences {
            interface_language: "ko".to_owned(),
            wiki_enabled: true,
            wiki_language: "both".to_owned(),
            persona_id: "friendly".to_owned(),
            persona_custom_description: None,
            selected_project_skills: vec!["setup-harness".to_owned()],
            usage_guard_enabled: true,
            codexbar_fallback_enabled: false,
            usage_stop_remaining_percent: 20,
        };
        let error = require_operational_update_preferences("0.8.0", None)
            .expect_err("operational update must not publish legacy defaults");

        assert!(matches!(error, RenderError::Unsupported(_)));
        assert!(error.to_string().contains("validated user setup"));
        assert!(error.to_string().contains("shared-registry binding"));
        require_operational_update_preferences("0.7.0", None)
            .expect("historical non-operational update remains supported");
        require_operational_update_preferences("0.8.0", Some(&preferences))
            .expect("validated connected preferences satisfy the operational boundary");
    }

    #[test]
    fn historical_unconnected_07_install_cannot_publish_operational_08() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let target_dir = open_target_capability(&target).expect("target capability");
        let historical = historical_project_upgrade_candidate_in(&target_dir, "0.7.0")
            .expect("frozen 0.7 project base");
        let mut frozen_files = BTreeMap::new();
        for entry in historical.files {
            let path = PathBuf::from(&entry.path);
            let destination = target.join(&path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).expect("historical projection parent");
            }
            fs::write(&destination, &entry.content).expect("historical projection");
            frozen_files.insert(path, entry.content);
        }
        let mut active: ActiveSkills = serde_yaml::from_slice(
            &fs::read(target.join(".hive/config/active-skills.yml"))
                .expect("current active Skill ledger"),
        )
        .expect("active Skill ledger");
        active.skills.retain(|skill| {
            frozen_files.contains_key(&PathBuf::from(format!(
                ".agents/skills/{}/SKILL.md",
                skill.name
            )))
        });
        for skill in &mut active.skills {
            let path = PathBuf::from(format!(".agents/skills/{}/SKILL.md", skill.name));
            skill.content_digest = sha256_digest(
                frozen_files
                    .get(&path)
                    .expect("historical Skill must have projection bytes"),
            );
        }
        fs::write(
            target.join(".hive/config/active-skills.yml"),
            serde_yaml::to_string(&active).expect("historical active Skill ledger"),
        )
        .expect("historical active Skill ledger");
        fs::write(
            target.join(".hive/config/project-base.json"),
            render_project_base(&frozen_files).expect("historical project-base ledger"),
        )
        .expect("historical project-base ledger");
        let before = snapshot_tree(&target);

        let error = execute_release_update_for_target_in(
            &SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::Apply,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            },
            &target_dir,
            env!("CARGO_PKG_VERSION"),
            "0.8.0",
        )
        .expect_err("unconnected 0.7 fixture must not publish an operational 0.8 harness");

        assert!(matches!(error, RenderError::Unsupported(_)));
        assert!(error.to_string().contains("validated user setup"));
        assert_eq!(
            snapshot_tree(&target),
            before,
            "rejected historical update must leave the complete install tree unchanged"
        );
    }

    #[test]
    fn custom_project_cannot_enable_wiki_when_global_wiki_is_disabled() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let answers_path = target.join("custom-answers.yml");
        let answers = fs::read_to_string(fixture("answers-base.yml"))
            .expect("base answers")
            .replace(
                "setup_mode: expedited\n",
                "setup_mode: custom\ninterface_language: ko\nwiki:\n  enabled: true\n  language: both\npersona:\n  id: friendly\nskills:\n  mode: individual\n  selected:\n    - setup-harness\n",
            );
        fs::write(&answers_path, answers).expect("custom answers");

        let error = execute_setup(&SetupRequest {
            target: &target,
            answers: &answers_path,
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::DryRun,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(GlobalProjectPreferences {
                interface_language: "en".to_owned(),
                wiki_enabled: false,
                wiki_language: "both".to_owned(),
                persona_id: "balanced".to_owned(),
                persona_custom_description: None,
                selected_project_skills: vec!["setup-harness".to_owned()],
                usage_guard_enabled: false,
                codexbar_fallback_enabled: false,
                usage_stop_remaining_percent: 20,
            }),
        })
        .expect_err("project Wiki enable must respect the global disable boundary");

        assert_eq!(error.code(), "hive.setup-safety-blocked");
    }

    #[test]
    fn custom_project_selection_resolves_dependency_closure_before_projection() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().canonicalize().expect("canonical target");
        let answers_path = target.join("custom-answers.yml");
        let answers = fs::read_to_string(fixture("answers-base.yml"))
            .expect("base answers")
            .replace(
                "setup_mode: expedited\n",
                "setup_mode: custom\ninterface_language: ko\nwiki:\n  enabled: true\n  language: both\npersona:\n  id: friendly\nskills:\n  mode: individual\n  selected:\n    - hive-knowledge-promote\n",
            );
        fs::write(&answers_path, answers).expect("custom answers");

        let outcome = execute_setup(&SetupRequest {
            target: &target,
            answers: &answers_path,
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: Some(GlobalProjectPreferences {
                interface_language: "en".to_owned(),
                wiki_enabled: true,
                wiki_language: "both".to_owned(),
                persona_id: "balanced".to_owned(),
                persona_custom_description: None,
                selected_project_skills: vec!["setup-harness".to_owned()],
                usage_guard_enabled: true,
                codexbar_fallback_enabled: false,
                usage_stop_remaining_percent: 20,
            }),
        })
        .expect("custom selection dependency closure should compile");

        let effective = outcome
            .effective_preferences
            .expect("effective preferences should be returned");
        assert_eq!(
            effective.selected_project_skills,
            [
                "hive-knowledge-capture",
                "hive-knowledge-promote",
                "hive-knowledge-query",
            ]
        );
        for skill in [
            "hive-knowledge-capture",
            "hive-knowledge-promote",
            "hive-knowledge-query",
        ] {
            assert!(target
                .join(format!(".agents/skills/{skill}/SKILL.md"))
                .is_file());
        }
        let harness =
            fs::read_to_string(target.join(".hive/config/harness.toml")).expect("harness config");
        assert!(harness.contains(
            "selected_project_skills = [\"hive-knowledge-capture\", \"hive-knowledge-promote\", \"hive-knowledge-query\"]"
        ));
        assert!(harness.contains("codexbar_fallback_enabled = false"));
    }

    #[test]
    fn custom_project_rejects_user_only_setup_hive_selection() {
        let selection = ProjectSkillSelection {
            mode: "individual".to_owned(),
            recommended_suite: None,
            selected: Some(vec!["setup-hive".to_owned()]),
        };

        let error = resolve_project_skill_selection(&selection)
            .expect_err("setup-hive must remain user-scope only");

        assert_eq!(error.code(), "hive.setup-invalid-input");
        assert!(error.to_string().contains("setup-hive is user-scope only"));
    }

    #[test]
    fn release_update_entrypoint_rejects_an_unbound_historical_source() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        apply_fixture(
            temporary.path(),
            "answers-base.yml",
            "capabilities-codex-omx.json",
        );
        let target = temporary.path().canonicalize().expect("canonical target");
        let target_dir = open_target_capability(&target).expect("target capability");

        let error = execute_release_update_in(
            &SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::DryRun,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            },
            &target_dir,
            "0.6.0",
        )
        .expect_err("caller must not select an unbound relaxed historical source");

        assert!(matches!(error, RenderError::Verification(_)));
        assert!(error
            .to_string()
            .contains("does not match the installed harness"));
    }

    #[test]
    fn frozen_0_7_full_registry_matches_release_and_ignores_current_candidate_mutation() {
        for (answers, capabilities) in [
            ("answers-base.yml", "capabilities-codex-omx.json"),
            ("answers-claude.yml", "capabilities-claude-omc.json"),
        ] {
            let temporary = tempfile::tempdir().expect("temporary directory");
            apply_fixture(temporary.path(), answers, capabilities);
            let target = open_target_capability(temporary.path()).expect("target capability");
            let mut current = project_upgrade_candidate_in(&target).expect("current candidate");
            let historical =
                historical_project_upgrade_candidate_in(&target, "0.7.0").expect("0.7 registry");
            let historical_files = historical
                .files
                .iter()
                .map(|entry| (entry.path.clone(), entry.content.clone()))
                .collect::<BTreeMap<_, _>>();
            let added_since_0_7 = current
                .files
                .keys()
                .filter(|path| !historical_files.contains_key(*path))
                .cloned()
                .collect::<Vec<_>>();
            let mut expected_added = vec![".agents/skills/auto-setup-harness/SKILL.md".to_owned()];
            for name in [
                "auto-setup-harness",
                "hive-judge-package",
                "hive-knowledge-capture",
                "hive-knowledge-maintenance",
                "hive-knowledge-promote",
                "hive-knowledge-query",
                "hive-migrate",
                "hive-project-upgrade",
                "hive-prompt-refine",
                "hive-role-handoff",
                "hive-run-checkpoint",
                "hive-run-resume",
                "hive-simple-question",
                "hive-update",
                "hive-usage-guard",
                "setup-harness",
            ] {
                expected_added.push(format!(".agents/skills/{name}/agents/openai.yaml"));
            }
            if capabilities == "capabilities-claude-omc.json" {
                expected_added.push(".claude/skills/auto-setup-harness/SKILL.md".to_owned());
            }
            expected_added.sort();
            assert_eq!(added_since_0_7, expected_added);
            let changed_since_0_7 = [
                "AGENTS.md",
                "/directives/01-project-knowledge.md",
                "/hive-knowledge-capture/SKILL.md",
                "/hive-knowledge-maintenance/SKILL.md",
                "/hive-knowledge-query/SKILL.md",
                "/hive-prompt-refine/SKILL.md",
                "/hive-simple-question/SKILL.md",
                "/hive-usage-guard/SKILL.md",
                "/setup-harness/SKILL.md",
            ];
            for (path, historical_content) in &historical_files {
                if changed_since_0_7
                    .iter()
                    .any(|suffix| path.ends_with(suffix))
                {
                    assert_ne!(historical_content, &current.files[path]);
                } else {
                    assert_eq!(historical_content, &current.files[path], "{path}");
                }
            }

            current
                .files
                .get_mut(".agents/directives/00-project-harness.md")
                .expect("current directive")
                .extend_from_slice(b"simulated future template change\n");
            let rerendered =
                historical_project_upgrade_candidate_in(&target, "0.7.0").expect("frozen rerender");
            assert_eq!(rerendered, historical);
            assert_ne!(
                rerendered
                    .files
                    .iter()
                    .find(|entry| entry.path == ".agents/directives/00-project-harness.md")
                    .expect("historical directive")
                    .content,
                current.files[".agents/directives/00-project-harness.md"]
            );
        }
    }

    #[test]
    fn frozen_0_7_registry_authenticates_approved_path_skill_and_rejects_source_tamper() {
        for (answers_fixture, capabilities_fixture, expected_paths) in [
            (
                "answers-base.yml",
                "capabilities-codex-omx.json",
                vec![".agents/skills/local-inspect/SKILL.md"],
            ),
            (
                "answers-claude.yml",
                "capabilities-claude-omc.json",
                vec![
                    ".agents/skills/local-inspect/SKILL.md",
                    ".claude/skills/local-inspect/SKILL.md",
                ],
            ),
        ] {
            let temporary = tempfile::tempdir().expect("temporary directory");
            let source = temporary
                .path()
                .join("vendor-skills/local-inspect/SKILL.md");
            fs::create_dir_all(source.parent().expect("source parent")).expect("source parent");
            let skill_bytes = optional_skill_bytes("v1");
            fs::write(&source, &skill_bytes).expect("optional source");
            let (mut answers, _) =
                load_answers(&fixture(answers_fixture)).expect("answers should load");
            answers.approved_optional_skills = vec![signed_local_skill(&skill_bytes)];
            let resolution =
                load_resolution(&fixture(capabilities_fixture)).expect("resolution should load");
            apply_optional_skill_fixture(temporary.path(), &answers, &resolution);
            let target = open_target_capability(temporary.path()).expect("target capability");

            let historical =
                historical_project_upgrade_candidate_in(&target, "0.7.0").expect("0.7 registry");
            for path in expected_paths {
                let entry = historical
                    .files
                    .iter()
                    .find(|entry| entry.path == path)
                    .expect("approved optional projection");
                assert_eq!(entry.kind, "skill");
                assert_eq!(entry.content, skill_bytes);
                assert_eq!(entry.content_digest, sha256_digest(&skill_bytes));
            }

            fs::write(&source, optional_skill_bytes("tampered")).expect("tampered source");
            assert!(matches!(
                historical_project_upgrade_candidate_in(&target, "0.7.0"),
                Err(RenderError::Verification(_))
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn pinned_setup_entrypoint_does_not_reopen_replaced_ambient_target() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let parent = temporary.path().canonicalize().expect("canonical parent");
        let target = parent.join("consumer");
        let displaced = parent.join("consumer-displaced");
        fs::create_dir(&target).expect("target");
        let target_dir = open_target_capability(&target).expect("target capability");
        fs::rename(&target, &displaced).expect("displace target");
        fs::create_dir(&target).expect("replacement target");
        fs::write(target.join("sentinel"), b"replacement").expect("replacement sentinel");

        let outcome = execute_setup_in(
            &SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::DryRun,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            },
            &target_dir,
        )
        .expect("pinned dry-run");

        assert!(!outcome.changed_paths.is_empty());
        assert_eq!(
            fs::read(target.join("sentinel")).expect("replacement sentinel"),
            b"replacement"
        );
        assert!(fs::read_dir(&displaced)
            .expect("displaced target")
            .next()
            .is_none());
    }

    #[cfg(windows)]
    #[test]
    fn pinned_target_capability_blocks_ambient_replacement_while_open() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let parent = temporary.path().canonicalize().expect("canonical parent");
        let target = parent.join("consumer");
        let displaced = parent.join("consumer-displaced");
        fs::create_dir(&target).expect("target");
        let target_dir = open_target_capability(&target).expect("target capability");

        fs::rename(&target, &displaced)
            .expect_err("open Windows target capability should block replacement");
        assert!(target.is_dir());
        assert!(!displaced.exists());

        drop(target_dir);
        fs::rename(&target, &displaced).expect("rename after target capability release");
    }

    #[cfg(unix)]
    #[test]
    fn pinned_setup_entrypoint_does_not_use_the_ambient_parent_for_staging() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let root = temporary.path().canonicalize().expect("canonical root");
        let ambient_parent = root.join("ambient");
        let displaced_parent = root.join("ambient-displaced");
        let target = ambient_parent.join("consumer");
        fs::create_dir_all(&target).expect("target");
        let target_dir = open_target_capability(&target).expect("target capability");
        fs::rename(&ambient_parent, &displaced_parent).expect("displace ambient parent");
        fs::write(&ambient_parent, b"replacement-parent").expect("replacement parent");

        let outcome = execute_setup_in(
            &SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::DryRun,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            },
            &target_dir,
        )
        .expect("pinned dry-run");

        assert!(!outcome.changed_paths.is_empty());
        assert_eq!(
            fs::read(&ambient_parent).expect("replacement parent"),
            b"replacement-parent"
        );
        assert!(fs::read_dir(displaced_parent.join("consumer"))
            .expect("displaced target")
            .next()
            .is_none());
    }

    #[test]
    fn validates_core_scalar_formats() {
        assert!(valid_role_id("reviewer"));
        assert!(!valid_role_id("Reviewer"));
        assert!(valid_digest(&format!("sha256:{}", "a".repeat(64))));
        assert!(valid_timestamp("2026-07-23T00:00:00Z"));
    }

    #[test]
    fn marker_merge_preserves_surrounding_bytes() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let path = temporary.path().join("AGENTS.md");
        fs::write(
            &path,
            format!("prefix\r\n{MARKER_START}\nold\n{MARKER_END}\r\nsuffix"),
        )
        .expect("fixture should be written");
        let replacement = format!("{MARKER_START}\nnew\n{MARKER_END}\n");
        let merged = merge_shared_marker(
            temporary.path(),
            std::path::Path::new("AGENTS.md"),
            replacement.as_bytes(),
        )
        .expect("merge should succeed");
        assert!(merged.starts_with(b"prefix\r\n"));
        assert!(merged.ends_with(b"\r\nsuffix"));
        assert!(merged.windows(3).any(|window| window == b"new"));
    }

    #[test]
    fn compiled_marker_exactly_matches_the_product_template() {
        let (answers, _) = load_answers(&fixture("answers-base.yml")).expect("answers should load");
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let global = GlobalProjectPreferences {
            interface_language: "en".to_owned(),
            wiki_enabled: true,
            wiki_language: "both".to_owned(),
            persona_id: "balanced".to_owned(),
            persona_custom_description: None,
            selected_project_skills: vec!["setup-harness".to_owned()],
            usage_guard_enabled: true,
            codexbar_fallback_enabled: true,
            usage_stop_remaining_percent: 20,
        };
        let effective = resolve_effective_project_preferences(&answers, Some(&global))
            .expect("preferences should resolve")
            .expect("global preferences should produce an effective projection");
        let expected = include_str!("../../../harness/template/AGENTS.md.jinja")
            .replace("{{ project_name }}", &answers.project_name)
            .replace("{{ project_kind }}", &answers.project_kind)
            .replace("{{ setup_mode }}", &answers.setup_mode)
            .replace("{{ preference_provenance }}", effective.provenance)
            .replace("{{ interface_language }}", &effective.interface_language)
            .replace(
                "{{ \"enabled\" if wiki_enabled else \"disabled\" }}",
                if effective.wiki_enabled {
                    "enabled"
                } else {
                    "disabled"
                },
            )
            .replace("{{ wiki_language }}", &effective.wiki_language)
            .replace("{{ persona_id }}", &effective.persona_id)
            .replace("{{ primary_host }}", &answers.primary_host)
            .replace(
                "{{ capability_resolution.resolved_owner }}",
                &resolution.resolved_owner,
            )
            .replace(
                "{{ capability_resolution.evidence_digest }}",
                &resolution.evidence_digest,
            );

        assert_eq!(
            render_agents_marker(&answers, &resolution, Some(&effective)),
            expected
        );
        assert!(expected.contains("agent-reviewed task-fact autocapture"));
        assert!(expected.contains("originating request"));
        assert!(expected.contains("raw transcript"));
    }

    #[test]
    fn shared_marker_foreign_digest_ignores_only_the_exact_hive_block() {
        let first = format!("prefix\r\n{MARKER_START}\nold\n{MARKER_END}\r\nsuffix");
        let second = format!("prefix\r\n{MARKER_START}\nnew\n{MARKER_END}\r\nsuffix");
        assert_eq!(
            shared_marker_foreign_digest(first.as_bytes()).expect("first"),
            shared_marker_foreign_digest(second.as_bytes()).expect("second")
        );
        let changed = format!("different\r\n{MARKER_START}\nnew\n{MARKER_END}\r\nsuffix");
        assert_ne!(
            shared_marker_foreign_digest(first.as_bytes()).expect("first"),
            shared_marker_foreign_digest(changed.as_bytes()).expect("changed")
        );
    }

    #[test]
    fn nested_markers_are_a_conflict() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let path = temporary.path().join("AGENTS.md");
        fs::write(
            &path,
            format!("{MARKER_START}\n{MARKER_START}\n{MARKER_END}"),
        )
        .expect("fixture should be written");
        assert!(merge_shared_marker(
            temporary.path(),
            std::path::Path::new("AGENTS.md"),
            b"replacement"
        )
        .is_err());
    }

    #[test]
    #[allow(clippy::unicode_not_nfc)]
    fn rfc_8785_unicode_and_number_vectors_use_the_reviewed_library() {
        let input: JsonValue = serde_json::from_str(
            r#"{
              "numbers":[333333333.33333329,1E30,4.50,2e-3,0.000000000000000000000000001],
              "string":"€$\u000f\nA'B\"\\\\\"/",
              "literals":[null,true,false]
            }"#,
        )
        .expect("official RFC 8785 sample is valid JSON");
        let canonical =
            serde_json_canonicalizer::to_string(&input).expect("JCS serialization should work");
        assert_eq!(
            canonical,
            "{\"literals\":[null,true,false],\"numbers\":[333333333.3333333,1e+30,4.5,0.002,1e-27],\"string\":\"€$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\"}"
        );

        let unicode: JsonValue = serde_json::from_str(
            r#"{"€":"euro","\r":"cr","דּ":"hebrew","1":"one","😀":"emoji","\u0080":"control","ö":"latin"}"#,
        )
        .expect("Unicode ordering sample is valid JSON");
        let canonical =
            serde_json_canonicalizer::to_string(&unicode).expect("JCS serialization should work");
        assert_eq!(
            canonical,
            "{\"\\r\":\"cr\",\"1\":\"one\",\"\":\"control\",\"ö\":\"latin\",\"€\":\"euro\",\"😀\":\"emoji\",\"דּ\":\"hebrew\"}"
        );
    }

    fn signed_skill() -> SkillApproval {
        let mut approval = SkillApproval {
            consent_version: 1,
            name: "fixture-readonly".to_owned(),
            source: "immutable:fixture".to_owned(),
            revision: "v1".to_owned(),
            content_digest: format!("sha256:{}", "3".repeat(64)),
            requested_capabilities: vec!["filesystem-read".to_owned(), "network".to_owned()],
            approved_capabilities: vec!["filesystem-read".to_owned()],
            approved_at: "2026-07-23T00:00:00Z".to_owned(),
            consent_digest: String::new(),
        };
        approval.consent_digest =
            calculate_consent_digest(&approval).expect("consent should canonicalize");
        approval
    }

    fn optional_skill_bytes(version: &str) -> Vec<u8> {
        format!(
            "---\nname: local-inspect\ndescription: Inspect local fixture state without network access.\n---\n\n# Local Inspect\n\nVersion {version}.\n"
        )
        .into_bytes()
    }

    fn signed_local_skill(bytes: &[u8]) -> SkillApproval {
        let digest = sha256_digest(bytes);
        let mut approval = SkillApproval {
            consent_version: 1,
            name: "local-inspect".to_owned(),
            source: "path:vendor-skills/local-inspect/SKILL.md".to_owned(),
            revision: digest.clone(),
            content_digest: digest,
            requested_capabilities: vec!["filesystem-read".to_owned()],
            approved_capabilities: vec!["filesystem-read".to_owned()],
            approved_at: "2026-07-23T00:00:00Z".to_owned(),
            consent_digest: String::new(),
        };
        approval.consent_digest =
            calculate_consent_digest(&approval).expect("local Skill consent should canonicalize");
        approval
    }

    fn apply_optional_skill_fixture(
        target: &Path,
        answers: &SetupAnswers,
        resolution: &CapabilityResolution,
    ) {
        let target_dir = open_target_capability(target).expect("target capability should open");
        let planned = render_tree(&target_dir, answers, resolution, &BTreeSet::new())
            .expect("optional Skill tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, answers)
            .expect("optional Skill projection preflight should succeed");
        activate_staged_impl(
            target,
            &target_dir,
            &planned,
            &transition.deletions,
            answers,
            resolution,
            &transition.expected_before,
            None,
            None,
            None,
            None,
        )
        .expect("optional Skill fixture should activate");
    }

    fn projection_recovery_bytes(parent: &Path) -> Vec<Vec<u8>> {
        let mut recovered = Vec::new();
        for entry in fs::read_dir(parent).expect("projection parent should be readable") {
            let entry = entry.expect("projection recovery entry should be readable");
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.path().is_dir() && name.starts_with(".aigent-hive-claim-") {
                for recovery_name in ["claimed-SKILL.md", "prior-SKILL.md"] {
                    let recovery = entry.path().join(recovery_name);
                    if recovery.is_file() {
                        recovered.push(
                            fs::read(recovery).expect("projection recovery should be readable"),
                        );
                    }
                }
            } else if entry.path().is_file() && name.starts_with(".aigent-hive-projection-publish")
            {
                recovered.push(
                    fs::read(entry.path()).expect("projection publish temp should be readable"),
                );
            }
        }
        recovered
    }

    fn absent_resolution() -> CapabilityResolution {
        CapabilityResolution {
            schema_version: 1,
            host: "codex".to_owned(),
            host_version: "fixture".to_owned(),
            surface: "cli".to_owned(),
            detection: "absent".to_owned(),
            external_runtime: None,
            resolved_owner: "host-native".to_owned(),
            capabilities: BTreeMap::new(),
            hook_events: None,
            evidence_digest: format!("sha256:{}", "4".repeat(64)),
            evidence: Vec::new(),
        }
    }

    fn signed_hook() -> HookApproval {
        let mut approval = HookApproval {
            consent_version: 1,
            capability: "checkpoint-reminder".to_owned(),
            event: "Stop".to_owned(),
            path: ".hive/hooks/checkpoint-reminder".to_owned(),
            command: format!(
                "hive hook --capability checkpoint-reminder --event Stop \
                 --capabilities {FRESH_CAPABILITY_RESOLUTION_PATH} --output json"
            ),
            content_digest: String::new(),
            approved_at: "2026-07-23T00:00:00Z".to_owned(),
            consent_digest: String::new(),
        };
        approval.content_digest = sha256_digest(
            &hook_descriptor_bytes(&approval).expect("descriptor should canonicalize"),
        );
        approval.consent_digest =
            calculate_consent_digest(&approval).expect("consent should canonicalize");
        approval
    }

    #[test]
    fn every_skill_consent_field_and_order_constraint_is_bound() {
        let valid = signed_skill();
        assert!(validate_skill_approvals(std::slice::from_ref(&valid)).is_ok());

        let mut mutations = Vec::new();
        let mut changed = valid.clone();
        changed.consent_version = 2;
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.name.push_str("-changed");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.source.push_str("-changed");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.revision.push_str("-changed");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.content_digest = format!("sha256:{}", "4".repeat(64));
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.requested_capabilities.reverse();
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.approved_capabilities = vec!["shell".to_owned()];
        changed.consent_digest =
            calculate_consent_digest(&changed).expect("mutated consent should canonicalize");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.approved_at = "2026-02-30T00:00:00Z".to_owned();
        changed.consent_digest =
            calculate_consent_digest(&changed).expect("mutated consent should canonicalize");
        mutations.push(changed);
        let mut changed = valid;
        changed.consent_digest = format!("sha256:{}", "5".repeat(64));
        mutations.push(changed);

        for mutation in mutations {
            assert!(validate_skill_approvals(&[mutation]).is_err());
        }
    }

    #[test]
    fn every_hook_consent_field_and_descriptor_byte_is_bound() {
        let valid = signed_hook();
        let resolution = absent_resolution();
        assert!(validate_hook_approvals(std::slice::from_ref(&valid), &resolution).is_ok());

        let mut mutations = Vec::new();
        let mut changed = valid.clone();
        changed.consent_version = 2;
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.capability = "protect-hive-owned-state".to_owned();
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.event = "PreCompact".to_owned();
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.path.push_str("-changed");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.command.push_str(" --changed");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.content_digest = format!("sha256:{}", "6".repeat(64));
        changed.consent_digest =
            calculate_consent_digest(&changed).expect("mutated consent should canonicalize");
        mutations.push(changed);
        let mut changed = valid.clone();
        changed.approved_at = "2025-02-29T00:00:00Z".to_owned();
        changed.consent_digest =
            calculate_consent_digest(&changed).expect("mutated consent should canonicalize");
        mutations.push(changed);
        let mut changed = valid;
        changed.consent_digest = format!("sha256:{}", "7".repeat(64));
        mutations.push(changed);

        for mutation in mutations {
            assert!(validate_hook_approvals(&[mutation], &resolution).is_err());
        }
    }

    #[test]
    fn resolver_derives_owner_from_positive_evidence_and_rejects_contradictions() {
        let mut resolution = absent_resolution();
        resolution.detection = "available".to_owned();
        resolution.external_runtime = Some("omx".to_owned());
        resolution.resolved_owner = "omx".to_owned();
        resolution.evidence = vec![CapabilityEvidence {
            source: "host-catalog".to_owned(),
            locator: "fixture:catalog".to_owned(),
            outcome: "compatible".to_owned(),
            digest: format!("sha256:{}", "8".repeat(64)),
        }];
        assert_eq!(
            derive_resolution(&resolution).expect("catalog evidence should be sufficient"),
            ("available", "omx", Some("omx"))
        );

        resolution.evidence.push(CapabilityEvidence {
            source: "public-executable".to_owned(),
            locator: "fixture:not-found".to_owned(),
            outcome: "absent".to_owned(),
            digest: format!("sha256:{}", "9".repeat(64)),
        });
        assert!(derive_resolution(&resolution).is_err());

        resolution.host = "antigravity".to_owned();
        resolution.external_runtime = None;
        assert!(derive_resolution(&resolution).is_err());
    }

    #[test]
    fn role_materialization_matches_the_versioned_known_answer() {
        let seed = RoleSeed {
            role_id: "reviewer".to_owned(),
            display_name: "Hostile Reviewer".to_owned(),
            responsibilities: vec!["Verify acceptance criteria independently".to_owned()],
            non_responsibilities: vec!["Implement the artifact under review".to_owned()],
            context_paths: vec!["docs/".to_owned()],
            allowed_capabilities: vec!["filesystem-read".to_owned(), "shell".to_owned()],
            write_scope: vec![".hive/runs/".to_owned()],
            verification_duties: vec!["Attach reproducible evidence to every finding".to_owned()],
        };
        let profile = RoleProfile::from_seed(&seed);
        let body = "# Hostile Reviewer\n\n## Current assignment\n\n_Unassigned._\n\n## Handoff\n\n_No handoff yet._\n";
        let rendered = encode_role(&profile, body).expect("role should materialize");
        assert_eq!(
            rendered,
            include_bytes!("../../../tests/fixtures/expected/reviewer-role.md")
        );
    }

    #[test]
    fn role_parser_accepts_crlf_and_preserves_body_line_endings() {
        let bytes = include_bytes!("../../../tests/fixtures/expected/reviewer-role.md");
        let crlf = String::from_utf8(bytes.to_vec())
            .expect("fixture should be UTF-8")
            .replace('\n', "\r\n");
        let (_, body) = parse_role(crlf.as_bytes()).expect("CRLF role should parse");
        assert!(body.contains("\r\n"));
        assert!(body.ends_with("\r\n"));
    }

    #[test]
    fn canonical_protected_seed_bytes_survive_repeated_setup() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        apply_fixture(
            temporary.path(),
            "answers-base.yml",
            "capabilities-codex-omx.json",
        );
        let wiki = temporary.path().join(".hive/knowledge/Wiki/index.md");
        let suppression = temporary.path().join(".hive/knowledge/suppression.yml");
        fs::write(&wiki, b"user-maintained wiki bytes\n").expect("Wiki should be editable");
        fs::write(
            &suppression,
            b"schema_version: 1\nentries:\n  - fingerprint: fixture\n",
        )
        .expect("suppression should be editable");

        apply_fixture(
            temporary.path(),
            "answers-base.yml",
            "capabilities-codex-omx.json",
        );

        assert_eq!(
            fs::read(wiki).expect("Wiki should remain"),
            b"user-maintained wiki bytes\n"
        );
        assert_eq!(
            fs::read(suppression).expect("suppression should remain"),
            b"schema_version: 1\nentries:\n  - fingerprint: fixture\n"
        );
    }

    #[test]
    fn projected_skill_tamper_blocks_reconfigure_without_overwrite() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let projected = target.join(".agents/skills/hive-simple-question/SKILL.md");
        fs::write(&projected, b"user collision bytes\x00\xff\n")
            .expect("projected fixture should be tampered");

        let error = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect_err("tampered projection must block reconfigure");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(projected).expect("tampered file should remain"),
            b"user collision bytes\x00\xff\n"
        );
    }

    #[test]
    fn foreign_directive_collision_blocks_initial_setup_without_overwrite() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let directive = target.join(".agents/directives/00-project-harness.md");
        fs::create_dir_all(directive.parent().expect("directive should have a parent"))
            .expect("directive parent should be created");
        fs::write(&directive, b"foreign directive bytes\x00\xff\n")
            .expect("foreign directive should exist");

        let error = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect_err("foreign directive must block initial setup");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(directive).expect("foreign directive should remain"),
            b"foreign directive bytes\x00\xff\n"
        );
    }

    #[test]
    fn setup_preserves_foreign_directive_sibling_bytes() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let foreign = target.join(".agents/directives/user-owned.md");
        fs::create_dir_all(foreign.parent().expect("directive should have a parent"))
            .expect("directive parent should be created");
        fs::write(&foreign, b"foreign sibling bytes\x00\xff\n")
            .expect("foreign directive should exist");

        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");

        assert_eq!(
            fs::read(foreign).expect("foreign directive should remain"),
            b"foreign sibling bytes\x00\xff\n"
        );
    }

    #[test]
    fn projected_directive_tamper_blocks_reconfigure_without_overwrite() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let directive = target.join(".agents/directives/00-project-harness.md");
        fs::write(&directive, b"user directive bytes\x00\xff\n")
            .expect("projected directive should be tampered");

        let error = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Apply,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect_err("tampered directive must block reconfigure");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(directive).expect("tampered directive should remain"),
            b"user directive bytes\x00\xff\n"
        );
    }

    #[test]
    fn wildcard_projection_shape_is_not_ownership_proof() {
        let desired = ValidatedProjectionOwnership::default();
        for path in [
            ".agents/skills/arbitrary-safe-name/SKILL.md",
            ".claude/skills/arbitrary-safe-name/SKILL.md",
        ] {
            let path = PathBuf::from(path);
            assert!(
                validate_project_relative(&path).is_err(),
                "generic validation must reject host discovery paths"
            );
            let paths = [path.clone()];
            let error = validate_owned_paths(paths.iter(), &desired, None)
                .expect_err("manifest wildcard shape alone must not prove Hive ownership");
            assert_eq!(error.code(), "hive.setup-safety-blocked");
            assert!(error
                .to_string()
                .contains("lacks exact Hive ownership proof"));
        }
    }

    #[test]
    fn activation_rejects_foreign_projection_created_after_preflight() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let (answers, _) = load_answers(&fixture("answers-base.yml")).expect("answers should load");
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("first-install projection preflight should prove absence");
        let projected = target.join(".agents/skills/hive-simple-question/SKILL.md");
        let create_foreign = || {
            fs::create_dir_all(projected.parent().expect("projection should have a parent"))
                .expect("foreign projection parent should be created");
            fs::write(&projected, b"foreign race bytes\x00\xff\n")
                .expect("foreign projection should win the race");
        };

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            None,
            Some(&create_foreign),
            None,
            None,
        )
        .expect_err("post-preflight foreign projection must block activation");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(projected).expect("foreign race bytes should remain"),
            b"foreign race bytes\x00\xff\n"
        );
    }

    #[test]
    fn activation_rejects_foreign_directive_created_after_preflight() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let (answers, _) = load_answers(&fixture("answers-base.yml")).expect("answers should load");
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("first-install projection preflight should prove absence");
        let directive = target.join(".agents/directives/00-project-harness.md");
        let create_foreign = || {
            fs::create_dir_all(directive.parent().expect("directive should have a parent"))
                .expect("foreign directive parent should be created");
            fs::write(&directive, b"foreign directive race bytes\x00\xff\n")
                .expect("foreign directive should win the race");
        };

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            None,
            Some(&create_foreign),
            None,
            None,
        )
        .expect_err("post-preflight foreign directive must block activation");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(directive).expect("foreign directive race bytes should remain"),
            b"foreign directive race bytes\x00\xff\n"
        );
    }

    #[test]
    fn activation_rejects_projection_tampered_after_preflight() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.project_name = "projection-race-project".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("changed tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("installed projection ownership should verify");
        let projected = target.join(".agents/skills/hive-simple-question/SKILL.md");
        let tamper_projection = || {
            fs::write(&projected, b"tampered race bytes\x00\xff\n")
                .expect("projected Skill should be tampered after preflight");
        };

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            None,
            Some(&tamper_projection),
            None,
            None,
        )
        .expect_err("post-preflight projection tamper must block activation");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(projected).expect("tampered race bytes should remain"),
            b"tampered race bytes\x00\xff\n"
        );
    }

    #[test]
    fn exact_projection_replace_claim_preserves_a_racing_foreign_replacement() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let relative = Path::new(".agents/skills/hive-claim-race/SKILL.md");
        let projected = target.join(relative);
        let raced_original = projected
            .parent()
            .expect("projection should have a parent")
            .join("raced-original");
        fs::create_dir_all(projected.parent().expect("projection should have a parent"))
            .expect("projection parent should exist");
        fs::write(&projected, b"prior exact bytes\n").expect("prior projection should exist");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let race = |claimed: &Path| {
            assert_eq!(claimed, relative);
            fs::rename(&projected, &raced_original)
                .expect("racer should atomically retain the original bytes");
            fs::write(&projected, b"foreign replacement bytes\x00\xff\n")
                .expect("racer should publish foreign bytes");
        };

        let error = mutate_exact_projection_claimed(
            &target_dir,
            relative,
            b"prior exact bytes\n",
            ExactProjectionMutation::Replace(b"desired Hive bytes\n"),
            Some(&race),
            None,
        )
        .expect_err("racing replacement must fail the claimed-byte check");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert!(error.to_string().contains("without overwrite"));
        assert_eq!(
            fs::read(projected).expect("foreign replacement should remain"),
            b"foreign replacement bytes\x00\xff\n"
        );
        assert_eq!(
            fs::read(raced_original).expect("prior exact bytes should remain recoverable"),
            b"prior exact bytes\n"
        );
    }

    #[test]
    fn exact_projection_delete_claim_preserves_a_racing_foreign_replacement() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let relative = Path::new(".claude/skills/hive-claim-race/SKILL.md");
        let projected = target.join(relative);
        let raced_original = projected
            .parent()
            .expect("projection should have a parent")
            .join("raced-original");
        fs::create_dir_all(projected.parent().expect("projection should have a parent"))
            .expect("projection parent should exist");
        fs::write(&projected, b"prior exact bytes\n").expect("prior projection should exist");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let race = |claimed: &Path| {
            assert_eq!(claimed, relative);
            fs::rename(&projected, &raced_original)
                .expect("racer should atomically retain the original bytes");
            fs::write(&projected, b"foreign deletion-race bytes\x00\xff\n")
                .expect("racer should publish foreign bytes");
        };

        let error = mutate_exact_projection_claimed(
            &target_dir,
            relative,
            b"prior exact bytes\n",
            ExactProjectionMutation::Delete,
            Some(&race),
            None,
        )
        .expect_err("racing replacement must not be deleted");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert!(error.to_string().contains("without overwrite"));
        assert_eq!(
            fs::read(projected).expect("foreign deletion-race bytes should remain"),
            b"foreign deletion-race bytes\x00\xff\n"
        );
        assert_eq!(
            fs::read(raced_original).expect("prior exact bytes should remain recoverable"),
            b"prior exact bytes\n"
        );
    }

    #[test]
    fn exact_projection_claim_publishes_replacement_exclusively() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let relative = Path::new(".agents/skills/hive-claim-success/SKILL.md");
        let projected = target.join(relative);
        fs::create_dir_all(projected.parent().expect("projection should have a parent"))
            .expect("projection parent should exist");
        fs::write(&projected, b"prior exact bytes\n").expect("prior projection should exist");
        let target_dir = open_target_capability(&target).expect("target capability should open");

        mutate_exact_projection_claimed(
            &target_dir,
            relative,
            b"prior exact bytes\n",
            ExactProjectionMutation::Replace(b"desired Hive bytes\n"),
            None,
            None,
        )
        .expect("exact claimed replacement should publish");

        assert_eq!(
            fs::read(&projected).expect("replacement should exist"),
            b"desired Hive bytes\n"
        );
        let children = fs::read_dir(projected.parent().expect("projection should have a parent"))
            .expect("projection parent should be readable")
            .map(|entry| {
                entry
                    .expect("projection child should be readable")
                    .file_name()
            })
            .collect::<Vec<_>>();
        assert!(
            children
                .iter()
                .all(|name| !name.to_string_lossy().starts_with(".aigent-hive-")),
            "successful claim must leave no private recovery artifact"
        );
    }

    #[test]
    fn replacement_cleanup_failure_rolls_back_live_projection_without_racer() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let source = target.join("vendor-skills/local-inspect/SKILL.md");
        fs::create_dir_all(
            source
                .parent()
                .expect("optional source should have a parent"),
        )
        .expect("optional source parent should exist");
        let v1 = optional_skill_bytes("v1");
        fs::write(&source, &v1).expect("v1 optional source should exist");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.approved_optional_skills = vec![signed_local_skill(&v1)];
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        apply_optional_skill_fixture(&target, &answers, &resolution);

        let v2 = optional_skill_bytes("v2");
        fs::write(&source, &v2).expect("v2 optional source should replace v1");
        answers.approved_optional_skills = vec![signed_local_skill(&v2)];
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("v2 optional Skill tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("v1 projection ownership should verify");
        let relative = Path::new(".agents/skills/local-inspect/SKILL.md");
        let projected = target.join(relative);

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &transition.deletions,
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: usize::MAX,
                fail_rollback: false,
                projection_cleanup: Some(ProjectionCleanupFault::Replacement),
            }),
            None,
            None,
            None,
        )
        .expect_err("post-publication cleanup failure must trigger rollback");

        assert_eq!(error.code(), "hive.activation-rollback-failed");
        assert!(error
            .to_string()
            .contains("live paths were rolled back but projection cleanup evidence remains"));
        assert_eq!(
            fs::read(&projected).expect("rollback should restore the live v1 projection"),
            v1,
            "the applied replacement must be tracked before cleanup failure is reported"
        );
    }

    #[test]
    fn deletion_cleanup_failure_rolls_back_live_projection_without_racer() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let source = target.join("vendor-skills/local-inspect/SKILL.md");
        fs::create_dir_all(
            source
                .parent()
                .expect("optional source should have a parent"),
        )
        .expect("optional source parent should exist");
        let v1 = optional_skill_bytes("v1");
        fs::write(&source, &v1).expect("v1 optional source should exist");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.approved_optional_skills = vec![signed_local_skill(&v1)];
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        apply_optional_skill_fixture(&target, &answers, &resolution);

        answers.approved_optional_skills.clear();
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("optional Skill removal tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("v1 projection ownership should verify");
        let relative = Path::new(".agents/skills/local-inspect/SKILL.md");
        assert!(transition.deletions.contains(relative));
        let projected = target.join(relative);

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &transition.deletions,
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: usize::MAX,
                fail_rollback: false,
                projection_cleanup: Some(ProjectionCleanupFault::Deletion),
            }),
            None,
            None,
            None,
        )
        .expect_err("post-claim deletion cleanup failure must trigger rollback");

        assert_eq!(error.code(), "hive.activation-rollback-failed");
        assert!(error
            .to_string()
            .contains("live paths were rolled back but projection cleanup evidence remains"));
        assert_eq!(
            fs::read(&projected).expect("rollback should restore the live v1 projection"),
            v1,
            "the applied deletion must be tracked before cleanup failure is reported"
        );
    }

    #[test]
    fn replacement_cleanup_failure_records_applied_before_safe_rollback() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let source = target.join("vendor-skills/local-inspect/SKILL.md");
        fs::create_dir_all(
            source
                .parent()
                .expect("optional source should have a parent"),
        )
        .expect("optional source parent should exist");
        let v1 = optional_skill_bytes("v1");
        fs::write(&source, &v1).expect("v1 optional source should exist");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.approved_optional_skills = vec![signed_local_skill(&v1)];
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        apply_optional_skill_fixture(&target, &answers, &resolution);

        let v2 = optional_skill_bytes("v2");
        fs::write(&source, &v2).expect("v2 optional source should replace v1");
        answers.approved_optional_skills = vec![signed_local_skill(&v2)];
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("v2 optional Skill tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("v1 projection ownership should verify");
        let relative = Path::new(".agents/skills/local-inspect/SKILL.md");
        let projected = target.join(relative);
        let raced_v2 = projected
            .parent()
            .expect("projection should have a parent")
            .join("raced-v2");
        let inject_foreign_before_rollback = || {
            assert_eq!(
                fs::read(&projected).expect("v2 should be live before rollback"),
                v2
            );
            fs::rename(&projected, &raced_v2).expect("racer should retain the published v2 bytes");
            fs::write(&projected, b"foreign replacement-cleanup bytes\x00\xff\n")
                .expect("foreign bytes should occupy the live path");
        };

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &transition.deletions,
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: usize::MAX,
                fail_rollback: false,
                projection_cleanup: Some(ProjectionCleanupFault::Replacement),
            }),
            None,
            None,
            Some(&inject_foreign_before_rollback),
        )
        .expect_err("post-publication cleanup failure must not report rollback success");

        assert_eq!(error.code(), "hive.activation-rollback-failed");
        assert!(error.to_string().contains("recoverable at"));
        assert_eq!(
            fs::read(&projected).expect("foreign bytes should remain live"),
            b"foreign replacement-cleanup bytes\x00\xff\n"
        );
        assert_eq!(
            fs::read(raced_v2).expect("published v2 bytes should remain recoverable"),
            v2
        );
        assert!(
            projection_recovery_bytes(projected.parent().expect("projection should have a parent"))
                .contains(&v1),
            "prior v1 bytes must remain in an explicit recovery artifact"
        );
    }

    #[test]
    fn deletion_cleanup_failure_records_applied_before_safe_rollback() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let source = target.join("vendor-skills/local-inspect/SKILL.md");
        fs::create_dir_all(
            source
                .parent()
                .expect("optional source should have a parent"),
        )
        .expect("optional source parent should exist");
        let v1 = optional_skill_bytes("v1");
        fs::write(&source, &v1).expect("v1 optional source should exist");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.approved_optional_skills = vec![signed_local_skill(&v1)];
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        apply_optional_skill_fixture(&target, &answers, &resolution);

        answers.approved_optional_skills.clear();
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("optional Skill removal tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("v1 projection ownership should verify");
        let relative = Path::new(".agents/skills/local-inspect/SKILL.md");
        assert!(transition.deletions.contains(relative));
        let projected = target.join(relative);
        let inject_foreign_before_rollback = || {
            assert!(
                !projected.exists(),
                "atomic claim should remove the live path before cleanup fails"
            );
            fs::write(&projected, b"foreign deletion-cleanup bytes\x00\xff\n")
                .expect("foreign bytes should occupy the deleted path");
        };

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &transition.deletions,
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: usize::MAX,
                fail_rollback: false,
                projection_cleanup: Some(ProjectionCleanupFault::Deletion),
            }),
            None,
            None,
            Some(&inject_foreign_before_rollback),
        )
        .expect_err("post-claim deletion cleanup failure must not report rollback success");

        assert_eq!(error.code(), "hive.activation-rollback-failed");
        assert!(error.to_string().contains("recoverable at"));
        assert_eq!(
            fs::read(&projected).expect("foreign bytes should remain live"),
            b"foreign deletion-cleanup bytes\x00\xff\n"
        );
        assert!(
            projection_recovery_bytes(projected.parent().expect("projection should have a parent"))
                .contains(&v1),
            "prior v1 bytes must remain in an explicit recovery artifact"
        );
    }

    #[test]
    fn host_reconfigure_removes_only_exact_proven_projection_files() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(
            &target,
            "answers-claude.yml",
            "capabilities-claude-omc.json",
        );
        let foreign = target.join(".claude/skills/foreign-skill/SKILL.md");
        fs::create_dir_all(foreign.parent().expect("foreign file should have a parent"))
            .expect("foreign directory should be created");
        fs::write(&foreign, b"foreign discovery bytes\x00\xff\n")
            .expect("foreign fixture should exist");

        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.primary_host = "codex".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("Codex resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("Codex projection should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("old Claude projection ownership should verify");
        let deletions = &transition.deletions;

        assert_eq!(deletions.len(), 16);
        assert!(deletions
            .iter()
            .all(|path| path.starts_with(".claude/skills")));
        assert!(!deletions.contains(Path::new(".claude/skills/foreign-skill/SKILL.md")));

        activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            deletions,
            &answers,
            &resolution,
            &transition.expected_before,
            None,
            None,
            None,
            None,
        )
        .expect("host reconfigure should activate");

        assert_eq!(
            fs::read(foreign).expect("foreign file should remain"),
            b"foreign discovery bytes\x00\xff\n"
        );
        assert!(target
            .join(".agents/skills/hive-simple-question/SKILL.md")
            .is_file());
        assert!(!target
            .join(".claude/skills/hive-simple-question/SKILL.md")
            .exists());
    }

    #[test]
    fn rollback_preserves_foreign_projection_after_successful_deletion() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(
            &target,
            "answers-claude.yml",
            "capabilities-claude-omc.json",
        );
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.primary_host = "codex".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("Codex resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("Codex projection should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("old Claude projection ownership should verify");
        let raced_relative = transition
            .deletions
            .iter()
            .next()
            .cloned()
            .expect("host change should delete prior projections");
        let raced_path = target.join(&raced_relative);
        let prior_bytes =
            fs::read(&raced_path).expect("prior projected bytes should exist before activation");
        let inject_foreign_before_rollback = || {
            assert!(
                !raced_path.exists(),
                "the selected projection deletion must succeed before rollback"
            );
            fs::write(&raced_path, b"foreign rollback-race bytes\x00\xff\n")
                .expect("foreign bytes should occupy the deleted path before rollback");
        };

        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &transition.deletions,
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: planned.len() + 1,
                fail_rollback: false,
                projection_cleanup: None,
            }),
            None,
            None,
            Some(&inject_foreign_before_rollback),
        )
        .expect_err("later injected failure must exercise projection-safe rollback");

        assert_eq!(error.code(), "hive.activation-rollback-failed");
        assert!(error.to_string().contains("refused to overwrite"));
        assert!(error.to_string().contains("recoverable at"));
        assert_eq!(
            fs::read(&raced_path).expect("foreign rollback-race bytes should remain"),
            b"foreign rollback-race bytes\x00\xff\n"
        );
        let recovery_files = fs::read_dir(
            raced_path
                .parent()
                .expect("projection should have a parent directory"),
        )
        .expect("projection parent should remain readable")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".aigent-hive-claim-")
                .then(|| entry.path().join("prior-SKILL.md"))
        })
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
        assert_eq!(
            recovery_files.len(),
            1,
            "one exact prior-byte recovery artifact must remain"
        );
        assert_eq!(
            fs::read(&recovery_files[0]).expect("prior recovery should remain readable"),
            prior_bytes
        );
    }

    #[test]
    fn fresh_non_absent_capability_makes_hook_inert_before_target_read() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let fresh = target.join(FRESH_CAPABILITY_RESOLUTION_PATH);
        fs::create_dir_all(fresh.parent().expect("fresh evidence should have a parent"))
            .expect("fresh evidence parent should exist");
        fs::write(
            &fresh,
            fs::read(fixture("capabilities-codex-omx.json"))
                .expect("fresh evidence fixture should be readable"),
        )
        .expect("fresh evidence should be written with a current timestamp");
        let authorization = authorize_hook_with_resolution(
            &target,
            "protect-hive-owned-state",
            "PreToolUse",
            Path::new(FRESH_CAPABILITY_RESOLUTION_PATH),
        )
        .expect("fresh external runtime should make the fallback inert");
        assert_eq!(authorization, HookAuthorization::Inert);
        assert_eq!(
            capability_detection(&fixture("capabilities-codex-omx.json"))
                .expect("legacy capability matrix should remain valid"),
            "available"
        );
        assert_eq!(
            capability_detection(&phase3_fixture("capabilities-codex-enriched.json"))
                .expect("enriched capability matrix should validate"),
            "available"
        );
    }

    #[test]
    fn public_fresh_authorization_signature_requires_a_path() {
        let _: fn(&Path, &str, &str, &Path) -> Result<HookAuthorization, super::RenderError> =
            authorize_hook_with_resolution;
    }

    #[test]
    fn legacy_authorization_without_fresh_evidence_is_always_inert() {
        let authorization = authorize_hook(
            Path::new("missing-consumer-target"),
            "protect-hive-owned-state",
            "PreToolUse",
        )
        .expect("missing fresh evidence should be neutral, not an error");
        assert_eq!(authorization, HookAuthorization::Inert);
    }

    #[test]
    fn installed_host_reports_only_its_expected_external_runtime() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        assert_eq!(
            expected_external_runtime(&target).expect("installed host should validate"),
            Some("omx")
        );
    }

    #[cfg(unix)]
    #[test]
    fn setup_rejects_a_shared_file_symlink_before_reading_its_target() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        let outside = target.join("outside");
        fs::write(&outside, b"foreign bytes").expect("outside fixture should exist");
        symlink(&outside, target.join("AGENTS.md")).expect("symlink should be created");
        let result = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::DryRun,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        });

        assert!(result.is_err());
        assert_eq!(
            fs::read(outside).expect("outside fixture should remain"),
            b"foreign bytes"
        );
    }

    #[cfg(unix)]
    #[test]
    fn render_reads_protected_bytes_from_the_initially_pinned_target() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target_path = temporary.path().join("consumer");
        let pinned_path = temporary.path().join("consumer-pinned");
        fs::create_dir(&target_path).expect("consumer target should exist");
        apply_fixture(
            &target_path,
            "answers-base.yml",
            "capabilities-codex-omx.json",
        );
        let wiki_relative = Path::new(".hive/knowledge/Wiki/index.md");
        fs::write(target_path.join(wiki_relative), b"pinned wiki bytes\n")
            .expect("protected fixture should exist");
        let target_dir =
            open_target_capability(&target_path).expect("target capability should open");
        fs::rename(&target_path, &pinned_path).expect("target should move after pin");
        fs::create_dir(&target_path).expect("replacement ambient target should exist");
        fs::write(target_path.join("sentinel"), b"replacement bytes")
            .expect("replacement sentinel should exist");
        let (answers, _) = load_answers(&fixture("answers-base.yml")).expect("answers should load");
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");

        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("pinned tree should render");

        assert_eq!(planned[wiki_relative], b"pinned wiki bytes\n");
        assert_eq!(
            fs::read(target_path.join("sentinel")).expect("replacement sentinel should remain"),
            b"replacement bytes"
        );
    }

    #[cfg(unix)]
    #[test]
    fn activation_rolls_back_pinned_target_when_ambient_path_is_retargeted() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let root = temporary
            .path()
            .canonicalize()
            .expect("fixture root should have a stable path");
        let target_path = root.join("consumer");
        let pinned_path = root.join("consumer-pinned");
        let outside_path = root.join("outside");
        fs::create_dir(&target_path).expect("consumer target should exist");
        fs::create_dir(&outside_path).expect("outside target should exist");
        fs::write(outside_path.join("sentinel"), b"foreign bytes")
            .expect("outside sentinel should exist");
        apply_fixture(
            &target_path,
            "answers-base.yml",
            "capabilities-codex-omx.json",
        );

        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.project_name = "capability-pinned-project".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir =
            open_target_capability(&target_path).expect("target capability should open");
        let before_harness =
            fs::read(target_path.join(".hive/config/harness.toml")).expect("harness should exist");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("changed tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("installed projection ownership should verify");
        let retarget = || {
            fs::rename(&target_path, &pinned_path).expect("target should move after handle open");
            symlink(&outside_path, &target_path).expect("ambient target should be retargeted");
        };

        activate_staged_impl(
            &target_path,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            None,
            Some(&retarget),
            None,
            None,
        )
        .expect_err("retargeted ambient path must fail post-activation identity");

        assert_eq!(
            fs::read(outside_path.join("sentinel")).expect("outside sentinel should remain"),
            b"foreign bytes"
        );
        assert!(
            !outside_path.join(".hive").exists(),
            "retargeted outside directory must not receive Hive artifacts"
        );
        assert_eq!(
            fs::read(pinned_path.join(".hive/config/harness.toml"))
                .expect("pinned target should be rolled back"),
            before_harness
        );
    }

    #[cfg(unix)]
    #[test]
    fn activation_rejects_an_ancestor_symlink_swapped_after_target_open() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let root = temporary
            .path()
            .canonicalize()
            .expect("fixture root should have a stable path");
        let target_path = root.join("consumer");
        let outside_path = root.join("outside");
        fs::create_dir(&target_path).expect("consumer target should exist");
        fs::create_dir(&outside_path).expect("outside target should exist");
        fs::write(outside_path.join("sentinel"), b"foreign bytes")
            .expect("outside sentinel should exist");
        apply_fixture(
            &target_path,
            "answers-base.yml",
            "capabilities-codex-omx.json",
        );

        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.project_name = "ancestor-race-project".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir =
            open_target_capability(&target_path).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("changed tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("installed projection ownership should verify");
        let pinned_hive = target_path.join(".hive-pinned");
        let swap_ancestor = || {
            fs::rename(target_path.join(".hive"), &pinned_hive)
                .expect("managed ancestor should move after root handle open");
            symlink(&outside_path, target_path.join(".hive"))
                .expect("managed ancestor should be retargeted");
        };

        let error = activate_staged_impl(
            &target_path,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            None,
            Some(&swap_ancestor),
            None,
            None,
        )
        .expect_err("ancestor symlink swap must be rejected");

        assert_eq!(error.code(), "hive.setup-conflict");
        assert_eq!(
            fs::read(outside_path.join("sentinel")).expect("outside sentinel should remain"),
            b"foreign bytes"
        );
        assert!(
            !outside_path.join("config").exists(),
            "symlinked ancestor target must not receive Hive artifacts"
        );
    }

    #[test]
    fn injected_activation_failure_rolls_back_every_applied_file() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let harness = target.join(".hive/config/harness.toml");
        let agents = target.join("AGENTS.md");
        let before_harness = fs::read(&harness).expect("harness should exist");
        let before_agents = fs::read(&agents).expect("AGENTS should exist");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.project_name = "changed-project".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("changed tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("installed projection ownership should verify");
        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: 2,
                fail_rollback: false,
                projection_cleanup: None,
            }),
            None,
            None,
            None,
        )
        .expect_err("injected activation should fail");

        assert_eq!(error.code(), "hive.internal-error");
        assert!(error.to_string().contains("rolled back"));
        assert_eq!(
            fs::read(harness).expect("harness should remain"),
            before_harness
        );
        assert_eq!(
            fs::read(agents).expect("AGENTS should remain"),
            before_agents
        );
    }

    #[test]
    fn rollback_failure_has_a_stable_diagnostic_code() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let (mut answers, _) =
            load_answers(&fixture("answers-base.yml")).expect("answers should load");
        answers.project_name = "changed-project".to_owned();
        let resolution = load_resolution(&fixture("capabilities-codex-omx.json"))
            .expect("resolution should load");
        let target_dir = open_target_capability(&target).expect("target capability should open");
        let planned = render_tree(&target_dir, &answers, &resolution, &BTreeSet::new())
            .expect("changed tree should render");
        let transition = prepare_projection_transition(&target_dir, &planned, &answers)
            .expect("installed projection ownership should verify");
        let error = activate_staged_impl(
            &target,
            &target_dir,
            &planned,
            &BTreeSet::new(),
            &answers,
            &resolution,
            &transition.expected_before,
            Some(ActivationFault {
                fail_after_operations: 2,
                fail_rollback: true,
                projection_cleanup: None,
            }),
            None,
            None,
            None,
        )
        .expect_err("injected rollback should fail");

        assert_eq!(error.code(), "hive.activation-rollback-failed");
        assert!(error
            .to_string()
            .starts_with("hive.activation-rollback-failed"));
    }

    #[test]
    fn windows_style_replace_failure_restores_the_previous_destination() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let relative = Path::new("managed.txt");
        fs::write(temporary.path().join(relative), b"previous bytes")
            .expect("destination fixture should exist");

        let target =
            open_target_capability(temporary.path()).expect("target capability should open");
        let error = replace_capability_file_impl(
            &target,
            relative,
            b"replacement bytes",
            ReplacePolicy {
                destination_requires_backup: true,
                fail_after_backup: true,
            },
        )
        .expect_err("injected Windows-style replacement should fail");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(
            fs::read(temporary.path().join(relative)).expect("destination should be restored"),
            b"previous bytes"
        );
        let entries = fs::read_dir(temporary.path())
            .expect("temporary directory should remain readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("directory entries should be readable");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn windows_style_replace_success_leaves_no_backup_residue() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let relative = Path::new("managed.txt");
        fs::write(temporary.path().join(relative), b"previous bytes")
            .expect("destination fixture should exist");

        let target =
            open_target_capability(temporary.path()).expect("target capability should open");
        replace_capability_file_impl(
            &target,
            relative,
            b"replacement bytes",
            ReplacePolicy {
                destination_requires_backup: true,
                fail_after_backup: false,
            },
        )
        .expect("Windows-style replacement should succeed");

        assert_eq!(
            fs::read(temporary.path().join(relative)).expect("destination should be replaced"),
            b"replacement bytes"
        );
        let entries = fs::read_dir(temporary.path())
            .expect("temporary directory should remain readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("directory entries should be readable");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn hook_revocation_removes_only_previously_approved_artifacts() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        apply_fixture(
            temporary.path(),
            "answers-partial-hooks.yml",
            "capabilities-absent.json",
        );
        let sentinel = temporary.path().join(".hive/hooks/user-sentinel");
        fs::write(&sentinel, b"user bytes").expect("sentinel should be written");

        apply_fixture(
            temporary.path(),
            "answers-no-role-no-hook.yml",
            "capabilities-absent.json",
        );

        assert!(!temporary
            .path()
            .join(".hive/config/approved-hooks.yml")
            .exists());
        assert!(!temporary
            .path()
            .join(".hive/hooks/protect-hive-owned-state")
            .exists());
        assert!(!temporary
            .path()
            .join(".hive/hooks/checkpoint-reminder")
            .exists());
        assert_eq!(
            fs::read(sentinel).expect("sentinel should remain"),
            b"user bytes"
        );
    }

    #[test]
    fn validate_rejects_missing_required_role_and_digest_includes_wildcards() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let before = installed_tree_digest(&target).expect("digest should succeed");
        fs::write(
            target.join(".hive/knowledge/Wiki/custom.md"),
            b"custom canonical page\n",
        )
        .expect("custom Wiki page should be written");
        let after = installed_tree_digest(&target).expect("digest should succeed");
        assert_ne!(before, after);

        fs::remove_file(target.join(".hive/team/roles/reviewer.md"))
            .expect("role should be removable in fixture");
        let error = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Validate,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect_err("missing role must fail validation");
        assert_eq!(error.exit_code(), 5);
    }

    #[test]
    fn validate_rejects_missing_or_tampered_editing_discipline() {
        for mutation in ["missing", "tampered"] {
            let temporary = tempfile::tempdir().expect("temporary directory");
            let target = temporary
                .path()
                .canonicalize()
                .expect("fixture target should have a stable path");
            apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
            let directive = target.join(".hive/directives/00-editing-discipline.md");
            match mutation {
                "missing" => fs::remove_file(&directive).expect("directive should be removable"),
                "tampered" => {
                    fs::write(&directive, b"tampered\n").expect("directive should be writable");
                }
                _ => unreachable!(),
            }

            let error = execute_setup(&SetupRequest {
                target: &target,
                answers: &fixture("answers-base.yml"),
                capabilities: &fixture("capabilities-codex-omx.json"),
                mode: SetupMode::Validate,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            })
            .expect_err("invalid editing discipline must fail validation");
            assert_eq!(error.exit_code(), 5, "mutation: {mutation}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn validate_rejects_symlinked_editing_discipline_without_reading_target() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary
            .path()
            .canonicalize()
            .expect("fixture target should have a stable path");
        apply_fixture(&target, "answers-base.yml", "capabilities-codex-omx.json");
        let directive = target.join(".hive/directives/00-editing-discipline.md");
        fs::remove_file(&directive).expect("directive should be removable");
        let outside = target.join("outside-editing-discipline");
        fs::write(&outside, b"foreign bytes\n").expect("outside fixture should exist");
        symlink(&outside, &directive).expect("directive symlink should be created");

        let error = execute_setup(&SetupRequest {
            target: &target,
            answers: &fixture("answers-base.yml"),
            capabilities: &fixture("capabilities-codex-omx.json"),
            mode: SetupMode::Validate,
            reconfigure_roles: BTreeSet::new(),
            global_preferences: None,
        })
        .expect_err("symlinked editing discipline must fail validation");
        assert_eq!(error.exit_code(), 3);
        assert_eq!(
            fs::read(&outside).expect("outside bytes should remain readable"),
            b"foreign bytes\n"
        );
    }
}
