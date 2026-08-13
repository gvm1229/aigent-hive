//! Deterministic team and multi-goal state shared by Hive-native Skills.

use crate::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::{Component, Path};

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MailboxMessage {
    pub message_id: String,
    pub sender: String,
    pub recipient: String,
    pub sequence: u64,
    pub body_digest: String,
    pub bytes: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MessageDisposition {
    Accepted,
    Duplicate,
}

#[derive(Debug, Default)]
pub struct Mailbox {
    digests: BTreeMap<String, String>,
    sender_sequences: BTreeMap<String, u64>,
    count: usize,
    bytes: u64,
}

impl Mailbox {
    /// Bind one immutable bounded message.
    ///
    /// # Errors
    ///
    /// Rejects conflicts, stale sequence, invalid identity, or exceeded bounds.
    pub fn bind(
        &mut self,
        message: &MailboxMessage,
        max_messages: usize,
        max_total_bytes: u64,
    ) -> Result<MessageDisposition, WorkflowError> {
        validate_message(message)?;
        let digest = sha256_digest(
            &serde_json_canonicalizer::to_vec(message)
                .map_err(|error| WorkflowError::Malformed(error.to_string()))?,
        );
        if let Some(existing) = self.digests.get(&message.message_id) {
            return if existing == &digest {
                Ok(MessageDisposition::Duplicate)
            } else {
                Err(WorkflowError::MessageConflict)
            };
        }
        let last = self
            .sender_sequences
            .get(&message.sender)
            .copied()
            .unwrap_or(0);
        if message.sequence != last + 1 {
            return Err(WorkflowError::StaleSequence);
        }
        let next_bytes = self
            .bytes
            .checked_add(u64::from(message.bytes))
            .ok_or(WorkflowError::BoundExceeded)?;
        if self.count >= max_messages || next_bytes > max_total_bytes {
            return Err(WorkflowError::BoundExceeded);
        }
        self.digests.insert(message.message_id.clone(), digest);
        self.sender_sequences
            .insert(message.sender.clone(), message.sequence);
        self.count += 1;
        self.bytes = next_bytes;
        Ok(MessageDisposition::Accepted)
    }
}

fn validate_message(message: &MailboxMessage) -> Result<(), WorkflowError> {
    if message.sequence == 0
        || message.bytes == 0
        || !valid_id(&message.message_id)
        || !valid_id(&message.sender)
        || !valid_id(&message.recipient)
        || !valid_digest(&message.body_digest)
    {
        return Err(WorkflowError::InvalidMessage);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LaneState {
    Pending,
    Running,
    Passed,
    Failed,
    Cancelled,
    Quarantined,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Barrier {
    pub revision: u64,
    pub members: BTreeSet<String>,
    pub quorum: usize,
    pub fail_on_failed_lane: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BarrierResult {
    Pending,
    Passed,
    Failed,
}

impl Barrier {
    /// Evaluate the exact committed membership revision.
    #[must_use]
    pub fn evaluate(&self, lanes: &BTreeMap<String, LaneState>) -> BarrierResult {
        if self.revision == 0
            || self.members.is_empty()
            || self.quorum == 0
            || self.quorum > self.members.len()
            || self
                .members
                .iter()
                .any(|member| !lanes.contains_key(member))
        {
            return BarrierResult::Failed;
        }
        let failed = self.members.iter().any(|member| {
            matches!(
                lanes.get(member),
                Some(LaneState::Failed | LaneState::Cancelled | LaneState::Quarantined)
            )
        });
        if failed && self.fail_on_failed_lane {
            return BarrierResult::Failed;
        }
        let passed = self
            .members
            .iter()
            .filter(|member| lanes.get(*member) == Some(&LaneState::Passed))
            .count();
        if passed >= self.quorum {
            BarrierResult::Passed
        } else {
            BarrierResult::Pending
        }
    }
}

/// Normalize a shared path conservatively and reject non-ASCII Unicode.
///
/// # Errors
///
/// Rejects absolute, parent, prefix, empty, and non-ASCII paths.
pub fn canonical_shared_path(value: &str) -> Result<String, WorkflowError> {
    if value.is_empty() || !value.is_ascii() {
        return Err(WorkflowError::UnsafePath);
    }
    let replaced = value.replace('\\', "/");
    let path = Path::new(&replaced);
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let text = part.to_str().ok_or(WorkflowError::UnsafePath)?;
                parts.push(text.to_ascii_lowercase());
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(WorkflowError::UnsafePath);
            }
        }
    }
    if parts.is_empty() {
        return Err(WorkflowError::UnsafePath);
    }
    Ok(parts.join("/"))
}

/// Return true for equal or parent-child overlapping canonical paths.
#[must_use]
pub fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalAggregation {
    And,
    Or,
    Quorum,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalState {
    Pending,
    Executing,
    Verifying,
    Complete,
    Blocked,
    Failed,
    Cancelled,
    Quarantined,
}

/// User-selected boundary for routing an independent Judge.
///
/// This is deliberately a narrow routing decision, not an execution request.
/// The active host remains responsible for any native dispatch after Hive has
/// verified the exact role, capability, and attestation contracts.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JudgeInvocationPolicy {
    Explicit,
    Implicit,
}

/// Closed work categories used when deciding whether a Judge is eligible.
///
/// Strict terminal acceptance cannot be downgraded by user preference. All
/// non-terminal maintenance and low-risk routes remain excluded in both modes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JudgeRoute {
    StrictTerminalAcceptance,
    MaterialRisk,
    SimpleQuestion,
    ReadOnly,
    FormatOnly,
    SchedulerTick,
    Heartbeat,
    Retry,
    DeterministicFailure,
    UnsupportedOrUnattestedHost,
}

impl JudgeInvocationPolicy {
    /// Return whether the policy permits routing the exact work category to a
    /// Judge. This does not authorize host execution.
    #[must_use]
    pub const fn permits(self, route: JudgeRoute) -> bool {
        match route {
            JudgeRoute::StrictTerminalAcceptance => true,
            JudgeRoute::MaterialRisk => matches!(self, Self::Implicit),
            JudgeRoute::SimpleQuestion
            | JudgeRoute::ReadOnly
            | JudgeRoute::FormatOnly
            | JudgeRoute::SchedulerTick
            | JudgeRoute::Heartbeat
            | JudgeRoute::Retry
            | JudgeRoute::DeterministicFailure
            | JudgeRoute::UnsupportedOrUnattestedHost => false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Goal {
    pub goal_id: String,
    pub aggregation: GoalAggregation,
    pub quorum: Option<usize>,
    pub children: BTreeMap<String, GoalState>,
    pub evidence_verified: BTreeSet<String>,
    pub judge_verified: bool,
}

impl Goal {
    /// Evaluate aggregation while keeping Judge as a terminal-only gate.
    #[must_use]
    pub fn evaluate(&self) -> GoalState {
        if self.children.is_empty() || !valid_id(&self.goal_id) {
            return GoalState::Quarantined;
        }
        if self
            .children
            .values()
            .any(|state| *state == GoalState::Quarantined)
        {
            return GoalState::Quarantined;
        }
        if self
            .children
            .values()
            .any(|state| *state == GoalState::Cancelled)
        {
            return GoalState::Cancelled;
        }
        let complete = self
            .children
            .values()
            .filter(|state| **state == GoalState::Complete)
            .count();
        let required = match self.aggregation {
            GoalAggregation::And => self.children.len(),
            GoalAggregation::Or => 1,
            GoalAggregation::Quorum => self.quorum.unwrap_or(0),
        };
        if required == 0 || required > self.children.len() {
            return GoalState::Quarantined;
        }
        let evidence_complete = self
            .children
            .keys()
            .filter(|child| self.children[*child] == GoalState::Complete)
            .all(|child| self.evidence_verified.contains(child));
        if complete >= required && evidence_complete {
            if self.judge_verified {
                GoalState::Complete
            } else {
                GoalState::Verifying
            }
        } else if self
            .children
            .values()
            .all(|state| matches!(state, GoalState::Failed | GoalState::Blocked))
        {
            GoalState::Failed
        } else {
            GoalState::Executing
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GoalBudget {
    pub total: u64,
    pub available: u64,
    allocations: BTreeMap<String, (u64, bool)>,
}

impl GoalBudget {
    #[must_use]
    pub fn new(total: u64) -> Self {
        Self {
            total,
            available: total,
            allocations: BTreeMap::new(),
        }
    }

    /// Allocate one child budget exactly once.
    ///
    /// # Errors
    ///
    /// Rejects invalid identity, zero or excessive allocation, and duplicate allocation.
    pub fn allocate(&mut self, child: &str, amount: u64) -> Result<(), WorkflowError> {
        if !valid_id(child)
            || amount == 0
            || amount > self.available
            || self.allocations.contains_key(child)
        {
            return Err(WorkflowError::BudgetRejected);
        }
        self.available -= amount;
        self.allocations.insert(child.to_owned(), (amount, false));
        Ok(())
    }

    /// Refund unused child budget exactly once.
    ///
    /// # Errors
    ///
    /// Rejects unknown children, excessive refunds, replay, and total overflow.
    pub fn refund(&mut self, child: &str, unused: u64) -> Result<(), WorkflowError> {
        let allocation = self
            .allocations
            .get_mut(child)
            .ok_or(WorkflowError::BudgetRejected)?;
        if allocation.1 || unused > allocation.0 {
            return Err(WorkflowError::BudgetRejected);
        }
        self.available = self
            .available
            .checked_add(unused)
            .filter(|available| *available <= self.total)
            .ok_or(WorkflowError::BudgetRejected)?;
        allocation.1 = true;
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkflowError {
    Malformed(String),
    InvalidMessage,
    MessageConflict,
    StaleSequence,
    BoundExceeded,
    UnsafePath,
    BudgetRejected,
}

impl Display for WorkflowError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(error) => write!(formatter, "malformed workflow data: {error}"),
            Self::InvalidMessage => formatter.write_str("invalid mailbox message"),
            Self::MessageConflict => formatter.write_str("mailbox message id conflict"),
            Self::StaleSequence => formatter.write_str("mailbox sender sequence is stale"),
            Self::BoundExceeded => formatter.write_str("workflow bound exceeded"),
            Self::UnsafePath => formatter.write_str("unsafe or unsupported shared path"),
            Self::BudgetRejected => formatter.write_str("goal budget operation rejected"),
        }
    }
}

impl Error for WorkflowError {}

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
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> String {
        format!("sha256:{}", "a".repeat(64))
    }

    #[test]
    fn mailbox_deduplicates_and_quarantines_conflicts() {
        let mut mailbox = Mailbox::default();
        let message = MailboxMessage {
            message_id: "message-1".to_owned(),
            sender: "lane-1".to_owned(),
            recipient: "lane-2".to_owned(),
            sequence: 1,
            body_digest: digest(),
            bytes: 12,
        };
        assert_eq!(
            mailbox.bind(&message, 2, 24),
            Ok(MessageDisposition::Accepted)
        );
        assert_eq!(
            mailbox.bind(&message, 2, 24),
            Ok(MessageDisposition::Duplicate)
        );
        let mut conflict = message;
        conflict.bytes = 13;
        assert_eq!(
            mailbox.bind(&conflict, 2, 24),
            Err(WorkflowError::MessageConflict)
        );
    }

    #[test]
    fn barrier_uses_exact_membership_and_failure_policy() {
        let barrier = Barrier {
            revision: 1,
            members: ["lane-1".to_owned(), "lane-2".to_owned()]
                .into_iter()
                .collect(),
            quorum: 2,
            fail_on_failed_lane: true,
        };
        let lanes = BTreeMap::from([
            ("lane-1".to_owned(), LaneState::Passed),
            ("lane-2".to_owned(), LaneState::Failed),
        ]);
        assert_eq!(barrier.evaluate(&lanes), BarrierResult::Failed);
    }

    #[test]
    fn shared_paths_are_casefolded_and_overlap_is_detected() {
        let parent = canonical_shared_path("Src\\Module").expect("parent");
        let child = canonical_shared_path("src/module/file.rs").expect("child");
        assert!(paths_overlap(&parent, &child));
        assert_eq!(
            canonical_shared_path("../escape"),
            Err(WorkflowError::UnsafePath)
        );
        assert_eq!(
            canonical_shared_path("소스/file"),
            Err(WorkflowError::UnsafePath)
        );
    }

    #[test]
    fn goal_requires_evidence_and_terminal_judge() {
        let mut goal = Goal {
            goal_id: "goal-1".to_owned(),
            aggregation: GoalAggregation::And,
            quorum: None,
            children: BTreeMap::from([
                ("child-1".to_owned(), GoalState::Complete),
                ("child-2".to_owned(), GoalState::Complete),
            ]),
            evidence_verified: ["child-1".to_owned(), "child-2".to_owned()]
                .into_iter()
                .collect(),
            judge_verified: false,
        };
        assert_eq!(goal.evaluate(), GoalState::Verifying);
        goal.judge_verified = true;
        assert_eq!(goal.evaluate(), GoalState::Complete);
    }

    #[test]
    fn judge_invocation_policy_is_closed_and_strict_terminal_is_not_optional() {
        use super::{JudgeInvocationPolicy, JudgeRoute};

        for policy in [
            JudgeInvocationPolicy::Explicit,
            JudgeInvocationPolicy::Implicit,
        ] {
            assert!(policy.permits(JudgeRoute::StrictTerminalAcceptance));
            for excluded in [
                JudgeRoute::SimpleQuestion,
                JudgeRoute::ReadOnly,
                JudgeRoute::FormatOnly,
                JudgeRoute::SchedulerTick,
                JudgeRoute::Heartbeat,
                JudgeRoute::Retry,
                JudgeRoute::DeterministicFailure,
                JudgeRoute::UnsupportedOrUnattestedHost,
            ] {
                assert!(!policy.permits(excluded));
            }
        }
        assert!(!JudgeInvocationPolicy::Explicit.permits(JudgeRoute::MaterialRisk));
        assert!(JudgeInvocationPolicy::Implicit.permits(JudgeRoute::MaterialRisk));
    }

    #[test]
    fn nested_budget_refunds_exactly_once() {
        let mut budget = GoalBudget::new(10);
        budget.allocate("child-1", 7).expect("allocate");
        budget.refund("child-1", 2).expect("refund");
        assert_eq!(budget.available, 5);
        assert_eq!(
            budget.refund("child-1", 1),
            Err(WorkflowError::BudgetRejected)
        );
    }
}
