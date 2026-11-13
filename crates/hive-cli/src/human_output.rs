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
        // Native hooks opt into their host's protocol with --stdin-json.
        || (arguments.first().is_some_and(|arg| arg == "policy")
            && arguments.get(1).is_some_and(|arg| arg == "hook")
            && arguments.iter().any(|arg| arg == "--stdin-json"));
    OPTIONS.with(|options| {
        *options.borrow_mut() = Options {
            json,
            language: crate::user_setup::output_language(arguments),
            family: arguments.first().cloned().unwrap_or_default(),
        }
    });
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
        return failure(value, language, &ko, &en);
    }
    if action == "SetUsageThreshold" {
        if let Some(text) = threshold_message(data, language) {
            return text;
        }
    }
    let mut output = if language == InterfaceLanguage::Ko {
        // Setup already supplies complete localized messages.
        if message.chars().any(|c| ('가'..='힣').contains(&c)) {
            format!("완료. {message}")
        } else {
            format!("완료. {ko}.")
        }
    } else if message.is_empty() {
        format!("Success. {en}.")
    } else {
        format!("Success. {}", sentence(message))
    };
    if !value["changed_paths"].as_array().is_none_or(Vec::is_empty) {
        let preview = value["code"]
            .as_str()
            .is_some_and(|code| code.contains("preview") || code.contains("dry-run"))
            || data["mode"] == "dry-run";
        output.push_str(&if preview {
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
    details(data, language, &mut output, 0);
    if let Some(next) = value["next_action"]
        .as_str()
        .filter(|next| !next.is_empty())
    {
        output.push_str(&choose(language, "\n다음 단계: ", "\nNext: "));
        if language == InterfaceLanguage::Ko {
            output.push_str(&next_ko(next));
        } else {
            output.push_str(&next.replace(" --output json", ""));
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
            "\n전역과 프로젝트 기준 중 더 높은 값으로 보호합니다.",
            "\nProtection uses the higher of the global and project thresholds.",
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

fn failure(value: &Value, language: InterfaceLanguage, ko: &str, en: &str) -> String {
    let message = value["message"]
        .as_str()
        .unwrap_or("The request could not be completed.");
    let message = message.lines().next().unwrap_or(message);
    if language == InterfaceLanguage::En {
        let remedy = value["next_action"].as_str().unwrap_or(
            "Check the command options with --help. Correct the named input or file, then retry.",
        );
        return format!(
            "Could not complete: {en}. {}\nNext: {remedy}",
            sentence(message)
        );
    }
    let cause = korean_cause(message);
    let mut output = format!(
        "처리 실패: {}. {cause}",
        ko.trim_end_matches(" 처리 완료").trim_end_matches(" 완료")
    );
    output.push_str("\n다음 단계: ");
    output.push_str(&value["next_action"].as_str().map_or_else(
        || "--help로 입력 방법을 확인하세요. 자세한 원인은 같은 명령의 --output json에서 확인할 수 있습니다.".into(), next_ko));
    output
}

fn korean_cause(message: &str) -> String {
    if message.chars().any(|c| ('가'..='힣').contains(&c)) {
        return message.into();
    }
    let lower = message.to_ascii_lowercase();
    let plain = if (lower.contains("source workspace") || lower.contains("source root"))
        && lower.contains("consumer")
    {
        Some("Hive 자체 코드를 고치는 폴더에서는 사용자·프로젝트 설정을 적용할 수 없습니다. Hive를 사용할 다른 프로젝트 폴더를 대상으로 골라 주세요.")
    } else if lower.contains("symlink")
        || lower.contains("reparse")
        || lower.contains("symbolic link")
    {
        Some("다른 위치로 연결된 경로라서 안전하게 처리할 수 없습니다. 연결 경로 대신 실제 폴더의 전체 경로를 선택하세요.")
    } else if lower.contains("orchestration") && lower.contains("disabled") {
        Some("이번 버전의 작업자 자동 실행은 꺼져 있으며 켜는 경로가 없습니다. 작업 내용의 미리보기와 전달 준비만 사용할 수 있습니다.")
    } else if lower.contains("--summary") && lower.contains("preview") {
        Some("변경 승인용 미리보기와 질문 설명에는 요약 옵션을 함께 쓸 수 없습니다. --summary를 빼고 확인하세요.")
    } else if lower.contains("output") && lower.contains("json") {
        Some("직접 지정할 수 있는 기계 출력 형식은 JSON입니다. 사람이 읽는 결과는 --output 옵션을 빼고 실행하세요.")
    } else if lower.contains("usable terminal") {
        Some("대화형 실행에는 직접 입력할 수 있는 터미널이 필요합니다. 터미널에서 다시 실행하거나 --interactive를 빼고 필요한 입력을 지정하세요.")
    } else if lower.contains("software update approval") {
        Some("프로그램 업데이트 설치를 승인하지 않아 설치하지 않았습니다. 설치를 원할 때 터미널에서 내용을 확인하고 승인하세요.")
    } else if lower.contains("integer") && lower.contains("99") {
        Some(
            "사용량 기준은 0부터 99까지의 정수여야 합니다. 소수점이나 범위를 벗어난 값을 고치세요.",
        )
    } else if lower.contains("credential") || lower.contains("secret") {
        Some("비밀 정보가 포함되어 이 작업을 진행할 수 없습니다. 비밀 값을 제외한 자료만 선택하세요.")
    } else if lower.contains("signature") || lower.contains("digest") || lower.contains("checksum")
    {
        Some("현재 파일과 승인한 내용이 같은지 확인하지 못했습니다. 신뢰할 수 있는 원본과 변경 미리보기를 확인한 뒤 다시 승인하세요.")
    } else if lower.contains("manifest") || lower.contains("ownership") {
        Some("설치 기록과 파일의 소유를 확인하지 못했습니다. 원래 파일을 보존하고 설치 기록과 변경 미리보기를 먼저 확인하세요.")
    } else if lower.contains("absolute") {
        Some("대상 폴더의 전체 경로가 필요합니다. 현재 폴더에서 시작하는 짧은 경로 대신 전체 경로를 지정하세요.")
    } else if lower.contains("schema") {
        Some("입력 파일의 형식이나 버전을 확인하지 못했습니다. 해당 명령의 --help에서 필요한 파일 형식을 확인하세요.")
    } else if lower.contains("conflict") && message.contains("--") {
        Some("함께 사용할 수 없는 옵션을 선택했습니다. --help에서 한 가지 실행 방식만 골라 다시 실행하세요.")
    } else {
        None
    };
    if let Some(plain) = plain {
        return plain.into();
    }
    if message.contains("duplicate") {
        "같은 입력을 두 번 지정했습니다. 중복된 옵션을 지우세요.".into()
    } else if message.contains("missing")
        || message.contains("requires")
        || message.contains("required")
    {
        let options: Vec<_> = message
            .split_whitespace()
            .filter(|word| word.starts_with("--"))
            .collect();
        format!(
            "필요한 입력이나 파일이 없습니다. {} 값을 확인하세요.",
            options.join(" ")
        )
    } else if message.contains("unknown") || message.contains("unsupported") {
        "사용할 수 없는 명령이나 옵션입니다. --help에서 사용할 이름을 확인하세요.".into()
    } else if message.contains("conflict")
        || message.contains("changed")
        || message.contains("stale")
    {
        "파일이나 승인 정보가 이전 확인 때와 다릅니다. 현재 상태를 살펴본 뒤 미리보기를 다시 확인하세요.".into()
    } else if message.contains("permission") || message.contains("denied") {
        "파일이나 작업에 접근할 수 없습니다. 대상 경로와 접근 권한을 확인하세요.".into()
    } else if message.contains("usage") || message.contains("quota") {
        "사용량을 확인하지 못했거나 보호 기준에 도달했습니다. 사용량과 다음 초기화 시간을 확인하세요.".into()
    } else {
        "입력이나 현재 상태를 확인하지 못했습니다. 대상 파일과 저장된 설정을 확인하세요.".into()
    }
}

fn next_ko(next: &str) -> String {
    let readable = next.replace(" --output json", "");
    let next = readable.as_str();
    if next.starts_with("hive ") {
        return format!("다음 명령으로 이어가세요: {next}");
    }
    if next.chars().any(|c| ('가'..='힣').contains(&c)) {
        return next.into();
    }
    let commands: Vec<_> = next
        .split('`')
        .enumerate()
        .filter_map(|(i, text)| (i % 2 == 1 && text.starts_with("hive ")).then_some(text))
        .collect();
    if !commands.is_empty() {
        return format!("다음 명령으로 상태를 확인하세요: {}", commands.join("; "));
    }
    if next.contains("approv") || next.contains("consent") {
        "미리보기의 대상과 변경 내용을 확인하고 승인한 뒤 다시 실행하세요.".into()
    } else if next.contains("session") || next.contains("reservation") {
        "사용 중인 세션과 파일 예약을 확인하고 충돌을 해결한 뒤 다시 실행하세요.".into()
    } else if next.contains("repair") || next.contains("fix") {
        "표시한 문제를 고친 뒤 같은 검사를 다시 실행하세요.".into()
    } else {
        "현재 상태와 필요한 입력을 확인하세요. 자세한 다음 행동은 같은 명령의 --output json에서 확인할 수 있습니다.".into()
    }
}

fn label(key: &str, language: InterfaceLanguage) -> Option<String> {
    let (ko, en) = match key {
        "enabled" => ("사용 여부", "Enabled"),
        "active" => ("실제 동작 여부", "Active"),
        "guard_enabled" | "enforcement_enabled" => ("일반 사용량 보호", "General usage protection"),
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
                        if !value.as_array().is_some_and(Vec::is_empty) {
                            write!(output, "\n{label}:").expect("formatting a String");
                            details(value, language, output, depth + 1);
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
