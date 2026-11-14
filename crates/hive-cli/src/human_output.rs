//! One display boundary for action receipts. Machine serialization stays at its original call site.
use crate::user_setup::InterfaceLanguage;
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::fmt::Write as _;

#[derive(Clone)]
struct Options {
    json: bool,
    language: InterfaceLanguage,
    family: String,
}
thread_local! {
    // Unit tests of machine contracts keep their original mode until process configuration.
    static OPTIONS: RefCell<Options> = const { RefCell::new(Options {
        json: true, language: InterfaceLanguage::En, family: String::new(),
    }) };
}

pub(crate) fn configure(arguments: &[String]) {
    let json = arguments.windows(2).any(|pair| pair == ["--output", "json"])
        // Private worker names explicitly select an internal wire protocol, not a user command.
        || arguments.first().is_some_and(|argument| argument.starts_with("__"))
        // Hook, capture and inbound commands are read by hosts and scripts, never by a person.
        || machine_protocol(arguments);
    OPTIONS.with(|options| {
        *options.borrow_mut() = Options {
            json,
            language: crate::user_setup::output_language(arguments),
            family: arguments.first().cloned().unwrap_or_default(),
        }
    });
}

fn machine_protocol(arguments: &[String]) -> bool {
    let first = arguments.first().map(String::as_str);
    let second = arguments.get(1).map(String::as_str);
    matches!(
        (first, second),
        (Some("hook"), _)
            | (Some("policy"), Some("hook"))
            | (Some("usage"), Some("capture"))
            | (Some("discord"), Some("inbound"))
    )
}

/// Scans, validations, previews and dry runs list paths that would change, not paths that did.
fn planned_only(value: &Value, data: &Value) -> bool {
    let action = value["action"].as_str().unwrap_or("");
    let code = value["code"].as_str().unwrap_or("");
    action.starts_with("Scan")
        || action.starts_with("Validate")
        || action.starts_with("Preview")
        || code.ends_with("-available")
        || code.ends_with("-planned")
        || code.contains("preview")
        || code.contains("dry-run")
        || data["mode"] == "dry-run"
}

pub(crate) fn emit_if_human(result: &impl Serialize) -> bool {
    OPTIONS.with(|options| {
        let options = options.borrow();
        if options.json { return false; }
        let Ok(value) = serde_json::to_value(result) else {
            println!("{}", choose(options.language,
                "결과를 표시하지 못했습니다. 같은 명령을 --output json으로 다시 실행해 원인을 확인하세요.",
                "Could not display the result. Run the command with --output json to inspect the cause."));
            return true;
        };
        println!("{}", render(&value, options.language, &options.family));
        true
    })
}

fn choose(language: InterfaceLanguage, ko: &str, en: &str) -> String {
    if language == InterfaceLanguage::Ko {
        ko
    } else {
        en
    }
    .to_owned()
}

// The topic names also form the bilingual action coverage inventory.
include!("human_output/actions.rs");
include!("human_output/codes.rs");
include!("human_output/families.rs");

pub(crate) fn render(value: &Value, language: InterfaceLanguage, family: &str) -> String {
    let action = value["action"].as_str().unwrap_or(family);
    let (ko, en) = topic(action, family);
    let failed = value["exit_code"].as_u64().is_some_and(|code| code != 0)
        || matches!(
            value["status"].as_str(),
            Some("error" | "unsupported" | "conflict" | "blocked")
        );
    let message = value["message"].as_str().unwrap_or("");
    let empty = Value::Null;
    let data = value
        .get("data")
        .filter(|v| !v.is_null())
        .unwrap_or_else(|| {
            if value.get("action").is_some() {
                &empty
            } else {
                value
            }
        });
    if failed {
        return failure(value, language, &ko);
    }
    if action == "SetUsageThreshold" {
        if let Some(text) = threshold_message(data, language) {
            return text;
        }
    }
    let coded = code_sentence(value["code"].as_str().unwrap_or(""));
    let message = readable(message);
    let mut output = if language == InterfaceLanguage::Ko {
        if let Some((ko, _)) = coded {
            ko.to_owned()
        } else if has_hangul(&message) {
            // Setup already supplies complete localized messages.
            format!("완료. {message}")
        } else if message.is_empty() {
            // Topics already end with their outcome word (완료), so no second prefix.
            format!("{ko}.")
        } else {
            // Never replace a specific result with a generic topic; keep the original sentence.
            format!("{ko}.\n안내(영문 원문): {}", sentence(&message))
        }
    } else if let Some((_, en)) = coded.filter(|(_, en)| !en.is_empty()) {
        format!("Success. {en}")
    } else if message.is_empty() {
        format!("Success. {en}.")
    } else {
        format!("Success. {}", sentence(&message))
    };
    if !value["changed_paths"].as_array().is_none_or(Vec::is_empty) {
        output.push_str(&if planned_only(value, data) {
            choose(language, "\n변경 예정 파일:", "\nPlanned file changes:")
        } else {
            choose(language, "\n바뀐 파일:", "\nChanged files:")
        });
        for path in value["changed_paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            write!(output, "\n- {path}").expect("formatting a String");
        }
    }
    match family_details(action, value, data, language) {
        Some(text) => output.push_str(&text),
        None => details(data, language, &mut output, 0),
    }
    if let Some(next) = value["next_action"]
        .as_str()
        .filter(|next| !next.is_empty())
    {
        output.push_str(&choose(language, "\n다음 단계: ", "\nNext: "));
        if language == InterfaceLanguage::Ko {
            output.push_str(&next_ko(next));
        } else {
            output.push_str(&readable(next));
        }
    }
    if data["session_recheck_required"] == true {
        output.push_str(&choose(
            language,
            "\n다음 단계: 현재 세션의 사용량 사전 확인을 다시 실행하세요.",
            "\nNext: run the usage preflight again for the current session.",
        ));
    }
    if action == "PruneStorage"
        && data["mode"] == "dry-run"
        && data["report"]["candidates"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    {
        output.push_str(&choose(language,
            "\n다음 단계: 삭제 예정 파일을 확인하고, 삭제를 원할 때 --dry-run을 --confirm으로 바꾸세요.",
            "\nNext: inspect the proposed files. To approve removal, replace --dry-run with --confirm."));
    }
    output
}

fn threshold_message(data: &Value, language: InterfaceLanguage) -> Option<String> {
    let before = data["previous_remaining_percent"].as_u64()?;
    let after = data["threshold_remaining_percent"].as_u64()?;
    let mut text = if before == after {
        choose(
            language,
            &format!("확인 완료. 사용량 보호 기준은 이미 {after}%입니다. 변경한 설정이 없습니다."),
            &format!(
                "Success. Hive usage guard threshold is already {after}%. No settings changed."
            ),
        )
    } else {
        let direction = if before > after { "lowered" } else { "raised" };
        choose(
            language,
            &format!("변경 완료. 사용량 보호 기준을 {before}%에서 {after}%로 바꿨습니다."),
            &format!("Success. Hive usage guard threshold {direction} from {before}% to {after}%."),
        )
    };
    if let Some(scope) = data["scope"]
        .as_str()
        .filter(|scope| matches!(*scope, "global" | "project"))
    {
        let (ko, en) = if scope == "global" {
            ("전역 설정", "global settings")
        } else {
            ("현재 프로젝트 설정", "current project settings")
        };
        text.push_str(&choose(
            language,
            &format!("\n대상: {ko}"),
            &format!("\nScope: {en}"),
        ));
        text.push_str(&choose(
            language,
            "\n기준 보호가 켜져 있을 때 전역과 프로젝트 중 더 높은 기준을 적용합니다.",
            "\nWhen threshold protection is enabled, it uses the higher of the global and project thresholds.",
        ));
    }
    if data["session_recheck_required"] == true {
        text.push_str(&choose(
            language,
            "\n다음 단계: 현재 세션의 사용량 사전 확인을 다시 실행하세요.",
            "\nNext: run the usage preflight again for the current session.",
        ));
    }
    Some(text)
}

fn sentence(message: &str) -> String {
    // Internal component tags such as "[Hades]" stay in the JSON message only.
    let message = message
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("] "))
        .map_or(message, |(_, rest)| rest);
    let mut chars = message.chars();
    let mut text = chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().collect::<String>());
    text.push_str(chars.as_str());
    if !text.ends_with(['.', '!', '?']) {
        text.push('.');
    }
    text
}

fn failure(value: &Value, language: InterfaceLanguage, ko: &str) -> String {
    let original = value["message"]
        .as_str()
        .unwrap_or("The request could not be completed.");
    let message = readable(original.lines().next().unwrap_or(original));
    let coded = code_sentence(value["code"].as_str().unwrap_or(""));
    let diagnostic = value["data"]["diagnostic"]["original_message"]
        .as_str()
        .map(readable)
        .filter(|cause| !cause.is_empty() && *cause != message);
    let option = missing_option(&message);
    let next = value["next_action"]
        .as_str()
        .filter(|next| !next.is_empty())
        .map(str::to_owned)
        .or_else(|| setup_first(original).then(|| "hive setup".to_owned()));
    if let Some(option) =
        option.filter(|_| !next.as_deref().is_some_and(|n| n.starts_with("hive ")))
    {
        // A missing option has one exact fix; say it instead of a generic --help pointer.
        return choose(
            language,
            &format!("처리 실패: {}. 필요한 옵션 {option}을(를) 지정하지 않았습니다.\n다음 단계: {option} 값을 넣어 같은 명령을 다시 실행하세요. 사용법은 --help로 볼 수 있습니다.",
                ko.trim_end_matches(" 처리 완료").trim_end_matches(" 완료")),
            &format!("Failed. The required option {option} is missing.\nNext: add {option} and run the same command again. See --help for usage."),
        );
    }
    if language == InterfaceLanguage::En {
        let cause = coded
            .filter(|(_, en)| !en.is_empty())
            .map_or_else(|| sentence(&message), |(_, en)| en.to_owned());
        let mut output = format!("Failed. {cause}");
        if let Some(diagnostic) = diagnostic {
            write!(output, "\nCause: {}", sentence(&diagnostic)).expect("formatting a String");
        }
        let remedy = next.map_or_else(
            || {
                "Check the command options with --help, correct the named input, then retry."
                    .to_owned()
            },
            |next| readable(&next),
        );
        write!(output, "\nNext: {remedy}").expect("formatting a String");
        return output;
    }
    let topic = ko.trim_end_matches(" 처리 완료").trim_end_matches(" 완료");
    let mut output = format!("처리 실패: {topic}. ");
    if let Some((ko, _)) = coded {
        output.push_str(ko);
        if !has_hangul(&message) {
            // Keep the specific detail the code sentence cannot know.
            write!(output, "\n세부 내용(영문 원문): {}", sentence(&message))
                .expect("formatting a String");
        }
    } else if has_hangul(&message) {
        output.push_str(&message);
    } else {
        // Do not guess a Korean cause from English words; show the real one.
        write!(output, "원인(영문 원문): {}", sentence(&message)).expect("formatting a String");
    }
    if let Some(diagnostic) = diagnostic {
        write!(output, "\n원래 원인: {diagnostic}").expect("formatting a String");
    }
    output.push_str("\n다음 단계: ");
    output.push_str(&next.map_or_else(
        || "--help로 사용 방법을 확인하고, 표시된 입력을 고친 뒤 다시 실행하세요.".to_owned(),
        |next| next_ko(&next),
    ));
    output
}

fn code_sentence(code: &str) -> Option<(&'static str, &'static str)> {
    CODES
        .iter()
        .find(|(name, _, _)| *name == code)
        .map(|(_, ko, en)| (*ko, *en))
}

/// The option named by the fixed "missing required option", "missing value for" or "requires" shapes.
fn missing_option(message: &str) -> Option<String> {
    let lower = message.to_ascii_lowercase();
    let after = [
        "missing required option ",
        "missing value for ",
        " requires ",
    ]
    .iter()
    .find_map(|marker| lower.find(marker).map(|at| &message[at + marker.len()..]))?;
    let option = after
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | ',' | ';' | ':'))
        .next()?;
    // --output only chooses a machine format; never advise adding it.
    (option.starts_with("--") && option.len() > 2 && option != "--output")
        .then(|| option.to_owned())
}

fn has_hangul(text: &str) -> bool {
    text.chars().any(|c| ('가'..='힣').contains(&c))
}

/// A first run without global setup should point at `hive setup`, not at --help.
fn setup_first(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("global hive setup is required") || lower.contains(".hive/config/user-setup.yml")
}

/// What a person sees: no JSON pointers, no machine flags, no Windows verbatim path prefix.
fn readable(text: &str) -> String {
    let text = text.replace(" --output json", "").replace(r"\\?\", "");
    let kept: Vec<&str> = text
        .split_inclusive(['.', '!', '?'])
        .filter(|part| {
            let lower = part.to_ascii_lowercase();
            !(lower.contains("json") && (lower.contains("diagnostic") || lower.contains("output")))
                && !part.contains("JSON 출력")
        })
        .collect();
    kept.concat().trim().to_owned()
}

fn next_ko(next: &str) -> String {
    let readable = readable(next);
    let next = readable.as_str();
    if next.starts_with("hive ") {
        return format!("다음 명령으로 이어가세요: {next}");
    }
    if has_hangul(next) {
        return next.into();
    }
    let commands: Vec<_> = next
        .split('`')
        .enumerate()
        .filter_map(|(i, text)| (i % 2 == 1 && text.starts_with("hive ")).then_some(text))
        .collect();
    if !commands.is_empty() {
        return format!("다음 명령으로 이어가세요: {}", commands.join("; "));
    }
    format!("안내(영문 원문): {}", sentence(next))
}

fn label(key: &str, language: InterfaceLanguage) -> Option<String> {
    let (ko, en) = match key {
        "enabled" => ("사용 여부", "Enabled"),
        "active" => ("실제 동작 여부", "Active"),
        "guard_enabled" => ("일반 사용량 보호", "General usage protection"),
        // Distinct from the saved switch: whether protection is actually being applied now.
        "enforcement_enabled" => ("사용량 보호 적용 중", "Usage protection applied now"),
        "quota_reset_guard_enabled" => ("사용량 초기화 보호", "Usage reset protection"),
        "zero_quota_guard_enabled" => ("사용량 소진 보호", "Zero-quota protection"),
        "mode" => ("동작", "Mode"),
        "candidates" => ("대상 후보", "Candidates"),
        "removed" => ("제거한 항목", "Removed items"),
        "skipped" => ("건너뛴 항목", "Skipped items"),
        "preserved" => ("보존한 항목", "Preserved items"),
        "fts" => ("단어 검색 결과", "Keyword search results"),
        "relations" | "related" | "nodes" | "edges" => ("연결된 자료", "Related material"),
        "state" | "status" => ("상태", "State"),
        "count" => ("항목 수", "Items"),
        "remaining_percent" => ("남은 사용량(%)", "Remaining usage (%)"),
        "threshold_remaining_percent" => ("남은 사용량 기준(%)", "Remaining usage threshold (%)"),
        "used_percent" => ("사용한 양(%)", "Used (%)"),
        "current_version" => ("현재 버전", "Current version"),
        "latest_version" => ("최신 버전", "Latest version"),
        "update_available" => ("업데이트 가능", "Update available"),
        "host" | "provider" => ("사용하는 앱", "Host"),
        "role" => ("역할", "Role"),
        "model" => ("모델", "Model"),
        "name" | "title" => ("이름", "Name"),
        "text" | "content" | "body" | "excerpt" | "snippet" => ("내용", "Content"),
        "path" | "relative_path" | "locator" => ("위치", "Location"),
        "reason" | "message" => ("안내", "Information"),
        "decision" => ("판정", "Decision"),
        "version" => ("버전", "Version"),
        "from_version" => ("이전 버전", "Previous version"),
        "to_version" => ("갱신 버전", "Updated version"),
        "previous" | "before" => ("변경 전", "Before"),
        "current" | "after" => ("변경 후", "After"),
        "controller_state" => ("감시 실행 상태", "Monitor process state"),
        "controller_alive" => ("감시 실행 중", "Monitor process running"),
        "monitoring_verified" => ("실제 감시 확인", "Monitoring verified"),
        "saved_enabled" | "verified_saved_selection" => {
            ("저장된 사용 선택", "Saved enabled selection")
        }
        "configuration" => ("설정 확인 상태", "Configuration state"),
        "role_id" => ("역할 이름", "Role name"),
        "effort" => ("작업 수준", "Work effort"),
        "category" => ("종류", "Category"),
        "description" => ("설명", "Description"),
        "label" => ("선택지", "Choice"),
        "prompt" => ("질문", "Question"),
        "question" | "questions" | "sections" | "options" | "choices" => {
            ("설정 내용", "Setup details")
        }
        "markdown" => ("본문", "Text"),
        "source_workspace" => ("Hive 소스 폴더", "Hive source workspace"),
        "model_api_client" => ("모델 API 호출", "Model API client"),
        "setup_renderer" => ("설정 도구", "Setup renderer"),
        "project_kind" => ("프로젝트 종류", "Project kind"),
        "total_bytes" | "bytes" => ("크기(바이트)", "Size (bytes)"),
        "reset_at" | "resets_at" => ("초기화 시각", "Reset time"),
        "hits" | "items" | "entries" | "results" | "issues" | "warnings" | "files" | "roles"
        | "roster" | "snapshots" | "windows" | "observation" | "summary" | "profile"
        | "snapshot" | "five_hour" | "weekly" | "page" | "pages" | "categories" | "report"
        | "matches" | "settings_changes" | "stored" => ("확인 내용", "Details"),
        _ => return None,
    };
    Some(choose(language, ko, en))
}

fn details(value: &Value, language: InterfaceLanguage, output: &mut String, depth: usize) {
    if depth > 8 {
        return;
    }
    match value {
        Value::Object(fields) => {
            let language_key = if language == InterfaceLanguage::Ko {
                "ko"
            } else {
                "en"
            };
            if let Some(text) = fields.get(language_key).and_then(Value::as_str) {
                write!(output, "\n{text}").expect("formatting a String");
                return;
            }
            for (key, value) in fields {
                if matches!(key.as_str(), "roles" | "providers" | "gods") {
                    if let Some(entries) = value.as_object() {
                        for (name, entry) in entries {
                            write!(output, "\n- {name}:").expect("formatting a String");
                            if entry.is_object() {
                                details(entry, language, output, depth + 1);
                            } else {
                                output.push_str(&scalar(entry, language));
                            }
                        }
                        continue;
                    }
                }
                let Some(label) = label(key, language) else {
                    continue;
                };
                match value {
                    Value::Null => {}
                    Value::Array(_) | Value::Object(_) => {
                        // Print a heading only when something readable sits under it.
                        let mut nested = String::new();
                        details(value, language, &mut nested, depth + 1);
                        if !nested.is_empty() {
                            write!(output, "\n{label}:{nested}").expect("formatting a String");
                        }
                    }
                    _ => {
                        write!(output, "\n{label}: {}", field_value(key, value, language))
                            .expect("formatting a String");
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                if item.is_object() || item.is_array() {
                    details(item, language, output, depth + 1);
                } else {
                    write!(
                        output,
                        "\n- {}",
                        item.as_str()
                            .map_or_else(|| scalar(item, language), str::to_owned)
                    )
                    .expect("formatting a String");
                }
            }
        }
        _ => {}
    }
}

fn field_value(key: &str, value: &Value, language: InterfaceLanguage) -> String {
    if matches!(
        key,
        "text"
            | "content"
            | "body"
            | "excerpt"
            | "snippet"
            | "markdown"
            | "name"
            | "title"
            | "path"
            | "relative_path"
            | "locator"
            | "description"
            | "prompt"
            | "label"
    ) {
        if let Some(text) = value.as_str() {
            return text.into();
        }
    }
    scalar(value, language)
}

fn scalar(value: &Value, language: InterfaceLanguage) -> String {
    match value {
        Value::String(text) => match text.as_str() {
            "success" | "complete" | "completed" => choose(language, "완료", text),
            "disabled" | "inactive" => choose(language, "꺼짐", text),
            "enabled" | "active" => choose(language, "켜짐", text),
            "allow" => choose(language, "허용", text),
            "block" | "deny" => choose(language, "차단", text),
            "running" => choose(language, "실행 중", text),
            "stopped" => choose(language, "종료 확인", text),
            "verified" => choose(language, "확인됨", text),
            "unverified" => choose(language, "확인하지 못함", text),
            "not-configured" | "absent" => choose(language, "설정 없음", text),
            "dry-run" => choose(language, "미리보기, 실제 적용 없음", "preview, not applied"),
            "confirm" => choose(language, "확인 후 적용", "confirmed application"),
            "prepared" => choose(language, "준비됨, 실행하지 않음", "prepared, not executed"),
            _ => text.clone(),
        },
        Value::Bool(true) => choose(language, "예", "yes"),
        Value::Bool(false) => choose(language, "아니요", "no"),
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    }
}

pub(crate) fn diagnostic(message: &str) {
    OPTIONS.with(|options| {
        if options.borrow().json {
            eprintln!("error: {message}");
        }
    });
}

pub(crate) fn error(message: &str) -> std::process::ExitCode {
    crate::emit_action_result(&crate::ActionResult {
        schema_version: 1,
        action: "UnknownAction",
        status: "error",
        exit_code: 2,
        code: "hive.invalid-input",
        message: message.into(),
        changed_paths: vec![],
        evidence: vec![],
        next_action: None,
        data: None,
    })
}

pub(crate) fn guidance(lines: &[String]) {
    OPTIONS.with(|options| {
        if !options.borrow().json {
            for line in lines {
                println!("{line}");
            }
        }
    });
}

pub(crate) fn machine() -> bool {
    OPTIONS.with(|options| options.borrow().json)
}

pub(crate) fn emit_version(text: &str) {
    OPTIONS.with(|options| {
        let display = if options.borrow().language == InterfaceLanguage::Ko {
            text.replace("local developer build", "로컬 개발판")
                .replace("developer test build", "개발 시험판")
                .replace("released ", "출시 ")
                .replace("built ", "만든 날짜 ")
        } else {
            text.to_owned()
        };
        println!("{display}");
    });
}

#[cfg(test)]
mod tests;
