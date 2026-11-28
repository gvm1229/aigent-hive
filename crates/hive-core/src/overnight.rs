//! Overnight loop contracts: inputs, iteration state, stop conditions and recovery (0.17.0).
//!
//! Hive owns only the goal, scorecard, budget and deterministic state. Every model call and
//! worker process belongs to the host: Claude continues inside the interactive host with its
//! official `/loop`, scheduled tasks, hooks or subagents; Codex runs as an official
//! non-interactive worker launched by a host-owned handler. Antigravity and the retired Gemini
//! CLI are never workers or critics (Google's Antigravity terms). This module performs no I/O,
//! launches no process and calls no provider. Worker authority comes from signed summons
//! (ADR-0027); the file inbox only carries plan envelopes and results.

use crate::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Current overnight-loop document schema version.
pub const OVERNIGHT_SCHEMA_VERSION: u32 = 1;
/// Largest number of iterations one loop may run.
pub const MAX_ITERATIONS: u32 = 200;
/// Longest wall-clock budget: one day.
pub const MAX_WALL_SECONDS: u64 = 24 * 60 * 60;
/// Shortest wall-clock budget.
pub const MIN_WALL_SECONDS: u64 = 60;
/// Largest scorecard.
pub const MAX_SCORE_ITEMS: usize = 32;
/// Largest decision-point list.
pub const MAX_DECISION_POINTS: usize = 32;
/// Weight tolerance when checking that scorecard weights sum to one.
const WEIGHT_TOLERANCE: f64 = 1e-6;

/// Goal document reference. Hive stores its digest, never a model-written rewrite.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopGoal {
    pub goal_id: String,
    pub title: String,
    pub document_digest: String,
    pub acceptance: Vec<String>,
}

/// What one scorecard item measures.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScoreKind {
    Build,
    Test,
    Lint,
    Accessibility,
    VisualDiff,
    Critique,
    Custom,
}

impl ScoreKind {
    /// Deterministic checks run the same way every time; critique comes from another provider.
    #[must_use]
    pub const fn deterministic(self) -> bool {
        !matches!(self, Self::Critique)
    }
}

/// One scored item: a deterministic check or a critique, scored 0.00–1.00.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScoreItem {
    pub id: String,
    pub kind: ScoreKind,
    pub weight: f64,
    pub pass_threshold: f64,
}

/// Weighted scorecard. Weights sum to 1.00.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scorecard {
    pub items: Vec<ScoreItem>,
}

/// Iteration, time and usage limits plus the plateau and repeated-failure rules.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopBudget {
    pub max_iterations: u32,
    pub max_wall_seconds: u64,
    /// Stop before the host's remaining usage falls below this percent.
    pub usage_floor_percent: u8,
    /// Stop after this many consecutive iterations without a higher total score.
    pub plateau_iterations: u32,
    /// Stop after the same failing check set repeats this many times.
    pub repeated_failure_limit: u32,
}

/// A point where a person must decide; the loop never decides it alone.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionPoint {
    pub id: String,
    pub question: String,
    /// The step cannot be undone (deleting data, publishing, pushing, merging, paying).
    pub irreversible: bool,
}

/// Complete, validated input for one overnight loop.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopInput {
    pub schema_version: u32,
    pub goal: LoopGoal,
    pub scorecard: Scorecard,
    pub budget: LoopBudget,
    pub decision_points: Vec<DecisionPoint>,
}

fn valid_slug(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    })
}

/// A score is a finite number from 0.00 to 1.00 with at most two decimals.
#[must_use]
pub fn valid_score(value: f64) -> bool {
    value.is_finite()
        && (0.0..=1.0).contains(&value)
        && ((value * 100.0).round() - value * 100.0).abs() < 1e-9
}

fn text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}

impl LoopInput {
    /// Reject malformed goals, scorecards, budgets and decision points.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason for the first invalid field.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != OVERNIGHT_SCHEMA_VERSION {
            return Err(format!(
                "loop input schema_version must be {OVERNIGHT_SCHEMA_VERSION}"
            ));
        }
        let goal = &self.goal;
        if !valid_slug(&goal.goal_id)
            || !text(&goal.title, 200)
            || !valid_digest(&goal.document_digest)
        {
            return Err(
                "goal needs a slug id, a title and the sha256 digest of the goal document".into(),
            );
        }
        if goal.acceptance.is_empty()
            || goal.acceptance.len() > 32
            || !goal.acceptance.iter().all(|item| text(item, 400))
        {
            return Err("goal needs 1 to 32 acceptance statements".into());
        }
        let items = &self.scorecard.items;
        if items.is_empty() || items.len() > MAX_SCORE_ITEMS {
            return Err(format!("scorecard needs 1 to {MAX_SCORE_ITEMS} items"));
        }
        let mut ids = BTreeSet::new();
        for item in items {
            if !valid_slug(&item.id) || !ids.insert(item.id.as_str()) {
                return Err(format!(
                    "scorecard item id is invalid or duplicated: {}",
                    item.id
                ));
            }
            if !(item.weight.is_finite() && item.weight > 0.0 && item.weight <= 1.0) {
                return Err(format!(
                    "scorecard weight must be above 0 and at most 1: {}",
                    item.id
                ));
            }
            if !valid_score(item.pass_threshold) {
                return Err(format!(
                    "pass threshold must be 0.00–1.00 with two decimals: {}",
                    item.id
                ));
            }
        }
        let total: f64 = items.iter().map(|item| item.weight).sum();
        if (total - 1.0).abs() > WEIGHT_TOLERANCE {
            return Err(format!(
                "scorecard weights must sum to 1.00, got {total:.4}"
            ));
        }
        if !items.iter().any(|item| item.kind.deterministic()) {
            return Err("scorecard needs at least one deterministic check".into());
        }
        let budget = &self.budget;
        if !(1..=MAX_ITERATIONS).contains(&budget.max_iterations) {
            return Err(format!("max_iterations must be 1 to {MAX_ITERATIONS}"));
        }
        if !(MIN_WALL_SECONDS..=MAX_WALL_SECONDS).contains(&budget.max_wall_seconds) {
            return Err(format!(
                "max_wall_seconds must be {MIN_WALL_SECONDS} to {MAX_WALL_SECONDS}"
            ));
        }
        if budget.usage_floor_percent > 99 {
            return Err("usage_floor_percent must be 0 to 99".into());
        }
        if !(1..=20).contains(&budget.plateau_iterations)
            || !(1..=20).contains(&budget.repeated_failure_limit)
        {
            return Err("plateau_iterations and repeated_failure_limit must be 1 to 20".into());
        }
        if self.decision_points.len() > MAX_DECISION_POINTS {
            return Err(format!("at most {MAX_DECISION_POINTS} decision points"));
        }
        let mut decision_ids = BTreeSet::new();
        for point in &self.decision_points {
            if !valid_slug(&point.id)
                || !decision_ids.insert(point.id.as_str())
                || !text(&point.question, 400)
            {
                return Err(format!(
                    "decision point is invalid or duplicated: {}",
                    point.id
                ));
            }
        }
        Ok(())
    }

    /// Stable digest of the validated input; every state record binds to it.
    ///
    /// # Errors
    ///
    /// Returns an error when the input cannot be serialized.
    pub fn digest(&self) -> Result<String, String> {
        serde_json::to_vec(self)
            .map(|bytes| sha256_digest(&bytes))
            .map_err(|error| error.to_string())
    }

    /// Weighted total of one iteration's scores; missing items count as zero.
    #[must_use]
    pub fn total(&self, scores: &BTreeMap<String, f64>) -> f64 {
        let total: f64 = self
            .scorecard
            .items
            .iter()
            .map(|item| item.weight * scores.get(&item.id).copied().unwrap_or(0.0))
            .sum();
        (total * 100.0).round() / 100.0
    }

    /// Items whose score is below their pass threshold, sorted by id.
    #[must_use]
    pub fn failing(&self, scores: &BTreeMap<String, f64>) -> Vec<String> {
        self.scorecard
            .items
            .iter()
            .filter(|item| scores.get(&item.id).copied().unwrap_or(0.0) < item.pass_threshold)
            .map(|item| item.id.clone())
            .collect()
    }
}

/// Which host runs the work in one iteration.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Claude,
    Codex,
}

/// Step reached inside one iteration.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    /// The host's official continuation started this iteration.
    Continued,
    /// Hive prepared the plan envelope.
    Planned,
    /// The worker returned a receipt.
    Worked,
    /// Deterministic checks were scored.
    Checked,
    /// Another provider's critique was recorded.
    Critiqued,
    /// Hive judged the iteration.
    Judged,
    /// The worker receipt is missing after its lease; never re-dispatched automatically.
    DispatchUncertain,
}

/// Why a loop stopped.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "reason")]
pub enum StopReason {
    /// Every check passed and no critique was severe.
    Passed,
    /// The iteration budget is spent.
    IterationBudget,
    /// The wall-clock budget is spent.
    TimeBudget,
    /// The host's remaining usage reached the floor.
    UsageFloor { remaining_percent: u8 },
    /// The total score did not rise for the plateau count.
    Plateau,
    /// The same failing checks repeated the limit.
    RepeatedFailure { failing: Vec<String> },
    /// The next step cannot be undone and needs a person.
    IrreversibleStep { decision: String },
    /// A person stopped the loop.
    Cancelled,
    /// A worker receipt went missing; a person must confirm a safe reclaim.
    DispatchUncertain { iteration: u32 },
}

/// One iteration's durable record.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Iteration {
    pub number: u32,
    /// Duplicate-call guard: a second continuation or receipt with this key is refused.
    pub dedupe_key: String,
    pub phase: Phase,
    pub worker: Provider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_digest: Option<String>,
    #[serde(default)]
    pub scores: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub critic: Option<Provider>,
    #[serde(default)]
    pub severe_critiques: u32,
    pub started_at: u64,
}

/// Loop lifecycle.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "status")]
pub enum LoopStatus {
    Running,
    Stopped(StopReason),
}

/// Whole loop state, bound to one validated input.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopRun {
    pub schema_version: u32,
    pub loop_id: String,
    pub input_digest: String,
    pub branch: String,
    pub started_at: u64,
    pub status: LoopStatus,
    pub iterations: Vec<Iteration>,
}

/// One state-changing event the host or a person reports.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "event")]
pub enum LoopEvent {
    /// The host's official continuation started an iteration.
    Continued {
        dedupe_key: String,
        worker: Provider,
        now: u64,
    },
    /// Hive prepared the plan envelope for the current iteration.
    Planned { plan_digest: String },
    /// The worker's host returned a receipt for exactly this iteration.
    Receipt {
        dedupe_key: String,
        receipt_digest: String,
        changes_digest: String,
    },
    /// The receipt did not arrive within its lease.
    ReceiptMissing { dedupe_key: String },
    /// Deterministic checks produced scores.
    Checked { scores: BTreeMap<String, f64> },
    /// Another provider critiqued the result.
    Critiqued {
        critic: Provider,
        score: f64,
        severe: u32,
    },
    /// A person stopped the loop.
    Cancel,
}

/// Branch the loop may commit to. Push, merge and release stay with a person.
#[must_use]
pub fn loop_branch(loop_id: &str) -> String {
    format!("loop/{loop_id}")
}

/// Whether the loop may commit to `branch`: only its own loop branch.
#[must_use]
pub fn commit_allowed(run: &LoopRun, branch: &str) -> bool {
    branch == run.branch && branch.starts_with("loop/")
}

/// Duplicate-call guard for one iteration.
#[must_use]
pub fn dedupe_key(loop_id: &str, input_digest: &str, number: u32) -> String {
    sha256_digest(format!("{loop_id}\0{input_digest}\0{number}").as_bytes())
}

impl LoopRun {
    /// Start a loop for a validated input.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid input or loop id.
    pub fn start(loop_id: &str, input: &LoopInput, now: u64) -> Result<Self, String> {
        input.validate()?;
        if !valid_slug(loop_id) {
            return Err("loop id must be a lowercase slug".into());
        }
        Ok(Self {
            schema_version: OVERNIGHT_SCHEMA_VERSION,
            loop_id: loop_id.to_owned(),
            input_digest: input.digest()?,
            branch: loop_branch(loop_id),
            started_at: now,
            status: LoopStatus::Running,
            iterations: Vec::new(),
        })
    }

    /// The dedupe key the next continuation must present.
    #[must_use]
    pub fn next_dedupe_key(&self) -> String {
        dedupe_key(&self.loop_id, &self.input_digest, self.next_number())
    }

    fn next_number(&self) -> u32 {
        u32::try_from(self.iterations.len())
            .unwrap_or(u32::MAX)
            .saturating_add(1)
    }

    fn current(&mut self) -> Result<&mut Iteration, String> {
        self.iterations
            .last_mut()
            .ok_or_else(|| "no iteration has started".to_owned())
    }

    /// Apply one event. Out-of-order, duplicate and foreign events are refused unchanged.
    ///
    /// # Errors
    ///
    /// Returns why the event was refused; the state is not modified.
    #[allow(clippy::too_many_lines)]
    pub fn apply(&mut self, input: &LoopInput, event: &LoopEvent) -> Result<(), String> {
        if input.digest()? != self.input_digest {
            return Err("loop input changed after start".into());
        }
        if let LoopStatus::Stopped(reason) = &self.status {
            return Err(format!(
                "loop already stopped: {}",
                serde_json::to_string(reason).unwrap_or_default()
            ));
        }
        let mut next = self.clone();
        match event {
            LoopEvent::Continued {
                dedupe_key,
                worker,
                now,
            } => {
                if next
                    .iterations
                    .iter()
                    .any(|iteration| &iteration.dedupe_key == dedupe_key)
                {
                    return Err("duplicate continuation for an iteration that already ran".into());
                }
                if next
                    .iterations
                    .last()
                    .is_some_and(|last| last.phase != Phase::Judged)
                {
                    return Err("the previous iteration has not been judged".into());
                }
                if dedupe_key != &next.next_dedupe_key() {
                    return Err("continuation key does not belong to the next iteration".into());
                }
                let number = next.next_number();
                next.iterations.push(Iteration {
                    number,
                    dedupe_key: dedupe_key.clone(),
                    phase: Phase::Continued,
                    worker: *worker,
                    plan_digest: None,
                    receipt_digest: None,
                    changes_digest: None,
                    scores: BTreeMap::new(),
                    total: None,
                    critic: None,
                    severe_critiques: 0,
                    started_at: *now,
                });
            }
            LoopEvent::Planned { plan_digest } => {
                let current = next.current()?;
                if current.phase != Phase::Continued || !valid_digest(plan_digest) {
                    return Err("a plan envelope follows a continuation and needs a digest".into());
                }
                current.plan_digest = Some(plan_digest.clone());
                current.phase = Phase::Planned;
            }
            LoopEvent::Receipt {
                dedupe_key,
                receipt_digest,
                changes_digest,
            } => {
                let current = next.current()?;
                if &current.dedupe_key != dedupe_key {
                    return Err("receipt belongs to another iteration".into());
                }
                if current.phase != Phase::Planned {
                    return Err("duplicate or out-of-order worker receipt".into());
                }
                if !valid_digest(receipt_digest) || !valid_digest(changes_digest) {
                    return Err("receipt needs receipt and changes digests".into());
                }
                current.receipt_digest = Some(receipt_digest.clone());
                current.changes_digest = Some(changes_digest.clone());
                current.phase = Phase::Worked;
            }
            LoopEvent::ReceiptMissing { dedupe_key } => {
                let current = next.current()?;
                if &current.dedupe_key != dedupe_key || current.phase != Phase::Planned {
                    return Err("only a planned iteration can lose its receipt".into());
                }
                current.phase = Phase::DispatchUncertain;
                let number = current.number;
                next.status =
                    LoopStatus::Stopped(StopReason::DispatchUncertain { iteration: number });
            }
            LoopEvent::Checked { scores } => {
                let deterministic: BTreeSet<&str> = input
                    .scorecard
                    .items
                    .iter()
                    .filter(|item| item.kind.deterministic())
                    .map(|item| item.id.as_str())
                    .collect();
                if scores.keys().any(|id| !deterministic.contains(id.as_str()))
                    || !scores.values().all(|score| valid_score(*score))
                {
                    return Err("checks may score only deterministic items with 0.00–1.00".into());
                }
                let current = next.current()?;
                if current.phase != Phase::Worked {
                    return Err("checks follow a worker receipt".into());
                }
                current.scores.extend(scores.clone());
                current.phase = Phase::Checked;
            }
            LoopEvent::Critiqued {
                critic,
                score,
                severe,
            } => {
                let critique_ids: Vec<&str> = input
                    .scorecard
                    .items
                    .iter()
                    .filter(|item| !item.kind.deterministic())
                    .map(|item| item.id.as_str())
                    .collect();
                let current = next.current()?;
                if current.phase != Phase::Checked {
                    return Err("critique follows the checks".into());
                }
                if *critic == current.worker {
                    return Err("critique must come from another provider than the worker".into());
                }
                if !valid_score(*score) {
                    return Err("critique score must be 0.00–1.00".into());
                }
                for id in critique_ids {
                    current.scores.insert(id.to_owned(), *score);
                }
                current.critic = Some(*critic);
                current.severe_critiques = *severe;
                current.phase = Phase::Critiqued;
            }
            LoopEvent::Cancel => {
                next.status = LoopStatus::Stopped(StopReason::Cancelled);
            }
        }
        *self = next;
        Ok(())
    }

    /// Judge the current iteration and decide whether to stop. `remaining_percent` is the
    /// host's remaining usage; `irreversible_next` names a decision point the next step needs.
    ///
    /// # Errors
    ///
    /// Returns an error unless the current iteration has been critiqued.
    pub fn judge(
        &mut self,
        input: &LoopInput,
        now: u64,
        remaining_percent: Option<u8>,
        irreversible_next: Option<&str>,
    ) -> Result<Option<StopReason>, String> {
        let total = {
            let current = self.current()?;
            if current.phase != Phase::Critiqued {
                return Err("judge follows the critique".into());
            }
            input.total(&current.scores)
        };
        if let Some(current) = self.iterations.last_mut() {
            current.total = Some(total);
            current.phase = Phase::Judged;
        }
        let reason = stop_condition(self, input, now, remaining_percent, irreversible_next);
        if let Some(reason) = &reason {
            self.status = LoopStatus::Stopped(reason.clone());
        }
        Ok(reason)
    }
}

/// The first stop condition that holds after a judged iteration, in priority order.
#[must_use]
pub fn stop_condition(
    run: &LoopRun,
    input: &LoopInput,
    now: u64,
    remaining_percent: Option<u8>,
    irreversible_next: Option<&str>,
) -> Option<StopReason> {
    let judged: Vec<&Iteration> = run
        .iterations
        .iter()
        .filter(|iteration| iteration.phase == Phase::Judged)
        .collect();
    let last = judged.last()?;
    let budget = &input.budget;
    if input.failing(&last.scores).is_empty() && last.severe_critiques == 0 {
        return Some(StopReason::Passed);
    }
    if let Some(decision) = irreversible_next {
        if input
            .decision_points
            .iter()
            .any(|point| point.id == decision && point.irreversible)
        {
            return Some(StopReason::IrreversibleStep {
                decision: decision.to_owned(),
            });
        }
    }
    if let Some(remaining) = remaining_percent {
        if remaining <= budget.usage_floor_percent {
            return Some(StopReason::UsageFloor {
                remaining_percent: remaining,
            });
        }
    }
    if now.saturating_sub(run.started_at) >= budget.max_wall_seconds {
        return Some(StopReason::TimeBudget);
    }
    if judged.len() >= budget.max_iterations as usize {
        return Some(StopReason::IterationBudget);
    }
    let failing = input.failing(&last.scores);
    let limit = budget.repeated_failure_limit as usize;
    if judged.len() >= limit
        && judged[judged.len() - limit..]
            .iter()
            .all(|iteration| input.failing(&iteration.scores) == failing)
    {
        return Some(StopReason::RepeatedFailure { failing });
    }
    let plateau = budget.plateau_iterations as usize;
    if judged.len() > plateau {
        let best_before = judged[..judged.len() - plateau]
            .iter()
            .filter_map(|iteration| iteration.total)
            .fold(f64::MIN, f64::max);
        if judged[judged.len() - plateau..]
            .iter()
            .all(|iteration| iteration.total.unwrap_or(0.0) <= best_before)
        {
            return Some(StopReason::Plateau);
        }
    }
    None
}

/// What a restarted host may do with one interrupted loop.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "action")]
pub enum RecoveryAction {
    /// Nothing is in flight; the next continuation may start with this key.
    ContinueWith { dedupe_key: String },
    /// A receipt found on disk completes the interrupted iteration; apply it, do not rerun.
    AdoptReceipt { iteration: u32 },
    /// The iteration is still waiting for its receipt within the lease.
    WaitForReceipt { iteration: u32 },
    /// The receipt is missing after the lease; a person must confirm a safe reclaim.
    NeedsReclaimConfirmation { iteration: u32 },
    /// The loop already stopped; only the morning report remains.
    Stopped,
}

/// Reconcile a loop after an interruption with the receipts present on disk. Finished
/// iterations are never rerun, and nothing reruns automatically without a reclaim proof.
#[must_use]
pub fn recover(
    run: &LoopRun,
    receipts_on_disk: &BTreeSet<String>,
    now: u64,
    lease_seconds: u64,
) -> RecoveryAction {
    if matches!(run.status, LoopStatus::Stopped(_)) {
        return RecoveryAction::Stopped;
    }
    let Some(last) = run.iterations.last() else {
        return RecoveryAction::ContinueWith {
            dedupe_key: run.next_dedupe_key(),
        };
    };
    match last.phase {
        Phase::Judged => RecoveryAction::ContinueWith {
            dedupe_key: run.next_dedupe_key(),
        },
        Phase::Continued | Phase::Planned if receipts_on_disk.contains(&last.dedupe_key) => {
            RecoveryAction::AdoptReceipt {
                iteration: last.number,
            }
        }
        Phase::Continued | Phase::Planned
            if now.saturating_sub(last.started_at) < lease_seconds =>
        {
            RecoveryAction::WaitForReceipt {
                iteration: last.number,
            }
        }
        Phase::Continued | Phase::Planned | Phase::DispatchUncertain => {
            RecoveryAction::NeedsReclaimConfirmation {
                iteration: last.number,
            }
        }
        // Checks and critique are recomputed by the host from the recorded receipt.
        Phase::Worked | Phase::Checked | Phase::Critiqued => RecoveryAction::AdoptReceipt {
            iteration: last.number,
        },
    }
}

/// How the host cancelled in-flight work after a stop.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CancelProof {
    /// The host returned an official cancel receipt.
    Receipt,
    /// The host has no official cancel; the in-flight work is not proven stopped.
    Unproven,
    /// Nothing was in flight.
    NothingInFlight,
}

/// Cancellation outcome for a stopped loop: no new envelope, and an honest proof label.
#[must_use]
pub fn cancel_proof(run: &LoopRun, host_cancel_receipt: bool) -> CancelProof {
    let in_flight = run
        .iterations
        .last()
        .is_some_and(|iteration| matches!(iteration.phase, Phase::Continued | Phase::Planned));
    match (in_flight, host_cancel_receipt) {
        (false, _) => CancelProof::NothingInFlight,
        (true, true) => CancelProof::Receipt,
        (true, false) => CancelProof::Unproven,
    }
}

/// Morning report in Markdown: outcome, score trend, waiting decisions, changes and usage.
#[must_use]
pub fn morning_report(
    run: &LoopRun,
    input: &LoopInput,
    korean: bool,
    remaining_percent: Option<u8>,
) -> String {
    let status = match &run.status {
        LoopStatus::Running => "running".to_owned(),
        LoopStatus::Stopped(reason) => serde_json::to_value(reason)
            .ok()
            .and_then(|value| {
                value
                    .get("reason")
                    .and_then(|reason| reason.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "stopped".to_owned()),
    };
    let mut lines = Vec::new();
    let heading = if korean {
        "아침 보고서"
    } else {
        "Morning report"
    };
    lines.push(format!("# {heading}: {}", input.goal.title));
    lines.push(String::new());
    let labels = if korean {
        [
            "상태",
            "반복",
            "점수 추이",
            "대기 결정",
            "변경",
            "남은 사용량",
        ]
    } else {
        [
            "Status",
            "Iterations",
            "Score trend",
            "Waiting decisions",
            "Changes",
            "Remaining usage",
        ]
    };
    lines.push(format!("- {}: `{status}`", labels[0]));
    lines.push(format!(
        "- {}: {}/{}",
        labels[1],
        run.iterations.len(),
        input.budget.max_iterations
    ));
    let trend: Vec<String> = run
        .iterations
        .iter()
        .filter_map(|iteration| iteration.total.map(|total| format!("{total:.2}")))
        .collect();
    lines.push(format!(
        "- {}: {}",
        labels[2],
        if trend.is_empty() {
            "-".to_owned()
        } else {
            trend.join(" → ")
        }
    ));
    let waiting: Vec<&str> = match &run.status {
        LoopStatus::Stopped(StopReason::IrreversibleStep { decision }) => vec![decision.as_str()],
        LoopStatus::Stopped(StopReason::DispatchUncertain { .. }) => vec!["reclaim-confirmation"],
        _ => Vec::new(),
    };
    lines.push(format!(
        "- {}: {}",
        labels[3],
        if waiting.is_empty() {
            "-".to_owned()
        } else {
            waiting.join(", ")
        }
    ));
    let changes = run
        .iterations
        .iter()
        .filter(|iteration| iteration.changes_digest.is_some())
        .count();
    lines.push(format!(
        "- {}: {changes} ({} `{}`)",
        labels[4],
        if korean { "브랜치" } else { "branch" },
        run.branch
    ));
    lines.push(format!(
        "- {}: {}",
        labels[5],
        remaining_percent.map_or_else(|| "-".to_owned(), |value| format!("{value}%"))
    ));
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod terms_tests {
    use super::Provider;

    #[test]
    fn overnight_provider_refuses_gemini_and_antigravity() {
        for name in ["gemini", "antigravity"] {
            assert!(serde_json::from_str::<Provider>(&format!("\"{name}\"")).is_err());
        }
        assert_eq!(
            serde_json::from_str::<Provider>("\"codex\"").unwrap(),
            Provider::Codex
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(seed: &str) -> String {
        sha256_digest(seed.as_bytes())
    }

    fn input() -> LoopInput {
        LoopInput {
            schema_version: 1,
            goal: LoopGoal {
                goal_id: "static-site".into(),
                title: "One static website".into(),
                document_digest: digest("goal"),
                acceptance: vec!["builds".into(), "passes accessibility".into()],
            },
            scorecard: Scorecard {
                items: vec![
                    ScoreItem {
                        id: "build".into(),
                        kind: ScoreKind::Build,
                        weight: 0.4,
                        pass_threshold: 1.0,
                    },
                    ScoreItem {
                        id: "a11y".into(),
                        kind: ScoreKind::Accessibility,
                        weight: 0.3,
                        pass_threshold: 0.9,
                    },
                    ScoreItem {
                        id: "review".into(),
                        kind: ScoreKind::Critique,
                        weight: 0.3,
                        pass_threshold: 0.8,
                    },
                ],
            },
            budget: LoopBudget {
                max_iterations: 10,
                max_wall_seconds: 8 * 60 * 60,
                usage_floor_percent: 15,
                plateau_iterations: 3,
                repeated_failure_limit: 3,
            },
            decision_points: vec![DecisionPoint {
                id: "publish".into(),
                question: "Publish the site?".into(),
                irreversible: true,
            }],
        }
    }

    fn iterate(
        run: &mut LoopRun,
        input: &LoopInput,
        now: u64,
        scores: (f64, f64, f64),
        severe: u32,
    ) -> Option<StopReason> {
        let key = run.next_dedupe_key();
        run.apply(
            input,
            &LoopEvent::Continued {
                dedupe_key: key.clone(),
                worker: Provider::Codex,
                now,
            },
        )
        .unwrap();
        run.apply(
            input,
            &LoopEvent::Planned {
                plan_digest: digest(&format!("plan{now}")),
            },
        )
        .unwrap();
        run.apply(
            input,
            &LoopEvent::Receipt {
                dedupe_key: key,
                receipt_digest: digest(&format!("r{now}")),
                changes_digest: digest(&format!("c{now}")),
            },
        )
        .unwrap();
        let checks = BTreeMap::from([
            ("build".to_owned(), scores.0),
            ("a11y".to_owned(), scores.1),
        ]);
        run.apply(input, &LoopEvent::Checked { scores: checks })
            .unwrap();
        run.apply(
            input,
            &LoopEvent::Critiqued {
                critic: Provider::Claude,
                score: scores.2,
                severe,
            },
        )
        .unwrap();
        run.judge(input, now, Some(80), None).unwrap()
    }

    #[test]
    fn invalid_scorecards_and_budgets_are_refused() {
        assert!(input().validate().is_ok());
        let mut bad = input();
        bad.scorecard.items[0].weight = 0.5;
        assert!(bad.validate().unwrap_err().contains("sum to 1.00"));
        let mut bad = input();
        bad.scorecard.items[1].pass_threshold = 0.955;
        assert!(bad.validate().unwrap_err().contains("two decimals"));
        let mut bad = input();
        bad.scorecard.items[1].id = "build".into();
        assert!(bad.validate().unwrap_err().contains("duplicated"));
        let mut bad = input();
        bad.scorecard.items = vec![ScoreItem {
            id: "review".into(),
            kind: ScoreKind::Critique,
            weight: 1.0,
            pass_threshold: 0.5,
        }];
        assert!(bad.validate().unwrap_err().contains("deterministic"));
        let mut bad = input();
        bad.budget.max_iterations = 0;
        assert!(bad.validate().is_err());
        let mut bad = input();
        bad.budget.max_wall_seconds = 10;
        assert!(bad.validate().is_err());
        let mut bad = input();
        bad.budget.usage_floor_percent = 100;
        assert!(bad.validate().is_err());
        let mut bad = input();
        bad.goal.document_digest = "not-a-digest".into();
        assert!(bad.validate().is_err());
        let mut bad = input();
        bad.scorecard.items[0].weight = f64::NAN;
        assert!(bad.validate().is_err());
        assert!(serde_json::from_str::<LoopBudget>(r#"{"max_iterations":1,"max_wall_seconds":60,"usage_floor_percent":0,"plateau_iterations":1,"repeated_failure_limit":1,"extra":1}"#).is_err());
    }

    #[test]
    fn three_iterations_advance_and_duplicates_are_refused() {
        let input = input();
        let mut run = LoopRun::start("site-1", &input, 1000).unwrap();
        assert_eq!(run.branch, "loop/site-1");
        assert!(commit_allowed(&run, "loop/site-1"));
        assert!(!commit_allowed(&run, "develop"));
        assert_eq!(iterate(&mut run, &input, 1100, (1.0, 0.5, 0.5), 1), None);
        let first_key = run.iterations[0].dedupe_key.clone();
        let duplicate = run.apply(
            &input,
            &LoopEvent::Continued {
                dedupe_key: first_key.clone(),
                worker: Provider::Codex,
                now: 1150,
            },
        );
        assert!(duplicate.unwrap_err().contains("duplicate continuation"));
        assert_eq!(iterate(&mut run, &input, 1200, (1.0, 0.7, 0.6), 0), None);
        // A restart after judged iterations continues with a new key; finished ones never rerun.
        let finished: BTreeSet<String> = run
            .iterations
            .iter()
            .map(|iteration| iteration.dedupe_key.clone())
            .collect();
        match recover(&run, &finished, 99_999, 900) {
            RecoveryAction::ContinueWith { dedupe_key } => assert!(!finished.contains(&dedupe_key)),
            other => panic!("unexpected recovery {other:?}"),
        }
        let key = run.next_dedupe_key();
        run.apply(
            &input,
            &LoopEvent::Continued {
                dedupe_key: key.clone(),
                worker: Provider::Codex,
                now: 1300,
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Planned {
                plan_digest: digest("p3"),
            },
        )
        .unwrap();
        let receipt = LoopEvent::Receipt {
            dedupe_key: key.clone(),
            receipt_digest: digest("r3"),
            changes_digest: digest("c3"),
        };
        run.apply(&input, &receipt).unwrap();
        assert!(run
            .apply(&input, &receipt)
            .unwrap_err()
            .contains("duplicate"));
        let foreign = LoopEvent::Receipt {
            dedupe_key: first_key,
            receipt_digest: digest("x"),
            changes_digest: digest("y"),
        };
        assert!(run.apply(&input, &foreign).is_err());
        run.apply(
            &input,
            &LoopEvent::Checked {
                scores: BTreeMap::from([("build".into(), 1.0), ("a11y".into(), 0.95)]),
            },
        )
        .unwrap();
        assert!(run
            .apply(
                &input,
                &LoopEvent::Critiqued {
                    critic: Provider::Codex,
                    score: 0.9,
                    severe: 0
                }
            )
            .unwrap_err()
            .contains("another provider"));
        run.apply(
            &input,
            &LoopEvent::Critiqued {
                critic: Provider::Claude,
                score: 0.9,
                severe: 0,
            },
        )
        .unwrap();
        assert_eq!(
            run.judge(&input, 1400, Some(80), None).unwrap(),
            Some(StopReason::Passed)
        );
        assert_eq!(run.iterations.len(), 3);
        assert_eq!(run.iterations[2].total, Some(0.96));
        let report = morning_report(&run, &input, false, Some(80));
        assert!(report.contains("`passed`"));
        assert!(report.contains("0.70 → 0.79 → 0.96"));
        assert!(run
            .apply(&input, &LoopEvent::Cancel)
            .unwrap_err()
            .contains("already stopped"));
    }

    #[test]
    fn lost_receipt_is_dispatch_uncertain_and_never_redispatched() {
        let input = input();
        let mut run = LoopRun::start("site-2", &input, 1000).unwrap();
        let key = run.next_dedupe_key();
        run.apply(
            &input,
            &LoopEvent::Continued {
                dedupe_key: key.clone(),
                worker: Provider::Codex,
                now: 1000,
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Planned {
                plan_digest: digest("p"),
            },
        )
        .unwrap();
        assert_eq!(
            recover(&run, &BTreeSet::new(), 1100, 900),
            RecoveryAction::WaitForReceipt { iteration: 1 }
        );
        assert_eq!(
            recover(&run, &BTreeSet::new(), 3000, 900),
            RecoveryAction::NeedsReclaimConfirmation { iteration: 1 }
        );
        assert_eq!(
            recover(&run, &BTreeSet::from([key.clone()]), 3000, 900),
            RecoveryAction::AdoptReceipt { iteration: 1 }
        );
        run.apply(
            &input,
            &LoopEvent::ReceiptMissing {
                dedupe_key: key.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            run.status,
            LoopStatus::Stopped(StopReason::DispatchUncertain { iteration: 1 })
        );
        assert!(run
            .apply(
                &input,
                &LoopEvent::Continued {
                    dedupe_key: run.next_dedupe_key(),
                    worker: Provider::Codex,
                    now: 3100
                }
            )
            .is_err());
        assert_eq!(
            recover(&run, &BTreeSet::new(), 3100, 900),
            RecoveryAction::Stopped
        );
        assert!(morning_report(&run, &input, true, None).contains("reclaim-confirmation"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn six_stop_conditions_each_stop_the_loop() {
        let input = input();
        // 1. Passed.
        let mut run = LoopRun::start("a", &input, 0).unwrap();
        assert_eq!(
            iterate(&mut run, &input, 10, (1.0, 1.0, 1.0), 0),
            Some(StopReason::Passed)
        );
        // 2. Iteration budget.
        let mut small = input.clone();
        small.budget.max_iterations = 2;
        small.budget.plateau_iterations = 5;
        small.budget.repeated_failure_limit = 5;
        let mut run = LoopRun::start("b", &small, 0).unwrap();
        assert_eq!(iterate(&mut run, &small, 10, (0.5, 0.5, 0.5), 0), None);
        assert_eq!(
            iterate(&mut run, &small, 20, (0.6, 0.6, 0.6), 0),
            Some(StopReason::IterationBudget)
        );
        // 3. Time budget.
        let mut run = LoopRun::start("c", &input, 0).unwrap();
        assert_eq!(
            iterate(&mut run, &input, 9 * 60 * 60, (0.5, 0.5, 0.5), 0),
            Some(StopReason::TimeBudget)
        );
        // 4. Usage floor.
        let mut run = LoopRun::start("d", &input, 0).unwrap();
        let key = run.next_dedupe_key();
        run.apply(
            &input,
            &LoopEvent::Continued {
                dedupe_key: key.clone(),
                worker: Provider::Claude,
                now: 1,
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Planned {
                plan_digest: digest("p"),
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Receipt {
                dedupe_key: key,
                receipt_digest: digest("r"),
                changes_digest: digest("c"),
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Checked {
                scores: BTreeMap::from([("build".into(), 0.0)]),
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Critiqued {
                critic: Provider::Codex,
                score: 0.1,
                severe: 2,
            },
        )
        .unwrap();
        assert_eq!(
            run.judge(&input, 2, Some(10), None).unwrap(),
            Some(StopReason::UsageFloor {
                remaining_percent: 10
            })
        );
        // 5. Plateau and 6. repeated failure.
        let mut plateau = input.clone();
        plateau.budget.repeated_failure_limit = 10;
        let mut run = LoopRun::start("e", &plateau, 0).unwrap();
        assert_eq!(iterate(&mut run, &plateau, 10, (1.0, 0.8, 0.5), 0), None);
        assert_eq!(iterate(&mut run, &plateau, 20, (1.0, 0.7, 0.5), 0), None);
        assert_eq!(iterate(&mut run, &plateau, 30, (1.0, 0.6, 0.5), 0), None);
        assert_eq!(
            iterate(&mut run, &plateau, 40, (1.0, 0.8, 0.5), 0),
            Some(StopReason::Plateau)
        );
        let mut run = LoopRun::start("f", &input, 0).unwrap();
        assert_eq!(iterate(&mut run, &input, 10, (0.0, 0.5, 0.4), 0), None);
        assert_eq!(iterate(&mut run, &input, 20, (0.0, 0.6, 0.5), 0), None);
        assert_eq!(
            iterate(&mut run, &input, 30, (0.0, 0.7, 0.6), 0),
            Some(StopReason::RepeatedFailure {
                failing: vec!["build".into(), "a11y".into(), "review".into()]
            })
        );
        // Irreversible step.
        let mut run = LoopRun::start("g", &input, 0).unwrap();
        let key = run.next_dedupe_key();
        run.apply(
            &input,
            &LoopEvent::Continued {
                dedupe_key: key.clone(),
                worker: Provider::Codex,
                now: 1,
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Planned {
                plan_digest: digest("p"),
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Receipt {
                dedupe_key: key,
                receipt_digest: digest("r"),
                changes_digest: digest("c"),
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Checked {
                scores: BTreeMap::from([("build".into(), 1.0), ("a11y".into(), 0.5)]),
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Critiqued {
                critic: Provider::Claude,
                score: 0.5,
                severe: 0,
            },
        )
        .unwrap();
        assert_eq!(
            run.judge(&input, 2, Some(80), Some("publish")).unwrap(),
            Some(StopReason::IrreversibleStep {
                decision: "publish".into()
            })
        );
    }

    #[test]
    fn stop_creates_no_envelope_and_labels_cancel_proof_honestly() {
        let input = input();
        let mut run = LoopRun::start("h", &input, 0).unwrap();
        let key = run.next_dedupe_key();
        run.apply(
            &input,
            &LoopEvent::Continued {
                dedupe_key: key,
                worker: Provider::Codex,
                now: 1,
            },
        )
        .unwrap();
        run.apply(
            &input,
            &LoopEvent::Planned {
                plan_digest: digest("p"),
            },
        )
        .unwrap();
        assert_eq!(cancel_proof(&run, false), CancelProof::Unproven);
        assert_eq!(cancel_proof(&run, true), CancelProof::Receipt);
        run.apply(&input, &LoopEvent::Cancel).unwrap();
        assert_eq!(run.status, LoopStatus::Stopped(StopReason::Cancelled));
        assert!(
            run.apply(
                &input,
                &LoopEvent::Planned {
                    plan_digest: digest("q")
                }
            )
            .is_err(),
            "no new envelope after stop"
        );
        let fresh = LoopRun::start("i", &input, 0).unwrap();
        assert_eq!(cancel_proof(&fresh, false), CancelProof::NothingInFlight);
        let mut changed = input.clone();
        changed.budget.max_iterations = 3;
        let mut other = LoopRun::start("j", &input, 0).unwrap();
        assert!(other
            .apply(&changed, &LoopEvent::Cancel)
            .unwrap_err()
            .contains("input changed"));
    }
}
