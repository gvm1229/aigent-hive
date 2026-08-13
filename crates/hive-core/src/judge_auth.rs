//! Ed25519 authentication for judge assignments, verdicts, and approvals.
//!
//! Hive verifies detached signatures only. Public keys come exclusively from
//! an external, agent-write-denied trust root. Private-key generation, loading,
//! custody, and signing are outside the product boundary.

use crate::judge::{
    aggregate_verdicts, AggregateOutcome, AggregateStatus, HumanApproval, JudgeAssignment,
    JudgePackage, JudgeVerdict, RiskTier,
};
use crate::{sha256_digest, validate_json_schema};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const TRUST_ROOT_SCHEMA: &str = include_str!("../../../schemas/judge-trust-root.schema.json");
const ATTESTATION_SCHEMA: &str = include_str!("../../../schemas/judge-attestation.schema.json");
const SIGNATURE_DOMAIN: &[u8] = b"AIGENT-HIVE\0JUDGE-ATTESTATION\0V1\0";
const ED25519_PREFIX: &str = "ed25519:";

/// Authorized use of one trusted public key.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyPurpose {
    JudgeAssignment,
    JudgeVerdict,
    JudgeApproval,
    /// External signed host-model catalog.
    HostModelCatalog,
}

/// Revocation state recorded by the external authority.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyStatus {
    Active,
    Revoked,
}

/// One public verification key enrolled by the external authority.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedJudgeKey {
    pub key_id: String,
    pub principal_id: String,
    pub purpose: KeyPurpose,
    pub algorithm: String,
    pub public_key: String,
    pub status: KeyStatus,
    pub valid_from: String,
    pub valid_until: String,
}

/// Public-key trust root loaded from an external protected TOML file.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JudgeTrustRoot {
    pub schema_version: u32,
    pub trust_root_id: String,
    pub revision: u64,
    pub issued_at: String,
    pub keys: Vec<TrustedJudgeKey>,
    pub root_digest: String,
}

#[derive(Serialize)]
struct DigestExcludedTrustRoot<'a> {
    schema_version: u32,
    trust_root_id: &'a str,
    revision: u64,
    issued_at: &'a str,
    keys: &'a [TrustedJudgeKey],
}

impl JudgeTrustRoot {
    /// Validate schema, digest, key encoding, lifecycle bounds, and global key
    /// uniqueness.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed or unsafe trust-root data.
    pub fn validate(&self) -> Result<(), JudgeAuthError> {
        let value = serde_json::to_value(self)
            .map_err(|error| JudgeAuthError::Malformed(error.to_string()))?;
        validate_json_schema(TRUST_ROOT_SCHEMA, &value, "judge trust root")
            .map_err(JudgeAuthError::Schema)?;
        if self.computed_digest()? != self.root_digest {
            return Err(JudgeAuthError::TrustRootDigestMismatch);
        }

        let mut key_ids = HashSet::new();
        let mut public_keys = HashSet::new();
        for key in &self.keys {
            if key.algorithm != "ed25519"
                || key.valid_from > key.valid_until
                || !key_ids.insert(key.key_id.as_str())
            {
                return Err(JudgeAuthError::InvalidTrustRoot);
            }
            let public_key = decode_public_key(&key.public_key)?;
            VerifyingKey::from_bytes(&public_key).map_err(|_| JudgeAuthError::InvalidTrustRoot)?;
            if !public_keys.insert(public_key) {
                return Err(JudgeAuthError::DuplicatePublicKey);
            }
        }
        Ok(())
    }

    /// Compute the RFC 8785/JCS digest over every root field except
    /// `root_digest`.
    ///
    /// # Errors
    ///
    /// Returns an error when canonical serialization fails.
    pub fn computed_digest(&self) -> Result<String, JudgeAuthError> {
        jcs_digest(&DigestExcludedTrustRoot {
            schema_version: self.schema_version,
            trust_root_id: &self.trust_root_id,
            revision: self.revision,
            issued_at: &self.issued_at,
            keys: &self.keys,
        })
    }

    fn key(&self, key_id: &str) -> Option<&TrustedJudgeKey> {
        self.keys.iter().find(|key| key.key_id == key_id)
    }
}

/// Kind of judge artifact authenticated by a detached signature.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Assignment,
    Verdict,
    Approval,
    /// Signed host-model catalog used for custom-agent recommendations.
    #[serde(rename = "host-model-catalog")]
    HostModelCatalog,
}

impl ArtifactKind {
    const fn required_purpose(self) -> KeyPurpose {
        match self {
            Self::Assignment => KeyPurpose::JudgeAssignment,
            Self::Verdict => KeyPurpose::JudgeVerdict,
            Self::Approval => KeyPurpose::JudgeApproval,
            Self::HostModelCatalog => KeyPurpose::HostModelCatalog,
        }
    }
}

/// Detached signature metadata. It never embeds or self-selects a public key.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JudgeAttestation {
    pub schema_version: u32,
    pub trust_root_id: String,
    pub artifact_kind: ArtifactKind,
    pub artifact_digest: String,
    pub principal_id: String,
    pub key_id: String,
    pub signature: String,
}

#[derive(Serialize)]
struct SignaturePayload<'a> {
    schema_version: u32,
    trust_root_id: &'a str,
    artifact_kind: ArtifactKind,
    artifact_digest: &'a str,
    principal_id: &'a str,
    key_id: &'a str,
}

/// Verified signer material used to enforce distinct judge keys.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VerifiedAttestation {
    pub public_key: [u8; 32],
}

/// Aggregate result plus whether every identity needed for that result was
/// authenticated against the external trust root.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AuthenticatedOutcome {
    pub aggregate: AggregateOutcome,
    pub authenticated: bool,
}

/// Complete read-only input for authenticated quorum evaluation.
pub struct AuthenticatedQuorumInput<'a> {
    pub trust_root: &'a JudgeTrustRoot,
    pub package: &'a JudgePackage,
    pub assignment: &'a JudgeAssignment,
    pub assignment_attestation: &'a JudgeAttestation,
    pub normal_requested: bool,
    pub verdicts: &'a [JudgeVerdict],
    pub verdict_attestations: &'a [Option<JudgeAttestation>],
    pub human_approval: Option<&'a HumanApproval>,
    pub approval_attestation: Option<&'a JudgeAttestation>,
}

impl JudgeAttestation {
    /// Parse one closed-schema detached attestation.
    ///
    /// # Errors
    ///
    /// Rejects malformed JSON and unknown fields before cryptographic use.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, JudgeAuthError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| JudgeAuthError::Malformed(error.to_string()))?;
        validate_json_schema(ATTESTATION_SCHEMA, &value, "judge attestation")
            .map_err(JudgeAuthError::Schema)?;
        serde_json::from_value(value).map_err(|error| JudgeAuthError::Malformed(error.to_string()))
    }

    /// Verify the exact artifact, signer role and identity, key lifecycle, and
    /// strict Ed25519 signature.
    ///
    /// `signed_at` is taken from the signed artifact, not from caller input.
    /// Revoked keys are rejected regardless of that timestamp.
    ///
    /// # Errors
    ///
    /// Returns a non-sensitive authentication failure when any binding or
    /// signature check fails.
    pub fn verify<T: Serialize>(
        &self,
        trust_root: &JudgeTrustRoot,
        expected_kind: ArtifactKind,
        artifact: &T,
        expected_principal: &str,
        signed_at: &str,
    ) -> Result<VerifiedAttestation, JudgeAuthError> {
        trust_root.validate()?;
        let value = serde_json::to_value(self)
            .map_err(|error| JudgeAuthError::Malformed(error.to_string()))?;
        validate_json_schema(ATTESTATION_SCHEMA, &value, "judge attestation")
            .map_err(JudgeAuthError::Schema)?;
        let artifact_digest = jcs_digest(artifact)?;
        if self.trust_root_id != trust_root.trust_root_id
            || self.artifact_kind != expected_kind
            || self.artifact_digest != artifact_digest
            || self.principal_id != expected_principal
        {
            return Err(JudgeAuthError::AuthenticationFailed);
        }
        let key = trust_root
            .key(&self.key_id)
            .ok_or(JudgeAuthError::AuthenticationFailed)?;
        if key.status != KeyStatus::Active
            || key.purpose != expected_kind.required_purpose()
            || key.principal_id != expected_principal
            || signed_at < key.valid_from.as_str()
            || signed_at > key.valid_until.as_str()
        {
            return Err(JudgeAuthError::AuthenticationFailed);
        }

        let public_key =
            decode_public_key(&key.public_key).map_err(|_| JudgeAuthError::AuthenticationFailed)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| JudgeAuthError::AuthenticationFailed)?;
        let signature_bytes =
            decode_signature(&self.signature).map_err(|_| JudgeAuthError::AuthenticationFailed)?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|_| JudgeAuthError::AuthenticationFailed)?;
        verifying_key
            .verify_strict(&self.signing_message()?, &signature)
            .map_err(|_| JudgeAuthError::AuthenticationFailed)?;
        Ok(VerifiedAttestation { public_key })
    }

    /// Return the exact domain-separated bytes external signers must sign.
    ///
    /// # Errors
    ///
    /// Returns an error only when JCS serialization fails.
    pub fn signing_message(&self) -> Result<Vec<u8>, JudgeAuthError> {
        let payload = SignaturePayload {
            schema_version: self.schema_version,
            trust_root_id: &self.trust_root_id,
            artifact_kind: self.artifact_kind,
            artifact_digest: &self.artifact_digest,
            principal_id: &self.principal_id,
            key_id: &self.key_id,
        };
        let canonical = serde_json_canonicalizer::to_vec(&payload)
            .map_err(|error| JudgeAuthError::Malformed(error.to_string()))?;
        let mut message = Vec::with_capacity(SIGNATURE_DOMAIN.len() + canonical.len());
        message.extend_from_slice(SIGNATURE_DOMAIN);
        message.extend_from_slice(&canonical);
        Ok(message)
    }
}

/// Authenticate the owner-sealed roster, each eligible verdict, and the
/// critical human approval before applying the existing deterministic quorum.
///
/// Invalid, missing, duplicated, or key-reused attestations can never count as
/// eligible verdicts. A legacy unsigned chain therefore cannot return PASS.
#[must_use]
pub fn aggregate_authenticated_verdicts(
    input: &AuthenticatedQuorumInput<'_>,
) -> AuthenticatedOutcome {
    let indeterminate = |excluded_count| AuthenticatedOutcome {
        aggregate: AggregateOutcome {
            status: AggregateStatus::Indeterminate,
            eligible_count: 0,
            pass_count: 0,
            indeterminate_count: 0,
            excluded_count,
            approval_valid: false,
        },
        authenticated: false,
    };
    if input.package.validate().is_err()
        || input
            .assignment
            .validate_for_package(input.package)
            .is_err()
        || input.verdicts.len() != input.verdict_attestations.len()
    {
        return indeterminate(input.verdicts.len());
    }
    let Ok(owner) = input.assignment_attestation.verify(
        input.trust_root,
        ArtifactKind::Assignment,
        input.assignment,
        &input.assignment.resolved_owner_id,
        &input.assignment.created_at,
    ) else {
        return indeterminate(input.verdicts.len());
    };

    let candidates = authenticate_verdicts(input, owner.public_key);
    let mut key_counts = std::collections::HashMap::<[u8; 32], usize>::new();
    for (_, key) in &candidates {
        *key_counts.entry(*key).or_default() += 1;
    }
    let authenticated_candidates = candidates
        .into_iter()
        .filter(|(_, key)| key_counts.get(key) == Some(&1))
        .collect::<Vec<_>>();
    let authenticated_keys = authenticated_candidates
        .iter()
        .map(|(_, key)| *key)
        .collect::<HashSet<_>>();
    let authenticated_verdicts = authenticated_candidates
        .into_iter()
        .map(|(verdict, _)| verdict)
        .collect::<Vec<_>>();

    let authenticated_approval =
        authenticate_approval(input, owner.public_key, &authenticated_keys);

    let mut aggregate = aggregate_verdicts(
        input.package,
        input.assignment,
        input.normal_requested,
        authenticated_approval,
        &authenticated_verdicts,
    );
    let authentication_excluded = input
        .verdicts
        .len()
        .saturating_sub(authenticated_verdicts.len());
    aggregate.excluded_count = aggregate
        .excluded_count
        .saturating_add(authentication_excluded);
    let expected = match input.package.risk_tier {
        RiskTier::Normal => usize::from(input.normal_requested),
        RiskTier::Elevated | RiskTier::Critical => 3,
    };
    let all_verdicts_authenticated = authenticated_verdicts.len() == expected;
    let approval_authenticated =
        input.package.risk_tier != RiskTier::Critical || authenticated_approval.is_some();
    let approval_required_for_result =
        input.package.risk_tier == RiskTier::Critical && aggregate.status != AggregateStatus::Fail;
    AuthenticatedOutcome {
        authenticated: all_verdicts_authenticated
            && (!approval_required_for_result || approval_authenticated),
        aggregate,
    }
}

fn authenticate_verdicts(
    input: &AuthenticatedQuorumInput<'_>,
    owner_public_key: [u8; 32],
) -> Vec<(JudgeVerdict, [u8; 32])> {
    input
        .verdicts
        .iter()
        .zip(input.verdict_attestations)
        .filter_map(|(verdict, attestation)| {
            let verified = attestation
                .as_ref()?
                .verify(
                    input.trust_root,
                    ArtifactKind::Verdict,
                    verdict,
                    &verdict.judge_instance_id,
                    &verdict.created_at,
                )
                .ok()?;
            (verified.public_key != owner_public_key)
                .then(|| (verdict.clone(), verified.public_key))
        })
        .collect()
}

fn authenticate_approval<'a>(
    input: &AuthenticatedQuorumInput<'a>,
    owner_public_key: [u8; 32],
    judge_public_keys: &HashSet<[u8; 32]>,
) -> Option<&'a HumanApproval> {
    input
        .human_approval
        .zip(input.approval_attestation)
        .filter(|(approval, _)| {
            approval.approver_id != input.assignment.resolved_owner_id
                && input
                    .assignment
                    .slots
                    .iter()
                    .all(|slot| slot.judge_instance_id != approval.approver_id)
        })
        .and_then(|(approval, attestation)| {
            let verified = attestation
                .verify(
                    input.trust_root,
                    ArtifactKind::Approval,
                    approval,
                    &approval.approver_id,
                    &approval.created_at,
                )
                .ok()?;
            (verified.public_key != owner_public_key
                && !judge_public_keys.contains(&verified.public_key))
            .then_some(approval)
        })
}

fn jcs_digest(value: &impl Serialize) -> Result<String, JudgeAuthError> {
    let canonical = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| JudgeAuthError::Malformed(error.to_string()))?;
    Ok(sha256_digest(&canonical))
}

fn decode_public_key(value: &str) -> Result<[u8; 32], JudgeAuthError> {
    decode_prefixed_hex::<32>(value)
}

fn decode_signature(value: &str) -> Result<[u8; 64], JudgeAuthError> {
    decode_prefixed_hex::<64>(value)
}

fn decode_prefixed_hex<const N: usize>(value: &str) -> Result<[u8; N], JudgeAuthError> {
    let encoded = value
        .strip_prefix(ED25519_PREFIX)
        .ok_or(JudgeAuthError::InvalidEncoding)?;
    if encoded.len() != N * 2
        || !encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(JudgeAuthError::InvalidEncoding);
    }
    let mut decoded = [0_u8; N];
    for (index, pair) in encoded.as_bytes().chunks_exact(2).enumerate() {
        decoded[index] = hex_nibble(pair[0])? << 4 | hex_nibble(pair[1])?;
    }
    Ok(decoded)
}

const fn hex_nibble(byte: u8) -> Result<u8, JudgeAuthError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(JudgeAuthError::InvalidEncoding),
    }
}

/// Non-sensitive Ed25519 trust and attestation errors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JudgeAuthError {
    Malformed(String),
    Schema(String),
    InvalidEncoding,
    InvalidTrustRoot,
    DuplicatePublicKey,
    TrustRootDigestMismatch,
    AuthenticationFailed,
}

impl Display for JudgeAuthError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(message) | Self::Schema(message) => formatter.write_str(message),
            Self::InvalidEncoding => {
                formatter.write_str("Ed25519 value uses an invalid canonical encoding")
            }
            Self::InvalidTrustRoot => {
                formatter.write_str("judge trust root is semantically invalid")
            }
            Self::DuplicatePublicKey => {
                formatter.write_str("judge trust root reuses public key material")
            }
            Self::TrustRootDigestMismatch => {
                formatter.write_str("judge trust root digest does not match its JCS payload")
            }
            Self::AuthenticationFailed => {
                formatter.write_str("judge artifact authentication failed")
            }
        }
    }
}

impl Error for JudgeAuthError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judge::{HumanApproval, JudgeAssignment, JudgePackage, JudgeVerdict};
    use serde_json::json;

    fn rfc_8032_root() -> JudgeTrustRoot {
        let mut root = JudgeTrustRoot {
            schema_version: 1,
            trust_root_id: "rfc-8032-root".to_owned(),
            revision: 1,
            issued_at: "2026-07-24T00:00:00Z".to_owned(),
            keys: vec![TrustedJudgeKey {
                key_id: "rfc-8032-key-1".to_owned(),
                principal_id: "owner-1".to_owned(),
                purpose: KeyPurpose::JudgeAssignment,
                algorithm: "ed25519".to_owned(),
                public_key: concat!(
                    "ed25519:",
                    "d75a980182b10ab7d54bfed3c964073a",
                    "0ee172f3daa62325af021a68f707511a"
                )
                .to_owned(),
                status: KeyStatus::Active,
                valid_from: "2026-01-01T00:00:00Z".to_owned(),
                valid_until: "2026-12-31T23:59:59Z".to_owned(),
            }],
            root_digest: format!("sha256:{}", "0".repeat(64)),
        };
        root.root_digest = root.computed_digest().expect("root digest");
        root
    }

    #[test]
    fn trust_root_rejects_duplicate_public_key_material() {
        let mut root = rfc_8032_root();
        let mut duplicate = root.keys[0].clone();
        duplicate.key_id = "another-id".to_owned();
        duplicate.principal_id = "another-principal".to_owned();
        root.keys.push(duplicate);
        root.root_digest = root.computed_digest().expect("root digest");
        assert_eq!(root.validate(), Err(JudgeAuthError::DuplicatePublicKey));
    }

    #[test]
    fn trust_root_rejects_impossible_calendar_timestamps() {
        let mut root = rfc_8032_root();
        root.keys[0].valid_until = "2026-99-31T23:59:59Z".to_owned();
        root.root_digest = root.computed_digest().expect("root digest");
        assert!(matches!(root.validate(), Err(JudgeAuthError::Schema(_))));
    }

    #[test]
    fn exact_signature_domain_is_stable() {
        let attestation = JudgeAttestation {
            schema_version: 1,
            trust_root_id: "rfc-8032-root".to_owned(),
            artifact_kind: ArtifactKind::Assignment,
            artifact_digest: format!("sha256:{}", "a".repeat(64)),
            principal_id: "owner-1".to_owned(),
            key_id: "rfc-8032-key-1".to_owned(),
            signature: format!("ed25519:{}", "0".repeat(128)),
        };
        assert_eq!(
            String::from_utf8(attestation.signing_message().expect("message"))
                .expect("ASCII message"),
            concat!(
                "AIGENT-HIVE\0JUDGE-ATTESTATION\0V1\0",
                "{\"artifact_digest\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"artifact_kind\":\"assignment\",",
                "\"key_id\":\"rfc-8032-key-1\",\"principal_id\":\"owner-1\",",
                "\"schema_version\":1,\"trust_root_id\":\"rfc-8032-root\"}"
            )
        );
    }

    #[test]
    fn strict_verification_accepts_external_known_answer() {
        let artifact = json!({"created_at": "2026-07-24T00:00:01Z", "value": 1});
        let mut root = JudgeTrustRoot {
            schema_version: 1,
            trust_root_id: "fixture-root".to_owned(),
            revision: 1,
            issued_at: "2026-07-24T00:00:00Z".to_owned(),
            keys: vec![TrustedJudgeKey {
                key_id: "fixture-key-1".to_owned(),
                principal_id: "owner-1".to_owned(),
                purpose: KeyPurpose::JudgeAssignment,
                algorithm: "ed25519".to_owned(),
                public_key: concat!(
                    "ed25519:",
                    "8b54c08468bffa860fd90acd05fd2440",
                    "cd7f0286d87807e5762ef928cc90ee87"
                )
                .to_owned(),
                status: KeyStatus::Active,
                valid_from: "2026-01-01T00:00:00Z".to_owned(),
                valid_until: "2026-12-31T23:59:59Z".to_owned(),
            }],
            root_digest: format!("sha256:{}", "0".repeat(64)),
        };
        root.root_digest = root.computed_digest().expect("root digest");
        let attestation = JudgeAttestation {
            schema_version: 1,
            trust_root_id: root.trust_root_id.clone(),
            artifact_kind: ArtifactKind::Assignment,
            artifact_digest: concat!(
                "sha256:",
                "4bea7868c13ef8ab4230f01f69237924",
                "a3037b778b4675c7f8fae9da4654a66e"
            )
            .to_owned(),
            principal_id: "owner-1".to_owned(),
            key_id: "fixture-key-1".to_owned(),
            signature: concat!(
                "ed25519:",
                "4539d9551d71b984a5c7c5c8826473d5bb70d858f81cdb22e44dce31e5cf7fab",
                "18354ecd734e9590e04ce0b0969d8fc802c8b626cf59193a5321a5b74e060008"
            )
            .to_owned(),
        };
        assert!(attestation
            .verify(
                &root,
                ArtifactKind::Assignment,
                &artifact,
                "owner-1",
                "2026-07-24T00:00:01Z"
            )
            .is_ok());
    }

    #[test]
    fn artifact_tamper_role_mismatch_and_revocation_fail_closed() {
        let root = rfc_8032_root();
        let artifact = json!({"created_at": "2026-07-24T00:00:01Z", "value": 1});
        let mut attestation = JudgeAttestation {
            schema_version: 1,
            trust_root_id: root.trust_root_id.clone(),
            artifact_kind: ArtifactKind::Assignment,
            artifact_digest: jcs_digest(&artifact).expect("artifact digest"),
            principal_id: "owner-1".to_owned(),
            key_id: "rfc-8032-key-1".to_owned(),
            signature: format!("ed25519:{}", "0".repeat(128)),
        };
        assert_eq!(
            attestation.verify(
                &root,
                ArtifactKind::Verdict,
                &artifact,
                "owner-1",
                "2026-07-24T00:00:01Z"
            ),
            Err(JudgeAuthError::AuthenticationFailed)
        );
        attestation.artifact_digest = jcs_digest(&json!({
            "created_at": "2026-07-24T00:00:01Z",
            "value": 2
        }))
        .expect("artifact digest");
        assert_eq!(
            attestation.verify(
                &root,
                ArtifactKind::Assignment,
                &artifact,
                "owner-1",
                "2026-07-24T00:00:01Z"
            ),
            Err(JudgeAuthError::AuthenticationFailed)
        );
        let mut revoked = root;
        revoked.keys[0].status = KeyStatus::Revoked;
        revoked.root_digest = revoked.computed_digest().expect("root digest");
        assert_eq!(
            attestation.verify(
                &revoked,
                ArtifactKind::Assignment,
                &artifact,
                "owner-1",
                "2026-07-24T00:00:01Z"
            ),
            Err(JudgeAuthError::AuthenticationFailed)
        );
    }

    fn critical_fixture_root() -> JudgeTrustRoot {
        let values = [
            (
                "owner-key",
                "omx-run-42",
                KeyPurpose::JudgeAssignment,
                "4abd35193ca765d97d6e11b0d562279150d22e425aaee3c5c2a61d6b512ba9bf",
            ),
            (
                "judge-key-1",
                "c-judge-instance-1",
                KeyPurpose::JudgeVerdict,
                "822666efcece28d20d6dc5d290de9f78ffd69296d24fc0078f702bdc525834cb",
            ),
            (
                "judge-key-2",
                "c-judge-instance-2",
                KeyPurpose::JudgeVerdict,
                "4398543232d5bda0e2d7a83600ccd65627f347bdb33973f5b209e1d74938735a",
            ),
            (
                "judge-key-3",
                "c-judge-instance-3",
                KeyPurpose::JudgeVerdict,
                "74c9b7e060468861274e48879b629508230a805773ca99b4b93278b595799fb9",
            ),
            (
                "human-key",
                "human-security-reviewer",
                KeyPurpose::JudgeApproval,
                "67b94d4b0475e09950a67be97ac3375abb5a301ac500d01b9fa58fc2dea4f9cb",
            ),
        ];
        let mut root = JudgeTrustRoot {
            schema_version: 1,
            trust_root_id: "phase5-fixture-root".to_owned(),
            revision: 1,
            issued_at: "2026-07-24T00:00:00Z".to_owned(),
            keys: values
                .into_iter()
                .map(
                    |(key_id, principal_id, purpose, public_key)| TrustedJudgeKey {
                        key_id: key_id.to_owned(),
                        principal_id: principal_id.to_owned(),
                        purpose,
                        algorithm: "ed25519".to_owned(),
                        public_key: format!("ed25519:{public_key}"),
                        status: KeyStatus::Active,
                        valid_from: "2026-01-01T00:00:00Z".to_owned(),
                        valid_until: "2026-12-31T23:59:59Z".to_owned(),
                    },
                )
                .collect(),
            root_digest: format!("sha256:{}", "0".repeat(64)),
        };
        root.root_digest = root.computed_digest().expect("root digest");
        root
    }

    #[test]
    fn authenticated_critical_fixture_requires_all_distinct_signatures() {
        let package = JudgePackage::parse_json(include_bytes!(
            "../../../tests/fixtures/phase5/judge/package-critical.json"
        ))
        .expect("package");
        let assignment = JudgeAssignment::parse_json(
            include_bytes!("../../../tests/fixtures/phase5/judge/assignment-critical.json"),
            &package,
        )
        .expect("assignment");
        let assignment_attestation = JudgeAttestation::parse_json(include_bytes!(
            "../../../tests/fixtures/phase5/judge/assignment-critical-attestation.json"
        ))
        .expect("assignment attestation");
        let verdicts = [
            include_bytes!("../../../tests/fixtures/phase5/judge/verdict-critical-pass-a.json")
                .as_slice(),
            include_bytes!("../../../tests/fixtures/phase5/judge/verdict-critical-pass-b.json")
                .as_slice(),
            include_bytes!("../../../tests/fixtures/phase5/judge/verdict-critical-pass-c.json")
                .as_slice(),
        ]
        .into_iter()
        .map(|bytes| JudgeVerdict::parse_json(bytes).expect("verdict"))
        .collect::<Vec<_>>();
        let attestations = [
            include_bytes!(
                "../../../tests/fixtures/phase5/judge/verdict-critical-pass-a-attestation.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/fixtures/phase5/judge/verdict-critical-pass-b-attestation.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/fixtures/phase5/judge/verdict-critical-pass-c-attestation.json"
            )
            .as_slice(),
        ]
        .into_iter()
        .map(|bytes| Some(JudgeAttestation::parse_json(bytes).expect("attestation")))
        .collect::<Vec<_>>();
        let approval = HumanApproval::parse_json(include_bytes!(
            "../../../tests/fixtures/phase5/judge/approval-critical.json"
        ))
        .expect("approval");
        let approval_attestation = JudgeAttestation::parse_json(include_bytes!(
            "../../../tests/fixtures/phase5/judge/approval-critical-attestation.json"
        ))
        .expect("approval attestation");
        let root = critical_fixture_root();

        let outcome = aggregate_authenticated_verdicts(&AuthenticatedQuorumInput {
            trust_root: &root,
            package: &package,
            assignment: &assignment,
            assignment_attestation: &assignment_attestation,
            normal_requested: true,
            verdicts: &verdicts,
            verdict_attestations: &attestations,
            human_approval: Some(&approval),
            approval_attestation: Some(&approval_attestation),
        });
        assert_eq!(outcome.aggregate.status, AggregateStatus::Pass);
        assert!(outcome.authenticated);
        assert!(outcome.aggregate.approval_valid);

        let mut tampered = attestations;
        tampered[1]
            .as_mut()
            .expect("attestation")
            .signature
            .replace_range(8..9, "f");
        let rejected = aggregate_authenticated_verdicts(&AuthenticatedQuorumInput {
            trust_root: &root,
            package: &package,
            assignment: &assignment,
            assignment_attestation: &assignment_attestation,
            normal_requested: true,
            verdicts: &verdicts,
            verdict_attestations: &tampered,
            human_approval: Some(&approval),
            approval_attestation: Some(&approval_attestation),
        });
        assert_ne!(rejected.aggregate.status, AggregateStatus::Pass);
        assert!(!rejected.authenticated);
        assert_eq!(rejected.aggregate.excluded_count, 1);
    }
}
