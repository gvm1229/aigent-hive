//! God notices: who acted, on exactly which targets, and with what result (OV170-034).
//!
//! Hive builds the notice from the action result, so a session can pass it on unchanged
//! instead of summarizing from memory. The actor comes from the command family owner in
//! `harness/gods.toml`; the items come from per-action extractors below.
use crate::user_setup::InterfaceLanguage;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::Path;

/// Items shown before the rest is folded into one "and N more" line.
pub(crate) const MAX_SHOWN: usize = 10;

/// Command families that belong to no god and display as "Hive".
/// `advanced` only groups moved commands; each keeps its own owner.
#[cfg(test)]
pub(crate) const UNASSIGNED_FAMILIES: &[&str] = &["advanced"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Done,
    Skipped,
    Failed,
    Planned,
}

impl Outcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Planned => "planned",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "done" => Self::Done,
            "skipped" => Self::Skipped,
            "failed" => Self::Failed,
            "planned" => Self::Planned,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) target: String,
    pub(crate) outcome: Outcome,
    pub(crate) reason: Option<String>,
    pub(crate) secret: bool,
}

impl Item {
    pub(crate) fn new(target: impl Into<String>, outcome: Outcome) -> Self {
        Self {
            target: target.into(),
            outcome,
            reason: None,
            secret: false,
        }
    }

    pub(crate) fn because(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    fn to_value(&self) -> Value {
        let mut value = json!({
            "target": if self.secret { "[secret]" } else { self.target.as_str() },
            "outcome": self.outcome.as_str(),
        });
        if let Some(reason) = &self.reason {
            value["reason"] = json!(reason);
        }
        if self.secret {
            value["secret"] = json!(true);
        }
        value
    }

    fn from_value(value: &Value) -> Option<Self> {
        Some(Self {
            target: value["target"].as_str()?.to_owned(),
            outcome: Outcome::parse(value["outcome"].as_str()?)?,
            reason: value["reason"].as_str().map(str::to_owned),
            secret: value["secret"] == true,
        })
    }
}

/// Words for one kind of action: the operation and the result labels.
struct Words {
    operation: (&'static str, &'static str),
    planned: (&'static str, &'static str),
    done: (&'static str, &'static str),
    skipped: (&'static str, &'static str),
    nothing: (&'static str, &'static str),
}

const KNOWLEDGE: Words = Words {
    operation: ("지식 기록", "record knowledge"),
    planned: ("지식 기록 예정", "planned knowledge records"),
    done: ("새로 기록", "recorded"),
    skipped: ("같은 내용 있음", "already present"),
    nothing: ("기록할 새 지식 없음", "no new knowledge to record"),
};
const PRUNE: Words = Words {
    operation: ("오래된 파일 삭제", "remove unused files"),
    planned: ("삭제 예정", "planned removal"),
    done: ("삭제", "removed"),
    skipped: ("건너뜀", "skipped"),
    nothing: ("삭제할 파일 없음", "nothing to remove"),
};
const CHANGE: Words = Words {
    operation: ("파일 변경", "change files"),
    planned: ("변경 예정", "planned file changes"),
    done: ("변경", "changed"),
    skipped: ("건너뜀", "skipped"),
    nothing: ("바뀐 파일 없음", "no file changes"),
};
const SYNC: Words = Words {
    operation: ("프로젝트 Hive 맞춤", "align project Hive"),
    planned: ("프로젝트 Hive 맞춤 예정", "planned project Hive alignment"),
    done: ("변경", "changed"),
    skipped: ("건너뜀", "skipped"),
    nothing: ("맞출 파일 없음", "project Hive already aligned"),
};

/// Write actions that carry a notice. Read-only actions stay quiet.
fn words(action: &str) -> Option<&'static Words> {
    Some(match action {
        "RememberKnowledge" => &KNOWLEDGE,
        "PruneStorage" => &PRUNE,
        "SyncProjectHarness" => &SYNC,
        "AddKnowledge"
        | "DeleteKnowledge"
        | "SuppressKnowledge"
        | "PromoteKnowledge"
        | "ImportKnowledge"
        | "LintKnowledge"
        | "UpdateProjectHarness"
        | "SetupHarness"
        | "SetupHiveUser"
        | "InstallHiveUser"
        | "UpdateHiveUser"
        | "UninstallHiveUser"
        | "RestoreHiveBackup"
        | "ManageHiveBackups" => &CHANGE,
        _ => return None,
    })
}

/// Owner god of a command family, from `harness/gods.toml`.
pub(crate) fn actor_for(family: &str) -> Option<&'static str> {
    hive_core::gods::GODS
        .iter()
        .find(|god| god.commands.contains(&family))
        .map(|god| god.display_name)
}

fn kind_label(kind: &str, ko: bool) -> &str {
    match (kind, ko) {
        ("decision", true) => "결정",
        ("preference", true) => "선호",
        ("convention", true) => "관례",
        ("workflow", true) => "작업 방식",
        ("project-profile", true) => "프로젝트 정보",
        ("dependency-evidence", true) => "의존성 근거",
        ("outcome", true) => "결과",
        ("question", true) => "질문",
        (other, _) => other,
    }
}

/// Paths under the working directory become relative; the home directory becomes `~`.
pub(crate) fn display_path(path: &str) -> String {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        if let Ok(current) = std::env::current_dir() {
            if let Ok(relative) = candidate.strip_prefix(&current) {
                return relative.to_string_lossy().replace('\\', "/");
            }
        }
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            if let Ok(relative) = candidate.strip_prefix(Path::new(&home)) {
                return format!("~/{}", relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    path.replace('\\', "/")
}

fn planned(value: &Value) -> bool {
    let code = value["code"].as_str().unwrap_or("");
    code.contains("preview")
        || code.contains("dry-run")
        || code.ends_with("-planned")
        || value["data"]["mode"] == "dry-run"
}

fn knowledge_items(data: &Value) -> Vec<Item> {
    data["facts"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|fact| {
            let kind = fact["kind"].as_str().unwrap_or("");
            let subject = fact["subject"].as_str().unwrap_or("");
            let text = fact["normalized_fact"].as_str().unwrap_or("");
            let target = if subject.is_empty() {
                format!("[{kind}] {text}")
            } else {
                format!("[{kind}] {subject}: {text}")
            };
            let outcome = if fact["disposition"] == "noop" {
                Outcome::Skipped
            } else {
                Outcome::Done
            };
            Item {
                target,
                outcome,
                reason: None,
                secret: fact["visibility"] == "confidential",
            }
        })
        .collect()
}

fn prune_items(data: &Value, planned: bool) -> Vec<Item> {
    let report = &data["report"];
    let paths = |key: &str| report[key].as_array().cloned().unwrap_or_default();
    if planned {
        return paths("candidates")
            .iter()
            .filter_map(Value::as_str)
            .map(|path| Item::new(display_path(path), Outcome::Planned))
            .collect();
    }
    let mut items: Vec<Item> = paths("removed")
        .iter()
        .filter_map(Value::as_str)
        .map(|path| Item::new(display_path(path), Outcome::Done))
        .collect();
    for skipped in paths("skipped") {
        let (path, reason) = match &skipped {
            Value::String(path) => (path.clone(), None),
            other => (
                other["path"].as_str().unwrap_or("").to_owned(),
                other["reason"].as_str().map(str::to_owned),
            ),
        };
        let item = Item::new(display_path(&path), Outcome::Skipped);
        items.push(match reason {
            Some(reason) => item.because(reason),
            None => item,
        });
    }
    items
}

fn path_items(value: &Value, planned: bool) -> Vec<Item> {
    let outcome = if planned {
        Outcome::Planned
    } else {
        Outcome::Done
    };
    value["changed_paths"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|path| Item::new(display_path(path), outcome))
        .collect()
}

/// Items for one result: explicit `items` win, then the per-action extractor.
pub(crate) fn items(value: &Value) -> Option<Vec<Item>> {
    if let Some(existing) = value["items"].as_array() {
        return Some(existing.iter().filter_map(Item::from_value).collect());
    }
    let action = value["action"].as_str()?;
    words(action)?;
    let planned = planned(value);
    let data = &value["data"];
    Some(match action {
        "RememberKnowledge" if data["facts"].is_array() => knowledge_items(data),
        "PruneStorage" if data["report"].is_object() => prune_items(data, planned),
        _ => path_items(value, planned),
    })
}

fn failed(value: &Value) -> bool {
    value["exit_code"].as_u64().is_some_and(|code| code != 0)
}

/// Adds `actor` and `items` to a successful write result. Returns whether it changed.
pub(crate) fn decorate(value: &mut Value, family: &str) -> bool {
    if failed(value) || value.get("actor").is_some() {
        return false;
    }
    let Some(items) = items(value) else {
        return false;
    };
    let actor = value["action"].as_str().and_then(|_| actor_for(family));
    let Some(object) = value.as_object_mut() else {
        return false;
    };
    object.insert(
        "actor".to_owned(),
        actor.map_or(Value::Null, |name| json!(name)),
    );
    object.insert(
        "items".to_owned(),
        Value::Array(items.iter().map(Item::to_value).collect()),
    );
    true
}

/// Serializes a result and appends `actor`/`items` after its existing fields, keeping their order.
pub(crate) fn json_line(
    result: &impl serde::Serialize,
    family: &str,
) -> serde_json::Result<String> {
    let json = serde_json::to_string(result)?;
    let mut value: Value = serde_json::from_str(&json)?;
    if !decorate(&mut value, family) {
        return Ok(json);
    }
    let mut extra = json!({"actor": value["actor"], "items": value["items"]});
    if let Some(text) = render(&value, crate::human_output::language()) {
        // The exact text a session copies into its final response (OV170-035).
        extra["notice"] = json!(text);
    }
    let extra = serde_json::to_string(&extra)?;
    let mut spliced = json;
    spliced.pop();
    spliced.push(',');
    spliced.push_str(&extra[1..]);
    Ok(spliced)
}

fn count(items: &[Item], outcome: Outcome) -> usize {
    items.iter().filter(|item| item.outcome == outcome).count()
}

/// Human notice block, or `None` when the result carries no notice.
pub(crate) fn render(value: &Value, language: InterfaceLanguage) -> Option<String> {
    let ko = language == InterfaceLanguage::Ko;
    let pick = |pair: (&'static str, &'static str)| if ko { pair.0 } else { pair.1 };
    let action = value["action"].as_str()?;
    let words = words(action)?;
    let items: Vec<Item> = value["items"]
        .as_array()?
        .iter()
        .filter_map(Item::from_value)
        .collect();
    let actor = value["actor"].as_str().unwrap_or("Hive");
    let called = if ko {
        format!("{actor} 호출")
    } else {
        format!("{actor} called")
    };
    let (done, skipped, failures, planned) = (
        count(&items, Outcome::Done),
        count(&items, Outcome::Skipped),
        count(&items, Outcome::Failed),
        count(&items, Outcome::Planned),
    );
    let versions = (
        value["data"]["from_version"].as_str(),
        value["data"]["to_version"].as_str(),
    );
    if done == 0 && failures == 0 && planned == 0 {
        let mut line = format!("{called} · {}", pick(words.nothing));
        if skipped > 0 {
            if ko {
                write!(
                    line,
                    "({} {skipped}개 이미 있음)",
                    pick(words.skipped).trim_end_matches(" 있음")
                )
            } else {
                write!(line, " ({skipped} {})", pick(words.skipped))
            }
            .expect("formatting a String");
        }
        return Some(line);
    }
    let operation = match versions {
        (Some(from), Some(to)) if action == "SyncProjectHarness" => {
            if ko {
                format!("프로젝트 Hive {from} → {to}, 파일")
            } else {
                format!("project Hive {from} → {to}, files")
            }
        }
        _ if planned > 0 && done == 0 => pick(words.planned).to_owned(),
        _ => pick(words.operation).to_owned(),
    };
    let total = items.len();
    let mut out = if ko {
        format!("{called} · {operation} {total}개")
    } else {
        format!("{called} · {operation}: {total}")
    };
    push_items(&mut out, &items, action, ko);
    if total > MAX_SHOWN {
        let rest = total - MAX_SHOWN;
        if ko {
            write!(out, "\n- 외 {rest}개(전체 목록: 같은 명령에 --output json)")
        } else {
            write!(
                out,
                "\n- and {rest} more (full list: rerun with --output json)"
            )
        }
        .expect("formatting a String");
    }
    let mut parts = Vec::new();
    if planned > 0 {
        parts.push(format!("{} {planned}", pick(words.planned)));
    }
    if done > 0 || planned == 0 {
        parts.push(format!("{} {done}", pick(words.done)));
    }
    parts.push(format!("{} {skipped}", pick(words.skipped)));
    if failures > 0 {
        parts.push(format!("{} {failures}", pick(("실패", "failed"))));
    }
    write!(out, "\n{} {}", pick(("결과:", "Result:")), parts.join(", "))
        .expect("formatting a String");
    Some(out)
}

/// One line per shown item; skipped and failed items carry their reason.
fn push_items(out: &mut String, items: &[Item], action: &str, ko: bool) {
    let pick = |pair: (&'static str, &'static str)| if ko { pair.0 } else { pair.1 };
    for item in items.iter().take(MAX_SHOWN) {
        let target = if item.secret {
            pick(("비밀 정보라 표시 안 함", "hidden: secret")).to_owned()
        } else if action == "RememberKnowledge" {
            relabel_kind(&item.target, ko)
        } else {
            item.target.clone()
        };
        out.push_str("\n- ");
        out.push_str(&target);
        let mark = match item.outcome {
            Outcome::Skipped => Some(pick(("건너뜀", "skipped"))),
            Outcome::Failed => Some(pick(("실패", "failed"))),
            Outcome::Done | Outcome::Planned => None,
        };
        if let Some(mark) = mark.filter(|_| action != "RememberKnowledge") {
            write!(out, " ({mark}").expect("formatting a String");
            if let Some(reason) = &item.reason {
                write!(out, ": {reason}").expect("formatting a String");
            }
            out.push(')');
        }
    }
}

fn relabel_kind(target: &str, ko: bool) -> String {
    let Some(rest) = target.strip_prefix('[') else {
        return target.to_owned();
    };
    let Some((kind, tail)) = rest.split_once(']') else {
        return target.to_owned();
    };
    format!("[{}]{tail}", kind_label(kind, ko))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::needless_pass_by_value)]
    fn result(action: &str, data: Value, paths: &[&str]) -> Value {
        json!({"schema_version": 1, "action": action, "status": "success", "exit_code": 0,
               "code": "hive.test", "message": "m", "changed_paths": paths, "evidence": [],
               "next_action": null, "data": data})
    }

    fn fact(kind: &str, subject: &str, text: &str, disposition: &str) -> Value {
        json!({"kind": kind, "subject": subject, "normalized_fact": text,
               "disposition": disposition, "visibility": "shared"})
    }

    #[test]
    fn knowledge_notice_names_god_and_every_fact() {
        let mut value = result(
            "RememberKnowledge",
            json!({"facts": [
                fact("decision", "hive/setup", "Hive가 없는 프로젝트에는 .hive/를 만들지 않음", "insert"),
                fact("preference", "hive/update", "전역 Hive만 업데이트하면 프로젝트가 따라옴", "insert"),
                fact("decision", "hive/update", ".hive/project.toml·.hive/project.lock 형식", "insert"),
            ]}),
            &[],
        );
        assert!(decorate(&mut value, "knowledge"));
        assert_eq!(value["actor"], "Apollon");
        assert_eq!(
            render(&value, InterfaceLanguage::Ko).unwrap(),
            "Apollon 호출 · 지식 기록 3개\n\
             - [결정] hive/setup: Hive가 없는 프로젝트에는 .hive/를 만들지 않음\n\
             - [선호] hive/update: 전역 Hive만 업데이트하면 프로젝트가 따라옴\n\
             - [결정] hive/update: .hive/project.toml·.hive/project.lock 형식\n\
             결과: 새로 기록 3, 같은 내용 있음 0"
        );
    }

    #[test]
    fn unchanged_knowledge_is_one_line() {
        let mut value = result(
            "RememberKnowledge",
            json!({"facts": [
                fact("decision", "a", "x", "noop"), fact("decision", "a", "y", "noop"),
                fact("decision", "a", "z", "noop"),
            ]}),
            &[],
        );
        decorate(&mut value, "knowledge");
        assert_eq!(
            render(&value, InterfaceLanguage::Ko).unwrap(),
            "Apollon 호출 · 기록할 새 지식 없음(같은 내용 3개 이미 있음)"
        );
        assert_eq!(
            render(&value, InterfaceLanguage::En).unwrap(),
            "Apollon called · no new knowledge to record (3 already present)"
        );
    }

    #[test]
    fn prune_notice_counts_removed_and_skipped_with_reasons() {
        let mut value = result(
            "PruneStorage",
            json!({"report": {
            "candidates": [], "removed": ["docs/plan/00.md", "docs/plan/02.md"],
            "skipped": [{"path": "docs/plan/03.md", "reason": "in use"}]}}),
            &[],
        );
        decorate(&mut value, "storage");
        assert_eq!(
            render(&value, InterfaceLanguage::Ko).unwrap(),
            "Hades 호출 · 오래된 파일 삭제 3개\n- docs/plan/00.md\n- docs/plan/02.md\n\
             - docs/plan/03.md (건너뜀: in use)\n결과: 삭제 2, 건너뜀 1"
        );
    }

    #[test]
    fn preview_lists_planned_removals_before_approval() {
        let mut value = result(
            "PruneStorage",
            json!({"mode": "dry-run", "report": {
            "candidates": ["a.exe"], "removed": [], "skipped": []}}),
            &[],
        );
        value["code"] = json!("hive.storage-prune-previewed");
        decorate(&mut value, "prune");
        assert_eq!(
            render(&value, InterfaceLanguage::Ko).unwrap(),
            "Hades 호출 · 삭제 예정 1개\n- a.exe\n결과: 삭제 예정 1, 건너뜀 0"
        );
    }

    #[test]
    fn long_lists_fold_after_ten_and_secrets_stay_hidden() {
        let paths: Vec<String> = (0..12).map(|n| format!("f{n}.md")).collect();
        let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
        let mut value = result("UpdateProjectHarness", json!({}), &refs);
        decorate(&mut value, "project");
        let text = render(&value, InterfaceLanguage::En).unwrap();
        assert!(text.starts_with("Eileithyia called · change files: 12\n- f0.md"));
        assert!(text.contains("\n- f9.md\n- and 2 more (full list: rerun with --output json)\n"));
        assert!(!text.contains("f10.md"));
        assert!(text.ends_with("Result: changed 12, skipped 0"));
        let mut secret = result(
            "RememberKnowledge",
            json!({"facts": [
            {"kind": "decision", "normalized_fact": "token is abc", "disposition": "insert",
             "visibility": "confidential"}]}),
            &[],
        );
        decorate(&mut secret, "knowledge");
        assert_eq!(secret["items"][0]["target"], "[secret]");
        let text = render(&secret, InterfaceLanguage::Ko).unwrap();
        assert!(
            text.contains("- 비밀 정보라 표시 안 함") && !text.contains("abc"),
            "{text}"
        );
    }

    #[test]
    fn failures_carry_reasons_and_unassigned_families_show_hive() {
        let mut value = result("UpdateProjectHarness", json!({}), &[]);
        value["items"] = json!([{"target": "a.md", "outcome": "done"},
                                {"target": "b.md", "outcome": "failed", "reason": "locked"}]);
        assert_eq!(
            render(&value, InterfaceLanguage::Ko).unwrap(),
            "Hive 호출 · 파일 변경 2개\n- a.md\n- b.md (실패: locked)\n결과: 변경 1, 건너뜀 0, 실패 1"
        );
    }

    #[test]
    fn sync_notice_names_versions() {
        let mut value = result(
            "SyncProjectHarness",
            json!({"from_version": "0.17.1", "to_version": "0.17.2"}),
            &["a", "b", "c", "d"],
        );
        decorate(&mut value, "project");
        let text = render(&value, InterfaceLanguage::Ko).unwrap();
        assert!(
            text.starts_with("Eileithyia 호출 · 프로젝트 Hive 0.17.1 → 0.17.2, 파일 4개\n"),
            "{text}"
        );
    }

    #[test]
    fn failed_results_and_read_only_actions_get_no_notice() {
        let mut failed = result("RememberKnowledge", json!({}), &[]);
        failed["exit_code"] = json!(2);
        assert!(!decorate(&mut failed, "knowledge"));
        let mut query = result("QueryKnowledge", json!({}), &["x"]);
        assert!(!decorate(&mut query, "knowledge"));
    }

    #[test]
    fn json_line_keeps_field_order_and_appends_notice() {
        let value = result("UpdateProjectHarness", json!({}), &["a.md"]);
        let line = json_line(&value, "project").unwrap();
        assert!(line.contains(",\"actor\":\"Eileithyia\",\"items\":[{\"outcome\":\"done\",\"target\":\"a.md\"}],\"notice\":\"Eileithyia called"), "{line}");
        let plain = result("QueryKnowledge", json!({}), &[]);
        assert_eq!(
            json_line(&plain, "knowledge").unwrap(),
            serde_json::to_string(&plain).unwrap()
        );
    }

    #[test]
    fn every_command_family_has_an_owner_or_is_explicitly_unassigned() {
        for family in crate::cli_help::COMMANDS.iter().map(|command| command.name) {
            assert!(
                actor_for(family).is_some() || UNASSIGNED_FAMILIES.contains(&family),
                "command family {family} has no god in harness/gods.toml"
            );
        }
    }
}
