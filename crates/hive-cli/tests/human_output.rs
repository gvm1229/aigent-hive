use std::{fs, process::Command};

fn run(arguments: &[&str], language: &str) -> std::process::Output {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join(".hive/config");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("user-setup.yml"),
        format!("interface_language: {language}\n"),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_hive"))
        .args(arguments)
        .env("USERPROFILE", root.path())
        .env("HOME", root.path())
        .output()
        .unwrap()
}

#[test]
fn every_command_family_defaults_to_human_errors_in_both_languages() {
    for arguments in [
        vec!["usage", "show"],
        vec!["usage", "threshold"],
        vec!["usage", "monitor", "status"],
        vec!["install"],
        vec!["uninstall", "--unknown"],
        vec!["source-wiki", "query"],
        vec!["knowledge", "query"],
        vec!["report", "preview"],
        vec!["project", "upgrade"],
        vec!["session", "check"],
        vec!["index", "rebuild"],
        vec!["route"],
        vec!["prompt", "validate"],
        vec!["policy", "evaluate"],
        vec!["storage", "--unknown"],
        vec!["prune", "--unknown"],
        vec!["summon"],
        vec!["roster"],
        vec!["role", "validate"],
        vec!["agent", "validate"],
        vec!["run", "closure"],
        vec!["loop", "validate"],
        vec!["orchestration", "status"],
        vec!["judge", "package"],
        vec!["korean", "inspect"],
        vec!["release", "verify"],
        vec!["update", "--check"],
    ] {
        for language in ["ko", "en"] {
            let output = run(&arguments, language);
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(!stdout.trim().starts_with('{'), "{arguments:?}: {stdout}");
            assert!(
                !stdout.contains("schema_version"),
                "{arguments:?}: {stdout}"
            );
            assert!(
                !stdout.is_empty(),
                "{arguments:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                if language == "ko" {
                    stdout.contains("다음 단계")
                } else {
                    stdout.contains("Next:")
                },
                "{arguments:?}: {stdout}"
            );
            let mut json_arguments = arguments.clone();
            json_arguments.extend(["--output", "json"]);
            let json_output = run(&json_arguments, language);
            let receipt: serde_json::Value = serde_json::from_slice(&json_output.stdout)
                .unwrap_or_else(|e| {
                    panic!(
                        "{arguments:?}: {e}: {}",
                        String::from_utf8_lossy(&json_output.stdout)
                    )
                });
            assert!(
                !receipt["message"]
                    .as_str()
                    .unwrap_or("")
                    .contains("requires --output json"),
                "{arguments:?}"
            );
        }
    }
}

#[test]
fn doctor_and_target_are_readable_and_keep_their_prior_json_refusal() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().to_str().unwrap();
    for args in [vec!["doctor"], vec!["check-target", target]] {
        let output = run(&args, "ko");
        assert!(output.status.success());
        let text = String::from_utf8_lossy(&output.stdout);
        let first = text.lines().next().unwrap_or("");
        assert!(
            first.ends_with("완료.") && first.matches("완료").count() == 1,
            "{first}"
        );
        let mut args = args;
        args.extend(["--output", "json"]);
        let output = run(&args, "ko");
        assert_eq!(output.status.code(), Some(2));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["action"], "UnknownAction");
        assert_eq!(value["code"], "hive.unknown-action");
    }
}

#[test]
fn successful_storage_results_use_the_same_localized_renderer() {
    for language in ["ko", "en"] {
        let output = run(&["storage"], language);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.starts_with(if language == "ko" {
            "저장 공간 사용량을 확인했습니다. 바꾼 것은 없습니다."
        } else {
            "Success. Hive storage measured. Nothing was changed."
        }));
        assert!(!text.contains("[Hades]"));
        assert!(text.contains(if language == "ko" {
            "전체 크기"
        } else {
            "Total size"
        }));
        assert!(!text.trim().starts_with('{'));
    }
}

#[test]
fn host_protocol_commands_keep_the_wire_result_without_the_output_option() {
    for language in ["ko", "en"] {
        for arguments in [
            vec!["hook", "--event", "Stop"],
            vec!["hook", "--event", "Stop", "--output", "json"],
        ] {
            let output = run(&arguments, language);
            assert!(output.status.success());
            assert_eq!(
                output.stdout,
                b"{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}
"
            );
        }
        let output = run(&["discord", "inbound"], language);
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["action"], "InspectDiscordInbound");
    }
}

#[test]
fn usage_help_lists_the_usage_commands() {
    for arguments in [vec!["usage", "--help"], vec!["usage"]] {
        let output = run(&arguments, "en");
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        for command in [
            "hive usage threshold",
            "hive usage monitor",
            "hive usage status",
        ] {
            assert!(text.contains(command), "{text}");
        }
        assert!(!text.contains("Failed."));
    }
}

#[test]
fn project_setup_description_works_without_the_output_option() {
    for language in ["ko", "en"] {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join(".hive/config");
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("user-setup.yml"),
            format!(
                "schema_version: 1
interface_language: {language}
wiki:
  enabled: true
  language: both
profile:
  contexts:
  - web-developer
persona:
  id: balanced
selected_hosts:
- codex
skills:
  mode: individual
  selected:
  - user-setup
usage_guard:
  enabled: false
  stop_remaining_percent: 20
  codexbar_fallback_enabled: false
"
            ),
        )
        .unwrap();
        let target = root.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_hive"))
            .args([
                "setup",
                "--target",
                target.to_str().unwrap(),
                "--describe",
                "--section",
                "hosts",
            ])
            .env("USERPROFILE", root.path())
            .env("HOME", root.path())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{text}");
        assert!(!text.trim().starts_with('{'), "{text}");
        assert!(!text.contains("diagnostic"), "{text}");
    }
}
