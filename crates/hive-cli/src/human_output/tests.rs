use super::*;
use serde_json::json;

#[test]
fn bilingual_action_family_snapshots() {
    let mut snapshot = String::new();
    for (action, _, _) in ACTIONS {
        let value = json!({"action":action,"status":"success","exit_code":0});
        write!(
            snapshot,
            "{action}\nko: {}\nen: {}\n\n",
            render(&value, InterfaceLanguage::Ko, action),
            render(&value, InterfaceLanguage::En, action)
        )
        .unwrap();
    }
    snapshot.pop();
    assert_eq!(snapshot, include_str!("actions.snap"));
}

#[test]
fn threshold_shows_before_after_direction_and_recheck_without_changing_receipt() {
    for (before, after, verb) in [(30, 10, "lowered"), (10, 40, "raised"), (10, 10, "already")] {
        let result = crate::ActionResult {
            schema_version: 1,
            action: "SetUsageThreshold",
            status: "success",
            exit_code: 0,
            code: "hive.usage-global-threshold-updated",
            message: "global usage threshold was updated atomically".into(),
            changed_paths: vec![".hive/config/user-setup.yml".into()],
            evidence: vec![],
            next_action: None,
            data: Some(
                json!({"previous_remaining_percent":before,"threshold_remaining_percent":after,
                "session_recheck_required":before!=after}),
            ),
        };
        let original = serde_json::to_string(&result).unwrap();
        let value = serde_json::to_value(&result).unwrap();
        let en = render(&value, InterfaceLanguage::En, "usage");
        let ko = render(&value, InterfaceLanguage::Ko, "usage");
        assert!(en.contains(verb));
        assert!(ko.contains(&format!("{after}%")));
        if before == after {
            assert!(!en.contains("Next:"));
        } else {
            assert!(en.contains(&format!("from {before}% to {after}%")));
            assert!(ko.contains(&format!("{before}%에서 {after}%")));
            assert!(en.contains("Next:"));
        }
        assert_eq!(serde_json::to_string(&result).unwrap(), original);
        assert!(!ko.contains("schema_version"));
    }
}

#[test]
fn errors_explain_missing_input_and_how_to_recover_in_both_languages() {
    let value = json!({"action":"QueryKnowledge","status":"error","exit_code":2,
        "message":"knowledge query requires --target","code":"hive.invalid-input"});
    assert_eq!(render(&value, InterfaceLanguage::Ko, "knowledge"),
        "처리 실패: 지식 검색. 필요한 입력이나 파일이 없습니다. --target 값을 확인하세요.\n다음 단계: --help로 입력 방법을 확인하세요. 자세한 원인은 같은 명령의 --output json에서 확인할 수 있습니다.");
    let english = render(&value, InterfaceLanguage::En, "knowledge");
    assert!(english.contains("requires --target"));
    assert!(english.contains("Next:"));
    let unknown = json!({"action":"UnknownAction","status":"error","exit_code":2,
        "message":"unknown command: example\n\nAigent Hive\nUSAGE: many commands"});
    let output = render(&unknown, InterfaceLanguage::En, "example");
    assert!(!output.contains("USAGE:"));
    assert!(
        output.contains("unknown command: example") || output.contains("Unknown command: example")
    );
}

#[test]
fn human_search_and_preview_show_content_and_preserve_safety_state() {
    let value = json!({"action":"QueryKnowledge","status":"success","exit_code":0,"next_action":"hive knowledge read --target . --page-id example --output json",
        "data":{"count":1,"hits":[{"title":"A useful fact","path":"facts/a.md","snippet":"Keep the original file."}],
        "enabled":false,"state":"prepared","internal_digest":"sha256:private"}});
    for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
        let output = render(&value, language, "knowledge");
        assert!(output.contains("A useful fact"));
        assert!(output.contains("facts/a.md"));
        assert!(output.contains("Keep the original file."));
        assert!(!output.contains("private"));
        assert!(!output.contains(" --output json"));
        assert!(!output.contains('{'));
    }
}

#[test]
fn question_descriptions_show_the_selected_language_and_choices() {
    let value = json!({"action":"DescribeHiveSetupQuestion","status":"success","exit_code":0,
        "data":{"questions":[{"question":{"prompt":{"en":"Choose your language", "ko":"사용할 언어 선택"},
            "options":[{"label":{"en":"Korean", "ko":"한국어"}},{"label":{"en":"English", "ko":"영어"}}]}}]}});
    let korean = render(&value, InterfaceLanguage::Ko, "setup");
    let english = render(&value, InterfaceLanguage::En, "setup");
    assert!(korean.contains("사용할 언어 선택"));
    assert!(korean.contains("한국어"));
    assert!(!korean.contains("Choose your language"));
    assert!(english.contains("Choose your language"));
    assert!(english.contains("Korean"));
    assert!(!english.contains("사용할 언어 선택"));
}

#[test]
fn storage_preview_and_session_changes_report_outcomes_without_rewriting_user_text() {
    let preview = json!({"action":"PruneStorage","status":"success","exit_code":0,
        "data":{"mode":"dry-run","report":{"candidates":[{"path":"active","bytes":12}],"removed":[],"skipped":[]}}});
    let output = render(&preview, InterfaceLanguage::Ko, "prune");
    assert!(output.contains("삭제 예정 파일"));
    assert!(output.contains("위치: active"));
    assert!(output.contains("--confirm"));
    let session = json!({"action":"ControlUsageSession","status":"success","exit_code":0,
        "data":{"guard_enabled":false,"quota_reset_guard_enabled":true,"session_recheck_required":true}});
    let output = render(&session, InterfaceLanguage::Ko, "usage");
    assert!(output.contains("일반 사용량 보호: 아니요"));
    assert!(output.contains("사용량 초기화 보호: 예"));
    assert!(output.contains("다음 단계:"));
    let page = json!({"action":"ReadKnowledge","status":"success","exit_code":0,"data":{"body":"prepared"}});
    assert!(render(&page, InterfaceLanguage::Ko, "knowledge").contains("내용: prepared"));
}
