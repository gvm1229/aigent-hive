//! Role-to-role file inbox contract for message mediation (OV160-002).
//!
//! This module owns deterministic data only: envelopes, receipts, the lifecycle state table and
//! the project-relative layout under `.hive/mediation/`. It never performs I/O, launches a host
//! process or calls a model provider.
//!
//! Delivery is asynchronous by contract. Every operation that uses these types returns the
//! current state immediately (`queued` after a send); nothing waits for a result. A result
//! arrives later, either through an official host hook event (OV160-003) or through a host
//! `/loop` that reads `results/`. A claimed message whose receipt is missing after the claim
//! lease becomes `dispatch-uncertain` and is never dispatched again automatically.
//!
//! Message bodies and results come from other sessions. They are untrusted data, never
//! instructions; every envelope and receipt carries `trust = "untrusted-data"`.

use crate::orchestration::{valid_digest, valid_id};
use serde::{Deserialize, Serialize};

/// Current envelope, receipt and record schema version.
pub const MESSAGE_SCHEMA_VERSION: u32 = 1;
/// Largest accepted message body. Large enough for a task brief or review note, small enough
/// that a reader can show a whole inbox without paging.
pub const MAX_BODY_BYTES: usize = 64 * 1024;
/// Largest accepted acknowledgement result.
pub const MAX_RESULT_BYTES: usize = 256 * 1024;
/// Largest accepted caller-supplied idempotency key.
pub const MAX_IDEMPOTENCY_KEY_BYTES: usize = 256;
/// Default claim lease; a claim older than this without a receipt is `dispatch-uncertain`.
pub const DEFAULT_CLAIM_LEASE_SECONDS: u64 = 15 * 60;
/// Default time an unclaimed message stays deliverable before it is `expired`.
pub const DEFAULT_MESSAGE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
/// Longest accepted claim lease or message lifetime.
pub const MAX_DURATION_SECONDS: u64 = 30 * 24 * 60 * 60;
/// Trust label carried by every envelope, receipt and rendered body.
pub const UNTRUSTED_DATA: &str = "untrusted-data";
/// Project-relative root of the file inbox.
pub const MEDIATION_ROOT: &str = ".hive/mediation";

/// Lifecycle state of one message. Persisted files are immutable; the state is derived.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageState {
    /// Envelope published, nobody holds the consumption right yet.
    Queued,
    /// One consumer holds the claim inside its lease.
    Claimed,
    /// A receipt binds the claim and the result.
    Acknowledged,
    /// A claim exists but its receipt is missing after the lease or a claimant restart.
    DispatchUncertain,
    /// Nobody claimed the message inside its lifetime.
    Expired,
    /// A tampered envelope, unresolvable sender, invalid receipt or late result.
    Quarantined,
    /// Claimed work inside an active stop (OV160-005). Hive did not interrupt it; it waits for
    /// the host's official cancel receipt or the end of its lease.
    CancelRequested,
    /// A host official cancel receipt binds the claim. Any later result is quarantined.
    Cancelled,
}

impl MessageState {
    /// Stable kebab-case name used in JSON and human output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Claimed => "claimed",
            Self::Acknowledged => "acknowledged",
            Self::DispatchUncertain => "dispatch-uncertain",
            Self::Expired => "expired",
            Self::Quarantined => "quarantined",
            Self::CancelRequested => "cancel-requested",
            Self::Cancelled => "cancelled",
        }
    }

    /// No later state can follow.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Acknowledged | Self::Expired | Self::Quarantined | Self::Cancelled
        )
    }
}

/// Closed transition table. `None` is "no envelope yet".
///
/// `dispatch-uncertain` never returns to `queued` or `claimed`: a retry is an explicit new
/// message with a new idempotency key, never an automatic re-dispatch. `cancel-requested`
/// returns to `claimed` only through an explicit stop clear; `cancelled` needs a host receipt.
#[must_use]
pub const fn allowed_transition(from: Option<MessageState>, to: MessageState) -> bool {
    use MessageState as S;
    matches!(
        (from, to),
        (None, S::Queued)
            | (Some(S::Queued), S::Claimed | S::Expired | S::Quarantined)
            | (
                Some(S::Claimed),
                S::Acknowledged | S::DispatchUncertain | S::Quarantined | S::CancelRequested
            )
            | (
                Some(S::CancelRequested),
                S::Claimed | S::Cancelled | S::DispatchUncertain | S::Quarantined
            )
            | (Some(S::DispatchUncertain), S::Quarantined)
    )
}

/// Sender identity resolved from the roster at send time, never taken from the caller alone.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MessageSender {
    pub role: String,
    pub god: Option<String>,
    pub provider: String,
    pub model: String,
    pub effort: String,
}

/// Immutable inbox envelope. The body lives in a sibling file bound by digest and size.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MessageEnvelope {
    pub schema_version: u32,
    pub message_id: String,
    pub sequence: u64,
    pub recipient_role: String,
    pub recipient_task: Option<String>,
    pub sender: MessageSender,
    pub idempotency_digest: String,
    pub body_file: String,
    pub body_digest: String,
    pub body_bytes: u64,
    pub created_at: u64,
    pub expires_at: u64,
    pub trust: String,
    /// Canonical digest of every other field (computed with this field empty).
    pub envelope_digest: String,
}

impl MessageEnvelope {
    /// Canonical digest of the envelope with `envelope_digest` cleared.
    /// # Errors
    /// Returns a serialization error.
    pub fn computed_digest(&self) -> Result<String, String> {
        let mut unsealed = self.clone();
        unsealed.envelope_digest = String::new();
        canonical_digest(&unsealed)
    }

    /// Fill `envelope_digest` and validate the result.
    /// # Errors
    /// Rejects invalid fields.
    pub fn seal(mut self) -> Result<Self, String> {
        self.envelope_digest = self.computed_digest()?;
        self.validate()?;
        Ok(self)
    }

    /// Check every field, the exact layout path and the self digest.
    /// # Errors
    /// Rejects unsafe ids, wrong paths, oversize bodies, a wrong trust label or a digest mismatch.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != MESSAGE_SCHEMA_VERSION
            || self.trust != UNTRUSTED_DATA
            || !valid_id(&self.message_id)
            || self.message_id != message_id_for(&self.idempotency_digest)
            || self.sequence == 0
            || !valid_id(&self.recipient_role)
            || self
                .recipient_task
                .as_deref()
                .is_some_and(|task| !valid_id(task))
            || !valid_id(&self.sender.role)
            || self.sender.god.as_deref().is_some_and(|god| !valid_id(god))
            || [
                &self.sender.provider,
                &self.sender.model,
                &self.sender.effort,
            ]
            .iter()
            .any(|value| !valid_id(value))
            || !valid_digest(&self.idempotency_digest)
            || !valid_digest(&self.body_digest)
            || self.body_bytes == 0
            || self.body_bytes > MAX_BODY_BYTES as u64
            || self.expires_at <= self.created_at
            || self.expires_at - self.created_at > MAX_DURATION_SECONDS
        {
            return Err("invalid message envelope".into());
        }
        if self.body_file != body_path(&self.recipient_role, &self.message_id) {
            return Err("message body path is outside its exact inbox".into());
        }
        if self.envelope_digest != self.computed_digest()? {
            return Err("message envelope digest does not match its contents".into());
        }
        Ok(())
    }

    /// Check that `body` is exactly the bound body.
    /// # Errors
    /// Rejects a size or digest mismatch.
    pub fn verify_body(&self, body: &[u8]) -> Result<(), String> {
        if body.len() as u64 != self.body_bytes || crate::sha256_digest(body) != self.body_digest {
            return Err("message body does not match its envelope".into());
        }
        Ok(())
    }
}

/// Deterministic idempotency record published before the envelope (no-clobber).
///
/// Identical requests produce identical bytes, so concurrent duplicates converge on one record;
/// a different body under the same key is refused.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MessageKeyRecord {
    pub schema_version: u32,
    pub idempotency_digest: String,
    pub message_id: String,
    pub recipient_role: String,
    pub recipient_task: Option<String>,
    pub sender_role: String,
    pub body_digest: String,
}

impl MessageKeyRecord {
    /// # Errors
    /// Rejects unsafe ids or digests.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != MESSAGE_SCHEMA_VERSION
            || !valid_digest(&self.idempotency_digest)
            || self.message_id != message_id_for(&self.idempotency_digest)
            || !valid_id(&self.recipient_role)
            || self
                .recipient_task
                .as_deref()
                .is_some_and(|task| !valid_id(task))
            || !valid_id(&self.sender_role)
            || !valid_digest(&self.body_digest)
        {
            return Err("invalid message key record".into());
        }
        Ok(())
    }

    /// Same request: same recipient, task, sender and body.
    #[must_use]
    pub fn same_request(&self, other: &Self) -> bool {
        self == other
    }

    /// The record must describe exactly this envelope.
    #[must_use]
    pub fn binds(&self, envelope: &MessageEnvelope) -> bool {
        self.idempotency_digest == envelope.idempotency_digest
            && self.message_id == envelope.message_id
            && self.recipient_role == envelope.recipient_role
            && self.recipient_task == envelope.recipient_task
            && self.sender_role == envelope.sender.role
            && self.body_digest == envelope.body_digest
    }
}

/// One reserved sequence number (no-clobber). Gaps are allowed; numbers are never reused.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceReservation {
    pub schema_version: u32,
    pub sequence: u64,
    pub message_id: String,
}

/// The single consumption right, published no-clobber; the first publisher wins.
///
/// `claim_token` is a fresh random fencing value, so two concurrent claims never produce
/// identical bytes and exactly one of them can win the no-clobber publication.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRecord {
    pub schema_version: u32,
    pub message_id: String,
    pub envelope_digest: String,
    pub claimant_role: String,
    pub claim_token: String,
    pub claimed_at: u64,
    pub lease_until: u64,
}

impl ClaimRecord {
    /// # Errors
    /// Rejects unsafe fields or a lease outside the accepted range.
    pub fn validate(&self, envelope: &MessageEnvelope) -> Result<(), String> {
        if self.schema_version != MESSAGE_SCHEMA_VERSION
            || self.message_id != envelope.message_id
            || self.envelope_digest != envelope.envelope_digest
            || self.claimant_role != envelope.recipient_role
            || !valid_id(&self.claim_token)
            || self.lease_until <= self.claimed_at
            || self.lease_until - self.claimed_at > MAX_DURATION_SECONDS
        {
            return Err("claim does not bind this message".into());
        }
        Ok(())
    }

    /// Canonical digest bound by the receipt.
    /// # Errors
    /// Returns a serialization error.
    pub fn digest(&self) -> Result<String, String> {
        canonical_digest(self)
    }
}

/// Receipt binding the message, its claim and the result bytes.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MessageReceipt {
    pub schema_version: u32,
    pub message_id: String,
    pub envelope_digest: String,
    pub claim_digest: String,
    pub result_file: String,
    pub result_digest: String,
    pub result_bytes: u64,
    pub acknowledged_at: u64,
    pub trust: String,
}

impl MessageReceipt {
    /// # Errors
    /// Rejects a receipt that does not bind this exact envelope, claim and result path.
    pub fn validate(&self, envelope: &MessageEnvelope, claim: &ClaimRecord) -> Result<(), String> {
        if self.schema_version != MESSAGE_SCHEMA_VERSION
            || self.trust != UNTRUSTED_DATA
            || self.message_id != envelope.message_id
            || self.envelope_digest != envelope.envelope_digest
            || self.claim_digest != claim.digest()?
            || self.result_file != result_path(&envelope.message_id)
            || !valid_digest(&self.result_digest)
            || self.result_bytes > MAX_RESULT_BYTES as u64
            || self.acknowledged_at < claim.claimed_at
            || self.acknowledged_at > claim.lease_until
        {
            return Err("receipt does not bind this message and claim".into());
        }
        Ok(())
    }

    /// Check that `result` is exactly the bound result file.
    /// # Errors
    /// Rejects a missing file or a size or digest mismatch.
    pub fn verify_result(&self, result: Option<&[u8]>) -> Result<(), String> {
        let Some(result) = result else {
            return Err("result file named by the receipt is missing".into());
        };
        if result.len() as u64 != self.result_bytes
            || crate::sha256_digest(result) != self.result_digest
        {
            return Err("result file does not match its receipt digest and size".into());
        }
        Ok(())
    }
}

/// Facts observed on disk for one message; `None` means the file is absent.
pub struct Observation<'a> {
    /// Envelope parsed, self digest, body, key record and sender all verified.
    pub envelope: Result<&'a MessageEnvelope, String>,
    pub claim: Option<&'a ClaimRecord>,
    pub receipt: Option<&'a MessageReceipt>,
    /// Bytes of `results/<id>.result`; `None` when the file is absent.
    pub result: Option<&'a [u8]>,
    /// A late or conflicting result was moved to `quarantine/`.
    pub quarantined_result: bool,
    /// An active stop covers this message (project, its task or its flow).
    pub cancel_requested: bool,
    /// Host official cancel receipt, already parsed.
    pub cancel_receipt: Option<&'a crate::containment::CancelReceipt>,
    pub now: u64,
}

/// Derive the current state and, for `quarantined`, the reason. Pure and repeatable.
#[must_use]
pub fn derive_state(observation: &Observation<'_>) -> (MessageState, Option<String>) {
    let envelope = match observation.envelope {
        Ok(envelope) => envelope,
        Err(ref reason) => return (MessageState::Quarantined, Some(reason.clone())),
    };
    let Some(claim) = observation.claim else {
        if observation.receipt.is_some() {
            return (
                MessageState::Quarantined,
                Some("receipt exists without a claim".into()),
            );
        }
        return if observation.now >= envelope.expires_at {
            (MessageState::Expired, None)
        } else {
            (MessageState::Queued, None)
        };
    };
    if let Err(reason) = claim.validate(envelope) {
        return (MessageState::Quarantined, Some(reason));
    }
    if let Some(receipt) = observation.receipt {
        return match receipt
            .validate(envelope, claim)
            .and_then(|()| receipt.verify_result(observation.result))
        {
            Ok(()) => (MessageState::Acknowledged, None),
            Err(reason) => (MessageState::Quarantined, Some(reason)),
        };
    }
    if let Some(cancel) = observation.cancel_receipt {
        return match cancel.validate(envelope, claim) {
            Ok(()) => (MessageState::Cancelled, None),
            Err(reason) => (MessageState::Quarantined, Some(reason)),
        };
    }
    if observation.quarantined_result {
        let reason = if observation.cancel_requested {
            "a result arrived after the stop; it was not delivered"
        } else {
            "a late result arrived after the claim lease"
        };
        return (MessageState::Quarantined, Some(reason.into()));
    }
    if observation.now >= claim.lease_until {
        (MessageState::DispatchUncertain, None)
    } else if observation.cancel_requested {
        (MessageState::CancelRequested, None)
    } else {
        (MessageState::Claimed, None)
    }
}

/// Validate a caller-supplied idempotency key and return its digest.
/// # Errors
/// Rejects empty, oversize or control-character keys.
pub fn idempotency_digest(key: &str) -> Result<String, String> {
    if key.is_empty() || key.len() > MAX_IDEMPOTENCY_KEY_BYTES || key.chars().any(char::is_control)
    {
        return Err(format!(
            "idempotency key must contain 1..={MAX_IDEMPOTENCY_KEY_BYTES} bytes without control characters"
        ));
    }
    Ok(crate::sha256_digest(key.as_bytes()))
}

/// Default key when the caller supplies none: identical sends are one message.
#[must_use]
pub fn default_idempotency_key(
    sender_role: &str,
    recipient_role: &str,
    recipient_task: Option<&str>,
    body_digest: &str,
) -> String {
    format!(
        "auto:{sender_role}>{recipient_role}:{}:{body_digest}",
        recipient_task.unwrap_or("-")
    )
}

/// Message ids are derived from the idempotency digest, so duplicates share one name.
#[must_use]
pub fn message_id_for(idempotency_digest: &str) -> String {
    let hex = idempotency_digest.strip_prefix("sha256:").unwrap_or("");
    format!("msg-{}", hex.get(..32).unwrap_or(hex))
}

/// `.hive/mediation/inbox/<role>`
#[must_use]
pub fn inbox_dir(role: &str) -> String {
    format!("{MEDIATION_ROOT}/inbox/{role}")
}

/// `.hive/mediation/inbox/<role>/<message_id>.json`
#[must_use]
pub fn envelope_path(role: &str, message_id: &str) -> String {
    format!("{}/{message_id}.json", inbox_dir(role))
}

/// `.hive/mediation/inbox/<role>/<message_id>.body`
#[must_use]
pub fn body_path(role: &str, message_id: &str) -> String {
    format!("{}/{message_id}.body", inbox_dir(role))
}

/// `.hive/mediation/keys/<sha256 hex of the idempotency key>.json`
#[must_use]
pub fn key_path(idempotency_digest: &str) -> String {
    let hex = idempotency_digest
        .strip_prefix("sha256:")
        .unwrap_or(idempotency_digest);
    format!("{MEDIATION_ROOT}/keys/{hex}.json")
}

/// `.hive/mediation/sequence/<20-digit sequence>.json`
#[must_use]
pub fn sequence_path(sequence: u64) -> String {
    format!("{MEDIATION_ROOT}/sequence/{sequence:020}.json")
}

/// `.hive/mediation/claims/<message_id>.json`
#[must_use]
pub fn claim_path(message_id: &str) -> String {
    format!("{MEDIATION_ROOT}/claims/{message_id}.json")
}

/// `.hive/mediation/receipts/<message_id>.json`
#[must_use]
pub fn receipt_path(message_id: &str) -> String {
    format!("{MEDIATION_ROOT}/receipts/{message_id}.json")
}

/// `.hive/mediation/results/<message_id>.result`
#[must_use]
pub fn result_path(message_id: &str) -> String {
    format!("{MEDIATION_ROOT}/results/{message_id}.result")
}

/// `.hive/mediation/quarantine/<message_id>-<first 16 hex of the result digest>.result`
#[must_use]
pub fn quarantine_path(message_id: &str, result_digest: &str) -> String {
    let hex = result_digest.strip_prefix("sha256:").unwrap_or("");
    format!(
        "{MEDIATION_ROOT}/quarantine/{message_id}-{}.result",
        hex.get(..16).unwrap_or(hex)
    )
}

/// Validate a role, task or message id with the orchestration id rules.
#[must_use]
pub fn is_valid_id(value: &str) -> bool {
    valid_id(value)
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json_canonicalizer::to_vec(value)
        .map(|bytes| crate::sha256_digest(&bytes))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(body: &[u8]) -> MessageEnvelope {
        let key = idempotency_digest("key-1").unwrap();
        let id = message_id_for(&key);
        MessageEnvelope {
            schema_version: 1,
            message_id: id.clone(),
            sequence: 1,
            recipient_role: "hive-research-specialist".into(),
            recipient_task: Some("task-1".into()),
            sender: MessageSender {
                role: "hive-complex-implementer".into(),
                god: Some("athena".into()),
                provider: "codex".into(),
                model: "gpt-5.6-terra".into(),
                effort: "max".into(),
            },
            idempotency_digest: key,
            body_file: body_path("hive-research-specialist", &id),
            body_digest: crate::sha256_digest(body),
            body_bytes: body.len() as u64,
            created_at: 100,
            expires_at: 100 + DEFAULT_MESSAGE_TTL_SECONDS,
            trust: UNTRUSTED_DATA.into(),
            envelope_digest: String::new(),
        }
        .seal()
        .unwrap()
    }

    fn claim(envelope: &MessageEnvelope) -> ClaimRecord {
        ClaimRecord {
            schema_version: 1,
            message_id: envelope.message_id.clone(),
            envelope_digest: envelope.envelope_digest.clone(),
            claimant_role: envelope.recipient_role.clone(),
            claim_token: "0123456789abcdef".into(),
            claimed_at: 200,
            lease_until: 200 + DEFAULT_CLAIM_LEASE_SECONDS,
        }
    }

    #[test]
    fn mediation_transition_table_is_closed() {
        use MessageState as S;
        let all = [
            S::Queued,
            S::Claimed,
            S::Acknowledged,
            S::DispatchUncertain,
            S::Expired,
            S::Quarantined,
            S::CancelRequested,
            S::Cancelled,
        ];
        let allowed: Vec<(Option<S>, S)> = std::iter::once(None)
            .chain(all.iter().copied().map(Some))
            .flat_map(|from| all.iter().map(move |to| (from, *to)))
            .filter(|(from, to)| allowed_transition(*from, *to))
            .collect();
        assert_eq!(
            allowed,
            vec![
                (None, S::Queued),
                (Some(S::Queued), S::Claimed),
                (Some(S::Queued), S::Expired),
                (Some(S::Queued), S::Quarantined),
                (Some(S::Claimed), S::Acknowledged),
                (Some(S::Claimed), S::DispatchUncertain),
                (Some(S::Claimed), S::Quarantined),
                (Some(S::Claimed), S::CancelRequested),
                (Some(S::DispatchUncertain), S::Quarantined),
                (Some(S::CancelRequested), S::Claimed),
                (Some(S::CancelRequested), S::DispatchUncertain),
                (Some(S::CancelRequested), S::Quarantined),
                (Some(S::CancelRequested), S::Cancelled),
            ]
        );
        for state in all {
            if state.is_terminal() {
                assert!(all.iter().all(|to| !allowed_transition(Some(state), *to)));
            }
        }
        // No automatic re-dispatch: an uncertain claim never returns to a dispatchable state.
        assert!(!allowed_transition(Some(S::DispatchUncertain), S::Queued));
        assert!(!allowed_transition(Some(S::DispatchUncertain), S::Claimed));
        assert_eq!(
            serde_json::to_value(S::DispatchUncertain).unwrap(),
            "dispatch-uncertain"
        );
    }

    #[test]
    fn mediation_envelope_binds_fields_body_and_paths() {
        let body = b"review the parser";
        let sealed = envelope(body);
        sealed.validate().unwrap();
        sealed.verify_body(body).unwrap();
        assert!(sealed.verify_body(b"review the parsex").is_err());
        let mut forged = sealed.clone();
        forged.sender.model = "another-model".into();
        assert!(forged.validate().unwrap_err().contains("digest"));
        let mut escaped = sealed.clone();
        escaped.body_file = ".hive/mediation/inbox/../../escape.body".into();
        assert!(escaped.seal().is_err());
        for role in ["../escape", "/abs", "a/b", ""] {
            let mut bad = sealed.clone();
            bad.recipient_role = role.into();
            assert!(bad.seal().is_err(), "{role}");
        }
        let mut oversize = sealed.clone();
        oversize.body_bytes = MAX_BODY_BYTES as u64 + 1;
        assert!(oversize.seal().is_err());
        let mut trusted = sealed.clone();
        trusted.trust = "instructions".into();
        assert!(trusted.seal().is_err());
        let mut wrong_id = sealed;
        wrong_id.message_id = "msg-other".into();
        assert!(wrong_id.seal().is_err());
    }

    #[test]
    fn mediation_keys_are_bounded_and_ids_are_deterministic() {
        assert!(idempotency_digest("").is_err());
        assert!(idempotency_digest("a\nb").is_err());
        assert!(idempotency_digest(&"k".repeat(MAX_IDEMPOTENCY_KEY_BYTES + 1)).is_err());
        let digest = idempotency_digest("same").unwrap();
        assert_eq!(message_id_for(&digest), message_id_for(&digest));
        assert_ne!(
            message_id_for(&digest),
            message_id_for(&idempotency_digest("other").unwrap())
        );
        assert!(is_valid_id(&message_id_for(&digest)));
        assert_eq!(
            default_idempotency_key("a", "b", None, "sha256:x"),
            "auto:a>b:-:sha256:x"
        );
        for path in [
            key_path(&digest),
            claim_path("msg-1"),
            receipt_path("msg-1"),
            result_path("msg-1"),
            sequence_path(7),
            quarantine_path("msg-1", &digest),
        ] {
            assert!(path.starts_with(".hive/mediation/"), "{path}");
            crate::validate_project_relative(std::path::Path::new(&path)).unwrap();
        }
    }

    #[test]
    fn mediation_state_derivation_covers_loss_and_quarantine() {
        let sealed = envelope(b"body");
        let claimed = claim(&sealed);
        let observe = |claim, receipt, quarantined_result, now| {
            derive_state(&Observation {
                envelope: Ok(&sealed),
                claim,
                receipt,
                result: Some(&b"done"[..]),
                quarantined_result,
                cancel_requested: false,
                cancel_receipt: None,
                now,
            })
            .0
        };
        assert_eq!(observe(None, None, false, 150), MessageState::Queued);
        assert_eq!(
            observe(None, None, false, sealed.expires_at),
            MessageState::Expired
        );
        assert_eq!(
            observe(Some(&claimed), None, false, 250),
            MessageState::Claimed
        );
        // Lost receipt after the lease: uncertain, never re-queued.
        assert_eq!(
            observe(Some(&claimed), None, false, claimed.lease_until),
            MessageState::DispatchUncertain
        );
        assert_eq!(
            observe(Some(&claimed), None, true, claimed.lease_until),
            MessageState::Quarantined
        );
        let receipt = MessageReceipt {
            schema_version: 1,
            message_id: sealed.message_id.clone(),
            envelope_digest: sealed.envelope_digest.clone(),
            claim_digest: claimed.digest().unwrap(),
            result_file: result_path(&sealed.message_id),
            result_digest: crate::sha256_digest(b"done"),
            result_bytes: 4,
            acknowledged_at: 300,
            trust: UNTRUSTED_DATA.into(),
        };
        assert_eq!(
            observe(Some(&claimed), Some(&receipt), false, 10_000_000),
            MessageState::Acknowledged
        );
        let mut forged = receipt.clone();
        forged.claim_digest = crate::sha256_digest(b"other claim");
        assert_eq!(
            observe(Some(&claimed), Some(&forged), false, 300),
            MessageState::Quarantined
        );
        assert_eq!(
            observe(None, Some(&receipt), false, 300),
            MessageState::Quarantined
        );
        // A receipt recorded after the claim lease never acknowledges (late `--now`).
        let mut late = receipt.clone();
        late.acknowledged_at = claimed.lease_until + 1;
        assert!(late.validate(&sealed, &claimed).is_err());
        // The result file must match the receipt digest and size (B2).
        let check = |result: Option<&[u8]>| {
            derive_state(&Observation {
                envelope: Ok(&sealed),
                claim: Some(&claimed),
                receipt: Some(&receipt),
                result,
                quarantined_result: false,
                cancel_requested: false,
                cancel_receipt: None,
                now: 300,
            })
        };
        assert_eq!(check(Some(b"done")).0, MessageState::Acknowledged);
        let overwritten = check(Some(b"evil"));
        assert_eq!(overwritten.0, MessageState::Quarantined);
        assert!(overwritten.1.unwrap().contains("receipt digest"));
        assert_eq!(check(None).0, MessageState::Quarantined);
        let mut wrong_claimant = claimed.clone();
        wrong_claimant.claimant_role = "hive-routine-implementer".into();
        assert_eq!(
            observe(Some(&wrong_claimant), None, false, 250),
            MessageState::Quarantined
        );
        let tampered = derive_state(&Observation {
            envelope: Err("message envelope digest does not match its contents".into()),
            claim: None,
            receipt: None,
            result: None,
            quarantined_result: false,
            cancel_requested: false,
            cancel_receipt: None,
            now: 0,
        });
        assert_eq!(tampered.0, MessageState::Quarantined);
        assert!(tampered.1.unwrap().contains("digest"));
    }

    #[test]
    fn mediation_stop_overlay_needs_a_host_receipt_to_cancel() {
        let sealed = envelope(b"body");
        let claimed = claim(&sealed);
        let cancel = crate::containment::CancelReceipt {
            schema_version: 1,
            message_id: sealed.message_id.clone(),
            envelope_digest: sealed.envelope_digest.clone(),
            claim_digest: claimed.digest().unwrap(),
            stop_id: "stop-0123456789abcdef01234567".into(),
            host: "codex".into(),
            method: crate::containment::HOST_OFFICIAL_CANCEL.into(),
            cancelled_at: 300,
            forced: false,
            trust: UNTRUSTED_DATA.into(),
        };
        let observe = |requested, receipt, quarantined_result, now| {
            derive_state(&Observation {
                envelope: Ok(&sealed),
                claim: Some(&claimed),
                receipt: None,
                result: None,
                quarantined_result,
                cancel_requested: requested,
                cancel_receipt: receipt,
                now,
            })
        };
        assert_eq!(
            observe(true, None, false, 250).0,
            MessageState::CancelRequested
        );
        // No host receipt: the lease still decides; Hive never claims a forced cancel.
        assert_eq!(
            observe(true, None, false, claimed.lease_until).0,
            MessageState::DispatchUncertain
        );
        assert_eq!(
            observe(true, Some(&cancel), false, 250).0,
            MessageState::Cancelled
        );
        assert!(MessageState::Cancelled.is_terminal());
        let late = observe(true, None, true, 250);
        assert_eq!(late.0, MessageState::Quarantined);
        assert!(late.1.unwrap().contains("after the stop"));
        let mut forced = cancel.clone();
        forced.forced = true;
        assert_eq!(
            observe(true, Some(&forced), false, 250).0,
            MessageState::Quarantined
        );
        let mut keys = cancel.clone();
        keys.method = "send-keys".into();
        assert_eq!(
            observe(true, Some(&keys), false, 250).0,
            MessageState::Quarantined
        );
    }

    #[test]
    fn mediation_key_record_detects_conflicting_body() {
        let sealed = envelope(b"body");
        let record = MessageKeyRecord {
            schema_version: 1,
            idempotency_digest: sealed.idempotency_digest.clone(),
            message_id: sealed.message_id.clone(),
            recipient_role: sealed.recipient_role.clone(),
            recipient_task: sealed.recipient_task.clone(),
            sender_role: sealed.sender.role.clone(),
            body_digest: sealed.body_digest.clone(),
        };
        record.validate().unwrap();
        assert!(record.binds(&sealed));
        let mut other = record.clone();
        other.body_digest = crate::sha256_digest(b"different");
        assert!(!record.same_request(&other));
        assert!(!other.binds(&sealed));
    }
}
