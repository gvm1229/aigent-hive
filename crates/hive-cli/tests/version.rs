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
