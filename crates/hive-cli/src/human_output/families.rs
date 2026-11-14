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
