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
}
