use hive_core::{ensure_no_symlink_ancestors, sha256_digest};
use hive_wiki::shared::{
    load_project_registry, query_shared, rebuild_shared_index, validate_shared_index,
    SHARED_INDEX_RELATIVE,
};
use hive_wiki::{
    delete_page, ingest, lint, promote, query, rebuild_index, suppress, LintIssue, LintSeverity,
    PromotionCategory, PromotionMode, SuppressionEntry, WikiError,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const KNOWLEDGE_USAGE: &str = "\
Canonical Markdown knowledge and disposable SQLite index.

USAGE:
    hive knowledge ingest --target <dir> --source <file> --wiki <draft.md> [--user-root <dir>] --output json
    hive knowledge query --target <dir> (--text <query>|--tag <tag>) [--limit <1..100>] [--user-root <dir>] --output json
    hive knowledge promote --target <project> --user-root <dir> --page-id <id> --category fact|preference|workflow (--dry-run|--apply) --output json
    hive knowledge lint --target <dir> [--user-root <dir>] --output json
    hive knowledge delete --target <dir> --page-id <id> --reason <text> [--replacement <locator>] --timestamp <RFC3339> [--user-root <dir>] --output json
    hive knowledge suppress --target <dir> --fingerprint <sha256:...> --source-locator <locator> --reason <text> [--replacement <locator>] --timestamp <RFC3339> [--user-root <dir>] --output json
    hive index rebuild (--target <legacy-project>|--user-root <dir>) --output json
";

const LEGACY_DERIVED_RELATIVES: [&str; 4] = [
    ".hive/index/hive.sqlite3",
    ".hive/index/hive.sqlite3-wal",
    ".hive/index/hive.sqlite3-shm",
    ".hive/index/.stale",
];

#[derive(Serialize)]
struct KnowledgeResult {
    schema_version: u32,
    action: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
    changed_paths: Vec<String>,
    evidence: Vec<KnowledgeEvidence>,
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize)]
struct KnowledgeEvidence {
    kind: &'static str,
    locator: String,
    digest: String,
}

pub(crate) fn run_knowledge(arguments: &[String]) -> ExitCode {
    if is_help(arguments) {
        print!("{KNOWLEDGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result =
        match arguments.first().map(String::as_str) {
            Some("ingest") => run_ingest(&arguments[1..])
                .unwrap_or_else(|error| failure("IngestKnowledge", &error)),
            Some("query") => {
                run_query(&arguments[1..]).unwrap_or_else(|error| failure("QueryKnowledge", &error))
            }
            Some("promote") => run_promote(&arguments[1..])
                .unwrap_or_else(|error| failure("PromoteKnowledge", &error)),
            Some("lint") => {
                run_lint(&arguments[1..]).unwrap_or_else(|error| failure("LintKnowledge", &error))
            }
            Some("delete") => run_delete(&arguments[1..])
                .unwrap_or_else(|error| failure("DeleteKnowledge", &error)),
            Some("suppress") => run_suppress(&arguments[1..])
                .unwrap_or_else(|error| failure("SuppressKnowledge", &error)),
            Some(action) => failure(
                "IngestKnowledge",
                &WikiError::InvalidInput(format!("unknown knowledge action: {action}")),
            ),
            None => failure(
                "IngestKnowledge",
                &WikiError::InvalidInput("knowledge requires an action".to_owned()),
            ),
        };
    emit(&result);
    ExitCode::from(result.exit_code)
}

pub(crate) fn run_index(arguments: &[String]) -> ExitCode {
    if is_help(arguments) {
        print!("{KNOWLEDGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = if arguments.first().map(String::as_str) == Some("rebuild") {
        parse_options(&arguments[1..], &["--target", "--user-root"]).and_then(|options| {
            let user_root = optional(&options, "--user-root");
            let target = optional(&options, "--target");
            let (changed_paths, logical_digest, data) = match (target, user_root) {
                (None, Some(user_root)) => {
                    require_shared_wiki_enabled(Path::new(user_root))?;
                    let outcome = rebuild_shared_index(&PathBuf::from(user_root))?;
                    let changed_paths = outcome.changed_paths.clone();
                    let logical_digest = outcome.logical_digest.clone();
                    let data = serde_json::to_value(&outcome)
                        .map_err(|error| WikiError::Io(error.to_string()))?;
                    (changed_paths, logical_digest, data)
                }
                (Some(target), None) => {
                    authorize_legacy_target(Path::new(target))?;
                    let outcome = rebuild_index(&PathBuf::from(target))?;
                    let changed_paths = outcome.changed_paths.clone();
                    let logical_digest = outcome.logical_digest.clone();
                    let data = serde_json::to_value(&outcome)
                        .map_err(|error| WikiError::Io(error.to_string()))?;
                    (changed_paths, logical_digest, data)
                }
                _ => {
                    return Err(WikiError::InvalidInput(
                        "index rebuild requires exactly one of --target or --user-root".to_owned(),
                    ));
                }
            };
            Ok(success(
                "RebuildKnowledgeIndex",
                "hive.index-rebuilt",
                "knowledge index rebuilt from canonical sources",
                changed_paths,
                SHARED_INDEX_RELATIVE,
                &logical_digest,
                data,
            ))
        })
    } else {
        Err(WikiError::InvalidInput(
            "index requires the rebuild action".to_owned(),
        ))
    }
    .unwrap_or_else(|error| failure("RebuildKnowledgeIndex", &error));
    emit(&result);
    ExitCode::from(result.exit_code)
}

fn run_ingest(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &["--target", "--source", "--wiki", "--user-root"],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let source = PathBuf::from(required(&options, "--source")?);
    let wiki = PathBuf::from(required(&options, "--wiki")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let outcome = ingest(&target, &source, &wiki)?;
    let mutation =
        serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = finish_shared_mutation(
        &target,
        shared.as_ref(),
        outcome.changed_paths,
        mutation,
        ".hive/knowledge",
    )?;
    Ok(success(
        "IngestKnowledge",
        "hive.knowledge-ingested",
        "Raw revision and Wiki page integrated serially",
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

fn run_query(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &["--target", "--text", "--tag", "--limit", "--user-root"],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let text = optional(&options, "--text");
    let tag = optional(&options, "--tag");
    let limit = optional(&options, "--limit").map_or(Ok(20_usize), |value| {
        value
            .parse::<usize>()
            .map_err(|_| WikiError::InvalidInput("query limit must be an integer".to_owned()))
    })?;
    if let Some(user_root) = optional(&options, "--user-root") {
        return run_shared_query(&target, &PathBuf::from(user_root), text, tag, limit);
    }
    authorize_legacy_target(&target)?;
    let project_hits = query(&target, text, tag, limit)?;
    let mut seen = BTreeSet::new();
    let mut hits = Vec::new();
    for hit in &project_hits {
        seen.insert(hit.content_digest.clone());
        hits.push(scoped_hit("project", "project-local", hit)?);
    }
    let root_hit_count = 0_usize;
    let data = json!({
        "hits": hits,
        "project_hit_count": project_hits.len(),
        "root_hit_count": root_hit_count,
        "precedence": "project-first"
    });
    let digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok(success(
        "QueryKnowledge",
        "hive.knowledge-query-complete",
        "knowledge query completed",
        Vec::new(),
        ".hive/index/hive.sqlite3",
        &digest,
        data,
    ))
}

fn run_shared_query(
    target: &Path,
    user_root: &Path,
    text: Option<&str>,
    tag: Option<&str>,
    limit: usize,
) -> Result<KnowledgeResult, WikiError> {
    require_shared_wiki_enabled(user_root)?;
    let canonical_user = user_root
        .canonicalize()
        .map_err(|error| WikiError::Io(format!("cannot canonicalize user root: {error}")))?;
    let canonical_target = target
        .canonicalize()
        .map_err(|error| WikiError::Io(format!("cannot canonicalize query target: {error}")))?;
    let registry = load_project_registry(user_root)?;
    let current_project_id = if canonical_target == canonical_user {
        None
    } else {
        Some(
            registry
                .projects
                .iter()
                .find(|project| project.enabled && project.root == canonical_target)
                .map(|project| project.id.clone())
                .ok_or_else(|| {
                    WikiError::InvalidInput(
                        "query target is not enabled in the project registry".to_owned(),
                    )
                })?,
        )
    };
    let hits = query_shared(
        user_root,
        (canonical_target != canonical_user).then_some(canonical_target.as_path()),
        text,
        tag,
        limit,
    )?;
    let project_hit_count = hits
        .iter()
        .filter(|hit| {
            current_project_id
                .as_ref()
                .is_some_and(|project_id| hit.source_project == *project_id)
        })
        .count();
    let root_hit_count = hits
        .iter()
        .filter(|hit| hit.source_project == "user-root")
        .count();
    let cross_project_hit_count = hits
        .len()
        .saturating_sub(project_hit_count + root_hit_count);
    let data = json!({
        "hits": hits,
        "project_hit_count": project_hit_count,
        "root_hit_count": root_hit_count,
        "cross_project_hit_count": cross_project_hit_count,
        "precedence": "own-project,user-root,shared"
    });
    let digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok(success(
        "QueryKnowledge",
        "hive.knowledge-query-complete",
        "shared knowledge query completed",
        Vec::new(),
        SHARED_INDEX_RELATIVE,
        &digest,
        data,
    ))
}

fn scoped_hit(
    scope: &str,
    provenance: &str,
    hit: &hive_wiki::QueryHit,
) -> Result<Value, WikiError> {
    let mut value = serde_json::to_value(hit).map_err(|error| WikiError::Io(error.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| WikiError::Io("query hit did not serialize as an object".to_owned()))?;
    object.insert("scope".to_owned(), Value::String(scope.to_owned()));
    object.insert(
        "provenance".to_owned(),
        Value::String(provenance.to_owned()),
    );
    Ok(value)
}

fn run_promote(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let (options, mode) = parse_promotion_options(arguments)?;
    let target = PathBuf::from(required(&options, "--target")?);
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    shared_mutation_target(&target, &user_root, false)?;
    let page_id = required(&options, "--page-id")?;
    let category = match required(&options, "--category")? {
        "fact" => PromotionCategory::Fact,
        "preference" => PromotionCategory::Preference,
        "workflow" => PromotionCategory::Workflow,
        _ => {
            return Err(WikiError::InvalidInput(
                "promotion category must be fact, preference, or workflow".to_owned(),
            ));
        }
    };
    let outcome = promote(&target, &user_root, page_id, category, mode)?;
    let changed_paths = outcome.changed_paths.clone();
    let code = if mode == PromotionMode::Apply {
        "hive.knowledge-promoted"
    } else {
        "hive.knowledge-promotion-planned"
    };
    let message = if mode == PromotionMode::Apply {
        "project knowledge promoted into the canonical user-root store"
    } else {
        "knowledge promotion dry run completed without canonical mutation"
    };
    let digest = outcome.plan_digest.clone();
    let data = serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = if mode == PromotionMode::Apply {
        finish_shared_mutation(
            &user_root,
            Some(&SharedMutationTarget {
                user_root: user_root.clone(),
                target_kind: SharedTargetKind::UserRoot,
            }),
            changed_paths,
            data,
            ".hive/knowledge",
        )?
    } else {
        (changed_paths, ".hive/knowledge", digest, data)
    };
    Ok(success(
        "PromoteKnowledge",
        code,
        message,
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

type ValuedOptions<'a> = Vec<(&'a str, &'a str)>;

fn parse_promotion_options(
    arguments: &[String],
) -> Result<(ValuedOptions<'_>, PromotionMode), WikiError> {
    let mut valued = Vec::new();
    let mut mode = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if matches!(option, "--dry-run" | "--apply") {
            let candidate = if option == "--apply" {
                PromotionMode::Apply
            } else {
                PromotionMode::DryRun
            };
            if mode.replace(candidate).is_some() {
                return Err(WikiError::InvalidInput(
                    "promotion requires exactly one mode".to_owned(),
                ));
            }
            index += 1;
            continue;
        }
        if !matches!(
            option,
            "--target" | "--user-root" | "--page-id" | "--category" | "--output"
        ) {
            return Err(WikiError::InvalidInput(format!(
                "unknown knowledge option: {option}"
            )));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| WikiError::InvalidInput(format!("missing value for {option}")))?;
        if valued.iter().any(|(existing, _)| *existing == option) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate knowledge option: {option}"
            )));
        }
        valued.push((option, value.as_str()));
        index += 2;
    }
    if optional(&valued, "--output") != Some("json") {
        return Err(WikiError::InvalidInput(
            "knowledge commands require --output json".to_owned(),
        ));
    }
    Ok((
        valued,
        mode.ok_or_else(|| {
            WikiError::InvalidInput("promotion requires --dry-run or --apply".to_owned())
        })?,
    ))
}

fn run_lint(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--target", "--user-root"])?;
    let target = PathBuf::from(required(&options, "--target")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let mut issues = lint(&target)?;
    let shared_digest = if let Some(shared) = &shared {
        issues
            .retain(|issue| issue.code != "stale-index" || issue.locator != SHARED_INDEX_RELATIVE);
        match validate_shared_index(&shared.user_root) {
            Ok(digest) => Some(digest),
            Err(error) => {
                issues.push(LintIssue {
                    code: "stale-index".to_owned(),
                    severity: LintSeverity::Error,
                    locator: SHARED_INDEX_RELATIVE.to_owned(),
                    message: error.to_string(),
                });
                None
            }
        }
    } else {
        None
    };
    issues.sort_by(|left, right| {
        (&left.code, &left.locator, &left.message).cmp(&(
            &right.code,
            &right.locator,
            &right.message,
        ))
    });
    let has_error = issues
        .iter()
        .any(|issue| issue.severity == LintSeverity::Error);
    let data = json!({
        "issues": issues,
        "error_count": issue_count(&issues, true),
        "warning_count": issue_count(&issues, false),
        "shared_index_digest": shared_digest.clone(),
        "shared_index_path": shared.as_ref().map(|_| SHARED_INDEX_RELATIVE)
    });
    let report_digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    let digest = shared_digest.as_deref().unwrap_or(&report_digest);
    let locator = if shared.is_some() {
        SHARED_INDEX_RELATIVE
    } else {
        ".hive/knowledge"
    };
    if has_error {
        Ok(KnowledgeResult {
            schema_version: 1,
            action: "LintKnowledge",
            status: "verification-failed",
            exit_code: 5,
            code: "hive.knowledge-lint-failed",
            message: "knowledge lint found canonical or derived-state errors".to_owned(),
            changed_paths: Vec::new(),
            evidence: vec![KnowledgeEvidence {
                kind: "report",
                locator: locator.to_owned(),
                digest: digest.to_owned(),
            }],
            next_action: Some("repair reported issues and run knowledge lint again".to_owned()),
            data: Some(data),
        })
    } else {
        Ok(success(
            "LintKnowledge",
            "hive.knowledge-lint-passed",
            "knowledge lint completed without errors",
            Vec::new(),
            locator,
            digest,
            data,
        ))
    }
}

fn issue_count(issues: &[LintIssue], errors: bool) -> usize {
    issues
        .iter()
        .filter(|issue| (issue.severity == LintSeverity::Error) == errors)
        .count()
}

fn run_delete(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--page-id",
            "--reason",
            "--replacement",
            "--timestamp",
            "--user-root",
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let outcome = delete_page(
        &target,
        required(&options, "--page-id")?,
        required(&options, "--reason")?,
        optional(&options, "--replacement"),
        required(&options, "--timestamp")?,
    )?;
    let mutation =
        serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = finish_shared_mutation(
        &target,
        shared.as_ref(),
        outcome.changed_paths,
        mutation,
        ".hive/knowledge/suppression.yml",
    )?;
    Ok(success(
        "DeleteKnowledge",
        "hive.knowledge-deleted",
        "active Wiki page deleted and minimal suppression metadata recorded",
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

fn run_suppress(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--fingerprint",
            "--source-locator",
            "--reason",
            "--replacement",
            "--timestamp",
            "--user-root",
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let entry = SuppressionEntry {
        fingerprint: required(&options, "--fingerprint")?.to_owned(),
        source_locator: required(&options, "--source-locator")?.to_owned(),
        reason: required(&options, "--reason")?.to_owned(),
        replacement: optional(&options, "--replacement").map(ToOwned::to_owned),
        timestamp: required(&options, "--timestamp")?.to_owned(),
    };
    let outcome = suppress(&target, entry)?;
    let mutation =
        serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = finish_shared_mutation(
        &target,
        shared.as_ref(),
        outcome.changed_paths,
        mutation,
        ".hive/knowledge/suppression.yml",
    )?;
    Ok(success(
        "SuppressKnowledge",
        "hive.knowledge-suppressed",
        "minimal source fingerprint suppression recorded",
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

#[derive(Debug, Eq, PartialEq)]
enum SharedTargetKind {
    UserRoot,
    RegisteredProject,
}

#[derive(Debug)]
struct SharedMutationTarget {
    user_root: PathBuf,
    target_kind: SharedTargetKind,
}

fn shared_mutation_target(
    target: &Path,
    user_root: &Path,
    allow_user_root: bool,
) -> Result<SharedMutationTarget, WikiError> {
    require_shared_wiki_enabled(user_root)?;
    let canonical_user = user_root
        .canonicalize()
        .map_err(|error| WikiError::Io(format!("cannot canonicalize user root: {error}")))?;
    let canonical_target = target
        .canonicalize()
        .map_err(|error| WikiError::Io(format!("cannot canonicalize knowledge target: {error}")))?;
    let registry = load_project_registry(&canonical_user)?;
    let target_kind = if canonical_target == canonical_user {
        if !allow_user_root {
            return Err(WikiError::InvalidInput(
                "knowledge target must be an enabled registered project".to_owned(),
            ));
        }
        SharedTargetKind::UserRoot
    } else if registry
        .projects
        .iter()
        .any(|project| project.enabled && project.root == canonical_target)
    {
        SharedTargetKind::RegisteredProject
    } else {
        return Err(WikiError::InvalidInput(
            "knowledge target is not enabled in the project registry".to_owned(),
        ));
    };
    Ok(SharedMutationTarget {
        user_root: canonical_user,
        target_kind,
    })
}

fn require_shared_wiki_enabled(user_root: &Path) -> Result<(), WikiError> {
    match super::user_setup::operational_wiki_enabled(user_root) {
        Ok(true) => Ok(()),
        Ok(false) => Err(WikiError::Conflict(
            "global Wiki is disabled; canonical Markdown is preserved and shared knowledge operations are unavailable"
                .to_owned(),
        )),
        Err(error) => Err(WikiError::Verification(format!(
            "cannot authorize shared knowledge operation: {error}"
        ))),
    }
}

fn authorize_legacy_target(target: &Path) -> Result<(), WikiError> {
    super::project_upgrade::authenticate_legacy_knowledge_target(target).map_err(|error| {
        WikiError::Verification(format!(
            "legacy project-local knowledge route is not authenticated: {error}"
        ))
    })
}

fn finish_shared_mutation(
    target: &Path,
    shared: Option<&SharedMutationTarget>,
    mut changed_paths: Vec<String>,
    mutation: Value,
    legacy_locator: &'static str,
) -> Result<(Vec<String>, &'static str, String, Value), WikiError> {
    let Some(shared) = shared else {
        let digest = mutation
            .get("logical_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| WikiError::Io("mutation digest is missing".to_owned()))?
            .to_owned();
        return Ok((changed_paths, legacy_locator, digest, mutation));
    };
    let removed_legacy_project_indexes =
        if shared.target_kind == SharedTargetKind::RegisteredProject {
            remove_legacy_project_indexes(target)?
        } else {
            Vec::new()
        };
    changed_paths.retain(|path| !LEGACY_DERIVED_RELATIVES.contains(&path.as_str()));
    let rebuilt = rebuild_shared_index(&shared.user_root)?;
    changed_paths.extend(
        rebuilt
            .changed_paths
            .iter()
            .map(|path| format!("user-root:{path}")),
    );
    let digest = rebuilt.logical_digest.clone();
    let mut data = mutation;
    let data_object = data
        .as_object_mut()
        .ok_or_else(|| WikiError::Io("mutation result is not an object".to_owned()))?;
    data_object.insert(
        "shared_index".to_owned(),
        serde_json::to_value(&rebuilt).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    data_object.insert(
        "removed_legacy_project_indexes".to_owned(),
        serde_json::to_value(removed_legacy_project_indexes)
            .map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok((changed_paths, SHARED_INDEX_RELATIVE, digest, data))
}

fn remove_legacy_project_indexes(target: &Path) -> Result<Vec<String>, WikiError> {
    let mut existing = Vec::new();
    for relative in LEGACY_DERIVED_RELATIVES {
        ensure_no_symlink_ancestors(target, Path::new(relative))
            .map_err(|error| WikiError::Conflict(error.to_string()))?;
        let path = target.join(relative);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(WikiError::Conflict(format!(
                    "legacy project index path is not a regular no-follow file: {relative}"
                )));
            }
            Ok(_) => existing.push((relative, path)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(WikiError::Io(format!(
                    "cannot inspect legacy project index {relative}: {error}"
                )));
            }
        }
    }
    let mut removed = Vec::new();
    for (relative, path) in existing {
        std::fs::remove_file(&path).map_err(|error| {
            WikiError::Io(format!(
                "cannot remove legacy project index {relative}: {error}"
            ))
        })?;
        removed.push(relative.to_owned());
    }
    Ok(removed)
}

fn parse_options<'a>(
    arguments: &'a [String],
    allowed: &[&str],
) -> Result<Vec<(&'a str, &'a str)>, WikiError> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--output" {
            let value = arguments
                .get(index + 1)
                .ok_or_else(|| WikiError::InvalidInput("missing value for --output".to_owned()))?;
            if value != "json" {
                return Err(WikiError::InvalidInput(
                    "knowledge commands require --output json".to_owned(),
                ));
            }
            output.push((option, value.as_str()));
            index += 2;
            continue;
        }
        if !allowed.contains(&option) {
            return Err(WikiError::InvalidInput(format!(
                "unknown knowledge option: {option}"
            )));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| WikiError::InvalidInput(format!("missing value for {option}")))?;
        if output.iter().any(|(existing, _)| *existing == option) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate knowledge option: {option}"
            )));
        }
        output.push((option, value.as_str()));
        index += 2;
    }
    if optional(&output, "--output") != Some("json") {
        return Err(WikiError::InvalidInput(
            "knowledge commands require --output json".to_owned(),
        ));
    }
    Ok(output)
}

fn required<'a>(options: &[(&'a str, &'a str)], name: &str) -> Result<&'a str, WikiError> {
    optional(options, name)
        .ok_or_else(|| WikiError::InvalidInput(format!("missing required option {name}")))
}

fn optional<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(option, value)| (*option == name).then_some(*value))
}

fn success(
    action: &'static str,
    code: &'static str,
    message: &str,
    mut changed_paths: Vec<String>,
    locator: &str,
    digest: &str,
    data: Value,
) -> KnowledgeResult {
    changed_paths.sort();
    changed_paths.dedup();
    KnowledgeResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![KnowledgeEvidence {
            kind: "report",
            locator: locator.to_owned(),
            digest: digest.to_owned(),
        }],
        next_action: None,
        data: Some(data),
    }
}

fn failure(action: &'static str, error: &WikiError) -> KnowledgeResult {
    KnowledgeResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.to_string(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn emit(result: &KnowledgeResult) {
    match serde_json::to_string(result) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            println!("{{\"schema_version\":1,\"action\":\"UnknownAction\",\"status\":\"error\",\"exit_code\":10,\"code\":\"hive.internal-error\",\"message\":\"JSON serialization failed\",\"changed_paths\":[],\"evidence\":[],\"next_action\":null}}");
            eprintln!("error: {error}");
        }
    }
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
}

fn is_help(arguments: &[String]) -> bool {
    matches!(arguments, [argument] if argument == "-h" || argument == "--help")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_wiki::shared::{
        register_project, KnowledgeLanguage, KnowledgeVisibility, RegisteredProject,
    };
    use std::fs;
    use tempfile::TempDir;

    fn temp_root() -> TempDir {
        TempDir::new_in(std::env::current_dir().expect("current directory"))
            .expect("temporary root")
    }

    fn write_empty_knowledge(root: &Path) {
        fs::create_dir_all(root.join(".hive/knowledge/Wiki")).expect("Wiki directory");
        fs::create_dir_all(root.join(".hive/knowledge/Raw")).expect("Raw directory");
        fs::write(
            root.join(".hive/knowledge/suppression.yml"),
            "schema_version: 1\nentries: []\n",
        )
        .expect("suppression ledger");
    }

    fn write_user_setup(root: &Path, wiki_enabled: bool) {
        fs::create_dir_all(root.join(".hive/config")).expect("user config");
        fs::write(
            root.join(".hive/config/user-setup.yml"),
            format!(
                "schema_version: 1\ninterface_language: en\nwiki:\n  enabled: {wiki_enabled}\n  language: both\nprofile:\n  id: web-developer\npersona:\n  id: balanced\nselected_hosts:\n  - codex\nskills:\n  mode: individual\n  selected:\n    - setup-hive\nusage_guard:\n  enabled: false\n  stop_remaining_percent: 20\n  codexbar_fallback_enabled: false\n"
            ),
        )
        .expect("user setup");
    }

    fn registered_roots(enabled: bool) -> (TempDir, TempDir) {
        let user = temp_root();
        let project = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        write_empty_knowledge(project.path());
        register_project(
            user.path(),
            RegisteredProject {
                id: "project-test".to_owned(),
                root: project.path().canonicalize().expect("canonical project"),
                enabled,
                language: KnowledgeLanguage::En,
                visibility: KnowledgeVisibility::ProjectPrivate,
            },
        )
        .expect("register project");
        (user, project)
    }

    #[test]
    fn disabled_global_wiki_blocks_shared_operations_and_preserves_markdown() {
        let (user, project) = registered_roots(true);
        write_user_setup(user.path(), false);
        let page = project.path().join(".hive/knowledge/Wiki/example.md");
        fs::write(&page, b"canonical markdown\n").expect("canonical page");

        let error = shared_mutation_target(project.path(), user.path(), true)
            .expect_err("disabled Wiki must block shared mutation");
        assert_eq!(error.code(), "hive.knowledge-conflict");
        assert!(error.to_string().contains("global Wiki is disabled"));
        let Err(query_error) =
            run_shared_query(project.path(), user.path(), Some("canonical"), None, 20)
        else {
            panic!("disabled Wiki must block shared query");
        };
        assert_eq!(query_error.code(), "hive.knowledge-conflict");
        let rebuild_arguments = vec![
            "rebuild".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert_ne!(run_index(&rebuild_arguments), ExitCode::SUCCESS);
        assert_eq!(
            fs::read(&page).expect("preserved canonical page"),
            b"canonical markdown\n"
        );
        assert!(!user.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn explicit_user_root_missing_registry_never_falls_back_to_project_index() {
        let user = temp_root();
        let project = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        write_empty_knowledge(project.path());
        let arguments = vec![
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--text".to_owned(),
            "missing".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let Err(error) = run_query(&arguments) else {
            panic!("missing registry must fail closed");
        };

        assert_eq!(error.code(), "hive.knowledge-io-error");
        assert!(error.to_string().contains("cannot open project registry"));
        assert!(!user.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn unauthenticated_legacy_target_never_creates_a_project_index() {
        let project = temp_root();
        write_empty_knowledge(project.path());
        let arguments = vec![
            "rebuild".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        assert_ne!(run_index(&arguments), ExitCode::SUCCESS);
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn shared_mutations_require_the_enabled_registered_target() {
        let (user, enabled) = registered_roots(true);
        let disabled = temp_root();
        let unregistered = temp_root();
        register_project(
            user.path(),
            RegisteredProject {
                id: "project-disabled".to_owned(),
                root: disabled.path().canonicalize().expect("canonical disabled"),
                enabled: false,
                language: KnowledgeLanguage::Ko,
                visibility: KnowledgeVisibility::ProjectPrivate,
            },
        )
        .expect("register disabled project");

        assert_eq!(
            shared_mutation_target(enabled.path(), user.path(), false)
                .expect("enabled project")
                .target_kind,
            SharedTargetKind::RegisteredProject
        );
        assert_eq!(
            shared_mutation_target(user.path(), user.path(), true)
                .expect("exact user root")
                .target_kind,
            SharedTargetKind::UserRoot
        );
        assert!(shared_mutation_target(user.path(), user.path(), false).is_err());
        assert!(shared_mutation_target(disabled.path(), user.path(), true).is_err());
        assert!(shared_mutation_target(unregistered.path(), user.path(), true).is_err());
    }

    #[test]
    fn shared_mutation_removes_only_fixed_project_indexes_and_rebuilds_root() {
        let (user, project) = registered_roots(true);
        fs::create_dir_all(project.path().join(".hive/index")).expect("project index directory");
        for relative in LEGACY_DERIVED_RELATIVES {
            fs::write(project.path().join(relative), b"legacy").expect("legacy index artifact");
        }
        let shared =
            shared_mutation_target(project.path(), user.path(), true).expect("registered project");
        let (changed_paths, locator, digest, data) = finish_shared_mutation(
            project.path(),
            Some(&shared),
            vec![
                ".hive/knowledge/Wiki/example.md".to_owned(),
                SHARED_INDEX_RELATIVE.to_owned(),
            ],
            json!({"logical_digest": "sha256:legacy"}),
            ".hive/knowledge",
        )
        .expect("finish shared mutation");

        assert_eq!(locator, SHARED_INDEX_RELATIVE);
        assert!(digest.starts_with("sha256:"));
        assert!(user.path().join(SHARED_INDEX_RELATIVE).is_file());
        for relative in LEGACY_DERIVED_RELATIVES {
            assert!(!project.path().join(relative).exists());
        }
        assert_eq!(
            changed_paths,
            vec![
                ".hive/knowledge/Wiki/example.md",
                "user-root:.hive/index/hive.sqlite3"
            ]
        );
        assert_eq!(
            data["shared_index"]["logical_digest"],
            Value::String(digest)
        );
        assert_eq!(data["logical_digest"], "sha256:legacy");
        assert!(data.get("mutation").is_none());
    }

    #[test]
    fn shared_lint_uses_shared_freshness_without_requiring_a_project_index() {
        let (user, project) = registered_roots(true);
        let rebuilt = rebuild_shared_index(user.path()).expect("shared rebuild");
        let arguments = vec![
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let result = run_lint(&arguments).expect("shared lint");

        assert_eq!(result.status, "success");
        assert_eq!(result.evidence[0].locator, SHARED_INDEX_RELATIVE);
        assert_eq!(result.evidence[0].digest, rebuilt.logical_digest);
        assert!(!result.data.expect("lint data")["issues"]
            .as_array()
            .expect("issues")
            .iter()
            .any(|issue| issue["code"] == "stale-index"));
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn project_index_cleanup_rejects_symlinks_before_removing_regular_files() {
        use std::os::unix::fs::symlink;

        let project = temp_root();
        let external = temp_root();
        fs::create_dir_all(project.path().join(".hive/index")).expect("index directory");
        fs::write(project.path().join(SHARED_INDEX_RELATIVE), b"legacy")
            .expect("regular legacy index");
        let sentinel = external.path().join("sentinel");
        fs::write(&sentinel, b"outside").expect("external sentinel");
        symlink(
            &sentinel,
            project.path().join(".hive/index/hive.sqlite3-wal"),
        )
        .expect("unsafe sidecar");

        assert!(remove_legacy_project_indexes(project.path()).is_err());
        assert!(project.path().join(SHARED_INDEX_RELATIVE).is_file());
        assert_eq!(fs::read(sentinel).expect("external bytes"), b"outside");
    }
}
