//! Private source-debug download inputs; absent from release binaries.
use std::{
    env,
    path::{Path, PathBuf},
};

pub(crate) fn download(url: &str, maximum: usize) -> Result<Option<Vec<u8>>, String> {
    let Some(root) = env::var_os("HIVE_TEST_UPDATE_FIXTURE") else {
        return Ok(None);
    };
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|_| "update qualification source is unavailable")?;
    let source_marker = super::read_optional_regular(&source.join("hive-source.json"), 4096)?
        .ok_or("update qualification source marker is missing")?;
    let source_marker: serde_json::Value = serde_json::from_slice(&source_marker)
        .map_err(|_| "update qualification source marker is invalid")?;
    if source_marker["kind"] != "aigent-hive-source-workspace" {
        return Err("update qualification requires the source workspace".into());
    }
    let allowed = source.join("tests/work");
    if allowed
        .canonicalize()
        .map_err(|_| "update qualification work root is unavailable")?
        != allowed
    {
        return Err("linked update qualification root refused".into());
    }
    let requested = PathBuf::from(root);
    let pinned = crate::run::PinnedTarget::open_usage(&requested)
        .map_err(|_| "linked update qualification root refused")?;
    let root = requested
        .canonicalize()
        .map_err(|_| "update qualification root missing")?;
    if root == allowed || !root.starts_with(&allowed) {
        return Err("update qualification escaped source tests/work".into());
    }
    let marker = super::read_optional_regular(&root.join("fixture-only"), 64)?;
    if marker.as_deref() != Some(b"0.14.0-direct-fixture") {
        return Err("update qualification marker differs".into());
    }
    let executable = env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|_| "update qualification executable missing")?;
    let args: Vec<_> = env::args_os().collect();
    let user = args
        .windows(2)
        .find(|p| p[0] == "--user-root")
        .map(|p| PathBuf::from(&p[1]))
        .ok_or("update qualification requires an explicit user root")?;
    let user = user
        .canonicalize()
        .map_err(|_| "update qualification user root missing")?;
    if !user.starts_with(&root) || !executable.starts_with(&root) {
        return Err("update qualification escaped its private executable or user root".into());
    }
    let name = if url == "https://registry.npmjs.org/aigent-hive" {
        "registry.json"
    } else if let Some(version) = url
        .strip_prefix("https://unpkg.com/aigent-hive@")
        .and_then(|s| s.strip_suffix("/install.ps1"))
    {
        let package = super::parse_package_version(version)?;
        if package.product.to_string() != env!("CARGO_PKG_VERSION") {
            return Err("update qualification must use the active product version".into());
        }
        "installer.ps1"
    } else {
        return Err("unexpected update qualification download".into());
    };
    let path = root.join(name);
    if path
        .canonicalize()
        .map_err(|_| "update qualification input missing")?
        .parent()
        != Some(root.as_path())
    {
        return Err("linked update qualification input refused".into());
    }
    let bytes = super::read_optional_regular(&path, maximum as u64)?
        .map(Some)
        .ok_or_else(|| "update qualification input missing".to_owned())?;
    pinned
        .verify_current()
        .map_err(|_| "update qualification root changed")?;
    Ok(bytes)
}
