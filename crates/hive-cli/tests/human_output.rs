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
        vec!["discord", "inbound"],
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
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("완료."));
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
            "완료."
        } else {
            "Success."
        }));
        assert!(text.contains(if language == "ko" {
            "크기(바이트)"
        } else {
            "Size (bytes)"
        }));
        assert!(!text.trim().starts_with('{'));
    }
}

#[test]
fn inactive_hook_has_readable_default_and_the_identical_explicit_wire_result() {
    for language in ["ko", "en"] {
        let output = run(&["hook", "--event", "Stop"], language);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(if language == "ko" {
            "실제 동작 여부: 아니요"
        } else {
            "Active: no"
        }));
        assert!(!text.trim().starts_with('{'));
        let output = run(&["hook", "--event", "Stop", "--output", "json"], language);
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            b"{\"schema_version\":1,\"decision\":\"allow\",\"active\":false}\n"
        );
    }
}
