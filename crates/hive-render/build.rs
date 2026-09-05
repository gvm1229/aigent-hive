use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema_version: u32,
    releases: Vec<Release>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Release {
    version: String,
    compatibility_epoch: String,
    renderer: String,
    state_schema: String,
    migration_id: String,
    project_base_digest: String,
    user_projection_digest: Option<String>,
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest.join("../..");
    let registry_path = root.join("harness/project-bases/registry.yml");
    println!("cargo:rerun-if-changed={}", registry_path.display());
    let registry: Registry = serde_yaml::from_slice(
        &fs::read(&registry_path).expect("read historical project-base registry"),
    )
    .expect("parse historical project-base registry");
    assert_eq!(registry.schema_version, 1, "project-base registry schema");
    assert!(
        !registry.releases.is_empty(),
        "project-base registry is empty"
    );

    let mut previous = None::<String>;
    let mut generated = String::new();
    generated.push_str("pub const FULL_HISTORICAL_PROJECT_BASE_VERSIONS: &[&str] = &[\n");
    for release in &registry.releases {
        assert!(valid_version(&release.version), "invalid release version");
        assert!(
            valid_version(&release.compatibility_epoch),
            "invalid compatibility epoch"
        );
        assert!(
            release
                .renderer
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || value == '_'),
            "invalid historical renderer"
        );
        assert_eq!(
            release.state_schema, "project-v1",
            "unsupported state schema"
        );
        assert_eq!(
            release.migration_id, "same-major-render-v1",
            "unsupported migration id"
        );
        if let Some(previous) = previous.replace(release.version.clone()) {
            assert!(
                version_key(&previous) < version_key(&release.version),
                "registry order"
            );
        }
        let project = root.join("harness/project-bases").join(&release.version);
        println!("cargo:rerun-if-changed={}", project.display());
        assert_eq!(
            tree_digest(&project),
            release.project_base_digest,
            "project base digest"
        );
        match &release.user_projection_digest {
            Some(expected) => {
                let user = root.join("harness/user-bases").join(&release.version);
                println!("cargo:rerun-if-changed={}", user.display());
                assert_eq!(tree_digest(&user), *expected, "user projection digest");
            }
            None => assert!(
                !root
                    .join("harness/user-bases")
                    .join(&release.version)
                    .exists(),
                "unexpected user projection base"
            ),
        }
        writeln!(generated, "    \"{}\",", release.version).expect("version source");
    }
    generated.push_str("];\n\n");
    generated.push_str(
        "fn frozen_project_base_from_registry(target_dir: &Dir, version: &str) -> Result<BTreeMap<String, Vec<u8>>, RenderError> {\n    match version {\n",
    );
    for release in &registry.releases {
        writeln!(
            generated,
            "        \"{}\" => {}(target_dir),",
            release.version, release.renderer
        )
        .expect("dispatch source");
    }
    generated.push_str(
        "        _ => Err(RenderError::Unsupported(format!(\"historical full project base is not embedded: {version}\"))),\n    }\n}\n",
    );
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("historical_project_base_registry.rs");
    fs::write(output, generated).expect("write generated project-base registry");
}

fn valid_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3 && parts.iter().all(|part| part.parse::<u64>().is_ok())
}

fn version_key(value: &str) -> (u64, u64, u64) {
    let mut parts = value
        .split('.')
        .map(|part| part.parse::<u64>().expect("version part"));
    (
        parts.next().expect("major"),
        parts.next().expect("minor"),
        parts.next().expect("patch"),
    )
}

fn tree_digest(root: &Path) -> String {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut digest = Sha256::new();
    for (relative, path) in files {
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(fs::read(path).expect("read registered base file"));
        digest.update([0]);
    }
    let mut encoded = String::from("sha256:");
    for byte in digest.finalize() {
        write!(encoded, "{byte:02x}").expect("encode tree digest");
    }
    encoded
}

fn collect_files(root: &Path, current: &Path, files: &mut Vec<(String, PathBuf)>) {
    let mut entries = fs::read_dir(current)
        .expect("read registered base directory")
        .map(|entry| entry.expect("read registered base entry"))
        .collect::<Vec<_>>();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).expect("inspect registered base entry");
        assert!(
            !metadata.file_type().is_symlink(),
            "registered base contains symlink"
        );
        if metadata.is_dir() {
            collect_files(root, &path, files);
        } else {
            assert!(metadata.is_file(), "registered base contains non-file");
            let relative = path
                .strip_prefix(root)
                .expect("registered base descendant")
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, path));
        }
    }
}
