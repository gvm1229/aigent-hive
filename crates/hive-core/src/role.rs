//! Pure persistent-role Markdown contracts.

use crate::{sha256_digest, validate_json_schema, validate_project_relative};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;

const ROLE_SCHEMA: &str = include_str!("../../../schemas/role-profile.schema.json");
const MAX_ROLE_DOCUMENT_BYTES: usize = 512 * 1024;
const MAX_ROLE_FRONTMATTER_BYTES: usize = 64 * 1024;
const MAX_ROLE_BODY_BYTES: usize = 256 * 1024;

/// Persistent role profile stored in Markdown frontmatter.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleProfile {
    /// Capabilities this role may request from the resolved owner.
    pub allowed_capabilities: Vec<String>,
    /// Project-relative selectors defining the role's bounded read context.
    pub context_paths: Vec<String>,
    /// Current assignment, independent of any one host session.
    pub current_assignment: Option<String>,
    /// Human-facing role name.
    pub display_name: String,
    /// Project-relative Markdown handoff path below `.hive/runs/`.
    pub handoff_path: Option<String>,
    /// Work this role must not absorb.
    pub non_responsibilities: Vec<String>,
    /// Work this role owns.
    pub responsibilities: Vec<String>,
    /// Stable role identifier.
    pub role_id: String,
    /// Role contract version.
    pub schema_version: u32,
    /// Verification responsibilities.
    pub verification_duties: Vec<String>,
    /// Project-relative selectors defining the role's bounded write scope.
    pub write_scope: Vec<String>,
}

/// Parsed role profile with the exact Markdown body bytes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RoleDocument {
    profile: RoleProfile,
    body: Vec<u8>,
}

impl RoleDocument {
    /// Parse a schema-valid role document for an expected stable role id.
    ///
    /// The frontmatter accepts YAML-compatible syntax. The body is validated as
    /// bounded UTF-8 Markdown and retained byte-for-byte.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed frontmatter, schema violations, an
    /// id mismatch, or an oversized document. Version-1 legacy path values are
    /// retained for diagnosis; call [`Self::validate_runtime`] before using
    /// path selectors or handoff data.
    pub fn parse(bytes: &[u8], expected_role_id: &str) -> Result<Self, RoleContractError> {
        if bytes.len() > MAX_ROLE_DOCUMENT_BYTES {
            return Err(RoleContractError::TooLarge("role document"));
        }
        if !valid_role_id(expected_role_id) {
            return Err(RoleContractError::InvalidRoleId(
                expected_role_id.to_owned(),
            ));
        }
        let (frontmatter, body) = split_frontmatter(bytes)?;
        if frontmatter.len() > MAX_ROLE_FRONTMATTER_BYTES {
            return Err(RoleContractError::TooLarge("role frontmatter"));
        }
        if body.len() > MAX_ROLE_BODY_BYTES {
            return Err(RoleContractError::TooLarge("role Markdown body"));
        }
        std::str::from_utf8(body)
            .map_err(|_| RoleContractError::InvalidUtf8("role Markdown body"))?;
        let value: serde_json::Value = serde_yaml::from_slice(frontmatter)
            .map_err(|error| RoleContractError::Malformed(error.to_string()))?;
        validate_json_schema(ROLE_SCHEMA, &value, "role profile")
            .map_err(RoleContractError::Schema)?;
        let profile: RoleProfile = serde_json::from_value(value)
            .map_err(|error| RoleContractError::Malformed(error.to_string()))?;
        validate_profile_identity(&profile)?;
        if profile.role_id != expected_role_id {
            return Err(RoleContractError::RoleIdMismatch {
                expected: expected_role_id.to_owned(),
                actual: profile.role_id,
            });
        }
        Ok(Self {
            profile,
            body: body.to_vec(),
        })
    }

    /// Return the validated role profile.
    #[must_use]
    pub const fn profile(&self) -> &RoleProfile {
        &self.profile
    }

    /// Return the exact Markdown body bytes supplied to [`Self::parse`].
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Validate role paths before runtime use.
    ///
    /// # Errors
    ///
    /// Returns a typed error for traversal, absolute paths, foreign host
    /// namespaces, or a handoff outside the Hive run namespace.
    pub fn validate_runtime(&self) -> Result<(), RoleContractError> {
        validate_profile_runtime(&self.profile)
    }

    /// Encode canonical JSON frontmatter with LF delimiters and the exact body.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the profile or body no longer satisfies the
    /// bounded role contract.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, RoleContractError> {
        validate_profile_identity(&self.profile)?;
        if self.body.len() > MAX_ROLE_BODY_BYTES {
            return Err(RoleContractError::TooLarge("role Markdown body"));
        }
        std::str::from_utf8(&self.body)
            .map_err(|_| RoleContractError::InvalidUtf8("role Markdown body"))?;
        let value = serde_json::to_value(&self.profile)
            .map_err(|error| RoleContractError::Malformed(error.to_string()))?;
        validate_json_schema(ROLE_SCHEMA, &value, "role profile")
            .map_err(RoleContractError::Schema)?;
        let frontmatter = serde_json_canonicalizer::to_string(&self.profile)
            .map_err(|error| RoleContractError::Malformed(error.to_string()))?;
        let mut output = Vec::with_capacity(frontmatter.len() + self.body.len() + 10);
        output.extend_from_slice(b"---\n");
        output.extend_from_slice(frontmatter.as_bytes());
        output.extend_from_slice(b"\n---\n");
        output.extend_from_slice(&self.body);
        Ok(output)
    }

    /// Return the digest of the deterministic canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns a typed error when canonical encoding fails validation.
    pub fn canonical_digest(&self) -> Result<String, RoleContractError> {
        self.encode_canonical().map(|bytes| sha256_digest(&bytes))
    }
}

/// Pure role-contract validation errors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RoleContractError {
    /// A bounded role artifact exceeded its maximum size.
    TooLarge(&'static str),
    /// A Markdown component is not UTF-8.
    InvalidUtf8(&'static str),
    /// Frontmatter or typed data is malformed.
    Malformed(String),
    /// Frontmatter violates the role JSON Schema.
    Schema(String),
    /// The requested or stored role id is invalid.
    InvalidRoleId(String),
    /// The path-selected role does not match the stored stable identity.
    RoleIdMismatch { expected: String, actual: String },
    /// A role path is unsafe or outside its allowed canonical surface.
    UnsafePath { field: &'static str, path: String },
}

impl Display for RoleContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge(label) => write!(formatter, "{label} exceeds the bounded contract"),
            Self::InvalidUtf8(label) => write!(formatter, "{label} must be UTF-8"),
            Self::Malformed(message) | Self::Schema(message) => formatter.write_str(message),
            Self::InvalidRoleId(role_id) => write!(formatter, "invalid role id: {role_id}"),
            Self::RoleIdMismatch { expected, actual } => write!(
                formatter,
                "role id mismatch: expected {expected}, found {actual}"
            ),
            Self::UnsafePath { field, path } => {
                write!(formatter, "{field} contains an unsafe project path: {path}")
            }
        }
    }
}

impl Error for RoleContractError {}

fn split_frontmatter(bytes: &[u8]) -> Result<(&[u8], &[u8]), RoleContractError> {
    let (remainder, delimiter) = if let Some(remainder) = bytes.strip_prefix(b"---\n") {
        (remainder, b"\n---\n".as_slice())
    } else if let Some(remainder) = bytes.strip_prefix(b"---\r\n") {
        (remainder, b"\r\n---\r\n".as_slice())
    } else {
        return Err(RoleContractError::Malformed(
            "role frontmatter start is missing".to_owned(),
        ));
    };
    let index = find_subslice(remainder, delimiter).ok_or_else(|| {
        RoleContractError::Malformed("role frontmatter end is missing".to_owned())
    })?;
    Ok((&remainder[..index], &remainder[index + delimiter.len()..]))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn validate_profile_identity(profile: &RoleProfile) -> Result<(), RoleContractError> {
    if profile.schema_version != 1 || !valid_role_id(&profile.role_id) {
        return Err(RoleContractError::InvalidRoleId(profile.role_id.clone()));
    }
    Ok(())
}

fn validate_profile_runtime(profile: &RoleProfile) -> Result<(), RoleContractError> {
    validate_profile_identity(profile)?;
    for path in &profile.context_paths {
        validate_scope_path("context_paths", path)?;
    }
    for path in &profile.write_scope {
        validate_scope_path("write_scope", path)?;
    }
    if let Some(path) = profile.handoff_path.as_deref() {
        validate_handoff_path(path)?;
    }
    Ok(())
}

fn validate_scope_path(field: &'static str, value: &str) -> Result<(), RoleContractError> {
    let normalized = value.strip_suffix('/').unwrap_or(value);
    if normalized.is_empty()
        || value.contains(['\0', '\r', '\n'])
        || validate_project_relative(Path::new(normalized)).is_err()
    {
        return Err(RoleContractError::UnsafePath {
            field,
            path: value.to_owned(),
        });
    }
    Ok(())
}

fn validate_handoff_path(value: &str) -> Result<(), RoleContractError> {
    if value.contains(['\0', '\r', '\n', '*', '?', '[', ']'])
        || !value.starts_with(".hive/runs/")
        || Path::new(value).extension() != Some(std::ffi::OsStr::new("md"))
        || validate_project_relative(Path::new(value)).is_err()
    {
        return Err(RoleContractError::UnsafePath {
            field: "handoff_path",
            path: value.to_owned(),
        });
    }
    Ok(())
}

fn valid_role_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (2..=63).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::{RoleContractError, RoleDocument};

    fn role(frontmatter: &str, body: &[u8]) -> Vec<u8> {
        let mut bytes = format!("---\n{frontmatter}\n---\n").into_bytes();
        bytes.extend_from_slice(body);
        bytes
    }

    fn valid_frontmatter() -> &'static str {
        "schema_version: 1\nrole_id: reviewer\ndisplay_name: Reviewer\nresponsibilities:\n  - verify acceptance criteria\nnon_responsibilities:\n  - implement artifacts\ncontext_paths:\n  - docs/**\nallowed_capabilities:\n  - filesystem-read\n  - subagents\nwrite_scope:\n  - .hive/runs/\nverification_duties:\n  - attach reproducible evidence\ncurrent_assignment: audit phase four\nhandoff_path: .hive/runs/demo/HANDOFF.md"
    }

    #[test]
    fn yaml_frontmatter_preserves_exact_body_bytes() {
        let body = b"# Reviewer\r\n\r\nCurrent assignment.\r\n";
        let document =
            RoleDocument::parse(&role(valid_frontmatter(), body), "reviewer").expect("valid role");

        assert_eq!(document.body(), body);
        assert_eq!(document.profile().role_id, "reviewer");
        assert_eq!(
            document.profile().handoff_path.as_deref(),
            Some(".hive/runs/demo/HANDOFF.md")
        );
    }

    #[test]
    fn rejects_missing_and_mismatched_role_identity() {
        let missing = valid_frontmatter().replace("role_id: reviewer\n", "");
        assert!(matches!(
            RoleDocument::parse(&role(&missing, b"body\n"), "reviewer"),
            Err(RoleContractError::Schema(_))
        ));
        assert!(matches!(
            RoleDocument::parse(&role(valid_frontmatter(), b"body\n"), "implementer"),
            Err(RoleContractError::RoleIdMismatch { .. })
        ));
        assert!(matches!(
            RoleDocument::parse(b"---\n{bad\n---\nbody\n", "reviewer"),
            Err(RoleContractError::Malformed(_))
        ));
    }

    #[test]
    fn rejects_traversal_absolute_and_foreign_paths() {
        for hostile in [
            ("docs/**", "../outside"),
            ("docs/**", "/tmp/outside"),
            ("docs/**", ".omx/state"),
            (".hive/runs/demo/HANDOFF.md", "../outside.md"),
        ] {
            let frontmatter = valid_frontmatter().replace(hostile.0, hostile.1);
            let document = RoleDocument::parse(&role(&frontmatter, b"body\n"), "reviewer")
                .expect("legacy schema-valid role should parse for diagnosis");
            assert!(
                matches!(
                    document.validate_runtime(),
                    Err(RoleContractError::UnsafePath { .. })
                ),
                "hostile path was accepted: {}",
                hostile.1
            );
        }
    }

    #[test]
    fn legacy_optional_fields_and_duplicate_path_arrays_remain_accepted() {
        let legacy = "schema_version: 1\nrole_id: reviewer\ndisplay_name: Legacy Reviewer\nresponsibilities: [verify]\nnon_responsibilities: []\ncontext_paths: [docs/**, docs/**]\nallowed_capabilities: [filesystem-read]\nwrite_scope: [.hive/runs/, .hive/runs/]\nverification_duties: [record evidence]";
        let document = RoleDocument::parse(&role(legacy, b"# Legacy role\n"), "reviewer")
            .expect("legacy 0.4 role should parse");

        assert_eq!(document.profile().current_assignment, None);
        assert_eq!(document.profile().handoff_path, None);
        assert_eq!(document.profile().context_paths.len(), 2);
        assert_eq!(document.profile().write_scope.len(), 2);
    }

    #[test]
    fn rejects_oversized_markdown_body() {
        let body = vec![b'x'; super::MAX_ROLE_BODY_BYTES + 1];
        assert!(matches!(
            RoleDocument::parse(&role(valid_frontmatter(), &body), "reviewer"),
            Err(RoleContractError::TooLarge("role Markdown body"))
        ));
    }

    #[test]
    fn canonical_encoding_and_digest_are_deterministic() {
        let first = RoleDocument::parse(
            &role(valid_frontmatter(), b"# Reviewer\n\nBody.\n"),
            "reviewer",
        )
        .expect("valid role");
        let second = RoleDocument::parse(
            &role(valid_frontmatter(), b"# Reviewer\n\nBody.\n"),
            "reviewer",
        )
        .expect("valid role");

        assert_eq!(
            first.encode_canonical().expect("canonical role"),
            second.encode_canonical().expect("canonical role")
        );
        assert_eq!(
            first.canonical_digest().expect("role digest"),
            second.canonical_digest().expect("role digest")
        );
    }
}
