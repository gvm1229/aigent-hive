use hive_core::sha256_digest;
use hive_wiki::{
    delete_page, ingest, lint, query, rebuild_index, suppress, LintIssue, LintSeverity,
    SuppressionEntry, WikiError,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::ExitCode;

const KNOWLEDGE_USAGE: &str = "\
Canonical Markdown knowledge and disposable SQLite index.

USAGE:
    hive knowledge ingest --target <dir> --source <file> --wiki <draft.md> --output json
    hive knowledge query --target <dir> (--text <query>|--tag <tag>) [--limit <1..100>] --output json
    hive knowledge lint --target <dir> --output json
    hive knowledge delete --target <dir> --page-id <id> --reason <text> [--replacement <locator>] --timestamp <RFC3339> --output json
    hive knowledge suppress --target <dir> --fingerprint <sha256:...> --source-locator <locator> --reason <text> [--replacement <locator>] --timestamp <RFC3339> --output json
    hive index rebuild --target <dir> --output json
";

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
        parse_options(&arguments[1..], &["--target"]).and_then(|options| {
            let target = required(&options, "--target")?;
            let outcome = rebuild_index(&PathBuf::from(target))?;
            let changed_paths = outcome.changed_paths.clone();
            let data =
                serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
            Ok(success(
                "RebuildKnowledgeIndex",
                "hive.index-rebuilt",
                "knowledge index rebuilt from canonical sources",
                changed_paths,
                ".hive/index/hive.sqlite3",
                &outcome.logical_digest,
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
    let options = parse_options(arguments, &["--target", "--source", "--wiki"])?;
    let target = PathBuf::from(required(&options, "--target")?);
    let source = PathBuf::from(required(&options, "--source")?);
    let wiki = PathBuf::from(required(&options, "--wiki")?);
    let outcome = ingest(&target, &source, &wiki)?;
    let data = serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    Ok(success(
        "IngestKnowledge",
        "hive.knowledge-ingested",
        "Raw revision and Wiki page integrated serially",
        outcome.changed_paths,
        ".hive/knowledge",
        &outcome.logical_digest,
        data,
    ))
}

fn run_query(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--target", "--text", "--tag", "--limit"])?;
    let target = PathBuf::from(required(&options, "--target")?);
    let text = optional(&options, "--text");
    let tag = optional(&options, "--tag");
    let limit = optional(&options, "--limit").map_or(Ok(20_usize), |value| {
        value
            .parse::<usize>()
            .map_err(|_| WikiError::InvalidInput("query limit must be an integer".to_owned()))
    })?;
    let hits = query(&target, text, tag, limit)?;
    let data = json!({"hits": hits});
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

fn run_lint(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--target"])?;
    let target = PathBuf::from(required(&options, "--target")?);
    let issues = lint(&target)?;
    let has_error = issues
        .iter()
        .any(|issue| issue.severity == LintSeverity::Error);
    let data = json!({"issues": issues, "error_count": issue_count(&issues, true), "warning_count": issue_count(&issues, false)});
    let digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
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
                locator: ".hive/knowledge".to_owned(),
                digest,
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
            ".hive/knowledge",
            &digest,
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
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let outcome = delete_page(
        &target,
        required(&options, "--page-id")?,
        required(&options, "--reason")?,
        optional(&options, "--replacement"),
        required(&options, "--timestamp")?,
    )?;
    let data = serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    Ok(success(
        "DeleteKnowledge",
        "hive.knowledge-deleted",
        "active Wiki page deleted and minimal suppression metadata recorded",
        outcome.changed_paths,
        ".hive/knowledge/suppression.yml",
        &outcome.logical_digest,
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
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let entry = SuppressionEntry {
        fingerprint: required(&options, "--fingerprint")?.to_owned(),
        source_locator: required(&options, "--source-locator")?.to_owned(),
        reason: required(&options, "--reason")?.to_owned(),
        replacement: optional(&options, "--replacement").map(ToOwned::to_owned),
        timestamp: required(&options, "--timestamp")?.to_owned(),
    };
    let outcome = suppress(&target, entry)?;
    let data = serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    Ok(success(
        "SuppressKnowledge",
        "hive.knowledge-suppressed",
        "minimal source fingerprint suppression recorded",
        outcome.changed_paths,
        ".hive/knowledge/suppression.yml",
        &outcome.logical_digest,
        data,
    ))
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
