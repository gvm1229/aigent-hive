//! Damage containment for worker runs (OV160-005).
//!
//! Pure data only: the least-privilege policy a host must apply when it runs a worker, the
//! argv escalation check, digest-pinned score and check inputs, the `max_workers` limits with
//! their `max_panes` migration, stop records and host cancel receipts. Nothing here performs
//! I/O, launches a process, sends keystrokes or calls a model provider.
//!
//! Hive never launches a worker. It publishes the policy beside every message and refuses a
//! profile whose argv would grant more than the policy. A host that ignores the policy is not
//! stopped by Hive; the real boundary is the host's own sandbox (for Codex `--sandbox`).

use crate::mediation::{ClaimRecord, MessageEnvelope, MEDIATION_ROOT, UNTRUSTED_DATA};
use crate::orchestration::{valid_digest, valid_id};
use crate::roster::WorkerProfile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Schema version of every containment record.
pub const CONTAINMENT_SCHEMA_VERSION: u32 = 1;
/// Default number of claimed (running) worker messages per project.
pub const DEFAULT_MAX_WORKERS: u32 = 4;
/// Largest accepted `max_workers`.
pub const MAX_MAX_WORKERS: u32 = 64;
/// Default per-claim time ceiling: no claim lease may run longer than this.
pub const DEFAULT_MAX_WORKER_SECONDS: u64 = 60 * 60;
/// Longest accepted `max_worker_seconds` (one day).
pub const MAX_MAX_WORKER_SECONDS: u64 = 24 * 60 * 60;
/// Current key in `.hive/config/harness.toml`.
pub const MAX_WORKERS_KEY: &str = "max_workers";
/// Per-claim time ceiling key in `.hive/config/harness.toml`.
pub const MAX_WORKER_SECONDS_KEY: &str = "max_worker_seconds";
/// Retired key; its value is preserved as `max_workers`.
pub const LEGACY_MAX_PANES_KEY: &str = "max_panes";
/// Network rule carried by every policy.
pub const NETWORK_NONE: &str = "none";
/// Remote push rule carried by every policy.
pub const GIT_PUSH_DENY: &str = "deny";
/// Writable-root placeholder; the host binds it to the task worktree (Codex `-C {target}`).
pub const WORKTREE_PLACEHOLDER: &str = "{target}";
/// The only cancel method a receipt may name.
pub const HOST_OFFICIAL_CANCEL: &str = "host-official-cancel";
/// Hive state inside the bound worktree (`.hive/`, including `.hive/mediation/`). The host must
/// keep it out of the worker's writable roots: a worker that could write it could forge claims,
/// receipts or results of other messages.
pub const EXCLUDED_HIVE_ROOT: &str = "{target}/.hive";

// ---------------------------------------------------------------------------------------------
// Worker policy.

/// Sandbox mode a host applies to one worker invocation.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sandbox {
    /// Codex `--sandbox read-only`: no writes at all.
    ReadOnly,
    /// Codex `--sandbox workspace-write`: writes only inside the bound worktree, network off.
    WorkspaceWrite,
    /// An interactive host (Claude) or unverified peer: the host's own permission mode applies;
    /// Hive can only publish the policy as guidance.
    HostGuidance,
}

impl Sandbox {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::HostGuidance => "host-guidance",
        }
    }
}

/// Sandbox the role may use: judges and researchers never write.
#[must_use]
pub fn role_sandbox(role: &str) -> Sandbox {
    if matches!(role, "hive-independent-judge" | "hive-research-specialist") {
        Sandbox::ReadOnly
    } else {
        Sandbox::WorkspaceWrite
    }
}

/// Least-privilege policy for one worker message. Hosts must apply it; Hive cannot enforce it.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerPolicy {
    pub schema_version: u32,
    pub role: String,
    pub provider: String,
    pub sandbox: Sandbox,
    /// Always `none`: a worker needs no network, so it cannot reach a remote.
    pub network: String,
    /// Always `deny`: a worker never pushes. Enforced by the sandbox network rule and `git_env`.
    pub git_push: String,
    /// Paths the worker may write; `{target}` is the task worktree the host binds.
    pub writable_roots: Vec<String>,
    /// Paths under a writable root that the host must keep read-only or hidden from the worker.
    /// Always `{target}/.hive`: Hive's own records, never worker output.
    pub excluded_roots: Vec<String>,
    /// Environment the host sets for the worker so any `git push`/`fetch` transport fails.
    pub git_env: BTreeMap<String, String>,
    /// Digest of the exact profile argv the policy was checked against.
    pub argv_digest: String,
    /// `host-sandbox` (Codex applies `--sandbox`) or `host-guidance` (no executable profile).
    pub enforced_by: String,
    /// Always false: Hive never runs the worker.
    pub hive_launches_worker: bool,
    /// Canonical digest of every other field (computed with this field empty).
    pub policy_digest: String,
}

impl WorkerPolicy {
    /// Build the policy for an already resolved profile after the argv escalation check.
    /// # Errors
    /// Rejects a profile whose argv grants more than the role's policy.
    pub fn for_profile(role: &str, profile: &WorkerProfile) -> Result<Self, String> {
        validate_worker_argv(role, profile)?;
        let executable = !profile.argv.is_empty();
        let sandbox = if executable {
            role_sandbox(role)
        } else {
            Sandbox::HostGuidance
        };
        let writable_roots = if sandbox == Sandbox::ReadOnly {
            Vec::new()
        } else {
            vec![WORKTREE_PLACEHOLDER.to_owned()]
        };
        let argv = serde_json_canonicalizer::to_vec(&profile.argv).map_err(|e| e.to_string())?;
        let policy = Self {
            schema_version: CONTAINMENT_SCHEMA_VERSION,
            role: role.to_owned(),
            provider: profile.provider.clone(),
            sandbox,
            network: NETWORK_NONE.to_owned(),
            git_push: GIT_PUSH_DENY.to_owned(),
            writable_roots,
            excluded_roots: vec![EXCLUDED_HIVE_ROOT.to_owned()],
            git_env: git_push_guard_env(),
            argv_digest: crate::sha256_digest(&argv),
            enforced_by: if executable {
                "host-sandbox"
            } else {
                "host-guidance"
            }
            .to_owned(),
            hive_launches_worker: false,
            policy_digest: String::new(),
        };
        policy.seal()
    }

    fn seal(mut self) -> Result<Self, String> {
        self.policy_digest = self.computed_digest()?;
        self.validate()?;
        Ok(self)
    }

    /// Canonical digest with `policy_digest` cleared.
    /// # Errors
    /// Returns a serialization error.
    pub fn computed_digest(&self) -> Result<String, String> {
        let mut unsealed = self.clone();
        unsealed.policy_digest = String::new();
        canonical_digest(&unsealed)
    }

    /// Fixed rules: no network, no push, no write outside the worktree, never launched by Hive.
    /// # Errors
    /// Rejects any field that grants more or a digest mismatch.
    pub fn validate(&self) -> Result<(), String> {
        let roots_ok = match self.sandbox {
            Sandbox::ReadOnly => self.writable_roots.is_empty(),
            Sandbox::WorkspaceWrite | Sandbox::HostGuidance => {
                self.writable_roots == [WORKTREE_PLACEHOLDER]
            }
        };
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || !valid_id(&self.role)
            || !valid_id(&self.provider)
            || self.network != NETWORK_NONE
            || self.git_push != GIT_PUSH_DENY
            || !roots_ok
            || self.excluded_roots != [EXCLUDED_HIVE_ROOT]
            || self.git_env != git_push_guard_env()
            || !valid_digest(&self.argv_digest)
            || !matches!(self.enforced_by.as_str(), "host-sandbox" | "host-guidance")
            || self.hive_launches_worker
        {
            return Err("worker policy grants more than least privilege".into());
        }
        if self.policy_digest != self.computed_digest()? {
            return Err("worker policy digest does not match its contents".into());
        }
        Ok(())
    }
}

/// Git configuration through the environment (`GIT_CONFIG_COUNT`): `protocol.allow=never`
/// makes every transport (file, ssh, git, http) fail, so `git push` and `git fetch` cannot
/// leave the worktree. A worker that edits its own environment or passes `-c` can undo this;
/// the host sandbox network rule is the boundary that does not depend on the worker.
#[must_use]
pub fn git_push_guard_env() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("GIT_CONFIG_COUNT".to_owned(), "1".to_owned()),
        ("GIT_CONFIG_KEY_0".to_owned(), "protocol.allow".to_owned()),
        ("GIT_CONFIG_VALUE_0".to_owned(), "never".to_owned()),
        ("GIT_TERMINAL_PROMPT".to_owned(), "0".to_owned()),
    ])
}

/// Argv tokens that disable the sandbox, widen writable roots or enable network.
const ESCALATING_TOKENS: &[&str] = &[
    "--dangerously-bypass-approvals-and-sandbox",
    "--yolo",
    "--full-auto",
    "--add-dir",
    "danger-full-access",
    "-s",
    "--dangerously-skip-permissions",
    "--permission-mode",
];
/// `-c key=value` overrides a worker profile may carry; anything else is refused.
const ALLOWED_CONFIG_KEYS: &[&str] = &["model_reasoning_effort", "forced_login_method"];

/// Refuse a profile whose argv grants more than its role's policy. Codex must carry exactly one
/// `--sandbox` equal to the role sandbox; Claude profiles must stay non-executable. Antigravity
/// and Gemini are refused under Google's Antigravity terms.
/// # Errors
/// Names the first escalating argument.
pub fn validate_worker_argv(role: &str, profile: &WorkerProfile) -> Result<(), String> {
    crate::automation_terms::refuse_automated(&profile.provider)?;
    let argv = &profile.argv;
    if let Some(token) = argv.iter().find(|arg| {
        let flag = arg.split_once('=').map_or(arg.as_str(), |(flag, _)| flag);
        ESCALATING_TOKENS.contains(&flag) || arg.contains("danger-full-access")
    }) {
        return Err(format!(
            "worker argv escalates beyond the role policy: {token}"
        ));
    }
    let mut sandboxes = Vec::new();
    for (index, arg) in argv.iter().enumerate() {
        if arg == "-c" || arg == "--config" {
            let value = argv.get(index + 1).map_or("", String::as_str);
            let key = value.split_once('=').map_or(value, |(key, _)| key).trim();
            if !ALLOWED_CONFIG_KEYS.contains(&key) {
                return Err(format!(
                    "worker argv overrides a configuration key outside the policy: {key}"
                ));
            }
        }
        if arg.starts_with("--sandbox=") || arg.starts_with("--config=") || arg.starts_with("-c=") {
            return Err(format!("worker argv uses an unreviewed inline form: {arg}"));
        }
        if arg == "--sandbox" {
            sandboxes.push(argv.get(index + 1).map_or("", String::as_str));
        }
    }
    match profile.provider.as_str() {
        "claude" | "codex" if argv.is_empty() => Ok(()),
        "codex" => {
            if sandboxes == [role_sandbox(role).as_str()] {
                Ok(())
            } else {
                Err(format!(
                    "worker argv must carry exactly one --sandbox {} for role {role}",
                    role_sandbox(role).as_str()
                ))
            }
        }
        "claude" => Err("claude worker profiles are not executable; argv must stay empty".into()),
        _ => Err("unknown worker provider".into()),
    }
}

// ---------------------------------------------------------------------------------------------
// Digest-pinned score and check inputs.

/// One score or check input outside the worker's writable worktree, pinned at flow start.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedCheck {
    /// Canonical absolute path, read-only for the worker.
    pub path: String,
    pub digest: String,
    pub bytes: u64,
}

/// Pins recorded once per flow (no-clobber), beside `flow.json`.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckPinRecord {
    pub schema_version: u32,
    pub flow_id: String,
    /// Canonical worktree the worker may write; every check lies outside it.
    pub worktree: String,
    pub checks: Vec<PinnedCheck>,
    pub record_digest: String,
}

impl CheckPinRecord {
    /// # Errors
    /// Returns a serialization error.
    pub fn computed_digest(&self) -> Result<String, String> {
        let mut unsealed = self.clone();
        unsealed.record_digest = String::new();
        canonical_digest(&unsealed)
    }

    /// # Errors
    /// Rejects invalid fields.
    pub fn seal(mut self) -> Result<Self, String> {
        self.record_digest = self.computed_digest()?;
        self.validate()?;
        Ok(self)
    }

    /// # Errors
    /// Rejects unsafe ids, digests, a check inside the worktree or a digest mismatch.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || !valid_id(&self.flow_id)
            || self.checks.is_empty()
            || self.checks.iter().any(|check| {
                !valid_digest(&check.digest)
                    || !check_is_outside(Path::new(&check.path), Path::new(&self.worktree))
            })
        {
            return Err("invalid check pin record".into());
        }
        if self.record_digest != self.computed_digest()? {
            return Err("check pin record digest does not match its contents".into());
        }
        Ok(())
    }
}

/// Recorded when a pinned check no longer matches; the flow stops and never continues.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckDrift {
    pub schema_version: u32,
    pub flow_id: String,
    pub path: String,
    pub pinned_digest: String,
    /// `None` when the file disappeared or became unreadable.
    pub observed_digest: Option<String>,
    pub detected_at: u64,
}

/// Both paths must already be canonical and absolute. A check equal to or under the worktree
/// is writable by the worker and is refused for scoring.
#[must_use]
pub fn check_is_outside(check: &Path, worktree: &Path) -> bool {
    check.is_absolute() && worktree.is_absolute() && !check.starts_with(worktree)
}

/// `.hive/mediation/flows/<flow_id>/checks.json`
#[must_use]
pub fn check_pin_path(flow_id: &str) -> String {
    format!("{}/{flow_id}/checks.json", crate::flow::FLOWS_ROOT)
}

/// `.hive/mediation/flows/<flow_id>/check-drift.json`
#[must_use]
pub fn check_drift_path(flow_id: &str) -> String {
    format!("{}/{flow_id}/check-drift.json", crate::flow::FLOWS_ROOT)
}

// ---------------------------------------------------------------------------------------------
// Worker limits and the `max_panes` migration.

/// Where `max_workers` came from.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LimitSource {
    Default,
    MaxWorkers,
    /// Only the retired key exists; its value is used and moves on the next upgrade write.
    LegacyMaxPanes,
    /// Both keys hold the same value; the retired key is dropped on the next upgrade write.
    BothSame,
}

/// Concurrency and time ceilings for claimed worker messages.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkerLimits {
    pub max_workers: u32,
    pub max_worker_seconds: u64,
    pub source: LimitSource,
}

impl Default for WorkerLimits {
    fn default() -> Self {
        Self {
            max_workers: DEFAULT_MAX_WORKERS,
            max_worker_seconds: DEFAULT_MAX_WORKER_SECONDS,
            source: LimitSource::Default,
        }
    }
}

/// Why the limits could not be read.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LimitError {
    /// `max_workers` and `max_panes` disagree; nothing is overwritten.
    Conflict {
        max_workers: u64,
        max_panes: u64,
    },
    Invalid(String),
}

impl std::fmt::Display for LimitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict {
                max_workers,
                max_panes,
            } => write!(
                formatter,
                "max_workers = {max_workers} conflicts with the retired max_panes = {max_panes}; keep one value and remove the other by hand"
            ),
            Self::Invalid(reason) => formatter.write_str(reason),
        }
    }
}

fn integer(table: &toml::Table, key: &str, range: (u64, u64)) -> Result<Option<u64>, LimitError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };
    value
        .as_integer()
        .and_then(|value| u64::try_from(value).ok())
        .filter(|value| (range.0..=range.1).contains(value))
        .map(Some)
        .ok_or_else(|| {
            LimitError::Invalid(format!(
                "{key} must be an integer in {}..={}",
                range.0, range.1
            ))
        })
}

/// Read the limits from `.hive/config/harness.toml` text (top-level keys only).
/// # Errors
/// Rejects malformed TOML, out-of-range values and a `max_workers`/`max_panes` conflict.
pub fn worker_limits_from_harness(text: &str) -> Result<WorkerLimits, LimitError> {
    let table: toml::Table = toml::from_str(text)
        .map_err(|error| LimitError::Invalid(format!("invalid harness config: {error}")))?;
    let workers_range = (1, u64::from(MAX_MAX_WORKERS));
    let current = integer(&table, MAX_WORKERS_KEY, workers_range)?;
    let legacy = integer(&table, LEGACY_MAX_PANES_KEY, workers_range)?;
    let seconds = integer(&table, MAX_WORKER_SECONDS_KEY, (1, MAX_MAX_WORKER_SECONDS))?;
    let (value, source) = match (current, legacy) {
        (None, None) => (u64::from(DEFAULT_MAX_WORKERS), LimitSource::Default),
        (Some(value), None) => (value, LimitSource::MaxWorkers),
        (None, Some(value)) => (value, LimitSource::LegacyMaxPanes),
        (Some(current), Some(legacy)) if current == legacy => (current, LimitSource::BothSame),
        (Some(max_workers), Some(max_panes)) => {
            return Err(LimitError::Conflict {
                max_workers,
                max_panes,
            })
        }
    };
    Ok(WorkerLimits {
        max_workers: u32::try_from(value).unwrap_or(MAX_MAX_WORKERS),
        max_worker_seconds: seconds.unwrap_or(DEFAULT_MAX_WORKER_SECONDS),
        source,
    })
}

/// Top-level `key = value` line (before any table header), exactly one.
fn top_level_key_line(text: &str, key: &str) -> Result<Option<(usize, usize)>, LimitError> {
    let mut offset = 0;
    let mut found = None;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix(key) {
            if rest.trim_start().starts_with('=') {
                if found.is_some() {
                    return Err(LimitError::Invalid(format!("{key} appears more than once")));
                }
                found = Some((offset, offset + line.len()));
            }
        }
        offset += line.len();
    }
    Ok(found)
}

/// Preserve-and-move migration for the upgrade write: `max_panes = N` becomes
/// `max_workers = N` (the rest of the line, including a comment, is kept); when both keys hold
/// the same value the retired line is removed. Every other byte is unchanged.
/// `Ok(None)` means there is nothing to migrate.
/// # Errors
/// Refuses a conflict or a value that cannot be read; the caller writes nothing.
pub fn migrate_harness_worker_limits(text: &str) -> Result<Option<String>, LimitError> {
    let before = worker_limits_from_harness(text)?;
    if !matches!(
        before.source,
        LimitSource::LegacyMaxPanes | LimitSource::BothSame
    ) {
        return Ok(None);
    }
    let Some((start, end)) = top_level_key_line(text, LEGACY_MAX_PANES_KEY)? else {
        return Err(LimitError::Invalid(
            "max_panes must be a plain top-level line to migrate".into(),
        ));
    };
    let line = &text[start..end];
    let replacement = if before.source == LimitSource::BothSame {
        String::new()
    } else {
        let indent = &line[..line.len() - line.trim_start().len()];
        let rest = &line.trim_start()[LEGACY_MAX_PANES_KEY.len()..];
        format!("{indent}{MAX_WORKERS_KEY}{rest}")
    };
    let migrated = format!("{}{replacement}{}", &text[..start], &text[end..]);
    let after = worker_limits_from_harness(&migrated)?;
    if after.source != LimitSource::MaxWorkers
        || after.max_workers != before.max_workers
        || after.max_worker_seconds != before.max_worker_seconds
    {
        return Err(LimitError::Invalid(
            "max_panes migration did not preserve the value".into(),
        ));
    }
    Ok(Some(migrated))
}

// ---------------------------------------------------------------------------------------------
// Stop records.

/// What a stop covers.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StopScope {
    Project,
    Flow,
    Task,
}

impl StopScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Flow => "flow",
            Self::Task => "task",
        }
    }
}

/// Cancel event, published no-clobber. While it has no clear record, no new envelope, flow
/// step, requeue, claim or hook continuation is created in its scope.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StopRecord {
    pub schema_version: u32,
    pub stop_id: String,
    pub scope: StopScope,
    /// Flow or task id; `None` for the project scope.
    pub scope_id: Option<String>,
    /// 1 for the first stop of this scope, +1 after each explicit clear.
    pub generation: u32,
    pub reason: Option<String>,
    pub requested_at: u64,
    /// Always false: a stop never interrupts a running worker or sends keys.
    pub forced: bool,
    pub record_digest: String,
}

impl StopRecord {
    /// # Errors
    /// Returns a serialization error.
    pub fn computed_digest(&self) -> Result<String, String> {
        let mut unsealed = self.clone();
        unsealed.record_digest = String::new();
        canonical_digest(&unsealed)
    }

    /// # Errors
    /// Rejects invalid fields.
    pub fn seal(mut self) -> Result<Self, String> {
        self.record_digest = self.computed_digest()?;
        self.validate()?;
        Ok(self)
    }

    /// # Errors
    /// Rejects a wrong id, scope, reason or digest.
    pub fn validate(&self) -> Result<(), String> {
        let scope_ok = match self.scope {
            StopScope::Project => self.scope_id.is_none(),
            StopScope::Flow | StopScope::Task => self.scope_id.as_deref().is_some_and(valid_id),
        };
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || !scope_ok
            || self.generation == 0
            || self.stop_id != stop_id_for(self.scope, self.scope_id.as_deref(), self.generation)
            || self.forced
            || self
                .reason
                .as_deref()
                .is_some_and(|reason| reason.len() > 240 || reason.chars().any(char::is_control))
        {
            return Err("invalid stop record".into());
        }
        if self.record_digest != self.computed_digest()? {
            return Err("stop record digest does not match its contents".into());
        }
        Ok(())
    }

    /// Does this stop cover work in `flow` and/or `task`?
    #[must_use]
    pub fn covers(&self, flow: Option<&str>, task: Option<&str>) -> bool {
        match self.scope {
            StopScope::Project => true,
            StopScope::Flow => flow.is_some() && self.scope_id.as_deref() == flow,
            StopScope::Task => task.is_some() && self.scope_id.as_deref() == task,
        }
    }
}

/// Explicit resume: binds the exact stop record digest the user confirmed.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StopClear {
    pub schema_version: u32,
    pub stop_id: String,
    pub stop_digest: String,
    pub cleared_at: u64,
}

impl StopClear {
    /// # Errors
    /// Rejects a clear that does not bind this stop.
    pub fn validate(&self, stop: &StopRecord) -> Result<(), String> {
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || self.stop_id != stop.stop_id
            || self.stop_digest != stop.record_digest
            || self.cleared_at < stop.requested_at
        {
            return Err("stop clear does not bind this stop".into());
        }
        Ok(())
    }
}

/// `stop-` + first 24 hex of SHA-256(`scope:id:generation`).
#[must_use]
pub fn stop_id_for(scope: StopScope, scope_id: Option<&str>, generation: u32) -> String {
    let digest = crate::sha256_digest(
        format!(
            "{}:{}:{generation}",
            scope.as_str(),
            scope_id.unwrap_or("-")
        )
        .as_bytes(),
    );
    format!("stop-{}", &digest[7..31])
}

/// `.hive/mediation/stops`
pub const STOPS_ROOT: &str = ".hive/mediation/stops";

/// `.hive/mediation/stops/<stop_id>.json`
#[must_use]
pub fn stop_path(stop_id: &str) -> String {
    format!("{STOPS_ROOT}/{stop_id}.json")
}

/// `.hive/mediation/stops/<stop_id>.clear.json`
#[must_use]
pub fn stop_clear_path(stop_id: &str) -> String {
    format!("{STOPS_ROOT}/{stop_id}.clear.json")
}

// ---------------------------------------------------------------------------------------------
// Host cancel.

/// Whether a host offers an official cancel for in-progress worker work that Hive has verified.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CancelSupport {
    /// No documented cancel for an in-progress invocation: Hive shows `unsupported` and the
    /// claim ends by its lease (dispatch-uncertain); a cancel receipt is refused.
    Unsupported,
    /// The host owns the worker process and may end it through its own official control; a
    /// host receipt is accepted but the round trip is not verified by Hive.
    Unverified,
}

impl CancelSupport {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::Unverified => "unverified",
        }
    }
}

/// Cancel support per provider, with the reason shown to the user.
#[must_use]
pub fn host_cancel_support(provider: &str) -> (CancelSupport, &'static str) {
    match provider {
        "codex" => (
            CancelSupport::Unverified,
            "the host that ran `codex exec` owns that process and may end it with its own process control; Hive has not verified a cancel round trip",
        ),
        "claude" => (
            CancelSupport::Unsupported,
            "the interactive Claude host documents no cancel for a running turn or subagent that another process can request; Hive never sends keys",
        ),
        "gemini" | "antigravity" => (
            CancelSupport::Unsupported,
            crate::automation_terms::ANTIGRAVITY_AUTOMATION_REFUSAL,
        ),
        _ => (CancelSupport::Unsupported, "unknown provider"),
    }
}

/// Host-written receipt that the host's own official cancel stopped this claimed message.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CancelReceipt {
    pub schema_version: u32,
    pub message_id: String,
    pub envelope_digest: String,
    pub claim_digest: String,
    pub stop_id: String,
    pub host: String,
    pub method: String,
    pub cancelled_at: u64,
    /// Must be false: a forced stop (kill without the host's cancel, keystrokes) is never
    /// recorded as a cancel.
    pub forced: bool,
    pub trust: String,
}

impl CancelReceipt {
    /// # Errors
    /// Rejects a receipt that does not bind this envelope and claim or claims a forced stop.
    pub fn validate(&self, envelope: &MessageEnvelope, claim: &ClaimRecord) -> Result<(), String> {
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || self.message_id != envelope.message_id
            || self.envelope_digest != envelope.envelope_digest
            || self.claim_digest != claim.digest()?
            || !self.stop_id.starts_with("stop-")
            || !valid_id(&self.stop_id)
            || !valid_id(&self.host)
            || self.method != HOST_OFFICIAL_CANCEL
            || self.forced
            || self.trust != UNTRUSTED_DATA
            || self.cancelled_at < claim.claimed_at
        {
            return Err("cancel receipt does not bind this message and claim".into());
        }
        Ok(())
    }
}

/// `.hive/mediation/cancels/<message_id>.json`
#[must_use]
pub fn cancel_receipt_path(message_id: &str) -> String {
    format!("{MEDIATION_ROOT}/cancels/{message_id}.json")
}

/// `.hive/mediation/policies/<message_id>.json`
#[must_use]
pub fn policy_path(message_id: &str) -> String {
    format!("{MEDIATION_ROOT}/policies/{message_id}.json")
}

/// `.hive/mediation/slots/<slot>`: generations of one `max_workers` slot.
#[must_use]
pub fn slot_dir(slot: u32) -> String {
    format!("{MEDIATION_ROOT}/slots/{slot}")
}

/// `.hive/mediation/slots/<slot>/<20-digit generation>.json`
#[must_use]
pub fn slot_path(slot: u32, generation: u64) -> String {
    format!("{}/{generation:020}.json", slot_dir(slot))
}

/// One reservation of a `max_workers` slot, published no-clobber before the claim it admits.
///
/// Generations of one slot are contiguous: generation `n + 1` is published only after the
/// holder of generation `n` was observed released, so two claimants racing for one slot can
/// never both win it. A holder is released when its claim lost the claim race (another token),
/// reached a terminal or uncertain state, or was never published before `lease_until`.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerSlot {
    pub schema_version: u32,
    pub slot: u32,
    pub generation: u64,
    pub message_id: String,
    pub claim_token: String,
    pub reserved_at: u64,
    pub lease_until: u64,
}

impl WorkerSlot {
    /// # Errors
    /// Rejects unsafe ids, an out-of-range slot or an empty lease.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || self.slot >= MAX_MAX_WORKERS
            || !valid_id(&self.message_id)
            || !valid_id(&self.claim_token)
            || self.lease_until <= self.reserved_at
        {
            return Err("invalid worker slot reservation".into());
        }
        Ok(())
    }
}

/// Worker policy bound to one envelope, published beside it (no-clobber).
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MessagePolicyRecord {
    pub schema_version: u32,
    pub message_id: String,
    pub policy: WorkerPolicy,
}

impl MessagePolicyRecord {
    /// # Errors
    /// Rejects a record for another message or role, or an invalid policy.
    pub fn validate(&self, envelope: &MessageEnvelope) -> Result<(), String> {
        if self.schema_version != CONTAINMENT_SCHEMA_VERSION
            || self.message_id != envelope.message_id
            || self.policy.role != envelope.recipient_role
        {
            return Err("worker policy record does not bind this message".into());
        }
        self.policy.validate()
    }
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json_canonicalizer::to_vec(value)
        .map(|bytes| crate::sha256_digest(&bytes))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roster::Roster;

    #[test]
    fn containment_policy_is_least_privilege_for_every_default_role() {
        let roster = Roster::defaults().unwrap();
        for (role, sandbox) in [
            ("hive-routine-implementer", Sandbox::WorkspaceWrite),
            ("hive-complex-implementer", Sandbox::WorkspaceWrite),
            ("hive-independent-judge", Sandbox::ReadOnly),
            ("hive-research-specialist", Sandbox::ReadOnly),
        ] {
            let profile = roster.worker_profile(role).unwrap();
            let policy = WorkerPolicy::for_profile(role, &profile).unwrap();
            assert_eq!(policy.sandbox, sandbox, "{role}");
            assert_eq!(policy.network, "none");
            assert_eq!(policy.git_push, "deny");
            assert!(!policy.hive_launches_worker);
            assert_eq!(policy.git_env["GIT_CONFIG_KEY_0"], "protocol.allow");
            assert_eq!(policy.git_env["GIT_CONFIG_VALUE_0"], "never");
            if sandbox == Sandbox::ReadOnly {
                assert_eq!(policy.writable_roots, Vec::<String>::new());
            } else {
                assert_eq!(policy.writable_roots, ["{target}"]);
            }
            let mut widened = policy.clone();
            widened.network = "enabled".into();
            assert!(widened.validate().is_err());
            let mut pushes = policy.clone();
            pushes.git_push = "allow".into();
            assert!(pushes.validate().is_err());
            let mut roots = policy.clone();
            roots.writable_roots = vec!["/".into()];
            assert!(roots.validate().is_err());
            // `.hive/` (mediation records included) is never a worker-writable path (S9).
            assert_eq!(policy.excluded_roots, ["{target}/.hive"]);
            let mut exposed = policy.clone();
            exposed.excluded_roots.clear();
            assert!(exposed.validate().is_err());
        }
        let mut claude = Roster::defaults().unwrap();
        claude.workspace.provider = Some("claude".into());
        let profile = claude.worker_profile("hive-routine-implementer").unwrap();
        let policy = WorkerPolicy::for_profile("hive-routine-implementer", &profile).unwrap();
        assert_eq!(policy.sandbox, Sandbox::HostGuidance);
        assert_eq!(policy.enforced_by, "host-guidance");
    }

    #[test]
    fn containment_argv_escalation_is_refused() {
        let roster = Roster::defaults().unwrap();
        let base = roster.worker_profile("hive-routine-implementer").unwrap();
        validate_worker_argv("hive-routine-implementer", &base).unwrap();
        let replace = |from: &str, to: &str| {
            let mut profile = base.clone();
            for arg in &mut profile.argv {
                if arg == from {
                    to.clone_into(arg);
                }
            }
            profile
        };
        let mut cases = vec![
            replace("workspace-write", "danger-full-access"),
            replace("--ephemeral", "--dangerously-bypass-approvals-and-sandbox"),
            replace("--ephemeral", "--full-auto"),
            replace(
                "forced_login_method=\"chatgpt\"",
                "sandbox_workspace_write.network_access=true",
            ),
        ];
        let mut add_dir = base.clone();
        add_dir.argv.splice(2..2, ["--add-dir".into(), "/".into()]);
        cases.push(add_dir);
        let mut second = base.clone();
        second
            .argv
            .splice(2..2, ["--sandbox".into(), "workspace-write".into()]);
        cases.push(second);
        let mut inline = base.clone();
        inline.argv.push("--sandbox=danger-full-access".into());
        cases.push(inline);
        for profile in cases {
            let error = validate_worker_argv("hive-routine-implementer", &profile).unwrap_err();
            assert!(WorkerPolicy::for_profile("hive-routine-implementer", &profile).is_err());
            assert!(error.contains("worker argv"), "{error}");
        }
        // A read-only role may not run with workspace-write.
        assert!(validate_worker_argv("hive-independent-judge", &base).is_err());
        let mut claude = base;
        claude.provider = "claude".into();
        assert!(validate_worker_argv("hive-routine-implementer", &claude).is_err());
    }

    #[test]
    fn containment_git_guard_makes_push_fail() {
        // Real `git` in a temporary folder only; no network, host or worker. Skipped without git.
        let Ok(version) = std::process::Command::new("git").arg("--version").output() else {
            return;
        };
        if !version.status.success() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let remote = root.path().join("remote.git");
        let work = root.path().join("work");
        let git = |dir: &Path, args: &[&str], guarded: bool| {
            let mut command = std::process::Command::new("git");
            command
                .current_dir(dir)
                .args(args)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("HOME", root.path())
                .env_remove("GIT_CONFIG_COUNT")
                .env_remove("GIT_CONFIG_PARAMETERS");
            if guarded {
                command.envs(git_push_guard_env());
            }
            command.output().unwrap()
        };
        std::fs::create_dir_all(&work).unwrap();
        assert!(
            git(root.path(), &["init", "-q", "--bare", "remote.git"], false)
                .status
                .success()
        );
        assert!(git(&work, &["init", "-q"], false).status.success());
        assert!(git(
            &work,
            &[
                "-c",
                "user.email=t@example.invalid",
                "-c",
                "user.name=t",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "x"
            ],
            false
        )
        .status
        .success());
        let remote_text = remote.to_str().unwrap();
        let denied = git(&work, &["push", remote_text, "HEAD:refs/heads/main"], true);
        assert!(!denied.status.success(), "guarded push must fail");
        assert!(
            String::from_utf8_lossy(&denied.stderr).contains("not allowed"),
            "{}",
            String::from_utf8_lossy(&denied.stderr)
        );
        // Control: the same push without the guard reaches the local remote.
        let allowed = git(&work, &["push", remote_text, "HEAD:refs/heads/main"], false);
        assert!(allowed.status.success());
    }

    #[test]
    fn containment_limits_preserve_and_refuse_conflicts() {
        let base = "schema_version = 1\nharness_version = \"0.15.3\"\n";
        assert_eq!(
            worker_limits_from_harness(base).unwrap(),
            WorkerLimits::default()
        );
        assert_eq!(migrate_harness_worker_limits(base).unwrap(), None);
        let legacy = format!("{base}max_panes = 3 # old name\nmax_worker_seconds = 600\n");
        let limits = worker_limits_from_harness(&legacy).unwrap();
        assert_eq!(limits.max_workers, 3);
        assert_eq!(limits.max_worker_seconds, 600);
        assert_eq!(limits.source, LimitSource::LegacyMaxPanes);
        let migrated = migrate_harness_worker_limits(&legacy).unwrap().unwrap();
        assert_eq!(
            migrated,
            format!("{base}max_workers = 3 # old name\nmax_worker_seconds = 600\n")
        );
        assert_eq!(migrate_harness_worker_limits(&migrated).unwrap(), None);
        let same = format!("{base}max_workers = 2\nmax_panes = 2\n[extra]\nmax_panes = 9\n");
        assert_eq!(
            worker_limits_from_harness(&same).unwrap().source,
            LimitSource::BothSame
        );
        assert_eq!(
            migrate_harness_worker_limits(&same).unwrap().unwrap(),
            format!("{base}max_workers = 2\n[extra]\nmax_panes = 9\n")
        );
        let conflict = format!("{base}max_workers = 2\nmax_panes = 5\n");
        let error = worker_limits_from_harness(&conflict).unwrap_err();
        assert_eq!(
            error,
            LimitError::Conflict {
                max_workers: 2,
                max_panes: 5
            }
        );
        assert!(error.to_string().contains("conflicts"));
        assert_eq!(migrate_harness_worker_limits(&conflict).unwrap_err(), error);
        for bad in [
            "max_workers = 0\n",
            "max_panes = 65\n",
            "max_workers = \"4\"\n",
        ] {
            assert!(matches!(
                worker_limits_from_harness(&format!("{base}{bad}")),
                Err(LimitError::Invalid(_))
            ));
        }
    }

    #[test]
    fn containment_stop_and_cancel_records_bind_exact_scope() {
        let stop = StopRecord {
            schema_version: 1,
            stop_id: stop_id_for(StopScope::Task, Some("task-1"), 1),
            scope: StopScope::Task,
            scope_id: Some("task-1".into()),
            generation: 1,
            reason: Some("user stop".into()),
            requested_at: 10,
            forced: false,
            record_digest: String::new(),
        }
        .seal()
        .unwrap();
        assert!(stop.covers(None, Some("task-1")));
        assert!(!stop.covers(Some("task-1"), Some("task-2")));
        let mut forced = stop.clone();
        forced.forced = true;
        assert!(forced.validate().is_err());
        let mut moved = stop.clone();
        moved.scope_id = Some("task-2".into());
        assert!(moved.validate().is_err());
        assert_ne!(
            stop_id_for(StopScope::Task, Some("task-1"), 2),
            stop.stop_id
        );
        let clear = StopClear {
            schema_version: 1,
            stop_id: stop.stop_id.clone(),
            stop_digest: stop.record_digest.clone(),
            cleared_at: 11,
        };
        clear.validate(&stop).unwrap();
        let mut wrong = clear;
        wrong.stop_digest = crate::sha256_digest(b"other");
        assert!(wrong.validate(&stop).is_err());
        assert_eq!(host_cancel_support("claude").0, CancelSupport::Unsupported);
        assert_eq!(host_cancel_support("gemini").0, CancelSupport::Unsupported);
        assert_eq!(
            host_cancel_support("antigravity").1,
            crate::automation_terms::ANTIGRAVITY_AUTOMATION_REFUSAL
        );
        assert_eq!(host_cancel_support("codex").0, CancelSupport::Unverified);
        for path in [
            stop_path(&stop.stop_id),
            stop_clear_path(&stop.stop_id),
            cancel_receipt_path("msg-1"),
            policy_path("msg-1"),
            check_pin_path("flow-1"),
            check_drift_path("flow-1"),
        ] {
            crate::validate_project_relative(Path::new(&path)).unwrap();
            assert!(path.starts_with(".hive/mediation/"), "{path}");
        }
    }

    #[test]
    fn containment_checks_must_lie_outside_the_worktree() {
        // A POSIX-rooted path is not absolute on Windows; give it a drive there.
        let abs = |path: &str| {
            if cfg!(windows) {
                format!("C:{path}")
            } else {
                path.to_owned()
            }
        };
        let worktree_path = abs("/work/task-1");
        let worktree = Path::new(&worktree_path);
        assert!(check_is_outside(
            Path::new(&abs("/work/scores/s.md")),
            worktree
        ));
        assert!(check_is_outside(
            Path::new(&abs("/work/task-10/s.md")),
            worktree
        ));
        assert!(!check_is_outside(
            Path::new(&abs("/work/task-1/s.md")),
            worktree
        ));
        assert!(!check_is_outside(worktree, worktree));
        assert!(!check_is_outside(Path::new("relative.md"), worktree));
        let record = CheckPinRecord {
            schema_version: 1,
            flow_id: "flow-1".into(),
            worktree: abs("/work/task-1"),
            checks: vec![PinnedCheck {
                path: abs("/work/scores/s.md"),
                digest: crate::sha256_digest(b"score"),
                bytes: 5,
            }],
            record_digest: String::new(),
        }
        .seal()
        .unwrap();
        let mut inside = record.clone();
        inside.checks[0].path = abs("/work/task-1/s.md");
        assert!(inside.seal().is_err());
        let mut tampered = record;
        tampered.checks[0].digest = crate::sha256_digest(b"other");
        assert!(tampered.validate().is_err());
    }
}
