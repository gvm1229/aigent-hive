use super::{emit_action_result, ActionResult, Evidence};
use crate::judge::{read_protected_external_file, read_protected_file};
use crate::run::{AdapterError, PinnedTarget};
use hive_core::{sha256_digest, validate_project_relative};
use hive_update::{
    execute_update, recover_update, verify_release_repository_for_publication, MajorApproval,
    SemVersion, UpdateError, UpdateMode, UpdateRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_RELEASE_ROOT_BYTES: usize = 1024 * 1024;
const MAX_CONFIRMATION_BYTES: usize = 256 * 1024;

const UPDATE_USAGE: &str = "\
Verify and activate an offline signed Aigent Hive release.

USAGE:
    hive update --target <dir> --bundle <release-dir> \
        --trust-root <external-protected-root.json> (--dry-run|--apply) --output json
    hive update --target <dir> --recover --output json

BREAKING RELEASES:
    --exact-major-target <X.Y.Z> --major-confirmation <confirmation.json>
";
const RELEASE_USAGE: &str = "\
Verify a complete offline signed Aigent Hive release without mutation.

USAGE:
    hive release verify --bundle <release-dir> \
        --trust-root <external-protected-root.json> --output json
";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MajorConfirmation {
    schema_version: u32,
    source_version: String,
    exact_target: String,
    release_plan_digest: String,
    compatibility_report_digest: String,
    migration_table_digest: String,
    confirmed: bool,
}

struct UpdateArguments {
    target: PathBuf,
    bundle: Option<PathBuf>,
    trust_root: Option<PathBuf>,
    mode: Option<UpdateMode>,
    recover: bool,
    exact_major_target: Option<String>,
    major_confirmation: Option<PathBuf>,
}

pub(crate) fn run_update(arguments: &[String]) -> ExitCode {
    if arguments == ["--help"] {
        print!("{UPDATE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = parse_arguments(arguments).and_then(|arguments| update(&arguments));
    let result = result.unwrap_or_else(|error| failure_result(&error));
    emit_action_result(&result)
}

pub(crate) fn run_release(arguments: &[String]) -> ExitCode {
    if arguments == ["verify", "--help"] {
        print!("{RELEASE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = parse_release_arguments(arguments)
        .and_then(|(bundle, trust_root)| verify_release(&bundle, &trust_root))
        .unwrap_or_else(|error| release_failure_result(&error));
    emit_action_result(&result)
}

fn parse_release_arguments(arguments: &[String]) -> Result<(PathBuf, PathBuf), AdapterError> {
    if arguments.first().map(String::as_str) != Some("verify") {
        return Err(AdapterError::Input(
            "release requires the verify action".to_owned(),
        ));
    }
    let mut bundle = None;
    let mut trust_root = None;
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        if !matches!(option, "--bundle" | "--trust-root" | "--output") {
            return Err(AdapterError::Input(format!(
                "unknown release option: {option}"
            )));
        }
        let value = arguments
            .get(index)
            .ok_or_else(|| AdapterError::Input(format!("missing value for {option}")))?;
        index += 1;
        let slot = match option {
            "--bundle" => &mut bundle,
            "--trust-root" => &mut trust_root,
            "--output" => &mut output,
            _ => unreachable!(),
        };
        if slot.replace(value.clone()).is_some() {
            return Err(AdapterError::Input(format!(
                "duplicate release option: {option}"
            )));
        }
    }
    if output.as_deref() != Some("json") {
        return Err(AdapterError::Input(
            "release verify requires --output json".to_owned(),
        ));
    }
    Ok((
        PathBuf::from(bundle.ok_or_else(|| AdapterError::Input("missing --bundle".to_owned()))?),
        PathBuf::from(
            trust_root.ok_or_else(|| AdapterError::Input("missing --trust-root".to_owned()))?,
        ),
    ))
}

fn verify_release(bundle: &Path, trust_root: &Path) -> Result<ActionResult, AdapterError> {
    let trust_root = read_protected_file(
        trust_root,
        MAX_RELEASE_ROOT_BYTES,
        None,
        "release trust root",
    )?;
    let now_unix = current_unix_time()?;
    let verified = verify_release_repository_for_publication(&trust_root, bundle, now_unix, None)
        .map_err(map_update_error)?;
    Ok(ActionResult {
        schema_version: 1,
        action: "VerifyWork",
        status: "success",
        exit_code: 0,
        code: "hive.release-verified",
        message: "offline signed release verification completed".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "release",
            locator: bundle.display().to_string(),
            digest: verified.manifest_digest.clone(),
        }],
        next_action: None,
        data: Some(json!({
            "release_version": verified.manifest.release_version,
            "release_sequence": verified.manifest.release_sequence,
            "source_commit": verified.manifest.source.commit,
            "manifest_digest": verified.manifest_digest,
            "verified_target_count": verified.targets.len()
        })),
    })
}

fn parse_arguments(arguments: &[String]) -> Result<UpdateArguments, AdapterError> {
    let mut target = None;
    let mut bundle = None;
    let mut trust_root = None;
    let mut mode = None;
    let mut recover = false;
    let mut exact_major_target = None;
    let mut major_confirmation = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        match option {
            "--dry-run" if mode.is_none() => mode = Some(UpdateMode::DryRun),
            "--apply" if mode.is_none() => mode = Some(UpdateMode::Apply),
            "--recover" if !recover => recover = true,
            "--target"
            | "--bundle"
            | "--trust-root"
            | "--exact-major-target"
            | "--major-confirmation"
            | "--output" => {
                let value = arguments
                    .get(index)
                    .ok_or_else(|| AdapterError::Input(format!("missing value for {option}")))?;
                index += 1;
                let slot = match option {
                    "--target" => &mut target,
                    "--bundle" => &mut bundle,
                    "--trust-root" => &mut trust_root,
                    "--exact-major-target" => &mut exact_major_target,
                    "--major-confirmation" => &mut major_confirmation,
                    "--output" => &mut output,
                    _ => unreachable!(),
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(AdapterError::Input(format!(
                        "duplicate update option: {option}"
                    )));
                }
            }
            "--dry-run" | "--apply" | "--recover" => {
                return Err(AdapterError::Input(format!(
                    "duplicate or conflicting update option: {option}"
                )))
            }
            _ => {
                return Err(AdapterError::Input(format!(
                    "unknown update option: {option}"
                )))
            }
        }
    }
    if output.as_deref() != Some("json") {
        return Err(AdapterError::Input(
            "update requires --output json".to_owned(),
        ));
    }
    let target = target.ok_or_else(|| AdapterError::Input("missing --target".to_owned()))?;
    if recover {
        if mode.is_some()
            || bundle.is_some()
            || trust_root.is_some()
            || exact_major_target.is_some()
            || major_confirmation.is_some()
        {
            return Err(AdapterError::Input(
                "--recover cannot be combined with release activation options".to_owned(),
            ));
        }
    } else if mode.is_none() || bundle.is_none() || trust_root.is_none() {
        return Err(AdapterError::Input(
            "update requires --bundle, --trust-root, and exactly one of --dry-run or --apply"
                .to_owned(),
        ));
    }
    if major_confirmation.is_some() && exact_major_target.is_none() {
        return Err(AdapterError::Input(
            "--major-confirmation requires --exact-major-target".to_owned(),
        ));
    }
    if mode == Some(UpdateMode::Apply)
        && exact_major_target.is_some()
        && major_confirmation.is_none()
    {
        return Err(AdapterError::Input(
            "breaking release apply requires --major-confirmation".to_owned(),
        ));
    }
    Ok(UpdateArguments {
        target: PathBuf::from(target),
        bundle: bundle.map(PathBuf::from),
        trust_root: trust_root.map(PathBuf::from),
        mode,
        recover,
        exact_major_target,
        major_confirmation: major_confirmation.map(PathBuf::from),
    })
}

fn update(arguments: &UpdateArguments) -> Result<ActionResult, AdapterError> {
    if arguments.recover {
        recover_update(&arguments.target).map_err(map_update_error)?;
        return Ok(ActionResult {
            schema_version: 1,
            action: "UpdateHarness",
            status: "success",
            exit_code: 0,
            code: "hive.update-recovered",
            message: "incomplete update recovery completed".to_owned(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
            data: None,
        });
    }
    let target = PinnedTarget::open(&arguments.target)?;
    let trust_root_path = arguments
        .trust_root
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing update trust root".to_owned()))?;
    let trust_root = read_protected_external_file(
        &target,
        trust_root_path,
        MAX_RELEASE_ROOT_BYTES,
        "release trust root",
    )?;
    let (exact_major_target, major_approval) = load_major_authority(
        &target,
        arguments.exact_major_target.as_deref(),
        arguments.major_confirmation.as_deref(),
    )?;
    let mode = arguments
        .mode
        .ok_or_else(|| AdapterError::Input("missing update mode".to_owned()))?;
    let bundle = arguments
        .bundle
        .as_deref()
        .ok_or_else(|| AdapterError::Input("missing release bundle".to_owned()))?;
    let now_unix = current_unix_time()?;
    let outcome = execute_update(&UpdateRequest {
        target: target.requested_path(),
        repository: bundle,
        trusted_root_bytes: &trust_root,
        now_unix,
        mode,
        exact_major_target,
        major_approval: major_approval.as_ref(),
    })
    .map_err(map_update_error)?;
    let (code, message, changed_paths) = match mode {
        UpdateMode::DryRun => (
            "hive.update-dry-run-complete",
            "signed update dry-run completed without target mutation",
            Vec::new(),
        ),
        UpdateMode::Apply => (
            "hive.update-complete",
            "signed update activated and the disposable index was rebuilt",
            outcome.changed_paths.clone(),
        ),
    };
    Ok(ActionResult {
        schema_version: 1,
        action: "UpdateHarness",
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![Evidence {
            kind: "release",
            locator: bundle.display().to_string(),
            digest: outcome.plan_digest.clone(),
        }],
        next_action: None,
        data: Some(json!({
            "source_version": outcome.source_version,
            "target_version": outcome.target_version,
            "migration_id": outcome.migration_id,
            "binary_owner_action": "unchanged",
            "plan_digest": outcome.plan_digest,
            "compatibility_report_digest": outcome.compatibility_report_digest,
            "migration_table_digest": outcome.migration_table_digest,
            "planned_paths": outcome.changed_paths,
            "backup_id": outcome.backup_id,
            "index_digest": outcome.index_digest
        })),
    })
}

fn current_unix_time() -> Result<i64, AdapterError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AdapterError::Internal("system clock precedes Unix epoch".to_owned()))?
        .as_secs()
        .try_into()
        .map_err(|_| AdapterError::Internal("system clock is out of range".to_owned()))
}

fn load_major_authority(
    target: &PinnedTarget,
    exact_target: Option<&str>,
    confirmation_path: Option<&Path>,
) -> Result<(Option<SemVersion>, Option<MajorApproval>), AdapterError> {
    let Some(exact_target) = exact_target else {
        return Ok((None, None));
    };
    let exact_target: SemVersion = exact_target
        .parse()
        .map_err(|error: hive_update::ReleasePolicyError| AdapterError::Input(error.to_string()))?;
    let Some(confirmation_path) = confirmation_path else {
        return Ok((Some(exact_target), None));
    };
    validate_project_relative(confirmation_path)
        .map_err(|error| AdapterError::Safety(error.to_string()))?;
    let relative = confirmation_path;
    let bytes = target.read_required(relative, MAX_CONFIRMATION_BYTES)?;
    let confirmation: MajorConfirmation = serde_json::from_slice(&bytes).map_err(|error| {
        AdapterError::Input(format!("invalid major confirmation JSON: {error}"))
    })?;
    let source_version: SemVersion =
        confirmation
            .source_version
            .parse()
            .map_err(|error: hive_update::ReleasePolicyError| {
                AdapterError::Input(format!("invalid confirmation source version: {error}"))
            })?;
    let confirmed_target: SemVersion =
        confirmation
            .exact_target
            .parse()
            .map_err(|error: hive_update::ReleasePolicyError| {
                AdapterError::Input(format!("invalid confirmation target version: {error}"))
            })?;
    if confirmation.schema_version != 1
        || confirmed_target != exact_target
        || !confirmation.confirmed
        || ![
            &confirmation.release_plan_digest,
            &confirmation.compatibility_report_digest,
            &confirmation.migration_table_digest,
        ]
        .iter()
        .all(|digest| is_sha256_digest(digest))
    {
        return Err(AdapterError::Safety(
            "major confirmation does not bind the exact target and release reports".to_owned(),
        ));
    }
    let canonical = serde_json_canonicalizer::to_vec(&confirmation).map_err(|error| {
        AdapterError::Internal(format!("cannot canonicalize major confirmation: {error}"))
    })?;
    Ok((
        Some(exact_target),
        Some(MajorApproval {
            source_version,
            exact_target,
            release_plan_digest: confirmation.release_plan_digest,
            compatibility_report_digest: confirmation.compatibility_report_digest,
            migration_table_digest: confirmation.migration_table_digest,
            human_confirmed: true,
            confirmation_digest: sha256_digest(&canonical),
        }),
    ))
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn map_update_error(error: UpdateError) -> AdapterError {
    match error {
        UpdateError::Input(message) => AdapterError::Input(message),
        UpdateError::Verification(message) => AdapterError::Verification(message),
        UpdateError::Compatibility(message) => AdapterError::Safety(message),
        UpdateError::Unsupported(message) => AdapterError::Unsupported(message),
        UpdateError::Conflict(message) => AdapterError::Conflict(message),
        UpdateError::Internal(message) => AdapterError::Internal(message),
        UpdateError::Rollback(message) => AdapterError::Rollback(message),
    }
}

fn failure_result(error: &AdapterError) -> ActionResult {
    let (status, exit_code, code) = match error {
        AdapterError::Input(_) => ("error", 2, "hive.update-invalid-input"),
        AdapterError::Safety(_) | AdapterError::OwnerBlocked(_) => {
            ("blocked", 3, "hive.update-compatibility-blocked")
        }
        AdapterError::Conflict(_) => ("conflict", 3, "hive.update-conflict"),
        AdapterError::Unsupported(_) | AdapterError::OwnerUnsupported(_) => {
            ("unsupported", 4, "hive.update-migration-unsupported")
        }
        AdapterError::Verification(_) => (
            "verification-failed",
            5,
            "hive.update-release-verification-failed",
        ),
        AdapterError::Internal(_) => ("error", 10, "hive.internal-error"),
        AdapterError::Rollback(_) => ("error", 10, "hive.update-rollback-failed"),
    };
    ActionResult {
        schema_version: 1,
        action: "UpdateHarness",
        status,
        exit_code,
        code,
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn release_failure_result(error: &AdapterError) -> ActionResult {
    let mut result = failure_result(error);
    result.action = "VerifyWork";
    result.code = match error {
        AdapterError::Input(_) => "hive.release-invalid-input",
        AdapterError::Safety(_) | AdapterError::OwnerBlocked(_) => {
            "hive.release-trust-root-blocked"
        }
        AdapterError::Conflict(_) => "hive.release-conflict",
        AdapterError::Unsupported(_) | AdapterError::OwnerUnsupported(_) => {
            "hive.release-unsupported"
        }
        AdapterError::Verification(_) => "hive.release-verification-failed",
        AdapterError::Internal(_) => "hive.internal-error",
        AdapterError::Rollback(_) => "hive.release-rollback-failed",
    };
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_requires_exactly_one_mode_and_a_complete_offline_source() {
        for invalid in [
            vec![
                "--target".to_owned(),
                "fixture".to_owned(),
                "--output".to_owned(),
                "json".to_owned(),
            ],
            vec![
                "--target".to_owned(),
                "fixture".to_owned(),
                "--bundle".to_owned(),
                "bundle".to_owned(),
                "--trust-root".to_owned(),
                "/root.json".to_owned(),
                "--dry-run".to_owned(),
                "--apply".to_owned(),
                "--output".to_owned(),
                "json".to_owned(),
            ],
        ] {
            assert!(parse_arguments(&invalid).is_err());
        }
    }

    #[test]
    fn major_dry_run_can_prepare_confirmation_but_apply_requires_it() {
        let dry_run = vec![
            "--target".to_owned(),
            "fixture".to_owned(),
            "--bundle".to_owned(),
            "bundle".to_owned(),
            "--trust-root".to_owned(),
            "/root.json".to_owned(),
            "--dry-run".to_owned(),
            "--exact-major-target".to_owned(),
            "1.0.0".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert!(parse_arguments(&dry_run).is_ok());
        let mut apply = dry_run.clone();
        let mode = apply
            .iter()
            .position(|argument| argument == "--dry-run")
            .expect("mode");
        apply[mode] = "--apply".to_owned();
        assert!(parse_arguments(&apply).is_err());

        let mut confirmation_without_target = dry_run;
        let option = confirmation_without_target
            .iter()
            .position(|argument| argument == "--exact-major-target")
            .expect("target option");
        confirmation_without_target[option] = "--major-confirmation".to_owned();
        confirmation_without_target[option + 1] = "confirmation.json".to_owned();
        assert!(parse_arguments(&confirmation_without_target).is_err());
    }

    #[test]
    fn release_parser_and_failures_use_the_read_only_release_contract() {
        let valid = vec![
            "verify".to_owned(),
            "--bundle".to_owned(),
            "bundle".to_owned(),
            "--trust-root".to_owned(),
            "/root.json".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert!(parse_release_arguments(&valid).is_ok());
        assert!(parse_release_arguments(&valid[1..]).is_err());
        let failure = release_failure_result(&AdapterError::Verification("tampered".to_owned()));
        assert_eq!(failure.action, "VerifyWork");
        assert_eq!(failure.code, "hive.release-verification-failed");
        assert!(failure.changed_paths.is_empty());
    }
}
