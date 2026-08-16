use cap_std::ambient_authority;
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use hive_render::historical_project_upgrade_candidate_in;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir_in, TempDir};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn secure_tempdir() -> TempDir {
    let root = fs::canonicalize(std::env::temp_dir()).expect("canonical temporary root");
    tempdir_in(root).expect("temporary consumer")
}

fn run_hive(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hive"))
        .args(arguments)
        .output()
        .expect("run Hive binary")
}

fn run_hive_with_environment(arguments: &[&str], key: &str, value: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hive"))
        .args(arguments)
        .env(key, value)
        .output()
        .expect("run Hive binary")
}

fn require_success(output: &std::process::Output, action: &str) {
    assert!(
        output.status.success(),
        "{action} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).expect("JSON action result");
    assert_eq!(result["status"], "success", "{action}");
}

fn seed_historical_project(target: &Path, version: &str) {
    let repository = root();
    let answers = repository.join("tests/fixtures/phase1/answers-base.yml");
    let capabilities = repository.join("tests/fixtures/phase1/capabilities-codex-omx.json");
    let user_root = target.parent().expect("consumer parent").join("user-root");
    let user_config = user_root.join(".hive/config");
    fs::create_dir_all(&user_config).expect("user root");
    fs::write(
        user_config.join("user-setup.yml"),
        "schema_version: 1\ninterface_language: ko\nwiki:\n  enabled: true\n  language: both\nprofile:\n  contexts:\n  - web-developer\npersona:\n  id: balanced\nselected_hosts:\n- codex\nskills:\n  mode: individual\n  selected:\n  - user-setup\nusage_guard:\n  enabled: false\n  stop_remaining_percent: 20\n  codexbar_fallback_enabled: false\n",
    )
    .expect("global setup config");
    require_success(
        &run_hive(&[
            "setup",
            "--target",
            target.to_str().expect("target UTF-8"),
            "--answers",
            answers.to_str().expect("answers UTF-8"),
            "--capabilities",
            capabilities.to_str().expect("capabilities UTF-8"),
            "--user-root",
            user_root.to_str().expect("user root UTF-8"),
            "--apply",
            "--output",
            "json",
        ]),
        "setup",
    );

    let harness = target.join(".hive/config/harness.toml");
    let historical_harness = fs::read_to_string(&harness)
        .expect("harness config")
        .replace(
            &format!("harness_version = \"{}\"", env!("CARGO_PKG_VERSION")),
            &format!("harness_version = \"{version}\""),
        )
        .replace(
            &format!("source_release_version = \"{}\"", env!("CARGO_PKG_VERSION")),
            &format!("source_release_version = \"{version}\""),
        );
    fs::write(&harness, historical_harness).expect("historical harness config");

    let capability = Dir::open_ambient_dir(target, ambient_authority()).expect("target capability");
    let historical = historical_project_upgrade_candidate_in(&capability, version)
        .expect("embedded historical project base");
    for file in &historical.files {
        let path = target.join(&file.path);
        fs::create_dir_all(path.parent().expect("historical parent"))
            .expect("historical directory");
        fs::write(path, &file.content).expect("historical project byte");
    }
    let files = historical
        .files
        .iter()
        .map(|file| {
            json!({
                "path": file.path,
                "kind": file.kind,
                "content_digest": file.content_digest,
                "content": String::from_utf8_lossy(&file.content),
            })
        })
        .collect::<Vec<_>>();
    let mut ledger = json!({
        "schema_version": 1,
        "product_version": version,
        "files": files,
    });
    let digest = sha256_digest(
        &serde_json_canonicalizer::to_vec(&ledger).expect("canonical unsigned ledger"),
    );
    ledger
        .as_object_mut()
        .expect("ledger object")
        .insert("ledger_digest".to_owned(), Value::String(digest));
    let mut bytes = serde_json_canonicalizer::to_vec(&ledger).expect("canonical ledger");
    bytes.push(b'\n');
    fs::write(target.join(".hive/config/project-base.json"), bytes).expect("historical ledger");
}

#[test]
fn compiled_cli_upgrades_each_full_historical_project_and_preserves_local_and_foreign_bytes() {
    for version in ["0.9.1", "0.9.2", "0.9.3"] {
        let temporary = secure_tempdir();
        let target = temporary.path().join("consumer");
        fs::create_dir_all(&target).expect("consumer directory");
        seed_historical_project(&target, version);

        let local_skill = target.join("AGENTS.md");
        let local_suffix = b"\n<!-- local preference -->\n";
        let mut local_bytes = fs::read(&local_skill).expect("historical local skill");
        local_bytes.extend_from_slice(local_suffix);
        fs::write(&local_skill, &local_bytes).expect("local projection edit");
        let foreign = target.join("FOREIGN.md");
        let foreign_bytes = b"foreign bytes must remain exact\r\n";
        fs::write(&foreign, foreign_bytes).expect("foreign bytes");

        for mode in ["--scan", "--dry-run"] {
            require_success(
                &run_hive(&[
                    "project",
                    "upgrade",
                    "--target",
                    target.to_str().expect("target UTF-8"),
                    mode,
                    "--output",
                    "json",
                ]),
                mode,
            );
        }
        let failed_apply = run_hive_with_environment(
            &[
                "project",
                "upgrade",
                "--target",
                target.to_str().expect("target UTF-8"),
                "--apply",
                "--output",
                "json",
            ],
            "HIVE_PROJECT_UPGRADE_FAIL_AFTER",
            "1",
        );
        assert!(!failed_apply.status.success());
        assert!(fs::read(&local_skill)
            .expect("local skill after rollback")
            .ends_with(local_suffix));
        assert_eq!(
            fs::read(&foreign).expect("foreign after rollback"),
            foreign_bytes
        );
        for mode in ["--apply", "--validate"] {
            require_success(
                &run_hive(&[
                    "project",
                    "upgrade",
                    "--target",
                    target.to_str().expect("target UTF-8"),
                    mode,
                    "--output",
                    "json",
                ]),
                mode,
            );
        }
        assert!(fs::read(&local_skill)
            .expect("updated local skill")
            .ends_with(local_suffix));
        assert_eq!(
            fs::read(&foreign).expect("foreign after upgrade"),
            foreign_bytes
        );
    }
}

#[test]
fn compiled_cli_rejects_a_tampered_092_base_without_mutation() {
    let temporary = secure_tempdir();
    let target = temporary.path().join("consumer");
    fs::create_dir_all(&target).expect("consumer directory");
    seed_historical_project(&target, "0.9.2");
    let ledger = target.join(".hive/config/project-base.json");
    let mut value: Value =
        serde_json::from_slice(&fs::read(&ledger).expect("ledger bytes")).expect("ledger JSON");
    value["files"][0]["content"] = Value::String("tampered\n".to_owned());
    fs::write(
        &ledger,
        serde_json_canonicalizer::to_vec(&value).expect("tampered canonical ledger"),
    )
    .expect("tampered ledger");
    let before = fs::read(&ledger).expect("before ledger");

    let output = run_hive(&[
        "project",
        "upgrade",
        "--target",
        target.to_str().expect("target UTF-8"),
        "--scan",
        "--output",
        "json",
    ]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&ledger).expect("ledger after failure"), before);
}
