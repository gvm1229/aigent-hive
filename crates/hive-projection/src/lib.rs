//! Pure, provider-neutral compilation for portable Skill projections.
//!
//! This crate deliberately has no filesystem, process, network, model, or host
//! runtime adapter. Callers supply normalized routing facts and any optional
//! Skill source bytes they have already read from an approved local source.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};

const CATALOG_YAML: &str = include_str!("../../../harness/skills/catalog.yml");
const RETIRED_SKILL_NAMES_YAML: &str = include_str!("../../../harness/skills/retired-names.yml");
const HISTORICAL_BUILTINS_YAML: &str =
    include_str!("../../../harness/skills/historical-builtins.yml");
const SETUP_HIVE: &[u8] = include_bytes!("../../../harness/skills/configure/SKILL.md");
const SETUP_HARNESS: &[u8] = include_bytes!("../../../harness/skills/setup-project/SKILL.md");
const AUTO_SETUP_HARNESS: &[u8] =
    include_bytes!("../../../harness/skills/auto-setup-project/SKILL.md");
const SIMPLE_QUESTION: &[u8] = include_bytes!("../../../harness/skills/answer/SKILL.md");
const PROMPT_REFINE: &[u8] = include_bytes!("../../../harness/skills/refine-prompt/SKILL.md");
const KNOWLEDGE_CAPTURE: &[u8] =
    include_bytes!("../../../harness/skills/record-knowledge/SKILL.md");
const KNOWLEDGE_QUERY: &[u8] = include_bytes!("../../../harness/skills/search-knowledge/SKILL.md");
const KNOWLEDGE_PROMOTE: &[u8] = include_bytes!("../../../harness/skills/share-knowledge/SKILL.md");
const KNOWLEDGE_MAINTENANCE: &[u8] =
    include_bytes!("../../../harness/skills/maintain-knowledge/SKILL.md");
const RUN_CHECKPOINT: &[u8] = include_bytes!("../../../harness/skills/save-progress/SKILL.md");
const RUN_RESUME: &[u8] = include_bytes!("../../../harness/skills/resume-work/SKILL.md");
const USAGE_GUARD: &[u8] = include_bytes!("../../../harness/skills/manage-usage/SKILL.md");
const ROLE_HANDOFF: &[u8] = include_bytes!("../../../harness/skills/handoff-role/SKILL.md");
const JUDGE_PACKAGE: &[u8] = include_bytes!("../../../harness/skills/verify-package/SKILL.md");
const UPDATE_HARNESS: &[u8] = include_bytes!("../../../harness/skills/update-hive/SKILL.md");
const MIGRATE_HARNESS: &[u8] = include_bytes!("../../../harness/skills/migrate-project/SKILL.md");
const PROJECT_UPGRADE: &[u8] = include_bytes!("../../../harness/skills/upgrade-project/SKILL.md");
const LOOP_ENGINEERING: &[u8] = include_bytes!("../../../harness/skills/engineer-run/SKILL.md");
const HIVE_WIKI: &[u8] = include_bytes!("../../../harness/skills/manage-wiki/SKILL.md");
const AI_SLOP_CLEANER: &[u8] = include_bytes!("../../../harness/skills/clean-ai-slop/SKILL.md");
const BEST_PRACTICE_RESEARCH: &[u8] =
    include_bytes!("../../../harness/skills/research-practices/SKILL.md");
const KNOWLEDGE_SCAN: &[u8] =
    include_bytes!("../../../harness/skills/import-repository-knowledge/SKILL.md");

/// A stable validation or compilation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionError {
    code: &'static str,
    message: String,
}

impl ProjectionError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Stable, machine-readable Hive error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Human-readable detail suitable for a CLI diagnostic.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ProjectionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for ProjectionError {}

/// Supported subscription-authenticated agent hosts.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Host {
    Codex,
    Claude,
    Antigravity,
}

/// Language for Hive-owned Skill labels and concise descriptions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorLanguage {
    En,
    Ko,
}

/// Maps a public or retired built-in Skill name to its current public name.
///
/// Retired names are accepted only as migration input. Every new projection
/// and active-Skill record uses the returned public name.
pub fn canonical_builtin_skill_name(name: &str) -> Result<Option<String>, ProjectionError> {
    let catalog = embedded_catalog()?;
    if catalog.skills.iter().any(|skill| skill.name == name) {
        return Ok(Some(name.to_owned()));
    }
    Ok(retired_builtin_skill_names()?.get(name).cloned())
}

impl Host {
    /// Project-local discovery root for this host.
    #[must_use]
    pub const fn skill_root(self) -> &'static str {
        match self {
            Self::Codex | Self::Antigravity => ".agents/skills",
            Self::Claude => ".claude/skills",
        }
    }
}

/// Skill side-effect declaration from the canonical catalog.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SideEffectClass {
    None,
    ReadOnly,
    ProjectWrite,
    ExternalWrite,
    Orchestration,
}

/// Provider-neutral capability declared by a Skill.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    ExternalApp,
    FilesystemRead,
    FilesystemWrite,
    MemoryRead,
    MemoryWrite,
    Network,
    RunState,
    Shell,
    Subagents,
}

/// Whether a catalog entry has shipped executable instructions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Availability {
    Implemented,
    CatalogOnly,
}

/// One canonical portable Skill catalog entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCatalogEntry {
    pub name: String,
    pub description: String,
    pub provided_by: String,
    pub superseded_by_external: Vec<String>,
    pub invocation_intents: Vec<String>,
    pub side_effect_class: SideEffectClass,
    pub capabilities: Vec<Capability>,
    pub availability: Availability,
}

/// The embedded canonical Skill catalog.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCatalog {
    pub schema_version: u32,
    pub skills: Vec<SkillCatalogEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetiredSkillNameLedger {
    schema_version: u32,
    retired_names: BTreeMap<String, String>,
}

/// Returns the canonical retired-ID ledger used by selection migration and
/// ownership-aware projection cleanup.
///
/// Every entry is flattened to the current public ID. When a public Skill is
/// renamed again, its former public ID and every earlier retired ID remain in
/// this ledger with the new current ID, so resolution stays transitive.
pub fn retired_builtin_skill_names() -> Result<BTreeMap<String, String>, ProjectionError> {
    let ledger: RetiredSkillNameLedger =
        serde_yaml::from_str(RETIRED_SKILL_NAMES_YAML).map_err(|error| {
            ProjectionError::new(
                "hive.skill-name-ledger-invalid",
                format!("retired Skill name ledger is not valid YAML: {error}"),
            )
        })?;
    if ledger.schema_version != 1 {
        return Err(ProjectionError::new(
            "hive.skill-name-ledger-invalid",
            "retired Skill name ledger schema_version must be 1",
        ));
    }
    let catalog: SkillCatalog = serde_yaml::from_str(CATALOG_YAML).map_err(|error| {
        ProjectionError::new(
            "hive.skill-catalog-invalid",
            format!("embedded catalog is not valid YAML: {error}"),
        )
    })?;
    let current = catalog
        .skills
        .iter()
        .map(|skill| skill.name.as_str())
        .collect::<BTreeSet<_>>();
    for (retired, current_name) in &ledger.retired_names {
        validate_skill_name(retired)?;
        validate_skill_name(current_name)?;
        if current.contains(retired.as_str()) || !current.contains(current_name.as_str()) {
            return Err(ProjectionError::new(
                "hive.skill-name-ledger-invalid",
                format!("retired Skill mapping is not a retired-to-current mapping: {retired}"),
            ));
        }
    }
    Ok(ledger.retired_names)
}

/// Parses and semantically validates the catalog embedded at build time.
///
/// # Errors
///
/// Returns an error when the embedded catalog or Skill/body availability
/// relationship violates the shipped contract.
pub fn embedded_catalog() -> Result<SkillCatalog, ProjectionError> {
    let catalog: SkillCatalog = serde_yaml::from_str(CATALOG_YAML).map_err(|error| {
        ProjectionError::new(
            "hive.skill-catalog-invalid",
            format!("embedded catalog is not valid YAML: {error}"),
        )
    })?;
    if catalog.schema_version != 1 {
        return Err(ProjectionError::new(
            "hive.skill-catalog-invalid",
            "embedded catalog schema_version must be 1",
        ));
    }

    let mut names = BTreeSet::new();
    for skill in &catalog.skills {
        validate_skill_name(&skill.name)?;
        if skill.provided_by != "hive" {
            return Err(ProjectionError::new(
                "hive.skill-catalog-invalid",
                format!("built-in Skill {} is not provided by Hive", skill.name),
            ));
        }
        if !names.insert(skill.name.as_str()) {
            return Err(ProjectionError::new(
                "hive.skill-catalog-invalid",
                format!("duplicate Skill catalog entry {}", skill.name),
            ));
        }
        validate_sorted_unique(&skill.capabilities, "catalog capabilities")?;
        validate_sorted_unique(
            &skill.superseded_by_external,
            "catalog external supersession",
        )?;
        validate_sorted_unique(&skill.invocation_intents, "catalog invocation intents")?;
    }
    // Validate the ledger whenever the catalog is loaded so every call site
    // fails closed when a future rename is incomplete or ambiguous.
    let retired_names = retired_builtin_skill_names()?;
    if retired_names
        .values()
        .any(|name| !names.contains(name.as_str()))
    {
        return Err(ProjectionError::new(
            "hive.skill-name-ledger-invalid",
            "retired Skill name ledger targets a missing catalog Skill",
        ));
    }

    for (name, _) in embedded_skill_sources() {
        let entry = catalog
            .skills
            .iter()
            .find(|entry| entry.name == name)
            .ok_or_else(|| {
                ProjectionError::new(
                    "hive.skill-catalog-invalid",
                    format!("embedded Skill {name} is absent from the catalog"),
                )
            })?;
        if entry.availability != Availability::Implemented {
            return Err(ProjectionError::new(
                "hive.skill-catalog-invalid",
                format!("embedded Skill {name} is not marked implemented"),
            ));
        }
    }
    for entry in &catalog.skills {
        let source_exists = embedded_skill_source(&entry.name).is_some();
        if (entry.availability == Availability::Implemented) != source_exists {
            return Err(ProjectionError::new(
                "hive.skill-catalog-invalid",
                format!(
                    "Skill {} availability does not match its embedded body",
                    entry.name
                ),
            ));
        }
    }

    Ok(catalog)
}

/// Consent payload bound to one optional Skill.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionalSkillConsent {
    pub consent_version: u32,
    pub name: String,
    pub source: String,
    pub revision: String,
    pub content_digest: String,
    pub requested_capabilities: Vec<Capability>,
    pub approved_capabilities: Vec<Capability>,
    pub approved_at: String,
    pub consent_digest: String,
}

#[derive(Serialize)]
struct ConsentPayload<'a> {
    consent_version: u32,
    name: &'a str,
    source: &'a str,
    revision: &'a str,
    content_digest: &'a str,
    requested_capabilities: &'a [Capability],
    approved_capabilities: &'a [Capability],
    approved_at: &'a str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillFrontmatter {
    name: String,
    description: String,
}

/// Exact optional source proof supplied by the filesystem-owning caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionalSkillSource {
    pub consent: OptionalSkillConsent,
    /// Exact logical source locator that the caller used to obtain `skill_md`.
    pub source_locator: String,
    pub skill_md: Vec<u8>,
}

/// Compiled portable Skill tree and its logical active config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    pub files: BTreeMap<String, Vec<u8>>,
    pub active_skills: ActiveSkills,
}

/// Logical `.hive/config/active-skills.yml` document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveSkills {
    pub schema_version: u32,
    pub skills: Vec<ActiveSkill>,
}

/// One active projected Skill.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveSkill {
    pub name: String,
    pub source_type: SkillSourceType,
    pub content_digest: String,
    pub side_effect_class: SideEffectClass,
    pub capabilities: Vec<Capability>,
    pub consent_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalBuiltInCatalog {
    schema_version: u32,
    releases: Vec<HistoricalBuiltInRelease>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalBuiltInRelease {
    version: String,
    skills: Vec<HistoricalBuiltInSkill>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalBuiltInSkill {
    name: String,
    content_digest: String,
    side_effect_class: SideEffectClass,
    capabilities: Vec<Capability>,
}

/// Origin of active Skill bytes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSourceType {
    BuiltIn,
    ApprovedOptional,
}

/// Returns the exact built-in Skill metadata shipped by a supported historical
/// Hive release.
///
/// This registry lets an updater authenticate an older projection from known
/// release bytes without trusting the consumer-writable active-Skill ledger.
/// The registry intentionally contains digests and public metadata only.
///
/// # Errors
///
/// Returns an error when the embedded registry is malformed or `version` is
/// not one of the supported historical releases.
pub fn historical_builtin_skills(version: &str) -> Result<Vec<ActiveSkill>, ProjectionError> {
    const SUPPORTED: [&str; 6] = ["0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0"];
    let catalog: HistoricalBuiltInCatalog = serde_yaml::from_str(HISTORICAL_BUILTINS_YAML)
        .map_err(|error| {
            ProjectionError::new(
                "hive.skill-history-invalid",
                format!("embedded historical built-in registry is not valid YAML: {error}"),
            )
        })?;
    if catalog.schema_version != 1
        || catalog.releases.len() != SUPPORTED.len()
        || catalog
            .releases
            .iter()
            .zip(SUPPORTED)
            .any(|(release, expected)| release.version != expected)
    {
        return Err(ProjectionError::new(
            "hive.skill-history-invalid",
            "historical built-in registry release coverage is not exact",
        ));
    }

    let release = catalog
        .releases
        .into_iter()
        .find(|release| release.version == version)
        .ok_or_else(|| {
            ProjectionError::new(
                "hive.skill-history-unsupported",
                format!("unsupported historical Skill projection release: {version}"),
            )
        })?;
    let mut names = BTreeSet::new();
    let mut skills = Vec::with_capacity(release.skills.len());
    for skill in release.skills {
        validate_skill_name(&skill.name)?;
        validate_history_digest(&skill.content_digest)?;
        validate_sorted_unique(&skill.capabilities, "historical built-in capabilities")?;
        if !names.insert(skill.name.clone()) {
            return Err(ProjectionError::new(
                "hive.skill-history-invalid",
                format!(
                    "duplicate historical built-in Skill {} in release {version}",
                    skill.name
                ),
            ));
        }
        skills.push(ActiveSkill {
            name: skill.name,
            source_type: SkillSourceType::BuiltIn,
            content_digest: skill.content_digest,
            side_effect_class: skill.side_effect_class,
            capabilities: skill.capabilities,
            consent_digest: None,
        });
    }
    skills.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(skills)
}

/// Compiles all implemented built-ins plus verified optional source proofs.
///
/// Catalog-only future Skills remain absent. The returned map is logical only:
/// this crate never reads or writes a consumer project.
///
/// # Errors
///
/// Returns an error if the embedded catalog is invalid, an optional Skill lacks
/// exact source/consent/capability proof, or projected names collide.
pub fn compile_projection(
    host: Host,
    optional_sources: &[OptionalSkillSource],
) -> Result<Projection, ProjectionError> {
    let catalog = embedded_catalog()?;
    let selected = catalog
        .skills
        .iter()
        .filter(|entry| {
            entry.availability == Availability::Implemented && entry.name != "configure"
        })
        .map(|entry| entry.name.clone())
        .collect();
    compile_selected(
        host,
        optional_sources,
        &catalog,
        &selected,
        false,
        DescriptorLanguage::En,
    )
}

/// Compiles an exact selected user-scope built-in set plus verified optional
/// source proofs.
///
/// `configure` is intentionally user-scope only and remains absent from
/// project projections compiled by [`compile_projection`].
///
/// # Errors
///
/// Returns an error when a selected name is unknown, unavailable, duplicated,
/// or the optional source proof is invalid.
pub fn compile_user_projection(
    host: Host,
    selected_names: &[String],
    optional_sources: &[OptionalSkillSource],
) -> Result<Projection, ProjectionError> {
    compile_user_projection_localized(
        host,
        selected_names,
        optional_sources,
        DescriptorLanguage::En,
    )
}

/// Compiles a user projection whose Hive-owned labels and descriptions match
/// the selected interface language.
pub fn compile_user_projection_localized(
    host: Host,
    selected_names: &[String],
    optional_sources: &[OptionalSkillSource],
    language: DescriptorLanguage,
) -> Result<Projection, ProjectionError> {
    compile_named_projection(host, selected_names, optional_sources, true, language)
}

/// Compiles an exact selected project-scope built-in set plus verified
/// optional source proofs.
///
/// The user-only `configure` Skill is rejected. Callers must resolve and
/// preview dependency closure before invoking this deterministic projection.
///
/// # Errors
///
/// Returns an error when a selected name is unknown, unavailable, duplicated,
/// user-only, or the optional source proof is invalid.
pub fn compile_project_projection(
    host: Host,
    selected_names: &[String],
    optional_sources: &[OptionalSkillSource],
) -> Result<Projection, ProjectionError> {
    compile_named_projection(
        host,
        selected_names,
        optional_sources,
        false,
        DescriptorLanguage::En,
    )
}

fn compile_named_projection(
    host: Host,
    selected_names: &[String],
    optional_sources: &[OptionalSkillSource],
    allow_user_only: bool,
    language: DescriptorLanguage,
) -> Result<Projection, ProjectionError> {
    let catalog = embedded_catalog()?;
    let selected: BTreeSet<String> = selected_names
        .iter()
        .map(|name| {
            canonical_builtin_skill_name(name)
                .map(|canonical| canonical.unwrap_or_else(|| name.clone()))
        })
        .collect::<Result<_, _>>()?;
    if selected.len() != selected_names.len() {
        return Err(ProjectionError::new(
            "hive.skill-selection-invalid",
            "selected user Skills must be unique",
        ));
    }
    for name in &selected {
        let entry = catalog
            .skills
            .iter()
            .find(|entry| entry.name == *name)
            .ok_or_else(|| {
                ProjectionError::new(
                    "hive.skill-selection-invalid",
                    format!("selected user Skill is unknown: {name}"),
                )
            })?;
        if entry.availability != Availability::Implemented {
            return Err(ProjectionError::new(
                "hive.skill-selection-invalid",
                format!("selected user Skill is unavailable: {name}"),
            ));
        }
        if !allow_user_only && name == "configure" {
            return Err(ProjectionError::new(
                "hive.skill-selection-invalid",
                "configure is user-scope only",
            ));
        }
    }
    compile_selected(
        host,
        optional_sources,
        &catalog,
        &selected,
        allow_user_only,
        language,
    )
}

fn compile_selected(
    host: Host,
    optional_sources: &[OptionalSkillSource],
    catalog: &SkillCatalog,
    selected: &BTreeSet<String>,
    preserve_implicit_metadata: bool,
    language: DescriptorLanguage,
) -> Result<Projection, ProjectionError> {
    let mut files = BTreeMap::new();
    let mut active = Vec::new();
    let mut names = BTreeSet::new();
    let retired_names = retired_builtin_skill_names()?;

    for entry in catalog
        .skills
        .iter()
        .filter(|entry| selected.contains(&entry.name))
    {
        let source = embedded_skill_source(&entry.name).ok_or_else(|| {
            ProjectionError::new(
                "hive.skill-catalog-invalid",
                format!("implemented Skill {} has no embedded bytes", entry.name),
            )
        })?;
        names.insert(entry.name.clone());
        let source = localized_skill_source(&entry.name, source, language)?;
        files.insert(skill_path(host, &entry.name), source.clone());
        if matches!(host, Host::Codex | Host::Antigravity) {
            let metadata = embedded_skill_metadata(&entry.name).ok_or_else(|| {
                ProjectionError::new(
                    "hive.skill-catalog-invalid",
                    format!("implemented Skill {} has no Codex metadata", entry.name),
                )
            })?;
            files.insert(
                skill_metadata_path(host, &entry.name),
                if preserve_implicit_metadata {
                    localized_skill_metadata(&entry.name, metadata, language)?
                } else {
                    explicit_only_metadata(&localized_skill_metadata(
                        &entry.name,
                        metadata,
                        language,
                    )?)?
                },
            );
        }
        active.push(ActiveSkill {
            name: entry.name.clone(),
            source_type: SkillSourceType::BuiltIn,
            content_digest: sha256_digest(&source),
            side_effect_class: entry.side_effect_class,
            capabilities: entry.capabilities.clone(),
            consent_digest: None,
        });
    }

    for optional in optional_sources {
        validate_optional_source(optional)?;
        let name = &optional.consent.name;
        if !names.insert(name.clone()) {
            return Err(ProjectionError::new(
                "hive.skill-projection-conflict",
                format!("Skill name {name} is already active or reserved"),
            ));
        }
        if catalog.skills.iter().any(|entry| entry.name == *name)
            || retired_names.contains_key(name)
        {
            return Err(ProjectionError::new(
                "hive.skill-projection-conflict",
                format!("optional Skill {name} collides with a current or retired built-in Skill"),
            ));
        }

        files.insert(skill_path(host, name), optional.skill_md.clone());
        active.push(ActiveSkill {
            name: name.clone(),
            source_type: SkillSourceType::ApprovedOptional,
            content_digest: optional.consent.content_digest.clone(),
            side_effect_class: side_effect_class_for_capabilities(
                &optional.consent.approved_capabilities,
            ),
            capabilities: optional.consent.approved_capabilities.clone(),
            consent_digest: Some(optional.consent.consent_digest.clone()),
        });
    }

    active.sort_by(|left, right| left.name.cmp(&right.name));
    let active_skills = ActiveSkills {
        schema_version: 1,
        skills: active,
    };
    let active_yaml = serde_yaml::to_string(&active_skills).map_err(|error| {
        ProjectionError::new(
            "hive.skill-projection-invalid",
            format!("active Skill config serialization failed: {error}"),
        )
    })?;
    files.insert(
        ".hive/config/active-skills.yml".to_owned(),
        active_yaml.into_bytes(),
    );

    Ok(Projection {
        files,
        active_skills,
    })
}

fn validate_optional_source(optional: &OptionalSkillSource) -> Result<(), ProjectionError> {
    let consent = &optional.consent;
    if consent.consent_version != 1 {
        return Err(optional_inert("consent_version must be 1"));
    }
    if optional.source_locator != consent.source {
        return Err(optional_inert(
            "caller source locator does not exactly match the approved source",
        ));
    }
    validate_skill_name(&consent.name).map_err(|error| optional_inert(error.to_string()))?;
    validate_local_source_locator(&consent.source)?;
    validate_sha256(&consent.revision, "revision")?;
    validate_sha256(&consent.content_digest, "content_digest")?;
    validate_sha256(&consent.consent_digest, "consent_digest")?;
    validate_utc_seconds(&consent.approved_at)?;
    validate_sorted_unique(&consent.requested_capabilities, "requested capabilities")?;
    validate_sorted_unique(&consent.approved_capabilities, "approved capabilities")?;

    if consent.approved_capabilities != consent.requested_capabilities {
        return Err(optional_inert(
            "the host projection cannot enforce partial grants; every requested capability must be approved",
        ));
    }
    let source_digest = sha256_digest(&optional.skill_md);
    if consent.content_digest != source_digest || consent.revision != source_digest {
        return Err(optional_inert(
            "optional Skill source bytes do not match content_digest and immutable revision",
        ));
    }
    validate_skill_frontmatter_name(&optional.skill_md, &consent.name)?;

    let payload = consent_payload(consent);
    let canonical = serde_json_canonicalizer::to_vec(&payload).map_err(|error| {
        optional_inert(format!("consent payload canonicalization failed: {error}"))
    })?;
    if consent.consent_digest != sha256_digest(&canonical) {
        return Err(optional_inert("consent digest does not match its payload"));
    }

    Ok(())
}

fn consent_payload(consent: &OptionalSkillConsent) -> ConsentPayload<'_> {
    ConsentPayload {
        consent_version: consent.consent_version,
        name: &consent.name,
        source: &consent.source,
        revision: &consent.revision,
        content_digest: &consent.content_digest,
        requested_capabilities: &consent.requested_capabilities,
        approved_capabilities: &consent.approved_capabilities,
        approved_at: &consent.approved_at,
    }
}

fn optional_inert(message: impl Into<String>) -> ProjectionError {
    ProjectionError::new("hive.optional-skill-inert", message)
}

fn side_effect_class_for_capabilities(capabilities: &[Capability]) -> SideEffectClass {
    if capabilities
        .iter()
        .any(|capability| matches!(capability, Capability::Subagents | Capability::RunState))
    {
        SideEffectClass::Orchestration
    } else if capabilities
        .iter()
        .any(|capability| matches!(capability, Capability::Network | Capability::ExternalApp))
    {
        SideEffectClass::ExternalWrite
    } else if capabilities.iter().any(|capability| {
        matches!(
            capability,
            Capability::FilesystemWrite | Capability::MemoryWrite | Capability::Shell
        )
    }) {
        SideEffectClass::ProjectWrite
    } else if capabilities.is_empty() {
        SideEffectClass::None
    } else {
        SideEffectClass::ReadOnly
    }
}

fn validate_local_source_locator(source: &str) -> Result<(), ProjectionError> {
    let Some(path) = source.strip_prefix("path:") else {
        return Err(optional_inert(
            "optional Skill activation requires an exact project-local path: source",
        ));
    };
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || has_windows_drive_prefix(path)
        || path
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
        || !(path == "SKILL.md" || path.ends_with("/SKILL.md"))
    {
        return Err(optional_inert(
            "optional Skill source must be a safe project-relative path",
        ));
    }
    Ok(())
}

fn validate_skill_frontmatter_name(bytes: &[u8], expected: &str) -> Result<(), ProjectionError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| optional_inert("optional Skill source is not valid UTF-8"))?;
    let (rest, delimiter) = if let Some(rest) = text.strip_prefix("---\n") {
        (rest, "\n---\n")
    } else if let Some(rest) = text.strip_prefix("---\r\n") {
        (rest, "\r\n---\r\n")
    } else {
        return Err(optional_inert(
            "optional Skill source has no YAML frontmatter",
        ));
    };
    let Some((frontmatter, _)) = rest.split_once(delimiter) else {
        return Err(optional_inert(
            "optional Skill source has unterminated YAML frontmatter",
        ));
    };
    let value: SkillFrontmatter = serde_yaml::from_str(frontmatter)
        .map_err(|error| optional_inert(format!("invalid Skill frontmatter: {error}")))?;
    if value.name != expected || value.description.trim().is_empty() {
        return Err(optional_inert(
            "optional Skill frontmatter name or description does not match approval",
        ));
    }
    Ok(())
}

fn skill_path(host: Host, name: &str) -> String {
    format!("{}/{name}/SKILL.md", host.skill_root())
}

fn skill_metadata_path(host: Host, name: &str) -> String {
    format!("{}/{name}/agents/openai.yaml", host.skill_root())
}

fn explicit_only_metadata(metadata: &[u8]) -> Result<Vec<u8>, ProjectionError> {
    const IMPLICIT: &str = "  allow_implicit_invocation: true";
    const EXPLICIT: &str = "  allow_implicit_invocation: false";

    let text = std::str::from_utf8(metadata).map_err(|_| {
        ProjectionError::new(
            "hive.skill-catalog-invalid",
            "embedded Codex Skill metadata must be UTF-8",
        )
    })?;
    if !text.contains(IMPLICIT) && !text.contains(EXPLICIT) {
        return Err(ProjectionError::new(
            "hive.skill-catalog-invalid",
            "embedded Codex Skill metadata lacks an invocation policy",
        ));
    }
    Ok(text.replace(IMPLICIT, EXPLICIT).into_bytes())
}

fn localized_skill_source(
    name: &str,
    source: &[u8],
    language: DescriptorLanguage,
) -> Result<Vec<u8>, ProjectionError> {
    if language == DescriptorLanguage::En {
        return Ok(source.to_vec());
    }
    let (_, description) = localized_skill_text(name, language).ok_or_else(|| {
        ProjectionError::new(
            "hive.skill-catalog-invalid",
            format!("implemented Skill {name} has no localized description"),
        )
    })?;
    replace_metadata_line(
        source,
        "description:",
        &format!("description: {description}"),
    )
}

fn localized_skill_metadata(
    name: &str,
    metadata: &[u8],
    language: DescriptorLanguage,
) -> Result<Vec<u8>, ProjectionError> {
    let (display_name, description) = localized_skill_text(name, language).ok_or_else(|| {
        ProjectionError::new(
            "hive.skill-catalog-invalid",
            format!("implemented Skill {name} has no localized metadata"),
        )
    })?;
    let metadata = replace_metadata_line(
        metadata,
        "  display_name:",
        &format!("  display_name: {display_name:?}"),
    )?;
    replace_metadata_line(
        &metadata,
        "  short_description:",
        &format!("  short_description: {description:?}"),
    )
}

fn replace_metadata_line(
    bytes: &[u8],
    prefix: &str,
    replacement: &str,
) -> Result<Vec<u8>, ProjectionError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        ProjectionError::new(
            "hive.skill-catalog-invalid",
            "embedded Hive Skill text must be UTF-8",
        )
    })?;
    let mut replaced = false;
    let mut rendered = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if line.trim_end_matches('\n').starts_with(prefix) {
            rendered.push_str(replacement);
            rendered.push('\n');
            replaced = true;
        } else {
            rendered.push_str(line);
        }
    }
    if !replaced {
        return Err(ProjectionError::new(
            "hive.skill-catalog-invalid",
            format!("embedded Hive Skill text lacks required {prefix} field"),
        ));
    }
    Ok(rendered.into_bytes())
}

fn localized_skill_text(
    name: &str,
    language: DescriptorLanguage,
) -> Option<(&'static str, &'static str)> {
    let (en_name, en_description, ko_name, ko_description) = match name {
        "configure" => (
            "Configure Hive",
            "Configure or reconfigure global Aigent Hive preferences.",
            "Hive 설정",
            "전역 Aigent Hive 환경 설정과 재설정을 진행합니다.",
        ),
        "setup-project" => (
            "Set up project",
            "Configure or reconfigure Aigent Hive for one project.",
            "프로젝트 설정",
            "프로젝트별 Aigent Hive 환경을 설정하거나 다시 설정합니다.",
        ),
        "auto-setup-project" => (
            "Automatically set up project",
            "Set up a project from repository evidence with minimal questions.",
            "프로젝트 자동 설정",
            "저장소 근거를 바탕으로 프로젝트 환경을 자동 설정합니다.",
        ),
        "answer" => (
            "Answer",
            "Answer a simple question without inspecting or changing a project.",
            "간단히 답하기",
            "프로젝트를 검사하거나 변경하지 않고 간단한 질문에 답합니다.",
        ),
        "refine-prompt" => (
            "Refine prompt",
            "Turn a request into an approval-ready prompt before execution.",
            "프롬프트 다듬기",
            "실행 전 승인받을 수 있도록 요청을 명확한 프롬프트로 다듬습니다.",
        ),
        "record-knowledge" => (
            "Record knowledge",
            "Record reviewed facts and workflows as canonical knowledge.",
            "지식 기록",
            "검토한 사실과 작업 방식을 정본 지식으로 기록합니다.",
        ),
        "search-knowledge" => (
            "Search knowledge",
            "Retrieve only the Hive knowledge needed for the current work.",
            "지식 검색",
            "작업에 필요한 Hive 지식을 제한된 범위에서 검색합니다.",
        ),
        "share-knowledge" => (
            "Share knowledge",
            "Promote reviewed project knowledge to global Hive knowledge.",
            "지식 공유",
            "검토한 프로젝트 지식을 전역 Hive 지식으로 승격합니다.",
        ),
        "maintain-knowledge" => (
            "Maintain knowledge",
            "Check, maintain, and rebuild the derived knowledge index.",
            "지식 관리",
            "정본 지식을 검사·정리하고 파생 색인을 다시 만듭니다.",
        ),
        "save-progress" => (
            "Save progress",
            "Save durable, resumable work progress in canonical Markdown.",
            "진행 상황 저장",
            "재개 가능한 작업 진행 상황을 정본 Markdown에 저장합니다.",
        ),
        "resume-work" => (
            "Resume work",
            "Validate a work handoff and read safe resume context.",
            "작업 재개",
            "저장한 작업 인계를 검증하고 새 세션의 재개 정보를 읽습니다.",
        ),
        "manage-usage" => (
            "Manage usage",
            "Inspect or adjust usage safeguards for the current session.",
            "사용량 보호 관리",
            "사용량 보호 기준과 현재 세션의 안전 상태를 확인하거나 바꿉니다.",
        ),
        "handoff-role" => (
            "Hand off role",
            "Update canonical role assignment and handoff records safely.",
            "역할 인계",
            "역할 배정과 인계 기록을 정본 Markdown에 안전하게 갱신합니다.",
        ),
        "verify-package" => (
            "Verify package",
            "Verify a work package and its signed attestations.",
            "패키지 검증",
            "검증용 작업 패키지와 서명된 확인 정보를 검사합니다.",
        ),
        "update-hive" => (
            "Update Hive",
            "Preview, verify, and safely apply a signed Hive update.",
            "Hive 업데이트",
            "서명된 Hive 업데이트를 미리 보기·검증·안전 적용합니다.",
        ),
        "upgrade-project" => (
            "Upgrade project",
            "Upgrade Hive-generated project guidance while preserving local edits.",
            "프로젝트 업그레이드",
            "Hive가 생성한 프로젝트 지침과 Skill을 충돌 보존 방식으로 올립니다.",
        ),
        "migrate-project" => (
            "Migrate project",
            "Migrate a supported Hive project format with backup and validation.",
            "프로젝트 이전",
            "지원되는 Hive 프로젝트 형식을 백업과 검증을 거쳐 이전합니다.",
        ),
        "engineer-run" => (
            "Engineer run",
            "Prepare a Hive work run with planning, verification, and handoff.",
            "작업 실행 설계",
            "계획·검증·인계를 연결한 Hive 작업 실행을 준비합니다.",
        ),
        "manage-wiki" => (
            "Manage Wiki",
            "Manage records, search, and checks for the Hive Markdown Wiki.",
            "Wiki 관리",
            "Hive Markdown Wiki의 기록·검색·검사를 관리합니다.",
        ),
        "import-repository-knowledge" => (
            "Import repository knowledge",
            "Import reviewed repository content as Hive knowledge.",
            "저장소 지식 가져오기",
            "저장소의 검토된 내용을 Hive 지식으로 가져옵니다.",
        ),
        "clean-ai-slop" => (
            "Clean AI slop",
            "Review and clean unnecessary AI-generated wording and structure.",
            "AI 군더더기 정리",
            "AI가 만든 불필요한 표현과 구조를 검토해 정리합니다.",
        ),
        "research-practices" => (
            "Research practices",
            "Research implementation practices from reliable current sources.",
            "관행 조사",
            "신뢰할 수 있는 최신 자료를 바탕으로 구현 관행을 조사합니다.",
        ),
        _ => return None,
    };
    Some(match language {
        DescriptorLanguage::En => (en_name, en_description),
        DescriptorLanguage::Ko => (ko_name, ko_description),
    })
}

fn embedded_skill_source(name: &str) -> Option<&'static [u8]> {
    embedded_skill_sources()
        .into_iter()
        .find_map(|(candidate, bytes)| (candidate == name).then_some(bytes))
}

fn embedded_skill_metadata(name: &str) -> Option<&'static [u8]> {
    match name {
        "configure" => Some(include_bytes!(
            "../../../harness/skills/configure/agents/openai.yaml"
        )),
        "setup-project" => Some(include_bytes!(
            "../../../harness/skills/setup-project/agents/openai.yaml"
        )),
        "auto-setup-project" => Some(include_bytes!(
            "../../../harness/skills/auto-setup-project/agents/openai.yaml"
        )),
        "answer" => Some(include_bytes!(
            "../../../harness/skills/answer/agents/openai.yaml"
        )),
        "refine-prompt" => Some(include_bytes!(
            "../../../harness/skills/refine-prompt/agents/openai.yaml"
        )),
        "record-knowledge" => Some(include_bytes!(
            "../../../harness/skills/record-knowledge/agents/openai.yaml"
        )),
        "search-knowledge" => Some(include_bytes!(
            "../../../harness/skills/search-knowledge/agents/openai.yaml"
        )),
        "share-knowledge" => Some(include_bytes!(
            "../../../harness/skills/share-knowledge/agents/openai.yaml"
        )),
        "maintain-knowledge" => Some(include_bytes!(
            "../../../harness/skills/maintain-knowledge/agents/openai.yaml"
        )),
        "save-progress" => Some(include_bytes!(
            "../../../harness/skills/save-progress/agents/openai.yaml"
        )),
        "resume-work" => Some(include_bytes!(
            "../../../harness/skills/resume-work/agents/openai.yaml"
        )),
        "manage-usage" => Some(include_bytes!(
            "../../../harness/skills/manage-usage/agents/openai.yaml"
        )),
        "handoff-role" => Some(include_bytes!(
            "../../../harness/skills/handoff-role/agents/openai.yaml"
        )),
        "verify-package" => Some(include_bytes!(
            "../../../harness/skills/verify-package/agents/openai.yaml"
        )),
        "update-hive" => Some(include_bytes!(
            "../../../harness/skills/update-hive/agents/openai.yaml"
        )),
        "migrate-project" => Some(include_bytes!(
            "../../../harness/skills/migrate-project/agents/openai.yaml"
        )),
        "upgrade-project" => Some(include_bytes!(
            "../../../harness/skills/upgrade-project/agents/openai.yaml"
        )),
        "engineer-run" => Some(include_bytes!(
            "../../../harness/skills/engineer-run/agents/openai.yaml"
        )),
        "manage-wiki" => Some(include_bytes!(
            "../../../harness/skills/manage-wiki/agents/openai.yaml"
        )),
        "clean-ai-slop" => Some(include_bytes!(
            "../../../harness/skills/clean-ai-slop/agents/openai.yaml"
        )),
        "research-practices" => Some(include_bytes!(
            "../../../harness/skills/research-practices/agents/openai.yaml"
        )),
        "import-repository-knowledge" => Some(include_bytes!(
            "../../../harness/skills/import-repository-knowledge/agents/openai.yaml"
        )),
        _ => None,
    }
}

fn embedded_skill_sources() -> [(&'static str, &'static [u8]); 22] {
    [
        ("configure", SETUP_HIVE),
        ("setup-project", SETUP_HARNESS),
        ("auto-setup-project", AUTO_SETUP_HARNESS),
        ("answer", SIMPLE_QUESTION),
        ("refine-prompt", PROMPT_REFINE),
        ("record-knowledge", KNOWLEDGE_CAPTURE),
        ("search-knowledge", KNOWLEDGE_QUERY),
        ("share-knowledge", KNOWLEDGE_PROMOTE),
        ("maintain-knowledge", KNOWLEDGE_MAINTENANCE),
        ("save-progress", RUN_CHECKPOINT),
        ("resume-work", RUN_RESUME),
        ("manage-usage", USAGE_GUARD),
        ("handoff-role", ROLE_HANDOFF),
        ("verify-package", JUDGE_PACKAGE),
        ("update-hive", UPDATE_HARNESS),
        ("migrate-project", MIGRATE_HARNESS),
        ("upgrade-project", PROJECT_UPGRADE),
        ("engineer-run", LOOP_ENGINEERING),
        ("manage-wiki", HIVE_WIKI),
        ("clean-ai-slop", AI_SLOP_CLEANER),
        ("research-practices", BEST_PRACTICE_RESEARCH),
        ("import-repository-knowledge", KNOWLEDGE_SCAN),
    ]
}

/// Logical actions accepted from a normalized host routing surface.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LogicalAction {
    AnswerSimpleQuestion,
    RefinePrompt,
    RunWork,
    ResumeWork,
    VerifyWork,
    IngestKnowledge,
    QueryKnowledge,
    UpdateHarness,
}

/// External compatible Skill candidate normalized by the active host.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalCandidate {
    pub name: String,
    pub provided_by: ExternalProvider,
    pub compatible: bool,
    /// Whether the user explicitly selected this external compatibility layer.
    #[serde(default)]
    pub explicit_selection: bool,
}

/// Supported external orchestration providers. They remain runtime-owned.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalProvider {
    Omx,
    Omc,
}

/// Prompt refinement mode.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefineMode {
    RefineOnly,
    RefineAndRun,
}

/// Host-normalized prompt quality used for a bounded automatic refinement gate.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptQuality {
    /// Goal and execution details are sufficient for the selected action.
    #[default]
    Sufficient,
    /// More than one materially different interpretation remains.
    Ambiguous,
    /// Goal, scope, constraints, acceptance, or output details are materially absent.
    MissingCoreDetails,
}

/// Already-normalized facts supplied by a host. This is not raw prompt text.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct RoutingRequest {
    pub schema_version: u32,
    pub host: Host,
    pub explicit_action: Option<LogicalAction>,
    pub explicit_skill: Option<String>,
    pub plain_answer: bool,
    pub simple_question: bool,
    pub project_context_required: bool,
    pub external_candidate: Option<ExternalCandidate>,
    pub hive_candidate: Option<String>,
    pub active_hive_skills: Vec<ActiveHiveSkillProof>,
    pub refine_mode: Option<RefineMode>,
    pub explicit_run_intent: bool,
    #[serde(default)]
    pub prompt_quality: PromptQuality,
}

/// Digest-bound proof that a Hive Skill is present in the active projection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveHiveSkillProof {
    pub name: String,
    pub source_type: SkillSourceType,
    pub content_digest: String,
    pub side_effect_class: SideEffectClass,
    pub capabilities: Vec<Capability>,
    pub consent_digest: Option<String>,
    pub consent: Option<OptionalSkillConsent>,
}

/// Selected routing lane.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Route {
    Direct,
    SimpleQuestion,
    ExternalSkill,
    HiveSkill,
    HostNative,
    Blocked,
}

/// Provider owning the selected operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteProvider {
    Hive,
    Omx,
    Omc,
    HostNative,
}

/// Deterministic routing decision. `load_skill_bodies` contains at most one item.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecision {
    pub schema_version: u32,
    pub route: Route,
    pub logical_action: LogicalAction,
    pub selected_skill: Option<String>,
    pub provided_by: Option<RouteProvider>,
    pub mode: Option<RefineMode>,
    pub load_skill_bodies: Vec<String>,
    pub next_action: Option<LogicalAction>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub refine_suggestion: bool,
}

/// Resolves normalized routing facts without inspecting or classifying a prompt.
///
/// # Errors
///
/// Returns an error when normalized facts have an invalid schema version, Skill
/// name, provider namespace, or host/provider combination.
pub fn resolve_route(request: &RoutingRequest) -> Result<RoutingDecision, ProjectionError> {
    if request.schema_version != 1 {
        return Err(ProjectionError::new(
            "hive.routing-invalid",
            "routing request schema_version must be 1",
        ));
    }
    let fallback_action = request.explicit_action.unwrap_or(LogicalAction::RunWork);

    // Explicit direct/plain intent is the highest-precedence no-workflow lane.
    let resolved = if request.plain_answer {
        decision(
            Route::Direct,
            LogicalAction::AnswerSimpleQuestion,
            None,
            None,
            None,
            None,
        )
    } else {
        resolve_non_plain_route(request, fallback_action)?
    };
    if should_automatically_refine(request, &resolved) {
        return resolve_hive_skill(
            request,
            "refine-prompt",
            LogicalAction::RefinePrompt,
            Route::HiveSkill,
        );
    }
    Ok(resolved)
}

fn should_automatically_refine(request: &RoutingRequest, resolved: &RoutingDecision) -> bool {
    matches!(
        request.prompt_quality,
        PromptQuality::Ambiguous | PromptQuality::MissingCoreDetails
    ) && !request.plain_answer
        && !request.simple_question
        && resolved.route == Route::HostNative
        && resolved.logical_action == LogicalAction::RunWork
        && resolved.selected_skill.is_none()
        && resolved.load_skill_bodies.is_empty()
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(value: &bool) -> bool {
    !*value
}

fn resolve_non_plain_route(
    request: &RoutingRequest,
    fallback_action: LogicalAction,
) -> Result<RoutingDecision, ProjectionError> {
    if let Some(skill) = request.explicit_skill.as_deref() {
        validate_routing_skill_name(skill)?;
    }
    let explicit_refinement_requested = request.explicit_action
        == Some(LogicalAction::RefinePrompt)
        || request.explicit_skill.as_deref() == Some("refine-prompt");
    if explicit_refinement_requested
        && request.refine_mode == Some(RefineMode::RefineAndRun)
        && !request.explicit_run_intent
    {
        return Ok(decision(
            Route::Blocked,
            LogicalAction::RefinePrompt,
            None,
            None,
            None,
            None,
        ));
    }
    if request.explicit_action == Some(LogicalAction::RefinePrompt)
        && request.explicit_skill.is_none()
    {
        return resolve_hive_skill(
            request,
            "refine-prompt",
            LogicalAction::RefinePrompt,
            Route::HiveSkill,
        );
    }

    if let Some(explicit) = resolve_explicit_skill(request, fallback_action)? {
        return Ok(explicit);
    }

    if let Some(simple_question) = resolve_simple_question_route(request)? {
        return Ok(simple_question);
    }

    if let Some(skill) = request
        .hive_candidate
        .as_deref()
        .filter(|skill| is_phase_four_data_contract_skill(skill))
    {
        validate_skill_name(skill)?;
        let action = action_for_skill(skill).unwrap_or(fallback_action);
        return resolve_hive_skill(request, skill, action, Route::HiveSkill);
    }

    if let Some(external) = request
        .external_candidate
        .as_ref()
        .filter(|candidate| candidate.compatible && candidate.explicit_selection)
    {
        validate_skill_name(&external.name)?;
        if external_provider_matches_host(external.provided_by, request.host) {
            return Ok(decision(
                Route::ExternalSkill,
                fallback_action,
                Some(external.name.clone()),
                Some(route_provider(external.provided_by)),
                refinement_mode(fallback_action, request),
                None,
            ));
        }
    }

    if let Some(skill) = request.hive_candidate.as_deref() {
        validate_skill_name(skill)?;
        let action = action_for_skill(skill).unwrap_or(fallback_action);
        if action == LogicalAction::RefinePrompt
            && request.refine_mode == Some(RefineMode::RefineAndRun)
            && !request.explicit_run_intent
        {
            return Ok(decision(
                Route::Blocked,
                LogicalAction::RefinePrompt,
                None,
                None,
                None,
                None,
            ));
        }
        return resolve_hive_skill(request, skill, action, Route::HiveSkill);
    }

    Ok(decision(
        Route::HostNative,
        fallback_action,
        None,
        Some(RouteProvider::HostNative),
        refinement_mode(fallback_action, request),
        None,
    ))
}

fn resolve_simple_question_route(
    request: &RoutingRequest,
) -> Result<Option<RoutingDecision>, ProjectionError> {
    // An explicit simple action also enters the isolation gate.
    if request.explicit_action != Some(LogicalAction::AnswerSimpleQuestion)
        && (request.explicit_action.is_some() || !request.simple_question)
    {
        return Ok(None);
    }
    if request.project_context_required {
        return Ok(Some(decision(
            Route::Blocked,
            LogicalAction::AnswerSimpleQuestion,
            None,
            None,
            None,
            Some(LogicalAction::RunWork),
        )));
    }
    Ok(Some(resolve_hive_skill(
        request,
        "answer",
        LogicalAction::AnswerSimpleQuestion,
        Route::SimpleQuestion,
    )?))
}

fn resolve_explicit_skill(
    request: &RoutingRequest,
    fallback_action: LogicalAction,
) -> Result<Option<RoutingDecision>, ProjectionError> {
    let Some(skill) = request.explicit_skill.as_deref() else {
        return Ok(None);
    };
    let action = action_for_skill(skill).unwrap_or(fallback_action);
    let provider = explicit_skill_provider(skill, request.host)?;
    if provider == RouteProvider::Hive {
        return resolve_hive_skill(request, skill, action, Route::HiveSkill).map(Some);
    }
    let route = match provider {
        RouteProvider::Omx | RouteProvider::Omc => Route::ExternalSkill,
        RouteProvider::Hive => unreachable!("Hive Skills return above"),
        RouteProvider::HostNative => Route::HostNative,
    };
    Ok(Some(decision(
        route,
        action,
        Some(skill.to_owned()),
        Some(provider),
        refinement_mode(action, request),
        None,
    )))
}

fn resolve_hive_skill(
    request: &RoutingRequest,
    skill: &str,
    action: LogicalAction,
    route: Route,
) -> Result<RoutingDecision, ProjectionError> {
    if !validate_active_hive_skill(request, skill)? {
        return Ok(decision(Route::Blocked, action, None, None, None, None));
    }
    Ok(decision(
        route,
        action,
        Some(skill.to_owned()),
        Some(RouteProvider::Hive),
        refinement_mode(action, request),
        None,
    ))
}

fn validate_active_hive_skill(
    request: &RoutingRequest,
    skill: &str,
) -> Result<bool, ProjectionError> {
    let mut matching = request
        .active_hive_skills
        .iter()
        .filter(|proof| proof.name == skill);
    let Some(proof) = matching.next() else {
        return Ok(false);
    };
    if matching.next().is_some() {
        return Err(routing_proof_invalid(format!(
            "active Hive Skill proof is duplicated: {skill}"
        )));
    }

    validate_skill_name(&proof.name).map_err(|error| routing_proof_invalid(error.to_string()))?;
    match proof.source_type {
        SkillSourceType::BuiltIn => validate_builtin_routing_proof(proof)?,
        SkillSourceType::ApprovedOptional => validate_optional_routing_proof(proof)?,
    }
    Ok(true)
}

fn validate_builtin_routing_proof(proof: &ActiveHiveSkillProof) -> Result<(), ProjectionError> {
    let catalog = embedded_catalog()?;
    let entry = catalog
        .skills
        .iter()
        .find(|entry| entry.name == proof.name && entry.availability == Availability::Implemented)
        .ok_or_else(|| {
            routing_proof_invalid(format!(
                "built-in Skill is not implemented in the embedded catalog: {}",
                proof.name
            ))
        })?;
    let source = embedded_skill_source(&proof.name).ok_or_else(|| {
        routing_proof_invalid(format!(
            "built-in Skill has no embedded source: {}",
            proof.name
        ))
    })?;
    if proof.content_digest != sha256_digest(source)
        || proof.side_effect_class != entry.side_effect_class
        || proof.capabilities != entry.capabilities
        || proof.consent_digest.is_some()
        || proof.consent.is_some()
    {
        return Err(routing_proof_invalid(format!(
            "built-in active Skill proof does not match embedded metadata: {}",
            proof.name
        )));
    }
    Ok(())
}

fn validate_optional_routing_proof(proof: &ActiveHiveSkillProof) -> Result<(), ProjectionError> {
    if embedded_catalog()?
        .skills
        .iter()
        .any(|entry| entry.name == proof.name)
    {
        return Err(routing_proof_invalid(format!(
            "approved optional Skill collides with the embedded catalog: {}",
            proof.name
        )));
    }
    let consent = proof.consent.as_ref().ok_or_else(|| {
        routing_proof_invalid("approved optional Skill proof has no exact consent evidence")
    })?;
    validate_routing_consent(consent)?;
    if consent.name != proof.name
        || consent.content_digest != proof.content_digest
        || proof.capabilities != consent.approved_capabilities
        || proof.side_effect_class != side_effect_class_for_capabilities(&proof.capabilities)
        || proof.consent_digest.as_deref() != Some(consent.consent_digest.as_str())
    {
        return Err(routing_proof_invalid(
            "approved optional Skill proof does not match its exact consent evidence",
        ));
    }
    Ok(())
}

fn validate_routing_consent(consent: &OptionalSkillConsent) -> Result<(), ProjectionError> {
    if consent.consent_version != 1 {
        return Err(routing_proof_invalid("consent_version must be 1"));
    }
    validate_skill_name(&consent.name).map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_local_source_locator(&consent.source)
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_sha256(&consent.revision, "revision")
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_sha256(&consent.content_digest, "content_digest")
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_sha256(&consent.consent_digest, "consent_digest")
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_utc_seconds(&consent.approved_at)
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_sorted_unique(&consent.requested_capabilities, "requested capabilities")
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    validate_sorted_unique(&consent.approved_capabilities, "approved capabilities")
        .map_err(|error| routing_proof_invalid(error.to_string()))?;
    if consent.requested_capabilities != consent.approved_capabilities
        || consent.revision != consent.content_digest
    {
        return Err(routing_proof_invalid(
            "approved optional routing proof requires an exact immutable capability grant",
        ));
    }
    let canonical =
        serde_json_canonicalizer::to_vec(&consent_payload(consent)).map_err(|error| {
            routing_proof_invalid(format!("consent payload canonicalization failed: {error}"))
        })?;
    if consent.consent_digest != sha256_digest(&canonical) {
        return Err(routing_proof_invalid(
            "approved optional routing proof has a forged consent digest",
        ));
    }
    Ok(())
}

fn routing_proof_invalid(message: impl Into<String>) -> ProjectionError {
    ProjectionError::new("hive.routing-proof-invalid", message)
}

fn decision(
    route: Route,
    logical_action: LogicalAction,
    selected_skill: Option<String>,
    provided_by: Option<RouteProvider>,
    mode: Option<RefineMode>,
    next_action: Option<LogicalAction>,
) -> RoutingDecision {
    let load_skill_bodies = selected_skill.iter().cloned().collect();
    RoutingDecision {
        schema_version: 1,
        route,
        logical_action,
        selected_skill,
        provided_by,
        mode,
        load_skill_bodies,
        next_action,
        refine_suggestion: false,
    }
}

fn refinement_mode(action: LogicalAction, request: &RoutingRequest) -> Option<RefineMode> {
    if action != LogicalAction::RefinePrompt {
        return None;
    }
    let mode = request.refine_mode.unwrap_or(RefineMode::RefineOnly);
    Some(mode)
}

fn action_for_skill(skill: &str) -> Option<LogicalAction> {
    match skill {
        "answer" => Some(LogicalAction::AnswerSimpleQuestion),
        "refine-prompt" => Some(LogicalAction::RefinePrompt),
        "record-knowledge"
        | "maintain-knowledge"
        | "share-knowledge"
        | "import-repository-knowledge" => Some(LogicalAction::IngestKnowledge),
        "search-knowledge" | "manage-wiki" => Some(LogicalAction::QueryKnowledge),
        "clean-ai-slop" | "research-practices" | "engineer-run" | "save-progress"
        | "handoff-role" | "manage-usage" => Some(LogicalAction::RunWork),
        "resume-work" => Some(LogicalAction::ResumeWork),
        "update-hive" | "migrate-project" | "upgrade-project" => Some(LogicalAction::UpdateHarness),
        _ => None,
    }
}

fn is_phase_four_data_contract_skill(skill: &str) -> bool {
    matches!(skill, "save-progress" | "resume-work" | "handoff-role")
}

fn explicit_skill_provider(skill: &str, host: Host) -> Result<RouteProvider, ProjectionError> {
    if let Some((namespace, _)) = skill.split_once(':') {
        return match (namespace, host) {
            ("omx", Host::Codex) => Ok(RouteProvider::Omx),
            ("omc", Host::Claude) => Ok(RouteProvider::Omc),
            ("omx" | "omc", _) => Err(ProjectionError::new(
                "hive.routing-unsupported",
                "explicit external Skill namespace is incompatible with the active host",
            )),
            _ => Err(ProjectionError::new(
                "hive.routing-invalid",
                "unknown explicit Skill provider namespace",
            )),
        };
    }
    Ok(RouteProvider::Hive)
}

fn external_provider_matches_host(provider: ExternalProvider, host: Host) -> bool {
    matches!(
        (provider, host),
        (ExternalProvider::Omx, Host::Codex) | (ExternalProvider::Omc, Host::Claude)
    )
}

const fn route_provider(provider: ExternalProvider) -> RouteProvider {
    match provider {
        ExternalProvider::Omx => RouteProvider::Omx,
        ExternalProvider::Omc => RouteProvider::Omc,
    }
}

/// Fields whose exact text locators must survive prompt refinement.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Preservation {
    pub must: Vec<String>,
    pub must_not: Vec<String>,
    pub scope: Vec<String>,
    pub target_output: Vec<String>,
    pub user_authority: Vec<String>,
    pub tone: Vec<String>,
    pub tool_provider_selection: Vec<String>,
}

/// Provider-neutral prompt refinement input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PromptRefinementInput {
    pub schema_version: u32,
    pub original_prompt: String,
    pub target_agent: Option<String>,
    pub target_host: Option<Host>,
    pub mode: RefineMode,
    pub explicit_run_intent: bool,
    pub sufficiently_specific: bool,
    pub project_grounding: bool,
    pub preserve: Preservation,
}

/// Side effects that a pure refinement envelope must never perform.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct RefinementSideEffects {
    pub project_write: bool,
    pub network: bool,
    pub subagent: bool,
    pub memory_capture: bool,
    pub run_creation: bool,
    pub model_execution: bool,
}

/// Lifecycle state returned after a refinement envelope passes validation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptRefinementState {
    /// The result is presentation-only until an exact follow-up approval.
    AwaitingApproval,
    /// Same-request explicit `--run` or a validated exact approval permits host handoff.
    Authorized,
}

/// Digest-bound result state that a host can use without retaining the raw prompt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PromptRefinementLifecycle {
    pub schema_version: u32,
    pub state: PromptRefinementState,
    pub refined_prompt_digest: String,
    pub target_host: Option<Host>,
    pub execution_authorized: bool,
    pub side_effects: RefinementSideEffects,
}

/// Candidate prompt refinement output to validate before returning.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PromptRefinementResult {
    pub schema_version: u32,
    pub mode: RefineMode,
    pub original_prompt: String,
    pub intent_summary: String,
    pub assumptions: Vec<String>,
    pub unresolved_items: Vec<String>,
    pub required_question: Option<String>,
    pub preserved: Preservation,
    pub refined_prompt: String,
    pub project_reads: Vec<String>,
    pub execution_authorized: bool,
    pub side_effects: RefinementSideEffects,
}

/// Validates a `refine-prompt` envelope without refining or executing it.
///
/// # Errors
///
/// Returns an error when the envelope changes meaning or authority, exceeds the
/// growth budget, uses unapproved provider syntax, or implies hidden execution
/// or another side effect.
pub fn validate_prompt_refinement(
    input: &PromptRefinementInput,
    result: &PromptRefinementResult,
) -> Result<(), ProjectionError> {
    if input.schema_version != 1 || result.schema_version != 1 {
        return Err(refinement_invalid("schema_version must be 1"));
    }
    if input.original_prompt.is_empty()
        || result.original_prompt != input.original_prompt
        || result.intent_summary.trim().is_empty()
        || result.refined_prompt.trim().is_empty()
    {
        return Err(refinement_invalid(
            "original prompt must remain immutable and output text must be non-empty",
        ));
    }
    if input.mode != result.mode {
        return Err(refinement_invalid(
            "result mode differs from requested mode",
        ));
    }
    if input.mode == RefineMode::RefineAndRun && !input.explicit_run_intent {
        return Err(ProjectionError::new(
            "hive.refine-run-not-authorized",
            "refine-and-run requires explicit run intent",
        ));
    }
    let should_authorize = input.mode == RefineMode::RefineAndRun && input.explicit_run_intent;
    if result.execution_authorized != should_authorize {
        return Err(ProjectionError::new(
            "hive.refine-run-not-authorized",
            "execution authorization does not match explicit run intent",
        ));
    }
    if any_side_effect(&result.side_effects) {
        return Err(refinement_invalid(
            "prompt refinement validation cannot perform execution or side effects",
        ));
    }
    if input.preserve != result.preserved {
        return Err(ProjectionError::new(
            "hive.prompt-meaning-drift",
            "preservation fields differ from the normalized original",
        ));
    }
    validate_preservation(&input.preserve, &result.refined_prompt)?;
    validate_unique(&result.assumptions, "assumptions")?;
    validate_unique(&result.unresolved_items, "unresolved items")?;
    validate_unique(&result.project_reads, "project reads")?;
    if !input.project_grounding && !result.project_reads.is_empty() {
        return Err(refinement_invalid(
            "project reads require explicit project grounding",
        ));
    }
    for path in &result.project_reads {
        validate_project_relative_path(path)?;
    }
    if result.required_question.is_some() && result.unresolved_items.is_empty() {
        return Err(refinement_invalid(
            "a required question must correspond to an unresolved item",
        ));
    }
    let question_count = result
        .required_question
        .as_deref()
        .map_or(0, |question| question.matches('?').count())
        .max(
            result
                .unresolved_items
                .iter()
                .filter(|item| item.starts_with("Question:"))
                .count(),
        );
    if question_count > 1 {
        return Err(refinement_invalid(
            "prompt refinement may ask at most one required question at a time",
        ));
    }
    if input.target_host.is_none() && contains_provider_specific_syntax(&result.refined_prompt) {
        return Err(ProjectionError::new(
            "hive.prompt-provider-syntax-forbidden",
            "provider-specific syntax requires an explicit target host",
        ));
    }
    if input.sufficiently_specific {
        let original_chars = input.original_prompt.chars().count();
        let additive_budget = original_chars.saturating_add(700);
        let proportional_budget = original_chars.saturating_mul(3).saturating_add(1) / 2;
        let maximum = additive_budget.max(proportional_budget);
        if result.refined_prompt.chars().count() > maximum {
            return Err(refinement_invalid(
                "an already-specific prompt was expanded beyond the refinement budget",
            ));
        }
    }
    Ok(())
}

/// Build the exact state emitted after a successful refinement validation.
#[must_use]
pub fn prompt_refinement_lifecycle(
    input: &PromptRefinementInput,
    result: &PromptRefinementResult,
) -> PromptRefinementLifecycle {
    PromptRefinementLifecycle {
        schema_version: 1,
        state: if result.execution_authorized {
            PromptRefinementState::Authorized
        } else {
            PromptRefinementState::AwaitingApproval
        },
        refined_prompt_digest: sha256_digest(result.refined_prompt.as_bytes()),
        target_host: input.target_host,
        execution_authorized: result.execution_authorized,
        side_effects: result.side_effects.clone(),
    }
}

fn validate_preservation(
    preservation: &Preservation,
    refined_prompt: &str,
) -> Result<(), ProjectionError> {
    let normalized_refined = refined_prompt.to_lowercase();
    let fields = [
        ("must", &preservation.must),
        ("must-not", &preservation.must_not),
        ("scope", &preservation.scope),
        ("target output", &preservation.target_output),
        ("user authority", &preservation.user_authority),
        ("tone", &preservation.tone),
        (
            "tool/provider selection",
            &preservation.tool_provider_selection,
        ),
    ];
    for (kind, locators) in fields {
        validate_unique(locators, kind)?;
        for locator in locators {
            if !normalized_refined.contains(&locator.to_lowercase()) {
                return Err(ProjectionError::new(
                    "hive.prompt-meaning-drift",
                    format!("{kind} locator is absent from refined prompt: {locator}"),
                ));
            }
        }
    }
    Ok(())
}

fn any_side_effect(side_effects: &RefinementSideEffects) -> bool {
    side_effects.project_write
        || side_effects.network
        || side_effects.subagent
        || side_effects.memory_capture
        || side_effects.run_creation
        || side_effects.model_execution
}

fn contains_provider_specific_syntax(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
        ".codex/",
        ".claude/",
        ".agents/",
        "$skill",
        "/skill",
        "omx ",
        "omc ",
        "codex ",
        "claude ",
        "antigravity ",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn refinement_invalid(message: impl Into<String>) -> ProjectionError {
    ProjectionError::new("hive.prompt-refinement-invalid", message)
}

fn validate_project_relative_path(path: &str) -> Result<(), ProjectionError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || has_windows_drive_prefix(path)
        || path.split(['/', '\\']).any(|component| component == "..")
    {
        return Err(refinement_invalid(
            "project read locator must be a safe project-relative path",
        ));
    }
    Ok(())
}

fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn validate_routing_skill_name(name: &str) -> Result<(), ProjectionError> {
    let mut parts = name.split(':');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if parts.next().is_some() {
        return Err(ProjectionError::new(
            "hive.routing-invalid",
            "Skill name contains more than one provider namespace",
        ));
    }
    match second {
        Some(skill) if matches!(first, "omx" | "omc") => validate_skill_name(skill),
        Some(_) => Err(ProjectionError::new(
            "hive.routing-invalid",
            "unknown Skill provider namespace",
        )),
        None => validate_skill_name(first),
    }
}

fn validate_skill_name(name: &str) -> Result<(), ProjectionError> {
    let valid = (2..=63).contains(&name.len())
        && name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !valid {
        return Err(ProjectionError::new(
            "hive.skill-name-invalid",
            format!("invalid Skill name: {name}"),
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, field: &str) -> Result<(), ProjectionError> {
    let valid = value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(optional_inert(format!("{field} is not a SHA-256 digest")));
    }
    Ok(())
}

fn validate_history_digest(value: &str) -> Result<(), ProjectionError> {
    let valid = value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(ProjectionError::new(
            "hive.skill-history-invalid",
            "historical built-in content_digest is not a SHA-256 digest",
        ));
    }
    Ok(())
}

fn validate_utc_seconds(value: &str) -> Result<(), ProjectionError> {
    let bytes = value.as_bytes();
    let separators = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (19, b'Z'),
    ];
    let separators_valid = bytes.len() == 20
        && separators
            .iter()
            .all(|(index, expected)| bytes.get(*index) == Some(expected));
    let digits_valid = bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit());
    let component = |start: usize, end: usize| {
        value
            .get(start..end)
            .and_then(|part| part.parse::<u32>().ok())
    };
    let ranges_valid = matches!(component(5, 7), Some(1..=12))
        && matches!(component(8, 10), Some(1..=31))
        && matches!(component(11, 13), Some(0..=23))
        && matches!(component(14, 16), Some(0..=59))
        && matches!(component(17, 19), Some(0..=59));
    if !separators_valid || !digits_valid || !ranges_valid {
        return Err(optional_inert("approved_at must use UTC seconds precision"));
    }
    Ok(())
}

fn validate_sorted_unique<T: Ord>(values: &[T], field: &str) -> Result<(), ProjectionError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ProjectionError::new(
            "hive.normalized-input-invalid",
            format!("{field} must be lexicographically sorted and unique"),
        ));
    }
    Ok(())
}

fn validate_unique<T: Ord>(values: &[T], field: &str) -> Result<(), ProjectionError> {
    let unique = values.iter().collect::<BTreeSet<_>>();
    if unique.len() != values.len() {
        return Err(ProjectionError::new(
            "hive.normalized-input-invalid",
            format!("{field} must not contain duplicates"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routing_request() -> RoutingRequest {
        RoutingRequest {
            schema_version: 1,
            host: Host::Codex,
            explicit_action: Some(LogicalAction::RunWork),
            explicit_skill: None,
            plain_answer: false,
            simple_question: false,
            project_context_required: false,
            external_candidate: None,
            hive_candidate: None,
            active_hive_skills: Vec::new(),
            refine_mode: None,
            explicit_run_intent: false,
            prompt_quality: PromptQuality::Sufficient,
        }
    }

    #[test]
    fn ambiguous_host_native_work_automatically_loads_refine_only() {
        let mut request = routing_request();
        request.explicit_action = None;
        request.prompt_quality = PromptQuality::Ambiguous;
        request.active_hive_skills = vec![builtin_proof("refine-prompt")];

        let resolved = resolve_route(&request).expect("routing succeeds");

        assert_eq!(resolved.route, Route::HiveSkill);
        assert_eq!(resolved.logical_action, LogicalAction::RefinePrompt);
        assert!(!resolved.refine_suggestion);
        assert_eq!(resolved.selected_skill.as_deref(), Some("refine-prompt"));
        assert_eq!(resolved.load_skill_bodies, ["refine-prompt"]);
        assert_eq!(resolved.mode, Some(RefineMode::RefineOnly));
    }

    #[test]
    fn clear_work_and_simple_questions_never_automatically_refine() {
        let clear = resolve_route(&routing_request()).expect("clear work routes");
        assert!(!clear.refine_suggestion);
        assert_eq!(clear.route, Route::HostNative);

        let mut simple = routing_request();
        simple.explicit_action = Some(LogicalAction::AnswerSimpleQuestion);
        simple.simple_question = true;
        simple.prompt_quality = PromptQuality::MissingCoreDetails;
        simple.hive_candidate = Some("answer".to_owned());
        simple.active_hive_skills = vec![builtin_proof("answer")];
        let simple = resolve_route(&simple).expect("simple question routes");
        assert!(!simple.refine_suggestion);
        assert_eq!(simple.route, Route::SimpleQuestion);
    }

    #[test]
    fn explicit_refinement_action_loads_the_hive_skill_without_a_host_candidate() {
        let mut request = routing_request();
        request.explicit_action = Some(LogicalAction::RefinePrompt);
        request.active_hive_skills = vec![builtin_proof("refine-prompt")];

        let resolved = resolve_route(&request).expect("explicit refine route");

        assert_eq!(resolved.route, Route::HiveSkill);
        assert_eq!(resolved.selected_skill.as_deref(), Some("refine-prompt"));
        assert_eq!(resolved.mode, Some(RefineMode::RefineOnly));
    }

    fn builtin_proof(name: &str) -> ActiveHiveSkillProof {
        let catalog = embedded_catalog().expect("embedded catalog");
        let entry = catalog
            .skills
            .iter()
            .find(|entry| entry.name == name)
            .expect("built-in catalog entry");
        ActiveHiveSkillProof {
            name: name.to_owned(),
            source_type: SkillSourceType::BuiltIn,
            content_digest: sha256_digest(
                embedded_skill_source(name).expect("embedded Skill source"),
            ),
            side_effect_class: entry.side_effect_class,
            capabilities: entry.capabilities.clone(),
            consent_digest: None,
            consent: None,
        }
    }

    fn preservation() -> Preservation {
        Preservation {
            must: vec!["Run cargo test".to_owned()],
            must_not: vec!["Do not add dependencies".to_owned()],
            scope: vec!["src/config.rs".to_owned()],
            target_output: vec!["Return changed files".to_owned()],
            user_authority: vec!["Stop before changing the public API".to_owned()],
            tone: Vec::new(),
            tool_provider_selection: Vec::new(),
        }
    }

    fn refinement_input() -> PromptRefinementInput {
        PromptRefinementInput {
            schema_version: 1,
            original_prompt: "Improve these instructions.".to_owned(),
            target_agent: None,
            target_host: None,
            mode: RefineMode::RefineOnly,
            explicit_run_intent: false,
            sufficiently_specific: false,
            project_grounding: false,
            preserve: preservation(),
        }
    }

    fn refinement_result() -> PromptRefinementResult {
        PromptRefinementResult {
            schema_version: 1,
            mode: RefineMode::RefineOnly,
            original_prompt: "Improve these instructions.".to_owned(),
            intent_summary: "Produce safe implementation instructions.".to_owned(),
            assumptions: Vec::new(),
            unresolved_items: Vec::new(),
            required_question: None,
            preserved: preservation(),
            refined_prompt: [
                "Goal",
                "Update src/config.rs.",
                "Constraints and prohibited actions",
                "Do not add dependencies.",
                "Acceptance and verification",
                "Run cargo test.",
                "Output contract",
                "Return changed files.",
                "Stop, blocker, and escalation conditions",
                "Stop before changing the public API.",
            ]
            .join("\n"),
            project_reads: Vec::new(),
            execution_authorized: false,
            side_effects: RefinementSideEffects {
                project_write: false,
                network: false,
                subagent: false,
                memory_capture: false,
                run_creation: false,
                model_execution: false,
            },
        }
    }

    fn optional_source() -> OptionalSkillSource {
        let skill_md = br"---
name: local-inspect
description: Inspect one local file without changing it.
---

# Local Inspect
"
        .to_vec();
        let digest = sha256_digest(&skill_md);
        let mut consent = OptionalSkillConsent {
            consent_version: 1,
            name: "local-inspect".to_owned(),
            source: "path:vendor-skills/local-inspect/SKILL.md".to_owned(),
            revision: digest.clone(),
            content_digest: digest,
            requested_capabilities: vec![Capability::FilesystemRead],
            approved_capabilities: vec![Capability::FilesystemRead],
            approved_at: "2026-07-24T00:00:00Z".to_owned(),
            consent_digest: String::new(),
        };
        consent.consent_digest = sha256_digest(
            &serde_json_canonicalizer::to_vec(&consent_payload(&consent))
                .expect("canonical consent"),
        );
        OptionalSkillSource {
            source_locator: consent.source.clone(),
            consent,
            skill_md,
        }
    }

    fn approved_optional_proof(source: &OptionalSkillSource) -> ActiveHiveSkillProof {
        ActiveHiveSkillProof {
            name: source.consent.name.clone(),
            source_type: SkillSourceType::ApprovedOptional,
            content_digest: source.consent.content_digest.clone(),
            side_effect_class: SideEffectClass::ReadOnly,
            capabilities: source.consent.approved_capabilities.clone(),
            consent_digest: Some(source.consent.consent_digest.clone()),
            consent: Some(source.consent.clone()),
        }
    }

    #[test]
    fn compiles_exact_builtins_to_each_host_root_deterministically() {
        for host in [Host::Codex, Host::Claude, Host::Antigravity] {
            let first = compile_projection(host, &[]).expect("projection");
            let second = compile_projection(host, &[]).expect("projection");
            assert_eq!(first, second);
            assert_eq!(first.active_skills.skills.len(), 21);
            let expected_file_count = if host == Host::Claude { 22 } else { 43 };
            assert_eq!(first.files.len(), expected_file_count);
            for skill in [
                "clean-ai-slop",
                "auto-setup-project",
                "research-practices",
                "verify-package",
                "import-repository-knowledge",
                "engineer-run",
                "refine-prompt",
                "save-progress",
                "resume-work",
                "handoff-role",
                "update-hive",
                "manage-usage",
                "manage-wiki",
                "migrate-project",
            ] {
                assert!(first
                    .files
                    .contains_key(&format!("{}/{skill}/SKILL.md", host.skill_root())));
            }
        }
    }

    #[test]
    fn user_projection_contains_exact_caller_resolved_skill_selection() {
        let selected = vec![
            "configure".to_owned(),
            "update-hive".to_owned(),
            "manage-usage".to_owned(),
        ];

        let projection =
            compile_user_projection(Host::Codex, &selected, &[]).expect("user projection");

        let expected_files = BTreeSet::from([
            ".agents/skills/configure/SKILL.md",
            ".agents/skills/configure/agents/openai.yaml",
            ".agents/skills/update-hive/SKILL.md",
            ".agents/skills/update-hive/agents/openai.yaml",
            ".agents/skills/manage-usage/SKILL.md",
            ".agents/skills/manage-usage/agents/openai.yaml",
            ".hive/config/active-skills.yml",
        ]);
        assert_eq!(
            projection
                .files
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            expected_files
        );
        assert_eq!(
            projection
                .active_skills
                .skills
                .iter()
                .map(|skill| skill.name.as_str())
                .collect::<Vec<_>>(),
            ["configure", "manage-usage", "update-hive"]
        );
        for (name, expected_implicit) in [
            ("configure", true),
            ("update-hive", false),
            ("manage-usage", true),
        ] {
            let metadata = std::str::from_utf8(
                projection
                    .files
                    .get(&format!(".agents/skills/{name}/agents/openai.yaml"))
                    .expect("selected Skill metadata"),
            )
            .expect("selected Skill metadata should be UTF-8");
            assert_eq!(
                metadata.contains("allow_implicit_invocation: true"),
                expected_implicit,
                "{name} user metadata policy"
            );
        }
    }

    #[test]
    fn legacy_user_selection_emits_only_short_public_names() {
        let projection = compile_user_projection(
            Host::Codex,
            &[
                "setup-hive".to_owned(),
                "hive-knowledge-capture".to_owned(),
                "ai-slop-cleaner".to_owned(),
            ],
            &[],
        )
        .expect("legacy selection migration");

        let names = projection
            .active_skills
            .skills
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["clean-ai-slop", "configure", "record-knowledge"]);
        assert!(projection
            .files
            .contains_key(".agents/skills/clean-ai-slop/SKILL.md"));
        assert!(!projection
            .files
            .contains_key(".agents/skills/ai-slop-cleaner/SKILL.md"));
    }

    #[test]
    fn retired_skill_name_ledger_resolves_every_old_name_to_a_current_id() {
        let catalog = embedded_catalog().expect("embedded catalog");
        let current = catalog
            .skills
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<BTreeSet<_>>();
        let retired = retired_builtin_skill_names().expect("retired name ledger");

        assert_eq!(retired.len(), 22);
        for (old_name, current_name) in retired {
            assert!(
                !current.contains(old_name.as_str()),
                "retired ID remains public"
            );
            assert!(
                current.contains(current_name.as_str()),
                "missing current target"
            );
            assert_eq!(
                canonical_builtin_skill_name(&old_name).expect("ledger resolves old ID"),
                Some(current_name),
            );
        }
    }

    #[test]
    fn localized_user_projection_renders_skill_descriptors_in_selected_language() {
        let selected = ["configure".to_owned()];
        let english =
            compile_user_projection_localized(Host::Codex, &selected, &[], DescriptorLanguage::En)
                .expect("English projection");
        let korean =
            compile_user_projection_localized(Host::Codex, &selected, &[], DescriptorLanguage::Ko)
                .expect("Korean projection");

        let english_skill = std::str::from_utf8(
            english
                .files
                .get(".agents/skills/configure/SKILL.md")
                .expect("English Skill source"),
        )
        .expect("UTF-8 English Skill source");
        let korean_skill = std::str::from_utf8(
            korean
                .files
                .get(".agents/skills/configure/SKILL.md")
                .expect("Korean Skill source"),
        )
        .expect("UTF-8 Korean Skill source");
        let korean_metadata = std::str::from_utf8(
            korean
                .files
                .get(".agents/skills/configure/agents/openai.yaml")
                .expect("Korean Skill metadata"),
        )
        .expect("UTF-8 Korean Skill metadata");

        assert!(english_skill.contains("description: Configure or reconfigure global user-scope"));
        assert!(
            korean_skill.contains("description: 전역 Aigent Hive 환경 설정과 재설정을 진행합니다.")
        );
        assert!(korean_metadata.contains("display_name: \"Hive 설정\""));
        assert!(korean_metadata
            .contains("short_description: \"전역 Aigent Hive 환경 설정과 재설정을 진행합니다.\""));
        assert_ne!(
            english.active_skills.skills[0].content_digest,
            korean.active_skills.skills[0].content_digest
        );
    }

    #[test]
    fn setup_hive_is_available_in_user_projection() {
        let projection = compile_user_projection(Host::Claude, &["configure".to_owned()], &[])
            .expect("configure user projection");

        assert!(projection
            .files
            .contains_key(".claude/skills/configure/SKILL.md"));
        assert!(!projection
            .files
            .contains_key(".claude/skills/configure/agents/openai.yaml"));
    }

    #[test]
    fn project_projection_excludes_setup_hive() {
        let projection = compile_projection(Host::Codex, &[]).expect("project projection");

        assert!(!projection
            .files
            .contains_key(".agents/skills/configure/SKILL.md"));
        assert!(projection
            .active_skills
            .skills
            .iter()
            .all(|skill| skill.name != "configure"));
        let metadata = std::str::from_utf8(
            projection
                .files
                .get(".agents/skills/auto-setup-project/agents/openai.yaml")
                .expect("project Skill metadata"),
        )
        .expect("project Skill metadata should be UTF-8");
        assert!(metadata.contains("allow_implicit_invocation: false"));
    }

    #[test]
    fn user_projection_rejects_unknown_skill_selection() {
        let error = compile_user_projection(Host::Antigravity, &["unknown-skill".to_owned()], &[])
            .expect_err("unknown selection must fail");

        assert_eq!(error.code(), "hive.skill-selection-invalid");
        assert_eq!(
            error.message(),
            "selected user Skill is unknown: unknown-skill"
        );
    }

    #[test]
    fn historical_builtin_registry_covers_every_supported_release_exactly() {
        let expected_counts = [
            ("0.1.0", 0),
            ("0.2.0", 0),
            ("0.3.0", 0),
            ("0.4.0", 6),
            ("0.5.0", 9),
            ("0.6.0", 10),
        ];
        for (version, count) in expected_counts {
            let skills = historical_builtin_skills(version).expect("historical release");
            assert_eq!(skills.len(), count);
            assert!(skills.windows(2).all(|pair| pair[0].name < pair[1].name));
            assert!(skills.iter().all(|skill| {
                skill.source_type == SkillSourceType::BuiltIn
                    && skill.consent_digest.is_none()
                    && skill.content_digest.starts_with("sha256:")
            }));
        }
    }

    #[test]
    fn historical_builtin_registry_rejects_unshipped_versions() {
        let error = historical_builtin_skills("0.6.1").expect_err("patch release is not shipped");
        assert_eq!(error.code(), "hive.skill-history-unsupported");
    }

    fn assert_projected_builtin_sources<const N: usize>(expected: [(&str, &[u8], &[u8]); N]) {
        let projection = compile_projection(Host::Codex, &[]).expect("projection");
        for (name, embedded, template) in expected {
            assert_eq!(embedded, template);
            assert_eq!(
                projection
                    .files
                    .get(&format!(".agents/skills/{name}/SKILL.md"))
                    .map(Vec::as_slice),
                Some(embedded)
            );
            let active = projection
                .active_skills
                .skills
                .iter()
                .find(|skill| skill.name == name)
                .expect("active built-in");
            assert_eq!(active.content_digest, sha256_digest(embedded));
        }
    }

    #[test]
    fn v09_skill_sources_templates_embeddings_and_digests_match() {
        let expected = [
            (
                "clean-ai-slop",
                AI_SLOP_CLEANER,
                include_bytes!("../../../harness/template/.agents/skills/clean-ai-slop/SKILL.md")
                    .as_slice(),
            ),
            (
                "research-practices",
                BEST_PRACTICE_RESEARCH,
                include_bytes!(
                    "../../../harness/template/.agents/skills/research-practices/SKILL.md"
                )
                .as_slice(),
            ),
            (
                "import-repository-knowledge",
                KNOWLEDGE_SCAN,
                include_bytes!(
                    "../../../harness/template/.agents/skills/import-repository-knowledge/SKILL.md"
                )
                .as_slice(),
            ),
            (
                "engineer-run",
                LOOP_ENGINEERING,
                include_bytes!("../../../harness/template/.agents/skills/engineer-run/SKILL.md")
                    .as_slice(),
            ),
            (
                "manage-wiki",
                HIVE_WIKI,
                include_bytes!("../../../harness/template/.agents/skills/manage-wiki/SKILL.md")
                    .as_slice(),
            ),
        ];
        assert_projected_builtin_sources(expected);
    }

    #[test]
    fn data_skill_sources_templates_embeddings_and_digests_match() {
        let expected = [
            (
                "verify-package",
                JUDGE_PACKAGE,
                include_bytes!("../../../harness/template/.agents/skills/verify-package/SKILL.md")
                    .as_slice(),
            ),
            (
                "save-progress",
                RUN_CHECKPOINT,
                include_bytes!("../../../harness/template/.agents/skills/save-progress/SKILL.md")
                    .as_slice(),
            ),
            (
                "resume-work",
                RUN_RESUME,
                include_bytes!("../../../harness/template/.agents/skills/resume-work/SKILL.md")
                    .as_slice(),
            ),
            (
                "handoff-role",
                ROLE_HANDOFF,
                include_bytes!("../../../harness/template/.agents/skills/handoff-role/SKILL.md")
                    .as_slice(),
            ),
            (
                "update-hive",
                UPDATE_HARNESS,
                include_bytes!("../../../harness/template/.agents/skills/update-hive/SKILL.md")
                    .as_slice(),
            ),
            (
                "migrate-project",
                MIGRATE_HARNESS,
                include_bytes!("../../../harness/template/.agents/skills/migrate-project/SKILL.md")
                    .as_slice(),
            ),
            (
                "manage-usage",
                USAGE_GUARD,
                include_bytes!("../../../harness/template/.agents/skills/manage-usage/SKILL.md")
                    .as_slice(),
            ),
        ];
        assert_projected_builtin_sources(expected);
    }

    #[test]
    fn approved_optional_skill_requires_every_exact_proof() {
        let source = optional_source();
        let projection =
            compile_projection(Host::Codex, std::slice::from_ref(&source)).expect("projection");
        assert_eq!(
            projection
                .files
                .get(".agents/skills/local-inspect/SKILL.md"),
            Some(&source.skill_md)
        );

        let mut tampered_bytes = source.clone();
        tampered_bytes.skill_md.extend_from_slice(b"\ntampered\n");
        assert_eq!(
            compile_projection(Host::Codex, &[tampered_bytes])
                .expect_err("tampered bytes")
                .code(),
            "hive.optional-skill-inert"
        );

        let mut partial_grant = source.clone();
        partial_grant.consent.approved_capabilities.clear();
        assert_eq!(
            compile_projection(Host::Codex, &[partial_grant])
                .expect_err("partial grant")
                .code(),
            "hive.optional-skill-inert"
        );

        let mut traversal = source.clone();
        traversal.consent.source = "path:../outside/SKILL.md".to_owned();
        assert_eq!(
            compile_projection(Host::Codex, &[traversal])
                .expect_err("source traversal")
                .code(),
            "hive.optional-skill-inert"
        );

        let mut wrong_name = source;
        wrong_name.consent.name = "other-name".to_owned();
        assert_eq!(
            compile_projection(Host::Codex, &[wrong_name])
                .expect_err("frontmatter mismatch")
                .code(),
            "hive.optional-skill-inert"
        );
    }

    #[test]
    fn plain_answer_precedes_explicit_and_candidate_skills() {
        let mut request = routing_request();
        request.plain_answer = true;
        request.explicit_skill = Some("refine-prompt".to_owned());
        request.hive_candidate = Some("search-knowledge".to_owned());

        let result = resolve_route(&request).expect("route");

        assert_eq!(result.route, Route::Direct);
        assert!(result.load_skill_bodies.is_empty());
    }

    #[test]
    fn explicit_skill_precedes_simple_and_external_candidates() {
        let mut request = routing_request();
        request.explicit_skill = Some("search-knowledge".to_owned());
        request.active_hive_skills = vec![builtin_proof("search-knowledge")];
        request.simple_question = true;
        request.external_candidate = Some(ExternalCandidate {
            name: "analyze".to_owned(),
            provided_by: ExternalProvider::Omx,
            compatible: true,
            explicit_selection: true,
        });

        let result = resolve_route(&request).expect("route");

        assert_eq!(result.route, Route::HiveSkill);
        assert_eq!(result.selected_skill.as_deref(), Some("search-knowledge"));
        assert_eq!(result.load_skill_bodies.len(), 1);
    }

    #[test]
    fn inactive_explicit_or_automatic_hive_skills_are_blocked() {
        let mut automatic = routing_request();
        automatic.hive_candidate = Some("arbitrary-skill".to_owned());
        let automatic_decision = resolve_route(&automatic).expect("blocked automatic route");
        assert_eq!(automatic_decision.route, Route::Blocked);
        assert!(automatic_decision.load_skill_bodies.is_empty());

        let mut explicit = routing_request();
        explicit.explicit_skill = Some("arbitrary-skill".to_owned());
        let explicit_decision = resolve_route(&explicit).expect("blocked explicit route");
        assert_eq!(explicit_decision.route, Route::Blocked);
        assert!(explicit_decision.load_skill_bodies.is_empty());
    }

    #[test]
    fn forged_or_unapproved_hive_skill_proofs_are_rejected() {
        let mut forged = routing_request();
        forged.hive_candidate = Some("search-knowledge".to_owned());
        let mut proof = builtin_proof("search-knowledge");
        proof.content_digest = format!("sha256:{}", "0".repeat(64));
        forged.active_hive_skills = vec![proof];
        assert_eq!(
            resolve_route(&forged).expect_err("forged digest").code(),
            "hive.routing-proof-invalid"
        );

        let source = optional_source();
        let mut unapproved = routing_request();
        unapproved.hive_candidate = Some(source.consent.name.clone());
        let mut proof = approved_optional_proof(&source);
        proof.consent = None;
        unapproved.active_hive_skills = vec![proof];
        assert_eq!(
            resolve_route(&unapproved)
                .expect_err("missing consent")
                .code(),
            "hive.routing-proof-invalid"
        );
    }

    #[test]
    fn approved_optional_hive_skill_selects_exactly_one_body() {
        let source = optional_source();
        let mut request = routing_request();
        request.hive_candidate = Some(source.consent.name.clone());
        request.active_hive_skills = vec![approved_optional_proof(&source)];

        let decision = resolve_route(&request).expect("approved optional route");

        assert_eq!(decision.route, Route::HiveSkill);
        assert_eq!(
            decision.selected_skill.as_deref(),
            Some(source.consent.name.as_str())
        );
        assert_eq!(decision.load_skill_bodies, vec![source.consent.name]);
    }

    #[test]
    fn simple_question_blocks_project_dependent_work_without_loading_a_skill() {
        let mut request = routing_request();
        request.explicit_action = None;
        request.simple_question = true;
        request.project_context_required = true;

        let result = resolve_route(&request).expect("route");

        assert_eq!(result.route, Route::Blocked);
        assert_eq!(result.next_action, Some(LogicalAction::RunWork));
        assert!(result.load_skill_bodies.is_empty());
    }

    #[test]
    fn simple_question_gate_precedes_automatic_data_candidates() {
        for skill in ["resume-work", "verify-package"] {
            let mut request = routing_request();
            request.explicit_action = None;
            request.simple_question = true;
            request.hive_candidate = Some(skill.to_owned());
            request.active_hive_skills = vec![builtin_proof("answer"), builtin_proof(skill)];

            let result = resolve_route(&request).expect("simple route");

            assert_eq!(result.route, Route::SimpleQuestion);
            assert_eq!(result.selected_skill.as_deref(), Some("answer"));
            assert_eq!(result.load_skill_bodies, vec!["answer".to_owned()]);
        }
    }

    #[test]
    fn phase_four_data_contract_skills_precede_unrelated_external_workflows() {
        for (skill, action) in [
            ("save-progress", LogicalAction::RunWork),
            ("resume-work", LogicalAction::ResumeWork),
            ("handoff-role", LogicalAction::RunWork),
        ] {
            let mut request = routing_request();
            request.external_candidate = Some(ExternalCandidate {
                name: "analyze".to_owned(),
                provided_by: ExternalProvider::Omx,
                compatible: true,
                explicit_selection: false,
            });
            request.hive_candidate = Some(skill.to_owned());
            request.active_hive_skills = vec![builtin_proof(skill)];

            let result = resolve_route(&request).expect("route");

            assert_eq!(result.route, Route::HiveSkill);
            assert_eq!(result.logical_action, action);
            assert_eq!(result.selected_skill.as_deref(), Some(skill));
            assert_eq!(result.load_skill_bodies, vec![skill.to_owned()]);
        }
    }

    #[test]
    fn explicitly_selected_compatible_external_candidate_precedes_hive_candidates() {
        for (host, provider) in [
            (Host::Codex, ExternalProvider::Omx),
            (Host::Claude, ExternalProvider::Omc),
        ] {
            for skill in ["search-knowledge", "verify-package"] {
                let mut request = routing_request();
                request.host = host;
                request.external_candidate = Some(ExternalCandidate {
                    name: "analyze".to_owned(),
                    provided_by: provider,
                    compatible: true,
                    explicit_selection: true,
                });
                request.hive_candidate = Some(skill.to_owned());
                request.active_hive_skills = vec![builtin_proof(skill)];

                let result = resolve_route(&request).expect("route");

                assert_eq!(result.route, Route::ExternalSkill);
                assert_eq!(result.selected_skill.as_deref(), Some("analyze"));
                assert_eq!(result.load_skill_bodies.len(), 1);
            }
        }
    }

    #[test]
    fn compatible_but_unselected_external_candidate_does_not_precede_hive() {
        let mut request = routing_request();
        request.external_candidate = Some(ExternalCandidate {
            name: "analyze".to_owned(),
            provided_by: ExternalProvider::Omx,
            compatible: true,
            explicit_selection: false,
        });
        request.hive_candidate = Some("search-knowledge".to_owned());
        request.active_hive_skills = vec![builtin_proof("search-knowledge")];

        let result = resolve_route(&request).expect("host-native default route");

        assert_eq!(result.route, Route::HiveSkill);
        assert_eq!(result.selected_skill.as_deref(), Some("search-knowledge"));
        assert_eq!(result.provided_by, Some(RouteProvider::Hive));
    }

    #[test]
    fn refine_and_run_requires_explicit_execution_intent() {
        let mut request = routing_request();
        request.explicit_action = Some(LogicalAction::RefinePrompt);
        request.hive_candidate = Some("refine-prompt".to_owned());
        request.refine_mode = Some(RefineMode::RefineAndRun);

        let decision = resolve_route(&request).expect("blocked route");

        assert_eq!(decision.route, Route::Blocked);
        assert!(decision.load_skill_bodies.is_empty());
    }

    #[test]
    fn refinement_preserves_exact_normalized_constraints() {
        validate_prompt_refinement(&refinement_input(), &refinement_result())
            .expect("valid refinement");
    }

    #[test]
    fn refinement_rejects_missing_constraint() {
        let mut result = refinement_result();
        result.refined_prompt = result
            .refined_prompt
            .replace("Do not add dependencies.", "");

        let error =
            validate_prompt_refinement(&refinement_input(), &result).expect_err("meaning drift");

        assert_eq!(error.code(), "hive.prompt-meaning-drift");
    }

    #[test]
    fn refine_and_run_preserves_user_authority() {
        let mut input = refinement_input();
        input.mode = RefineMode::RefineAndRun;
        input.explicit_run_intent = true;
        let mut result = refinement_result();
        result.mode = RefineMode::RefineAndRun;
        result.execution_authorized = true;
        result.refined_prompt = result
            .refined_prompt
            .replace("Stop before changing the public API.", "");

        let error =
            validate_prompt_refinement(&input, &result).expect_err("missing user authority");

        assert_eq!(error.code(), "hive.prompt-meaning-drift");
    }

    #[test]
    fn refinement_rejects_hidden_execution_and_provider_syntax() {
        let input = refinement_input();
        let mut execution = refinement_result();
        execution.side_effects.model_execution = true;
        assert!(validate_prompt_refinement(&input, &execution).is_err());

        let mut syntax = refinement_result();
        syntax.refined_prompt.push_str("\nRun omx team.");
        let error = validate_prompt_refinement(&input, &syntax).expect_err("provider syntax");
        assert_eq!(error.code(), "hive.prompt-provider-syntax-forbidden");
    }

    #[test]
    fn sufficiently_specific_prompt_uses_exact_growth_budget() {
        let mut input = refinement_input();
        input.sufficiently_specific = true;
        let original_chars = input.original_prompt.chars().count();
        let maximum = original_chars
            .saturating_add(700)
            .max(original_chars.saturating_mul(3).saturating_add(1) / 2);
        let mut result = refinement_result();
        let current = result.refined_prompt.chars().count();
        result
            .refined_prompt
            .push_str(&"x".repeat(maximum.saturating_sub(current)));
        validate_prompt_refinement(&input, &result).expect("exact budget is valid");
        result.refined_prompt.push('x');

        let error = validate_prompt_refinement(&input, &result).expect_err("growth budget");

        assert_eq!(error.code(), "hive.prompt-refinement-invalid");
        assert!(error.message().contains("refinement budget"));
    }
}
