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
        "AIGENT_HIVE_PACKAGE_VERSION must equal the product version or use product-test[.N]"
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
    fn package_version_requires_the_current_product_or_a_test_suffix() {
        assert!(valid_package_version("0.9.0", "0.9.0"));
        assert!(valid_package_version("0.9.0", "0.9.0-test"));
        assert!(valid_package_version("0.9.0", "0.9.0-test.2"));
        for invalid in ["0.9.0-test.0", "0.9.0-test.02", "0.9.1-test.2"] {
            assert!(!valid_package_version("0.9.0", invalid), "{invalid}");
        }
    }
}
