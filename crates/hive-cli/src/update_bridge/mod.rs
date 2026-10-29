//! Compatibility transport for a frozen updater that pipes the newly installed CLI.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
};
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod posix;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;

static STARTED: OnceLock<Instant> = OnceLock::new();

pub(crate) fn mark_process_start() {
    STARTED.get_or_init(Instant::now);
}

#[derive(Clone)]
pub(crate) struct LegacyBudget {
    started: Instant,
    connected: Arc<AtomicBool>,
}

impl LegacyBudget {
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    pub(crate) fn current() -> Self {
        Self {
            started: *STARTED.get_or_init(Instant::now),
            connected: Arc::new(AtomicBool::new(true)),
        }
    }

    #[cfg(any(windows, target_os = "linux", target_os = "macos", test))]
    pub(crate) fn question_deadline(&self) -> Instant {
        self.started + Duration::from_mins(7)
    }

    pub(crate) fn overall_deadline(&self) -> Instant {
        self.started + Duration::from_mins(9)
    }

    pub(crate) fn apply_deadline(&self) -> Option<Instant> {
        let now = Instant::now();
        (self.overall_deadline().saturating_duration_since(now) >= Duration::from_mins(2))
            .then_some(now + Duration::from_secs(90))
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    #[cfg(any(windows, target_os = "linux", target_os = "macos", test))]
    pub(crate) fn disconnect(&self) {
        self.connected.store(false, Ordering::Release);
    }

    #[cfg(test)]
    pub(crate) fn testing() -> Self {
        Self {
            started: Instant::now(),
            connected: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[cfg(windows)]
pub(crate) fn maybe_setup(
    user_root: &std::path::Path,
    hosts: &[String],
) -> Result<Option<crate::ActionResult>, String> {
    // Unit-level installer calls are not child CLI entrypoints.
    if !std::env::args().any(|argument| argument == "--scope") {
        return Ok(None);
    }
    let budget = LegacyBudget::current();
    let Some(connection) = windows::connect(user_root, budget.clone())? else {
        return Ok(None);
    };
    let result = match connection {
        windows::Connection::Terminal(reader, writer) => {
            crate::user_setup::update_with_terminal(user_root, reader, writer, budget, hosts)
        }
        windows::Connection::Preserve(watch) => {
            let result = crate::user_setup::update_without_terminal(user_root, hosts, &budget);
            drop(watch);
            result
        }
    };
    Ok(Some(result))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) fn maybe_setup(
    user_root: &std::path::Path,
    hosts: &[String],
) -> Result<Option<crate::ActionResult>, String> {
    if !std::env::args().any(|argument| argument == "--scope") {
        return Ok(None);
    }
    let budget = LegacyBudget::current();
    let Some(connection) = platform::connect(user_root, budget.clone())? else {
        return Ok(None);
    };
    let result = match connection {
        posix::Connection::Terminal(reader, writer) => {
            crate::user_setup::update_with_terminal(user_root, reader, writer, budget, hosts)
        }
        posix::Connection::Preserve(watch) => {
            let result = crate::user_setup::update_without_terminal(user_root, hosts, &budget);
            drop(watch);
            result
        }
    };
    Ok(Some(result))
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn maybe_setup(
    _user_root: &std::path::Path,
    _hosts: &[String],
) -> Result<Option<crate::ActionResult>, String> {
    // POSIX still uses the existing preserving installer until its /dev/tty adapter is qualified.
    Ok(None)
}

pub(crate) fn resume_command(root: &std::path::Path) -> String {
    let path = root.to_string_lossy();
    let quoted = if cfg!(windows) {
        path.replace('\'', "''")
    } else {
        path.replace('\'', "'\"'\"'")
    };
    format!("hive update --user-root '{quoted}'")
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn same_installation_path(
    previous: &std::path::Path,
    launch: &str,
    current: &std::path::Path,
) -> Result<bool, String> {
    use std::path::Path;

    let previous_path = previous.canonicalize();
    let current = current
        .canonicalize()
        .map_err(|_| "current install path unavailable")?;
    if previous_path.as_ref().is_ok_and(|path| path == &current) {
        return Ok(true);
    }
    // The caller already verified the running predecessor image and its owner.
    // npm retires that image in a hidden sibling while argv[0] retains the live entry.
    let retired_npm = (|| {
        let native = current.parent()?.parent()?;
        let scope = native.parent()?;
        let bundled = scope.parent()?;
        let package = bundled.parent()?;
        let modules = package.parent()?;
        if scope.file_name()? != "@aigent-hive"
            || bundled.file_name()? != "node_modules"
            || package.file_name()? != "aigent-hive"
            || modules.file_name()? != "node_modules"
        {
            return None;
        }
        let retired = previous.ancestors().nth(5)?;
        let suffix = retired
            .file_name()?
            .to_str()?
            .strip_prefix(".aigent-hive-")?;
        if suffix.len() != 8
            || !suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
            || std::fs::symlink_metadata(retired).is_ok_and(|m| m.file_type().is_symlink())
            || retired.parent()?.canonicalize().ok()? != modules.canonicalize().ok()?
            || previous.strip_prefix(retired).ok()? != current.strip_prefix(package).ok()?
            || !Path::new(launch).is_absolute()
            || Path::new(launch).canonicalize().ok()? != current
        {
            return None;
        }
        Some(crate::update_activation::is_npm_installation(&current))
    })()
    .unwrap_or(false);
    if retired_npm {
        return Ok(true);
    }
    previous_path
        .map(|_| false)
        .map_err(|_| "legacy install path unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_questions_cannot_consume_the_apply_and_cleanup_reserve() {
        let early = LegacyBudget {
            started: Instant::now()
                .checked_sub(Duration::from_secs(419))
                .unwrap(),
            connected: Arc::new(AtomicBool::new(true)),
        };
        assert!(early.question_deadline() > Instant::now());
        assert!(early.apply_deadline().is_some());
        let late = LegacyBudget {
            started: Instant::now()
                .checked_sub(Duration::from_secs(421))
                .unwrap(),
            connected: Arc::new(AtomicBool::new(true)),
        };
        assert!(late.question_deadline() < Instant::now());
        assert!(late.apply_deadline().is_none());
        assert!(late.overall_deadline() > Instant::now());
    }

    fn npm_paths() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", _) => "win32-x64",
            ("macos", "aarch64") => "darwin-arm64",
            ("macos", _) => "darwin-x64",
            ("linux", "aarch64") => "linux-arm64",
            _ => "linux-x64",
        };
        let executable = if cfg!(windows) { "hive.exe" } else { "hive" };
        let relative = format!("node_modules/@aigent-hive/{platform}/bin/{executable}");
        let modules = root.path().join("node_modules");
        let current = modules.join("aigent-hive").join(&relative);
        let previous = modules.join(".aigent-hive-ABCDEFG1").join(relative);
        std::fs::create_dir_all(current.parent().unwrap()).unwrap();
        std::fs::create_dir_all(previous.parent().unwrap()).unwrap();
        std::fs::write(&current, b"new image").unwrap();
        std::fs::write(&previous, b"verified predecessor image").unwrap();
        let version = env!("CARGO_PKG_VERSION");
        std::fs::write(
            current
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("package.json"),
            serde_json::to_vec(&serde_json::json!({
                "name":format!("@aigent-hive/{platform}"), "version":version,
                "aigentHive":{"productVersion":version}
            }))
            .unwrap(),
        )
        .unwrap();
        (root, previous, current)
    }

    #[test]
    fn same_installation_preserves_direct_paths_and_accepts_retired_npm_images() {
        let (_root, previous, current) = npm_paths();
        let launch = current.to_str().unwrap();
        assert!(same_installation_path(&current, "unused", &current).unwrap());
        assert!(same_installation_path(&previous, launch, &current).unwrap());
        std::fs::remove_file(&previous).unwrap();
        assert!(same_installation_path(&previous, launch, &current).unwrap());
    }

    #[test]
    fn retired_npm_scope_rejects_different_launch_prefix_tail_and_manifest() {
        let (root, previous, current) = npm_paths();
        let launch = current.to_str().unwrap();
        for (old, invoked) in [
            (previous.clone(), "hive"),
            (previous.clone(), previous.to_str().unwrap()),
            (
                root.path()
                    .join("elsewhere")
                    .join(previous.strip_prefix(root.path()).unwrap()),
                launch,
            ),
            (previous.with_file_name("another-program"), launch),
            (
                std::path::PathBuf::from(
                    previous
                        .to_string_lossy()
                        .replace(".aigent-hive-ABCDEFG1", ".foreign-ABCDEFG1"),
                ),
                launch,
            ),
        ] {
            assert!(!same_installation_path(&old, invoked, &current).unwrap_or(false));
        }
        std::fs::write(
            current
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("package.json"),
            b"{}",
        )
        .unwrap();
        assert!(!same_installation_path(&previous, launch, &current).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn retired_npm_scope_rejects_linked_retirement_directory() {
        let (root, previous, current) = npm_paths();
        let retired = previous.ancestors().nth(5).unwrap();
        let linked = root.path().join("node_modules/.aigent-hive-12345678");
        std::os::unix::fs::symlink(retired, &linked).unwrap();
        assert!(!same_installation_path(
            &linked.join(previous.strip_prefix(retired).unwrap()),
            current.to_str().unwrap(),
            &current,
        )
        .unwrap());
    }
}
