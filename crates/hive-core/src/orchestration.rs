//! Provider-neutral orchestration events, receipts, authority, leases, and scheduling.
//!
//! This module owns deterministic data transitions only. It never calls a
//! model provider, reads provider credentials, or launches a host process.

use crate::{sha256_digest, validate_json_schema};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const RECEIPT_SCHEMA: &str =
    include_str!("../../../schemas/host-orchestration-receipt.schema.json");
const AUTHORITY_SCHEMA: &str = include_str!("../../../schemas/orchestration-authority.schema.json");
const TRUST_ROOT_SCHEMA: &str =
    include_str!("../../../schemas/orchestration-trust-root.schema.json");
const AUTHORITY_DOMAIN: &[u8] = b"AIGENT-HIVE\0ORCHESTRATION-AUTHORITY\0V1\0";

/// Durable lifecycle state for one dispatch action.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DispatchState {
    Reserved,
    Prepared,
    Claimed,
    DispatchUncertain,
    Acknowledged,
    Running,
    CancelRequested,
    ResultReceived,
    Expired,
    Quarantined,
}

/// Immutable event intent.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    Reserve,
    IssueAuthority,
    RevokeAuthority,
    RebuildProjection,
    Migrate,
    Prepare,
    Claim,
    MarkDispatchUncertain,
    Acknowledge,
    Heartbeat,
    RequestCancel,
    ReceiveResult,
    Expire,
    Quarantine,
    Recover,
}

/// One immutable event revision. The TOML file containing this value is canonical.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrchestrationEvent {
    pub schema_version: u32,
    pub event_id: String,
    pub run_id: String,
    pub action_id: String,
    pub sequence: u64,
    pub predecessor_digest: Option<String>,
    pub control_epoch: u64,
    pub kind: EventKind,
    pub from_state: Option<DispatchState>,
    pub to_state: DispatchState,
    pub authority_id: String,
    pub request_digest: String,
    pub payload_digest: String,
    pub occurred_at: String,
}

/// The only mutable canonical pointer for a run event chain.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventHead {
    pub schema_version: u32,
    pub generation: u64,
    pub sequence: u64,
    pub event_digest: String,
    pub control_epoch: u64,
}

impl EventHead {
    #[must_use]
    pub fn binding(&self) -> String {
        format!("{}:{}", self.sequence, self.event_digest)
    }
}

/// Materialized reducer state rebuilt from immutable events.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ReducerState {
    pub state: Option<DispatchState>,
    pub sequence: u64,
    pub event_digest: Option<String>,
    pub control_epoch: u64,
    pub consumed_authorities: BTreeSet<String>,
    pub receipt_digests: BTreeMap<String, String>,
    pub last_progress_sequence: Option<u64>,
}

impl ReducerState {
    /// Apply one event with exact predecessor, transition, epoch, and one-time authority checks.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid data, a stale head or epoch, an illegal transition, or replay.
    pub fn apply_event(
        &mut self,
        event: &OrchestrationEvent,
    ) -> Result<String, OrchestrationError> {
        event.validate()?;
        if event.sequence != self.sequence + 1
            || event.predecessor_digest != self.event_digest
            || event.from_state != self.state
        {
            return Err(OrchestrationError::HeadMismatch);
        }
        if event.control_epoch < self.control_epoch
            || (event.kind == EventKind::RequestCancel
                && event.control_epoch != self.control_epoch + 1)
            || (event.kind != EventKind::RequestCancel && event.control_epoch != self.control_epoch)
        {
            return Err(OrchestrationError::ControlEpochMismatch);
        }
        if !allowed_transition(event.kind, event.from_state, event.to_state) {
            return Err(OrchestrationError::IllegalTransition);
        }
        if !self.consumed_authorities.insert(event.authority_id.clone()) {
            return Err(OrchestrationError::AuthorityReplay);
        }
        let digest = digest_value(event)?;
        self.state = Some(event.to_state);
        self.sequence = event.sequence;
        self.event_digest = Some(digest.clone());
        self.control_epoch = event.control_epoch;
        Ok(digest)
    }

    /// Bind a receipt by exact bytes. Exact duplicates are no-ops; conflicting bytes quarantine.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed, stale, late, conflicting, or state-incompatible receipts.
    pub fn bind_receipt(
        &mut self,
        receipt: &HostReceipt,
    ) -> Result<ReceiptDisposition, OrchestrationError> {
        receipt.validate()?;
        let digest = digest_value(receipt)?;
        if let Some(existing) = self.receipt_digests.get(&receipt.receipt_id) {
            return if existing == &digest {
                Ok(ReceiptDisposition::Duplicate)
            } else {
                self.state = Some(DispatchState::Quarantined);
                Err(OrchestrationError::ReceiptConflict)
            };
        }
        let current = self.state.ok_or(OrchestrationError::IllegalTransition)?;
        validate_receipt_state(receipt, current)?;
        if receipt.kind == ReceiptKind::Heartbeat {
            let progress = receipt
                .progress_sequence
                .ok_or(OrchestrationError::InvalidReceipt)?;
            if self
                .last_progress_sequence
                .is_some_and(|last| progress <= last)
            {
                return Err(OrchestrationError::StaleReceipt);
            }
            self.last_progress_sequence = Some(progress);
        }
        self.receipt_digests
            .insert(receipt.receipt_id.clone(), digest);
        Ok(ReceiptDisposition::Accepted)
    }
}

impl OrchestrationEvent {
    /// Return the canonical digest used by the event head.
    ///
    /// # Errors
    ///
    /// Returns an error when canonical serialization fails.
    pub fn digest(&self) -> Result<String, OrchestrationError> {
        digest_value(self)
    }

    /// Validate bounded event identifiers and digest syntax.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError::InvalidEvent`] for an invalid event.
    pub fn validate(&self) -> Result<(), OrchestrationError> {
        if self.schema_version != 1
            || self.sequence == 0
            || !valid_id(&self.event_id)
            || !valid_id(&self.run_id)
            || !valid_id(&self.action_id)
            || !valid_id(&self.authority_id)
            || !valid_digest(&self.request_digest)
            || !valid_digest(&self.payload_digest)
            || self
                .predecessor_digest
                .as_ref()
                .is_some_and(|digest| !valid_digest(digest))
            || self.occurred_at.is_empty()
        {
            return Err(OrchestrationError::InvalidEvent);
        }
        Ok(())
    }
}

fn allowed_transition(kind: EventKind, from: Option<DispatchState>, to: DispatchState) -> bool {
    use DispatchState as S;
    use EventKind as E;
    if matches!(
        kind,
        E::IssueAuthority | E::RevokeAuthority | E::RebuildProjection
    ) {
        return from == Some(to);
    }
    matches!(
        (kind, from, to),
        (E::Reserve | E::Migrate, None, S::Reserved)
            | (E::Prepare, Some(S::Reserved | S::Expired), S::Prepared)
            | (E::Claim, Some(S::Prepared), S::Claimed)
            | (
                E::MarkDispatchUncertain,
                Some(S::Claimed),
                S::DispatchUncertain
            )
            | (
                E::Acknowledge,
                Some(S::Claimed | S::DispatchUncertain),
                S::Acknowledged
            )
            | (E::Heartbeat, Some(S::Acknowledged | S::Running), S::Running)
            | (
                E::RequestCancel,
                Some(
                    S::Prepared | S::Claimed | S::DispatchUncertain | S::Acknowledged | S::Running
                ),
                S::CancelRequested
            )
            | (
                E::ReceiveResult,
                Some(S::Acknowledged | S::Running),
                S::ResultReceived
            )
            | (E::Expire, Some(S::Reserved | S::Prepared), S::Expired)
            | (
                E::Quarantine,
                Some(
                    S::Claimed
                        | S::DispatchUncertain
                        | S::Acknowledged
                        | S::Running
                        | S::CancelRequested
                ),
                S::Quarantined
            )
            | (E::Recover, Some(S::Expired | S::Quarantined), S::Prepared)
    )
}

/// Supported cooperative host receipt kinds.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptKind {
    Claim,
    LaunchAck,
    Heartbeat,
    Lookup,
    NonLaunchProof,
    CancelAck,
    FinalResult,
}

/// Receipt accepted from a qualified host-owned lifecycle.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostReceipt {
    pub schema_version: u32,
    pub receipt_id: String,
    pub kind: ReceiptKind,
    pub run_id: String,
    pub action_id: String,
    pub event_head: String,
    pub control_epoch: u64,
    pub idempotency_key: String,
    pub fencing_token: u64,
    pub host: String,
    #[serde(default)]
    pub native_task_id: Option<String>,
    #[serde(default)]
    pub progress_sequence: Option<u64>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub evidence_locator: Option<String>,
    pub host_capability_digest: String,
    pub source_locator: String,
    pub source_digest: String,
    pub issued_at: String,
    pub received_at: String,
    #[serde(default)]
    pub provenance: Option<String>,
}

impl HostReceipt {
    /// Parse and validate one closed-schema host receipt.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, schema violations, or semantic mismatches.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, OrchestrationError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
        validate_json_schema(RECEIPT_SCHEMA, &value, "host orchestration receipt")
            .map_err(OrchestrationError::Schema)?;
        let receipt: Self = serde_json::from_value(value)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
        receipt.validate()?;
        Ok(receipt)
    }

    /// Validate semantic requirements that depend on receipt kind.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError::InvalidReceipt`] for invalid bindings or fields.
    pub fn validate(&self) -> Result<(), OrchestrationError> {
        if self.schema_version != 1
            || self.fencing_token == 0
            || !valid_id(&self.receipt_id)
            || !valid_id(&self.run_id)
            || !valid_id(&self.action_id)
            || !valid_id(&self.idempotency_key)
            || !valid_head(&self.event_head)
            || !valid_digest(&self.host_capability_digest)
            || !valid_digest(&self.source_digest)
            || !matches!(self.host.as_str(), "codex" | "claude" | "antigravity")
            || self.source_locator.is_empty()
            || self.issued_at.is_empty()
            || self.received_at.is_empty()
            || self.issued_at > self.received_at
        {
            return Err(OrchestrationError::InvalidReceipt);
        }
        let native_required = matches!(
            self.kind,
            ReceiptKind::LaunchAck
                | ReceiptKind::Heartbeat
                | ReceiptKind::CancelAck
                | ReceiptKind::FinalResult
        );
        if native_required != self.native_task_id.is_some()
            || (self.kind == ReceiptKind::Heartbeat) != self.progress_sequence.is_some()
            || matches!(
                self.kind,
                ReceiptKind::Lookup | ReceiptKind::CancelAck | ReceiptKind::FinalResult
            ) != self.outcome.is_some()
            || (self.kind == ReceiptKind::FinalResult) != self.evidence_locator.is_some()
        {
            return Err(OrchestrationError::InvalidReceipt);
        }
        Ok(())
    }
}

fn validate_receipt_state(
    receipt: &HostReceipt,
    state: DispatchState,
) -> Result<(), OrchestrationError> {
    use DispatchState as S;
    use ReceiptKind as R;
    let allowed = matches!(
        (receipt.kind, state),
        (R::Claim, S::Prepared)
            | (R::LaunchAck, S::Claimed | S::DispatchUncertain)
            | (R::Heartbeat | R::FinalResult, S::Acknowledged | S::Running)
            | (R::Lookup, S::DispatchUncertain | S::CancelRequested)
            | (R::NonLaunchProof, S::DispatchUncertain | S::Expired)
            | (R::CancelAck, S::CancelRequested)
    );
    if !allowed {
        return Err(if matches!(state, S::CancelRequested | S::Quarantined) {
            OrchestrationError::LateReceipt
        } else {
            OrchestrationError::IllegalTransition
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReceiptDisposition {
    Accepted,
    Duplicate,
}

/// Action names bound into signed one-time authorities.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorityAction {
    Plan,
    Dispatch,
    Receipt,
    Cancel,
    Recover,
    Migrate,
    IssueAuthority,
    RevokeAuthority,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorityKeyStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedAuthorityKey {
    pub key_id: String,
    pub principal_id: String,
    pub algorithm: String,
    pub public_key: String,
    pub status: AuthorityKeyStatus,
    pub valid_from: String,
    pub valid_until: String,
    pub allowed_actions: BTreeSet<AuthorityAction>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrchestrationTrustRoot {
    pub schema_version: u32,
    pub trust_root_id: String,
    pub revision: u64,
    pub issued_at: String,
    pub keys: Vec<TrustedAuthorityKey>,
    pub root_digest: String,
}

#[derive(Serialize)]
struct TrustRootPayload<'a> {
    schema_version: u32,
    trust_root_id: &'a str,
    revision: u64,
    issued_at: &'a str,
    keys: &'a [TrustedAuthorityKey],
}

impl OrchestrationTrustRoot {
    /// Parse and validate one external orchestration trust root.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, schema violations, or invalid keys and digests.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, OrchestrationError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
        validate_json_schema(TRUST_ROOT_SCHEMA, &value, "orchestration trust root")
            .map_err(OrchestrationError::Schema)?;
        let root: Self = serde_json::from_value(value)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
        root.validate()?;
        Ok(root)
    }

    /// Compute the canonical digest excluding `root_digest`.
    ///
    /// # Errors
    ///
    /// Returns an error when canonical serialization fails.
    pub fn computed_digest(&self) -> Result<String, OrchestrationError> {
        digest_value(&TrustRootPayload {
            schema_version: self.schema_version,
            trust_root_id: &self.trust_root_id,
            revision: self.revision,
            issued_at: &self.issued_at,
            keys: &self.keys,
        })
    }

    /// Validate root identity, digest, key uniqueness, and Ed25519 encodings.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError::InvalidTrustRoot`] for an invalid trust root.
    pub fn validate(&self) -> Result<(), OrchestrationError> {
        if self.schema_version != 1
            || self.revision == 0
            || !valid_id(&self.trust_root_id)
            || self.computed_digest()? != self.root_digest
        {
            return Err(OrchestrationError::InvalidTrustRoot);
        }
        let mut ids = BTreeSet::new();
        let mut keys = BTreeSet::new();
        for key in &self.keys {
            let bytes = decode_prefixed_hex::<32>(&key.public_key, "ed25519:")?;
            VerifyingKey::from_bytes(&bytes).map_err(|_| OrchestrationError::InvalidTrustRoot)?;
            if key.algorithm != "ed25519"
                || key.valid_from > key.valid_until
                || key.allowed_actions.is_empty()
                || !ids.insert(key.key_id.as_str())
                || !keys.insert(bytes)
            {
                return Err(OrchestrationError::InvalidTrustRoot);
            }
        }
        Ok(())
    }
}

/// Signed authority consumed by exactly one successful event commit.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionAuthority {
    pub schema_version: u32,
    pub authority_id: String,
    pub trust_root_id: String,
    pub principal_id: String,
    pub role_id: String,
    pub action: AuthorityAction,
    pub target_digest: String,
    pub expected_head: String,
    pub control_epoch: u64,
    pub request_digest: String,
    pub nonce: String,
    pub issued_at: String,
    pub valid_until: String,
    pub key_id: String,
    pub signature: String,
}

/// Exact caller expectation bound into one authority verification.
#[derive(Debug, Clone, Copy)]
pub struct AuthorityExpectation<'a> {
    pub action: AuthorityAction,
    pub target_digest: &'a str,
    pub head: &'a str,
    pub control_epoch: u64,
    pub request_digest: &'a str,
    pub now: &'a str,
}

#[derive(Serialize)]
struct AuthorityPayload<'a> {
    schema_version: u32,
    authority_id: &'a str,
    trust_root_id: &'a str,
    principal_id: &'a str,
    role_id: &'a str,
    action: AuthorityAction,
    target_digest: &'a str,
    expected_head: &'a str,
    control_epoch: u64,
    request_digest: &'a str,
    nonce: &'a str,
    issued_at: &'a str,
    valid_until: &'a str,
    key_id: &'a str,
}

impl ActionAuthority {
    /// Parse one closed-schema signed action authority.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON or a schema violation.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, OrchestrationError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
        validate_json_schema(AUTHORITY_SCHEMA, &value, "orchestration authority")
            .map_err(OrchestrationError::Schema)?;
        serde_json::from_value(value)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))
    }

    /// Verify an action authority against an external public-key root and exact expectation.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError::AuthorityRejected`] for any binding or signature mismatch.
    pub fn verify(
        &self,
        root: &OrchestrationTrustRoot,
        expected: AuthorityExpectation<'_>,
    ) -> Result<(), OrchestrationError> {
        root.validate()?;
        if self.schema_version != 1
            || self.trust_root_id != root.trust_root_id
            || self.action != expected.action
            || self.target_digest != expected.target_digest
            || self.expected_head != expected.head
            || self.control_epoch != expected.control_epoch
            || self.request_digest != expected.request_digest
            || self.issued_at > self.valid_until
            || expected.now < self.issued_at.as_str()
            || expected.now > self.valid_until.as_str()
        {
            return Err(OrchestrationError::AuthorityRejected);
        }
        let key = root
            .keys
            .iter()
            .find(|key| key.key_id == self.key_id)
            .ok_or(OrchestrationError::AuthorityRejected)?;
        if key.status != AuthorityKeyStatus::Active
            || key.principal_id != self.principal_id
            || !key.allowed_actions.contains(&self.action)
            || self.issued_at < key.valid_from
            || self.valid_until > key.valid_until
        {
            return Err(OrchestrationError::AuthorityRejected);
        }
        let public_key = decode_prefixed_hex::<32>(&key.public_key, "ed25519:")?;
        let signature = decode_prefixed_hex::<64>(&self.signature, "ed25519:")?;
        VerifyingKey::from_bytes(&public_key)
            .map_err(|_| OrchestrationError::AuthorityRejected)?
            .verify_strict(&self.signing_message()?, &Signature::from_bytes(&signature))
            .map_err(|_| OrchestrationError::AuthorityRejected)
    }

    /// Return domain-separated canonical bytes for an external signer.
    ///
    /// # Errors
    ///
    /// Returns an error when canonical serialization fails.
    pub fn signing_message(&self) -> Result<Vec<u8>, OrchestrationError> {
        let payload = AuthorityPayload {
            schema_version: self.schema_version,
            authority_id: &self.authority_id,
            trust_root_id: &self.trust_root_id,
            principal_id: &self.principal_id,
            role_id: &self.role_id,
            action: self.action,
            target_digest: &self.target_digest,
            expected_head: &self.expected_head,
            control_epoch: self.control_epoch,
            request_digest: &self.request_digest,
            nonce: &self.nonce,
            issued_at: &self.issued_at,
            valid_until: &self.valid_until,
            key_id: &self.key_id,
        };
        let bytes = serde_json_canonicalizer::to_vec(&payload)
            .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
        let mut message = Vec::with_capacity(AUTHORITY_DOMAIN.len() + bytes.len());
        message.extend_from_slice(AUTHORITY_DOMAIN);
        message.extend_from_slice(&bytes);
        Ok(message)
    }
}

/// One scheduler candidate. Model execution remains host-owned.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScheduleCandidate {
    pub run_id: String,
    pub action_id: String,
    pub explicit_priority: i32,
    pub not_before: u64,
    pub enqueued_at: u64,
    pub budget_cost: u64,
}

/// Deterministic bounded scheduler inputs.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SchedulerLimits {
    pub now: u64,
    pub aging_interval: u64,
    pub aging_cap: i32,
    pub available_budget: u64,
    pub active_global: usize,
    pub max_active_global: usize,
}

#[must_use]
pub fn select_candidate(
    candidates: &[ScheduleCandidate],
    limits: SchedulerLimits,
) -> Option<&ScheduleCandidate> {
    if limits.active_global >= limits.max_active_global || limits.aging_interval == 0 {
        return None;
    }
    candidates
        .iter()
        .filter(|candidate| {
            candidate.not_before <= limits.now && candidate.budget_cost <= limits.available_budget
        })
        .max_by(|left, right| compare_candidates(left, right, limits))
}

fn compare_candidates(
    left: &ScheduleCandidate,
    right: &ScheduleCandidate,
    limits: SchedulerLimits,
) -> Ordering {
    let effective = |candidate: &ScheduleCandidate| {
        let age = limits.now.saturating_sub(candidate.enqueued_at) / limits.aging_interval;
        let bounded = i32::try_from(age).unwrap_or(i32::MAX).min(limits.aging_cap);
        candidate.explicit_priority.saturating_add(bounded)
    };
    effective(left)
        .cmp(&effective(right))
        .then_with(|| right.not_before.cmp(&left.not_before))
        .then_with(|| right.run_id.cmp(&left.run_id))
        .then_with(|| right.action_id.cmp(&left.action_id))
}

/// Fenced lease with atomic budget reservation semantics.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Lease {
    pub lease_id: String,
    pub run_id: String,
    pub action_id: String,
    pub fencing_epoch: u64,
    pub issued_at: u64,
    pub deadline: u64,
    pub budget_reserved: u64,
    pub refunded: bool,
}

impl Lease {
    /// Determine active or expired state while rejecting clock rollback.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError::ClockRollback`] for invalid fencing or time order.
    pub fn validate_time(&self, now: u64) -> Result<LeaseState, OrchestrationError> {
        if self.fencing_epoch == 0 || self.deadline < self.issued_at || now < self.issued_at {
            return Err(OrchestrationError::ClockRollback);
        }
        Ok(if now >= self.deadline {
            LeaseState::Expired
        } else {
            LeaseState::Active
        })
    }

    /// Refund one reservation exactly once.
    ///
    /// # Errors
    ///
    /// Returns an error for a replayed refund or integer overflow.
    pub fn refund(&mut self, budget: &mut u64) -> Result<(), OrchestrationError> {
        if self.refunded {
            return Err(OrchestrationError::BudgetReplay);
        }
        *budget = budget
            .checked_add(self.budget_reserved)
            .ok_or(OrchestrationError::BudgetOverflow)?;
        self.refunded = true;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LeaseState {
    Active,
    Expired,
}

/// Reserve a positive budget amount atomically.
///
/// # Errors
///
/// Returns [`OrchestrationError::BudgetUnavailable`] when the amount cannot be reserved.
pub fn reserve_budget(available: &mut u64, requested: u64) -> Result<u64, OrchestrationError> {
    if requested == 0 || requested > *available {
        return Err(OrchestrationError::BudgetUnavailable);
    }
    *available -= requested;
    Ok(requested)
}

/// Rebuild reducer state from an ordered immutable event sequence.
///
/// # Errors
///
/// Returns the first event validation or transition error.
pub fn replay_events(events: &[OrchestrationEvent]) -> Result<ReducerState, OrchestrationError> {
    let mut state = ReducerState::default();
    for event in events {
        state.apply_event(event)?;
    }
    Ok(state)
}

fn digest_value<T: Serialize>(value: &T) -> Result<String, OrchestrationError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| OrchestrationError::Malformed(error.to_string()))?;
    Ok(sha256_digest(&bytes))
}

fn valid_id(value: &str) -> bool {
    (2..=127).contains(&value.len())
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && b"._-".contains(&byte))
        })
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_head(value: &str) -> bool {
    value
        .split_once(':')
        .is_some_and(|(sequence, digest)| sequence.parse::<u64>().is_ok() && valid_digest(digest))
}

fn decode_prefixed_hex<const N: usize>(
    value: &str,
    prefix: &str,
) -> Result<[u8; N], OrchestrationError> {
    let encoded = value
        .strip_prefix(prefix)
        .ok_or(OrchestrationError::AuthorityRejected)?;
    if encoded.len() != N * 2 {
        return Err(OrchestrationError::AuthorityRejected);
    }
    let mut output = [0_u8; N];
    for (index, chunk) in encoded.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(chunk).map_err(|_| OrchestrationError::AuthorityRejected)?;
        output[index] =
            u8::from_str_radix(text, 16).map_err(|_| OrchestrationError::AuthorityRejected)?;
    }
    Ok(output)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrchestrationError {
    Malformed(String),
    Schema(String),
    InvalidEvent,
    InvalidReceipt,
    InvalidTrustRoot,
    HeadMismatch,
    ControlEpochMismatch,
    IllegalTransition,
    AuthorityReplay,
    AuthorityRejected,
    ReceiptConflict,
    StaleReceipt,
    LateReceipt,
    ClockRollback,
    BudgetUnavailable,
    BudgetReplay,
    BudgetOverflow,
}

impl Display for OrchestrationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(error) => write!(formatter, "malformed orchestration data: {error}"),
            Self::Schema(error) => write!(formatter, "orchestration schema violation: {error}"),
            Self::InvalidEvent => formatter.write_str("invalid orchestration event"),
            Self::InvalidReceipt => formatter.write_str("invalid host receipt"),
            Self::InvalidTrustRoot => formatter.write_str("invalid orchestration trust root"),
            Self::HeadMismatch => formatter.write_str("event head does not match"),
            Self::ControlEpochMismatch => formatter.write_str("control epoch does not match"),
            Self::IllegalTransition => formatter.write_str("illegal orchestration transition"),
            Self::AuthorityReplay => formatter.write_str("one-time authority was already consumed"),
            Self::AuthorityRejected => formatter.write_str("orchestration authority rejected"),
            Self::ReceiptConflict => {
                formatter.write_str("receipt id conflicts with existing bytes")
            }
            Self::StaleReceipt => formatter.write_str("receipt sequence is stale"),
            Self::LateReceipt => {
                formatter.write_str("receipt arrived after cancellation or quarantine")
            }
            Self::ClockRollback => formatter.write_str("lease clock moved backward"),
            Self::BudgetUnavailable => formatter.write_str("budget reservation unavailable"),
            Self::BudgetReplay => formatter.write_str("budget was already refunded"),
            Self::BudgetOverflow => formatter.write_str("budget refund overflow"),
        }
    }
}

impl Error for OrchestrationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn digest(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn event(
        sequence: u64,
        kind: EventKind,
        from_state: Option<DispatchState>,
        to_state: DispatchState,
        predecessor_digest: Option<String>,
        epoch: u64,
    ) -> OrchestrationEvent {
        OrchestrationEvent {
            schema_version: 1,
            event_id: format!("event-{sequence}"),
            run_id: "run-1".to_owned(),
            action_id: "action-1".to_owned(),
            sequence,
            predecessor_digest,
            control_epoch: epoch,
            kind,
            from_state,
            to_state,
            authority_id: format!("authority-{sequence}"),
            request_digest: digest('a'),
            payload_digest: digest('b'),
            occurred_at: "2026-08-12T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn reducer_is_deterministic_and_head_bound() {
        let mut state = ReducerState::default();
        let reserve = event(
            1,
            EventKind::Reserve,
            None,
            DispatchState::Reserved,
            None,
            0,
        );
        let first = state.apply_event(&reserve).expect("reserve");
        let prepare = event(
            2,
            EventKind::Prepare,
            Some(DispatchState::Reserved),
            DispatchState::Prepared,
            Some(first),
            0,
        );
        state.apply_event(&prepare).expect("prepare");
        assert_eq!(state.state, Some(DispatchState::Prepared));
        assert_eq!(replay_events(&[reserve, prepare]).expect("replay"), state);
    }

    #[test]
    fn authority_ledger_events_preserve_dispatch_state() {
        let mut state = ReducerState::default();
        let reserve = event(
            1,
            EventKind::Reserve,
            None,
            DispatchState::Reserved,
            None,
            0,
        );
        let head = state.apply_event(&reserve).expect("reserve");
        let issue = event(
            2,
            EventKind::IssueAuthority,
            Some(DispatchState::Reserved),
            DispatchState::Reserved,
            Some(head),
            0,
        );
        let head = state.apply_event(&issue).expect("issue");
        let revoke = event(
            3,
            EventKind::RevokeAuthority,
            Some(DispatchState::Reserved),
            DispatchState::Reserved,
            Some(head),
            0,
        );
        state.apply_event(&revoke).expect("revoke");
        assert_eq!(state.state, Some(DispatchState::Reserved));
        assert_eq!(state.sequence, 3);
    }

    #[test]
    fn migration_bootstraps_a_separate_native_chain() {
        let mut migration = event(
            1,
            EventKind::Migrate,
            None,
            DispatchState::Reserved,
            None,
            0,
        );
        migration.run_id = "native-migration-1".to_owned();
        let state = replay_events(&[migration]).expect("migration");
        assert_eq!(state.state, Some(DispatchState::Reserved));
        assert_eq!(state.sequence, 1);
    }

    #[test]
    fn cancel_requires_new_epoch_and_blocks_late_result() {
        let mut state = ReducerState::default();
        let reserve = event(
            1,
            EventKind::Reserve,
            None,
            DispatchState::Reserved,
            None,
            0,
        );
        let head = state.apply_event(&reserve).expect("reserve");
        let prepare = event(
            2,
            EventKind::Prepare,
            Some(DispatchState::Reserved),
            DispatchState::Prepared,
            Some(head),
            0,
        );
        let head = state.apply_event(&prepare).expect("prepare");
        let cancel = event(
            3,
            EventKind::RequestCancel,
            Some(DispatchState::Prepared),
            DispatchState::CancelRequested,
            Some(head),
            1,
        );
        state.apply_event(&cancel).expect("cancel");
        let mut receipt = valid_receipt(ReceiptKind::FinalResult);
        receipt.native_task_id = Some("native-1".to_owned());
        receipt.outcome = Some("succeeded".to_owned());
        receipt.evidence_locator = Some("evidence/result.json".to_owned());
        assert_eq!(
            state.bind_receipt(&receipt),
            Err(OrchestrationError::LateReceipt)
        );
    }

    #[test]
    fn exact_duplicate_receipt_is_noop_and_conflict_quarantines() {
        let mut state = ReducerState {
            state: Some(DispatchState::Prepared),
            ..ReducerState::default()
        };
        let receipt = valid_receipt(ReceiptKind::Claim);
        assert_eq!(
            state.bind_receipt(&receipt),
            Ok(ReceiptDisposition::Accepted)
        );
        assert_eq!(
            state.bind_receipt(&receipt),
            Ok(ReceiptDisposition::Duplicate)
        );
        let mut conflict = receipt;
        conflict.source_digest = digest('d');
        assert_eq!(
            state.bind_receipt(&conflict),
            Err(OrchestrationError::ReceiptConflict)
        );
        assert_eq!(state.state, Some(DispatchState::Quarantined));
    }

    #[test]
    fn scheduler_applies_aging_budget_backpressure_and_stable_ties() {
        let candidates = vec![
            ScheduleCandidate {
                run_id: "run-b".to_owned(),
                action_id: "a".to_owned(),
                explicit_priority: 2,
                not_before: 0,
                enqueued_at: 90,
                budget_cost: 2,
            },
            ScheduleCandidate {
                run_id: "run-a".to_owned(),
                action_id: "a".to_owned(),
                explicit_priority: 1,
                not_before: 0,
                enqueued_at: 0,
                budget_cost: 2,
            },
        ];
        let limits = SchedulerLimits {
            now: 100,
            aging_interval: 10,
            aging_cap: 10,
            available_budget: 2,
            active_global: 0,
            max_active_global: 1,
        };
        assert_eq!(
            select_candidate(&candidates, limits).map(|item| item.run_id.as_str()),
            Some("run-a")
        );
        assert!(select_candidate(
            &candidates,
            SchedulerLimits {
                active_global: 1,
                ..limits
            }
        )
        .is_none());
    }

    #[test]
    fn lease_rejects_clock_rollback_and_refund_replay() {
        let mut available = 10;
        let reserved = reserve_budget(&mut available, 4).expect("reserve budget");
        let mut lease = Lease {
            lease_id: "lease-1".to_owned(),
            run_id: "run-1".to_owned(),
            action_id: "action-1".to_owned(),
            fencing_epoch: 1,
            issued_at: 10,
            deadline: 20,
            budget_reserved: reserved,
            refunded: false,
        };
        assert_eq!(
            lease.validate_time(9),
            Err(OrchestrationError::ClockRollback)
        );
        assert_eq!(lease.validate_time(20), Ok(LeaseState::Expired));
        lease.refund(&mut available).expect("refund");
        assert_eq!(available, 10);
        assert_eq!(
            lease.refund(&mut available),
            Err(OrchestrationError::BudgetReplay)
        );
    }

    #[test]
    fn authority_binds_exact_head_epoch_action_and_request() {
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let mut root = OrchestrationTrustRoot {
            schema_version: 1,
            trust_root_id: "root-1".to_owned(),
            revision: 1,
            issued_at: "2026-08-12T00:00:00Z".to_owned(),
            keys: vec![TrustedAuthorityKey {
                key_id: "key-1".to_owned(),
                principal_id: "user-1".to_owned(),
                algorithm: "ed25519".to_owned(),
                public_key: format!(
                    "ed25519:{}",
                    encode_hex(signing_key.verifying_key().as_bytes())
                ),
                status: AuthorityKeyStatus::Active,
                valid_from: "2026-08-12T00:00:00Z".to_owned(),
                valid_until: "2026-08-12T02:00:00Z".to_owned(),
                allowed_actions: BTreeSet::from([AuthorityAction::Dispatch]),
            }],
            root_digest: String::new(),
        };
        root.root_digest = root.computed_digest().expect("root digest");
        let mut authority = ActionAuthority {
            schema_version: 1,
            authority_id: "authority-1".to_owned(),
            trust_root_id: "root-1".to_owned(),
            principal_id: "user-1".to_owned(),
            role_id: "worker-1".to_owned(),
            action: AuthorityAction::Dispatch,
            target_digest: digest('a'),
            expected_head: format!("1:{}", digest('b')),
            control_epoch: 2,
            request_digest: digest('c'),
            nonce: "nonce-1".to_owned(),
            issued_at: "2026-08-12T00:30:00Z".to_owned(),
            valid_until: "2026-08-12T01:00:00Z".to_owned(),
            key_id: "key-1".to_owned(),
            signature: format!("ed25519:{}", "00".repeat(64)),
        };
        let signature = signing_key.sign(&authority.signing_message().expect("message"));
        authority.signature = format!("ed25519:{}", encode_hex(&signature.to_bytes()));
        let expected = AuthorityExpectation {
            action: AuthorityAction::Dispatch,
            target_digest: &digest('a'),
            head: &format!("1:{}", digest('b')),
            control_epoch: 2,
            request_digest: &digest('c'),
            now: "2026-08-12T00:45:00Z",
        };
        authority.verify(&root, expected).expect("valid authority");
        assert_eq!(
            authority.verify(
                &root,
                AuthorityExpectation {
                    control_epoch: 3,
                    ..expected
                }
            ),
            Err(OrchestrationError::AuthorityRejected)
        );
    }

    fn valid_receipt(kind: ReceiptKind) -> HostReceipt {
        HostReceipt {
            schema_version: 1,
            receipt_id: "receipt-1".to_owned(),
            kind,
            run_id: "run-1".to_owned(),
            action_id: "action-1".to_owned(),
            event_head: format!("1:{}", digest('a')),
            control_epoch: 0,
            idempotency_key: "dispatch-1".to_owned(),
            fencing_token: 1,
            host: "codex".to_owned(),
            native_task_id: None,
            progress_sequence: None,
            outcome: None,
            evidence_locator: None,
            host_capability_digest: digest('b'),
            source_locator: "host:codex".to_owned(),
            source_digest: digest('c'),
            issued_at: "2026-08-12T00:00:00Z".to_owned(),
            received_at: "2026-08-12T00:00:01Z".to_owned(),
            provenance: None,
        }
    }

    fn encode_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        encoded
    }
}
