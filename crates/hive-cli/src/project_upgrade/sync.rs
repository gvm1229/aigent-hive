//! Project sync from the global Hive (OV170-029–032).
//!
//! `.hive/project.toml` is edited by people and holds the version range. `.hive/project.lock`
//! is written by Hive only and records the Hive version, the authenticated project base and
//! the digest of every Hive-managed file. A project without `.hive/` is never touched.
use super::{
    apply, ensure_pinned_consumer_target, failure, open_target_capability,
    prepare_with_reviewed_skills, read_base_ledger, read_bounded_optional, BaseLedger,
    ProjectEvidence, ProjectResult, BASE_PATH, JOURNAL_PATH, MAX_LEDGER_BYTES,
};
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use hive_update::UpdateError;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

pub(super) const PROJECT_TOML: &str = ".hive/project.toml";
pub(super) const PROJECT_LOCK: &str = ".hive/project.lock";
const MAX_SETTINGS_BYTES: u64 = 64 * 1024;
const ACTION: &str = "SyncProjectHarness";

/// Old settings files that keep their own paths in 0.17.x, with their protection class.
///
/// `.hive/setup-answers.yml` stays because `copier.yml` names it as Copier's YAML answers file.
const SECTIONS: &[(&str, &str, &str)] = &[
    ("setup", ".hive/setup-answers.yml", "hive-managed-config"),
    (
        "roles",
        ".hive/config/role-seeds.yml",
        "user-answer-protected",
    ),
    (
        "knowledge",
        ".hive/config/knowledge-scope.yml",
        "user-answer-protected",
    ),
    (
        "skills",
        ".hive/config/approved-skills.yml",
        "user-consent-protected",
    ),
    (
        "hooks",
        ".hive/config/approved-hooks.yml",
        "user-consent-protected",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Range {
    major: u64,
    minor: u64,
}

impl Range {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let trimmed = text.strip_suffix(".x").unwrap_or(text);
        let mut parts = trimmed.split('.');
        let (Some(major), Some(minor), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(format!(
                "range {text:?} must be MAJOR.MINOR, such as \"0.17\""
            ));
        };
        let number = |part: &str| {
            (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| part.parse::<u64>().ok())
                .flatten()
                .ok_or_else(|| format!("range {text:?} must be MAJOR.MINOR, such as \"0.17\""))
        };
        Ok(Self {
            major: number(major)?,
            minor: number(minor)?,
        })
    }

    fn of(version: (u64, u64, u64)) -> Self {
        Self {
            major: version.0,
            minor: version.1,
        }
    }

    fn contains(self, version: (u64, u64, u64)) -> bool {
        self.major == version.0 && self.minor == version.1
    }

    fn render(self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

/// `MAJOR.MINOR.PATCH`, ignoring a pre-release or build suffix.
pub(crate) fn version(text: &str) -> Option<(u64, u64, u64)> {
    let core = text.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(Some(a)), Some(Some(b)), Some(Some(c)), None) => Some((a, b, c)),
        _ => None,
    }
}

/// What the global Hive may do with a project.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Gate {
    /// Same version as recorded: only the lock may be refreshed.
    Current,
    /// Newer and inside the range: update automatically.
    Upgrade,
    /// Older than the recorded version: never write (forward only).
    Older,
    /// Outside the range: warn only until a person raises the range.
    OutOfRange,
}

pub(crate) fn gate(range: Range, recorded: (u64, u64, u64), global: (u64, u64, u64)) -> Gate {
    if global < recorded {
        Gate::Older
    } else if !range.contains(global) {
        Gate::OutOfRange
    } else if global == recorded {
        Gate::Current
    } else {
        Gate::Upgrade
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    schema_version: u32,
    hive: HiveSection,
    #[serde(default)]
    #[allow(dead_code)]
    sections: Option<toml::Table>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HiveSection {
    range: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lock {
    schema_version: u32,
    hive_version: String,
    range: String,
    base_ledger_digest: String,
    #[serde(default)]
    files: Vec<LockFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LockFile {
    #[allow(dead_code)]
    path: String,
    #[allow(dead_code)]
    digest: String,
}

pub(crate) fn render_settings(range: Range) -> String {
    let mut text = String::from(
        "# Aigent Hive project settings. People edit this file; Hive never rewrites it.\n\
         # [hive] range: Hive versions that may update this project's Hive files automatically.\n\
         # Raise it by hand (for example to the next MAJOR.MINOR) to accept a larger update.\n\
         schema_version = 1\n\n[hive]\n",
    );
    writeln!(text, "range = \"{}\"", range.render()).expect("formatting a String");
    text.push_str(
        "\n# These settings keep their own files in 0.17.x. Each keeps its protection class:\n\
         # Hive changes a consent-protected file only with the user's approval.\n",
    );
    for (name, path, protection) in SECTIONS {
        write!(
            text,
            "\n[sections.{name}]\npath = \"{path}\"\nprotection = \"{protection}\"\n"
        )
        .expect("formatting a String");
    }
    text
}

/// The exact lock for an authenticated base ledger and range.
pub(crate) fn render_lock(
    ledger_version: &str,
    ledger_digest: &str,
    range: Range,
    files: &[(String, String)],
) -> String {
    let mut text = format!(
        "# Written by Aigent Hive. Do not edit by hand.\n\
         # After a Git conflict in this file, run: hive project sync --target .\n\
         schema_version = 1\nhive_version = \"{ledger_version}\"\nrange = \"{}\"\n\
         base_ledger_digest = \"{ledger_digest}\"\n",
        range.render()
    );
    for (path, digest) in files {
        write!(
            text,
            "\n[[files]]\npath = \"{path}\"\ndigest = \"{digest}\"\n"
        )
        .expect("formatting a String");
    }
    text
}

fn expected_lock(ledger: &BaseLedger, range: Range) -> String {
    let files: Vec<(String, String)> = ledger
        .files
        .iter()
        .map(|file| (file.path.clone(), file.content_digest.clone()))
        .collect();
    render_lock(
        &ledger.product_version,
        &ledger.ledger_digest,
        range,
        &files,
    )
}

pub(crate) fn has_conflict_markers(text: &str) -> bool {
    text.lines().any(|line| {
        line.starts_with("<<<<<<<")
            || line.starts_with(">>>>>>>")
            || line == "======="
            || line.starts_with("======= ")
    })
}

/// Problems the check-only mode reports for a project with `.hive/`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LockIssue {
    MissingSettings,
    InvalidSettings(String),
    MissingLock,
    ConflictMarkers,
    InvalidLock(String),
    /// Hive files and `project.lock` changed in different commits.
    OneSided,
    Stale,
}

impl LockIssue {
    fn message(&self) -> String {
        match self {
            Self::MissingSettings => format!("{PROJECT_TOML} is missing"),
            Self::InvalidSettings(error) => format!("{PROJECT_TOML} is invalid: {error}"),
            Self::MissingLock => format!("{PROJECT_LOCK} is missing"),
            Self::ConflictMarkers => format!("{PROJECT_LOCK} has Git conflict markers"),
            Self::InvalidLock(error) => format!("{PROJECT_LOCK} is invalid: {error}"),
            Self::OneSided => format!(
                "Hive files and {PROJECT_LOCK} changed in different commits; commit them together"
            ),
            Self::Stale => format!("{PROJECT_LOCK} does not match the Hive files"),
        }
    }
}

fn parse_settings(text: &str) -> Result<Range, String> {
    let settings: Settings = toml::from_str(text).map_err(|error| error.message().to_owned())?;
    if settings.schema_version != 1 {
        return Err("schema_version must be 1".to_owned());
    }
    Range::parse(&settings.hive.range)
}

fn parse_lock(text: &str) -> Result<Lock, String> {
    let lock: Lock = toml::from_str(text).map_err(|error| error.message().to_owned())?;
    if lock.schema_version != 1 || version(&lock.hive_version).is_none() {
        return Err("schema_version or hive_version is invalid".to_owned());
    }
    Range::parse(&lock.range)?;
    if lock.files.iter().any(|file| file.path.is_empty()) {
        return Err("file path is empty".to_owned());
    }
    Ok(lock)
}

/// Compares the settings and lock with the authenticated base, without writing.
pub(crate) fn lock_issues(
    settings: Option<&str>,
    lock: Option<&str>,
    ledger_version: &str,
    ledger_digest: &str,
    expected: impl Fn(Range) -> String,
) -> Vec<LockIssue> {
    let mut issues = Vec::new();
    let range = match settings.map(parse_settings) {
        None => {
            issues.push(LockIssue::MissingSettings);
            None
        }
        Some(Err(error)) => {
            issues.push(LockIssue::InvalidSettings(error));
            None
        }
        Some(Ok(range)) => Some(range),
    };
    let Some(lock) = lock else {
        issues.push(LockIssue::MissingLock);
        return issues;
    };
    if has_conflict_markers(lock) {
        issues.push(LockIssue::ConflictMarkers);
        return issues;
    }
    match parse_lock(lock) {
        Err(error) => issues.push(LockIssue::InvalidLock(error)),
        Ok(parsed) => {
            if parsed.base_ledger_digest != ledger_digest || parsed.hive_version != ledger_version {
                issues.push(LockIssue::OneSided);
            } else if range.is_some_and(|range| expected(range) != lock) {
                issues.push(LockIssue::Stale);
            }
        }
    }
    issues
}

fn read_text(dir: &Dir, relative: &str) -> Result<Option<String>, UpdateError> {
    read_bounded_optional(dir, Path::new(relative), MAX_SETTINGS_BYTES)?
        .map(|bytes| {
            String::from_utf8(bytes)
                .map_err(|_| UpdateError::Verification(format!("{relative} is not UTF-8")))
        })
        .transpose()
}

fn write_text(dir: &Dir, relative: &str, text: &str) -> Result<(), UpdateError> {
    let temporary = format!("{relative}.tmp-sync");
    dir.write(&temporary, text.as_bytes())
        .and_then(|()| dir.rename(&temporary, dir, relative))
        .map_err(|error| {
            let _ = dir.remove_file(&temporary);
            UpdateError::Internal(format!("could not write {relative}: {error}"))
        })
}

/// True only for a real `.hive` directory; a missing or symlinked one means "not a Hive project".
fn has_hive(target: &Path) -> bool {
    std::fs::symlink_metadata(target.join(".hive")).is_ok_and(|meta| meta.is_dir())
}

fn git(target: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .args(arguments)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Why an automatic sync waits for a later session, if it must.
pub(crate) fn skip_reason(target: &Path, managed: &[String]) -> Option<&'static str> {
    let git_dir = git(target, &["rev-parse", "--absolute-git-dir"])?;
    let git_dir = PathBuf::from(git_dir);
    for marker in [
        "rebase-merge",
        "rebase-apply",
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
    ] {
        if git_dir.join(marker).exists() {
            return Some("rebase-or-merge");
        }
    }
    let mut arguments = vec![
        "status",
        "--porcelain",
        "--untracked-files=no",
        "--",
        PROJECT_TOML,
        PROJECT_LOCK,
        BASE_PATH,
        ".hive/config",
        ".hive/setup-answers.yml",
    ];
    arguments.extend(managed.iter().map(String::as_str));
    let status = git(target, &arguments)?;
    (!status.is_empty()).then_some("uncommitted-hive-changes")
}

fn result(code: &'static str, message: String, changed: Vec<String>, data: Value) -> ProjectResult {
    ProjectResult {
        schema_version: 1,
        action: ACTION,
        status: "success",
        exit_code: 0,
        code,
        message,
        changed_paths: changed,
        evidence: Vec::new(),
        next_action: None,
        data: Some(data),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Apply,
    Check,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Hook {
    None,
    Claude,
    Codex,
}

struct SyncArguments {
    target: PathBuf,
    mode: Mode,
    hook: Hook,
}

fn parse(arguments: &[String]) -> Result<SyncArguments, UpdateError> {
    let mut target = None;
    let mut mode = Mode::Apply;
    let mut hook = Hook::None;
    let mut stdin_json = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--target" => {
                index += 1;
                target = Some(PathBuf::from(arguments.get(index).ok_or_else(|| {
                    UpdateError::Input("--target requires a directory".to_owned())
                })?));
            }
            "--check" => mode = Mode::Check,
            "--apply" => mode = Mode::Apply,
            "--hook" => {
                index += 1;
                hook = match arguments.get(index).map(String::as_str) {
                    Some("claude") => Hook::Claude,
                    Some("codex") => Hook::Codex,
                    _ => {
                        return Err(UpdateError::Input(
                            "--hook must be claude or codex".to_owned(),
                        ))
                    }
                };
            }
            "--stdin-json" => stdin_json = true,
            "--output" => index += 1,
            other => {
                return Err(UpdateError::Input(format!(
                    "unknown project sync option {other}"
                )))
            }
        }
        index += 1;
    }
    if target.is_none() && stdin_json {
        let mut input = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin().take(64 * 1024), &mut input)
            .map_err(|error| UpdateError::Input(error.to_string()))?;
        let value: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
        target = value["cwd"].as_str().map(PathBuf::from);
    }
    let target = match target {
        Some(target) => target,
        None if hook != Hook::None => {
            std::env::current_dir().map_err(|error| UpdateError::Input(error.to_string()))?
        }
        None => {
            return Err(UpdateError::Input(
                "project sync requires --target <dir>".to_owned(),
            ))
        }
    };
    if mode == Mode::Check && hook != Hook::None {
        return Err(UpdateError::Input(
            "--check cannot run as a hook".to_owned(),
        ));
    }
    Ok(SyncArguments { target, mode, hook })
}

use std::io::Read as _;

fn check(target: &Path) -> Result<ProjectResult, UpdateError> {
    let dir = open_target_capability(target)?;
    ensure_pinned_consumer_target(&dir)?;
    let Some(ledger) = read_base_ledger(&dir, None)? else {
        return Ok(result(
            "hive.project-sync-no-base",
            "project has no Hive base ledger; nothing to check".to_owned(),
            Vec::new(),
            json!({"mode": "check"}),
        ));
    };
    let settings = read_text(&dir, PROJECT_TOML)?;
    let lock = read_text(&dir, PROJECT_LOCK)?;
    let issues = lock_issues(
        settings.as_deref(),
        lock.as_deref(),
        &ledger.product_version,
        &ledger.ledger_digest,
        |range| expected_lock(&ledger, range),
    );
    if issues.is_empty() {
        return Ok(result(
            "hive.project-sync-checked",
            "project Hive files match project.lock".to_owned(),
            Vec::new(),
            json!({"mode": "check", "hive_version": ledger.product_version}),
        ));
    }
    let messages: Vec<String> = issues.iter().map(LockIssue::message).collect();
    Err(UpdateError::Verification(messages.join("; ")))
}

#[allow(clippy::too_many_lines)]
fn sync(target: &Path) -> Result<ProjectResult, UpdateError> {
    let dir = open_target_capability(target)?;
    ensure_pinned_consumer_target(&dir)?;
    if read_bounded_optional(&dir, Path::new(JOURNAL_PATH), MAX_LEDGER_BYTES)?.is_some() {
        return Err(UpdateError::RecoveryRequired(
            "project upgrade journal requires explicit hive project upgrade --recover".to_owned(),
        ));
    }
    let Some(ledger) = read_base_ledger(&dir, None)? else {
        return Ok(result(
            "hive.project-sync-no-base",
            "project has no Hive base ledger; run project setup once to start automatic sync"
                .to_owned(),
            Vec::new(),
            json!({}),
        ));
    };
    let managed: Vec<String> = ledger.files.iter().map(|file| file.path.clone()).collect();
    if let Some(reason) = skip_reason(target, &managed) {
        return Ok(result(
            "hive.project-sync-skipped",
            format!("project sync waits for a later session: {reason}"),
            Vec::new(),
            json!({"reason": reason}),
        ));
    }
    let global = env!("CARGO_PKG_VERSION");
    let global_version = version(global)
        .ok_or_else(|| UpdateError::Internal("product version is invalid".to_owned()))?;
    let ledger_version = version(&ledger.product_version)
        .ok_or_else(|| UpdateError::Verification("project base version is invalid".to_owned()))?;
    let settings_text = read_text(&dir, PROJECT_TOML)?;
    let range = match settings_text.as_deref() {
        Some(text) => parse_settings(text)
            .map_err(|error| UpdateError::Input(format!("{PROJECT_TOML} is invalid: {error}")))?,
        None => Range::of(ledger_version),
    };
    let lock_text = read_text(&dir, PROJECT_LOCK)?;
    let lock_version = lock_text
        .as_deref()
        .filter(|text| !has_conflict_markers(text))
        .and_then(|text| parse_lock(text).ok())
        .and_then(|lock| version(&lock.hive_version));
    let recorded = lock_version.map_or(ledger_version, |lock| lock.max(ledger_version));
    let recorded_text = format!("{}.{}.{}", recorded.0, recorded.1, recorded.2);
    let data = |extra: Value| {
        let mut value = json!({"from_version": recorded_text, "to_version": global,
            "range": range.render()});
        if let (Some(base), Value::Object(extra)) = (value.as_object_mut(), extra) {
            base.extend(extra);
        }
        value
    };
    match gate(range, recorded, global_version) {
        Gate::Older => {
            return Ok(result("hive.project-sync-older-hive", format!(
                "global Hive {global} is older than this project's Hive {recorded_text}; no files changed"),
                Vec::new(), data(json!({}))));
        }
        Gate::OutOfRange => {
            return Ok(result("hive.project-sync-out-of-range", format!(
                "global Hive {global} is outside this project's range {}; raise [hive] range in {PROJECT_TOML} to accept it",
                range.render()), Vec::new(), data(json!({}))));
        }
        Gate::Current | Gate::Upgrade => {}
    }
    let mut changed = Vec::new();
    let mut ledger = ledger;
    if gate(range, recorded, global_version) == Gate::Upgrade {
        let plan = prepare_with_reviewed_skills(
            &dir,
            &hive_render::skill_delivery::VerifiedPlugins::default(),
            None,
        )?;
        if plan
            .reports
            .iter()
            .any(|report| report.omitted_incoming_hunks > 0)
        {
            let mut needs_review = result(
                "hive.project-sync-needs-review",
                "overlapping user edits need review; no files changed".to_owned(),
                Vec::new(),
                data(json!({"conflicts": plan.reports.iter()
                    .filter(|report| report.omitted_incoming_hunks > 0)
                    .map(|report| report.path.clone()).collect::<Vec<_>>()})),
            );
            needs_review.next_action = Some(format!(
                "review with hive project upgrade --target {} --dry-run, then --apply",
                target.display()
            ));
            return Ok(needs_review);
        }
        let applied = apply(&dir, &plan)?;
        changed.extend(applied.changed_paths);
        ledger = read_base_ledger(&dir, None)?.ok_or_else(|| {
            UpdateError::Verification("project base disappeared after the update".to_owned())
        })?;
    }
    if settings_text.is_none() {
        write_text(&dir, PROJECT_TOML, &render_settings(range))?;
        changed.push(PROJECT_TOML.to_owned());
    }
    let expected = expected_lock(&ledger, range);
    let recomputed = lock_text.as_deref().is_some_and(has_conflict_markers);
    if lock_text.as_deref() != Some(expected.as_str()) {
        write_text(&dir, PROJECT_LOCK, &expected)?;
        changed.push(PROJECT_LOCK.to_owned());
    }
    let code = if changed
        .iter()
        .any(|path| path != PROJECT_TOML && path != PROJECT_LOCK)
    {
        "hive.project-synced"
    } else if changed.is_empty() {
        "hive.project-sync-current"
    } else {
        "hive.project-sync-recorded"
    };
    let mut synced = result(
        code,
        format!("project Hive matches global Hive {global}"),
        changed,
        data(json!({"lock_recomputed": recomputed})),
    );
    synced.evidence.push(ProjectEvidence {
        kind: "file",
        locator: PROJECT_LOCK.to_owned(),
        digest: sha256_digest(expected.as_bytes()),
    });
    Ok(synced)
}

/// The one line a session start shows, or `None` when nothing needs saying.
fn hook_line(result: &ProjectResult) -> Option<String> {
    let ko = crate::human_output::language() == crate::user_setup::InterfaceLanguage::Ko;
    let data = result.data.clone().unwrap_or(Value::Null);
    let from = data["from_version"].as_str().unwrap_or("");
    let to = data["to_version"].as_str().unwrap_or("");
    match result.code {
        "hive.project-synced" => {
            let mut value = serde_json::to_value(result).ok()?;
            crate::notice::decorate(&mut value, "project");
            let text = crate::notice::render(&value, crate::human_output::language())?;
            text.lines().next().map(str::to_owned)
        }
        "hive.project-sync-needs-review" => Some(if ko {
            format!("Eileithyia 호출 · 프로젝트 Hive {from} → {to}: 겹치는 수정이 있어 바꾸지 않음. `hive project upgrade --target . --dry-run`으로 확인")
        } else {
            format!("Eileithyia called · project Hive {from} → {to}: overlapping edits, nothing changed. Review with `hive project upgrade --target . --dry-run`")
        }),
        "hive.project-sync-older-hive" => Some(if ko {
            format!("Eileithyia 호출 · 전역 Hive {to}가 이 프로젝트의 {from}보다 오래되어 파일을 바꾸지 않음")
        } else {
            format!("Eileithyia called · global Hive {to} is older than this project's {from}; nothing changed")
        }),
        "hive.project-sync-out-of-range" => Some(if ko {
            format!("Eileithyia 호출 · 전역 Hive {to}가 프로젝트 범위 {} 밖이라 바꾸지 않음. .hive/project.toml의 range를 올리면 적용", data["range"].as_str().unwrap_or(""))
        } else {
            format!("Eileithyia called · global Hive {to} is outside the project range {}; raise range in .hive/project.toml to apply it", data["range"].as_str().unwrap_or(""))
        }),
        _ => None,
    }
}

pub(super) fn run(arguments: &[String]) -> ExitCode {
    let parsed = match parse(arguments) {
        Ok(parsed) => parsed,
        Err(error) => {
            let mut failed = failure(&error);
            failed.action = ACTION;
            super::emit(&failed);
            return ExitCode::from(failed.exit_code);
        }
    };
    if parsed.hook != Hook::None
        && std::env::var("HIVE_PROJECT_SYNC").is_ok_and(|value| value == "off")
    {
        return ExitCode::SUCCESS;
    }
    // A project without `.hive/` is never written, prompted or offered setup (OV170-032).
    if !has_hive(&parsed.target) {
        if parsed.hook == Hook::None {
            super::emit(&result(
                "hive.project-sync-no-hive",
                "this folder has no .hive/; Hive changes nothing here".to_owned(),
                Vec::new(),
                json!({}),
            ));
        }
        return ExitCode::SUCCESS;
    }
    let outcome = match parsed.mode {
        Mode::Check => check(&parsed.target),
        Mode::Apply => sync(&parsed.target),
    };
    let result = outcome.unwrap_or_else(|error| {
        let mut failed = failure(&error);
        failed.action = ACTION;
        failed
    });
    match parsed.hook {
        Hook::None => {
            super::emit(&result);
            ExitCode::from(result.exit_code)
        }
        // Hooks never block a session: the line is the whole output, and failures stay quiet.
        Hook::Claude => {
            if let Some(line) = hook_line(&result) {
                println!(
                    "{}",
                    json!({"systemMessage": line, "hookSpecificOutput": {
                    "hookEventName": "SessionStart", "additionalContext": line}})
                );
            }
            ExitCode::SUCCESS
        }
        Hook::Codex => {
            if let Some(line) = hook_line(&result) {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_accepts_major_minor_only() {
        assert_eq!(
            Range::parse("0.17").unwrap(),
            Range {
                major: 0,
                minor: 17
            }
        );
        assert_eq!(
            Range::parse("0.17.x").unwrap(),
            Range {
                major: 0,
                minor: 17
            }
        );
        for bad in ["0", "0.17.1", "^0.17", "0.x", "", "a.b", "0.17 "] {
            assert!(Range::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn gate_is_forward_only_and_range_bound() {
        let range = Range::parse("0.17").unwrap();
        assert_eq!(gate(range, (0, 17, 1), (0, 17, 2)), Gate::Upgrade);
        assert_eq!(gate(range, (0, 17, 2), (0, 17, 2)), Gate::Current);
        assert_eq!(gate(range, (0, 17, 2), (0, 17, 1)), Gate::Older);
        assert_eq!(gate(range, (0, 17, 2), (0, 18, 0)), Gate::OutOfRange);
        assert_eq!(
            gate(Range::parse("0.18").unwrap(), (0, 17, 2), (0, 18, 0)),
            Gate::Upgrade
        );
        assert_eq!(version("0.17.0-test.3"), Some((0, 17, 0)));
        assert_eq!(version("0.17"), None);
    }

    fn lock(version: &str, digest: &str) -> String {
        render_lock(
            version,
            digest,
            Range::parse("0.17").unwrap(),
            &[("AGENTS.md".to_owned(), "sha256:aa".to_owned())],
        )
    }

    #[test]
    fn check_detects_missing_conflicted_one_sided_and_stale_locks() {
        let settings = render_settings(Range::parse("0.17").unwrap());
        let good = lock("0.17.1", "sha256:base");
        let expected = |_| good.clone();
        let issues = |settings: Option<&str>, lock: Option<&str>| {
            lock_issues(settings, lock, "0.17.1", "sha256:base", expected)
        };
        assert_eq!(
            issues(Some(&settings), Some(&good)),
            Vec::<LockIssue>::new()
        );
        assert_eq!(issues(None, Some(&good)), vec![LockIssue::MissingSettings]);
        assert_eq!(issues(Some(&settings), None), vec![LockIssue::MissingLock]);
        let conflicted = format!(
            "<<<<<<< ours\n{good}=======\n{}>>>>>>> theirs\n",
            lock("0.17.2", "sha256:x")
        );
        assert_eq!(
            issues(Some(&settings), Some(&conflicted)),
            vec![LockIssue::ConflictMarkers]
        );
        assert_eq!(
            issues(Some(&settings), Some(&lock("0.17.2", "sha256:other"))),
            vec![LockIssue::OneSided]
        );
        let edited = good.replace("sha256:aa", "sha256:bb");
        assert_eq!(
            issues(Some(&settings), Some(&edited)),
            vec![LockIssue::Stale]
        );
        assert!(matches!(
            issues(
                Some("schema_version = 1\n[hive]\nrange = \"latest\"\n"),
                Some(&good)
            )[0],
            LockIssue::InvalidSettings(_)
        ));
    }

    #[test]
    fn settings_and_lock_are_valid_toml_and_name_protection_classes() {
        let settings = render_settings(Range::parse("0.17").unwrap());
        assert_eq!(
            parse_settings(&settings).unwrap(),
            Range::parse("0.17").unwrap()
        );
        assert!(settings.contains("[sections.skills]\npath = \".hive/config/approved-skills.yml\"\nprotection = \"user-consent-protected\""));
        let parsed = parse_lock(&lock("0.17.1", "sha256:base")).unwrap();
        assert_eq!(parsed.files.len(), 1);
        assert!(!has_conflict_markers(&settings));
    }

    #[test]
    fn folders_without_hive_are_never_written() {
        let folder = tempfile::tempdir().unwrap();
        assert!(!has_hive(folder.path()));
        let status = run(&[
            "--target".to_owned(),
            folder.path().display().to_string(),
            "--hook".to_owned(),
            "claude".to_owned(),
        ]);
        assert_eq!(format!("{status:?}"), format!("{:?}", ExitCode::SUCCESS));
        assert_eq!(
            std::fs::read_dir(folder.path()).unwrap().count(),
            0,
            "no file may appear"
        );
    }
}
