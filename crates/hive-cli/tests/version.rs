use std::process::Command;

#[test]
fn version_aliases_report_product_version_and_release_date() {
    let expected = format!(
        "hive {} (released {})\n",
        env!("CARGO_PKG_VERSION"),
        env!("HIVE_RELEASE_DATE")
    );
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
