//! Explicit, local-only developer problem reports.
//!
//! Reports contain bounded environment and product context. They never collect a
//! prompt or transcript, never upload automatically, and keep any manually
//! supplied summary subject to credential-oriented redaction.

use super::{emit_action_result, ActionResult, Evidence};
use crate::run::{open_directory_nofollow_path, AdapterError, PinnedTarget};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::OpenOptions;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const REPORT_USAGE: &str = "\
Create a local, sanitized report for an Aigent Hive developer.

USAGE:
    hive report preview --target <dir> --output json
    hive report collect --target <dir> --issue-code <safe-id> [--summary <manual-summary>] --output json
    hive report export --target <dir> --report <report-id> --destination <absolute-new-file> --output json

Reports exclude raw prompts and transcripts, redact likely credentials in an
optional manual summary, and never upload automatically.
";

const REPORT_DIRECTORY: &str = ".hive/runtime/reports";
const MAX_REPORT_BYTES: usize = 16 * 1024;
const MAX_SUMMARY_BYTES: usize = 512;

#[derive(Debug)]
struct PreviewArguments {
    target: PathBuf,
}

#[derive(Debug)]
struct CollectArguments {
    target: PathBuf,
    issue_code: String,
    summary: Option<String>,
}

#[derive(Debug)]
struct ExportArguments {
    target: PathBuf,
    report_id: String,
    destination: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct DeveloperReport {
    schema_version: u32,
    report_id: String,
    created_at: u64,
    product_version: String,
    target_digest: String,
    issue_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_summary: Option<String>,
    environment: ReportEnvironment,
    raw_prompt_included: bool,
    transcript_included: bool,
    automatic_upload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ReportEnvironment {
    operating_system: String,
    architecture: String,
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.is_empty() || is_help(arguments) {
        print!("{REPORT_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = match arguments.first().map(String::as_str) {
        Some("preview") => parse_preview(&arguments[1..]).and_then(preview),
        Some("collect") => parse_collect(&arguments[1..]).and_then(collect),
        Some("export") => parse_export(&arguments[1..]).and_then(export),
        Some(action) => Err(AdapterError::Input(format!(
            "unknown report action: {action}"
        ))),
        None => unreachable!("empty arguments returned above"),
    };
    emit_action_result(&result.unwrap_or_else(|error| failure(&error)))
}

fn is_help(arguments: &[String]) -> bool {
    arguments == ["--help"] || arguments.iter().any(|argument| argument == "--help")
}

fn parse_preview(arguments: &[String]) -> Result<PreviewArguments, AdapterError> {
    let options = parse_options(arguments, &["--target"])?;
    Ok(PreviewArguments {
        target: PathBuf::from(required(&options, "--target")?),
    })
}

fn parse_collect(arguments: &[String]) -> Result<CollectArguments, AdapterError> {
    let options = parse_options(arguments, &["--target", "--issue-code", "--summary"])?;
    Ok(CollectArguments {
        target: PathBuf::from(required(&options, "--target")?),
        issue_code: validate_issue_code(required(&options, "--issue-code")?)?,
        summary: options
            .iter()
            .find_map(|(key, value)| (*key == "--summary").then_some((*value).to_owned()))
            .map(|value| sanitize_manual_summary(&value))
            .transpose()?,
    })
}

fn parse_export(arguments: &[String]) -> Result<ExportArguments, AdapterError> {
    let options = parse_options(arguments, &["--target", "--report", "--destination"])?;
    let destination = PathBuf::from(required(&options, "--destination")?);
    if !destination.is_absolute() {
        return Err(AdapterError::Input(
            "report export destination must be an absolute new file path".to_owned(),
        ));
    }
    Ok(ExportArguments {
        target: PathBuf::from(required(&options, "--target")?),
        report_id: validate_report_id(required(&options, "--report")?)?,
        destination,
    })
}

fn parse_options<'a>(
    arguments: &'a [String],
    allowed: &[&str],
) -> Result<Vec<(&'a str, &'a str)>, AdapterError> {
    let mut options = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let key = arguments[index].as_str();
        if key == "--output" {
            let value = arguments.get(index + 1).ok_or_else(|| {
                AdapterError::Input("missing value for report option --output".to_owned())
            })?;
            if value != "json" {
                return Err(AdapterError::Input(
                    "report requires --output json".to_owned(),
                ));
            }
            index += 2;
            continue;
        }
        if !allowed.contains(&key) {
            return Err(AdapterError::Input(format!("unknown report option: {key}")));
        }
        let value = arguments
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| AdapterError::Input(format!("missing value for report option {key}")))?;
        if options.iter().any(|(existing, _)| *existing == key) {
            return Err(AdapterError::Input(format!(
                "duplicate report option: {key}"
            )));
        }
        options.push((key, value.as_str()));
        index += 2;
    }
    if !arguments
        .windows(2)
        .any(|pair| pair == ["--output", "json"])
    {
        return Err(AdapterError::Input(
            "report requires --output json".to_owned(),
        ));
    }
    Ok(options)
}

fn required<'a>(options: &[(&'a str, &'a str)], key: &str) -> Result<&'a str, AdapterError> {
    options
        .iter()
        .find_map(|(option, value)| (*option == key).then_some(*value))
        .ok_or_else(|| AdapterError::Input(format!("missing required report option {key}")))
}

fn preview(arguments: PreviewArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    Ok(ActionResult {
        schema_version: 1,
        action: "PreviewDeveloperReport",
        status: "success",
        exit_code: 0,
        code: "hive.report-preview",
        message: "developer report preview contains no raw prompt, transcript, credential, or upload action"
            .to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "target",
            locator: "consumer-target".to_owned(),
            digest: sha256_digest(target.requested_path().to_string_lossy().as_bytes()),
        }],
        next_action: None,
        data: Some(json!({
            "included": [
                "product_version", "target_digest", "issue_code", "manual_summary",
                "operating_system", "architecture"
            ],
            "excluded": ["raw_prompt", "transcript", "credential", "automatic_upload"],
            "outbound": false,
        })),
    })
}

fn collect(arguments: CollectArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let report = DeveloperReport {
        schema_version: 1,
        report_id: generate_report_id()?,
        created_at: current_unix_time()?,
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        target_digest: sha256_digest(target.requested_path().to_string_lossy().as_bytes()),
        issue_code: arguments.issue_code,
        manual_summary: arguments.summary,
        environment: ReportEnvironment {
            operating_system: std::env::consts::OS.to_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
        },
        raw_prompt_included: false,
        transcript_included: false,
        automatic_upload: false,
    };
    let relative = report_relative(&report.report_id)?;
    let desired = canonical_report_bytes(&report)?;
    let snapshot = target.snapshot_bounded(&relative, MAX_REPORT_BYTES)?;
    if snapshot.bytes().is_some() {
        return Err(AdapterError::Conflict(
            "generated developer report identifier already exists; retry collection".to_owned(),
        ));
    }
    let changed = target.publish_runtime(&relative, &snapshot, &desired)?;
    Ok(ActionResult {
        schema_version: 1,
        action: "CollectDeveloperReport",
        status: "success",
        exit_code: 0,
        code: "hive.report-collected",
        message: "sanitized developer report was collected locally without upload".to_owned(),
        changed_paths: changed
            .then(|| relative.to_string_lossy().replace('\\', "/"))
            .into_iter()
            .collect(),
        evidence: vec![Evidence {
            kind: "file",
            locator: relative.to_string_lossy().replace('\\', "/"),
            digest: sha256_digest(&desired),
        }],
        next_action: Some("inspect with `hive report export` before sharing".to_owned()),
        data: Some(json!({
            "report_id": report.report_id,
            "raw_prompt_included": false,
            "automatic_upload": false,
        })),
    })
}

fn export(arguments: ExportArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let relative = report_relative(&arguments.report_id)?;
    let bytes = target.read_required(&relative, MAX_REPORT_BYTES)?;
    let report: DeveloperReport = serde_json::from_slice(&bytes).map_err(|_| {
        AdapterError::Verification("stored developer report is malformed".to_owned())
    })?;
    if report.report_id != arguments.report_id
        || report.raw_prompt_included
        || report.transcript_included
        || report.automatic_upload
    {
        return Err(AdapterError::Verification(
            "stored developer report violates its privacy contract".to_owned(),
        ));
    }
    export_new_file(&arguments.destination, &bytes)?;
    Ok(ActionResult {
        schema_version: 1,
        action: "ExportDeveloperReport",
        status: "success",
        exit_code: 0,
        code: "hive.report-exported",
        message: "sanitized developer report was exported to a new local file".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "file",
            locator: relative.to_string_lossy().replace('\\', "/"),
            digest: sha256_digest(&bytes),
        }],
        next_action: Some(
            "share the exported file only through a user-selected channel".to_owned(),
        ),
        data: Some(json!({
            "report_id": report.report_id,
            "automatic_upload": false,
            "destination_digest": sha256_digest(arguments.destination.to_string_lossy().as_bytes()),
        })),
    })
}

fn failure(error: &AdapterError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "DeveloperReport",
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn validate_issue_code(value: &str) -> Result<String, AdapterError> {
    if value.is_empty()
        || value.len() > 80
        || !value.chars().all(
            |character| matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.'),
        )
    {
        return Err(AdapterError::Input(
            "report issue-code must contain only letters, digits, '-', '_', or '.'".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

fn sanitize_manual_summary(value: &str) -> Result<String, AdapterError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() || normalized.len() > MAX_SUMMARY_BYTES {
        return Err(AdapterError::Input(format!(
            "manual report summary must be between 1 and {MAX_SUMMARY_BYTES} bytes after normalization"
        )));
    }
    let lower = normalized.to_ascii_lowercase();
    let secret_like = [
        "discord.com/api/webhooks/",
        "discordapp.com/api/webhooks/",
        "authorization:",
        "bearer ",
        "api_key",
        "api-key",
        "sk-",
        "xoxb-",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    if secret_like {
        Ok("[redacted: possible credential]".to_owned())
    } else {
        Ok(normalized)
    }
}

fn generate_report_id() -> Result<String, AdapterError> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| {
        AdapterError::Internal(format!("cannot obtain report entropy: {error}"))
    })?;
    let mut value = String::from("rpt-");
    for byte in bytes {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(value)
}

fn validate_report_id(value: &str) -> Result<String, AdapterError> {
    let valid = value.len() == 36
        && value.starts_with("rpt-")
        && value[4..]
            .chars()
            .all(|character| matches!(character, '0'..='9' | 'a'..='f'));
    if !valid {
        return Err(AdapterError::Input(
            "report identifier must be a generated rpt- followed by 32 lowercase hex characters"
                .to_owned(),
        ));
    }
    Ok(value.to_owned())
}

fn report_relative(report_id: &str) -> Result<PathBuf, AdapterError> {
    let report_id = validate_report_id(report_id)?;
    Ok(Path::new(REPORT_DIRECTORY).join(format!("{report_id}.json")))
}

fn canonical_report_bytes(report: &DeveloperReport) -> Result<Vec<u8>, AdapterError> {
    let mut bytes = serde_json_canonicalizer::to_vec(report).map_err(|error| {
        AdapterError::Internal(format!("cannot encode developer report: {error}"))
    })?;
    bytes.push(b'\n');
    if bytes.len() > MAX_REPORT_BYTES {
        return Err(AdapterError::Input(
            "developer report exceeds the bounded runtime size".to_owned(),
        ));
    }
    Ok(bytes)
}

fn current_unix_time() -> Result<u64, AdapterError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| AdapterError::Internal("system clock is before Unix epoch".to_owned()))
}

fn export_new_file(destination: &Path, bytes: &[u8]) -> Result<(), AdapterError> {
    let parent = destination.parent().ok_or_else(|| {
        AdapterError::Input("report export destination must have a parent directory".to_owned())
    })?;
    let name = destination.file_name().ok_or_else(|| {
        AdapterError::Input("report export destination must name a file".to_owned())
    })?;
    let directory = open_directory_nofollow_path(parent)?;
    match directory.symlink_metadata(name) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => {
            return Err(AdapterError::Conflict(
                "report export refuses to overwrite an existing destination".to_owned(),
            ));
        }
        Err(error) => {
            return Err(AdapterError::Safety(format!(
                "cannot inspect report export destination: {error}"
            )));
        }
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = directory.open_with(name, &options).map_err(|error| {
        AdapterError::Safety(format!("cannot create report export destination: {error}"))
    })?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        return Err(AdapterError::Internal(format!(
            "cannot write report export destination: {error}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        collect, export, sanitize_manual_summary, validate_report_id, CollectArguments,
        ExportArguments,
    };
    use std::fs;

    #[test]
    fn manual_summary_redacts_webhook_secrets() {
        let summary = sanitize_manual_summary(
            "webhook https://discord.com/api/webhooks/123456789/a-private-token failed",
        )
        .expect("summary");

        assert_eq!(summary, "[redacted: possible credential]");
    }

    #[test]
    fn collect_and_export_keep_raw_prompt_out_of_the_report() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let target = temporary.path().join("consumer");
        let destination = temporary.path().join("export.json");
        fs::create_dir_all(&target).expect("consumer directory");

        let collected = collect(CollectArguments {
            target: target.clone(),
            issue_code: "usage-halt".to_owned(),
            summary: None,
        })
        .expect("collect report");
        let report_id = collected
            .data
            .as_ref()
            .and_then(|data| data.get("report_id"))
            .and_then(serde_json::Value::as_str)
            .expect("report identifier")
            .to_owned();
        let bytes = fs::read(
            target
                .join(".hive/runtime/reports")
                .join(format!("{report_id}.json")),
        )
        .expect("stored report");
        let text = String::from_utf8(bytes).expect("report UTF-8");
        let report: serde_json::Value = serde_json::from_str(&text).expect("report JSON");
        assert!(report.get("raw_prompt").is_none());
        assert!(report.get("transcript").is_none());
        assert!(text.contains("\"raw_prompt_included\":false"));
        assert!(text.contains("\"automatic_upload\":false"));

        export(ExportArguments {
            target,
            report_id,
            destination: destination.clone(),
        })
        .expect("export report");
        assert!(destination.is_file());
    }

    #[test]
    fn report_identifiers_reject_uppercase_hex() {
        assert!(validate_report_id("rpt-0123456789abcdef0123456789abcdef").is_ok());
        assert!(validate_report_id("rpt-0123456789ABCDEF0123456789abcdef").is_err());
    }
}
