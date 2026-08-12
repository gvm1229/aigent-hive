use cap_fs_ext::{DirExt, OpenOptionsFollowExt};
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use hive_core::{
    ensure_consumer_target, is_hive_directive_projection_path, is_hive_skill_projection_path,
    sha256_digest, validate_hive_directive_projection_relative,
    validate_hive_skill_projection_relative, validate_project_relative,
};
use hive_projection::historical_builtin_skills;
use hive_render::{
    historical_project_upgrade_candidate_in, project_upgrade_candidate_in, HistoricalProjectBase,
    RenderError,
};
use hive_update::{three_way_merge, MergeDisposition, UpdateError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
#[cfg(test)]
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const BASE_PATH: &str = ".hive/config/project-base.json";
const OVERRIDES_PATH: &str = ".hive/config/project-overrides.json";
const JOURNAL_PATH: &str = ".hive/runtime/project-upgrade-journal.json";
const LEGACY_DERIVED_INDEX_PATHS: [&str; 4] = [
    ".hive/index/hive.sqlite3",
    ".hive/index/hive.sqlite3-wal",
    ".hive/index/hive.sqlite3-shm",
    ".hive/index/.stale",
];
const MAX_LEDGER_BYTES: u64 = 8 * 1024 * 1024;
const CLAIMED_JOURNAL_LOCATOR_MARKER: &str = "; claimed journal retained at ";

#[derive(Clone, Copy, Eq, PartialEq)]
enum CommandMode {
    Scan,
    DryRun,
    Apply,
    Validate,
    Recover,
}

#[derive(Serialize)]
struct ProjectResult {
    schema_version: u32,
    action: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
    changed_paths: Vec<String>,
    evidence: Vec<ProjectEvidence>,
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize)]
struct ProjectEvidence {
    kind: &'static str,
    locator: String,
    digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaseLedger {
    schema_version: u32,
    product_version: String,
    files: Vec<BaseFile>,
    ledger_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaseFile {
    path: String,
    kind: String,
    content_digest: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OverrideFile {
    path: String,
    base_digest: String,
    local_digest: String,
    omitted_incoming_hunks: usize,
}

#[derive(Debug, Serialize)]
struct PathReport {
    path: String,
    base_digest: Option<String>,
    local_digest: Option<String>,
    incoming_digest: Option<String>,
    final_digest: Option<String>,
    disposition: MergeDisposition,
    omitted_incoming_hunks: usize,
    local_priority: bool,
}

struct UpgradePlan {
    source_version: String,
    target_version: String,
    reports: Vec<PathReport>,
    final_files: BTreeMap<String, Option<Vec<u8>>>,
    expected_before: BTreeMap<String, ExpectedBefore>,
    changed_paths: Vec<String>,
    plan_digest: String,
}

#[derive(Clone)]
struct ExpectedBefore {
    bytes: Option<Vec<u8>>,
    digest: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpgradeJournal {
    schema_version: u32,
    transaction_id: String,
    plan_digest: String,
    backup_root: String,
    changes: Vec<JournalChange>,
    journal_digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalChange {
    path: String,
    claim_path: String,
    before_digest: Option<String>,
    after_digest: Option<String>,
    backup_path: Option<String>,
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    let result = parse(arguments).and_then(|(target, mode)| match mode {
        CommandMode::Recover => {
            ensure_consumer_target(&target)
                .map_err(|error| UpdateError::Input(error.to_string()))?;
            let target_dir = open_target_capability(&target)?;
            ensure_pinned_consumer_target(&target_dir)?;
            recover(&target_dir)
        }
        CommandMode::Scan | CommandMode::DryRun | CommandMode::Validate | CommandMode::Apply => {
            ensure_consumer_target(&target)
                .map_err(|error| UpdateError::Input(error.to_string()))?;
            let target_dir = open_target_capability(&target)?;
            ensure_pinned_consumer_target(&target_dir)?;
            if read_bounded_optional(&target_dir, Path::new(JOURNAL_PATH), MAX_LEDGER_BYTES)?
                .is_some()
            {
                return Err(UpdateError::RecoveryRequired(
                    "project upgrade journal requires explicit --recover".to_owned(),
                ));
            }
            let plan = prepare(&target_dir)?;
            if mode == CommandMode::Validate && !plan.changed_paths.is_empty() {
                return Err(UpdateError::Verification(
                    "installed project harness has applicable upgrades".to_owned(),
                ));
            }
            if mode == CommandMode::Apply {
                apply(&target_dir, &plan)
            } else {
                Ok(plan_result(&plan, mode))
            }
        }
    });
    let result = result.unwrap_or_else(|error| failure(&error));
    emit(&result);
    ExitCode::from(result.exit_code)
}

pub(crate) fn authenticate_legacy_knowledge_target(target: &Path) -> Result<(), UpdateError> {
    ensure_consumer_target(target).map_err(|error| UpdateError::Input(error.to_string()))?;
    let target_dir = open_target_capability(target)?;
    ensure_pinned_consumer_target(&target_dir)?;
    let candidate = project_upgrade_candidate_in(&target_dir).map_err(render_error)?;
    let ledger = read_base_ledger(&target_dir, &candidate.files)?.ok_or_else(|| {
        UpdateError::Verification(
            "legacy project-local knowledge requires an authenticated project base".to_owned(),
        )
    })?;
    let version = release_version(&ledger.product_version).ok_or_else(|| {
        UpdateError::Verification("project base product version is invalid".to_owned())
    })?;
    if version > (0, 7, 0) {
        return Err(UpdateError::Unsupported(
            "project-local SQLite is supported only for authenticated Hive 0.7 or earlier; use --user-root"
                .to_owned(),
        ));
    }
    Ok(())
}

fn parse(arguments: &[String]) -> Result<(PathBuf, CommandMode), UpdateError> {
    if arguments.first().map(String::as_str) != Some("upgrade") {
        return Err(UpdateError::Input(
            "project requires the upgrade action".to_owned(),
        ));
    }
    let mut target = None;
    let mut mode = None;
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        match option {
            "--scan" | "--dry-run" | "--apply" | "--validate" | "--recover" => {
                let candidate = match option {
                    "--scan" => CommandMode::Scan,
                    "--dry-run" => CommandMode::DryRun,
                    "--apply" => CommandMode::Apply,
                    "--validate" => CommandMode::Validate,
                    "--recover" => CommandMode::Recover,
                    _ => unreachable!(),
                };
                if mode.replace(candidate).is_some() {
                    return Err(UpdateError::Input(
                        "project upgrade requires exactly one mode".to_owned(),
                    ));
                }
                index += 1;
            }
            "--target" | "--output" => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| UpdateError::Input(format!("missing value for {option}")))?;
                if option == "--target" {
                    if target.replace(PathBuf::from(value)).is_some() {
                        return Err(UpdateError::Input(
                            "duplicate project upgrade target".to_owned(),
                        ));
                    }
                } else if output.replace(value.as_str()).is_some() {
                    return Err(UpdateError::Input(
                        "duplicate project upgrade output".to_owned(),
                    ));
                }
                index += 2;
            }
            _ => {
                return Err(UpdateError::Input(format!(
                    "unknown project upgrade option: {option}"
                )));
            }
        }
    }
    if output != Some("json") {
        return Err(UpdateError::Input(
            "project upgrade requires --output json".to_owned(),
        ));
    }
    Ok((
        target.ok_or_else(|| UpdateError::Input("missing --target".to_owned()))?,
        mode.ok_or_else(|| {
            UpdateError::Input(
                "project upgrade requires --scan, --dry-run, --apply, --validate, or --recover"
                    .to_owned(),
            )
        })?,
    ))
}

#[allow(clippy::too_many_lines)]
fn prepare(target: &Dir) -> Result<UpgradePlan, UpdateError> {
    let candidate = project_upgrade_candidate_in(target).map_err(render_error)?;
    let base = read_base_ledger(target, &candidate.files)?;
    let source_version = base.as_ref().map_or_else(
        || "legacy-unbased".to_owned(),
        |ledger| ledger.product_version.clone(),
    );
    let base_files = base
        .as_ref()
        .map(|ledger| {
            ledger
                .files
                .iter()
                .map(|entry| (entry.path.clone(), entry.content.as_bytes().to_vec()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut paths = base_files.keys().cloned().collect::<BTreeSet<_>>();
    paths.extend(candidate.files.keys().cloned());
    let mut reports = Vec::new();
    let mut final_files = BTreeMap::new();
    let mut overrides = Vec::new();
    for path in paths {
        validate_merge_path(Path::new(&path))?;
        let base_bytes = base_files.get(&path).map(Vec::as_slice);
        let local = read_mergeable(target, &path)?;
        let incoming = candidate.files.get(&path).map(Vec::as_slice);
        if base.is_none() && local.as_deref() == incoming {
            // Safe migration for an unmodified installation created before the
            // project-base ledger existed.
        } else if base.is_none() && local.is_some() {
            return Err(UpdateError::Conflict(format!(
                "modified legacy projection cannot be upgraded without an authenticated base: {path}"
            )));
        }
        let effective_base = if base.is_none() { incoming } else { base_bytes };
        let merged = three_way_merge(Path::new(&path), effective_base, local.as_deref(), incoming)?;
        let final_full = materialize_final(target, &path, merged.bytes.as_deref())?;
        let local_full = read_full_optional(target, Path::new(&path))?;
        let changed = local_full.as_deref() != final_full.as_deref();
        if changed {
            final_files.insert(path.clone(), final_full);
        }
        if let (Some(incoming), Some(final_bytes)) = (incoming, merged.bytes.as_deref()) {
            if final_bytes != incoming {
                overrides.push(OverrideFile {
                    path: path.clone(),
                    base_digest: sha256_digest(incoming),
                    local_digest: sha256_digest(final_bytes),
                    omitted_incoming_hunks: merged.omitted_incoming_hunks,
                });
            }
        }
        reports.push(PathReport {
            path,
            base_digest: effective_base.map(sha256_digest),
            local_digest: local.as_deref().map(sha256_digest),
            incoming_digest: incoming.map(sha256_digest),
            final_digest: merged.bytes.as_deref().map(sha256_digest),
            disposition: merged.disposition,
            omitted_incoming_hunks: merged.omitted_incoming_hunks,
            local_priority: merged.local_priority,
        });
    }
    for (path, bytes) in candidate.support_files {
        insert_exact_if_changed(target, &mut final_files, path, bytes)?;
    }
    insert_exact_if_changed(
        target,
        &mut final_files,
        BASE_PATH.to_owned(),
        candidate.base_ledger,
    )?;
    let override_bytes = render_override_ledger(&candidate.product_version, overrides)?;
    insert_exact_if_changed(
        target,
        &mut final_files,
        OVERRIDES_PATH.to_owned(),
        override_bytes,
    )?;
    plan_legacy_derived_index_cleanup(
        target,
        &mut final_files,
        &source_version,
        &candidate.product_version,
    )?;
    let mut changed_paths = final_files.keys().cloned().collect::<Vec<_>>();
    changed_paths.sort();
    let expected_before = changed_paths
        .iter()
        .map(|path| {
            let bytes = read_full_optional(target, Path::new(path))?;
            Ok((
                path.clone(),
                ExpectedBefore {
                    digest: bytes.as_deref().map(sha256_digest),
                    bytes,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, UpdateError>>()?;
    let plan_value = json!({
        "schema_version": 1,
        "source_version": source_version,
        "target_version": candidate.product_version,
        "reports": reports,
        "changes": plan_changes(&changed_paths, &final_files, &expected_before)
    });
    let canonical = serde_json_canonicalizer::to_vec(&plan_value)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    Ok(UpgradePlan {
        source_version,
        target_version: candidate.product_version,
        reports,
        final_files,
        expected_before,
        changed_paths,
        plan_digest: sha256_digest(&canonical),
    })
}

fn plan_legacy_derived_index_cleanup(
    target: &Dir,
    final_files: &mut BTreeMap<String, Option<Vec<u8>>>,
    source_version: &str,
    target_version: &str,
) -> Result<(), UpdateError> {
    if !is_legacy_070_migration(source_version, target_version) {
        return Ok(());
    }
    for path in LEGACY_DERIVED_INDEX_PATHS {
        if read_full_optional(target, Path::new(path))?.is_some() {
            final_files.insert(path.to_owned(), None);
        }
    }
    Ok(())
}

fn is_legacy_070_migration(source_version: &str, target_version: &str) -> bool {
    source_version == "0.7.0"
        && release_version(target_version).is_some_and(|target| target > (0, 7, 0))
}

fn release_version(version: &str) -> Option<(u64, u64, u64)> {
    let release = version
        .split_once('-')
        .map_or(version, |(release, _)| release);
    let mut components = release.split('.');
    let version = (
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
    );
    components.next().is_none().then_some(version)
}

fn read_base_ledger(
    target: &Dir,
    incoming: &BTreeMap<String, Vec<u8>>,
) -> Result<Option<BaseLedger>, UpdateError> {
    let Some(bytes) = read_bounded_optional(target, Path::new(BASE_PATH), MAX_LEDGER_BYTES)? else {
        return Ok(None);
    };
    let ledger: BaseLedger = serde_json::from_slice(&bytes)
        .map_err(|error| UpdateError::Verification(format!("invalid project base: {error}")))?;
    if ledger.schema_version != 1 || ledger.product_version.is_empty() {
        return Err(UpdateError::Verification(
            "project base metadata is invalid".to_owned(),
        ));
    }
    let mut value =
        serde_json::to_value(&ledger).map_err(|error| UpdateError::Internal(error.to_string()))?;
    value
        .as_object_mut()
        .expect("base ledger is an object")
        .remove("ledger_digest");
    let canonical = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    if ledger.ledger_digest != sha256_digest(&canonical) {
        return Err(UpdateError::Verification(
            "project base digest is invalid".to_owned(),
        ));
    }
    if bytes != canonical_json(&ledger)? {
        return Err(UpdateError::Verification(
            "project base bytes are not canonical".to_owned(),
        ));
    }
    let mut previous = None;
    for entry in &ledger.files {
        validate_merge_path(Path::new(&entry.path))?;
        if previous.is_some_and(|path: &str| path >= entry.path.as_str())
            || entry.content_digest != sha256_digest(entry.content.as_bytes())
            || !matches!(entry.kind.as_str(), "skill" | "directive" | "shared-marker")
        {
            return Err(UpdateError::Verification(
                "project base file inventory is invalid".to_owned(),
            ));
        }
        previous = Some(&entry.path);
    }
    if ledger.product_version == env!("CARGO_PKG_VERSION") {
        authenticate_current_base(&ledger, incoming)?;
    } else {
        authenticate_historical_base(target, &ledger)?;
    }
    Ok(Some(ledger))
}

fn authenticate_current_base(
    ledger: &BaseLedger,
    incoming: &BTreeMap<String, Vec<u8>>,
) -> Result<(), UpdateError> {
    if ledger.files.len() != incoming.len() {
        return Err(UpdateError::Verification(
            "current-version project base inventory differs from the authenticated binary"
                .to_owned(),
        ));
    }
    for entry in &ledger.files {
        if incoming.get(&entry.path).map(Vec::as_slice) != Some(entry.content.as_bytes())
            || entry.kind != expected_base_kind(&entry.path)?
        {
            return Err(UpdateError::Verification(format!(
                "current-version project base differs from the authenticated binary: {}",
                entry.path
            )));
        }
    }
    Ok(())
}

fn expected_base_kind(path: &str) -> Result<&'static str, UpdateError> {
    if is_shared(path) {
        Ok("shared-marker")
    } else if is_hive_skill_projection_path(Path::new(path)) {
        Ok("skill")
    } else if is_hive_directive_projection_path(Path::new(path)) {
        Ok("directive")
    } else {
        Err(UpdateError::Verification(format!(
            "project base contains an unsupported authenticated path: {path}"
        )))
    }
}

fn authenticate_historical_base(target: &Dir, ledger: &BaseLedger) -> Result<(), UpdateError> {
    if matches!(ledger.product_version.as_str(), "0.7.0" | "0.8.0" | "0.9.0") {
        let expected = historical_project_upgrade_candidate_in(target, &ledger.product_version)
            .map_err(render_error)?;
        return authenticate_full_historical_base(ledger, &expected);
    }
    let skills = historical_builtin_skills(&ledger.product_version).map_err(|error| {
        if error.code() == "hive.skill-history-unsupported" {
            UpdateError::Unsupported(format!(
                "historical project base {} is not embedded in this binary",
                ledger.product_version
            ))
        } else {
            UpdateError::Internal(format!(
                "embedded historical project base registry is invalid: {error}"
            ))
        }
    })?;
    let expected_agents = skills
        .iter()
        .map(|skill| {
            (
                format!(".agents/skills/{}/SKILL.md", skill.name),
                skill.content_digest.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let expected_claude = skills
        .iter()
        .map(|skill| {
            (
                format!(".claude/skills/{}/SKILL.md", skill.name),
                skill.content_digest.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let actual = ledger
        .files
        .iter()
        .map(|entry| (entry.path.clone(), entry.content_digest.as_str()))
        .collect::<BTreeMap<_, _>>();
    if ledger.files.iter().any(|entry| entry.kind != "skill")
        || (actual != expected_agents && actual != expected_claude)
    {
        return Err(UpdateError::Verification(format!(
            "historical project base {} differs from the authenticated built-in registry",
            ledger.product_version
        )));
    }
    Ok(())
}

fn authenticate_full_historical_base(
    ledger: &BaseLedger,
    expected: &HistoricalProjectBase,
) -> Result<(), UpdateError> {
    if ledger.product_version != expected.product_version
        || ledger.files.len() != expected.files.len()
    {
        return Err(UpdateError::Verification(format!(
            "historical project base {} has a partial full-ledger inventory",
            ledger.product_version
        )));
    }
    for (entry, expected_entry) in ledger.files.iter().zip(&expected.files) {
        if entry.path != expected_entry.path
            || entry.kind != expected_entry.kind
            || entry.content_digest != expected_entry.content_digest
            || entry.content.as_bytes() != expected_entry.content
        {
            return Err(UpdateError::Verification(format!(
                "historical project base {} differs from the authenticated full registry: {}",
                ledger.product_version, entry.path
            )));
        }
    }
    Ok(())
}

fn render_override_ledger(
    product_version: &str,
    mut files: Vec<OverrideFile>,
) -> Result<Vec<u8>, UpdateError> {
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut value = json!({
        "schema_version": 1,
        "product_version": product_version,
        "files": files
    });
    let canonical = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    value
        .as_object_mut()
        .expect("override payload is an object")
        .insert(
            "ledger_digest".to_owned(),
            Value::String(sha256_digest(&canonical)),
        );
    let mut bytes = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn insert_exact_if_changed(
    target: &Dir,
    final_files: &mut BTreeMap<String, Option<Vec<u8>>>,
    path: String,
    bytes: Vec<u8>,
) -> Result<(), UpdateError> {
    validate_mutation_path(Path::new(&path))?;
    if read_full_optional(target, Path::new(&path))?.as_deref() != Some(bytes.as_slice()) {
        final_files.insert(path, Some(bytes));
    }
    Ok(())
}

fn materialize_final(
    target: &Dir,
    path: &str,
    mergeable: Option<&[u8]>,
) -> Result<Option<Vec<u8>>, UpdateError> {
    if !is_shared(path) {
        return Ok(mergeable.map(<[u8]>::to_vec));
    }
    let Some(marker) = mergeable else {
        return Err(UpdateError::Conflict(format!(
            "shared guidance marker cannot be deleted: {path}"
        )));
    };
    let current = read_full_optional(target, Path::new(path))?.unwrap_or_default();
    replace_marker(&current, marker).map(Some)
}

fn read_mergeable(target: &Dir, path: &str) -> Result<Option<Vec<u8>>, UpdateError> {
    let Some(bytes) = read_full_optional(target, Path::new(path))? else {
        return Ok(None);
    };
    if is_shared(path) {
        extract_marker(&bytes).map(Some)
    } else {
        Ok(Some(bytes))
    }
}

fn extract_marker(bytes: &[u8]) -> Result<Vec<u8>, UpdateError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| UpdateError::Conflict("shared guidance is not UTF-8".to_owned()))?;
    let starts = text
        .match_indices("<!-- AIGENT-HIVE:START -->")
        .collect::<Vec<_>>();
    let ends = text
        .match_indices("<!-- AIGENT-HIVE:END -->")
        .collect::<Vec<_>>();
    if starts.len() != 1 || ends.len() != 1 || starts[0].0 >= ends[0].0 {
        return Err(UpdateError::Conflict(
            "shared guidance has missing, duplicate, nested, or malformed Hive markers".to_owned(),
        ));
    }
    let end = ends[0].0 + ends[0].1.len();
    let mut marker = text.as_bytes()[starts[0].0..end].to_vec();
    marker.push(b'\n');
    Ok(marker)
}

fn replace_marker(current: &[u8], marker: &[u8]) -> Result<Vec<u8>, UpdateError> {
    if current.is_empty() {
        return Ok(marker.to_vec());
    }
    let text = std::str::from_utf8(current)
        .map_err(|_| UpdateError::Conflict("shared guidance is not UTF-8".to_owned()))?;
    let starts = text
        .match_indices("<!-- AIGENT-HIVE:START -->")
        .collect::<Vec<_>>();
    let ends = text
        .match_indices("<!-- AIGENT-HIVE:END -->")
        .collect::<Vec<_>>();
    if starts.is_empty() && ends.is_empty() {
        let mut output = current.to_vec();
        if !output.ends_with(b"\n") {
            output.push(b'\n');
        }
        output.push(b'\n');
        output.extend_from_slice(marker);
        return Ok(output);
    }
    if starts.len() != 1 || ends.len() != 1 || starts[0].0 >= ends[0].0 {
        return Err(UpdateError::Conflict(
            "shared guidance has malformed Hive markers".to_owned(),
        ));
    }
    let end = ends[0].0 + ends[0].1.len();
    let mut output = Vec::new();
    output.extend_from_slice(&current[..starts[0].0]);
    output.extend_from_slice(marker.strip_suffix(b"\n").unwrap_or(marker));
    output.extend_from_slice(&current[end..]);
    Ok(output)
}

fn is_shared(path: &str) -> bool {
    matches!(path, "AGENTS.md" | "CLAUDE.md" | "GEMINI.md")
}

fn validate_merge_path(path: &Path) -> Result<(), UpdateError> {
    if is_hive_skill_projection_path(path) {
        validate_hive_skill_projection_relative(path)
    } else if is_hive_directive_projection_path(path) {
        validate_hive_directive_projection_relative(path)
    } else if path.to_str().is_some_and(is_shared) {
        validate_project_relative(path)
    } else {
        return Err(UpdateError::Verification(format!(
            "project base contains a non-mergeable path: {}",
            path.display()
        )));
    }
    .map_err(|error| UpdateError::Verification(error.to_string()))
}

fn validate_mutation_path(path: &Path) -> Result<(), UpdateError> {
    if is_hive_skill_projection_path(path) {
        validate_hive_skill_projection_relative(path)
    } else if is_hive_directive_projection_path(path) {
        validate_hive_directive_projection_relative(path)
    } else {
        validate_project_relative(path)
    }
    .map_err(|error| UpdateError::Verification(error.to_string()))
}

fn read_full_optional(target: &Dir, relative: &Path) -> Result<Option<Vec<u8>>, UpdateError> {
    read_bounded_optional(target, relative, u64::MAX)
}

fn read_bounded_optional(
    target: &Dir,
    relative: &Path,
    limit: u64,
) -> Result<Option<Vec<u8>>, UpdateError> {
    validate_mutation_path(relative)?;
    let Some((parent, file_name)) = capability_parent(target, relative, false)? else {
        return Ok(None);
    };
    match parent.symlink_metadata(&file_name) {
        Ok(metadata)
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > limit =>
        {
            Err(UpdateError::Verification(format!(
                "bounded project ledger is not a safe regular file: {}",
                relative.display()
            )))
        }
        Ok(metadata) => {
            let mut options = OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No);
            let mut file = parent.open_with(&file_name, &options).map_err(io_error)?;
            let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
            Read::by_ref(&mut file)
                .take(limit.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(io_error)?;
            if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > limit {
                return Err(UpdateError::Conflict(format!(
                    "project upgrade path changed during read: {}",
                    relative.display()
                )));
            }
            Ok(Some(bytes))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(error)),
    }
}

#[allow(clippy::too_many_lines)]
fn apply(target: &Dir, plan: &UpgradePlan) -> Result<ProjectResult, UpdateError> {
    let fail_after = if cfg!(debug_assertions) {
        std::env::var("HIVE_PROJECT_UPGRADE_FAIL_AFTER")
            .ok()
            .and_then(|value| value.parse().ok())
    } else {
        None
    };
    apply_with_failure_after(target, plan, fail_after)
}

#[allow(clippy::too_many_lines)]
fn apply_with_failure_after(
    target: &Dir,
    plan: &UpgradePlan,
    fail_after: Option<usize>,
) -> Result<ProjectResult, UpdateError> {
    if plan.changed_paths.is_empty() {
        return Ok(plan_result(plan, CommandMode::Apply));
    }
    verify_expected_before(target, plan)?;
    let transaction_id = transaction_id()?;
    let backup_relative = format!(".hive/backups/project-upgrade/{transaction_id}");
    let changes = match (|| {
        let mut changes = Vec::new();
        for (index, path) in plan.changed_paths.iter().enumerate() {
            let expected = plan.expected_before.get(path).ok_or_else(|| {
                UpdateError::Internal(format!("planned expected-before is missing: {path}"))
            })?;
            verify_one_expected_before(target, path, expected)?;
            let before = expected.bytes.clone();
            let backup_path = before.as_ref().map(|bytes| {
                let relative = format!("files/{index:04}.bin");
                (relative, bytes)
            });
            if let Some((relative, bytes)) = &backup_path {
                write_new(target, &Path::new(&backup_relative).join(relative), bytes)?;
            }
            changes.push(JournalChange {
                path: path.clone(),
                claim_path: claim_recovery_path(Path::new(path))?
                    .to_string_lossy()
                    .into_owned(),
                before_digest: before.as_deref().map(sha256_digest),
                after_digest: plan
                    .final_files
                    .get(path)
                    .and_then(|value| value.as_deref())
                    .map(sha256_digest),
                backup_path: backup_path.map(|(relative, _)| relative),
            });
        }
        Ok(changes)
    })() {
        Ok(changes) => changes,
        Err(error) => return Err(resolve_prejournal_failure(target, &backup_relative, error)),
    };
    let journal = UpgradeJournal {
        schema_version: 1,
        transaction_id,
        plan_digest: plan.plan_digest.clone(),
        backup_root: backup_relative,
        changes,
        journal_digest: String::new(),
    };
    let journal = sign_journal(journal)?;
    if let Err(error) = write_new(target, Path::new(JOURNAL_PATH), &canonical_json(&journal)?) {
        cleanup_backup_root(target, &journal.backup_root)?;
        return Err(error);
    }
    if let Err(error) = verify_expected_before(target, plan) {
        cleanup_transaction(target, &journal)?;
        return Err(error);
    }
    let mut applied = Vec::new();
    let result = (|| {
        for (index, path) in plan.changed_paths.iter().enumerate() {
            if fail_after == Some(index) {
                return Err(UpdateError::Internal(
                    "injected project upgrade activation failure".to_owned(),
                ));
            }
            let expected = plan.expected_before.get(path).ok_or_else(|| {
                UpdateError::Internal(format!("planned expected-before is missing: {path}"))
            })?;
            verify_one_expected_before(target, path, expected)?;
            cas_activate_path(
                target,
                Path::new(path),
                expected.digest.as_deref(),
                plan.final_files.get(path).and_then(Option::as_deref),
                None,
                None,
                None,
            )?;
            applied.push(path.clone());
        }
        for change in &journal.changes {
            let current = read_full_optional(target, Path::new(&change.path))?;
            if current.as_deref().map(sha256_digest) != change.after_digest {
                return Err(UpdateError::Verification(format!(
                    "project upgrade activation digest mismatch: {}",
                    change.path
                )));
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        rollback_applied(target, &journal, &applied)?;
        if matches!(error, UpdateError::RecoveryRequired(_)) {
            return Err(error);
        }
        cleanup_transaction(target, &journal)?;
        return Err(error);
    }
    remove_journal_cas(target, &journal)?;
    let mut result = plan_result(plan, CommandMode::Apply);
    "project harness upgrade applied with local-priority merge".clone_into(&mut result.message);
    Ok(result)
}

fn resolve_prejournal_failure(
    target: &Dir,
    backup_relative: &str,
    primary: UpdateError,
) -> UpdateError {
    match cleanup_backup_root(target, backup_relative) {
        Ok(()) if matches!(primary, UpdateError::RecoveryRequired(_)) => UpdateError::Internal(
            format!("pre-journal project upgrade backup failure was cleaned safely: {primary}"),
        ),
        Ok(()) => primary,
        Err(cleanup) => UpdateError::Conflict(format!(
            "pre-journal project upgrade backup failed and cleanup could not complete; orphan retained at {backup_relative}: primary={primary}; cleanup={cleanup}"
        )),
    }
}

fn rollback_applied(
    target: &Dir,
    journal: &UpgradeJournal,
    applied: &[String],
) -> Result<(), UpdateError> {
    for path in applied.iter().rev() {
        let change = journal
            .changes
            .iter()
            .find(|change| &change.path == path)
            .ok_or_else(|| UpdateError::Rollback("rollback change is missing".to_owned()))?;
        restore_before(target, journal, change, change.after_digest.as_deref())?;
    }
    Ok(())
}

fn recover(target: &Dir) -> Result<ProjectResult, UpdateError> {
    let journal_path = Path::new(JOURNAL_PATH);
    let canonical_bytes = read_bounded_optional(target, journal_path, MAX_LEDGER_BYTES)?;
    let mut claimed_source = None;
    let bytes = if let Some(bytes) = canonical_bytes.as_ref() {
        bytes.clone()
    } else {
        let recovery_path = claim_recovery_path(journal_path)?;
        if let Some(bytes) = read_bounded_optional(target, &recovery_path, MAX_LEDGER_BYTES)
            .map_err(|error| {
                UpdateError::RecoveryRequired(format!(
                    "project upgrade journal claim requires recovery at {}: {error}",
                    recovery_path.display()
                ))
            })?
        {
            claimed_source = Some(recovery_path);
            bytes
        } else if cleanup_unauthenticated_empty_claim(target, journal_path)? {
            return Ok(orphaned_journal_cleanup_result());
        } else {
            return Err(UpdateError::RecoveryRequired(format!(
                "no project upgrade journal; checked canonical path and {}",
                recovery_path.display()
            )));
        }
    };
    let journal: UpgradeJournal = serde_json::from_slice(&bytes)
        .map_err(|error| UpdateError::Verification(format!("invalid upgrade journal: {error}")))
        .map_err(|error| retain_claimed_journal_error(error, claimed_source.as_deref()))?;
    validate_journal(&journal)
        .map_err(|error| retain_claimed_journal_error(error, claimed_source.as_deref()))?;
    let canonical = canonical_json(&journal)
        .map_err(|error| retain_claimed_journal_error(error, claimed_source.as_deref()))?;
    if bytes != canonical {
        return Err(retain_claimed_journal_error(
            UpdateError::Verification("project upgrade journal bytes are not canonical".to_owned()),
            claimed_source.as_deref(),
        ));
    }
    reconcile_discoverable_claim(target, journal_path, Some(&sha256_digest(&bytes)), None)?;
    for change in &journal.changes {
        reconcile_discoverable_claim(
            target,
            Path::new(&change.path),
            change.before_digest.as_deref(),
            change.after_digest.as_deref(),
        )?;
    }
    let mut after_count = 0;
    for change in &journal.changes {
        let current = read_full_optional(target, Path::new(&change.path))?;
        let digest = current.as_deref().map(sha256_digest);
        if digest == change.after_digest {
            after_count += 1;
        } else if digest != change.before_digest {
            return Err(UpdateError::Conflict(format!(
                "project upgrade recovery found a concurrent third digest: {}",
                change.path
            )));
        }
    }
    let outcome = if after_count == journal.changes.len() {
        "forward-complete"
    } else {
        for change in journal.changes.iter().rev() {
            let current = read_full_optional(target, Path::new(&change.path))?;
            if current.as_deref().map(sha256_digest) == change.after_digest {
                restore_before(target, &journal, change, change.after_digest.as_deref())?;
            }
        }
        "rolled-back"
    };
    remove_journal_cas(target, &journal)?;
    Ok(ProjectResult {
        schema_version: 1,
        action: "RecoverProjectUpgrade",
        status: "success",
        exit_code: 0,
        code: "hive.project-upgrade-recovered",
        message: format!("project upgrade recovery completed: {outcome}"),
        changed_paths: journal
            .changes
            .iter()
            .map(|change| change.path.clone())
            .collect(),
        evidence: vec![ProjectEvidence {
            kind: "recovery",
            locator: JOURNAL_PATH.to_owned(),
            digest: journal.plan_digest.clone(),
        }],
        next_action: None,
        data: Some(json!({"outcome": outcome, "transaction_id": journal.transaction_id})),
    })
}

fn orphaned_journal_cleanup_result() -> ProjectResult {
    ProjectResult {
        schema_version: 1,
        action: "RecoverProjectUpgrade",
        status: "success",
        exit_code: 0,
        code: "hive.project-upgrade-recovered",
        message: "orphaned project upgrade journal staging cleaned".to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({"outcome": "aborted-before-journal-publication"})),
    }
}

fn retain_claimed_journal_error(error: UpdateError, locator: Option<&Path>) -> UpdateError {
    let Some(locator) = locator else {
        return error;
    };
    let append = |message: String| {
        format!(
            "{message}{CLAIMED_JOURNAL_LOCATOR_MARKER}{}",
            locator.display()
        )
    };
    match error {
        UpdateError::Input(message) => UpdateError::Input(append(message)),
        UpdateError::Verification(message) => UpdateError::Verification(append(message)),
        UpdateError::Compatibility(message) => UpdateError::Compatibility(append(message)),
        UpdateError::Unsupported(message) => UpdateError::Unsupported(append(message)),
        UpdateError::Conflict(message) => UpdateError::Conflict(append(message)),
        UpdateError::RecoveryRequired(message) => UpdateError::RecoveryRequired(append(message)),
        UpdateError::Internal(message) => UpdateError::Internal(append(message)),
        UpdateError::Rollback(message) => UpdateError::Rollback(append(message)),
    }
}

fn cleanup_unauthenticated_empty_claim(target: &Dir, relative: &Path) -> Result<bool, UpdateError> {
    let Some((parent, _)) = capability_parent(target, relative, false)? else {
        return Ok(false);
    };
    let quarantine_name = claim_quarantine_name(relative)?;
    let quarantine = match parent.open_dir_nofollow(&quarantine_name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(UpdateError::RecoveryRequired(format!(
                "unauthenticated project upgrade claim cannot be opened at {}: {error}",
                claim_directory_path(relative)?.display()
            )))
        }
    };
    let mut entries = quarantine.entries().map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "unauthenticated project upgrade claim cannot be inspected at {}: {error}",
            claim_directory_path(relative)
                .unwrap_or_else(|_| relative.to_path_buf())
                .display()
        ))
    })?;
    if entries.next().is_some() {
        return Err(UpdateError::RecoveryRequired(format!(
            "unauthenticated project upgrade claim contains entries; every byte is preserved at {}",
            claim_directory_path(relative)?.display()
        )));
    }
    drop(entries);
    drop(quarantine);
    parent.remove_dir(&quarantine_name).map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "empty unauthenticated project upgrade claim cleanup failed at {}: {error}",
            claim_directory_path(relative)
                .unwrap_or_else(|_| relative.to_path_buf())
                .display()
        ))
    })?;
    Ok(true)
}

fn reconcile_discoverable_claim(
    target: &Dir,
    relative: &Path,
    before_digest: Option<&str>,
    after_digest: Option<&str>,
) -> Result<bool, UpdateError> {
    let Some((parent, destination_name)) = capability_parent(target, relative, false)? else {
        return Ok(false);
    };
    let quarantine_name = claim_quarantine_name(relative)?;
    let quarantine = match parent.open_dir_nofollow(&quarantine_name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade claim cannot be opened at {}: {error}",
                claim_recovery_path(relative)?.display()
            )))
        }
    };
    let mut claim = CasClaim {
        parent,
        quarantine: Some(quarantine),
        quarantine_name,
        destination_name,
        recovery_path: claim_recovery_path(relative)?,
    };
    let claimed_missing = match claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine")
        .symlink_metadata("claimed.bin")
    {
        Ok(_) => false,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => {
            return Err(post_claim_error(
                &claim,
                relative,
                "inspect claimed path",
                error,
            ))
        }
    };
    if claimed_missing {
        if reconcile_preclaim_quarantine(target, &mut claim, relative, before_digest)? {
            return Ok(true);
        }
        let expected_active = if relative == Path::new(JOURNAL_PATH) {
            before_digest
        } else {
            after_digest
        };
        reconcile_creation_claim(target, claim, relative, expected_active)?;
        return Ok(true);
    }
    if before_digest.is_none() {
        reconcile_creation_claim(target, claim, relative, after_digest)?;
        return Ok(true);
    }
    let claimed = read_claimed(&claim, relative)?;
    let claimed_digest = sha256_digest(&claimed);
    if Some(claimed_digest.as_str()) != before_digest {
        return Err(UpdateError::RecoveryRequired(format!(
            "project upgrade claim digest is not the journal prior digest at {}; exact object retained at {}",
            relative.display(),
            claim.recovery_path.display()
        )));
    }
    let active = read_full_optional(target, relative)?;
    let active_digest = active.as_deref().map(sha256_digest);
    if active_digest.is_none() {
        restore_claim_nonoverwriting(&mut claim, relative)?;
        return Ok(true);
    }
    if active_digest.as_deref() != before_digest && active_digest.as_deref() != after_digest {
        return Err(UpdateError::Conflict(format!(
            "project upgrade recovery found a concurrent third digest while a claim remains at {}",
            claim.recovery_path.display()
        )));
    }
    cleanup_retained_claim(claim, relative)?;
    Ok(true)
}

fn reconcile_preclaim_quarantine(
    target: &Dir,
    claim: &mut CasClaim,
    relative: &Path,
    before_digest: Option<&str>,
) -> Result<bool, UpdateError> {
    let active = read_full_optional(target, relative)?;
    if active.as_deref().map(sha256_digest).as_deref() != before_digest || before_digest.is_none() {
        return Ok(false);
    }
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    let entries = quarantine
        .entries()
        .map_err(|error| post_claim_error(claim, relative, "list pre-claim quarantine", error))?
        .map(|entry| {
            entry.map(|entry| entry.file_name()).map_err(|error| {
                post_claim_error(claim, relative, "read pre-claim quarantine entry", error)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if entries
        .iter()
        .any(|entry| entry != OsStr::new("replacement.bin"))
    {
        return Err(UpdateError::RecoveryRequired(format!(
            "pre-claim project upgrade quarantine contains foreign entries; preserved at {}",
            claim_directory_path(relative)?.display()
        )));
    }
    if !entries.is_empty() {
        return Ok(false);
    }
    drop(claim.quarantine.take());
    claim
        .parent
        .remove_dir(&claim.quarantine_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "pre-claim project upgrade quarantine cleanup failed at {}: {error}",
                claim_directory_path(relative).map_or_else(
                    |_| relative.display().to_string(),
                    |path| path.display().to_string(),
                )
            ))
        })?;
    Ok(true)
}

fn reconcile_creation_claim(
    target: &Dir,
    mut claim: CasClaim,
    relative: &Path,
    after_digest: Option<&str>,
) -> Result<(), UpdateError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    let replacement_digest = match quarantine.symlink_metadata("replacement.bin") {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            let mut options = OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No);
            let mut file = quarantine
                .open_with("replacement.bin", &options)
                .map_err(|error| {
                    post_claim_error(&claim, relative, "open creation staging file", error)
                })?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(|error| {
                post_claim_error(&claim, relative, "read creation staging file", error)
            })?;
            Some(sha256_digest(&bytes))
        }
        Ok(_) => {
            return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade creation staging object is nonregular; retained at {}",
                claim
                    .recovery_path
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .display()
            )))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(post_claim_error(
                &claim,
                relative,
                "inspect creation staging file",
                error,
            ))
        }
    };
    if after_digest.is_some()
        && replacement_digest
            .as_deref()
            .is_some_and(|digest| Some(digest) != after_digest)
    {
        return Err(UpdateError::RecoveryRequired(format!(
            "project upgrade creation staging digest is not the journal after digest; retained at {}",
            claim
                .recovery_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .display()
        )));
    }
    let active = read_full_optional(target, relative)?;
    let active_digest = active.as_deref().map(sha256_digest);
    if active_digest.is_some() && active_digest.as_deref() != after_digest {
        return Err(UpdateError::Conflict(format!(
            "project upgrade recovery found a concurrent path while creation staging remains at {}",
            claim
                .recovery_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .display()
        )));
    }
    if replacement_digest.is_some() {
        quarantine.remove_file("replacement.bin").map_err(|error| {
            post_claim_error(&claim, relative, "remove creation staging file", error)
        })?;
    }
    drop(claim.quarantine.take());
    claim
        .parent
        .remove_dir(&claim.quarantine_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "project upgrade creation claim cleanup failed at {}: {error}",
                claim
                    .recovery_path
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .display()
            ))
        })
}

fn cleanup_retained_claim(mut claim: CasClaim, relative: &Path) -> Result<(), UpdateError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    match quarantine.symlink_metadata("replacement.bin") {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            quarantine.remove_file("replacement.bin").map_err(|error| {
                post_claim_error(&claim, relative, "remove replacement staging file", error)
            })?;
        }
        Ok(_) => {
            return Err(UpdateError::RecoveryRequired(format!(
            "project upgrade replacement staging object is nonregular; exact claim retained at {}",
            claim.recovery_path.display()
        )))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(post_claim_error(
                &claim,
                relative,
                "inspect replacement staging file",
                error,
            ))
        }
    }
    quarantine.remove_file("claimed.bin").map_err(|error| {
        post_claim_error(&claim, relative, "remove reconciled claimed path", error)
    })?;
    drop(claim.quarantine.take());
    claim
        .parent
        .remove_dir(&claim.quarantine_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "project upgrade reconciled {}, but claim directory cleanup failed at {}: {error}",
                relative.display(),
                claim
                    .recovery_path
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .display()
            ))
        })
}

fn restore_before(
    target: &Dir,
    journal: &UpgradeJournal,
    change: &JournalChange,
    expected_current_digest: Option<&str>,
) -> Result<(), UpdateError> {
    if let Some(backup_path) = &change.backup_path {
        validate_project_relative(Path::new(backup_path))
            .map_err(|error| UpdateError::Verification(error.to_string()))?;
        let backup_relative = Path::new(&journal.backup_root).join(backup_path);
        let bytes = read_bounded_optional(target, &backup_relative, MAX_LEDGER_BYTES)?.ok_or_else(
            || {
                UpdateError::Rollback(format!(
                    "project upgrade backup is missing: {}",
                    change.path
                ))
            },
        )?;
        if Some(sha256_digest(&bytes)) != change.before_digest {
            return Err(UpdateError::Rollback(format!(
                "project upgrade backup digest mismatch: {}",
                change.path
            )));
        }
        cas_activate_path(
            target,
            Path::new(&change.path),
            expected_current_digest,
            Some(&bytes),
            None,
            None,
            None,
        )
    } else {
        cas_activate_path(
            target,
            Path::new(&change.path),
            expected_current_digest,
            None,
            None,
            None,
            None,
        )
    }
}

struct CasClaim {
    parent: Dir,
    quarantine: Option<Dir>,
    quarantine_name: OsString,
    destination_name: OsString,
    recovery_path: PathBuf,
}

type CasBarrier<'a> = Option<&'a dyn Fn(&Dir, &OsStr)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
enum CasStageFault {
    Open,
    Write,
    CreateReplacementCleanup,
    ClaimRenameCleanup,
}

fn cas_activate_path(
    target: &Dir,
    relative: &Path,
    expected_digest: Option<&str>,
    desired: Option<&[u8]>,
    before_claim: CasBarrier<'_>,
    after_claim: CasBarrier<'_>,
    stage_fault: Option<CasStageFault>,
) -> Result<(), UpdateError> {
    validate_mutation_path(relative)?;
    if expected_digest.is_none() {
        if let Some(barrier) = before_claim {
            let (parent, destination) =
                capability_parent(target, relative, true)?.ok_or_else(|| {
                    UpdateError::Internal("CAS parent disappeared during creation".to_owned())
                })?;
            barrier(&parent, &destination);
        }
        return match desired {
            Some(bytes) => create_new_cas(target, relative, bytes, stage_fault),
            None => {
                if read_full_optional(target, relative)?.is_none() {
                    Ok(())
                } else {
                    Err(UpdateError::Conflict(format!(
                        "project upgrade path appeared before atomic activation: {}",
                        relative.display()
                    )))
                }
            }
        };
    }

    if let Some(barrier) = before_claim {
        let (parent, destination) =
            capability_parent(target, relative, false)?.ok_or_else(|| {
                UpdateError::Conflict(format!(
                    "project upgrade path disappeared before atomic claim: {}",
                    relative.display()
                ))
            })?;
        barrier(&parent, &destination);
    }
    let mut claim = claim_destination(target, relative, stage_fault)?;
    let claimed = read_claimed(&claim, relative)?;
    let claimed_digest = sha256_digest(&claimed);
    if Some(claimed_digest.as_str()) != expected_digest {
        restore_claim_nonoverwriting(&mut claim, relative)?;
        return Err(UpdateError::Conflict(format!(
            "project upgrade claimed a concurrent third digest: {}",
            relative.display()
        )));
    }
    if let Some(barrier) = after_claim {
        barrier(&claim.parent, &claim.destination_name);
    }
    match desired {
        Some(bytes) => publish_claim_replacement(claim, relative, bytes, stage_fault),
        None => finish_claim_deletion(claim, relative),
    }
}

fn create_new_cas(
    target: &Dir,
    relative: &Path,
    bytes: &[u8],
    stage_fault: Option<CasStageFault>,
) -> Result<(), UpdateError> {
    let (parent, file_name) = capability_parent(target, relative, true)?.ok_or_else(|| {
        UpdateError::Internal("CAS parent disappeared during creation".to_owned())
    })?;
    let (quarantine, quarantine_name) = create_claim_quarantine(&parent, relative)?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = match quarantine.open_with("replacement.bin", &options) {
        Ok(file) => file,
        Err(error) => {
            drop(quarantine);
            if let Err(cleanup_error) = parent.remove_dir(&quarantine_name) {
                return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade staging open failed at {}, and staging directory cleanup failed at {}: {cleanup_error}",
                relative.display(),
                claim_directory_path(relative)?.display()
                )));
            }
            return Err(io_error(error));
        }
    };
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        if let Err(cleanup_error) = quarantine.remove_file("replacement.bin") {
            return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade staging write failed at {}, and staging cleanup failed at {}: {cleanup_error}",
                relative.display(),
                claim_directory_path(relative)?.display()
            )));
        }
        drop(quarantine);
        if let Err(cleanup_error) = parent.remove_dir(&quarantine_name) {
            return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade staging write failed at {}, and staging directory cleanup failed at {}: {cleanup_error}",
                relative.display(),
                claim_directory_path(relative)?.display()
            )));
        }
        return Err(io_error(error));
    }
    drop(file);
    if let Err(error) = quarantine.hard_link("replacement.bin", &parent, &file_name) {
        if let Err(cleanup_error) = quarantine.remove_file("replacement.bin") {
            return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade creation conflict at {}, and staging cleanup failed at {}: {cleanup_error}",
                relative.display(),
                claim_directory_path(relative)?.display()
            )));
        }
        drop(quarantine);
        if let Err(cleanup_error) = parent.remove_dir(&quarantine_name) {
            return Err(UpdateError::RecoveryRequired(format!(
                "project upgrade creation conflict at {}, and staging directory cleanup failed at {}: {cleanup_error}",
                relative.display(),
                claim_directory_path(relative)?.display()
            )));
        }
        return Err(UpdateError::Conflict(format!(
            "project upgrade destination appeared before exclusive activation at {}: {error}",
            relative.display()
        )));
    }
    if stage_fault == Some(CasStageFault::CreateReplacementCleanup) {
        return Err(UpdateError::RecoveryRequired(format!(
            "new project upgrade path published at {}, but injected staging cleanup failed at {}",
            relative.display(),
            claim_directory_path(relative)?.display()
        )));
    }
    quarantine.remove_file("replacement.bin").map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "new project upgrade path published at {}, but private staging cleanup failed: {error}",
            relative.display()
        ))
    })?;
    drop(quarantine);
    parent.remove_dir(&quarantine_name).map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "new project upgrade path published at {}, but private staging directory cleanup failed: {error}",
            relative.display()
        ))
    })
}

fn claim_destination(
    target: &Dir,
    relative: &Path,
    stage_fault: Option<CasStageFault>,
) -> Result<CasClaim, UpdateError> {
    let (parent, destination_name) =
        capability_parent(target, relative, false)?.ok_or_else(|| {
            UpdateError::Conflict(format!(
                "project upgrade path disappeared before atomic claim: {}",
                relative.display()
            ))
        })?;
    let (quarantine, quarantine_name) = create_claim_quarantine(&parent, relative)?;
    let rename = if stage_fault == Some(CasStageFault::ClaimRenameCleanup) {
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        quarantine
            .open_with("foreign-entry", &options)
            .map(drop)
            .map_err(io_error)?;
        Err(std::io::Error::other("injected destination rename failure"))
    } else {
        parent.rename(&destination_name, &quarantine, OsStr::new("claimed.bin"))
    };
    if let Err(error) = rename {
        drop(quarantine);
        return match parent.remove_dir(&quarantine_name) {
            Ok(()) => Err(UpdateError::Conflict(format!(
                "project upgrade path changed before atomic claim at {}: {error}",
                relative.display()
            ))),
            Err(cleanup_error) => Err(UpdateError::RecoveryRequired(format!(
                "project upgrade path changed before atomic claim at {}, and quarantine cleanup failed; retained at {}: rename={error}; cleanup={cleanup_error}",
                relative.display(),
                claim_directory_path(relative)?.display()
            ))),
        };
    }
    let recovery_path = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&quarantine_name)
        .join("claimed.bin");
    Ok(CasClaim {
        parent,
        quarantine: Some(quarantine),
        quarantine_name,
        destination_name,
        recovery_path,
    })
}

fn create_claim_quarantine(parent: &Dir, relative: &Path) -> Result<(Dir, OsString), UpdateError> {
    let name = claim_quarantine_name(relative)?;
    parent.create_dir(&name).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            UpdateError::RecoveryRequired(format!(
                "a prior project upgrade claim requires recovery at {}",
                claim_recovery_path(relative).map_or_else(
                    |_| relative.display().to_string(),
                    |path| path.display().to_string(),
                )
            ))
        } else {
            io_error(error)
        }
    })?;
    match parent.open_dir_nofollow(&name) {
        Ok(directory) => Ok((directory, name)),
        Err(error) => {
            if let Err(cleanup_error) = parent.remove_dir(&name) {
                return Err(UpdateError::RecoveryRequired(format!(
                    "project upgrade claim directory open failed, and cleanup failed at {}: {cleanup_error}",
                    claim_directory_path(relative)?.display()
                )));
            }
            Err(io_error(error))
        }
    }
}

fn claim_quarantine_name(relative: &Path) -> Result<OsString, UpdateError> {
    let path = relative.to_str().ok_or_else(|| {
        UpdateError::Input("project upgrade claim path must be valid UTF-8".to_owned())
    })?;
    let digest = sha256_digest(path.as_bytes());
    Ok(OsString::from(format!(
        ".hive-project-claim-{}",
        digest.trim_start_matches("sha256:")
    )))
}

fn claim_recovery_path(relative: &Path) -> Result<PathBuf, UpdateError> {
    Ok(relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(claim_quarantine_name(relative)?)
        .join("claimed.bin"))
}

fn claim_directory_path(relative: &Path) -> Result<PathBuf, UpdateError> {
    Ok(relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(claim_quarantine_name(relative)?))
}

fn read_claimed(claim: &CasClaim, relative: &Path) -> Result<Vec<u8>, UpdateError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    let metadata = quarantine
        .symlink_metadata("claimed.bin")
        .map_err(|error| post_claim_error(claim, relative, "inspect claimed path", error))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_LEDGER_BYTES * 16
    {
        return Err(UpdateError::RecoveryRequired(format!(
            "claimed project upgrade path at {} is not a bounded regular file; exact claimed object retained for recovery at {}",
            relative.display(),
            claim.recovery_path.display()
        )));
    }
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = quarantine
        .open_with("claimed.bin", &options)
        .map_err(|error| post_claim_error(claim, relative, "open claimed path", error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| post_claim_error(claim, relative, "read claimed path", error))?;
    Ok(bytes)
}

fn post_claim_error(
    claim: &CasClaim,
    relative: &Path,
    operation: &str,
    error: impl std::fmt::Display,
) -> UpdateError {
    UpdateError::RecoveryRequired(format!(
        "project upgrade could not {operation} at {}; exact claimed object retained for recovery at {}: {error}",
        relative.display(),
        claim.recovery_path.display()
    ))
}

fn restore_claim_nonoverwriting(claim: &mut CasClaim, relative: &Path) -> Result<(), UpdateError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    quarantine
        .hard_link("claimed.bin", &claim.parent, &claim.destination_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "project upgrade claim conflict at {}; concurrent bytes preserved and claimed bytes recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            ))
        })?;
    quarantine.remove_file("claimed.bin").map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "project upgrade claim restored at {}, but claim cleanup failed; claimed bytes remain recoverable at {}: {error}",
            relative.display(),
            claim.recovery_path.display()
        ))
    })?;
    drop(claim.quarantine.take());
    claim
        .parent
        .remove_dir(&claim.quarantine_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "project upgrade claim restored at {}, but claim directory cleanup failed: {error}",
                relative.display()
            ))
        })
}

fn publish_claim_replacement(
    mut claim: CasClaim,
    relative: &Path,
    bytes: &[u8],
    stage_fault: Option<CasStageFault>,
) -> Result<(), UpdateError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    if stage_fault == Some(CasStageFault::Open) {
        return Err(UpdateError::RecoveryRequired(format!(
            "project upgrade replacement staging open failed at {}; exact prior bytes retained for recovery at {}",
            relative.display(),
            claim.recovery_path.display()
        )));
    }
    let mut replacement = quarantine
        .open_with("replacement.bin", &options)
        .map_err(|error| {
            post_claim_error(&claim, relative, "open replacement staging file", error)
        })?;
    if stage_fault == Some(CasStageFault::Write) {
        drop(replacement);
        return Err(UpdateError::RecoveryRequired(format!(
            "project upgrade replacement staging write failed at {}; exact prior bytes retained for recovery at {}",
            relative.display(),
            claim.recovery_path.display()
        )));
    }
    replacement
        .write_all(bytes)
        .and_then(|()| replacement.sync_all())
        .map_err(|error| {
            post_claim_error(&claim, relative, "write replacement staging file", error)
        })?;
    drop(replacement);
    quarantine
        .hard_link("replacement.bin", &claim.parent, &claim.destination_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "concurrent destination appeared during project upgrade at {}; prior bytes recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            ))
        })?;
    quarantine
        .remove_file("replacement.bin")
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "project upgrade replacement published at {}, but staged-byte cleanup failed; prior bytes remain recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            ))
        })?;
    quarantine.remove_file("claimed.bin").map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "project upgrade replacement published at {}, but prior-byte cleanup failed; prior bytes remain recoverable at {}: {error}",
            relative.display(),
            claim.recovery_path.display()
        ))
    })?;
    drop(claim.quarantine.take());
    claim.parent.remove_dir(&claim.quarantine_name).map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "project upgrade replacement published at {}, but claim directory cleanup failed: {error}",
            relative.display()
        ))
    })
}

fn finish_claim_deletion(mut claim: CasClaim, relative: &Path) -> Result<(), UpdateError> {
    match claim.parent.symlink_metadata(&claim.destination_name) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => {
            return Err(UpdateError::RecoveryRequired(format!(
                "concurrent destination appeared during project deletion at {}; prior bytes recoverable at {}",
                relative.display(),
                claim.recovery_path.display()
            )))
        }
        Err(error) => {
            return Err(post_claim_error(
                &claim,
                relative,
                "inspect deletion destination",
                error,
            ))
        }
    }
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live CAS claim retains its quarantine");
    quarantine.remove_file("claimed.bin").map_err(|error| {
        UpdateError::RecoveryRequired(format!(
            "project deletion published at {}, but prior-byte cleanup failed; prior bytes remain recoverable at {}: {error}",
            relative.display(),
            claim.recovery_path.display()
        ))
    })?;
    drop(claim.quarantine.take());
    claim
        .parent
        .remove_dir(&claim.quarantine_name)
        .map_err(|error| {
            UpdateError::RecoveryRequired(format!(
                "project deletion published at {}, but claim directory cleanup failed: {error}",
                relative.display()
            ))
        })
}

fn write_new(target: &Dir, relative: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    create_new_cas(target, relative, bytes, None)
}

fn open_target_capability(target: &Path) -> Result<Dir, UpdateError> {
    let parent = target
        .parent()
        .ok_or_else(|| UpdateError::Input("project target has no parent directory".to_owned()))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = target
        .file_name()
        .ok_or_else(|| UpdateError::Input("project target has no directory name".to_owned()))?;
    let parent_dir = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|error| {
        UpdateError::Conflict(format!("cannot open project target parent: {error}"))
    })?;
    parent_dir.open_dir_nofollow(name).map_err(|error| {
        UpdateError::Conflict(format!(
            "project target cannot be opened as a no-follow directory {}: {error}",
            target.display()
        ))
    })
}

fn ensure_pinned_consumer_target(target: &Dir) -> Result<(), UpdateError> {
    if read_bounded_optional(target, Path::new("hive-source.json"), MAX_LEDGER_BYTES)?.is_some() {
        return Err(UpdateError::Input(
            "project upgrade commands are forbidden in the Hive source workspace".to_owned(),
        ));
    }
    Ok(())
}

fn capability_parent(
    target: &Dir,
    relative: &Path,
    create_missing: bool,
) -> Result<Option<(Dir, OsString)>, UpdateError> {
    validate_mutation_path(relative)?;
    let file_name = relative
        .file_name()
        .ok_or_else(|| UpdateError::Input("project upgrade path has no file name".to_owned()))?
        .to_os_string();
    let mut current = target.try_clone().map_err(|error| {
        UpdateError::Internal(format!("cannot clone target capability: {error}"))
    })?;
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let name = component.as_os_str();
            match current.symlink_metadata(name) {
                Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
                Ok(_) => {
                    return Err(UpdateError::Conflict(format!(
                        "project upgrade ancestor is not a no-follow directory: {}",
                        relative.display()
                    )))
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound && create_missing => {
                    current.create_dir(name).map_err(|error| {
                        UpdateError::Conflict(format!(
                            "cannot create project upgrade ancestor {}: {error}",
                            relative.display()
                        ))
                    })?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => {
                    return Err(UpdateError::Conflict(format!(
                        "cannot inspect project upgrade ancestor {}: {error}",
                        relative.display()
                    )))
                }
            }
            current = current.open_dir_nofollow(name).map_err(|error| {
                UpdateError::Conflict(format!(
                    "cannot open project upgrade ancestor {} no-follow: {error}",
                    relative.display()
                ))
            })?;
        }
    }
    Ok(Some((current, file_name)))
}

fn verify_expected_before(target: &Dir, plan: &UpgradePlan) -> Result<(), UpdateError> {
    for path in &plan.changed_paths {
        let expected = plan.expected_before.get(path).ok_or_else(|| {
            UpdateError::Internal(format!("planned expected-before is missing: {path}"))
        })?;
        verify_one_expected_before(target, path, expected)?;
    }
    Ok(())
}

fn verify_one_expected_before(
    target: &Dir,
    path: &str,
    expected: &ExpectedBefore,
) -> Result<(), UpdateError> {
    let current = read_full_optional(target, Path::new(path))?;
    let current_digest = current.as_deref().map(sha256_digest);
    if current != expected.bytes || current_digest != expected.digest {
        return Err(UpdateError::Conflict(format!(
            "project upgrade found a concurrent third digest before mutation: {path}"
        )));
    }
    Ok(())
}

fn cleanup_transaction(target: &Dir, journal: &UpgradeJournal) -> Result<(), UpdateError> {
    remove_journal_cas(target, journal)?;
    cleanup_backup_root(target, &journal.backup_root)
}

fn remove_journal_cas(target: &Dir, journal: &UpgradeJournal) -> Result<(), UpdateError> {
    let bytes = canonical_json(journal)?;
    cas_activate_path(
        target,
        Path::new(JOURNAL_PATH),
        Some(&sha256_digest(&bytes)),
        None,
        None,
        None,
        None,
    )
}

fn cleanup_backup_root(target: &Dir, backup_root: &str) -> Result<(), UpdateError> {
    remove_tree(target, Path::new(backup_root))?;
    for relative in [
        ".hive/backups/project-upgrade",
        ".hive/backups",
        ".hive/runtime",
    ] {
        remove_empty_dir(target, Path::new(relative))?;
    }
    Ok(())
}

fn remove_tree(target: &Dir, relative: &Path) -> Result<(), UpdateError> {
    let Some((parent, name)) = capability_parent(target, relative, false)? else {
        return Ok(());
    };
    let directory = match parent.open_dir_nofollow(&name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error(error)),
    };
    remove_dir_contents(&directory)?;
    drop(directory);
    parent.remove_dir(&name).map_err(io_error)
}

fn remove_dir_contents(directory: &Dir) -> Result<(), UpdateError> {
    let entries = directory.entries().map_err(io_error)?;
    for entry in entries {
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name();
        let metadata = directory.symlink_metadata(&name).map_err(io_error)?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let child = directory.open_dir_nofollow(&name).map_err(io_error)?;
            remove_dir_contents(&child)?;
            drop(child);
            directory.remove_dir(&name).map_err(io_error)?;
        } else if metadata.is_file() && !metadata.file_type().is_symlink() {
            directory.remove_file(&name).map_err(io_error)?;
        } else {
            return Err(UpdateError::Conflict(
                "project upgrade cleanup encountered a non-regular entry".to_owned(),
            ));
        }
    }
    Ok(())
}

fn remove_empty_dir(target: &Dir, relative: &Path) -> Result<(), UpdateError> {
    let Some((parent, name)) = capability_parent(target, relative, false)? else {
        return Ok(());
    };
    let directory = match parent.open_dir_nofollow(&name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error(error)),
    };
    if directory.entries().map_err(io_error)?.next().is_none() {
        drop(directory);
        parent.remove_dir(&name).map_err(io_error)?;
    }
    Ok(())
}

fn sign_journal(mut journal: UpgradeJournal) -> Result<UpgradeJournal, UpdateError> {
    journal.journal_digest.clear();
    let canonical = serde_json_canonicalizer::to_vec(&journal)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    journal.journal_digest = sha256_digest(&canonical);
    Ok(journal)
}

fn validate_journal(journal: &UpgradeJournal) -> Result<(), UpdateError> {
    let expected_backup_root = format!(".hive/backups/project-upgrade/{}", journal.transaction_id);
    if journal.schema_version != 1
        || journal.changes.is_empty()
        || !valid_transaction_id(&journal.transaction_id)
        || journal.backup_root != expected_backup_root
        || !valid_digest(&journal.plan_digest)
        || !valid_digest(&journal.journal_digest)
    {
        return Err(UpdateError::Verification(
            "project upgrade journal metadata is invalid".to_owned(),
        ));
    }
    let expected = sign_journal(UpgradeJournal {
        schema_version: journal.schema_version,
        transaction_id: journal.transaction_id.clone(),
        plan_digest: journal.plan_digest.clone(),
        backup_root: journal.backup_root.clone(),
        changes: journal
            .changes
            .iter()
            .map(|change| JournalChange {
                path: change.path.clone(),
                claim_path: change.claim_path.clone(),
                before_digest: change.before_digest.clone(),
                after_digest: change.after_digest.clone(),
                backup_path: change.backup_path.clone(),
            })
            .collect(),
        journal_digest: String::new(),
    })?;
    if expected.journal_digest != journal.journal_digest {
        return Err(UpdateError::Verification(
            "project upgrade journal digest is invalid".to_owned(),
        ));
    }
    let mut previous = None;
    for (index, change) in journal.changes.iter().enumerate() {
        validate_mutation_path(Path::new(&change.path))?;
        let expected_claim_path = claim_recovery_path(Path::new(&change.path))?;
        if previous.is_some_and(|path: &str| path >= change.path.as_str())
            || change.claim_path != expected_claim_path.to_string_lossy()
            || change.before_digest == change.after_digest
            || change
                .before_digest
                .iter()
                .chain(change.after_digest.iter())
                .any(|digest| !valid_digest(digest))
            || match (&change.before_digest, &change.backup_path) {
                (Some(_), Some(path)) => path != &format!("files/{index:04}.bin"),
                (None, None) => false,
                _ => true,
            }
        {
            return Err(UpdateError::Verification(
                "project upgrade journal change inventory is invalid".to_owned(),
            ));
        }
        previous = Some(change.path.as_str());
    }
    Ok(())
}

fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>, UpdateError> {
    let mut bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn transaction_id() -> Result<String, UpdateError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| UpdateError::Internal(error.to_string()))?
        .as_nanos();
    Ok(format!("upgrade-{nanos:x}-{}", std::process::id()))
}

fn valid_transaction_id(value: &str) -> bool {
    value.starts_with("upgrade-")
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn plan_changes(
    changed_paths: &[String],
    final_files: &BTreeMap<String, Option<Vec<u8>>>,
    expected_before: &BTreeMap<String, ExpectedBefore>,
) -> Vec<Value> {
    changed_paths
        .iter()
        .map(|path| {
            json!({
                "path": path,
                "before": expected_before.get(path).and_then(|expected| expected.digest.as_deref()),
                "after": final_files.get(path)
                    .and_then(|bytes| bytes.as_deref())
                    .map(sha256_digest)
            })
        })
        .collect()
}

fn plan_result(plan: &UpgradePlan, mode: CommandMode) -> ProjectResult {
    let action = match mode {
        CommandMode::Scan => "ScanProjectUpgrade",
        CommandMode::DryRun | CommandMode::Apply => "UpdateProjectHarness",
        CommandMode::Validate => "ValidateProjectUpgrade",
        CommandMode::Recover => unreachable!(),
    };
    ProjectResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code: if plan.changed_paths.is_empty() {
            "hive.project-upgrade-current"
        } else if mode == CommandMode::Scan {
            "hive.project-upgrade-available"
        } else if mode == CommandMode::Apply {
            "hive.project-upgrade-complete"
        } else {
            "hive.project-upgrade-planned"
        },
        message: if plan.changed_paths.is_empty() {
            "project harness is current".to_owned()
        } else {
            "project harness upgrade plan validated".to_owned()
        },
        changed_paths: plan.changed_paths.clone(),
        evidence: vec![ProjectEvidence {
            kind: "report",
            locator: BASE_PATH.to_owned(),
            digest: plan.plan_digest.clone(),
        }],
        next_action: None,
        data: Some(json!({
            "source_version": plan.source_version,
            "target_version": plan.target_version,
            "plan_digest": plan.plan_digest,
            "reports": plan.reports
        })),
    }
}

fn failure(error: &UpdateError) -> ProjectResult {
    let message = error.to_string();
    let next_action = if matches!(error, UpdateError::RecoveryRequired(_)) {
        Some("run hive project upgrade --recover".to_owned())
    } else {
        message
            .rsplit_once(CLAIMED_JOURNAL_LOCATOR_MARKER)
            .map(|(_, locator)| {
                format!(
                    "inspect retained claimed journal at {locator}, then retry hive project upgrade --recover"
                )
            })
    };
    ProjectResult {
        schema_version: 1,
        action: "UpdateProjectHarness",
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action,
        data: None,
    }
}

fn emit(result: &ProjectResult) {
    match serde_json::to_string(result) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            println!("{{\"schema_version\":1,\"action\":\"UpdateProjectHarness\",\"status\":\"error\",\"exit_code\":10,\"code\":\"hive.internal-error\",\"message\":\"JSON serialization failed\",\"changed_paths\":[],\"evidence\":[],\"next_action\":null}}");
            eprintln!("error: {error}");
        }
    }
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
}

fn render_error(error: RenderError) -> UpdateError {
    match error {
        RenderError::Input(message) => UpdateError::Input(message),
        RenderError::Conflict(message) | RenderError::Safety(message) => {
            UpdateError::Conflict(message)
        }
        RenderError::Verification(message) => UpdateError::Verification(message),
        RenderError::Unsupported(message) => UpdateError::Unsupported(message),
        RenderError::Internal(message) => UpdateError::Internal(message),
        RenderError::Rollback(message) => UpdateError::Rollback(message),
    }
}

fn io_error(error: impl std::fmt::Display) -> UpdateError {
    UpdateError::Internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn digest(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn signed_base_ledger(product_version: &str, files: &[BaseFile]) -> Vec<u8> {
        let mut value = json!({
            "schema_version": 1,
            "product_version": product_version,
            "files": files,
        });
        let digest = sha256_digest(
            &serde_json_canonicalizer::to_vec(&value).expect("canonical unsigned ledger"),
        );
        value
            .as_object_mut()
            .expect("object")
            .insert("ledger_digest".to_owned(), Value::String(digest));
        let mut bytes = serde_json_canonicalizer::to_vec(&value).expect("canonical signed ledger");
        bytes.push(b'\n');
        bytes
    }

    fn target_dir(path: &Path) -> Dir {
        Dir::open_ambient_dir(path, ambient_authority()).expect("target capability")
    }

    fn claimed_recovery_path(active: &Path) -> PathBuf {
        fs::read_dir(active.parent().expect("active parent"))
            .expect("parent entries")
            .filter_map(Result::ok)
            .find_map(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".hive-project-claim-")
                    .then(|| entry.path().join("claimed.bin"))
            })
            .expect("recoverable claim")
    }

    fn one_file_plan(path: &str, before: Option<&[u8]>, after: Option<&[u8]>) -> UpgradePlan {
        UpgradePlan {
            source_version: "0.7.0".to_owned(),
            target_version: "0.7.0".to_owned(),
            reports: Vec::new(),
            final_files: BTreeMap::from([(path.to_owned(), after.map(<[u8]>::to_vec))]),
            expected_before: BTreeMap::from([(
                path.to_owned(),
                ExpectedBefore {
                    bytes: before.map(<[u8]>::to_vec),
                    digest: before.map(sha256_digest),
                },
            )]),
            changed_paths: vec![path.to_owned()],
            plan_digest: sha256_digest(b"test-plan"),
        }
    }

    fn deletion_plan(target: &Dir, final_files: BTreeMap<String, Option<Vec<u8>>>) -> UpgradePlan {
        let changed_paths = final_files.keys().cloned().collect::<Vec<_>>();
        let expected_before = changed_paths
            .iter()
            .map(|path| {
                let bytes = read_full_optional(target, Path::new(path))
                    .expect("expected-before read")
                    .expect("planned deletion exists");
                (
                    path.clone(),
                    ExpectedBefore {
                        digest: Some(sha256_digest(&bytes)),
                        bytes: Some(bytes),
                    },
                )
            })
            .collect();
        UpgradePlan {
            source_version: "0.7.0".to_owned(),
            target_version: "0.8.0".to_owned(),
            reports: Vec::new(),
            final_files,
            expected_before,
            changed_paths,
            plan_digest: sha256_digest(b"legacy-derived-index-cleanup"),
        }
    }

    fn seed_legacy_derived_index(root: &Path) -> BTreeMap<String, Vec<u8>> {
        LEGACY_DERIVED_INDEX_PATHS
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let bytes = format!("derived-{index}\n").into_bytes();
                let active = root.join(path);
                fs::create_dir_all(active.parent().expect("derived parent"))
                    .expect("derived parent");
                fs::write(&active, &bytes).expect("derived bytes");
                ((*path).to_owned(), bytes)
            })
            .collect()
    }

    #[test]
    fn legacy_070_derived_index_cleanup_applies_and_preserves_markdown_knowledge() {
        let temporary = tempdir().expect("temporary target");
        let target = target_dir(temporary.path());
        seed_legacy_derived_index(temporary.path());
        let knowledge = temporary.path().join(".hive/knowledge/en/project.md");
        fs::create_dir_all(knowledge.parent().expect("knowledge parent"))
            .expect("knowledge parent");
        fs::write(&knowledge, b"# Canonical knowledge\n").expect("knowledge");
        let mut final_files = BTreeMap::new();

        plan_legacy_derived_index_cleanup(&target, &mut final_files, "0.7.0", "0.8.0")
            .expect("cleanup plan");
        let plan = deletion_plan(&target, final_files);
        let preview = plan_result(&plan, CommandMode::DryRun);
        let mut expected_paths = LEGACY_DERIVED_INDEX_PATHS.map(str::to_owned).to_vec();
        expected_paths.sort();

        assert_eq!(preview.changed_paths, expected_paths);
        apply_with_failure_after(&target, &plan, None).expect("cleanup apply");
        for path in LEGACY_DERIVED_INDEX_PATHS {
            assert!(!temporary.path().join(path).exists(), "{path}");
        }
        assert_eq!(
            fs::read(&knowledge).expect("preserved knowledge"),
            b"# Canonical knowledge\n"
        );
    }

    #[test]
    fn legacy_070_derived_index_cleanup_rolls_back_every_deleted_file_after_injected_failure() {
        let temporary = tempdir().expect("temporary target");
        let target = target_dir(temporary.path());
        let original = seed_legacy_derived_index(temporary.path());
        let mut final_files = BTreeMap::new();
        plan_legacy_derived_index_cleanup(&target, &mut final_files, "0.7.0", "0.8.0")
            .expect("cleanup plan");
        let plan = deletion_plan(&target, final_files);

        let Err(error) = apply_with_failure_after(&target, &plan, Some(2)) else {
            panic!("injected activation failure expected");
        };

        assert!(matches!(error, UpdateError::Internal(_)));
        for (path, bytes) in original {
            assert_eq!(
                fs::read(temporary.path().join(path)).expect("rolled-back derived bytes"),
                bytes
            );
        }
        assert!(!temporary.path().join(JOURNAL_PATH).exists());
        assert!(!temporary.path().join(".hive/backups").exists());
    }

    #[cfg(unix)]
    #[test]
    fn legacy_070_derived_index_cleanup_rejects_symlink_without_mutation() {
        use std::os::unix::fs::symlink;

        let temporary = tempdir().expect("temporary target");
        let index = temporary.path().join(".hive/index");
        fs::create_dir_all(&index).expect("index");
        fs::write(index.join("outside"), b"outside\n").expect("outside");
        symlink("outside", index.join("hive.sqlite3")).expect("derived symlink");
        let target = target_dir(temporary.path());

        let error =
            plan_legacy_derived_index_cleanup(&target, &mut BTreeMap::new(), "0.7.0", "0.8.0")
                .expect_err("symlink must be rejected");

        assert!(matches!(error, UpdateError::Verification(_)));
        assert_eq!(
            fs::read(index.join("outside")).expect("outside preserved"),
            b"outside\n"
        );
    }

    fn full_registry(files: &BTreeMap<String, Vec<u8>>) -> HistoricalProjectBase {
        HistoricalProjectBase {
            product_version: "0.7.0".to_owned(),
            files: files
                .iter()
                .map(|(path, content)| hive_render::HistoricalProjectBaseFile {
                    path: path.clone(),
                    kind: if is_shared(path) {
                        "shared-marker"
                    } else if path.contains("/skills/") {
                        "skill"
                    } else {
                        "directive"
                    }
                    .to_owned(),
                    content_digest: sha256_digest(content),
                    content: content.clone(),
                })
                .collect(),
        }
    }

    #[test]
    fn apply_rejects_edit_after_planning_without_mutation() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/test.md";
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::write(&active, b"planned\n").expect("planned bytes");
        let plan = one_file_plan(relative, Some(b"planned\n"), Some(b"incoming\n"));
        fs::write(&active, b"concurrent\n").expect("concurrent bytes");

        let Err(error) = apply(&target_dir(temporary.path()), &plan) else {
            panic!("CAS conflict expected");
        };

        assert!(matches!(error, UpdateError::Conflict(_)));
        assert_eq!(fs::read(&active).expect("active bytes"), b"concurrent\n");
        assert!(!temporary.path().join(".hive").exists());
    }

    #[cfg(unix)]
    #[test]
    fn pinned_target_survives_hostile_ancestor_swap_without_outside_write() {
        use std::os::unix::fs::symlink;

        let temporary = tempdir().expect("temporary workspace");
        let target = temporary.path().join("target");
        let pinned_location = temporary.path().join("pinned");
        let outside = temporary.path().join("outside");
        fs::create_dir_all(&target).expect("target");
        fs::create_dir_all(&outside).expect("outside");
        let target_dir = open_target_capability(&target).expect("pinned target");
        fs::rename(&target, &pinned_location).expect("swap target");
        symlink(&outside, &target).expect("outside symlink");

        let relative = ".agents/directives/test.md";
        let plan = one_file_plan(relative, None, Some(b"incoming\n"));
        apply(&target_dir, &plan).expect("apply through pinned target");

        assert_eq!(
            fs::read(pinned_location.join(relative)).expect("pinned output"),
            b"incoming\n"
        );
        assert!(!outside.join(relative).exists());
    }

    #[cfg(unix)]
    #[test]
    fn managed_ancestor_symlink_swap_aborts_without_outside_write() {
        use std::os::unix::fs::symlink;

        let temporary = tempdir().expect("temporary workspace");
        let target = temporary.path().join("target");
        let outside = temporary.path().join("outside");
        fs::create_dir_all(&target).expect("target");
        fs::create_dir_all(&outside).expect("outside");
        let target_dir = open_target_capability(&target).expect("pinned target");
        symlink(&outside, target.join(".agents")).expect("managed ancestor symlink");

        let relative = ".agents/directives/test.md";
        let plan = one_file_plan(relative, None, Some(b"incoming\n"));
        let Err(error) = apply(&target_dir, &plan) else {
            panic!("ancestor conflict expected");
        };

        assert!(matches!(
            error,
            UpdateError::Conflict(_) | UpdateError::Verification(_)
        ));
        assert!(!outside.join("directives/test.md").exists());
        assert!(!target.join(".hive").exists());
    }

    #[test]
    fn atomic_claim_restores_racing_bytes_without_overwrite() {
        let temporary = tempdir().expect("temporary target");
        let relative = Path::new(".agents/directives/test.md");
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::write(&active, b"expected\n").expect("expected bytes");
        let target = target_dir(temporary.path());
        let race = |parent: &Dir, destination: &OsStr| {
            let mut options = OpenOptions::new();
            options
                .write(true)
                .truncate(true)
                .follow(FollowSymlinks::No);
            let mut file = parent
                .open_with(destination, &options)
                .expect("open racing destination");
            file.write_all(b"racer\n").expect("racing bytes");
        };

        let error = cas_activate_path(
            &target,
            relative,
            Some(&sha256_digest(b"expected\n")),
            Some(b"incoming\n"),
            Some(&race),
            None,
            None,
        )
        .expect_err("third digest conflict");

        assert!(matches!(error, UpdateError::Conflict(_)));
        assert_eq!(fs::read(&active).expect("active racer"), b"racer\n");
        assert_eq!(
            fs::read_dir(active.parent().expect("active parent"))
                .expect("parent entries")
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".hive-project-claim-")
                })
                .count(),
            0
        );
    }

    #[test]
    fn atomic_publish_never_overwrites_destination_created_after_claim() {
        let temporary = tempdir().expect("temporary target");
        let relative = Path::new(".agents/directives/test.md");
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::write(&active, b"expected\n").expect("expected bytes");
        let target = target_dir(temporary.path());
        let race = |parent: &Dir, destination: &OsStr| {
            let mut options = OpenOptions::new();
            options
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            let mut file = parent
                .open_with(destination, &options)
                .expect("create racing destination");
            file.write_all(b"racer\n").expect("racing bytes");
        };

        let error = cas_activate_path(
            &target,
            relative,
            Some(&sha256_digest(b"expected\n")),
            Some(b"incoming\n"),
            None,
            Some(&race),
            None,
        )
        .expect_err("exclusive publish conflict");

        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert_eq!(fs::read(&active).expect("active racer"), b"racer\n");
        let claimed = fs::read_dir(active.parent().expect("active parent"))
            .expect("parent entries")
            .filter_map(Result::ok)
            .find_map(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".hive-project-claim-")
                    .then(|| fs::read(entry.path().join("claimed.bin")).expect("claimed prior"))
            })
            .expect("recoverable claim");
        assert_eq!(claimed, b"expected\n");
    }

    #[test]
    fn atomic_delete_never_removes_destination_created_after_claim() {
        let temporary = tempdir().expect("temporary target");
        let relative = Path::new(".agents/directives/test.md");
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::write(&active, b"expected\n").expect("expected bytes");
        let target = target_dir(temporary.path());
        let race = |parent: &Dir, destination: &OsStr| {
            let mut options = OpenOptions::new();
            options
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            let mut file = parent
                .open_with(destination, &options)
                .expect("create racing destination");
            file.write_all(b"racer\n").expect("racing bytes");
        };

        let error = cas_activate_path(
            &target,
            relative,
            Some(&sha256_digest(b"expected\n")),
            None,
            None,
            Some(&race),
            None,
        )
        .expect_err("exclusive deletion conflict");

        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert_eq!(fs::read(&active).expect("active racer"), b"racer\n");
    }

    #[cfg(unix)]
    #[test]
    fn atomic_claim_retains_symlink_object_without_following_it() {
        use std::os::unix::fs::symlink;

        let temporary = tempdir().expect("temporary target");
        let relative = Path::new(".agents/directives/test.md");
        let active = temporary.path().join(relative);
        let external = temporary.path().join("external.txt");
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::write(&external, b"external\n").expect("external bytes");
        symlink(&external, &active).expect("managed symlink");
        let target = target_dir(temporary.path());

        let error = cas_activate_path(
            &target,
            relative,
            Some(&sha256_digest(b"expected\n")),
            Some(b"incoming\n"),
            None,
            None,
            None,
        )
        .expect_err("nonregular claim must require recovery");

        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert!(!active.exists());
        let recovery = claimed_recovery_path(&active);
        assert!(fs::symlink_metadata(&recovery)
            .expect("recovery metadata")
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_link(&recovery).expect("symlink target"), external);
        assert_eq!(fs::read(&external).expect("external bytes"), b"external\n");
    }

    #[test]
    fn atomic_claim_retains_directory_object_for_recovery() {
        let temporary = tempdir().expect("temporary target");
        let relative = Path::new(".agents/directives/test.md");
        let active = temporary.path().join(relative);
        fs::create_dir_all(&active).expect("managed directory");
        fs::write(active.join("sentinel"), b"directory\n").expect("directory sentinel");
        let target = target_dir(temporary.path());

        let error = cas_activate_path(
            &target,
            relative,
            Some(&sha256_digest(b"expected\n")),
            Some(b"incoming\n"),
            None,
            None,
            None,
        )
        .expect_err("directory claim must require recovery");

        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert!(!active.exists());
        let recovery = claimed_recovery_path(&active);
        assert!(recovery.is_dir());
        assert_eq!(
            fs::read(recovery.join("sentinel")).expect("directory sentinel"),
            b"directory\n"
        );
    }

    #[test]
    fn replacement_staging_failures_retain_exact_claimed_bytes() {
        for fault in [CasStageFault::Open, CasStageFault::Write] {
            let temporary = tempdir().expect("temporary target");
            let relative = Path::new(".agents/directives/test.md");
            let active = temporary.path().join(relative);
            fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
            fs::write(&active, b"expected\n").expect("expected bytes");
            let target = target_dir(temporary.path());

            let error = cas_activate_path(
                &target,
                relative,
                Some(&sha256_digest(b"expected\n")),
                Some(b"incoming\n"),
                None,
                None,
                Some(fault),
            )
            .expect_err("injected staging failure");

            let UpdateError::RecoveryRequired(message) = error else {
                panic!("staging failure must require recovery");
            };
            let recovery = claimed_recovery_path(&active);
            assert!(message.contains(
                &recovery
                    .strip_prefix(temporary.path())
                    .expect("relative recovery")
                    .display()
                    .to_string()
            ));
            assert!(!active.exists());
            assert_eq!(fs::read(recovery).expect("claimed prior"), b"expected\n");
        }
    }

    #[test]
    fn preclaim_empty_quarantine_is_cleaned_but_foreign_entry_is_preserved() {
        let temporary = tempdir().expect("temporary target");
        let relative = Path::new(".agents/directives/test.md");
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::write(&active, b"before\n").expect("before bytes");
        let target = target_dir(temporary.path());
        let claim_directory = temporary
            .path()
            .join(claim_directory_path(relative).expect("claim directory"));
        fs::create_dir(&claim_directory).expect("empty pre-claim quarantine");

        assert!(reconcile_discoverable_claim(
            &target,
            relative,
            Some(&sha256_digest(b"before\n")),
            Some(&sha256_digest(b"after\n")),
        )
        .expect("empty pre-claim cleanup"));
        assert!(!claim_directory.exists());
        assert_eq!(fs::read(&active).expect("active before"), b"before\n");

        let error = cas_activate_path(
            &target,
            relative,
            Some(&sha256_digest(b"before\n")),
            Some(b"after\n"),
            None,
            None,
            Some(CasStageFault::ClaimRenameCleanup),
        )
        .expect_err("injected rename cleanup failure");
        let UpdateError::RecoveryRequired(message) = error else {
            panic!("rename cleanup failure must require recovery");
        };
        assert!(message.contains(
            &claim_directory_path(relative)
                .expect("claim directory")
                .display()
                .to_string()
        ));
        assert_eq!(fs::read(&active).expect("active before"), b"before\n");
        assert!(claim_directory.join("foreign-entry").is_file());

        let recovery_error = reconcile_discoverable_claim(
            &target,
            relative,
            Some(&sha256_digest(b"before\n")),
            Some(&sha256_digest(b"after\n")),
        )
        .expect_err("foreign pre-claim entry must remain");
        assert!(matches!(recovery_error, UpdateError::RecoveryRequired(_)));
        assert!(claim_directory.join("foreign-entry").is_file());
    }

    #[test]
    fn recovery_reconciles_a_discoverable_ordinary_claim() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/test.md";
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(temporary.path().join(".hive/runtime")).expect("runtime");
        fs::write(&active, b"before\n").expect("before bytes");
        let target = target_dir(temporary.path());
        let change = JournalChange {
            path: relative.to_owned(),
            claim_path: claim_recovery_path(Path::new(relative))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: Some(sha256_digest(b"before\n")),
            after_digest: Some(sha256_digest(b"after\n")),
            backup_path: Some("files/0000.bin".to_owned()),
        };
        let journal = sign_journal(UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-ordinary-claim".to_owned(),
            plan_digest: sha256_digest(b"plan"),
            backup_root: ".hive/backups/project-upgrade/upgrade-ordinary-claim".to_owned(),
            changes: vec![change],
            journal_digest: String::new(),
        })
        .expect("journal");
        fs::write(
            temporary.path().join(JOURNAL_PATH),
            canonical_json(&journal).expect("canonical journal"),
        )
        .expect("journal bytes");
        drop(claim_destination(&target, Path::new(relative), None).expect("ordinary claim"));

        let result = recover(&target).expect("claim recovery");

        assert_eq!(result.code, "hive.project-upgrade-recovered");
        assert_eq!(fs::read(&active).expect("restored before"), b"before\n");
        assert!(!temporary
            .path()
            .join(claim_recovery_path(Path::new(relative)).expect("claim path"))
            .exists());
        assert!(!temporary.path().join(JOURNAL_PATH).exists());
    }

    #[test]
    fn recovery_finds_a_journal_hidden_in_its_fixed_claim_locator() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/test.md";
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(temporary.path().join(".hive/runtime")).expect("runtime");
        fs::write(&active, b"before\n").expect("before bytes");
        let target = target_dir(temporary.path());
        let change = JournalChange {
            path: relative.to_owned(),
            claim_path: claim_recovery_path(Path::new(relative))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: Some(sha256_digest(b"before\n")),
            after_digest: Some(sha256_digest(b"after\n")),
            backup_path: Some("files/0000.bin".to_owned()),
        };
        let journal = sign_journal(UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-hidden-journal".to_owned(),
            plan_digest: sha256_digest(b"plan"),
            backup_root: ".hive/backups/project-upgrade/upgrade-hidden-journal".to_owned(),
            changes: vec![change],
            journal_digest: String::new(),
        })
        .expect("journal");
        fs::write(
            temporary.path().join(JOURNAL_PATH),
            canonical_json(&journal).expect("canonical journal"),
        )
        .expect("journal bytes");
        drop(claim_destination(&target, Path::new(JOURNAL_PATH), None).expect("journal claim"));
        let journal_recovery = temporary
            .path()
            .join(claim_recovery_path(Path::new(JOURNAL_PATH)).expect("journal recovery path"));
        assert!(journal_recovery.is_file());
        assert!(!temporary.path().join(JOURNAL_PATH).exists());

        let result = recover(&target).expect("hidden journal recovery");

        assert_eq!(result.code, "hive.project-upgrade-recovered");
        assert_eq!(fs::read(&active).expect("unchanged before"), b"before\n");
        assert!(!journal_recovery.exists());
        assert!(!temporary.path().join(JOURNAL_PATH).exists());
    }

    #[test]
    fn recovery_cleans_a_restored_journal_with_retained_claim() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/test.md";
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(temporary.path().join(".hive/runtime")).expect("runtime");
        fs::write(&active, b"before\n").expect("before bytes");
        let target = target_dir(temporary.path());
        let change = JournalChange {
            path: relative.to_owned(),
            claim_path: claim_recovery_path(Path::new(relative))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: Some(sha256_digest(b"before\n")),
            after_digest: Some(sha256_digest(b"after\n")),
            backup_path: Some("files/0000.bin".to_owned()),
        };
        let journal = sign_journal(UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-restored-journal".to_owned(),
            plan_digest: sha256_digest(b"plan"),
            backup_root: ".hive/backups/project-upgrade/upgrade-restored-journal".to_owned(),
            changes: vec![change],
            journal_digest: String::new(),
        })
        .expect("journal");
        fs::write(
            temporary.path().join(JOURNAL_PATH),
            canonical_json(&journal).expect("canonical journal"),
        )
        .expect("journal bytes");
        let claim = claim_destination(&target, Path::new(JOURNAL_PATH), None)
            .expect("journal should claim");
        claim
            .quarantine
            .as_ref()
            .expect("claim quarantine")
            .hard_link("claimed.bin", &claim.parent, &claim.destination_name)
            .expect("restore canonical journal without claim cleanup");
        drop(claim);

        let result = recover(&target).expect("retained journal claim recovery");

        assert_eq!(result.code, "hive.project-upgrade-recovered");
        assert!(!temporary.path().join(JOURNAL_PATH).exists());
        assert!(!temporary
            .path()
            .join(claim_directory_path(Path::new(JOURNAL_PATH)).expect("journal claim directory"))
            .exists());
    }

    #[test]
    fn recovery_cleans_post_publication_creation_staging() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/new.md";
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(temporary.path().join(".hive/runtime")).expect("runtime");
        let target = target_dir(temporary.path());
        let change = JournalChange {
            path: relative.to_owned(),
            claim_path: claim_recovery_path(Path::new(relative))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: None,
            after_digest: Some(sha256_digest(b"after\n")),
            backup_path: None,
        };
        let journal = sign_journal(UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-new-path".to_owned(),
            plan_digest: sha256_digest(b"plan"),
            backup_root: ".hive/backups/project-upgrade/upgrade-new-path".to_owned(),
            changes: vec![change],
            journal_digest: String::new(),
        })
        .expect("journal");
        fs::write(
            temporary.path().join(JOURNAL_PATH),
            canonical_json(&journal).expect("canonical journal"),
        )
        .expect("journal bytes");

        let error = cas_activate_path(
            &target,
            Path::new(relative),
            None,
            Some(b"after\n"),
            None,
            None,
            Some(CasStageFault::CreateReplacementCleanup),
        )
        .expect_err("injected cleanup failure");
        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert_eq!(fs::read(&active).expect("published bytes"), b"after\n");

        let result = recover(&target).expect("creation cleanup recovery");

        assert_eq!(result.code, "hive.project-upgrade-recovered");
        assert_eq!(fs::read(&active).expect("published bytes"), b"after\n");
        assert!(!temporary
            .path()
            .join(
                claim_recovery_path(Path::new(relative))
                    .expect("claim path")
                    .parent()
                    .expect("claim parent")
            )
            .exists());
        assert!(!temporary.path().join(JOURNAL_PATH).exists());
    }

    #[test]
    fn cleaned_prejournal_backup_failure_never_suggests_unusable_recovery() {
        let temporary = tempdir().expect("temporary target");
        let target = target_dir(temporary.path());
        let error = resolve_prejournal_failure(
            &target,
            ".hive/backups/project-upgrade/upgrade-prejournal",
            UpdateError::RecoveryRequired("injected backup cleanup failure".to_owned()),
        );

        assert!(matches!(error, UpdateError::Internal(_)));
        assert!(failure(&error).next_action.is_none());
    }

    #[test]
    fn rollback_cas_preserves_third_digest_instead_of_restoring_over_it() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/test.md";
        let active = temporary.path().join(relative);
        let backup = temporary
            .path()
            .join(".hive/backups/project-upgrade/upgrade-test/files/0000.bin");
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(backup.parent().expect("backup parent")).expect("backup parent");
        fs::write(&active, b"third\n").expect("third bytes");
        fs::write(&backup, b"before\n").expect("backup bytes");
        let target = target_dir(temporary.path());
        let journal = UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-test".to_owned(),
            plan_digest: sha256_digest(b"plan"),
            backup_root: ".hive/backups/project-upgrade/upgrade-test".to_owned(),
            changes: Vec::new(),
            journal_digest: String::new(),
        };
        let change = JournalChange {
            path: relative.to_owned(),
            claim_path: claim_recovery_path(Path::new(relative))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: Some(sha256_digest(b"before\n")),
            after_digest: Some(sha256_digest(b"after\n")),
            backup_path: Some("files/0000.bin".to_owned()),
        };

        let error = restore_before(&target, &journal, &change, change.after_digest.as_deref())
            .expect_err("rollback third digest conflict");

        assert!(matches!(error, UpdateError::Conflict(_)));
        assert_eq!(fs::read(&active).expect("active third"), b"third\n");
    }

    #[test]
    fn marker_free_shared_guidance_appends_without_changing_foreign_bytes() {
        let foreign = b"# Foreign\r\n<!-- omx:block -->\r\n";
        let marker = b"<!-- AIGENT-HIVE:START -->\n# Hive\n<!-- AIGENT-HIVE:END -->\n";
        let merged = replace_marker(foreign, marker).expect("marker append");
        assert!(merged.starts_with(foreign));
        assert!(merged.ends_with(marker));
        assert_eq!(
            merged
                .windows(b"<!-- AIGENT-HIVE:START -->".len())
                .filter(|window| *window == b"<!-- AIGENT-HIVE:START -->")
                .count(),
            1
        );
    }

    #[test]
    fn recovery_cleans_a_published_journal_creation_claim() {
        let temporary = tempdir().expect("temporary target");
        let relative = ".agents/directives/test.md";
        let active = temporary.path().join(relative);
        fs::create_dir_all(active.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(temporary.path().join(".hive/runtime")).expect("runtime");
        fs::write(&active, b"before\n").expect("before bytes");
        let target = target_dir(temporary.path());
        let change = JournalChange {
            path: relative.to_owned(),
            claim_path: claim_recovery_path(Path::new(relative))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: Some(sha256_digest(b"before\n")),
            after_digest: Some(sha256_digest(b"after\n")),
            backup_path: Some("files/0000.bin".to_owned()),
        };
        let journal = sign_journal(UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-journal-creation".to_owned(),
            plan_digest: sha256_digest(b"plan"),
            backup_root: ".hive/backups/project-upgrade/upgrade-journal-creation".to_owned(),
            changes: vec![change],
            journal_digest: String::new(),
        })
        .expect("journal");
        let journal_bytes = canonical_json(&journal).expect("canonical journal");
        let error = create_new_cas(
            &target,
            Path::new(JOURNAL_PATH),
            &journal_bytes,
            Some(CasStageFault::CreateReplacementCleanup),
        )
        .expect_err("injected journal staging cleanup failure");
        assert!(matches!(error, UpdateError::RecoveryRequired(_)));
        assert_eq!(
            fs::read(temporary.path().join(JOURNAL_PATH)).expect("published journal"),
            journal_bytes
        );

        let result = recover(&target).expect("published journal claim recovery");

        assert_eq!(result.code, "hive.project-upgrade-recovered");
        assert!(!temporary.path().join(JOURNAL_PATH).exists());
        assert!(!temporary
            .path()
            .join(claim_directory_path(Path::new(JOURNAL_PATH)).expect("claim directory"))
            .exists());
    }

    #[test]
    fn malformed_or_nested_shared_markers_fail_closed() {
        for bytes in [
            b"<!-- AIGENT-HIVE:START -->\nmissing".as_slice(),
            b"<!-- AIGENT-HIVE:START --><!-- AIGENT-HIVE:START --><!-- AIGENT-HIVE:END -->"
                .as_slice(),
            b"<!-- AIGENT-HIVE:END --><!-- AIGENT-HIVE:START -->".as_slice(),
        ] {
            assert!(matches!(
                extract_marker(bytes),
                Err(UpdateError::Conflict(_))
            ));
        }
    }

    #[test]
    fn journal_rejects_noncanonical_backup_binding_and_uppercase_digest() {
        let change = JournalChange {
            path: ".hive/config/project-overrides.json".to_owned(),
            claim_path: claim_recovery_path(Path::new(".hive/config/project-overrides.json"))
                .expect("claim path")
                .to_string_lossy()
                .into_owned(),
            before_digest: Some(digest('a')),
            after_digest: Some(digest('b')),
            backup_path: Some("files/0000.bin".to_owned()),
        };
        let valid = sign_journal(UpgradeJournal {
            schema_version: 1,
            transaction_id: "upgrade-abc-1".to_owned(),
            plan_digest: digest('c'),
            backup_root: ".hive/backups/project-upgrade/upgrade-abc-1".to_owned(),
            changes: vec![change],
            journal_digest: String::new(),
        })
        .expect("signed journal");
        validate_journal(&valid).expect("valid journal");

        let escaped = sign_journal(UpgradeJournal {
            backup_root: ".hive/backups/project-upgrade/other".to_owned(),
            journal_digest: String::new(),
            ..valid
        })
        .expect("resigned journal");
        assert!(matches!(
            validate_journal(&escaped),
            Err(UpdateError::Verification(_))
        ));
        assert!(!valid_digest(&format!("sha256:{}", "A".repeat(64))));
    }

    #[test]
    fn project_base_requires_exact_canonical_bytes() {
        let temporary = tempdir().expect("temporary target");
        let config = temporary.path().join(".hive/config");
        fs::create_dir_all(&config).expect("config");
        let content = "# directive\n";
        let mut unsigned = json!({
            "schema_version": 1,
            "product_version": env!("CARGO_PKG_VERSION"),
            "files": [{
                "path": ".agents/directives/test.md",
                "kind": "directive",
                "content_digest": sha256_digest(content.as_bytes()),
                "content": content
            }]
        });
        let digest = sha256_digest(
            &serde_json_canonicalizer::to_vec(&unsigned).expect("canonical unsigned ledger"),
        );
        unsigned
            .as_object_mut()
            .expect("object")
            .insert("ledger_digest".to_owned(), Value::String(digest));
        let mut bytes =
            serde_json_canonicalizer::to_vec(&unsigned).expect("canonical signed ledger");
        bytes.extend_from_slice(b"\n\n");
        fs::write(config.join("project-base.json"), bytes).expect("base ledger");
        let incoming = BTreeMap::from([(
            ".agents/directives/test.md".to_owned(),
            content.as_bytes().to_vec(),
        )]);
        assert!(matches!(
            read_base_ledger(&target_dir(temporary.path()), &incoming),
            Err(UpdateError::Verification(_))
        ));
    }

    #[test]
    fn current_project_base_rejects_exact_bytes_with_tampered_kind() {
        let content = "# directive\n";
        let ledger = BaseLedger {
            schema_version: 1,
            product_version: env!("CARGO_PKG_VERSION").to_owned(),
            files: vec![BaseFile {
                path: ".agents/directives/test.md".to_owned(),
                kind: "skill".to_owned(),
                content_digest: sha256_digest(content.as_bytes()),
                content: content.to_owned(),
            }],
            ledger_digest: digest('a'),
        };
        let incoming = BTreeMap::from([(
            ".agents/directives/test.md".to_owned(),
            content.as_bytes().to_vec(),
        )]);

        assert!(matches!(
            authenticate_current_base(&ledger, &incoming),
            Err(UpdateError::Verification(_))
        ));
    }

    #[test]
    fn supported_historical_project_base_is_accepted() {
        let temporary = tempdir().expect("temporary target");
        let config = temporary.path().join(".hive/config");
        fs::create_dir_all(&config).expect("config");
        fs::write(
            config.join("project-base.json"),
            signed_base_ledger("0.3.0", &[]),
        )
        .expect("base ledger");

        let ledger = read_base_ledger(&target_dir(temporary.path()), &BTreeMap::new())
            .expect("supported historical base")
            .expect("base ledger");

        assert_eq!(ledger.product_version, "0.3.0");
        assert!(ledger.files.is_empty());
    }

    #[test]
    fn historical_registry_covers_every_shipped_release_and_host_root() {
        let temporary = tempdir().expect("temporary target");
        let target = target_dir(temporary.path());
        for version in ["0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0"] {
            let skills = historical_builtin_skills(version).expect("historical registry release");
            for root in [".agents/skills", ".claude/skills"] {
                let ledger = BaseLedger {
                    schema_version: 1,
                    product_version: version.to_owned(),
                    files: skills
                        .iter()
                        .map(|skill| BaseFile {
                            path: format!("{root}/{}/SKILL.md", skill.name),
                            kind: "skill".to_owned(),
                            content_digest: skill.content_digest.clone(),
                            content: String::new(),
                        })
                        .collect(),
                    ledger_digest: String::new(),
                };
                authenticate_historical_base(&target, &ledger).expect("historical inventory");
            }
        }
    }

    #[test]
    fn unknown_historical_project_base_is_rejected() {
        let temporary = tempdir().expect("temporary target");
        let config = temporary.path().join(".hive/config");
        fs::create_dir_all(&config).expect("config");
        fs::write(
            config.join("project-base.json"),
            signed_base_ledger("0.6.1", &[]),
        )
        .expect("base ledger");

        assert!(matches!(
            read_base_ledger(&target_dir(temporary.path()), &BTreeMap::new()),
            Err(UpdateError::Unsupported(_))
        ));
    }

    #[test]
    fn full_historical_registry_authenticates_all_projection_kinds_exactly() {
        let expected = BTreeMap::from([
            ("AGENTS.md".to_owned(), b"marker\n".to_vec()),
            (
                ".agents/directives/01-behavior.md".to_owned(),
                b"directive\n".to_vec(),
            ),
            (
                ".agents/skills/setup-harness/SKILL.md".to_owned(),
                b"skill\n".to_vec(),
            ),
        ]);
        let files = expected
            .iter()
            .map(|(path, bytes)| BaseFile {
                path: path.clone(),
                kind: if is_shared(path) {
                    "shared-marker"
                } else if path.contains("/skills/") {
                    "skill"
                } else {
                    "directive"
                }
                .to_owned(),
                content_digest: sha256_digest(bytes),
                content: String::from_utf8(bytes.clone()).expect("UTF-8"),
            })
            .collect::<Vec<_>>();
        let ledger = BaseLedger {
            schema_version: 1,
            product_version: "0.7.0".to_owned(),
            files: files.clone(),
            ledger_digest: digest('a'),
        };
        let expected = full_registry(&expected);
        authenticate_full_historical_base(&ledger, &expected).expect("exact full registry");

        let mut partial = ledger.clone();
        partial.files.pop();
        assert!(matches!(
            authenticate_full_historical_base(&partial, &expected),
            Err(UpdateError::Verification(_))
        ));

        let mut tampered = ledger;
        tampered.files[0].content.push_str("tampered");
        assert!(matches!(
            authenticate_full_historical_base(&tampered, &expected),
            Err(UpdateError::Verification(_))
        ));

        let mut kind_tampered = BaseLedger {
            schema_version: 1,
            product_version: "0.7.0".to_owned(),
            files,
            ledger_digest: digest('b'),
        };
        kind_tampered.files[0].kind = "skill".to_owned();
        assert!(matches!(
            authenticate_full_historical_base(&kind_tampered, &expected),
            Err(UpdateError::Verification(_))
        ));
    }

    #[test]
    fn previous_full_release_fixture_upgrades_unmodified_bytes_to_current() {
        let previous = b"previous full-release directive\n";
        let current = b"current directive\n";
        let expected = full_registry(&BTreeMap::from([(
            ".agents/directives/01-behavior.md".to_owned(),
            previous.to_vec(),
        )]));
        let ledger = BaseLedger {
            schema_version: 1,
            product_version: "0.7.0".to_owned(),
            files: vec![BaseFile {
                path: ".agents/directives/01-behavior.md".to_owned(),
                kind: "directive".to_owned(),
                content_digest: sha256_digest(previous),
                content: String::from_utf8(previous.to_vec()).expect("UTF-8"),
            }],
            ledger_digest: digest('a'),
        };
        authenticate_full_historical_base(&ledger, &expected).expect("previous full registry");

        let merged = three_way_merge(
            Path::new(".agents/directives/01-behavior.md"),
            Some(previous),
            Some(previous),
            Some(current),
        )
        .expect("previous full release to current merge");

        assert_eq!(merged.bytes.as_deref(), Some(current.as_ref()));
        assert_eq!(merged.disposition, MergeDisposition::IncomingReplace);
    }

    #[test]
    fn missing_or_duplicate_historical_entries_are_rejected() {
        for files in [
            Vec::new(),
            vec![
                BaseFile {
                    path: ".agents/skills/hive-simple-question/SKILL.md".to_owned(),
                    kind: "skill".to_owned(),
                    content_digest: sha256_digest(b"duplicate\n"),
                    content: "duplicate\n".to_owned(),
                },
                BaseFile {
                    path: ".agents/skills/hive-simple-question/SKILL.md".to_owned(),
                    kind: "skill".to_owned(),
                    content_digest: sha256_digest(b"duplicate\n"),
                    content: "duplicate\n".to_owned(),
                },
            ],
        ] {
            let temporary = tempdir().expect("temporary target");
            let config = temporary.path().join(".hive/config");
            fs::create_dir_all(&config).expect("config");
            fs::write(
                config.join("project-base.json"),
                signed_base_ledger("0.4.0", &files),
            )
            .expect("base ledger");
            assert!(matches!(
                read_base_ledger(&target_dir(temporary.path()), &BTreeMap::new()),
                Err(UpdateError::Verification(_))
            ));
        }
    }

    #[test]
    fn tampered_historical_project_base_preserves_active_bytes() {
        let temporary = tempdir().expect("temporary target");
        let config = temporary.path().join(".hive/config");
        let active_path = temporary
            .path()
            .join(".agents/skills/hive-simple-question/SKILL.md");
        fs::create_dir_all(active_path.parent().expect("active parent")).expect("active parent");
        fs::create_dir_all(&config).expect("config");
        let active = b"user active bytes\n";
        fs::write(&active_path, active).expect("active bytes");
        let tampered = "tampered historical bytes\n";
        fs::write(
            config.join("project-base.json"),
            signed_base_ledger(
                "0.4.0",
                &[BaseFile {
                    path: ".agents/skills/hive-simple-question/SKILL.md".to_owned(),
                    kind: "skill".to_owned(),
                    content_digest: sha256_digest(tampered.as_bytes()),
                    content: tampered.to_owned(),
                }],
            ),
        )
        .expect("base ledger");

        assert!(matches!(
            read_base_ledger(&target_dir(temporary.path()), &BTreeMap::new()),
            Err(UpdateError::Verification(_))
        ));
        assert_eq!(
            fs::read(active_path).expect("active bytes after failure"),
            active
        );
    }

    #[test]
    fn recovery_without_authenticated_journal_cleans_only_empty_claim() {
        let empty = tempdir().expect("empty claim target");
        let empty_claim = empty
            .path()
            .join(claim_directory_path(Path::new(JOURNAL_PATH)).expect("claim directory"));
        fs::create_dir_all(&empty_claim).expect("empty claim directory");
        let empty_target = target_dir(empty.path());

        let result = recover(&empty_target).expect("empty unauthenticated claim cleanup");

        assert_eq!(result.code, "hive.project-upgrade-recovered");
        assert!(!empty_claim.exists());

        for entry_name in ["replacement.bin", "claimed.bin", "foreign-sentinel"] {
            let temporary = tempdir().expect("foreign claim target");
            let claim = temporary
                .path()
                .join(claim_directory_path(Path::new(JOURNAL_PATH)).expect("claim directory"));
            fs::create_dir_all(&claim).expect("claim directory");
            let sentinel = format!("foreign-{entry_name}\n").into_bytes();
            let sentinel_path = claim.join(entry_name);
            fs::write(&sentinel_path, &sentinel).expect("foreign sentinel");
            let digest = sha256_digest(&sentinel);
            let target = target_dir(temporary.path());

            let Err(error) = recover(&target) else {
                panic!("foreign claim must fail closed");
            };

            let result = failure(&error);
            assert_ne!(result.exit_code, 0);
            if entry_name == "claimed.bin" {
                let locator = claim_recovery_path(Path::new(JOURNAL_PATH))
                    .expect("claimed journal locator")
                    .display()
                    .to_string();
                assert!(matches!(error, UpdateError::Verification(_)));
                assert!(result.message.contains(&locator));
                assert!(result
                    .next_action
                    .as_deref()
                    .is_some_and(|action| action.contains(&locator)));
            }
            assert_eq!(
                sha256_digest(&fs::read(&sentinel_path).expect("preserved sentinel")),
                digest
            );
            assert_eq!(
                fs::read(&sentinel_path).expect("preserved sentinel"),
                sentinel
            );
        }
    }
}
