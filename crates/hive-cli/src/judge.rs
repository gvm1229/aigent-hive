use super::{emit_action_result, ActionResult, Evidence};
use crate::run::{
    open_directory_nofollow_path, parse_options, read_parent_file, required, AdapterError,
    PinnedTarget,
};
use cap_fs_ext::MetadataExt as CapMetadataExt;
use hive_core::judge::{
    aggregate_verdicts, AggregateStatus, HumanApproval, JudgeAssignment, JudgePackage,
    JudgePackageInput, JudgeVerdict, RiskTier,
};
use hive_core::judge_auth::{
    aggregate_authenticated_verdicts, AuthenticatedQuorumInput, JudgeAttestation, JudgeTrustRoot,
};
use hive_core::{sha256_digest, validate_project_relative};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use std::fs::OpenOptions as StdOpenOptions;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PACKAGE_REQUEST_SCHEMA: &str =
    include_str!("../../../schemas/judge-package-request.schema.json");
const QUORUM_REQUEST_SCHEMA: &str =
    include_str!("../../../schemas/judge-quorum-request.schema.json");
const MAX_REFERENCED_FILE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_REFERENCED_BYTES: usize = 4 * 1024 * 1024;
const MAX_QUORUM_DOCUMENT_BYTES: usize = 256 * 1024;
const MAX_TOTAL_QUORUM_BYTES: usize = 1024 * 1024;
const MAX_TRUST_ROOT_BYTES: usize = 256 * 1024;

const PACKAGE_USAGE: &str = "\
Build one digest-bound clean-context judge package without mutation.

USAGE:
    hive judge package --target <dir> --request <request.json> --output json
";
const QUORUM_USAGE: &str = "\
Aggregate independent final judge verdicts without mutation.

USAGE:
    hive judge quorum --target <dir> --request <request.json> \
        --trust-root <external-protected.toml> --output json
";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageRequest {
    schema_version: u32,
    subject_id: String,
    risk_tier: RiskTier,
    goal: String,
    acceptance_criteria: Vec<String>,
    artifact_refs: Vec<String>,
    evidence_refs: Vec<String>,
    known_constraints: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct QuorumRequest {
    schema_version: u32,
    package: String,
    assignment: String,
    #[serde(default)]
    assignment_attestation: Option<String>,
    verdicts: Vec<String>,
    #[serde(default)]
    verdict_attestations: Vec<String>,
    approval: Option<String>,
    #[serde(default)]
    approval_attestation: Option<String>,
}

struct JudgeArguments {
    target: PathBuf,
    request: PathBuf,
    trust_root: Option<PathBuf>,
}

struct QuorumSummary {
    result: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct QuorumEvaluation {
    aggregate: hive_core::judge::AggregateOutcome,
    authenticated: bool,
    authentication: &'static str,
}

pub(crate) fn run_judge(arguments: &[String]) -> ExitCode {
    if arguments == ["package", "--help"] {
        print!("{PACKAGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    if arguments == ["quorum", "--help"] {
        print!("{QUORUM_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = match arguments.first().map(String::as_str) {
        Some("package") => {
            parse_arguments(&arguments[1..], false).and_then(|parsed| package(&parsed))
        }
        Some("quorum") => parse_arguments(&arguments[1..], true).and_then(|parsed| quorum(&parsed)),
        Some(other) => Err(AdapterError::Input(format!(
            "unknown judge action: {other}"
        ))),
        None => Err(AdapterError::Input(
            "judge requires package or quorum".to_owned(),
        )),
    }
    .unwrap_or_else(|error| failure_result(&error));
    emit_action_result(&result)
}

fn parse_arguments(
    arguments: &[String],
    allow_trust_root: bool,
) -> Result<JudgeArguments, AdapterError> {
    let allowed = if allow_trust_root {
        &["--target", "--request", "--trust-root"][..]
    } else {
        &["--target", "--request"][..]
    };
    let options = parse_options(arguments, allowed)?;
    Ok(JudgeArguments {
        target: PathBuf::from(required(&options, "--target")?),
        request: PathBuf::from(required(&options, "--request")?),
        trust_root: options
            .iter()
            .find_map(|(option, value)| (*option == "--trust-root").then(|| PathBuf::from(value))),
    })
}

fn package(arguments: &JudgeArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let request_path = validate_target_relative(&arguments.request)?;
    let request_bytes = target.read_required(request_path, MAX_QUORUM_DOCUMENT_BYTES)?;
    let request = parse_json_bytes::<PackageRequest>(
        &request_bytes,
        PACKAGE_REQUEST_SCHEMA,
        "judge package request",
    )?;
    if request.schema_version != 1 {
        return Err(AdapterError::Input(
            "unsupported judge package request version".to_owned(),
        ));
    }
    let package = JudgePackage::build(JudgePackageInput {
        subject_id: request.subject_id,
        risk_tier: request.risk_tier,
        goal: request.goal,
        acceptance_criteria: request.acceptance_criteria,
        artifact_refs: request.artifact_refs,
        evidence_refs: request.evidence_refs,
        known_constraints: request.known_constraints,
    })
    .map_err(|error| AdapterError::Input(error.to_string()))?;
    let mut total_referenced_bytes = 0;
    validate_digest_refs(
        &target,
        &package.artifact_refs,
        "artifact",
        &mut total_referenced_bytes,
    )?;
    validate_digest_refs(
        &target,
        &package.evidence_refs,
        "evidence",
        &mut total_referenced_bytes,
    )?;
    Ok(ActionResult {
        schema_version: 1,
        action: "VerifyWork",
        status: "success",
        exit_code: 0,
        code: "hive.judge-package-ready",
        message: "clean-context judge package is digest-bound and ready".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "file",
            locator: request_path.to_string_lossy().into_owned(),
            digest: sha256_digest(&request_bytes),
        }],
        next_action: None,
        data: Some(json!({ "package": package })),
    })
}

#[allow(clippy::too_many_lines)]
fn quorum(arguments: &JudgeArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let request_path = validate_target_relative(&arguments.request)?;
    let evaluation = evaluate_quorum(&target, request_path, arguments.trust_root.as_deref())?;
    let summary = quorum_summary(evaluation.aggregate.status);
    Ok(ActionResult {
        schema_version: 1,
        action: "VerifyWork",
        status: summary.status,
        exit_code: summary.exit_code,
        code: summary.code,
        message: summary.message.to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({
            "quorum": {
                "result": summary.result,
                "pass_count": evaluation.aggregate.pass_count,
                "eligible_count": evaluation.aggregate.eligible_count,
                "indeterminate_count": evaluation.aggregate.indeterminate_count,
                "excluded_count": evaluation.aggregate.excluded_count,
                "approval_valid": evaluation.aggregate.approval_valid,
                "authenticated": evaluation.authenticated,
                "authentication": evaluation.authentication
            }
        })),
    })
}

/// Re-evaluate an externally authenticated quorum for one exact loop subject.
///
/// The loop adapter owns no signer and writes no judge state.  It only binds a
/// pre-existing, digest-addressed request to the active loop evidence before
/// accepting a terminal Judge result.
pub(crate) fn verify_authenticated_loop_quorum(
    target: &PinnedTarget,
    request_path: &Path,
    trust_root_path: &Path,
    expected_subject_id: &str,
) -> Result<(), AdapterError> {
    let evaluation = evaluate_quorum(target, request_path, Some(trust_root_path))?;
    if evaluation.authentication != "ed25519"
        || !evaluation.authenticated
        || evaluation.aggregate.status != AggregateStatus::Pass
    {
        return Err(AdapterError::Verification(
            "judge loop quorum is not an authenticated PASS".to_owned(),
        ));
    }

    let request_bytes = target.read_required(request_path, MAX_QUORUM_DOCUMENT_BYTES)?;
    let request = parse_json_bytes::<QuorumRequest>(
        &request_bytes,
        QUORUM_REQUEST_SCHEMA,
        "judge quorum request",
    )?;
    if request.schema_version != 2 {
        return Err(AdapterError::Verification(
            "judge loop quorum must use authenticated request version 2".to_owned(),
        ));
    }
    let mut total_bytes = request_bytes.len();
    let package_bytes = read_target_document(target, &request.package, &mut total_bytes)?;
    let package = JudgePackage::parse_json(&package_bytes).map_err(|_| {
        AdapterError::Verification("judge loop quorum package is invalid".to_owned())
    })?;
    if package.subject_id != expected_subject_id {
        return Err(AdapterError::Verification(
            "judge loop quorum subject is not bound to this evidence".to_owned(),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn evaluate_quorum(
    target: &PinnedTarget,
    request_path: &Path,
    trust_root_path: Option<&Path>,
) -> Result<QuorumEvaluation, AdapterError> {
    let request_bytes = target.read_required(request_path, MAX_QUORUM_DOCUMENT_BYTES)?;
    let request = parse_json_bytes::<QuorumRequest>(
        &request_bytes,
        QUORUM_REQUEST_SCHEMA,
        "judge quorum request",
    )?;
    if !matches!(request.schema_version, 1 | 2) {
        return Err(AdapterError::Input(
            "unsupported judge quorum request version".to_owned(),
        ));
    }
    let mut total_bytes = request_bytes.len();
    let package_bytes = read_target_document(target, &request.package, &mut total_bytes)?;
    let Ok(package) = JudgePackage::parse_json(&package_bytes) else {
        let authentication = if request.schema_version == 2 {
            "ed25519"
        } else {
            "none"
        };
        return Ok(indeterminate_evaluation(
            request.verdicts.len(),
            authentication,
        ));
    };
    let assignment_bytes = read_target_document(target, &request.assignment, &mut total_bytes)?;
    let Ok(assignment) = JudgeAssignment::parse_json(&assignment_bytes, &package) else {
        let authentication = if request.schema_version == 2 {
            "ed25519"
        } else {
            "none"
        };
        return Ok(indeterminate_evaluation(
            request.verdicts.len(),
            authentication,
        ));
    };
    let verdicts = load_verdicts(target, &request.verdicts, &mut total_bytes)?;
    let approval = if let Some(relative) = request.approval.as_deref() {
        let bytes = read_target_document(target, relative, &mut total_bytes)?;
        HumanApproval::parse_json(&bytes).ok()
    } else {
        None
    };
    let (aggregate, authenticated, authentication) = if request.schema_version == 1 {
        (
            legacy_unsigned_outcome(&package, &assignment, approval.as_ref(), &verdicts),
            false,
            "none",
        )
    } else {
        let trust_root_path = trust_root_path.ok_or_else(|| {
            AdapterError::Safety(
                "authenticated judge quorum requires an external protected trust root".to_owned(),
            )
        })?;
        let trust_root = load_protected_trust_root(target, trust_root_path)?;
        let assignment_attestation = match request.assignment_attestation.as_deref() {
            Some(relative) => load_optional_attestation(target, relative, &mut total_bytes)?,
            None => None,
        };
        let Some(assignment_attestation) = assignment_attestation else {
            return Ok(indeterminate_evaluation(verdicts.len(), "ed25519"));
        };
        let verdict_attestations =
            load_attestations(target, &request.verdict_attestations, &mut total_bytes)?;
        let approval_attestation = if let Some(relative) = request.approval_attestation.as_deref() {
            load_optional_attestation(target, relative, &mut total_bytes)?
        } else {
            None
        };
        let outcome = aggregate_authenticated_verdicts(&AuthenticatedQuorumInput {
            trust_root: &trust_root,
            package: &package,
            assignment: &assignment,
            assignment_attestation: &assignment_attestation,
            normal_requested: true,
            verdicts: &verdicts,
            verdict_attestations: &verdict_attestations,
            human_approval: approval.as_ref(),
            approval_attestation: approval_attestation.as_ref(),
        });
        (outcome.aggregate, outcome.authenticated, "ed25519")
    };
    Ok(QuorumEvaluation {
        aggregate,
        authenticated,
        authentication,
    })
}

fn load_attestations(
    target: &PinnedTarget,
    refs: &[String],
    total_bytes: &mut usize,
) -> Result<Vec<Option<JudgeAttestation>>, AdapterError> {
    refs.iter()
        .map(|relative| load_optional_attestation(target, relative, total_bytes))
        .collect()
}

fn load_optional_attestation(
    target: &PinnedTarget,
    relative: &str,
    total_bytes: &mut usize,
) -> Result<Option<JudgeAttestation>, AdapterError> {
    match read_target_document(target, relative, total_bytes) {
        Ok(bytes) => Ok(JudgeAttestation::parse_json(&bytes).ok()),
        Err(AdapterError::Input(_) | AdapterError::Verification(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

fn load_verdicts(
    target: &PinnedTarget,
    refs: &[String],
    total_bytes: &mut usize,
) -> Result<Vec<JudgeVerdict>, AdapterError> {
    let mut verdicts = Vec::new();
    for relative in refs {
        let relative_path = validate_target_relative(Path::new(relative))?;
        let bytes = target.read_required(relative_path, MAX_QUORUM_DOCUMENT_BYTES)?;
        add_quorum_bytes(total_bytes, bytes.len())?;
        if let Ok(verdict) = JudgeVerdict::parse_json(&bytes) {
            verdicts.push(verdict);
        }
    }
    Ok(verdicts)
}

fn read_target_document(
    target: &PinnedTarget,
    relative: &str,
    total_bytes: &mut usize,
) -> Result<Vec<u8>, AdapterError> {
    let relative = validate_target_relative(Path::new(relative))?;
    let bytes = target.read_required(relative, MAX_QUORUM_DOCUMENT_BYTES)?;
    add_quorum_bytes(total_bytes, bytes.len())?;
    Ok(bytes)
}

fn legacy_unsigned_outcome(
    package: &JudgePackage,
    assignment: &JudgeAssignment,
    approval: Option<&HumanApproval>,
    verdicts: &[JudgeVerdict],
) -> hive_core::judge::AggregateOutcome {
    let mut aggregate = aggregate_verdicts(package, assignment, true, approval, verdicts);
    if aggregate.status == AggregateStatus::Pass {
        aggregate.status = AggregateStatus::Indeterminate;
    }
    aggregate.approval_valid = false;
    aggregate
}

fn indeterminate_evaluation(
    excluded_count: usize,
    authentication: &'static str,
) -> QuorumEvaluation {
    QuorumEvaluation {
        aggregate: hive_core::judge::AggregateOutcome {
            status: AggregateStatus::Indeterminate,
            eligible_count: 0,
            pass_count: 0,
            indeterminate_count: 0,
            excluded_count,
            approval_valid: false,
        },
        authenticated: false,
        authentication,
    }
}

fn load_protected_trust_root(
    target: &PinnedTarget,
    path: &Path,
) -> Result<JudgeTrustRoot, AdapterError> {
    let bytes =
        read_protected_external_file(target, path, MAX_TRUST_ROOT_BYTES, "judge trust root")?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| AdapterError::Input("judge trust root is not UTF-8 TOML".to_owned()))?;
    let trust_root: JudgeTrustRoot = toml::from_str(text)
        .map_err(|error| AdapterError::Input(format!("invalid judge trust root TOML: {error}")))?;
    trust_root
        .validate()
        .map_err(|error| AdapterError::Input(error.to_string()))?;
    Ok(trust_root)
}

pub(crate) fn read_protected_external_file(
    target: &PinnedTarget,
    path: &Path,
    maximum_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, AdapterError> {
    read_protected_file(path, maximum_bytes, Some(target.requested_path()), label)
}

pub(crate) fn read_protected_file(
    path: &Path,
    maximum_bytes: usize,
    forbidden_root: Option<&Path>,
    label: &str,
) -> Result<Vec<u8>, AdapterError> {
    if !path.is_absolute() {
        return Err(AdapterError::Safety(format!(
            "{label} path must be absolute"
        )));
    }
    if forbidden_root.is_some_and(|root| path.starts_with(root)) {
        return Err(AdapterError::Safety(format!(
            "{label} must be outside the protected target"
        )));
    }
    let parent_path = path
        .parent()
        .ok_or_else(|| AdapterError::Safety(format!("{label} has no external parent")))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| AdapterError::Safety(format!("{label} has no file name")))?;
    validate_platform_trust_root_protection(path, label)?;
    let parent = open_directory_nofollow_path(parent_path)?;
    let before = parent
        .symlink_metadata(file_name)
        .map_err(|error| trust_root_safety_error("inspect", &error, label))?;
    if !before.is_file() || usize::try_from(before.len()).unwrap_or(usize::MAX) > maximum_bytes {
        return Err(AdapterError::Safety(format!(
            "{label} must be a bounded no-follow regular file"
        )));
    }
    let bytes = read_parent_file(&parent, file_name, maximum_bytes)
        .map_err(|error| trust_root_safety_error("read", &error, label))?;
    let after = parent
        .symlink_metadata(file_name)
        .map_err(|error| trust_root_safety_error("reinspect", &error, label))?;
    if CapMetadataExt::dev(&before) != CapMetadataExt::dev(&after)
        || CapMetadataExt::ino(&before) != CapMetadataExt::ino(&after)
        || before.len() != after.len()
    {
        return Err(AdapterError::Safety(format!(
            "{label} changed during verification"
        )));
    }
    validate_platform_trust_root_protection(path, label)?;
    Ok(bytes)
}

fn trust_root_safety_error(action: &str, error: &io::Error, label: &str) -> AdapterError {
    AdapterError::Safety(format!("cannot {action} external {label} safely: {error}"))
}

#[cfg(unix)]
fn validate_platform_trust_root_protection(path: &Path, label: &str) -> Result<(), AdapterError> {
    use cap_fs_ext::{AccessType, DirExt};
    use cap_primitives::fs::AccessModes;
    use std::os::unix::fs::MetadataExt;

    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| trust_root_safety_error("inspect", &error, label))?;
    if !metadata.file_type().is_file()
        || metadata.uid() != 0
        || metadata.mode() & 0o222 != 0
        || std::os::unix::fs::MetadataExt::nlink(&metadata) != 1
    {
        return Err(AdapterError::Safety(format!(
            "{label} must be a root-owned, non-writable, single-link regular file"
        )));
    }
    for ancestor in path.parent().into_iter().flat_map(Path::ancestors) {
        let metadata = std::fs::symlink_metadata(ancestor)
            .map_err(|error| trust_root_safety_error("inspect ancestor", &error, label))?;
        if !metadata.file_type().is_dir() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err(AdapterError::Safety(format!(
                "{label} ancestor is not admin-owned and replacement-safe"
            )));
        }
        let directory = open_directory_nofollow_path(ancestor)?;
        let mutation_access = AccessType::Access(AccessModes {
            readable: false,
            writable: true,
            executable: true,
        });
        match directory.access(".", mutation_access) {
            Ok(()) => {
                return Err(AdapterError::Safety(format!(
                    "current process can replace the {label} through an ancestor"
                )))
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem
                ) => {}
            Err(error) => {
                return Err(trust_root_safety_error(
                    "prove trust-root ancestor mutation denial for",
                    &error,
                    label,
                ))
            }
        }
    }
    match StdOpenOptions::new().write(true).open(path) {
        Ok(_) => {
            return Err(AdapterError::Safety(format!(
                "current process can write the {label}"
            )))
        }
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem
            ) => {}
        Err(error) => {
            return Err(trust_root_safety_error(
                "prove trust-root write denial for",
                &error,
                label,
            ))
        }
    }
    Ok(())
}

#[cfg(windows)]
fn validate_platform_trust_root_protection(path: &Path, label: &str) -> Result<(), AdapterError> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    const FILE_SHARE_ALL: u32 = 0x0000_0007;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const DELETE: u32 = 0x0001_0000;
    const WRITE_DAC: u32 = 0x0004_0000;
    const WRITE_OWNER: u32 = 0x0008_0000;
    const FILE_WRITE_DATA: u32 = 0x0000_0002;
    const FILE_APPEND_DATA: u32 = 0x0000_0004;
    const FILE_WRITE_EA: u32 = 0x0000_0010;
    const FILE_DELETE_CHILD: u32 = 0x0000_0040;
    const FILE_WRITE_ATTRIBUTES: u32 = 0x0000_0100;
    const FILE_MUTATION_RIGHTS: &[u32] = &[
        FILE_WRITE_DATA,
        FILE_APPEND_DATA,
        FILE_WRITE_EA,
        FILE_WRITE_ATTRIBUTES,
        DELETE,
        WRITE_DAC,
        WRITE_OWNER,
    ];
    const DIRECTORY_MUTATION_RIGHTS: &[u32] = &[
        FILE_WRITE_DATA,
        FILE_APPEND_DATA,
        FILE_WRITE_EA,
        FILE_DELETE_CHILD,
        FILE_WRITE_ATTRIBUTES,
        DELETE,
        WRITE_DAC,
        WRITE_OWNER,
    ];

    for (index, component) in path.ancestors().enumerate() {
        let metadata = std::fs::symlink_metadata(component)
            .map_err(|error| trust_root_safety_error("inspect path component", &error, label))?;
        let is_file = index == 0;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || (is_file && !metadata.file_type().is_file())
            || (!is_file && !metadata.file_type().is_dir())
        {
            return Err(AdapterError::Safety(format!(
                "Windows {label} contains a reparse or nonregular path component"
            )));
        }
        let rights = if is_file {
            FILE_MUTATION_RIGHTS
        } else {
            DIRECTORY_MUTATION_RIGHTS
        };
        let flags = FILE_FLAG_OPEN_REPARSE_POINT
            | if is_file {
                0
            } else {
                FILE_FLAG_BACKUP_SEMANTICS
            };
        for right in rights {
            let result = StdOpenOptions::new()
                .access_mode(*right)
                .share_mode(FILE_SHARE_ALL)
                .custom_flags(flags)
                .open(component);
            match result {
                Ok(_) => {
                    return Err(AdapterError::Safety(format!(
                        "current process can mutate the Windows {label} or an ancestor"
                    )))
                }
                Err(error) if error.raw_os_error() == Some(5) => {}
                Err(error) => {
                    return Err(trust_root_safety_error(
                        "prove Windows mutation denial for",
                        &error,
                        label,
                    ))
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn validate_platform_trust_root_protection(_path: &Path, label: &str) -> Result<(), AdapterError> {
    Err(AdapterError::Unsupported(format!(
        "protected {label} files are unsupported on this platform"
    )))
}

fn add_quorum_bytes(total_bytes: &mut usize, additional: usize) -> Result<(), AdapterError> {
    *total_bytes = total_bytes
        .checked_add(additional)
        .ok_or_else(|| AdapterError::Input("judge documents exceed byte limit".to_owned()))?;
    if *total_bytes > MAX_TOTAL_QUORUM_BYTES {
        return Err(AdapterError::Input(format!(
            "judge documents exceed {MAX_TOTAL_QUORUM_BYTES} total bytes"
        )));
    }
    Ok(())
}

fn parse_json_bytes<T: DeserializeOwned>(
    bytes: &[u8],
    schema: &str,
    label: &str,
) -> Result<T, AdapterError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| AdapterError::Input(format!("invalid {label} JSON: {error}")))?;
    let schema: serde_json::Value = serde_json::from_str(schema)
        .map_err(|error| AdapterError::Internal(format!("invalid embedded schema: {error}")))?;
    jsonschema::meta::validate(&schema)
        .map_err(|error| AdapterError::Internal(format!("invalid embedded schema: {error}")))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| AdapterError::Internal(format!("cannot compile schema: {error}")))?;
    validator
        .validate(&value)
        .map_err(|error| AdapterError::Input(format!("{label} violates schema: {error}")))?;
    serde_json::from_value(value)
        .map_err(|error| AdapterError::Input(format!("invalid {label}: {error}")))
}

fn validate_target_relative(path: &Path) -> Result<&Path, AdapterError> {
    if path.to_string_lossy().contains('\\') {
        return Err(AdapterError::Safety(
            "judge document path must use portable forward slashes".to_owned(),
        ));
    }
    validate_project_relative(path).map_err(|error| AdapterError::Safety(error.to_string()))?;
    Ok(path)
}

const fn quorum_summary(aggregate: AggregateStatus) -> QuorumSummary {
    match aggregate {
        AggregateStatus::Pass => QuorumSummary {
            result: "PASS",
            status: "success",
            exit_code: 0,
            code: "hive.judge-quorum-pass",
            message: "judge quorum passed",
        },
        AggregateStatus::Fail => QuorumSummary {
            result: "FAIL",
            status: "verification-failed",
            exit_code: 5,
            code: "hive.judge-quorum-fail",
            message: "judge quorum failed",
        },
        AggregateStatus::Indeterminate => QuorumSummary {
            result: "INDETERMINATE",
            status: "verification-failed",
            exit_code: 5,
            code: "hive.judge-quorum-indeterminate",
            message: "judge quorum is indeterminate",
        },
        AggregateStatus::NotRequired => QuorumSummary {
            result: "NOT_REQUIRED",
            status: "success",
            exit_code: 0,
            code: "hive.judge-quorum-not-required",
            message: "judge quorum was not required",
        },
    }
}

fn validate_digest_refs(
    target: &PinnedTarget,
    refs: &[String],
    label: &str,
    total_bytes: &mut usize,
) -> Result<(), AdapterError> {
    for reference in refs {
        let (path, digest) = reference.rsplit_once("#sha256:").ok_or_else(|| {
            AdapterError::Input(format!(
                "{label} reference must end with an exact sha256 digest"
            ))
        })?;
        if path.contains('\\')
            || digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(AdapterError::Input(format!(
                "{label} reference must be a portable project-relative path with a lowercase sha256 digest"
            )));
        }
        validate_project_relative(Path::new(path))
            .map_err(|error| AdapterError::Safety(error.to_string()))?;
        let bytes = target.read_required(Path::new(path), MAX_REFERENCED_FILE_BYTES)?;
        *total_bytes = total_bytes.checked_add(bytes.len()).ok_or_else(|| {
            AdapterError::Input("judge package references exceed the total byte limit".to_owned())
        })?;
        if *total_bytes > MAX_TOTAL_REFERENCED_BYTES {
            return Err(AdapterError::Input(format!(
                "judge package references exceed {MAX_TOTAL_REFERENCED_BYTES} total bytes"
            )));
        }
        let expected = format!("sha256:{digest}");
        let actual = sha256_digest(&bytes);
        if actual != expected {
            return Err(AdapterError::Verification(format!(
                "{label} reference digest mismatch for {path}: expected {expected}, computed {actual}"
            )));
        }
    }
    Ok(())
}

fn failure_result(error: &AdapterError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "VerifyWork",
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
