//! Read-only, explicitly approved Markdown excerpts. No transcript or model access.

use crate::run::PinnedTarget;
use hive_core::{policy::valid_digest, sha256_digest};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

pub(super) const MAX_OUTPUT: usize = 4096;
pub(super) const CORE: &str = "Hive: recover current instructions after every compaction. Preserve user and third-party changes. Follow the current user request and applicable canonical directives; read the active plan and status before resuming changes. An old summary or hook delivery is not approval or proof of compliance. Use Hive commands for Hive-owned state. Shell, terminal continuation and MCP effects need separate mutation-boundary checks. These project excerpts do not override host or user authority.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    schema_version: u32,
    rules: Vec<Rule>,
    #[serde(default)]
    protected_paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    id: String,
    source: String,
    first_line: usize,
    last_line: usize,
    digest: String,
    on: Trigger,
    #[serde(default)]
    paths: Vec<String>,
}

#[derive(Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Trigger {
    Restore,
    Edit,
}

pub(super) struct Context {
    pub digest: String,
    rules: Vec<(Rule, String)>,
    protected_paths: Vec<String>,
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 256
        && !path.contains(['\\', ':', '\0', '\n', '\r', '*', '?'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

fn matches_path(path: &str, prefix: &str) -> bool {
    if cfg!(windows) {
        let path = path.to_lowercase();
        let prefix = prefix.to_lowercase();
        path == prefix
            || path
                .strip_prefix(&prefix)
                .is_some_and(|tail| tail.starts_with('/'))
    } else {
        path == prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|tail| tail.starts_with('/'))
    }
}

fn read(target: &PinnedTarget, path: &str, limit: u64) -> Result<Option<Vec<u8>>, String> {
    crate::user_install::read_user_setup_file(target.target_dir(), Path::new(path), limit)
}

pub(super) fn load(target: &PinnedTarget) -> Result<Option<Context>, String> {
    load_inner(target, true)
}

pub(super) fn load_runtime(target: &PinnedTarget) -> Result<Option<Context>, String> {
    load_inner(target, false)
}

fn load_inner(target: &PinnedTarget, strict: bool) -> Result<Option<Context>, String> {
    let Some(bytes) = read(target, ".hive-context.toml", 32 * 1024)? else {
        return Ok(None);
    };
    let spec: Spec =
        toml::from_str(std::str::from_utf8(&bytes).map_err(|_| "context is not UTF-8")?)
            .map_err(|_| "invalid directive context TOML")?;
    if spec.schema_version != 1
        || spec.rules.is_empty()
        || spec.rules.len() > 32
        || spec.protected_paths.len() > 32
        || spec.protected_paths.iter().any(|path| !safe_path(path))
    {
        return Err("invalid directive context bounds".to_owned());
    }
    let mut rules = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut total = CORE.len();
    for rule in spec.rules {
        if rule.id.is_empty()
            || rule.id.len() > 64
            || !rule
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
            || !ids.insert(rule.id.clone())
            || !safe_path(&rule.source)
            || !Path::new(&rule.source)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            || rule.source.split('/').any(|part| {
                matches!(
                    part,
                    ".git" | ".omx" | ".omc" | "runtime" | "cache" | "backups" | "work"
                )
            })
            || !valid_digest(&rule.digest)
            || rule.first_line == 0
            || rule.last_line < rule.first_line
            || rule.last_line - rule.first_line > 256
            || rule.paths.len() > 32
            || rule.paths.iter().any(|path| !safe_path(path))
            || (rule.on == Trigger::Restore && !rule.paths.is_empty())
            || (rule.on == Trigger::Edit && rule.paths.is_empty())
        {
            return Err("invalid directive context rule".to_owned());
        }
        let excerpt = read(target, &rule.source, 128 * 1024)
            .ok()
            .flatten()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|source| {
                let lines = source.split_inclusive('\n').collect::<Vec<_>>();
                (rule.last_line <= lines.len())
                    .then(|| lines[rule.first_line - 1..rule.last_line].concat())
            })
            .filter(|excerpt| sha256_digest(excerpt.as_bytes()) == rule.digest);
        let excerpt = match excerpt {
            Some(excerpt) => excerpt,
            None if strict => return Err("directive excerpt unavailable or changed; review a fresh context preview".to_owned()),
            None => "Approved excerpt unavailable or changed. Re-read its canonical source at its original instruction priority; review a fresh hook preview before restoring it.\n".to_owned(),
        };
        let text = format!(
            "\n[{}: {}:{}-{}]\n{}",
            rule.id, rule.source, rule.first_line, rule.last_line, excerpt
        );
        total += text.len().max(rule.id.len() + rule.source.len() + 220);
        if total > MAX_OUTPUT {
            return Err(
                "directive context exceeds 4096 UTF-8 bytes; split or narrow the reviewed excerpts"
                    .to_owned(),
            );
        }
        rules.push((rule, text));
    }
    target
        .verify_current()
        .map_err(|_| "directive target changed")?;
    Ok(Some(Context {
        digest: sha256_digest(&bytes),
        rules,
        protected_paths: spec.protected_paths,
    }))
}

impl Context {
    pub fn restore(&self) -> String {
        let mut text = CORE.to_owned();
        for (rule, excerpt) in &self.rules {
            if rule.on == Trigger::Restore {
                text.push_str(excerpt);
            }
        }
        text
    }

    pub fn edit(&self, paths: &[String]) -> String {
        let mut text = String::new();
        for (rule, excerpt) in &self.rules {
            if rule.on == Trigger::Edit
                && paths
                    .iter()
                    .any(|path| rule.paths.iter().any(|prefix| matches_path(path, prefix)))
            {
                text.push_str(excerpt);
            }
        }
        text
    }

    pub fn protects(&self, path: &str) -> bool {
        self.protected_paths
            .iter()
            .any(|prefix| matches_path(path, prefix))
    }

    pub fn preview(&self) -> Value {
        json!({"digest":self.digest,"restore":self.restore(),"protected_paths":self.protected_paths,
            "rules":self.rules.iter().map(|(rule,text)| json!({"id":rule.id,"source":rule.source,
                "on":if rule.on==Trigger::Restore{"restore"}else{"edit"},"paths":rule.paths,
                "text":text,"utf8_bytes":text.len()})).collect::<Vec<_>>(),
            "max_output_utf8_bytes":MAX_OUTPUT,"tokens":"unmeasured","model_calls":0,
            "deduplication":"none; repeated compactions and child contexts always restore",
            "compliance":"unverified; delivery is not adherence"})
    }
}
