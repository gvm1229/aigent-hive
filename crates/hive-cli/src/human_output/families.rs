/// Plain layouts for the results people read most. `None` falls back to the generic field list.
fn family_details(
    action: &str,
    value: &Value,
    data: &Value,
    language: InterfaceLanguage,
) -> Option<String> {
    let code = value["code"].as_str().unwrap_or("");
    let ko = language == InterfaceLanguage::Ko;
    let mut out = String::new();
    match action {
        "ReportStorage" => storage_details(data, ko, &mut out)?,
        "ManageUsageMonitor" if data.get("controller_alive").is_some() => {
            monitor_details(data, language, &mut out);
        }
        "SetupHarness"
            if code == "hive.project-setup-shown"
                && data["stored"].is_null()
                && data.get("questions").is_none() =>
        {
            out.push_str(&choose(
                language,
                "\n이 프로젝트에 따로 저장한 설정이 없어 전역 설정을 그대로 씁니다.",
                "\nThis project has no settings of its own, so it uses your global settings.",
            ));
        }
        "SetupHarness"
            if code == "hive.project-setup-shown"
                && data["effective"].is_object()
                && data.get("questions").is_none() =>
        {
            effective_settings(&data["effective"], language, &mut out);
        }
        "PreviewDeveloperReport" if data["included"].is_array() => {
            report_details(data, language, &mut out);
        }
        "CheckHiveUpdate"
            if matches!(
                code,
                "hive.update-check-disabled" | "hive.update-check-throttled"
            ) => {}
        "ListKnowledge" | "QueryKnowledge" | "RetrieveKnowledge" | "QuerySourceWiki"
            if nothing_found(data) =>
        {
            out.push_str(&choose(language, "\n찾은 항목: 0개", "\nItems found: 0"));
        }
        "roster" if data.get("roster").is_some() => roster_details(data, language, &mut out),
        "Message" => message_details(data, ko, &mut out),
        _ if data["questions"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item.get("question").is_some())) =>
        {
            question_details(data, language, &mut out);
        }
        _ => return None,
    }
    Some(out)
}

fn label_for(ko: bool, korean: &str, english: &str) -> String {
    if ko { korean } else { english }.to_owned()
}

fn storage_details(data: &Value, ko: bool, out: &mut String) -> Option<()> {
    let total = data["total_bytes"].as_u64()?;
    let _ = write!(out, "\n{}: {}", label_for(ko, "전체 크기", "Total size"), size(total));
    for item in data["categories"].as_array().into_iter().flatten() {
        let name = storage_name(item["category"].as_str().unwrap_or("other"), ko);
        let files = item["files"].as_u64().unwrap_or(0);
        let bytes = size(item["bytes"].as_u64().unwrap_or(0));
        let _ = if ko {
            write!(out, "\n- {name}: {bytes} (파일 {files}개)")
        } else {
            let noun = if files == 1 { "file" } else { "files" };
            write!(out, "\n- {name}: {bytes} ({files} {noun})")
        };
    }
    Some(())
}

fn monitor_details(data: &Value, language: InterfaceLanguage, out: &mut String) {
    let ko = language == InterfaceLanguage::Ko;
    let yes_no = |flag: &Value| scalar(&Value::Bool(flag == true), language);
    let health = match (data["sensor_health"].as_str().unwrap_or("unknown"), ko) {
        ("healthy", true) => "정상",
        ("stopped", true) => "멈춤",
        ("running", true) => "실행 중",
        (_, true) => "알 수 없음",
        (other, false) => other,
    };
    let running = label_for(ko, "감시 프로그램 실행 중", "Monitor running");
    let verified = label_for(ko, "실제 감시 확인", "Monitoring verified");
    let reading = label_for(ko, "사용량 읽기 상태", "Usage reading");
    let _ = write!(out, "\n{running}: {}", yes_no(&data["controller_alive"]));
    let _ = write!(out, "\n{verified}: {}", yes_no(&data["monitoring_verified"]));
    let _ = write!(out, "\n{reading}: {health}");
}

fn effective_settings(settings: &Value, language: InterfaceLanguage, out: &mut String) {
    let ko = language == InterfaceLanguage::Ko;
    let on_off = |flag: &Value| scalar(&Value::String(if flag == true { "enabled" } else { "disabled" }.into()), language);
    let source = match (settings["provenance"].as_str().unwrap_or(""), ko) {
        ("global-inherited", true) => "전역 설정을 그대로 사용",
        ("global-inherited", false) => "inherited from your global settings",
        (_, true) => "이 프로젝트에 저장한 설정",
        (_, false) => "saved for this project",
    };
    let text = |key: &str| settings[key].as_str().unwrap_or("-").to_owned();
    let skills = settings["selected_project_skills"].as_array().map_or(0, Vec::len);
    let counts_used = crate::usage_control::native_usage_direction(
        settings["primary_host"].as_str().unwrap_or(""),
    ) == "used";
    let floor = settings["usage_stop_remaining_percent"].as_u64().unwrap_or(0);
    let stop_at = if counts_used { 100_u64.saturating_sub(floor) } else { floor };
    let rows = [
        (label_for(ko, "설정 출처", "Settings source"), source.to_owned()),
        (label_for(ko, "화면 언어", "Interface language"), text("interface_language")),
        (label_for(ko, "답변 방식", "Answer style"), text("persona_id")),
        (
            label_for(ko, "지식 위키", "Knowledge wiki"),
            format!("{} ({})", on_off(&settings["wiki_enabled"]), text("wiki_language")),
        ),
        (
            label_for(ko, "사용량 보호", "Usage protection"),
            format!(
                "{} ({} {}%)",
                on_off(&settings["usage_guard_enabled"]),
                if counts_used {
                    label_for(ko, "사용한 양 기준", "stop at used")
                } else {
                    label_for(ko, "남은 사용량 기준", "stop below")
                },
                stop_at
            ),
        ),
        (label_for(ko, "선택한 Skill", "Selected Skills"), skills.to_string()),
        (label_for(ko, "설정 방식", "Setup mode"), text("setup_mode")),
    ];
    for (label, value) in rows {
        let _ = write!(out, "\n{label}: {value}");
    }
}

fn report_details(data: &Value, language: InterfaceLanguage, out: &mut String) {
    let ko = language == InterfaceLanguage::Ko;
    let list = |key: &str| {
        data[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|item| report_item(item, ko))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let included = label_for(ko, "포함하는 정보", "Included");
    let excluded = label_for(ko, "포함하지 않는 정보", "Excluded");
    let outbound = label_for(ko, "외부 전송", "Sent anywhere");
    let _ = write!(out, "\n{included}: {}", list("included"));
    let _ = write!(out, "\n{excluded}: {}", list("excluded"));
    let _ = write!(out, "\n{outbound}: {}", scalar(&data["outbound"], language));
}

fn nothing_found(data: &Value) -> bool {
    data["count"].as_u64().map_or_else(
        || {
            ["hits", "items", "pages", "results"]
                .iter()
                .any(|key| data[key].as_array().is_some_and(Vec::is_empty))
        },
        |count| count == 0,
    )
}

/// Messages and results are other sessions' output: always labelled as data, never as steps.
fn message_details(data: &Value, ko: bool, out: &mut String) {
    let list = data["messages"].as_array();
    let messages: Vec<&Value> = list.map_or_else(
        || std::iter::once(&data["message"]).filter(|m| m.is_object()).collect(),
        |items| items.iter().collect(),
    );
    if list.is_some() {
        let count = messages.len();
        out.push_str(&label_for(ko, &format!("\n메시지: {count}개"), &format!("\nMessages: {count}")));
    }
    for message in messages {
        message_line(message, ko, out);
    }
    if let Some(summons) = data["summons"].as_array().filter(|items| !items.is_empty()) {
        let count = summons.len();
        out.push_str(&label_for(
            ko,
            &format!("\n준비된 소환 봉투: {count}개, 자동 실행 없음"),
            &format!("\nPrepared summon envelopes: {count}; nothing runs automatically"),
        ));
        for summon in summons {
            let _ = write!(out, "\n- {} · {}", summon["id"].as_str().unwrap_or("-"),
                summon["state"].as_str().unwrap_or("-"));
        }
    }
    flow_details(data, ko, out);
    stop_details(data, ko, out);
    if let Some(summon) = data.get("summon").filter(|summon| summon.is_object()) {
        let receipts = summon["receipts"].as_array().map_or(0, Vec::len);
        let _ = write!(out, "\n- {} · {} · {}", summon["id"].as_str().unwrap_or("-"),
            summon["state"].as_str().unwrap_or("-"),
            label_for(ko, &format!("확인증 {receipts}개"), &format!("receipts: {receipts}")));
    }
    crate::msg::decide::human_details(data, ko, out);
}

/// Stop results: claimed work is never interrupted; unsupported host cancel is said plainly.
fn stop_details(data: &Value, ko: bool, out: &mut String) {
    let stops: Vec<&Value> = data.get("stop").filter(|stop| stop.is_object()).map_or_else(
        || data["active"].as_array().map_or_else(Vec::new, |items| items.iter().collect()),
        |stop| vec![stop],
    );
    for stop in stops {
        let _ = write!(
            out,
            "\n{} {} · {}",
            label_for(ko, "중지", "Stop"),
            stop["record"]["stop_id"].as_str().unwrap_or("-"),
            label_for(ko, "새 메시지 거부, 강제 취소 없음", "new messages refused, no forced cancel"),
        );
        for entry in stop["in_progress"].as_array().into_iter().flatten() {
            let support = entry["host_cancel"].as_str().unwrap_or("-");
            let support_text = match (support, ko) {
                ("unsupported", true) => "공식 취소 미지원",
                ("unverified", true) => "공식 취소 미검증, 호스트 확인증 필요",
                ("unsupported", false) => "official cancel unsupported",
                ("unverified", false) => "official cancel unverified; host receipt needed",
                _ => support,
            };
            let _ = write!(
                out,
                "\n- {} · {} · {}",
                entry["message_id"].as_str().unwrap_or("-"),
                label_for(ko, "취소 요청", "cancel requested"),
                support_text,
            );
        }
    }
}

/// Flow status lines come from Hive records (`display` is built from the recorded profile and
/// the derived message state), never from a host or worker screen.
fn flow_details(data: &Value, ko: bool, out: &mut String) {
    let key = if ko { "ko" } else { "en" };
    if let Some(flow) = data.get("flow").filter(|flow| flow.is_object()) {
        let _ = write!(
            out,
            "\n{} {} · {}",
            label_for(ko, "흐름", "Flow"),
            flow["flow_id"].as_str().unwrap_or("-"),
            flow["state_label"][key].as_str().unwrap_or("-"),
        );
        for step in flow["steps"].as_array().into_iter().flatten() {
            let _ = write!(out, "\n- {}", step["display"][key].as_str().unwrap_or("-"));
        }
    }
    if let Some(flows) = data["flows"].as_array().filter(|items| !items.is_empty()) {
        let count = flows.len();
        out.push_str(&label_for(
            ko,
            &format!("\n진행 중인 흐름: {count}개"),
            &format!("\nFlows in progress: {count}"),
        ));
        for flow in flows {
            let line = flow["current_step"]["display"][key]
                .as_str()
                .or(flow["state"].as_str())
                .unwrap_or("-");
            let _ = write!(out, "\n- {} · {line}", flow["flow_id"].as_str().unwrap_or("-"));
        }
    }
    if let Some(error) = data.get("flow_error").filter(|error| error.is_object()) {
        let _ = write!(
            out,
            "\n{}: {}",
            label_for(ko, "흐름을 진행하지 못함", "Flow not advanced"),
            error["message"].as_str().unwrap_or("-"),
        );
    }
}

fn message_line(message: &Value, ko: bool, out: &mut String) {
    let state = message["state"].as_str().unwrap_or("-");
    let state_ko = match state {
        "queued" => "대기",
        "claimed" => "소비 중",
        "acknowledged" => "확인증 있음",
        "dispatch-uncertain" => "실행 불명, 자동 재호출 없음",
        "expired" => "만료",
        "quarantined" => "격리, 전달 안 함",
        "cancel-requested" => "취소 요청, Hive가 끊지 않음",
        "cancelled" => "호스트 공식 취소로 종료",
        _ => state,
    };
    let sender = &message["sender"];
    let _ = write!(
        out,
        "\n- {} #{} · {} · {} → {}",
        message["id"].as_str().unwrap_or("-"),
        message["sequence"].as_u64().unwrap_or(0),
        if ko { state_ko } else { state },
        sender["god"].as_str().or(sender["role"].as_str()).unwrap_or("-"),
        message["recipient_role"].as_str().unwrap_or("-"),
    );
    if let (Some(provider), Some(model)) = (sender["provider"].as_str(), sender["model"].as_str()) {
        let _ = write!(out, " ({provider} {model} · {})", sender["effort"].as_str().unwrap_or("-"));
    }
    if let Some(reason) = message["reason"].as_str() {
        let _ = write!(out, "\n  {}: {reason}", label_for(ko, "격리 이유", "Quarantine reason"));
    }
    for (view, heading_ko, heading_en) in [
        (&message["body"], "본문", "Body"),
        (&message["result"]["data"], "결과", "Result"),
    ] {
        let Some(text) = view["text"].as_str() else {
            continue;
        };
        let label = label_for(ko, crate::msg::DATA_LABEL_KO, crate::msg::DATA_LABEL_EN);
        let _ = write!(out, "\n  {} {label}", label_for(ko, heading_ko, heading_en));
        for line in text.lines() {
            // Terminal control sequences from another session never reach the screen.
            let line: String = line
                .chars()
                .map(|c| if c.is_control() && c != '\t' { '\u{fffd}' } else { c })
                .collect();
            let _ = write!(out, "\n  | {line}");
        }
    }
}

fn roster_details(data: &Value, language: InterfaceLanguage, out: &mut String) {
    let ko = language == InterfaceLanguage::Ko;
    let roster = &data["roster"];
    out.push_str(&choose(language, "\n신별 맡은 역할:", "\nRole for each god:"));
    for (god, role) in roster["gods"].as_object().into_iter().flatten() {
        let _ = write!(out, "\n- {god}: {}", role_name(role.as_str().unwrap_or(""), ko));
    }
    out.push_str(&choose(language, "\n앱별 기본 모델:", "\nDefault model per app:"));
    let effort_word = label_for(ko, "작업 수준", "effort");
    for (host, entry) in roster["providers"].as_object().into_iter().flatten() {
        let roles = entry["roles"].as_object().filter(|roles| !roles.is_empty());
        let Some(roles) = roles else {
            let _ = write!(out, "\n- {host}: {}", label_for(ko, "지정 없음", "none set"));
            continue;
        };
        let _ = write!(out, "\n- {host}:");
        for (role, setting) in roles {
            let model = setting["model"].as_str().unwrap_or("-");
            let effort = setting["effort"].as_str().unwrap_or("-");
            let _ = write!(out, "\n  - {}: {model} ({effort_word} {effort})", role_name(role, ko));
        }
    }
    let overridden = roster["roles"]
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(_, setting)| {
            setting
                .as_object()
                .is_some_and(|fields| fields.values().any(|v| !v.is_null()))
        })
        .count();
    let changed = label_for(ko, "이 프로젝트에서 바꾼 역할", "Roles changed in this project");
    let launch = label_for(ko, "작업자 자동 실행", "Automatic worker launch");
    let state = if data["enabled"] == true { "enabled" } else { "disabled" };
    let _ = write!(out, "\n{changed}: {overridden}");
    let _ = write!(out, "\n{launch}: {}", scalar(&Value::String(state.into()), language));
}

fn question_details(data: &Value, language: InterfaceLanguage, out: &mut String) {
    let key = if language == InterfaceLanguage::Ko { "ko" } else { "en" };
    let pick = |text: &Value| text[key].as_str().unwrap_or("").to_owned();
    let plain = |v: &Value| v.as_str().map_or_else(|| scalar(v, language), str::to_owned);
    for item in data["questions"].as_array().into_iter().flatten() {
        let question = &item["question"];
        let _ = write!(out, "\n\n{}", pick(&question["prompt"]));
        let help = pick(&question["help"]);
        if !help.is_empty() {
            let _ = write!(out, "\n({help})");
        }
        let options = question["options"].as_array().into_iter().flatten();
        for (index, option) in options.enumerate() {
            let _ = write!(out, "\n  {}. {}", index + 1, pick(&option["label"]));
        }
        let shown = match &item["current"] {
            Value::Null => continue,
            Value::Array(values) => values.iter().map(plain).collect::<Vec<_>>().join(", "),
            other => plain(other),
        };
        let current = label_for(language == InterfaceLanguage::Ko, "현재 값", "Current");
        let _ = write!(out, "\n{current}: {shown}");
    }
}

fn size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    // Integer tenths keep one decimal without float rounding of large sizes.
    let mut divisor: u128 = 1024;
    let mut unit = 0;
    while u128::from(bytes) >= divisor * 1024 && unit + 1 < UNITS.len() {
        divisor *= 1024;
        unit += 1;
    }
    let tenths = u128::from(bytes) * 10 / divisor;
    format!("{}.{} {}", tenths / 10, tenths % 10, UNITS[unit])
}

fn storage_name(category: &str, ko: bool) -> &'static str {
    match (category, ko) {
        ("vector", true) => "의미 검색 색인",
        ("vector", false) => "Meaning-search index",
        ("index", true) => "검색 색인",
        ("index", false) => "Search index",
        ("knowledge", true) => "저장한 지식",
        ("knowledge", false) => "Saved knowledge",
        ("monitor-binaries", true) => "사용량 감시 프로그램",
        ("monitor-binaries", false) => "Usage monitor programs",
        ("backups", true) => "백업",
        ("backups", false) => "Backups",
        (_, true) => "기타",
        (_, false) => "Other",
    }
}

fn role_name(role: &str, ko: bool) -> String {
    let plain = match (role, ko) {
        ("hive-article-writer", true) => "글쓰기",
        ("hive-article-writer", false) => "writing",
        ("hive-complex-implementer", true) => "복잡한 구현",
        ("hive-complex-implementer", false) => "complex implementation",
        ("hive-design-specialist", true) => "설계",
        ("hive-design-specialist", false) => "design",
        ("hive-independent-judge", true) => "독립 검토",
        ("hive-independent-judge", false) => "independent review",
        ("hive-research-specialist", true) => "조사",
        ("hive-research-specialist", false) => "research",
        ("hive-routine-implementer", true) => "일상 구현",
        ("hive-routine-implementer", false) => "routine implementation",
        _ => return role.to_owned(),
    };
    plain.to_owned()
}

fn report_item(item: &str, ko: bool) -> String {
    let plain = match (item, ko) {
        ("product_version", true) => "Hive 버전",
        ("product_version", false) => "Hive version",
        ("target_digest", true) => "프로젝트 폴더 식별값(내용 제외)",
        ("target_digest", false) => "project folder fingerprint (no contents)",
        ("issue_code", true) => "문제 코드",
        ("issue_code", false) => "problem code",
        ("manual_summary", true) => "직접 쓴 설명",
        ("manual_summary", false) => "your own summary",
        ("operating_system", true) => "운영체제",
        ("operating_system", false) => "operating system",
        ("architecture", true) => "컴퓨터 종류",
        ("architecture", false) => "processor type",
        ("raw_prompt", true) => "보낸 요청 원문",
        ("raw_prompt", false) => "your original requests",
        ("transcript", true) => "대화 기록",
        ("transcript", false) => "conversation transcript",
        ("credential", true) => "로그인·비밀 정보",
        ("credential", false) => "logins and secrets",
        ("automatic_upload", true) => "자동 업로드",
        ("automatic_upload", false) => "automatic upload",
        _ => return item.to_owned(),
    };
    plain.to_owned()
}

#[cfg(test)]
mod settings_direction_tests {
    use super::{effective_settings, InterfaceLanguage};
    use serde_json::json;

    #[test]
    fn settings_summary_follows_each_host_screen() {
        for (host, expected) in [
            ("claude", "사용한 양 기준 96%"),
            ("codex", "남은 사용량 기준 4%"),
            ("antigravity", "남은 사용량 기준 4%"),
        ] {
            let mut out = String::new();
            effective_settings(
                &json!({"primary_host":host,"usage_stop_remaining_percent":4}),
                InterfaceLanguage::Ko,
                &mut out,
            );
            assert!(out.contains(expected), "{host}: {out}");
        }
    }
}
