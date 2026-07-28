use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo provides CARGO_MANIFEST_DIR"),
    );
    let workspace_manifest = manifest_dir.join("../../Cargo.toml");
    println!(
        "cargo:rerun-if-changed={}",
        workspace_manifest.to_string_lossy()
    );

    let text = fs::read_to_string(&workspace_manifest)
        .expect("workspace Cargo.toml must be readable for release metadata");
    let manifest: toml::Value =
        toml::from_str(&text).expect("workspace Cargo.toml must contain valid TOML");
    let release_date = manifest
        .get("workspace")
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("hive"))
        .and_then(|value| value.get("release-date"))
        .and_then(toml::Value::as_str)
        .expect("workspace.metadata.hive.release-date must be defined");
    assert!(
        valid_release_date(release_date),
        "workspace.metadata.hive.release-date must use YYYY-MM-DD"
    );
    println!("cargo:rustc-env=HIVE_RELEASE_DATE={release_date}");
}

fn valid_release_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::valid_release_date;

    #[test]
    fn release_date_requires_exact_iso_calendar_shape() {
        assert!(valid_release_date("2026-07-24"));
        for invalid in ["", "2026-7-24", "2026/07/24", "v2026-07-24"] {
            assert!(!valid_release_date(invalid), "{invalid}");
        }
    }
}
