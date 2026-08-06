use std::process::Command;

#[test]
fn version_aliases_report_product_version_and_release_date() {
    let expected = if env!("HIVE_PACKAGE_VERSION") == env!("CARGO_PKG_VERSION") {
        format!(
            "AIgent Hive v{} (released {})\n",
            env!("CARGO_PKG_VERSION"),
            env!("HIVE_PACKAGE_RELEASE_DATE")
        )
    } else {
        format!(
            "AIgent Hive v{}-test{} · developer test build (released {})\n",
            env!("CARGO_PKG_VERSION"),
            env!("HIVE_PACKAGE_VERSION")
                .strip_prefix(&format!("{}-test", env!("CARGO_PKG_VERSION")))
                .expect("validated test package version")
                .replace('.', " #"),
            env!("HIVE_PACKAGE_RELEASE_DATE")
        )
    };
    for argument in ["--version", "-v", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_hive"))
            .arg(argument)
            .output()
            .expect("hive version command should execute");
        assert!(
            output.status.success(),
            "{argument} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).expect("version output should be UTF-8"),
            expected,
            "{argument} output"
        );
        assert!(output.stderr.is_empty(), "{argument} wrote to stderr");
    }
}
