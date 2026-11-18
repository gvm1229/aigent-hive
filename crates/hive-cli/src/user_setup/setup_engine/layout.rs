//! Readable terminal layout for setup questions and previews.
//!
//! Layout is the same with or without decoration. Colour and bold appear only on an interactive
//! terminal without `NO_COLOR`; otherwise the same lines are plain text without control bytes.
use serde_json::Value;
use std::fmt::Write as _;

/// Output width that every laid-out line fits.
pub(super) const WIDTH: usize = 80;
/// Indent for the help, choice and default block under a title line.
pub(super) const INDENT: &str = "  ";

/// Whether decoration is allowed for one output stream. `escape_codes` is false for a Windows
/// console that may not interpret escape sequences (outside Windows Terminal or VS Code).
pub(super) fn decoration_allowed(
    stream_is_terminal: bool,
    no_color: Option<&str>,
    escape_codes: bool,
) -> bool {
    stream_is_terminal && escape_codes && no_color.is_none_or(str::is_empty)
}

#[cfg(not(test))]
fn decorated() -> bool {
    use std::io::IsTerminal;
    static DECORATED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *DECORATED.get_or_init(|| {
        decoration_allowed(
            std::io::stderr().is_terminal(),
            std::env::var("NO_COLOR").ok().as_deref(),
            !cfg!(windows)
                || std::env::var_os("WT_SESSION").is_some()
                || std::env::var_os("TERM_PROGRAM").is_some(),
        )
    })
}

#[cfg(test)]
thread_local! {
    static FORCED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Tests are plain unless one test turns decoration on for its own thread.
#[cfg(test)]
fn decorated() -> bool {
    FORCED.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(super) fn force_decoration(enabled: bool) {
    FORCED.with(|forced| forced.set(enabled));
}

fn paint(text: &str, code: &str) -> String {
    if decorated() {
        format!("\u{1b}[{code}m{text}\u{1b}[0m")
    } else {
        text.to_owned()
    }
}

/// A question title or preview heading.
pub(super) fn title(text: &str) -> String {
    paint(text, "1;36")
}

/// A preview section name.
pub(super) fn section(text: &str) -> String {
    paint(text, "1")
}

/// The input marker on its own line. It stays plain: terminal drivers wait for the exact `> `.
pub(super) fn input_marker() -> &'static str {
    ">"
}

/// The line before a confirmation question.
pub(super) fn separator() -> String {
    "-".repeat(WIDTH - 8)
}

/// Terminal columns for one character: wide East Asian characters take two columns.
fn char_width(character: char) -> usize {
    let code = u32::from(character);
    if matches!(code,
        0x1100..=0x115F | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF | 0xA000..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6)
    {
        2
    } else {
        1
    }
}

/// Display columns of plain text.
pub(super) fn width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

/// Wraps plain text at spaces so that each line, with its indent, fits `WIDTH` columns.
/// A word longer than the line stays whole on its own line.
pub(super) fn wrap(text: &str, first: &str, rest: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = first.to_owned();
    let mut has_word = false;
    for word in text.split_whitespace() {
        if has_word && width(&line) + 1 + width(word) > WIDTH {
            lines.push(std::mem::replace(&mut line, rest.to_owned()));
            has_word = false;
        }
        if has_word {
            line.push(' ');
        }
        line.push_str(word);
        has_word = true;
    }
    lines.push(line);
    lines
}

/// Wraps each source line of a block under one indent.
pub(super) fn block(text: &str, indent: &str) -> Vec<String> {
    text.lines()
        .flat_map(|line| wrap(line, indent, indent))
        .collect()
}

/// One preview row: a short name and one or more value lines.
pub(super) struct Row {
    pub name: String,
    pub values: Vec<String>,
}

/// Renders `name: value` rows with aligned values. Extra value lines align under the first.
pub(super) fn table(rows: &[Row]) -> String {
    let name_width = rows.iter().map(|row| width(&row.name)).max().unwrap_or(0);
    let mut output = String::new();
    for row in rows {
        let pad = " ".repeat(name_width - width(&row.name));
        let lead = format!("{INDENT}{}{pad} : ", row.name);
        let rest = " ".repeat(width(&lead));
        for (index, value) in row.values.iter().enumerate() {
            let first = if index == 0 {
                lead.as_str()
            } else {
                rest.as_str()
            };
            for line in wrap(value, first, &rest) {
                writeln!(output, "{line}").expect("formatting a String");
            }
        }
    }
    output
}

/// Display lines for one preview value: empty values read `None`; lists use one line per item.
pub(super) fn values(
    question: &hive_core::setup::Question,
    value: &serde_json::Value,
    korean: bool,
) -> Vec<String> {
    let none = if korean { "없음" } else { "None" };
    let display = |item: &serde_json::Value| super::terminal::display_value(question, item, korean);
    match value {
        Value::Null => vec![none.to_owned()],
        Value::String(text) if text.trim().is_empty() => vec![none.to_owned()],
        Value::Array(items) if items.is_empty() => vec![none.to_owned()],
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Object(record) => record
                    .get("role_id")
                    .or_else(|| record.get("name"))
                    .and_then(Value::as_str)
                    .map_or_else(|| item.to_string(), str::to_owned),
                Value::String(_) if question.input == hive_core::setup::InputKind::Structured => {
                    item.as_str().unwrap_or_default().to_owned()
                }
                _ => display(item),
            })
            .collect(),
        Value::Object(record) if record.get("mode").and_then(Value::as_str).is_some() => {
            // A feature bundle: its mode, then the selected names wrapped several per line.
            let selected: Vec<&str> = record
                .get("selected")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            let mode = match (record["mode"].as_str(), korean) {
                (Some("all"), true) => "전체".to_owned(),
                (Some("all"), false) => "All".to_owned(),
                (Some("individual"), true) => format!("개별 선택 {}개", selected.len()),
                (Some("individual"), false) => format!("Individual, {}", selected.len()),
                (Some(other), _) => other.to_owned(),
                (None, _) => none.to_owned(),
            };
            let mut lines = vec![mode];
            if !selected.is_empty() {
                lines.push(selected.join(", "));
            }
            lines
        }
        Value::Object(record) => {
            let mut lines = Vec::new();
            for (key, item) in record {
                match item {
                    Value::Array(items) if items.is_empty() => lines.push(format!("{key}: {none}")),
                    Value::Array(items) => {
                        lines.push(format!("{key}:"));
                        lines.extend(items.iter().map(|item| {
                            format!(
                                "- {}",
                                item.as_str()
                                    .map_or_else(|| item.to_string(), str::to_owned)
                            )
                        }));
                    }
                    Value::String(text) => lines.push(format!("{key}: {text}")),
                    other => lines.push(format!("{key}: {other}")),
                }
            }
            if lines.is_empty() {
                lines.push(none.to_owned());
            }
            lines
        }
        _ => vec![display(value)],
    }
}

/// Short title; a per-host question names its app first, such as `Claude Code · Guarded limits`.
pub(super) fn display_title(id: &str, korean: bool) -> String {
    super::host_windows::instance(id).map_or_else(
        || question_title(id, korean).to_owned(),
        |(template, host)| {
            // A provider-only host answers one threshold, titled like the single legacy one.
            let title = if host == super::host_windows::PROVIDER_ONLY_HOST {
                question_title("usage-threshold", korean)
            } else {
                question_title(template, korean)
            };
            format!("{} · {title}", super::host_windows::host_label(host))
        },
    )
}

/// Short title for a question; the full question sentence stays in the block below it.
pub(super) fn question_title(id: &str, korean: bool) -> &'static str {
    let (ko, en) = match id {
        "user-language" | "project-language" => ("언어", "Language"),
        "user-mode" | "project-mode" => ("설정 방식", "Setup mode"),
        "user-section" => ("변경할 설정", "Settings to change"),
        "user-wiki" | "project-wiki" => ("지식 저장", "Knowledge storage"),
        "user-wiki-language" | "project-wiki-language" => ("지식 문서 언어", "Knowledge language"),
        "user-index-rebuild" => ("검색 색인 복구", "Search index repair"),
        "user-vector-search" => ("의미 검색", "Semantic search"),
        "user-persona" | "project-persona" => ("답변 방식", "Response style"),
        "user-persona-text" | "project-persona-text" => ("답변 방식 설명", "Response style text"),
        "user-judge" => ("독립 검토", "Independent review"),
        "user-skills" | "project-skills" => ("기능 묶음", "Feature bundle"),
        "user-contexts" => ("배경과 관심사", "Background"),
        "user-context-description" => ("추가 배경", "More background"),
        "user-hosts" => ("사용할 앱", "Apps"),
        "user-update-check" => ("새 버전 확인", "Update check"),
        "usage-enforcement" => ("사용량 보호", "Usage protection"),
        "usage-warnings" => ("사용량 안내", "Usage notices"),
        "usage-threshold" | "project-threshold" => ("잔여량 기준", "Remaining-quota threshold"),
        "usage-project-overrides" => ("프로젝트별 기준", "Project thresholds"),
        "usage-host-five-hour-limit" => ("5시간 한도 여부", "5-hour limit"),
        "usage-host-guard" => ("보호할 한도", "Guarded limits"),
        "usage-host-five-hour-threshold" => ("5시간 잔여량 기준", "5-hour threshold"),
        "usage-host-weekly-threshold" => ("주간 잔여량 기준", "Weekly threshold"),
        "usage-preflight-threshold" => ("시작 전 기준 이하", "Before start: threshold"),
        "usage-preflight-reset" => ("시작 전 기간 초기화", "Before start: period reset"),
        "usage-preflight-zero-quota" => ("시작 전 0%", "Before start: zero quota"),
        "usage-preflight-failure" => ("시작 전 조회 실패", "Before start: failed check"),
        "usage-monitor" => ("실행 중 감시", "Usage monitoring"),
        "user-monitor-activation" => ("감시 등록", "Monitor registration"),
        "user-monitor-withdrawal" => ("감시 동의 철회", "Monitor consent withdrawal"),
        "usage-midway-threshold" => ("실행 중 기준 이하", "During work: threshold"),
        "usage-midway-reset" => ("실행 중 기간 초기화", "During work: period reset"),
        "usage-midway-zero-quota" => ("실행 중 0%", "During work: zero quota"),
        "usage-midway-failure" => ("실행 중 조회 실패", "During work: failed check"),
        "codexbar-fallback" => ("CodexBar 대체 조회", "CodexBar fallback"),
        "discord-enabled" => ("Discord 알림", "Discord notices"),
        "discord-environment" => ("Discord 환경 변수", "Discord variable"),
        "discord-privacy" => ("Discord 공개 범위", "Discord privacy"),
        "discord-fields" => ("Discord 항목", "Discord fields"),
        "monitor-popup" => ("중단 알림 창", "Interruption notice"),
        "project-name" => ("프로젝트 이름", "Project name"),
        "project-kind" => ("프로젝트 종류", "Project kind"),
        "project-host" => ("사용할 앱", "App"),
        "project-roles" => ("역할", "Roles"),
        "project-role-update" => ("역할 변경 승인", "Role change approval"),
        "project-extras" => ("선택 기능 검토", "Optional feature review"),
        "project-extras-input" => ("선택 기능과 후크", "Optional features and hooks"),
        "project-extras-confirm" => ("선택 기능 승인", "Optional feature approval"),
        "project-knowledge-include" => ("지식 포함 경로", "Knowledge include paths"),
        "project-knowledge-exclude" => ("지식 제외 경로", "Knowledge exclude paths"),
        "project-knowledge-promotion" => ("전역 공유 종류", "Globally shared kinds"),
        "project-knowledge-confidential" => ("비공개 종류", "Confidential kinds"),
        "project-knowledge-root" => ("지식 공유 대상", "Knowledge destination"),
        _ => ("질문", "Question"),
    };
    if korean {
        ko
    } else {
        en
    }
}

/// Compares a plain screen with its checked-in snapshot; `HIVE_UPDATE_SNAPSHOTS` rewrites it.
#[cfg(test)]
pub(super) fn assert_snapshot(name: &str, actual: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/setup/screens")
        .join(name);
    // Snapshots drop trailing spaces (such as after the `> ` input marker) so they stay clean text.
    let trimmed = actual.lines().fold(String::new(), |mut text, line| {
        writeln!(text, "{}", line.trim_end()).expect("formatting a String");
        text
    });
    if std::env::var_os("HIVE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, &trimmed).unwrap();
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(trimmed, expected, "snapshot {name}");
    assert!(!actual.contains('\u{1b}'));
    for line in actual.lines() {
        assert!(width(line) <= WIDTH, "{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoration_needs_a_terminal_and_no_no_color_value() {
        assert!(decoration_allowed(true, None, true));
        assert!(decoration_allowed(true, Some(""), true));
        assert!(!decoration_allowed(true, Some("1"), true));
        assert!(!decoration_allowed(false, None, true));
        assert!(!decoration_allowed(true, None, false));
    }

    #[test]
    fn wrapping_counts_wide_characters_and_keeps_the_indent() {
        let text = "가나다라 ".repeat(30);
        for line in wrap(&text, "  ", "    ") {
            assert!(width(&line) <= WIDTH, "{line}");
            assert!(line.starts_with("  "));
        }
        assert_eq!(wrap("", "  ", "  "), vec!["  ".to_owned()]);
    }

    #[test]
    fn every_catalog_question_has_a_short_title() {
        for question in super::super::catalog().unwrap().questions {
            for korean in [true, false] {
                let title = question_title(&question.id, korean);
                assert_ne!(
                    title,
                    if korean { "질문" } else { "Question" },
                    "{}",
                    question.id
                );
            }
        }
    }

    #[test]
    fn table_aligns_values_and_continues_lists_under_the_first_value() {
        let rows = [
            Row {
                name: "이름".into(),
                values: vec!["a".into()],
            },
            Row {
                name: "Kind".into(),
                values: vec!["b".into(), "c".into()],
            },
        ];
        assert_eq!(table(&rows), "  이름 : a\n  Kind : b\n         c\n");
    }

    #[test]
    fn host_window_question_names_the_app_in_the_step_title() {
        let answers = serde_json::json!({"selected_hosts":["claude"]});
        let questions = super::super::host_windows::expand(
            &super::super::catalog().unwrap().questions,
            &answers,
        );
        let current = serde_json::json!("both");
        for (language, name) in [
            (
                crate::user_setup::InterfaceLanguage::Ko,
                "host-guard.ko.txt",
            ),
            (
                crate::user_setup::InterfaceLanguage::En,
                "host-guard.en.txt",
            ),
        ] {
            let mut screen = String::new();
            for (id, value, step, input) in [
                ("usage-host-five-hour-limit-claude", None, (5, 9), "yes\n"),
                ("usage-host-guard-claude", Some(&current), (6, 9), "\n"),
            ] {
                let question = questions.iter().find(|q| q.id == id).unwrap();
                let mut terminal = super::super::terminal::Terminal {
                    reader: std::io::Cursor::new(input.as_bytes()),
                    writer: Vec::new(),
                    language,
                };
                terminal.ask_step(question, value, Some(step)).unwrap();
                screen.push_str(&String::from_utf8(terminal.writer).unwrap());
            }
            assert_snapshot(name, &screen);
        }
    }

    #[test]
    fn user_preview_matches_the_plain_80_column_snapshot() {
        for (language, name) in [
            (
                crate::user_setup::InterfaceLanguage::Ko,
                "user-preview.ko.txt",
            ),
            (
                crate::user_setup::InterfaceLanguage::En,
                "user-preview.en.txt",
            ),
        ] {
            let mut terminal = super::super::terminal::Terminal {
                reader: std::io::Cursor::new(b""),
                writer: Vec::new(),
                language,
            };
            let answers = super::super::default_answers(None);
            super::super::interactive::display_preview(&answers, &mut terminal).unwrap();
            assert_snapshot(name, &String::from_utf8(terminal.writer).unwrap());
        }
    }
}
