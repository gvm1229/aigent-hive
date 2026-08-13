//! Explicit-consent custom-agent profile projection.
//!
//! This adapter persists only Hive-owned profile and ledger files plus the two
//! exact host-native agent definition paths. It never launches either host.

use crate::run::AdapterError;
use crate::{emit_action_result, ActionResult, Evidence};
use hive_core::custom_agent::{resolve_profiles, route_profile, AgentScope, CustomAgentProfile};
use hive_core::{ensure_consumer_target, sha256_digest};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
Preview, apply, validate, route, or remove one consented Hive custom-agent profile.\n\
\n\
USAGE:\n\
    hive agent preview --profile <profile.json> --root <dir> --output json\n\
    hive agent apply --profile <profile.json> --root <dir> --accept-definition-digest <sha256> --output json\n\
    hive agent validate --profile <profile.json> --root <dir> --output json\n\
    hive agent route --user-root <dir> --project-root <dir> --request <text> --output json\n\
    hive agent remove --role <role-id> --scope <user|project> --root <dir> --accept-definition-digest <sha256> --output json\n";

const PROFILE_DIRECTORY: &str = ".hive/config/custom-subagents";
const LEDGER_PATH: &str = ".hive/config/custom-subagents/OWNERSHIP.json";

#[derive(Debug)]
enum AgentCliError {
    Adapter(AdapterError),
    Input(String),
    Conflict(String),
    Verification(String),
}

impl From<AdapterError> for AgentCliError {
    fn from(error: AdapterError) -> Self {
        Self::Adapter(error)
    }
}

impl AgentCliError {
    fn status(&self) -> &'static str {
        match self {
            Self::Adapter(error) => error.status(),
            Self::Input(_) => "error",
            Self::Conflict(_) => "conflict",
            Self::Verification(_) => "verification-failed",
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Adapter(error) => error.exit_code(),
            Self::Input(_) => 2,
            Self::Conflict(_) => 3,
            Self::Verification(_) => 5,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Adapter(error) => error.code(),
            Self::Input(_) => "hive.invalid-input",
            Self::Conflict(_) => "hive.agent-ownership-conflict",
            Self::Verification(_) => "hive.agent-verification-failed",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Adapter(error) => error.message(),
            Self::Input(message) | Self::Conflict(message) | Self::Verification(message) => message,
        }
    }
}

#[derive(Debug)]
struct ProfileArguments {
    profile: PathBuf,
    root: PathBuf,
    accepted_digest: Option<String>,
}

#[derive(Debug)]
struct RemoveArguments {
    role_id: String,
    scope: AgentScope,
    root: PathBuf,
    accepted_digest: String,
}

#[derive(Debug)]
struct RouteArguments {
    user_root: PathBuf,
    project_root: PathBuf,
    request: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OwnershipLedger {
    schema_version: u32,
    entries: BTreeMap<String, OwnershipEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OwnershipEntry {
    scope: AgentScope,
    definition_digest: String,
    files: BTreeMap<String, String>,
}

impl Default for OwnershipLedger {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: BTreeMap::new(),
        }
    }
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.is_empty()
        || arguments
            .iter()
            .any(|argument| argument == "--help" || argument == "-h")
    {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let action = arguments.first().map_or("Agent", String::as_str);
    let result = execute(arguments).unwrap_or_else(|error| ActionResult {
        schema_version: 1,
        action: match action {
            "preview" => "PreviewAgent",
            "apply" => "ApplyAgent",
            "validate" => "ValidateAgent",
            "route" => "RouteAgent",
            "remove" => "RemoveAgent",
            _ => "Agent",
        },
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    });
    emit_action_result(&result)
}

fn execute(arguments: &[String]) -> Result<ActionResult, AgentCliError> {
    match arguments.first().map(String::as_str) {
        Some("preview") => preview(parse_profile_arguments(&arguments[1..], false)?),
        Some("apply") => apply(parse_profile_arguments(&arguments[1..], true)?),
        Some("validate") => validate(parse_profile_arguments(&arguments[1..], false)?),
        Some("route") => route(parse_route_arguments(&arguments[1..])?),
        Some("remove") => remove(parse_remove_arguments(&arguments[1..])?),
        Some(other) => Err(AgentCliError::Input(format!(
            "unknown agent action: {other}"
        ))),
        None => Err(AgentCliError::Input("missing agent action".to_owned())),
    }
}

fn parse_route_arguments(arguments: &[String]) -> Result<RouteArguments, AgentCliError> {
    let options = options(
        arguments,
        &["--user-root", "--project-root", "--request", "--output"],
    )?;
    if options.get("--output").is_some_and(|value| value != "json") {
        return Err(AgentCliError::Input("output must be json".to_owned()));
    }
    Ok(RouteArguments {
        user_root: PathBuf::from(required(&options, "--user-root")?),
        project_root: PathBuf::from(required(&options, "--project-root")?),
        request: required(&options, "--request")?.to_owned(),
    })
}

fn parse_profile_arguments(
    arguments: &[String],
    require_acceptance: bool,
) -> Result<ProfileArguments, AgentCliError> {
    let options = options(
        arguments,
        &[
            "--profile",
            "--root",
            "--accept-definition-digest",
            "--output",
        ],
    )?;
    if options.get("--output").is_some_and(|value| value != "json") {
        return Err(AgentCliError::Input("output must be json".to_owned()));
    }
    let accepted_digest = options.get("--accept-definition-digest").cloned();
    if require_acceptance && accepted_digest.is_none() {
        return Err(AgentCliError::Input(
            "apply requires --accept-definition-digest".to_owned(),
        ));
    }
    Ok(ProfileArguments {
        profile: PathBuf::from(required(&options, "--profile")?),
        root: PathBuf::from(required(&options, "--root")?),
        accepted_digest,
    })
}

fn parse_remove_arguments(arguments: &[String]) -> Result<RemoveArguments, AgentCliError> {
    let options = options(
        arguments,
        &[
            "--role",
            "--scope",
            "--root",
            "--accept-definition-digest",
            "--output",
        ],
    )?;
    if options.get("--output").is_some_and(|value| value != "json") {
        return Err(AgentCliError::Input("output must be json".to_owned()));
    }
    let scope = match required(&options, "--scope")? {
        "user" => AgentScope::User,
        "project" => AgentScope::Project,
        _ => {
            return Err(AgentCliError::Input(
                "scope must be user or project".to_owned(),
            ))
        }
    };
    Ok(RemoveArguments {
        role_id: required(&options, "--role")?.to_owned(),
        scope,
        root: PathBuf::from(required(&options, "--root")?),
        accepted_digest: required(&options, "--accept-definition-digest")?.to_owned(),
    })
}

fn options(
    arguments: &[String],
    allowed: &[&str],
) -> Result<BTreeMap<String, String>, AgentCliError> {
    if arguments.len() % 2 != 0 {
        return Err(AgentCliError::Input(
            "options require one value each".to_owned(),
        ));
    }
    let mut result = BTreeMap::new();
    for pair in arguments.chunks_exact(2) {
        if !allowed.contains(&pair[0].as_str())
            || result.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(AgentCliError::Input(format!(
                "unsupported or repeated option: {}",
                pair[0]
            )));
        }
    }
    Ok(result)
}

fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, AgentCliError> {
    options
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AgentCliError::Input(format!("missing {name}")))
}

fn parse_profile(path: &Path) -> Result<CustomAgentProfile, AgentCliError> {
    let bytes = fs::read(path).map_err(|error| {
        AgentCliError::Input(format!("cannot read profile {}: {error}", path.display()))
    })?;
    let profile = CustomAgentProfile::parse_json(&bytes)
        .map_err(|error| AgentCliError::Verification(error.to_string()))?;
    validate_reserved_authority(&profile)?;
    Ok(profile)
}

fn validate_reserved_authority(profile: &CustomAgentProfile) -> Result<(), AgentCliError> {
    if !profile.reserved {
        return Ok(());
    }
    let canonical = CustomAgentProfile::parse_json(include_bytes!(
        "../../../harness/roles/hive-independent-judge.json"
    ))
    .map_err(|error| AgentCliError::Verification(error.to_string()))?;
    if profile != &canonical {
        return Err(AgentCliError::Conflict(
            "reserved Judge profile may only use the bundled authoritative definition".to_owned(),
        ));
    }
    Ok(())
}

fn projection_files(
    profile: &CustomAgentProfile,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, AgentCliError> {
    let mut files = BTreeMap::new();
    files.insert(
        PathBuf::from(PROFILE_DIRECTORY).join(format!("{}.json", profile.role_id)),
        canonical_profile(profile)?,
    );
    files.insert(
        PathBuf::from(".codex/agents").join(format!("{}.toml", profile.role_id)),
        profile
            .render("codex")
            .map_err(|error| AgentCliError::Verification(error.to_string()))?,
    );
    files.insert(
        PathBuf::from(".claude/agents").join(format!("{}.md", profile.role_id)),
        profile
            .render("claude")
            .map_err(|error| AgentCliError::Verification(error.to_string()))?,
    );
    Ok(files)
}

fn canonical_profile(profile: &CustomAgentProfile) -> Result<Vec<u8>, AgentCliError> {
    serde_json_canonicalizer::to_vec(profile)
        .map(|mut bytes| {
            bytes.push(b'\n');
            bytes
        })
        .map_err(|error| AgentCliError::Verification(error.to_string()))
}

fn preview(arguments: ProfileArguments) -> Result<ActionResult, AgentCliError> {
    let profile = parse_profile(&arguments.profile)?;
    prepare_root(&arguments.root, profile.scope)?;
    let files = projection_files(&profile)?;
    Ok(success(
        "PreviewAgent",
        "hive.agent-preview",
        "custom agent projection preview prepared",
        Vec::new(),
        &profile,
        &files,
        Some("apply requires the displayed definition digest".to_owned()),
    ))
}

fn validate(arguments: ProfileArguments) -> Result<ActionResult, AgentCliError> {
    let profile = parse_profile(&arguments.profile)?;
    let root = prepare_root(&arguments.root, profile.scope)?;
    let files = projection_files(&profile)?;
    let ledger = read_ledger(&root)?;
    let entry = ledger.entries.get(&profile.role_id).ok_or_else(|| {
        AgentCliError::Verification("profile has no Hive ownership ledger entry".to_owned())
    })?;
    if entry.scope != profile.scope || entry.definition_digest != profile.definition_digest {
        return Err(AgentCliError::Verification(
            "profile ledger does not bind this definition".to_owned(),
        ));
    }
    validate_owned_files(&root, entry, &files)?;
    Ok(success(
        "ValidateAgent",
        "hive.agent-valid",
        "custom agent profile and owned projections are valid",
        Vec::new(),
        &profile,
        &files,
        None,
    ))
}

fn route(arguments: RouteArguments) -> Result<ActionResult, AgentCliError> {
    let user_root = prepare_root(&arguments.user_root, AgentScope::User)?;
    let project_root = prepare_root(&arguments.project_root, AgentScope::Project)?;
    let profiles = resolve_profiles(
        &read_profiles(&user_root, AgentScope::User)?,
        &read_profiles(&project_root, AgentScope::Project)?,
    )
    .map_err(|error| AgentCliError::Verification(error.to_string()))?;
    let selected = route_profile(&profiles, &arguments.request);
    let evidence = selected.map_or_else(Vec::new, |profile| {
        vec![Evidence {
            kind: "custom-agent-profile",
            locator: profile.role_id.clone(),
            digest: profile.definition_digest.clone(),
        }]
    });
    Ok(ActionResult {
        schema_version: 1,
        action: "RouteAgent",
        status: "success",
        exit_code: 0,
        code: "hive.agent-route",
        message: "custom agent route resolved without host launch".to_owned(),
        changed_paths: Vec::new(),
        evidence,
        next_action: None,
        data: Some(json!({
            "role_id": selected.map(|profile| &profile.role_id),
            "scope": selected.map(|profile| profile.scope),
            "definition_digest": selected.map(|profile| &profile.definition_digest),
            "host_mappings": selected.map(|profile| &profile.host_mappings),
            "spawned": false,
        })),
    })
}

fn apply(arguments: ProfileArguments) -> Result<ActionResult, AgentCliError> {
    let profile = parse_profile(&arguments.profile)?;
    let accepted = arguments
        .accepted_digest
        .expect("apply acceptance is required by parser");
    if accepted != profile.definition_digest {
        return Err(AgentCliError::Conflict(
            "accepted definition digest does not match profile".to_owned(),
        ));
    }
    let root = prepare_root(&arguments.root, profile.scope)?;
    let files = projection_files(&profile)?;
    let mut ledger = read_ledger(&root)?;
    let previous = ledger.entries.get(&profile.role_id).cloned();
    if let Some(previous) = previous.as_ref() {
        if previous.scope != profile.scope {
            return Err(AgentCliError::Conflict(
                "role id is already owned in another scope".to_owned(),
            ));
        }
    }
    for (path, bytes) in &files {
        validate_replacement(&root, path, bytes, previous.as_ref())?;
    }
    let mut changed = Vec::new();
    for (path, bytes) in &files {
        if write_if_changed(&root, path, bytes)? {
            changed.push(portable(path));
        }
    }
    let entry = OwnershipEntry {
        scope: profile.scope,
        definition_digest: profile.definition_digest.clone(),
        files: files
            .iter()
            .map(|(path, bytes)| (portable(path), sha256_digest(bytes)))
            .collect(),
    };
    ledger.entries.insert(profile.role_id.clone(), entry);
    let ledger_bytes = serde_json_canonicalizer::to_vec(&ledger)
        .map_err(|error| AgentCliError::Verification(error.to_string()))?;
    if write_if_changed(&root, Path::new(LEDGER_PATH), &ledger_bytes)? {
        changed.push(LEDGER_PATH.to_owned());
    }
    Ok(success(
        "ApplyAgent",
        "hive.agent-applied",
        "consented custom agent profile projection applied",
        changed,
        &profile,
        &files,
        None,
    ))
}

fn remove(arguments: RemoveArguments) -> Result<ActionResult, AgentCliError> {
    let root = prepare_root(&arguments.root, arguments.scope)?;
    let mut ledger = read_ledger(&root)?;
    let entry = ledger
        .entries
        .get(&arguments.role_id)
        .cloned()
        .ok_or_else(|| {
            AgentCliError::Verification("profile has no Hive ownership ledger entry".to_owned())
        })?;
    if entry.scope != arguments.scope || entry.definition_digest != arguments.accepted_digest {
        return Err(AgentCliError::Conflict(
            "accepted digest does not authorize this profile removal".to_owned(),
        ));
    }
    let mut changed = Vec::new();
    for (path, expected_digest) in &entry.files {
        let path = Path::new(path);
        let absolute = safe_path(&root, path)?;
        match fs::read(&absolute) {
            Ok(bytes) if sha256_digest(&bytes) == *expected_digest => {
                fs::remove_file(&absolute).map_err(|error| {
                    AgentCliError::Conflict(format!(
                        "cannot remove owned projection {}: {error}",
                        path.display()
                    ))
                })?;
                changed.push(portable(path));
            }
            Ok(_) => {
                return Err(AgentCliError::Conflict(format!(
                    "refusing to remove changed or foreign projection {}",
                    path.display()
                )))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(AgentCliError::Conflict(format!(
                    "cannot inspect projection {}: {error}",
                    path.display()
                )))
            }
        }
    }
    ledger.entries.remove(&arguments.role_id);
    let ledger_bytes = serde_json_canonicalizer::to_vec(&ledger)
        .map_err(|error| AgentCliError::Verification(error.to_string()))?;
    if write_if_changed(&root, Path::new(LEDGER_PATH), &ledger_bytes)? {
        changed.push(LEDGER_PATH.to_owned());
    }
    Ok(ActionResult {
        schema_version: 1,
        action: "RemoveAgent",
        status: "success",
        exit_code: 0,
        code: "hive.agent-removed",
        message: "consented custom agent profile removed without foreign overwrite".to_owned(),
        changed_paths: changed,
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({"role_id": arguments.role_id})),
    })
}

fn success(
    action: &'static str,
    code: &'static str,
    message: &'static str,
    changed_paths: Vec<String>,
    profile: &CustomAgentProfile,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    next_action: Option<String>,
) -> ActionResult {
    let projections = files
        .iter()
        .map(|(path, bytes)| json!({"path": portable(path), "digest": sha256_digest(bytes)}))
        .collect::<Vec<_>>();
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![Evidence {
            kind: "custom-agent-profile",
            locator: profile.role_id.clone(),
            digest: profile.definition_digest.clone(),
        }],
        next_action,
        data: Some(
            json!({"scope": profile.scope, "definition_digest": profile.definition_digest, "projections": projections, "consent_required": true, "spawned": false}),
        ),
    }
}

fn prepare_root(root: &Path, scope: AgentScope) -> Result<PathBuf, AgentCliError> {
    if scope == AgentScope::Project {
        ensure_consumer_target(root).map_err(|error| AgentCliError::Conflict(error.to_string()))?;
    }
    let metadata = fs::symlink_metadata(root).map_err(|error| {
        AgentCliError::Input(format!(
            "cannot inspect profile root {}: {error}",
            root.display()
        ))
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(AgentCliError::Conflict(
            "profile root must be a non-symlink directory".to_owned(),
        ));
    }
    fs::canonicalize(root)
        .map_err(|error| AgentCliError::Input(format!("cannot canonicalize profile root: {error}")))
}

fn read_ledger(root: &Path) -> Result<OwnershipLedger, AgentCliError> {
    let path = safe_path(root, Path::new(LEDGER_PATH))?;
    match fs::read(path) {
        Ok(bytes) => {
            let ledger: OwnershipLedger = serde_json::from_slice(&bytes).map_err(|error| {
                AgentCliError::Verification(format!(
                    "malformed custom agent ownership ledger: {error}"
                ))
            })?;
            if ledger.schema_version != 1 {
                return Err(AgentCliError::Verification(
                    "unsupported custom agent ownership ledger".to_owned(),
                ));
            }
            Ok(ledger)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(OwnershipLedger::default())
        }
        Err(error) => Err(AgentCliError::Conflict(format!(
            "cannot read custom agent ownership ledger: {error}"
        ))),
    }
}

fn read_profiles(
    root: &Path,
    expected_scope: AgentScope,
) -> Result<Vec<CustomAgentProfile>, AgentCliError> {
    let directory = safe_path(root, Path::new(PROFILE_DIRECTORY))?;
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(AgentCliError::Conflict(format!(
                "cannot read custom agent profile directory: {error}"
            )))
        }
    };
    let mut profiles = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            AgentCliError::Conflict(format!("cannot inspect custom agent profile: {error}"))
        })?;
        let file_type = entry.file_type().map_err(|error| {
            AgentCliError::Conflict(format!("cannot inspect custom agent profile type: {error}"))
        })?;
        if entry.file_name() == "OWNERSHIP.json" {
            if file_type.is_symlink() || !file_type.is_file() {
                return Err(AgentCliError::Conflict(
                    "custom agent ownership ledger is not a regular file".to_owned(),
                ));
            }
            continue;
        }
        if file_type.is_symlink() || !file_type.is_file() {
            return Err(AgentCliError::Conflict(
                "custom agent profile directory contains a non-regular entry".to_owned(),
            ));
        }
        if entry
            .path()
            .extension()
            .is_none_or(|extension| extension != "json")
        {
            return Err(AgentCliError::Verification(
                "custom agent profile directory contains a non-JSON definition".to_owned(),
            ));
        }
        let profile = parse_profile(&entry.path())?;
        if profile.scope != expected_scope {
            return Err(AgentCliError::Verification(
                "custom agent profile is stored in the wrong scope".to_owned(),
            ));
        }
        profiles.push(profile);
    }
    Ok(profiles)
}

fn validate_owned_files(
    root: &Path,
    entry: &OwnershipEntry,
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), AgentCliError> {
    if entry.files.len() != files.len() {
        return Err(AgentCliError::Verification(
            "profile ledger projection count differs".to_owned(),
        ));
    }
    for (path, expected) in files {
        let key = portable(path);
        if entry.files.get(&key) != Some(&sha256_digest(expected)) {
            return Err(AgentCliError::Verification(format!(
                "profile ledger does not bind {key}"
            )));
        }
        let actual = fs::read(safe_path(root, path)?).map_err(|error| {
            AgentCliError::Verification(format!("cannot read owned projection {key}: {error}"))
        })?;
        if actual != *expected {
            return Err(AgentCliError::Verification(format!(
                "owned projection drift: {key}"
            )));
        }
    }
    Ok(())
}

fn validate_replacement(
    root: &Path,
    path: &Path,
    expected: &[u8],
    previous: Option<&OwnershipEntry>,
) -> Result<(), AgentCliError> {
    let absolute = safe_path(root, path)?;
    match fs::read(&absolute) {
        Ok(actual) if actual == expected => Ok(()),
        Ok(actual) => {
            let key = portable(path);
            let owned = previous
                .and_then(|entry| entry.files.get(&key))
                .is_some_and(|digest| digest == &sha256_digest(&actual));
            if owned {
                Ok(())
            } else {
                Err(AgentCliError::Conflict(format!(
                    "refusing to replace foreign or changed host projection {key}"
                )))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AgentCliError::Conflict(format!(
            "cannot inspect projection {}: {error}",
            path.display()
        ))),
    }
}

fn write_if_changed(root: &Path, relative: &Path, bytes: &[u8]) -> Result<bool, AgentCliError> {
    let path = safe_path(root, relative)?;
    if fs::read(&path).ok().as_deref() == Some(bytes) {
        return Ok(false);
    }
    let parent = path
        .parent()
        .ok_or_else(|| AgentCliError::Conflict("projection has no parent".to_owned()))?;
    fs::create_dir_all(parent).map_err(|error| {
        AgentCliError::Conflict(format!(
            "cannot create Hive-owned projection directory: {error}"
        ))
    })?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            AgentCliError::Conflict(format!("cannot create projection temporary: {error}"))
        })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| AgentCliError::Conflict(format!("cannot write projection: {error}")))?;
    fs::rename(&temporary, &path)
        .map_err(|error| AgentCliError::Conflict(format!("cannot publish projection: {error}")))?;
    Ok(true)
}

fn safe_path(root: &Path, relative: &Path) -> Result<PathBuf, AgentCliError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AgentCliError::Input(
            "custom agent projection path is unsafe".to_owned(),
        ));
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            unreachable!()
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(AgentCliError::Conflict(format!(
                    "custom agent projection crosses symlink: {}",
                    current.display()
                )))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(AgentCliError::Conflict(format!(
                    "cannot inspect projection path: {error}"
                )))
            }
        }
    }
    Ok(current)
}

fn portable(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hive-agent-cli-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("root");
        path
    }

    fn profile() -> CustomAgentProfile {
        CustomAgentProfile::parse_json(include_bytes!(
            "../../../harness/roles/hive-complex-implementer.json"
        ))
        .expect("profile")
    }

    fn apply_profile(root: &Path, profile: &CustomAgentProfile, name: &str) {
        let input = root.join(name);
        fs::write(&input, canonical_profile(profile).expect("profile bytes")).expect("input");
        apply(ProfileArguments {
            profile: input,
            root: root.to_path_buf(),
            accepted_digest: Some(profile.definition_digest.clone()),
        })
        .expect("apply");
    }

    #[test]
    fn apply_validate_and_remove_preserve_foreign_bytes() {
        let root = temporary_root();
        let profile = profile();
        let input = root.join("input.json");
        fs::write(&input, canonical_profile(&profile).expect("profile bytes")).expect("input");
        let arguments = ProfileArguments {
            profile: input.clone(),
            root: root.clone(),
            accepted_digest: Some(profile.definition_digest.clone()),
        };
        apply(arguments).expect("apply");
        validate(ProfileArguments {
            profile: input,
            root: root.clone(),
            accepted_digest: None,
        })
        .expect("validate");
        let foreign = root.join(".codex/agents/hive-complex-implementer.toml");
        fs::write(&foreign, b"foreign\n").expect("foreign update");
        assert!(remove(RemoveArguments {
            role_id: profile.role_id,
            scope: profile.scope,
            root: root.clone(),
            accepted_digest: profile.definition_digest
        })
        .is_err());
        assert_eq!(fs::read(&foreign).expect("foreign bytes"), b"foreign\n");
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn apply_requires_exact_consent_digest() {
        let root = temporary_root();
        let profile = profile();
        let input = root.join("input.json");
        fs::write(&input, canonical_profile(&profile).expect("profile bytes")).expect("input");
        assert!(apply(ProfileArguments {
            profile: input,
            root: root.clone(),
            accepted_digest: Some("sha256:bad".to_owned())
        })
        .is_err());
        assert!(!root
            .join(".codex/agents/hive-complex-implementer.toml")
            .exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn apply_rejects_modified_reserved_judge_definition() {
        let root = temporary_root();
        let mut profile = CustomAgentProfile::parse_json(include_bytes!(
            "../../../harness/roles/hive-independent-judge.json"
        ))
        .expect("judge");
        profile.description =
            "A changed Judge definition must never replace the authority.".to_owned();
        profile.definition_digest = profile.computed_digest().expect("digest");
        let input = root.join("input.json");
        fs::write(&input, canonical_profile(&profile).expect("profile bytes")).expect("input");
        assert!(apply(ProfileArguments {
            profile: input,
            root: root.clone(),
            accepted_digest: Some(profile.definition_digest),
        })
        .is_err());
        assert!(!root
            .join(".codex/agents/hive-independent-judge.toml")
            .exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn route_uses_project_profile_before_user_profile_without_launching() {
        let user_root = temporary_root();
        let project_root = temporary_root();
        let mut user_profile = profile();
        user_profile.scope = AgentScope::User;
        user_profile.description = "User scope complex implementation profile.".to_owned();
        user_profile.definition_digest = user_profile.computed_digest().expect("user digest");
        let project_profile = profile();
        apply_profile(&user_root, &user_profile, "user.json");
        apply_profile(&project_root, &project_profile, "project.json");

        let result = route(RouteArguments {
            user_root: user_root.clone(),
            project_root: project_root.clone(),
            request: "Please perform a complex implementation.".to_owned(),
        })
        .expect("route");
        let data = result.data.expect("data");
        assert_eq!(data["role_id"], project_profile.role_id);
        assert_eq!(data["scope"], "project");
        assert_eq!(data["definition_digest"], project_profile.definition_digest);
        assert_eq!(data["spawned"], false);
        let excluded = route(RouteArguments {
            user_root: user_root.clone(),
            project_root: project_root.clone(),
            request: "format only complex implementation".to_owned(),
        })
        .expect("negative route");
        assert_eq!(excluded.data.expect("negative data")["role_id"], serde_json::Value::Null);
        fs::remove_dir_all(user_root).expect("user cleanup");
        fs::remove_dir_all(project_root).expect("project cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn apply_rejects_symlinked_projection_ancestor() {
        use std::os::unix::fs::symlink;

        let root = temporary_root();
        let outside = temporary_root();
        fs::create_dir_all(root.join(".codex")).expect("codex parent");
        symlink(&outside, root.join(".codex/agents")).expect("symlink");
        let profile = profile();
        let input = root.join("input.json");
        fs::write(&input, canonical_profile(&profile).expect("profile bytes")).expect("input");
        assert!(apply(ProfileArguments {
            profile: input,
            root: root.clone(),
            accepted_digest: Some(profile.definition_digest),
        })
        .is_err());
        assert!(!outside.join("hive-complex-implementer.toml").exists());
        fs::remove_dir_all(root).expect("cleanup");
        fs::remove_dir_all(outside).expect("cleanup");
    }
}
