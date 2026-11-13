use std::process::Command;

#[test]
fn version_aliases_report_product_version_and_release_date() {
    let product = env!("CARGO_PKG_VERSION");
    let package = env!("HIVE_PACKAGE_VERSION");
    let release_date = env!("HIVE_PACKAGE_RELEASE_DATE");
    let expected = if package == product {
        format!("AIgent Hive v{product} (released {release_date})\n")
    } else if package == format!("{product}-dev") {
        format!("AIgent Hive v{product}-dev · local developer build (built {release_date})\n")
    } else {
        format!(
            "AIgent Hive v{}-test{} · developer test build (released {})\n",
            product,
            package
                .strip_prefix(&format!("{product}-test"))
                .expect("validated test package version")
                .replace('.', " #"),
            release_date
        )
    };
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join(".hive/config");
    std::fs::create_dir_all(&config).unwrap();
    for language in ["en", "ko"] {
        std::fs::write(
            config.join("user-setup.yml"),
            format!("interface_language: {language}\n"),
        )
        .unwrap();
        let expected_language = if language == "ko" {
            expected
                .replace("local developer build", "로컬 개발판")
                .replace("developer test build", "개발 시험판")
                .replace("released ", "출시 ")
                .replace("built ", "만든 날짜 ")
        } else {
            expected.clone()
        };
        for argument in ["--version", "-v", "-V"] {
            let output = Command::new(env!("CARGO_BIN_EXE_hive"))
                .arg(argument)
                .env("USERPROFILE", root.path())
                .env("HOME", root.path())
                .output()
                .expect("hive version command should execute");
            assert!(
                output.status.success(),
                "{argument} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8(output.stdout).expect("version output should be UTF-8"),
                expected_language,
                "{argument} output"
            );
            assert!(output.stderr.is_empty(), "{argument} wrote to stderr");
        }
    }
}
