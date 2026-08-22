use sha2::{Digest, Sha256};
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
    let product_version = env::var("CARGO_PKG_VERSION")
        .expect("Cargo provides CARGO_PKG_VERSION for the Hive product version");
    let package_version =
        env::var("AIGENT_HIVE_PACKAGE_VERSION").unwrap_or_else(|_| product_version.clone());
    assert!(
        valid_package_version(&product_version, &package_version),
        "AIGENT_HIVE_PACKAGE_VERSION must equal the product version or use product-dev or product-test[.N]"
    );
    println!("cargo:rerun-if-env-changed=AIGENT_HIVE_PACKAGE_VERSION");
    println!("cargo:rustc-env=HIVE_PACKAGE_VERSION={package_version}");
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
    let package_release_date =
        env::var("AIGENT_HIVE_PACKAGE_RELEASE_DATE").unwrap_or_else(|_| release_date.to_owned());
    assert!(
        valid_release_date(&package_release_date),
        "AIGENT_HIVE_PACKAGE_RELEASE_DATE must use YYYY-MM-DD"
    );
    println!("cargo:rerun-if-env-changed=AIGENT_HIVE_PACKAGE_RELEASE_DATE");
    println!("cargo:rustc-env=HIVE_PACKAGE_RELEASE_DATE={package_release_date}");
    write_historical_095_table(&manifest_dir);
}

fn write_historical_095_table(manifest_dir: &PathBuf) {
    let base = manifest_dir.join("../../harness/user-bases/0.9.5/plugins/aigent-hive");
    println!("cargo:rerun-if-changed={}", base.to_string_lossy());
    let mut files = Vec::new();
    collect_files(&base, &base, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut generated =
        String::from("#[allow(dead_code)]\npub const HISTORICAL_095_FILES: &[(&str, &str)] = &[\n");
    for (relative, digest) in files {
        generated.push_str(&format!("    ({relative:?}, {digest:?}),\n"));
    }
    generated.push_str("];\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"))
        .join("historical_095.rs");
    fs::write(output, generated).expect("write historical 0.9.5 table");
}

fn collect_files(root: &PathBuf, current: &PathBuf, files: &mut Vec<(String, String)>) {
    let entries = fs::read_dir(current)
        .expect("read historical 0.9.5 base")
        .collect::<Result<Vec<_>, _>>()
        .expect("read historical 0.9.5 entries");
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).expect("inspect historical 0.9.5 entry");
        if metadata.is_dir() {
            collect_files(root, &path, files);
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .expect("base descendant")
                .to_string_lossy()
                .replace('\\', "/");
            let digest_bytes = Sha256::digest(fs::read(&path).expect("read historical 0.9.5 file"));
            let digest = format!(
                "sha256:{}",
                digest_bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            );
            files.push((relative, digest));
        } else {
            panic!("historical 0.9.5 base contains a non-regular entry");
        }
    }
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

fn valid_package_version(product: &str, package: &str) -> bool {
    package == product
        || package.strip_suffix("-dev") == Some(product)
        || package.strip_suffix("-test") == Some(product)
        || package
            .strip_prefix(&format!("{product}-test."))
            .is_some_and(|revision| {
                !revision.is_empty()
                    && !revision.starts_with('0')
                    && revision.bytes().all(|byte| byte.is_ascii_digit())
            })
}

#[cfg(test)]
mod tests {
    use super::{valid_package_version, valid_release_date};

    #[test]
    fn release_date_requires_exact_iso_calendar_shape() {
        assert!(valid_release_date("2026-07-24"));
        for invalid in ["", "2026-7-24", "2026/07/24", "v2026-07-24"] {
            assert!(!valid_release_date(invalid), "{invalid}");
        }
    }

    #[test]
    fn package_version_requires_the_current_product_or_a_developer_suffix() {
        assert!(valid_package_version("0.9.0", "0.9.0"));
        assert!(valid_package_version("0.9.0", "0.9.0-dev"));
        assert!(valid_package_version("0.9.0", "0.9.0-test"));
        assert!(valid_package_version("0.9.0", "0.9.0-test.2"));
        for invalid in [
            "0.9.0-dev.1",
            "0.9.0-test.0",
            "0.9.0-test.02",
            "0.9.1-test.2",
        ] {
            assert!(!valid_package_version("0.9.0", invalid), "{invalid}");
        }
    }
}
