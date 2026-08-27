use super::{emit_action_result, ActionResult, Evidence};
use hive_core::sha256_digest;
use hive_wiki::{source, LintSeverity, WikiError};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SOURCE_WIKI_USAGE: &str = "\
Provider-neutral bilingual source Wiki and disposable SQLite index.

USAGE:
    hive source-wiki lint --target <source-root> --output json
    hive source-wiki index --target <source-root> --output json
    hive source-wiki query --target <source-root> --language en|ko (--text <query>|--tag <tag>) [--limit <1..100>] --output json
    hive source-wiki graph preview|enable|status|rebuild|disable|query|export --target <source-root> [--engine native-markdown|graphify-code] [--consent-digest <sha256:...>] [--input <graph.json> --receipt <receipt.json>] [--node-id <id>] [--text <query>] [--format json|html] --output json
    hive source-wiki vector preview|enable|status|disable --help
";

const INDEX_RELATIVE: &str = ".agents/work/source-wiki/index.sqlite3";

#[derive(Debug, Eq, PartialEq)]
struct CommonArguments {
    target: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
struct QueryArguments {
    target: PathBuf,
    language: String,
    text: Option<String>,
    tag: Option<String>,
    limit: usize,
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if is_help(arguments) {
        print!("{SOURCE_WIKI_USAGE}");
        return ExitCode::SUCCESS;
    }
    if arguments.first().map(String::as_str) == Some("graph") {
        return super::knowledge::run_source_graph(&arguments[1..]);
    }
    if arguments.first().map(String::as_str) == Some("vector") {
        return super::knowledge::run_source_vector(&arguments[1..]);
    }
    let result = match arguments.first().map(String::as_str) {
        Some("lint") => parse_common(&arguments[1..])
            .and_then(|parsed| lint(&parsed))
            .unwrap_or_else(|error| failure("LintSourceWiki", &error, None)),
        Some("index") => parse_common(&arguments[1..])
            .and_then(|parsed| rebuild_index(&parsed))
            .unwrap_or_else(|error| failure("RebuildSourceWikiIndex", &error, None)),
        Some("query") => match parse_query(&arguments[1..]) {
            Ok(parsed) => {
                let next_action = index_next_action(&parsed.target);
                query(&parsed).unwrap_or_else(|error| {
                    let next_action =
                        matches!(error, WikiError::Verification(_) | WikiError::Sqlite(_))
                            .then_some(next_action);
                    failure("QuerySourceWiki", &error, next_action)
                })
            }
            Err(error) => failure("QuerySourceWiki", &error, None),
        },
        Some(action) => failure(
            "LintSourceWiki",
            &WikiError::InvalidInput(format!("unknown source-wiki action: {action}")),
            None,
        ),
        None => failure(
            "LintSourceWiki",
            &WikiError::InvalidInput("source-wiki requires an action".to_owned()),
            None,
        ),
    };
    emit_action_result(&result)
}

fn lint(arguments: &CommonArguments) -> Result<ActionResult, WikiError> {
    let outcome = source::lint(&arguments.target)?;
    let error_count = outcome
        .issues
        .iter()
        .filter(|issue| issue.severity == LintSeverity::Error)
        .count();
    let warning_count = outcome.issues.len().saturating_sub(error_count);
    let mut data =
        serde_json::to_value(outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let object = data
        .as_object_mut()
        .ok_or_else(|| WikiError::Io("source lint outcome is not an object".to_owned()))?;
    object.insert("error_count".to_owned(), json!(error_count));
    object.insert("warning_count".to_owned(), json!(warning_count));
    let digest = value_digest(&data)?;
    if error_count > 0 {
        Ok(ActionResult {
            schema_version: 1,
            action: "LintSourceWiki",
            status: "verification-failed",
            exit_code: 5,
            code: "hive.source-wiki-lint-failed",
            message: "source Wiki lint found canonical or derived-state errors".to_owned(),
            changed_paths: Vec::new(),
            evidence: vec![report_evidence("docs/facts", digest)],
            next_action: Some("repair reported issues and run source-wiki lint again".to_owned()),
            data: Some(data),
        })
    } else {
        Ok(success(
            "LintSourceWiki",
            "hive.source-wiki-lint-passed",
            "source Wiki lint completed without errors",
            Vec::new(),
            "docs/facts",
            digest,
            data,
        ))
    }
}

fn rebuild_index(arguments: &CommonArguments) -> Result<ActionResult, WikiError> {
    let outcome = source::rebuild_index(&arguments.target)?;
    index_success(outcome)
}

fn index_success(outcome: source::SourceIndexOutcome) -> Result<ActionResult, WikiError> {
    let changed_paths = outcome.changed_paths.clone();
    let data = serde_json::to_value(outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let digest = value_digest(&data)?;
    Ok(success(
        "RebuildSourceWikiIndex",
        "hive.source-wiki-index-rebuilt",
        "source Wiki index rebuilt from canonical bilingual pages",
        changed_paths,
        INDEX_RELATIVE,
        digest,
        data,
    ))
}

fn query(arguments: &QueryArguments) -> Result<ActionResult, WikiError> {
    let hits = source::query(
        &arguments.target,
        &arguments.language,
        arguments.text.as_deref(),
        arguments.tag.as_deref(),
        arguments.limit,
    )?;
    let data = json!({
        "count": hits.len(),
        "hits": hits,
        "language": arguments.language,
    });
    let digest = value_digest(&data)?;
    Ok(success(
        "QuerySourceWiki",
        "hive.source-wiki-query-complete",
        "source Wiki query completed",
        Vec::new(),
        INDEX_RELATIVE,
        digest,
        data,
    ))
}

fn parse_common(arguments: &[String]) -> Result<CommonArguments, WikiError> {
    let options = parse_options(arguments, &["--target"])?;
    Ok(CommonArguments {
        target: PathBuf::from(required(&options, "--target")?),
    })
}

fn parse_query(arguments: &[String]) -> Result<QueryArguments, WikiError> {
    let options = parse_options(
        arguments,
        &["--target", "--language", "--text", "--tag", "--limit"],
    )?;
    let language = required(&options, "--language")?;
    if !matches!(language, "en" | "ko") {
        return Err(WikiError::InvalidInput(
            "source-wiki query language must be en or ko".to_owned(),
        ));
    }
    let text = optional(&options, "--text");
    let tag = optional(&options, "--tag");
    if text.is_none() && tag.is_none() {
        return Err(WikiError::InvalidInput(
            "source-wiki query requires --text or --tag".to_owned(),
        ));
    }
    let limit = optional(&options, "--limit").map_or(Ok(20_usize), |value| {
        value.parse::<usize>().map_err(|_| {
            WikiError::InvalidInput("source-wiki query limit must be an integer".to_owned())
        })
    })?;
    if !(1..=100).contains(&limit) {
        return Err(WikiError::InvalidInput(
            "source-wiki query limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(QueryArguments {
        target: PathBuf::from(required(&options, "--target")?),
        language: language.to_owned(),
        text: text.map(ToOwned::to_owned),
        tag: tag.map(ToOwned::to_owned),
        limit,
    })
}

type Options<'a> = Vec<(&'a str, &'a str)>;

fn parse_options<'a>(arguments: &'a [String], allowed: &[&str]) -> Result<Options<'a>, WikiError> {
    let mut options = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option != "--output" && !allowed.contains(&option) {
            return Err(WikiError::InvalidInput(format!(
                "unknown source-wiki option: {option}"
            )));
        }
        if options.iter().any(|(existing, _)| *existing == option) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate source-wiki option: {option}"
            )));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| WikiError::InvalidInput(format!("missing value for {option}")))?;
        options.push((option, value.as_str()));
        index += 2;
    }
    if optional(&options, "--output") != Some("json") {
        return Err(WikiError::InvalidInput(
            "source-wiki commands require --output json".to_owned(),
        ));
    }
    Ok(options)
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
    changed_paths: Vec<String>,
    locator: &str,
    digest: String,
    data: Value,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![report_evidence(locator, digest)],
        next_action: None,
        data: Some(data),
    }
}

fn failure(action: &'static str, error: &WikiError, next_action: Option<String>) -> ActionResult {
    let code = match error {
        WikiError::InvalidInput(_) => "hive.source-wiki-invalid-input",
        WikiError::Conflict(_) => "hive.source-wiki-conflict",
        WikiError::Verification(_) => "hive.source-wiki-verification-failed",
        WikiError::Io(_) => "hive.source-wiki-io-error",
        WikiError::Sqlite(_) => "hive.source-wiki-index-error",
    };
    ActionResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code,
        message: error.to_string(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action,
        data: None,
    }
}

fn report_evidence(locator: &str, digest: String) -> Evidence {
    Evidence {
        kind: "report",
        locator: locator.to_owned(),
        digest,
    }
}

fn value_digest(value: &Value) -> Result<String, WikiError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| WikiError::Io(error.to_string()))
}

fn index_next_action(target: &Path) -> String {
    format!(
        "hive source-wiki index --target {} --output json",
        target.display()
    )
}

fn is_help(arguments: &[String]) -> bool {
    matches!(arguments, [argument] if argument == "-h" || argument == "--help")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn query_parser_requires_language_and_search_term() {
        let missing_language = parse_query(&strings(&[
            "--target", ".", "--text", "routing", "--output", "json",
        ]))
        .unwrap_err();
        assert!(missing_language.to_string().contains("--language"));

        let missing_term = parse_query(&strings(&[
            "--target",
            ".",
            "--language",
            "en",
            "--output",
            "json",
        ]))
        .unwrap_err();
        assert!(missing_term.to_string().contains("--text or --tag"));
    }

    #[test]
    fn query_parser_accepts_both_terms_and_default_limit() {
        let parsed = parse_query(&strings(&[
            "--target",
            ".",
            "--language",
            "ko",
            "--text",
            "구조",
            "--tag",
            "architecture",
            "--output",
            "json",
        ]))
        .unwrap();
        assert_eq!(parsed.limit, 20);
        assert_eq!(parsed.text.as_deref(), Some("구조"));
        assert_eq!(parsed.tag.as_deref(), Some("architecture"));
    }

    #[test]
    fn parser_rejects_duplicate_unknown_and_invalid_limit() {
        let duplicate = parse_common(&strings(&[
            "--target", ".", "--target", "other", "--output", "json",
        ]))
        .unwrap_err();
        assert!(duplicate.to_string().contains("duplicate"));

        let unknown = parse_common(&strings(&[
            "--target", ".", "--other", "x", "--output", "json",
        ]))
        .unwrap_err();
        assert!(unknown.to_string().contains("unknown"));

        let invalid_limit = parse_query(&strings(&[
            "--target",
            ".",
            "--language",
            "en",
            "--text",
            "x",
            "--limit",
            "101",
            "--output",
            "json",
        ]))
        .unwrap_err();
        assert!(invalid_limit.to_string().contains("between 1 and 100"));
    }

    #[test]
    fn query_rebuild_action_is_exact() {
        assert_eq!(
            index_next_action(Path::new("/tmp/source")),
            "hive source-wiki index --target /tmp/source --output json"
        );
    }

    #[test]
    fn index_success_forwards_core_changed_paths() {
        let changed_paths = vec![
            ".agents/work/source-wiki/.index.lock".to_owned(),
            INDEX_RELATIVE.to_owned(),
        ];
        let result = index_success(source::SourceIndexOutcome {
            changed_paths: changed_paths.clone(),
            page_count: 20,
            logical_digest: "sha256:index".to_owned(),
        })
        .expect("index result");

        assert_eq!(result.changed_paths, changed_paths);
        assert_eq!(
            result.data.expect("data")["changed_paths"],
            json!([
                ".agents/work/source-wiki/.index.lock",
                ".agents/work/source-wiki/index.sqlite3"
            ])
        );
    }
}
