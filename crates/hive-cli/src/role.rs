use super::{emit_action_result, ActionResult, Evidence};
use crate::run::{
    parse_options, portable_relative_path, read_json_request, required, role_path, run_path,
    AdapterError, FileSnapshot, PinnedTarget,
};
use hive_core::role::{RoleDocument, RoleProfile};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::OnceLock;

const HANDOFF_REQUEST_SCHEMA: &str =
    include_str!("../../../schemas/role-handoff-request.schema.json");
const MAX_ROLE_BYTES: usize = 512 * 1024;
const MAX_HANDOFF_BYTES: usize = 1024 * 1024;
const HANDOFF_BODY: &[u8] = b"# Role handoffs\n";
static RFC3339_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
const VALIDATE_USAGE: &str = "\
Validate one persistent role without mutation.

USAGE:
    hive role validate --target <dir> --role <role-id> --output json
";
const HANDOFF_USAGE: &str = "\
Record one explicit optimistic role handoff transaction.

USAGE:
    hive role handoff --target <dir> --request <request.json> --output json
";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HandoffRequest {
    schema_version: u32,
    role_id: String,
    run_id: String,
    expected_current_assignment: Option<String>,
    expected_handoff_path: Option<String>,
    expected_handoff_digest: Option<String>,
    handoff_markdown: String,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct HandoffEntry {
    markdown: String,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct HandoffEnvelope {
    schema_version: u32,
    run_id: String,
    updated_at: String,
    handoffs: BTreeMap<String, HandoffEntry>,
}

#[derive(Debug)]
struct HandoffDocument {
    envelope: HandoffEnvelope,
    body: Vec<u8>,
}

struct ValidateArguments {
    target: PathBuf,
    role_id: String,
}

struct HandoffArguments {
    target: PathBuf,
    request: PathBuf,
}

pub(crate) fn run_role(arguments: &[String]) -> ExitCode {
    if arguments == ["validate", "--help"] {
        print!("{VALIDATE_USAGE}");
        return ExitCode::SUCCESS;
    }
    if arguments == ["handoff", "--help"] {
        print!("{HANDOFF_USAGE}");
        return ExitCode::SUCCESS;
    }
    let (action, result) = match arguments.first().map(String::as_str) {
        Some("validate") => (
            "ValidateRole",
            parse_validate_arguments(&arguments[1..]).and_then(|parsed| validate(&parsed)),
        ),
        Some("handoff") => (
            "RecordRoleHandoff",
            parse_handoff_arguments(&arguments[1..]).and_then(|parsed| handoff(&parsed)),
        ),
        Some(other) => (
            "RoleWork",
            Err(AdapterError::Input(format!("unknown role action: {other}"))),
        ),
        None => (
            "RoleWork",
            Err(AdapterError::Input("role requires an action".to_owned())),
        ),
    };
    let result = result.unwrap_or_else(|error| failure_result(action, &error));
    emit_action_result(&result)
}

fn parse_validate_arguments(arguments: &[String]) -> Result<ValidateArguments, AdapterError> {
    let options = parse_options(arguments, &["--target", "--role"])?;
    Ok(ValidateArguments {
        target: PathBuf::from(required(&options, "--target")?),
        role_id: required(&options, "--role")?.to_owned(),
    })
}

fn parse_handoff_arguments(arguments: &[String]) -> Result<HandoffArguments, AdapterError> {
    let options = parse_options(arguments, &["--target", "--request"])?;
    Ok(HandoffArguments {
        target: PathBuf::from(required(&options, "--target")?),
        request: PathBuf::from(required(&options, "--request")?),
    })
}

fn failure_result(action: &'static str, error: &AdapterError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
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

fn validate(arguments: &ValidateArguments) -> Result<ActionResult, AdapterError> {
    let target = PinnedTarget::open(&arguments.target)?;
    let relative = role_path(&arguments.role_id)?;
    let bytes = target.read_required(&relative, MAX_ROLE_BYTES)?;
    let document = RoleDocument::parse(&bytes, &arguments.role_id)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    document
        .validate_runtime()
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    let data = json!({
        "profile": document.profile(),
        "body_digest": sha256_digest(document.body()),
        "canonical_digest": document
            .canonical_digest()
            .map_err(|error| AdapterError::Verification(error.to_string()))?
    });
    Ok(ActionResult {
        schema_version: 1,
        action: "ValidateRole",
        status: "success",
        exit_code: 0,
        code: "hive.role-valid",
        message: "persistent role contract is valid".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "file",
            locator: portable_relative_path(&relative),
            digest: sha256_digest(&bytes),
        }],
        next_action: None,
        data: Some(data),
    })
}

fn handoff(arguments: &HandoffArguments) -> Result<ActionResult, AdapterError> {
    handoff_with_fault(arguments, HandoffFault::None)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum HandoffFault {
    None,
    #[cfg(test)]
    AfterHandoffPublish,
}

#[allow(clippy::too_many_lines)]
fn handoff_with_fault(
    arguments: &HandoffArguments,
    fault: HandoffFault,
) -> Result<ActionResult, AdapterError> {
    let (request, request_bytes) = read_json_request::<HandoffRequest>(
        &arguments.request,
        HANDOFF_REQUEST_SCHEMA,
        "role handoff request",
    )?;
    if request.schema_version != 1 {
        return Err(AdapterError::Input(
            "unsupported role handoff request version".to_owned(),
        ));
    }
    let target = PinnedTarget::open(&arguments.target)?;
    let role_relative = role_path(&request.role_id)?;
    let plan_relative = run_path(&request.run_id, "PLAN.md")?;
    let handoff_relative = run_path(&request.run_id, "HANDOFF.md")?;
    let handoff_path = portable_relative_path(&handoff_relative);

    let _ = target.read_required(&plan_relative, MAX_HANDOFF_BYTES)?;
    let role_snapshot = target.snapshot(&role_relative)?;
    let role_bytes = role_snapshot.bytes().ok_or_else(|| {
        AdapterError::Input(format!(
            "required artifact is missing: {}",
            role_relative.display()
        ))
    })?;
    let role_document = RoleDocument::parse(role_bytes, &request.role_id)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    role_document
        .validate_runtime()
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    let handoff_snapshot = target.snapshot(&handoff_relative)?;

    let mut handoff_document = match handoff_snapshot.bytes() {
        Some(bytes) => HandoffDocument::parse(bytes)?,
        None => HandoffDocument::new(&request.run_id, &request.updated_at),
    };
    if handoff_document.envelope.run_id != request.run_id {
        return Err(AdapterError::Conflict(
            "shared handoff run_id does not match the requested run".to_owned(),
        ));
    }

    let desired_entry = HandoffEntry {
        markdown: request.handoff_markdown.clone(),
        updated_at: request.updated_at.clone(),
    };
    let desired_role = role_with_assignment(
        role_document.profile(),
        role_document.body(),
        &request.run_id,
        &handoff_path,
    )?;

    let exact_retry = role_bytes == desired_role.as_slice()
        && handoff_document.envelope.handoffs.get(&request.role_id) == Some(&desired_entry);
    if exact_retry {
        return Ok(handoff_result(
            arguments,
            &request,
            &request_bytes,
            &role_relative,
            &desired_role,
            &handoff_relative,
            handoff_snapshot.bytes().unwrap_or_default(),
            false,
            false,
        ));
    }

    if role_document.profile().current_assignment != request.expected_current_assignment {
        return Err(AdapterError::Conflict(
            "role current_assignment differs from the caller-observed value".to_owned(),
        ));
    }
    if role_document.profile().handoff_path != request.expected_handoff_path {
        return Err(AdapterError::Conflict(
            "role handoff_path differs from the caller-observed value".to_owned(),
        ));
    }
    verify_expected_handoff_digest(
        &handoff_snapshot,
        request.expected_handoff_digest.as_deref(),
    )?;

    handoff_document
        .envelope
        .handoffs
        .insert(request.role_id.clone(), desired_entry);
    handoff_document
        .envelope
        .updated_at
        .clone_from(&request.updated_at);
    let desired_handoff = handoff_document.encode()?;

    let handoff_changed = target.publish(&handoff_relative, &handoff_snapshot, &desired_handoff)?;
    if fault != HandoffFault::None {
        target.restore(&handoff_relative, &handoff_snapshot, &desired_handoff)?;
        return Err(AdapterError::Internal(
            "injected handoff transaction failure".to_owned(),
        ));
    }
    let role_changed = match target.publish(&role_relative, &role_snapshot, &desired_role) {
        Ok(changed) => changed,
        Err(error) => {
            if handoff_changed {
                target
                    .restore(&handoff_relative, &handoff_snapshot, &desired_handoff)
                    .map_err(|rollback| {
                        AdapterError::Rollback(format!(
                            "{}; handoff rollback failed: {}",
                            error.message(),
                            rollback.message()
                        ))
                    })?;
            }
            return Err(error);
        }
    };
    Ok(handoff_result(
        arguments,
        &request,
        &request_bytes,
        &role_relative,
        &desired_role,
        &handoff_relative,
        &desired_handoff,
        handoff_changed,
        role_changed,
    ))
}

fn verify_expected_handoff_digest(
    snapshot: &FileSnapshot,
    expected: Option<&str>,
) -> Result<(), AdapterError> {
    match (snapshot.bytes(), expected) {
        (None, None) => Ok(()),
        (Some(bytes), Some(expected)) if sha256_digest(bytes) == expected => Ok(()),
        (None, Some(_)) => Err(AdapterError::Conflict(
            "shared handoff is missing but the caller observed a file".to_owned(),
        )),
        (Some(_), None) => Err(AdapterError::Conflict(
            "shared handoff exists but the caller observed it as missing".to_owned(),
        )),
        (Some(_), Some(_)) => Err(AdapterError::Conflict(
            "shared handoff digest differs from the caller-observed bytes".to_owned(),
        )),
    }
}

fn role_with_assignment(
    source: &RoleProfile,
    body: &[u8],
    run_id: &str,
    handoff_path: &str,
) -> Result<Vec<u8>, AdapterError> {
    let mut profile = source.clone();
    profile.current_assignment = Some(run_id.to_owned());
    profile.handoff_path = Some(handoff_path.to_owned());
    let frontmatter = serde_json_canonicalizer::to_string(&profile)
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    let mut bytes = Vec::with_capacity(frontmatter.len() + body.len() + 10);
    bytes.extend_from_slice(b"---\n");
    bytes.extend_from_slice(frontmatter.as_bytes());
    bytes.extend_from_slice(b"\n---\n");
    bytes.extend_from_slice(body);
    let document = RoleDocument::parse(&bytes, &profile.role_id)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    document
        .validate_runtime()
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    Ok(bytes)
}

#[allow(clippy::too_many_arguments)]
fn handoff_result(
    arguments: &HandoffArguments,
    request: &HandoffRequest,
    request_bytes: &[u8],
    role_relative: &Path,
    role_bytes: &[u8],
    handoff_relative: &Path,
    handoff_bytes: &[u8],
    handoff_changed: bool,
    role_changed: bool,
) -> ActionResult {
    let changed = handoff_changed || role_changed;
    let mut changed_paths = Vec::with_capacity(2);
    if handoff_changed {
        changed_paths.push(portable_relative_path(handoff_relative));
    }
    if role_changed {
        changed_paths.push(portable_relative_path(role_relative));
    }
    ActionResult {
        schema_version: 1,
        action: "RecordRoleHandoff",
        status: "success",
        exit_code: 0,
        code: if changed {
            "hive.role-handoff-recorded"
        } else {
            "hive.role-handoff-idempotent"
        },
        message: if changed {
            "role assignment and shared handoff committed atomically".to_owned()
        } else {
            "identical role assignment and shared handoff already exist".to_owned()
        },
        changed_paths,
        evidence: vec![
            Evidence {
                kind: "file",
                locator: arguments.request.display().to_string(),
                digest: sha256_digest(request_bytes),
            },
            Evidence {
                kind: "file",
                locator: portable_relative_path(role_relative),
                digest: sha256_digest(role_bytes),
            },
            Evidence {
                kind: "file",
                locator: portable_relative_path(handoff_relative),
                digest: sha256_digest(handoff_bytes),
            },
        ],
        next_action: Some(format!(
            "checkpoint run {} with active role {}",
            request.run_id, request.role_id
        )),
        data: Some(json!({
            "role_id": request.role_id,
            "run_id": request.run_id,
            "handoff_path": portable_relative_path(handoff_relative),
            "handoff_digest": sha256_digest(handoff_bytes),
            "updated_at": request.updated_at
        })),
    }
}

impl HandoffDocument {
    fn new(run_id: &str, updated_at: &str) -> Self {
        Self {
            envelope: HandoffEnvelope {
                schema_version: 1,
                run_id: run_id.to_owned(),
                updated_at: updated_at.to_owned(),
                handoffs: BTreeMap::new(),
            },
            body: HANDOFF_BODY.to_vec(),
        }
    }

    fn parse(bytes: &[u8]) -> Result<Self, AdapterError> {
        if bytes.len() > MAX_HANDOFF_BYTES {
            return Err(AdapterError::Verification(
                "shared handoff exceeds the bounded contract".to_owned(),
            ));
        }
        let remainder = bytes.strip_prefix(b"---\n").ok_or_else(|| {
            AdapterError::Verification("shared handoff frontmatter start is missing".to_owned())
        })?;
        let delimiter = b"\n---\n";
        let end = remainder
            .windows(delimiter.len())
            .position(|window| window == delimiter)
            .ok_or_else(|| {
                AdapterError::Verification("shared handoff frontmatter end is missing".to_owned())
            })?;
        let frontmatter = &remainder[..end];
        let body = &remainder[end + delimiter.len()..];
        if body != HANDOFF_BODY {
            return Err(AdapterError::Verification(
                "shared handoff body is not canonical".to_owned(),
            ));
        }
        let envelope: HandoffEnvelope = serde_json::from_slice(frontmatter)
            .map_err(|error| AdapterError::Verification(error.to_string()))?;
        validate_handoff_envelope(&envelope)?;
        Ok(Self {
            envelope,
            body: body.to_vec(),
        })
    }

    fn encode(&self) -> Result<Vec<u8>, AdapterError> {
        validate_handoff_envelope(&self.envelope)?;
        let frontmatter = serde_json_canonicalizer::to_string(&self.envelope)
            .map_err(|error| AdapterError::Internal(error.to_string()))?;
        let mut output = Vec::with_capacity(frontmatter.len() + self.body.len() + 10);
        output.extend_from_slice(b"---\n");
        output.extend_from_slice(frontmatter.as_bytes());
        output.extend_from_slice(b"\n---\n");
        output.extend_from_slice(&self.body);
        Ok(output)
    }
}

fn validate_handoff_envelope(envelope: &HandoffEnvelope) -> Result<(), AdapterError> {
    if envelope.schema_version != 1 {
        return Err(AdapterError::Verification(
            "unsupported shared handoff version".to_owned(),
        ));
    }
    let _ = run_path(&envelope.run_id, "HANDOFF.md")?;
    if !valid_timestamp(&envelope.updated_at) {
        return Err(AdapterError::Verification(
            "shared handoff updated_at is not a bounded RFC 3339 timestamp".to_owned(),
        ));
    }
    if envelope.handoffs.is_empty() {
        return Err(AdapterError::Verification(
            "shared handoff must contain at least one role entry".to_owned(),
        ));
    }
    for (role_id, entry) in &envelope.handoffs {
        let _ = role_path(role_id)?;
        if entry.markdown.len() > 262_144 || entry.markdown.contains('\0') {
            return Err(AdapterError::Verification(format!(
                "shared handoff entry for {role_id} contains invalid Markdown"
            )));
        }
        if !valid_timestamp(&entry.updated_at) {
            return Err(AdapterError::Verification(format!(
                "shared handoff entry for {role_id} has an invalid updated_at"
            )));
        }
    }
    Ok(())
}

fn valid_timestamp(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 128
        || value.contains(['\0', '\r', '\n'])
        || !value.is_ascii()
    {
        return false;
    }
    let validator = RFC3339_VALIDATOR.get_or_init(|| {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "string",
            "format": "date-time",
            "maxLength": 128
        });
        jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .should_validate_formats(true)
            .build(&schema)
            .expect("embedded RFC 3339 timestamp schema must compile")
    });
    validator.is_valid(&Value::String(value.to_owned()))
}

pub(crate) fn validate_handoff_document(
    bytes: &[u8],
    role_id: &str,
    run_id: &str,
) -> Result<(), AdapterError> {
    let document = HandoffDocument::parse(bytes)?;
    if document.envelope.run_id != run_id {
        return Err(AdapterError::Conflict(
            "shared handoff run_id does not match the active run".to_owned(),
        ));
    }
    if !document.envelope.handoffs.contains_key(role_id) {
        return Err(AdapterError::Conflict(format!(
            "shared handoff has no entry for active role {role_id}"
        )));
    }
    Ok(())
}

pub(crate) fn handoff_entry(
    bytes: &[u8],
    role_id: &str,
    run_id: &str,
) -> Result<Value, AdapterError> {
    let document = HandoffDocument::parse(bytes)?;
    if document.envelope.run_id != run_id {
        return Err(AdapterError::Conflict(
            "shared handoff run_id does not match the active run".to_owned(),
        ));
    }
    let entry = document.envelope.handoffs.get(role_id).ok_or_else(|| {
        AdapterError::Conflict(format!(
            "shared handoff has no entry for active role {role_id}"
        ))
    })?;
    serde_json::to_value(entry).map_err(|error| AdapterError::Internal(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        handoff_with_fault, run_role, HandoffArguments, HandoffDocument, HandoffEntry, HandoffFault,
    };
    use hive_core::sha256_digest;
    use serde_json::json;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn role_bytes(role_id: &str, assignment: Option<&str>, handoff: Option<&str>) -> Vec<u8> {
        let assignment =
            assignment.map_or_else(|| "null".to_owned(), |value| format!("\"{value}\""));
        let handoff = handoff.map_or_else(|| "null".to_owned(), |value| format!("\"{value}\""));
        format!(
            "---\n{{\"allowed_capabilities\":[\"filesystem-read\"],\"context_paths\":[\"docs/\"],\"current_assignment\":{assignment},\"display_name\":\"Role\",\"handoff_path\":{handoff},\"non_responsibilities\":[\"other\"],\"responsibilities\":[\"work\"],\"role_id\":\"{role_id}\",\"schema_version\":1,\"verification_duties\":[\"verify\"],\"write_scope\":[\".hive/runs/\"]}}\n---\n# {role_id}\n\nUser body.\n"
        )
        .into_bytes()
    }

    fn setup_target(role_ids: &[&str]) -> TempDir {
        let target = TempDir::new().expect("temporary consumer");
        fs::create_dir_all(target.path().join(".hive/team/roles")).expect("role directory");
        fs::create_dir_all(target.path().join(".hive/runs/run-1")).expect("run directory");
        fs::write(
            target.path().join(".hive/runs/run-1/PLAN.md"),
            "# Plan\n\n- [ ] [done] finish\n",
        )
        .expect("plan");
        for role_id in role_ids {
            fs::write(
                target
                    .path()
                    .join(".hive/team/roles")
                    .join(format!("{role_id}.md")),
                role_bytes(role_id, None, None),
            )
            .expect("role");
        }
        target
    }

    fn write_request(
        directory: &Path,
        role_id: &str,
        expected_digest: Option<&str>,
        markdown: &str,
    ) -> std::path::PathBuf {
        let request = directory.join(format!("{role_id}-request.json"));
        fs::write(
            &request,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "role_id": role_id,
                "run_id": "run-1",
                "expected_current_assignment": null,
                "expected_handoff_path": null,
                "expected_handoff_digest": expected_digest,
                "handoff_markdown": markdown,
                "updated_at": "2026-07-24T00:00:00Z"
            }))
            .expect("request JSON"),
        )
        .expect("request");
        request.canonicalize().expect("canonical request")
    }

    fn target_path(target: &TempDir) -> std::path::PathBuf {
        target.path().canonicalize().expect("canonical target")
    }

    #[test]
    fn shared_handoff_preserves_multiple_roles() {
        let mut document = HandoffDocument::new("run-1", "2026-07-24T00:00:00Z");
        document.envelope.handoffs.insert(
            "writer".to_owned(),
            HandoffEntry {
                markdown: "writer".to_owned(),
                updated_at: "2026-07-24T00:00:00Z".to_owned(),
            },
        );
        document.envelope.handoffs.insert(
            "reviewer".to_owned(),
            HandoffEntry {
                markdown: "reviewer".to_owned(),
                updated_at: "2026-07-24T00:01:00Z".to_owned(),
            },
        );
        let encoded = document.encode().expect("encode shared handoff");
        let reparsed = HandoffDocument::parse(&encoded).expect("parse shared handoff");
        assert_eq!(reparsed.envelope.handoffs.len(), 2);
    }

    #[test]
    fn shared_handoff_rejects_tampered_entry_bounds() {
        let mut document = HandoffDocument::new("run-1", "2026-07-24T00:00:00Z");
        document.envelope.handoffs.insert(
            "writer".to_owned(),
            HandoffEntry {
                markdown: "unsafe\0markdown".to_owned(),
                updated_at: "2026-07-24T00:00:00Z".to_owned(),
            },
        );
        assert!(document.encode().is_err());
        document
            .envelope
            .handoffs
            .get_mut("writer")
            .expect("entry")
            .markdown = "safe".to_owned();
        document
            .envelope
            .handoffs
            .get_mut("writer")
            .expect("entry")
            .updated_at = "not-a-timestamp".to_owned();
        assert!(document.encode().is_err());
    }

    #[test]
    fn shared_handoff_rejects_impossible_rfc3339_calendar_time_and_offsets() {
        let invalid = [
            "2026-13-01T00:00:00Z",
            "2026-02-30T00:00:00Z",
            "2026-07-24T24:00:00Z",
            "2026-07-24T00:60:00Z",
            "2026-07-24T00:00:61Z",
            "2026-07-24T00:00:00+24:00",
            "2026-07-24T00:00:00+09:60",
        ];
        for timestamp in invalid {
            let mut document = HandoffDocument::new("run-1", timestamp);
            document.envelope.handoffs.insert(
                "writer".to_owned(),
                HandoffEntry {
                    markdown: "safe".to_owned(),
                    updated_at: "2026-07-24T00:00:00Z".to_owned(),
                },
            );
            assert!(
                document.encode().is_err(),
                "invalid envelope timestamp accepted: {timestamp}"
            );
            document.envelope.updated_at = "2026-07-24T00:00:00Z".to_owned();
            document
                .envelope
                .handoffs
                .get_mut("writer")
                .expect("entry")
                .updated_at = timestamp.to_owned();
            assert!(
                document.encode().is_err(),
                "invalid entry timestamp accepted: {timestamp}"
            );
        }

        let mut valid = HandoffDocument::new("run-1", "2024-02-29T23:59:59.123+09:00");
        valid.envelope.handoffs.insert(
            "writer".to_owned(),
            HandoffEntry {
                markdown: "safe".to_owned(),
                updated_at: "2026-07-24T00:00:00Z".to_owned(),
            },
        );
        assert!(valid.encode().is_ok());
    }

    #[test]
    fn handoff_is_atomic_idempotent_and_preserves_role_body() {
        let target = setup_target(&["writer"]);
        let original_role =
            fs::read(target.path().join(".hive/team/roles/writer.md")).expect("role");
        let request = write_request(target.path(), "writer", None, "next: verify");
        let arguments = HandoffArguments {
            target: target_path(&target),
            request: request.clone(),
        };
        let first =
            handoff_with_fault(&arguments, HandoffFault::None).expect("first handoff succeeds");
        assert_eq!(first.changed_paths.len(), 2);
        let assigned =
            fs::read(target.path().join(".hive/team/roles/writer.md")).expect("assigned");
        assert!(
            assigned.ends_with(original_role.splitn(2, |byte| *byte == b'#').nth(1).map_or(
                &[][..],
                |tail| {
                    let offset = original_role.len() - tail.len() - 1;
                    &original_role[offset..]
                }
            ))
        );
        let handoff_before =
            fs::read(target.path().join(".hive/runs/run-1/HANDOFF.md")).expect("handoff");

        let retry_arguments = HandoffArguments {
            target: target_path(&target),
            request,
        };
        let retry =
            handoff_with_fault(&retry_arguments, HandoffFault::None).expect("exact retry succeeds");
        assert!(retry.changed_paths.is_empty());
        assert_eq!(
            fs::read(target.path().join(".hive/runs/run-1/HANDOFF.md")).expect("handoff"),
            handoff_before
        );
    }

    #[test]
    fn shared_handoff_requires_exact_observed_digest_and_keeps_other_roles() {
        let target = setup_target(&["writer", "reviewer"]);
        let writer_request = write_request(target.path(), "writer", None, "writer next");
        let writer_arguments = HandoffArguments {
            target: target_path(&target),
            request: writer_request,
        };
        handoff_with_fault(&writer_arguments, HandoffFault::None).expect("writer handoff");
        let handoff_path = target.path().join(".hive/runs/run-1/HANDOFF.md");
        let observed = fs::read(&handoff_path).expect("observed handoff");
        let digest = sha256_digest(&observed);
        let reviewer_request =
            write_request(target.path(), "reviewer", Some(&digest), "reviewer next");
        let reviewer_arguments = HandoffArguments {
            target: target_path(&target),
            request: reviewer_request,
        };
        handoff_with_fault(&reviewer_arguments, HandoffFault::None).expect("reviewer handoff");
        let parsed =
            HandoffDocument::parse(&fs::read(&handoff_path).expect("handoff")).expect("parse");
        assert_eq!(parsed.envelope.handoffs.len(), 2);

        let stale = write_request(
            target.path(),
            "reviewer",
            Some(&format!("sha256:{}", "0".repeat(64))),
            "changed",
        );
        let stale_arguments = HandoffArguments {
            target: target_path(&target),
            request: stale,
        };
        let error = handoff_with_fault(&stale_arguments, HandoffFault::None)
            .err()
            .expect("stale handoff digest rejected");
        assert_eq!(error.status(), "conflict");
    }

    #[test]
    fn injected_second_write_failure_restores_first_artifact() {
        let target = setup_target(&["writer"]);
        let role_path = target.path().join(".hive/team/roles/writer.md");
        let role_before = fs::read(&role_path).expect("role");
        let request = write_request(target.path(), "writer", None, "next");
        let arguments = HandoffArguments {
            target: target_path(&target),
            request,
        };
        let error = handoff_with_fault(&arguments, HandoffFault::AfterHandoffPublish)
            .err()
            .expect("fault injected");
        assert_eq!(error.exit_code(), 10);
        assert_eq!(fs::read(&role_path).expect("role"), role_before);
        assert!(!target.path().join(".hive/runs/run-1/HANDOFF.md").exists());
    }

    #[test]
    fn role_action_help_is_read_only_and_successful() {
        for action in ["validate", "handoff"] {
            assert_eq!(
                run_role(&[action.to_owned(), "--help".to_owned()]),
                std::process::ExitCode::SUCCESS
            );
        }
    }
}
