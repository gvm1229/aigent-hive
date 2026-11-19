//! Typo suggestions, missing-option examples and help through the real binary.
use std::{fs, process::Command};

fn run(arguments: &[&str], language: &str) -> std::process::Output {
    let root = tempfile::tempdir_in(fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let config = root.path().join(".hive/config");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("user-setup.yml"),
        format!("interface_language: {language}\n"),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_hive"))
        .args(arguments)
        .current_dir(root.path())
        .env("USERPROFILE", root.path())
        .env("HOME", root.path())
        .output()
        .unwrap()
}

fn text(arguments: &[&str], language: &str) -> (Option<i32>, String) {
    let output = run(arguments, language);
    (
        output.status.code(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

fn json(arguments: &[&str]) -> (Option<i32>, serde_json::Value) {
    let mut arguments = arguments.to_vec();
    arguments.extend(["--output", "json"]);
    let output = run(&arguments, "en");
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{arguments:?}: {error}: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    (output.status.code(), value)
}

#[test]
fn a_mistyped_command_gets_a_suggestion_example_and_help() {
    let (code, ko) = text(&["instal"], "ko");
    assert_eq!(code, Some(2));
    assert_eq!(
        ko,
        "처리 실패: 알 수 없는 명령입니다: instal\n혹시 이것인가요? install\n\
         예시: hive install --scope user --hosts codex,claude --dry-run\n\
         다음 단계: hive install --help 로 옵션과 예시를 확인하세요.\n"
    );
    let (_, en) = text(&["instal"], "en");
    assert!(en.contains("Did you mean? install\n"), "{en}");
    assert!(en.contains("Example: hive install --scope user"), "{en}");
}

#[test]
fn mistyped_subcommands_and_options_get_real_candidates() {
    let (code, ko) = text(&["knowledge", "retreive"], "ko");
    assert_eq!(code, Some(2));
    assert!(ko.contains("혹시 이것인가요? retrieve\n"), "{ko}");
    assert!(
        ko.contains("예시: hive knowledge retrieve --user-root"),
        "{ko}"
    );
    let (_, en) = text(&["install", "--dryrun"], "en");
    assert!(en.contains("Did you mean? --dry-run\n"), "{en}");
    assert!(en.contains("Next: see hive install --help"), "{en}");
}

#[test]
fn nothing_close_still_points_to_help_without_a_guess() {
    let (code, ko) = text(&["install", "--flags"], "ko");
    assert_eq!(code, Some(2));
    assert!(!ko.contains("혹시"), "{ko}");
    assert!(
        ko.contains("예시: hive install --scope user --hosts codex,claude --dry-run"),
        "{ko}"
    );
    assert!(ko.contains("hive install --help"), "{ko}");
    let (_, en) = text(&["xyzzy"], "en");
    assert!(!en.contains("Did you mean"), "{en}");
    assert!(en.contains("Next: see hive --help"), "{en}");
}

#[test]
fn json_keeps_the_error_envelope_and_adds_suggestions() {
    let (code, value) = json(&["instal"]);
    assert_eq!(code, Some(2));
    assert_eq!(value["action"], "UnknownAction");
    assert_eq!(value["code"], "hive.unknown-action");
    assert_eq!(value["message"], "unknown top-level action: instal");
    assert_eq!(value["data"]["suggestions"], serde_json::json!(["install"]));
    assert_eq!(
        value["next_action"],
        "hive install --scope user --hosts codex,claude --dry-run"
    );
    let (_, option) = json(&["install", "--dryrun"]);
    assert_eq!(option["code"], "hive.invalid-input");
    assert_eq!(option["message"], "unknown option: --dryrun");
    assert_eq!(
        option["data"]["suggestions"],
        serde_json::json!(["--dry-run"])
    );
    assert_eq!(option["data"]["help_command"], "hive install --help");
    let (_, none) = json(&["xyzzy"]);
    assert_eq!(none["data"]["suggestions"], serde_json::json!([]));
    assert!(none["next_action"].is_null());
}

#[test]
fn a_missing_required_option_names_it_with_a_complete_example() {
    let (_, ko) = text(&["install"], "ko");
    assert!(ko.contains("필요한 옵션 --scope"), "{ko}");
    assert!(
        ko.contains("예시: hive install --scope user --hosts codex,claude --dry-run"),
        "{ko}"
    );
    assert!(ko.contains("hive install --help"), "{ko}");
    let (_, en) = text(&["project", "upgrade"], "en");
    assert!(
        en.contains("The required option --target is missing."),
        "{en}"
    );
    assert!(
        en.contains("Example: hive project upgrade --target . --scan"),
        "{en}"
    );
    assert!(en.contains("hive project upgrade --help"), "{en}");
}

#[test]
fn help_is_short_grouped_and_fits_eighty_columns() {
    for language in ["ko", "en"] {
        for arguments in [
            vec!["--help"],
            vec!["install", "--help"],
            vec!["setup", "--help"],
            vec!["update", "--help"],
            vec!["knowledge", "--help"],
            vec!["usage", "--help"],
            vec!["project", "upgrade", "--help"],
            vec!["project", "--help"],
        ] {
            let (code, output) = text(&arguments, language);
            assert_eq!(code, Some(0), "{arguments:?}");
            for line in output.lines() {
                let width: usize = line
                    .chars()
                    .map(|c| if ('가'..='힣').contains(&c) { 2 } else { 1 })
                    .sum();
                assert!(width <= 80, "{arguments:?}: {line}");
            }
        }
        let (_, top) = text(&["--help"], language);
        let groups = if language == "ko" {
            ["시작하기", "설정", "지식", "갱신", "진단", "고급"]
        } else {
            [
                "Getting started",
                "Configuration",
                "Knowledge",
                "Updates",
                "Diagnostics",
                "Advanced",
            ]
        };
        for group in groups {
            assert!(top.lines().any(|line| line == group), "{group}: {top}");
        }
        assert!(top.lines().count() < 65, "{top}");
    }
}

#[test]
fn doctor_and_check_target_support_json_like_their_help_says() {
    let (code, value) = json(&["doctor"]);
    assert_eq!(code, Some(0));
    assert_eq!(value["action"], "InspectInstallation");
    assert_eq!(value["data"]["version"], env!("CARGO_PKG_VERSION"));
    let (code, value) = json(&["check-target", "."]);
    assert_eq!(code, Some(0));
    assert_eq!(value["action"], "CheckTarget");
}
