//! Remove only the installer owner authenticated in the displayed removal plan.
use super::{
    env, fs, io, platform_package_name, read_optional_regular, sha256_digest,
    uninstall_owner_preview, wait_for_windows_direct_unlock, CommandRunner, Duration, Path,
    PathBuf, SystemCommandRunner, INSTALL_OUTPUT_LIMIT, INSTALL_TIMEOUT, MAX_BINARY_BYTES,
    MAX_MANIFEST_BYTES,
};
use serde_json::Value;

pub(crate) fn remove_owned_core(expected: &Value) -> Result<bool, String> {
    remove_using(expected, &SystemCommandRunner)
}

fn remove_using(expected: &Value, runner: &impl CommandRunner) -> Result<bool, String> {
    if expected["owner"] == "unverified" {
        return Ok(false);
    }
    let executable = PathBuf::from(
        expected["executable"]
            .as_str()
            .ok_or("missing approved program path")?,
    );
    if !executable.is_absolute()
        || !crate::is_sha256_digest(expected["executable_digest"].as_str().unwrap_or_default())
    {
        return Err("invalid approved program identity".into());
    }
    let present = match fs::symlink_metadata(&executable) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(_) => return Err("cannot inspect approved program".into()),
    };
    if present {
        if uninstall_owner_preview(&executable)? != *expected {
            return Err("program ownership changed after removal approval".into());
        }
        if cfg!(windows) {
            if env::current_exe()
                .ok()
                .and_then(|p| p.canonicalize().ok())
                .as_ref()
                == Some(&executable)
            {
                return Ok(false);
            }
            wait_for_windows_direct_unlock(&executable)?;
        }
    }
    match expected["owner"].as_str() {
        Some("direct") => remove_direct(expected, &executable),
        Some("npm") => remove_npm(expected, &executable, runner),
        _ => Err("unsupported program removal owner".into()),
    }
}

fn remove_direct(expected: &Value, executable: &Path) -> Result<bool, String> {
    let prefix = Path::new(expected["prefix"].as_str().ok_or("missing direct prefix")?);
    if executable.parent().and_then(Path::parent) != Some(prefix) {
        return Err("direct removal prefix differs".into());
    }
    let cap = crate::user_install::open_user_root_for_setup(prefix)?;
    let binary = Path::new("bin").join(if cfg!(windows) { "hive.exe" } else { "hive" });
    let receipt = Path::new("share/aigent-hive/install-receipt.json");
    let bytes = crate::user_install::read_user_setup_file(&cap, &binary, MAX_BINARY_BYTES)?;
    let installed = crate::user_install::read_user_setup_file(&cap, receipt, MAX_MANIFEST_BYTES)?;
    if bytes
        .as_deref()
        .is_some_and(|bytes| expected["executable_digest"] != sha256_digest(bytes))
        || installed
            .as_deref()
            .is_some_and(|bytes| expected["receipt_digest"] != sha256_digest(bytes))
        || (bytes.is_some() && installed.is_none())
    {
        return Err("direct program or receipt changed; files preserved".into());
    }
    // These are the two files the direct installer owns. Never remove the prefix tree.
    if let Some(bytes) = bytes {
        crate::user_install::replace_user_setup_file(&cap, &binary, Some(&bytes), None)?;
    }
    if let Some(bytes) = installed {
        crate::user_install::replace_user_setup_file(&cap, receipt, Some(&bytes), None)?;
    }
    Ok(
        crate::user_install::read_user_setup_file(&cap, &binary, MAX_BINARY_BYTES)?.is_none()
            && crate::user_install::read_user_setup_file(&cap, receipt, MAX_MANIFEST_BYTES)?
                .is_none(),
    )
}

fn npm_path(
    runner: &impl CommandRunner,
    npm: &crate::usage::QualifiedExecutable,
    operation: &str,
) -> Result<PathBuf, String> {
    let result = runner
        .run(
            npm,
            &[operation, "--global"],
            Duration::from_secs(15),
            16_384,
        )
        .map_err(|_| "cannot inspect npm install scope")?;
    if !result.success {
        return Err("npm install scope query failed".into());
    }
    let text = std::str::from_utf8(&result.stdout)
        .map_err(|_| "invalid npm install scope")?
        .trim();
    let path = PathBuf::from(text);
    if !path.is_absolute() {
        return Err("npm install scope is not absolute".into());
    }
    path.canonicalize()
        .map_err(|_| "cannot resolve npm install scope".into())
}

fn remove_npm(
    expected: &Value,
    executable: &Path,
    runner: &impl CommandRunner,
) -> Result<bool, String> {
    let npm = runner
        .qualify("npm")
        .map_err(|_| "npm is unavailable; program preserved")?;
    let root = npm_path(runner, &npm, "root")?;
    let prefix = npm_path(runner, &npm, "prefix")?;
    let platform = platform_package_name()?
        .strip_prefix('@')
        .ok_or("invalid platform package")?;
    let native = Path::new(&format!("@{platform}"))
        .join("bin")
        .join(if cfg!(windows) { "hive.exe" } else { "hive" });
    let within = executable == root.join(&native)
        || executable == root.join("aigent-hive/node_modules").join(&native);
    let rooted = root == prefix.join("node_modules") || root == prefix.join("lib/node_modules");
    if !within || !rooted {
        return Err("npm global scope differs from the approved executable; preserved".into());
    }
    let manifest = root.join("aigent-hive/package.json");
    let current = read_optional_regular(&manifest, MAX_MANIFEST_BYTES)?;
    if let Some(bytes) = current {
        let package: Value =
            serde_json::from_slice(&bytes).map_err(|_| "invalid npm main package manifest")?;
        if package["name"] != "aigent-hive" || package["version"] != expected["package_version"] {
            return Err("npm main package ownership differs; preserved".into());
        }
        let prefix = prefix.to_str().ok_or("npm prefix is not UTF-8")?;
        let result = runner
            .run(
                &npm,
                &[
                    "uninstall",
                    "--global",
                    "--prefix",
                    prefix,
                    "--ignore-scripts",
                    "aigent-hive",
                ],
                INSTALL_TIMEOUT,
                INSTALL_OUTPUT_LIMIT,
            )
            .map_err(|_| "npm removal did not finish; resume the saved removal")?;
        if !result.success {
            return Err("npm removal did not report success; inspect the saved removal".into());
        }
    }
    Ok(
        read_optional_regular(executable, MAX_BINARY_BYTES)?.is_none()
            && read_optional_regular(&manifest, MAX_MANIFEST_BYTES)?.is_none(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::{CommandOutput, QualifiedExecutable, SensorError};
    use std::cell::RefCell;

    struct NpmFixture {
        prefix: PathBuf,
        root: PathBuf,
        executable: PathBuf,
        calls: RefCell<Vec<Vec<String>>>,
    }

    impl CommandRunner for NpmFixture {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            assert_eq!(program, "npm");
            Ok(QualifiedExecutable::synthetic("npm"))
        }
        fn run(
            &self,
            _: &QualifiedExecutable,
            args: &[&str],
            _: Duration,
            _: usize,
        ) -> Result<CommandOutput, SensorError> {
            self.calls
                .borrow_mut()
                .push(args.iter().map(|s| (*s).into()).collect());
            let stdout = match args {
                ["root", "--global"] => self.root.to_string_lossy().as_bytes().to_vec(),
                ["prefix", "--global"] => self.prefix.to_string_lossy().as_bytes().to_vec(),
                ["uninstall", "--global", "--prefix", prefix, "--ignore-scripts", "aigent-hive"] => {
                    assert_eq!(Path::new(prefix), self.prefix);
                    assert!(self.executable.starts_with(&self.root));
                    fs::remove_file(&self.executable).unwrap();
                    fs::remove_file(self.root.join("aigent-hive/package.json")).unwrap();
                    Vec::new()
                }
                _ => panic!("unexpected package-manager command"),
            };
            Ok(CommandOutput {
                success: true,
                exit_code: Some(0),
                stdout,
                stderr: Vec::new(),
            })
        }
    }

    #[test]
    fn npm_removal_requires_matching_global_root_and_package_before_delegating() {
        let temp = tempfile::tempdir().unwrap();
        let prefix = temp.path().canonicalize().unwrap();
        let root = prefix.join("node_modules");
        let executable = root
            .join(platform_package_name().unwrap())
            .join("bin")
            .join(if cfg!(windows) { "hive.exe" } else { "hive" });
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::create_dir_all(root.join("aigent-hive")).unwrap();
        fs::write(&executable, b"fixture bytes, never executed").unwrap();
        let manifest = root.join("aigent-hive/package.json");
        fs::write(
            &manifest,
            br#"{"name":"aigent-hive","version":"0.14.0-test.1"}"#,
        )
        .unwrap();
        let foreign = prefix.join("foreign.txt");
        fs::write(&foreign, b"keep").unwrap();
        let fixture = NpmFixture {
            prefix,
            root,
            executable: executable.clone(),
            calls: RefCell::new(Vec::new()),
        };
        let expected = serde_json::json!({"owner":"npm","package_version":"0.14.0-test.1"});
        assert!(remove_npm(&expected, &temp.path().join("foreign/bin/hive"), &fixture).is_err());
        assert!(executable.exists());
        let wrong = serde_json::json!({"owner":"npm","package_version":"0.14.0-test.2"});
        assert!(remove_npm(&wrong, &executable, &fixture).is_err());
        assert!(!fixture
            .calls
            .borrow()
            .iter()
            .any(|args| args[0] == "uninstall"));
        assert!(remove_npm(&expected, &executable, &fixture).unwrap());
        assert_eq!(
            fixture
                .calls
                .borrow()
                .iter()
                .filter(|args| args[0] == "uninstall")
                .count(),
            1
        );
        assert_eq!(fs::read(foreign).unwrap(), b"keep");
    }

    #[test]
    fn direct_removal_preserves_neighbors_and_rejects_changed_owned_bytes() {
        let root = tempfile::tempdir().unwrap();
        let prefix = root.path().canonicalize().unwrap();
        let executable = prefix
            .join("bin")
            .join(if cfg!(windows) { "hive.exe" } else { "hive" });
        let receipt = prefix.join("share/aigent-hive/install-receipt.json");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::create_dir_all(receipt.parent().unwrap()).unwrap();
        fs::write(&executable, b"disposable fixture, not an executable").unwrap();
        fs::write(&receipt, b"owned receipt fixture").unwrap();
        let foreign = prefix.join("bin/foreign.txt");
        fs::write(&foreign, b"keep").unwrap();
        let plan = serde_json::json!({"owner":"direct","prefix":prefix,"executable":executable,
            "executable_digest":sha256_digest(&fs::read(&executable).unwrap()),"receipt_digest":sha256_digest(&fs::read(&receipt).unwrap())});
        fs::write(&executable, b"changed by another writer").unwrap();
        assert!(remove_direct(&plan, &executable).is_err());
        assert!(receipt.exists());
        fs::write(&executable, b"disposable fixture, not an executable").unwrap();
        assert!(remove_direct(&plan, &executable).unwrap());
        assert!(!executable.exists() && !receipt.exists());
        assert!(remove_direct(&plan, &executable).unwrap());
        assert_eq!(fs::read(foreign).unwrap(), b"keep");
    }
}
