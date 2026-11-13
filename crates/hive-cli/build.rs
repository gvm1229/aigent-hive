use sha2::{Digest, Sha256};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

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
    write_historical_user_plugin_tables(&manifest_dir);
    let mut policy = Sha256::new();
    for relative in [
        "src/policy/native.rs",
        "src/policy/configure.rs",
        "src/policy/directive_context.rs",
        "src/main.rs",
        "src/run.rs",
        "src/run/policy_review.rs",
        "../hive-core/src/run.rs",
        "../../schemas/host-policy-intent.schema.json",
        "../../schemas/policy-review.schema.json",
        "../../schemas/run-status.schema.json",
        "../hive-core/src/policy.rs",
        "../hive-core/src/lib.rs",
    ] {
        let path = manifest_dir.join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
        policy.update(relative.as_bytes());
        policy.update([0]);
        policy.update(fs::read(path).expect("native policy source is readable"));
    }
    let mut digest = String::from("sha256:");
    for byte in policy.finalize() {
        write!(digest, "{byte:02x}").expect("format policy digest");
    }
    println!("cargo:rustc-env=HIVE_NATIVE_POLICY_DIGEST={digest}");
}

fn write_historical_user_plugin_tables(manifest_dir: &Path) {
    let mut releases = Vec::new();
    let root = manifest_dir.join("../..");
    let registry = checked_registry_path(&root, "harness/project-bases/registry.yml", false)
        .expect("historical registry path");
    println!("cargo:rerun-if-changed={}", registry.display());
    let versions =
        registry_plugin_versions(&fs::read_to_string(registry).expect("read historical registry"))
            .expect("historical registry versions");
    validate_registered_user_bases(&root, &versions).expect("historical user-base coverage");
    for version in &versions {
        let base = manifest_dir.join(format!(
            "../../harness/user-bases/{version}/plugins/aigent-hive"
        ));
        println!("cargo:rerun-if-changed={}", base.to_string_lossy());
        let mut files = Vec::new();
        collect_files(&base, &base, &mut files);
        files.sort_by(|left, right| left.0.cmp(&right.0));
        assert!(
            files.windows(2).all(|pair| pair[0].0 != pair[1].0),
            "duplicate historical user path"
        );
        releases.push((version, files));
    }

    let mut generated = String::from(
        "pub const HISTORICAL_USER_PLUGIN_RELEASES: &[(&str, &[(&str, &str)])] = &[\n",
    );
    for (version, files) in &releases {
        writeln!(generated, "    ({version:?}, &[").expect("write release header");
        for (relative, digest) in files {
            writeln!(generated, "        ({relative:?}, {digest:?}),").expect("write release file");
        }
        generated.push_str("    ]),\n");
    }
    generated.push_str("];\n");
    generated.push_str("type HistoricalUserSkillContent = (&'static str, &'static [(&'static str, &'static [u8])]);\n");
    generated
        .push_str("pub const HISTORICAL_USER_SKILL_CONTENTS: &[HistoricalUserSkillContent] = &[\n");
    for (version, files) in &releases {
        writeln!(generated, "    ({version:?}, &[").expect("write content release");
        for (relative, _) in files.iter().filter(|(path, _)| path.starts_with("skills/")) {
            let path = manifest_dir.join(format!(
                "../../harness/user-bases/{version}/plugins/aigent-hive/{relative}"
            ));
            writeln!(
                generated,
                "        ({relative:?}, include_bytes!({:?}).as_slice()),",
                path.to_string_lossy()
            )
            .expect("write content file");
        }
        generated.push_str("    ]),\n");
    }
    generated.push_str("];\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"))
        .join("historical_user_plugins.rs");
    fs::write(output, generated).expect("write historical user plugin tables");
}

fn registry_version(value: &str) -> Option<(u64, u64, u64)> {
    let [major, minor, patch]: [&str; 3] = value.split('.').collect::<Vec<_>>().try_into().ok()?;
    let part = |part: &str| {
        if part.is_empty()
            || (part.len() > 1 && part.starts_with('0'))
            || !part.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        part.parse::<u64>().ok()
    };
    Some((part(major)?, part(minor)?, part(patch)?))
}

fn registry_plugin_versions(text: &str) -> Result<Vec<String>, String> {
    let registry: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    if registry["schema_version"].as_u64() != Some(1) {
        return Err("unsupported registry schema".into());
    }
    let releases = registry["releases"]
        .as_sequence()
        .filter(|entries| !entries.is_empty())
        .ok_or("registry releases are missing")?;
    let mut versions = Vec::new();
    let mut previous = None;
    for release in releases {
        let version = release["version"]
            .as_str()
            .ok_or("registry version is missing")?;
        let key = registry_version(version).ok_or("invalid registry version")?;
        if previous.is_some_and(|previous| previous >= key) {
            return Err("registry versions must be sorted and unique".into());
        }
        previous = Some(key);
        let digest = release
            .get("user_projection_digest")
            .ok_or("user projection registration is missing")?;
        if !digest.is_null() {
            let digest = digest.as_str().ok_or("invalid user projection digest")?;
            if !digest.strip_prefix("sha256:").is_some_and(|hex| {
                hex.len() == 64
                    && hex
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            }) {
                return Err("invalid user projection digest".into());
            }
            // User plugins start at 0.9; the 0.8 adapter layout has its existing separate reader.
            if key >= (0, 9, 0) {
                versions.push(version.to_owned());
            }
        }
    }
    Ok(versions)
}

fn reparse_entry(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

fn checked_registry_path(root: &Path, relative: &str, directory: bool) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
        || Path::new(relative)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err("registry path is not portable and relative".into());
    }
    let mut path = root.to_path_buf();
    let parts = relative.split('/').collect::<Vec<_>>();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || reparse_entry(&metadata)
            || (index + 1 < parts.len() || directory) && !metadata.is_dir()
            || index + 1 == parts.len() && !directory && !metadata.is_file()
        {
            return Err("registry path contains a link or unsupported entry".into());
        }
    }
    Ok(path)
}

fn validate_registered_user_bases(root: &Path, versions: &[String]) -> Result<(), String> {
    let directory = checked_registry_path(root, "harness/user-bases", true)?;
    let mut present = std::collections::BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "user-base name is not UTF-8")?;
        if registry_version(&name).is_some_and(|version| version >= (0, 9, 0)) {
            checked_registry_path(
                root,
                &format!("harness/user-bases/{name}/plugins/aigent-hive"),
                true,
            )?;
            present.insert(name);
        }
    }
    if present != versions.iter().cloned().collect() {
        return Err("registry omits or references a missing user plugin base".into());
    }
    Ok(())
}

fn collect_files(root: &PathBuf, current: &PathBuf, files: &mut Vec<(String, String)>) {
    let entries = fs::read_dir(current)
        .expect("read historical user plugin base")
        .collect::<Result<Vec<_>, _>>()
        .expect("read historical user plugin entries");
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).expect("inspect historical user plugin entry");
        assert!(
            !metadata.file_type().is_symlink() && !reparse_entry(&metadata),
            "historical user base contains a link"
        );
        if metadata.is_dir() {
            collect_files(root, &path, files);
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .expect("base descendant")
                .to_str()
                .expect("historical user path must be UTF-8")
                .replace('\\', "/");
            assert!(
                !relative
                    .split('/')
                    .any(|part| part.is_empty() || matches!(part, "." | "..")),
                "historical user path escapes its base"
            );
            let digest_bytes = Sha256::digest(fs::read(&path).expect("read historical 0.9.5 file"));
            let mut digest = String::from("sha256:");
            for byte in digest_bytes {
                write!(digest, "{byte:02x}").expect("write digest");
            }
            files.push((relative, digest));
        } else {
            panic!("historical user plugin base contains a non-regular entry");
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
    fn registry_rejects_missing_duplicate_and_escaping_versions() {
        let digest = "a".repeat(64);
        let valid = format!("schema_version: 1\nreleases:\n  - version: 0.8.0\n    user_projection_digest: null\n  - version: 0.9.0\n    user_projection_digest: sha256:{digest}\n");
        assert_eq!(
            super::registry_plugin_versions(&valid).unwrap(),
            vec!["0.9.0"]
        );
        for bad in [
            "/outside",
            "../outside",
            "0.9.00",
            "0.9",
            "0.9.0-test.1",
            "0.8.0",
        ] {
            assert!(
                super::registry_plugin_versions(
                    &valid.replace("version: 0.9.0", &format!("version: {bad}"))
                )
                .is_err(),
                "{bad}"
            );
        }
        assert!(super::registry_plugin_versions("schema_version: 1\n").is_err());
        assert!(super::registry_plugin_versions(
            &valid.replace("user_projection_digest: null", "ignored: null")
        )
        .is_err());
    }

    #[test]
    fn user_base_coverage_and_path_escape_fail_before_reading_contents() {
        let temporary = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(
            temporary
                .path()
                .join("harness/user-bases/0.9.0/plugins/aigent-hive"),
        )
        .unwrap();
        assert!(super::validate_registered_user_bases(temporary.path(), &["0.9.0".into()]).is_ok());
        assert!(super::validate_registered_user_bases(temporary.path(), &[]).is_err());
        assert!(super::validate_registered_user_bases(
            temporary.path(),
            &["0.9.0".into(), "0.9.1".into()]
        )
        .is_err());
        for path in [
            "../outside",
            "/outside",
            "harness/../outside",
            "harness//user-bases",
            "harness/./user-bases",
        ] {
            assert!(super::checked_registry_path(temporary.path(), path, true).is_err());
        }
    }

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
